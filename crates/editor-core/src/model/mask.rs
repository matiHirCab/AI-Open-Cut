//! Bounded authored mask metadata. Schema-33 activates bounded owner-local mask rendering and animation.
use crate::{
    CoreError, ErrorCode, Paint, PositionUnit, Transform2D, TransformAnchor, TransformPosition,
    VectorPath,
};
use serde::{Deserialize, Deserializer, Serialize};

pub const MAX_MASKS_PER_ITEM: usize = 16;
pub const MAX_MASKS_PER_COMPOSITION: usize = 4096;
pub const MAX_MASKS_PER_PROJECT: usize = 16384;
pub const MAX_MASK_COMMANDS_PER_COMPOSITION: usize = 65536;
pub const MAX_MASK_COMMANDS_PER_PROJECT: usize = 262144;

// Preserve streaming duplicate detection while disallowing positional records.
fn object<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    struct Visitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = T;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an object with named fields")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, a: A) -> Result<T, A::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(a))
        }
    }
    d.deserialize_map(Visitor(std::marker::PhantomData))
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mask {
    pub id: String,
    pub source: MaskSource,
    pub channel: MaskChannel,
    pub operation: MaskOperation,
    pub inverted: bool,
    pub transform: Transform2D,
    pub feather_px: f64,
    pub expansion_px: f64,
}

impl<'de> Deserialize<'de> for Mask {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            id: String,
            source: MaskSource,
            channel: MaskChannel,
            operation: MaskOperation,
            inverted: bool,
            #[serde(deserialize_with = "transform")]
            transform: Transform2D,
            feather_px: f64,
            expansion_px: f64,
        }
        let v: Fields = object(d)?;
        Ok(Self {
            id: v.id,
            source: v.source,
            channel: v.channel,
            operation: v.operation,
            inverted: v.inverted,
            transform: v.transform,
            feather_px: v.feather_px,
            expansion_px: v.expansion_px,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum MaskSource {
    Path { path: VectorPath, paint: Paint },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskChannel {
    Alpha,
    Luma,
}
impl<'de> Deserialize<'de> for MaskChannel {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "alpha" => Ok(Self::Alpha),
            "luma" => Ok(Self::Luma),
            _ => Err(serde::de::Error::custom(
                "mask channel must be alpha or luma",
            )),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskOperation {
    Add,
    Subtract,
    Intersect,
    Exclude,
}
impl<'de> Deserialize<'de> for MaskOperation {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "add" => Ok(Self::Add),
            "subtract" => Ok(Self::Subtract),
            "intersect" => Ok(Self::Intersect),
            "exclude" => Ok(Self::Exclude),
            _ => Err(serde::de::Error::custom("unsupported mask operation")),
        }
    }
}

fn position<'de, D: Deserializer<'de>>(d: D) -> Result<TransformPosition, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fields {
        x: f64,
        y: f64,
        unit: String,
    }
    let v: Fields = object(d)?;
    let unit = match v.unit.as_str() {
        "pixels" => PositionUnit::Pixels,
        "normalized" => PositionUnit::Normalized,
        _ => return Err(serde::de::Error::custom("invalid mask position unit")),
    };
    Ok(TransformPosition {
        x: v.x,
        y: v.y,
        unit,
    })
}
fn anchor<'de, D: Deserializer<'de>>(d: D) -> Result<TransformAnchor, D::Error> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fields {
        x: f64,
        y: f64,
    }
    let v: Fields = object(d)?;
    Ok(TransformAnchor { x: v.x, y: v.y })
}
fn transform<'de, D: Deserializer<'de>>(d: D) -> Result<Transform2D, D::Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Fields {
        #[serde(deserialize_with = "position")]
        position: TransformPosition,
        #[serde(deserialize_with = "anchor")]
        anchor: TransformAnchor,
        scale_x: f64,
        scale_y: f64,
        rotation_deg: f64,
        skew_x_deg: f64,
        skew_y_deg: f64,
        opacity: f64,
    }
    let v: Fields = object(d)?;
    Ok(Transform2D {
        position: v.position,
        anchor: v.anchor,
        scale_x: v.scale_x,
        scale_y: v.scale_y,
        rotation_deg: v.rotation_deg,
        skew_x_deg: v.skew_x_deg,
        skew_y_deg: v.skew_y_deg,
        opacity: v.opacity,
    })
}
impl Mask {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.is_empty()
            || self.id.len() > 128
            || !self.feather_px.is_finite()
            || !(0.0..=128.0).contains(&self.feather_px)
            || !self.expansion_px.is_finite()
            || !(-128.0..=128.0).contains(&self.expansion_px)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "mask identifier or local pixel parameter exceeds bounds",
            ));
        }
        let MaskSource::Path { path, paint } = &self.source;
        path.validate()?;
        paint.validate()?;
        self.transform.validate()
    }
    pub fn command_count(&self) -> usize {
        let MaskSource::Path { path, .. } = &self.source;
        path.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_catalog_schema_markers_match_current_core() {
        for source in [
            include_str!("../../../../contracts/animation-channels-v1.json"),
            include_str!("../../../../contracts/animation-presets-v1.json"),
            include_str!("../../../../contracts/extended-visual-animation-v1.json"),
            include_str!("../../../../contracts/inherited-animation-timing-v1.json"),
            include_str!("../../../../contracts/motion-blur-sampling-v1.json"),
            include_str!("../../../../contracts/initial-motion-preset-pack-v1.json"),
            include_str!("../../../../contracts/mask-models-v1.json"),
        ] {
            let catalog: serde_json::Value = serde_json::from_str(source).unwrap();
            assert_eq!(
                catalog["projectSchemaVersion"],
                crate::PROJECT_SCHEMA_VERSION
            );
        }
    }

    #[test]
    fn mask_model_matches_canonical_cases_and_raw_representation() {
        let c: serde_json::Value =
            serde_json::from_str(include_str!("../../../../contracts/mask-models-v1.json"))
                .unwrap();
        for case in c["cases"].as_array().unwrap() {
            let value = case["value"].clone();
            let accepted =
                serde_json::from_value::<Mask>(value.clone()).is_ok_and(|m| m.validate().is_ok());
            assert_eq!(
                accepted,
                case["accepted"].as_bool().unwrap(),
                "{}",
                case["name"]
            );
            if accepted {
                let authored = serde_json::from_value::<Mask>(value).unwrap();
                let round_trip =
                    serde_json::from_value::<Mask>(serde_json::to_value(&authored).unwrap())
                        .unwrap();
                assert_eq!(round_trip, authored);
            }
        }
        for case in c["rawRejectedCases"].as_array().unwrap() {
            assert!(
                serde_json::from_str::<Mask>(case["json"].as_str().unwrap()).is_err(),
                "{}",
                case["name"]
            );
        }
    }
}

#[cfg(test)]
mod activation_wire_tests {
    #[test]
    fn approved_mask_animation_wire_is_closed_and_decodes() {
        let value = serde_json::json!({"property":"mask.transform.opacity","target":{"kind":"mask","scope":"root","id":"reveal"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.25},"curve":"linear"}]});
        let channel = serde_json::from_value::<crate::AnimationChannel>(value);
        assert!(
            channel.is_ok(),
            "approved mask target/property must decode: {channel:?}"
        );
    }
}
