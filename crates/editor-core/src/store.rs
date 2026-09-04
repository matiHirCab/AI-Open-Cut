use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    Asset, AudioSettings, AudioTrackRole, BatchEditOperation, CaptionItem, CaptionSource,
    CaptionStyle, CaptionWord, ContentHash, CoreError, EditOperation, ErrorCode,
    GeneratedAssetOrigin, History, MediaItem, MediaProbeFacts, MediaType, PROJECT_SCHEMA_VERSION,
    PathPolicy, Project, ProjectSettings, ProjectState, TimelineItem, Track, TrackType,
    assets::{
        ASSET_GC_FAILED, DraftAssetOperations, blocking_asset_reference, garbage_collect,
        generated_display_name, migrate_project_assets, missing_draft_asset_reference,
        store_content_addressed, validate_draft_asset_references,
        validate_retained_project_references,
    },
    drafts::{
        DRAFT_VERSION, EditDraft, count_drafts, draft_dir, draft_path, read_all_drafts, read_draft,
        remove_draft,
    },
    migrations::migrate_project_documents,
    persistence::{
        FileSystemStorage, PERSISTENCE_RECOVERY_PENDING, PersistenceFaults, Storage,
        TRANSACTION_FILE, history_path, list_project_directories, persist_transaction,
        project_path, read_json, recover_transaction, transaction_path, write_json_atomic,
    },
    timeline::{
        apply_operation, bump_revision, check_revision, is_single_id_creator, now_ms, push_undo,
        resolve_operation_aliases, validate_alias, validate_operations_against,
    },
    validation::{
        validate_color, validate_draft_label, validate_duration, validate_project_settings,
        validate_project_visual_properties, validate_text, validate_track_media,
    },
};

#[cfg(test)]
use crate::persistence::{
    DRAFT_CLEANUP_FAILED, PersistencePhase, ProjectTransaction, StorageEntryKind,
    TRANSACTION_VERSION,
};

