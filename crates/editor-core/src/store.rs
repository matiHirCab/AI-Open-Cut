use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    Asset, AudioSettings, AudioTrackRole, BatchEditOperation, CaptionItem, CaptionSource,
    CaptionStyle, CaptionWord, ContentHash, CoreError, DuckingSettings, EditOperation, ErrorCode,
    GeneratedAssetOrigin, History, Keyframe, KeyframeProperty, KeyframeValue, MediaItem,
    MediaProbeFacts, MediaType, PROJECT_SCHEMA_VERSION, PathPolicy, Project, ProjectSettings,
    ProjectState, RectangleItem, SolidColorItem, TextItem, TextStyle, TimelineItem, Track,
    TrackType, Transform, TransitionItem, animation::split_keyframes,
};

const HISTORY_LIMIT: usize = 100;
const DRAFT_VERSION: u32 = 1;
const DRAFT_LIMIT: usize = 100;
const EDIT_LIMIT: usize = 100;
const TRANSACTION_VERSION: u32 = 1;
const TRANSACTION_FILE: &str = ".project-transaction.json";
const PERSISTENCE_RECOVERY_PENDING: &str = "PERSISTENCE_RECOVERY_PENDING";
const DRAFT_CLEANUP_FAILED: &str = "DRAFT_CLEANUP_FAILED";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectTransaction {
    version: u32,
    project: Project,
    history: History,
    committed_draft_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PersistencePhase {
    BeforeJournal,
    AfterJournal,
    AfterProject,
    AfterHistory,
    AfterDraftCleanup,
    AfterJournalCleanup,
}

#[cfg(test)]
thread_local! {
    static PERSISTENCE_FAULT: std::cell::Cell<Option<PersistencePhase>> = const {
        std::cell::Cell::new(None)
    };
}

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
pub struct EditDraft {
    pub version: u32,
    pub id: String,
    pub project_id: String,
    pub base_revision: u64,
    pub label: Option<String>,
    pub operations: Vec<EditOperation>,
    pub created_at_ms: u64,
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
}

impl EditorCore {
    pub fn new(paths: PathPolicy) -> Self {
        Self { paths }
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
        std::fs::create_dir_all(project_dir.join("assets"))
            .map_err(|error| CoreError::io("cannot create project assets", error))?;
        std::fs::create_dir_all(project_dir.join("previews"))
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
        let _lock = ProjectLock::exclusive(&project_dir)?;
        let warnings = persist(&project_dir, &project, &History::default())?;
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (project, history) = load_project_data(&dir)?;
        let _ = garbage_collect(&dir, &project, &history);
        Ok(project)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectSummary>, CoreError> {
        let mut summaries = Vec::new();
        let entries = std::fs::read_dir(self.paths.projects_root())
            .map_err(|error| CoreError::io("cannot list projects", error))?;
        for entry in entries {
            let entry = entry.map_err(|error| CoreError::io("cannot read project entry", error))?;
            if !entry
                .file_type()
                .map_err(|error| CoreError::io("cannot inspect project entry", error))?
                .is_dir()
            {
                continue;
            }
            let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !entry.path().join("project.json").is_file()
                && !entry.path().join(TRANSACTION_FILE).is_file()
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        let asset_id = Uuid::new_v4().to_string();
        let stored = store_content_addressed(&dir, &source)?;
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
        let warnings = persist(&dir, &project, &history)?;
        let mut result = write_result(&project, vec![asset_id], "Imported asset");
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
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
        let stored = store_content_addressed(&dir, &source)?;
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
                transform: Transform::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
                hidden: false,
            }));
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(&dir, &project, &history)?;
        let warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        if project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .any(|item| matches!(item, TimelineItem::Media(media) if media.asset_id == asset_id))
        {
            return Err(CoreError::new(
                ErrorCode::AssetInUse,
                "asset is used by the timeline",
            ));
        }
        let index = project
            .assets
            .iter()
            .position(|asset| asset.id == asset_id)
            .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
        let previous = project.clone();
        project.assets.remove(index);
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(&dir, &project, &history)?;
        let mut result = write_result(&project, vec![asset_id.to_owned()], "Deleted asset");
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
        check_revision(&project, request.expected_revision)?;
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
        let stored = store_content_addressed(&dir, &source)?;
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
        if !project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .any(|item| matches!(item, TimelineItem::Media(media) if media.asset_id == replaced_asset_id))
        {
            project.assets.retain(|asset| asset.id != replaced_asset_id);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(&dir, &project, &history)?;
        let warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        let previous = project.clone();
        let (changed_ids, summary) = apply_operation(&mut project, operation)?;
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(&dir, &project, &history)?;
        let mut result = write_result(&project, changed_ids, summary);
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
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
        let warnings = persist(&dir, &project, &history)?;
        let mut result = write_result(&project, changed_ids, "Applied timeline edit batch");
        result.aliases = aliases;
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (project, _) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        validate_operations_against(&project, &operations)?;
        let drafts = draft_dir(&dir);
        std::fs::create_dir_all(&drafts)
            .map_err(|error| CoreError::io("cannot create draft directory", error))?;
        if count_drafts(&drafts)? >= DRAFT_LIMIT {
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
        write_json_atomic(&draft_path(&dir, &draft.id)?, &draft)?;
        Ok(draft)
    }

    pub fn get_draft(&self, project_id: &str, draft_id: &str) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = ProjectLock::exclusive(&dir)?;
        let _ = load_project_data(&dir)?;
        read_draft(&dir, draft_id)
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (project, _) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        let mut draft = read_draft(&dir, draft_id)?;
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
        write_json_atomic(&draft_path(&dir, draft_id)?, &draft)?;
        Ok(draft)
    }

    pub fn rebase_draft(
        &self,
        project_id: &str,
        draft_id: &str,
        expected_revision: u64,
    ) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = ProjectLock::exclusive(&dir)?;
        let (project, _) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        let mut draft = read_draft(&dir, draft_id)?;
        validate_operations_against(&project, &draft.operations)?;
        draft.base_revision = expected_revision;
        draft.updated_at_ms = now_ms()?;
        write_json_atomic(&draft_path(&dir, draft_id)?, &draft)?;
        Ok(draft)
    }

    pub fn get_draft_state(
        &self,
        project_id: &str,
        draft_id: &str,
    ) -> Result<ProjectState, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, _) = load_project_data(&dir)?;
        let draft = read_draft(&dir, draft_id)?;
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
        check_revision(&project, expected_revision)?;
        let draft = read_draft(&dir, draft_id)?;
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
        let warnings = persist_transaction(&dir, &project, &history, Some(draft_id))?;
        let mut result = write_result(&project, changed_ids, "Committed edit draft");
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
        Ok(result)
    }

    pub fn discard_draft(&self, project_id: &str, draft_id: &str) -> Result<EditDraft, CoreError> {
        let dir = self.existing_project_dir(project_id)?;
        let _lock = ProjectLock::exclusive(&dir)?;
        let _ = load_project_data(&dir)?;
        let draft = read_draft(&dir, draft_id)?;
        std::fs::remove_file(draft_path(&dir, draft_id)?)
            .map_err(|error| CoreError::io("cannot discard draft", error))?;
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (mut project, mut history) = load_project_data(&dir)?;
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
                    hidden: false,
                }));
            changed_ids.push(id);
        }
        push_undo(&mut history, &previous);
        bump_revision(&mut project)?;
        let warnings = persist(&dir, &project, &history)?;
        let mut result = write_result(&project, changed_ids, "Committed transcription captions");
        result.warnings = finish_persistence(&dir, &project, &history, warnings);
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
        let resolved = candidate
            .canonicalize()
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
        let _lock = ProjectLock::exclusive(&dir)?;
        let (project, mut history) = load_project_data(&dir)?;
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
        if undo {
            history.redo.push(project);
        } else {
            history.undo.push(project);
        }
        let warnings = persist(&dir, &restored, &history)?;
        let mut result = write_result(
            &restored,
            vec![],
            if undo {
                "Undid last edit"
            } else {
                "Redid last edit"
            },
        );
        result.warnings = finish_persistence(&dir, &restored, &history, warnings);
        Ok(result)
    }

    fn existing_project_dir(&self, project_id: &str) -> Result<PathBuf, CoreError> {
        let dir = self.paths.project_dir(project_id)?;
        if !project_path(&dir).is_file() && !transaction_path(&dir).is_file() {
            return Err(CoreError::new(
                ErrorCode::ProjectNotFound,
                "project was not found",
            ));
        }
        dir.canonicalize()
            .map_err(|error| CoreError::io("cannot resolve project directory", error))
    }
}

