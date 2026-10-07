//! Owned controlled-isolation facts. No authored lookup is needed by execution.
use super::composition_resources::{add, capacity_bytes, channel_heap, mul};
use super::*;
pub(super) mod continuous;
pub(crate) mod frame;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AggregateGraph {
    pub nodes: Vec<Aggregate>,
    pub leaf_orders: Vec<InstanceOrder>,
    pub leaf_intervals: Vec<Vec<OccurrenceInterval>>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Aggregate {
    pub authored_id: String,
    pub order: InstanceOrder,
    pub basis: (u32, u32),
    pub clip: bool,
    pub effects: Vec<crate::VisualEffect>,
    pub stages: Vec<EvaluatedAncestorStage>,
    pub intervals: Vec<OccurrenceInterval>,
    pub clock: EvaluatedInstance,
    pub start_ms: u64,
    pub parent: Option<usize>,
}
impl Aggregate {
    pub(crate) fn visible_at(&self, at: u64) -> bool {
        at as f64 >= self.clock.start_ms && (at as f64) < self.clock.end_ms
    }
    pub(crate) fn time(&self, at: u64) -> crate::animation::SampleTime {
        crate::animation::SampleTime::local(
            at,
            self.start_ms,
            Some((self.clock.rate, self.clock.offset)),
        )
    }
}
/// Output factors stop before the immediately enclosing controlled owner.
/// Its frame is supplied by the caller at that enclosing output sample.
pub(crate) fn outward_stages(node: &Aggregate) -> Result<&[EvaluatedAncestorStage], CoreError> {
    let start = if let Some(parent) = node.parent {
        node.stages
            .iter()
            .rposition(|stage| stage.aggregate == Some(parent))
            .ok_or_else(|| invalid("aggregate parent stage missing"))?
            + 1
    } else {
        0
    };
    Ok(&node.stages[start..])
}
pub(crate) fn relative_layer(
    layer: &EvaluatedVisualLayer,
    owner: usize,
) -> Result<EvaluatedVisualLayer, CoreError> {
    let boundary = layer
        .ancestor_stages
        .iter()
        .rposition(|stage| stage.aggregate == Some(owner))
        .ok_or_else(|| invalid("controlled leaf owner stage missing"))?;
    let mut local = layer.clone();
    local.ancestor_stages.drain(..=boundary);
    local.ancestors = None;
    local.affine = None;
    local.sampling_tiles = None;
    Ok(local)
}

pub(crate) fn controlled(item: &TimelineItem) -> bool {
    matches!(
        item,
        TimelineItem::Group(_) | TimelineItem::ComponentInstance(_)
    ) && (item.visual_properties().clip.is_some() || !item.visual_properties().effects.is_empty())
}
pub(crate) fn heap_bytes(graph: &AggregateGraph) -> Result<u64, CoreError> {
    let mut bytes = add(
        std::mem::size_of::<AggregateGraph>() as u64,
        capacity_bytes(&graph.nodes)?,
    )?;
    bytes = add(bytes, capacity_bytes(&graph.leaf_orders)?)?;
    for order in &graph.leaf_orders {
        bytes = add(bytes, order_bytes(order)?)?;
    }
    bytes = add(bytes, capacity_bytes(&graph.leaf_intervals)?)?;
    for intervals in &graph.leaf_intervals {
        bytes = add(bytes, capacity_bytes(intervals)?)?;
        for interval in intervals {
            bytes = add(bytes, interval.item_id.capacity() as u64)?;
        }
    }
    for node in &graph.nodes {
        bytes = add(bytes, node_bytes(node)?)?;
    }
    Ok(bytes)
}
fn node_bytes(node: &Aggregate) -> Result<u64, CoreError> {
    let mut bytes = 0;
    bytes = add(bytes, node.authored_id.capacity() as u64)?;
    bytes = add(bytes, order_bytes(&node.order)?)?;
    bytes = add(bytes, capacity_bytes(&node.effects)?)?;
    for effect in &node.effects {
        let id = match effect {
            crate::VisualEffect::GaussianBlur { id, .. }
            | crate::VisualEffect::Glow { id, .. }
            | crate::VisualEffect::ColorTint { id, .. }
            | crate::VisualEffect::Vignette { id, .. }
            | crate::VisualEffect::ColorAdjustment { id, .. }
            | crate::VisualEffect::ScreenFlash { id, .. }
            | crate::VisualEffect::ParticleOverlay { id, .. } => id,
        };
        bytes = add(bytes, id.capacity() as u64)?;
    }
    bytes = add(bytes, capacity_bytes(&node.stages)?)?;
    for stage in &node.stages {
        bytes = add(bytes, stage.item_id.capacity() as u64)?;
        if let Some(animation) = &stage.animation {
            bytes = add(bytes, channel_heap(&animation.channels)?)?;
            bytes = add(
                bytes,
                mul(
                    animation.keyframes.len() as u64,
                    std::mem::size_of::<EvaluatedKeyframe>() as u64,
                )?,
            )?;
        }
    }
    bytes = add(bytes, capacity_bytes(&node.intervals)?)?;
    for interval in &node.intervals {
        bytes = add(bytes, interval.item_id.capacity() as u64)?;
    }
    Ok(bytes)
}

fn order_bytes(order: &InstanceOrder) -> Result<u64, CoreError> {
    let mut bytes = capacity_bytes(order)?;
    for (_, _, _, id) in order {
        bytes = add(bytes, id.capacity() as u64)?;
    }
    Ok(bytes)
}
pub(crate) fn leaf_owner(layer: &EvaluatedVisualLayer) -> Option<usize> {
    layer
        .ancestor_stages
        .iter()
        .rev()
        .find_map(|stage| stage.aggregate)
}

pub(super) fn publish_leaf_facts(
    scene: &mut EvaluatedScene,
    orders: &HashMap<String, InstanceOrder>,
    projection: &[ProjectedVisualCopy],
) -> Result<(), CoreError> {
    let Some(graph) = &scene.aggregates else {
        return Ok(());
    };
    if graph
        .nodes
        .len()
        .checked_add(scene.visual_layers.len())
        .is_none_or(|n| n > MAX_EVALUATED_VISUAL_LAYERS)
    {
        return Err(invalid("expanded aggregate occurrence limit exceeded"));
    }
    let mut reservation = mul(
        scene.visual_layers.len() as u64,
        (std::mem::size_of::<InstanceOrder>() + std::mem::size_of::<Vec<OccurrenceInterval>>())
            as u64,
    )?;
    for layer in &scene.visual_layers {
        reservation = add(reservation, order_bytes(&orders[&layer.item_id])?)?;
        let projected = projection
            .iter()
            .find(|copy| copy.item_id == layer.item_id)
            .ok_or_else(|| invalid("aggregate leaf occurrence facts missing"))?;
        reservation = add(reservation, capacity_bytes(&projected.intervals)?)?;
        for interval in &projected.intervals {
            reservation = add(reservation, interval.item_id.capacity() as u64)?;
        }
    }
    if add(
        add(
            composition_resources::composition_heap_bytes(scene)?,
            mul(reservation, 4)?,
        )?,
        mattes::MATTE_CACHE_RESERVATION,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("aggregate leaf facts exceed shared live memory"));
    }
    let graph = scene.aggregates.as_mut().unwrap();
    graph.leaf_orders = scene
        .visual_layers
        .iter()
        .map(|layer| orders[&layer.item_id].clone())
        .collect();
    graph.leaf_intervals = scene
        .visual_layers
        .iter()
        .map(|layer| {
            projection
                .iter()
                .find(|copy| copy.item_id == layer.item_id)
                .unwrap()
                .intervals
                .clone()
        })
        .collect();
    if add(
        composition_resources::composition_heap_bytes(scene)?,
        mattes::MATTE_CACHE_RESERVATION,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid(
            "aggregate leaf fact capacities exceed shared live memory",
        ));
    }
    Ok(())
}
pub(crate) fn relative_visible_at(
    scene: &EvaluatedScene,
    index: usize,
    owner: usize,
    at: u64,
) -> Result<bool, CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate facts absent"))?;
    let node = graph
        .nodes
        .get(owner)
        .ok_or_else(|| invalid("aggregate owner absent"))?;
    let stage = node
        .stages
        .last()
        .ok_or_else(|| invalid("aggregate owner stage absent"))?;
    let intervals = graph
        .leaf_intervals
        .get(index)
        .ok_or_else(|| invalid("aggregate leaf intervals absent"))?;
    let boundary = intervals
        .iter()
        .rposition(|part| part.scope == stage.scope && part.item_id == stage.item_id)
        .ok_or_else(|| invalid("aggregate leaf interval boundary absent"))?;
    Ok(intervals[boundary + 1..]
        .iter()
        .all(|part| at as f64 >= part.start_ms && (at as f64) < part.end_ms))
}

