use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkParameters {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_cleanup"
    )]
    pub cleanup: Option<ArtworkCleanup>,
    pub outline: ArtworkOutline,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkCleanup {
    pub color: [u8; 3],
    pub tolerance: u8,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkOutline {
    pub radius: u32,
    pub color: [u8; 4],
}
impl ArtworkParameters {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.outline.radius > 16 {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "artwork outline radius exceeds 16",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkDerivation {
    pub version: u32,
    pub source_asset_id: String,
    pub source_hash: String,
    pub parameters: ArtworkParameters,
    pub source_width: u32,
    pub source_height: u32,
    pub content_offset: [u32; 2],
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkReviewHashes {
    pub result: String,
    pub mask: String,
    pub light: String,
    pub dark: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkManifest {
    pub version: u32,
    pub project_id: String,
    pub expected_revision: u64,
    pub source_asset_id: String,
    pub source_hash: String,
    pub parameters: ArtworkParameters,
    pub result_hash: String,
    pub review_hashes: ArtworkReviewHashes,
    pub width: u32,
    pub height: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub content_offset: [u32; 2],
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkPreviewFiles {
    pub result: String,
    pub mask: String,
    pub light: String,
    pub dark: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtworkPreview {
    pub manifest: ArtworkManifest,
    pub files: ArtworkPreviewFiles,
}

fn deserialize_cleanup<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ArtworkCleanup>, D::Error> {
    ArtworkCleanup::deserialize(deserializer).map(Some)
}
pub(super) fn deserialize_derivation<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ArtworkDerivation>, D::Error> {
    ArtworkDerivation::deserialize(deserializer).map(Some)
}