fn validate_alias(alias: &str) -> Result<(), CoreError> {
    if alias.is_empty()
        || alias.len() > 64
        || !alias
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        || !alias
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "resultAlias has an invalid format",
        ));
    }
    Ok(())
}

fn is_single_id_creator(edit: &EditOperation) -> bool {
    matches!(
        edit,
        EditOperation::AddMedia { .. }
            | EditOperation::AddText { .. }
            | EditOperation::AddSolidColor { .. }
            | EditOperation::AddRectangle { .. }
            | EditOperation::AddTransition { .. }
            | EditOperation::CreateTrack { .. }
    )
}

fn resolve_alias(value: &mut String, aliases: &BTreeMap<String, String>) -> Result<(), CoreError> {
    let Some(alias) = value.strip_prefix('@') else {
        return Ok(());
    };
    *value = aliases.get(alias).cloned().ok_or_else(|| {
        CoreError::new(
            ErrorCode::ValidationFailed,
            format!("batch alias @{alias} is missing or referenced before creation"),
        )
    })?;
    Ok(())
}

fn resolve_operation_aliases(
    edit: &mut EditOperation,
    aliases: &BTreeMap<String, String>,
) -> Result<(), CoreError> {
    match edit {
        EditOperation::AddMedia { track_id, .. }
        | EditOperation::AddText { track_id, .. }
        | EditOperation::AddSolidColor { track_id, .. }
        | EditOperation::AddRectangle { track_id, .. } => resolve_alias(track_id, aliases)?,
        EditOperation::UpdateItem { item_id, .. }
        | EditOperation::TrimItem { item_id, .. }
        | EditOperation::DeleteItem { item_id }
        | EditOperation::SetKeyframes { item_id, .. }
        | EditOperation::SetAudio { item_id, .. }
        | EditOperation::SplitItem { item_id, .. }
        | EditOperation::SetItemVisibility { item_id, .. } => resolve_alias(item_id, aliases)?,
        EditOperation::MoveItem {
            item_id, track_id, ..
        } => {
            resolve_alias(item_id, aliases)?;
            resolve_alias(track_id, aliases)?;
        }
        EditOperation::AddTransition {
            track_id,
            from_item_id,
            to_item_id,
            ..
        } => {
            resolve_alias(track_id, aliases)?;
            resolve_alias(from_item_id, aliases)?;
            if let Some(value) = to_item_id {
                resolve_alias(value, aliases)?;
            }
        }
        EditOperation::DuplicateItems { item_ids, .. } => {
            for value in item_ids {
                resolve_alias(value, aliases)?;
            }
        }
        EditOperation::UpdateTrack { track_id, .. } | EditOperation::DeleteTrack { track_id } => {
            resolve_alias(track_id, aliases)?
        }
        EditOperation::CreateTrack { .. } => {}
    }
    Ok(())
}

