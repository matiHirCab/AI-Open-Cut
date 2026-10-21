//! Closed sentence-manifest DTOs. No provider or transport ownership.
use crate::{GeneratedAssetOrigin, MediaProbeFacts, SpeechMarkerPolicy, SpeechSynthesisRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MAX_NARRATION_BLOCKS: usize = 32;
pub const MAX_NARRATION_SOURCE_BYTES: usize = 32_768;
pub const MAX_NARRATION_TEXT_BYTES: usize = 4096;
pub const MAX_NARRATION_MANIFEST_BYTES: usize = 524_288;
pub const MAX_NARRATION_PROJECT_BYTES: usize = 4_194_304;
pub const MAX_NARRATION_INPUT_BYTES: usize = 16_777_216;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationPlanInput {
    pub source_text: String,
    pub profile_id: String,
    pub expected_version: u64,
    pub max_text_characters: usize,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub boundaries: Option<Vec<usize>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationPlannedBlock {
    pub id: String,
    pub start_scalar: usize,
    pub end_scalar: usize,
    pub paragraph: usize,
    pub display_text: String,
    pub request: SpeechSynthesisRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationPlan {
    pub id: String,
    pub source_text: String,
    pub max_text_characters: usize,
    pub blocks: Vec<NarrationPlannedBlock>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationPreparedBlock {
    pub block: NarrationPlannedBlock,
    pub spoken_segments: Vec<String>,
    pub spoken_text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationCaptionRef {
    pub id: String,
    pub text: String,
    pub start_ms: u64,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationCueRef {
    pub id: String,
    pub name: String,
    pub time_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationBinding {
    pub track_id: String,
    pub item_id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub source_in_ms: u64,
    pub caption_refs: Vec<NarrationCaptionRef>,
    pub cue_refs: Vec<NarrationCueRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationStoredBlock {
    pub id: String,
    pub start_scalar: usize,
    pub end_scalar: usize,
    pub paragraph: usize,
    pub display_text: String,
    pub request: SpeechSynthesisRequest,
    pub spoken_segments: Vec<String>,
    pub spoken_text: String,
    pub binding: NarrationBinding,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationManifest {
    pub marker_policy: SpeechMarkerPolicy,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub caption: Option<NarrationCaptionOptions>,
    pub id: String,
    pub source_text: String,
    pub max_text_characters: usize,
    pub blocks: Vec<NarrationStoredBlock>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationGeneratedBlock {
    pub prepared: NarrationPreparedBlock,
    pub path: PathBuf,
    pub duration_ms: u64,
    pub origin: GeneratedAssetOrigin,
    pub probe: MediaProbeFacts,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NarrationCaptionGranularity {
    Sentence,
    Word,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationCaptionOptions {
    pub track_id: String,
    pub granularity: NarrationCaptionGranularity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitNarrationBlocksInput {
    pub project_id: String,
    pub expected_revision: u64,
    pub plan: NarrationPlan,
    pub track_id: String,
    pub start_ms: u64,
    pub blocks: Vec<NarrationGeneratedBlock>,
    #[serde(default)]
    pub marker_policy: SpeechMarkerPolicy,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub caption: Option<NarrationCaptionOptions>,
}

fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}
