//! Owned, renderer-neutral evaluation of root timelines and component occurrences.
//!
//! This module deliberately contains no filesystem, process, renderer, or artifact
//! concerns. Production render planning consumes this representation through a
//! separate path-bearing resource-binding sidecar.
use std::collections::{HashMap, HashSet};

#[cfg(test)]
pub(crate) mod repeater_conformance;
pub(crate) mod shapes;
use crate::{
    AnchorPoint, Asset, AudioTrackRole, CoreError, Easing, ErrorCode, Keyframe, KeyframeProperty,
    KeyframeValue, MediaType, Project, TextAlignment, TextStyle, TimelineItem, Track, Transform,
    TransitionItem, TransitionType, animation::positive_scalar_ranges,
    validation::validate_project_stacking,
};

pub(crate) const MAX_EVALUATED_VISUAL_LAYERS: usize = 4_096;
pub(crate) const MAX_EVALUATED_MEDIA_RESOURCES: usize = 4_096;
pub(crate) const MAX_EVALUATED_AUDIO_LAYERS: usize = 4_096;
pub(crate) const MAX_EVALUATED_TRANSITION_FACTS: usize = 4_096;
pub(crate) const MAX_EVALUATED_KEYFRAMES_PER_CHANNEL: usize = 10_000;
pub(crate) const MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedInstance {
    /// The leaf composition clock in milliseconds: local = rate * root + offset.
    pub(crate) rate: f64,
    pub(crate) offset: f64,
    pub(crate) start_ms: f64,
    pub(crate) end_ms: f64,
    pub(crate) canvas: (u32, u32),
}
impl EvaluatedInstance {
    pub(crate) fn root_ms(self, local_ms: u64) -> f64 {
        (local_ms as f64 - self.offset) / self.rate
    }
    fn child(
        self,
        item: &crate::ComponentInstanceItem,
        canvas: (u32, u32),
    ) -> Result<Self, CoreError> {
        let child = Self {
            rate: self.rate * item.time_scale,
            offset: item.trim_start_ms as f64
                + (self.offset - item.start_ms as f64) * item.time_scale,
            start_ms: self.start_ms.max(self.root_ms(item.start_ms)),
            end_ms: self.end_ms.min(self.root_ms(item.end_ms())),
            canvas,
        };
        if [child.rate, child.offset, child.start_ms, child.end_ms]
            .iter()
            .any(|v| !v.is_finite())
            || child.rate <= 0.0
        {
            return Err(invalid("non-finite composed component clock"));
        }
        Ok(child)
    }
}
impl crate::ComponentInstanceItem {
    fn end_ms(&self) -> u64 {
        self.start_ms + self.duration_ms
    }
}
type InstanceOrder = Vec<(usize, i32, usize, String)>;

pub(crate) fn evaluate_project(
    project: &Project,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<EvaluatedSceneResult, CoreError> {
    crate::validation::validate_recursive_graphs(project)?;
    shapes::preflight_svg_documents(project)?;
    let mut result = evaluate_project_inner(project, width, height, fps)?;
    shapes::refine_scene(&mut result.scene)?;
    Ok(result)
}

fn evaluate_project_inner(
    project: &Project,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<EvaluatedSceneResult, CoreError> {
    let definitions = project
        .components
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect::<std::collections::BTreeMap<_, _>>();
    preflight_instance_occurrences(project, &definitions)?;
    preflight_repeater_arithmetic(project, (width, height))?;
    let retained = std::iter::once(&project.tracks)
        .chain(project.components.iter().map(|c| &c.tracks))
        .flat_map(|tracks| tracks.iter())
        .flat_map(|track| &track.items)
        .any(|item| matches!(item, TimelineItem::Repeater(_)));
    if !retained
        && !project.tracks.iter().flat_map(|t| &t.items).any(|i| {
            matches!(
                i,
                TimelineItem::ComponentInstance(_) | TimelineItem::Repeater(_)
            )
        })
    {
        return evaluate_flat_project(
            project,
            width,
            height,
            fps,
            shapes::MAX_SCENE_SEGMENTS,
            false,
        );
    }
    crate::validation::validate_project_visual_properties(project)?;
    let duration_ms = checked_project_duration(project)?.max(1);
    let mut result = EvaluatedSceneResult {
        scene: EvaluatedScene {
            instance_voiceover_intervals: Some(vec![]),
            voiceover_activity_range_count: 0,
            canvas: EvaluatedCanvas { width, height, fps },
            duration_ms,
            resources: vec![],
            visual_layers: vec![],
            audio_layers: vec![],
            voiceover_intervals: vec![],
        },
        resource_bindings: SceneResourceBindings {
            media: vec![],
            fonts: vec![],
        },
    };
    let clock = EvaluatedInstance {
        rate: 1.0,
        offset: 0.0,
        start_ms: 0.0,
        end_ms: duration_ms as f64,
        canvas: (width, height),
    };
    preflight_instance_clocks(
        &project.tracks,
        clock,
        project,
        &definitions,
        &mut HashSet::new(),
    )?;
    let mut orders = HashMap::new();
    let context = InstanceTraversal {
        project,
        definitions: &definitions,
        retained,
    };
    // Independent definitions are validated before any root copies are published.
    if retained {
        for (index, component) in project.components.iter().enumerate() {
            let effective =
                crate::validation::resolve_component_slots(project, component, None, &definitions)?;
            let mut domain = EvaluatedSceneResult {
                scene: EvaluatedScene {
                    instance_voiceover_intervals: Some(vec![]),
                    voiceover_activity_range_count: 0,
                    canvas: EvaluatedCanvas {
                        width: component.width,
                        height: component.height,
                        fps,
                    },
                    duration_ms: component.duration_ms,
                    resources: vec![],
                    visual_layers: vec![],
                    audio_layers: vec![],
                    voiceover_intervals: vec![],
                },
                resource_bindings: SceneResourceBindings {
                    media: vec![],
                    fonts: vec![],
                },
            };
            let domain_clock = EvaluatedInstance {
                canvas: (component.width, component.height),
                end_ms: component.duration_ms as f64,
                ..clock
            };
            let mut projection = Vec::new();
            let rich_text_overrides = component
                .slots
                .iter()
                .filter_map(|slot| {
                    if slot.binding.property != crate::SlotProperty::TextDocument {
                        return None;
                    }
                    match slot.default_value.as_ref() {
                        Some(crate::SlotValue::RichText(document)) => {
                            Some((slot.binding.target_layer_id.clone(), document.clone()))
                        }
                        _ => None,
                    }
                })
                .collect();
            context.expand(
                &effective.tracks,
                InstanceScope {
                    clock: domain_clock,
                    outer: IDENTITY_MATRIX,
                    outer_inverse: IDENTITY_MATRIX,
                    opacity: 1.0,
                    visual_start: 0.0,
                    visual_end: domain_clock.end_ms,
                    prefix: &[],
                    rich_text_overrides,
                    audio_visible: true,
                },
                &mut HashMap::new(),
                &mut domain,
                &mut HashSet::from([index]),
                &mut projection,
            )?;
            validate_projection(&domain, &projection)?;
        }
    }
    let mut projection = Vec::new();
    context.expand(
        &project.tracks,
        InstanceScope {
            clock,
            outer: IDENTITY_MATRIX,
            outer_inverse: IDENTITY_MATRIX,
            opacity: 1.0,
            visual_start: clock.start_ms,
            visual_end: clock.end_ms,
            prefix: &[],
            rich_text_overrides: HashMap::new(),
            audio_visible: true,
        },
        &mut orders,
        &mut result,
        &mut HashSet::new(),
        &mut projection,
    )?;
    validate_projection(&result, &projection)?;
    let mut published = Vec::new();
    // Clone only generated visible occurrences after every domain succeeded.
    for copy in projection
        .iter()
        .filter(|copy| copy.generated && copy.instance.start_ms < copy.instance.end_ms)
    {
        #[cfg(test)]
        GENERATED_MATERIALIZATIONS.with(|count| count.set(count.get() + 1));
        let mut layer = result.scene.visual_layers[copy.base_index].clone();
        layer.item_id = copy.item_id.clone();
        layer.instance = Some(copy.instance);
        layer.ancestors = Some(copy.ancestors);
        layer.affine = None;
        layer.sampling_tiles = None;
        orders.insert(layer.item_id.clone(), copy.order.clone());
        published.push(layer);
    }
    published.extend(
        std::mem::take(&mut result.scene.visual_layers)
            .into_iter()
            .filter(|layer| layer.instance.is_some_and(|c| c.start_ms < c.end_ms)),
    );
    result.scene.visual_layers = published;
    result
        .scene
        .audio_layers
        .retain(|layer| layer.instance.is_some_and(|c| c.start_ms < c.end_ms));
    if retained {
        let media = result
            .scene
            .visual_layers
            .iter()
            .filter_map(|layer| match &layer.source {
                EvaluatedVisualSource::Media { asset_id, .. } => Some(asset_id.as_str()),
                _ => None,
            })
            .chain(
                result
                    .scene
                    .audio_layers
                    .iter()
                    .map(|layer| layer.asset_id.as_str()),
            )
            .collect::<HashSet<_>>();
        result
            .scene
            .resources
            .retain(|resource| media.contains(resource.asset_id.as_str()));
        result
            .resource_bindings
            .media
            .retain(|binding| media.contains(binding.asset_id.as_str()));
        let fonts = result
            .scene
            .visual_layers
            .iter()
            .filter_map(|layer| match &layer.source {
                EvaluatedVisualSource::Text(text) => text.font_resource_id.as_deref(),
                _ => None,
            })
            .collect::<HashSet<_>>();
        result
            .resource_bindings
            .fonts
            .retain(|binding| fonts.contains(binding.font_resource_id.as_str()));
    }
    result
        .scene
        .visual_layers
        .sort_by(|a, b| orders[&a.item_id].cmp(&orders[&b.item_id]));
    result
        .scene
        .audio_layers
        .sort_by(|a, b| orders[&a.item_id].cmp(&orders[&b.item_id]));
    let mut all = orders.iter().collect::<Vec<_>>();
    all.sort_by(|a, b| a.1.cmp(b.1));
    let ranks = all
        .into_iter()
        .enumerate()
        .map(|(i, (id, _))| (id.clone(), i))
        .collect::<HashMap<_, _>>();
    for layer in &mut result.scene.visual_layers {
        layer.order = EvaluatedLayerOrder {
            track_index: ranks[&layer.item_id],
            item_index: 0,
        };
    }
    for layer in &mut result.scene.audio_layers {
        layer.order = EvaluatedLayerOrder {
            track_index: ranks[&layer.item_id],
            item_index: 0,
        };
    }
    Ok(result)
}

#[cfg(test)]
thread_local! { static GENERATED_MATERIALIZATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

fn validate_projection(
    result: &EvaluatedSceneResult,
    projection: &[ProjectedVisualCopy],
) -> Result<(), CoreError> {
    let mut transition_facts = 0usize;
    for occurrence in projection {
        transition_facts = transition_facts
            .checked_add(occurrence.transition_facts)
            .filter(|count| *count <= MAX_EVALUATED_TRANSITION_FACTS)
            .ok_or_else(|| invalid("expanded transition limit exceeded"))?;
        if occurrence
            .ancestors
            .matrix
            .iter()
            .chain(&occurrence.ancestors.inverse)
            .chain([occurrence.ancestors.opacity].iter())
            .any(|value| !value.is_finite())
        {
            return Err(invalid("non-finite retained occurrence transform"));
        }
        let layer = &result.scene.visual_layers[occurrence.base_index];
        if !matches!(layer.source, EvaluatedVisualSource::Shape(_))
            && let Some(size) = layer.source_size
        {
            measure_layer_affine(
                layer,
                size,
                (result.scene.canvas.width, result.scene.canvas.height),
                Some(occurrence.ancestors),
            )?;
        }
    }
    shapes::preflight_shape_layers(
        projection.iter().map(|copy| {
            (
                &result.scene.visual_layers[copy.base_index],
                Some(copy.ancestors),
            )
        }),
        (result.scene.canvas.width, result.scene.canvas.height),
    )
}
fn preflight_instance_occurrences(
    project: &Project,
    definitions: &std::collections::BTreeMap<&str, usize>,
) -> Result<(), CoreError> {
    const LIMIT: usize = 65_536;

    fn add(total: &mut usize, value: usize) -> Result<(), CoreError> {
        *total = total
            .checked_add(value)
            .filter(|value| *value <= LIMIT)
            .ok_or_else(|| invalid("maxExpandedOccurrences exceeded"))?;
        Ok(())
    }

    fn count_source(
        source: &TimelineItem,
        tracks: &[Track],
        project: &Project,
        definitions: &std::collections::BTreeMap<&str, usize>,
        memo: &mut HashMap<usize, usize>,
        active_components: &mut HashSet<usize>,
        active_groups: &mut HashSet<String>,
    ) -> Result<usize, CoreError> {
        let mut total = 1usize;
        match source {
            TimelineItem::Group(group) => {
                if !active_groups.insert(group.id.clone()) {
                    return Err(invalid("parent cycle"));
                }
                for child in tracks.iter().flat_map(|track| &track.items).filter(|item| {
                    item.visual_properties()
                        .parent
                        .as_ref()
                        .is_some_and(|parent| parent.id == group.id)
                }) {
                    add(
                        &mut total,
                        count_source(
                            child,
                            tracks,
                            project,
                            definitions,
                            memo,
                            active_components,
                            active_groups,
                        )?,
                    )?;
                }
                active_groups.remove(&group.id);
            }
            TimelineItem::ComponentInstance(instance) => {
                let id = *definitions
                    .get(instance.component_id.as_str())
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "component definition missing")
                    })?;
                let descendants = if let Some(value) = memo.get(&id) {
                    *value
                } else {
                    if !active_components.insert(id) {
                        return Err(invalid("component dependency cycle"));
                    }
                    let value = count_tracks(
                        &project.components[id].tracks,
                        project,
                        definitions,
                        memo,
                        active_components,
                    )?;
                    active_components.remove(&id);
                    memo.insert(id, value);
                    value
                };
                add(&mut total, descendants)?;
            }
            _ => {}
        }
        Ok(total)
    }

    fn count_item(
        item: &TimelineItem,
        tracks: &[Track],
        project: &Project,
        definitions: &std::collections::BTreeMap<&str, usize>,
        memo: &mut HashMap<usize, usize>,
        active_components: &mut HashSet<usize>,
    ) -> Result<usize, CoreError> {
        let mut total = 1usize;
        match item {
            TimelineItem::ComponentInstance(_) => {
                let expanded = count_source(
                    item,
                    tracks,
                    project,
                    definitions,
                    memo,
                    active_components,
                    &mut HashSet::new(),
                )?;
                add(&mut total, expanded - 1)?;
            }
            TimelineItem::Repeater(repeater) => {
                let source = tracks
                    .iter()
                    .flat_map(|track| &track.items)
                    .find(|item| item.id() == repeater.repeater.source.id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "repeater source missing")
                    })?;
                let source_count = count_source(
                    source,
                    tracks,
                    project,
                    definitions,
                    memo,
                    active_components,
                    &mut HashSet::new(),
                )?;
                let generated = source_count
                    .checked_mul(usize::from(repeater.repeater.copies))
                    .ok_or_else(|| invalid("maxExpandedOccurrences exceeded"))?;
                add(&mut total, generated)?;
            }
            _ => {}
        }
        Ok(total)
    }

    fn count_tracks(
        tracks: &[Track],
        project: &Project,
        definitions: &std::collections::BTreeMap<&str, usize>,
        memo: &mut HashMap<usize, usize>,
        active_components: &mut HashSet<usize>,
    ) -> Result<usize, CoreError> {
        let mut total = 0usize;
        for item in tracks.iter().flat_map(|t| &t.items) {
            add(
                &mut total,
                count_item(item, tracks, project, definitions, memo, active_components)?,
            )?;
        }
        Ok(total)
    }

    // Validation has already established component acyclicity. Preflight the root
    // expansion and every retained definition, including unused/hidden content.
    count_tracks(
        &project.tracks,
        project,
        definitions,
        &mut HashMap::new(),
        &mut HashSet::new(),
    )?;
    for (component_index, component) in project.components.iter().enumerate() {
        count_tracks(
            &component.tracks,
            project,
            definitions,
            &mut HashMap::new(),
            &mut HashSet::from([component_index]),
        )?;
    }
    Ok(())
}

fn preflight_repeater_arithmetic(
    project: &Project,
    root_canvas: (u32, u32),
) -> Result<(), CoreError> {
    let scopes = std::iter::once((&project.tracks[..], root_canvas)).chain(
        project
            .components
            .iter()
            .map(|component| (&component.tracks[..], (component.width, component.height))),
    );
    for (tracks, canvas) in scopes {
        for repeater in tracks
            .iter()
            .flat_map(|track| &track.items)
            .filter_map(|item| {
                if let TimelineItem::Repeater(repeater) = item {
                    Some(repeater)
                } else {
                    None
                }
            })
        {
            let offset = &repeater.repeater.transform_offset;
            let transform = crate::Transform2D {
                position: offset.position,
                scale_x: offset.scale_x,
                scale_y: offset.scale_y,
                rotation_deg: offset.rotation_deg,
                skew_x_deg: offset.skew_x_deg,
                skew_y_deg: offset.skew_y_deg,
                opacity: 1.0,
                ..Default::default()
            };
            let (step, step_inverse) = transform_matrices(transform, canvas, canvas)?;
            let mut power = IDENTITY_MATRIX;
            let mut inverse_power = IDENTITY_MATRIX;
            for copy_index in 1..=usize::from(repeater.repeater.copies) {
                power = multiply_matrix(power, step);
                inverse_power = multiply_matrix(step_inverse, inverse_power);
                let opacity = 1.0 + copy_index as f64 * repeater.repeater.opacity_offset;
                if power
                    .iter()
                    .chain(&inverse_power)
                    .chain([opacity].iter())
                    .any(|value| !value.is_finite())
                {
                    return Err(invalid("non-finite repeater transform expansion"));
                }
            }
        }
    }
    Ok(())
}
fn preflight_instance_clocks(
    tracks: &[Track],
    clock: EvaluatedInstance,
    project: &Project,
    definitions: &std::collections::BTreeMap<&str, usize>,
    active_components: &mut HashSet<usize>,
) -> Result<(), CoreError> {
    for item in tracks.iter().flat_map(|t| &t.items) {
        if !clock.root_ms(item.start_ms()).is_finite() || !clock.root_ms(item.end_ms()).is_finite()
        {
            return Err(invalid("non-finite derived component interval"));
        }
        if let TimelineItem::ComponentInstance(instance) = item {
            let component_index = definitions[instance.component_id.as_str()];
            if !active_components.insert(component_index) {
                return Err(invalid("component dependency cycle"));
            }
            let component = &project.components[component_index];
            let child = clock.child(instance, (component.width, component.height))?;
            preflight_instance_clocks(
                &component.tracks,
                child,
                project,
                definitions,
                active_components,
            )?;
            active_components.remove(&component_index);
        }
    }
    Ok(())
}
struct InstanceTraversal<'a> {
    project: &'a Project,
    definitions: &'a std::collections::BTreeMap<&'a str, usize>,
    retained: bool,
}

struct InstanceScope<'a> {
    clock: EvaluatedInstance,
    outer: [f64; 6],
    outer_inverse: [f64; 6],
    opacity: f64,
    visual_start: f64,
    visual_end: f64,
    prefix: &'a [(usize, i32, usize, String)],
    rich_text_overrides: HashMap<String, crate::RichTextDocument>,
    audio_visible: bool,
}

#[derive(Clone)]
struct ProjectedVisualCopy {
    base_index: usize,
    item_id: String,
    instance: EvaluatedInstance,
    ancestors: EvaluatedAncestors,
    order: InstanceOrder,
    generated: bool,
    transition_facts: usize,
}