fn apply_operation(
    project: &mut Project,
    operation: EditOperation,
) -> Result<(Vec<String>, &'static str), CoreError> {
    match operation {
        EditOperation::AddMedia {
            track_id,
            asset_id,
            start_ms,
            duration_ms,
            source_in_ms,
        } => {
            validate_duration(duration_ms)?;
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == asset_id)
                .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
            let asset_media_type = asset.media_type;
            if let Some(asset_duration) = asset.duration_ms
                && asset_media_type != MediaType::Image
                && source_in_ms.saturating_add(duration_ms) > asset_duration
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "source range exceeds the asset duration",
                ));
            }
            let track = editable_track_mut(project, &track_id)?;
            validate_track_media(track.track_type, asset_media_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Media(MediaItem {
                id: id.clone(),
                asset_id,
                start_ms,
                duration_ms,
                source_in_ms,
                transform: Transform::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
                hidden: false,
            }));
            Ok((vec![id], "Added media item"))
        }
        EditOperation::AddText {
            track_id,
            text,
            start_ms,
            duration_ms,
            font_size,
            color,
            font_family,
            font_path,
            style,
            transform,
        } => {
            validate_duration(duration_ms)?;
            validate_transform(&transform)?;
            validate_text(&text, font_size, &color)?;
            validate_text_style(&style)?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "text items require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Text(TextItem {
                id: id.clone(),
                text,
                start_ms,
                duration_ms,
                font_size,
                color,
                font_family,
                font_path,
                style,
                transform,
                keyframes: vec![],
                hidden: false,
            }));
            Ok((vec![id], "Added text item"))
        }
        EditOperation::AddSolidColor {
            track_id,
            color,
            start_ms,
            duration_ms,
            transform,
        } => {
            validate_duration(duration_ms)?;
            validate_color(&color)?;
            validate_transform(&transform)?;
            let track = editable_track_mut(project, &track_id)?;
            validate_visual_track(track.track_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::SolidColor(SolidColorItem {
                id: id.clone(),
                color,
                start_ms,
                duration_ms,
                transform,
                keyframes: vec![],
                hidden: false,
            }));
            Ok((vec![id], "Added solid color item"))
        }
        EditOperation::AddRectangle {
            track_id,
            color,
            width,
            height,
            start_ms,
            duration_ms,
            transform,
        } => {
            validate_duration(duration_ms)?;
            validate_color(&color)?;
            validate_dimensions(width, height)?;
            validate_transform(&transform)?;
            let track = editable_track_mut(project, &track_id)?;
            validate_visual_track(track.track_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Rectangle(RectangleItem {
                id: id.clone(),
                color,
                width,
                height,
                start_ms,
                duration_ms,
                transform,
                keyframes: vec![],
                hidden: false,
            }));
            Ok((vec![id], "Added rectangle item"))
        }
        EditOperation::UpdateItem {
            item_id,
            transform,
            text,
            color,
            width,
            height,
            font_family,
            font_path,
            style,
        } => {
            let item = find_editable_item_mut(project, &item_id)?;
            if let Some(transform) = transform {
                validate_transform(&transform)?;
                match item {
                    TimelineItem::Media(media) => media.transform = transform,
                    TimelineItem::Text(text_item) => text_item.transform = transform,
                    TimelineItem::SolidColor(item) => item.transform = transform,
                    TimelineItem::Rectangle(item) => item.transform = transform,
                    TimelineItem::Caption(_) => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "captions do not have transforms",
                        ));
                    }
                    TimelineItem::Transition(_) => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "transitions do not have transforms",
                        ));
                    }
                }
            }
            if let Some(text) = text {
                match item {
                    TimelineItem::Text(text_item) => {
                        validate_text(&text, text_item.font_size, &text_item.color)?;
                        text_item.text = text;
                    }
                    TimelineItem::Caption(caption) => {
                        validate_text(&text, caption.style.font_size, &caption.style.color)?;
                        caption.text = text;
                    }
                    _ => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "only text items accept text updates",
                        ));
                    }
                }
            }
            if let Some(color) = color {
                validate_color(&color)?;
                match item {
                    TimelineItem::Text(text) => text.color = color,
                    TimelineItem::SolidColor(shape) => shape.color = color,
                    TimelineItem::Rectangle(shape) => shape.color = color,
                    _ => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "item does not accept color updates",
                        ));
                    }
                }
            }
            if width.is_some() || height.is_some() {
                let TimelineItem::Rectangle(rectangle) = item else {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "dimensions require a rectangle item",
                    ));
                };
                let width = width.unwrap_or(rectangle.width);
                let height = height.unwrap_or(rectangle.height);
                validate_dimensions(width, height)?;
                rectangle.width = width;
                rectangle.height = height;
            }
            if font_family.is_some() || font_path.is_some() || style.is_some() {
                let TimelineItem::Text(text) = item else {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "font and style updates require a text item",
                    ));
                };
                if let Some(value) = font_family {
                    text.font_family = value;
                }
                if let Some(value) = font_path {
                    text.font_path = value;
                }
                if let Some(value) = style {
                    validate_text_style(&value)?;
                    text.style = value;
                }
            }
            Ok((vec![item_id], "Updated timeline item"))
        }
        EditOperation::MoveItem {
            item_id,
            track_id,
            start_ms,
        } => {
            ensure_item_track_unlocked(project, &item_id)?;
            let mut item = remove_item(project, &item_id)?;
            set_item_start(&mut item, start_ms);
            let track = editable_track_mut(project, &track_id)?;
            validate_item_track(&item, track.track_type)?;
            track.items.push(item);
            Ok((vec![item_id], "Moved timeline item"))
        }
        EditOperation::TrimItem {
            item_id,
            start_ms,
            duration_ms,
            source_in_ms,
        } => {
            validate_duration(duration_ms)?;
            let item = find_editable_item_mut(project, &item_id)?;
            match item {
                TimelineItem::Media(media) => {
                    media.start_ms = start_ms;
                    media.duration_ms = duration_ms;
                    if let Some(source_in_ms) = source_in_ms {
                        media.source_in_ms = source_in_ms;
                    }
                }
                TimelineItem::Text(text) => {
                    text.start_ms = start_ms;
                    text.duration_ms = duration_ms;
                }
                TimelineItem::SolidColor(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Rectangle(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Caption(caption) => {
                    caption.start_ms = start_ms;
                    caption.duration_ms = duration_ms;
                }
                TimelineItem::Transition(transition) => {
                    transition.start_ms = start_ms;
                    transition.duration_ms = duration_ms;
                }
            }
            Ok((vec![item_id], "Trimmed timeline item"))
        }
        EditOperation::DeleteItem { item_id } => {
            ensure_item_track_unlocked(project, &item_id)?;
            remove_item(project, &item_id)?;
            for track in &mut project.tracks {
                track.items.retain(|item| match item {
                    TimelineItem::Transition(transition) => {
                        transition.from_item_id != item_id
                            && transition.to_item_id.as_deref() != Some(&item_id)
                    }
                    _ => true,
                });
            }
            Ok((vec![item_id], "Deleted timeline item"))
        }
        EditOperation::SetKeyframes { item_id, keyframes } => {
            validate_keyframes(&keyframes)?;
            let item = find_editable_item_mut(project, &item_id)?;
            if keyframes
                .iter()
                .any(|keyframe| keyframe.property == KeyframeProperty::Volume)
                && !matches!(item, TimelineItem::Media(_))
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "volume keyframes require a media item",
                ));
            }
            let destination = item.keyframes_mut().ok_or_else(|| {
                CoreError::new(
                    ErrorCode::ValidationFailed,
                    "transitions do not accept transform keyframes",
                )
            })?;
            *destination = keyframes;
            Ok((vec![item_id], "Set item keyframes"))
        }
        EditOperation::AddTransition {
            track_id,
            transition_type,
            from_item_id,
            to_item_id,
            start_ms,
            duration_ms,
        } => {
            validate_duration(duration_ms)?;
            if project.find_item(&from_item_id).is_none()
                || to_item_id
                    .as_ref()
                    .is_some_and(|id| project.find_item(id).is_none())
            {
                return Err(CoreError::new(
                    ErrorCode::ItemNotFound,
                    "transition endpoint was not found",
                ));
            }
            let track = editable_track_mut(project, &track_id)?;
            if matches!(track.track_type, TrackType::Audio | TrackType::Caption) {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "visual transitions cannot be added to audio tracks",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Transition(TransitionItem {
                id: id.clone(),
                transition_type,
                from_item_id,
                to_item_id,
                start_ms,
                duration_ms,
                hidden: false,
            }));
            Ok((vec![id], "Added transition"))
        }
        EditOperation::SetAudio { item_id, audio } => {
            validate_audio(&audio)?;
            let item = find_editable_item_mut(project, &item_id)?;
            match item {
                TimelineItem::Media(media) => media.audio = audio,
                _ => {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "audio settings require a media item",
                    ));
                }
            }
            Ok((vec![item_id], "Updated item audio"))
        }
        EditOperation::SplitItem { item_id, split_ms } => {
            let (track_index, item_index) = find_item_location(project, &item_id)?;
            if project.tracks[track_index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            let item = &mut project.tracks[track_index].items[item_index];
            if split_ms <= item.start_ms() || split_ms >= item.end_ms() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "split time must be strictly inside the item",
                ));
            }
            let right_id = Uuid::new_v4().to_string();
            let right_duration = item.end_ms() - split_ms;
            let left_duration = split_ms - item.start_ms();
            let right = match item {
                TimelineItem::Media(media) => {
                    let mut right = media.clone();
                    let (left_keyframes, right_keyframes) =
                        split_keyframes(&media.keyframes, left_duration, media.duration_ms);
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    right.source_in_ms = right.source_in_ms.saturating_add(left_duration);
                    right.keyframes = right_keyframes;
                    media.duration_ms = left_duration;
                    media.keyframes = left_keyframes;
                    TimelineItem::Media(right)
                }
                TimelineItem::Text(text) => {
                    let mut right = text.clone();
                    let (left_keyframes, right_keyframes) =
                        split_keyframes(&text.keyframes, left_duration, text.duration_ms);
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    right.keyframes = right_keyframes;
                    text.duration_ms = left_duration;
                    text.keyframes = left_keyframes;
                    TimelineItem::Text(right)
                }
                TimelineItem::SolidColor(shape) => {
                    let mut right = shape.clone();
                    let (left_keyframes, right_keyframes) =
                        split_keyframes(&shape.keyframes, left_duration, shape.duration_ms);
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    right.keyframes = right_keyframes;
                    shape.duration_ms = left_duration;
                    shape.keyframes = left_keyframes;
                    TimelineItem::SolidColor(right)
                }
                TimelineItem::Rectangle(shape) => {
                    let mut right = shape.clone();
                    let (left_keyframes, right_keyframes) =
                        split_keyframes(&shape.keyframes, left_duration, shape.duration_ms);
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    right.keyframes = right_keyframes;
                    shape.duration_ms = left_duration;
                    shape.keyframes = left_keyframes;
                    TimelineItem::Rectangle(right)
                }
                TimelineItem::Caption(caption) => {
                    let mut right = caption.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    caption.duration_ms = left_duration;
                    TimelineItem::Caption(right)
                }
                TimelineItem::Transition(_) => {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "transitions cannot be split",
                    ));
                }
            };
            project.tracks[track_index]
                .items
                .insert(item_index + 1, right);
            Ok((vec![item_id, right_id], "Split timeline item"))
        }
        EditOperation::DuplicateItems {
            item_ids,
            offset_ms,
        } => {
            if item_ids.is_empty() || item_ids.len() > 100 {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "duplicate requires between 1 and 100 items",
                ));
            }
            let mut copies = Vec::with_capacity(item_ids.len());
            for item_id in item_ids {
                let (track_index, item_index) = find_item_location(project, &item_id)?;
                if project.tracks[track_index].locked {
                    return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
                }
                let mut copy = project.tracks[track_index].items[item_index].clone();
                if matches!(copy, TimelineItem::Transition(_)) {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "transitions cannot be duplicated",
                    ));
                }
                let new_start = copy.start_ms().checked_add(offset_ms).ok_or_else(|| {
                    CoreError::new(ErrorCode::ValidationFailed, "duplicate time overflow")
                })?;
                let new_id = Uuid::new_v4().to_string();
                set_item_id(&mut copy, new_id.clone());
                set_item_start(&mut copy, new_start);
                copies.push((track_index, copy, new_id));
            }
            let mut changed_ids = Vec::with_capacity(copies.len());
            for (track_index, copy, id) in copies {
                project.tracks[track_index].items.push(copy);
                changed_ids.push(id);
            }
            Ok((changed_ids, "Duplicated timeline items"))
        }
        EditOperation::CreateTrack {
            name,
            track_type,
            index,
            audio_role,
            ducking,
        } => {
            let name = name.trim();
            if name.is_empty() || name.chars().count() > 128 {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "track name must be non-empty and at most 128 characters",
                ));
            }
            let id = Uuid::new_v4().to_string();
            validate_track_audio_settings(track_type, audio_role, ducking.as_ref())?;
            let track = Track {
                id: id.clone(),
                name: name.into(),
                track_type,
                locked: false,
                hidden: false,
                muted: false,
                audio_role,
                ducking,
                items: vec![],
            };
            let index = index.unwrap_or(project.tracks.len());
            if index > project.tracks.len() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "track index is outside the timeline",
                ));
            }
            project.tracks.insert(index, track);
            Ok((vec![id], "Created track"))
        }
        EditOperation::UpdateTrack {
            track_id,
            name,
            index,
            locked,
            hidden,
            muted,
            audio_role,
            ducking,
        } => {
            let current_index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))?;
            if project.tracks[current_index].locked
                && !(locked == Some(false)
                    && name.is_none()
                    && index.is_none()
                    && hidden.is_none()
                    && muted.is_none()
                    && audio_role.is_none()
                    && ducking.is_none())
            {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            if let Some(name) = name {
                let name = name.trim();
                if name.is_empty() || name.chars().count() > 128 {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "track name must be non-empty and at most 128 characters",
                    ));
                }
                project.tracks[current_index].name = name.into();
            }
            if let Some(locked) = locked {
                project.tracks[current_index].locked = locked;
            }
            if let Some(hidden) = hidden {
                project.tracks[current_index].hidden = hidden;
            }
            if let Some(muted) = muted {
                project.tracks[current_index].muted = muted;
            }
            if audio_role.is_some() || ducking.is_some() {
                let role = audio_role.unwrap_or(project.tracks[current_index].audio_role);
                let settings =
                    ducking.unwrap_or_else(|| project.tracks[current_index].ducking.clone());
                validate_track_audio_settings(
                    project.tracks[current_index].track_type,
                    role,
                    settings.as_ref(),
                )?;
                project.tracks[current_index].audio_role = role;
                project.tracks[current_index].ducking = settings;
            }
            if let Some(index) = index {
                if index >= project.tracks.len() {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "track index is outside the timeline",
                    ));
                }
                let track = project.tracks.remove(current_index);
                project.tracks.insert(index, track);
            }
            Ok((vec![track_id], "Updated track"))
        }
        EditOperation::DeleteTrack { track_id } => {
            let index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))?;
            if project.tracks[index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            if !project.tracks[index].items.is_empty() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "only empty tracks can be deleted",
                ));
            }
            project.tracks.remove(index);
            Ok((vec![track_id], "Deleted track"))
        }
        EditOperation::SetItemVisibility { item_id, hidden } => {
            let item = find_editable_item_mut(project, &item_id)?;
            item.set_hidden(hidden);
            Ok((vec![item_id], "Updated item visibility"))
        }
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