const DRAFT_LIMIT: usize = 100;
const EDIT_LIMIT: usize = 100;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    pub project_id: String,
    pub revision: u64,
    pub changed_ids: Vec<String>,
    pub summary: String,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub settings: ProjectSettings,
    pub duration_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: Option<f64>,
    #[serde(default)]
    pub words: Vec<CaptionWord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitTranscriptionRequest {
    pub project_id: String,
    pub expected_revision: u64,
    pub asset_id: String,
    pub caption_track_id: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub model_version: Option<String>,
    pub language: String,
    pub generated_at_ms: u64,
    pub segments: Vec<TranscriptionSegment>,
    pub style: CaptionStyle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedAssetInput {
    pub project_id: String,
    pub revision: u64,
    pub asset_id: String,
    pub path: PathBuf,
    pub content_hash: Option<ContentHash>,
    pub probe: Option<MediaProbeFacts>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitGeneratedAssetRequest {
    pub project_id: String,
    pub expected_revision: u64,
    pub path: PathBuf,
    pub track_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub display_name: String,
    pub origin: GeneratedAssetOrigin,
    #[serde(default)]
    pub probe: MediaProbeFacts,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitGeneratedAssetResult {
    pub project_id: String,
    pub revision: u64,
    pub asset_id: String,
    pub item_id: String,
    pub summary: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplaceGeneratedAssetRequest {
    pub project_id: String,
    pub expected_revision: u64,
    pub path: PathBuf,
    pub item_id: String,
    pub duration_ms: u64,
    pub origin: GeneratedAssetOrigin,
    #[serde(default)]
    pub probe: MediaProbeFacts,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplaceGeneratedAssetResult {
    pub project_id: String,
    pub revision: u64,
    pub asset_id: String,
    pub item_id: String,
    pub replaced_asset_id: String,
    pub summary: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EditorCore {
    paths: PathPolicy,
    storage: Arc<dyn Storage>,
    persistence_faults: PersistenceFaults,
}

impl EditorCore {
    pub fn new(paths: PathPolicy) -> Self {
        Self {
            paths,
            storage: Arc::new(FileSystemStorage),
            persistence_faults: PersistenceFaults::default(),
        }
    }

    #[cfg(test)]
    fn with_storage(paths: PathPolicy, storage: Arc<dyn Storage>) -> Self {
        Self {
            paths,
            storage,
            persistence_faults: PersistenceFaults::default(),
        }
    }

    pub fn paths(&self) -> &PathPolicy {
        &self.paths
    }

    pub fn create_project(
        &self,
        name: &str,
        settings: ProjectSettings,
    ) -> Result<WriteResult, CoreError> {
        validate_project_settings(&settings)?;
        if name.trim().is_empty() {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "project name cannot be empty",
            ));
        }
        let id = Uuid::new_v4().to_string();
        let project_dir = self.paths.project_dir(&id)?;
        self.storage
            .create_dir_all(&project_dir.join("assets"))
            .map_err(|error| CoreError::io("cannot create project assets", error))?;
        self.storage
            .create_dir_all(&project_dir.join("previews"))
            .map_err(|error| CoreError::io("cannot create project previews", error))?;
        let now = now_ms()?;
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: id.clone(),
            revision: 0,
            name: name.trim().to_owned(),
            created_at_ms: now,
            updated_at_ms: now,
            settings,
            assets: vec![],
            tracks: vec![
                Track {
                    id: Uuid::new_v4().to_string(),
                    name: "Video".into(),
                    track_type: TrackType::Video,
                    locked: false,
                    hidden: false,
                    muted: false,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![],
                },
                Track {
                    id: Uuid::new_v4().to_string(),
                    name: "Overlay".into(),
                    track_type: TrackType::Overlay,
                    locked: false,
                    hidden: false,
                    muted: false,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![],
                },
                Track {
                    id: Uuid::new_v4().to_string(),
                    name: "Audio".into(),
                    track_type: TrackType::Audio,
                    locked: false,
                    hidden: false,
                    muted: false,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![],
                },
                Track {
                    id: Uuid::new_v4().to_string(),
                    name: "Captions".into(),
                    track_type: TrackType::Caption,
                    locked: false,
                    hidden: false,
                    muted: false,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![],
                },
            ],
        };
        let _lock = self.storage.lock_exclusive(&project_dir)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &project_dir,
            &project,
            &History::default(),
        )?;
        Ok(WriteResult {
            project_id: id.clone(),
            revision: 0,
            changed_ids: vec![id],
            summary: "Created project".into(),
            warnings,
            aliases: BTreeMap::new(),
        })
    }

    pub fn get_project(&self, project_id: &str) -> Result<Project, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        let _ = collect_asset_garbage(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        );
        Ok(project)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectSummary>, CoreError> {
        let mut summaries = Vec::new();
        let entries = list_project_directories(self.storage.as_ref(), self.paths.projects_root())?;
        for path in entries {
            let Some(id) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
            else {
                continue;
            };
            if !self
                .storage
                .storage_path_is_file(&path.join("project.json"))
                && !self
                    .storage
                    .storage_path_is_file(&path.join(TRANSACTION_FILE))
            {
                continue;
            }
            let project = self.get_project(&id)?;
            summaries.push(ProjectSummary {
                id: project.id.clone(),
                name: project.name.clone(),
                revision: project.revision,
                settings: project.settings.clone(),
                duration_ms: project.duration_ms(),
                updated_at_ms: project.updated_at_ms,
            });
        }
        summaries.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(summaries)
    }

    pub fn get_state(
        &self,
        project_id: &str,
        range: Option<(u64, u64)>,
    ) -> Result<ProjectState, CoreError> {
        let mut project = self.get_project(project_id)?;
        let duration_ms = project.duration_ms();
        if let Some((start_ms, end_ms)) = range {
            if start_ms >= end_ms {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "time range must satisfy startMs < endMs",
                ));
            }
            for track in &mut project.tracks {
                track.items.retain(|item| item.overlaps(start_ms, end_ms));
            }
        }
        Ok(ProjectState {
            project,
            duration_ms,
        })
    }

    pub fn import_asset(
        &self,
        project_id: &str,
        expected_revision: u64,
        requested_path: impl AsRef<Path>,
        media_type: MediaType,
        probe: MediaProbeFacts,
    ) -> Result<WriteResult, CoreError> {
        let source = self.paths.import_path(requested_path)?;
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let asset_id = Uuid::new_v4().to_string();
        let stored = store_content_addressed(self.storage.as_ref(), &dir, &source)?;
        push_undo(&mut history, &project);
        project.assets.push(Asset {
            id: asset_id.clone(),
            media_type,
            file_name: source
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("media")
                .to_owned(),
            project_relative_path: stored.relative_path,
            duration_ms: probe.duration_ms,
            has_audio: probe.has_audio,
            origin: None,
            content_hash: Some(stored.content_hash),
            size_bytes: Some(stored.size_bytes),
            probe: Some(probe),
        });
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let mut result = write_result(&project, vec![asset_id], "Imported asset");
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn commit_generated_asset(
        &self,
        mut request: CommitGeneratedAssetRequest,
    ) -> Result<CommitGeneratedAssetResult, CoreError> {
        validate_duration(request.duration_ms)?;
        request.origin.validate()?;
        request.probe.duration_ms = Some(request.duration_ms);
        request.probe.has_audio = true;
        let source = self.paths.generated_media_path(&request.path)?;
        let dir = self.existing_project_dir(&request.project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, request.expected_revision)?;
        let track_index = project
            .tracks
            .iter()
            .position(|track| track.id == request.track_id)
            .ok_or_else(|| CoreError::new(ErrorCode::ValidationFailed, "track was not found"))?;
        validate_track_media(project.tracks[track_index].track_type, MediaType::Audio)?;

        let previous = project.clone();
        let asset_id = Uuid::new_v4().to_string();
        let item_id = Uuid::new_v4().to_string();
        let stored = store_content_addressed(self.storage.as_ref(), &dir, &source)?;
        let display_name = generated_display_name(&request.origin);

        project.assets.push(Asset {
            id: asset_id.clone(),
            media_type: MediaType::Audio,
            file_name: display_name,
            project_relative_path: stored.relative_path,
            duration_ms: Some(request.duration_ms),
            has_audio: true,
            origin: Some(request.origin),
            content_hash: Some(stored.content_hash),
            size_bytes: Some(stored.size_bytes),
            probe: Some(request.probe),
        });
        project.tracks[track_index]
            .items
            .push(TimelineItem::Media(MediaItem {
                id: item_id.clone(),
                asset_id: asset_id.clone(),
                start_ms: request.start_ms,
                duration_ms: request.duration_ms,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
            }));
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(CommitGeneratedAssetResult {
            project_id: project.id.clone(),
            revision: project.revision,
            asset_id,
            item_id,
            summary: "Generated and inserted speech".into(),
            warnings,
        })
    }

    pub fn delete_asset(
        &self,
        project_id: &str,
        expected_revision: u64,
        asset_id: &str,
    ) -> Result<WriteResult, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let index = project
            .assets
            .iter()
            .position(|asset| asset.id == asset_id)
            .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
        let drafts = read_all_drafts(self.storage.as_ref(), &dir)?;
        let asset_drafts = draft_asset_operations(&drafts);
        validate_draft_asset_references(&project, &asset_drafts)?;
        if let Some(reference) = blocking_asset_reference(&project, &asset_drafts, asset_id) {
            return Err(CoreError::new(
                ErrorCode::AssetInUse,
                format!(
                    "asset is referenced by {} {}",
                    reference.kind.label(),
                    reference.owner_id
                ),
            ));
        }
        let previous = project.clone();
        project.assets.remove(index);
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let mut result = write_result(&project, vec![asset_id.to_owned()], "Deleted asset");
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn replace_generated_asset(
        &self,
        mut request: ReplaceGeneratedAssetRequest,
    ) -> Result<ReplaceGeneratedAssetResult, CoreError> {
        validate_duration(request.duration_ms)?;
        request.origin.validate()?;
        request.probe.duration_ms = Some(request.duration_ms);
        request.probe.has_audio = true;
        let source = self.paths.generated_media_path(&request.path)?;
        let dir = self.existing_project_dir(&request.project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, request.expected_revision)?;
        let drafts = read_all_drafts(self.storage.as_ref(), &dir)?;
        let asset_drafts = draft_asset_operations(&drafts);
        validate_draft_asset_references(&project, &asset_drafts)?;
        let (track_index, item_index) = project
            .tracks
            .iter()
            .enumerate()
            .find_map(|(track_index, track)| {
                track
                    .items
                    .iter()
                    .position(|item| item.id() == request.item_id)
                    .map(|item_index| (track_index, item_index))
            })
            .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "item was not found"))?;
        let replaced_asset_id = match &project.tracks[track_index].items[item_index] {
            TimelineItem::Media(item) => item.asset_id.clone(),
            _ => {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "speech regeneration requires a media item",
                ));
            }
        };
        let replaced_asset = project
            .assets
            .iter()
            .find(|asset| asset.id == replaced_asset_id)
            .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
        if !matches!(
            replaced_asset.origin,
            Some(GeneratedAssetOrigin::SpeechSynthesis(_))
        ) {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "item does not contain persisted speech intent",
            ));
        }

        let previous = project.clone();
        let asset_id = Uuid::new_v4().to_string();
        let stored = store_content_addressed(self.storage.as_ref(), &dir, &source)?;
        project.assets.push(Asset {
            id: asset_id.clone(),
            media_type: MediaType::Audio,
            file_name: generated_display_name(&request.origin),
            project_relative_path: stored.relative_path,
            duration_ms: Some(request.duration_ms),
            has_audio: true,
            origin: Some(request.origin),
            content_hash: Some(stored.content_hash),
            size_bytes: Some(stored.size_bytes),
            probe: Some(request.probe),
        });
        let TimelineItem::Media(item) = &mut project.tracks[track_index].items[item_index] else {
            unreachable!("media item was checked above")
        };
        item.asset_id = asset_id.clone();
        item.duration_ms = request.duration_ms;
        if blocking_asset_reference(&project, &asset_drafts, &replaced_asset_id).is_none() {
            project.assets.retain(|asset| asset.id != replaced_asset_id);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(ReplaceGeneratedAssetResult {
            project_id: project.id,
            revision: project.revision,
            asset_id,
            item_id: request.item_id,
            replaced_asset_id,
            summary: "Regenerated speech in place".into(),
            warnings,
        })
    }

    pub fn edit(
        &self,
        project_id: &str,
        expected_revision: u64,
        operation: EditOperation,
    ) -> Result<WriteResult, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let previous = project.clone();
        let (changed_ids, summary) = apply_operation(&mut project, operation)?;
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let mut result = write_result(&project, changed_ids, summary);
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn edit_batch<T: Into<BatchEditOperation>>(
        &self,
        project_id: &str,
        expected_revision: u64,
        operations: Vec<T>,
    ) -> Result<WriteResult, CoreError> {
        if operations.is_empty() || operations.len() > EDIT_LIMIT {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "edit batches must contain between 1 and 100 operations",
            ));
        }
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let previous = project.clone();
        let mut changed_ids = Vec::new();
        let mut aliases = BTreeMap::new();
        for batch_operation in operations {
            let BatchEditOperation {
                mut edit,
                result_alias,
            } = batch_operation.into();
            resolve_operation_aliases(&mut edit, &aliases)?;
            if result_alias.is_some() && !is_single_id_creator(&edit) {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "resultAlias requires an operation that creates exactly one ID",
                ));
            }
            if let Some(alias) = result_alias.as_deref() {
                validate_alias(alias)?;
                if aliases.contains_key(alias) {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "resultAlias must be unique within the batch",
                    ));
                }
            }
            let operation = edit;
            let (ids, _) = apply_operation(&mut project, operation)?;
            if let Some(alias) = result_alias {
                let id = ids.first().ok_or_else(|| {
                    CoreError::new(ErrorCode::InternalError, "aliased operation returned no ID")
                })?;
                aliases.insert(alias, id.clone());
            }
            changed_ids.extend(ids);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let mut result = write_result(&project, changed_ids, "Applied timeline edit batch");
        result.aliases = aliases;
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn create_draft(
        &self,
        project_id: &str,
        expected_revision: u64,
        operations: Vec<EditOperation>,
        label: Option<String>,
    ) -> Result<EditDraft, CoreError> {
        validate_operations(&operations)?;
        validate_draft_label(label.as_deref())?;
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, _) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        validate_operations_against(&project, &operations)?;
        let drafts = draft_dir(&dir);
        self.storage
            .create_dir_all(&drafts)
            .map_err(|error| CoreError::io("cannot create draft directory", error))?;
        if count_drafts(self.storage.as_ref(), &drafts)? >= DRAFT_LIMIT {
            return Err(CoreError::new(
                ErrorCode::DraftLimitReached,
                "project has reached the retained draft limit",
            ));
        }
        let now = now_ms()?;
        let draft = EditDraft {
            version: DRAFT_VERSION,
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            base_revision: expected_revision,
            label,
            operations,
            created_at_ms: now,
            updated_at_ms: now,
        };
        write_json_atomic(self.storage.as_ref(), &draft_path(&dir, &draft.id)?, &draft)?;
        Ok(draft)
    }

    pub fn get_draft(&self, project_id: &str, draft_id: &str) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, _) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        let draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        validate_single_draft_assets(&project, &draft)?;
        Ok(draft)
    }

    pub fn update_draft(
        &self,
        project_id: &str,
        draft_id: &str,
        expected_revision: u64,
        operations: Vec<EditOperation>,
        label: Option<String>,
    ) -> Result<EditDraft, CoreError> {
        validate_operations(&operations)?;
        validate_draft_label(label.as_deref())?;
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, _) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let mut draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        if draft.base_revision != expected_revision {
            return Err(CoreError::new(
                ErrorCode::RevisionConflict,
                format!(
                    "draft is based on revision {}, current revision is {expected_revision}",
                    draft.base_revision
                ),
            ));
        }
        validate_operations_against(&project, &operations)?;
        draft.operations = operations;
        draft.label = label;
        draft.updated_at_ms = now_ms()?;
        write_json_atomic(self.storage.as_ref(), &draft_path(&dir, draft_id)?, &draft)?;
        Ok(draft)
    }

    pub fn rebase_draft(
        &self,
        project_id: &str,
        draft_id: &str,
        expected_revision: u64,
    ) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, _) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let mut draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        validate_single_draft_assets(&project, &draft)?;
        validate_operations_against(&project, &draft.operations)?;
        draft.base_revision = expected_revision;
        draft.updated_at_ms = now_ms()?;
        write_json_atomic(self.storage.as_ref(), &draft_path(&dir, draft_id)?, &draft)?;
        Ok(draft)
    }

    pub fn get_draft_state(
        &self,
        project_id: &str,
        draft_id: &str,
    ) -> Result<ProjectState, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, _) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        let draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        validate_single_draft_assets(&project, &draft)?;
        check_revision(&project, draft.base_revision)?;
        for operation in draft.operations {
            apply_operation(&mut project, operation)?;
        }
        let duration_ms = project.duration_ms();
        Ok(ProjectState {
            project,
            duration_ms,
        })
    }

    pub fn commit_draft(
        &self,
        project_id: &str,
        draft_id: &str,
        expected_revision: u64,
    ) -> Result<WriteResult, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        validate_single_draft_assets(&project, &draft)?;
        if draft.base_revision != expected_revision {
            return Err(CoreError::new(
                ErrorCode::RevisionConflict,
                format!(
                    "draft is based on revision {}, current revision is {expected_revision}",
                    draft.base_revision
                ),
            ));
        }
        let previous = project.clone();
        let mut changed_ids = Vec::new();
        for operation in draft.operations {
            let (ids, _) = apply_operation(&mut project, operation)?;
            changed_ids.extend(ids);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist_transaction(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            Some(draft_id),
        )?;
        let mut result = write_result(&project, changed_ids, "Committed edit draft");
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn discard_draft(&self, project_id: &str, draft_id: &str) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let _ = load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        let draft = read_draft(self.storage.as_ref(), &dir, draft_id)?;
        remove_draft(self.storage.as_ref(), &dir, draft_id)?;
        Ok(draft)
    }

    pub fn project_directory(&self, project_id: &str) -> Result<PathBuf, CoreError> {
        self.existing_project_dir(project_id)
    }

    pub fn resolve_asset_input(
        &self,
        project_id: &str,
        asset_id: &str,
    ) -> Result<ResolvedAssetInput, CoreError> {
        let project = self.get_project(project_id)?;
        let asset = project
            .assets
            .iter()
            .find(|asset| asset.id == asset_id)
            .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
        if !asset.has_audio {
            return Err(CoreError::new(
                ErrorCode::UnsupportedMedia,
                "transcription requires an asset with audio",
            ));
        }
        Ok(ResolvedAssetInput {
            project_id: project.id.clone(),
            revision: project.revision,
            asset_id: asset.id.clone(),
            path: self.project_asset_path(project_id, &asset.project_relative_path)?,
            content_hash: asset.content_hash.clone(),
            probe: asset.probe.clone(),
        })
    }

    pub fn commit_transcription(
        &self,
        request: CommitTranscriptionRequest,
    ) -> Result<WriteResult, CoreError> {
        validate_transcription_request(&request)?;
        let dir = self.existing_project_dir(&request.project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (mut project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, request.expected_revision)?;
        let asset = project
            .assets
            .iter()
            .find(|asset| asset.id == request.asset_id)
            .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
        if !asset.has_audio {
            return Err(CoreError::new(
                ErrorCode::UnsupportedMedia,
                "transcription requires an asset with audio",
            ));
        }
        let previous = project.clone();
        let mut changed_ids = Vec::new();
        let track_index = if let Some(track_id) = request.caption_track_id.as_deref() {
            let index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))?;
            if project.tracks[index].track_type != TrackType::Caption {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "transcription requires a caption track",
                ));
            }
            if project.tracks[index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            index
        } else if let Some(index) = project
            .tracks
            .iter()
            .position(|track| track.track_type == TrackType::Caption && !track.locked)
        {
            index
        } else {
            let track_id = Uuid::new_v4().to_string();
            changed_ids.push(track_id.clone());
            project.tracks.push(Track {
                id: track_id,
                name: "Captions".into(),
                track_type: TrackType::Caption,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![],
            });
            project.tracks.len() - 1
        };
        for segment in request.segments {
            let id = Uuid::new_v4().to_string();
            let source = CaptionSource {
                asset_id: request.asset_id.clone(),
                provider_id: request.provider_id.clone(),
                model_id: request.model_id.clone(),
                model_version: request.model_version.clone(),
                language: request.language.clone(),
                generated_at_ms: request.generated_at_ms,
                original_text: segment.text.clone(),
                confidence: segment.confidence,
                words: segment.words,
            };
            project.tracks[track_index]
                .items
                .push(TimelineItem::Caption(CaptionItem {
                    id: id.clone(),
                    text: segment.text,
                    start_ms: segment.start_ms,
                    duration_ms: segment.end_ms - segment.start_ms,
                    style: request.style.clone(),
                    source,
                    visual_properties: crate::VisualProperties::default(),
                }));
            changed_ids.push(id);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
        )?;
        let mut result = write_result(&project, changed_ids, "Committed transcription captions");
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &project,
            &history,
            warnings,
        );
        Ok(result)
    }

    pub fn undo(&self, project_id: &str, expected_revision: u64) -> Result<WriteResult, CoreError> {
        self.apply_history(project_id, expected_revision, true)
    }

    pub fn redo(&self, project_id: &str, expected_revision: u64) -> Result<WriteResult, CoreError> {
        self.apply_history(project_id, expected_revision, false)
    }

    pub fn validate_revision(
        &self,
        project_id: &str,
        expected_revision: u64,
    ) -> Result<Project, CoreError> {
        let project = self.get_project(project_id)?;
        check_revision(&project, expected_revision)?;
        Ok(project)
    }

    pub fn project_asset_path(
        &self,
        project_id: &str,
        relative: &str,
    ) -> Result<PathBuf, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let candidate = dir.join(relative);
        let resolved = self
            .storage
            .canonicalize_storage_path(&candidate)
            .map_err(|error| CoreError::io("cannot resolve project asset", error))?;
        if !resolved.starts_with(&dir) {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "project asset path escapes its project",
            ));
        }
        Ok(resolved)
    }

    fn apply_history(
        &self,
        project_id: &str,
        expected_revision: u64,
        undo: bool,
    ) -> Result<WriteResult, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        let (project, mut history) =
            load_project_data(self.storage.as_ref(), &self.persistence_faults, &dir)?;
        check_revision(&project, expected_revision)?;
        let target = if undo {
            history.undo.pop()
        } else {
            history.redo.pop()
        }
        .ok_or_else(|| {
            CoreError::new(
                ErrorCode::ValidationFailed,
                if undo {
                    "nothing to undo"
                } else {
                    "nothing to redo"
                },
            )
        })?;
        let mut restored = target;
        restored.revision = project
            .revision
            .checked_add(1)
            .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "project revision overflow"))?;
        restored.updated_at_ms = now_ms()?;
        let drafts = read_all_drafts(self.storage.as_ref(), &dir)?;
        let asset_drafts = draft_asset_operations(&drafts);
        validate_draft_asset_references(&project, &asset_drafts)?;
        if let Some(reference) = missing_draft_asset_reference(&restored, &asset_drafts) {
            return Err(CoreError::new(
                ErrorCode::AssetInUse,
                format!(
                    "history change would detach {} {} from asset {}",
                    reference.kind.label(),
                    reference.owner_id,
                    reference.asset_id
                ),
            ));
        }
        if undo {
            history.redo.push(project);
        } else {
            history.undo.push(project);
        }
        let warnings = persist(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &restored,
            &history,
        )?;
        let mut result = write_result(
            &restored,
            vec![],
            if undo {
                "Undid last edit"
            } else {
                "Redid last edit"
            },
        );
        result.warnings = finish_persistence(
            self.storage.as_ref(),
            &self.persistence_faults,
            &dir,
            &restored,
            &history,
            warnings,
        );
        Ok(result)
    }

    fn existing_project_dir(&self, project_id: &str) -> Result<PathBuf, CoreError> {
        let dir = self.paths.project_dir(project_id)?;
        let resolved = match self.storage.canonicalize_storage_path(&dir) {
            Ok(resolved) => resolved,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CoreError::new(
                    ErrorCode::ProjectNotFound,
                    "project was not found",
                ));
            }
            Err(error) => return Err(CoreError::io("cannot resolve project directory", error)),
        };
        if !resolved.starts_with(self.paths.projects_root()) {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "project directory escapes the configured project root",
            ));
        }
        if !self.storage.storage_path_is_file(&project_path(&resolved))
            && !self
                .storage
                .storage_path_is_file(&transaction_path(&resolved))
        {
            return Err(CoreError::new(
                ErrorCode::ProjectNotFound,
                "project was not found",
            ));
        }
        Ok(resolved)
    }
}

fn validate_operations(operations: &[EditOperation]) -> Result<(), CoreError> {
    if operations.is_empty() || operations.len() > EDIT_LIMIT {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "edit batches must contain between 1 and 100 operations",
        ));
    }
    Ok(())
}

