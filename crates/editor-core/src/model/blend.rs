//! Closed, static destination blend selection; source assembly is unchanged.
use serde::{Deserialize, Deserializer, Serialize, de::Error};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Darken,
    Lighten,
}

impl BlendMode {
    pub fn is_normal(&self) -> bool {
        *self == Self::Normal
    }
}

impl<'de> Deserialize<'de> for BlendMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer).map_err(|error| {
            D::Error::custom(format!(
                "{}{error}",
                crate::error::BLEND_DECODE_ERROR_PREFIX
            ))
        })?;
        match value.as_str() {
            "normal" => Ok(Self::Normal),
            "multiply" => Ok(Self::Multiply),
            "screen" => Ok(Self::Screen),
            "overlay" => Ok(Self::Overlay),
            "add" => Ok(Self::Add),
            "darken" => Ok(Self::Darken),
            "lighten" => Ok(Self::Lighten),
            _ => Err(D::Error::custom(format!(
                "{}unknown blend mode {value}",
                crate::error::BLEND_DECODE_ERROR_PREFIX
            ))),
        }
    }
}
