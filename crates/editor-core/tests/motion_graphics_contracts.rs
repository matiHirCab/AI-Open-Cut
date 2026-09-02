use std::collections::BTreeSet;

use serde_json::{Map, Value};

#[path = "fixtures/motion_graphics_contract.rs"]
mod strict_contract;

const CATALOG_SOURCE: &str = include_str!("../../../contracts/motion-graphics-v1.json");
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

const CONCEPTS: &[&str] = &[
    "audio_event",
    "component",
    "curve",
    "effect",
    "layer",
    "marker",
    "mask",
    "slot",
    "transform",
];

const LIMITS: &[&str] = &[
    "maxAudioEventsPerComposition",
    "maxComponentDefinitions",
    "maxComponentDepth",
    "maxEffectsPerLayer",
    "maxKeyframesPerChannel",
    "maxLayersPerComposition",
    "maxMarkersPerComposition",
    "maxMasksPerLayer",
    "maxParentDepth",
    "maxSlotsPerComponent",
];

const FAILURE_CLASSIFICATIONS: &[&str] = &[
    "ambiguous_reference",
    "invalid_input",
    "missing_reference",
    "reference_cycle",
];

const IDENTIFIER_SETS: &[(&str, &[&str])] = &[
    ("positionUnits", &["normalized", "pixels"]),
    (
        "blendModes",
        &[
            "add", "darken", "lighten", "multiply", "normal", "overlay", "screen",
        ],
    ),
    (
        "slotKinds",
        &[
            "asset",
            "boolean",
            "color",
            "duration",
            "enum",
            "number",
            "rich_text",
            "text",
        ],
    ),
    ("timeExpressionTypes", &["marker", "milliseconds"]),
    ("curveTypes", &["cubic_bezier", "hold", "linear", "spring"]),
    ("maskSourceTypes", &["layer", "path"]),
    ("maskChannels", &["alpha", "luma"]),
    (
        "maskOperations",
        &["add", "exclude", "intersect", "subtract"],
    ),
    (
        "effectTypes",
        &[
            "color_adjustment",
            "color_tint",
            "directional_blur",
            "gaussian_blur",
            "glow",
            "particle_overlay",
            "screen_flash",
            "vignette",
        ],
    ),
    (
        "referenceKinds",
        &[
            "asset",
            "audio_bus",
            "audio_event",
            "component",
            "curve",
            "effect",
            "layer",
            "marker",
            "mask",
            "slot",
            "sound_definition",
            "transform",
        ],
    ),
];

const LAYER_ORDERING: &[&str] = &[
    "track_array_index_ascending",
    "z_index_ascending",
    "item_array_index_ascending",
    "item_id_ascending_final_tie_break",
];

const VISUAL_PIPELINE: &[&str] = &[
    "source",
    "crop_and_local_clip",
    "masks_in_declared_order",
    "effects_in_declared_order",
    "local_anchor_scale_skew_rotation_position",
    "ancestor_transforms_nearest_first",
    "track_matte",
    "inherited_opacity",
    "destination_blend",
];

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{key} must be an array"))
}

fn strings(value: &Value, label: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{label} must contain only strings"))
        })
        .collect()
}

fn reference_keys(value: &Value, label: &str) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let entry = object(entry, &format!("{label}[{index}]"))?;
            assert_closed_record(
                entry,
                &["id", "kind", "scope"],
                &format!("{label}[{index}]"),
            )?;
            let kind = entry
                .get("kind")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{label}[{index}].kind must be a string"))?;
            let scope = entry
                .get("scope")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{label}[{index}].scope must be a string"))?;
            let id = entry
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{label}[{index}].id must be a string"))?;
            Ok(format!("{scope}|{kind}|{id}"))
        })
        .collect()
}

fn assert_closed_record(
    record: &Map<String, Value>,
    expected: &[&str],
    label: &str,
) -> Result<(), String> {
    let actual = record.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{label} fields differ: expected {expected:?}, found {actual:?}"
        ))
    }
}