fn validate_operations_against(
    project: &Project,
    operations: &[EditOperation],
) -> Result<(), CoreError> {
    let mut candidate = project.clone();
    for operation in operations.iter().cloned() {
        apply_operation(&mut candidate, operation)?;
    }
    Ok(())
}

fn validate_draft_label(label: Option<&str>) -> Result<(), CoreError> {
    if label.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 200) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "draft label must be non-empty and at most 200 characters",
        ));
    }
    Ok(())
}

fn draft_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("drafts")
}

fn draft_path(project_dir: &Path, draft_id: &str) -> Result<PathBuf, CoreError> {
    if draft_id.is_empty()
        || !draft_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "draft ID contains unsupported characters",
        ));
    }
    Ok(draft_dir(project_dir).join(format!("{draft_id}.json")))
}

fn read_draft(project_dir: &Path, draft_id: &str) -> Result<EditDraft, CoreError> {
    let path = draft_path(project_dir, draft_id)?;
    if !path.is_file() {
        return Err(CoreError::new(
            ErrorCode::DraftNotFound,
            "draft was not found",
        ));
    }
    let draft: EditDraft = read_json(&path)?;
    if draft.version != DRAFT_VERSION {
        return Err(CoreError::new(
            ErrorCode::InternalError,
            "draft has an unsupported version",
        ));
    }
    Ok(draft)
}

