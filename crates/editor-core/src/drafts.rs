//! Durable draft lifecycle owner.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{CoreError, EditOperation, ErrorCode, persistence::read_json};

pub(crate) const DRAFT_VERSION: u32 = 1;

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

pub(crate) fn draft_dir(project_dir: &Path) -> PathBuf {
    project_dir.join("drafts")
}

pub(crate) fn draft_path(project_dir: &Path, draft_id: &str) -> Result<PathBuf, CoreError> {
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

pub(crate) fn read_draft(project_dir: &Path, draft_id: &str) -> Result<EditDraft, CoreError> {
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

pub(crate) fn read_all_drafts(project_dir: &Path) -> Result<Vec<EditDraft>, CoreError> {
    let directory = draft_dir(project_dir);
    if !directory.exists() {
        return Ok(vec![]);
    }
    let mut draft_ids = Vec::new();
    for path in crate::persistence::list_paths(&directory)? {
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "draft has an invalid identifier",
                )
            })?
            .to_owned();
        draft_ids.push(id);
    }
    draft_ids.sort();
    draft_ids
        .into_iter()
        .map(|draft_id| read_draft(project_dir, &draft_id))
        .collect()
}

pub(crate) fn count_drafts(directory: &Path) -> Result<usize, CoreError> {
    Ok(crate::persistence::list_paths(directory)?
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .count())
}