fn validate_transcription_request(request: &CommitTranscriptionRequest) -> Result<(), CoreError> {
    if request.provider_id.trim().is_empty()
        || request.model_id.trim().is_empty()
        || request.language.trim().is_empty()
        || request.generated_at_ms == 0
        || request.segments.is_empty()
        || request.segments.len() > 10_000
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "transcription metadata or segment count is invalid",
        ));
    }
    validate_text("caption", request.style.font_size, &request.style.color)?;
    validate_color(&request.style.background_color)?;
    if request.style.bottom_margin_px > 4_320 {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "caption bottom margin is outside supported bounds",
        ));
    }
    let mut previous_end = 0;
    for segment in &request.segments {
        if segment.text.trim().is_empty()
            || segment.text.len() > 4_096
            || segment.start_ms >= segment.end_ms
            || segment.start_ms < previous_end
            || segment
                .confidence
                .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "transcription segments must be ordered, non-overlapping, and valid",
            ));
        }
        for word in &segment.words {
            if word.word.trim().is_empty()
                || word.start_ms >= word.end_ms
                || word.start_ms < segment.start_ms
                || word.end_ms > segment.end_ms
                || word
                    .confidence
                    .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "transcription word timestamps are invalid",
                ));
            }
        }
        previous_end = segment.end_ms;
    }
    Ok(())
}

fn write_result(project: &Project, changed_ids: Vec<String>, summary: &str) -> WriteResult {
    WriteResult {
        project_id: project.id.clone(),
        revision: project.revision,
        changed_ids,
        summary: summary.into(),
        warnings: vec![],
        aliases: BTreeMap::new(),
    }
}

fn persist(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    project: &Project,
    history: &History,
) -> Result<Vec<String>, CoreError> {
    persist_transaction(storage, faults, dir, project, history, None)
}

fn load_project_data(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
) -> Result<(Project, History), CoreError> {
    recover_transaction(storage, faults, dir)?;
    let project_file = project_path(dir);
    let history_file = history_path(dir);
    let mut project: Project = read_json(storage, &project_file)?;
    let mut history: History = if storage.storage_path_exists(&history_file) {
        read_json(storage, &history_file)?
    } else {
        History::default()
    };

    let mut changed = migrate_project_documents(&mut project, &mut history)?;
    validate_project_visual_properties(&project)?;
    for snapshot in history.undo.iter().chain(&history.redo) {
        validate_project_visual_properties(snapshot)?;
    }
    validate_retained_project_references(&project, &history)?;
    changed |= migrate_project_assets(storage, &mut project, dir)?;
    for snapshot in history.undo.iter_mut().chain(&mut history.redo) {
        changed |= migrate_project_assets(storage, snapshot, dir)?;
    }
    if changed {
        let _ = persist(storage, faults, dir, &project, &history)?;
    }
    Ok((project, history))
}

fn collect_asset_garbage(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    project: &Project,
    history: &History,
) -> Vec<String> {
    let drafts = match read_all_drafts(storage, dir) {
        Ok(drafts) => drafts,
        Err(_) => return vec![ASSET_GC_FAILED.into()],
    };
    let asset_drafts = draft_asset_operations(&drafts);
    garbage_collect(storage, faults, dir, project, history, &asset_drafts)
}

fn draft_asset_operations(drafts: &[EditDraft]) -> Vec<DraftAssetOperations<'_>> {
    drafts
        .iter()
        .map(|draft| DraftAssetOperations {
            id: &draft.id,
            operations: &draft.operations,
        })
        .collect()
}

fn validate_single_draft_assets(project: &Project, draft: &EditDraft) -> Result<(), CoreError> {
    validate_draft_asset_references(
        project,
        &[DraftAssetOperations {
            id: &draft.id,
            operations: &draft.operations,
        }],
    )
}