fn count_drafts(directory: &Path) -> Result<usize, CoreError> {
    let entries =
        std::fs::read_dir(directory).map_err(|error| CoreError::io("cannot list drafts", error))?;
    let mut count = 0;
    for entry in entries {
        let entry = entry.map_err(|error| CoreError::io("cannot read draft entry", error))?;
        if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
            count += 1;
        }
    }
    Ok(count)
}

fn validate_project_settings(settings: &ProjectSettings) -> Result<(), CoreError> {
    if settings.width == 0
        || settings.height == 0
        || settings.width > 7_680
        || settings.height > 4_320
        || !(1..=120).contains(&settings.fps)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "resolution or frame rate is outside supported bounds",
        ));
    }
    Ok(())
}

fn validate_duration(duration_ms: u64) -> Result<(), CoreError> {
    if duration_ms == 0 {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "duration must be greater than zero",
        ));
    }
    Ok(())
}

fn validate_transform(transform: &Transform) -> Result<(), CoreError> {
    if !transform.position_x.is_finite()
        || !transform.position_y.is_finite()
        || !transform.scale.is_finite()
        || !transform.opacity.is_finite()
        || transform.scale <= 0.0
        || transform.scale > 100.0
        || !(0.0..=1.0).contains(&transform.opacity)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "transform contains an invalid position, scale, or opacity",
        ));
    }
    Ok(())
}

fn validate_text(text: &str, font_size: u32, color: &str) -> Result<(), CoreError> {
    if text.is_empty() || text.len() > 4_096 || !(1..=1_000).contains(&font_size) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text, font size, or color is invalid",
        ));
    }
    validate_color(color)?;
    Ok(())
}

fn validate_text_style(style: &TextStyle) -> Result<(), CoreError> {
    validate_color(&style.outline_color)?;
    validate_color(&style.shadow.color)?;
    validate_color(&style.background_color)?;
    if style
        .wrap_width_px
        .is_some_and(|value| value == 0 || value > 7_680)
        || style.outline_width_px > 100
        || !style.shadow.opacity.is_finite()
        || !(0.0..=1.0).contains(&style.shadow.opacity)
        || !style.background_opacity.is_finite()
        || !(0.0..=1.0).contains(&style.background_opacity)
        || style.line_spacing_px.unsigned_abs() > 4_320
        || [
            style.padding.top,
            style.padding.right,
            style.padding.bottom,
            style.padding.left,
        ]
        .into_iter()
        .any(|value| value > 4_320)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text style is outside supported bounds",
        ));
    }
    Ok(())
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), CoreError> {
    if width == 0 || height == 0 || width > 7_680 || height > 4_320 {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "shape dimensions are outside supported bounds",
        ));
    }
    Ok(())
}

fn validate_color(color: &str) -> Result<(), CoreError> {
    if color.len() != 7
        || !color.starts_with('#')
        || !color.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "color must use #RRGGBB format",
        ));
    }
    Ok(())
}

fn validate_audio(audio: &AudioSettings) -> Result<(), CoreError> {
    if !audio.volume.is_finite() || !(0.0..=4.0).contains(&audio.volume) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "audio volume must be between 0 and 4",
        ));
    }
    Ok(())
}

fn validate_keyframes(keyframes: &[Keyframe]) -> Result<(), CoreError> {
    let mut previous_by_property = BTreeMap::new();
    for keyframe in keyframes {
        match (keyframe.property, &keyframe.value) {
            (KeyframeProperty::Position, KeyframeValue::Position { x, y })
                if x.is_finite() && y.is_finite() => {}
            (KeyframeProperty::Scale, KeyframeValue::Scalar { value })
                if value.is_finite() && *value > 0.0 && *value <= 100.0 => {}
            (KeyframeProperty::Opacity, KeyframeValue::Scalar { value })
                if value.is_finite() && (0.0..=1.0).contains(value) => {}
            (KeyframeProperty::Volume, KeyframeValue::Scalar { value })
                if value.is_finite() && (0.0..=4.0).contains(value) => {}
            _ => {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "keyframe value does not match its property",
                ));
            }
        }
        if let Some(time_ms) = previous_by_property.get(&keyframe.property)
            && keyframe.time_ms <= *time_ms
        {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "keyframes for a property must be strictly increasing",
            ));
        }
        previous_by_property.insert(keyframe.property, keyframe.time_ms);
    }
    Ok(())
}

