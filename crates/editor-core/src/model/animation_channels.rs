use serde::{Deserialize, Serialize};

use super::ParentReference;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AnimationChannelProperty {
    #[serde(rename = "transform.position_x")]
    PositionX,
    #[serde(rename = "transform.position_y")]
    PositionY,
    #[serde(rename = "transform.scale_x")]
    ScaleX,
    #[serde(rename = "transform.scale_y")]
    ScaleY,
    #[serde(rename = "transform.rotation_deg")]
    RotationDeg,
    #[serde(rename = "transform.skew_x_deg")]
    SkewXDeg,
    #[serde(rename = "transform.skew_y_deg")]
    SkewYDeg,
    #[serde(rename = "transform.anchor_x")]
    AnchorX,
    #[serde(rename = "transform.anchor_y")]
    AnchorY,
    #[serde(rename = "transform.opacity")]
    Opacity,
    #[serde(rename = "media.crop_x")]
    CropX,
    #[serde(rename = "media.crop_y")]
    CropY,
    #[serde(rename = "media.crop_width")]
    CropWidth,
    #[serde(rename = "media.crop_height")]
    CropHeight,
    #[serde(rename = "media.source_position_ms")]
    SourcePositionMs,
    #[serde(rename = "media.playback_rate")]
    PlaybackRate,
    #[serde(rename = "graphic.path_points")]
    PathPoints,
    #[serde(rename = "graphic.path_trim")]
    PathTrim,
    #[serde(rename = "graphic.fill_color")]
    FillColor,
    #[serde(rename = "graphic.stroke_color")]
    StrokeColor,
    #[serde(rename = "graphic.stroke_width")]
    StrokeWidth,
    #[serde(rename = "graphic.gradient_stops")]
    GradientStops,
    #[serde(rename = "effect.blur_radius")]
    BlurRadius,
    #[serde(rename = "effect.glow_radius")]
    GlowRadius,
    #[serde(rename = "effect.tint_color")]
    TintColor,
    #[serde(rename = "effect.vignette_amount")]
    VignetteAmount,
    #[serde(rename = "effect.particle_amount")]
    ParticleAmount,
    #[serde(rename = "audio.gain_db")]
    GainDb,
    #[serde(rename = "audio.pan")]
    Pan,
}

impl AnimationChannelProperty {
    pub(crate) fn active(self) -> bool {
        matches!(
            self,
            Self::PositionX
                | Self::PositionY
                | Self::ScaleX
                | Self::ScaleY
                | Self::Opacity
                | Self::GainDb
        )
    }

    pub(crate) fn visual(self) -> bool {
        self.active() && self != Self::GainDb
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationCurve {
    Hold,
    Linear,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnimationChannelValue {
    Scalar { value: f64 },
    Point { x: f64, y: f64 },
    Rgba { r: f64, g: f64, b: f64, a: f64 },
    PathPoints { points: Vec<AnimationPoint> },
    GradientStops { stops: Vec<AnimationGradientStop> },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationGradientStop {
    pub offset: f64,
    pub color: [f64; 4],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannelKeyframe {
    pub time_ms: u64,
    pub value: AnimationChannelValue,
    pub curve: AnimationCurve,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannel {
    pub property: AnimationChannelProperty,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<ParentReference>,
    pub keyframes: Vec<AnimationChannelKeyframe>,
}