/// Exact integer transitions of the frozen relative occurrence predicate.
pub(super) fn relative_integer_visibility(
    scene: &EvaluatedScene,
    index: usize,
    owner: usize,
) -> Result<(u64, u64), CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate facts absent"))?;
    let stage = graph
        .nodes
        .get(owner)
        .and_then(|n| n.stages.last())
        .ok_or_else(|| invalid("aggregate owner stage absent"))?;
    let intervals = graph
        .leaf_intervals
        .get(index)
        .ok_or_else(|| invalid("aggregate leaf intervals absent"))?;
    let boundary = intervals
        .iter()
        .rposition(|p| p.scope == stage.scope && p.item_id == stage.item_id)
        .ok_or_else(|| invalid("aggregate leaf interval boundary absent"))?;
    let start = intervals[boundary + 1..]
        .iter()
        .map(|p| p.start_ms)
        .fold(0., f64::max);
    let end = intervals[boundary + 1..]
        .iter()
        .map(|p| p.end_ms)
        .fold(f64::INFINITY, f64::min);
    let first = |bound: f64| {
        let (mut low, mut high) = (0u64, u64::MAX);
        while low < high {
            let middle = low + (high - low) / 2;
            if middle as f64 >= bound {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        low
    };
    Ok((first(start), first(end)))
}

pub(super) struct AggregateScope<'a, 'b> {
    pub tracks: &'a [Track],
    pub temporal: &'b ScopeTiming<'a>,
    pub clock: EvaluatedInstance,
    pub prefix: &'b [(usize, i32, usize, String)],
    pub transform_prefix: &'b [EvaluatedAncestorStage],
    pub interval_prefix: &'b [OccurrenceInterval],
    pub visual_start: f64,
    pub visual_end: f64,
    pub orders: &'b HashMap<String, InstanceOrder>,
}
pub(super) fn bind_stages(
    stages: &mut [EvaluatedAncestorStage],
    ids: &std::collections::BTreeMap<String, usize>,
) {
    for stage in stages {
        stage.aggregate = ids.get(&stage.item_id).copied();
    }
}
pub(super) fn register_scope(
    traversal: &InstanceTraversal<'_>,
    scope: &AggregateScope<'_, '_>,
    scene: &mut EvaluatedScene,
) -> Result<std::collections::BTreeMap<String, usize>, CoreError> {
    let Some(graph) = scene.aggregates.as_ref() else {
        return Ok(Default::default());
    };
    let count = scope
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|item| controlled(item))
        .count();
    if count == 0 {
        return Ok(Default::default());
    }
    if graph
        .nodes
        .len()
        .checked_add(count)
        .and_then(|n| n.checked_add(scene.visual_layers.len()))
        .is_none_or(|n| n > MAX_EVALUATED_VISUAL_LAYERS)
    {
        return Err(invalid("expanded aggregate descriptor limit exceeded"));
    }
    // Pre-admit growth, bounded map/identity staging and each complete copied
    // ancestor/channel payload before creating node records or cloning programs.
    let mut reserved = mul(count as u64, 4096)?;
    for item in scope
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|item| controlled(item))
    {
        for stage in scope.transform_prefix {
            reserved = add(reserved, stage.item_id.capacity() as u64)?;
            if let Some(animation) = &stage.animation {
                reserved = add(reserved, channel_heap(&animation.channels)?)?;
                reserved = add(
                    reserved,
                    mul(
                        animation.keyframes.len() as u64,
                        std::mem::size_of::<EvaluatedKeyframe>() as u64,
                    )?,
                )?;
            }
        }
        for (ancestor, _) in scope.temporal.path(item.id())? {
            reserved = add(reserved, ancestor.id().len() as u64)?;
            reserved = add(
                reserved,
                channel_heap(&ancestor.visual_properties().animation_channels)?,
            )?;
            reserved = add(
                reserved,
                mul(
                    ancestor
                        .visual_properties()
                        .animation_channels
                        .iter()
                        .map(|c| c.keyframes.len() as u64)
                        .sum(),
                    std::mem::size_of::<EvaluatedKeyframe>() as u64,
                )?,
            )?;
        }
        reserved = add(
            reserved,
            mul(
                scope.transform_prefix.len() as u64 + 33,
                std::mem::size_of::<EvaluatedAncestorStage>() as u64,
            )?,
        )?;
        reserved = add(
            reserved,
            mul(
                scope.interval_prefix.len() as u64 + 33,
                std::mem::size_of::<OccurrenceInterval>() as u64,
            )?,
        )?;
        reserved = add(
            reserved,
            mul(
                item.visual_properties().effects.capacity() as u64,
                std::mem::size_of::<crate::VisualEffect>() as u64,
            )?,
        )?;
        for effect in &item.visual_properties().effects {
            reserved = add(reserved, effect.id().len() as u64)?;
        }
        reserved = add(reserved, order_bytes(&scope.orders[item.id()])?)?;
    }
    let base = composition_resources::composition_heap_bytes(scene)?;
    if add(
        add(base, mul(reserved, 4)?)?,
        mattes::MATTE_CACHE_RESERVATION,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("aggregate facts exceed shared live memory"));
    }
    let mut ids = std::collections::BTreeMap::new();
    for item in scope
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|item| controlled(item))
    {
        let basis = if let TimelineItem::ComponentInstance(instance) = item {
            let definition = &traversal.project.components
                [traversal.definitions[instance.component_id.as_str()]];
            (definition.width, definition.height)
        } else {
            scope.clock.canvas
        };
        let timing = scope.temporal.window(item.id())?;
        let mut intervals = scope.interval_prefix.to_vec();
        intervals.extend(
            scope
                .temporal
                .root_path(item.id(), scope.clock, scope.prefix.len())?,
        );
        let (start, end) = occurrence_window(&intervals, scene.duration_ms);
        let graph = scene.aggregates.as_mut().unwrap();
        let index = graph.nodes.len();
        ids.insert(item.id().to_owned(), index);
        graph.nodes.push(Aggregate {
            authored_id: item.id().to_owned(),
            order: scope.orders[item.id()].clone(),
            basis,
            clip: item.visual_properties().clip.is_some(),
            effects: item.visual_properties().effects.clone(),
            stages: Vec::new(),
            intervals,
            clock: EvaluatedInstance {
                offset: scope.clock.offset - timing.delay_ms as f64,
                start_ms: start.max(scope.visual_start),
                end_ms: end.min(scope.visual_end),
                ..scope.clock
            },
            start_ms: item.start_ms(),
            parent: None,
        });
    }
    for (id, index) in &ids {
        let mut stages = scope.transform_prefix.to_vec();
        let mut local =
            traversal.stages_for(scope.temporal, id, scope.clock, scope.prefix.len(), true)?;
        bind_stages(&mut local, &ids);
        stages.extend(local);
        let parent = stages
            .iter()
            .rev()
            .skip(1)
            .find_map(|stage| stage.aggregate);
        let node = &mut scene.aggregates.as_mut().unwrap().nodes[*index];
        node.stages = stages;
        node.parent = parent;
    }
    Ok(ids)
}

