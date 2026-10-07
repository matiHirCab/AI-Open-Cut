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
    ScreenFlash {
        id: String,
        start_ms: u32,
        duration_ms: u32,
        intensity: f64,
        color: crate::VectorColor,
    },
    ParticleOverlay {
        id: String,
        count: u16,
        seed: u32,
        radius_px: f64,
        speed_px_per_second: f64,
        lifetime_ms: u32,
        color: crate::VectorColor,
    },
}

impl VisualEffect {
    pub fn id(&self) -> &str {
        match self {
            Self::GaussianBlur { id, .. }
            | Self::Glow { id, .. }
            | Self::ColorTint { id, .. }
            | Self::Vignette { id, .. }
            | Self::ColorAdjustment { id, .. }
            | Self::ScreenFlash { id, .. }
            | Self::ParticleOverlay { id, .. } => id,
        }
    }
}

/// Explicit composition-local clipping, independent of temporal item spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompositionClip {
    CompositionBounds,
}

impl<'de> Deserialize<'de> for CompositionClip {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            #[serde(rename = "type")]
            kind: String,
        }
        struct Object;
        impl<'de> serde::de::Visitor<'de> for Object {
            type Value = Wire;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a composition clip object with named fields")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Wire, A::Error> {
                Wire::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }
        deserializer
            .deserialize_map(Object)
            .and_then(|wire| match wire.kind.as_str() {
                "composition_bounds" => Ok(Self::CompositionBounds),
                _ => Err(serde::de::Error::custom(
                    "clip type must be the string composition_bounds",
                )),
            })
            .map_err(|error| {
                serde::de::Error::custom(format!(
                    "{}{error}",
                    crate::error::GROUP_COMPOSITING_DECODE_ERROR_PREFIX
                ))
            })
    }
}

#[cfg(test)]
mod clip_tests {
    use super::*;
    use crate::{CoreError, ErrorCode};
    use serde_json::json;
    #[test]
    fn closed_clip_classifies_only_its_own_decode_errors() {
        for value in [
            json!(null),
            json!({}),
            json!({"type":"unknown"}),
            json!({"type":"composition_bounds","extra":1}),
            json!([]),
            json!(["composition_bounds"]),
            json!(["composition_bounds", "extra"]),
            json!({"type":{"composition_bounds":null}}),
            json!({"type":["composition_bounds"]}),
            json!({"type":null}),
            json!(true),
            json!(42),
        ] {
            for error in [
                serde_json::from_value::<CompositionClip>(value.clone()).unwrap_err(),
                serde_json::from_str::<CompositionClip>(&value.to_string()).unwrap_err(),
            ] {
                let error = CoreError::from(error);
                assert_eq!(error.code, ErrorCode::InvalidArgument);
                assert!(!error.retryable);
            }
        }
        let duplicate = r#"{"type":"composition_bounds","type":"composition_bounds"}"#;
        let error =
            CoreError::from(serde_json::from_str::<CompositionClip>(duplicate).unwrap_err());
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(
            serde_json::from_value::<CompositionClip>(json!({"type":"composition_bounds"}))
                .unwrap(),
            CompositionClip::CompositionBounds
        );
        for value in [
            json!(null),
            json!({"clip":null}),
            json!({"clip":{"type":"composition_bounds","extra":true}}),
        ] {
            if value.is_null() {
                continue;
            }
            assert_eq!(
                CoreError::from(
                    serde_json::from_value::<crate::VisualProperties>(value).unwrap_err()
                )
                .code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            serde_json::from_value::<crate::VisualProperties>(json!({}))
                .unwrap()
                .clip,
            None
        );
        let clear: crate::EditOperation =
            serde_json::from_value(json!({"operation":"update_item","itemId":"g","clip":null}))
                .unwrap();
        assert_eq!(serde_json::to_value(clear).unwrap()["clip"], json!(null));
        use serde::de::Error;
        for message in [
            "invalid group compositing metadata",
            "user field \u{1e}OPENCUT_GROUP_COMPOSITING_DECODE: bad clip",
        ] {
            assert_eq!(
                CoreError::from(serde_json::Error::custom(message)).code,
                ErrorCode::InternalError
            );
        }
    }
}
