//! Durable draft lifecycle owner.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    CoreError, EditOperation, ErrorCode,
    persistence::{Storage, StorageEntryKind, list_paths, read_json},
};

pub(crate) const DRAFT_VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditDraft {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub font_catalog: Option<std::collections::BTreeMap<String, crate::FontRecord>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    pub font_steps: Option<Vec<std::collections::BTreeMap<String, crate::FontBinding>>>,
    pub version: u32,
    pub id: String,
    pub project_id: String,
    pub base_revision: u64,
    pub label: Option<String>,
    pub operations: Vec<EditOperation>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

fn deserialize_present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
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

pub(crate) fn read_draft(
    storage: &dyn Storage,
    project_dir: &Path,
    draft_id: &str,
) -> Result<EditDraft, CoreError> {
    let path = draft_path(project_dir, draft_id)?;
    if storage.entry_kind(&path).ok() != Some(StorageEntryKind::File) {
        return Err(CoreError::new(
            ErrorCode::DraftNotFound,
            "draft was not found",
        ));
    }
    let draft: EditDraft = read_json(storage, &path)?;
    if !matches!(draft.version, 1 | DRAFT_VERSION) {
        return Err(CoreError::new(
            ErrorCode::InternalError,
            "draft has an unsupported version",
        ));
    }
    if draft.version == 1 && (draft.font_catalog.is_some() || draft.font_steps.is_some()) {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "draft font fields require version 2",
        ));
    }
    if draft.version == DRAFT_VERSION
        && (draft.font_catalog.is_none()
            || draft
                .font_steps
                .as_ref()
                .is_none_or(|steps| steps.len() != draft.operations.len()))
    {
        return Err(CoreError::new(
            ErrorCode::AssetIntegrityFailed,
            "draft font bindings are incomplete",
        ));
    }
    Ok(draft)
}

pub(crate) fn remove_draft(
    storage: &dyn Storage,
    project_dir: &Path,
    draft_id: &str,
) -> Result<(), CoreError> {
    storage
        .remove_durable(&draft_path(project_dir, draft_id)?)
        .map_err(|error| CoreError::io("cannot discard draft", error))
}

pub(crate) fn read_all_drafts(
    storage: &dyn Storage,
    project_dir: &Path,
) -> Result<Vec<EditDraft>, CoreError> {
    let directory = draft_dir(project_dir);
    if !storage.storage_path_exists(&directory) {
        return Ok(vec![]);
    }
    let mut draft_ids = Vec::new();
    for path in list_paths(storage, &directory)? {
        if storage.entry_kind(&path).ok() != Some(StorageEntryKind::File)
            || path.extension().and_then(|value| value.to_str()) != Some("json")
        {
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
        .map(|draft_id| read_draft(storage, project_dir, &draft_id))
        .collect()
}

pub(crate) fn count_drafts(storage: &dyn Storage, directory: &Path) -> Result<usize, CoreError> {
    Ok(list_paths(storage, directory)?
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .count())
}
