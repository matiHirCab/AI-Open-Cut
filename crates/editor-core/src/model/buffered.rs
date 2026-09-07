//! Lossless object-entry buffering for typed decoding after legacy preprocessing.
use serde::de::{self, DeserializeOwned, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub(super) enum Node {
    Scalar(Value),
    Array(Vec<Node>),
    Object(Vec<(String, Node)>),
}

impl Node {
    fn value(&self) -> Value {
        match self {
            Self::Scalar(v) => v.clone(),
            Self::Array(v) => Value::Array(v.iter().map(Self::value).collect()),
            Self::Object(v) => {
                Value::Object(v.iter().map(|(k, v)| (k.clone(), v.value())).collect())
            }
        }
    }

    /// Apply only migration edits; unchanged nested objects retain all entries.
    fn reconcile(self, value: &Value) -> Self {
        if self.value() == *value {
            return self;
        }
        match (self, value) {
            (Self::Array(nodes), Value::Array(values)) if nodes.len() == values.len() => {
                Self::Array(
                    nodes
                        .into_iter()
                        .zip(values)
                        .map(|(n, v)| n.reconcile(v))
                        .collect(),
                )
            }
            (Self::Object(nodes), Value::Object(values)) => {
                let mut result = Vec::new();
                for (k, n) in nodes {
                    if let Some(v) = values.get(&k) {
                        result.push((k, n.reconcile(v)));
                    }
                }
                for (k, v) in values {
                    if !result.iter().any(|(key, _)| key == k) {
                        result.push((k.clone(), Self::from(v.clone())));
                    }
                }
                Self::Object(result)
            }
            (_, v) => Self::from(v.clone()),
        }
    }
}

impl From<Value> for Node {
    fn from(v: Value) -> Self {
        match v {
            Value::Array(v) => Self::Array(v.into_iter().map(Self::from).collect()),
            Value::Object(v) => {
                Self::Object(v.into_iter().map(|(k, v)| (k, Self::from(v))).collect())
            }
            v => Self::Scalar(v),
        }
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct NodeVisitor;
        impl<'de> Visitor<'de> for NodeVisitor {
            type Value = Node;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Node, E> {
                Ok(Node::Scalar(Value::Null))
            }
            fn visit_none<E: de::Error>(self) -> Result<Node, E> {
                self.visit_unit()
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Node, E> {
                Ok(Node::Scalar(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Node, E> {
                Ok(Node::Scalar(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Node, E> {
                Ok(Node::Scalar(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Node, E> {
                serde_json::Number::from_f64(v)
                    .map(|v| Node::Scalar(Value::Number(v)))
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Node, E> {
                self.visit_string(v.into())
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Node, E> {
                Ok(Node::Scalar(v.into()))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Node, A::Error> {
                let mut v = Vec::new();
                while let Some(n) = a.next_element()? {
                    v.push(n);
                }
                Ok(Node::Array(v))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Node, A::Error> {
                let mut v = Vec::new();
                while let Some(n) = a.next_entry()? {
                    v.push(n);
                }
                Ok(Node::Object(v))
            }
        }
        d.deserialize_any(NodeVisitor)
    }
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for Node {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for Node {
    type Error = serde_json::Error;
    fn deserialize_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Scalar(s) => s.deserialize_any(v),
            Self::Array(a) => v.visit_seq(de::value::SeqDeserializer::new(a.into_iter())),
            Self::Object(o) => v.visit_map(de::value::MapDeserializer::new(o.into_iter())),
        }
    }
    fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        if matches!(self, Self::Scalar(Value::Null)) {
            v.visit_none()
        } else {
            v.visit_some(self)
        }
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Self::Error> {
        v.visit_newtype_struct(self)
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            Self::Scalar(s) => s.deserialize_enum(name, variants, v),
            Self::Object(o) => {
                de::value::MapDeserializer::new(o.into_iter()).deserialize_enum(name, variants, v)
            }
            _ => Err(de::Error::custom("expected enum")),
        }
    }
    serde::forward_to_deserialize_any! { bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier ignored_any }
}

/// A compatibility inspection view plus original entries for typed replay.
#[derive(Debug)]
pub(super) struct BufferedValue {
    node: Node,
    value: Value,
}
impl<'de> Deserialize<'de> for BufferedValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let node = Node::deserialize(d)?;
        Ok(Self {
            value: node.value(),
            node,
        })
    }
}
impl From<Value> for BufferedValue {
    fn from(value: Value) -> Self {
        Self {
            node: Node::from(value.clone()),
            value,
        }
    }
}
impl Deref for BufferedValue {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.value
    }
}
impl DerefMut for BufferedValue {
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.value
    }
}
impl BufferedValue {
    pub(super) fn deserialize_with<T>(
        self,
        decode: impl FnOnce(Node) -> Result<T, serde_json::Error>,
    ) -> Result<T, serde_json::Error> {
        decode(self.node.reconcile(&self.value))
    }
    pub(super) fn decode<T: DeserializeOwned>(self) -> Result<T, serde_json::Error> {
        T::deserialize(self.node.reconcile(&self.value))
    }
}
