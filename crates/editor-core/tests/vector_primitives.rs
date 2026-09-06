use opencut_editor_core::{
    CoreError, CornerRadii, ErrorCode, FillRule, GradientStop, LineCap, LineJoin,
    MAX_VECTOR_PATH_COMMANDS, Paint, PathCommand, Stroke, VectorColor, VectorPath, VectorPoint,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::collections::BTreeSet;

// Force the map visitor even when a format also supports positional struct sequences.
// Streaming MapAccess preserves duplicate fields for the derived Fields decoder.
fn deserialize_object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for ObjectVisitor<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an object with named fields")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(ObjectVisitor(std::marker::PhantomData))
}

struct Catalog {
    version: u32,
    activation: String,
    limits: Value,
    identifiers: Value,
    fixtures: Vec<Fixture>,
}
impl<'de> Deserialize<'de> for Catalog {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            version: u32,
            activation: String,
            #[serde(deserialize_with = "required_value")]
            limits: Value,
            #[serde(deserialize_with = "required_value")]
            identifiers: Value,
            fixtures: Vec<Fixture>,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            version: fields.version,
            activation: fields.activation,
            limits: fields.limits,
            identifiers: fields.identifiers,
            fixtures: fields.fixtures,
        })
    }
}

// A deserialize_with field remains required even when its payload is arbitrary JSON.
fn required_value<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Value, D::Error> {
    Value::deserialize(deserializer)
}

struct Fixture {
    id: String,
    kind: Kind,
    value: Value,
    valid: bool,
    structural_invalid: bool,
}
impl<'de> Deserialize<'de> for Fixture {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            id: String,
            kind: Kind,
            #[serde(deserialize_with = "required_value")]
            value: Value,
            valid: bool,
            #[serde(default)]
            structural_invalid: bool,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            id: fields.id,
            kind: fields.kind,
            value: fields.value,
            valid: fields.valid,
            structural_invalid: fields.structural_invalid,
        })
    }
}

enum Kind {
    Color,
    Point,
    Paint,
    Stroke,
    CornerRadii,
    Path,
    GradientStop,
    LineCap,
    LineJoin,
    FillRule,
    PathCommand,
}
impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "color" => Ok(Self::Color),
            "point" => Ok(Self::Point),
            "paint" => Ok(Self::Paint),
            "stroke" => Ok(Self::Stroke),
            "cornerRadii" => Ok(Self::CornerRadii),
            "path" => Ok(Self::Path),
            "gradientStop" => Ok(Self::GradientStop),
            "lineCap" => Ok(Self::LineCap),
            "lineJoin" => Ok(Self::LineJoin),
            "fillRule" => Ok(Self::FillRule),
            "pathCommand" => Ok(Self::PathCommand),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &[
                    "color",
                    "point",
                    "paint",
                    "stroke",
                    "cornerRadii",
                    "path",
                    "gradientStop",
                    "lineCap",
                    "lineJoin",
                    "fillRule",
                    "pathCommand",
                ],
            )),
        }
    }
}

fn check<T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug>(
    value: &Value,
    validate: impl Fn(&T) -> Result<(), CoreError>,
) -> bool {
    let decoded = serde_json::from_value::<T>(value.clone());
    let raw_decoded = serde_json::from_str::<T>(&value.to_string());
    assert_eq!(
        decoded.is_ok(),
        raw_decoded.is_ok(),
        "raw/Value structural parity"
    );
    let Ok(decoded) = decoded else {
        return false;
    };
    assert_eq!(decoded, raw_decoded.unwrap());
    if let Err(error) = validate(&decoded) {
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        return false;
    }
    let wire = serde_json::to_value(&decoded).unwrap();
    assert_eq!(numeric_json(&wire), numeric_json(value));
    assert_eq!(serde_json::from_value::<T>(wire).unwrap(), decoded);
    assert_eq!(
        serde_json::from_str::<T>(&serde_json::to_string(value).unwrap()).unwrap(),
        decoded
    );
    true
}

// JSON distinguishes integer/floating number storage, but the wire contract does not.
fn numeric_json(value: &Value) -> Value {
    match value {
        Value::Number(n) => json!(n.as_f64().unwrap()),
        Value::Array(values) => Value::Array(values.iter().map(numeric_json).collect()),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(k, v)| (k.clone(), numeric_json(v)))
                .collect(),
        ),
        value => value.clone(),
    }
}