fn finish_persistence(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    project: &Project,
    history: &History,
    mut warnings: Vec<String>,
) -> Vec<String> {
    if !warnings
        .iter()
        .any(|warning| warning == PERSISTENCE_RECOVERY_PENDING)
    {
        warnings.extend(collect_asset_garbage(
            storage, faults, dir, project, history,
        ));
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DuckingSettings, Keyframe, KeyframeProperty, KeyframeValue, TextStyle, Transform,
        timeline::HISTORY_LIMIT, validation::validate_keyframes,
    };
    use tempfile::tempdir;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum StorageFailure {
        Lock,
        Create,
        List,
        Read,
        AtomicReplace,
        DraftRemove,
        AssetClassification,
        CanonicalEscape,
        LinkedProjectEntry,
    }

    #[derive(Debug, Default)]
    struct FailingFacadeStorage {
        next: std::sync::Mutex<Option<StorageFailure>>,
        lock_calls: std::sync::atomic::AtomicUsize,
        read_calls: std::sync::atomic::AtomicUsize,
        list_calls: std::sync::atomic::AtomicUsize,
        file_probe_calls: std::sync::atomic::AtomicUsize,
    }

    impl FailingFacadeStorage {
        fn fail_next(&self, failure: StorageFailure) {
            *self.next.lock().unwrap() = Some(failure);
        }

        fn take(&self, failure: StorageFailure) -> bool {
            let mut next = self.next.lock().unwrap();
            if *next == Some(failure) {
                *next = None;
                true
            } else {
                false
            }
        }
    }

    impl Storage for FailingFacadeStorage {
        fn lock_exclusive(
            &self,
            dir: &Path,
        ) -> Result<Box<dyn crate::persistence::StorageLock>, CoreError> {
            self.lock_calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if self.take(StorageFailure::Lock) {
                Err(CoreError::io(
                    "cannot lock project",
                    std::io::Error::other("injected lock failure"),
                ))
            } else {
                FileSystemStorage.lock_exclusive(dir)
            }
        }

        fn open_read(&self, path: &Path) -> std::io::Result<Box<dyn std::io::Read>> {
            FileSystemStorage.open_read(path)
        }

        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            self.read_calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if self.take(StorageFailure::Read) {
                Err(std::io::Error::other("injected read failure"))
            } else {
                FileSystemStorage.read(path)
            }
        }

        fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
            self.list_calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if self.take(StorageFailure::List) {
                Err(std::io::Error::other("injected list failure"))
            } else {
                FileSystemStorage.list(path)
            }
        }

        fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
            if self.take(StorageFailure::Create) {
                Err(std::io::Error::other("injected create failure"))
            } else {
                FileSystemStorage.create_dir_all(path)
            }
        }

        fn copy(&self, from: &Path, to: &Path) -> std::io::Result<u64> {
            FileSystemStorage.copy(from, to)
        }

        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            FileSystemStorage.rename(from, to)
        }

        fn storage_path_is_file(&self, path: &Path) -> bool {
            self.file_probe_calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            FileSystemStorage.storage_path_is_file(path)
        }

        fn storage_path_exists(&self, path: &Path) -> bool {
            FileSystemStorage.storage_path_exists(path)
        }

        fn entry_kind(&self, path: &Path) -> std::io::Result<StorageEntryKind> {
            if self.take(StorageFailure::LinkedProjectEntry) {
                Ok(StorageEntryKind::Symlink)
            } else if path.extension().is_some()
                && path
                    .components()
                    .any(|component| component.as_os_str() == "assets")
                && self.take(StorageFailure::AssetClassification)
            {
                Err(std::io::Error::other(
                    "injected asset classification failure",
                ))
            } else {
                FileSystemStorage.entry_kind(path)
            }
        }

        fn canonicalize_storage_path(&self, path: &Path) -> std::io::Result<PathBuf> {
            if self.take(StorageFailure::CanonicalEscape) {
                Ok(PathBuf::from("outside-project-root"))
            } else {
                FileSystemStorage.canonicalize_storage_path(path)
            }
        }

        fn atomic_replace(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
            if self.take(StorageFailure::AtomicReplace) {
                Err(std::io::Error::other("injected atomic replacement failure"))
            } else {
                FileSystemStorage.atomic_replace(path, bytes)
            }
        }

        fn remove_durable(&self, path: &Path) -> std::io::Result<()> {
            if path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "drafts")
                && self.take(StorageFailure::DraftRemove)
            {
                Err(std::io::Error::other("injected draft removal failure"))
            } else {
                FileSystemStorage.remove_durable(path)
            }
        }
    }

    fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CoreError> {
        crate::persistence::read_json(&FileSystemStorage, path)
    }

    fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), CoreError> {
        crate::persistence::write_json_atomic(&FileSystemStorage, path, value)
    }

    fn core() -> (EditorCore, tempfile::TempDir) {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        std::fs::create_dir_all(&media).unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap();
        (EditorCore::new(policy), root)
    }

    fn core_with_storage() -> (EditorCore, Arc<FailingFacadeStorage>, tempfile::TempDir) {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        std::fs::create_dir_all(&media).unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap();
        let storage = Arc::new(FailingFacadeStorage::default());
        (
            EditorCore::with_storage(policy, storage.clone()),
            storage,
            root,
        )
    }

    #[test]
    fn facade_storage_injects_lock_create_list_read_and_atomic_failures() {
        for (failure, expected_message) in [
            (StorageFailure::Create, "cannot create project assets"),
            (StorageFailure::Lock, "cannot lock project"),
            (StorageFailure::AtomicReplace, "cannot publish data"),
        ] {
            let (core, storage, _) = core_with_storage();
            storage.fail_next(failure);
            let error = core
                .create_project("Injected", ProjectSettings::default())
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::InternalError);
            assert!(error.message.contains(expected_message));
        }

        let (core, storage, _) = core_with_storage();
        let created = core
            .create_project("Injected", ProjectSettings::default())
            .unwrap();
        storage.fail_next(StorageFailure::List);
        assert!(
            core.list_projects()
                .unwrap_err()
                .message
                .contains("cannot list projects")
        );
        storage.fail_next(StorageFailure::List);
        assert_eq!(
            core.get_project(&created.project_id).unwrap_err().code,
            ErrorCode::ProjectRecoveryFailed
        );
        storage.fail_next(StorageFailure::Read);
        assert!(
            core.get_project(&created.project_id)
                .unwrap_err()
                .message
                .contains("cannot read persisted JSON")
        );
    }

    #[test]
    fn facade_rejects_canonical_project_escape_before_lock_read_or_gc() {
        let (core, storage, _) = core_with_storage();
        let file_probes = storage
            .file_probe_calls
            .load(std::sync::atomic::Ordering::Relaxed);
        let locks = storage
            .lock_calls
            .load(std::sync::atomic::Ordering::Relaxed);
        let reads = storage
            .read_calls
            .load(std::sync::atomic::Ordering::Relaxed);
        let lists = storage
            .list_calls
            .load(std::sync::atomic::Ordering::Relaxed);

        storage.fail_next(StorageFailure::CanonicalEscape);
        let error = core.get_project("external-empty").unwrap_err();

        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert_eq!(
            storage
                .file_probe_calls
                .load(std::sync::atomic::Ordering::Relaxed),
            file_probes
        );
        assert_eq!(
            storage
                .lock_calls
                .load(std::sync::atomic::Ordering::Relaxed),
            locks
        );
        assert_eq!(
            storage
                .read_calls
                .load(std::sync::atomic::Ordering::Relaxed),
            reads
        );
        assert_eq!(
            storage
                .list_calls
                .load(std::sync::atomic::Ordering::Relaxed),
            lists
        );
    }

    #[test]
    fn facade_preserves_project_not_found_for_an_ordinary_missing_directory() {
        let (core, _, _) = core_with_storage();

        assert_eq!(
            core.get_project("ordinary-missing").unwrap_err().code,
            ErrorCode::ProjectNotFound
        );
    }

    #[test]
    fn facade_skips_entries_classified_as_project_links() {
        let (core, storage, _) = core_with_storage();
        core.create_project("Linked", ProjectSettings::default())
            .unwrap();

        storage.fail_next(StorageFailure::LinkedProjectEntry);

        assert!(core.list_projects().unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn linked_project_directories_are_not_listed_or_opened() {
        use std::os::unix::fs::symlink;

        let (core, root) = core();
        let created = core
            .create_project("Linked", ProjectSettings::default())
            .unwrap();
        let project_dir = root.path().join("projects").join(&created.project_id);
        let external_dir = root.path().join("external-project");
        std::fs::remove_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&external_dir).unwrap();
        symlink(&external_dir, &project_dir).unwrap();

        assert!(core.list_projects().unwrap().is_empty());
        assert_eq!(
            core.get_project(&created.project_id).unwrap_err().code,
            ErrorCode::PathNotAllowed
        );
    }

    #[test]
    fn facade_storage_preserves_draft_cleanup_and_gc_warnings() {
        let (core, storage, _) = core_with_storage();
        let created = core
            .create_project("Injected", ProjectSettings::default())
            .unwrap();
        let draft = core
            .create_draft(
                &created.project_id,
                created.revision,
                vec![create_test_track()],
                None,
            )
            .unwrap();
        storage.fail_next(StorageFailure::DraftRemove);
        let committed = core
            .commit_draft(&created.project_id, &draft.id, created.revision)
            .unwrap();
        assert!(committed.warnings.contains(&DRAFT_CLEANUP_FAILED.into()));
        assert!(
            committed
                .warnings
                .contains(&PERSISTENCE_RECOVERY_PENDING.into())
        );

        let current = core.get_project(&created.project_id).unwrap();
        let project_dir = core.project_directory(&created.project_id).unwrap();
        std::fs::write(project_dir.join("assets/orphan.bin"), b"orphan").unwrap();
        storage.fail_next(StorageFailure::AssetClassification);
        let edited = core
            .edit(
                &created.project_id,
                current.revision,
                EditOperation::CreateTrack {
                    name: "GC warning".into(),
                    track_type: TrackType::Overlay,
                    index: None,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                },
            )
            .unwrap();
        assert!(edited.warnings.contains(&ASSET_GC_FAILED.into()));
    }

    fn set_persistence_fault(core: &EditorCore, phase: PersistencePhase) {
        core.persistence_faults.inject(phase);
    }

    fn create_test_track() -> EditOperation {
        EditOperation::CreateTrack {
            name: "Recovery track".into(),
            track_type: TrackType::Overlay,
            index: None,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
        }
    }

    fn import_test_audio(
        core: &EditorCore,
        root: &tempfile::TempDir,
        project_id: &str,
        expected_revision: u64,
        name: &str,
    ) -> String {
        let source = root.path().join("media").join(name);
        std::fs::write(&source, format!("audio fixture {name}")).unwrap();
        core.import_asset(
            project_id,
            expected_revision,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1_000),
                has_audio: true,
                ..MediaProbeFacts::default()
            },
        )
        .unwrap()
        .changed_ids
        .remove(0)
    }

    fn commit_test_caption(
        core: &EditorCore,
        project_id: &str,
        expected_revision: u64,
        asset_id: &str,
    ) -> WriteResult {
        core.commit_transcription(CommitTranscriptionRequest {
            project_id: project_id.into(),
            expected_revision,
            asset_id: asset_id.into(),
            caption_track_id: None,
            provider_id: "test-provider".into(),
            model_id: "test-model".into(),
            model_version: Some("1".into()),
            language: "en".into(),
            generated_at_ms: 1,
            segments: vec![TranscriptionSegment {
                text: "Caption source".into(),
                start_ms: 0,
                end_ms: 500,
                confidence: Some(0.9),
                words: vec![],
            }],
            style: CaptionStyle::default(),
        })
        .unwrap()
    }

    fn assert_no_managed_transaction_files(dir: &Path) {
        let names = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name == TRANSACTION_FILE || name.contains(".tmp-"))
            .collect::<Vec<_>>();
        assert!(
            names.is_empty(),
            "managed transaction files remain: {names:?}"
        );
    }

    fn assert_no_content_addressed_assets(dir: &Path) {
        assert!(
            !dir.join("assets/sha256").exists(),
            "content-addressed assets were published"
        );
    }

    fn add_legacy_asset(project: &mut serde_json::Value, dir: &Path) {
        project["assets"] = serde_json::json!([{
            "id": "legacy-asset",
            "mediaType": "audio",
            "fileName": "legacy.wav",
            "projectRelativePath": "assets/legacy.wav",
            "durationMs": 1_000,
            "hasAudio": true
        }]);
        std::fs::write(dir.join("assets/legacy.wav"), b"legacy audio").unwrap();
    }

    fn speech_origin() -> GeneratedAssetOrigin {
        GeneratedAssetOrigin::SpeechSynthesis(crate::SpeechGeneration {
            request: crate::SpeechSynthesisRequest {
                text: "Local speech".into(),
                language: "en-US".into(),
                voice_id: crate::SpeechVoiceId("af_heart".into()),
                speed: 1.0,
                text_options: crate::SpeechTextOptions::default(),
            },
            provider_id: "kokoro".into(),
            model_id: "hexgrad/Kokoro-82M".into(),
            model_version: None,
            sample_rate_hz: 24_000,
            generated_at_ms: 1_777_000_000_000,
        })
    }

    #[test]
    fn generated_speech_names_are_portable_human_readable_and_bounded() {
        let mut origin = speech_origin();
        {
            let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut origin;
            generation.request.voice_id = crate::SpeechVoiceId("provider_voice_one".into());
            generation.request.text = "  Héllo   ../ AUX:\nworld***  and a deliberately very long Unicode excerpt that must be shortened  ".into();
        }
        let name = generated_display_name(&origin);
        assert!(name.starts_with("Provider Voice One - Héllo"));
        assert!(name.ends_with(".wav"));
        assert!(!name.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
        }));
        assert!(name.chars().count() <= 100);

        {
            let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut origin;
            generation.request.voice_id = crate::SpeechVoiceId("af_heart".into());
            generation.request.text = "   ".into();
        }
        assert_eq!(generated_display_name(&origin), "Heart - Speech.wav");
    }

    #[test]
    fn schema_one_projects_and_history_are_migrated_before_persisting() {
        let (core, _) = core();
        let created = core
            .create_project("legacy", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let path = project_path(&dir);
        let mut legacy: serde_json::Value = read_json(&path).unwrap();
        legacy["schemaVersion"] = serde_json::json!(1);
        legacy["assets"] = serde_json::json!([{
            "id": "legacy-asset",
            "mediaType": "audio",
            "fileName": "legacy.wav",
            "projectRelativePath": "assets/legacy.wav",
            "durationMs": 1_000,
            "hasAudio": true
        }]);
        std::fs::write(dir.join("assets/legacy.wav"), b"legacy audio").unwrap();
        write_json_atomic(&path, &legacy).unwrap();

        let migrated = core.get_project(&created.project_id).unwrap();
        assert_eq!(migrated.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(migrated.assets[0].origin.is_none());

        let overlay = migrated
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let edited = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddText {
                    track_id: overlay,
                    text: "migrate".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 48,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();

        let persisted: serde_json::Value = read_json(&path).unwrap();
        assert_eq!(persisted["schemaVersion"], PROJECT_SCHEMA_VERSION);
        assert!(persisted["assets"][0]["origin"].is_null());
        let history: serde_json::Value = read_json(&history_path(&dir)).unwrap();
        assert_eq!(history["undo"][0]["schemaVersion"], PROJECT_SCHEMA_VERSION);
        assert!(history["undo"][0]["assets"][0]["origin"].is_null());

        core.undo(&created.project_id, edited.revision).unwrap();
        let restored: serde_json::Value = read_json(&path).unwrap();
        assert_eq!(restored["schemaVersion"], PROJECT_SCHEMA_VERSION);
        assert!(restored["assets"][0]["origin"].is_null());
    }

    #[test]
    fn schema_six_common_visual_defaults_migrate_current_and_retained_history() {
        let (core, _) = core();
        let created = core
            .create_project("visual migration", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_file = project_path(&dir);
        let history_file = history_path(&dir);
        let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
        legacy["schemaVersion"] = serde_json::json!(6);
        legacy["assets"] = serde_json::json!([{
            "id": "caption-asset",
            "mediaType": "audio",
            "fileName": "caption.wav",
            "projectRelativePath": "assets/caption.wav",
            "durationMs": 1_000,
            "hasAudio": true
        }]);
        std::fs::write(dir.join("assets/caption.wav"), b"caption audio").unwrap();
        let caption_track = legacy["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|track| track["trackType"] == "caption")
            .unwrap();
        caption_track["items"] = serde_json::json!([{
            "type": "caption",
            "id": "caption",
            "text": "Migrated caption",
            "startMs": 0,
            "durationMs": 1_000,
            "style": {
                "fontSize": 48,
                "color": "#ffffff",
                "backgroundColor": "#000000",
                "bottomMarginPx": 64
            },
            "source": {
                "assetId": "caption-asset",
                "providerId": "provider",
                "modelId": "model",
                "modelVersion": null,
                "language": "en",
                "generatedAtMs": 1,
                "originalText": "Migrated caption",
                "confidence": null,
                "words": []
            },
            "hidden": true
        }]);
        let overlay_track = legacy["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|track| track["trackType"] == "overlay")
            .unwrap();
        overlay_track["items"] = serde_json::json!([{
            "type": "rectangle",
            "id": "transformed-rectangle",
            "color": "#123456",
            "width": 320,
            "height": 180,
            "startMs": 25,
            "durationMs": 900,
            "transform": {
                "positionX": 17.0,
                "positionY": -9.0,
                "scale": 1.25,
                "opacity": 0.625
            },
            "keyframes": [],
            "hidden": false
        }]);
        let mut oldest = legacy.clone();
        oldest["schemaVersion"] = serde_json::json!(1);
        oldest["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|track| track["trackType"] == "caption")
            .unwrap()["items"][0]["hidden"] = serde_json::json!(false);
        write_json_atomic(&project_file, &legacy).unwrap();
        write_json_atomic(
            &history_file,
            &serde_json::json!({ "undo": [oldest], "redo": [legacy] }),
        )
        .unwrap();

        let migrated = core.get_project(&created.project_id).unwrap();
        let caption = migrated.find_item("caption").unwrap();
        assert_eq!(caption.visual_properties().transform, Transform::default());
        assert!(caption.hidden());
        let rectangle = migrated.find_item("transformed-rectangle").unwrap();
        assert_eq!(rectangle.visual_properties().transform.position_x, 17.0);
        assert_eq!(rectangle.visual_properties().transform.position_y, -9.0);
        assert_eq!(rectangle.visual_properties().transform.scale, 1.25);
        assert_eq!(rectangle.visual_properties().transform.opacity, 0.625);
        let persisted: serde_json::Value = read_json(&project_file).unwrap();
        assert_eq!(persisted["schemaVersion"], 7);
        assert_eq!(
            persisted["tracks"][3]["items"][0]["transform"]["scale"],
            1.0
        );
        let history: serde_json::Value = read_json(&history_file).unwrap();
        for snapshot in history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap())
        {
            assert_eq!(snapshot["schemaVersion"], 7);
            assert_eq!(snapshot["tracks"][3]["items"][0]["transform"]["scale"], 1.0);
            let rectangle = snapshot["tracks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|track| track["trackType"] == "overlay")
                .unwrap()["items"][0]
                .clone();
            assert_eq!(rectangle["transform"]["positionX"], 17.0);
            assert_eq!(rectangle["transform"]["positionY"], -9.0);
            assert_eq!(rectangle["transform"]["scale"], 1.25);
            assert_eq!(rectangle["transform"]["opacity"], 0.625);
        }

        let project_bytes = std::fs::read(&project_file).unwrap();
        let history_bytes = std::fs::read(&history_file).unwrap();
        core.get_project(&created.project_id).unwrap();
        assert_eq!(std::fs::read(&project_file).unwrap(), project_bytes);
        assert_eq!(std::fs::read(&history_file).unwrap(), history_bytes);
    }

    #[test]
    fn invalid_retained_visual_properties_abort_migration_without_rewrite() {
        let (core, _) = core();
        let created = core
            .create_project("invalid retained visual", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_file = project_path(&dir);
        let history_file = history_path(&dir);
        let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
        legacy["schemaVersion"] = serde_json::json!(6);
        add_legacy_asset(&mut legacy, &dir);
        let mut invalid_snapshot = legacy.clone();
        invalid_snapshot["tracks"][0]["items"] = serde_json::json!([{
            "type": "transition",
            "id": "invalid-transition",
            "transitionType": "fade",
            "fromItemId": "source",
            "toItemId": null,
            "startMs": 0,
            "durationMs": 100,
            "transform": {
                "positionX": 0.0,
                "positionY": 0.0,
                "scale": 1.0,
                "opacity": 2.0
            },
            "hidden": false
        }]);
        write_json_atomic(&project_file, &legacy).unwrap();
        write_json_atomic(
            &history_file,
            &serde_json::json!({ "undo": [invalid_snapshot], "redo": [] }),
        )
        .unwrap();
        let project_before = std::fs::read(&project_file).unwrap();
        let history_before = std::fs::read(&history_file).unwrap();

        let error = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::ValidationFailed);
        assert_eq!(std::fs::read(&project_file).unwrap(), project_before);
        assert_eq!(std::fs::read(&history_file).unwrap(), history_before);
        assert_no_managed_transaction_files(&dir);
        assert_no_content_addressed_assets(&dir);
    }

    #[test]
    fn dangling_retained_asset_reference_aborts_before_asset_publication() {
        let (core, _) = core();
        let created = core
            .create_project("dangling retained asset", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_file = project_path(&dir);
        let history_file = history_path(&dir);
        let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
        legacy["schemaVersion"] = serde_json::json!(6);
        add_legacy_asset(&mut legacy, &dir);
        let mut invalid_snapshot = legacy.clone();
        let audio_track = invalid_snapshot["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|track| track["trackType"] == "audio")
            .unwrap();
        audio_track["items"] = serde_json::json!([{
            "type": "media",
            "id": "dangling-media",
            "assetId": "missing-asset",
            "startMs": 0,
            "durationMs": 500,
            "sourceInMs": 0,
            "audio": {
                "volume": 1.0,
                "muted": false,
                "fadeInMs": 0,
                "fadeOutMs": 0
            },
            "keyframes": [],
            "transform": Transform::default(),
            "hidden": false
        }]);
        write_json_atomic(&project_file, &legacy).unwrap();
        write_json_atomic(
            &history_file,
            &serde_json::json!({ "undo": [invalid_snapshot], "redo": [] }),
        )
        .unwrap();
        let project_before = std::fs::read(&project_file).unwrap();
        let history_before = std::fs::read(&history_file).unwrap();

        let error = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("undo history snapshot"));
        assert_eq!(std::fs::read(&project_file).unwrap(), project_before);
        assert_eq!(std::fs::read(&history_file).unwrap(), history_before);
        assert_no_managed_transaction_files(&dir);
        assert_no_content_addressed_assets(&dir);
    }

    #[test]
    fn unsupported_project_schema_versions_are_rejected_without_rewrite() {
        let (core, _) = core();
        let created = core
            .create_project("future", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let path = project_path(&dir);
        let mut future: serde_json::Value = read_json(&path).unwrap();
        future["schemaVersion"] = serde_json::json!(PROJECT_SCHEMA_VERSION + 1);
        write_json_atomic(&path, &future).unwrap();

        let error = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert!(error.message.contains("unsupported project schema version"));

        let unchanged: serde_json::Value = read_json(&path).unwrap();
        assert_eq!(unchanged["schemaVersion"], PROJECT_SCHEMA_VERSION + 1);
    }

    #[test]
    fn schema_zero_current_and_retained_state_are_rejected_without_publication() {
        for retained in [false, true] {
            let (core, _) = core();
            let created = core
                .create_project(
                    if retained {
                        "schema zero retained"
                    } else {
                        "schema zero current"
                    },
                    ProjectSettings::default(),
                )
                .unwrap();
            let dir = core.paths().project_dir(&created.project_id).unwrap();
            let project_file = project_path(&dir);
            let history_file = history_path(&dir);
            let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
            legacy["schemaVersion"] = serde_json::json!(6);
            add_legacy_asset(&mut legacy, &dir);
            let mut history = serde_json::json!({ "undo": [], "redo": [] });
            if retained {
                let mut invalid_snapshot = legacy.clone();
                invalid_snapshot["schemaVersion"] = serde_json::json!(0);
                history["undo"] = serde_json::json!([invalid_snapshot]);
            } else {
                legacy["schemaVersion"] = serde_json::json!(0);
            }
            write_json_atomic(&project_file, &legacy).unwrap();
            write_json_atomic(&history_file, &history).unwrap();
            let project_before = std::fs::read(&project_file).unwrap();
            let history_before = std::fs::read(&history_file).unwrap();

            let error = core.get_project(&created.project_id).unwrap_err();
            assert_eq!(error.code, ErrorCode::InternalError);
            assert!(
                error
                    .message
                    .contains("unsupported project schema version 0")
            );
            assert_eq!(std::fs::read(&project_file).unwrap(), project_before);
            assert_eq!(std::fs::read(&history_file).unwrap(), history_before);
            assert_no_managed_transaction_files(&dir);
            assert_no_content_addressed_assets(&dir);
        }
    }

    #[test]
    fn schema_seven_missing_visual_fields_default_without_read_rewrite() {
        let (core, _) = core();
        let created = core
            .create_project("schema seven defaults", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_file = project_path(&dir);
        let mut project: serde_json::Value = read_json(&project_file).unwrap();
        let overlay = project["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|track| track["trackType"] == "overlay")
            .unwrap();
        overlay["items"] = serde_json::json!([{
            "type": "rectangle",
            "id": "defaulted-rectangle",
            "color": "#123456",
            "width": 320,
            "height": 180,
            "startMs": 0,
            "durationMs": 500,
            "keyframes": []
        }]);
        write_json_atomic(&project_file, &project).unwrap();
        let before = std::fs::read(&project_file).unwrap();

        let state = core.get_state(&created.project_id, None).unwrap();
        let item = state.project.find_item("defaulted-rectangle").unwrap();
        assert_eq!(
            item.visual_properties(),
            &crate::VisualProperties::default()
        );
        let response = serde_json::to_value(&state.project).unwrap();
        let response_item = &response["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|track| track["trackType"] == "overlay")
            .unwrap()["items"][0];
        assert!(response_item.get("transform").is_some());
        assert_eq!(response_item["hidden"], false);
        assert_eq!(std::fs::read(&project_file).unwrap(), before);

        core.edit(&created.project_id, 0, create_test_track())
            .unwrap();
        let persisted: serde_json::Value = read_json(&project_file).unwrap();
        let persisted_item = &persisted["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|track| track["trackType"] == "overlay")
            .unwrap()["items"][0];
        assert!(persisted_item.get("transform").is_some());
        assert_eq!(persisted_item["hidden"], false);
    }

    #[test]
    fn schema_four_track_flags_migrate_in_state_and_history_without_caption_ids() {
        let (core, _) = core();
        let created = core
            .create_project("version four", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let path = project_path(&dir);
        let mut legacy: serde_json::Value = read_json(&path).unwrap();
        legacy["schemaVersion"] = serde_json::json!(4);
        legacy["tracks"]
            .as_array_mut()
            .unwrap()
            .retain(|track| track["trackType"] != serde_json::Value::String("caption".into()));
        for track in legacy["tracks"].as_array_mut().unwrap() {
            let object = track.as_object_mut().unwrap();
            object.remove("locked");
            object.remove("hidden");
            object.remove("muted");
        }
        write_json_atomic(&path, &legacy).unwrap();
        write_json_atomic(
            &history_path(&dir),
            &serde_json::json!({ "undo": [legacy], "redo": [] }),
        )
        .unwrap();

        let migrated = core.get_project(&created.project_id).unwrap();
        assert_eq!(migrated.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(migrated.tracks.iter().all(|track| {
            !track.locked
                && !track.hidden
                && !track.muted
                && track.audio_role == AudioTrackRole::Unassigned
                && track.ducking.is_none()
                && track.track_type != TrackType::Caption
        }));
        let history: History = read_json(&history_path(&dir)).unwrap();
        assert_eq!(history.undo[0].schema_version, PROJECT_SCHEMA_VERSION);
        assert!(history.undo[0].tracks.iter().all(|track| {
            !track.locked
                && !track.hidden
                && !track.muted
                && track.audio_role == AudioTrackRole::Unassigned
                && track.ducking.is_none()
                && track.track_type != TrackType::Caption
        }));
    }

    #[test]
    fn edits_conflict_and_undo_redo_are_revisioned() {
        let (core, _) = core();
        let created = core
            .create_project("test", ProjectSettings::default())
            .unwrap();
        let state = core.get_state(&created.project_id, None).unwrap();
        let overlay = state
            .project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let added = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddText {
                    track_id: overlay,
                    text: "hello".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 48,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();
        assert_eq!(added.revision, 1);
        assert_eq!(
            core.undo(&created.project_id, 0).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(core.undo(&created.project_id, 1).unwrap().revision, 2);
        assert_eq!(core.redo(&created.project_id, 2).unwrap().revision, 3);
    }

    #[test]
    fn validates_keyframe_shapes_and_order() {
        let invalid = vec![
            Keyframe {
                property: KeyframeProperty::Scale,
                time_ms: 100,
                value: KeyframeValue::Scalar { value: 1.0 },
                easing: crate::Easing::Linear,
            },
            Keyframe {
                property: KeyframeProperty::Scale,
                time_ms: 100,
                value: KeyframeValue::Scalar { value: 2.0 },
                easing: crate::Easing::Linear,
            },
        ];
        assert_eq!(
            validate_keyframes(&invalid).unwrap_err().code,
            ErrorCode::ValidationFailed
        );

        let interleaved = vec![
            Keyframe {
                property: KeyframeProperty::Volume,
                time_ms: 1_000,
                value: KeyframeValue::Scalar { value: 1.0 },
                easing: crate::Easing::Linear,
            },
            Keyframe {
                property: KeyframeProperty::Opacity,
                time_ms: 0,
                value: KeyframeValue::Scalar { value: 1.0 },
                easing: crate::Easing::Linear,
            },
            Keyframe {
                property: KeyframeProperty::Volume,
                time_ms: 500,
                value: KeyframeValue::Scalar { value: 0.5 },
                easing: crate::Easing::Linear,
            },
        ];
        assert_eq!(
            validate_keyframes(&interleaved).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
    }

    #[test]
    fn nullable_item_and_track_updates_clear_stored_values() {
        let (core, _) = core();
        let created = core
            .create_project("nullable updates", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let overlay = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let audio = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let added = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddText {
                    track_id: overlay,
                    text: "clear selectors".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 48,
                    color: "#ffffff".into(),
                    font_family: Some("Requested Family".into()),
                    font_path: Some("requested.ttf".into()),
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();
        let item_id = added.changed_ids[0].clone();
        let clear_item: EditOperation = serde_json::from_value(serde_json::json!({
            "operation": "update_item",
            "itemId": item_id,
            "fontFamily": null,
            "fontPath": null
        }))
        .unwrap();
        core.edit(&created.project_id, 1, clear_item).unwrap();

        core.edit(
            &created.project_id,
            2,
            EditOperation::UpdateTrack {
                track_id: audio.clone(),
                name: None,
                index: None,
                locked: None,
                hidden: None,
                muted: None,
                audio_role: Some(AudioTrackRole::Music),
                ducking: Some(Some(DuckingSettings {
                    enabled: true,
                    gain: 0.25,
                    attack_ms: 100,
                    release_ms: 200,
                })),
            },
        )
        .unwrap();
        let clear_ducking: EditOperation = serde_json::from_value(serde_json::json!({
            "operation": "update_track",
            "trackId": audio,
            "ducking": null
        }))
        .unwrap();
        core.edit(&created.project_id, 3, clear_ducking).unwrap();

        let project = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Text(text) = project.find_item(&item_id).unwrap() else {
            panic!("expected text item")
        };
        assert_eq!(text.font_family, None);
        assert_eq!(text.font_path, None);
        let audio = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap();
        assert_eq!(audio.audio_role, AudioTrackRole::Music);
        assert_eq!(audio.ducking, None);
    }

    #[test]
    fn media_add_move_trim_delete_and_undo_are_persistent() {
        let (core, root) = core();
        let media_path = root.path().join("media").join("frame.png");
        std::fs::write(&media_path, b"fixture").unwrap();
        let created = core
            .create_project("commands", ProjectSettings::default())
            .unwrap();
        let imported = core
            .import_asset(
                &created.project_id,
                0,
                &media_path,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    ..MediaProbeFacts::default()
                },
            )
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let video_track = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Video)
            .unwrap()
            .id
            .clone();
        let asset_id = imported.changed_ids[0].clone();
        let added = core
            .edit(
                &created.project_id,
                1,
                EditOperation::AddMedia {
                    track_id: video_track.clone(),
                    asset_id,
                    start_ms: 0,
                    duration_ms: 5_000,
                    source_in_ms: 0,
                },
            )
            .unwrap();
        let item_id = added.changed_ids[0].clone();
        core.edit(
            &created.project_id,
            2,
            EditOperation::MoveItem {
                item_id: item_id.clone(),
                track_id: video_track,
                start_ms: 500,
            },
        )
        .unwrap();
        core.edit(
            &created.project_id,
            3,
            EditOperation::TrimItem {
                item_id: item_id.clone(),
                start_ms: 750,
                duration_ms: 2_000,
                source_in_ms: Some(250),
            },
        )
        .unwrap();
        core.edit(
            &created.project_id,
            4,
            EditOperation::DeleteItem {
                item_id: item_id.clone(),
            },
        )
        .unwrap();
        assert!(
            core.get_project(&created.project_id)
                .unwrap()
                .find_item(&item_id)
                .is_none()
        );
        assert_eq!(core.undo(&created.project_id, 5).unwrap().revision, 6);
        let restored = core.get_project(&created.project_id).unwrap();
        let restored = restored.find_item(&item_id).unwrap();
        assert_eq!(restored.start_ms(), 750);
        assert_eq!(restored.duration_ms(), 2_000);
    }

    #[test]
    fn generated_audio_is_imported_and_inserted_atomically() {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        let generated = root.path().join("generated");
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&generated).unwrap();
        let speech = generated.join("speech.wav");
        std::fs::write(&speech, b"fixture").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();
        let core = EditorCore::new(policy);
        let created = core
            .create_project("speech", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let audio_track = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();

        let request = CommitGeneratedAssetRequest {
            project_id: created.project_id.clone(),
            expected_revision: 0,
            path: speech,
            track_id: audio_track,
            start_ms: 250,
            duration_ms: 1_500,
            display_name: "tts-af_heart.wav".into(),
            origin: speech_origin(),
            probe: MediaProbeFacts::default(),
        };
        let result = core.commit_generated_asset(request.clone()).unwrap();
        assert_eq!(result.revision, 1);
        assert!(!result.asset_id.is_empty());
        assert!(!result.item_id.is_empty());
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.assets.len(), 1);
        assert_eq!(project.assets[0].media_type, MediaType::Audio);
        assert_eq!(project.assets[0].file_name, "Heart - Local speech.wav");
        assert_eq!(
            project.assets[0].content_hash.as_ref().unwrap().algorithm,
            "sha256"
        );
        assert_eq!(project.assets[0].origin.as_ref(), Some(&speech_origin()));
        let item = project.find_item(&result.item_id).unwrap();
        assert_eq!(item.start_ms(), 250);
        assert_eq!(item.duration_ms(), 1_500);
        assert_eq!(
            core.commit_generated_asset(request).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(core.undo(&created.project_id, 1).unwrap().revision, 2);
        let undone = core.get_project(&created.project_id).unwrap();
        assert!(undone.assets.is_empty());
        assert!(undone.find_item(&result.item_id).is_none());
        assert_eq!(core.redo(&created.project_id, 2).unwrap().revision, 3);
        let redone = core.get_project(&created.project_id).unwrap();
        assert_eq!(redone.assets[0].origin.as_ref(), Some(&speech_origin()));
        assert!(redone.find_item(&result.item_id).is_some());
    }

    #[test]
    fn generated_audio_replacement_preserves_item_and_is_undoable() {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        let generated = root.path().join("generated");
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&generated).unwrap();
        let first = generated.join("first.wav");
        let second = generated.join("second.wav");
        std::fs::write(&first, b"first speech").unwrap();
        std::fs::write(&second, b"second speech").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();
        let core = EditorCore::new(policy);
        let created = core
            .create_project("replace speech", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let audio_track = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let inserted = core
            .commit_generated_asset(CommitGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 0,
                path: first,
                track_id: audio_track,
                start_ms: 250,
                duration_ms: 1_500,
                display_name: "ignored.wav".into(),
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap();
        let audio = AudioSettings {
            volume: 0.5,
            muted: false,
            fade_in_ms: 50,
            fade_out_ms: 75,
        };
        core.edit(
            &created.project_id,
            1,
            EditOperation::SetAudio {
                item_id: inserted.item_id.clone(),
                audio: audio.clone(),
            },
        )
        .unwrap();

        let mut replacement_origin = speech_origin();
        let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut replacement_origin;
        generation.request.text = "Replacement speech".into();
        generation.request.voice_id = crate::SpeechVoiceId("bf_emma".into());
        assert_eq!(
            core.replace_generated_asset(ReplaceGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 1,
                path: second.clone(),
                item_id: inserted.item_id.clone(),
                duration_ms: 2_250,
                origin: replacement_origin.clone(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap_err()
            .code,
            ErrorCode::RevisionConflict
        );
        let replaced = core
            .replace_generated_asset(ReplaceGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 2,
                path: second,
                item_id: inserted.item_id.clone(),
                duration_ms: 2_250,
                origin: replacement_origin,
                probe: MediaProbeFacts::default(),
            })
            .unwrap();
        assert_eq!(replaced.item_id, inserted.item_id);
        assert_eq!(replaced.replaced_asset_id, inserted.asset_id);
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.assets.len(), 1);
        assert_eq!(project.assets[0].id, replaced.asset_id);
        let TimelineItem::Media(item) = project.find_item(&inserted.item_id).unwrap() else {
            panic!("speech item must remain media")
        };
        assert_eq!(item.id, inserted.item_id);
        assert_eq!(item.start_ms, 250);
        assert_eq!(item.duration_ms, 2_250);
        assert_eq!(item.audio.volume, audio.volume);
        assert_eq!(item.audio.muted, audio.muted);
        assert_eq!(item.audio.fade_in_ms, audio.fade_in_ms);
        assert_eq!(item.audio.fade_out_ms, audio.fade_out_ms);

        core.undo(&created.project_id, 3).unwrap();
        let undone = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Media(item) = undone.find_item(&inserted.item_id).unwrap() else {
            panic!("speech item must remain media")
        };
        assert_eq!(item.asset_id, inserted.asset_id);
        assert_eq!(item.duration_ms, 1_500);
        core.redo(&created.project_id, 4).unwrap();
        let redone = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Media(item) = redone.find_item(&inserted.item_id).unwrap() else {
            panic!("speech item must remain media")
        };
        assert_eq!(item.asset_id, replaced.asset_id);
        assert_eq!(item.duration_ms, 2_250);
    }

    #[test]
    fn generated_audio_replacement_retains_a_shared_old_asset() {
        let root = tempdir().unwrap();
        let generated = root.path().join("generated");
        let media = root.path().join("media");
        std::fs::create_dir_all(&generated).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        let first = generated.join("shared.wav");
        let second = generated.join("replacement.wav");
        std::fs::write(&first, b"shared speech").unwrap();
        std::fs::write(&second, b"replacement speech").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();
        let core = EditorCore::new(policy);
        let created = core
            .create_project("shared speech", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let inserted = core
            .commit_generated_asset(CommitGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 0,
                path: first,
                track_id: track_id.clone(),
                start_ms: 0,
                duration_ms: 1_000,
                display_name: "ignored.wav".into(),
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap();
        let shared = core
            .edit(
                &created.project_id,
                1,
                EditOperation::AddMedia {
                    track_id,
                    asset_id: inserted.asset_id.clone(),
                    start_ms: 2_000,
                    duration_ms: 1_000,
                    source_in_ms: 0,
                },
            )
            .unwrap();
        let shared_item_id = shared.changed_ids[0].clone();
        let replaced = core
            .replace_generated_asset(ReplaceGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 2,
                path: second,
                item_id: inserted.item_id.clone(),
                duration_ms: 1_500,
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.assets.len(), 2);
        assert!(
            project
                .assets
                .iter()
                .any(|asset| asset.id == inserted.asset_id)
        );
        assert!(
            project
                .assets
                .iter()
                .any(|asset| asset.id == replaced.asset_id)
        );
        let TimelineItem::Media(shared) = project.find_item(&shared_item_id).unwrap() else {
            panic!("shared item must remain media")
        };
        assert_eq!(shared.asset_id, inserted.asset_id);
    }

    #[test]
    fn generated_audio_replacement_retains_a_caption_source_asset() {
        let root = tempdir().unwrap();
        let generated = root.path().join("generated");
        let media = root.path().join("media");
        std::fs::create_dir_all(&generated).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        let first = generated.join("caption-source.wav");
        let second = generated.join("caption-replacement.wav");
        std::fs::write(&first, b"caption source speech").unwrap();
        std::fs::write(&second, b"caption replacement speech").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();
        let core = EditorCore::new(policy);
        let created = core
            .create_project("captioned speech", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let inserted = core
            .commit_generated_asset(CommitGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 0,
                path: first,
                track_id,
                start_ms: 0,
                duration_ms: 1_000,
                display_name: "ignored.wav".into(),
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap();
        commit_test_caption(&core, &created.project_id, 1, &inserted.asset_id);
        let replaced = core
            .replace_generated_asset(ReplaceGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 2,
                path: second,
                item_id: inserted.item_id.clone(),
                duration_ms: 1_500,
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap();

        let project = core.get_project(&created.project_id).unwrap();
        assert!(
            project
                .assets
                .iter()
                .any(|asset| asset.id == inserted.asset_id)
        );
        assert!(
            project
                .assets
                .iter()
                .any(|asset| asset.id == replaced.asset_id)
        );
        assert!(project.tracks.iter().flat_map(|track| &track.items).any(
            |item| matches!(item, TimelineItem::Caption(item) if item.source.asset_id == inserted.asset_id)
        ));
        let TimelineItem::Media(item) = project.find_item(&inserted.item_id).unwrap() else {
            panic!("speech item must remain media")
        };
        assert_eq!(item.asset_id, replaced.asset_id);
    }

    #[test]
    fn duplicate_imports_share_storage_but_keep_logical_assets() {
        let (core, root) = core();
        let source = root.path().join("media/same.bin");
        std::fs::write(&source, b"same content").unwrap();
        let created = core
            .create_project("dedup", ProjectSettings::default())
            .unwrap();
        let facts = MediaProbeFacts {
            has_audio: true,
            ..MediaProbeFacts::default()
        };
        core.import_asset(
            &created.project_id,
            0,
            &source,
            MediaType::Audio,
            facts.clone(),
        )
        .unwrap();
        core.import_asset(&created.project_id, 1, &source, MediaType::Audio, facts)
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.assets.len(), 2);
        assert_ne!(project.assets[0].id, project.assets[1].id);
        assert_eq!(
            project.assets[0].project_relative_path,
            project.assets[1].project_relative_path
        );
        assert_eq!(
            project.assets[0].content_hash,
            project.assets[1].content_hash
        );
    }

    #[test]
    fn asset_gc_keeps_undo_roots_then_collects_after_history_eviction() {
        let (core, root) = core();
        let source = root.path().join("media/unused.bin");
        std::fs::write(&source, b"history-owned content").unwrap();
        let created = core
            .create_project("gc", ProjectSettings::default())
            .unwrap();
        let imported = core
            .import_asset(
                &created.project_id,
                0,
                &source,
                MediaType::Audio,
                MediaProbeFacts {
                    has_audio: true,
                    ..MediaProbeFacts::default()
                },
            )
            .unwrap();
        let asset_id = imported.changed_ids[0].clone();
        let project = core.get_project(&created.project_id).unwrap();
        let stored_path = core
            .project_asset_path(
                &created.project_id,
                &project.assets[0].project_relative_path,
            )
            .unwrap();

        core.delete_asset(&created.project_id, 1, &asset_id)
            .unwrap();
        assert!(
            stored_path.is_file(),
            "undo history must retain physical media"
        );
        core.undo(&created.project_id, 2).unwrap();
        assert_eq!(
            core.get_project(&created.project_id).unwrap().assets.len(),
            1
        );
        core.redo(&created.project_id, 3).unwrap();

        let overlay = core
            .get_project(&created.project_id)
            .unwrap()
            .tracks
            .into_iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id;
        for index in 0..=HISTORY_LIMIT {
            core.edit(
                &created.project_id,
                4 + index as u64,
                EditOperation::AddText {
                    track_id: overlay.clone(),
                    text: format!("history {index}"),
                    start_ms: index as u64,
                    duration_ms: 1,
                    font_size: 16,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();
        }
        assert!(
            !stored_path.exists(),
            "evicted history must release physical media"
        );
    }

    #[test]
    fn asset_gc_failure_warns_after_metadata_commit() {
        let (core, _) = core();
        let created = core
            .create_project("gc warning", ProjectSettings::default())
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let overlay = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let orphan = dir.join("assets/orphan.bin");
        std::fs::write(&orphan, b"unreachable").unwrap();

        set_persistence_fault(&core, PersistencePhase::GarbageCollection);
        let result = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddText {
                    track_id: overlay,
                    text: "committed despite cleanup warning".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 24,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();

        assert_eq!(result.revision, 1);
        assert_eq!(result.warnings, vec!["ASSET_GC_FAILED"]);
        assert!(orphan.is_file());
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.revision, 1);
        assert!(project.find_item(&result.changed_ids[0]).is_some());
        assert!(!orphan.exists());
    }

    #[test]
    fn asset_delete_rejects_in_use_assets_and_integrity_drift() {
        let (core, root) = core();
        let source = root.path().join("media/audio.bin");
        std::fs::write(&source, b"audio").unwrap();
        let created = core
            .create_project("integrity", ProjectSettings::default())
            .unwrap();
        let imported = core
            .import_asset(
                &created.project_id,
                0,
                &source,
                MediaType::Audio,
                MediaProbeFacts {
                    duration_ms: Some(1_000),
                    has_audio: true,
                    ..MediaProbeFacts::default()
                },
            )
            .unwrap();
        let asset_id = imported.changed_ids[0].clone();
        let project = core.get_project(&created.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        core.edit(
            &created.project_id,
            1,
            EditOperation::AddMedia {
                track_id,
                asset_id: asset_id.clone(),
                start_ms: 0,
                duration_ms: 500,
                source_in_ms: 0,
            },
        )
        .unwrap();
        assert_eq!(
            core.delete_asset(&created.project_id, 2, &asset_id)
                .unwrap_err()
                .code,
            ErrorCode::AssetInUse
        );
        let path = core
            .project_asset_path(
                &created.project_id,
                &project.assets[0].project_relative_path,
            )
            .unwrap();
        std::fs::write(path, b"changed").unwrap();
        assert_eq!(
            core.get_project(&created.project_id).unwrap_err().code,
            ErrorCode::AssetIntegrityFailed
        );
    }

    #[test]
    fn caption_source_blocks_deletion_and_survives_reopen_undo_redo() {
        let (core, root) = core();
        let created = core
            .create_project("caption ownership", ProjectSettings::default())
            .unwrap();
        let asset_id = import_test_audio(&core, &root, &created.project_id, 0, "caption.wav");
        let asset_path = {
            let project = core.get_project(&created.project_id).unwrap();
            core.project_asset_path(
                &created.project_id,
                &project.assets[0].project_relative_path,
            )
            .unwrap()
        };
        let caption = commit_test_caption(&core, &created.project_id, 1, &asset_id);
        let project_dir = core.paths().project_dir(&created.project_id).unwrap();
        let persisted_before_reopen = std::fs::read(project_path(&project_dir)).unwrap();

        let error = core
            .delete_asset(&created.project_id, caption.revision, &asset_id)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetInUse);
        assert!(error.message.contains("caption source"));
        let reopened = core.get_project(&created.project_id).unwrap();
        assert_eq!(
            std::fs::read(project_path(&project_dir)).unwrap(),
            persisted_before_reopen,
            "valid caption provenance must not require a schema rewrite"
        );
        assert_eq!(reopened.assets[0].id, asset_id);
        assert!(reopened.tracks.iter().flat_map(|track| &track.items).any(
            |item| matches!(item, TimelineItem::Caption(item) if item.source.asset_id == asset_id)
        ));
        assert!(asset_path.is_file());

        core.undo(&created.project_id, caption.revision).unwrap();
        assert!(asset_path.is_file());
        let redone = core
            .redo(&created.project_id, caption.revision + 1)
            .unwrap();
        assert_eq!(redone.revision, caption.revision + 2);
        let project = core.get_project(&created.project_id).unwrap();
        assert!(project.tracks.iter().flat_map(|track| &track.items).any(
            |item| matches!(item, TimelineItem::Caption(item) if item.source.asset_id == asset_id)
        ));
        assert!(asset_path.is_file());
    }

    #[test]
    fn durable_draft_reference_blocks_deletion_until_discarded() {
        let (core, root) = core();
        let created = core
            .create_project("draft ownership", ProjectSettings::default())
            .unwrap();
        let asset_id = import_test_audio(&core, &root, &created.project_id, 0, "draft.wav");
        let project = core.get_project(&created.project_id).unwrap();
        let audio_track = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let draft = core
            .create_draft(
                &created.project_id,
                1,
                vec![EditOperation::AddMedia {
                    track_id: audio_track,
                    asset_id: asset_id.clone(),
                    start_ms: 0,
                    duration_ms: 500,
                    source_in_ms: 0,
                }],
                Some("retained media".into()),
            )
            .unwrap();

        let error = core
            .delete_asset(&created.project_id, 1, &asset_id)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetInUse);
        assert!(error.message.contains("draft operation"));
        assert_eq!(
            core.get_draft(&created.project_id, &draft.id).unwrap().id,
            draft.id
        );
        assert_eq!(core.get_project(&created.project_id).unwrap().revision, 1);
        let undo_error = core.undo(&created.project_id, 1).unwrap_err();
        assert_eq!(undo_error.code, ErrorCode::AssetInUse);
        assert!(undo_error.message.contains("draft operation"));
        assert_eq!(core.get_project(&created.project_id).unwrap().revision, 1);

        core.discard_draft(&created.project_id, &draft.id).unwrap();
        assert_eq!(
            core.delete_asset(&created.project_id, 1, &asset_id)
                .unwrap()
                .revision,
            2
        );
    }

    #[test]
    fn dangling_persisted_references_fail_closed_with_deterministic_classes() {
        let (core, root) = core();

        let caption_project = core
            .create_project("dangling caption", ProjectSettings::default())
            .unwrap();
        let caption_asset = import_test_audio(
            &core,
            &root,
            &caption_project.project_id,
            0,
            "dangling-caption.wav",
        );
        commit_test_caption(&core, &caption_project.project_id, 1, &caption_asset);
        let caption_dir = core
            .paths()
            .project_dir(&caption_project.project_id)
            .unwrap();
        let mut project: Project = read_json(&project_path(&caption_dir)).unwrap();
        project.assets.clear();
        write_json_atomic(&project_path(&caption_dir), &project).unwrap();
        let error = core.get_project(&caption_project.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("caption source"));

        let media_project = core
            .create_project("dangling media", ProjectSettings::default())
            .unwrap();
        let media_asset = import_test_audio(
            &core,
            &root,
            &media_project.project_id,
            0,
            "dangling-media.wav",
        );
        let media_dir = core.paths().project_dir(&media_project.project_id).unwrap();
        let mut project = core.get_project(&media_project.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        core.edit(
            &media_project.project_id,
            1,
            EditOperation::AddMedia {
                track_id,
                asset_id: media_asset,
                start_ms: 0,
                duration_ms: 500,
                source_in_ms: 0,
            },
        )
        .unwrap();
        project = read_json(&project_path(&media_dir)).unwrap();
        project.assets.clear();
        write_json_atomic(&project_path(&media_dir), &project).unwrap();
        let error = core.get_project(&media_project.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("media item"));

        let history_project = core
            .create_project("dangling history", ProjectSettings::default())
            .unwrap();
        let history_asset = import_test_audio(
            &core,
            &root,
            &history_project.project_id,
            0,
            "dangling-history.wav",
        );
        let history_dir = core
            .paths()
            .project_dir(&history_project.project_id)
            .unwrap();
        let project = core.get_project(&history_project.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        core.edit(
            &history_project.project_id,
            1,
            EditOperation::AddMedia {
                track_id,
                asset_id: history_asset,
                start_ms: 0,
                duration_ms: 500,
                source_in_ms: 0,
            },
        )
        .unwrap();
        let mut corrupt_snapshot: Project = read_json(&project_path(&history_dir)).unwrap();
        corrupt_snapshot.assets.clear();
        let mut history: History = read_json(&history_path(&history_dir)).unwrap();
        history.undo.push(corrupt_snapshot);
        write_json_atomic(&history_path(&history_dir), &history).unwrap();
        let error = core.get_project(&history_project.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("undo history snapshot"));
        assert!(error.message.contains("media item"));

        let draft_project = core
            .create_project("dangling draft", ProjectSettings::default())
            .unwrap();
        let draft_asset = import_test_audio(
            &core,
            &root,
            &draft_project.project_id,
            0,
            "dangling-draft.wav",
        );
        let draft_dir_path = core.paths().project_dir(&draft_project.project_id).unwrap();
        let project = core.get_project(&draft_project.project_id).unwrap();
        let track_id = project
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone();
        let draft = core
            .create_draft(
                &draft_project.project_id,
                1,
                vec![EditOperation::AddMedia {
                    track_id,
                    asset_id: draft_asset,
                    start_ms: 0,
                    duration_ms: 500,
                    source_in_ms: 0,
                }],
                None,
            )
            .unwrap();
        let retained_path = core
            .project_asset_path(
                &draft_project.project_id,
                &project.assets[0].project_relative_path,
            )
            .unwrap();
        let mut project: Project = read_json(&project_path(&draft_dir_path)).unwrap();
        project.assets.clear();
        write_json_atomic(&project_path(&draft_dir_path), &project).unwrap();
        core.get_project(&draft_project.project_id).unwrap();
        assert!(
            retained_path.is_file(),
            "garbage collection must fail safe while a retained draft is dangling"
        );
        let error = core
            .get_draft(&draft_project.project_id, &draft.id)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("draft operation"));
        core.discard_draft(&draft_project.project_id, &draft.id)
            .unwrap();
    }

    #[test]
    fn generated_audio_failure_does_not_mutate_project() {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        let generated = root.path().join("generated");
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&generated).unwrap();
        let speech = generated.join("speech.wav");
        std::fs::write(&speech, b"fixture").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();
        let core = EditorCore::new(policy);
        let created = core
            .create_project("speech", ProjectSettings::default())
            .unwrap();
        let video_track = core
            .get_project(&created.project_id)
            .unwrap()
            .tracks
            .into_iter()
            .find(|track| track.track_type == TrackType::Video)
            .unwrap()
            .id;
        assert_eq!(
            core.commit_generated_asset(CommitGeneratedAssetRequest {
                project_id: created.project_id.clone(),
                expected_revision: 0,
                path: speech,
                track_id: video_track,
                start_ms: 0,
                duration_ms: 1_000,
                display_name: "speech.wav".into(),
                origin: speech_origin(),
                probe: MediaProbeFacts::default(),
            })
            .unwrap_err()
            .code,
            ErrorCode::ValidationFailed
        );
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.revision, 0);
        assert!(project.assets.is_empty());
    }

    #[test]
    fn batch_edits_roll_back_atomically_and_commit_one_revision() {
        let (core, _) = core();
        let created = core
            .create_project("batch", ProjectSettings::default())
            .unwrap();
        let before = core.get_project(&created.project_id).unwrap();
        let error = core
            .edit_batch(
                &created.project_id,
                0,
                vec![
                    EditOperation::CreateTrack {
                        name: "Temporary".into(),
                        track_type: TrackType::Overlay,
                        index: None,
                        audio_role: AudioTrackRole::Unassigned,
                        ducking: None,
                    },
                    EditOperation::DeleteTrack {
                        track_id: "missing".into(),
                    },
                ],
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::TrackNotFound);
        assert_eq!(
            serde_json::to_value(core.get_project(&created.project_id).unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );

        let result = core
            .edit_batch(
                &created.project_id,
                0,
                vec![
                    EditOperation::CreateTrack {
                        name: "Captions 2".into(),
                        track_type: TrackType::Caption,
                        index: None,
                        audio_role: AudioTrackRole::Unassigned,
                        ducking: None,
                    },
                    EditOperation::CreateTrack {
                        name: "B-roll".into(),
                        track_type: TrackType::Video,
                        index: Some(1),
                        audio_role: AudioTrackRole::Unassigned,
                        ducking: None,
                    },
                ],
            )
            .unwrap();
        assert_eq!(result.revision, 1);
        assert_eq!(result.changed_ids.len(), 2);
        assert_eq!(core.undo(&created.project_id, 1).unwrap().revision, 2);
        assert_eq!(
            serde_json::to_value(core.get_project(&created.project_id).unwrap().tracks).unwrap(),
            serde_json::to_value(before.tracks).unwrap()
        );
    }

    #[test]
    fn locked_tracks_block_item_edits_and_split_duplicate_keep_timing() {
        let (core, _) = core();
        let created = core
            .create_project("locked", ProjectSettings::default())
            .unwrap();
        let overlay = core
            .get_project(&created.project_id)
            .unwrap()
            .tracks
            .into_iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id;
        let added = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddText {
                    track_id: overlay.clone(),
                    text: "Split me".into(),
                    start_ms: 100,
                    duration_ms: 1_000,
                    font_size: 48,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    transform: Transform::default(),
                },
            )
            .unwrap();
        let item_id = added.changed_ids[0].clone();
        core.edit(
            &created.project_id,
            1,
            EditOperation::UpdateTrack {
                track_id: overlay.clone(),
                name: None,
                index: None,
                locked: Some(true),
                hidden: None,
                muted: None,
                audio_role: None,
                ducking: None,
            },
        )
        .unwrap();
        assert_eq!(
            core.edit(
                &created.project_id,
                2,
                EditOperation::SplitItem {
                    item_id: item_id.clone(),
                    split_ms: 600,
                },
            )
            .unwrap_err()
            .code,
            ErrorCode::TrackLocked
        );
        core.edit(
            &created.project_id,
            2,
            EditOperation::UpdateTrack {
                track_id: overlay,
                name: None,
                index: None,
                locked: Some(false),
                hidden: None,
                muted: None,
                audio_role: None,
                ducking: None,
            },
        )
        .unwrap();
        let split = core
            .edit(
                &created.project_id,
                3,
                EditOperation::SplitItem {
                    item_id: item_id.clone(),
                    split_ms: 600,
                },
            )
            .unwrap();
        assert_eq!(split.changed_ids[0], item_id);
        let duplicate = core
            .edit(
                &created.project_id,
                4,
                EditOperation::DuplicateItems {
                    item_ids: vec![split.changed_ids[1].clone()],
                    offset_ms: 250,
                },
            )
            .unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let items = project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .collect::<Vec<_>>();
        assert_eq!(project.find_item(&item_id).unwrap().start_ms(), 100);
        assert_eq!(
            project.find_item(&split.changed_ids[1]).unwrap().start_ms(),
            600
        );
        assert_eq!(
            project
                .find_item(&duplicate.changed_ids[0])
                .unwrap()
                .start_ms(),
            850
        );
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn splitting_an_animated_shape_rebases_keyframes_and_is_undoable() {
        let (core, _) = core();
        let created = core
            .create_project("animated split", ProjectSettings::default())
            .unwrap();
        let overlay = core
            .get_project(&created.project_id)
            .unwrap()
            .tracks
            .into_iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id;
        let added = core
            .edit(
                &created.project_id,
                0,
                EditOperation::AddRectangle {
                    track_id: overlay,
                    color: "#336699".into(),
                    width: 320,
                    height: 180,
                    start_ms: 100,
                    duration_ms: 1_000,
                    transform: Transform::default(),
                },
            )
            .unwrap();
        let item_id = added.changed_ids[0].clone();
        let keyframes = vec![
            Keyframe {
                property: KeyframeProperty::Opacity,
                time_ms: 0,
                value: KeyframeValue::Scalar { value: 0.0 },
                easing: crate::Easing::Linear,
            },
            Keyframe {
                property: KeyframeProperty::Opacity,
                time_ms: 1_000,
                value: KeyframeValue::Scalar { value: 1.0 },
                easing: crate::Easing::Linear,
            },
        ];
        core.edit(
            &created.project_id,
            1,
            EditOperation::SetKeyframes {
                item_id: item_id.clone(),
                keyframes: keyframes.clone(),
            },
        )
        .unwrap();
        let split = core
            .edit(
                &created.project_id,
                2,
                EditOperation::SplitItem {
                    item_id: item_id.clone(),
                    split_ms: 600,
                },
            )
            .unwrap();
        assert_eq!(split.revision, 3);
        let right_id = &split.changed_ids[1];
        let project = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Rectangle(left) = project.find_item(&item_id).unwrap() else {
            panic!("expected left rectangle")
        };
        let TimelineItem::Rectangle(right) = project.find_item(right_id).unwrap() else {
            panic!("expected right rectangle")
        };
        assert_eq!(left.duration_ms, 500);
        assert_eq!(right.duration_ms, 500);
        assert_eq!(left.keyframes.last().unwrap().time_ms, 500);
        assert_eq!(right.keyframes.first().unwrap().time_ms, 0);
        assert_eq!(
            left.keyframes.last().unwrap().value,
            right.keyframes[0].value
        );
        assert!(
            right
                .keyframes
                .iter()
                .all(|keyframe| keyframe.time_ms <= right.duration_ms)
        );

        core.undo(&created.project_id, 3).unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Rectangle(restored) = project.find_item(&item_id).unwrap() else {
            panic!("expected restored rectangle")
        };
        assert_eq!(restored.duration_ms, 1_000);
        assert_eq!(restored.keyframes.len(), keyframes.len());
        assert!(project.find_item(right_id).is_none());

        core.redo(&created.project_id, 4).unwrap();
        let project = core.get_project(&created.project_id).unwrap();
        let TimelineItem::Rectangle(redone_left) = project.find_item(&item_id).unwrap() else {
            panic!("expected redone left rectangle")
        };
        let TimelineItem::Rectangle(redone_right) = project.find_item(right_id).unwrap() else {
            panic!("expected redone right rectangle")
        };
        assert_eq!(redone_left.duration_ms, 500);
        assert_eq!(redone_right.start_ms, 600);
        assert_eq!(redone_right.keyframes.first().unwrap().time_ms, 0);
    }

    #[test]
    fn persistence_transaction_recovers_every_publication_phase() {
        let phases = [
            PersistencePhase::AfterJournal,
            PersistencePhase::AfterProject,
            PersistencePhase::AfterHistory,
            PersistencePhase::AfterDraftCleanup,
            PersistencePhase::AfterJournalCleanup,
        ];

        for phase in phases {
            let (core, _) = core();
            let created = core
                .create_project(&format!("phase {phase:?}"), ProjectSettings::default())
                .unwrap();
            let dir = core.paths().project_dir(&created.project_id).unwrap();
            set_persistence_fault(&core, phase);
            let committed = core
                .edit(&created.project_id, 0, create_test_track())
                .unwrap();
            assert_eq!(committed.revision, 1);
            if phase == PersistencePhase::AfterJournalCleanup {
                assert!(committed.warnings.is_empty());
            } else {
                assert_eq!(
                    committed.warnings,
                    vec![PERSISTENCE_RECOVERY_PENDING.to_owned()]
                );
                assert!(transaction_path(&dir).is_file());
            }

            let reopened = EditorCore::new(core.paths().clone());
            let project = reopened.get_project(&created.project_id).unwrap();
            let history: History = read_json(&history_path(&dir)).unwrap();
            assert_eq!(project.revision, 1, "failed after {phase:?}");
            assert_eq!(history.undo.len(), 1, "failed after {phase:?}");
            assert_eq!(history.undo[0].revision, 0, "failed after {phase:?}");
            assert_no_managed_transaction_files(&dir);
        }
    }

    #[test]
    fn schema_six_migration_recovers_every_publication_phase() {
        let phases = [
            PersistencePhase::AfterJournal,
            PersistencePhase::AfterProject,
            PersistencePhase::AfterHistory,
            PersistencePhase::AfterDraftCleanup,
            PersistencePhase::AfterJournalCleanup,
        ];

        for phase in phases {
            let (core, _) = core();
            let created = core
                .create_project(
                    &format!("migration phase {phase:?}"),
                    ProjectSettings::default(),
                )
                .unwrap();
            let dir = core.paths().project_dir(&created.project_id).unwrap();
            let project_file = project_path(&dir);
            let history_file = history_path(&dir);
            let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
            legacy["schemaVersion"] = serde_json::json!(6);
            let mut oldest = legacy.clone();
            oldest["schemaVersion"] = serde_json::json!(1);
            write_json_atomic(&project_file, &legacy).unwrap();
            write_json_atomic(
                &history_file,
                &serde_json::json!({ "undo": [oldest], "redo": [legacy] }),
            )
            .unwrap();

            set_persistence_fault(&core, phase);
            let opened = core.get_project(&created.project_id).unwrap();
            assert_eq!(opened.schema_version, PROJECT_SCHEMA_VERSION);

            let reopened = EditorCore::new(core.paths().clone());
            let recovered = reopened.get_project(&created.project_id).unwrap();
            let history: History = read_json(&history_file).unwrap();
            assert_eq!(recovered.schema_version, PROJECT_SCHEMA_VERSION);
            assert!(
                history
                    .undo
                    .iter()
                    .chain(&history.redo)
                    .all(|snapshot| snapshot.schema_version == PROJECT_SCHEMA_VERSION),
                "mixed schema versions after recovery from {phase:?}"
            );
            assert_no_managed_transaction_files(&dir);
        }
    }

    #[test]
    fn schema_six_migration_before_journal_failure_preserves_generation() {
        let (core, _) = core();
        let created = core
            .create_project("migration pre-commit", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_file = project_path(&dir);
        let history_file = history_path(&dir);
        let mut legacy: serde_json::Value = read_json(&project_file).unwrap();
        legacy["schemaVersion"] = serde_json::json!(6);
        write_json_atomic(&project_file, &legacy).unwrap();
        write_json_atomic(
            &history_file,
            &serde_json::json!({ "undo": [legacy], "redo": [] }),
        )
        .unwrap();
        let project_before = std::fs::read(&project_file).unwrap();
        let history_before = std::fs::read(&history_file).unwrap();

        set_persistence_fault(&core, PersistencePhase::BeforeJournal);
        let error = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(std::fs::read(&project_file).unwrap(), project_before);
        assert_eq!(std::fs::read(&history_file).unwrap(), history_before);
        assert_no_managed_transaction_files(&dir);
    }

    #[test]
    fn persistence_transaction_rejects_before_commit_without_mutation() {
        let (core, _) = core();
        let created = core
            .create_project("pre-commit", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_before = std::fs::read(project_path(&dir)).unwrap();
        let history_before = std::fs::read(history_path(&dir)).unwrap();

        set_persistence_fault(&core, PersistencePhase::BeforeJournal);
        let error = core
            .edit(&created.project_id, 0, create_test_track())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(std::fs::read(project_path(&dir)).unwrap(), project_before);
        assert_eq!(std::fs::read(history_path(&dir)).unwrap(), history_before);
        assert_no_managed_transaction_files(&dir);
    }

    #[test]
    fn persistence_transaction_recovery_is_repeatable_after_interruption() {
        let (core, _) = core();
        let created = core
            .create_project("repeat recovery", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();

        set_persistence_fault(&core, PersistencePhase::AfterJournal);
        core.edit(&created.project_id, 0, create_test_track())
            .unwrap();
        set_persistence_fault(&core, PersistencePhase::AfterProject);
        let interrupted = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(interrupted.code, ErrorCode::ProjectRecoveryFailed);
        assert!(transaction_path(&dir).is_file());

        let recovered = core.get_project(&created.project_id).unwrap();
        assert_eq!(recovered.revision, 1);
        let history: History = read_json(&history_path(&dir)).unwrap();
        assert_eq!(history.undo.len(), 1);
        assert_no_managed_transaction_files(&dir);
    }

    #[test]
    fn persistence_transaction_reaps_only_recognized_orphan_temps() {
        let (core, _) = core();
        let created = core
            .create_project("orphan cleanup", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let orphan_names = [
            format!("project.tmp-{}", Uuid::new_v4()),
            format!("history.tmp-{}", Uuid::new_v4()),
            format!(".project-transaction.tmp-{}", Uuid::new_v4()),
        ];
        for name in &orphan_names {
            std::fs::write(dir.join(name), b"interrupted write").unwrap();
        }
        let unrelated_file = dir.join("project.tmp-not-a-uuid");
        let unrelated_directory = dir.join(format!("history.tmp-{}", Uuid::new_v4()));
        std::fs::write(&unrelated_file, b"preserve me").unwrap();
        std::fs::create_dir(&unrelated_directory).unwrap();

        let reopened = EditorCore::new(core.paths().clone());
        assert_eq!(
            reopened.get_project(&created.project_id).unwrap().revision,
            0
        );
        for name in orphan_names {
            assert!(!dir.join(name).exists());
        }
        assert!(unrelated_file.is_file());
        assert!(unrelated_directory.is_dir());
    }

    #[test]
    fn persistence_transaction_rejects_irrecoverable_journals_without_rewrite() {
        let (core, _) = core();
        let created = core
            .create_project("invalid recovery", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_before = std::fs::read(project_path(&dir)).unwrap();
        let history_before = std::fs::read(history_path(&dir)).unwrap();

        std::fs::write(transaction_path(&dir), b"not-json").unwrap();
        let malformed = core.get_project(&created.project_id).unwrap_err();
        assert_eq!(malformed.code, ErrorCode::ProjectRecoveryFailed);
        assert!(transaction_path(&dir).is_file());
        assert_eq!(std::fs::read(project_path(&dir)).unwrap(), project_before);
        assert_eq!(std::fs::read(history_path(&dir)).unwrap(), history_before);

        let project: Project = read_json(&project_path(&dir)).unwrap();
        let history: History = read_json(&history_path(&dir)).unwrap();
        let invalid_transactions = [
            ProjectTransaction {
                version: TRANSACTION_VERSION + 1,
                project: project.clone(),
                history: history.clone(),
                committed_draft_id: None,
            },
            ProjectTransaction {
                version: TRANSACTION_VERSION,
                project: Project {
                    id: "different-project".into(),
                    ..project.clone()
                },
                history,
                committed_draft_id: None,
            },
        ];
        for transaction in invalid_transactions {
            write_json_atomic(&transaction_path(&dir), &transaction).unwrap();
            let error = core.get_project(&created.project_id).unwrap_err();
            assert_eq!(error.code, ErrorCode::ProjectRecoveryFailed);
            assert!(transaction_path(&dir).is_file());
            assert_eq!(std::fs::read(project_path(&dir)).unwrap(), project_before);
            assert_eq!(std::fs::read(history_path(&dir)).unwrap(), history_before);
        }
    }

    #[test]
    fn draft_commit_recovery_warns_and_never_applies_twice() {
        let (core, _) = core();
        let created = core
            .create_project("draft recovery", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let draft = core
            .create_draft(&created.project_id, 0, vec![create_test_track()], None)
            .unwrap();

        set_persistence_fault(&core, PersistencePhase::AfterHistory);
        let committed = core
            .commit_draft(&created.project_id, &draft.id, 0)
            .unwrap();
        assert_eq!(committed.revision, 1);
        assert_eq!(
            committed.warnings,
            vec![
                PERSISTENCE_RECOVERY_PENDING.to_owned(),
                DRAFT_CLEANUP_FAILED.to_owned()
            ]
        );
        assert!(draft_path(&dir, &draft.id).unwrap().is_file());
        assert!(transaction_path(&dir).is_file());

        let retry = core
            .commit_draft(&created.project_id, &draft.id, 1)
            .unwrap_err();
        assert_eq!(retry.code, ErrorCode::DraftNotFound);
        let project = core.get_project(&created.project_id).unwrap();
        assert_eq!(project.revision, 1);
        assert_eq!(
            project
                .tracks
                .iter()
                .filter(|track| track.name == "Recovery track")
                .count(),
            1
        );
        assert_no_managed_transaction_files(&dir);
    }

    #[test]
    fn draft_commit_recovery_preserves_draft_before_commit_point() {
        let (core, _) = core();
        let created = core
            .create_project("draft pre-commit", ProjectSettings::default())
            .unwrap();
        let draft = core
            .create_draft(&created.project_id, 0, vec![create_test_track()], None)
            .unwrap();

        set_persistence_fault(&core, PersistencePhase::BeforeJournal);
        assert!(
            core.commit_draft(&created.project_id, &draft.id, 0)
                .is_err()
        );
        assert!(core.get_draft(&created.project_id, &draft.id).is_ok());
        let committed = core
            .commit_draft(&created.project_id, &draft.id, 0)
            .unwrap();
        assert_eq!(committed.revision, 1);

        let discarded = core
            .create_draft(&created.project_id, 1, vec![create_test_track()], None)
            .unwrap();
        core.discard_draft(&created.project_id, &discarded.id)
            .unwrap();
        assert_eq!(core.get_project(&created.project_id).unwrap().revision, 1);
        assert_eq!(
            core.get_draft(&created.project_id, &discarded.id)
                .unwrap_err()
                .code,
            ErrorCode::DraftNotFound
        );
    }

    #[test]
    fn drafts_are_durable_conflict_safe_and_commit_atomically() {
        let (core, _) = core();
        let created = core
            .create_project("draft", ProjectSettings::default())
            .unwrap();
        let draft = core
            .create_draft(
                &created.project_id,
                0,
                vec![EditOperation::CreateTrack {
                    name: "Agent captions".into(),
                    track_type: TrackType::Caption,
                    index: None,
                    audio_role: AudioTrackRole::Unassigned,
                    ducking: None,
                }],
                Some("caption plan".into()),
            )
            .unwrap();
        assert_eq!(core.get_project(&created.project_id).unwrap().revision, 0);
        assert_eq!(
            core.get_draft(&created.project_id, &draft.id)
                .unwrap()
                .label
                .as_deref(),
            Some("caption plan")
        );

        core.edit(
            &created.project_id,
            0,
            EditOperation::CreateTrack {
                name: "Concurrent".into(),
                track_type: TrackType::Overlay,
                index: None,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
            },
        )
        .unwrap();
        assert_eq!(
            core.commit_draft(&created.project_id, &draft.id, 1)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert!(core.get_draft(&created.project_id, &draft.id).is_ok());
        core.rebase_draft(&created.project_id, &draft.id, 1)
            .unwrap();
        assert_eq!(
            core.commit_draft(&created.project_id, &draft.id, 1)
                .unwrap()
                .revision,
            2
        );
        assert_eq!(
            core.get_draft(&created.project_id, &draft.id)
                .unwrap_err()
                .code,
            ErrorCode::DraftNotFound
        );
    }

    #[test]
    fn batch_aliases_resolve_shapes_and_roll_back_forward_references() {
        let (core, _) = core();
        let created = core
            .create_project("aliases", ProjectSettings::default())
            .unwrap();
        let overlay = core
            .get_project(&created.project_id)
            .unwrap()
            .tracks
            .into_iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap()
            .id;
        let operations: Vec<BatchEditOperation> = serde_json::from_value(serde_json::json!([
            {
                "operation": "add_rectangle", "trackId": overlay, "color": "#112233",
                "width": 320, "height": 180, "startMs": 100, "durationMs": 1000,
                "transform": { "positionX": 10.0, "positionY": 20.0, "scale": 1.0, "opacity": 1.0 },
                "resultAlias": "panel"
            },
            {
                "operation": "set_keyframes", "itemId": "@panel",
                "keyframes": [{ "property": "opacity", "timeMs": 0, "value": { "type": "scalar", "value": 0.5 }, "easing": "linear" }]
            }
        ])).unwrap();
        let result = core.edit_batch(&created.project_id, 0, operations).unwrap();
        let panel = result.aliases.get("panel").unwrap();
        assert_eq!(result.revision, 1);
        assert!(matches!(
            core.get_project(&created.project_id)
                .unwrap()
                .find_item(panel),
            Some(TimelineItem::Rectangle(_))
        ));
        core.undo(&created.project_id, 1).unwrap();
        assert!(
            core.get_project(&created.project_id)
                .unwrap()
                .find_item(panel)
                .is_none()
        );

        let before = core.get_project(&created.project_id).unwrap();
        let invalid: Vec<BatchEditOperation> = serde_json::from_value(serde_json::json!([
            { "operation": "delete_item", "itemId": "@later" },
            {
                "operation": "add_solid_color", "trackId": overlay, "color": "#000000",
                "startMs": 0, "durationMs": 100, "transform": { "positionX": 0.0, "positionY": 0.0, "scale": 1.0, "opacity": 1.0 },
                "resultAlias": "later"
            }
        ])).unwrap();
        assert_eq!(
            core.edit_batch(&created.project_id, before.revision, invalid)
                .unwrap_err()
                .code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(
            core.get_project(&created.project_id).unwrap().revision,
            before.revision
        );
    }
}
