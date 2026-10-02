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
pub struct AnimationPresetParameters {
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
    remote = "AnimationPresetParameters",
    rename_all = "camelCase",
    deny_unknown_fields
)]
struct AnimationPresetParametersDef {
    property: AnimationChannelProperty,
    start_ms: u64,
    duration_ms: u64,
    from: f64,
    to: f64,
    #[serde(default = "linear")]
    curve: AnimationCurve,
}

impl<'de> Deserialize<'de> for AnimationPresetParameters {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = BufferedValue::deserialize(deserializer)?;
        if !value.is_object() {
            return Err(serde::de::Error::custom(
                "preset parameters require an object",
            ));
        }
        value
            .deserialize_with(AnimationPresetParametersDef::deserialize)
            .map_err(serde::de::Error::custom)
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
    if value.get("curve").is_none() {
        return Err(serde::de::Error::custom(
            "persisted preset parameters require an effective curve",
        ));
    }
    value.decode().map_err(serde::de::Error::custom)
}