fn validate_catalog(value: Value) -> Result<Catalog, String> {
    let catalog: Catalog = serde_json::from_value(value).map_err(|error| error.to_string())?;
    if catalog.version != 1
        || catalog.activation != "core_primitives_only"
        || catalog.limits
            != json!({
                "maxCoordinate": 1000000, "maxGradientStops": 64, "maxDashEntries": 64,
                "maxPathCommands": 4096, "maxDimension": 16384, "maxMiterLimit": 1000
            })
        || catalog.identifiers
            != json!({
                "paint": ["solid", "linearGradient", "radialGradient"],
                "lineCap": ["butt", "round", "square"], "lineJoin": ["miter", "round", "bevel"],
                "fillRule": ["nonzero", "evenodd"],
                "command": ["moveTo", "lineTo", "quadraticTo", "cubicTo", "close"]
            })
    {
        return Err("catalog metadata drift".into());
    }
    let mut ids = BTreeSet::new();
    for fixture in &catalog.fixtures {
        if fixture.id.is_empty() || !ids.insert(&fixture.id) {
            return Err("fixture identity is empty or duplicated".into());
        }
        if fixture.structural_invalid && fixture.valid {
            return Err("structural rejection cannot be a valid fixture".into());
        }
    }
    Ok(catalog)
}

#[test]
fn canonical_vector_primitives() {
    let catalog = validate_catalog(
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap(),
    )
    .unwrap();
    for fixture in catalog.fixtures {
        let accepted = match fixture.kind {
            Kind::Color => check::<VectorColor>(&fixture.value, VectorColor::validate),
            Kind::Point => check::<VectorPoint>(&fixture.value, VectorPoint::validate),
            Kind::Paint => check::<Paint>(&fixture.value, Paint::validate),
            Kind::Stroke => check::<Stroke>(&fixture.value, Stroke::validate),
            Kind::CornerRadii => check::<CornerRadii>(&fixture.value, CornerRadii::validate),
            Kind::Path => check::<VectorPath>(&fixture.value, VectorPath::validate),
            Kind::GradientStop => check::<GradientStop>(&fixture.value, GradientStop::validate),
            Kind::LineCap => check::<LineCap>(&fixture.value, |_| Ok(())),
            Kind::LineJoin => check::<LineJoin>(&fixture.value, |_| Ok(())),
            Kind::FillRule => check::<FillRule>(&fixture.value, |_| Ok(())),
            Kind::PathCommand => check::<PathCommand>(&fixture.value, |_| Ok(())),
        };
        assert_eq!(accepted, fixture.valid, "{}", fixture.id);
    }
}

#[test]
fn rejects_malformed_catalog_wrappers() {
    let source: Value =
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap();
    for pointer in ["", "/fixtures/0", "/limits", "/identifiers"] {
        let mut v = source.clone();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(1));
        assert!(validate_catalog(v).is_err());
    }
    for field in ["version", "activation", "limits", "identifiers", "fixtures"] {
        let mut v = source.clone();
        v.as_object_mut().unwrap().remove(field);
        assert!(validate_catalog(v).is_err());
    }
    for (pointer, replacement) in [
        ("/version", json!(2)),
        ("/activation", json!("runtime")),
        ("/limits/maxPathCommands", json!(4097)),
        ("/identifiers/command", json!(["arcTo"])),
        ("/fixtures/0/kind", json!("svg")),
        ("/fixtures/0/id", json!("")),
    ] {
        let mut v = source.clone();
        *v.pointer_mut(pointer).unwrap() = replacement;
        assert!(validate_catalog(v).is_err());
    }
    let mut duplicate = source.clone();
    duplicate["fixtures"]
        .as_array_mut()
        .unwrap()
        .push(source["fixtures"][0].clone());
    assert!(validate_catalog(duplicate).is_err());
}

fn rejects_both_decoders<T: DeserializeOwned>(value: &Value) {
    assert!(
        serde_json::from_value::<T>(value.clone()).is_err(),
        "Value decoder accepted {value}"
    );
    assert!(
        serde_json::from_str::<T>(&value.to_string()).is_err(),
        "raw decoder accepted {value}"
    );
}

