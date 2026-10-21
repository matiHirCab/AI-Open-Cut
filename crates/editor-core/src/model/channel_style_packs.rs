//! Portable native content. Validation and storage remain in their core owners.
use crate::{
    ComponentDefinition, ContentHash, MediaProbeFacts, ParentReference, SlotValue, Transform,
    Transform2D,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CHANNEL_PACK_FORMAT_VERSION: u32 = 1;
/// CP1 component semantics remain version46 across unrelated project migrations.
pub const CHANNEL_PACK_COMPONENT_SCHEMA_VERSION: u32 = 46;
pub const MAX_CHANNEL_PACK_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CHANNEL_LIBRARY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_CHANNEL_PACKS: usize = 16;
pub const MAX_CHANNEL_RECIPES: usize = 16;
pub const MAX_CHANNEL_ITEMS: usize = 1024;
pub const MAX_CHANNEL_MARKERS: usize = 32;
pub const MAX_CHANNEL_IMAGES: usize = 32;
pub const MAX_CHANNEL_FONTS: usize = 32;
pub const MAX_CHANNEL_PLACEMENTS: usize = 32;
pub const MAX_CHANNEL_JSON_DEPTH: usize = 64;
pub const MAX_CHANNEL_VERSION: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelProfileRef {
    pub profile_id: String,
    pub expected_version: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelCaptionRole {
    pub recipe_key: String,
    pub text_slot_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelRecipe {
    pub key: String,
    pub definition: ComponentDefinition,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelFontFace {
    pub sha256: String,
    pub size_bytes: u64,
    pub face_index: u32,
    pub data_base64: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelImageDependency {
    pub key: String,
    pub content_hash: ContentHash,
    pub size_bytes: u64,
    pub probe: MediaProbeFacts,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPackPackage {
    pub format_version: u32,
    pub component_schema_version: u32,
    pub name: String,
    pub recipes: Vec<ChannelRecipe>,
    pub fonts: Vec<ChannelFontFace>,
    pub images: Vec<ChannelImageDependency>,
    // A required nullable field distinguishes an explicit choice from missing data.
    #[serde(deserialize_with = "required_nullable")]
    pub caption_role: Option<ChannelCaptionRole>,
    #[serde(deserialize_with = "required_nullable")]
    pub narration_profile: Option<ChannelProfileRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPack {
    pub id: String,
    pub version: u64,
    pub package: ChannelPackPackage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPackMetadata {
    pub id: String,
    pub version: u64,
    pub name: String,
    pub recipe_count: usize,
    pub font_count: usize,
    pub image_count: usize,
    pub has_caption_role: bool,
    pub has_narration_profile: bool,
    pub package_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelCaptureInput {
    pub project_id: String,
    pub expected_revision: u64,
    pub component_ids: Vec<String>,
    pub name: String,
    #[serde(deserialize_with = "required_nullable")]
    pub caption_role: Option<ChannelCaptionRole>,
    #[serde(deserialize_with = "required_nullable")]
    pub narration_profile: Option<ChannelProfileRef>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelImageBinding {
    pub dependency_key: String,
    pub asset_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChannelNarrationChoice {
    Saved {},
    Disabled {},
    Replacement { reference: ChannelProfileRef },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPlacement {
    pub recipe_key: String,
    pub track_id: String,
    pub start_ms: u64,
    pub trim_start_ms: u64,
    pub duration_ms: u64,
    pub time_scale: f64,
    pub stagger_ms: u64,
    pub slot_values: BTreeMap<String, SlotValue>,
    pub transform: Transform,
    #[serde(deserialize_with = "required_nullable")]
    pub transform2d: Option<Transform2D>,
    pub hidden: bool,
    pub z_index: i32,
    #[serde(deserialize_with = "required_nullable")]
    pub parent: Option<ParentReference>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelComponentMapping {
    pub recipe_key: String,
    pub component_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChannelApplyMode {
    Instantiate {
        placements: Vec<ChannelPlacement>,
    },
    Reapply {
        components: Vec<ChannelComponentMapping>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelApplyInput {
    pub project_id: String,
    pub expected_revision: u64,
    pub pack_id: String,
    pub expected_version: u64,
    pub images: Vec<ChannelImageBinding>,
    pub narration: ChannelNarrationChoice,
    pub application: ChannelApplyMode,
}

fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
