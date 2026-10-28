use super::*;
use crate::{
    ArtworkDerivation, ArtworkManifest, ArtworkParameters, ArtworkPreview, ArtworkPreviewFiles,
    ArtworkReviewHashes,
};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::io::Read;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl EditorCore {
    pub fn prepare_artwork(
        &self,
        project_id: &str,
        expected_revision: u64,
        source_asset_id: &str,
        parameters: ArtworkParameters,
    ) -> Result<ArtworkPreview, CoreError> {
        parameters.validate()?;
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, _) = load_project_data(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &self.font_config,
            Some(expected_revision),
        )?;
        let (manifest, prepared) =
            self.prepare_loaded_artwork(&project, source_asset_id, parameters)?;
        let encode = |v: &[u8]| base64::engine::general_purpose::STANDARD.encode(v);
        Ok(ArtworkPreview {
            manifest,
            files: ArtworkPreviewFiles {
                result: encode(&prepared.files.result),
                mask: encode(&prepared.files.mask),
                light: encode(&prepared.files.light),
                dark: encode(&prepared.files.dark),
            },
        })
    }

    fn prepare_loaded_artwork(
        &self,
        project: &Project,
        source_asset_id: &str,
        parameters: ArtworkParameters,
    ) -> Result<(ArtworkManifest, crate::assets::artwork::PreparedArtwork), CoreError> {
        let source = project
            .assets
            .iter()
            .find(|a| a.id == source_asset_id)
            .ok_or_else(|| {
                CoreError::new(ErrorCode::AssetNotFound, "artwork source was not found")
            })?;
        if source.media_type != MediaType::Image {
            return Err(CoreError::new(
                ErrorCode::UnsupportedMedia,
                "artwork requires a managed PNG Image",
            ));
        }
        let path = self.project_asset_path(&project.id, &source.project_relative_path)?;
        let mut bytes = Vec::new();
        self.storage
            .open_read(&path)
            .map_err(|e| CoreError::io("cannot read artwork source", e))?
            .take(4 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| CoreError::io("cannot read artwork source", e))?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "artwork source exceeds 4 MiB",
            ));
        }
        let source_hash = hash(&bytes);
        if !source
            .content_hash
            .as_ref()
            .is_some_and(|h| h.algorithm == "sha256" && h.digest == source_hash)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "artwork source hash mismatch",
            ));
        }
        let prepared = crate::assets::artwork::prepare_png(&bytes, &parameters)?;
        if source
            .probe
            .as_ref()
            .is_none_or(|p| p.video_width.is_none() || p.video_height.is_none())
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "artwork source is missing required dimension facts; reimport the authorized PNG using the existing allowed import workflow",
            ));
        }

        if !source.probe.as_ref().is_some_and(|p| {
            p.video_width == Some(prepared.source_width)
                && p.video_height == Some(prepared.source_height)
        }) {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "artwork source dimension facts mismatch",
            ));
        }
        let hashes = ArtworkReviewHashes {
            result: hash(&prepared.files.result),
            mask: hash(&prepared.files.mask),
            light: hash(&prepared.files.light),
            dark: hash(&prepared.files.dark),
        };
        let manifest = ArtworkManifest {
            version: 1,
            project_id: project.id.clone(),
            expected_revision: project.revision,
            source_asset_id: source_asset_id.into(),
            source_hash,
            parameters,
            result_hash: hashes.result.clone(),
            review_hashes: hashes,
            width: prepared.width,
            height: prepared.height,
            source_width: prepared.source_width,
            source_height: prepared.source_height,
            content_offset: prepared.content_offset,
        };
        Ok((manifest, prepared))
    }

    pub fn commit_artwork(&self, manifest: ArtworkManifest) -> Result<WriteResult, CoreError> {
        manifest.parameters.validate()?;
        let dir = self.existing_project_dir(&manifest.project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let mut rollback = crate::assets::UncommittedResources::default();
        let result = (|| {
            let PreparedProject {
                mut project,
                mut history,
                fonts,
                draft_updates,
                ..
            } = prepare_project_data(
                self.storage.as_ref(),
                &self.persistence_faults,
                &dir,
                &self.font_config,
                Some(manifest.expected_revision),
                Some(&mut rollback),
            )?;
            let (actual, prepared) = self.prepare_loaded_artwork(
                &project,
                &manifest.source_asset_id,
                manifest.parameters.clone(),
            )?;
            if actual != manifest {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "artwork manifest does not match reviewed source and result",
                ));
            }
            let stored = crate::assets::store_artwork_content(
                self.storage.as_ref(),
                &dir,
                &prepared.files.result,
                &mut rollback,
            )?;
            let previous = project.clone();
            let id = Uuid::new_v4().to_string();
            project.assets.push(Asset {
                artwork_derivation: Some(ArtworkDerivation {
                    version: 1,
                    source_asset_id: manifest.source_asset_id.clone(),
                    source_hash: manifest.source_hash.clone(),
                    parameters: manifest.parameters.clone(),
                    source_width: manifest.source_width,
                    source_height: manifest.source_height,
                    content_offset: manifest.content_offset,
                }),
                id: id.clone(),
                media_type: MediaType::Image,
                file_name: "artwork-derivative.png".into(),
                project_relative_path: stored.relative_path,
                duration_ms: None,
                has_audio: false,
                origin: None,
                content_hash: Some(stored.content_hash),
                size_bytes: Some(stored.size_bytes),
                probe: Some(MediaProbeFacts {
                    has_video: true,
                    format_name: Some("png_pipe".into()),
                    video_codec: Some("png".into()),
                    video_width: Some(manifest.width),
                    video_height: Some(manifest.height),
                    ..MediaProbeFacts::default()
                }),
            });
            validate_retained_project_references(&project, &history)?;
            push_undo(&mut history, &previous);
            bump_revision(&mut project)?;
            crate::assets::fonts::publish_fonts_tracked(
                self.storage.as_ref(),
                &dir,
                &fonts,
                &mut rollback,
            )?;
            let warnings = match crate::persistence::persist_transaction_with_drafts(
                self.storage.as_ref(),
                &self.persistence_faults,
                &dir,
                &project,
                &history,
                None,
                draft_updates,
            ) {
                Ok(warnings) => warnings,
                Err(_) if self.storage.storage_path_exists(&transaction_path(&dir)) => {
                    vec![PERSISTENCE_RECOVERY_PENDING.into()]
                }
                Err(error) => return Err(error),
            };
            let mut result = write_result(&project, vec![id], "Committed artwork derivative");
            result.warnings = finish_persistence(
                self.storage.as_ref(),
                &self.persistence_faults,
                &dir,
                &project,
                &history,
                warnings,
            );
            Ok(result)
        })();
        if result.is_err() && !self.storage.storage_path_exists(&transaction_path(&dir)) {
            rollback.rollback(self.storage.as_ref())?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::PersistencePhase;

    fn prepared() -> (tempfile::TempDir, EditorCore, ArtworkManifest) {
        let root = tempfile::tempdir().unwrap();
        let core = EditorCore::new(
            PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let id = core
            .create_project("Fault artwork", ProjectSettings::default())
            .unwrap()
            .project_id;
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&[200, 30, 50, 255])
                .unwrap();
        }
        let path = root.path().join("original.png");
        std::fs::write(&path, bytes).unwrap();
        let source = core
            .import_asset(
                &id,
                0,
                &path,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    video_width: Some(1),
                    video_height: Some(1),
                    ..MediaProbeFacts::default()
                },
            )
            .unwrap()
            .changed_ids
            .remove(0);
        let manifest = core
            .prepare_artwork(
                &id,
                1,
                &source,
                ArtworkParameters {
                    outline: crate::ArtworkOutline {
                        radius: 1,
                        color: [10, 20, 30, 255],
                    },
                    cleanup: None,
                },
            )
            .unwrap()
            .manifest;
        (root, core, manifest)
    }
    #[test]
    fn artwork_failure_before_durable_journal_rolls_back_new_owned_bytes() {
        let (root, core, manifest) = prepared();
        let path = root.path().join("projects").join(&manifest.project_id);
        let project = std::fs::read(path.join("project.json")).unwrap();
        let history = std::fs::read(path.join("history.json")).unwrap();
        let target = path.join(crate::assets::hash_relative_path(&manifest.result_hash));
        assert!(!target.exists());
        core.persistence_faults
            .inject(PersistencePhase::BeforeJournal);
        assert!(core.commit_artwork(manifest.clone()).is_err());
        assert_eq!(std::fs::read(path.join("project.json")).unwrap(), project);
        assert_eq!(std::fs::read(path.join("history.json")).unwrap(), history);
        assert!(!target.exists());
        assert_eq!(core.get_project(&manifest.project_id).unwrap().revision, 1);
    }
    #[test]
    fn artwork_failure_after_commit_is_reported_committed_and_recovers_without_duplicate() {
        for phase in [
            PersistencePhase::AfterJournal,
            PersistencePhase::AfterProject,
            PersistencePhase::AfterHistory,
        ] {
            let (_root, core, manifest) = prepared();
            core.persistence_faults.inject(phase);
            let result = core.commit_artwork(manifest.clone()).unwrap();
            assert_eq!(result.revision, 2);
            assert!(
                result
                    .warnings
                    .iter()
                    .any(|w| w == PERSISTENCE_RECOVERY_PENDING)
            );
            let project = core.get_project(&manifest.project_id).unwrap();
            assert_eq!(project.revision, 2);
            assert_eq!(project.assets.len(), 2);
            assert_eq!(
                core.commit_artwork(manifest).unwrap_err().code,
                ErrorCode::RevisionConflict
            );
        }
    }
}