fn validate_finite_values(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::Null | Value::Bool(_) => Ok(()),
        Value::Number(number) => {
            let finite = number
                .as_f64()
                .is_some_and(|candidate| candidate.is_finite());
            if finite {
                Ok(())
            } else {
                Err(format!("{path} contains a non-finite number"))
            }
        }
        Value::String(_) => Ok(()),
        Value::Array(entries) => entries.iter().enumerate().try_for_each(|(index, entry)| {
            validate_finite_values(entry, &format!("{path}[{index}]"))
        }),
        Value::Object(entries) => entries
            .iter()
            .try_for_each(|(key, entry)| validate_finite_values(entry, &format!("{path}.{key}"))),
    }
}

fn validate_limit_value(name: &str, value: &Value) -> Result<(), String> {
    if value
        .as_u64()
        .is_some_and(|limit| (1..=MAX_SAFE_INTEGER).contains(&limit))
    {
        Ok(())
    } else {
        Err(format!("{name} must be a positive JavaScript-safe integer"))
    }
}

fn validate_catalog(catalog: &Value) -> Result<(), String> {
    let root = object(catalog, "catalog")?;
    assert_closed_record(
        root,
        &[
            "contract",
            "identifiers",
            "invalidFixtures",
            "limits",
            "managedResources",
            "semantics",
            "status",
            "validFixtures",
            "version",
        ],
        "catalog",
    )?;
    if root.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("catalog version must be 1".into());
    }
    if root.get("contract").and_then(Value::as_str) != Some("motion-graphics-v1") {
        return Err("catalog contract must be motion-graphics-v1".into());
    }
    if root.get("status").and_then(Value::as_str) != Some("fixture_only") {
        return Err("catalog status must be fixture_only".into());
    }

    let semantics = object(
        root.get("semantics")
            .ok_or_else(|| "semantics must exist".to_string())?,
        "semantics",
    )?;
    assert_closed_record(
        semantics,
        &[
            "alphaMode",
            "compositingLight",
            "coordinateSystem",
            "layerOrdering",
            "resourcePolicy",
            "time",
            "variantCase",
            "visualPipeline",
            "wireFieldCase",
        ],
        "semantics",
    )?;
    let coordinate_system = object(
        semantics
            .get("coordinateSystem")
            .ok_or_else(|| "coordinateSystem must exist".to_string())?,
        "coordinateSystem",
    )?;
    assert_closed_record(
        coordinate_system,
        &["origin", "positionUnits", "positiveX", "positiveY"],
        "coordinateSystem",
    )?;
    let time = object(
        semantics
            .get("time")
            .ok_or_else(|| "time semantics must exist".to_string())?,
        "time",
    )?;
    assert_closed_record(time, &["interval", "unit"], "time")?;
    let scalar_semantics = [
        ("wireFieldCase", "lower_camel_case"),
        ("variantCase", "lower_snake_case"),
        ("alphaMode", "premultiplied"),
        ("compositingLight", "linear"),
        ("resourcePolicy", "managed_or_content_addressed_only"),
    ];
    for (key, expected) in scalar_semantics {
        if semantics.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(format!("semantics.{key} must be {expected}"));
        }
    }
    for (key, expected) in [
        ("origin", "top_left"),
        ("positiveX", "right"),
        ("positiveY", "down"),
    ] {
        if coordinate_system.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(format!("coordinateSystem.{key} must be {expected}"));
        }
    }
    if time.get("unit").and_then(Value::as_str) != Some("integer_milliseconds")
        || time.get("interval").and_then(Value::as_str) != Some("half_open")
    {
        return Err("time semantics differ".into());
    }
    for (key, expected) in [
        ("layerOrdering", LAYER_ORDERING),
        ("visualPipeline", VISUAL_PIPELINE),
    ] {
        let actual = strings(
            semantics
                .get(key)
                .ok_or_else(|| format!("semantics.{key} must exist"))?,
            &format!("semantics.{key}"),
        )?;
        if actual.iter().map(String::as_str).collect::<Vec<_>>() != expected {
            return Err(format!("semantics.{key} order differs"));
        }
    }

    let expected_concepts = CONCEPTS.iter().copied().collect::<BTreeSet<_>>();
    let identifiers = root
        .get("identifiers")
        .ok_or_else(|| "identifiers must exist".to_string())?;
    let identifiers = object(identifiers, "identifiers")?;
    assert_closed_record(
        identifiers,
        &[
            "blendModes",
            "concepts",
            "curveTypes",
            "effectTypes",
            "failureClassifications",
            "maskChannels",
            "maskOperations",
            "maskSourceTypes",
            "positionUnits",
            "referenceKinds",
            "slotKinds",
            "timeExpressionTypes",
        ],
        "identifiers",
    )?;
    let catalog_concepts = strings(
        identifiers
            .get("concepts")
            .ok_or_else(|| "identifiers.concepts must exist".to_string())?,
        "identifiers.concepts",
    )?;
    let catalog_concepts = catalog_concepts
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if catalog_concepts != expected_concepts {
        return Err(format!(
            "concept catalog differs: expected {expected_concepts:?}, found {catalog_concepts:?}"
        ));
    }

    let classifications = strings(
        identifiers
            .get("failureClassifications")
            .ok_or_else(|| "failure classifications must exist".to_string())?,
        "identifiers.failureClassifications",
    )?;
    let classifications = classifications
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected_classifications = FAILURE_CLASSIFICATIONS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if classifications != expected_classifications {
        return Err("failure classification catalog differs".into());
    }
    for (key, expected) in IDENTIFIER_SETS {
        let actual = strings(
            identifiers
                .get(*key)
                .ok_or_else(|| format!("identifiers.{key} must exist"))?,
            &format!("identifiers.{key}"),
        )?;
        let actual = actual.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let expected = expected.iter().copied().collect::<BTreeSet<_>>();
        if actual != expected {
            return Err(format!("identifier set {key} differs"));
        }
    }
    let coordinate_units = strings(
        coordinate_system
            .get("positionUnits")
            .ok_or_else(|| "coordinateSystem.positionUnits must exist".to_string())?,
        "coordinateSystem.positionUnits",
    )?;
    let identifier_units = strings(
        identifiers
            .get("positionUnits")
            .ok_or_else(|| "identifiers.positionUnits must exist".to_string())?,
        "identifiers.positionUnits",
    )?;
    if coordinate_units != identifier_units {
        return Err("coordinate and identifier position units differ".into());
    }

    let limits = root
        .get("limits")
        .ok_or_else(|| "limits must exist".to_string())?;
    let limits = object(limits, "limits")?;
    let actual_limits = limits.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_limits = LIMITS.iter().copied().collect::<BTreeSet<_>>();
    if actual_limits != expected_limits {
        return Err(format!(
            "limit keys differ: expected {expected_limits:?}, found {actual_limits:?}"
        ));
    }
    for (name, value) in limits {
        validate_limit_value(name, value)?;
    }

    let valid = array(catalog, "validFixtures")?;
    let invalid = array(catalog, "invalidFixtures")?;
    let mut fixture_ids = BTreeSet::new();
    let mut definitions = reference_keys(
        root.get("managedResources")
            .ok_or_else(|| "managedResources must exist".to_string())?,
        "managedResources",
    )?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut references = Vec::new();
    let mut covered_concepts = BTreeSet::new();

    for fixture in valid {
        let fixture = object(fixture, "valid fixture")?;
        assert_closed_record(
            fixture,
            &["concept", "defines", "id", "references", "value"],
            "valid fixture",
        )?;
        let id = fixture
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "valid fixture id must be a string".to_string())?;
        if !fixture_ids.insert(id.to_owned()) {
            return Err(format!("duplicate fixture id: {id}"));
        }
        let concept = fixture
            .get("concept")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{id} concept must be a string"))?;
        if !expected_concepts.contains(concept) {
            return Err(format!("{id} has unknown concept {concept}"));
        }
        covered_concepts.insert(concept);
        for definition in reference_keys(&fixture["defines"], &format!("{id}.defines"))? {
            if !definitions.insert(definition.clone()) {
                return Err(format!("duplicate logical definition: {definition}"));
            }
        }
        references.extend(
            reference_keys(&fixture["references"], &format!("{id}.references"))?
                .into_iter()
                .map(|reference| (id.to_owned(), reference)),
        );
        validate_finite_values(&fixture["value"], &format!("validFixtures.{id}.value"))?;
    }
    if covered_concepts != expected_concepts {
        return Err(format!(
            "valid fixture concept coverage differs: expected {expected_concepts:?}, found {covered_concepts:?}"
        ));
    }
    for (fixture_id, reference) in references {
        if !definitions.contains(&reference) {
            return Err(format!(
                "valid fixture {fixture_id} has unresolved reference {reference}"
            ));
        }
    }

    let mut observed_classifications = BTreeSet::new();
    for fixture in invalid {
        let fixture = object(fixture, "invalid fixture")?;
        assert_closed_record(
            fixture,
            &["classification", "concept", "id", "reason", "value"],
            "invalid fixture",
        )?;
        let id = fixture
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "invalid fixture id must be a string".to_string())?;
        if !fixture_ids.insert(id.to_owned()) {
            return Err(format!("duplicate fixture id: {id}"));
        }
        let concept = fixture
            .get("concept")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{id} concept must be a string"))?;
        if !expected_concepts.contains(concept) {
            return Err(format!("{id} has unknown concept {concept}"));
        }
        let classification = fixture
            .get("classification")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{id} classification must be a string"))?;
        if !expected_classifications.contains(classification) {
            return Err(format!("{id} has unknown classification {classification}"));
        }
        observed_classifications.insert(classification);
        if fixture
            .get("reason")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!("{id} must have a reason"));
        }
    }
    if observed_classifications != expected_classifications {
        return Err("invalid fixtures must cover every failure classification".into());
    }

    Ok(())
}