fn validate_ducking(settings: &DuckingSettings) -> Result<(), CoreError> {
    if !settings.gain.is_finite()
        || !(0.0..=1.0).contains(&settings.gain)
        || settings.attack_ms > 60_000
        || settings.release_ms > 60_000
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "ducking settings are invalid",
        ));
    }
    Ok(())
}

fn validate_track_audio_settings(
    track_type: TrackType,
    role: AudioTrackRole,
    ducking: Option<&DuckingSettings>,
) -> Result<(), CoreError> {
    if track_type != TrackType::Audio && (role != AudioTrackRole::Unassigned || ducking.is_some()) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "audio roles require an audio track",
        ));
    }
    if ducking.is_some() && role != AudioTrackRole::Music {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "ducking settings require a music track",
        ));
    }
    if let Some(settings) = ducking {
        validate_ducking(settings)?;
    }
    Ok(())
}

fn validate_visual_track(track: TrackType) -> Result<(), CoreError> {
    if matches!(track, TrackType::Video | TrackType::Overlay) {
        Ok(())
    } else {
        Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "visual items require a video or overlay track",
        ))
    }
}

fn validate_track_media(track: TrackType, media: MediaType) -> Result<(), CoreError> {
    let allowed = matches!(
        (track, media),
        (TrackType::Video, MediaType::Image | MediaType::Video)
            | (TrackType::Overlay, MediaType::Image | MediaType::Video)
            | (TrackType::Audio, MediaType::Audio)
    );
    if !allowed {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "media type is incompatible with the destination track",
        ));
    }
    Ok(())
}

fn validate_item_track(item: &TimelineItem, track: TrackType) -> Result<(), CoreError> {
    match item {
        TimelineItem::Text(_) if track != TrackType::Overlay => Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text items require an overlay track",
        )),
        TimelineItem::SolidColor(_) | TimelineItem::Rectangle(_)
            if !matches!(track, TrackType::Video | TrackType::Overlay) =>
        {
            Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "shape items require a video or overlay track",
            ))
        }
        TimelineItem::Caption(_) if track != TrackType::Caption => Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "caption items require a caption track",
        )),
        TimelineItem::Transition(_) if matches!(track, TrackType::Audio | TrackType::Caption) => {
            Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "visual transitions require a video or overlay track",
            ))
        }
        _ => Ok(()),
    }
}

fn find_track_mut<'a>(
    project: &'a mut Project,
    track_id: &str,
) -> Result<&'a mut Track, CoreError> {
    project
        .tracks
        .iter_mut()
        .find(|track| track.id == track_id)
        .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))
}

fn editable_track_mut<'a>(
    project: &'a mut Project,
    track_id: &str,
) -> Result<&'a mut Track, CoreError> {
    let track = find_track_mut(project, track_id)?;
    if track.locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(track)
}

fn find_editable_item_mut<'a>(
    project: &'a mut Project,
    item_id: &str,
) -> Result<&'a mut TimelineItem, CoreError> {
    let (track_index, item_index) = find_item_location(project, item_id)?;
    if project.tracks[track_index].locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(&mut project.tracks[track_index].items[item_index])
}

fn find_item_location(project: &Project, item_id: &str) -> Result<(usize, usize), CoreError> {
    project
        .tracks
        .iter()
        .enumerate()
        .find_map(|(track_index, track)| {
            track
                .items
                .iter()
                .position(|item| item.id() == item_id)
                .map(|item_index| (track_index, item_index))
        })
        .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "timeline item was not found"))
}

fn ensure_item_track_unlocked(project: &Project, item_id: &str) -> Result<(), CoreError> {
    let (track_index, _) = find_item_location(project, item_id)?;
    if project.tracks[track_index].locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(())
}

fn remove_item(project: &mut Project, item_id: &str) -> Result<TimelineItem, CoreError> {
    for track in &mut project.tracks {
        if let Some(index) = track.items.iter().position(|item| item.id() == item_id) {
            return Ok(track.items.remove(index));
        }
    }
    Err(CoreError::new(
        ErrorCode::ItemNotFound,
        "timeline item was not found",
    ))
}

fn set_item_start(item: &mut TimelineItem, start_ms: u64) {
    match item {
        TimelineItem::Media(media) => media.start_ms = start_ms,
        TimelineItem::Text(text) => text.start_ms = start_ms,
        TimelineItem::SolidColor(shape) => shape.start_ms = start_ms,
        TimelineItem::Rectangle(shape) => shape.start_ms = start_ms,
        TimelineItem::Caption(caption) => caption.start_ms = start_ms,
        TimelineItem::Transition(transition) => transition.start_ms = start_ms,
    }
}

fn set_item_id(item: &mut TimelineItem, id: String) {
    match item {
        TimelineItem::Media(media) => media.id = id,
        TimelineItem::Text(text) => text.id = id,
        TimelineItem::SolidColor(shape) => shape.id = id,
        TimelineItem::Rectangle(shape) => shape.id = id,
        TimelineItem::Caption(caption) => caption.id = id,
        TimelineItem::Transition(transition) => transition.id = id,
    }
}

fn check_revision(project: &Project, expected_revision: u64) -> Result<(), CoreError> {
    if project.revision != expected_revision {
        return Err(CoreError::new(
            ErrorCode::RevisionConflict,
            format!(
                "expected revision {expected_revision}, current revision is {}",
                project.revision
            ),
        ));
    }
    Ok(())
}

fn bump_revision(project: &mut Project) -> Result<(), CoreError> {
    project.revision = project
        .revision
        .checked_add(1)
        .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "project revision overflow"))?;
    project.updated_at_ms = now_ms()?;
    Ok(())
}

