use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationTargetKind {
    GraphicGeometry,
    GraphicFill,
    GraphicStroke,
    Effect,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationTarget {
    pub kind: AnimationTargetKind,
    pub scope: String,
    pub id: String,
}

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
                | Self::RotationDeg
                | Self::CropX
                | Self::CropY
                | Self::CropWidth
                | Self::CropHeight
                | Self::PathPoints
                | Self::PathTrim
                | Self::GradientStops
                | Self::BlurRadius
                | Self::GlowRadius
                | Self::TintColor
                | Self::VignetteAmount
        )
    }

    pub(crate) fn visual(self) -> bool {
        self.active() && self != Self::GainDb
    }

    pub(crate) fn legacy_visual(self) -> bool {
        matches!(
            self,
            Self::PositionX | Self::PositionY | Self::ScaleX | Self::ScaleY | Self::Opacity
        )
    }

    pub(crate) fn extended(self) -> bool {
        self.active() && !self.legacy_visual() && self != Self::GainDb
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AnimationCurve {
    Simple(SimpleAnimationCurve),
    Parameterized(ParameterizedAnimationCurve),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleAnimationCurve {
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ParameterizedAnimationCurve {
    CubicBezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Spring {
        mass: f64,
        stiffness: f64,
        damping: f64,
        #[serde(rename = "initialVelocity")]
        initial_velocity: f64,
    },
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationLoopMode {
    Repeat,
    PingPong,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AnimationLoopIterations {
    Finite(u32),
    Infinite(AnimationInfiniteIterations),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationInfiniteIterations {
    Infinite,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationLoop {
    pub mode: AnimationLoopMode,
    pub iterations: AnimationLoopIterations,
}

fn deserialize_present_loop<'de, D>(deserializer: D) -> Result<Option<AnimationLoop>, D::Error>
where
    D: Deserializer<'de>,
{
    AnimationLoop::deserialize(deserializer).map(Some)
}

/// Retains the original source function when an item window is edited.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationClock {
    pub offset_ms: i64,
    pub source_duration_ms: u64,
}

impl<'de> Deserialize<'de> for AnimationClock {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            offset_ms: i64,
            source_duration_ms: u64,
        }
        let fields = Fields::deserialize(deserializer)?;
        const SAFE: u64 = 9_007_199_254_740_991;
        if fields.offset_ms.unsigned_abs() > SAFE
            || fields.source_duration_ms == 0
            || fields.source_duration_ms > SAFE
        {
            return Err(serde::de::Error::custom(
                "retained animation clock exceeds safe bounds",
            ));
        }
        Ok(Self {
            offset_ms: fields.offset_ms,
            source_duration_ms: fields.source_duration_ms,
        })
    }
}

fn deserialize_present_clock<'de, D>(deserializer: D) -> Result<Option<AnimationClock>, D::Error>
where
    D: Deserializer<'de>,
{
    AnimationClock::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannel {
    #[serde(
        default,
        deserialize_with = "deserialize_present_clock",
        skip_serializing_if = "Option::is_none"
    )]
    pub clock: Option<AnimationClock>,
    pub property: AnimationChannelProperty,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<AnimationTarget>,
    pub keyframes: Vec<AnimationChannelKeyframe>,
    #[serde(
        default,
        deserialize_with = "deserialize_present_loop",
        skip_serializing_if = "Option::is_none"
    )]
    pub r#loop: Option<AnimationLoop>,
}
