use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaCrop {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for MediaCrop {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum VisualEffect {
    GaussianBlur {
        id: String,
        radius_px: f64,
    },
    Glow {
        id: String,
        radius_px: f64,
        intensity: f64,
        color: crate::VectorColor,
    },
    ColorTint {
        id: String,
        color: crate::VectorColor,
    },
    Vignette {
        id: String,
        amount: f64,
    },
    ColorAdjustment {
        id: String,
        exposure_stops: f64,
        contrast: f64,
        saturation: f64,
    },
}

impl VisualEffect {
    pub fn id(&self) -> &str {
        match self {
            Self::GaussianBlur { id, .. }
            | Self::Glow { id, .. }
            | Self::ColorTint { id, .. }
            | Self::Vignette { id, .. }
            | Self::ColorAdjustment { id, .. } => id,
        }
    }
}