fn push_undo(history: &mut History, project: &Project) {
    history.undo.push(project.clone());
    if history.undo.len() > HISTORY_LIMIT {
        history.undo.remove(0);
    }
    history.redo.clear();
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

fn persist(dir: &Path, project: &Project, history: &History) -> Result<Vec<String>, CoreError> {
    persist_transaction(dir, project, history, None)
}

fn persist_transaction(
    dir: &Path,
    project: &Project,
    history: &History,
    committed_draft_id: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    inject_persistence_fault(PersistencePhase::BeforeJournal)?;
    let transaction = ProjectTransaction {
        version: TRANSACTION_VERSION,
        project: project.clone(),
        history: history.clone(),
        committed_draft_id: committed_draft_id.map(str::to_owned),
    };
    write_json_atomic(&transaction_path(dir), &transaction)?;

    let mut warnings = Vec::new();
    if inject_persistence_fault(PersistencePhase::AfterJournal).is_err()
        || write_json_atomic(&project_path(dir), &transaction.project).is_err()
        || inject_persistence_fault(PersistencePhase::AfterProject).is_err()
        || write_json_atomic(&history_path(dir), &transaction.history).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        return Ok(warnings);
    }

    if inject_persistence_fault(PersistencePhase::AfterHistory).is_err() {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        if transaction.committed_draft_id.is_some() {
            warnings.push(DRAFT_CLEANUP_FAILED.into());
        }
        return Ok(warnings);
    }

    if let Some(draft_id) = transaction.committed_draft_id.as_deref()
        && remove_file_if_exists(&draft_path(dir, draft_id)?).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        warnings.push(DRAFT_CLEANUP_FAILED.into());
        return Ok(warnings);
    }

    if inject_persistence_fault(PersistencePhase::AfterDraftCleanup).is_err()
        || remove_file_durable(&transaction_path(dir)).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        return Ok(warnings);
    }

    let _ = inject_persistence_fault(PersistencePhase::AfterJournalCleanup);
    Ok(warnings)
}

fn project_path(dir: &Path) -> PathBuf {
    dir.join("project.json")
}

fn history_path(dir: &Path) -> PathBuf {
    dir.join("history.json")
}

fn transaction_path(dir: &Path) -> PathBuf {
    dir.join(TRANSACTION_FILE)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, CoreError> {
    let mut file = File::open(path).map_err(|error| CoreError::io("cannot open data", error))?;
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|error| CoreError::io("cannot read data", error))?;
    Ok(serde_json::from_str(&data)?)
}

fn load_project_data(dir: &Path) -> Result<(Project, History), CoreError> {
    recover_transaction(dir)?;
    let project_file = project_path(dir);
    let history_file = history_path(dir);
    let mut project: Project = read_json(&project_file)?;
    let mut history: History = if history_file.exists() {
        read_json(&history_file)?
    } else {
        History::default()
    };

    let mut changed = migrate_project(&mut project)? | migrate_project_assets(&mut project, dir)?;
    for snapshot in history.undo.iter_mut().chain(&mut history.redo) {
        changed |= migrate_project(snapshot)?;
        changed |= migrate_project_assets(snapshot, dir)?;
    }
    if changed {
        let _ = persist(dir, &project, &history)?;
    }
    Ok((project, history))
}

fn recover_transaction(dir: &Path) -> Result<(), CoreError> {
    let path = transaction_path(dir);
    if !path.exists() {
        return Ok(());
    }
    let transaction = read_transaction(&path)?;
    validate_transaction(dir, &transaction)?;
    replay_transaction(dir, &transaction)
}

fn read_transaction(path: &Path) -> Result<ProjectTransaction, CoreError> {
    let data = std::fs::read(path)
        .map_err(|error| recovery_error(format!("cannot read transaction journal: {error}")))?;
    serde_json::from_slice(&data)
        .map_err(|error| recovery_error(format!("invalid transaction journal: {error}")))
}

fn validate_transaction(dir: &Path, transaction: &ProjectTransaction) -> Result<(), CoreError> {
    if transaction.version != TRANSACTION_VERSION {
        return Err(recovery_error(format!(
            "unsupported transaction journal version {}",
            transaction.version
        )));
    }
    let directory_id = dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| recovery_error("project directory has no valid identifier"))?;
    if transaction.project.id != directory_id {
        return Err(recovery_error(
            "transaction project identity is inconsistent",
        ));
    }
    for snapshot in std::iter::once(&transaction.project)
        .chain(transaction.history.undo.iter())
        .chain(transaction.history.redo.iter())
    {
        if snapshot.id != transaction.project.id {
            return Err(recovery_error(
                "transaction history contains a different project identity",
            ));
        }
        if snapshot.schema_version != PROJECT_SCHEMA_VERSION {
            return Err(recovery_error(format!(
                "transaction contains unsupported project schema version {}",
                snapshot.schema_version
            )));
        }
    }
    if let Some(draft_id) = transaction.committed_draft_id.as_deref() {
        draft_path(dir, draft_id)
            .map_err(|_| recovery_error("transaction contains an invalid draft identifier"))?;
    }
    Ok(())
}

fn replay_transaction(dir: &Path, transaction: &ProjectTransaction) -> Result<(), CoreError> {
    inject_persistence_fault(PersistencePhase::AfterJournal).map_err(as_recovery_error)?;
    write_json_atomic(&project_path(dir), &transaction.project).map_err(as_recovery_error)?;
    inject_persistence_fault(PersistencePhase::AfterProject).map_err(as_recovery_error)?;
    write_json_atomic(&history_path(dir), &transaction.history).map_err(as_recovery_error)?;
    inject_persistence_fault(PersistencePhase::AfterHistory).map_err(as_recovery_error)?;
    if let Some(draft_id) = transaction.committed_draft_id.as_deref() {
        remove_file_if_exists(&draft_path(dir, draft_id)?).map_err(as_recovery_error)?;
    }
    inject_persistence_fault(PersistencePhase::AfterDraftCleanup).map_err(as_recovery_error)?;
    remove_file_durable(&transaction_path(dir)).map_err(as_recovery_error)?;
    inject_persistence_fault(PersistencePhase::AfterJournalCleanup).map_err(as_recovery_error)
}

fn recovery_error(message: impl Into<String>) -> CoreError {
    CoreError::new(ErrorCode::ProjectRecoveryFailed, message)
}

fn as_recovery_error(error: CoreError) -> CoreError {
    recovery_error(error.message)
}

fn migrate_project(project: &mut Project) -> Result<bool, CoreError> {
    match project.schema_version {
        1..=5 => {
            project.schema_version = PROJECT_SCHEMA_VERSION;
            Ok(true)
        }
        PROJECT_SCHEMA_VERSION => Ok(false),
        version => Err(CoreError::new(
            ErrorCode::InternalError,
            format!(
                "unsupported project schema version {version}; this build supports up to {PROJECT_SCHEMA_VERSION}"
            ),
        )),
    }
}