/// Select actual immutable occurrences, including empty and nested owners, then
/// admit the entire multiplicity before allocating remaps or cloning records.
pub(super) fn repeater_sources(
    scene: &EvaluatedScene,
    range: std::ops::Range<usize>,
    source: &str,
    scope: usize,
    copies: usize,
    projected_leaves: usize,
    copied_leaves: usize,
) -> Result<Vec<usize>, CoreError> {
    let Some(graph) = &scene.aggregates else {
        return Ok(Vec::new());
    };
    let selected = |node: &Aggregate| {
        node.stages
            .iter()
            .any(|stage| stage.scope == scope && stage.item_id == source)
    };
    let count = graph.nodes[range.clone()]
        .iter()
        .filter(|node| selected(node))
        .count();
    let descriptors = count
        .checked_add(copied_leaves)
        .and_then(|n| n.checked_mul(copies))
        .and_then(|n| n.checked_add(graph.nodes.len()))
        .and_then(|n| n.checked_add(projected_leaves))
        .filter(|n| *n <= MAX_EVALUATED_VISUAL_LAYERS)
        .ok_or_else(|| invalid("repeated aggregate occurrence limit exceeded"))?;
    let mut reservation = mul(descriptors as u64, 512)?;
    for node in graph.nodes[range.clone()]
        .iter()
        .filter(|node| selected(node))
    {
        reservation = add(
            reservation,
            mul(add(node_bytes(node)?, 4096)?, copies as u64)?,
        )?;
    }
    if add(
        add(
            composition_resources::composition_heap_bytes(scene)?,
            mul(reservation, 4)?,
        )?,
        mattes::MATTE_CACHE_RESERVATION,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid(
            "repeated aggregate facts exceed shared live memory",
        ));
    }
    Ok(range
        .filter(|index| selected(&graph.nodes[*index]))
        .collect())
}
pub(super) struct AggregateCopy<'a> {
    pub sources: &'a [usize],
    pub source_id: &'a str,
    pub scope: usize,
    pub shifted_root_ms: f64,
    pub intervals: &'a [OccurrenceInterval],
    pub matrix: [f64; 6],
    pub inverse: [f64; 6],
    pub opacity: f64,
    pub controller_id: &'a str,
    pub prefix: &'a [(usize, i32, usize, String)],
    pub controller_order: (usize, i32, usize),
    pub copy_index: usize,
}
pub(super) fn clone_repeater(
    scene: &mut EvaluatedScene,
    copy: &AggregateCopy<'_>,
) -> Result<std::collections::BTreeMap<usize, usize>, CoreError> {
    let Some(graph) = &mut scene.aggregates else {
        return Ok(Default::default());
    };
    let first = graph.nodes.len();
    let remap: std::collections::BTreeMap<_, _> = copy
        .sources
        .iter()
        .enumerate()
        .map(|(offset, source)| (*source, first + offset))
        .collect();
    for source in copy.sources {
        let mut node = graph.nodes[*source].clone();
        let boundary = node
            .intervals
            .iter()
            .position(|part| part.scope == copy.scope && part.item_id == copy.source_id)
            .ok_or_else(|| invalid("aggregate repeater timing boundary missing"))?;
        for interval in &mut node.intervals[boundary..] {
            interval.start_ms += copy.shifted_root_ms;
            interval.end_ms += copy.shifted_root_ms;
        }
        node.intervals
            .splice(boundary..boundary, copy.intervals.iter().cloned());
        let (start, end) = occurrence_window(&node.intervals, scene.duration_ms);
        node.clock.offset -= copy.shifted_root_ms * node.clock.rate;
        node.clock.start_ms = start;
        node.clock.end_ms = end;
        let boundary = node
            .stages
            .iter()
            .position(|stage| stage.scope == copy.scope && stage.item_id == copy.source_id)
            .ok_or_else(|| invalid("aggregate repeater transform boundary missing"))?;
        for stage in &mut node.stages[boundary..] {
            if let Some(animation) = &mut stage.animation {
                animation.clock.offset -= copy.shifted_root_ms * animation.clock.rate;
            }
        }
        for stage in &mut node.stages {
            if let Some(index) = stage.aggregate {
                stage.aggregate = Some(remap.get(&index).copied().unwrap_or(index));
            }
        }
        node.stages.insert(
            boundary,
            EvaluatedAncestorStage {
                aggregate: None,
                scope: copy.scope,
                item_id: copy.controller_id.to_owned(),
                matrix: copy.matrix,
                inverse: copy.inverse,
                opacity: copy.opacity,
                animation: None,
            },
        );
        node.parent = node
            .parent
            .map(|index| remap.get(&index).copied().unwrap_or(index));
        let mut order = copy.prefix.to_vec();
        let (track, z, stack) = copy.controller_order;
        order.push((track, z, stack, copy.controller_id.to_owned()));
        order.push((copy.copy_index, 0, 0, String::new()));
        order.extend(node.order.iter().skip(copy.prefix.len()).cloned());
        node.order = order;
        if !node.clock.offset.is_finite()
            || node
                .intervals
                .iter()
                .any(|part| !part.start_ms.is_finite() || !part.end_ms.is_finite())
        {
            return Err(invalid("nonfinite repeated aggregate clock"));
        }
        graph.nodes.push(node);
    }
    Ok(remap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture(leaf: bool) -> Project {
        let mut items = vec![
            json!({"type":"group","id":"group","startMs":0,"durationMs":1000,
            "zIndex":0,"stackOrder":0,"clip":{"type":"composition_bounds"}}),
            json!({"type":"group","id":"nested","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,
                "parent":{"scope":"root","id":"group"},"clip":{"type":"composition_bounds"}}),
        ];
        if leaf {
            items.push(json!({"type":"rectangle","id":"leaf","startMs":0,"durationMs":1000,"width":4,"height":3,"color":"#ff0000",
            "keyframes":[],"zIndex":0,"stackOrder":2,"parent":{"scope":"root","id":"nested"}}));
        }
        items.push(json!({"type":"repeater","id":"copies","startMs":200,"durationMs":700,"zIndex":1,"stackOrder":items.len(),
            "repeater":{"source":{"scope":"root","id":"group"},"copies":2,"timeOffsetMs":50,
                "transformOffset":{"position":{"x":7,"y":2,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.2}}));
        serde_json::from_value(json!({"schemaVersion":37,"id":"p","revision":0,"name":"P","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":64,"height":64,"fps":10},"assets":[],"components":[],"markers":[],"fonts":{},
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":items}]})).unwrap()
    }
    #[test]
    fn repeated_controlled_owners_preserve_empty_and_nested_occurrence_identity() {
        for leaf in [false, true] {
            let project = fixture(leaf);
            let before = serde_json::to_value(&project).unwrap();
            let scene = evaluate_project(&project, 64, 64, 10).unwrap().scene;
            let graph = scene.aggregates.as_ref().unwrap();
            assert_eq!(graph.nodes.len(), 6);
            assert_eq!(scene.visual_layers.len(), if leaf { 3 } else { 0 });
            for (copy, first) in [(0, 0), (1, 2), (2, 4)] {
                let owner = &graph.nodes[first];
                let child = &graph.nodes[first + 1];
                assert_eq!(owner.parent, None);
                assert_eq!(child.parent, Some(first));
                assert_eq!(owner.basis, (64, 64));
                assert!(owner.clip);
                assert!(owner.effects.is_empty());
                assert_eq!(owner.stages.last().unwrap().aggregate, Some(first));
                assert_eq!(child.stages.last().unwrap().aggregate, Some(first + 1));
                assert_eq!(
                    child.stages.iter().rev().nth(1).unwrap().aggregate,
                    Some(first)
                );
                assert_eq!(owner.visible_at(199), copy == 0);
                assert!(owner.visible_at(300));
                assert_eq!(owner.visible_at(900), copy == 0);
                assert_eq!(
                    owner.time(300).compare(300 - copy * 50),
                    Some(std::cmp::Ordering::Equal)
                );
                if copy > 0 {
                    let stage = &owner.stages[0];
                    assert_eq!(stage.item_id, "copies");
                    assert_eq!(
                        stage.matrix,
                        [1., 0., 0., 1., 7. * copy as f64, 2. * copy as f64]
                    );
                    assert!((stage.opacity - (1. - copy as f64 * 0.2)).abs() < 1e-12);
                    assert_eq!(owner.order[0].3, "copies");
                    assert_eq!(owner.order[1].0, copy as usize);
                }
            }
            if leaf {
                let owners: Vec<_> = scene.visual_layers.iter().map(leaf_owner).collect();
                assert_eq!(owners, [Some(1), Some(3), Some(5)]);
            }
            assert_eq!(serde_json::to_value(&project).unwrap(), before);
        }
    }
}