#[test]
fn structural_fixtures_fail_before_semantic_validation() {
    let catalog = validate_catalog(
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap(),
    )
    .unwrap();
    for fixture in catalog.fixtures.iter().filter(|f| f.structural_invalid) {
        assert!(!fixture.valid);
        match fixture.kind {
            Kind::Color => rejects_both_decoders::<VectorColor>(&fixture.value),
            Kind::Point => rejects_both_decoders::<VectorPoint>(&fixture.value),
            Kind::Paint => rejects_both_decoders::<Paint>(&fixture.value),
            Kind::Stroke => rejects_both_decoders::<Stroke>(&fixture.value),
            Kind::CornerRadii => rejects_both_decoders::<CornerRadii>(&fixture.value),
            Kind::Path => rejects_both_decoders::<VectorPath>(&fixture.value),
            Kind::GradientStop => rejects_both_decoders::<GradientStop>(&fixture.value),
            Kind::LineCap => rejects_both_decoders::<LineCap>(&fixture.value),
            Kind::LineJoin => rejects_both_decoders::<LineJoin>(&fixture.value),
            Kind::FillRule => rejects_both_decoders::<FillRule>(&fixture.value),
            Kind::PathCommand => rejects_both_decoders::<PathCommand>(&fixture.value),
        }
    }
}

#[test]
fn catalog_wrappers_require_objects_and_string_kinds() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap();
    let fixture =
        json!({"id":"record","kind":"color","value":{"r":1,"g":0,"b":0,"a":1},"valid":true});
    rejects_both_decoders::<Catalog>(&json!([
        1,
        "core_primitives_only",
        catalog["limits"],
        catalog["identifiers"],
        catalog["fixtures"]
    ]));
    rejects_both_decoders::<Fixture>(&json!(["record", "color", fixture["value"], true, false]));
    rejects_both_decoders::<Kind>(&json!({"color":null}));
    let mut object_kind = fixture.clone();
    object_kind["kind"] = json!({"color":null});
    rejects_both_decoders::<Fixture>(&object_kind);
    for field in ["id", "kind", "value", "valid"] {
        let mut value = fixture.clone();
        value.as_object_mut().unwrap().remove(field);
        rejects_both_decoders::<Fixture>(&value);
    }
    for raw in [
        r#"{"id":"a","id":"b","kind":"color","value":null,"valid":false}"#,
        r#"{"id":"a","kind":"color","value":null,"value":0,"valid":false}"#,
    ] {
        assert!(serde_json::from_str::<Fixture>(raw).is_err());
    }
}

