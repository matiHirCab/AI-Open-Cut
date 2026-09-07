//! Strict, resource-free procedural grid records. Validation belongs to validation/grid.
use crate::{Paint, Stroke};
use serde::{Deserialize, Serialize};

pub const MAX_GRID_MARKS: usize = 4096;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GridDescriptor {
    pub width: f64,
    pub height: f64,
    pub pattern: GridPattern,
}

impl<'de> Deserialize<'de> for GridDescriptor {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            width: f64,
            height: f64,
            pattern: GridPattern,
        }
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Fields;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a grid descriptor object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Fields, A::Error> {
                Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))
            }
        }
        let v = d.deserialize_map(Visitor)?;
        Ok(Self {
            width: v.width,
            height: v.height,
            pattern: v.pattern,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum GridPattern {
    Rectangular {
        spacing_x: f64,
        spacing_y: f64,
        stroke: Stroke,
    },
    Diagonal {
        spacing: f64,
        stroke: Stroke,
    },
    Dot {
        spacing_x: f64,
        spacing_y: f64,
        radius: f64,
        paint: Paint,
    },
    Isometric {
        spacing: f64,
        stroke: Stroke,
    },
}
