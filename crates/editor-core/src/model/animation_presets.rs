use serde::{Deserialize, Serialize};

use super::{AnimationChannelProperty, AnimationCurve, SimpleAnimationCurve};

pub const ANIMATION_PRESET_COMPILER_VERSION: u32 = 1;
pub const MAX_PRESET_TIME_MS: u64 = 9_007_199_254_740_991;

fn linear() -> AnimationCurve {
    AnimationCurve::Simple(SimpleAnimationCurve::Linear)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPresetCollisionPolicy {
    #[default]
    Reject,
    Replace,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetProvenance {
    pub preset_id: String,
    pub preset_version: u32,
    pub compiler_version: u32,
    #[serde(deserialize_with = "effective_parameters")]
    pub parameters: AnimationPresetParameters,
}

fn effective_parameters<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<AnimationPresetParameters, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.get("curve").is_none() {
        return Err(serde::de::Error::custom(
            "persisted preset parameters require an effective curve",
        ));
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}
