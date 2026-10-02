use serde::{Deserialize, Serialize};

use super::{
    AnimationChannelProperty, AnimationCurve, SimpleAnimationCurve, buffered::BufferedValue,
};

pub const ANIMATION_PRESET_COMPILER_VERSION: u32 = 1;
pub const MAX_PRESET_TIME_MS: u64 = 9_007_199_254_740_991;

fn linear() -> AnimationCurve {
    AnimationCurve::Simple(SimpleAnimationCurve::Linear)
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScalarTweenParameters {
    pub property: AnimationChannelProperty,
    pub start_ms: u64,
    pub duration_ms: u64,
    pub from: f64,
    pub to: f64,
    #[serde(default = "linear")]
    pub curve: AnimationCurve,
}

#[derive(Deserialize)]
#[serde(
    remote = "ScalarTweenParameters",
    rename_all = "camelCase",
    deny_unknown_fields
)]
struct ScalarTweenParametersDef {
    property: AnimationChannelProperty,
    start_ms: u64,
    duration_ms: u64,
    from: f64,
    to: f64,
    #[serde(default = "linear")]
    curve: AnimationCurve,
}

impl<'de> Deserialize<'de> for ScalarTweenParameters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = BufferedValue::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom(
                "preset parameters require an object",
            ));
        }
        value
            .deserialize_with(ScalarTweenParametersDef::deserialize)
            .map_err(serde::de::Error::custom)
    }
}

pub const MOTION_PRESET_COMPILER_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum AnimationPresetParameters {
    Scalar(ScalarTweenParameters),
    Pack(MotionPresetParameters),
}

impl From<ScalarTweenParameters> for AnimationPresetParameters {
    fn from(value: ScalarTweenParameters) -> Self {
        Self::Scalar(value)
    }
}

impl<'de> Deserialize<'de> for AnimationPresetParameters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = BufferedValue::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom(
                "preset parameters require an object",
            ));
        }
        if value.get("kind").is_some() {
            value
                .decode()
                .map(Self::Pack)
                .map_err(serde::de::Error::custom)
        } else {
            value
                .decode()
                .map(Self::Scalar)
                .map_err(serde::de::Error::custom)
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MotionPresetParameters {
    ImpactSlam {
        start_ms: u64,
        duration_ms: u64,
        center_x: f64,
        center_y: f64,
        shake_amplitude_px: f64,
        scale_from: f64,
        scale_overshoot: f64,
        scale_to: f64,
        opacity_from: f64,
        opacity_to: f64,
        flash_opacity: f64,
        #[serde(deserialize_with = "object_motion_blur")]
        motion_blur: super::MotionBlur,
    },
    SlideLeft {
        start_ms: u64,
        duration_ms: u64,
        position_from_x: f64,
        position_to_x: f64,
    },
    Scan {
        start_ms: u64,
        duration_ms: u64,
        position_from_x: f64,
        position_to_x: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_iterations"
        )]
        iterations: Option<super::AnimationLoopIterations>,
    },
    Pulse {
        start_ms: u64,
        duration_ms: u64,
        scale_from: f64,
        scale_peak: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_iterations"
        )]
        iterations: Option<super::AnimationLoopIterations>,
    },
    RadarExpand {
        start_ms: u64,
        duration_ms: u64,
        scale_from: f64,
        scale_to: f64,
        opacity_peak: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "present_iterations"
        )]
        iterations: Option<super::AnimationLoopIterations>,
    },
}

fn object_motion_blur<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<super::MotionBlur, D::Error> {
    let value = BufferedValue::deserialize(deserializer)?;
    if !value.is_object() {
        return Err(serde::de::Error::custom(
            "preset motionBlur requires an object",
        ));
    }
    value.decode().map_err(serde::de::Error::custom)
}
fn present_iterations<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<super::AnimationLoopIterations>, D::Error> {
    super::AnimationLoopIterations::deserialize(deserializer).map(Some)
}

impl AnimationPresetParameters {
    pub fn properties(&self) -> Vec<super::AnimationChannelProperty> {
        use super::AnimationChannelProperty as P;
        match self {
            Self::Scalar(parameters) => vec![parameters.property],
            Self::Pack(MotionPresetParameters::ImpactSlam { .. }) => {
                vec![P::PositionX, P::PositionY, P::ScaleX, P::ScaleY, P::Opacity]
            }
            Self::Pack(
                MotionPresetParameters::SlideLeft { .. } | MotionPresetParameters::Scan { .. },
            ) => vec![P::PositionX],
            Self::Pack(MotionPresetParameters::Pulse { .. }) => vec![P::ScaleX, P::ScaleY],
            Self::Pack(MotionPresetParameters::RadarExpand { .. }) => {
                vec![P::ScaleX, P::ScaleY, P::Opacity]
            }
        }
    }
    pub fn is_impact(&self) -> bool {
        matches!(self, Self::Pack(MotionPresetParameters::ImpactSlam { .. }))
    }
}

