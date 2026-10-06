//! Closed scoped visual matte metadata, introduced by schema34.
use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Deserializer, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatteChannel {
    Alpha,
    Luma,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MatteReference {
    pub source_id: String,
    pub channel: MatteChannel,
}
impl<'de> Deserialize<'de> for MatteReference {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            source_id: String,
            channel: MatteChannel,
        }
        struct Object;
        impl<'de> serde::de::Visitor<'de> for Object {
            type Value = Fields;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a matte object with named fields")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, a: A) -> Result<Fields, A::Error> {
                Fields::deserialize(serde::de::value::MapAccessDeserializer::new(a))
            }
        }
        let fields = d.deserialize_map(Object)?;
        Ok(Self {
            source_id: fields.source_id,
            channel: fields.channel,
        })
    }
}
impl MatteReference {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.source_id.is_empty() || self.source_id.len() > 128 {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "matte sourceId requires1 through128 UTF-8 bytes",
            ));
        }
        Ok(())
    }
}