struct StoredAsset {
    content_hash: ContentHash,
    relative_path: String,
    size_bytes: u64,
}

fn generated_display_name(origin: &GeneratedAssetOrigin) -> String {
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

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + characters.as_str()
    })
}

fn sanitize_file_name(value: &str) -> String {
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

fn hash_file(path: &Path) -> Result<(String, u64), CoreError> {
    let mut file = File::open(path)
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

fn hash_relative_path(digest: &str) -> String {
    format!("assets/sha256/{}/{}", &digest[..2], digest)
}

fn store_content_addressed(dir: &Path, source: &Path) -> Result<StoredAsset, CoreError> {
    let (digest, size_bytes) = hash_file(source)?;
    let relative_path = hash_relative_path(&digest);
    let destination = dir.join(&relative_path);
    if !destination.is_file() {
        let parent = destination
            .parent()
            .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "asset path has no parent"))?;
        std::fs::create_dir_all(parent)
            .map_err(|error| CoreError::io("cannot create asset store", error))?;
        let temporary = parent.join(format!(".{}.{}.tmp", digest, Uuid::new_v4()));
        std::fs::copy(source, &temporary)
            .map_err(|error| CoreError::io("cannot copy asset", error))?;
        if let Err(error) = std::fs::rename(&temporary, &destination) {
            if !destination.is_file() {
                let _ = std::fs::remove_file(&temporary);
                return Err(CoreError::io("cannot publish asset", error));
            }
            let _ = std::fs::remove_file(&temporary);
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

fn migrate_project_assets(project: &mut Project, dir: &Path) -> Result<bool, CoreError> {
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
        let (digest, size_bytes) = hash_file(&source)?;
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
        let stored = store_content_addressed(dir, &source)?;
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

fn garbage_collect(dir: &Path, project: &Project, history: &History) -> Vec<String> {
    let referenced = std::iter::once(project)
        .chain(history.undo.iter())
        .chain(history.redo.iter())
        .flat_map(|snapshot| snapshot.assets.iter())
        .map(|asset| asset.project_relative_path.replace('\\', "/"))
        .collect::<std::collections::HashSet<_>>();
    let root = dir.join("assets");
    let mut files = vec![];
    collect_files(&root, &mut files);
    let mut failed = false;
    for file in files {
        let relative = file
            .strip_prefix(dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        if !referenced.contains(&relative) && std::fs::remove_file(&file).is_err() {
            failed = true;
        }
    }
    if failed {
        vec!["ASSET_GC_FAILED".into()]
    } else {
        vec![]
    }
}

fn finish_persistence(
    dir: &Path,
    project: &Project,
    history: &History,
    mut warnings: Vec<String>,
) -> Vec<String> {
    if !warnings
        .iter()
        .any(|warning| warning == PERSISTENCE_RECOVERY_PENDING)
    {
        warnings.extend(garbage_collect(dir, project, history));
    }
    warnings
}

fn collect_files(directory: &Path, output: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, output);
        } else if path.is_file() {
            output.push(path);
        }
    }
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), CoreError> {
    let temp = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    let result = (|| {
        let data = serde_json::to_vec_pretty(value)?;
        let mut file =
            File::create(&temp).map_err(|error| CoreError::io("cannot create data", error))?;
        file.write_all(&data)
            .map_err(|error| CoreError::io("cannot write data", error))?;
        file.sync_all()
            .map_err(|error| CoreError::io("cannot sync data", error))?;
        drop(file);
        std::fs::rename(&temp, path)
            .map_err(|error| CoreError::io("cannot replace data", error))?;
        sync_parent(path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn remove_file_if_exists(path: &Path) -> Result<(), CoreError> {
    match std::fs::remove_file(path) {
        Ok(()) => sync_parent(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CoreError::io("cannot remove data", error)),
    }
}

fn remove_file_durable(path: &Path) -> Result<(), CoreError> {
    std::fs::remove_file(path).map_err(|error| CoreError::io("cannot remove data", error))?;
    sync_parent(path)
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> Result<(), CoreError> {
    let parent = path
        .parent()
        .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "data path has no parent"))?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| CoreError::io("cannot sync data directory", error))
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> Result<(), CoreError> {
    Ok(())
}

fn inject_persistence_fault(phase: PersistencePhase) -> Result<(), CoreError> {
    #[cfg(test)]
    if PERSISTENCE_FAULT.with(|fault| {
        if fault.get() == Some(phase) {
            fault.set(None);
            true
        } else {
            false
        }
    }) {
        return Err(CoreError::new(
            ErrorCode::InternalError,
            format!("injected persistence fault after {phase:?}"),
        ));
    }
    #[cfg(not(test))]
    let _ = phase;
    Ok(())
}

fn now_ms() -> Result<u64, CoreError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoreError::new(ErrorCode::InternalError, error.to_string()))?
        .as_millis();
    u64::try_from(millis)
        .map_err(|_| CoreError::new(ErrorCode::InternalError, "system time overflow"))
}

struct ProjectLock(File);

impl ProjectLock {
    fn exclusive(dir: &Path) -> Result<Self, CoreError> {
        let file = open_lock(dir)?;
        file.lock_exclusive()
            .map_err(|error| CoreError::io("cannot lock project", error))?;
        Ok(Self(file))
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn open_lock(dir: &Path) -> Result<File, CoreError> {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("project.lock"))
        .map_err(|error| CoreError::io("cannot open project lock", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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

    fn set_persistence_fault(phase: PersistencePhase) {
        PERSISTENCE_FAULT.with(|fault| fault.set(Some(phase)));
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
            set_persistence_fault(phase);
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
    fn persistence_transaction_rejects_before_commit_without_mutation() {
        let (core, _) = core();
        let created = core
            .create_project("pre-commit", ProjectSettings::default())
            .unwrap();
        let dir = core.paths().project_dir(&created.project_id).unwrap();
        let project_before = std::fs::read(project_path(&dir)).unwrap();
        let history_before = std::fs::read(history_path(&dir)).unwrap();

        set_persistence_fault(PersistencePhase::BeforeJournal);
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

        set_persistence_fault(PersistencePhase::AfterJournal);
        core.edit(&created.project_id, 0, create_test_track())
            .unwrap();
        set_persistence_fault(PersistencePhase::AfterProject);
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

        set_persistence_fault(PersistencePhase::AfterHistory);
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

        set_persistence_fault(PersistencePhase::BeforeJournal);
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