impl MotionPresetParameters {
    pub fn id(&self) -> &'static str {
        match self {
            Self::ImpactSlam { .. } => "impact_slam",
            Self::SlideLeft { .. } => "slide_left",
            Self::Scan { .. } => "scan",
            Self::Pulse { .. } => "pulse",
            Self::RadarExpand { .. } => "radar_expand",
        }
    }
    pub fn timing(&self) -> (u64, u64) {
        match self {
            Self::ImpactSlam {
                start_ms,
                duration_ms,
                ..
            }
            | Self::SlideLeft {
                start_ms,
                duration_ms,
                ..
            }
            | Self::Scan {
                start_ms,
                duration_ms,
                ..
            }
            | Self::Pulse {
                start_ms,
                duration_ms,
                ..
            }
            | Self::RadarExpand {
                start_ms,
                duration_ms,
                ..
            } => (*start_ms, *duration_ms),
        }
    }
    pub fn iterations(&self) -> Option<super::AnimationLoopIterations> {
        match self {
            Self::Scan { iterations, .. }
            | Self::Pulse { iterations, .. }
            | Self::RadarExpand { iterations, .. } => *iterations,
            _ => None,
        }
    }
    pub(crate) fn materialize_iterations(&mut self) {
        use super::{AnimationInfiniteIterations, AnimationLoopIterations};
        match self {
            Self::Scan { iterations, .. } | Self::RadarExpand { iterations, .. } => {
                iterations.get_or_insert(AnimationLoopIterations::Infinite(
                    AnimationInfiniteIterations::Infinite,
                ));
            }
            Self::Pulse { iterations, .. } => {
                iterations.get_or_insert(AnimationLoopIterations::Finite(1));
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPresetCollisionPolicy {
    #[default]
    Reject,
    Replace,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetProvenance {
    pub preset_id: String,
    pub preset_version: u32,
    pub compiler_version: u32,
    #[serde(deserialize_with = "effective_parameters")]
    pub parameters: AnimationPresetParameters,
}

#[derive(Deserialize)]
#[serde(
    remote = "AnimationPresetProvenance",
    rename_all = "camelCase",
    deny_unknown_fields
)]
struct AnimationPresetProvenanceDef {
    preset_id: String,
    preset_version: u32,
    compiler_version: u32,
    #[serde(deserialize_with = "effective_parameters")]
    parameters: AnimationPresetParameters,
}

impl<'de> Deserialize<'de> for AnimationPresetProvenance {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = BufferedValue::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom(
                "preset provenance requires an object",
            ));
        }
        value
            .deserialize_with(AnimationPresetProvenanceDef::deserialize)
            .map_err(serde::de::Error::custom)
    }
}

fn effective_parameters<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<AnimationPresetParameters, D::Error> {
    let value = BufferedValue::deserialize(deserializer)?;
    if value.get("kind").is_none() && value.get("curve").is_none() {
        return Err(serde::de::Error::custom(
            "persisted preset parameters require an effective curve",
        ));
    }
    let missing_iterations = value.get("iterations").is_none();
    let parameters: AnimationPresetParameters = value.decode().map_err(serde::de::Error::custom)?;
    if matches!(
        &parameters,
        AnimationPresetParameters::Pack(
            MotionPresetParameters::Scan { .. }
                | MotionPresetParameters::Pulse { .. }
                | MotionPresetParameters::RadarExpand { .. }
        )
    ) && missing_iterations
    {
        return Err(serde::de::Error::custom(
            "persisted looped preset parameters require effective iterations",
        ));
    }
    Ok(parameters)
}

pub(super) fn provenance_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<AnimationChannelProperty, AnimationPresetProvenance>, D::Error>
{
    struct SourceMap;
    impl<'de> serde::de::Visitor<'de> for SourceMap {
        type Value =
            std::collections::BTreeMap<AnimationChannelProperty, AnimationPresetProvenance>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an object with unique animation preset properties")
        }

        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut entries: M,
        ) -> Result<Self::Value, M::Error> {
            let mut sources = Self::Value::new();
            while let Some(property) = entries.next_key::<AnimationChannelProperty>()? {
                if sources.contains_key(&property) {
                    return Err(serde::de::Error::custom(
                        "duplicate animation preset provenance property",
                    ));
                }
                sources.insert(property, entries.next_value()?);
            }
            Ok(sources)
        }
    }
    deserializer.deserialize_map(SourceMap)
}