// Emit reverse key order directly, avoiding serde_json::Map's canonical key sorting.
fn reversed_json(value: &Value) -> String {
    match value {
        Value::Object(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .rev()
                .map(|(key, value)| format!("{}:{}", json!(key), reversed_json(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(reversed_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        value => value.to_string(),
    }
}

// Mutate one object at a time, including nested objects, retaining duplicate keys in raw JSON.
fn duplicate_field_json(value: &Value) -> Vec<String> {
    match value {
        Value::Object(fields) => {
            let mut result = Vec::new();
            for (key, child) in fields {
                let entry = format!("{}:{}", json!(key), child);
                result.push(format!(
                    "{{{entry},{}}}",
                    &value.to_string()[1..value.to_string().len() - 1]
                ));
                for nested in duplicate_field_json(child) {
                    result.push(format!(
                        "{{{}}}",
                        fields
                            .iter()
                            .map(|(other, v)| format!(
                                "{}:{}",
                                json!(other),
                                if other == key {
                                    nested.clone()
                                } else {
                                    v.to_string()
                                }
                            ))
                            .collect::<Vec<_>>()
                            .join(",")
                    ));
                }
            }
            result
        }
        Value::Array(values) => values
            .iter()
            .enumerate()
            .flat_map(|(index, child)| {
                duplicate_field_json(child).into_iter().map(move |nested| {
                    format!(
                        "[{}]",
                        values
                            .iter()
                            .enumerate()
                            .map(|(i, v)| if i == index {
                                nested.clone()
                            } else {
                                v.to_string()
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                })
            })
            .collect(),
        _ => vec![],
    }
}

fn strict_roundtrip<T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug>(value: &Value) {
    let canonical: T = serde_json::from_value(value.clone()).unwrap();
    let reordered: T = serde_json::from_str(&reversed_json(value)).unwrap();
    assert_eq!(canonical, reordered);
    assert_eq!(
        numeric_json(&serde_json::to_value(reordered).unwrap()),
        numeric_json(value)
    );
    for duplicate in duplicate_field_json(value) {
        assert!(
            serde_json::from_str::<T>(&duplicate).is_err(),
            "accepted duplicate: {duplicate}"
        );
    }
}

#[test]
fn reordered_objects_and_raw_duplicate_fields_at_all_depths() {
    let catalog = validate_catalog(
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap(),
    )
    .unwrap();
    for fixture in catalog.fixtures.iter().filter(|f| f.valid) {
        match fixture.kind {
            Kind::Color => strict_roundtrip::<VectorColor>(&fixture.value),
            Kind::Point => strict_roundtrip::<VectorPoint>(&fixture.value),
            Kind::Paint => strict_roundtrip::<Paint>(&fixture.value),
            Kind::Stroke => strict_roundtrip::<Stroke>(&fixture.value),
            Kind::CornerRadii => strict_roundtrip::<CornerRadii>(&fixture.value),
            Kind::Path => strict_roundtrip::<VectorPath>(&fixture.value),
            Kind::GradientStop => strict_roundtrip::<GradientStop>(&fixture.value),
            Kind::LineCap => strict_roundtrip::<LineCap>(&fixture.value),
            Kind::LineJoin => strict_roundtrip::<LineJoin>(&fixture.value),
            Kind::FillRule => strict_roundtrip::<FillRule>(&fixture.value),
            Kind::PathCommand => strict_roundtrip::<PathCommand>(&fixture.value),
        }
    }
}

#[test]
fn path_limit_and_preflight() {
    assert_eq!(MAX_VECTOR_PATH_COMMANDS, 4096);
    let p = VectorPoint { x: 0., y: 0. };
    let mut path = VectorPath {
        fill_rule: FillRule::Nonzero,
        commands: vec![PathCommand::MoveTo { to: p }; MAX_VECTOR_PATH_COMMANDS],
    };
    path.validate().unwrap();
    path.commands.push(PathCommand::Close {});
    assert!(path.validate().unwrap_err().message.contains("commands"));
}

#[test]
fn radii_use_one_independent_scale_and_preserve_input() {
    let radii = CornerRadii {
        top_left: 80.,
        top_right: 20.,
        bottom_right: 60.,
        bottom_left: 40.,
    };
    let before = radii;
    // Height / left sum = 60 / 120 = 0.5 is the limiting ratio.
    assert_eq!(
        radii.resolve(100., 60.).unwrap(),
        CornerRadii {
            top_left: 40.,
            top_right: 10.,
            bottom_right: 30.,
            bottom_left: 20.
        }
    );
    assert_eq!(radii, before);
    let zero = CornerRadii {
        top_left: 0.,
        top_right: 0.,
        bottom_right: 0.,
        bottom_left: 0.,
    };
    assert_eq!(zero.resolve(1., 1.).unwrap(), zero);
    assert_eq!(radii.resolve(16384., 16384.).unwrap(), radii);
    for n in [0., -1., 16385., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(radii.resolve(n, 1.).is_err());
        assert!(radii.resolve(1., n).is_err());
    }
}

#[test]
fn nonfinite_constructed_values_are_rejected_everywhere() {
    let source: Value =
        serde_json::from_str(include_str!("../../../contracts/vector-primitives-v1.json")).unwrap();
    let stroke: Stroke = serde_json::from_value(
        source["fixtures"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["id"] == "stroke-0")
            .unwrap()["value"]
            .clone(),
    )
    .unwrap();
    for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for channel in 0..4 {
            let mut c = VectorColor {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            };
            match channel {
                0 => c.r = n,
                1 => c.g = n,
                2 => c.b = n,
                _ => c.a = n,
            }
            assert!(c.validate().is_err());
        }
        for p in [VectorPoint { x: n, y: 0. }, VectorPoint { x: 0., y: n }] {
            assert!(p.validate().is_err());
            let path = VectorPath {
                fill_rule: FillRule::Evenodd,
                commands: vec![PathCommand::MoveTo { to: p }],
            };
            assert!(path.validate().is_err());
        }
        for field in 0..4 {
            let mut s = stroke.clone();
            match field {
                0 => s.width = n,
                1 => s.dash_offset = n,
                2 => s.miter_limit = n,
                _ => s.dash = vec![n, 1.],
            }
            assert!(s.validate().is_err());
            let mut r = CornerRadii {
                top_left: 0.,
                top_right: 0.,
                bottom_right: 0.,
                bottom_left: 0.,
            };
            match field {
                0 => r.top_left = n,
                1 => r.top_right = n,
                2 => r.bottom_right = n,
                _ => r.bottom_left = n,
            }
            assert!(r.validate().is_err());
        }
        let mut paint: Paint = serde_json::from_value(
            source["fixtures"]
                .as_array()
                .unwrap()
                .iter()
                .find(|f| f["id"] == "paint-1")
                .unwrap()["value"]
                .clone(),
        )
        .unwrap();
        if let Paint::LinearGradient { stops, .. } = &mut paint {
            stops[0].offset = n;
        }
        assert!(paint.validate().is_err());
    }
}

#[test]
fn nonfinite_nested_paints_and_all_command_coordinates() {
    let zero = VectorPoint { x: 0., y: 0. };
    let color = VectorColor {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
    let stops = vec![
        GradientStop { offset: 0., color },
        GradientStop { offset: 1., color },
    ];
    for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            Paint::RadialGradient {
                center: zero,
                radius: n,
                stops: stops.clone()
            }
            .validate()
            .is_err()
        );
        let bad_color = VectorColor { a: n, ..color };
        for paint in [
            Paint::Solid { color: bad_color },
            Paint::LinearGradient {
                start: zero,
                end: VectorPoint { x: 1., y: 0. },
                stops: vec![
                    GradientStop {
                        offset: 0.,
                        color: bad_color,
                    },
                    stops[1].clone(),
                ],
            },
        ] {
            assert!(paint.validate().is_err());
        }
        for bad in [VectorPoint { x: n, y: 0. }, VectorPoint { x: 0., y: n }] {
            for paint in [
                Paint::LinearGradient {
                    start: bad,
                    end: zero,
                    stops: stops.clone(),
                },
                Paint::LinearGradient {
                    start: zero,
                    end: bad,
                    stops: stops.clone(),
                },
                Paint::RadialGradient {
                    center: bad,
                    radius: 1.,
                    stops: stops.clone(),
                },
            ] {
                assert!(paint.validate().is_err());
            }
            for command in [
                PathCommand::MoveTo { to: bad },
                PathCommand::LineTo { to: bad },
                PathCommand::QuadraticTo {
                    control: bad,
                    to: zero,
                },
                PathCommand::QuadraticTo {
                    control: zero,
                    to: bad,
                },
                PathCommand::CubicTo {
                    control1: bad,
                    control2: zero,
                    to: zero,
                },
                PathCommand::CubicTo {
                    control1: zero,
                    control2: bad,
                    to: zero,
                },
                PathCommand::CubicTo {
                    control1: zero,
                    control2: zero,
                    to: bad,
                },
            ] {
                assert!(
                    VectorPath {
                        fill_rule: FillRule::Nonzero,
                        commands: vec![PathCommand::MoveTo { to: zero }, command]
                    }
                    .validate()
                    .is_err()
                );
            }
        }
    }
}

#[test]
fn radius_resolution_checks_each_limiting_side() {
    // Rotate one asymmetric shape to independently exercise all four limiting sums.
    let cases = [
        (
            CornerRadii {
                top_left: 80.,
                top_right: 40.,
                bottom_right: 20.,
                bottom_left: 60.,
            },
            60.,
            16384.,
        ),
        (
            CornerRadii {
                top_left: 20.,
                top_right: 60.,
                bottom_right: 80.,
                bottom_left: 40.,
            },
            60.,
            16384.,
        ),
        (
            CornerRadii {
                top_left: 80.,
                top_right: 20.,
                bottom_right: 60.,
                bottom_left: 40.,
            },
            16384.,
            60.,
        ),
        (
            CornerRadii {
                top_left: 20.,
                top_right: 80.,
                bottom_right: 40.,
                bottom_left: 60.,
            },
            16384.,
            60.,
        ),
    ];
    for (radii, w, h) in cases {
        assert_eq!(
            radii.resolve(w, h).unwrap(),
            CornerRadii {
                top_left: radii.top_left / 2.,
                top_right: radii.top_right / 2.,
                bottom_right: radii.bottom_right / 2.,
                bottom_left: radii.bottom_left / 2.,
            }
        );
    }
}
