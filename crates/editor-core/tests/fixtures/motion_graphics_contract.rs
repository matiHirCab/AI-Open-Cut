use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::Value;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct Reference {
    id: String,
    kind: ReferenceKind,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CatalogEnvelope {
    contract: String,
    identifiers: Value,
    invalid_fixtures: Vec<InvalidFixtureEnvelope>,
    limits: LimitsEnvelope,
    managed_resources: Vec<ManagedAssetReference>,
    semantics: Value,
    status: String,
    valid_fixtures: Vec<ValidFixtureEnvelope>,
    version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
struct ManagedAssetReference {
    id: String,
    kind: ManagedAssetKind,
    scope: ManagedAssetScope,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum ManagedAssetKind {
    Asset,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum ManagedAssetScope {
    Project,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidFixtureEnvelope {
    concept: String,
    defines: Vec<Reference>,
    id: String,
    references: Vec<Reference>,
    value: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvalidFixtureEnvelope {
    classification: String,
    concept: String,
    id: String,
    reason: String,
    value: Value,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct LimitsEnvelope {
    max_audio_events_per_composition: u64,
    max_component_definitions: u64,
    max_component_depth: u64,
    max_effects_per_layer: u64,
    max_keyframes_per_channel: u64,
    max_layers_per_composition: u64,
    max_markers_per_composition: u64,
    max_masks_per_layer: u64,
    max_parent_depth: u64,
    max_slots_per_component: u64,
}

fn validate_limit_values(limits: &LimitsEnvelope) -> Result<(), String> {
    for (name, value) in [
        (
            "maxAudioEventsPerComposition",
            limits.max_audio_events_per_composition,
        ),
        ("maxComponentDefinitions", limits.max_component_definitions),
        ("maxComponentDepth", limits.max_component_depth),
        ("maxEffectsPerLayer", limits.max_effects_per_layer),
        ("maxKeyframesPerChannel", limits.max_keyframes_per_channel),
        ("maxLayersPerComposition", limits.max_layers_per_composition),
        (
            "maxMarkersPerComposition",
            limits.max_markers_per_composition,
        ),
        ("maxMasksPerLayer", limits.max_masks_per_layer),
        ("maxParentDepth", limits.max_parent_depth),
        ("maxSlotsPerComponent", limits.max_slots_per_component),
    ] {
        if value == 0 || value > MAX_SAFE_INTEGER {
            return Err(format!("{name} must be a positive JavaScript-safe integer"));
        }
    }
    Ok(())
}

fn limit_usize(value: u64, name: &str) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| format!("{name} does not fit usize"))
}

fn ensure_unique_values<I>(values: I, label: &str) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    let mut observed = BTreeSet::new();
    for value in values {
        if !observed.insert(value) {
            return Err(format!("{label} duplicate payload definition"));
        }
    }
    Ok(())
}

fn duplicate_keys<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .filter_map(|(value, count)| (count > 1).then_some(value))
        .collect()
}

fn assert_only_declared_duplicate_key(
    duplicates: Vec<String>,
    allowed_keys: BTreeSet<String>,
    label: &str,
) -> Result<(), String> {
    if duplicates.is_empty() {
        return Err(format!("{label} declared ambiguity key is not duplicated"));
    }
    if duplicates.len() != 1 || !allowed_keys.contains(&duplicates[0]) {
        return Err(format!(
            "{label} duplicate payload definition outside declared ambiguity"
        ));
    }
    Ok(())
}

fn assert_single_declared_duplicate<I>(
    values: I,
    allowed_keys: BTreeSet<String>,
    label: &str,
) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    assert_only_declared_duplicate_key(duplicate_keys(values), allowed_keys, label)
}

fn ensure_at_most_per_owner<I>(owners: I, limit: usize, label: &str) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    let mut counts = BTreeMap::new();
    for owner in owners {
        let count = counts.entry(owner).or_insert(0usize);
        *count += 1;
        ensure_at_most(*count, limit, label)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum ReferenceKind {
    Asset,
    AudioBus,
    AudioEvent,
    Component,
    Curve,
    Effect,
    Layer,
    Marker,
    Mask,
    Slot,
    SoundDefinition,
    Transform,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TransformPayload {
    anchor: Point,
    id: String,
    opacity: f64,
    position: Position,
    rotation_deg: f64,
    scale_x: f64,
    scale_y: f64,
    scope: String,
    skew_x_deg: f64,
    skew_y_deg: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Position {
    unit: PositionUnit,
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PositionUnit {
    Pixels,
    Normalized,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct LayerSetPayload {
    layers: Vec<Layer>,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Layer {
    animation_channels: Vec<String>,
    blend_mode: BlendMode,
    clip: Option<Clip>,
    effects: Vec<String>,
    hidden: bool,
    id: String,
    masks: Vec<String>,
    parent_id: Option<String>,
    stable_item_index: u64,
    transform_id: Option<String>,
    z_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
    Darken,
    Lighten,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Clip {
    CompositionBounds,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentPayload {
    definition: ComponentDefinition,
    instance: ComponentInstance,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ComponentDefinition {
    duration_ms: u64,
    height: u32,
    id: String,
    layers: Vec<String>,
    marker_ids: Vec<String>,
    name: String,
    slot_ids: Vec<String>,
    track_ids: Vec<String>,
    width: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ComponentInstance {
    component_id: String,
    duration_ms: u64,
    id: String,
    slot_values: BTreeMap<String, String>,
    start_ms: u64,
    time_scale: f64,
    trim_start_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SlotPayload {
    binding: SlotBinding,
    constraints: SlotConstraints,
    default_value: String,
    id: String,
    kind: SlotKind,
    name: String,
    required: bool,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SlotBinding {
    property: String,
    target_layer_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SlotConstraints {
    max_length: usize,
    min_length: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SlotKind {
    Text,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MarkerPayload {
    absolute_time: TimeExpression,
    marker: Marker,
    relative_time: TimeExpression,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Marker {
    id: String,
    kind: MarkerKind,
    name: String,
    scope: String,
    time_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MarkerKind {
    Cue,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TimeExpression {
    Milliseconds {
        #[serde(rename = "valueMs")]
        value_ms: u64,
    },
    Marker {
        #[serde(rename = "markerName")]
        marker_name: String,
        #[serde(rename = "offsetMs")]
        offset_ms: i64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CurveSetPayload {
    curves: Vec<Curve>,
    id: String,
    keyframes: Vec<Keyframe>,
    scope: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Curve {
    Hold,
    Linear,
    CubicBezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Spring {
        mass: f64,
        stiffness: f64,
        damping: f64,
        #[serde(rename = "initialVelocity")]
        initial_velocity: f64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Keyframe {
    curve: Curve,
    time: TimeExpression,
    value: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MaskPayload {
    layer_id: String,
    masks: Vec<Mask>,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Mask {
    channel: MaskChannel,
    expansion_px: f64,
    feather_px: f64,
    id: String,
    inverted: bool,
    operation: MaskOperation,
    source: MaskSource,
    transform_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MaskChannel {
    Alpha,
    Luma,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MaskOperation {
    Add,
    Subtract,
    Intersect,
    Exclude,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum MaskSource {
    Path {
        commands: Vec<PathCommand>,
    },
    Layer {
        #[serde(rename = "layerId")]
        layer_id: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum PathCommand {
    MoveTo { x: f64, y: f64 },
    LineTo { x: f64, y: f64 },
    Close,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct EffectPayload {
    effects: Vec<Effect>,
    layer_id: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Effect {
    GaussianBlur {
        id: String,
        #[serde(rename = "radiusPx")]
        radius_px: f64,
    },
    Glow {
        color: String,
        id: String,
        intensity: f64,
        #[serde(rename = "radiusPx")]
        radius_px: f64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AudioPayload {
    bus: AudioBus,
    event: AudioEvent,
    sound_definition: SoundDefinition,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AudioBus {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AudioEvent {
    at: TimeExpression,
    bus_id: String,
    event: String,
    gain_db: f64,
    id: String,
    scope: String,
    variant_seed: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SoundDefinition {
    bus_id: String,
    default_gain_db: f64,
    event: String,
    variant_asset_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct DependencyScenario {
    component_ids: Vec<String>,
    dependencies: Vec<DependencyEdge>,
    entry_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyEdge {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TaggedSlotValue {
    Text { value: String },
    Number { value: f64 },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SlotScenario {
    definition: SlotScenarioDefinition,
    target_layer_ids: Vec<String>,
    supplied_value: Option<TaggedSlotValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SlotScenarioDefinition {
    binding: SlotBinding,
    constraints: SlotConstraints,
    default_value: TaggedSlotValue,
    id: String,
    kind: SlotKind,
    name: String,
    required: bool,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MarkerScenario {
    lookup_name: String,
    markers: Vec<Marker>,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AudioScenario {
    assets: Vec<String>,
    buses: Vec<AudioBus>,
    events: Vec<AudioEvent>,
    markers: Vec<Marker>,
    #[serde(rename = "soundDefinitions")]
    sound_definitions: Vec<SoundDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MaskScenario {
    available_layer_ids: Vec<String>,
    layer_id: String,
    masks: Vec<InvalidMask>,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct InvalidMask {
    channel: MaskChannel,
    expansion_px: f64,
    feather_px: f64,
    id: String,
    inverted: bool,
    operation: MaskOperation,
    source: InvalidMaskSource,
    transform_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum InvalidMaskSource {
    Path {
        commands: Vec<PathCommand>,
    },
    Layer {
        #[serde(rename = "layerId")]
        layer_id: String,
    },
    InlineSvg {
        svg: String,
    },
    File {
        path: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RendererExpressionScenario {
    effects: Vec<RendererExpressionEffect>,
    layer_id: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RendererExpressionEffect {
    expression: String,
    id: String,
    #[serde(rename = "type")]
    kind: RendererExpressionKind,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RendererExpressionKind {
    RendererExpression,
}

#[derive(Debug)]
struct DerivedReferences {
    defines: BTreeSet<Reference>,
    references: BTreeSet<Reference>,
}

fn valid_id(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
        && value.len() <= 128
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn validate_scope(scope: &str) -> Result<(), String> {
    if matches!(scope, "project" | "root") || scope.strip_prefix("component:").is_some_and(valid_id)
    {
        Ok(())
    } else {
        Err(format!("invalid reference scope: {scope}"))
    }
}

fn validate_reference(reference: &Reference) -> Result<(), String> {
    validate_scope(&reference.scope)?;
    let composition_scope = reference.scope == "root" || reference.scope.starts_with("component:");
    let legal_scope = match reference.kind {
        ReferenceKind::Asset
        | ReferenceKind::AudioBus
        | ReferenceKind::Component
        | ReferenceKind::SoundDefinition => reference.scope == "project",
        ReferenceKind::Slot => reference.scope.starts_with("component:"),
        ReferenceKind::AudioEvent
        | ReferenceKind::Curve
        | ReferenceKind::Effect
        | ReferenceKind::Layer
        | ReferenceKind::Marker
        | ReferenceKind::Mask
        | ReferenceKind::Transform => composition_scope,
    };
    if valid_id(&reference.id) && legal_scope {
        Ok(())
    } else if !legal_scope {
        Err(format!("illegal kind/scope tuple: {reference:?}"))
    } else {
        Err(format!("invalid logical id: {}", reference.id))
    }
}

fn reference(kind: ReferenceKind, scope: &str, id: &str) -> Result<Reference, String> {
    let reference = Reference {
        id: id.to_owned(),
        kind,
        scope: scope.to_owned(),
    };
    validate_reference(&reference)?;
    Ok(reference)
}

fn finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn validate_time(time: &TimeExpression) -> Result<(), String> {
    match time {
        TimeExpression::Milliseconds { value_ms } => {
            if *value_ms <= MAX_SAFE_INTEGER {
                Ok(())
            } else {
                Err("time exceeds JavaScript safe integer range".into())
            }
        }
        TimeExpression::Marker {
            marker_name,
            offset_ms,
        } => {
            if valid_id(marker_name) && offset_ms.unsigned_abs() <= MAX_SAFE_INTEGER {
                Ok(())
            } else {
                Err("invalid marker name".into())
            }
        }
    }
}

fn validate_curve(curve: &Curve) -> Result<(), String> {
    match curve {
        Curve::Hold | Curve::Linear => Ok(()),
        Curve::CubicBezier { x1, y1, x2, y2 } => {
            if finite(&[*x1, *y1, *x2, *y2]) && (0.0..=1.0).contains(x1) && (0.0..=1.0).contains(x2)
            {
                Ok(())
            } else {
                Err("invalid cubic bezier".into())
            }
        }
        Curve::Spring {
            mass,
            stiffness,
            damping,
            initial_velocity,
        } => {
            if finite(&[*mass, *stiffness, *damping, *initial_velocity])
                && *mass > 0.0
                && *stiffness > 0.0
                && *damping > 0.0
            {
                Ok(())
            } else {
                Err("invalid spring".into())
            }
        }
    }
}

fn validate_composition_scope(scope: &str) -> Result<(), String> {
    validate_scope(scope)?;
    if scope == "root" || scope.starts_with("component:") {
        Ok(())
    } else {
        Err(format!("composition scope required: {scope}"))
    }
}

fn validate_component_scope(scope: &str) -> Result<(), String> {
    validate_scope(scope)?;
    if scope.starts_with("component:") {
        Ok(())
    } else {
        Err(format!("component scope required: {scope}"))
    }
}

fn validate_layer_scenario_fields(value: &LayerSetPayload) -> Result<(), String> {
    validate_composition_scope(&value.scope)?;
    if value.layers.is_empty() {
        return Err("layer set must not be empty".into());
    }
    let mut layer_ids = BTreeSet::new();
    for layer in &value.layers {
        if !layer_ids.insert(layer.id.as_str()) {
            return Err(format!("duplicate payload definition: {}", layer.id));
        }
        reference(ReferenceKind::Layer, &value.scope, &layer.id)?;
        if layer.stable_item_index > MAX_SAFE_INTEGER {
            return Err("stable item index exceeds JavaScript safe integer range".into());
        }
        if layer.animation_channels.iter().any(String::is_empty) {
            return Err("animation channel must not be empty".into());
        }
        if let Some(parent_id) = &layer.parent_id {
            reference(ReferenceKind::Layer, &value.scope, parent_id)?;
        }
        if let Some(transform_id) = &layer.transform_id {
            reference(ReferenceKind::Transform, &value.scope, transform_id)?;
        }
        for mask_id in &layer.masks {
            reference(ReferenceKind::Mask, &value.scope, mask_id)?;
        }
        for effect_id in &layer.effects {
            reference(ReferenceKind::Effect, &value.scope, effect_id)?;
        }
    }
    Ok(())
}

fn validate_dependency_scenario_fields(
    scenario: &DependencyScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    if scenario.component_ids.is_empty()
        || scenario.component_ids.iter().any(|id| !valid_id(id))
        || !valid_id(&scenario.entry_id)
        || scenario
            .dependencies
            .iter()
            .any(|edge| !valid_id(&edge.from) || !valid_id(&edge.to))
    {
        Err("invalid component dependency envelope".into())
    } else {
        ensure_unique_values(
            scenario.component_ids.iter().cloned(),
            "component invalid envelope",
        )?;
        ensure_unique_values(
            scenario
                .dependencies
                .iter()
                .map(|edge| format!("{}\0{}", edge.from, edge.to)),
            "component invalid envelope dependency edge",
        )?;
        ensure_at_most(
            scenario.component_ids.len(),
            limit_usize(limits.max_component_definitions, "maxComponentDefinitions")?,
            "maxComponentDefinitions",
        )
    }
}

fn validate_marker_candidate(marker: &Marker) -> Result<(), String> {
    validate_composition_scope(&marker.scope)?;
    if valid_id(&marker.id) && valid_id(&marker.name) && marker.time_ms <= MAX_SAFE_INTEGER {
        Ok(())
    } else {
        Err("invalid marker candidate".into())
    }
}

fn tagged_slot_text_length(value: &TaggedSlotValue) -> Result<Option<usize>, String> {
    match value {
        TaggedSlotValue::Text { value } => Ok(Some(value.chars().count())),
        TaggedSlotValue::Number { value } if value.is_finite() => Ok(None),
        TaggedSlotValue::Number { .. } => Err("non-finite tagged slot value".into()),
    }
}

fn validate_slot_scenario_fields(
    id: &str,
    scenario: &SlotScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    let definition = &scenario.definition;
    validate_component_scope(&definition.scope)?;
    if !valid_id(&definition.id)
        || definition.name.is_empty()
        || definition.name.chars().count() > 200
        || !valid_id(&definition.binding.target_layer_id)
        || definition.binding.property.is_empty()
        || definition.constraints.max_length == 0
        || definition.constraints.max_length > 4096
        || definition.constraints.min_length > definition.constraints.max_length
        || scenario.target_layer_ids.is_empty()
        || scenario.target_layer_ids.iter().any(|id| !valid_id(id))
    {
        return Err("invalid slot scenario envelope".into());
    }
    ensure_unique_values(
        scenario.target_layer_ids.iter().cloned(),
        "slot invalid envelope target layer",
    )?;
    ensure_at_most(
        scenario.target_layer_ids.len(),
        limit_usize(limits.max_layers_per_composition, "maxLayersPerComposition")?,
        "maxLayersPerComposition",
    )?;
    if definition.binding.property != "text.document" && id != "slot.arbitrary_path" {
        return Err("unexpected unstable binding target".into());
    }
    let default_length = tagged_slot_text_length(&definition.default_value)?;
    if default_length.is_none() && id != "slot.invalid_default" {
        return Err("unexpected slot default type mismatch".into());
    }
    if default_length.is_some_and(|length| {
        length < definition.constraints.min_length || length > definition.constraints.max_length
    }) {
        return Err("slot default constraint violation".into());
    }
    if scenario.supplied_value.is_none() && id != "slot.required_value_missing" {
        return Err("unexpected missing slot value".into());
    }
    if let Some(supplied) = &scenario.supplied_value {
        let supplied_length = tagged_slot_text_length(supplied)?;
        if supplied_length.is_none() && id != "slot.type_mismatch" {
            return Err("unexpected slot value type mismatch".into());
        }
        if supplied_length.is_some_and(|length| {
            (length < definition.constraints.min_length
                || length > definition.constraints.max_length)
                && id != "slot.constraint_violation"
        }) {
            return Err("unexpected slot value constraint violation".into());
        }
    }
    Ok(())
}

fn validate_marker_scenario_fields(
    scenario: &MarkerScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    validate_composition_scope(&scenario.scope)?;
    if !valid_id(&scenario.lookup_name) || scenario.markers.is_empty() {
        return Err("invalid marker scenario envelope".into());
    }
    for marker in &scenario.markers {
        validate_marker_candidate(marker)?;
        if marker.scope != scenario.scope {
            return Err("marker scope differs from scenario scope".into());
        }
    }
    ensure_unique_values(
        scenario
            .markers
            .iter()
            .map(|marker| format!("{}\0{}", marker.scope, marker.id)),
        "marker invalid envelope",
    )?;
    ensure_at_most(
        scenario.markers.len(),
        limit_usize(
            limits.max_markers_per_composition,
            "maxMarkersPerComposition",
        )?,
        "maxMarkersPerComposition",
    )
}

fn validate_audio_scenario_fields(
    id: &str,
    scenario: &AudioScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    if scenario.assets.is_empty()
        || scenario.assets.iter().any(|asset| !valid_id(asset))
        || scenario.buses.is_empty()
        || scenario.events.is_empty()
        || scenario.markers.is_empty()
        || scenario.sound_definitions.is_empty()
    {
        return Err("invalid audio scenario collection".into());
    }
    ensure_unique_values(
        scenario.assets.iter().cloned(),
        "audio invalid envelope asset",
    )?;
    ensure_unique_values(
        scenario
            .events
            .iter()
            .map(|event| format!("{}\0{}", event.scope, event.id)),
        "audio invalid envelope event",
    )?;
    ensure_unique_values(
        scenario
            .markers
            .iter()
            .map(|marker| format!("{}\0{}", marker.scope, marker.id)),
        "audio invalid envelope marker",
    )?;
    let bus_ids = scenario.buses.iter().map(|bus| bus.id.clone());
    if id == "audio_event.ambiguous_bus" {
        assert_single_declared_duplicate(
            bus_ids,
            scenario
                .events
                .iter()
                .map(|event| event.bus_id.clone())
                .collect(),
            "audio invalid envelope bus",
        )?;
    } else {
        ensure_unique_values(bus_ids, "audio invalid envelope bus")?;
    }
    let declared_bus_ids = scenario
        .buses
        .iter()
        .map(|bus| bus.id.as_str())
        .collect::<BTreeSet<_>>();
    if scenario
        .sound_definitions
        .iter()
        .any(|definition| !declared_bus_ids.contains(definition.bus_id.as_str()))
    {
        return Err("audio invalid envelope sound definition bus missing reference".into());
    }
    let marker_names = scenario
        .markers
        .iter()
        .map(|marker| format!("{}\0{}", marker.scope, marker.name));
    if id == "audio_event.ambiguous_marker" {
        assert_single_declared_duplicate(
            marker_names,
            scenario
                .events
                .iter()
                .filter_map(|event| match &event.at {
                    TimeExpression::Marker { marker_name, .. } => {
                        Some(format!("{}\0{}", event.scope, marker_name))
                    }
                    TimeExpression::Milliseconds { .. } => None,
                })
                .collect(),
            "audio invalid envelope marker name",
        )?;
    } else {
        ensure_unique_values(marker_names, "audio invalid envelope marker name")?;
    }
    let sound_events = scenario
        .sound_definitions
        .iter()
        .map(|definition| definition.event.clone());
    if id == "audio_event.ambiguous_sound_definition" {
        assert_single_declared_duplicate(
            sound_events,
            scenario
                .events
                .iter()
                .map(|event| event.event.clone())
                .collect(),
            "audio invalid envelope sound definition",
        )?;
    } else {
        ensure_unique_values(sound_events, "audio invalid envelope sound definition")?;
    }
    ensure_at_most_per_owner(
        scenario.events.iter().map(|event| event.scope.clone()),
        limit_usize(
            limits.max_audio_events_per_composition,
            "maxAudioEventsPerComposition",
        )?,
        "maxAudioEventsPerComposition",
    )?;
    ensure_at_most_per_owner(
        scenario.markers.iter().map(|marker| marker.scope.clone()),
        limit_usize(
            limits.max_markers_per_composition,
            "maxMarkersPerComposition",
        )?,
        "maxMarkersPerComposition",
    )?;
    for bus in &scenario.buses {
        if !valid_id(&bus.id) {
            return Err("invalid audio bus ID".into());
        }
    }
    for marker in &scenario.markers {
        validate_marker_candidate(marker)?;
    }
    for event in &scenario.events {
        validate_composition_scope(&event.scope)?;
        validate_time(&event.at)?;
        if !valid_id(&event.id)
            || !valid_id(&event.bus_id)
            || !valid_id(&event.event)
            || event.variant_seed > MAX_SAFE_INTEGER
            || !event.gain_db.is_finite()
            || !(-120.0..=24.0).contains(&event.gain_db)
        {
            return Err("invalid audio event envelope".into());
        }
    }
    for definition in &scenario.sound_definitions {
        if !valid_id(&definition.bus_id)
            || !valid_id(&definition.event)
            || definition.variant_asset_ids.is_empty()
            || !definition.default_gain_db.is_finite()
            || !(-120.0..=24.0).contains(&definition.default_gain_db)
        {
            return Err("invalid sound definition envelope".into());
        }
        for asset in &definition.variant_asset_ids {
            if asset.is_empty() || (!valid_id(asset) && id != "audio_event.network_variant") {
                return Err("unexpected invalid sound variant ID".into());
            }
        }
        if id != "audio_event.ambiguous_variant" {
            ensure_unique_values(
                definition.variant_asset_ids.iter().cloned(),
                "audio invalid envelope sound variant",
            )?;
        }
    }
    if id == "audio_event.ambiguous_variant" {
        let duplicates = scenario
            .sound_definitions
            .iter()
            .flat_map(|definition| {
                duplicate_keys(definition.variant_asset_ids.iter().cloned())
                    .into_iter()
                    .map(|asset| format!("{}\0{}", definition.event, asset))
                    .collect::<Vec<_>>()
            })
            .collect();
        let referenced_events = scenario
            .events
            .iter()
            .map(|event| event.event.as_str())
            .collect::<BTreeSet<_>>();
        let allowed = scenario
            .sound_definitions
            .iter()
            .filter(|definition| referenced_events.contains(definition.event.as_str()))
            .flat_map(|definition| {
                definition
                    .variant_asset_ids
                    .iter()
                    .map(|asset| format!("{}\0{}", definition.event, asset))
            })
            .collect();
        assert_only_declared_duplicate_key(
            duplicates,
            allowed,
            "audio invalid envelope sound variant",
        )?;
    }
    Ok(())
}

fn validate_mask_scenario_fields(
    scenario: &MaskScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    validate_composition_scope(&scenario.scope)?;
    if scenario.available_layer_ids.is_empty()
        || scenario.available_layer_ids.iter().any(|id| !valid_id(id))
        || !valid_id(&scenario.layer_id)
        || scenario.masks.is_empty()
    {
        return Err("invalid mask scenario envelope".into());
    }
    ensure_unique_values(
        scenario.available_layer_ids.iter().cloned(),
        "mask invalid envelope available layer",
    )?;
    ensure_at_most(
        scenario.available_layer_ids.len(),
        limit_usize(limits.max_layers_per_composition, "maxLayersPerComposition")?,
        "maxLayersPerComposition",
    )?;
    ensure_at_most(
        scenario.masks.len(),
        limit_usize(limits.max_masks_per_layer, "maxMasksPerLayer")?,
        "maxMasksPerLayer",
    )?;
    let mut ids = BTreeSet::new();
    for mask in &scenario.masks {
        let _ = (&mask.channel, mask.inverted, &mask.operation);
        if !ids.insert(mask.id.as_str()) {
            return Err(format!("duplicate payload definition: {}", mask.id));
        }
        if !valid_id(&mask.id)
            || !valid_id(&mask.transform_id)
            || !finite(&[mask.expansion_px, mask.feather_px])
            || mask.feather_px < 0.0
        {
            return Err("invalid mask envelope".into());
        }
        match &mask.source {
            InvalidMaskSource::Layer { layer_id } if !valid_id(layer_id) => {
                return Err("invalid mask layer source ID".into());
            }
            InvalidMaskSource::Path { commands } => {
                if commands.is_empty() {
                    return Err("mask path must not be empty".into());
                }
                for command in commands {
                    match command {
                        PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y }
                            if !finite(&[*x, *y]) =>
                        {
                            return Err("non-finite mask path".into());
                        }
                        _ => {}
                    }
                }
            }
            InvalidMaskSource::InlineSvg { svg } if svg.is_empty() => {
                return Err("inline SVG must not be empty".into());
            }
            InvalidMaskSource::File { path } if path.is_empty() => {
                return Err("mask file path must not be empty".into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_renderer_expression_scenario(
    scenario: &RendererExpressionScenario,
    limits: &LimitsEnvelope,
) -> Result<(), String> {
    validate_composition_scope(&scenario.scope)?;
    if !valid_id(&scenario.layer_id) || scenario.effects.is_empty() {
        return Err("invalid renderer expression envelope".into());
    }
    ensure_at_most(
        scenario.effects.len(),
        limit_usize(limits.max_effects_per_layer, "maxEffectsPerLayer")?,
        "maxEffectsPerLayer",
    )?;
    let mut ids = BTreeSet::new();
    for effect in &scenario.effects {
        let _ = &effect.kind;
        if !ids.insert(effect.id.as_str()) {
            return Err(format!("duplicate payload definition: {}", effect.id));
        }
        if !valid_id(&effect.id) || effect.expression.is_empty() {
            return Err("invalid renderer expression effect".into());
        }
    }
    Ok(())
}

fn contains_svg_event_handler(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        index += 1;
        if bytes.get(index..index + 2) != Some(b"on") {
            continue;
        }
        index += 2;
        let name_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_lowercase) {
            index += 1;
        }
        if index == name_start {
            continue;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'=') {
            return true;
        }
    }
    false
}

fn derive_transform(value: Value) -> Result<DerivedReferences, String> {
    let value: TransformPayload =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    let _ = value.position.unit;
    if !finite(&[
        value.anchor.x,
        value.anchor.y,
        value.opacity,
        value.position.x,
        value.position.y,
        value.rotation_deg,
        value.scale_x,
        value.scale_y,
        value.skew_x_deg,
        value.skew_y_deg,
    ]) || !(0.0..=1.0).contains(&value.anchor.x)
        || !(0.0..=1.0).contains(&value.anchor.y)
        || !(0.0..=1.0).contains(&value.opacity)
        || value.scale_x <= 0.0
        || value.scale_y <= 0.0
    {
        return Err("invalid transform range".into());
    }
    Ok(DerivedReferences {
        defines: [reference(
            ReferenceKind::Transform,
            &value.scope,
            &value.id,
        )?]
        .into(),
        references: BTreeSet::new(),
    })
}

fn derive_layer(value: Value, max_parent_depth: usize) -> Result<DerivedReferences, String> {
    let value: LayerSetPayload =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    if value.layers.is_empty() {
        return Err("layer set must not be empty".into());
    }
    let mut defines = BTreeSet::new();
    let mut references = BTreeSet::new();
    let parents = value
        .layers
        .iter()
        .map(|layer| (layer.id.as_str(), layer.parent_id.as_deref()))
        .collect::<BTreeMap<_, _>>();
    for layer in &value.layers {
        let _ = (
            &layer.animation_channels,
            &layer.blend_mode,
            &layer.clip,
            layer.hidden,
            layer.stable_item_index,
            layer.z_index,
        );
        if layer.stable_item_index > MAX_SAFE_INTEGER {
            return Err("stable item index exceeds JavaScript safe integer range".into());
        }
        if layer.animation_channels.iter().any(String::is_empty) {
            return Err("animation channel must not be empty".into());
        }
        let definition = reference(ReferenceKind::Layer, &value.scope, &layer.id)?;
        if !defines.insert(definition.clone()) {
            return Err(format!("duplicate payload definition: {definition:?}"));
        }
        if let Some(parent) = &layer.parent_id {
            references.insert(reference(ReferenceKind::Layer, &value.scope, parent)?);
        }
        if let Some(transform) = &layer.transform_id {
            references.insert(reference(
                ReferenceKind::Transform,
                &value.scope,
                transform,
            )?);
        }
        for mask in &layer.masks {
            references.insert(reference(ReferenceKind::Mask, &value.scope, mask)?);
        }
        for effect in &layer.effects {
            references.insert(reference(ReferenceKind::Effect, &value.scope, effect)?);
        }
        let mut seen = BTreeSet::new();
        let mut current = Some(layer.id.as_str());
        let mut depth = 0;
        while let Some(id) = current {
            if !seen.insert(id) {
                return Err("parent_cycle".into());
            }
            current = parents.get(id).copied().flatten();
            depth += 1;
            if depth > max_parent_depth {
                return Err("max_parent_depth_exceeded".into());
            }
        }
    }
    Ok(DerivedReferences {
        defines,
        references,
    })
}

fn derive_component(value: Value) -> Result<DerivedReferences, String> {
    let value: ComponentPayload =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    let definition = &value.definition;
    let instance = &value.instance;
    let _ = (
        definition.duration_ms,
        definition.height,
        &definition.name,
        &definition.track_ids,
        definition.width,
        instance.duration_ms,
        &instance.id,
        &instance.slot_values,
        instance.start_ms,
        instance.trim_start_ms,
    );
    if definition.duration_ms == 0
        || definition.duration_ms > MAX_SAFE_INTEGER
        || definition.width == 0
        || definition.width > 16_384
        || definition.height == 0
        || definition.height > 16_384
        || definition.layers.is_empty()
        || definition.track_ids.is_empty()
        || definition.track_ids.iter().any(|id| !valid_id(id))
        || definition.name.is_empty()
        || definition.name.chars().count() > 200
        || !instance.time_scale.is_finite()
        || instance.time_scale <= 0.0
        || instance.duration_ms == 0
        || instance.duration_ms > MAX_SAFE_INTEGER
        || instance.start_ms > MAX_SAFE_INTEGER
        || instance.trim_start_ms > MAX_SAFE_INTEGER
        || !valid_id(&instance.id)
        || instance.slot_values.keys().any(|id| !valid_id(id))
    {
        return Err("invalid component range".into());
    }
    let scope = format!("component:{}", definition.id);
    let mut defines = BTreeSet::from([reference(
        ReferenceKind::Component,
        "project",
        &definition.id,
    )?]);
    for layer in &definition.layers {
        let definition = reference(ReferenceKind::Layer, &scope, layer)?;
        if !defines.insert(definition.clone()) {
            return Err(format!("duplicate payload definition: {definition:?}"));
        }
    }
    let mut references = BTreeSet::from([reference(
        ReferenceKind::Component,
        "project",
        &instance.component_id,
    )?]);
    for slot in &definition.slot_ids {
        references.insert(reference(ReferenceKind::Slot, &scope, slot)?);
    }
    for marker in &definition.marker_ids {
        references.insert(reference(ReferenceKind::Marker, &scope, marker)?);
    }
    Ok(DerivedReferences {
        defines,
        references,
    })
}

fn derive_slot(value: Value) -> Result<DerivedReferences, String> {
    let value: SlotPayload = serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    let _ = (&value.kind, &value.name, value.required);
    if value.binding.property != "text.document"
        || value.name.is_empty()
        || value.name.chars().count() > 200
        || value.constraints.max_length == 0
        || value.constraints.max_length > 4096
        || value.constraints.min_length > value.constraints.max_length
        || value.default_value.chars().count() < value.constraints.min_length
        || value.default_value.chars().count() > value.constraints.max_length
    {
        return Err("invalid slot constraint".into());
    }
    Ok(DerivedReferences {
        defines: [reference(ReferenceKind::Slot, &value.scope, &value.id)?].into(),
        references: [reference(
            ReferenceKind::Layer,
            &value.scope,
            &value.binding.target_layer_id,
        )?]
        .into(),
    })
}

fn derive_marker(value: Value) -> Result<DerivedReferences, String> {
    let value: MarkerPayload = serde_json::from_value(value).map_err(|error| error.to_string())?;
    let _ = (&value.marker.kind, &value.marker.name, value.marker.time_ms);
    validate_scope(&value.marker.scope)?;
    if !valid_id(&value.marker.name) || value.marker.time_ms > MAX_SAFE_INTEGER {
        return Err("invalid marker range".into());
    }
    validate_time(&value.absolute_time)?;
    validate_time(&value.relative_time)?;
    let mut references = BTreeSet::new();
    if let TimeExpression::Marker { marker_name, .. } = &value.relative_time {
        references.insert(reference(
            ReferenceKind::Marker,
            &value.marker.scope,
            marker_name,
        )?);
    }
    Ok(DerivedReferences {
        defines: [reference(
            ReferenceKind::Marker,
            &value.marker.scope,
            &value.marker.id,
        )?]
        .into(),
        references,
    })
}

fn derive_curve(value: Value) -> Result<DerivedReferences, String> {
    let value: CurveSetPayload =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    if value.curves.is_empty() {
        return Err("curve set must not be empty".into());
    }
    for curve in &value.curves {
        validate_curve(curve)?;
    }
    let mut references = BTreeSet::new();
    for keyframe in &value.keyframes {
        validate_curve(&keyframe.curve)?;
        validate_time(&keyframe.time)?;
        if !keyframe.value.is_finite() {
            return Err("non-finite keyframe".into());
        }
        if let TimeExpression::Marker { marker_name, .. } = &keyframe.time {
            references.insert(reference(ReferenceKind::Marker, &value.scope, marker_name)?);
        }
    }
    Ok(DerivedReferences {
        defines: [reference(ReferenceKind::Curve, &value.scope, &value.id)?].into(),
        references,
    })
}

fn derive_mask(value: Value) -> Result<DerivedReferences, String> {
    let value: MaskPayload = serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    let mut defines = BTreeSet::new();
    let mut references = BTreeSet::from([reference(
        ReferenceKind::Layer,
        &value.scope,
        &value.layer_id,
    )?]);
    for mask in &value.masks {
        let _ = (&mask.channel, mask.inverted, &mask.operation);
        if !finite(&[mask.expansion_px, mask.feather_px]) || mask.feather_px < 0.0 {
            return Err("invalid mask range".into());
        }
        let definition = reference(ReferenceKind::Mask, &value.scope, &mask.id)?;
        if !defines.insert(definition.clone()) {
            return Err(format!("duplicate payload definition: {definition:?}"));
        }
        references.insert(reference(
            ReferenceKind::Transform,
            &value.scope,
            &mask.transform_id,
        )?);
        match &mask.source {
            MaskSource::Layer { layer_id } => {
                references.insert(reference(ReferenceKind::Layer, &value.scope, layer_id)?);
            }
            MaskSource::Path { commands } => {
                if commands.is_empty() {
                    return Err("mask path must not be empty".into());
                }
                for command in commands {
                    match command {
                        PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => {
                            if !finite(&[*x, *y]) {
                                return Err("non-finite mask path".into());
                            }
                        }
                        PathCommand::Close => {}
                    }
                }
            }
        }
    }
    Ok(DerivedReferences {
        defines,
        references,
    })
}

fn derive_effect(value: Value) -> Result<DerivedReferences, String> {
    let value: EffectPayload = serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.scope)?;
    let mut defines = BTreeSet::new();
    for effect in &value.effects {
        match effect {
            Effect::GaussianBlur { id, radius_px } => {
                if !radius_px.is_finite() || !(0.0..=4096.0).contains(radius_px) {
                    return Err("invalid blur radius".into());
                }
                let definition = reference(ReferenceKind::Effect, &value.scope, id)?;
                if !defines.insert(definition.clone()) {
                    return Err(format!("duplicate payload definition: {definition:?}"));
                }
            }
            Effect::Glow {
                color,
                id,
                intensity,
                radius_px,
            } => {
                if color.len() != 9
                    || !color.starts_with('#')
                    || !color[1..]
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())
                    || !finite(&[*intensity, *radius_px])
                    || !(0.0..=100.0).contains(intensity)
                    || !(0.0..=4096.0).contains(radius_px)
                {
                    return Err("invalid glow".into());
                }
                let definition = reference(ReferenceKind::Effect, &value.scope, id)?;
                if !defines.insert(definition.clone()) {
                    return Err(format!("duplicate payload definition: {definition:?}"));
                }
            }
        }
    }
    Ok(DerivedReferences {
        defines,
        references: [reference(
            ReferenceKind::Layer,
            &value.scope,
            &value.layer_id,
        )?]
        .into(),
    })
}

fn derive_audio(value: Value) -> Result<DerivedReferences, String> {
    let value: AudioPayload = serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_scope(&value.event.scope)?;
    validate_time(&value.event.at)?;
    if value.event.variant_seed > MAX_SAFE_INTEGER {
        return Err("variant seed exceeds JavaScript safe integer range".into());
    }
    if !finite(&[value.event.gain_db, value.sound_definition.default_gain_db])
        || !(-120.0..=24.0).contains(&value.event.gain_db)
        || !(-120.0..=24.0).contains(&value.sound_definition.default_gain_db)
        || value.sound_definition.variant_asset_ids.is_empty()
    {
        return Err("invalid audio event range".into());
    }
    let mut references = BTreeSet::from([
        reference(
            ReferenceKind::SoundDefinition,
            "project",
            &value.event.event,
        )?,
        reference(ReferenceKind::AudioBus, "project", &value.event.bus_id)?,
        reference(
            ReferenceKind::AudioBus,
            "project",
            &value.sound_definition.bus_id,
        )?,
    ]);
    if let TimeExpression::Marker { marker_name, .. } = &value.event.at {
        references.insert(reference(
            ReferenceKind::Marker,
            &value.event.scope,
            marker_name,
        )?);
    }
    for asset in &value.sound_definition.variant_asset_ids {
        references.insert(reference(ReferenceKind::Asset, "project", asset)?);
    }
    Ok(DerivedReferences {
        defines: BTreeSet::from([
            reference(ReferenceKind::AudioBus, "project", &value.bus.id)?,
            reference(
                ReferenceKind::SoundDefinition,
                "project",
                &value.sound_definition.event,
            )?,
            reference(
                ReferenceKind::AudioEvent,
                &value.event.scope,
                &value.event.id,
            )?,
        ]),
        references,
    })
}

fn derive_payload(
    concept: &str,
    value: Value,
    max_parent_depth: usize,
) -> Result<DerivedReferences, String> {
    match concept {
        "transform" => derive_transform(value),
        "layer" => derive_layer(value, max_parent_depth),
        "component" => derive_component(value),
        "slot" => derive_slot(value),
        "marker" => derive_marker(value),
        "curve" => derive_curve(value),
        "mask" => derive_mask(value),
        "effect" => derive_effect(value),
        "audio_event" => derive_audio(value),
        _ => Err(format!("unknown concept: {concept}")),
    }
}

fn metadata_references(value: &Value, label: &str) -> Result<BTreeSet<Reference>, String> {
    let references: Vec<Reference> =
        serde_json::from_value(value.clone()).map_err(|error| format!("{label}: {error}"))?;
    for reference in &references {
        validate_reference(reference)?;
    }
    let unique = references.into_iter().collect::<BTreeSet<_>>();
    if unique.len()
        != value
            .as_array()
            .ok_or_else(|| format!("{label} must be an array"))?
            .len()
    {
        return Err(format!("{label} contains duplicates"));
    }
    Ok(unique)
}

fn managed_asset_references(value: &Value) -> Result<BTreeSet<Reference>, String> {
    let resources: Vec<ManagedAssetReference> = serde_json::from_value(value.clone())
        .map_err(|error| format!("managedResources: {error}"))?;
    let mut references = BTreeSet::new();
    for resource in resources {
        let ManagedAssetReference {
            id,
            kind: ManagedAssetKind::Asset,
            scope: ManagedAssetScope::Project,
        } = resource;
        if !valid_id(&id) {
            return Err(format!("invalid managed asset id: {id}"));
        }
        let reference = Reference {
            id,
            kind: ReferenceKind::Asset,
            scope: "project".into(),
        };
        if !references.insert(reference.clone()) {
            return Err(format!(
                "managedResources contains duplicates: {reference:?}"
            ));
        }
    }
    Ok(references)
}

type FixtureFailure = (&'static str, &'static str);

fn reachable_component_depth<'a>(
    component_id: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    active: &mut BTreeSet<&'a str>,
    memoized: &mut BTreeMap<&'a str, usize>,
) -> Result<usize, ()> {
    if active.contains(component_id) {
        return Err(());
    }
    if let Some(depth) = memoized.get(component_id) {
        return Ok(*depth);
    }
    active.insert(component_id);
    let mut longest = 1;
    if let Some(targets) = edges.get(component_id) {
        for target in targets {
            longest = longest.max(
                reachable_component_depth(target, edges, active, memoized)?
                    .checked_add(1)
                    .ok_or(())?,
            );
        }
    }
    active.remove(component_id);
    memoized.insert(component_id, longest);
    Ok(longest)
}

fn component_failure(value: Value, limits: &LimitsEnvelope) -> Result<FixtureFailure, String> {
    let scenario: DependencyScenario =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_dependency_scenario_fields(&scenario, limits)?;
    let ids = scenario.component_ids.iter().collect::<BTreeSet<_>>();
    if !ids.contains(&scenario.entry_id) {
        return Ok(("missing_reference", "component_not_found"));
    }
    let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &scenario.dependencies {
        if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
            return Ok(("missing_reference", "component_not_found"));
        }
        edges
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let depth = match reachable_component_depth(
        &scenario.entry_id,
        &edges,
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ) {
        Ok(depth) => depth,
        Err(()) => return Ok(("reference_cycle", "component_cycle")),
    };
    if depth > limit_usize(limits.max_component_depth, "maxComponentDepth")? {
        return Ok(("invalid_input", "max_component_depth_exceeded"));
    }
    Err("component candidate has no intended failure".into())
}

fn layer_failure(
    id: &str,
    value: Value,
    limits: &LimitsEnvelope,
) -> Result<FixtureFailure, String> {
    if id == "layer.cross_scope_parent" {
        let scope = value["scope"]
            .as_str()
            .ok_or_else(|| "layer scope missing".to_string())?;
        let mut sanitized = value.clone();
        let sanitized_layers = sanitized["layers"]
            .as_array_mut()
            .ok_or_else(|| "layers missing".to_string())?;
        let mut parent_scopes = Vec::new();
        for layer in sanitized_layers {
            let parent_scope = layer
                .as_object_mut()
                .and_then(|fields| fields.remove("parentScope"))
                .and_then(|scope| scope.as_str().map(str::to_owned))
                .ok_or_else(|| "parentScope must be a string".to_string())?;
            validate_scope(&parent_scope)?;
            parent_scopes.push(parent_scope);
        }
        let scenario: LayerSetPayload =
            serde_json::from_value(sanitized).map_err(|error| error.to_string())?;
        validate_layer_scenario_fields(&scenario)?;
        ensure_at_most(
            scenario.layers.len(),
            limit_usize(limits.max_layers_per_composition, "maxLayersPerComposition")?,
            "maxLayersPerComposition",
        )?;
        if parent_scopes.iter().any(|parent| parent != scope) {
            return Ok(("invalid_input", "parent_scope_mismatch"));
        }
        return Err("cross-scope candidate has no intended failure".into());
    }
    let scenario: LayerSetPayload =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_layer_scenario_fields(&scenario)?;
    ensure_at_most(
        scenario.layers.len(),
        limit_usize(limits.max_layers_per_composition, "maxLayersPerComposition")?,
        "maxLayersPerComposition",
    )?;
    let parents = scenario
        .layers
        .iter()
        .map(|layer| (layer.id.as_str(), layer.parent_id.as_deref()))
        .collect::<BTreeMap<_, _>>();
    for layer in &scenario.layers {
        if layer
            .parent_id
            .as_deref()
            .is_some_and(|parent| !parents.contains_key(parent))
        {
            return Ok(("missing_reference", "parent_not_found"));
        }
        if layer.parent_id.as_deref() == Some(layer.id.as_str()) {
            return Ok(("reference_cycle", "direct_parent_cycle"));
        }
        let mut current = layer.parent_id.as_deref();
        let mut seen = BTreeSet::from([layer.id.as_str()]);
        while let Some(parent) = current {
            if !seen.insert(parent) {
                return Ok(("reference_cycle", "parent_cycle"));
            }
            current = parents.get(parent).copied().flatten();
        }
    }
    Err(format!("{id} has no intended failure"))
}

fn slot_failure(id: &str, value: Value, limits: &LimitsEnvelope) -> Result<FixtureFailure, String> {
    let scenario: SlotScenario =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_slot_scenario_fields(id, &scenario, limits)?;
    let definition = scenario.definition;
    let _ = (
        &definition.id,
        &definition.name,
        &definition.scope,
        &definition.kind,
    );
    if definition.binding.property != "text.document" {
        return Ok(("invalid_input", "unstable_binding_target"));
    }
    if !scenario
        .target_layer_ids
        .contains(&definition.binding.target_layer_id)
    {
        return Ok(("missing_reference", "slot_target_not_found"));
    }
    if !matches!(definition.default_value, TaggedSlotValue::Text { .. }) {
        return Ok(("invalid_input", "slot_default_type_mismatch"));
    }
    let Some(supplied) = scenario.supplied_value else {
        return if definition.required {
            Ok(("invalid_input", "required_slot_value_missing"))
        } else {
            Err("optional slot candidate has no failure".into())
        };
    };
    let TaggedSlotValue::Text { value } = supplied else {
        if let TaggedSlotValue::Number { value } = supplied {
            let _ = value;
        }
        return Ok(("invalid_input", "slot_value_type_mismatch"));
    };
    let length = value.chars().count();
    if length < definition.constraints.min_length || length > definition.constraints.max_length {
        return Ok(("invalid_input", "slot_constraint_violation"));
    }
    Err("slot candidate has no intended failure".into())
}

fn audio_failure(
    id: &str,
    value: Value,
    limits: &LimitsEnvelope,
) -> Result<FixtureFailure, String> {
    let scenario: AudioScenario =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    validate_audio_scenario_fields(id, &scenario, limits)?;
    for definition in &scenario.sound_definitions {
        for asset in &definition.variant_asset_ids {
            if !valid_id(asset) {
                return Ok(("invalid_input", "network_resource_forbidden"));
            }
            if !scenario.assets.contains(asset) {
                return Ok(("missing_reference", "sound_variant_not_found"));
            }
            if definition
                .variant_asset_ids
                .iter()
                .filter(|candidate| *candidate == asset)
                .count()
                > 1
            {
                return Ok(("ambiguous_reference", "sound_variant_ambiguous"));
            }
        }
    }
    for event in &scenario.events {
        let bus_count = scenario
            .buses
            .iter()
            .filter(|bus| bus.id == event.bus_id)
            .count();
        if bus_count > 1 {
            return Ok(("ambiguous_reference", "audio_bus_ambiguous"));
        }
        if bus_count == 0 {
            return Ok(("missing_reference", "audio_bus_not_found"));
        }
        let sound_count = scenario
            .sound_definitions
            .iter()
            .filter(|definition| definition.event == event.event)
            .count();
        if sound_count > 1 {
            return Ok(("ambiguous_reference", "sound_definition_ambiguous"));
        }
        if sound_count == 0 {
            return Ok(("missing_reference", "sound_definition_not_found"));
        }
        if let TimeExpression::Marker { marker_name, .. } = &event.at {
            let marker_count = scenario
                .markers
                .iter()
                .filter(|marker| marker.scope == event.scope && marker.name == *marker_name)
                .count();
            if marker_count > 1 {
                return Ok(("ambiguous_reference", "marker_ambiguous"));
            }
            if marker_count == 0 {
                return Ok(("missing_reference", "marker_not_found"));
            }
        }
    }
    Err("audio candidate has no intended failure".into())
}

fn classify_invalid(
    id: &str,
    mut value: Value,
    limits: &LimitsEnvelope,
) -> Result<FixtureFailure, String> {
    if id == "transform.non_finite" {
        if value["scaleX"] != "NaN" {
            return Err("non-finite token missing".into());
        }
        value["scaleX"] = Value::from(1.0);
        derive_transform(value)?;
        return Ok(("invalid_input", "non_finite_value"));
    }
    if id.starts_with("layer.") {
        return layer_failure(id, value, limits);
    }
    if id.starts_with("component.") {
        return component_failure(value, limits);
    }
    if id.starts_with("slot.") {
        return slot_failure(id, value, limits);
    }
    if id == "marker.ambiguous_name" {
        let scenario: MarkerScenario =
            serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
        validate_marker_scenario_fields(&scenario, limits)?;
        assert_single_declared_duplicate(
            scenario
                .markers
                .iter()
                .map(|marker| format!("{}\0{}", marker.scope, marker.name)),
            BTreeSet::from([format!("{}\0{}", scenario.scope, scenario.lookup_name)]),
            "marker invalid envelope name",
        )?;
        if scenario
            .markers
            .iter()
            .filter(|marker| marker.name == scenario.lookup_name)
            .count()
            > 1
        {
            return Ok(("ambiguous_reference", "duplicate_marker_name"));
        }
    }
    if id == "curve.invalid_spring"
        && value["curves"][0]["type"] == "spring"
        && value["curves"][0]["mass"] == 0.0
    {
        value["curves"][0]["mass"] = Value::from(1.0);
        let scenario: CurveSetPayload =
            serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
        ensure_at_most(
            scenario.keyframes.len(),
            limit_usize(limits.max_keyframes_per_channel, "maxKeyframesPerChannel")?,
            "maxKeyframesPerChannel",
        )?;
        derive_curve(value)?;
        return Ok(("invalid_input", "spring_parameter_out_of_range"));
    }
    if id.starts_with("mask.") {
        let scenario: MaskScenario =
            serde_json::from_value(value).map_err(|error| error.to_string())?;
        validate_mask_scenario_fields(&scenario, limits)?;
        for mask in &scenario.masks {
            match &mask.source {
                InvalidMaskSource::InlineSvg { svg } => {
                    let lower = svg.to_ascii_lowercase();
                    if lower.contains("<script") || contains_svg_event_handler(&lower) {
                        return Ok(("invalid_input", "executable_svg"));
                    }
                }
                InvalidMaskSource::File { .. } => {
                    return Ok(("invalid_input", "arbitrary_path_forbidden"));
                }
                InvalidMaskSource::Layer { layer_id }
                    if !scenario.available_layer_ids.contains(layer_id) =>
                {
                    return Ok(("missing_reference", "mask_source_not_found"));
                }
                _ => {}
            }
        }
        return Err("mask candidate has no intended failure".into());
    }
    if id == "effect.renderer_expression" {
        let scenario: RendererExpressionScenario =
            serde_json::from_value(value).map_err(|error| error.to_string())?;
        validate_renderer_expression_scenario(&scenario, limits)?;
        return Ok(("invalid_input", "renderer_expression_forbidden"));
    }
    if id == "effect.limit_exceeded" {
        let effect: EffectPayload =
            serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
        derive_effect(value.clone())?;
        ensure_unique_values(
            effect.effects.iter().map(|effect| match effect {
                Effect::GaussianBlur { id, .. } | Effect::Glow { id, .. } => id.clone(),
            }),
            "effect limit invalid envelope",
        )?;
        if effect.effects.len() > limit_usize(limits.max_effects_per_layer, "maxEffectsPerLayer")? {
            return Ok(("invalid_input", "max_effects_per_layer_exceeded"));
        }
    }
    if id.starts_with("audio_event.") {
        return audio_failure(id, value, limits);
    }
    Err(format!("{id} has no intended deterministic failure"))
}

fn catalog_limit(root: &serde_json::Map<String, Value>, key: &str) -> Result<usize, String> {
    root["limits"][key]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{key} must be a positive integer"))
}

fn validate_closed_envelope(catalog: &Value) -> Result<(), String> {
    let parsed: CatalogEnvelope =
        serde_json::from_value(catalog.clone()).map_err(|error| error.to_string())?;
    let _ = (
        &parsed.contract,
        &parsed.identifiers,
        &parsed.managed_resources,
        &parsed.semantics,
        &parsed.status,
        parsed.version,
    );
    for fixture in &parsed.valid_fixtures {
        let _ = (
            &fixture.concept,
            &fixture.defines,
            &fixture.id,
            &fixture.references,
            &fixture.value,
        );
    }
    for fixture in &parsed.invalid_fixtures {
        let _ = (
            &fixture.classification,
            &fixture.concept,
            &fixture.id,
            &fixture.reason,
            &fixture.value,
        );
    }
    validate_limit_values(&parsed.limits)?;
    Ok(())
}

fn array_length(value: &Value, pointer: &str) -> Result<usize, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(Vec::len)
        .ok_or_else(|| format!("{pointer} must be an array"))
}

fn ensure_at_most(count: usize, limit: usize, label: &str) -> Result<(), String> {
    if count > limit {
        Err(format!("{label} exceeded"))
    } else {
        Ok(())
    }
}

fn validate_payload_limits(
    root: &serde_json::Map<String, Value>,
    concept: &str,
    value: &Value,
) -> Result<(), String> {
    let checks = match concept {
        "layer" => vec![(array_length(value, "/layers")?, "maxLayersPerComposition")],
        "component" => vec![
            (
                array_length(value, "/definition/layers")?,
                "maxLayersPerComposition",
            ),
            (
                array_length(value, "/definition/slotIds")?,
                "maxSlotsPerComponent",
            ),
            (
                array_length(value, "/definition/markerIds")?,
                "maxMarkersPerComposition",
            ),
        ],
        "marker" => Vec::new(),
        "curve" => vec![(array_length(value, "/keyframes")?, "maxKeyframesPerChannel")],
        "mask" => vec![(array_length(value, "/masks")?, "maxMasksPerLayer")],
        "effect" => vec![(array_length(value, "/effects")?, "maxEffectsPerLayer")],
        "audio_event" => Vec::new(),
        "slot" | "transform" => Vec::new(),
        _ => return Err(format!("unknown concept: {concept}")),
    };
    for (count, key) in checks {
        ensure_at_most(count, catalog_limit(root, key)?, key)?;
    }
    Ok(())
}

#[derive(Default)]
struct AggregateCounts {
    audio_events: BTreeMap<String, usize>,
    components: usize,
    layers: BTreeMap<String, usize>,
    markers: BTreeMap<String, usize>,
    slots: BTreeMap<String, usize>,
}

fn increment_scoped_count(
    counts: &mut BTreeMap<String, usize>,
    scope: &str,
    limit: usize,
    label: &str,
) -> Result<(), String> {
    let count = counts.entry(scope.to_owned()).or_default();
    *count += 1;
    ensure_at_most(*count, limit, label)
}

fn count_aggregate_definition(
    counts: &mut AggregateCounts,
    definition: &Reference,
    root: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    match definition.kind {
        ReferenceKind::Component => {
            counts.components += 1;
            ensure_at_most(
                counts.components,
                catalog_limit(root, "maxComponentDefinitions")?,
                "maxComponentDefinitions",
            )
        }
        ReferenceKind::Layer => increment_scoped_count(
            &mut counts.layers,
            &definition.scope,
            catalog_limit(root, "maxLayersPerComposition")?,
            "maxLayersPerComposition",
        ),
        ReferenceKind::Marker => increment_scoped_count(
            &mut counts.markers,
            &definition.scope,
            catalog_limit(root, "maxMarkersPerComposition")?,
            "maxMarkersPerComposition",
        ),
        ReferenceKind::Slot => increment_scoped_count(
            &mut counts.slots,
            &definition.scope,
            catalog_limit(root, "maxSlotsPerComponent")?,
            "maxSlotsPerComponent",
        ),
        ReferenceKind::AudioEvent => increment_scoped_count(
            &mut counts.audio_events,
            &definition.scope,
            catalog_limit(root, "maxAudioEventsPerComposition")?,
            "maxAudioEventsPerComposition",
        ),
        _ => Ok(()),
    }
}

fn validate_unique_fixture_ids(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let mut fixture_ids = BTreeSet::new();
    for collection in ["validFixtures", "invalidFixtures"] {
        for fixture in root[collection]
            .as_array()
            .ok_or_else(|| format!("{collection} must be an array"))?
        {
            let id = fixture["id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .ok_or_else(|| format!("{collection} fixture id must be a non-empty string"))?;
            if !fixture_ids.insert(id) {
                return Err(format!("duplicate fixture id: {id}"));
            }
        }
    }
    Ok(())
}

fn expected_invalid() -> BTreeMap<&'static str, (&'static str, &'static str, &'static str)> {
    BTreeMap::from([
        (
            "audio_event.ambiguous_bus",
            ("audio_event", "ambiguous_reference", "audio_bus_ambiguous"),
        ),
        (
            "audio_event.ambiguous_marker",
            ("audio_event", "ambiguous_reference", "marker_ambiguous"),
        ),
        (
            "audio_event.ambiguous_sound_definition",
            (
                "audio_event",
                "ambiguous_reference",
                "sound_definition_ambiguous",
            ),
        ),
        (
            "audio_event.ambiguous_variant",
            (
                "audio_event",
                "ambiguous_reference",
                "sound_variant_ambiguous",
            ),
        ),
        (
            "audio_event.missing_bus",
            ("audio_event", "missing_reference", "audio_bus_not_found"),
        ),
        (
            "audio_event.missing_marker",
            ("audio_event", "missing_reference", "marker_not_found"),
        ),
        (
            "audio_event.missing_sound_definition",
            (
                "audio_event",
                "missing_reference",
                "sound_definition_not_found",
            ),
        ),
        (
            "audio_event.missing_variant",
            (
                "audio_event",
                "missing_reference",
                "sound_variant_not_found",
            ),
        ),
        (
            "audio_event.network_variant",
            ("audio_event", "invalid_input", "network_resource_forbidden"),
        ),
        (
            "component.depth_limit",
            ("component", "invalid_input", "max_component_depth_exceeded"),
        ),
        (
            "component.missing_definition",
            ("component", "missing_reference", "component_not_found"),
        ),
        (
            "component.recursive",
            ("component", "reference_cycle", "component_cycle"),
        ),
        (
            "curve.invalid_spring",
            ("curve", "invalid_input", "spring_parameter_out_of_range"),
        ),
        (
            "effect.limit_exceeded",
            ("effect", "invalid_input", "max_effects_per_layer_exceeded"),
        ),
        (
            "effect.renderer_expression",
            ("effect", "invalid_input", "renderer_expression_forbidden"),
        ),
        (
            "layer.cross_scope_parent",
            ("layer", "invalid_input", "parent_scope_mismatch"),
        ),
        (
            "layer.direct_parent_cycle",
            ("layer", "reference_cycle", "direct_parent_cycle"),
        ),
        (
            "layer.missing_parent",
            ("layer", "missing_reference", "parent_not_found"),
        ),
        (
            "layer.parent_cycle",
            ("layer", "reference_cycle", "parent_cycle"),
        ),
        (
            "marker.ambiguous_name",
            ("marker", "ambiguous_reference", "duplicate_marker_name"),
        ),
        (
            "mask.arbitrary_path",
            ("mask", "invalid_input", "arbitrary_path_forbidden"),
        ),
        (
            "mask.missing_source",
            ("mask", "missing_reference", "mask_source_not_found"),
        ),
        (
            "mask.unsafe_svg",
            ("mask", "invalid_input", "executable_svg"),
        ),
        (
            "slot.arbitrary_path",
            ("slot", "invalid_input", "unstable_binding_target"),
        ),
        (
            "slot.constraint_violation",
            ("slot", "invalid_input", "slot_constraint_violation"),
        ),
        (
            "slot.invalid_default",
            ("slot", "invalid_input", "slot_default_type_mismatch"),
        ),
        (
            "slot.missing_target",
            ("slot", "missing_reference", "slot_target_not_found"),
        ),
        (
            "slot.required_value_missing",
            ("slot", "invalid_input", "required_slot_value_missing"),
        ),
        (
            "slot.type_mismatch",
            ("slot", "invalid_input", "slot_value_type_mismatch"),
        ),
        (
            "transform.non_finite",
            ("transform", "invalid_input", "non_finite_value"),
        ),
    ])
}

pub fn validate_catalog(catalog: &Value) -> Result<(), String> {
    validate_closed_envelope(catalog)?;
    let root = catalog
        .as_object()
        .ok_or_else(|| "catalog must be an object".to_string())?;
    validate_unique_fixture_ids(root)?;
    if root.get("version").and_then(Value::as_u64) != Some(1)
        || root.get("status").and_then(Value::as_str) != Some("fixture_only")
    {
        return Err("motion-graphics catalog version/status differs".into());
    }
    let limits: LimitsEnvelope = serde_json::from_value(root["limits"].clone())
        .map_err(|error| format!("limits: {error}"))?;
    validate_limit_values(&limits)?;
    let max_parent_depth = limit_usize(limits.max_parent_depth, "maxParentDepth")?;
    let mut definitions = managed_asset_references(&root["managedResources"])?;
    let mut all_references = BTreeSet::new();
    let mut aggregate_counts = AggregateCounts::default();
    for fixture in root["validFixtures"]
        .as_array()
        .ok_or_else(|| "validFixtures must be an array".to_string())?
    {
        let id = fixture["id"]
            .as_str()
            .ok_or_else(|| "valid fixture id must be a string".to_string())?;
        let concept = fixture["concept"]
            .as_str()
            .ok_or_else(|| format!("{id} concept must be a string"))?;
        let derived = derive_payload(concept, fixture["value"].clone(), max_parent_depth)
            .map_err(|error| format!("{id}: {error}"))?;
        validate_payload_limits(root, concept, &fixture["value"])
            .map_err(|error| format!("{id}: {error}"))?;
        let declared_defines = metadata_references(&fixture["defines"], &format!("{id}.defines"))?;
        let declared_references =
            metadata_references(&fixture["references"], &format!("{id}.references"))?;
        if declared_defines != derived.defines {
            return Err(format!("{id}.defines metadata differs from payload"));
        }
        if declared_references != derived.references {
            return Err(format!("{id}.references metadata differs from payload"));
        }
        for definition in derived.defines {
            if !definitions.insert(definition.clone()) {
                return Err(format!("duplicate logical definition: {definition:?}"));
            }
            count_aggregate_definition(&mut aggregate_counts, &definition, root)
                .map_err(|error| format!("{id}: {error}"))?;
        }
        all_references.extend(derived.references);
    }
    for reference in all_references {
        if !definitions.contains(&reference) {
            return Err(format!("unresolved logical reference: {reference:?}"));
        }
    }

    let expected = expected_invalid();
    let mut observed = BTreeMap::new();
    for fixture in root["invalidFixtures"]
        .as_array()
        .ok_or_else(|| "invalidFixtures must be an array".to_string())?
    {
        let id = fixture["id"]
            .as_str()
            .ok_or_else(|| "invalid fixture id must be a string".to_string())?;
        let concept = fixture["concept"]
            .as_str()
            .ok_or_else(|| format!("{id} concept must be a string"))?;
        let (expected_concept, _, _) = expected
            .get(id)
            .copied()
            .ok_or_else(|| format!("unexpected invalid fixture ID: {id}"))?;
        if concept != expected_concept {
            return Err(format!(
                "{id} concept mismatch: expected {expected_concept}, received {concept}"
            ));
        }
        let actual = classify_invalid(id, fixture["value"].clone(), &limits)
            .map_err(|error| format!("{id} invalid envelope: {error}"))?;
        let classification = fixture["classification"]
            .as_str()
            .ok_or_else(|| format!("{id} classification must be a string"))?;
        let reason = fixture["reason"]
            .as_str()
            .ok_or_else(|| format!("{id} reason must be a string"))?;
        if actual != (classification, reason) {
            return Err(format!(
                "{id} observed failure differs from declared failure"
            ));
        }
        observed.insert(id, (concept, actual.0, actual.1));
    }
    if observed != expected {
        return Err("invalid fixture IDs/classifications/reasons differ".into());
    }
    Ok(())
}

fn valid_fixtures_mut(catalog: &mut Value) -> Result<&mut Vec<Value>, String> {
    catalog["validFixtures"]
        .as_array_mut()
        .ok_or_else(|| "validFixtures must be an array".to_string())
}

fn append_aggregate_component(catalog: &mut Value, suffix: &str) -> Result<(), String> {
    let component_id = format!("aggregate_component_{suffix}");
    let layer_id = format!("aggregate_layer_{suffix}");
    valid_fixtures_mut(catalog)?.push(serde_json::json!({
        "concept": "component",
        "defines": [
            { "id": component_id, "kind": "component", "scope": "project" },
            {
                "id": layer_id,
                "kind": "layer",
                "scope": format!("component:{component_id}"),
            },
        ],
        "id": format!("component.aggregate_{suffix}"),
        "references": [
            { "id": component_id, "kind": "component", "scope": "project" },
        ],
        "value": {
            "definition": {
                "durationMs": 1000,
                "height": 1080,
                "id": component_id,
                "layers": [layer_id],
                "markerIds": [],
                "name": format!("Aggregate component {suffix}"),
                "slotIds": [],
                "trackIds": [format!("aggregate_track_{suffix}")],
                "width": 1920,
            },
            "instance": {
                "componentId": component_id,
                "durationMs": 1000,
                "id": format!("aggregate_instance_{suffix}"),
                "slotValues": {},
                "startMs": 0,
                "timeScale": 1,
                "trimStartMs": 0,
            },
        },
    }));
    Ok(())
}

fn append_aggregate_layer(catalog: &mut Value, suffix: &str) -> Result<(), String> {
    let layer_id = format!("aggregate_root_layer_{suffix}");
    valid_fixtures_mut(catalog)?.push(serde_json::json!({
        "concept": "layer",
        "defines": [{ "id": layer_id, "kind": "layer", "scope": "root" }],
        "id": format!("layer.aggregate_{suffix}"),
        "references": [],
        "value": {
            "layers": [{
                "animationChannels": [],
                "blendMode": "normal",
                "clip": null,
                "effects": [],
                "hidden": false,
                "id": layer_id,
                "masks": [],
                "parentId": null,
                "stableItemIndex": 0,
                "transformId": null,
                "zIndex": 0,
            }],
            "scope": "root",
        },
    }));
    Ok(())
}

fn append_aggregate_marker(catalog: &mut Value, suffix: &str) -> Result<(), String> {
    let marker_id = format!("aggregate_marker_{suffix}");
    valid_fixtures_mut(catalog)?.push(serde_json::json!({
        "concept": "marker",
        "defines": [{
            "id": marker_id,
            "kind": "marker",
            "scope": "component:rule_card",
        }],
        "id": format!("marker.aggregate_{suffix}"),
        "references": [],
        "value": {
            "absoluteTime": { "type": "milliseconds", "valueMs": 0 },
            "marker": {
                "id": marker_id,
                "kind": "cue",
                "name": marker_id,
                "scope": "component:rule_card",
                "timeMs": 0,
            },
            "relativeTime": { "type": "milliseconds", "valueMs": 0 },
        },
    }));
    Ok(())
}

fn append_aggregate_slot(catalog: &mut Value, suffix: &str) -> Result<(), String> {
    let slot_id = format!("aggregate_slot_{suffix}");
    valid_fixtures_mut(catalog)?.push(serde_json::json!({
        "concept": "slot",
        "defines": [{ "id": slot_id, "kind": "slot", "scope": "component:rule_card" }],
        "id": format!("slot.aggregate_{suffix}"),
        "references": [{
            "id": "card_title",
            "kind": "layer",
            "scope": "component:rule_card",
        }],
        "value": {
            "binding": { "property": "text.document", "targetLayerId": "card_title" },
            "constraints": { "maxLength": 100, "minLength": 0 },
            "defaultValue": "",
            "id": slot_id,
            "kind": "text",
            "name": format!("Aggregate slot {suffix}"),
            "required": false,
            "scope": "component:rule_card",
        },
    }));
    Ok(())
}

fn append_aggregate_audio_event(catalog: &mut Value, suffix: &str) -> Result<(), String> {
    let asset_id = format!("aggregate_asset_{suffix}");
    let bus_id = format!("aggregate_bus_{suffix}");
    let event_id = format!("aggregate_event_{suffix}");
    let sound_id = format!("aggregate_sound_{suffix}");
    catalog["managedResources"]
        .as_array_mut()
        .ok_or_else(|| "managedResources must be an array".to_string())?
        .push(serde_json::json!({
            "id": asset_id,
            "kind": "asset",
            "scope": "project",
        }));
    valid_fixtures_mut(catalog)?.push(serde_json::json!({
        "concept": "audio_event",
        "defines": [
            { "id": bus_id, "kind": "audio_bus", "scope": "project" },
            { "id": sound_id, "kind": "sound_definition", "scope": "project" },
            {
                "id": event_id,
                "kind": "audio_event",
                "scope": "component:rule_card",
            },
        ],
        "id": format!("audio_event.aggregate_{suffix}"),
        "references": [
            { "id": asset_id, "kind": "asset", "scope": "project" },
            { "id": bus_id, "kind": "audio_bus", "scope": "project" },
            { "id": sound_id, "kind": "sound_definition", "scope": "project" },
        ],
        "value": {
            "bus": { "id": bus_id },
            "event": {
                "at": { "type": "milliseconds", "valueMs": 0 },
                "busId": bus_id,
                "event": sound_id,
                "gainDb": 0,
                "id": event_id,
                "scope": "component:rule_card",
                "variantSeed": 0,
            },
            "soundDefinition": {
                "busId": bus_id,
                "defaultGainDb": 0,
                "event": sound_id,
                "variantAssetIds": [asset_id],
            },
        },
    }));
    Ok(())
}

fn set_limit(catalog: &mut Value, name: &str, value: u64) {
    catalog["limits"][name] = Value::from(value);
}

fn expect_catalog_failure(
    catalog: &Value,
    label: &str,
    expected_invariant: &str,
) -> Result<(), String> {
    match validate_catalog(catalog) {
        Ok(()) => Err(format!("{label} unexpectedly passed validation")),
        Err(error) if error.contains(expected_invariant) => Ok(()),
        Err(error) => Err(format!(
            "{label} failed for an unrelated invariant: expected {expected_invariant}, received {error}"
        )),
    }
}

fn mutate_fixture_value<F>(
    catalog: &mut Value,
    collection: &str,
    fixture_id: &str,
    mutate: F,
) -> Result<(), String>
where
    F: FnOnce(&mut Value),
{
    let fixture = catalog[collection]
        .as_array_mut()
        .ok_or_else(|| format!("{collection} must be an array"))?
        .iter_mut()
        .find(|fixture| fixture["id"] == fixture_id)
        .ok_or_else(|| format!("fixture {fixture_id} missing"))?;
    mutate(&mut fixture["value"]);
    Ok(())
}

fn expect_invalid_envelope_mutation<F>(
    catalog: &Value,
    fixture_id: &str,
    label: &str,
    mutate: F,
) -> Result<(), String>
where
    F: FnOnce(&mut Value),
{
    let mut candidate = catalog.clone();
    mutate_fixture_value(&mut candidate, "invalidFixtures", fixture_id, mutate)?;
    expect_catalog_failure(&candidate, label, &format!("{fixture_id} invalid envelope"))
}

pub fn validate_malformed_regressions(catalog: &Value) -> Result<(), String> {
    let fixtures = catalog["validFixtures"]
        .as_array()
        .ok_or_else(|| "validFixtures must be an array".to_string())?;
    let transform = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "transform.complete")
        .ok_or_else(|| "transform fixture missing".to_string())?;
    let mut wrong_type = transform["value"].clone();
    wrong_type["opacity"] = Value::String("opaque".into());
    if derive_transform(wrong_type).is_ok() {
        return Err("string opacity passed strict validation".into());
    }
    let mut unknown_field = transform["value"].clone();
    unknown_field["rawPath"] = Value::String("/tmp/x".into());
    if derive_transform(unknown_field).is_ok() {
        return Err("unknown resource field passed strict validation".into());
    }
    let mut missing = transform["value"].clone();
    missing
        .as_object_mut()
        .ok_or_else(|| "transform value must be an object".to_string())?
        .remove("position");
    if derive_transform(missing).is_ok() {
        return Err("missing required field passed strict validation".into());
    }
    for unsafe_resource in [
        "/tmp/mask.svg",
        "C:/outside/mask.svg",
        r"C:\outside\mask.svg",
        r"\\server\share\mask.svg",
        "../mask.svg",
        "https://example.invalid/mask.svg",
        "filter_complex=overlay",
    ] {
        let mut candidate = transform["value"].clone();
        candidate["resource"] = Value::String(unsafe_resource.into());
        if derive_transform(candidate).is_ok() {
            return Err(format!("unsafe resource field passed: {unsafe_resource}"));
        }
    }
    let slot = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "slot.typed_title")
        .ok_or_else(|| "slot fixture missing".to_string())?;
    let mut text = slot["value"].clone();
    text["defaultValue"] = Value::String("Read https://example.com safely".into());
    derive_slot(text).map_err(|error| format!("ordinary URL-like text was rejected: {error}"))?;

    let component = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "component.rule_card")
        .ok_or_else(|| "component fixture missing".to_string())?;
    let derived = derive_component(component["value"].clone())?;
    let mut drifted = component["references"].clone();
    drifted[1]["scope"] = Value::String("root".into());
    if metadata_references(&drifted, "scope mismatch")
        .is_ok_and(|references| references == derived.references)
    {
        return Err("cross-scope metadata drift passed validation".into());
    }

    let mut duplicate_resource = catalog.clone();
    let resource = duplicate_resource["managedResources"][0].clone();
    duplicate_resource["managedResources"]
        .as_array_mut()
        .ok_or_else(|| "managedResources must be an array".to_string())?
        .push(resource);
    expect_catalog_failure(
        &duplicate_resource,
        "duplicate managed resource",
        "managedResources contains duplicates",
    )?;

    let mut non_asset_resource = catalog.clone();
    non_asset_resource["managedResources"]
        .as_array_mut()
        .ok_or_else(|| "managedResources must be an array".to_string())?
        .push(serde_json::json!({
            "id": "not_an_asset",
            "kind": "component",
            "scope": "project",
        }));
    expect_catalog_failure(
        &non_asset_resource,
        "non-asset managed resource",
        "expected `asset`",
    )?;

    let mut non_project_resource = catalog.clone();
    non_project_resource["managedResources"]
        .as_array_mut()
        .ok_or_else(|| "managedResources must be an array".to_string())?
        .push(serde_json::json!({
            "id": "wrong_scope_asset",
            "kind": "asset",
            "scope": "root",
        }));
    expect_catalog_failure(
        &non_project_resource,
        "non-project managed resource",
        "expected `project`",
    )?;

    let mut unmanaged_resource = catalog.clone();
    unmanaged_resource["managedResources"]
        .as_array_mut()
        .ok_or_else(|| "managedResources must be an array".to_string())?
        .retain(|resource| resource["id"] != "sfx_impact_a");
    expect_catalog_failure(
        &unmanaged_resource,
        "unmanaged resource reference",
        "unresolved logical reference",
    )?;

    for (target_collection, source_collection) in [
        ("validFixtures", "validFixtures"),
        ("invalidFixtures", "invalidFixtures"),
        ("invalidFixtures", "validFixtures"),
    ] {
        let mut duplicate_id = catalog.clone();
        let source_id = duplicate_id[source_collection][0]["id"].clone();
        if target_collection == source_collection {
            let duplicate = duplicate_id[source_collection][0].clone();
            duplicate_id[target_collection]
                .as_array_mut()
                .ok_or_else(|| format!("{target_collection} must be an array"))?
                .push(duplicate);
        } else {
            duplicate_id[target_collection][0]["id"] = source_id;
        }
        expect_catalog_failure(
            &duplicate_id,
            "duplicate fixture ID",
            "duplicate fixture id",
        )?;
    }

    let mut mislabeled_invalid = catalog.clone();
    let mislabeled = mislabeled_invalid["invalidFixtures"]
        .as_array_mut()
        .ok_or_else(|| "invalidFixtures must be an array".to_string())?
        .iter_mut()
        .find(|fixture| fixture["id"] == "slot.required_value_missing")
        .ok_or_else(|| "required slot fixture missing".to_string())?;
    mislabeled["concept"] = Value::String("layer".into());
    expect_catalog_failure(
        &mislabeled_invalid,
        "mislabeled invalid fixture",
        "concept mismatch",
    )?;

    for (fixture_id, collection) in [
        ("layer.ordered_visual", "layers"),
        ("mask.ordered_pair", "masks"),
        ("effect.ordered_stack", "effects"),
    ] {
        let mut duplicate_definition = catalog.clone();
        mutate_fixture_value(
            &mut duplicate_definition,
            "validFixtures",
            fixture_id,
            |value| {
                let duplicate = value[collection][0].clone();
                value[collection].as_array_mut().unwrap().push(duplicate);
            },
        )?;
        expect_catalog_failure(
            &duplicate_definition,
            &format!("duplicate {fixture_id} payload definition"),
            "duplicate payload definition",
        )?;
    }

    for (fixture_id, label, invariant, mutate) in [
        (
            "component.recursive",
            "duplicate invalid component ID",
            "component invalid envelope duplicate payload definition",
            (|value: &mut Value| {
                value["componentIds"]
                    .as_array_mut()
                    .unwrap()
                    .push(Value::String("a".into()));
            }) as fn(&mut Value),
        ),
        (
            "component.recursive",
            "duplicate invalid dependency edge",
            "component invalid envelope dependency edge duplicate payload definition",
            |value: &mut Value| {
                value["dependencies"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({ "from": "a", "to": "b" }));
            },
        ),
        (
            "slot.required_value_missing",
            "duplicate invalid slot target",
            "slot invalid envelope target layer duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["targetLayerIds"][0].clone();
                value["targetLayerIds"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            },
        ),
        (
            "marker.ambiguous_name",
            "duplicate invalid marker ID",
            "marker invalid envelope duplicate payload definition",
            |value: &mut Value| {
                value["markers"][1]["id"] = value["markers"][0]["id"].clone();
            },
        ),
        (
            "mask.missing_source",
            "duplicate invalid mask context layer",
            "mask invalid envelope available layer duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["availableLayerIds"][0].clone();
                value["availableLayerIds"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate invalid audio asset",
            "audio invalid envelope asset duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["assets"][0].clone();
                value["assets"].as_array_mut().unwrap().push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate invalid audio event",
            "audio invalid envelope event duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["events"][0].clone();
                value["events"].as_array_mut().unwrap().push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate invalid audio marker",
            "audio invalid envelope marker duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["markers"][0].clone();
                value["markers"].as_array_mut().unwrap().push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate unrelated audio bus",
            "audio invalid envelope bus duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["buses"][0].clone();
                value["buses"].as_array_mut().unwrap().push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate unrelated sound definition",
            "audio invalid envelope sound definition duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["soundDefinitions"][0].clone();
                value["soundDefinitions"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            },
        ),
        (
            "audio_event.missing_bus",
            "duplicate unrelated sound variant",
            "audio invalid envelope sound variant duplicate payload definition",
            |value: &mut Value| {
                let duplicate = value["soundDefinitions"][0]["variantAssetIds"][0].clone();
                value["soundDefinitions"][0]["variantAssetIds"]
                    .as_array_mut()
                    .unwrap()
                    .push(duplicate);
            },
        ),
    ] {
        let mut candidate = catalog.clone();
        mutate_fixture_value(&mut candidate, "invalidFixtures", fixture_id, mutate)?;
        expect_catalog_failure(&candidate, label, invariant)?;
    }

    let mut branching_cycle = catalog.clone();
    mutate_fixture_value(
        &mut branching_cycle,
        "invalidFixtures",
        "component.recursive",
        |value| {
            value["componentIds"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("c".into()));
            value["dependencies"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "from": "a", "to": "c" }));
        },
    )?;
    validate_catalog(&branching_cycle)?;

    let mut direct_cycle = catalog.clone();
    mutate_fixture_value(
        &mut direct_cycle,
        "invalidFixtures",
        "component.recursive",
        |value| value["dependencies"] = serde_json::json!([{ "from": "a", "to": "a" }]),
    )?;
    validate_catalog(&direct_cycle)?;

    let mut missing_entry = catalog.clone();
    mutate_fixture_value(
        &mut missing_entry,
        "invalidFixtures",
        "component.missing_definition",
        |value| {
            value["dependencies"] = Value::Array(Vec::new());
            value["entryId"] = Value::String("absent_entry".into());
        },
    )?;
    validate_catalog(&missing_entry)?;

    let mut branching_depth = catalog.clone();
    mutate_fixture_value(
        &mut branching_depth,
        "invalidFixtures",
        "component.depth_limit",
        |value| {
            value["componentIds"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("short_branch".into()));
            value["dependencies"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "from": "c0", "to": "short_branch" }));
        },
    )?;
    validate_catalog(&branching_depth)?;

    let mut duplicate_edge = catalog.clone();
    mutate_fixture_value(
        &mut duplicate_edge,
        "invalidFixtures",
        "component.recursive",
        |value| {
            value["dependencies"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "from": "a", "to": "b" }));
        },
    )?;
    expect_catalog_failure(
        &duplicate_edge,
        "duplicate branching dependency edge",
        "component invalid envelope dependency edge duplicate payload definition",
    )?;

    let mut duplicate_component_layer = catalog.clone();
    mutate_fixture_value(
        &mut duplicate_component_layer,
        "validFixtures",
        "component.rule_card",
        |value| {
            let duplicate = value["definition"]["layers"][0].clone();
            value["definition"]["layers"]
                .as_array_mut()
                .unwrap()
                .push(duplicate);
        },
    )?;
    expect_catalog_failure(
        &duplicate_component_layer,
        "duplicate component-layer payload definition",
        "duplicate payload definition",
    )?;

    for (fixture_id, collection) in [
        ("layer.missing_parent", "layers"),
        ("mask.unsafe_svg", "masks"),
        ("effect.renderer_expression", "effects"),
    ] {
        let mut duplicate_definition = catalog.clone();
        mutate_fixture_value(
            &mut duplicate_definition,
            "invalidFixtures",
            fixture_id,
            |value| {
                let duplicate = value[collection][0].clone();
                value[collection].as_array_mut().unwrap().push(duplicate);
            },
        )?;
        expect_catalog_failure(
            &duplicate_definition,
            &format!("duplicate {fixture_id} invalid definition"),
            "duplicate payload definition",
        )?;
    }

    let mut invalid_component_boundary = catalog.clone();
    set_limit(
        &mut invalid_component_boundary,
        "maxComponentDefinitions",
        18,
    );
    validate_catalog(&invalid_component_boundary)?;
    let mut invalid_component_overflow = invalid_component_boundary.clone();
    mutate_fixture_value(
        &mut invalid_component_overflow,
        "invalidFixtures",
        "component.depth_limit",
        |value| {
            value["componentIds"]
                .as_array_mut()
                .unwrap()
                .push(Value::String("overflow_component".into()));
        },
    )?;
    expect_catalog_failure(
        &invalid_component_overflow,
        "invalid component-definition limit",
        "maxComponentDefinitions",
    )?;

    let mut invalid_layer_boundary = catalog.clone();
    set_limit(&mut invalid_layer_boundary, "maxLayersPerComposition", 3);
    mutate_fixture_value(
        &mut invalid_layer_boundary,
        "invalidFixtures",
        "layer.missing_parent",
        |value| {
            for id in ["boundary_layer_one", "boundary_layer_two"] {
                let mut layer = value["layers"][0].clone();
                layer["id"] = Value::String(id.into());
                layer["parentId"] = Value::Null;
                value["layers"].as_array_mut().unwrap().push(layer);
            }
        },
    )?;
    validate_catalog(&invalid_layer_boundary)?;
    let mut invalid_layer_overflow = invalid_layer_boundary.clone();
    mutate_fixture_value(
        &mut invalid_layer_overflow,
        "invalidFixtures",
        "layer.missing_parent",
        |value| {
            let mut layer = value["layers"][0].clone();
            layer["id"] = Value::String("overflow_layer".into());
            layer["parentId"] = Value::Null;
            value["layers"].as_array_mut().unwrap().push(layer);
        },
    )?;
    expect_catalog_failure(
        &invalid_layer_overflow,
        "invalid layer limit",
        "maxLayersPerComposition",
    )?;

    let mut invalid_marker_boundary = catalog.clone();
    set_limit(&mut invalid_marker_boundary, "maxMarkersPerComposition", 2);
    validate_catalog(&invalid_marker_boundary)?;
    let mut invalid_marker_overflow = invalid_marker_boundary.clone();
    mutate_fixture_value(
        &mut invalid_marker_overflow,
        "invalidFixtures",
        "marker.ambiguous_name",
        |value| {
            value["markers"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "id": "overflow_marker",
                    "kind": "cue",
                    "name": "unrelated_marker",
                    "scope": "component:rule_card",
                    "timeMs": 300,
                }));
        },
    )?;
    expect_catalog_failure(
        &invalid_marker_overflow,
        "invalid marker limit",
        "maxMarkersPerComposition",
    )?;

    let mut invalid_keyframe_boundary = catalog.clone();
    set_limit(&mut invalid_keyframe_boundary, "maxKeyframesPerChannel", 2);
    mutate_fixture_value(
        &mut invalid_keyframe_boundary,
        "invalidFixtures",
        "curve.invalid_spring",
        |value| {
            value["keyframes"] = serde_json::json!([
                { "curve": { "type": "linear" }, "time": { "type": "milliseconds", "valueMs": 0 }, "value": 0 },
                { "curve": { "type": "hold" }, "time": { "type": "milliseconds", "valueMs": 1 }, "value": 1 }
            ]);
        },
    )?;
    validate_catalog(&invalid_keyframe_boundary)?;
    let mut invalid_keyframe_overflow = invalid_keyframe_boundary.clone();
    mutate_fixture_value(
        &mut invalid_keyframe_overflow,
        "invalidFixtures",
        "curve.invalid_spring",
        |value| {
            value["keyframes"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "curve": { "type": "linear" },
                    "time": { "type": "milliseconds", "valueMs": 2 },
                    "value": 2,
                }));
        },
    )?;
    expect_catalog_failure(
        &invalid_keyframe_overflow,
        "invalid keyframe limit",
        "maxKeyframesPerChannel",
    )?;

    let safe_mask = |id: &str| {
        serde_json::json!({
            "channel": "alpha",
            "expansionPx": 0,
            "featherPx": 0,
            "id": id,
            "inverted": false,
            "operation": "add",
            "source": { "commands": [{ "type": "close" }], "type": "path" },
            "transformId": "hero",
        })
    };
    let mut invalid_mask_boundary = catalog.clone();
    set_limit(&mut invalid_mask_boundary, "maxMasksPerLayer", 16);
    mutate_fixture_value(
        &mut invalid_mask_boundary,
        "invalidFixtures",
        "mask.missing_source",
        |value| {
            for index in 1..16 {
                value["masks"]
                    .as_array_mut()
                    .unwrap()
                    .push(safe_mask(&format!("boundary_mask_{index}")));
            }
        },
    )?;
    validate_catalog(&invalid_mask_boundary)?;
    let mut invalid_mask_overflow = invalid_mask_boundary.clone();
    mutate_fixture_value(
        &mut invalid_mask_overflow,
        "invalidFixtures",
        "mask.missing_source",
        |value| {
            value["masks"]
                .as_array_mut()
                .unwrap()
                .push(safe_mask("overflow_mask"));
        },
    )?;
    expect_catalog_failure(
        &invalid_mask_overflow,
        "invalid mask limit",
        "maxMasksPerLayer",
    )?;

    let mut invalid_effect_boundary = catalog.clone();
    set_limit(&mut invalid_effect_boundary, "maxEffectsPerLayer", 2);
    mutate_fixture_value(
        &mut invalid_effect_boundary,
        "invalidFixtures",
        "effect.renderer_expression",
        |value| {
            value["effects"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "expression": "opacity * 0.5",
                    "id": "boundary_expression",
                    "type": "renderer_expression",
                }));
        },
    )?;
    validate_catalog(&invalid_effect_boundary)?;
    let mut invalid_effect_overflow = invalid_effect_boundary.clone();
    mutate_fixture_value(
        &mut invalid_effect_overflow,
        "invalidFixtures",
        "effect.renderer_expression",
        |value| {
            value["effects"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "expression": "opacity * 0.25",
                    "id": "overflow_expression",
                    "type": "renderer_expression",
                }));
        },
    )?;
    expect_catalog_failure(
        &invalid_effect_overflow,
        "invalid effect limit",
        "maxEffectsPerLayer",
    )?;

    let mut invalid_audio_boundary = catalog.clone();
    set_limit(
        &mut invalid_audio_boundary,
        "maxAudioEventsPerComposition",
        2,
    );
    mutate_fixture_value(
        &mut invalid_audio_boundary,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["id"] = Value::String("impact_02".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    validate_catalog(&invalid_audio_boundary)?;
    let mut invalid_audio_overflow = invalid_audio_boundary.clone();
    mutate_fixture_value(
        &mut invalid_audio_overflow,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["id"] = Value::String("impact_03".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    expect_catalog_failure(
        &invalid_audio_overflow,
        "invalid audio-event limit",
        "maxAudioEventsPerComposition",
    )?;

    for fixture_id in [
        "audio_event.missing_bus",
        "audio_event.ambiguous_bus",
        "audio_event.missing_marker",
        "audio_event.ambiguous_marker",
        "audio_event.missing_sound_definition",
        "audio_event.ambiguous_sound_definition",
        "audio_event.missing_variant",
        "audio_event.ambiguous_variant",
        "audio_event.network_variant",
    ] {
        let mut missing_definition_bus = catalog.clone();
        mutate_fixture_value(
            &mut missing_definition_bus,
            "invalidFixtures",
            fixture_id,
            |value| {
                value["soundDefinitions"][0]["busId"] = Value::String("unresolved_bus".into());
            },
        )?;
        expect_catalog_failure(
            &missing_definition_bus,
            &format!("{fixture_id} missing sound-definition bus"),
            "audio invalid envelope sound definition bus missing reference",
        )?;

        let mut restored_definition_bus = missing_definition_bus;
        mutate_fixture_value(
            &mut restored_definition_bus,
            "invalidFixtures",
            fixture_id,
            |value| {
                value["buses"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({ "id": "unresolved_bus" }));
            },
        )?;
        validate_catalog(&restored_definition_bus)?;
    }

    let mut scoped_events = catalog.clone();
    mutate_fixture_value(
        &mut scoped_events,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["at"] = serde_json::json!({ "type": "milliseconds", "valueMs": 0 });
            event["scope"] = Value::String("root".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    validate_catalog(&scoped_events)?;

    let mut scoped_markers = catalog.clone();
    mutate_fixture_value(
        &mut scoped_markers,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut marker = value["markers"][0].clone();
            marker["name"] = Value::String("root_impact".into());
            marker["scope"] = Value::String("root".into());
            value["markers"].as_array_mut().unwrap().push(marker);
        },
    )?;
    validate_catalog(&scoped_markers)?;

    let mut distributed_events = catalog.clone();
    set_limit(&mut distributed_events, "maxAudioEventsPerComposition", 1);
    mutate_fixture_value(
        &mut distributed_events,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["at"] = serde_json::json!({ "type": "milliseconds", "valueMs": 0 });
            event["id"] = Value::String("root_event".into());
            event["scope"] = Value::String("root".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    validate_catalog(&distributed_events)?;

    let mut scoped_event_boundary = catalog.clone();
    set_limit(
        &mut scoped_event_boundary,
        "maxAudioEventsPerComposition",
        2,
    );
    mutate_fixture_value(
        &mut scoped_event_boundary,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["id"] = Value::String("impact_02".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    validate_catalog(&scoped_event_boundary)?;
    let mut scoped_event_overflow = scoped_event_boundary.clone();
    mutate_fixture_value(
        &mut scoped_event_overflow,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut event = value["events"][0].clone();
            event["id"] = Value::String("impact_03".into());
            value["events"].as_array_mut().unwrap().push(event);
        },
    )?;
    expect_catalog_failure(
        &scoped_event_overflow,
        "same-owner audio-event overflow",
        "maxAudioEventsPerComposition",
    )?;

    let mut distributed_markers = catalog.clone();
    set_limit(&mut distributed_markers, "maxMarkersPerComposition", 2);
    mutate_fixture_value(
        &mut distributed_markers,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            for (id, scope) in [("root_marker", "root"), ("other_marker", "component:other")] {
                let mut marker = value["markers"][0].clone();
                marker["id"] = Value::String(id.into());
                marker["name"] = Value::String(id.into());
                marker["scope"] = Value::String(scope.into());
                value["markers"].as_array_mut().unwrap().push(marker);
            }
        },
    )?;
    validate_catalog(&distributed_markers)?;

    let mut scoped_marker_boundary = catalog.clone();
    set_limit(&mut scoped_marker_boundary, "maxMarkersPerComposition", 2);
    mutate_fixture_value(
        &mut scoped_marker_boundary,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            for id in ["root_one", "root_two"] {
                let mut marker = value["markers"][0].clone();
                marker["id"] = Value::String(id.into());
                marker["name"] = Value::String(id.into());
                marker["scope"] = Value::String("root".into());
                value["markers"].as_array_mut().unwrap().push(marker);
            }
        },
    )?;
    validate_catalog(&scoped_marker_boundary)?;
    let mut scoped_marker_overflow = scoped_marker_boundary.clone();
    mutate_fixture_value(
        &mut scoped_marker_overflow,
        "invalidFixtures",
        "audio_event.missing_bus",
        |value| {
            let mut marker = value["markers"][0].clone();
            marker["id"] = Value::String("root_three".into());
            marker["name"] = Value::String("root_three".into());
            marker["scope"] = Value::String("root".into());
            value["markers"].as_array_mut().unwrap().push(marker);
        },
    )?;
    expect_catalog_failure(
        &scoped_marker_overflow,
        "same-owner marker overflow",
        "maxMarkersPerComposition",
    )?;

    for (fixture_id, invariant, mutate) in [
        (
            "marker.ambiguous_name",
            "marker invalid envelope name duplicate payload definition outside declared ambiguity",
            (|value: &mut Value| {
                for id in ["other_one", "other_two"] {
                    let mut marker = value["markers"][0].clone();
                    marker["id"] = Value::String(id.into());
                    marker["name"] = Value::String("other".into());
                    value["markers"].as_array_mut().unwrap().push(marker);
                }
            }) as fn(&mut Value),
        ),
        (
            "audio_event.ambiguous_bus",
            "audio invalid envelope bus duplicate payload definition outside declared ambiguity",
            |value: &mut Value| {
                value["buses"].as_array_mut().unwrap().extend([
                    serde_json::json!({ "id": "other" }),
                    serde_json::json!({ "id": "other" }),
                ]);
            },
        ),
        (
            "audio_event.ambiguous_marker",
            "audio invalid envelope marker name duplicate payload definition outside declared ambiguity",
            |value: &mut Value| {
                for id in ["other_one", "other_two"] {
                    let mut marker = value["markers"][0].clone();
                    marker["id"] = Value::String(id.into());
                    marker["name"] = Value::String("other".into());
                    value["markers"].as_array_mut().unwrap().push(marker);
                }
            },
        ),
        (
            "audio_event.ambiguous_sound_definition",
            "audio invalid envelope sound definition duplicate payload definition outside declared ambiguity",
            |value: &mut Value| {
                value["soundDefinitions"].as_array_mut().unwrap().extend([
                    serde_json::json!({ "busId": "sfx", "defaultGainDb": 0, "event": "other", "variantAssetIds": ["sfx_impact_a"] }),
                    serde_json::json!({ "busId": "sfx", "defaultGainDb": 0, "event": "other", "variantAssetIds": ["sfx_impact_a"] }),
                ]);
            },
        ),
        (
            "audio_event.ambiguous_variant",
            "audio invalid envelope sound variant duplicate payload definition outside declared ambiguity",
            |value: &mut Value| {
                value["soundDefinitions"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({
                        "busId": "sfx",
                        "defaultGainDb": 0,
                        "event": "other",
                        "variantAssetIds": ["sfx_impact_a", "sfx_impact_a"],
                    }));
            },
        ),
    ] {
        let mut candidate = catalog.clone();
        mutate_fixture_value(&mut candidate, "invalidFixtures", fixture_id, mutate)?;
        expect_catalog_failure(
            &candidate,
            &format!("{fixture_id} unrelated ambiguity duplicate"),
            invariant,
        )?;
    }

    for (fixture_id, mutate) in [
        (
            "marker.ambiguous_name",
            (|value: &mut Value| {
                let mut marker = value["markers"][0].clone();
                marker["id"] = Value::String("third_impact".into());
                value["markers"].as_array_mut().unwrap().push(marker);
            }) as fn(&mut Value),
        ),
        ("audio_event.ambiguous_bus", |value: &mut Value| {
            let bus_id = value["events"][0]["busId"].clone();
            value["buses"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({ "id": bus_id }));
        }),
        ("audio_event.ambiguous_marker", |value: &mut Value| {
            let mut marker = value["markers"][0].clone();
            marker["id"] = Value::String("third_marker".into());
            value["markers"].as_array_mut().unwrap().push(marker);
        }),
        (
            "audio_event.ambiguous_sound_definition",
            |value: &mut Value| {
                let definition = value["soundDefinitions"][0].clone();
                value["soundDefinitions"]
                    .as_array_mut()
                    .unwrap()
                    .push(definition);
            },
        ),
        ("audio_event.ambiguous_variant", |value: &mut Value| {
            let variant = value["soundDefinitions"][0]["variantAssetIds"][0].clone();
            value["soundDefinitions"][0]["variantAssetIds"]
                .as_array_mut()
                .unwrap()
                .push(variant);
        }),
    ] {
        let mut candidate = catalog.clone();
        mutate_fixture_value(&mut candidate, "invalidFixtures", fixture_id, mutate)?;
        validate_catalog(&candidate)?;
    }

    for svg in [
        "<svg onclick=\"run()\"></svg>",
        "<svg OnErRoR   =\"run()\"></svg>",
    ] {
        let mut handler_variant = catalog.clone();
        mutate_fixture_value(
            &mut handler_variant,
            "invalidFixtures",
            "mask.unsafe_svg",
            |value| value["masks"][0]["source"]["svg"] = Value::String(svg.into()),
        )?;
        validate_catalog(&handler_variant)
            .map_err(|error| format!("SVG event handler {svg} was not classified: {error}"))?;
    }

    let mut later_unsafe_mask = catalog.clone();
    mutate_fixture_value(
        &mut later_unsafe_mask,
        "invalidFixtures",
        "mask.unsafe_svg",
        |value| {
            value["masks"].as_array_mut().unwrap().insert(
                0,
                serde_json::json!({
                    "channel": "alpha",
                    "expansionPx": 0,
                    "featherPx": 0,
                    "id": "safe_mask",
                    "inverted": false,
                    "operation": "add",
                    "source": { "commands": [{ "type": "close" }], "type": "path" },
                    "transformId": "hero",
                }),
            );
        },
    )?;
    validate_catalog(&later_unsafe_mask)
        .map_err(|error| format!("non-first executable SVG was not classified: {error}"))?;

    for limit in [
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
    ] {
        let mut boundary = catalog.clone();
        set_limit(&mut boundary, limit, MAX_SAFE_INTEGER);
        validate_closed_envelope(&boundary)
            .map_err(|error| format!("{limit} safe-integer boundary failed: {error}"))?;
        let mut overflow = catalog.clone();
        set_limit(&mut overflow, limit, MAX_SAFE_INTEGER + 1);
        match validate_closed_envelope(&overflow) {
            Err(error) if error.contains(limit) && error.contains("JavaScript-safe integer") => {}
            Err(error) => {
                return Err(format!(
                    "{limit} overflow failed for an unrelated invariant: {error}"
                ));
            }
            Ok(()) => return Err(format!("{limit} unsafe limit passed validation")),
        }
    }

    expect_invalid_envelope_mutation(
        catalog,
        "transform.non_finite",
        "malformed transform invalid envelope",
        |value| value["position"]["x"] = Value::String("bad".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "layer.missing_parent",
        "malformed layer invalid envelope",
        |value| value["layers"][0]["stableItemIndex"] = Value::String("bad".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "component.recursive",
        "malformed component invalid envelope",
        |value| value["entryId"] = Value::String("bad.id".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "layer.missing_parent",
        "empty layer ID invalid envelope",
        |value| value["layers"][0]["id"] = Value::String(String::new()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "slot.required_value_missing",
        "malformed slot invalid envelope",
        |value| value["definition"]["name"] = Value::String(String::new()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "slot.required_value_missing",
        "illegal slot scope invalid envelope",
        |value| value["definition"]["scope"] = Value::String("root".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "slot.required_value_missing",
        "invalid slot constraints envelope",
        |value| {
            value["definition"]["constraints"]["minLength"] = Value::from(121);
            value["definition"]["constraints"]["maxLength"] = Value::from(120);
        },
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "marker.ambiguous_name",
        "malformed marker invalid envelope",
        |value| value["markers"][0]["timeMs"] = Value::from(MAX_SAFE_INTEGER + 1),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "marker.ambiguous_name",
        "missing marker collection invalid envelope",
        |value| value["markers"] = Value::Array(Vec::new()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "curve.invalid_spring",
        "malformed curve invalid envelope",
        |value| value["scope"] = Value::String("project".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "mask.missing_source",
        "malformed mask invalid envelope",
        |value| value["masks"][0]["transformId"] = Value::String("bad.id".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "mask.missing_source",
        "malformed numeric mask envelope",
        |value| value["masks"][0]["featherPx"] = Value::String("NaN".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "effect.renderer_expression",
        "malformed effect invalid envelope",
        |value| value["layerId"] = Value::String("bad.id".into()),
    )?;
    expect_invalid_envelope_mutation(
        catalog,
        "audio_event.missing_bus",
        "malformed audio invalid envelope",
        |value| value["events"][0]["gainDb"] = Value::from(25),
    )?;

    let mut unicode_name_boundary = catalog.clone();
    mutate_fixture_value(
        &mut unicode_name_boundary,
        "invalidFixtures",
        "slot.required_value_missing",
        |value| value["definition"]["name"] = Value::String("😀".repeat(200)),
    )?;
    validate_catalog(&unicode_name_boundary)?;
    expect_invalid_envelope_mutation(
        catalog,
        "slot.required_value_missing",
        "overlong astral-Unicode invalid envelope",
        |value| value["definition"]["name"] = Value::String("😀".repeat(201)),
    )?;

    let mut component_boundary = catalog.clone();
    set_limit(&mut component_boundary, "maxComponentDefinitions", 18);
    for index in 1..18 {
        append_aggregate_component(&mut component_boundary, &format!("boundary_{index}"))?;
    }
    validate_catalog(&component_boundary)?;
    let mut component_overflow = component_boundary.clone();
    append_aggregate_component(&mut component_overflow, "overflow")?;
    expect_catalog_failure(
        &component_overflow,
        "aggregate component limit",
        "maxComponentDefinitions",
    )?;

    let mut layer_boundary = catalog.clone();
    set_limit(&mut layer_boundary, "maxLayersPerComposition", 3);
    validate_catalog(&layer_boundary)?;
    append_aggregate_layer(&mut layer_boundary, "one")?;
    set_limit(&mut layer_boundary, "maxLayersPerComposition", 4);
    validate_catalog(&layer_boundary)?;
    let mut layer_overflow = layer_boundary.clone();
    append_aggregate_layer(&mut layer_overflow, "two")?;
    expect_catalog_failure(
        &layer_overflow,
        "aggregate layer limit",
        "maxLayersPerComposition",
    )?;

    let mut marker_boundary = catalog.clone();
    set_limit(&mut marker_boundary, "maxMarkersPerComposition", 2);
    append_aggregate_marker(&mut marker_boundary, "one")?;
    validate_catalog(&marker_boundary)?;
    let mut marker_overflow = marker_boundary.clone();
    append_aggregate_marker(&mut marker_overflow, "two")?;
    expect_catalog_failure(
        &marker_overflow,
        "aggregate marker limit",
        "maxMarkersPerComposition",
    )?;

    let mut slot_boundary = catalog.clone();
    set_limit(&mut slot_boundary, "maxSlotsPerComponent", 2);
    append_aggregate_slot(&mut slot_boundary, "one")?;
    validate_catalog(&slot_boundary)?;
    let mut slot_overflow = slot_boundary.clone();
    append_aggregate_slot(&mut slot_overflow, "two")?;
    expect_catalog_failure(
        &slot_overflow,
        "aggregate slot limit",
        "maxSlotsPerComponent",
    )?;

    let mut audio_boundary = catalog.clone();
    set_limit(&mut audio_boundary, "maxAudioEventsPerComposition", 2);
    append_aggregate_audio_event(&mut audio_boundary, "one")?;
    validate_catalog(&audio_boundary)?;
    let mut audio_overflow = audio_boundary.clone();
    append_aggregate_audio_event(&mut audio_overflow, "two")?;
    expect_catalog_failure(
        &audio_overflow,
        "aggregate audio-event limit",
        "maxAudioEventsPerComposition",
    )?;

    let mut swapped = catalog.clone();
    let invalid = swapped["invalidFixtures"]
        .as_array_mut()
        .ok_or_else(|| "invalidFixtures must be an array".to_string())?;
    let missing_bus = invalid
        .iter()
        .position(|fixture| fixture["id"] == "audio_event.missing_bus")
        .ok_or_else(|| "missing bus fixture missing".to_string())?;
    let non_finite = invalid
        .iter()
        .position(|fixture| fixture["id"] == "transform.non_finite")
        .ok_or_else(|| "non-finite fixture missing".to_string())?;
    let first = invalid[missing_bus]["value"].clone();
    invalid[missing_bus]["value"] = invalid[non_finite]["value"].clone();
    invalid[non_finite]["value"] = first;
    expect_catalog_failure(&swapped, "swapped invalid payloads", "invalid envelope")?;

    validate_time(&TimeExpression::Milliseconds {
        value_ms: MAX_SAFE_INTEGER,
    })?;
    if validate_time(&TimeExpression::Milliseconds {
        value_ms: MAX_SAFE_INTEGER + 1,
    })
    .is_ok()
    {
        return Err("unsafe integer passed validation".into());
    }

    let mut unicode = slot["value"].clone();
    unicode["defaultValue"] = Value::String("é".into());
    unicode["constraints"]["minLength"] = Value::from(1);
    unicode["constraints"]["maxLength"] = Value::from(1);
    derive_slot(unicode).map_err(|error| format!("Unicode scalar length diverged: {error}"))?;

    let layer = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "layer.ordered_visual")
        .ok_or_else(|| "layer fixture missing".to_string())?;
    let mut empty_animation_channel = layer["value"].clone();
    empty_animation_channel["layers"][0]["animationChannels"] = serde_json::json!([""]);
    if derive_layer(empty_animation_channel, usize::MAX).is_ok() {
        return Err("empty animation channel passed validation".into());
    }

    let mut invalid_track = component["value"].clone();
    invalid_track["definition"]["trackIds"] = serde_json::json!(["invalid.track"]);
    if derive_component(invalid_track).is_ok() {
        return Err("invalid component track ID passed validation".into());
    }

    let mut invalid_slot_key = component["value"].clone();
    invalid_slot_key["instance"]["slotValues"] = serde_json::json!({ "invalid.slot": "value" });
    if derive_component(invalid_slot_key).is_ok() {
        return Err("invalid slot-value key passed validation".into());
    }

    let marker = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "marker.scoped_impact")
        .ok_or_else(|| "marker fixture missing".to_string())?;
    let mut invalid_marker_id = marker["value"].clone();
    invalid_marker_id["marker"]["id"] = Value::String("invalid.marker".into());
    if derive_marker(invalid_marker_id).is_ok() {
        return Err("invalid marker ID passed validation".into());
    }
    let mut invalid_marker_name = marker["value"].clone();
    invalid_marker_name["marker"]["name"] = Value::String("invalid.marker".into());
    if derive_marker(invalid_marker_name).is_ok() {
        return Err("invalid marker name passed validation".into());
    }
    let mut invalid_marker_time = marker["value"].clone();
    invalid_marker_time["marker"]["timeMs"] = Value::from(MAX_SAFE_INTEGER + 1);
    if derive_marker(invalid_marker_time).is_ok() {
        return Err("unsafe marker timestamp passed validation".into());
    }

    let curve = fixtures
        .iter()
        .find(|fixture| fixture["id"] == "curve.complete_set")
        .ok_or_else(|| "curve fixture missing".to_string())?;
    let mut empty_curves = curve["value"].clone();
    empty_curves["curves"] = serde_json::json!([]);
    if derive_curve(empty_curves).is_ok() {
        return Err("empty curve collection passed validation".into());
    }

    let mut zero_maximum_slot = slot["value"].clone();
    zero_maximum_slot["constraints"]["maxLength"] = Value::from(0);
    zero_maximum_slot["constraints"]["minLength"] = Value::from(0);
    if derive_slot(zero_maximum_slot).is_ok() {
        return Err("zero slot maximum length passed validation".into());
    }

    let root = catalog
        .as_object()
        .ok_or_else(|| "catalog must be an object".to_string())?;
    for key in [
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
    ] {
        let limit = catalog_limit(root, key)?;
        ensure_at_most(limit, limit, key)?;
        if ensure_at_most(limit + 1, limit, key).is_ok() {
            return Err(format!("{key} overflow passed validation"));
        }
    }
    Ok(())
}
