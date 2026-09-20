//! Opt-in advanced layout over the pinned shaping profile.
use super::deserialize_present;
use serde::{Deserialize, Serialize};

pub(super) fn deserialize_text_layout<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Box<TextLayout>>, D::Error> {
    Box::<TextLayout>::deserialize(deserializer)
        .map(Some)
        .map_err(|error| {
            serde::de::Error::custom(format!(
                "{}{error}",
                crate::error::LAYOUT_DECODE_ERROR_PREFIX
            ))
        })
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextWrap {
    None,
    #[default]
    Word,
    Cluster,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextFit {
    #[default]
    None,
    Shrink,
    FitWidth,
    FitBox,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextVerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextBounds {
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub width_px: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub height_px: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextLayout {
    #[serde(default)]
    pub tracking_px: f64,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub line_height_px: Option<f64>,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub bounds: Option<TextBounds>,
    #[serde(default)]
    pub wrap: TextWrap,
    #[serde(default)]
    pub vertical_alignment: TextVerticalAlignment,
    #[serde(default)]
    pub fit: TextFit,
    #[serde(default)]
    pub background_corner_radius_px: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextLayoutDiagnostic {
    pub item_id: String,
    pub resolved_font_size: u32,
    pub line_count: usize,
    pub content_width_px: f64,
    pub content_height_px: f64,
    pub overflow_x: bool,
    pub overflow_y: bool,
}
