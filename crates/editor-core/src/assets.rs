//! Canonical asset ownership and integrity rules.

use std::{
    collections::HashSet,
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    ContentHash, CoreError, EditOperation, ErrorCode, GeneratedAssetOrigin, History,
    MediaProbeFacts, MediaType, Project, TimelineItem,
    persistence::{PersistenceFaults, PersistencePhase, Storage, StorageEntryKind},
};

pub(crate) const ASSET_GC_FAILED: &str = "ASSET_GC_FAILED";

#[derive(Clone, Copy, Debug)]
pub(crate) struct DraftAssetOperations<'a> {
    pub(crate) id: &'a str,
    pub(crate) operations: &'a [EditOperation],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssetReferenceKind {
    MediaItem,
    CaptionSource,
    DraftOperation,
}

impl AssetReferenceKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MediaItem => "media item",
            Self::CaptionSource => "caption source",
            Self::DraftOperation => "draft operation",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssetReference {
    pub(crate) asset_id: String,
    pub(crate) kind: AssetReferenceKind,
    pub(crate) owner_id: String,
}

pub(crate) fn project_asset_references(project: &Project) -> Vec<AssetReference> {
    project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .filter_map(|item| match item {
            TimelineItem::Media(media) => Some(AssetReference {
                asset_id: media.asset_id.clone(),
                kind: AssetReferenceKind::MediaItem,
                owner_id: media.id.clone(),
            }),
            TimelineItem::Caption(caption) => Some(AssetReference {
                asset_id: caption.source.asset_id.clone(),
                kind: AssetReferenceKind::CaptionSource,
                owner_id: caption.id.clone(),
            }),
            TimelineItem::Text(_)
            | TimelineItem::SolidColor(_)
            | TimelineItem::Rectangle(_)
            | TimelineItem::Transition(_) => None,
        })
        .collect()
}

pub(crate) fn draft_asset_references(draft: DraftAssetOperations<'_>) -> Vec<AssetReference> {
    draft
        .operations
        .iter()
        .filter_map(|operation| match operation {
            EditOperation::AddMedia { asset_id, .. } => Some(AssetReference {
                asset_id: asset_id.clone(),
                kind: AssetReferenceKind::DraftOperation,
                owner_id: draft.id.to_owned(),
            }),
            _ => None,
        })
        .collect()
}

pub(crate) fn blocking_asset_reference(
    project: &Project,
    drafts: &[DraftAssetOperations<'_>],
    asset_id: &str,
) -> Option<AssetReference> {
    project_asset_references(project)
        .into_iter()
        .chain(drafts.iter().copied().flat_map(draft_asset_references))
        .find(|reference| reference.asset_id == asset_id)
}

pub(crate) fn validate_project_asset_references(
    project: &Project,
    context: &str,
) -> Result<(), CoreError> {
    let assets = project
        .assets
        .iter()
        .map(|asset| asset.id.as_str())
        .collect::<HashSet<_>>();
    if let Some(reference) = project_asset_references(project)
        .into_iter()
        .find(|reference| !assets.contains(reference.asset_id.as_str()))
    {
        return Err(CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            format!(
                "{context} contains dangling {} {} reference to asset {}",
                reference.kind.label(),
                reference.owner_id,
                reference.asset_id
            ),
        ));
    }
    Ok(())
}

pub(crate) fn validate_retained_project_references(
    project: &Project,
    history: &History,
) -> Result<(), CoreError> {
    validate_project_asset_references(project, "current project")?;
    for (index, snapshot) in history.undo.iter().enumerate() {
        validate_project_asset_references(snapshot, &format!("undo history snapshot {index}"))?;
    }
    for (index, snapshot) in history.redo.iter().enumerate() {
        validate_project_asset_references(snapshot, &format!("redo history snapshot {index}"))?;
    }
    Ok(())
}

pub(crate) fn validate_draft_asset_references(
    project: &Project,
    drafts: &[DraftAssetOperations<'_>],
) -> Result<(), CoreError> {
    if let Some(reference) = missing_draft_asset_reference(project, drafts) {
        return Err(CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            format!(
                "draft {} contains dangling {} reference to asset {}",
                reference.owner_id,
                reference.kind.label(),
                reference.asset_id
            ),
        ));
    }
    Ok(())
}