#[test]
fn canonical_motion_graphics_catalog_is_complete_and_safe() {
    let catalog: Value = serde_json::from_str(CATALOG_SOURCE).expect("catalog must be valid JSON");
    validate_catalog(&catalog).unwrap_or_else(|message| panic!("{message}"));
    strict_contract::validate_catalog(&catalog).unwrap_or_else(|message| panic!("{message}"));
}

#[test]
fn motion_graphics_catalog_validation_rejects_duplicate_fixture_ids() {
    let mut catalog: Value =
        serde_json::from_str(CATALOG_SOURCE).expect("catalog must be valid JSON");
    let duplicate = catalog["validFixtures"][0].clone();
    catalog["validFixtures"]
        .as_array_mut()
        .expect("valid fixtures must be an array")
        .push(duplicate);

    let message = validate_catalog(&catalog).expect_err("duplicate fixture IDs must fail");
    assert!(message.contains("duplicate fixture id: transform.complete"));
}

#[test]
fn motion_graphics_catalog_limits_use_javascript_safe_integers() {
    for limit in LIMITS {
        validate_limit_value(limit, &Value::from(MAX_SAFE_INTEGER))
            .unwrap_or_else(|message| panic!("{limit} boundary failed: {message}"));
        let message = validate_limit_value(limit, &Value::from(MAX_SAFE_INTEGER + 1))
            .expect_err("the first unsafe integer must fail");
        assert!(message.contains(limit));
        assert!(message.contains("JavaScript-safe integer"));
    }
}

#[test]
fn strict_motion_graphics_contract_rejects_malformed_payloads_and_scope_drift() {
    let catalog: Value = serde_json::from_str(CATALOG_SOURCE).expect("catalog must be valid JSON");
    strict_contract::validate_malformed_regressions(&catalog)
        .unwrap_or_else(|message| panic!("{message}"));
}