impl InstanceTraversal<'_> {
    fn expand(
        &self,
        tracks: &[Track],
        scope: InstanceScope<'_>,
        orders: &mut HashMap<String, InstanceOrder>,
        result: &mut EvaluatedSceneResult,
        active_components: &mut HashSet<usize>,
        projection: &mut Vec<ProjectedVisualCopy>,
    ) -> Result<(), CoreError> {
        let InstanceScope {
            clock,
            outer,
            outer_inverse,
            opacity,
            visual_start,
            visual_end,
            prefix,
            rich_text_overrides,
            audio_visible,
        } = scope;
        if outer.iter().chain(&outer_inverse).any(|v| !v.is_finite()) || !opacity.is_finite() {
            return Err(invalid("non-finite composed component transform"));
        }
        let scope_first_layer = projection.len();
        let mut local = Project {
            schema_version: self.project.schema_version,
            id: self.project.id.clone(),
            revision: self.project.revision,
            name: self.project.name.clone(),
            created_at_ms: self.project.created_at_ms,
            updated_at_ms: self.project.updated_at_ms,
            settings: self.project.settings.clone(),
            assets: self.project.assets.clone(),
            tracks: tracks.to_vec(),
            components: vec![],
        };
        let mut item_orders = HashMap::new();
        for (ti, track) in tracks.iter().enumerate() {
            for (ii, item) in track.items.iter().enumerate() {
                let mut order = prefix.to_vec();
                order.push((
                    ti,
                    item.visual_properties().z_index,
                    ii,
                    item.id().to_owned(),
                ));
                item_orders.insert(item.id().to_owned(), order);
            }
        }
        let scope_project = local.clone();
        for track in &mut local.tracks {
            track.items.retain(|item| {
                !matches!(
                    item,
                    TimelineItem::ComponentInstance(_) | TimelineItem::Repeater(_)
                )
            });
            for (i, item) in track.items.iter_mut().enumerate() {
                item.visual_properties_mut().stack_order = i as u32;
                if let Some(parent) = &mut item.visual_properties_mut().parent {
                    parent.scope = "root".to_owned();
                }
                if let TimelineItem::Repeater(repeater) = item {
                    repeater.repeater.source.scope = "root".to_owned();
                }
            }
        }
        let mut evaluated = evaluate_flat_project(
            &local,
            clock.canvas.0,
            clock.canvas.1,
            result.scene.canvas.fps,
            shapes::MAX_SCENE_SEGMENTS
                - result
                    .scene
                    .visual_layers
                    .iter()
                    .filter_map(|l| match &l.source {
                        EvaluatedVisualSource::Shape(s) => Some(s.segments()),
                        _ => None,
                    })
                    .sum::<usize>(),
            self.retained,
        )?;
        // Retained validation counts roles without adding hidden facts to output.
        let mut retained_transition_counts = HashMap::<&str, usize>::new();
        if self.retained {
            for transition in local
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .filter_map(|item| {
                    if let TimelineItem::Transition(t) = item {
                        Some(t)
                    } else {
                        None
                    }
                })
            {
                for endpoint in std::iter::once(transition.from_item_id.as_str())
                    .chain(transition.to_item_id.as_deref())
                {
                    let count = retained_transition_counts.entry(endpoint).or_default();
                    *count = count
                        .checked_add(1)
                        .filter(|count| *count <= MAX_EVALUATED_TRANSITION_FACTS)
                        .ok_or_else(|| invalid("expanded transition limit exceeded"))?;
                }
            }
        }
        for layer in &mut evaluated.scene.visual_layers {
            if let Some(document) = rich_text_overrides.get(&layer.item_id)
                && let EvaluatedVisualSource::Text(text) = &mut layer.source
            {
                text.text = document.runs.iter().map(|run| run.text.as_str()).collect();
                text.rich_runs = Some(document.runs.clone());
            }
        }
        let root_has_component_instances = self
            .project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .any(|item| matches!(item, TimelineItem::ComponentInstance(_)));
        let identity = |order: EvaluatedLayerOrder, id: &str| -> (String, InstanceOrder) {
            // Length-prefixed scope positions avoid collisions even with repeated local identifiers.
            let owner_id = local.tracks[order.track_index].items[order.item_index].id();
            let instance_order = item_orders[owner_id].clone();
            let identity = if prefix.is_empty() && !root_has_component_instances {
                id.to_owned()
            } else {
                let mut identity = instance_order
                    .iter()
                    .map(|(t, _, i, _)| format!("{t}-{i}"))
                    .collect::<Vec<_>>()
                    .join("_");
                if owner_id != id {
                    identity.push(':');
                    identity.push_str(id);
                }
                identity
            };
            (identity, instance_order)
        };
        for layer in &mut evaluated.scene.visual_layers {
            let span = layer.visible_span();
            let start = clock
                .root_ms(span.start_ms)
                .max(clock.start_ms)
                .max(visual_start);
            let end = clock.root_ms(span.end_ms).min(clock.end_ms).min(visual_end);
            let owner_track = &local.tracks[layer.order.track_index];
            let owner = &owner_track.items[layer.order.item_index];
            let end = if owner_track.hidden || owner.hidden() {
                start
            } else {
                end
            };
            let (id, order) = identity(layer.order, &layer.item_id);
            orders.insert(id.clone(), order);
            layer.item_id = id.clone();
            let local_parent = layer.ancestors.unwrap_or(EvaluatedAncestors {
                matrix: IDENTITY_MATRIX,
                inverse: IDENTITY_MATRIX,
                opacity: 1.0,
                clip: layer.span,
            });
            layer.ancestors = Some(EvaluatedAncestors {
                matrix: multiply_matrix(outer, local_parent.matrix),
                inverse: multiply_matrix(local_parent.inverse, outer_inverse),
                opacity: opacity * local_parent.opacity,
                clip: local_parent.clip,
            });
            layer.instance = Some(EvaluatedInstance {
                start_ms: start,
                end_ms: end,
                ..clock
            });
            if matches!(layer.source, EvaluatedVisualSource::Media { .. })
                && !(2f64.powi(-32)..=2f64.powi(32)).contains(&clock.rate)
            {
                return Err(invalid("component media rate exceeds limits"));
            }
            layer.source_size = match &layer.source {
                EvaluatedVisualSource::Rectangle { width, height, .. } => Some((*width, *height)),
                EvaluatedVisualSource::Shape(shape) => Some(shape.size),
                EvaluatedVisualSource::SolidColor { .. } => Some(clock.canvas),
                _ => layer.source_size,
            };
            layer.affine = None;
            layer.sampling_tiles = None;
            if let EvaluatedVisualSource::Text(text) = &mut layer.source
                && let Some(font_id) = &mut text.font_resource_id
            {
                let old = font_id.clone();
                *font_id = format!("instance-font-{id}");
                for binding in &mut evaluated.resource_bindings.fonts {
                    if binding.font_resource_id == old {
                        binding.font_resource_id = font_id.clone();
                    }
                }
            }
        }
        if !self.retained {
            evaluated
                .scene
                .visual_layers
                .retain(|l| l.instance.is_some_and(|c| c.start_ms < c.end_ms));
        }
        let shape_segments: usize = result
            .scene
            .visual_layers
            .iter()
            .chain(&evaluated.scene.visual_layers)
            .filter_map(|layer| match &layer.source {
                EvaluatedVisualSource::Shape(shape) => Some(shape.segments()),
                _ => None,
            })
            .sum();
        if shape_segments > shapes::MAX_SCENE_SEGMENTS {
            return Err(invalid("scene shape segment limit exceeded"));
        }
        result.scene.voiceover_activity_range_count +=
            evaluated.scene.voiceover_activity_range_count;
        if result.scene.voiceover_activity_range_count > MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES {
            return Err(invalid("expanded voiceover interval limit exceeded"));
        }
        for span in &evaluated.scene.voiceover_intervals {
            let start = clock.root_ms(span.start_ms).max(clock.start_ms);
            let end = clock.root_ms(span.end_ms).min(clock.end_ms);
            if audio_visible && start < end {
                let intervals = result.scene.instance_voiceover_intervals.as_mut().unwrap();
                if intervals.len() >= MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES {
                    return Err(invalid("expanded voiceover interval limit exceeded"));
                }
                intervals.push((start, end));
            }
        }
        for layer in &mut evaluated.scene.audio_layers {
            layer.ducking = evaluate_ducking(&local.tracks[layer.order.track_index], true)?;
            if !(2f64.powi(-32)..=2f64.powi(32)).contains(&clock.rate) {
                return Err(invalid("component media rate exceeds tempo limits"));
            }
            let (id, order) = identity(layer.order, &layer.item_id);
            orders.insert(id.clone(), order);
            layer.item_id = id;
            layer.instance = Some(EvaluatedInstance {
                start_ms: clock.root_ms(layer.span.start_ms).max(clock.start_ms),
                end_ms: clock.root_ms(layer.span.end_ms).min(clock.end_ms),
                ..clock
            });
            let owner_track = &local.tracks[layer.order.track_index];
            if owner_track.hidden
                || owner_track.items[layer.order.item_index].hidden()
                || !audio_visible
            {
                let instance = layer.instance.as_mut().unwrap();
                instance.end_ms = instance.start_ms;
            }
        }
        if !self.retained {
            evaluated
                .scene
                .audio_layers
                .retain(|l| l.instance.is_some_and(|c| c.start_ms < c.end_ms));
        }
        if projection.len() + evaluated.scene.visual_layers.len() > MAX_EVALUATED_VISUAL_LAYERS
            || result.scene.audio_layers.len() + evaluated.scene.audio_layers.len()
                > MAX_EVALUATED_AUDIO_LAYERS
        {
            return Err(invalid("expanded scene layer limit exceeded"));
        }
        for resource in evaluated.scene.resources {
            if !result
                .scene
                .resources
                .iter()
                .any(|r| r.asset_id == resource.asset_id)
            {
                result.scene.resources.push(resource);
            }
        }
        for binding in evaluated.resource_bindings.media {
            if !result
                .resource_bindings
                .media
                .iter()
                .any(|b| b.asset_id == binding.asset_id)
            {
                result.resource_bindings.media.push(binding);
            }
        }
        if result.scene.resources.len() > MAX_EVALUATED_MEDIA_RESOURCES {
            return Err(invalid("expanded media resource limit exceeded"));
        }
        let transitions = result
            .scene
            .visual_layers
            .iter()
            .chain(&evaluated.scene.visual_layers)
            .map(|l| l.transitions.len())
            .sum::<usize>();
        if transitions > MAX_EVALUATED_TRANSITION_FACTS {
            return Err(invalid("expanded transition limit exceeded"));
        }
        result
            .resource_bindings
            .fonts
            .extend(evaluated.resource_bindings.fonts);
        for (index, layer) in evaluated.scene.visual_layers.iter().enumerate() {
            projection.push(ProjectedVisualCopy {
                base_index: result.scene.visual_layers.len() + index,
                item_id: layer.item_id.clone(),
                instance: layer.instance.unwrap(),
                ancestors: layer.ancestors.unwrap(),
                order: orders[&layer.item_id].clone(),
                generated: false,
                transition_facts: if self.retained {
                    let owner =
                        local.tracks[layer.order.track_index].items[layer.order.item_index].id();
                    retained_transition_counts.get(owner).copied().unwrap_or(0)
                } else {
                    layer.transitions.len()
                },
            });
        }
        result
            .scene
            .visual_layers
            .extend(evaluated.scene.visual_layers);
        result
            .scene
            .audio_layers
            .extend(evaluated.scene.audio_layers);
        for (ti, track) in tracks.iter().enumerate() {
            for (ii, item) in track.items.iter().enumerate() {
                let TimelineItem::ComponentInstance(instance) = item else {
                    continue;
                };
                if !self.retained && (track.hidden || item.hidden()) {
                    continue;
                }
                let component_index = self.definitions[instance.component_id.as_str()];
                if !active_components.insert(component_index) {
                    return Err(invalid("component dependency cycle"));
                }
                let component = &self.project.components[component_index];
                let effective = crate::validation::resolve_component_slots(
                    self.project,
                    component,
                    Some(&instance.slot_values),
                    self.definitions,
                )?;
                let child_clock = clock.child(instance, (component.width, component.height))?;
                if !self.retained && child_clock.start_ms >= child_clock.end_ms {
                    active_components.remove(&component_index);
                    continue;
                }
                let transform = instance.visual_properties.transform2d.unwrap_or_else(|| {
                    let t = &instance.visual_properties.transform;
                    crate::Transform2D {
                        position: crate::TransformPosition {
                            x: t.position_x,
                            y: t.position_y,
                            unit: crate::PositionUnit::Pixels,
                        },
                        scale_x: t.scale,
                        scale_y: t.scale,
                        opacity: t.opacity,
                        ..Default::default()
                    }
                });
                let (mut matrix, mut inverse) =
                    transform_matrices(transform, child_clock.canvas, clock.canvas)?;
                let mut child_opacity = opacity * transform.opacity;
                let mut start = visual_start.max(child_clock.start_ms);
                let mut end = visual_end.min(child_clock.end_ms);
                if track.hidden || item.hidden() {
                    end = start;
                }
                let mut node = item;
                while let Some(parent) = &node.visual_properties().parent {
                    let (pt, target) = tracks
                        .iter()
                        .find_map(|t| t.items.iter().find(|i| i.id() == parent.id).map(|i| (t, i)))
                        .ok_or_else(|| invalid("missing validated instance ancestor"))?;
                    let t = target.visual_properties().transform2d.unwrap_or_default();
                    let (m, inv) = transform_matrices(t, clock.canvas, clock.canvas)?;
                    matrix = multiply_matrix(m, matrix);
                    inverse = multiply_matrix(inverse, inv);
                    child_opacity *= t.opacity;
                    start = start.max(clock.root_ms(target.start_ms()));
                    end = end.min(clock.root_ms(target.end_ms()));
                    if pt.hidden || target.hidden() {
                        end = start;
                    }
                    node = target;
                }
                let mut order = prefix.to_vec();
                order.push((
                    ti,
                    item.visual_properties().z_index,
                    ii,
                    item.id().to_owned(),
                ));
                let rich_text_overrides = component
                    .slots
                    .iter()
                    .filter_map(|slot| {
                        if slot.binding.property != crate::SlotProperty::TextDocument {
                            return None;
                        }
                        match instance
                            .slot_values
                            .get(&slot.id)
                            .or(slot.default_value.as_ref())
                        {
                            Some(crate::SlotValue::RichText(document)) => {
                                Some((slot.binding.target_layer_id.clone(), document.clone()))
                            }
                            _ => None,
                        }
                    })
                    .collect();
                self.expand(
                    &effective.tracks,
                    InstanceScope {
                        clock: child_clock,
                        outer: multiply_matrix(outer, matrix),
                        outer_inverse: multiply_matrix(inverse, outer_inverse),
                        opacity: child_opacity,
                        visual_start: start,
                        visual_end: end,
                        prefix: &order,
                        rich_text_overrides,
                        audio_visible: audio_visible && !track.hidden && !item.hidden(),
                    },
                    orders,
                    result,
                    active_components,
                    projection,
                )?;
                active_components.remove(&component_index);
            }
        }

        // Same-scope repeaters resolve only against the immutable ordinary range.
        // Keep lightweight indices/order metadata, project the complete scope, and
        // clone layers only after every layer/geometry/memory budget has succeeded.
        let scope_ordinary_end = projection.len();
        let ordinary_layers = (scope_first_layer..scope_ordinary_end)
            .map(|index| (index, projection[index].order.clone()))
            .collect::<Vec<_>>();
        let mut projected = Vec::<ProjectedVisualCopy>::new();
        for (repeater_track, track) in tracks.iter().enumerate() {
            if !self.retained && track.hidden {
                continue;
            }
            for (repeater_index, candidate) in track.items.iter().enumerate() {
                let TimelineItem::Repeater(repeater) = candidate else {
                    continue;
                };
                if !self.retained && candidate.hidden() {
                    continue;
                }
                let source = tracks
                    .iter()
                    .flat_map(|track| &track.items)
                    .find(|item| item.id() == repeater.repeater.source.id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "repeater source missing")
                    })?;
                let mut bases = ordinary_layers
                    .iter()
                    .filter_map(|(base_index, order)| {
                        let owner_id = order.get(prefix.len())?.3.as_str();
                        let selected = match source {
                            TimelineItem::Shape(shape) => owner_id == shape.id,
                            TimelineItem::Group(group) => {
                                is_descendant_of(&scope_project, owner_id, &group.id)
                            }
                            TimelineItem::ComponentInstance(instance) => owner_id == instance.id,
                            _ => false,
                        };
                        selected.then(|| (*base_index, order.clone()))
                    })
                    .collect::<Vec<_>>();
                bases.sort_by(|left, right| left.1.cmp(&right.1));

                let offset = &repeater.repeater.transform_offset;
                let transform = crate::Transform2D {
                    position: offset.position,
                    scale_x: offset.scale_x,
                    scale_y: offset.scale_y,
                    rotation_deg: offset.rotation_deg,
                    skew_x_deg: offset.skew_x_deg,
                    skew_y_deg: offset.skew_y_deg,
                    opacity: 1.0,
                    ..Default::default()
                };
                let (step, step_inverse) =
                    transform_matrices(transform, clock.canvas, clock.canvas)?;
                let (parent, parent_inverse) = parent_matrices(tracks, source, clock.canvas)?;
                let parent = multiply_matrix(outer, parent);
                let parent_inverse = multiply_matrix(parent_inverse, outer_inverse);
                let mut power = IDENTITY_MATRIX;
                let mut inverse_power = IDENTITY_MATRIX;
                let repeater_span = checked_span(repeater.start_ms, repeater.duration_ms)?;
                let repeater_start = clock.root_ms(repeater.start_ms).max(visual_start);
                let repeater_end = clock
                    .root_ms(repeater.start_ms + repeater.duration_ms)
                    .min(visual_end);
                for copy_index in 1..=usize::from(repeater.repeater.copies) {
                    power = multiply_matrix(power, step);
                    inverse_power = multiply_matrix(step_inverse, inverse_power);
                    if power
                        .iter()
                        .chain(&inverse_power)
                        .any(|value| !value.is_finite())
                    {
                        return Err(invalid("non-finite repeater transform expansion"));
                    }
                    let copy_opacity = (1.0 + copy_index as f64 * repeater.repeater.opacity_offset)
                        .clamp(0.0, 1.0);
                    let conjugated =
                        multiply_matrix(multiply_matrix(parent, power), parent_inverse);
                    let inverse_conjugated =
                        multiply_matrix(multiply_matrix(parent, inverse_power), parent_inverse);
                    if conjugated
                        .iter()
                        .chain(&inverse_conjugated)
                        .any(|value| !value.is_finite())
                    {
                        return Err(invalid("non-finite repeater transform expansion"));
                    }
                    let mut visible_source_index = 0;
                    for (base_index, base_order) in &bases {
                        if projection.len() + projected.len() >= MAX_EVALUATED_VISUAL_LAYERS {
                            return Err(invalid("expanded scene layer limit exceeded"));
                        }
                        let base = &projection[*base_index];
                        let source_index = visible_source_index;
                        if base.instance.start_ms < base.instance.end_ms {
                            visible_source_index += 1;
                        }
                        let mut instance_data = base.instance;
                        instance_data.start_ms = instance_data.start_ms.max(repeater_start);
                        instance_data.end_ms = instance_data.end_ms.min(repeater_end);
                        if track.hidden || candidate.hidden() {
                            instance_data.end_ms = instance_data.start_ms;
                        }
                        if !self.retained && instance_data.start_ms >= instance_data.end_ms {
                            continue;
                        }
                        let base_id = base.item_id.clone();
                        let item_id = if matches!(source, TimelineItem::ComponentInstance(_)) {
                            format!("repeater:{}:{copy_index:03}:{base_id}", repeater.id)
                        } else {
                            format!(
                                "repeater:{}:{copy_index:03}:{source_index:06}:{base_id}",
                                repeater.id
                            )
                        };
                        let ancestors = base.ancestors;
                        let ancestors = EvaluatedAncestors {
                            matrix: multiply_matrix(conjugated, ancestors.matrix),
                            inverse: multiply_matrix(
                                multiply_matrix(
                                    multiply_matrix(ancestors.inverse, parent),
                                    inverse_power,
                                ),
                                parent_inverse,
                            ),
                            opacity: ancestors.opacity * copy_opacity,
                            clip: EvaluatedTimeSpan {
                                start_ms: ancestors.clip.start_ms.max(repeater_span.start_ms),
                                end_ms: ancestors.clip.end_ms.min(repeater_span.end_ms),
                            },
                        };
                        if ancestors
                            .matrix
                            .iter()
                            .chain(&ancestors.inverse)
                            .chain([ancestors.opacity].iter())
                            .any(|value| !value.is_finite())
                        {
                            return Err(invalid("non-finite repeater transform expansion"));
                        }
                        let mut copy_order = prefix.to_vec();
                        copy_order.push((
                            repeater_track,
                            repeater.visual_properties.z_index,
                            repeater_index,
                            repeater.id.clone(),
                        ));
                        copy_order.push((copy_index, 0, 0, String::new()));
                        copy_order.extend(base_order.iter().skip(prefix.len()).cloned());
                        projected.push(ProjectedVisualCopy {
                            base_index: base.base_index,
                            item_id,
                            instance: instance_data,
                            ancestors,
                            order: copy_order,
                            generated: true,
                            transition_facts: base.transition_facts,
                        });
                    }
                }
            }
        }
        projection.extend(projected);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedSceneResult {
    pub(crate) scene: EvaluatedScene,
    pub(crate) resource_bindings: SceneResourceBindings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SceneResourceBindings {
    pub(crate) media: Vec<MediaResourceBinding>,
    pub(crate) fonts: Vec<FontResourceBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MediaResourceBinding {
    pub(crate) asset_id: String,
    pub(crate) project_relative_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FontResourceBinding {
    pub(crate) font_resource_id: String,
    pub(crate) requested_path: Option<String>,
    pub(crate) requested_family: Option<String>,
}

impl std::fmt::Debug for EvaluatedScene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_struct("EvaluatedScene");
        value.field("canvas", &self.canvas);
        value.field("duration_ms", &self.duration_ms);
        value.field("resources", &self.resources);
        value.field("visual_layers", &self.visual_layers);
        value.field("audio_layers", &self.audio_layers);
        value.field("voiceover_intervals", &self.voiceover_intervals);
        if let Some(extra) = &self.instance_voiceover_intervals {
            value.field("instance_voiceover_intervals", extra);
        }
        value.finish()
    }
}
#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedScene {
    pub(crate) instance_voiceover_intervals: Option<Vec<(f64, f64)>>,
    voiceover_activity_range_count: usize,
    pub(crate) canvas: EvaluatedCanvas,
    pub(crate) duration_ms: u64,
    pub(crate) resources: Vec<EvaluatedMediaResource>,
    pub(crate) visual_layers: Vec<EvaluatedVisualLayer>,
    pub(crate) audio_layers: Vec<EvaluatedAudioLayer>,
    pub(crate) voiceover_intervals: Vec<EvaluatedTimeSpan>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedCanvas {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) fps: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedMediaKind {
    Image,
    Video,
    Audio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedMediaResource {
    pub(crate) asset_id: String,
    pub(crate) kind: EvaluatedMediaKind,
    pub(crate) has_audio: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct EvaluatedLayerOrder {
    pub(crate) track_index: usize,
    pub(crate) item_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTimeSpan {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedTransform {
    pub(crate) position_x: f64,
    pub(crate) position_y: f64,
    pub(crate) scale: f64,
    pub(crate) opacity: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedProperty {
    Position,
    Scale,
    Opacity,
    Volume,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedEasing {
    Hold,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EvaluatedKeyframeValue {
    Position { x: f64, y: f64 },
    Scalar { value: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedKeyframe {
    pub(crate) property: EvaluatedProperty,
    pub(crate) time_ms: u64,
    pub(crate) value: EvaluatedKeyframeValue,
    pub(crate) easing: EvaluatedEasing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTransitionRole {
    In,
    Out,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTransitionKind {
    Fade,
    Crossfade,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTransition {
    pub(crate) role: EvaluatedTransitionRole,
    pub(crate) kind: EvaluatedTransitionKind,
    pub(crate) span: EvaluatedTimeSpan,
}

#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedVisualLayer {
    pub(crate) instance: Option<EvaluatedInstance>,
    pub(crate) item_id: String,
    pub(crate) order: EvaluatedLayerOrder,
    pub(crate) span: EvaluatedTimeSpan,
    pub(crate) transform: EvaluatedTransform,
    pub(crate) transform2d: Option<crate::Transform2D>,
    pub(crate) affine: Option<EvaluatedAffine>,
    pub(crate) sampling_tiles: Option<Vec<EvaluatedAffine>>,
    pub(crate) ancestors: Option<EvaluatedAncestors>,
    pub(crate) source_size: Option<(u32, u32)>,
    pub(crate) keyframes: Vec<EvaluatedKeyframe>,
    pub(crate) transitions: Vec<EvaluatedTransition>,
    pub(crate) source: EvaluatedVisualSource,
}

// Absent additive facts carry no semantics; keep legacy scene diagnostics stable.
impl std::fmt::Debug for EvaluatedVisualLayer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut layer = formatter.debug_struct("EvaluatedVisualLayer");
        layer
            .field("item_id", &self.item_id)
            .field("order", &self.order)
            .field("span", &self.span)
            .field("transform", &self.transform);
        if let Some(value) = self.transform2d {
            layer.field("transform2d", &value);
        }
        if let Some(value) = self.ancestors {
            layer.field("ancestors", &value);
        }
        if let Some(value) = &self.sampling_tiles {
            layer.field("sampling_tiles", value);
        }
        if let Some(value) = self.affine {
            layer.field("affine", &value);
        }
        if let Some(value) = self.source_size {
            layer.field("source_size", &value);
        }
        layer
            .field("keyframes", &self.keyframes)
            .field("transitions", &self.transitions)
            .field("source", &self.source)
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EvaluatedVisualSource {
    Media {
        asset_id: String,
        source_in_ms: u64,
    },
    Text(Box<EvaluatedText>),
    SolidColor {
        color: String,
    },
    Shape(Box<shapes::EvaluatedShape>),
    Rectangle {
        color: String,
        width: u32,
        height: u32,
    },
    Caption(EvaluatedCaption),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedCaption {
    pub(crate) text: String,
    pub(crate) font_size: u32,
    pub(crate) color: String,
    pub(crate) background_color: String,
    pub(crate) bottom_margin_px: u32,
}

impl std::fmt::Debug for EvaluatedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_struct("EvaluatedText");
        value.field("text", &self.text);
        value.field("font_size", &self.font_size);
        value.field("color", &self.color);
        value.field("font_resource_id", &self.font_resource_id);
        value.field("style", &self.style);
        if let Some(extra) = &self.rich_runs {
            value.field("rich_runs", extra);
        }
        value.finish()
    }
}
#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedText {
    pub(crate) rich_runs: Option<Vec<crate::RichTextRun>>,
    pub(crate) text: String,
    pub(crate) font_size: u32,
    pub(crate) color: String,
    pub(crate) font_resource_id: Option<String>,
    pub(crate) style: EvaluatedTextStyle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedTextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedAnchorPoint {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluatedTextPadding {
    pub(crate) top: u32,
    pub(crate) right: u32,
    pub(crate) bottom: u32,
    pub(crate) left: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedTextShadow {
    pub(crate) color: String,
    pub(crate) opacity: f64,
    pub(crate) offset_x: i32,
    pub(crate) offset_y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedTextStyle {
    pub(crate) alignment: EvaluatedTextAlignment,
    pub(crate) wrap_width_px: Option<u32>,
    pub(crate) line_spacing_px: i32,
    pub(crate) outline_color: String,
    pub(crate) outline_width_px: u32,
    pub(crate) shadow: EvaluatedTextShadow,
    pub(crate) background_color: String,
    pub(crate) background_opacity: f64,
    pub(crate) padding: EvaluatedTextPadding,
    pub(crate) anchor: EvaluatedAnchorPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvaluatedAudioRole {
    Unassigned,
    Voiceover,
    Music,
    SoundEffects,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedDucking {
    pub(crate) gain: f64,
    pub(crate) attack_ms: u64,
    pub(crate) release_ms: u64,
}

impl std::fmt::Debug for EvaluatedAudioLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_struct("EvaluatedAudioLayer");
        value.field("item_id", &self.item_id);
        value.field("order", &self.order);
        value.field("asset_id", &self.asset_id);
        value.field("span", &self.span);
        value.field("source_in_ms", &self.source_in_ms);
        value.field("volume", &self.volume);
        value.field("fade_in_ms", &self.fade_in_ms);
        value.field("fade_out_ms", &self.fade_out_ms);
        value.field("volume_keyframes", &self.volume_keyframes);
        value.field("role", &self.role);
        value.field("ducking", &self.ducking);
        if let Some(extra) = &self.instance {
            value.field("instance", extra);
        }
        value.finish()
    }
}
#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedAudioLayer {
    pub(crate) instance: Option<EvaluatedInstance>,
    pub(crate) item_id: String,
    pub(crate) order: EvaluatedLayerOrder,
    pub(crate) asset_id: String,
    pub(crate) span: EvaluatedTimeSpan,
    pub(crate) source_in_ms: u64,
    pub(crate) volume: f64,
    pub(crate) fade_in_ms: u64,
    pub(crate) fade_out_ms: u64,
    pub(crate) volume_keyframes: Vec<EvaluatedKeyframe>,
    pub(crate) role: EvaluatedAudioRole,
    pub(crate) ducking: Option<EvaluatedDucking>,
}

struct EvaluationPreflight<'a> {
    visual_item_ids: HashSet<&'a str>,
    visual_layer_count: usize,
    media_resource_count: usize,
    audio_layer_count: usize,
    voiceover_activity_range_count: usize,
}

fn evaluate_flat_project(
    project: &Project,
    width: u32,
    height: u32,
    fps: u32,
    shape_budget: usize,
    retained: bool,
) -> Result<EvaluatedSceneResult, CoreError> {
    if width == 0 || height == 0 || fps == 0 {
        return Err(invalid(
            "evaluated canvas dimensions and frame rate must be positive",
        ));
    }

    let asset_by_id = project
        .assets
        .iter()
        .map(|asset| (asset.id.as_str(), asset))
        .collect::<HashMap<_, _>>();
    validate_referenced_assets(project, &asset_by_id, retained)?;
    validate_media_source_ranges(project, &asset_by_id, retained)?;
    let preflight = preflight_project(project, &asset_by_id, shape_budget, retained)?;
    validate_project_stacking(project)?;
    crate::validation::validate_parent_graph(project)?;
    let duration_ms = checked_project_duration(project)?.max(1);
    let transition_index = index_transitions(project, &preflight.visual_item_ids)?;
    let voiceover_intervals = audible_voiceover_intervals(
        project,
        &asset_by_id,
        preflight.voiceover_activity_range_count,
    )?;

    let mut resources = Vec::with_capacity(preflight.media_resource_count);
    let mut media_bindings = Vec::with_capacity(preflight.media_resource_count);
    let mut resource_indexes = HashSet::with_capacity(preflight.media_resource_count);
    let mut font_bindings = Vec::new();
    let mut visual_layers = Vec::with_capacity(preflight.visual_layer_count);
    let mut audio_layers = Vec::with_capacity(preflight.audio_layer_count);

    for (track_index, track) in project.tracks.iter().enumerate() {
        if !retained && track.hidden {
            continue;
        }
        for (item_index, item) in track.items.iter().enumerate() {
            if !retained && item.hidden() {
                continue;
            }
            let order = EvaluatedLayerOrder {
                track_index,
                item_index,
            };
            match item {
                TimelineItem::Media(media) => {
                    let span = checked_span(media.start_ms, media.duration_ms)?;
                    let asset = asset_by_id[media.asset_id.as_str()];
                    add_resource(
                        asset,
                        &mut resources,
                        &mut media_bindings,
                        &mut resource_indexes,
                    );
                    let keyframes = evaluate_keyframes(&media.keyframes)?;
                    let volume_keyframes = keyframes
                        .iter()
                        .copied()
                        .filter(|keyframe| keyframe.property == EvaluatedProperty::Volume)
                        .collect();
                    let transform = evaluate_transform(&media.transform)?;
                    if asset.media_type != MediaType::Audio {
                        visual_layers.push(EvaluatedVisualLayer {
                            instance: None,
                            transform2d: item.visual_properties().transform2d,
                            affine: None,
                            sampling_tiles: None,
                            ancestors: None,
                            source_size: None,
                            item_id: media.id.clone(),
                            order,
                            span,
                            transform,
                            keyframes,
                            transitions: transitions_for(&media.id, &transition_index),
                            source: EvaluatedVisualSource::Media {
                                asset_id: media.asset_id.clone(),
                                source_in_ms: media.source_in_ms,
                            },
                        });
                    }
                    if asset.has_audio && !track.muted && !media.audio.muted {
                        if !media.audio.volume.is_finite() {
                            return Err(invalid("evaluated audio volume must be finite"));
                        }
                        let ducking = evaluate_ducking(track, !voiceover_intervals.is_empty())?;
                        audio_layers.push(EvaluatedAudioLayer {
                            instance: None,
                            item_id: media.id.clone(),
                            order,
                            asset_id: media.asset_id.clone(),
                            span,
                            source_in_ms: media.source_in_ms,
                            volume: media.audio.volume,
                            fade_in_ms: media.audio.fade_in_ms,
                            fade_out_ms: media.audio.fade_out_ms,
                            volume_keyframes,
                            role: evaluate_audio_role(track.audio_role),
                            ducking,
                        });
                    }
                }
                TimelineItem::Text(text) => {
                    let font_resource_id = (text.font_path.is_some() || text.font_family.is_some())
                        .then(|| format!("text-font:{}", text.id));
                    if let Some(font_resource_id) = &font_resource_id {
                        font_bindings.push(FontResourceBinding {
                            font_resource_id: font_resource_id.clone(),
                            requested_path: text.font_path.clone(),
                            requested_family: text.font_family.clone(),
                        });
                    }
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: text.id.clone(),
                        order,
                        span: checked_span(text.start_ms, text.duration_ms)?,
                        transform: evaluate_transform(&text.transform)?,
                        keyframes: evaluate_keyframes(&text.keyframes)?,
                        transitions: transitions_for(&text.id, &transition_index),
                        source: EvaluatedVisualSource::Text(Box::new(EvaluatedText {
                            rich_runs: None,
                            text: text.text.clone(),
                            font_size: text.font_size,
                            color: text.color.clone(),
                            font_resource_id,
                            style: evaluate_text_style(&text.style)?,
                        })),
                    });
                }
                TimelineItem::SolidColor(color) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: color.id.clone(),
                        order,
                        span: checked_span(color.start_ms, color.duration_ms)?,
                        transform: evaluate_transform(&color.transform)?,
                        keyframes: evaluate_keyframes(&color.keyframes)?,
                        transitions: transitions_for(&color.id, &transition_index),
                        source: EvaluatedVisualSource::SolidColor {
                            color: color.color.clone(),
                        },
                    });
                }
                TimelineItem::Rectangle(rectangle) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: rectangle.id.clone(),
                        order,
                        span: checked_span(rectangle.start_ms, rectangle.duration_ms)?,
                        transform: evaluate_transform(&rectangle.transform)?,
                        keyframes: evaluate_keyframes(&rectangle.keyframes)?,
                        transitions: transitions_for(&rectangle.id, &transition_index),
                        source: EvaluatedVisualSource::Rectangle {
                            color: rectangle.color.clone(),
                            width: rectangle.width,
                            height: rectangle.height,
                        },
                    });
                }
                TimelineItem::Shape(rectangle) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: rectangle.id.clone(),
                        order,
                        span: checked_span(rectangle.start_ms, rectangle.duration_ms)?,
                        transform: evaluate_transform(&rectangle.transform)?,
                        keyframes: evaluate_keyframes(&rectangle.keyframes)?,
                        transitions: transitions_for(&rectangle.id, &transition_index),
                        source: EvaluatedVisualSource::Shape(Box::new(
                            shapes::EvaluatedShape::new(
                                rectangle.geometry.clone(),
                                rectangle.fill.clone(),
                                rectangle.stroke.clone(),
                                1.0,
                            )?,
                        )),
                    });
                }
                TimelineItem::Svg(rectangle) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: rectangle.id.clone(),
                        order,
                        span: checked_span(rectangle.start_ms, rectangle.duration_ms)?,
                        transform: evaluate_transform(&rectangle.transform)?,
                        keyframes: evaluate_keyframes(&rectangle.keyframes)?,
                        transitions: transitions_for(&rectangle.id, &transition_index),
                        source: EvaluatedVisualSource::Shape(Box::new(
                            shapes::EvaluatedShape::pending_svg(rectangle.document.clone()),
                        )),
                    });
                }
                TimelineItem::Grid(rectangle) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: rectangle.id.clone(),
                        order,
                        span: checked_span(rectangle.start_ms, rectangle.duration_ms)?,
                        transform: evaluate_transform(&rectangle.transform)?,
                        keyframes: evaluate_keyframes(&rectangle.keyframes)?,
                        transitions: transitions_for(&rectangle.id, &transition_index),
                        source: EvaluatedVisualSource::Shape(Box::new(
                            shapes::EvaluatedShape::pending_grid(rectangle.grid.clone()),
                        )),
                    });
                }
                TimelineItem::Caption(caption) => {
                    visual_layers.push(EvaluatedVisualLayer {
                        instance: None,
                        transform2d: item.visual_properties().transform2d,
                        affine: None,
                        sampling_tiles: None,
                        ancestors: None,
                        source_size: None,
                        item_id: caption.id.clone(),
                        order,
                        span: checked_span(caption.start_ms, caption.duration_ms)?,
                        transform: EvaluatedTransform {
                            position_x: 0.0,
                            position_y: 0.0,
                            scale: 1.0,
                            opacity: 1.0,
                        },
                        keyframes: vec![],
                        transitions: vec![],
                        source: EvaluatedVisualSource::Caption(EvaluatedCaption {
                            text: caption.text.clone(),
                            font_size: caption.style.font_size,
                            color: caption.style.color.clone(),
                            background_color: caption.style.background_color.clone(),
                            bottom_margin_px: caption.style.bottom_margin_px,
                        }),
                    });
                }
                TimelineItem::Transition(_)
                | TimelineItem::Group(_)
                | TimelineItem::ComponentInstance(_)
                | TimelineItem::Repeater(_) => {}
            }
        }
    }

    apply_ancestors(project, &mut visual_layers, (width, height), retained)?;
    for layer in &mut visual_layers {
        if matches!(layer.source, EvaluatedVisualSource::Shape(_)) && layer.ancestors.is_none() {
            layer.ancestors = Some(EvaluatedAncestors {
                matrix: IDENTITY_MATRIX,
                inverse: IDENTITY_MATRIX,
                opacity: 1.0,
                clip: layer.span,
            });
        }
        if let Some(value) = layer.transform2d {
            value.validate()?;
            if layer
                .keyframes
                .iter()
                .any(|key| key.property != EvaluatedProperty::Volume)
            {
                return Err(invalid("Transform2D cannot use legacy transform keyframes"));
            }
        }
        if layer.requires_affine() {
            layer.source_size = match &layer.source {
                EvaluatedVisualSource::Rectangle { width, height, .. } => Some((*width, *height)),
                EvaluatedVisualSource::Shape(shape) => Some(shape.size),
                EvaluatedVisualSource::SolidColor { .. } => Some((width, height)),
                _ => None,
            };
            if let Some(size) = layer.source_size
                && !matches!(layer.source, EvaluatedVisualSource::Shape(_))
            {
                layer.affine = Some(evaluate_layer_affine(layer, size, (width, height))?);
            }
        }
    }
    sort_visual_layers(project, &mut visual_layers);
    Ok(EvaluatedSceneResult {
        scene: EvaluatedScene {
            instance_voiceover_intervals: None,
            voiceover_activity_range_count: preflight.voiceover_activity_range_count,
            canvas: EvaluatedCanvas { width, height, fps },
            duration_ms,
            resources,
            visual_layers,
            audio_layers,
            voiceover_intervals,
        },
        resource_bindings: SceneResourceBindings {
            media: media_bindings,
            fonts: font_bindings,
        },
    })
}