pub(crate) fn missing_draft_asset_reference(
    project: &Project,
    drafts: &[DraftAssetOperations<'_>],
) -> Option<AssetReference> {
    let assets = project
        .assets
        .iter()
        .map(|asset| asset.id.as_str())
        .collect::<HashSet<_>>();
    drafts
        .iter()
        .copied()
        .flat_map(draft_asset_references)
        .find(|reference| !assets.contains(reference.asset_id.as_str()))
}

pub(crate) struct StoredAsset {
    pub(crate) content_hash: ContentHash,
    pub(crate) relative_path: String,
    pub(crate) size_bytes: u64,
}

pub(crate) fn generated_display_name(origin: &GeneratedAssetOrigin) -> String {
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = origin;
    let raw_voice = generation.request.voice_id.0.as_str();
    let mut parts = raw_voice
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() > 1
        && parts[0].len() == 2
        && parts[0]
            .chars()
            .all(|character| character.is_ascii_lowercase())
    {
        parts.remove(0);
    }
    let voice = parts
        .into_iter()
        .map(title_case)
        .collect::<Vec<_>>()
        .join(" ");
    let excerpt = generation
        .request
        .text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let excerpt = excerpt.chars().take(48).collect::<String>();
    let stem = sanitize_file_name(&format!(
        "{} - {}",
        if voice.is_empty() { "Voice" } else { &voice },
        if excerpt.is_empty() {
            "Speech"
        } else {
            &excerpt
        }
    ));
    format!("{stem}.wav")
}

pub(crate) fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + characters.as_str()
    })
}

pub(crate) fn sanitize_file_name(value: &str) -> String {
    let mut sanitized = value
        .chars()
        .map(|character| {
            if character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    sanitized = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");
    sanitized = sanitized
        .trim_matches([' ', '.'])
        .chars()
        .take(96)
        .collect();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if sanitized.is_empty() {
        "Generated speech".into()
    } else if reserved
        .iter()
        .any(|name| sanitized.eq_ignore_ascii_case(name))
    {
        format!("_{sanitized}")
    } else {
        sanitized
    }
}

pub(crate) fn hash_file(storage: &dyn Storage, path: &Path) -> Result<(String, u64), CoreError> {
    let mut file = storage
        .open_read(path)
        .map_err(|_| CoreError::new(ErrorCode::AssetIntegrityFailed, "asset file is missing"))?;
    let mut hasher = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|_| {
            CoreError::new(ErrorCode::AssetIntegrityFailed, "cannot verify asset file")
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size = size.saturating_add(read as u64);
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

pub(crate) fn hash_relative_path(digest: &str) -> String {
    format!("assets/sha256/{}/{}", &digest[..2], digest)
}

pub(crate) fn store_content_addressed(
    storage: &dyn Storage,
    dir: &Path,
    source: &Path,
) -> Result<StoredAsset, CoreError> {
    let (digest, size_bytes) = hash_file(storage, source)?;
    let relative_path = hash_relative_path(&digest);
    let destination = dir.join(&relative_path);
    if !storage.storage_path_is_file(&destination) {
        let parent = destination
            .parent()
            .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "asset path has no parent"))?;
        storage
            .create_dir_all(parent)
            .map_err(|error| CoreError::io("cannot create asset store", error))?;
        let temporary = parent.join(format!(".{}.{}.tmp", digest, Uuid::new_v4()));
        storage
            .copy(source, &temporary)
            .map_err(|error| CoreError::io("cannot copy asset", error))?;
        if let Err(error) = storage.rename(&temporary, &destination) {
            if !storage.storage_path_is_file(&destination) {
                let _ = storage.remove_durable(&temporary);
                return Err(CoreError::io("cannot publish asset", error));
            }
            let _ = storage.remove_durable(&temporary);
        }
    }
    Ok(StoredAsset {
        content_hash: ContentHash {
            algorithm: "sha256".into(),
            digest,
        },
        relative_path,
        size_bytes,
    })
}

pub(crate) fn migrate_project_assets(
    storage: &dyn Storage,
    project: &mut Project,
    dir: &Path,
) -> Result<bool, CoreError> {
    let mut changed = false;
    for asset in &mut project.assets {
        if let Some(probe) = &asset.probe
            && (probe.duration_ms != asset.duration_ms || probe.has_audio != asset.has_audio)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "asset probe facts do not match compatibility fields",
            ));
        }
        let source = dir.join(&asset.project_relative_path);
        let (digest, size_bytes) = hash_file(storage, &source)?;
        if let Some(content_hash) = &asset.content_hash
            && (content_hash.algorithm != "sha256" || content_hash.digest != digest)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "asset content hash does not match project metadata",
            ));
        }
        if asset.size_bytes.is_some_and(|stored| stored != size_bytes) {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "asset size does not match project metadata",
            ));
        }
        let stored = store_content_addressed(storage, dir, &source)?;
        if asset.project_relative_path != stored.relative_path {
            asset.project_relative_path = stored.relative_path;
            changed = true;
        }
        if asset.content_hash.is_none() {
            asset.content_hash = Some(stored.content_hash);
            changed = true;
        }
        if asset.size_bytes.is_none() {
            asset.size_bytes = Some(stored.size_bytes);
            changed = true;
        }
        if asset.probe.is_none() {
            asset.probe = Some(MediaProbeFacts {
                duration_ms: asset.duration_ms,
                has_audio: asset.has_audio,
                has_video: asset.media_type != MediaType::Audio,
                ..MediaProbeFacts::default()
            });
            changed = true;
        }
    }
    Ok(changed)
}

