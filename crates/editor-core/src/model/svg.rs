//! Reference-free normalized SVG document. Source parsing belongs to validation.
use crate::{Paint, ShapeGeometry, Stroke, VectorPoint};
use serde::{Deserialize, Serialize};

pub const MAX_SVG_BYTES: usize = 1_048_576;
pub const MAX_SVG_DEPTH: usize = 32;
pub const MAX_SVG_ELEMENTS: usize = 4096;
pub const MAX_SVG_ATTRIBUTES: usize = 32;
pub const MAX_SVG_COMMANDS: usize = 65536;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SvgDocument {
    pub version: u32,
    pub width: f64,
    pub height: f64,
    pub view_box: [f64; 4],
    pub shapes: Vec<SvgShape>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SvgShape {
    pub geometry: ShapeGeometry,
    pub offset: VectorPoint,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke>,
}

fn object<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    struct Visitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = T;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a normalized SVG object")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    d.deserialize_map(Visitor(std::marker::PhantomData))
}
impl<'de> Deserialize<'de> for SvgDocument {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            version: u32,
            width: f64,
            height: f64,
            view_box: [f64; 4],
            shapes: Vec<SvgShape>,
        }
        let v: Fields = object(d)?;
        Ok(Self {
            version: v.version,
            width: v.width,
            height: v.height,
            view_box: v.view_box,
            shapes: v.shapes,
        })
    }
}
impl<'de> Deserialize<'de> for SvgShape {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            geometry: ShapeGeometry,
            offset: VectorPoint,
            #[serde(deserialize_with = "super::shape::required_nullable")]
            fill: Option<Paint>,
            #[serde(deserialize_with = "super::shape::required_nullable")]
            stroke: Option<Stroke>,
        }
        let v: Fields = object(d)?;
        Ok(Self {
            geometry: v.geometry,
            offset: v.offset,
            fill: v.fill,
            stroke: v.stroke,
        })
    }
}