fn sort_visual_layers(project: &Project, visual_layers: &mut [EvaluatedVisualLayer]) {
    visual_layers.sort_by(|left, right| {
        let key = |layer: &EvaluatedVisualLayer| {
            let item = &project.tracks[layer.order.track_index].items[layer.order.item_index];
            (
                layer.order.track_index,
                item.visual_properties().z_index,
                layer.order.item_index,
            )
        };
        key(left)
            .cmp(&key(right))
            .then_with(|| left.item_id.cmp(&right.item_id))
    });
}

fn is_descendant_of(project: &Project, item_id: &str, group_id: &str) -> bool {
    let mut current = project.find_item(item_id);
    for _ in 0..=32 {
        let Some(parent_id) = current
            .and_then(|item| item.visual_properties().parent.as_ref())
            .map(|parent| parent.id.as_str())
        else {
            return false;
        };
        if parent_id == group_id {
            return true;
        }
        current = project.find_item(parent_id);
    }
    false
}

fn parent_matrices(
    tracks: &[Track],
    source: &TimelineItem,
    canvas: (u32, u32),
) -> Result<([f64; 6], [f64; 6]), CoreError> {
    let index = tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| (item.id(), item))
        .collect::<HashMap<_, _>>();
    let mut matrix = IDENTITY_MATRIX;
    let mut inverse = IDENTITY_MATRIX;
    let mut node = source;
    while let Some(parent) = &node.visual_properties().parent {
        let target = index
            .get(parent.id.as_str())
            .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "parent group missing"))?;
        let transform = target.visual_properties().transform2d.unwrap_or_default();
        let (next, next_inverse) = transform_matrices(transform, canvas, canvas)?;
        matrix = multiply_matrix(next, matrix);
        inverse = multiply_matrix(inverse, next_inverse);
        node = target;
    }
    Ok((matrix, inverse))
}

fn validate_referenced_assets(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
    retained: bool,
) -> Result<(), CoreError> {
    for track in project
        .tracks
        .iter()
        .filter(|track| retained || !track.hidden)
    {
        for item in track.items.iter().filter(|item| retained || !item.hidden()) {
            if let TimelineItem::Media(media) = item
                && !asset_by_id.contains_key(media.asset_id.as_str())
            {
                return Err(CoreError::new(
                    ErrorCode::AssetNotFound,
                    "timeline references a missing asset",
                ));
            }
        }
    }
    Ok(())
}

fn validate_media_source_ranges(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
    retained: bool,
) -> Result<(), CoreError> {
    for track in project
        .tracks
        .iter()
        .filter(|track| retained || !track.hidden)
    {
        for item in track.items.iter().filter(|item| retained || !item.hidden()) {
            let TimelineItem::Media(media) = item else {
                continue;
            };
            let asset = asset_by_id[media.asset_id.as_str()];
            if asset.media_type == MediaType::Image {
                continue;
            }
            let source_end_ms = media
                .source_in_ms
                .checked_add(media.duration_ms)
                .ok_or_else(|| invalid("evaluated media source interval overflows milliseconds"))?;
            if asset
                .duration_ms
                .is_some_and(|duration_ms| source_end_ms > duration_ms)
            {
                return Err(invalid(
                    "evaluated media source interval exceeds asset duration",
                ));
            }
        }
    }
    Ok(())
}