pub(crate) fn retained_managed_paths(
    project: &Project,
    history: &History,
    drafts: &[DraftAssetOperations<'_>],
) -> Result<HashSet<String>, CoreError> {
    validate_retained_project_references(project, history)?;
    validate_draft_asset_references(project, drafts)?;
    let mut referenced = std::iter::once(project)
        .chain(history.undo.iter())
        .chain(history.redo.iter())
        .flat_map(|snapshot| snapshot.assets.iter())
        .map(|asset| asset.project_relative_path.replace('\\', "/"))
        .collect::<HashSet<_>>();
    for reference in drafts.iter().copied().flat_map(draft_asset_references) {
        let asset = project
            .assets
            .iter()
            .find(|asset| asset.id == reference.asset_id)
            .expect("draft references were validated against current assets");
        referenced.insert(asset.project_relative_path.replace('\\', "/"));
    }
    Ok(referenced)
}

pub(crate) fn garbage_collect(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    project: &Project,
    history: &History,
    drafts: &[DraftAssetOperations<'_>],
) -> Vec<String> {
    let referenced = match retained_managed_paths(project, history, drafts) {
        Ok(referenced) => referenced,
        Err(_) => return vec![ASSET_GC_FAILED.into()],
    };
    let mut files = Vec::new();
    if collect_files(storage, &dir.join("assets"), &mut files).is_err() {
        return vec![ASSET_GC_FAILED.into()];
    }
    let mut failed = false;
    for file in files {
        let relative = file
            .strip_prefix(dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        if !referenced.contains(&relative)
            && (faults
                .checkpoint(PersistencePhase::GarbageCollection)
                .is_err()
                || storage.remove_durable(&file).is_err())
        {
            failed = true;
        }
    }
    if failed {
        vec![ASSET_GC_FAILED.into()]
    } else {
        Vec::new()
    }
}

fn collect_files(
    storage: &dyn Storage,
    directory: &Path,
    output: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    match storage.entry_kind(directory) {
        Ok(StorageEntryKind::Directory) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Ok(_) => return Ok(()),
        Err(error) => return Err(error),
    }
    for path in storage.list(directory)? {
        match storage.entry_kind(&path)? {
            StorageEntryKind::Directory => collect_files(storage, &path, output)?,
            StorageEntryKind::File => output.push(path),
            StorageEntryKind::Symlink | StorageEntryKind::Other => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Asset, AudioSettings, AudioTrackRole, MediaItem, PROJECT_SCHEMA_VERSION, ProjectSettings,
        Track, TrackType,
        persistence::{FileSystemStorage, StorageLock},
    };
    use std::sync::Mutex;

    #[derive(Debug)]
    struct GcStorage {
        classify_failure: bool,
        remove_failure: bool,
        removed: Mutex<Vec<PathBuf>>,
    }

    impl Storage for GcStorage {
        fn lock_exclusive(&self, _dir: &Path) -> Result<Box<dyn StorageLock>, CoreError> {
            Err(CoreError::new(ErrorCode::InternalError, "unused"))
        }
        fn open_read(&self, _path: &Path) -> std::io::Result<Box<dyn Read>> {
            Err(std::io::Error::other("unused"))
        }
        fn read(&self, _path: &Path) -> std::io::Result<Vec<u8>> {
            Err(std::io::Error::other("unused"))
        }
        fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
            Ok(vec![path.join("orphan.bin")])
        }
        fn create_dir_all(&self, _path: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("unused"))
        }
        fn copy(&self, _from: &Path, _to: &Path) -> std::io::Result<u64> {
            Err(std::io::Error::other("unused"))
        }
        fn rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("unused"))
        }
        fn storage_path_is_file(&self, path: &Path) -> bool {
            path.extension().is_some()
        }
        fn storage_path_exists(&self, _path: &Path) -> bool {
            true
        }
        fn entry_kind(&self, path: &Path) -> std::io::Result<StorageEntryKind> {
            if self.classify_failure && path.extension().is_some() {
                Err(std::io::Error::other("injected classification failure"))
            } else if path.extension().is_some() {
                Ok(StorageEntryKind::File)
            } else {
                Ok(StorageEntryKind::Directory)
            }
        }
        fn canonicalize_storage_path(&self, path: &Path) -> std::io::Result<PathBuf> {
            Ok(path.to_owned())
        }
        fn atomic_replace(&self, _path: &Path, _bytes: &[u8]) -> std::io::Result<()> {
            Err(std::io::Error::other("unused"))
        }
        fn remove_durable(&self, path: &Path) -> std::io::Result<()> {
            if self.remove_failure {
                Err(std::io::Error::other("injected remove failure"))
            } else {
                self.removed.lock().unwrap().push(path.to_owned());
                Ok(())
            }
        }
    }

    fn store_content_addressed(dir: &Path, source: &Path) -> Result<StoredAsset, CoreError> {
        super::store_content_addressed(&FileSystemStorage, dir, source)
    }

    fn garbage_collect_with(
        storage: &dyn Storage,
        faults: &PersistenceFaults,
        dir: &Path,
        project: &Project,
        history: &History,
        drafts: &[DraftAssetOperations<'_>],
    ) -> Vec<String> {
        super::garbage_collect(storage, faults, dir, project, history, drafts)
    }

    fn project_with_asset() -> Project {
        Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![Asset {
                id: "asset".into(),
                media_type: MediaType::Video,
                file_name: "clip.mp4".into(),
                project_relative_path: "assets/sha256/aa/aabb".into(),
                duration_ms: Some(1_000),
                has_audio: true,
                origin: None,
                content_hash: None,
                size_bytes: None,
                probe: None,
            }],
            tracks: vec![Track {
                id: "video".into(),
                name: "Video".into(),
                track_type: TrackType::Video,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![TimelineItem::Media(MediaItem {
                    id: "item".into(),
                    asset_id: "asset".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    source_in_ms: 0,
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                })],
            }],
        }
    }

    #[test]
    fn inventory_classifies_current_and_draft_references() {
        let project = project_with_asset();
        let operations = vec![EditOperation::AddMedia {
            track_id: "video".into(),
            asset_id: "asset".into(),
            start_ms: 1_000,
            duration_ms: 1_000,
            source_in_ms: 0,
        }];
        let draft = DraftAssetOperations {
            id: "draft",
            operations: &operations,
        };
        assert_eq!(
            project_asset_references(&project)[0].kind,
            AssetReferenceKind::MediaItem
        );
        assert_eq!(
            draft_asset_references(draft)[0].kind,
            AssetReferenceKind::DraftOperation
        );
        assert_eq!(
            blocking_asset_reference(&project, &[draft], "asset")
                .unwrap()
                .owner_id,
            "item"
        );
    }

    #[test]
    fn integrity_and_retention_share_the_same_inventory() {
        let project = project_with_asset();
        let history = History::default();
        validate_retained_project_references(&project, &history).unwrap();
        let retained = retained_managed_paths(&project, &history, &[]).unwrap();
        assert!(retained.contains("assets/sha256/aa/aabb"));

        let mut dangling = project;
        dangling.assets.clear();
        assert_eq!(
            validate_project_asset_references(&dangling, "current project")
                .unwrap_err()
                .code,
            ErrorCode::AssetIntegrityFailed
        );
    }

    #[test]
    fn content_addressing_is_deterministic() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        std::fs::write(&source, b"same bytes").unwrap();
        let first = store_content_addressed(directory.path(), &source).unwrap();
        let second = store_content_addressed(directory.path(), &source).unwrap();
        assert_eq!(first.relative_path, second.relative_path);
        assert_eq!(first.content_hash, second.content_hash);
        assert!(directory.path().join(first.relative_path).is_file());
    }

    #[test]
    fn garbage_collection_surfaces_storage_classification_and_deletion_failures() {
        let project = Project {
            assets: vec![],
            tracks: vec![],
            ..project_with_asset()
        };
        let root = Path::new("project");
        for storage in [
            GcStorage {
                classify_failure: true,
                remove_failure: false,
                removed: Mutex::new(vec![]),
            },
            GcStorage {
                classify_failure: false,
                remove_failure: true,
                removed: Mutex::new(vec![]),
            },
        ] {
            assert_eq!(
                garbage_collect_with(
                    &storage,
                    &PersistenceFaults::default(),
                    root,
                    &project,
                    &History::default(),
                    &[],
                ),
                vec![ASSET_GC_FAILED]
            );
        }

        let storage = GcStorage {
            classify_failure: false,
            remove_failure: false,
            removed: Mutex::new(vec![]),
        };
        assert!(
            garbage_collect_with(
                &storage,
                &PersistenceFaults::default(),
                root,
                &project,
                &History::default(),
                &[],
            )
            .is_empty()
        );
        assert_eq!(
            storage.removed.lock().unwrap().as_slice(),
            &[root.join("assets/orphan.bin")]
        );
    }
}