fn preflight_project<'a>(
    project: &'a Project,
    asset_by_id: &HashMap<&str, &Asset>,
    shape_budget: usize,
    retained: bool,
) -> Result<EvaluationPreflight<'a>, CoreError> {
    let mut visual_item_ids = HashSet::new();
    let mut media_resource_ids = HashSet::new();
    let mut visual_layer_count = 0_usize;
    let mut audio_layer_count = 0_usize;
    let mut voiceover_activity_range_count = 0_usize;
    let mut shape_segments = 0_usize;

    for track in project
        .tracks
        .iter()
        .filter(|track| retained || !track.hidden)
    {
        for item in track.items.iter().filter(|item| retained || !item.hidden()) {
            match item {
                TimelineItem::Media(media) => {
                    validate_keyframe_limit(&media.keyframes)?;
                    let asset = asset_by_id[media.asset_id.as_str()];
                    if media_resource_ids.insert(media.asset_id.as_str())
                        && media_resource_ids.len() > MAX_EVALUATED_MEDIA_RESOURCES
                    {
                        return Err(invalid("evaluated media resource limit exceeded"));
                    }
                    if asset.media_type != MediaType::Audio {
                        increment_bounded(
                            &mut visual_layer_count,
                            MAX_EVALUATED_VISUAL_LAYERS,
                            "evaluated visual layer limit exceeded",
                        )?;
                        visual_item_ids.insert(media.id.as_str());
                    }
                    if asset.has_audio && !track.muted && !media.audio.muted {
                        increment_bounded(
                            &mut audio_layer_count,
                            MAX_EVALUATED_AUDIO_LAYERS,
                            "evaluated audio layer limit exceeded",
                        )?;
                    }
                    if track.audio_role == AudioTrackRole::Voiceover
                        && !track.muted
                        && !media.audio.muted
                        && media.audio.volume != 0.0
                        && asset.has_audio
                    {
                        let item_range_count = positive_scalar_ranges(
                            &media.keyframes,
                            KeyframeProperty::Volume,
                            media.duration_ms,
                        )
                        .len();
                        voiceover_activity_range_count = voiceover_activity_range_count
                            .checked_add(item_range_count)
                            .ok_or_else(|| {
                                invalid("evaluated voiceover activity range limit exceeded")
                            })?;
                        if voiceover_activity_range_count > MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES
                        {
                            return Err(invalid(
                                "evaluated voiceover activity range limit exceeded",
                            ));
                        }
                    }
                }
                TimelineItem::Text(text) => {
                    validate_keyframe_limit(&text.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(text.id.as_str());
                }
                TimelineItem::SolidColor(color) => {
                    validate_keyframe_limit(&color.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(color.id.as_str());
                }
                TimelineItem::Rectangle(rectangle) => {
                    validate_keyframe_limit(&rectangle.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(rectangle.id.as_str());
                }
                TimelineItem::Shape(rectangle) => {
                    shape_segments += shapes::EvaluatedShape::new(
                        rectangle.geometry.clone(),
                        rectangle.fill.clone(),
                        rectangle.stroke.clone(),
                        1.0,
                    )?
                    .segments();
                    if shape_segments > shape_budget {
                        return Err(invalid("scene shape segment limit exceeded"));
                    }
                    validate_keyframe_limit(&rectangle.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(rectangle.id.as_str());
                }
                TimelineItem::Svg(rectangle) => {
                    crate::validation::svg::validate_document(&rectangle.document)?;
                    validate_keyframe_limit(&rectangle.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(rectangle.id.as_str());
                }
                TimelineItem::Grid(rectangle) => {
                    crate::validation::grid::validate_grid(&rectangle.grid)?;
                    validate_keyframe_limit(&rectangle.keyframes)?;
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(rectangle.id.as_str());
                }
                TimelineItem::Caption(caption) => {
                    increment_bounded(
                        &mut visual_layer_count,
                        MAX_EVALUATED_VISUAL_LAYERS,
                        "evaluated visual layer limit exceeded",
                    )?;
                    visual_item_ids.insert(caption.id.as_str());
                }
                TimelineItem::Transition(_)
                | TimelineItem::Group(_)
                | TimelineItem::ComponentInstance(_)
                | TimelineItem::Repeater(_) => {}
            }
        }
    }

    let mut transition_fact_count = 0_usize;
    for transition in visible_transitions(project) {
        if visual_item_ids.contains(transition.from_item_id.as_str()) {
            increment_bounded(
                &mut transition_fact_count,
                MAX_EVALUATED_TRANSITION_FACTS,
                "evaluated transition fact limit exceeded",
            )?;
        }
        if transition
            .to_item_id
            .as_deref()
            .is_some_and(|item_id| visual_item_ids.contains(item_id))
        {
            increment_bounded(
                &mut transition_fact_count,
                MAX_EVALUATED_TRANSITION_FACTS,
                "evaluated transition fact limit exceeded",
            )?;
        }
    }

    Ok(EvaluationPreflight {
        visual_item_ids,
        visual_layer_count,
        media_resource_count: media_resource_ids.len(),
        audio_layer_count,
        voiceover_activity_range_count,
    })
}

fn visible_transitions(project: &Project) -> impl Iterator<Item = &TransitionItem> {
    project
        .tracks
        .iter()
        .filter(|track| !track.hidden)
        .flat_map(|track| &track.items)
        .filter_map(|item| match item {
            TimelineItem::Transition(transition) if !transition.hidden => Some(transition),
            _ => None,
        })
}

fn index_transitions(
    project: &Project,
    visual_item_ids: &HashSet<&str>,
) -> Result<HashMap<String, Vec<EvaluatedTransition>>, CoreError> {
    let mut index = HashMap::with_capacity(visual_item_ids.len());
    for transition in visible_transitions(project) {
        let evaluated = EvaluatedTransition {
            role: EvaluatedTransitionRole::Out,
            kind: evaluate_transition_kind(transition.transition_type),
            span: checked_span(transition.start_ms, transition.duration_ms)?,
        };
        if visual_item_ids.contains(transition.from_item_id.as_str()) {
            index
                .entry(transition.from_item_id.clone())
                .or_insert_with(Vec::new)
                .push(evaluated);
        }
        if let Some(item_id) = transition
            .to_item_id
            .as_ref()
            .filter(|item_id| visual_item_ids.contains(item_id.as_str()))
        {
            index
                .entry(item_id.clone())
                .or_insert_with(Vec::new)
                .push(EvaluatedTransition {
                    role: EvaluatedTransitionRole::In,
                    ..evaluated
                });
        }
    }
    Ok(index)
}

fn transitions_for(
    item_id: &str,
    index: &HashMap<String, Vec<EvaluatedTransition>>,
) -> Vec<EvaluatedTransition> {
    index.get(item_id).cloned().unwrap_or_default()
}

fn add_resource(
    asset: &Asset,
    resources: &mut Vec<EvaluatedMediaResource>,
    bindings: &mut Vec<MediaResourceBinding>,
    indexes: &mut HashSet<String>,
) {
    if !indexes.insert(asset.id.clone()) {
        return;
    }
    resources.push(EvaluatedMediaResource {
        asset_id: asset.id.clone(),
        kind: evaluate_media_kind(asset.media_type),
        has_audio: asset.has_audio,
    });
    bindings.push(MediaResourceBinding {
        asset_id: asset.id.clone(),
        project_relative_path: asset.project_relative_path.clone(),
    });
}

fn checked_project_duration(project: &Project) -> Result<u64, CoreError> {
    project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .try_fold(0, |duration, item| {
            let end = item
                .start_ms()
                .checked_add(item.duration_ms())
                .ok_or_else(|| invalid("evaluated timeline interval overflows milliseconds"))?;
            Ok(duration.max(end))
        })
}

fn checked_span(start_ms: u64, duration_ms: u64) -> Result<EvaluatedTimeSpan, CoreError> {
    if duration_ms == 0 {
        return Err(invalid("evaluated timeline interval must be non-empty"));
    }
    let end_ms = start_ms
        .checked_add(duration_ms)
        .ok_or_else(|| invalid("evaluated timeline interval overflows milliseconds"))?;
    Ok(EvaluatedTimeSpan { start_ms, end_ms })
}

fn evaluate_transform(transform: &Transform) -> Result<EvaluatedTransform, CoreError> {
    if !transform.position_x.is_finite()
        || !transform.position_y.is_finite()
        || !transform.scale.is_finite()
        || !transform.opacity.is_finite()
    {
        return Err(invalid("evaluated transform values must be finite"));
    }
    Ok(EvaluatedTransform {
        position_x: transform.position_x,
        position_y: transform.position_y,
        scale: transform.scale,
        opacity: transform.opacity,
    })
}

fn evaluate_keyframes(keyframes: &[Keyframe]) -> Result<Vec<EvaluatedKeyframe>, CoreError> {
    validate_keyframe_limit(keyframes)?;
    let mut evaluated = Vec::with_capacity(keyframes.len());
    for keyframe in keyframes {
        let value = match keyframe.value {
            KeyframeValue::Position { x, y } if x.is_finite() && y.is_finite() => {
                EvaluatedKeyframeValue::Position { x, y }
            }
            KeyframeValue::Scalar { value } if value.is_finite() => {
                EvaluatedKeyframeValue::Scalar { value }
            }
            _ => return Err(invalid("evaluated keyframe values must be finite")),
        };
        evaluated.push(EvaluatedKeyframe {
            property: evaluate_property(keyframe.property),
            time_ms: keyframe.time_ms,
            value,
            easing: evaluate_easing(keyframe.easing),
        });
    }
    Ok(evaluated)
}

fn validate_keyframe_limit(keyframes: &[Keyframe]) -> Result<(), CoreError> {
    let mut counts = [0_usize; 4];
    for keyframe in keyframes {
        let count = &mut counts[match keyframe.property {
            KeyframeProperty::Position => 0,
            KeyframeProperty::Scale => 1,
            KeyframeProperty::Opacity => 2,
            KeyframeProperty::Volume => 3,
        }];
        increment_bounded(
            count,
            MAX_EVALUATED_KEYFRAMES_PER_CHANNEL,
            "evaluated keyframe channel limit exceeded",
        )?;
    }
    Ok(())
}

fn evaluate_transition_kind(transition_type: TransitionType) -> EvaluatedTransitionKind {
    match transition_type {
        TransitionType::Fade => EvaluatedTransitionKind::Fade,
        TransitionType::Crossfade => EvaluatedTransitionKind::Crossfade,
    }
}

fn audible_voiceover_intervals(
    project: &Project,
    asset_by_id: &HashMap<&str, &Asset>,
    activity_range_count: usize,
) -> Result<Vec<EvaluatedTimeSpan>, CoreError> {
    let mut intervals = Vec::with_capacity(activity_range_count);
    for track in project.tracks.iter().filter(|track| {
        !track.hidden && !track.muted && track.audio_role == AudioTrackRole::Voiceover
    }) {
        for item in &track.items {
            let TimelineItem::Media(media) = item else {
                continue;
            };
            if media.hidden || media.audio.muted || media.audio.volume == 0.0 {
                continue;
            }
            let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetNotFound,
                    "timeline references a missing asset",
                )
            })?;
            if !asset.has_audio {
                continue;
            }
            for (start, end) in positive_scalar_ranges(
                &media.keyframes,
                KeyframeProperty::Volume,
                media.duration_ms,
            ) {
                let start_ms = media.start_ms.checked_add(start).ok_or_else(|| {
                    invalid("evaluated voiceover interval overflows milliseconds")
                })?;
                let end_ms = media.start_ms.checked_add(end).ok_or_else(|| {
                    invalid("evaluated voiceover interval overflows milliseconds")
                })?;
                if start_ms < end_ms {
                    intervals.push(EvaluatedTimeSpan { start_ms, end_ms });
                }
            }
        }
    }
    intervals.sort_unstable_by_key(|span| (span.start_ms, span.end_ms));
    let mut merged: Vec<EvaluatedTimeSpan> = Vec::with_capacity(intervals.len());
    for span in intervals {
        if let Some(previous) = merged.last_mut()
            && span.start_ms <= previous.end_ms
        {
            previous.end_ms = previous.end_ms.max(span.end_ms);
        } else {
            merged.push(span);
        }
    }
    Ok(merged)
}

fn evaluate_ducking(
    track: &Track,
    has_voiceover_activity: bool,
) -> Result<Option<EvaluatedDucking>, CoreError> {
    let Some(settings) = track.ducking.as_ref().filter(|settings| settings.enabled) else {
        return Ok(None);
    };
    if track.audio_role != AudioTrackRole::Music || !has_voiceover_activity {
        return Ok(None);
    }
    if !settings.gain.is_finite() {
        return Err(invalid("evaluated ducking gain must be finite"));
    }
    Ok(Some(EvaluatedDucking {
        gain: settings.gain,
        attack_ms: settings.attack_ms,
        release_ms: settings.release_ms,
    }))
}

fn evaluate_text_style(style: &TextStyle) -> Result<EvaluatedTextStyle, CoreError> {
    if !style.shadow.opacity.is_finite() || !style.background_opacity.is_finite() {
        return Err(invalid("evaluated text style values must be finite"));
    }
    Ok(EvaluatedTextStyle {
        alignment: match style.alignment {
            TextAlignment::Left => EvaluatedTextAlignment::Left,
            TextAlignment::Center => EvaluatedTextAlignment::Center,
            TextAlignment::Right => EvaluatedTextAlignment::Right,
        },
        wrap_width_px: style.wrap_width_px,
        line_spacing_px: style.line_spacing_px,
        outline_color: style.outline_color.clone(),
        outline_width_px: style.outline_width_px,
        shadow: EvaluatedTextShadow {
            color: style.shadow.color.clone(),
            opacity: style.shadow.opacity,
            offset_x: style.shadow.offset_x,
            offset_y: style.shadow.offset_y,
        },
        background_color: style.background_color.clone(),
        background_opacity: style.background_opacity,
        padding: EvaluatedTextPadding {
            top: style.padding.top,
            right: style.padding.right,
            bottom: style.padding.bottom,
            left: style.padding.left,
        },
        anchor: match style.anchor {
            AnchorPoint::TopLeft => EvaluatedAnchorPoint::TopLeft,
            AnchorPoint::TopCenter => EvaluatedAnchorPoint::TopCenter,
            AnchorPoint::TopRight => EvaluatedAnchorPoint::TopRight,
            AnchorPoint::CenterLeft => EvaluatedAnchorPoint::CenterLeft,
            AnchorPoint::Center => EvaluatedAnchorPoint::Center,
            AnchorPoint::CenterRight => EvaluatedAnchorPoint::CenterRight,
            AnchorPoint::BottomLeft => EvaluatedAnchorPoint::BottomLeft,
            AnchorPoint::BottomCenter => EvaluatedAnchorPoint::BottomCenter,
            AnchorPoint::BottomRight => EvaluatedAnchorPoint::BottomRight,
        },
    })
}

fn evaluate_media_kind(kind: MediaType) -> EvaluatedMediaKind {
    match kind {
        MediaType::Image => EvaluatedMediaKind::Image,
        MediaType::Video => EvaluatedMediaKind::Video,
        MediaType::Audio => EvaluatedMediaKind::Audio,
    }
}

fn evaluate_property(property: KeyframeProperty) -> EvaluatedProperty {
    match property {
        KeyframeProperty::Position => EvaluatedProperty::Position,
        KeyframeProperty::Scale => EvaluatedProperty::Scale,
        KeyframeProperty::Opacity => EvaluatedProperty::Opacity,
        KeyframeProperty::Volume => EvaluatedProperty::Volume,
    }
}

fn evaluate_easing(easing: Easing) -> EvaluatedEasing {
    match easing {
        Easing::Hold => EvaluatedEasing::Hold,
        Easing::Linear => EvaluatedEasing::Linear,
        Easing::EaseIn => EvaluatedEasing::EaseIn,
        Easing::EaseOut => EvaluatedEasing::EaseOut,
        Easing::EaseInOut => EvaluatedEasing::EaseInOut,
    }
}

fn evaluate_audio_role(role: AudioTrackRole) -> EvaluatedAudioRole {
    match role {
        AudioTrackRole::Unassigned => EvaluatedAudioRole::Unassigned,
        AudioTrackRole::Voiceover => EvaluatedAudioRole::Voiceover,
        AudioTrackRole::Music => EvaluatedAudioRole::Music,
        AudioTrackRole::SoundEffects => EvaluatedAudioRole::SoundEffects,
    }
}

fn increment_bounded(
    count: &mut usize,
    limit: usize,
    message: &'static str,
) -> Result<(), CoreError> {
    if *count == limit {
        return Err(invalid(message));
    }
    *count += 1;
    Ok(())
}

fn invalid(message: &'static str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AudioSettings, DuckingSettings, MediaItem, ProjectSettings, RectangleItem, SolidColorItem,
        TextItem, TextStyle, TrackType, TransitionItem,
    };

    #[test]
    fn explicit_stacking_respects_tracks_z_index_array_ties_and_hidden_sources() {
        let mut p = project();
        let visual = |id: &str, z: i32, order: u32, hidden: bool| -> TimelineItem {
            serde_json::from_value(serde_json::json!({"type":"solid_color","id":id,"color":"#ff0000","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":z,"stackOrder":order,"hidden":hidden})).unwrap()
        };
        p.tracks = vec![
            track(
                "lower",
                TrackType::Overlay,
                vec![
                    visual("z-first", 4, 0, false),
                    visual("a-second", 4, 1, false),
                    visual("negative", -1, 2, false),
                    visual("hidden", -99, 3, true),
                ],
            ),
            track(
                "upper",
                TrackType::Overlay,
                vec![visual("upper-negative", i32::MIN, 0, false)],
            ),
        ];
        let before = serde_json::to_value(&p).unwrap();
        for _ in 0..3 {
            let evaluated = evaluate_project(&p, 160, 90, 10).unwrap();
            assert_eq!(
                evaluated
                    .scene
                    .visual_layers
                    .iter()
                    .map(|layer| layer.item_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["negative", "z-first", "a-second", "upper-negative"]
            );
            assert!(evaluated.scene.audio_layers.is_empty());
            let mut synthesized = vec![
                evaluated.scene.visual_layers[0].clone(),
                evaluated.scene.visual_layers[0].clone(),
            ];
            synthesized[0].item_id = "z-synthesized".into();
            synthesized[1].item_id = "a-synthesized".into();
            sort_visual_layers(&p, &mut synthesized);
            assert_eq!(synthesized[0].item_id, "a-synthesized");
        }
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
    }

    #[test]
    fn groups_compose_geometry_clip_visibility_and_preserve_source_time() {
        let mut p = project();
        let mut group_transform = crate::Transform2D::default();
        group_transform.position.x = 40.0;
        group_transform.position.y = 10.0;
        group_transform.rotation_deg = 90.0;
        group_transform.opacity = 0.5;
        p.tracks = serde_json::from_value(serde_json::json!([{
            "id":"overlay","name":"Overlay","trackType":"overlay","items":[
                {"type":"group","id":"outer","startMs":100,"durationMs":600,"zIndex":0,"stackOrder":0,"transform2d":group_transform},
                {"type":"group","id":"inner","startMs":200,"durationMs":600,"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"outer"},"transform2d":crate::Transform2D::default()},
                {"type":"rectangle","id":"child","startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","keyframes":[],"zIndex":0,"stackOrder":2,"parent":{"scope":"root","id":"inner"},"transform":{"positionX":5,"positionY":3,"scale":1,"opacity":0.5}}
            ]
        }])).unwrap();
        let before = serde_json::to_value(&p).unwrap();
        let scene = evaluate_project(&p, 64, 64, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 1);
        let layer = &scene.visual_layers[0];
        assert_eq!(
            layer.span,
            EvaluatedTimeSpan {
                start_ms: 0,
                end_ms: 1000
            }
        );
        assert_eq!(
            layer.visible_span(),
            EvaluatedTimeSpan {
                start_ms: 200,
                end_ms: 700
            }
        );
        let affine = layer.affine.unwrap();
        let [a, b, c, d, x, y] = affine.matrix;
        for (px, py) in [(0.0, 0.0), (20.0, 0.0), (0.0, 10.0), (20.0, 10.0)] {
            assert!((a * px + c * py + x - (37.0 - py)).abs() < 1e-9);
            assert!((b * px + d * py + y - (15.0 + px)).abs() < 1e-9);
        }
        assert_eq!(affine.opacity, 0.25);
        assert_eq!(evaluate_project(&p, 64, 64, 30).unwrap().scene, scene);
        assert_eq!(serde_json::to_value(&p).unwrap(), before);
        p.tracks[0].items[0].set_hidden(true);
        assert!(
            evaluate_project(&p, 64, 64, 30)
                .unwrap()
                .scene
                .visual_layers
                .is_empty()
        );
        p.tracks[0].items[0].visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "inner".into(),
        });
        assert_eq!(
            evaluate_project(&p, 64, 64, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn nested_group_oracle_covers_anchor_skew_scale_audio_and_overflow() {
        let mut p = project();
        let mut outer = crate::Transform2D::default();
        outer.position.unit = crate::PositionUnit::Normalized;
        outer.position.x = 0.4;
        outer.position.y = 0.3;
        outer.anchor.x = 0.2;
        outer.anchor.y = 0.7;
        outer.scale_x = 1.2;
        outer.scale_y = 0.8;
        outer.rotation_deg = 31.0;
        outer.skew_x_deg = 12.0;
        outer.skew_y_deg = -8.0;
        outer.opacity = 0.8;
        let mut inner = outer;
        inner.position.unit = crate::PositionUnit::Pixels;
        inner.position.x = 15.0;
        inner.position.y = 17.0;
        inner.rotation_deg = -18.0;
        inner.opacity = 0.5;
        p.tracks=serde_json::from_value(serde_json::json!([{"id":"parents","name":"Parents","trackType":"overlay","items":[
            {"type":"group","id":"outer","startMs":100,"durationMs":700,"stackOrder":0,"transform2d":outer},
            {"type":"group","id":"inner","startMs":200,"durationMs":700,"stackOrder":1,"transform2d":inner,"parent":{"scope":"root","id":"outer"}}
        ]}])).unwrap();
        p.assets = vec![asset("video", MediaType::Video, true)];
        let mut child = media("child", "video", 0);
        child.visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "inner".into(),
        });
        child.visual_properties_mut().transform2d = Some(crate::Transform2D::default());
        p.tracks
            .push(track("visual", TrackType::Video, vec![child]));
        let mut scene = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        finalize_affine_geometry(
            &mut scene,
            &HashMap::from([("child".to_string(), (23, 11))]),
        )
        .unwrap();
        let map = |t: crate::Transform2D, (x, y): (f64, f64)| {
            let x = (x - t.anchor.x * 320.0) * t.scale_x;
            let y = (y - t.anchor.y * 180.0) * t.scale_y;
            let x = x + y * t.skew_x_deg.to_radians().tan();
            let y = y + x * t.skew_y_deg.to_radians().tan();
            let (s, c) = t.rotation_deg.to_radians().sin_cos();
            let (fx, fy) = if t.position.unit == crate::PositionUnit::Normalized {
                (320.0, 180.0)
            } else {
                (1.0, 1.0)
            };
            (
                c * x - s * y + t.position.x * fx,
                s * x + c * y + t.position.y * fy,
            )
        };
        let affine = scene.visual_layers[0].affine.unwrap();
        for (x, y) in [(0.0, 0.0), (23.0, 0.0), (0.0, 11.0), (23.0, 11.0)] {
            let expected = map(outer, map(inner, (x, y)));
            let [a, b, c, d, tx, ty] = affine.matrix;
            assert!((a * x + c * y + tx - expected.0).abs() < 1e-9);
            assert!((b * x + d * y + ty - expected.1).abs() < 1e-9);
        }
        assert_eq!(affine.opacity, 0.4);
        let audio = scene.audio_layers.clone();
        p.tracks[0].hidden = true;
        let hidden = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        assert!(hidden.visual_layers.is_empty());
        assert_eq!(hidden.audio_layers, audio);
        p.tracks[0].hidden = false;
        for group in &mut p.tracks[0].items {
            let t = group.visual_properties_mut().transform2d.as_mut().unwrap();
            t.scale_x = 100.0;
            t.scale_y = 100.0;
        }
        let mut overflow = evaluate_project(&p, 320, 180, 30).unwrap().scene;
        assert_eq!(
            finalize_affine_geometry(
                &mut overflow,
                &HashMap::from([("child".to_string(), (23, 11))])
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn legacy_text_anchors_and_animated_sampling_are_composed_before_parents() {
        for (anchor, ax, ay) in [
            ("top_left", 0.0, 0.0),
            ("top_center", 0.5, 0.0),
            ("top_right", 1.0, 0.0),
            ("center_left", 0.0, 0.5),
            ("center", 0.5, 0.5),
            ("center_right", 1.0, 0.5),
            ("bottom_left", 0.0, 1.0),
            ("bottom_center", 0.5, 1.0),
            ("bottom_right", 1.0, 1.0),
        ] {
            let mut p = project();
            let mut t = crate::Transform2D::default();
            t.position.x = 50.0;
            t.rotation_deg = 90.0;
            p.tracks=serde_json::from_value(serde_json::json!([{"id":"track","name":"Overlay","trackType":"overlay","items":[
                {"type":"group","id":"g","startMs":0,"durationMs":1000,"stackOrder":0,"transform2d":t},
                {"type":"text","id":"text","text":"Anchor","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"stackOrder":1,"style":{"anchor":anchor},"transform":{"positionX":200,"positionY":180,"scale":1.5,"opacity":1},"keyframes":[],"parent":{"scope":"root","id":"g"}}
            ]}])).unwrap();
            let mut scene = evaluate_project(&p, 800, 600, 30).unwrap().scene;
            finalize_affine_geometry(
                &mut scene,
                &HashMap::from([("text".to_string(), (110, 30))]),
            )
            .unwrap();
            let [a, b, c, d, x, y] = scene.visual_layers[0].affine.unwrap().matrix;
            for (px, py) in [(0.0, 0.0), (110.0, 30.0)] {
                assert!(
                    (a * px + c * py + x - (50.0 - (180.0 + 1.5 * (py - ay * 30.0)))).abs() < 1e-9
                );
                assert!((b * px + d * py + y - (200.0 + 1.5 * (px - ax * 110.0))).abs() < 1e-9);
            }
        }
        let mut p = project();
        p.tracks=serde_json::from_value(serde_json::json!([{"id":"track","name":"Overlay","trackType":"overlay","items":[
            {"type":"group","id":"g","startMs":0,"durationMs":1000,"stackOrder":0},
            {"type":"rectangle","id":"r","startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","stackOrder":1,"keyframes":[{"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},{"property":"position","timeMs":1000,"value":{"type":"position","x":20000,"y":0},"easing":"linear"}],"parent":{"scope":"root","id":"g"}}
        ]}])).unwrap();
        let mut scene = evaluate_project(&p, 7680, 4320, 30).unwrap().scene;
        finalize_affine_geometry(&mut scene, &HashMap::new()).unwrap();
        let tiles = scene.visual_layers[0].sampling_tiles.as_ref().unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(
            (tiles[0].left, tiles[0].width, tiles[0].height),
            (0.0, 4096, 10)
        );
        assert_eq!(
            (tiles[1].left, tiles[1].width, tiles[1].height),
            (4096.0, 3584, 10)
        );
        p.tracks[0].items[0].visual_properties_mut().transform2d = Some(crate::Transform2D {
            position: crate::TransformPosition {
                x: 0.0,
                y: 5000.0,
                unit: crate::PositionUnit::Pixels,
            },
            ..Default::default()
        });
        let mut offscreen = evaluate_project(&p, 7680, 4320, 30).unwrap().scene;
        finalize_affine_geometry(&mut offscreen, &HashMap::new()).unwrap();
        assert!(offscreen.visual_layers.is_empty());
    }

    fn project() -> Project {
        Project {
            components: vec![],
            schema_version: crate::PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 7,
            name: "Evaluated scene".into(),
            created_at_ms: 1,
            updated_at_ms: 2,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    fn asset(id: &str, media_type: MediaType, has_audio: bool) -> Asset {
        Asset {
            id: id.into(),
            media_type,
            file_name: format!("{id}.media"),
            project_relative_path: format!("assets/{id}.media"),
            duration_ms: Some(20_000),
            has_audio,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        }
    }

    fn media(id: &str, asset_id: &str, start_ms: u64) -> TimelineItem {
        TimelineItem::Media(MediaItem {
            id: id.into(),
            asset_id: asset_id.into(),
            start_ms,
            duration_ms: 1_000,
            source_in_ms: 125,
            visual_properties: crate::VisualProperties::default(),
            audio: AudioSettings::default(),
            keyframes: vec![],
        })
    }

    fn track(id: &str, track_type: TrackType, mut items: Vec<TimelineItem>) -> Track {
        for (index, item) in items.iter_mut().enumerate() {
            item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
        }
        Track {
            id: id.into(),
            name: id.into(),
            track_type,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: AudioTrackRole::Unassigned,
            ducking: None,
            items,
        }
    }

    #[test]
    fn evaluates_owned_flat_layers_in_stable_order_without_mutating_project() {
        let mut input = project();
        input.assets = vec![
            asset("image", MediaType::Image, false),
            asset("video", MediaType::Video, true),
        ];
        input.tracks = vec![
            track(
                "base",
                TrackType::Video,
                vec![
                    TimelineItem::SolidColor(SolidColorItem {
                        id: "background".into(),
                        color: "#112233".into(),
                        start_ms: 0,
                        duration_ms: 4_000,
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![],
                    }),
                    media("photo", "image", 250),
                ],
            ),
            track(
                "overlay",
                TrackType::Overlay,
                vec![
                    TimelineItem::Rectangle(RectangleItem {
                        id: "panel".into(),
                        color: "#445566".into(),
                        width: 320,
                        height: 180,
                        start_ms: 500,
                        duration_ms: 2_000,
                        visual_properties: crate::VisualProperties::new(
                            Transform {
                                position_x: 20.0,
                                position_y: 30.0,
                                scale: 1.25,
                                opacity: 0.8,
                            },
                            false,
                        ),
                        keyframes: vec![],
                    }),
                    TimelineItem::Text(TextItem {
                        id: "title".into(),
                        text: "Title".into(),
                        start_ms: 750,
                        duration_ms: 1_000,
                        font_size: 48,
                        color: "#ffffff".into(),
                        font_family: Some("Inter".into()),
                        font_path: Some("fonts/private.ttf".into()),
                        style: TextStyle::default(),
                        visual_properties: crate::VisualProperties::default(),
                        keyframes: vec![Keyframe {
                            property: KeyframeProperty::Opacity,
                            time_ms: 0,
                            value: KeyframeValue::Scalar { value: 0.5 },
                            easing: Easing::Linear,
                        }],
                    }),
                    media("clip", "video", 1_000),
                    TimelineItem::Transition(TransitionItem {
                        id: "panel-to-title".into(),
                        transition_type: TransitionType::Crossfade,
                        from_item_id: "panel".into(),
                        to_item_id: Some("title".into()),
                        start_ms: 700,
                        duration_ms: 200,
                        visual_properties: crate::VisualProperties::default(),
                    }),
                ],
            ),
        ];
        let before = serde_json::to_string(&input).unwrap();

        let first = evaluate_project(&input, 1_280, 720, 30).unwrap();
        let second = evaluate_project(&input, 1_280, 720, 30).unwrap();

        assert_eq!(first, second);
        assert_eq!(serde_json::to_string(&input).unwrap(), before);
        assert_eq!(input.revision, 7);
        let scene = &first.scene;
        assert_eq!(
            scene.canvas,
            EvaluatedCanvas {
                width: 1_280,
                height: 720,
                fps: 30
            }
        );
        assert_eq!(scene.duration_ms, 4_000);
        assert_eq!(
            scene
                .visual_layers
                .iter()
                .map(|layer| (layer.item_id.as_str(), layer.order))
                .collect::<Vec<_>>(),
            vec![
                (
                    "background",
                    EvaluatedLayerOrder {
                        track_index: 0,
                        item_index: 0
                    }
                ),
                (
                    "photo",
                    EvaluatedLayerOrder {
                        track_index: 0,
                        item_index: 1
                    }
                ),
                (
                    "panel",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 0
                    }
                ),
                (
                    "title",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 1
                    }
                ),
                (
                    "clip",
                    EvaluatedLayerOrder {
                        track_index: 1,
                        item_index: 2
                    }
                ),
            ]
        );
        assert_eq!(scene.visual_layers[1].span.start_ms, 250);
        assert_eq!(scene.visual_layers[1].span.end_ms, 1_250);
        assert_eq!(
            scene
                .resources
                .iter()
                .map(|resource| resource.asset_id.as_str())
                .collect::<Vec<_>>(),
            vec!["image", "video"]
        );
        let title = scene
            .visual_layers
            .iter()
            .find(|layer| layer.item_id == "title")
            .unwrap();
        let EvaluatedVisualSource::Text(text) = &title.source else {
            panic!("title must evaluate as text");
        };
        assert_eq!(text.font_resource_id.as_deref(), Some("text-font:title"));
        assert!(!format!("{scene:?}").contains("fonts/private.ttf"));
        assert_eq!(
            first.resource_bindings.media,
            vec![
                MediaResourceBinding {
                    asset_id: "image".into(),
                    project_relative_path: "assets/image.media".into(),
                },
                MediaResourceBinding {
                    asset_id: "video".into(),
                    project_relative_path: "assets/video.media".into(),
                },
            ]
        );
        assert_eq!(
            first.resource_bindings.fonts,
            vec![FontResourceBinding {
                font_resource_id: "text-font:title".into(),
                requested_path: Some("fonts/private.ttf".into()),
                requested_family: Some("Inter".into()),
            }]
        );
        assert_eq!(
            scene.visual_layers[2].transitions[0].role,
            EvaluatedTransitionRole::Out
        );
        assert_eq!(title.transitions[0].role, EvaluatedTransitionRole::In);
        assert_eq!(
            title.transitions[0].kind,
            EvaluatedTransitionKind::Crossfade
        );
    }

    #[test]
    fn omits_hidden_visuals_and_resolves_audio_ducking() {
        let mut input = project();
        input.assets = vec![
            asset("voice", MediaType::Audio, true),
            asset("music", MediaType::Audio, true),
            asset("muted", MediaType::Audio, true),
        ];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![media("voice-item", "voice", 1_000)],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        let mut music_item = media("music-item", "music", 0);
        let TimelineItem::Media(music_media) = &mut music_item else {
            unreachable!()
        };
        music_media.duration_ms = 4_000;
        music_media.audio.volume = 0.75;
        music_media.audio.fade_in_ms = 100;
        music_media.audio.fade_out_ms = 200;
        music_media.keyframes = vec![Keyframe {
            property: KeyframeProperty::Volume,
            time_ms: 0,
            value: KeyframeValue::Scalar { value: 0.5 },
            easing: Easing::EaseIn,
        }];
        let mut music = track("music", TrackType::Audio, vec![music_item]);
        music.audio_role = AudioTrackRole::Music;
        music.ducking = Some(DuckingSettings {
            enabled: true,
            gain: 0.25,
            attack_ms: 50,
            release_ms: 75,
        });
        let mut muted = track(
            "muted",
            TrackType::Audio,
            vec![media("muted-item", "muted", 0)],
        );
        muted.muted = true;
        let mut hidden = track(
            "hidden",
            TrackType::Video,
            vec![TimelineItem::Rectangle(RectangleItem {
                id: "hidden-shape".into(),
                color: "#000000".into(),
                width: 10,
                height: 10,
                start_ms: 0,
                duration_ms: 100,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        );
        hidden.hidden = true;
        input.tracks = vec![voice, music, muted, hidden];

        let evaluated = evaluate_project(&input, 640, 360, 24).unwrap();
        let scene = evaluated.scene;

        assert!(scene.visual_layers.is_empty());
        assert_eq!(
            scene
                .audio_layers
                .iter()
                .map(|layer| layer.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["voice-item", "music-item"]
        );
        let music = &scene.audio_layers[1];
        assert_eq!(
            (music.volume, music.fade_in_ms, music.fade_out_ms),
            (0.75, 100, 200)
        );
        assert_eq!(music.volume_keyframes.len(), 1);
        let ducking = music.ducking.as_ref().unwrap();
        assert_eq!(
            (ducking.gain, ducking.attack_ms, ducking.release_ms),
            (0.25, 50, 75)
        );
        assert_eq!(
            scene.voiceover_intervals,
            vec![EvaluatedTimeSpan {
                start_ms: 1_000,
                end_ms: 2_000
            }]
        );
    }

    #[test]
    fn validates_non_image_source_ranges_after_missing_assets() {
        let evaluate_media = |media_type: MediaType,
                              asset_duration_ms: Option<u64>,
                              source_in_ms: u64,
                              duration_ms: u64| {
            let mut input = project();
            let mut source = asset("source", media_type, media_type != MediaType::Image);
            source.duration_ms = asset_duration_ms;
            input.assets = vec![source];
            let mut item = media("item", "source", 0);
            let TimelineItem::Media(media) = &mut item else {
                unreachable!()
            };
            media.source_in_ms = source_in_ms;
            media.duration_ms = duration_ms;
            input.tracks = vec![track("media", TrackType::Video, vec![item])];
            evaluate_project(&input, 16, 16, 1)
        };

        assert!(evaluate_media(MediaType::Video, None, u64::MAX - 1, 1).is_ok());
        assert_eq!(
            evaluate_media(MediaType::Audio, None, u64::MAX, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert!(evaluate_media(MediaType::Video, Some(1_125), 125, 1_000).is_ok());
        assert_eq!(
            evaluate_media(MediaType::Video, Some(1_124), 125, 1_000)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert!(evaluate_media(MediaType::Image, Some(1), u64::MAX, 1).is_ok());

        let mut missing = project();
        let mut missing_item = media("missing", "absent", 0);
        let TimelineItem::Media(media) = &mut missing_item else {
            unreachable!()
        };
        media.source_in_ms = u64::MAX;
        missing.tracks = vec![track("video", TrackType::Video, vec![missing_item])];
        assert_eq!(
            evaluate_project(&missing, 16, 16, 1).unwrap_err().code,
            ErrorCode::AssetNotFound
        );
    }

    #[test]
    fn missing_non_finite_and_invalid_timing_fail_closed() {
        let mut missing = project();
        missing.tracks = vec![track(
            "video",
            TrackType::Video,
            vec![media("missing", "absent", 0)],
        )];
        missing.tracks[0].items[0]
            .visual_properties_mut()
            .transform2d = Some(crate::Transform2D {
            scale_x: 100.0,
            scale_y: 100.0,
            ..Default::default()
        });
        assert_eq!(
            evaluate_project(&missing, 640, 360, 30).unwrap_err().code,
            ErrorCode::AssetNotFound
        );

        let mut non_finite = project();
        non_finite.tracks = vec![track(
            "overlay",
            TrackType::Overlay,
            vec![TimelineItem::Rectangle(RectangleItem {
                id: "bad".into(),
                color: "#ffffff".into(),
                width: 10,
                height: 10,
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::new(
                    Transform {
                        position_x: f64::NAN,
                        ..Transform::default()
                    },
                    false,
                ),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&non_finite, 640, 360, 30)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let mut empty = project();
        empty.tracks = vec![track(
            "base",
            TrackType::Video,
            vec![TimelineItem::SolidColor(SolidColorItem {
                id: "empty".into(),
                color: "#000000".into(),
                start_ms: 0,
                duration_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&empty, 640, 360, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );

        let mut overflow = project();
        overflow.tracks = vec![track(
            "base",
            TrackType::Video,
            vec![TimelineItem::SolidColor(SolidColorItem {
                id: "overflow".into(),
                color: "#000000".into(),
                start_ms: u64::MAX,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        )];
        assert_eq!(
            evaluate_project(&overflow, 640, 360, 30).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    fn with_empty_instance(mut input: Project) -> Project {
        for track in &mut input.tracks {
            for (index, item) in track.items.iter_mut().enumerate() {
                item.visual_properties_mut().stack_order = index as u32;
            }
        }
        input.components.push(serde_json::from_value(serde_json::json!({"id":"empty","name":"Empty","width":16,"height":16,"durationMs":1,"slots":[],"tracks":[]})).unwrap());
        input.tracks.push(track("instances",TrackType::Overlay,vec![serde_json::from_value(serde_json::json!({"type":"component_instance","id":"empty-instance","componentId":"empty","startMs":0,"durationMs":1,"trimStartMs":0,"timeScale":1,"slotValues":{}})).unwrap()]));
        input
    }

    #[test]
    fn accepts_each_scene_limit_and_rejects_boundary_plus_one() {
        let visual_project = |count: usize| {
            let mut input = project();
            input.tracks = vec![track(
                "base",
                TrackType::Video,
                (0..count)
                    .map(|index| {
                        TimelineItem::SolidColor(SolidColorItem {
                            id: format!("visual-{index}"),
                            color: "#000000".into(),
                            start_ms: 0,
                            duration_ms: 1,
                            visual_properties: crate::VisualProperties::default(),
                            keyframes: vec![],
                        })
                    })
                    .collect(),
            )];
            input
        };
        assert!(evaluate_project(&visual_project(MAX_EVALUATED_VISUAL_LAYERS), 16, 16, 1).is_ok());
        assert_eq!(
            evaluate_project(&visual_project(MAX_EVALUATED_VISUAL_LAYERS + 1), 16, 16, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let audio_project = |count: usize| {
            let mut input = project();
            input.assets = vec![asset("shared", MediaType::Audio, true)];
            input.tracks = vec![track(
                "audio",
                TrackType::Audio,
                (0..count)
                    .map(|index| media(&format!("audio-{index}"), "shared", 0))
                    .collect(),
            )];
            input
        };
        assert!(evaluate_project(&audio_project(MAX_EVALUATED_AUDIO_LAYERS), 16, 16, 1).is_ok());
        assert_eq!(
            evaluate_project(&audio_project(MAX_EVALUATED_AUDIO_LAYERS + 1), 16, 16, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );

        let resource_project = |count: usize| {
            let mut input = project();
            input.assets = (0..count)
                .map(|index| asset(&format!("asset-{index}"), MediaType::Audio, false))
                .collect();
            input.tracks = vec![track(
                "audio",
                TrackType::Audio,
                (0..count)
                    .map(|index| media(&format!("item-{index}"), &format!("asset-{index}"), 0))
                    .collect(),
            )];
            input
        };
        assert!(
            evaluate_project(&resource_project(MAX_EVALUATED_MEDIA_RESOURCES), 16, 16, 1).is_ok()
        );
        assert!(
            evaluate_project(
                &with_empty_instance(resource_project(MAX_EVALUATED_MEDIA_RESOURCES)),
                16,
                16,
                1
            )
            .is_ok()
        );
        assert_eq!(
            evaluate_project(
                &with_empty_instance(resource_project(MAX_EVALUATED_MEDIA_RESOURCES + 1)),
                16,
                16,
                1
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            evaluate_project(
                &resource_project(MAX_EVALUATED_MEDIA_RESOURCES + 1),
                16,
                16,
                1
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );

        let keyframe_project = |count: usize| {
            let mut input = project();
            input.tracks = vec![track(
                "overlay",
                TrackType::Overlay,
                vec![TimelineItem::Text(TextItem {
                    id: "animated".into(),
                    text: "Animated".into(),
                    start_ms: 0,
                    duration_ms: count as u64 + 1,
                    font_size: 20,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: TextStyle::default(),
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: (0..count)
                        .map(|index| Keyframe {
                            property: KeyframeProperty::Position,
                            time_ms: index as u64,
                            value: KeyframeValue::Position {
                                x: index as f64,
                                y: 0.0,
                            },
                            easing: Easing::Linear,
                        })
                        .collect(),
                })],
            )];
            input
        };
        assert!(
            evaluate_project(
                &keyframe_project(MAX_EVALUATED_KEYFRAMES_PER_CHANNEL),
                16,
                16,
                1
            )
            .is_ok()
        );
        assert_eq!(
            evaluate_project(
                &keyframe_project(MAX_EVALUATED_KEYFRAMES_PER_CHANNEL + 1),
                16,
                16,
                1
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn rejects_missing_assets_before_scene_complexity() {
        let mut input = project();
        let mut items = vec![media("missing", "absent", 0)];
        items.extend((0..=MAX_EVALUATED_VISUAL_LAYERS).map(|index| {
            TimelineItem::SolidColor(SolidColorItem {
                id: format!("visual-{index}"),
                color: "#000000".into(),
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })
        }));
        input.tracks = vec![track("base", TrackType::Video, items)];

        assert_eq!(
            evaluate_project(&input, 16, 16, 1).unwrap_err().code,
            ErrorCode::AssetNotFound
        );
    }

    #[test]
    fn preflights_voiceover_keyframes_before_interval_derivation() {
        let mut input = project();
        input.assets = vec![asset("voice", MediaType::Audio, true)];
        let mut voice_item = media("voice-item", "voice", 0);
        {
            let TimelineItem::Media(media) = &mut voice_item else {
                unreachable!()
            };
            media.duration_ms = MAX_EVALUATED_KEYFRAMES_PER_CHANNEL as u64 + 2;
            media.keyframes = (0..=MAX_EVALUATED_KEYFRAMES_PER_CHANNEL)
                .map(|index| Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: index as u64,
                    value: KeyframeValue::Scalar { value: 1.0 },
                    easing: Easing::Linear,
                })
                .collect();
        }
        let mut voice = track("voice", TrackType::Audio, vec![voice_item]);
        voice.audio_role = AudioTrackRole::Voiceover;
        input.tracks = vec![voice];

        assert_eq!(
            evaluate_project(&input, 16, 16, 1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn bounds_pre_merge_voiceover_activity_ranges() {
        fn voiceover_item(id: &str, asset_id: &str, start_ms: u64) -> TimelineItem {
            let mut item = media(id, asset_id, start_ms);
            let TimelineItem::Media(media) = &mut item else {
                unreachable!()
            };
            media.duration_ms = MAX_EVALUATED_KEYFRAMES_PER_CHANNEL as u64;
            media.keyframes = (0..MAX_EVALUATED_KEYFRAMES_PER_CHANNEL)
                .map(|index| Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: index as u64,
                    value: KeyframeValue::Scalar {
                        value: if index % 2 == 0 { 1.0 } else { 0.0 },
                    },
                    easing: Easing::Hold,
                })
                .collect();
            item
        }

        let mut exact = project();
        let mut voice_asset = asset("voice", MediaType::Audio, true);
        voice_asset.duration_ms = None;
        exact.assets = vec![voice_asset];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![
                voiceover_item("voice-a", "voice", 0),
                voiceover_item("voice-b", "voice", 20_000),
            ],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        exact.tracks = vec![voice];

        let expanded = evaluate_project(&with_empty_instance(exact.clone()), 16, 16, 1).unwrap();
        assert_eq!(
            expanded.scene.instance_voiceover_intervals.unwrap().len(),
            MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES
        );
        let evaluated = evaluate_project(&exact, 16, 16, 1).unwrap();
        assert_eq!(
            evaluated.scene.voiceover_intervals.len(),
            MAX_EVALUATED_VOICEOVER_ACTIVITY_RANGES
        );
        assert_eq!(
            evaluated.scene.voiceover_intervals[0],
            EvaluatedTimeSpan {
                start_ms: 0,
                end_ms: 1,
            }
        );
        assert_eq!(
            evaluated.scene.voiceover_intervals[5_000],
            EvaluatedTimeSpan {
                start_ms: 20_000,
                end_ms: 20_001,
            }
        );

        let mut repeated = with_empty_instance(exact.clone());
        let mut local = repeated.tracks.remove(0);
        if let TimelineItem::Media(m) = &mut local.items[1] {
            m.start_ms = 0;
        }
        repeated.components[0].tracks = vec![local];
        repeated.components[0].duration_ms = 10_000;
        if let TimelineItem::ComponentInstance(i) = &mut repeated.tracks[0].items[0] {
            i.duration_ms = 10_000;
        }
        let mut copy = repeated.tracks[0].items[0].clone();
        if let TimelineItem::ComponentInstance(i) = &mut copy {
            i.id = "repeated".into();
            i.visual_properties.stack_order = 1;
        }
        repeated.tracks[0].items.push(copy);
        assert!(
            evaluate_project(&repeated, 16, 16, 1)
                .unwrap_err()
                .message
                .contains("voiceover interval limit")
        );

        let mut overflow = exact;
        overflow.tracks[0]
            .items
            .push(media("voice-c", "voice", 40_000));
        assert_eq!(
            evaluate_project(&with_empty_instance(overflow), 16, 16, 1)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn multiple_ducked_layers_use_the_scene_voiceover_table() {
        let mut input = project();
        input.assets = vec![
            asset("voice", MediaType::Audio, true),
            asset("music", MediaType::Audio, true),
        ];
        let mut voice = track(
            "voice",
            TrackType::Audio,
            vec![media("voice-item", "voice", 100)],
        );
        voice.audio_role = AudioTrackRole::Voiceover;
        let music_track = |id: &str, item_id: &str, gain: f64| {
            let mut music = track(id, TrackType::Audio, vec![media(item_id, "music", 0)]);
            music.audio_role = AudioTrackRole::Music;
            music.ducking = Some(DuckingSettings {
                enabled: true,
                gain,
                attack_ms: 25,
                release_ms: 50,
            });
            music
        };
        input.tracks = vec![
            voice,
            music_track("music-a", "music-a-item", 0.2),
            music_track("music-b", "music-b-item", 0.3),
        ];

        let scene = evaluate_project(&input, 16, 16, 1).unwrap().scene;
        assert_eq!(
            scene.voiceover_intervals,
            vec![EvaluatedTimeSpan {
                start_ms: 100,
                end_ms: 1_100,
            }]
        );
        let ducking = scene.audio_layers[1..]
            .iter()
            .map(|layer| {
                let ducking = layer.ducking.as_ref().unwrap();
                (ducking.gain, ducking.attack_ms, ducking.release_ms)
            })
            .collect::<Vec<_>>();
        assert_eq!(ducking, vec![(0.2, 25, 50), (0.3, 25, 50)]);
    }

    #[test]
    fn transition_facts_are_bounded_ordered_and_include_both_self_roles() {
        let transition_project = |count: usize| {
            let mut input = project();
            let mut items = vec![TimelineItem::Rectangle(RectangleItem {
                id: "visual".into(),
                color: "#ffffff".into(),
                width: 1,
                height: 1,
                start_ms: 0,
                duration_ms: count as u64 + 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })];
            items.extend((0..count).map(|index| {
                TimelineItem::Transition(TransitionItem {
                    id: format!("transition-{index}"),
                    transition_type: if index % 2 == 0 {
                        TransitionType::Fade
                    } else {
                        TransitionType::Crossfade
                    },
                    from_item_id: "visual".into(),
                    to_item_id: None,
                    start_ms: index as u64,
                    duration_ms: 1,
                    visual_properties: crate::VisualProperties::default(),
                })
            }));
            input.tracks = vec![track("base", TrackType::Video, items)];
            input
        };

        let exact = evaluate_project(
            &transition_project(MAX_EVALUATED_TRANSITION_FACTS),
            16,
            16,
            1,
        )
        .unwrap();
        let facts = &exact.scene.visual_layers[0].transitions;
        assert_eq!(facts.len(), MAX_EVALUATED_TRANSITION_FACTS);
        assert_eq!(facts[0].kind, EvaluatedTransitionKind::Fade);
        assert_eq!(facts[1].kind, EvaluatedTransitionKind::Crossfade);
        assert_eq!(facts[0].span.start_ms, 0);
        assert_eq!(facts[1].span.start_ms, 1);
        assert_eq!(
            evaluate_project(
                &transition_project(MAX_EVALUATED_TRANSITION_FACTS + 1),
                16,
                16,
                1,
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );

        let mut self_endpoint = transition_project(0);
        self_endpoint.tracks[0]
            .items
            .push(TimelineItem::Transition(TransitionItem {
                id: "self".into(),
                transition_type: TransitionType::Crossfade,
                from_item_id: "visual".into(),
                to_item_id: Some("visual".into()),
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
            }));
        self_endpoint.tracks[0].items[1]
            .visual_properties_mut()
            .stack_order = 1;
        let evaluated = evaluate_project(&self_endpoint, 16, 16, 1).unwrap();
        assert_eq!(
            evaluated.scene.visual_layers[0]
                .transitions
                .iter()
                .map(|fact| fact.role)
                .collect::<Vec<_>>(),
            vec![EvaluatedTransitionRole::Out, EvaluatedTransitionRole::In,]
        );
    }

    #[test]
    fn font_bindings_preserve_selection_outside_the_scene() {
        let text = |id: &str, path: Option<&str>, family: Option<&str>| {
            TimelineItem::Text(TextItem {
                id: id.into(),
                text: id.into(),
                start_ms: 0,
                duration_ms: 1,
                font_size: 20,
                color: "#ffffff".into(),
                font_family: family.map(str::to_owned),
                font_path: path.map(str::to_owned),
                style: TextStyle::default(),
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })
        };
        let mut input = project();
        input.tracks = vec![track(
            "text",
            TrackType::Overlay,
            vec![
                text("path", Some("fonts/path.ttf"), None),
                text("family", None, Some("Inter")),
                text("both", Some("fonts/both.ttf"), Some("Source Sans")),
                text("default", None, None),
            ],
        )];

        let evaluated = evaluate_project(&input, 16, 16, 1).unwrap();
        let font_ids = evaluated
            .scene
            .visual_layers
            .iter()
            .map(|layer| match &layer.source {
                EvaluatedVisualSource::Text(text) => text.font_resource_id.as_deref(),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            font_ids,
            vec![
                Some("text-font:path"),
                Some("text-font:family"),
                Some("text-font:both"),
                None,
            ]
        );
        assert_eq!(
            evaluated.resource_bindings.fonts,
            vec![
                FontResourceBinding {
                    font_resource_id: "text-font:path".into(),
                    requested_path: Some("fonts/path.ttf".into()),
                    requested_family: None,
                },
                FontResourceBinding {
                    font_resource_id: "text-font:family".into(),
                    requested_path: None,
                    requested_family: Some("Inter".into()),
                },
                FontResourceBinding {
                    font_resource_id: "text-font:both".into(),
                    requested_path: Some("fonts/both.ttf".into()),
                    requested_family: Some("Source Sans".into()),
                },
            ]
        );
        let scene_debug = format!("{:?}", evaluated.scene);
        assert!(!scene_debug.contains("fonts/path.ttf"));
        assert!(!scene_debug.contains("fonts/both.ttf"));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedAncestors {
    pub(crate) matrix: [f64; 6],
    pub(crate) inverse: [f64; 6],
    pub(crate) opacity: f64,
    pub(crate) clip: EvaluatedTimeSpan,
}

impl EvaluatedVisualLayer {
    pub(crate) fn requires_affine(&self) -> bool {
        self.transform2d.is_some()
            || self.ancestors.is_some()
            || matches!(self.source, EvaluatedVisualSource::Shape(_))
    }
    pub(crate) fn has_animated_geometry(&self) -> bool {
        (self.ancestors.is_some() || matches!(self.source, EvaluatedVisualSource::Shape(_)))
            && self.transform2d.is_none()
            && self.keyframes.iter().any(|key| {
                matches!(
                    key.property,
                    EvaluatedProperty::Position | EvaluatedProperty::Scale
                )
            })
    }
    pub(crate) fn legacy_anchor(&self, source: (u32, u32)) -> (f64, f64) {
        if let EvaluatedVisualSource::Shape(shape) = &self.source {
            return (-shape.origin.0, -shape.origin.1);
        }
        let EvaluatedVisualSource::Text(text) = &self.source else {
            return (0.0, 0.0);
        };
        use EvaluatedAnchorPoint::*;
        let x = match text.style.anchor {
            TopCenter | Center | BottomCenter => 0.5,
            TopRight | CenterRight | BottomRight => 1.0,
            _ => 0.0,
        };
        let y = match text.style.anchor {
            CenterLeft | Center | CenterRight => 0.5,
            BottomLeft | BottomCenter | BottomRight => 1.0,
            _ => 0.0,
        };
        (x * f64::from(source.0), y * f64::from(source.1))
    }
    pub(crate) fn visible_span(&self) -> EvaluatedTimeSpan {
        self.ancestors.map_or(self.span, |parent| parent.clip)
    }
}

const IDENTITY_MATRIX: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn multiply_matrix(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
    let [a, b, c, d, x, y] = left;
    let [e, f, g, h, u, v] = right;
    [
        a * e + c * f,
        b * e + d * f,
        a * g + c * h,
        b * g + d * h,
        a * u + c * v + x,
        b * u + d * v + y,
    ]
}

fn apply_ancestors(
    project: &Project,
    layers: &mut Vec<EvaluatedVisualLayer>,
    canvas: (u32, u32),
    retained: bool,
) -> Result<(), CoreError> {
    let index: HashMap<_, _> = project
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .items
                .iter()
                .map(move |item| (item.id(), (track.hidden, item)))
        })
        .collect();
    for layer in layers.iter_mut() {
        let mut node = index[layer.item_id.as_str()].1;
        if node.visual_properties().parent.is_none() {
            continue;
        }
        let mut ancestors = EvaluatedAncestors {
            matrix: IDENTITY_MATRIX,
            inverse: IDENTITY_MATRIX,
            opacity: 1.0,
            clip: layer.span,
        };
        while let Some(parent) = &node.visual_properties().parent {
            let (track_hidden, target) = index[parent.id.as_str()];
            let transform = target.visual_properties().transform2d.unwrap_or_default();
            let (matrix, inverse) = transform_matrices(transform, canvas, canvas)?;
            ancestors.matrix = multiply_matrix(matrix, ancestors.matrix);
            ancestors.inverse = multiply_matrix(ancestors.inverse, inverse);
            ancestors.opacity *= transform.opacity;
            if ancestors
                .matrix
                .iter()
                .chain(&ancestors.inverse)
                .any(|v| !v.is_finite())
            {
                return Err(invalid("non-finite ancestor matrix"));
            }
            ancestors.clip.start_ms = ancestors.clip.start_ms.max(target.start_ms());
            ancestors.clip.end_ms = ancestors.clip.end_ms.min(target.end_ms());
            if track_hidden || target.hidden() {
                ancestors.clip.end_ms = ancestors.clip.start_ms;
            }
            node = target;
        }
        layer.ancestors = Some(ancestors);
    }
    layers.retain(|layer| {
        let span = layer.visible_span();
        retained || span.start_ms < span.end_ms
    });
    Ok(())
}

/// One canonical source-to-composition affine map and its outward-rounded raster bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EvaluatedAffine {
    pub(crate) matrix: [f64; 6],
    pub(crate) inverse: [f64; 6],
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) opacity: f64,
}

fn transform_matrices(
    transform: crate::Transform2D,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<([f64; 6], [f64; 6]), CoreError> {
    transform.validate()?;
    if source.0 == 0 || source.1 == 0 {
        return Err(invalid("Transform2D source dimensions must be positive"));
    }
    let (sin, cos) = transform.rotation_deg.to_radians().sin_cos();
    let kx = transform.skew_x_deg.to_radians().tan();
    let ky = transform.skew_y_deg.to_radians().tan();
    let a = (cos - sin * ky) * transform.scale_x;
    let b = (sin + cos * ky) * transform.scale_x;
    let c = (cos * kx - sin * (1.0 + ky * kx)) * transform.scale_y;
    let d = (sin * kx + cos * (1.0 + ky * kx)) * transform.scale_y;
    let factor = match transform.position.unit {
        crate::PositionUnit::Pixels => (1.0, 1.0),
        crate::PositionUnit::Normalized => (f64::from(canvas.0), f64::from(canvas.1)),
    };
    let ax = transform.anchor.x * f64::from(source.0);
    let ay = transform.anchor.y * f64::from(source.1);
    let tx = transform.position.x * factor.0 - a * ax - c * ay;
    let ty = transform.position.y * factor.1 - b * ax - d * ay;
    let matrix = [a, b, c, d, tx, ty];
    let det = transform.scale_x * transform.scale_y;
    let inverse = [
        d / det,
        -b / det,
        -c / det,
        a / det,
        (c * ty - d * tx) / det,
        (b * tx - a * ty) / det,
    ];
    if matrix.iter().chain(&inverse).any(|v| !v.is_finite()) {
        return Err(invalid("Transform2D derived matrix must be finite"));
    }
    Ok((matrix, inverse))
}

pub(crate) fn evaluate_affine(
    transform: crate::Transform2D,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    let (matrix, inverse) = transform_matrices(transform, source, canvas)?;
    affine_from_matrices(matrix, inverse, source, transform.opacity)
}

fn affine_from_matrices(
    matrix: [f64; 6],
    inverse: [f64; 6],
    source: (u32, u32),
    opacity: f64,
) -> Result<EvaluatedAffine, CoreError> {
    let [a, b, c, d, tx, ty] = matrix;
    let corners = [
        (0.0, 0.0),
        (f64::from(source.0), 0.0),
        (0.0, f64::from(source.1)),
        (f64::from(source.0), f64::from(source.1)),
    ];
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for (x, y) in corners {
        let px = a * x + c * y + tx;
        let py = b * x + d * y + ty;
        if !px.is_finite() || !py.is_finite() {
            return Err(invalid("Transform2D derived coordinate must be finite"));
        }
        left = left.min(px);
        right = right.max(px);
        top = top.min(py);
        bottom = bottom.max(py);
    }
    // Snap trigonometric roundoff at integer edges, preserving exact right-angle bounds.
    let snap = |v: f64| {
        if (v - v.round()).abs() < 1e-9 {
            v.round()
        } else {
            v
        }
    };
    left = snap(left).floor();
    top = snap(top).floor();
    let w = (snap(right).ceil() - left).max(1.0);
    let h = (snap(bottom).ceil() - top).max(1.0);
    if w > 16_384.0 || h > 16_384.0 || w * h > 16_777_216.0 {
        return Err(invalid(
            "Transform2D transformed raster bounds exceed complexity limits",
        ));
    }
    Ok(EvaluatedAffine {
        matrix,
        inverse,
        left,
        top,
        width: w as u32,
        height: h as u32,
        opacity,
    })
}

pub(crate) fn evaluate_layer_affine(
    layer: &EvaluatedVisualLayer,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    measure_layer_affine(layer, source, canvas, layer.ancestors)
}

fn measure_layer_affine(
    layer: &EvaluatedVisualLayer,
    source: (u32, u32),
    canvas: (u32, u32),
    ancestors: Option<EvaluatedAncestors>,
) -> Result<EvaluatedAffine, CoreError> {
    if let EvaluatedVisualSource::Shape(shape) = &layer.source {
        return shapes::affine(layer, shape, canvas);
    }
    let canvas = layer.instance.map_or(canvas, |instance| instance.canvas);
    if ancestors.is_none() {
        return evaluate_affine(
            layer
                .transform2d
                .ok_or_else(|| invalid("missing local affine transform"))?,
            source,
            canvas,
        );
    }
    let parent = ancestors.unwrap();
    let (local, inverse, opacity) = if let Some(transform) = layer.transform2d {
        let (matrix, inverse) = transform_matrices(transform, source, canvas)?;
        (matrix, inverse, transform.opacity)
    } else {
        let mut x = layer.transform.position_x;
        let mut y = layer.transform.position_y;
        if let EvaluatedVisualSource::Caption(caption) = &layer.source {
            x = (f64::from(canvas.0) - f64::from(source.0)) / 2.0;
            y = f64::from(canvas.1) - f64::from(source.1) - f64::from(caption.bottom_margin_px)
                + 12.0;
        }
        let scale = layer.transform.scale;
        let (ax, ay) = layer.legacy_anchor(source);
        x -= scale * ax;
        y -= scale * ay;
        (
            [scale, 0.0, 0.0, scale, x, y],
            [1.0 / scale, 0.0, 0.0, 1.0 / scale, -x / scale, -y / scale],
            layer.transform.opacity,
        )
    };
    let matrix = multiply_matrix(parent.matrix, local);
    let inverse = multiply_matrix(inverse, parent.inverse);
    if matrix.iter().chain(&inverse).any(|v| !v.is_finite()) {
        return Err(invalid("non-finite composed matrix"));
    }
    let mut affine = affine_from_matrices(matrix, inverse, source, opacity * parent.opacity)?;
    if layer.has_animated_geometry() {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut scales = Vec::new();
        for key in &layer.keyframes {
            match (key.property, key.value) {
                (EvaluatedProperty::Position, EvaluatedKeyframeValue::Position { x, y }) => {
                    xs.push(x);
                    ys.push(y);
                }
                (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) => {
                    scales.push(value)
                }
                _ => {}
            }
        }
        let extremes = |values: &[f64], default: f64| {
            if values.is_empty() {
                [default, default]
            } else {
                [
                    values.iter().copied().fold(f64::INFINITY, f64::min),
                    values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                ]
            }
        };
        let mut left = f64::INFINITY;
        let mut top = f64::INFINITY;
        let mut right = f64::NEG_INFINITY;
        let mut bottom = f64::NEG_INFINITY;
        // Existing easings are bounded by endpoint values. This fixed-size envelope
        // bounds all combinations without expanding animation into per-frame facts.
        for x in extremes(&xs, layer.transform.position_x) {
            for y in extremes(&ys, layer.transform.position_y) {
                for scale in extremes(&scales, layer.transform.scale) {
                    if !scale.is_finite() || scale <= 0.0 {
                        return Err(invalid("invalid animated scale"));
                    }
                    let (ax, ay) = layer.legacy_anchor(source);
                    let x = x - scale * ax;
                    let y = y - scale * ay;
                    let matrix = multiply_matrix(parent.matrix, [scale, 0.0, 0.0, scale, x, y]);
                    let inverse = multiply_matrix(
                        [1.0 / scale, 0.0, 0.0, 1.0 / scale, -x / scale, -y / scale],
                        parent.inverse,
                    );
                    let bounds =
                        affine_from_matrices(matrix, inverse, source, opacity * parent.opacity)?;
                    left = left.min(bounds.left);
                    top = top.min(bounds.top);
                    right = right.max(bounds.left + f64::from(bounds.width));
                    bottom = bottom.max(bounds.top + f64::from(bounds.height));
                }
            }
        }
        // Geometry was validated above before clipping. Travel only determines
        // which composition pixels may need sampling, never the object size limit.
        left = left.max(0.0).min(f64::from(canvas.0));
        top = top.max(0.0).min(f64::from(canvas.1));
        right = right.max(0.0).min(f64::from(canvas.0));
        bottom = bottom.max(0.0).min(f64::from(canvas.1));
        affine.left = left;
        affine.top = top;
        affine.width = (right - left).max(0.0) as u32;
        affine.height = (bottom - top).max(0.0) as u32;
    }
    Ok(affine)
}

pub(crate) fn finalize_affine_geometry(
    scene: &mut EvaluatedScene,
    measurements: &HashMap<String, (u32, u32)>,
) -> Result<(), CoreError> {
    shapes::refine_scene(scene)?;
    // Collect first so a bad measurement cannot partially finalize the scene.
    let resolved = scene
        .visual_layers
        .iter()
        .map(|layer| {
            if !layer.requires_affine() {
                return Ok(None);
            }
            let size = layer
                .source_size
                .or_else(|| measurements.get(&layer.item_id).copied())
                .ok_or_else(|| invalid("Transform2D source measurement is missing"))?;
            Ok(Some((
                size,
                evaluate_layer_affine(layer, size, (scene.canvas.width, scene.canvas.height))?,
            )))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;
    for (layer, result) in scene.visual_layers.iter_mut().zip(resolved) {
        if let Some((size, affine)) = result {
            layer.source_size = Some(size);
            layer.affine = Some(affine);
            if layer.has_animated_geometry() {
                let mut tiles = Vec::new();
                for y in (0..affine.height).step_by(4096) {
                    for x in (0..affine.width).step_by(4096) {
                        tiles.push(EvaluatedAffine {
                            left: affine.left + f64::from(x),
                            top: affine.top + f64::from(y),
                            width: (affine.width - x).min(4096),
                            height: (affine.height - y).min(4096),
                            ..affine
                        });
                    }
                }
                layer.sampling_tiles = Some(tiles);
            }
        }
    }
    scene
        .visual_layers
        .retain(|layer| !layer.sampling_tiles.as_ref().is_some_and(Vec::is_empty));
    Ok(())
}

#[cfg(test)]
mod affine_tests {
    use super::*;
    #[test]
    fn independent_sequential_oracle_and_units() {
        let mut t = crate::Transform2D {
            anchor: crate::TransformAnchor { x: 0.25, y: 0.75 },
            scale_x: 1.7,
            scale_y: 0.6,
            skew_x_deg: 17.,
            skew_y_deg: -11.,
            rotation_deg: 33.,
            position: crate::TransformPosition {
                x: 70.,
                y: 90.,
                unit: crate::PositionUnit::Pixels,
            },
            ..Default::default()
        };
        let affine = evaluate_affine(t, (53, 29), (200, 120)).unwrap();
        for (x, y) in [(0., 0.), (53., 0.), (0., 29.), (53., 29.), (12.3, 17.1)] {
            let sx = (x - 0.25 * 53.) * 1.7;
            let sy = (y - 0.75 * 29.) * 0.6;
            let kx = sx + 17_f64.to_radians().tan() * sy;
            let ky = sy + (-11_f64).to_radians().tan() * kx;
            let (sin, cos) = 33_f64.to_radians().sin_cos();
            let expected = (kx * cos - ky * sin + 70., kx * sin + ky * cos + 90.);
            let [a, b, c, d, tx, ty] = affine.matrix;
            assert!((a * x + c * y + tx - expected.0).abs() < 1e-9);
            assert!((b * x + d * y + ty - expected.1).abs() < 1e-9);
            let [a, b, c, d, tx, ty] = affine.inverse;
            assert!((a * expected.0 + c * expected.1 + tx - x).abs() < 1e-9);
            assert!((b * expected.0 + d * expected.1 + ty - y).abs() < 1e-9);
        }
        t.position.unit = crate::PositionUnit::Normalized;
        t.position.x = 70. / 200.;
        t.position.y = 90. / 120.;
        assert_eq!(evaluate_affine(t, (53, 29), (200, 120)).unwrap(), affine);
    }
    #[test]
    fn geometry_boundaries_are_checked_before_clipping() {
        let t = crate::Transform2D::default();
        for size in [(16384, 1), (4096, 4096)] {
            assert!(evaluate_affine(t, size, (1, 1)).is_ok());
        }
        for size in [(16385, 1), (4096, 4097), (0, 1)] {
            assert_eq!(
                evaluate_affine(t, size, (1, 1)).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        let mut t = t;
        t.rotation_deg = 90.;
        let a = evaluate_affine(t, (100, 20), (200, 200)).unwrap();
        assert_eq!((a.width, a.height, a.left, a.top), (20, 100, -20., 0.));
    }
}

#[cfg(test)]
mod instance_tests {
    use super::*;
    use serde_json::json;

    fn project() -> Project {
        let shape = json!({"type":"rectangle","id":"same","startMs":0,"durationMs":500,"width":10,"height":20,"color":"#ff0000","transform":{"positionX":3,"positionY":0,"scale":1,"opacity":1},"keyframes":[]});
        let track =
            |items| json!({"id":"local","name":"Local","trackType":"overlay","items":items});
        let inner = json!({"type":"component_instance","id":"same","componentId":"leaf","startMs":20,"trimStartMs":10,"durationMs":700,"timeScale":0.5,"slotValues":{},"transform":{"positionX":10,"positionY":0,"scale":1,"opacity":0.5}});
        let root = json!({"type":"component_instance","id":"same","componentId":"outer","startMs":100,"trimStartMs":50,"durationMs":400,"timeScale":1.5,"slotValues":{},"zIndex":0,"stackOrder":0,"transform":{"positionX":20,"positionY":0,"scale":1,"opacity":0.5}});
        serde_json::from_value(json!({"schemaVersion":13,"id":"project","revision":0,"name":"Instances","createdAtMs":1,"updatedAtMs":1,"settings":{"width":100,"height":100,"fps":30},"assets":[],"tracks":[track(json!([root]))],"components":[{"id":"leaf","name":"Leaf","width":100,"height":100,"durationMs":500,"slots":[],"tracks":[track(json!([shape]))]},{"id":"outer","name":"Outer","width":100,"height":100,"durationMs":1000,"slots":[],"tracks":[track(json!([inner]))]}]})).unwrap()
    }

    #[test]
    fn svg_component_sampling_precision() {
        let mut p = project();
        p.schema_version = 15;
        let document = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 .1 .1\"><path d=\"M8388.60825 0 L8388.73625 0 L8388.73625 .01 Z\"/></svg>").unwrap();
        p.components[0].tracks[0].items = vec![serde_json::from_value(json!({"type":"svg","id":"svg","document":document,"startMs":0,"durationMs":500,"keyframes":[]})).unwrap()];
        assert!(evaluate_project(&p, 100, 100, 30).is_ok());
        p.tracks[0].items[0].visual_properties_mut().transform.scale = 2.;
        let error = evaluate_project(&p, 100, 100, 30).unwrap_err();
        assert!(error.message.contains("coordinate conversion precision"));
        p.tracks[0].hidden = true;
        assert!(evaluate_project(&p, 100, 100, 30).is_err());
    }
    #[test]
    fn svg_component_scale_cancellation() {
        let mut p = project();
        p.schema_version = 15;
        let document = crate::validation::svg::parse("<svg width=\"100\" height=\"100\"><rect width=\"100\" height=\"100\" fill=\"#f00\"/></svg>").unwrap();
        p.components[0].tracks[0].items = vec![serde_json::from_value(json!({"type":"svg","id":"svg","document":document,"startMs":0,"durationMs":500,"keyframes":[],"transform":{"positionX":0,"positionY":0,"scale":100,"opacity":1}})).unwrap()];
        p.tracks[0].items[0].visual_properties_mut().transform.scale = 0.01;
        let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
        let EvaluatedVisualSource::Shape(shape) = &scene.visual_layers[0].source else {
            panic!("SVG missing")
        };
        assert_eq!(shape.size, (100, 100));
        let mut identity = p.clone();
        identity.components[0].tracks[0].items[0]
            .visual_properties_mut()
            .transform
            .scale = 1.;
        identity.tracks[0].items[0]
            .visual_properties_mut()
            .transform
            .scale = 1.;
        let identity = evaluate_project(&identity, 100, 100, 30).unwrap().scene;
        let EvaluatedVisualSource::Shape(expected) = &identity.visual_layers[0].source else {
            panic!()
        };
        assert_eq!(shape, expected);
        let mut second = p.tracks[0].items[0].clone();
        if let TimelineItem::ComponentInstance(i) = &mut second {
            i.id = "second".into();
        }
        second.visual_properties_mut().stack_order = 1;
        second.visual_properties_mut().transform.scale = 0.02;
        p.tracks[0].items.push(second);
        let two = evaluate_project(&p, 100, 100, 30).unwrap().scene;
        let EvaluatedVisualSource::Shape(second) = &two.visual_layers[1].source else {
            panic!()
        };
        assert_eq!(second.size, (200, 200));
        p.tracks[0].items.pop();
        // Unreachable definitions retain their own identity-root contract.
        let mut unused = p.clone();
        unused.tracks.clear();
        assert!(evaluate_project(&unused, 100, 100, 30).is_err());
        let legacy = p.tracks[0].items[0].visual_properties().transform.clone();
        p.tracks[0].items[0].visual_properties_mut().transform = Transform::default();
        p.tracks[0].items[0].visual_properties_mut().transform2d = Some(crate::Transform2D {
            scale_x: 0.01,
            scale_y: 0.01,
            ..Default::default()
        });
        assert!(evaluate_project(&p, 100, 100, 30).is_ok());
        p.tracks[0].items[0].visual_properties_mut().transform2d = None;
        p.tracks[0].items[0].visual_properties_mut().transform = legacy;
        let local = &mut p.components[0].tracks[0].items[0];
        local.visual_properties_mut().transform.scale = 10.;
        local.visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "component:leaf".into(),
            id: "group".into(),
        });
        let group = serde_json::from_value(json!({"type":"group","id":"group","startMs":0,"durationMs":500,"stackOrder":1,"transform2d":crate::Transform2D { scale_x: 10., scale_y: 10., ..Default::default() }})).unwrap();
        p.components[0].tracks[0].items.push(group);
        assert!(evaluate_project(&p, 100, 100, 30).is_ok());
        p.tracks[0].hidden = true;
        assert!(evaluate_project(&p, 100, 100, 30).is_ok());
        p.tracks[0].items[0].visual_properties_mut().transform.scale = 1.;
        assert!(evaluate_project(&p, 100, 100, 30).is_err());
    }
    #[test]
    fn nested_fractional_clock_affine_and_repeated_identity() {
        let mut project = project();
        let before = serde_json::to_value(&project).unwrap();
        let mut evaluated = evaluate_project(&project, 100, 100, 30).unwrap();
        assert_eq!(evaluated.scene.visual_layers.len(), 1);
        let layer = &evaluated.scene.visual_layers[0];
        let clock = layer.instance.unwrap();
        assert_eq!(clock.rate, 0.75);
        assert_eq!(clock.offset, -50.0);
        assert_eq!((clock.start_ms, clock.end_ms), (100.0, 500.0));
        assert!((clock.root_ms(26) - 101.33333333333333).abs() < 1e-10);
        finalize_affine_geometry(&mut evaluated.scene, &HashMap::new()).unwrap();
        let affine = evaluated.scene.visual_layers[0].affine.unwrap();
        assert_eq!(affine.matrix, [1.0, 0.0, 0.0, 1.0, 33.0, 0.0]);
        assert_eq!(affine.opacity, 0.25);
        assert_eq!(serde_json::to_value(&project).unwrap(), before);
        let mut repeated = project.tracks[0].items[0].clone();
        if let TimelineItem::ComponentInstance(instance) = &mut repeated {
            instance.id = "second".into();
            instance.visual_properties.stack_order = 1;
        }
        project.tracks[0].items.push(repeated);
        let scene = evaluate_project(&project, 100, 100, 30).unwrap();
        assert_eq!(scene.scene.visual_layers.len(), 2);
        assert_ne!(
            scene.scene.visual_layers[0].item_id,
            scene.scene.visual_layers[1].item_id
        );
        assert_eq!(scene, evaluate_project(&project, 100, 100, 30).unwrap());
    }

    #[test]
    fn component_dimensions_affine_oracle_and_contiguous_order() {
        let mut p = project();
        p.components[0].width = 200;
        p.components[0].height = 80;
        p.components[1].width = 320;
        p.components[1].height = 180;
        let nested = crate::Transform2D {
            position: crate::TransformPosition {
                x: 0.25,
                y: 0.5,
                unit: crate::PositionUnit::Normalized,
            },
            anchor: crate::TransformAnchor { x: 0.2, y: 0.1 },
            rotation_deg: 30.0,
            skew_x_deg: 10.0,
            scale_x: 1.2,
            scale_y: 0.8,
            opacity: 0.5,
            ..Default::default()
        };
        let outer = crate::Transform2D {
            position: crate::TransformPosition {
                x: 0.5,
                y: 0.25,
                unit: crate::PositionUnit::Normalized,
            },
            anchor: crate::TransformAnchor { x: 0.2, y: 0.7 },
            rotation_deg: -15.0,
            skew_y_deg: 5.0,
            scale_x: 0.9,
            scale_y: 1.1,
            opacity: 0.4,
            ..Default::default()
        };
        for (item, t) in [
            (&mut p.components[1].tracks[0].items[0], nested),
            (&mut p.tracks[0].items[0], outer),
        ] {
            item.visual_properties_mut().transform = Transform::default();
            item.visual_properties_mut().transform2d = Some(t);
        }
        p.components[0].tracks[0].items[0]
            .visual_properties_mut()
            .z_index = 1000;
        p.tracks[0].items.push(serde_json::from_value(json!({"type":"rectangle","id":"sibling","startMs":0,"durationMs":1000,"width":1,"height":1,"color":"#0000ff","keyframes":[],"zIndex":1,"stackOrder":1})).unwrap());
        let mut scene = evaluate_project(&p, 640, 360, 30).unwrap().scene;
        finalize_affine_geometry(&mut scene, &HashMap::new()).unwrap();
        assert!(
            matches!(&scene.visual_layers[1].source,EvaluatedVisualSource::Rectangle{color,..} if color=="#0000ff")
        );
        let map =
            |t: crate::Transform2D, source: (f64, f64), parent: (f64, f64), (x, y): (f64, f64)| {
                let x = (x - t.anchor.x * source.0) * t.scale_x;
                let y = (y - t.anchor.y * source.1) * t.scale_y;
                let x = x + y * t.skew_x_deg.to_radians().tan();
                let y = y + x * t.skew_y_deg.to_radians().tan();
                let (sin, cos) = t.rotation_deg.to_radians().sin_cos();
                (
                    cos * x - sin * y + t.position.x * parent.0,
                    sin * x + cos * y + t.position.y * parent.1,
                )
            };
        let affine = scene.visual_layers[0].affine.unwrap();
        for (x, y) in [(0.0, 0.0), (10.0, 0.0), (0.0, 20.0), (10.0, 20.0)] {
            let expected = map(
                outer,
                (320.0, 180.0),
                (640.0, 360.0),
                map(nested, (200.0, 80.0), (320.0, 180.0), (x + 3.0, y)),
            );
            let [a, b, c, d, tx, ty] = affine.matrix;
            assert!((a * x + c * y + tx - expected.0).abs() < 1e-9);
            assert!((b * x + d * y + ty - expected.1).abs() < 1e-9);
        }
        assert!((affine.opacity - 0.2).abs() < 1e-12);
    }

    #[test]
    fn audio_expansion_limits_and_instance_vs_group_visibility() {
        let mut p = project();
        p.assets=serde_json::from_value(json!([{"id":"sound","mediaType":"audio","fileName":"sound.wav","projectRelativePath":"assets/sound.wav","durationMs":1000,"hasAudio":true}])).unwrap();
        p.components[0].tracks=serde_json::from_value(json!([{"id":"audio","name":"Audio","trackType":"audio","items":[{"type":"media","id":"sound","assetId":"sound","startMs":0,"durationMs":500,"sourceInMs":125,"audio":{"volume":1,"muted":false,"fadeInMs":10,"fadeOutMs":20},"keyframes":[]}]}])).unwrap();
        let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.audio_layers.len(), 1);
        assert_eq!(scene.audio_layers[0].source_in_ms, 125);
        assert_eq!(scene.audio_layers[0].instance.unwrap().rate, 0.75);
        let mut group:TimelineItem=serde_json::from_value(json!({"type":"group","id":"group","startMs":0,"durationMs":1000,"hidden":true,"stackOrder":1,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0,"y":0},"opacity":1}})).unwrap();
        group.visual_properties_mut().stack_order = 1;
        p.tracks[0].items[0].visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "group".into(),
        });
        p.tracks[0].items.push(group);
        assert_eq!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .audio_layers
                .len(),
            1
        );
        p.tracks[0].items[0].visual_properties_mut().hidden = true;
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .audio_layers
                .is_empty()
        );
        p.tracks[0].items.pop();
        p.tracks[0].items[0].visual_properties_mut().hidden = false;
        p.tracks[0].items[0].visual_properties_mut().parent = None;
        p.components[0].tracks[0].muted = true;
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .audio_layers
                .is_empty()
        );
        p.components[0].tracks[0].muted = false;
        let leaf = p.components[0].tracks[0].items[0].clone();
        p.components[0].tracks[0].items = (0..64)
            .map(|i| {
                let mut v = leaf.clone();
                if let TimelineItem::Media(m) = &mut v {
                    m.id = format!("m{i}");
                    m.visual_properties.stack_order = i;
                }
                v
            })
            .collect();
        let instance = p.tracks[0].items[0].clone();
        p.tracks[0].items = (0..64)
            .map(|i| {
                let mut v = instance.clone();
                if let TimelineItem::ComponentInstance(m) = &mut v {
                    m.id = format!("i{i}");
                    m.visual_properties.stack_order = i;
                }
                v
            })
            .collect();
        assert_eq!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .audio_layers
                .len(),
            4096
        );
        let mut extra = instance;
        if let TimelineItem::ComponentInstance(i) = &mut extra {
            i.id = "extra".into();
            i.visual_properties.stack_order = 64;
        }
        p.tracks[0].items.push(extra);
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap_err()
                .message
                .contains("layer limit")
        );
    }

    #[test]
    fn canonical_half_open_clock_cases_and_hidden_nonfinite_clock() {
        let fixtures: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/component-evaluation-v1.json"
        ))
        .unwrap();
        for case in fixtures["clockCases"].as_array().unwrap() {
            let parent = EvaluatedInstance {
                rate: 1.0,
                offset: 0.0,
                start_ms: 0.0,
                end_ms: 1000.0,
                canvas: (100, 100),
            };
            let mut p = project();
            let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] else {
                unreachable!()
            };
            i.start_ms = case["startMs"].as_u64().unwrap();
            i.trim_start_ms = case["trimStartMs"].as_u64().unwrap();
            i.duration_ms = case["durationMs"].as_u64().unwrap();
            i.time_scale = case["timeScale"].as_f64().unwrap();
            let clock = parent.child(i, (100, 100)).unwrap();
            let t = case["parentMs"].as_f64().unwrap();
            if let Some(expected) = case["localMs"].as_f64() {
                assert_eq!(clock.rate * t + clock.offset, expected);
            } else {
                assert!(!(clock.start_ms <= t && t < clock.end_ms));
            }
        }
        let mut p = project();
        p.tracks[0].hidden = true;
        if let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] {
            i.time_scale = 1e-310;
        }
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap_err()
                .message
                .contains("non-finite")
        );
    }

    #[test]
    fn expanded_transition_facts_have_an_inclusive_limit() {
        let mut p = project();
        for (index, start) in [0, 400].into_iter().enumerate() {
            p.components[0].tracks[0].items.push(serde_json::from_value(json!({"type":"transition","id":format!("fade{index}"),"transitionType":"fade","fromItemId":"same","startMs":start,"durationMs":100,"stackOrder":index+1})).unwrap());
        }
        let instance = p.tracks[0].items[0].clone();
        p.tracks[0].items = (0..2048)
            .map(|n| {
                let mut v = instance.clone();
                if let TimelineItem::ComponentInstance(i) = &mut v {
                    i.id = format!("i{n}");
                    i.visual_properties.stack_order = n;
                }
                v
            })
            .collect();
        let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
        assert_eq!(
            scene
                .visual_layers
                .iter()
                .map(|l| l.transitions.len())
                .sum::<usize>(),
            4096
        );
        let mut extra = instance;
        if let TimelineItem::ComponentInstance(i) = &mut extra {
            i.id = "extra".into();
            i.visual_properties.stack_order = 2048;
        }
        p.tracks[0].items.push(extra);
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap_err()
                .message
                .contains("transition limit")
        );
    }

    #[test]
    fn expanded_visual_limit_is_inclusive_and_hidden_graph_is_validated() {
        let mut p = project();
        let leaf = p.components[0].tracks[0].items[0].clone();
        p.components[0].tracks[0].items = (0..64)
            .map(|i| {
                let mut v = leaf.clone();
                if let TimelineItem::Rectangle(r) = &mut v {
                    r.id = format!("r{i}");
                    r.visual_properties.stack_order = i;
                }
                v
            })
            .collect();
        let instance = p.tracks[0].items[0].clone();
        p.tracks[0].items = (0..64)
            .map(|i| {
                let mut v = instance.clone();
                if let TimelineItem::ComponentInstance(r) = &mut v {
                    r.id = format!("i{i}");
                    r.visual_properties.stack_order = i;
                }
                v
            })
            .collect();
        assert_eq!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .visual_layers
                .len(),
            4096
        );
        let mut extra = instance;
        if let TimelineItem::ComponentInstance(r) = &mut extra {
            r.id = "extra".into();
            r.visual_properties.stack_order = 64;
        }
        p.tracks[0].items.push(extra);
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap_err()
                .message
                .contains("layer limit")
        );
        p.tracks[0].hidden = true;
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .visual_layers
                .is_empty()
        );
        if let TimelineItem::ComponentInstance(r) = &mut p.components[1].tracks[0].items[0] {
            r.component_id = "missing".into();
        }
        assert_eq!(
            evaluate_project(&p, 100, 100, 30).unwrap_err().code,
            ErrorCode::ItemNotFound
        );
    }

    #[test]
    fn every_canonical_slot_kind_is_resolved_in_root_occurrences() {
        let slots: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/template-slots-v1.json"))
                .unwrap();
        for fixture in slots["valid"].as_array().unwrap() {
            let mut p = project();
            p.components.truncate(1);
            p.components[0].duration_ms = 1000;
            p.components[0].tracks[0].items=serde_json::from_value(json!([{"type":"text","id":"title","text":"Base","fontSize":20,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]}])).unwrap();
            p.components[0].slots = serde_json::from_value(json!([fixture["slot"]])).unwrap();
            if let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] {
                i.component_id = "leaf".into();
                i.time_scale = 1.0;
                i.start_ms = 0;
                i.trim_start_ms = 0;
                i.duration_ms = 1000;
                i.visual_properties.transform = Transform::default();
            }
            let kind = fixture["id"].as_str().unwrap();
            if kind == "asset" {
                p.assets=serde_json::from_value(json!([{"id":"managed_asset","mediaType":"image","fileName":"image.png","projectRelativePath":"assets/image.png","durationMs":null,"hasAudio":false}])).unwrap();
                p.components[0].tracks[0].items=serde_json::from_value(json!([{"type":"media","id":"media","assetId":"managed_asset","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}])).unwrap();
            }
            let before = serde_json::to_value(&p).unwrap();
            let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
            let layer = &scene.visual_layers[0];
            if let EvaluatedVisualSource::Text(t) = &layer.source {
                match kind {
                    "text" | "rich_text" => assert_eq!(t.text, "Hello"),
                    "color" => assert_eq!(t.color, "#12AbEf"),
                    "enum" => assert_eq!(t.style.alignment, EvaluatedTextAlignment::Center),
                    "duration" => assert_eq!(layer.instance.unwrap().end_ms, 500.0),
                    "number" => assert_eq!(layer.transform.opacity, 0.5),
                    "boolean" => assert_eq!(scene.visual_layers.len(), 1),
                    _ => unreachable!(),
                }
            } else {
                assert_eq!(scene.resources[0].asset_id, "managed_asset");
            }
            assert_eq!(serde_json::to_value(&p).unwrap(), before);
        }
    }

    #[test]
    fn root_slots_keep_independent_rich_runs_colors_and_defaults() {
        let mut p = project();
        let mut value = serde_json::to_value(&p).unwrap();
        value["components"][0]["tracks"][0]["items"] = json!([{"type":"text","id":"same","text":"Base","fontSize":20,"color":"#ffffff","startMs":0,"durationMs":500,"keyframes":[],"stackOrder":0}]);
        value["components"][0]["slots"] = json!([{"id":"__proto__","name":"Title","kind":"rich_text","required":true,"defaultValue":{"type":"rich_text","value":{"runs":[{"text":"Default","bold":true}]}},"binding":{"targetLayerId":"same","property":"text.document"},"constraints":{}}]);
        value["tracks"][0]["items"][0]["componentId"] = json!("leaf");
        value["tracks"][0]["items"][0]["durationMs"] = json!(300);
        let mut second = value["tracks"][0]["items"][0].clone();
        second["id"] = json!("second");
        second["stackOrder"] = json!(1);
        second["slotValues"] = json!({"__proto__":{"type":"rich_text","value":{"runs":[{"text":"Red","color":"#ff0000","italic":true},{"text":"Blue","color":"#0000ff"}]}}});
        value["tracks"][0]["items"]
            .as_array_mut()
            .unwrap()
            .push(second);
        p = serde_json::from_value(value.clone()).unwrap();
        let result = evaluate_project(&p, 100, 100, 30).unwrap();
        let texts = result
            .scene
            .visual_layers
            .iter()
            .map(|l| match &l.source {
                EvaluatedVisualSource::Text(t) => t,
                _ => panic!("not text"),
            })
            .collect::<Vec<_>>();
        assert_eq!(texts[0].text, "Default");
        assert_eq!(texts[0].rich_runs.as_ref().unwrap()[0].bold, Some(true));
        assert_eq!(texts[1].text, "RedBlue");
        assert_eq!(texts[1].rich_runs.as_ref().unwrap()[0].italic, Some(true));
        assert_eq!(
            texts[1].rich_runs.as_ref().unwrap()[1].color.as_deref(),
            Some("#0000ff")
        );
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            serde_json::to_value(serde_json::from_value::<Project>(value).unwrap()).unwrap()
        );
    }

    #[test]
    fn bounded_shared_graph_rejects_exponential_expansion() {
        let mut project = project();
        project.components.clear();
        for depth in 0..16 {
            let mut definition:crate::ComponentDefinition=serde_json::from_value(json!({"id":format!("c{depth}"),"name":"C","width":100,"height":100,"durationMs":1000,"slots":[],"tracks":[]})).unwrap();
            if depth > 0 {
                let items=(0..2).map(|i|json!({"type":"component_instance","id":format!("i{i}"),"componentId":format!("c{}",depth-1),"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"stackOrder":i})).collect::<Vec<_>>();
                definition.tracks = serde_json::from_value(
                    json!([{"id":"t","name":"T","trackType":"overlay","items":items}]),
                )
                .unwrap();
            }
            project.components.push(definition);
        }
        if let TimelineItem::ComponentInstance(instance) = &mut project.tracks[0].items[0] {
            instance.component_id = "c15".into();
        }
        project.tracks[0].hidden = true;
        assert!(evaluate_project(&project, 100, 100, 30).is_ok()); // 65535 occurrences.
        let mut empty = project.tracks[0].items[0].clone();
        if let TimelineItem::ComponentInstance(i) = &mut empty {
            i.id = "empty".into();
            i.component_id = "c0".into();
            i.visual_properties.stack_order = 1;
        }
        project.tracks[0].items.push(empty);
        assert!(evaluate_project(&project, 100, 100, 30).is_ok()); // Exactly 65536.
        project.tracks[0].items.pop();
        let mut second = project.tracks[0].items[0].clone();
        if let TimelineItem::ComponentInstance(instance) = &mut second {
            instance.id = "second".into();
            instance.visual_properties.stack_order = 1;
        }
        project.tracks[0].items.push(second);
        let error = evaluate_project(&project, 100, 100, 30).unwrap_err();
        assert!(
            error.message.contains("maxExpandedOccurrences"),
            "{error:?}"
        );
    }

    #[test]
    fn invalid_repeater_graphs_fail_in_a_subprocess_without_aborting() {
        for mode in ["component", "group"] {
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "evaluated_scene::instance_tests::invalid_repeater_graph_child",
                ])
                .env("OPENCUT_INVALID_REPEATER_GRAPH", mode)
                .status()
                .unwrap();
            assert!(status.success(), "{mode} cycle aborted its subprocess");
        }
    }

    #[test]
    #[ignore = "subprocess helper for invalid graph isolation"]
    fn invalid_repeater_graph_child() {
        let mode = std::env::var("OPENCUT_INVALID_REPEATER_GRAPH").unwrap();
        let project: Project = if mode == "component" {
            serde_json::from_value(json!({
                "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
                "settings":{"width":100,"height":100,"fps":30},"assets":[],"tracks":[],
                "components":[
                    {"id":"a","name":"A","width":100,"height":100,"durationMs":1000,"slots":[],
                     "tracks":[{"id":"a-track","name":"A","trackType":"overlay","hidden":true,"items":[
                        {"type":"component_instance","id":"to-b","componentId":"b","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"stackOrder":0}
                     ]}]},
                    {"id":"b","name":"B","width":100,"height":100,"durationMs":1000,"slots":[],
                     "tracks":[{"id":"b-track","name":"B","trackType":"overlay","hidden":true,"items":[
                        {"type":"component_instance","id":"to-a","componentId":"a","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"stackOrder":0}
                     ]}]}
                ]
            }))
            .unwrap()
        } else {
            serde_json::from_value(json!({
                "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
                "settings":{"width":100,"height":100,"fps":30},"assets":[],"components":[],
                "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
                    {"type":"group","id":"a","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"parent":{"scope":"root","id":"b"}},
                    {"type":"group","id":"b","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"a"}},
                    {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2,
                     "repeater":{"source":{"scope":"root","id":"a"},"copies":1,
                     "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
                ]}]
            }))
            .unwrap()
        };
        let error = evaluate_project(&project, 100, 100, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
    }

    #[test]
    fn missing_repeater_source_keeps_item_not_found_precedence() {
        let mut project = project();
        project.schema_version = 17;
        project.components.clear();
        project.tracks = serde_json::from_value(json!([{
            "id":"root","name":"Root","trackType":"overlay","items":[
                {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"stackOrder":0,
                 "repeater":{"source":{"scope":"root","id":"missing"},"copies":1,
                 "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]
        }]))
        .unwrap();
        assert_eq!(
            evaluate_project(&project, 100, 100, 30).unwrap_err().code,
            ErrorCode::ItemNotFound
        );
    }

    #[test]
    fn repeater_adds_stable_translated_and_faded_shape_occurrences() {
        let mut value = project();
        value.schema_version = crate::PROJECT_SCHEMA_VERSION;
        value.components.clear();
        value.tracks = serde_json::from_value(serde_json::json!([{
            "id":"overlay","name":"Overlay","trackType":"overlay","items":[
                {"type":"shape","id":"source","geometry":{"type":"rectangle","width":10,"height":10},
                 "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
                 "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0,
                 "transform2d":{"position":{"x":10,"y":10,"unit":"pixels"},"anchor":{"x":0,"y":0},
                 "scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}},
                {"type":"repeater","id":"copies","startMs":100,"durationMs":800,"zIndex":0,"stackOrder":1,
                 "repeater":{"source":{"scope":"root","id":"source"},"copies":3,
                 "transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                 "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.25}}
            ]
        }])).unwrap();
        let evaluated = evaluate_project(&value, 100, 100, 30).unwrap();
        assert_eq!(evaluated.scene.visual_layers.len(), 4);
        assert_eq!(evaluated.scene.visual_layers[0].item_id, "source");
        for (index, layer) in evaluated.scene.visual_layers[1..].iter().enumerate() {
            assert_eq!(
                layer.item_id,
                format!("repeater:copies:{:03}:000000:source", index + 1)
            );
            assert_eq!(
                layer.visible_span(),
                EvaluatedTimeSpan {
                    start_ms: 100,
                    end_ms: 900
                }
            );
            let affine = layer.affine.expect("shape affine");
            assert!((affine.opacity - 0.8 * (1.0 - (index + 1) as f64 * 0.25)).abs() < 1e-12);
        }
    }

    #[test]
    fn ordinary_components_do_not_materialize_repeater_copies() {
        GENERATED_MATERIALIZATIONS.with(|count| count.set(0));
        assert!(
            !evaluate_project(&project(), 100, 100, 30)
                .unwrap()
                .scene
                .visual_layers
                .is_empty()
        );
        assert_eq!(GENERATED_MATERIALIZATIONS.with(|count| count.get()), 0);
    }

    #[test]
    fn repeater_visual_layer_preflight_accepts_4096_and_rejects_4097() {
        fn fixture(last_copies: u16) -> Project {
            let mut items = vec![json!({
                "type":"shape","id":"source","startMs":0,"durationMs":1000,
                "geometry":{"type":"rectangle","width":1,"height":1},
                "fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,
                "keyframes":[],"zIndex":0,"stackOrder":0
            })];
            for index in 0..16 {
                let copies = if index == 15 { last_copies } else { 256 };
                items.push(json!({
                    "type":"repeater","id":format!("copies-{index}"),"startMs":0,"durationMs":1000,
                    "zIndex":0,"stackOrder":index + 1,
                    "repeater":{"source":{"scope":"root","id":"source"},"copies":copies,
                    "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                    "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}
                }));
            }
            serde_json::from_value(json!({
                "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
                "settings":{"width":16,"height":16,"fps":30},"assets":[],"components":[],
                "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":items}]
            }))
            .unwrap()
        }

        let exact = evaluate_project(&fixture(255), 16, 16, 30).unwrap();
        assert_eq!(exact.scene.visual_layers.len(), MAX_EVALUATED_VISUAL_LAYERS);
        let mut missing = fixture(255);
        missing.tracks[0].hidden = true;
        missing.tracks[0].items.push(serde_json::from_value(json!({
            "type":"media","id":"missing-media","assetId":"absent","startMs":0,"durationMs":1000,
            "sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},
            "keyframes":[],"stackOrder":17,"zIndex":0
        })).unwrap());
        assert_eq!(
            evaluate_project(&missing, 16, 16, 30).unwrap_err().code,
            ErrorCode::AssetNotFound
        );
        let error = evaluate_project(&fixture(256), 16, 16, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("expanded scene layer limit"));
        for mode in ["track", "repeater", "instance", "clipped", "unused"] {
            for copies in [255, 256] {
                let mut retained = fixture(copies);
                match mode {
                    "track" => retained.tracks[0].hidden = true,
                    "repeater" => {
                        for item in &mut retained.tracks[0].items[1..] {
                            item.visual_properties_mut().hidden = true;
                        }
                    }
                    _ => {
                        let mut tracks = serde_json::to_value(&retained.tracks).unwrap();
                        for item in tracks[0]["items"].as_array_mut().unwrap() {
                            if item["type"] == "repeater" {
                                item["repeater"]["source"]["scope"] = json!("component:definition");
                            }
                        }
                        retained.components = serde_json::from_value(json!([{
                            "id":"definition","name":"Definition","width":16,"height":16,
                            "durationMs":2000,"tracks":tracks,"slots":[]
                        }]))
                        .unwrap();
                        retained.tracks[0].items = if mode == "unused" {
                            vec![]
                        } else {
                            serde_json::from_value(json!([{"type":"component_instance","id":"instance",
                                "componentId":"definition","startMs":0,"durationMs":1000,
                                "trimStartMs":if mode == "clipped" {1000} else {0},"timeScale":1,
                                "hidden":mode == "instance","zIndex":0,"stackOrder":0,"slotValues":{}}])).unwrap()
                        };
                        if mode == "unused" && copies == 255 {
                            let mut second = retained.components[0].clone();
                            second.id = "independent".into();
                            for item in &mut second.tracks[0].items {
                                if let TimelineItem::Repeater(r) = item {
                                    r.repeater.source.scope = format!("component:{}", second.id);
                                }
                            }
                            retained.components.push(second);
                        }
                    }
                }
                GENERATED_MATERIALIZATIONS.with(|count| count.set(0));
                let result = evaluate_project(&retained, 16, 16, 30);
                if copies == 256 {
                    assert_eq!(
                        result.unwrap_err().code,
                        ErrorCode::InvalidArgument,
                        "{mode}"
                    );
                } else {
                    assert_eq!(
                        result.unwrap().scene.visual_layers.len(),
                        usize::from(mode == "repeater"),
                        "{mode}"
                    );
                    retained.components.reverse();
                    assert!(evaluate_project(&retained, 16, 16, 30).is_ok(), "{mode}");
                    if mode == "unused" {
                        let TimelineItem::Repeater(last) =
                            retained.components[1].tracks[0].items.last_mut().unwrap()
                        else {
                            unreachable!()
                        };
                        last.repeater.copies = 256;
                        assert_eq!(
                            evaluate_project(&retained, 16, 16, 30).unwrap_err().code,
                            ErrorCode::InvalidArgument
                        );
                    }
                }
                assert_eq!(
                    GENERATED_MATERIALIZATIONS.with(|count| count.get()),
                    0,
                    "{mode}"
                );
            }
        }
    }

    #[test]
    fn repeater_preflight_rejects_non_finite_parent_conjugation() {
        let project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":16,"height":16,"fps":30},"assets":[],"components":[],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
                {"type":"group","id":"parent","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
                 "transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},
                 "scaleX":100,"scaleY":100,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},
                {"type":"shape","id":"source","startMs":0,"durationMs":1000,
                 "geometry":{"type":"rectangle","width":1,"height":1},
                 "fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,
                 "keyframes":[],"zIndex":0,"stackOrder":1,
                 "parent":{"scope":"root","id":"parent"}},
                {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2,
                 "repeater":{"source":{"scope":"root","id":"source"},"copies":154,
                 "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":100,"scaleY":100,
                 "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        }))
        .unwrap();
        let error = evaluate_project(&project, 16, 16, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("non-finite repeater"));
        for hidden in [false, true] {
            let mut retained = project.clone();
            retained.tracks[0].hidden = hidden;
            GENERATED_MATERIALIZATIONS.with(|count| count.set(0));
            assert_eq!(
                evaluate_project(&retained, 16, 16, 30).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(GENERATED_MATERIALIZATIONS.with(|count| count.get()), 0);
        }
    }

    #[test]
    fn retained_repeater_surface_boundaries_precede_materialization() {
        for hidden in [false, true] {
            for width in [2047.0_f64, f64::from_bits(2047.0_f64.to_bits() + 1)] {
                let project: Project = serde_json::from_value(json!({
                    "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
                    "settings":{"width":16,"height":16,"fps":30},"assets":[],"components":[],
                    "tracks":[{"id":"root","name":"Root","trackType":"overlay","hidden":hidden,"items":[
                        {"type":"shape","id":"source","startMs":0,"durationMs":1000,
                         "geometry":{"type":"rectangle","width":width,"height":2047},
                         "fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,
                         "keyframes":[],"zIndex":0,"stackOrder":0},
                        {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,
                         "zIndex":0,"stackOrder":1,"repeater":{"source":{"scope":"root","id":"source"},"copies":1,
                         "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":2,"scaleY":2,
                         "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
                    ]}]
                })).unwrap();
                GENERATED_MATERIALIZATIONS.with(|count| count.set(0));
                let evaluated = evaluate_project(&project, 16, 16, 30);
                if width == 2047.0 {
                    assert_eq!(
                        evaluated.unwrap().scene.visual_layers.len(),
                        if hidden { 0 } else { 2 }
                    );
                } else {
                    assert_eq!(evaluated.unwrap_err().code, ErrorCode::InvalidArgument);
                    assert_eq!(GENERATED_MATERIALIZATIONS.with(|count| count.get()), 0);
                }
            }
        }
    }

    #[test]
    fn repeater_group_uses_source_parent_origin_and_preserves_subtree_order() {
        let project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],"components":[],
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":[
                {"type":"group","id":"group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
                 "transform2d":{"position":{"x":10,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}},
                {"type":"rectangle","id":"second","startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#00ff00","keyframes":[],"zIndex":0,"stackOrder":1,
                 "parent":{"scope":"root","id":"group"},"transform2d":{"position":{"x":5,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.5}},
                {"type":"rectangle","id":"first","startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#ff0000","keyframes":[],"zIndex":0,"stackOrder":2,"parent":{"scope":"root","id":"group"}},
                {"type":"repeater","id":"copies","startMs":200,"durationMs":300,"zIndex":1,"stackOrder":3,
                 "repeater":{"source":{"scope":"root","id":"group"},"copies":1,
                 "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":2,"scaleY":2,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.5}}
            ]}]
        })).unwrap();
        let before = serde_json::to_value(&project).unwrap();
        let mut scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        let generated = scene
            .visual_layers
            .iter()
            .filter(|layer| layer.item_id.starts_with("repeater:copies:"))
            .map(|layer| layer.item_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            generated,
            [
                "repeater:copies:001:000000:second",
                "repeater:copies:001:000001:first"
            ]
        );
        finalize_affine_geometry(&mut scene, &HashMap::new()).unwrap();
        let copy = scene
            .visual_layers
            .iter()
            .find(|layer| layer.item_id.ends_with(":second"))
            .unwrap();
        assert_eq!(
            copy.visible_span(),
            EvaluatedTimeSpan {
                start_ms: 200,
                end_ms: 500
            }
        );
        let affine = copy.affine.unwrap();
        assert_eq!(affine.matrix, [2.0, 0.0, 0.0, 2.0, 30.0, 0.0]);
        assert!((affine.opacity - 0.2).abs() < 1e-12);
        assert_eq!(serde_json::to_value(&project).unwrap(), before);
    }

    #[test]
    fn repeater_preflight_accepts_exact_occurrence_limit_and_rejects_one_over() {
        let groups = (0..256).map(|index| json!({
            "type":"group","id":format!("g{index}"),"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":index
        })).collect::<Vec<_>>();
        let mut project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[{"id":"definition","name":"Definition","width":100,"height":100,"durationMs":1000,"slots":[],
                "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":groups}]}],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","hidden":true,"items":[
                {"type":"component_instance","id":"source","componentId":"definition","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0},
                {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
                 "repeater":{"source":{"scope":"root","id":"source"},"copies":254,
                 "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        })).unwrap();
        assert!(evaluate_project(&project, 100, 100, 30).is_ok());
        project.tracks[0].items.push(
            serde_json::from_value(json!({
                "type":"group","id":"one-over","startMs":0,"durationMs":1,"zIndex":0,"stackOrder":2
            }))
            .unwrap(),
        );
        let error = evaluate_project(&project, 100, 100, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("maxExpandedOccurrences"));
    }

    #[test]
    fn nested_group_repeater_preflight_counts_exact_limit_and_one_over() {
        let nested_groups = (0..255)
            .map(|index| {
                json!({
                    "type":"group","id":format!("leaf-{index}"),"startMs":0,"durationMs":1000,
                    "hidden":true,"zIndex":0,"stackOrder":index + 2,
                    "parent":{"scope":"root","id":"inner"}
                })
            })
            .collect::<Vec<_>>();
        let mut items = vec![
            json!({"type":"group","id":"outer","startMs":0,"durationMs":1000,"hidden":true,"zIndex":0,"stackOrder":0}),
            json!({"type":"group","id":"inner","startMs":0,"durationMs":1000,"hidden":true,"zIndex":0,"stackOrder":1,
                   "parent":{"scope":"root","id":"outer"}}),
        ];
        items.extend(nested_groups);
        items.push(json!({
            "type":"repeater","id":"copies","startMs":0,"durationMs":1000,"hidden":true,
            "zIndex":0,"stackOrder":257,
            "repeater":{"source":{"scope":"root","id":"outer"},"copies":254,
            "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
            "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}
        }));
        let mut project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],"components":[],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","hidden":true,"items":items}]
        }))
        .unwrap();

        assert!(evaluate_project(&project, 100, 100, 30).is_ok());
        project.tracks[0].items.push(
            serde_json::from_value(json!({
                "type":"group","id":"one-over","startMs":0,"durationMs":1,
                "hidden":true,"zIndex":0,"stackOrder":258
            }))
            .unwrap(),
        );
        let error = evaluate_project(&project, 100, 100, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("maxExpandedOccurrences"));
    }

    #[test]
    fn hidden_unused_repeater_transform_overflow_fails_preflight() {
        let mut project = project();
        project.schema_version = 17;
        project.tracks.clear();
        project.components.truncate(1);
        project.components[0].tracks[0].hidden = true;
        project.components[0].tracks[0].items =
            vec![serde_json::from_value(json!({
            "type":"shape","id":"source","geometry":{"type":"rectangle","width":1,"height":1},
            "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
            "startMs":0,"durationMs":500,"keyframes":[],"zIndex":0,"stackOrder":0
        })).unwrap()];
        project.components[0].tracks[0].items.push(serde_json::from_value(json!({
            "type":"repeater","id":"copies","startMs":0,"durationMs":500,"hidden":true,"zIndex":0,"stackOrder":1,
            "repeater":{"source":{"scope":"component:leaf","id":"source"},"copies":256,
            "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":100,"scaleY":100,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}
        })).unwrap());
        let error = evaluate_project(&project, 100, 100, 30).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("non-finite repeater"));
        if let TimelineItem::Repeater(item) = &mut project.components[0].tracks[0].items[1] {
            item.repeater.transform_offset.scale_x = 1.0;
            item.repeater.transform_offset.scale_y = 1.0;
        }
        assert!(evaluate_project(&project, 100, 100, 30).is_ok());
    }

    #[test]
    fn repeater_clones_complete_component_occurrence_and_clock() {
        let project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[{"id":"badge","name":"Badge","width":50,"height":50,"durationMs":1000,"slots":[],
              "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[
                {"type":"shape","id":"shape","geometry":{"type":"rectangle","width":10,"height":10},
                 "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
                 "startMs":100,"durationMs":600,"keyframes":[],"zIndex":0,"stackOrder":0}
              ]}]}],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"component_instance","id":"source","componentId":"badge","startMs":200,"trimStartMs":0,"durationMs":500,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0},
              {"type":"repeater","id":"copies","startMs":300,"durationMs":300,"zIndex":0,"stackOrder":1,
               "repeater":{"source":{"scope":"root","id":"source"},"copies":2,
               "transformOffset":{"position":{"x":10,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        })).unwrap();
        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 3);
        let ordinary = &scene.visual_layers[0];
        assert_eq!(ordinary.instance.unwrap().start_ms, 300.0);
        assert_eq!(ordinary.instance.unwrap().end_ms, 700.0);
        let ordinary_x = ordinary.affine.unwrap().matrix[4];
        for (index, copy) in scene.visual_layers[1..].iter().enumerate() {
            assert!(
                copy.item_id
                    .starts_with(&format!("repeater:copies:{:03}:", index + 1))
            );
            let instance = copy.instance.unwrap();
            assert_eq!((instance.start_ms, instance.end_ms), (300.0, 600.0));
            assert_eq!(
                copy.affine.unwrap().matrix[4] - ordinary_x,
                10.0 * (index + 1) as f64
            );
        }
    }

    #[test]
    fn group_repeater_includes_nested_component_and_local_repeater_occurrences() {
        let project: Project = serde_json::from_str(r#"{
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[{"id":"leaf","name":"Leaf","width":100,"height":100,"durationMs":1000,"slots":[],
              "tracks":[{"id":"leaf-track","name":"Leaf","trackType":"overlay","items":[
                {"type":"shape","id":"leaf-shape","geometry":{"type":"rectangle","width":10,"height":10},
                 "fill":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"stroke":null,
                 "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0},
                {"type":"repeater","id":"leaf-copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
                 "repeater":{"source":{"scope":"component:leaf","id":"leaf-shape"},"copies":1,
                 "transformOffset":{"position":{"x":5,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                 "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
              ]}]}],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"group","id":"outer","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
               "transform2d":{"position":{"x":10,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},
               "scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}},
              {"type":"group","id":"inner","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
               "parent":{"scope":"root","id":"outer"}},
              {"type":"shape","id":"flat","geometry":{"type":"rectangle","width":10,"height":10},
               "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
               "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":2,
               "parent":{"scope":"root","id":"outer"}},
              {"type":"component_instance","id":"component","componentId":"leaf","startMs":0,"trimStartMs":0,
               "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":3,
               "parent":{"scope":"root","id":"inner"}},
              {"type":"repeater","id":"group-copy-a","startMs":100,"durationMs":500,"zIndex":1,"stackOrder":4,
               "repeater":{"source":{"scope":"root","id":"outer"},"copies":1,
               "transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
               "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.5}},
              {"type":"repeater","id":"group-copy-b","startMs":200,"durationMs":400,"zIndex":2,"stackOrder":5,
               "repeater":{"source":{"scope":"root","id":"outer"},"copies":1,
               "transformOffset":{"position":{"x":30,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
               "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        }"#).unwrap();

        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 9);
        for repeater_id in ["group-copy-a", "group-copy-b"] {
            let copies = scene
                .visual_layers
                .iter()
                .filter(|layer| {
                    layer
                        .item_id
                        .starts_with(&format!("repeater:{repeater_id}:"))
                })
                .collect::<Vec<_>>();
            assert_eq!(copies.len(), 3, "{repeater_id} omitted a group descendant");
            assert_eq!(
                copies
                    .iter()
                    .filter(|layer| layer.item_id.contains("leaf-copies"))
                    .count(),
                1,
                "{repeater_id} omitted the nested local repeater occurrence"
            );
            assert!(copies.iter().all(|layer| {
                let instance = layer.instance.unwrap();
                instance.start_ms
                    >= if repeater_id == "group-copy-a" {
                        100.0
                    } else {
                        200.0
                    }
                    && instance.end_ms <= 600.0
            }));
        }
        let faded_component = scene
            .visual_layers
            .iter()
            .find(|layer| {
                layer.item_id.starts_with("repeater:group-copy-a:")
                    && layer.item_id.ends_with("0-3_0-0")
            })
            .unwrap();
        assert!((faded_component.affine.unwrap().opacity - 0.4).abs() < 1e-12);
    }

    #[test]
    fn component_repeater_preserves_numeric_order_for_eleven_equal_z_siblings() {
        let siblings = (0..11)
            .map(|index| json!({
                "type":"shape","id":format!("layer-{index}"),
                "geometry":{"type":"rectangle","width":10,"height":10},
                "fill":{"type":"solid","color":{"r":index as f64 / 10.0,"g":0,"b":0,"a":1}},"stroke":null,
                "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":index
            }))
            .collect::<Vec<_>>();
        let project: Project = serde_json::from_value(json!({
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[{"id":"stack","name":"Stack","width":100,"height":100,"durationMs":1000,"slots":[],
              "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":siblings}]}],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"component_instance","id":"source","componentId":"stack","startMs":0,"trimStartMs":0,
               "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0},
              {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
               "repeater":{"source":{"scope":"root","id":"source"},"copies":1,
               "transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
               "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        })).unwrap();

        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        let generated = scene
            .visual_layers
            .iter()
            .filter(|layer| layer.item_id.starts_with("repeater:copies:"))
            .map(|layer| layer.item_id.rsplit('_').next().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            generated,
            (0..11)
                .map(|index| format!("0-{index}"))
                .collect::<Vec<_>>()
        );
        let EvaluatedVisualSource::Shape(top) = &scene.visual_layers.last().unwrap().source else {
            panic!("topmost generated sibling is not a shape")
        };
        let Some(crate::Paint::Solid { color }) = &top.fill else {
            panic!("topmost generated sibling has no solid fill")
        };
        assert_eq!(color.r, 1.0);
    }

    #[test]
    fn component_local_group_repeater_includes_nested_component_occurrences() {
        let project: Project = serde_json::from_str(r#"{
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[
              {"id":"leaf","name":"Leaf","width":100,"height":100,"durationMs":1000,"slots":[],
               "tracks":[{"id":"leaf-track","name":"Leaf","trackType":"overlay","items":[
                 {"type":"shape","id":"leaf-shape","geometry":{"type":"rectangle","width":10,"height":10},
                  "fill":{"type":"solid","color":{"r":0,"g":1,"b":0,"a":1}},"stroke":null,
                  "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0}
               ]}]},
              {"id":"container","name":"Container","width":100,"height":100,"durationMs":1000,"slots":[],
               "tracks":[{"id":"container-track","name":"Container","trackType":"overlay","items":[
                 {"type":"group","id":"local-group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0},
                 {"type":"component_instance","id":"local-component","componentId":"leaf","startMs":0,"trimStartMs":0,
                  "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":1,
                  "parent":{"scope":"component:container","id":"local-group"}},
                 {"type":"repeater","id":"local-group-copy","startMs":100,"durationMs":500,"zIndex":1,"stackOrder":2,
                  "repeater":{"source":{"scope":"component:container","id":"local-group"},"copies":1,
                  "transformOffset":{"position":{"x":15,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                  "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
               ]}]}
            ],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"component_instance","id":"root-component","componentId":"container","startMs":0,"trimStartMs":0,
               "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0}
            ]}]
        }"#).unwrap();

        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 2);
        let generated = scene
            .visual_layers
            .iter()
            .find(|layer| layer.item_id.contains("repeater:local-group-copy:"))
            .expect("component-local group copy omitted its nested component");
        let instance = generated.instance.unwrap();
        assert_eq!((instance.start_ms, instance.end_ms), (100.0, 600.0));
    }

    #[test]
    fn rich_text_bindings_reach_local_and_outer_repeater_copies_without_scope_leakage() {
        let project: Project = serde_json::from_str(r##"{
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[
              {"id":"leaf","name":"Leaf","width":100,"height":100,"durationMs":1000,"slots":[],
               "tracks":[{"id":"leaf-track","name":"Leaf","trackType":"overlay","items":[
                 {"type":"text","id":"title","text":"Nested","fontSize":20,"color":"#ffffff",
                  "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0}
               ]}]},
              {"id":"container","name":"Container","width":100,"height":100,"durationMs":1000,
               "slots":[{"id":"copy","name":"Copy","kind":"rich_text","required":true,
                 "defaultValue":{"type":"rich_text","value":{"runs":[{"text":"Default","bold":true}]}},
                 "binding":{"targetLayerId":"title","property":"text.document"},"constraints":{}}],
               "tracks":[{"id":"container-track","name":"Container","trackType":"overlay","items":[
                 {"type":"group","id":"local-group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0},
                 {"type":"text","id":"title","text":"Base","fontSize":20,"color":"#ffffff",
                  "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":1,
                  "parent":{"scope":"component:container","id":"local-group"}},
                 {"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,
                  "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":2,
                  "parent":{"scope":"component:container","id":"local-group"}},
                 {"type":"repeater","id":"local-copies","startMs":50,"durationMs":700,"zIndex":1,"stackOrder":3,
                  "repeater":{"source":{"scope":"component:container","id":"local-group"},"copies":1,
                  "transformOffset":{"position":{"x":10,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                  "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
               ]}]}
            ],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"component_instance","id":"source","componentId":"container","startMs":100,"trimStartMs":0,
               "durationMs":800,"timeScale":1,"slotValues":{"copy":{"type":"rich_text","value":{"runs":[
                 {"text":"Red","color":"#ff0000","italic":true},{"text":"Blue","color":"#0000ff"}
               ]}}},"zIndex":0,"stackOrder":0},
              {"type":"repeater","id":"outer-copies","startMs":200,"durationMs":500,"zIndex":1,"stackOrder":1,
               "repeater":{"source":{"scope":"root","id":"source"},"copies":1,
               "transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
               "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
            ]}]
        }"##)
        .unwrap();

        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 8);
        let mut overridden = 0;
        let mut nested = 0;
        for layer in &scene.visual_layers {
            let EvaluatedVisualSource::Text(text) = &layer.source else {
                panic!("repeater fixture emitted a non-text visual layer")
            };
            if text.text == "RedBlue" {
                overridden += 1;
                let runs = text.rich_runs.as_ref().expect("rich runs were discarded");
                assert_eq!(runs.len(), 2);
                assert_eq!(runs[0].italic, Some(true));
                assert_eq!(runs[0].color.as_deref(), Some("#ff0000"));
                assert_eq!(runs[1].color.as_deref(), Some("#0000ff"));
            } else {
                nested += 1;
                assert_eq!(text.text, "Nested");
                assert!(
                    text.rich_runs.is_none(),
                    "rich text leaked into nested scope"
                );
            }
            if layer.item_id.starts_with("repeater:outer-copies:") {
                let instance = layer.instance.unwrap();
                assert_eq!((instance.start_ms, instance.end_ms), (200.0, 700.0));
            }
        }
        assert_eq!((overridden, nested), (4, 4));
    }

    #[test]
    fn component_local_repeater_occurrences_keep_unique_scoped_identities() {
        let project: Project = serde_json::from_str(r#"{
            "schemaVersion":17,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":100,"height":100,"fps":30},"assets":[],
            "components":[{"id":"badge","name":"Badge","width":100,"height":100,"durationMs":1000,"slots":[],
              "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[
                {"type":"shape","id":"shape","geometry":{"type":"rectangle","width":10,"height":10},
                 "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
                 "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0},
                {"type":"repeater","id":"local-copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
                 "repeater":{"source":{"scope":"component:badge","id":"shape"},"copies":2,
                 "transformOffset":{"position":{"x":10,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                 "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
              ]}]}],
            "tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[
              {"type":"component_instance","id":"source","componentId":"badge","startMs":0,"trimStartMs":0,
               "durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0}
            ]}]
        }"#)
        .unwrap();
        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 3);
        let ids = scene
            .visual_layers
            .iter()
            .map(|layer| layer.item_id.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(ids.len(), 3);
        assert_eq!(
            scene
                .visual_layers
                .iter()
                .filter(|layer| layer.item_id.contains("repeater:local-copies:"))
                .count(),
            2
        );
    }
}
