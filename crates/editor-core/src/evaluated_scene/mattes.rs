//! Immutable scoped matte bindings and pre-certified exact frame tasks.
use super::composition_resources::{
    add, capacity_bytes, composition_heap_bytes, layer_heap_bytes, mul,
};
use crate::MatteChannel;
pub(crate) const MAX_MATTE_REQUESTS: u64 = 4096;
pub(crate) const MAX_MATTE_WORK: u64 = 268_435_456;
pub(crate) const MAX_MATTE_LIVE_BYTES: u64 = 1_073_741_824;
pub(crate) const MATTE_CACHE_RESERVATION: u64 = 67_108_864;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CompositionOccurrenceId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MatteGroupId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MatteTaskId(pub usize);
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MatteProviderGroup {
    pub composition: CompositionOccurrenceId,
    pub authored_item_id: String,
    pub members: Vec<usize>,
    pub matte_only: bool,
    pub provider: Option<(MatteGroupId, MatteChannel)>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMatteRole {
    pub group: Option<MatteGroupId>,
    pub provider: Option<(MatteGroupId, MatteChannel)>,
    pub matte_only: bool,
    pub contributes: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMatteGraph {
    pub groups: Vec<MatteProviderGroup>,
    pub roles: Vec<EvaluatedMatteRole>,
    pub provider_first: Vec<MatteGroupId>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MatteTask {
    LeafSample {
        relative_owner: Option<usize>,
        layer_index: usize,
        at_ms: u64,
        provider: Option<(MatteTaskId, MatteChannel)>,
        source_live_bytes: u64,
    },
    AverageCopy {
        layer_index: usize,
        samples: Vec<MatteTaskId>,
    },
    AggregateProvider {
        copies: Vec<MatteTaskId>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MatteFrameCertificate {
    /// Exactly the number of AggregateProvider tasks; bound4096.
    pub provider_requests: u64,
    /// Conservative aggregate4P/copy+coverage5P/recipient; bound268435456.
    pub matte_work_units: u64,
    pub blend_work_units: u64,
    /// Caller destination/cache/facts/schedule and executor-slot metadata.
    pub fixed_live_bytes: u64,
    /// Actual schedule/nested Vec capacities plus128 bytes/task executor slots.
    pub descriptor_bytes: u64,
    /// Absolute shared peak including fixed storage, bound1073741824.
    pub peak_live_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DirectDraw {
    pub task: MatteTaskId,
    pub layer_index: usize,
    pub blend_mode: crate::BlendMode,
    pub destination_visits: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MatteFrameSchedule {
    pub query: Option<QueryFrame>,
    pub canvas: (u32, u32),
    pub tasks: Vec<MatteTask>,
    pub direct_draw: Vec<DirectDraw>,
    pub owner_modes: Vec<crate::BlendMode>,
    /// Task indices followed by direct draws at tasks.len()+draw_index.
    pub last_uses: Vec<usize>,
    pub certificate: MatteFrameCertificate,
}

/// A certified query raster maps its pixel centers directly into world space.
/// Signed local origins and singular world bases require no inverse-owner map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct QueryFrame {
    pub size: (u32, u32),
    pub origin: [f64; 2],
    pub axes: [[f64; 2]; 2],
}
impl QueryFrame {
    pub(crate) fn new(
        size: (u32, u32),
        origin: [f64; 2],
        axes: [[f64; 2]; 2],
    ) -> Result<Self, crate::CoreError> {
        super::extended_visual::validate_sampled_source_size(size)?;
        if origin
            .iter()
            .chain(axes.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(invalid("matte query frame must be finite"));
        }
        for x in [0.5, f64::from(size.0) - 0.5] {
            for y in [0.5, f64::from(size.1) - 0.5] {
                let px = origin[0] + axes[0][0] * x + axes[1][0] * y;
                let py = origin[1] + axes[0][1] * x + axes[1][1] * y;
                if !px.is_finite() || !py.is_finite() {
                    return Err(invalid("matte query mapped extent must be finite"));
                }
            }
        }
        Ok(Self { size, origin, axes })
    }
    pub(crate) fn matrix(self) -> [f64; 6] {
        [
            self.axes[0][0],
            self.axes[0][1],
            self.axes[1][0],
            self.axes[1][1],
            self.origin[0],
            self.origin[1],
        ]
    }
}
#[derive(Clone, Copy)]
pub(crate) struct QueryScope {
    pub frame: QueryFrame,
    pub owner: Option<usize>,
    pub local_origin: [f64; 2],
}
/// One output frame owns these counters across every local query and private
/// provider product; separate domain memo tables never reset these bounds.
pub(crate) struct OutputFrameBudget {
    pub sampled: super::extended_visual::SampledFrameBudget,
    pub retained_live_bytes: u64,
    requests: u64,
    work: u64,
    leaf_pixels: u64,
    blend_work: u64,
}
impl OutputFrameBudget {
    pub(crate) fn new(scene: &super::EvaluatedScene) -> Result<Self, crate::CoreError> {
        Ok(Self {
            sampled: super::extended_visual::SampledFrameBudget::new(scene)?,
            retained_live_bytes: 0,
            requests: 0,
            work: 0,
            leaf_pixels: 0,
            blend_work: 0,
        })
    }
}

fn invalid(message: &str) -> crate::CoreError {
    crate::CoreError::new(crate::ErrorCode::InvalidArgument, message)
}
/// Bind one actual composition occurrence before any copies are materialized.
/// Hidden/inactive authored leaves remain nodes with an empty member group.
pub(crate) fn bind_scope(
    graph: &mut EvaluatedMatteGraph,
    tracks: &[crate::Track],
) -> Result<std::collections::HashMap<String, MatteGroupId>, crate::CoreError> {
    let composition = CompositionOccurrenceId(graph.groups.len());
    let mut ids = std::collections::HashMap::new();
    if !tracks
        .iter()
        .flat_map(|t| &t.items)
        .any(|item| item.visual_properties().matte.is_some() || item.visual_properties().matte_only)
    {
        return Ok(ids);
    }
    let participants = tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter_map(|item| {
            item.visual_properties()
                .matte
                .as_ref()
                .map(|m| m.source_id.as_str())
        })
        .collect::<std::collections::HashSet<_>>();
    for item in tracks.iter().flat_map(|t| &t.items) {
        if item.visual_properties().matte.is_none()
            && !item.visual_properties().matte_only
            && !participants.contains(item.id())
        {
            continue;
        }
        admit_graph_group(graph, item.id().len())?;
        if !crate::validation::matte::eligible(item, None) {
            continue;
        }
        let group = MatteGroupId(graph.groups.len());
        graph.groups.push(MatteProviderGroup {
            composition,
            authored_item_id: item.id().to_owned(),
            members: Vec::new(),
            matte_only: item.visual_properties().matte_only,
            provider: None,
        });
        ids.insert(item.id().to_owned(), group);
    }
    for item in tracks.iter().flat_map(|t| &t.items) {
        if let Some(reference) = &item.visual_properties().matte {
            let group = *ids
                .get(item.id())
                .ok_or_else(|| invalid("matte owner is not an eligible leaf"))?;
            let provider = *ids.get(&reference.source_id).ok_or_else(|| {
                crate::CoreError::new(
                    crate::ErrorCode::ItemNotFound,
                    "matte provider is absent from its composition",
                )
            })?;
            graph.groups[group.0].provider = Some((provider, reference.channel));
        }
    }
    Ok(ids)
}
pub(crate) fn provider_order(graph: &mut EvaluatedMatteGraph) -> Result<(), crate::CoreError> {
    let mut states = vec![0u8; graph.groups.len()];
    let mut path = Vec::new();
    graph.provider_first.clear();
    for start in 0..graph.groups.len() {
        if states[start] != 0 {
            continue;
        }
        path.clear();
        let mut current = Some(start);
        while let Some(index) = current {
            if states[index] == 1 {
                return Err(invalid("derived matte provider cycle"));
            }
            if states[index] == 2 {
                break;
            }
            states[index] = 1;
            path.push(index);
            current = graph.groups[index].provider.map(|(p, _)| p.0);
        }
        while let Some(index) = path.pop() {
            states[index] = 2;
            graph.provider_first.push(MatteGroupId(index));
        }
    }
    Ok(())
}

pub(crate) fn clone_composition(
    graph: &mut EvaluatedMatteGraph,
    original: CompositionOccurrenceId,
) -> Result<std::collections::HashMap<MatteGroupId, MatteGroupId>, crate::CoreError> {
    let composition = CompositionOccurrenceId(graph.groups.len());
    let additions = graph
        .groups
        .iter()
        .filter(|g| g.composition == original)
        .count();
    let prospective = add(graph.groups.len() as u64, additions as u64)?;
    if add(MATTE_CACHE_RESERVATION, mul(prospective, 1024)?)? > MAX_MATTE_LIVE_BYTES {
        return Err(invalid(
            "matte occurrence bindings exceed shared memory limits",
        ));
    }
    let originals = graph
        .groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.composition == original)
        .map(|(i, g)| (MatteGroupId(i), g.clone()))
        .collect::<Vec<_>>();
    let mut remap = std::collections::HashMap::new();
    for (id, mut group) in originals {
        remap.insert(id, MatteGroupId(graph.groups.len()));
        group.composition = composition;
        group.members.clear();
        graph.groups.push(group);
    }
    for new in remap.values() {
        if let Some((provider, channel)) = graph.groups[new.0].provider {
            graph.groups[new.0].provider = Some((
                *remap
                    .get(&provider)
                    .ok_or_else(|| invalid("provider escaped composition occurrence"))?,
                channel,
            ));
        }
    }
    Ok(remap)
}

fn admit_graph_group(graph: &EvaluatedMatteGraph, id_bytes: usize) -> Result<(), crate::CoreError> {
    // Each new group can coexist with old/new group Vec buffers, its owning-ID
    // lookup and the immutable expansion projection. IDs obey model limits;
    // all final actual capacities are also charged by graph_heap_bytes.
    let nodes = add(graph.groups.len() as u64, 1)?;
    let bound = add(
        MATTE_CACHE_RESERVATION,
        mul(nodes, add(1024, id_bytes as u64)?)?,
    )?;
    if bound > MAX_MATTE_LIVE_BYTES {
        return Err(invalid(
            "matte occurrence bindings exceed shared memory limits",
        ));
    }
    Ok(())
}
pub(crate) fn graph_heap_bytes(graph: &EvaluatedMatteGraph) -> Result<u64, crate::CoreError> {
    let mut bytes = std::mem::size_of::<EvaluatedMatteGraph>() as u64;
    bytes = add(bytes, capacity_bytes(&graph.groups)?)?;
    bytes = add(bytes, capacity_bytes(&graph.roles)?)?;
    bytes = add(bytes, capacity_bytes(&graph.provider_first)?)?;
    for group in &graph.groups {
        bytes = add(bytes, group.authored_item_id.capacity() as u64)?;
        bytes = add(bytes, capacity_bytes(&group.members)?)?;
    }
    Ok(bytes)
}

/// Exact immutable memo schedule. Each request is admitted before expanding its
/// members or recursively enumerating their shutter requests.
/// Canonical integer shutter map. The exact MotionBlur owner supplies offsets;
/// no floating-point timestamp conversion enters composed request identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RequestClock {
    offset: i128,
    low: u64,
    high: u64,
}
impl RequestClock {
    fn at(self, root: u64) -> u64 {
        (i128::from(root) + self.offset).clamp(i128::from(self.low), i128::from(self.high)) as u64
    }
    fn shifted(self, delta: i128, end: u64) -> Result<Self, crate::CoreError> {
        let offset = self
            .offset
            .checked_add(delta)
            .ok_or_else(|| invalid("continuous matte clock overflow"))?;
        let clamp = |value: i128| value.clamp(0, i128::from(end)) as u64;
        let mut clock = Self {
            offset,
            low: clamp(i128::from(self.low) + delta),
            high: clamp(i128::from(self.high) + delta),
        };
        clock.low = clock.at(0);
        clock.high = clock.at(end);
        if clock.low == clock.high {
            clock.offset = 0;
        }
        Ok(clock)
    }
}
pub(super) fn shutter_offsets(
    layer: &super::EvaluatedVisualLayer,
) -> Result<Vec<i128>, crate::CoreError> {
    layer
        .extended
        .as_ref()
        .and_then(|e| e.motion_blur)
        .map_or_else(
            || Ok(vec![0]),
            |blur| {
                blur.sample_times(1000, layer.extended.as_ref().unwrap().frame_rate, 3000)
                    .map(|times| times.into_iter().map(|t| i128::from(t) - 1000).collect())
            },
        )
}

// Find the exact integer transitions of the existing visibility owner. In
// instance contexts binary64 conversion can identify adjacent large integers;
// binary search preserves that established predicate instead of rounding a
// fractional root bound through an unrelated local span.
fn integer_visibility(layer: &super::EvaluatedVisualLayer) -> (u64, u64) {
    if let Some(instance) = layer.instance {
        let first = |bound: f64| {
            let (mut low, mut high) = (0_u64, u64::MAX);
            while low < high {
                let middle = low + (high - low) / 2;
                if (middle as f64) >= bound {
                    high = middle;
                } else {
                    low = middle + 1;
                }
            }
            low
        };
        (first(instance.start_ms), first(instance.end_ms))
    } else {
        let span = layer.visible_span();
        (span.start_ms, span.end_ms)
    }
}

pub(super) fn admit_continuous_metadata(
    scene: &super::EvaluatedScene,
) -> Result<u64, crate::CoreError> {
    let pixels = mul(
        u64::from(scene.canvas.width),
        u64::from(scene.canvas.height),
    )?;
    let base = add(
        add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?,
        composition_heap_bytes(scene)?,
    )?;
    let base = add(base, mul(scene.visual_layers.len() as u64, 48)?)?;
    if base > MAX_MATTE_LIVE_BYTES {
        return Err(invalid("continuous matte counters exceed shared memory"));
    }
    Ok(base)
}

/// Visit every integer-clock collision region. Geometry remains certified by
/// the existing full source envelopes; only exact uncached request multiplicity
/// is established here. All expansion/partition work shares candidate nodes.
pub(super) fn certify_continuous_requests(
    scene: &super::EvaluatedScene,
    nodes: &mut usize,
    mut verify: impl FnMut(&[u64]) -> Result<(), crate::CoreError>,
) -> Result<(), crate::CoreError> {
    certify_continuous_queries(
        scene,
        nodes,
        &[ContinuousQuery {
            owner: None,
            size: (scene.canvas.width, scene.canvas.height),
        }],
        false,
        |visits, _| verify(visits),
    )
}
#[derive(Clone, Copy, Debug)]
pub(super) struct ContinuousQuery {
    pub owner: Option<usize>,
    pub size: (u32, u32),
}
/// Conservative actual-capacity envelope for all query schedules and the
/// largest overlapping task payload. The existing 8192-byte growth reservation
/// includes Vec/hash slots, old/new reallocations and <=16 shutter tasks/copy.
#[derive(Clone, Copy, Debug)]
pub(super) struct ContinuousMemory {
    pub descriptors: u64,
    pub query_peak: u64,
}
fn continuous_memory(
    queries: &[ContinuousQuery],
    copies: impl Fn(usize) -> u64,
    providers: impl Fn(usize) -> u64,
) -> Result<ContinuousMemory, crate::CoreError> {
    let mut result = ContinuousMemory {
        descriptors: 0,
        query_peak: 0,
    };
    for (query, domain) in queries.iter().enumerate() {
        let copies = copies(query);
        let providers = providers(query);
        let records = add(copies, providers)?;
        result.descriptors = add(result.descriptors, mul(add(records, 1)?, 8192)?)?;
        let tasks = add(mul(copies, 17)?, providers)?;
        let payload = mul(query_pixels(*domain)?, add(mul(tasks, 16)?, 56)?)?;
        result.query_peak = result.query_peak.max(payload);
    }
    Ok(result)
}
pub(super) fn certify_continuous_queries(
    scene: &super::EvaluatedScene,
    nodes: &mut usize,
    queries: &[ContinuousQuery],
    controlled: bool,
    mut verify: impl FnMut(&[u64], ContinuousMemory) -> Result<(), crate::CoreError>,
) -> Result<(), crate::CoreError> {
    use std::collections::HashSet;
    let graph = scene.mattes.as_ref();
    let end = scene.duration_ms.saturating_sub(1);
    let base = admit_continuous_metadata(scene)?;
    struct Classes<'a> {
        scene: &'a super::EvaluatedScene,
        graph: Option<&'a EvaluatedMatteGraph>,
        queries: &'a [ContinuousQuery],
        end: u64,
        base: u64,
        copies: HashSet<(usize, RequestClock, usize, Option<usize>)>,
        providers: HashSet<(MatteGroupId, RequestClock, usize)>,
        upper_visits: Vec<u64>,
        upper_source: u64,
        upper_work: u64,
    }
    impl Classes<'_> {
        fn admit(&self, nodes: &mut usize) -> Result<(), crate::CoreError> {
            super::extended_certification::charge(nodes)?;
            // Geometric set growth, old/new realloc overlap, partition points,
            // exact-frame sets and visit counters are admitted before insertion.
            let records = add(self.copies.len() as u64, self.providers.len() as u64)?;
            if add(self.base, mul(add(records, 1)?, 8192)?)? > MAX_MATTE_LIVE_BYTES {
                return Err(invalid("continuous matte descriptors exceed shared memory"));
            }
            Ok(())
        }
        fn provider(
            &mut self,
            group: MatteGroupId,
            clock: RequestClock,
            query: usize,
            nodes: &mut usize,
        ) -> Result<(), crate::CoreError> {
            if self.providers.contains(&(group, clock, query)) {
                return Ok(());
            }
            self.admit(nodes)?;
            self.providers.insert((group, clock, query));
            let pixels = query_pixels(self.queries[query])?;
            self.upper_work = add(
                self.upper_work,
                mul(
                    mul(pixels, 4)?,
                    self.graph
                        .ok_or_else(|| invalid("provider graph absent"))?
                        .groups[group.0]
                        .members
                        .len() as u64,
                )?,
            )?;
            for slot in 0..self
                .graph
                .ok_or_else(|| invalid("provider graph absent"))?
                .groups[group.0]
                .members
                .len()
            {
                self.copy(
                    self.graph
                        .ok_or_else(|| invalid("provider graph absent"))?
                        .groups[group.0]
                        .members[slot],
                    clock,
                    query,
                    None,
                    nodes,
                )?;
            }
            Ok(())
        }
        fn copy(
            &mut self,
            index: usize,
            clock: RequestClock,
            query: usize,
            relative: Option<usize>,
            nodes: &mut usize,
        ) -> Result<(), crate::CoreError> {
            if self.copies.contains(&(index, clock, query, relative)) {
                return Ok(());
            }
            self.admit(nodes)?;
            self.copies.insert((index, clock, query, relative));
            let layer = &self.scene.visual_layers[index];
            let pixels = query_pixels(self.queries[query])?;
            let (visible_start, visible_end) = continuous_visibility(self.scene, index, relative)?;
            for delta in shutter_offsets(layer)? {
                self.upper_source = add(self.upper_source, pixels)?;
                let sample = clock.shifted(delta, self.end)?;
                // Hidden/zero-span leaves are still resource-admitted elsewhere.
                if sample.high < visible_start || sample.low >= visible_end {
                    continue;
                }
                self.upper_visits[index] = add(self.upper_visits[index], 1)?;
                if self
                    .graph
                    .and_then(|graph| graph.roles[index].provider)
                    .is_some()
                {
                    self.upper_work = add(self.upper_work, mul(pixels, 5)?)?;
                }
                if let Some((provider, _)) =
                    self.graph.and_then(|graph| graph.roles[index].provider)
                {
                    self.provider(provider, sample, query, nodes)?;
                }
            }
            Ok(())
        }
    }
    let root_clock = RequestClock {
        offset: 0,
        low: 0,
        high: end,
    };
    let mut classes = Classes {
        scene,
        graph,
        queries,
        end,
        base,
        copies: HashSet::new(),
        providers: HashSet::new(),
        upper_visits: vec![0; scene.visual_layers.len()],
        upper_source: 0,
        upper_work: 0,
    };
    for (query, domain) in queries.iter().enumerate() {
        for (index, layer) in scene.visual_layers.iter().enumerate() {
            if (!controlled || super::group_compositing::leaf_owner(layer) == domain.owner)
                && graph.is_none_or(|graph| {
                    graph.roles[index].contributes && !graph.roles[index].matte_only
                })
            {
                classes.copy(
                    index,
                    root_clock,
                    query,
                    if controlled { domain.owner } else { None },
                    nodes,
                )?;
            }
        }
    }
    // Function identity is a conservative upper bound. It is sufficient when
    // every shared owner admits it; otherwise resolve actual integer collisions.
    if classes.providers.len() as u64 <= MAX_MATTE_REQUESTS
        && classes.upper_source <= 268_435_456
        && classes.upper_work <= MAX_MATTE_WORK
        && verify(
            &classes.upper_visits,
            continuous_memory(
                queries,
                |query| {
                    classes
                        .copies
                        .iter()
                        .filter(|(_, _, q, _)| *q == query)
                        .count() as u64
                },
                |query| {
                    classes
                        .providers
                        .iter()
                        .filter(|(_, _, q)| *q == query)
                        .count() as u64
                },
            )?,
        )
        .is_ok()
    {
        return Ok(());
    }
    let record_bound = add(classes.copies.len() as u64, classes.providers.len() as u64)?;
    let reserved = add(base, mul(add(record_bound, 1)?, 8192)?)?;
    let mut points = Vec::new();
    let mut point = |value: i128| -> Result<(), crate::CoreError> {
        for adjacent in [value - 1, value, value + 1] {
            if (0..=i128::from(end)).contains(&adjacent) {
                let next = points
                    .len()
                    .checked_add(1)
                    .ok_or_else(|| invalid("partition size overflow"))?;
                let capacity = next
                    .checked_next_power_of_two()
                    .ok_or_else(|| invalid("partition capacity overflow"))?
                    .max(4);
                // Point reallocation old+new and simultaneously built ordered
                // representatives (at most twice as many) are pre-admitted.
                let growth = mul(capacity as u64, 48)?;
                if add(reserved, growth)? > MAX_MATTE_LIVE_BYTES {
                    return Err(invalid("continuous matte partition exceeds shared memory"));
                }
                points.push(adjacent as u64);
            }
        }
        Ok(())
    };
    point(0)?;
    point(i128::from(end))?;
    if controlled {
        for query in queries {
            if let Some(owner) = query.owner {
                let node = &scene.aggregates.as_ref().unwrap().nodes[owner];
                point(node.clock.start_ms.ceil() as i128)?;
                point(node.clock.end_ms.ceil() as i128)?;
            }
        }
    }
    for &(index, clock, _, relative) in &classes.copies {
        point(i128::from(clock.low) - clock.offset)?;
        point(i128::from(clock.high) - clock.offset)?;
        let layer = &scene.visual_layers[index];
        let (visible_start, visible_end) = continuous_visibility(scene, index, relative)?;
        for delta in shutter_offsets(layer)? {
            let sample = clock.shifted(delta, end)?;
            point(i128::from(visible_start) - sample.offset)?;
            point(i128::from(visible_end) - sample.offset)?;
            point(i128::from(sample.low) - sample.offset)?;
            point(i128::from(sample.high) - sample.offset)?;
        }
    }
    let mut identities: Vec<_> = classes
        .providers
        .iter()
        .map(|(g, c, query)| (true, g.0, *query, None, *c))
        .chain(
            classes
                .copies
                .iter()
                .map(|(i, c, query, relative)| (false, *i, *query, *relative, *c)),
        )
        .collect();
    identities.sort_unstable();
    for group in identities.chunk_by(|a, b| a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3) {
        for (_, _, _, _, first) in group {
            for (_, _, _, _, second) in group {
                if first == second {
                    continue;
                }
                super::extended_certification::charge(nodes)?;
                // Distinct linear pieces cannot cross. A collision can change
                // only at a plateau boundary or a linear/constant equality.
                point(i128::from(second.low) - first.offset)?;
                point(i128::from(second.high) - first.offset)?;
            }
        }
    }
    points.sort_unstable();
    points.dedup();
    // Endpoints plus one representative of each open integer region suffice:
    // visibility and all exact-key equality relations are constant there.
    let mut representatives = Vec::new();
    for pair in points.windows(2) {
        representatives.push(pair[0]);
        if pair[1] - pair[0] > 1 {
            representatives.push(pair[0] + 1);
        }
    }
    representatives.push(end);
    struct Frame<'a> {
        scene: &'a super::EvaluatedScene,
        graph: Option<&'a EvaluatedMatteGraph>,
        queries: &'a [ContinuousQuery],
        copies: HashSet<(usize, u64, usize, Option<usize>)>,
        providers: HashSet<(MatteGroupId, u64, usize)>,
        visits: Vec<u64>,
        leaf_work: u64,
        work: u64,
        nodes: &'a mut usize,
    }
    impl Frame<'_> {
        fn provider(
            &mut self,
            group: MatteGroupId,
            time: u64,
            query: usize,
        ) -> Result<(), crate::CoreError> {
            if self.providers.contains(&(group, time, query)) {
                return Ok(());
            }
            super::extended_certification::charge(self.nodes)?;
            self.providers.insert((group, time, query));
            if self.providers.len() as u64 > MAX_MATTE_REQUESTS {
                return Err(invalid("continuous matte provider requests exceed bounds"));
            }
            let pixels = query_pixels(self.queries[query])?;
            self.work = add(
                self.work,
                mul(
                    mul(pixels, 4)?,
                    self.graph
                        .ok_or_else(|| invalid("provider graph absent"))?
                        .groups[group.0]
                        .members
                        .len() as u64,
                )?,
            )?;
            if self.work > MAX_MATTE_WORK {
                return Err(invalid("continuous matte work exceeds bounds"));
            }
            for slot in 0..self
                .graph
                .ok_or_else(|| invalid("provider graph absent"))?
                .groups[group.0]
                .members
                .len()
            {
                self.copy(
                    self.graph
                        .ok_or_else(|| invalid("provider graph absent"))?
                        .groups[group.0]
                        .members[slot],
                    time,
                    query,
                    None,
                )?;
            }
            Ok(())
        }
        fn copy(
            &mut self,
            index: usize,
            time: u64,
            query: usize,
            relative: Option<usize>,
        ) -> Result<(), crate::CoreError> {
            if self.copies.contains(&(index, time, query, relative)) {
                return Ok(());
            }
            super::extended_certification::charge(self.nodes)?;
            self.copies.insert((index, time, query, relative));
            let layer = &self.scene.visual_layers[index];
            let times = layer
                .extended
                .as_ref()
                .and_then(|e| e.motion_blur)
                .map_or_else(
                    || Ok(vec![time]),
                    |blur| {
                        blur.sample_times(
                            time,
                            layer.extended.as_ref().unwrap().frame_rate,
                            self.scene.duration_ms,
                        )
                    },
                )?;
            let pixels = query_pixels(self.queries[query])?;
            self.leaf_work = add(self.leaf_work, mul(pixels, times.len() as u64)?)?;
            if self.leaf_work > 268_435_456 {
                return Err(invalid("continuous matte source visits exceed bounds"));
            }
            for time in times {
                if !relative.map_or_else(
                    || Ok(layer.visible_at(time)),
                    |owner| {
                        super::group_compositing::relative_visible_at(
                            self.scene, index, owner, time,
                        )
                    },
                )? {
                    continue;
                }
                self.visits[index] = add(self.visits[index], 1)?;
                if let Some((provider, _)) =
                    self.graph.and_then(|graph| graph.roles[index].provider)
                {
                    self.work = add(self.work, mul(pixels, 5)?)?;
                    if self.work > MAX_MATTE_WORK {
                        return Err(invalid("continuous matte work exceeds bounds"));
                    }
                    self.provider(provider, time, query)?;
                }
            }
            Ok(())
        }
    }
    let record_bound = add(classes.copies.len() as u64, classes.providers.len() as u64)?;
    let partition_bytes = add(
        add(capacity_bytes(&points)?, capacity_bytes(&representatives)?)?,
        capacity_bytes(&identities)?,
    )?;
    let profile_bytes = mul(scene.visual_layers.len() as u64, 32)?;
    if add(
        base,
        add(
            mul(record_bound, 8192)?,
            add(partition_bytes, profile_bytes)?,
        )?,
    )? > MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("continuous matte partition exceeds shared memory"));
    }
    for root in representatives {
        super::extended_certification::charge(nodes)?;
        let mut frame = Frame {
            scene,
            graph,
            queries,
            copies: HashSet::new(),
            providers: HashSet::new(),
            visits: vec![0; scene.visual_layers.len()],
            leaf_work: 0,
            work: 0,
            nodes,
        };
        for (query, domain) in queries.iter().enumerate() {
            if controlled
                && domain.owner.is_some_and(|owner| {
                    !scene.aggregates.as_ref().unwrap().nodes[owner].visible_at(root)
                })
            {
                continue;
            }
            for (index, layer) in scene.visual_layers.iter().enumerate() {
                if (!controlled || super::group_compositing::leaf_owner(layer) == domain.owner)
                    && graph.is_none_or(|graph| {
                        graph.roles[index].contributes && !graph.roles[index].matte_only
                    })
                {
                    frame.copy(
                        index,
                        root,
                        query,
                        if controlled { domain.owner } else { None },
                    )?;
                }
            }
        }
        verify(
            &frame.visits,
            continuous_memory(
                queries,
                |query| {
                    frame
                        .copies
                        .iter()
                        .filter(|(_, _, q, _)| *q == query)
                        .count() as u64
                },
                |query| {
                    frame
                        .providers
                        .iter()
                        .filter(|(_, _, q)| *q == query)
                        .count() as u64
                },
            )?,
        )?;
    }
    Ok(())
}

fn query_pixels(query: ContinuousQuery) -> Result<u64, crate::CoreError> {
    super::extended_visual::validate_sampled_source_size(query.size)?;
    mul(u64::from(query.size.0), u64::from(query.size.1))
}
fn continuous_visibility(
    scene: &super::EvaluatedScene,
    index: usize,
    relative: Option<usize>,
) -> Result<(u64, u64), crate::CoreError> {
    let Some(owner) = relative else {
        return Ok(integer_visibility(&scene.visual_layers[index]));
    };
    super::group_compositing::relative_integer_visibility(scene, index, owner)
}

#[cfg(test)]
std::thread_local! { static METADATA_RESERVATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

pub(crate) fn frame_schedule(
    scene: &super::EvaluatedScene,
    at: u64,
) -> Result<MatteFrameSchedule, crate::CoreError> {
    let mut budget = OutputFrameBudget::new(scene)?;
    frame_schedule_in_domain(scene, at, None, &mut budget)
}
pub(crate) fn frame_schedule_in_domain(
    scene: &super::EvaluatedScene,
    at: u64,
    query: Option<QueryScope>,
    budget: &mut OutputFrameBudget,
) -> Result<MatteFrameSchedule, crate::CoreError> {
    struct Builder<'a> {
        scene: &'a super::EvaluatedScene,
        query: Option<QueryScope>,
        graph: Option<&'a EvaluatedMatteGraph>,
        tasks: Vec<MatteTask>,
        copies: std::collections::HashMap<(usize, u64, Option<usize>), MatteTaskId>,
        providers: std::collections::HashMap<(MatteGroupId, u64), MatteTaskId>,
        requests: u64,
        work: u64,
        leaf_pixels: u64,
        prior_requests: u64,
        prior_work: u64,
        prior_leaf_pixels: u64,
        pixels: u64,
        planning_fixed: u64,
        sampled_budget: &'a mut super::extended_visual::SampledFrameBudget,
    }
    impl Builder<'_> {
        fn admit_records(&self, records: u64, transient: u64) -> Result<(), crate::CoreError> {
            // Includes both memo tables, shared sampled-owner sets/maps, nested
            // ID vectors, geometric descriptor growth and transient reallocations.
            let per_record = add(mul(std::mem::size_of::<MatteTask>() as u64, 4)?, 512)?;
            if add(
                add(self.planning_fixed, mul(records, per_record)?)?,
                transient,
            )? > MAX_MATTE_LIVE_BYTES
            {
                return Err(invalid(
                    "matte planning descriptors exceed shared memory bounds",
                ));
            }
            Ok(())
        }
        fn push(&mut self, task: MatteTask) -> Result<MatteTaskId, crate::CoreError> {
            // Bounds on actual source visits plus requests constrain allocation
            // before constructing a potentially exponential nested shutter tree.
            let records = (self.tasks.len() as u64)
                .checked_add(1)
                .ok_or_else(|| invalid("matte task cardinality overflow"))?;
            self.admit_records(records, 0)?;
            let id = MatteTaskId(self.tasks.len());
            self.tasks.push(task);
            Ok(id)
        }
        fn provider(
            &mut self,
            group: MatteGroupId,
            at: u64,
            depth: usize,
        ) -> Result<MatteTaskId, crate::CoreError> {
            if let Some(id) = self.providers.get(&(group, at)) {
                return Ok(*id);
            }
            if depth > 32 {
                return Err(invalid("matte provider depth exceeds bounds"));
            }
            self.requests = add(self.requests, 1)?;
            if add(self.prior_requests, self.requests)? > MAX_MATTE_REQUESTS {
                return Err(invalid(
                    "matte provider requests exceed output-frame bounds",
                ));
            }
            let member_count = self
                .graph
                .ok_or_else(|| invalid("provider graph absent"))?
                .groups[group.0]
                .members
                .len();
            self.work = add(self.work, mul(mul(self.pixels, 4)?, member_count as u64)?)?;
            if add(self.prior_work, self.work)? > MAX_MATTE_WORK {
                return Err(invalid("matte work exceeds output-frame bounds"));
            }
            self.admit_records(add(self.tasks.len() as u64, member_count as u64)?, 0)?;
            let mut copies = Vec::new();
            for offset in 0..member_count {
                let member = self
                    .graph
                    .ok_or_else(|| invalid("provider graph absent"))?
                    .groups[group.0]
                    .members[offset];
                copies.push(self.copy(member, at, depth, None)?);
            }
            let id = self.push(MatteTask::AggregateProvider { copies })?;
            self.providers.insert((group, at), id);
            Ok(id)
        }
        fn copy(
            &mut self,
            index: usize,
            at: u64,
            depth: usize,
            relative_owner: Option<usize>,
        ) -> Result<MatteTaskId, crate::CoreError> {
            if let Some(id) = self.copies.get(&(index, at, relative_owner)) {
                return Ok(*id);
            }
            let layer = &self.scene.visual_layers[index];
            let times = layer
                .extended
                .as_ref()
                .and_then(|e| e.motion_blur)
                .map_or_else(
                    || Ok(vec![at]),
                    |blur| {
                        blur.sample_times(
                            at,
                            layer.extended.as_ref().unwrap().frame_rate,
                            self.scene.duration_ms,
                        )
                    },
                )?;
            // Pre-admit all visits before expanding transitive requests.
            self.leaf_pixels = add(self.leaf_pixels, mul(self.pixels, times.len() as u64)?)?;
            if add(self.prior_leaf_pixels, self.leaf_pixels)? > MAX_MATTE_WORK {
                return Err(invalid(
                    "linear composition output-frame pixel work exceeds limits",
                ));
            }
            let provider = self.graph.and_then(|graph| graph.roles[index].provider);

            let mut samples = Vec::new();
            for time in times {
                let visible = if let Some(owner) = relative_owner {
                    super::group_compositing::relative_visible_at(self.scene, index, owner, time)?
                } else {
                    layer.visible_at(time)
                };
                if !visible {
                    samples.push(self.push(MatteTask::LeafSample {
                        relative_owner,
                        layer_index: index,
                        at_ms: time,
                        provider: None,
                        source_live_bytes: 0,
                    })?);
                    continue;
                }
                if provider.is_some() {
                    self.work = add(self.work, mul(self.pixels, 5)?)?;
                }
                if add(self.prior_work, self.work)? > MAX_MATTE_WORK {
                    return Err(invalid("matte work exceeds output-frame bounds"));
                }
                let provider = provider
                    .map(|(group, channel)| {
                        self.provider(group, time, depth + 1)
                            .map(|id| (id, channel))
                    })
                    .transpose()?;
                self.admit_records(
                    add(self.tasks.len() as u64, 1)?,
                    mul(layer_heap_bytes(layer)?, 3)?,
                )?;
                let (base_memory, mask_memory) = self.sampled_budget.certify_in_domain(
                    self.scene,
                    index,
                    time,
                    self.query.map_or(
                        (self.scene.canvas.width, self.scene.canvas.height),
                        |query| query.frame.size,
                    ),
                    relative_owner,
                    self.query.map(|query| {
                        if relative_owner.is_some() {
                            [1., 0., 0., 1., query.local_origin[0], query.local_origin[1]]
                        } else {
                            query.frame.matrix()
                        }
                    }),
                )?;
                let mut source_live_bytes = base_memory
                    .checked_sub(add(MATTE_CACHE_RESERVATION, mul(self.pixels, 40)?)?)
                    .ok_or_else(|| invalid("matte source memory certification invalid"))?;
                source_live_bytes = add(source_live_bytes, mul(layer_heap_bytes(layer)?, 3)?)?;
                source_live_bytes = add(source_live_bytes, mask_memory)?;
                samples.push(self.push(MatteTask::LeafSample {
                    relative_owner,
                    layer_index: index,
                    at_ms: time,
                    provider,
                    source_live_bytes,
                })?);
            }
            let id = if samples.len() == 1 {
                samples[0]
            } else {
                self.push(MatteTask::AverageCopy {
                    layer_index: index,
                    samples,
                })?
            };
            self.copies.insert((index, at, relative_owner), id);
            Ok(id)
        }
    }
    let graph = scene.mattes.as_ref();
    if graph.is_some_and(|graph| graph.roles.len() != scene.visual_layers.len()) {
        return Err(invalid("matte role count does not match scene"));
    }
    if scene.composition_resources.is_none() {
        return Err(invalid("bounded composition resource facts absent"));
    }
    let canvas = query.map_or((scene.canvas.width, scene.canvas.height), |query| {
        query.frame.size
    });
    super::extended_visual::validate_sampled_source_size(canvas)?;
    let pixels = mul(u64::from(canvas.0), u64::from(canvas.1))?;
    let planning_fixed = add(
        add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?,
        add(composition_heap_bytes(scene)?, budget.retained_live_bytes)?,
    )?;
    // Admit the complete owning table and immutable direct metadata BEFORE
    // allocating either vector. Task admission also retains this reservation.
    let count = scene.visual_layers.len();
    let direct_count = (0..count)
        .filter(|&index| {
            query.is_none_or(|query| {
                super::group_compositing::leaf_owner(&scene.visual_layers[index]) == query.owner
            }) && graph.is_none_or(|graph| {
                graph.roles[index].contributes && !graph.roles[index].matte_only
            })
        })
        .count();
    let initial_descriptor = add(
        add(
            std::mem::size_of::<MatteFrameSchedule>() as u64,
            mul(count as u64, std::mem::size_of::<crate::BlendMode>() as u64)?,
        )?,
        mul(
            direct_count as u64,
            std::mem::size_of::<DirectDraw>() as u64,
        )?,
    )?;
    let planning_headers = add(
        std::mem::size_of::<super::extended_visual::SampledFrameBudget>() as u64,
        std::mem::size_of::<Vec<()>>() as u64,
    )?;
    if add(add(planning_fixed, initial_descriptor)?, planning_headers)? > MAX_MATTE_LIVE_BYTES {
        return Err(invalid(
            "composition owning metadata exceeds shared memory bounds",
        ));
    }
    let mut owner_modes = Vec::new();
    #[cfg(test)]
    METADATA_RESERVATIONS.with(|counter| counter.set(counter.get() + usize::from(count > 0)));
    owner_modes
        .try_reserve_exact(count)
        .map_err(|_| invalid("composition owner allocation failed"))?;
    owner_modes.extend(scene.visual_layers.iter().map(|layer| layer.blend_mode));
    let mut direct_draw = Vec::new();
    direct_draw
        .try_reserve_exact(direct_count)
        .map_err(|_| invalid("composition direct allocation failed"))?;
    let initial_descriptor = add(
        std::mem::size_of::<MatteFrameSchedule>() as u64,
        add(capacity_bytes(&owner_modes)?, capacity_bytes(&direct_draw)?)?,
    )?;
    if add(add(planning_fixed, initial_descriptor)?, planning_headers)? > MAX_MATTE_LIVE_BYTES {
        return Err(invalid(
            "composition metadata allocator capacity exceeds bounds",
        ));
    }
    let mut builder = Builder {
        scene,
        query,
        graph,
        tasks: Vec::new(),
        copies: std::collections::HashMap::new(),
        providers: std::collections::HashMap::new(),
        requests: 0,
        work: 0,
        leaf_pixels: 0,
        prior_requests: budget.requests,
        prior_work: budget.work,
        prior_leaf_pixels: budget.leaf_pixels,
        pixels,
        planning_fixed: add(add(planning_fixed, initial_descriptor)?, planning_headers)?,
        sampled_budget: &mut budget.sampled,
    };
    let mut blend_work = 0;
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        if query.is_some_and(|query| super::group_compositing::leaf_owner(layer) != query.owner)
            || graph.is_some_and(|graph| {
                !graph.roles[index].contributes || graph.roles[index].matte_only
            })
        {
            continue;
        }
        let task = builder.copy(index, at, 0, query.and_then(|query| query.owner))?;
        let destination_visits = if query.is_some() {
            pixels
        } else {
            match &builder.tasks[task.0] {
                MatteTask::AverageCopy { samples, .. } => {
                    let mut bounds: Option<[u32; 4]> = None;
                    for sample in samples {
                        let MatteTask::LeafSample { at_ms, .. } = builder.tasks[sample.0] else {
                            return Err(invalid("owning average sample is not a leaf"));
                        };
                        if !layer.visible_at(at_ms) {
                            continue;
                        }
                        let next = super::extended_visual::certified_destination_bounds(
                            layer,
                            at_ms,
                            (scene.canvas.width, scene.canvas.height),
                        )?;
                        if next[0] == next[2] || next[1] == next[3] {
                            continue;
                        }
                        bounds = Some(bounds.map_or(next, |old| {
                            [
                                old[0].min(next[0]),
                                old[1].min(next[1]),
                                old[2].max(next[2]),
                                old[3].max(next[3]),
                            ]
                        }));
                    }
                    bounds.map_or(0, |[left, top, right, bottom]| {
                        u64::from(right - left) * u64::from(bottom - top)
                    })
                }
                MatteTask::LeafSample { at_ms, .. } if layer.visible_at(*at_ms) => {
                    super::extended_visual::certified_destination_visits(
                        layer,
                        *at_ms,
                        (scene.canvas.width, scene.canvas.height),
                    )?
                }
                _ => 0,
            }
        };
        if !layer.blend_mode.is_normal() {
            blend_work = add(blend_work, mul(destination_visits, 32)?)?;
            if add(budget.blend_work, blend_work)? > MAX_MATTE_WORK {
                return Err(invalid("blend work exceeds output-frame bounds"));
            }
        }
        direct_draw.push(DirectDraw {
            task,
            layer_index: index,
            blend_mode: layer.blend_mode,
            destination_visits,
        });
    }
    builder.admit_records(
        builder.tasks.len() as u64,
        mul(
            builder.tasks.len() as u64,
            std::mem::size_of::<usize>() as u64,
        )?,
    )?;
    let mut last_uses = Vec::new();
    last_uses
        .try_reserve_exact(builder.tasks.len())
        .map_err(|_| invalid("composition last-use allocation failed"))?;
    last_uses.extend(0..builder.tasks.len());
    for (index, task) in builder.tasks.iter().enumerate() {
        match task {
            MatteTask::LeafSample {
                provider: Some((id, _)),
                ..
            } => last_uses[id.0] = last_uses[id.0].max(index),
            MatteTask::AverageCopy { samples, .. } => {
                for id in samples {
                    last_uses[id.0] = last_uses[id.0].max(index);
                }
            }
            MatteTask::AggregateProvider { copies } => {
                for id in copies {
                    last_uses[id.0] = last_uses[id.0].max(index);
                }
            }
            _ => {}
        }
    }
    for (index, draw) in direct_draw.iter().enumerate() {
        let id = draw.task;
        last_uses[id.0] = last_uses[id.0].max(builder.tasks.len() + index);
    }
    let bytes = |capacity: usize, size: usize| mul(capacity as u64, size as u64);
    let mut descriptor = std::mem::size_of::<MatteFrameSchedule>() as u64;
    descriptor = add(
        descriptor,
        bytes(builder.tasks.capacity(), std::mem::size_of::<MatteTask>())?,
    )?;
    descriptor = add(
        descriptor,
        bytes(last_uses.capacity(), std::mem::size_of::<usize>())?,
    )?;
    descriptor = add(
        descriptor,
        bytes(direct_draw.capacity(), std::mem::size_of::<DirectDraw>())?,
    )?;
    descriptor = add(
        descriptor,
        add(
            mul(builder.tasks.len() as u64, 128)?,
            std::mem::size_of::<Vec<()>>() as u64,
        )?,
    )?;
    for task in &builder.tasks {
        match task {
            MatteTask::AverageCopy { samples, .. } => {
                descriptor = add(
                    descriptor,
                    bytes(samples.capacity(), std::mem::size_of::<MatteTaskId>())?,
                )?
            }
            MatteTask::AggregateProvider { copies } => {
                descriptor = add(
                    descriptor,
                    bytes(copies.capacity(), std::mem::size_of::<MatteTaskId>())?,
                )?
            }
            _ => {}
        }
    }
    descriptor = add(
        descriptor,
        bytes(
            owner_modes.capacity(),
            std::mem::size_of::<crate::BlendMode>(),
        )?,
    )?;
    let facts = composition_heap_bytes(scene)?;
    let fixed = add(
        add(
            add(add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?, descriptor)?,
            facts,
        )?,
        budget.retained_live_bytes,
    )?;
    let release_count = builder
        .tasks
        .len()
        .checked_add(direct_draw.len())
        .ok_or_else(|| invalid("composition release cardinality overflow"))?;
    builder.admit_records(
        builder.tasks.len() as u64,
        add(
            capacity_bytes(&last_uses)?,
            mul(release_count as u64, std::mem::size_of::<u64>() as u64)?,
        )?,
    )?;
    let mut releases = Vec::new();
    releases
        .try_reserve_exact(release_count)
        .map_err(|_| invalid("composition release allocation failed"))?;
    releases.resize(release_count, 0u64);
    for &last in &last_uses {
        releases[last] = add(releases[last], 1)?;
    }
    let memo_bytes = add(
        mul(
            builder.copies.capacity() as u64,
            2 * (std::mem::size_of::<((usize, u64, Option<usize>), MatteTaskId)>() as u64 + 1),
        )?,
        mul(
            builder.providers.capacity() as u64,
            2 * (std::mem::size_of::<((MatteGroupId, u64), MatteTaskId)>() as u64 + 1),
        )?,
    )?;
    let planning_live = add(
        add(
            add(fixed, memo_bytes)?,
            builder.sampled_budget.heap_bytes()?,
        )?,
        bytes(releases.capacity(), std::mem::size_of::<u64>())?,
    )?;
    let mut live = fixed;
    let mut peak = planning_live;
    let plane = mul(pixels, 16)?;
    for (index, task) in builder.tasks.iter().enumerate() {
        let temporary = match task {
            MatteTask::LeafSample {
                source_live_bytes, ..
            } => *source_live_bytes,
            _ => plane,
        };
        peak = peak.max(add(live, temporary)?);
        live = add(live, plane)?;
        live = live
            .checked_sub(mul(releases[index], plane)?)
            .ok_or_else(|| invalid("matte lifetime arithmetic underflow"))?;
    }
    if peak > MAX_MATTE_LIVE_BYTES {
        return Err(invalid("matte shared live memory exceeds limits"));
    }
    budget.requests = add(budget.requests, builder.requests)?;
    budget.work = add(budget.work, builder.work)?;
    budget.leaf_pixels = add(budget.leaf_pixels, builder.leaf_pixels)?;
    budget.blend_work = add(budget.blend_work, blend_work)?;
    if budget.requests > MAX_MATTE_REQUESTS
        || budget.work > MAX_MATTE_WORK
        || budget.leaf_pixels > MAX_MATTE_WORK
        || budget.blend_work > MAX_MATTE_WORK
    {
        return Err(invalid(
            "cumulative query frame request/work exceeds bounds",
        ));
    }
    Ok(MatteFrameSchedule {
        query: query.map(|query| query.frame),
        canvas,
        tasks: builder.tasks,
        direct_draw,
        owner_modes,
        last_uses,
        certificate: MatteFrameCertificate {
            provider_requests: builder.requests,
            matte_work_units: builder.work,
            blend_work_units: blend_work,
            fixed_live_bytes: fixed,
            descriptor_bytes: descriptor,
            peak_live_bytes: peak,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::super::composition_resources::{
        admit_caller_scene_clone, adopt_font_payload, font_payload_admission, scene_heap_bytes,
    };
    use super::*;
    use crate::{ErrorCode, Project};
    use serde_json::json;
    fn project() -> Project {
        serde_json::from_value(json!({"schemaVersion":crate::PROJECT_SCHEMA_VERSION,"id":"p","revision":0,"name":"Matte facts","createdAtMs":1,"updatedAtMs":1,"settings":{"width":64,"height":64,"fps":10},"fonts":{},"markers":[],"assets":[],"components":[],"tracks":[{"id":"track","name":"Track","trackType":"overlay","items":[
            {"type":"rectangle","id":"provider","color":"#ffffff","width":4,"height":4,"startMs":0,"durationMs":1000,"matteOnly":true,"stackOrder":0,"zIndex":0,"keyframes":[]},
            {"type":"rectangle","id":"recipient","color":"#ff0000","width":4,"height":4,"startMs":0,"durationMs":1000,"matte":{"sourceId":"provider","channel":"alpha"},"stackOrder":1,"zIndex":0,"keyframes":[]}
        ]}]})).unwrap()
    }
    fn scene(project: &Project) -> super::super::EvaluatedScene {
        super::super::evaluate_project(
            project,
            project.settings.width,
            project.settings.height,
            project.settings.fps,
        )
        .unwrap()
        .scene
    }
    #[test]
    fn exact_nested_shutter_requests_use_each_recipient_time_and_provider_shutter_once() {
        let mut p = project();
        let mut middle = p.tracks[0].items[0].clone();
        if let crate::TimelineItem::Rectangle(item) = &mut middle {
            item.id = "middle".into();
        }
        middle.visual_properties_mut().matte = Some(crate::MatteReference {
            source_id: "provider".into(),
            channel: crate::MatteChannel::Alpha,
        });
        middle.visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 4,
        });
        middle.visual_properties_mut().stack_order = 1;
        p.tracks[0].items.insert(1, middle);
        let recipient = p.tracks[0].items[2].visual_properties_mut();
        recipient.stack_order = 2;
        recipient.matte.as_mut().unwrap().source_id = "middle".into();
        recipient.motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 4,
        });
        let scene = scene(&p);
        let schedule = frame_schedule(&scene, 400).unwrap();
        let mut root_times = Vec::new();
        let mut upper_times = Vec::new();
        let mut middle_times = Vec::new();
        for task in &schedule.tasks {
            if let MatteTask::LeafSample {
                layer_index, at_ms, ..
            } = task
            {
                match scene.visual_layers[*layer_index].item_id.as_str() {
                    "recipient" => root_times.push(*at_ms),
                    "middle" => middle_times.push(*at_ms),
                    "provider" => upper_times.push(*at_ms),
                    _ => panic!("unexpected layer"),
                }
            }
        }
        root_times.sort();
        upper_times.sort();
        middle_times.sort();
        assert_eq!(root_times, [381, 393, 406, 418]);
        assert_eq!(upper_times, [362, 374, 386, 387, 399, 411, 412, 424, 436]);
        assert_eq!(
            middle_times,
            [
                362, 374, 374, 386, 387, 387, 399, 399, 399, 399, 411, 411, 412, 424, 424, 436
            ]
        );
        assert_eq!(schedule.certificate.provider_requests, 13);
        assert_eq!(schedule.direct_draw.len(), 1);
        assert!(schedule.tasks.iter().enumerate().all(|(i, t)| match t {
            MatteTask::LeafSample {
                provider: Some((p, _)),
                ..
            } => p.0 < i,
            MatteTask::AverageCopy { samples, .. } => samples.iter().all(|s| s.0 < i),
            MatteTask::AggregateProvider { copies } => copies.iter().all(|s| s.0 < i),
            _ => true,
        }));
    }
    #[test]
    fn scoped_components_bind_same_local_ids_to_distinct_provider_groups_deterministically() {
        let mut p = project();
        let tracks = p.tracks.clone();
        p.components=serde_json::from_value(json!([{"id":"definition","name":"Definition","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[]}])).unwrap();
        p.tracks[0].items=serde_json::from_value(json!([
            {"type":"component_instance","id":"first","componentId":"definition","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{},"stackOrder":0,"zIndex":0},
            {"type":"component_instance","id":"second","componentId":"definition","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{},"stackOrder":1,"zIndex":0}
        ])).unwrap();
        let a = scene(&p);
        let b = scene(&p);
        assert_eq!(a, b);
        let graph = a.mattes.as_ref().unwrap();
        let providers = graph
            .groups
            .iter()
            .enumerate()
            .filter(|(_, g)| g.authored_item_id == "provider")
            .collect::<Vec<_>>();
        assert_eq!(providers.len(), 2);
        assert_ne!(providers[0].1.composition, providers[1].1.composition);
        for group in &graph.groups {
            if let Some((provider, _)) = group.provider {
                assert_eq!(graph.groups[provider.0].composition, group.composition);
            }
        }
        assert_eq!(
            frame_schedule(&a, 400)
                .unwrap()
                .certificate
                .provider_requests,
            2
        );
    }
    #[test]
    fn default_matte_metadata_bypasses_graph_and_empty_values_preserve_scene_exactly() {
        let mut p = project();
        for item in &mut p.tracks[0].items {
            item.visual_properties_mut().matte = None;
            item.visual_properties_mut().matte_only = false;
        }
        let first = scene(&p);
        assert!(first.mattes.is_none());
        let wire = serde_json::to_value(&p).unwrap();
        let mut explicit = wire;
        for item in explicit["tracks"][0]["items"].as_array_mut().unwrap() {
            item["matteOnly"] = json!(false);
        }
        let p: Project = serde_json::from_value(explicit).unwrap();
        assert_eq!(first, scene(&p));
    }
    #[test]
    fn basic_four_k_matte_certificate_accounts_callback_without_double_counting_caller_rasters() {
        let mut p = project();
        p.settings.width = 3840;
        p.settings.height = 2160;
        for item in &mut p.tracks[0].items {
            if let crate::TimelineItem::Rectangle(r) = item {
                r.width = 3840;
                r.height = 2160;
            }
        }
        let schedule = frame_schedule(&scene(&p), 400).unwrap();
        assert!(schedule.certificate.peak_live_bytes <= MAX_MATTE_LIVE_BYTES);
        assert!(schedule.certificate.peak_live_bytes > 900_000_000);
    }
    fn copies(count: usize) -> super::super::EvaluatedScene {
        let mut p = project();
        p.settings.width = 128;
        p.settings.height = 128;
        let mut scene = scene(&p);
        let recipient = scene
            .visual_layers
            .iter()
            .position(|l| l.item_id == "recipient")
            .unwrap();
        let original = scene.visual_layers[recipient].clone();
        let role = scene.mattes.as_ref().unwrap().roles[recipient].clone();
        for n in 1..count {
            let mut copy = original.clone();
            copy.item_id = format!("copy-{n}");
            scene.visual_layers.push(copy);
            scene.mattes.as_mut().unwrap().roles.push(role.clone());
        }
        scene
    }
    #[test]
    fn actual_main_frame_work_exact_boundary_and_excess_count_all_completed_copies() {
        let accepted = frame_schedule(&copies(3276), 400).unwrap();
        assert_eq!(accepted.certificate.matte_work_units, 268_435_456);
        assert_eq!(accepted.certificate.provider_requests, 1);
        assert_eq!(
            frame_schedule(&copies(3277), 400).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            mul(u64::MAX, 16).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            add(u64::MAX, 1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    #[test]
    fn actual_provider_tasks_preserve_large_integer_root_and_fractional_ping_pong_sampling() {
        let base = (1_u64 << 53) + 100;
        let mut p = project();
        for item in &mut p.tracks[0].items {
            if let crate::TimelineItem::Rectangle(r) = item {
                r.duration_ms = base + 1000;
            }
        }
        p.tracks[0].items[0]
            .visual_properties_mut()
            .animation_channels = serde_json::from_value(json!([
            {"property":"transform.opacity","keyframes":[
                {"timeMs":base,"value":{"type":"scalar","value":0.25},"curve":"hold"},
                {"timeMs":base+1,"value":{"type":"scalar","value":0.75},"curve":"hold"}]}]))
        .unwrap();
        let evaluated = scene(&p);
        for (root, expected) in [(base, 0.25), (base + 1, 0.75)] {
            let schedule = frame_schedule(&evaluated, root).unwrap();
            let (index, actual_time) = schedule
                .tasks
                .iter()
                .find_map(|task| match task {
                    MatteTask::LeafSample {
                        layer_index, at_ms, ..
                    } if evaluated.visual_layers[*layer_index].item_id == "provider" => {
                        Some((*layer_index, *at_ms))
                    }
                    _ => None,
                })
                .unwrap();
            assert_eq!(actual_time, root);
            let (mut sampled, _, _) =
                super::super::extended_visual::sample(&evaluated.visual_layers[index], actual_time)
                    .unwrap();
            let affine = super::super::extended_visual::sample_transform(
                &mut sampled,
                actual_time,
                (4, 4),
                (64, 64),
            )
            .unwrap();
            assert_eq!(affine.opacity, expected);
        }
        let mut p = project();
        p.tracks[0].items[0].visual_properties_mut().animation_channels = serde_json::from_value(json!([
            {"property":"transform.opacity","loop":{"mode":"ping_pong","iterations":"infinite"},"keyframes":[
                {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
                {"timeMs":125,"value":{"type":"scalar","value":1},"curve":"hold"}]}])).unwrap();
        let tracks = p.tracks.clone();
        p.components=serde_json::from_value(json!([{"id":"definition","name":"Definition","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[]}])).unwrap();
        p.tracks[0].items=serde_json::from_value(json!([
            {"type":"component_instance","id":"instance","componentId":"definition","startMs":100,"durationMs":900,"trimStartMs":0,"timeScale":0.5,"slotValues":{},"stackOrder":0,"zIndex":0}
        ])).unwrap();
        let evaluated = scene(&p);
        let schedule = frame_schedule(&evaluated, 401).unwrap();
        let (index, time) = schedule
            .tasks
            .iter()
            .find_map(|task| match task {
                MatteTask::LeafSample {
                    layer_index, at_ms, ..
                } if evaluated.mattes.as_ref().unwrap().roles[*layer_index]
                    .group
                    .is_some_and(|g| {
                        evaluated.mattes.as_ref().unwrap().groups[g.0].authored_item_id
                            == "provider"
                    }) =>
                {
                    Some((*layer_index, *at_ms))
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(time, 401);
        // Root401 -> (401−100)*.5=150.5; ping-pong around125 ->99.5.
        let (mut sampled, _, _) =
            super::super::extended_visual::sample(&evaluated.visual_layers[index], time).unwrap();
        let affine =
            super::super::extended_visual::sample_transform(&mut sampled, time, (4, 4), (64, 64))
                .unwrap();
        assert!((affine.opacity - 99.5 / 125.).abs() < 1e-12);
        // The inherited fraction is then added to a retained near-safe-limit
        // whole offset. This exercises actual Split selection in the sampler,
        // not a rounded f64 expectation or a standalone SampleTime helper.
        let safe = 9_007_199_254_740_991_u64;
        p.components[0].tracks[0].items[0].visual_properties_mut().animation_channels = serde_json::from_value(json!([
            {"property":"transform.opacity","clock":{"offsetMs":safe-2000,"sourceDurationMs":safe},"keyframes":[
                {"timeMs":safe-1900,"value":{"type":"scalar","value":0},"curve":"linear"},
                {"timeMs":safe-1800,"value":{"type":"scalar","value":1},"curve":"hold"}]}])).unwrap();
        let evaluated = scene(&p);
        let schedule = frame_schedule(&evaluated, 401).unwrap();
        let (index, time) = schedule
            .tasks
            .iter()
            .find_map(|task| match task {
                MatteTask::LeafSample {
                    layer_index, at_ms, ..
                } if evaluated.mattes.as_ref().unwrap().roles[*layer_index]
                    .group
                    .is_some_and(|g| {
                        evaluated.mattes.as_ref().unwrap().groups[g.0].authored_item_id
                            == "provider"
                    }) =>
                {
                    Some((*layer_index, *at_ms))
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(time, 401);
        let (mut sampled, _, _) =
            super::super::extended_visual::sample(&evaluated.visual_layers[index], time).unwrap();
        let affine =
            super::super::extended_visual::sample_transform(&mut sampled, time, (4, 4), (64, 64))
                .unwrap();
        assert!((affine.opacity - 0.505).abs() < 1e-12);
    }
    #[test]
    fn complete_scene_spare_source_and_program_capacities_are_additive_to_matte_certificate() {
        let mut evaluated = scene(&project());
        let before = scene_heap_bytes(&evaluated).unwrap();
        let fixed_before = frame_schedule(&evaluated, 400)
            .unwrap()
            .certificate
            .fixed_live_bytes;
        let old_capacity = evaluated.visual_layers[0].keyframes.capacity();
        evaluated.visual_layers[0].keyframes.reserve_exact(128);
        let extra = (evaluated.visual_layers[0].keyframes.capacity() - old_capacity)
            * std::mem::size_of::<super::super::EvaluatedKeyframe>();
        assert_eq!(scene_heap_bytes(&evaluated).unwrap() - before, extra as u64);
        assert_eq!(
            frame_schedule(&evaluated, 400)
                .unwrap()
                .certificate
                .fixed_live_bytes
                - fixed_before,
            extra as u64
        );
        let old_id_capacity = evaluated.resources.capacity();
        evaluated.resources.reserve_exact(64);
        assert_eq!(
            scene_heap_bytes(&evaluated).unwrap() - before,
            extra as u64
                + ((evaluated.resources.capacity() - old_id_capacity)
                    * std::mem::size_of::<super::super::EvaluatedMediaResource>())
                    as u64
        );
    }
    #[test]
    fn noncentral_group_and_local_rotated_recipient_keep_independent_canvas_mapping() {
        let mut p = project();
        p.settings.width = 96;
        p.settings.height = 64;
        if let crate::TimelineItem::Rectangle(r) = &mut p.tracks[0].items[0] {
            r.width = 96;
            r.height = 64;
            r.color = "#ff0000".into();
            r.visual_properties.transform.opacity = 0.5;
        }
        if let crate::TimelineItem::Rectangle(r) = &mut p.tracks[0].items[1] {
            r.width = 12;
            r.height = 8;
            r.visual_properties.parent =
                Some(serde_json::from_value(json!({"scope":"root","id":"parent"})).unwrap());
            r.visual_properties.transform2d=Some(serde_json::from_value(json!({"position":{"x":4,"y":4,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":90,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0.25,"y":0.75},"opacity":1})).unwrap());
            r.visual_properties.matte.as_mut().unwrap().channel = crate::MatteChannel::Luma;
        }
        p.tracks[0].items.push(serde_json::from_value(json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":2,"zIndex":0,"transform2d":{"position":{"x":64,"y":84,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0.25,"y":0.75},"opacity":0.5}})).unwrap());
        let evaluated = scene(&p);
        let (_, layer) = evaluated
            .visual_layers
            .iter()
            .enumerate()
            .find(|(index, _)| {
                evaluated.mattes.as_ref().unwrap().roles[*index]
                    .group
                    .is_some_and(|g| {
                        evaluated.mattes.as_ref().unwrap().groups[g.0].authored_item_id
                            == "recipient"
                    })
            })
            .unwrap();
        let (mut sampled, _, _) = super::super::extended_visual::sample(layer, 400).unwrap();
        let affine =
            super::super::extended_visual::sample_transform(&mut sampled, 400, (12, 8), (96, 64))
                .unwrap();
        // Local: x=10-y,y=1+x. Parent translation64−24,84−48.
        for (actual, expected) in affine.matrix.iter().zip([0., 1., -1., 0., 50., 37.]) {
            assert!(
                (actual - expected).abs() < 1e-10,
                "actual affine {:?}",
                affine.matrix
            );
        }
        assert_eq!(affine.opacity, 0.5);
    }
    #[test]
    fn transitive_shutter_leaf_samples_share_mask_work_before_materialization() {
        let mut p = project();
        let mask: crate::Mask = serde_json::from_value(json!({
            "id":"feather", "source":{"type":"path","path":{"fillRule":"nonzero","commands":[
                {"type":"moveTo","to":{"x":0,"y":0}}, {"type":"lineTo","to":{"x":64,"y":0}},
                {"type":"lineTo","to":{"x":64,"y":64}}, {"type":"lineTo","to":{"x":0,"y":64}}, {"type":"close"}
            ]},"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}},
            "channel":"alpha","operation":"intersect","inverted":false,"featherPx":8,"expansionPx":0,
            "transform":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}
        })).unwrap();
        p.tracks[0].items[0].visual_properties_mut().masks = vec![mask];
        p.tracks[0].items[0].visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 16,
        });
        let single = scene(&p);
        let accepted = frame_schedule(&single, 400).unwrap();
        assert_eq!(
            accepted
                .tasks
                .iter()
                .filter(|task| matches!(task, MatteTask::LeafSample { layer_index: 0, .. }))
                .count(),
            16
        );
        super::super::extended_visual::preflight_samples(&single, 400, 400, true).unwrap();
        p.tracks[0].items[1].visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 16,
        });
        let nested = scene(&p);
        let error = frame_schedule(&nested, 400).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("mask work"), "{error:?}");
        let error =
            super::super::extended_visual::preflight_samples(&nested, 400, 400, true).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("mask work"));
        let error =
            super::super::extended_certification::certify_scene(&nested, &p, &mut 0).unwrap_err();
        assert!(error.message.contains("mask work"), "{error:?}");
        let mut delayed = nested.clone();
        delayed.duration_ms = 4000;
        for layer in &mut delayed.visual_layers {
            layer.instance = Some(super::super::EvaluatedInstance {
                rate: 2.,
                offset: -6000.,
                start_ms: 3000.25,
                end_ms: 3500.5,
                canvas: (64, 64),
            });
        }
        assert_eq!(integer_visibility(&delayed.visual_layers[0]), (3001, 3501));
        let error =
            super::super::extended_certification::certify_scene(&delayed, &p, &mut 0).unwrap_err();
        assert!(
            error.message.contains("mask work"),
            "delayed interior must not be skipped: {error:?}"
        );
    }
    #[test]
    fn actual_main_uncached_request_limit_is_shared_across_scoped_provider_groups() {
        let mut p = project();
        let provider = p.tracks[0].items[0].clone();
        let recipient = p.tracks[0].items[1].clone();
        p.tracks[0].items.clear();
        for n in 0..2048 {
            let mut a = provider.clone();
            let mut b = recipient.clone();
            if let crate::TimelineItem::Rectangle(item) = &mut a {
                item.id = format!("p-{n}");
            }
            if let crate::TimelineItem::Rectangle(item) = &mut b {
                item.id = format!("r-{n}");
            }
            a.visual_properties_mut().stack_order = n * 2;
            b.visual_properties_mut().stack_order = n * 2 + 1;
            b.visual_properties_mut().matte.as_mut().unwrap().source_id = format!("p-{n}");
            b.visual_properties_mut().motion_blur = Some(crate::MotionBlur {
                shutter_angle_deg: 180.,
                sample_count: 2,
            });
            p.tracks[0].items.extend([a, b]);
        }
        let accepted = scene(&p);
        let schedule = frame_schedule(&accepted, 400).unwrap();
        assert_eq!(schedule.certificate.provider_requests, 4096);
        let mut continuous_nodes = 0;
        certify_continuous_requests(&accepted, &mut continuous_nodes, |_| Ok(())).unwrap();
        assert!(continuous_nodes < 65536);
        p.tracks[0].items[1]
            .visual_properties_mut()
            .motion_blur
            .as_mut()
            .unwrap()
            .sample_count = 3;
        let rejected = scene(&p);
        let error = frame_schedule(&rejected, 400).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("provider requests"), "{error:?}");
        let error = certify_continuous_requests(&rejected, &mut 0, |_| Ok(())).unwrap_err();
        assert!(error.message.contains("provider requests"), "{error:?}");
    }

    #[test]
    fn ordinary_and_mask_segments_share_root_scene_limit_with_duplicate_shutter_ticks() {
        let p = project();
        let mut evaluated = scene(&p);
        let points = (0..256)
            .map(|n| {
                let angle = n as f64 * std::f64::consts::TAU / 256.;
                crate::VectorPoint {
                    x: 2. + angle.cos(),
                    y: 2. + angle.sin(),
                }
            })
            .collect::<Vec<_>>();
        let ordinary_shape = super::super::shapes::EvaluatedShape::new(
            crate::ShapeGeometry::Polygon {
                points: points.clone(),
            },
            Some(crate::Paint::Solid {
                color: crate::VectorColor {
                    r: 1.,
                    g: 1.,
                    b: 1.,
                    a: 1.,
                },
            }),
            None,
            1.,
        )
        .unwrap();
        assert_eq!(ordinary_shape.segments(), 256);
        let mut ordinary = evaluated.visual_layers[1].clone();
        ordinary.source = super::super::EvaluatedVisualSource::Shape(Box::new(ordinary_shape));
        ordinary.extended = None;
        let role = EvaluatedMatteRole {
            group: None,
            provider: None,
            matte_only: false,
            contributes: true,
        };
        // Keep the provider as the sole masked occurrence and no direct recipient:
        // ordinary leaves are direct draws, and all requests are at root400.
        evaluated.visual_layers.truncate(1);
        evaluated.mattes.as_mut().unwrap().roles.truncate(1);
        evaluated.mattes.as_mut().unwrap().groups[1].members.clear();
        evaluated.mattes.as_mut().unwrap().roles[0].matte_only = false;
        let mut commands = Vec::new();
        commands.push(crate::PathCommand::MoveTo { to: points[0] });
        commands.extend(
            points
                .iter()
                .skip(1)
                .map(|point| crate::PathCommand::LineTo { to: *point }),
        );
        commands.push(crate::PathCommand::Close {});
        let mask = crate::Mask {
            id: "segments".into(),
            source: crate::MaskSource::Path {
                path: crate::VectorPath {
                    fill_rule: crate::FillRule::Nonzero,
                    commands,
                },
                paint: crate::Paint::Solid {
                    color: crate::VectorColor {
                        r: 1.,
                        g: 1.,
                        b: 1.,
                        a: 1.,
                    },
                },
            },
            channel: crate::MaskChannel::Alpha,
            operation: crate::MaskOperation::Intersect,
            inverted: false,
            transform: crate::Transform2D::default(),
            feather_px: 0.,
            expansion_px: 0.,
        };
        let provider = &mut evaluated.visual_layers[0];
        provider.extended = Some(super::super::extended_visual::ExtendedVisual {
            crop: None,
            motion_blur: None,
            frame_rate: 10,
            effects: Vec::new(),
            masks: Default::default(),
            channels: Default::default(),
        });
        provider.extended.as_mut().unwrap().masks =
            super::super::extended_visual::SharedMasks::from(vec![mask]);
        provider.extended.as_mut().unwrap().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 1.,
            sample_count: 4,
        });
        for n in 0..4095 {
            let mut layer = ordinary.clone();
            layer.item_id = format!("ordinary-{n}");
            evaluated.visual_layers.push(layer);
            evaluated.mattes.as_mut().unwrap().roles.push(role.clone());
        }
        let accepted = frame_schedule(&evaluated, 400).unwrap();
        let ticks = accepted
            .tasks
            .iter()
            .filter_map(|task| match task {
                MatteTask::LeafSample {
                    layer_index: 0,
                    at_ms,
                    ..
                } => Some(*at_ms),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ticks, [399, 399, 400, 400]);
        let mut masks = (*evaluated.visual_layers[0].extended.as_ref().unwrap().masks).clone();
        let crate::MaskSource::Path { path, .. } = &mut masks[0].source;
        path.commands.insert(
            1,
            crate::PathCommand::LineTo {
                to: crate::VectorPoint { x: 3., y: 2.001 },
            },
        );
        evaluated.visual_layers[0].extended.as_mut().unwrap().masks =
            super::super::extended_visual::SharedMasks::from(masks);
        let error = frame_schedule(&evaluated, 400).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("segment"), "{error:?}");
    }
    #[test]
    fn actual_spare_font_payload_capacity_and_caller_clone_join_shared_fixed_memory() {
        let mut evaluated = scene(&project());
        let initial = font_payload_admission(&evaluated).unwrap();
        evaluated
            .composition_resources
            .as_mut()
            .unwrap()
            .resource_live_bytes += initial - 4096;
        assert_eq!(font_payload_admission(&evaluated).unwrap(), 4096);
        let mut spare = Vec::<u8>::with_capacity(4097);
        spare.push(1);
        assert_eq!(spare.len(), 1);
        assert!(spare.capacity() > 4096);
        let before = evaluated.clone();
        let error = adopt_font_payload(&mut evaluated, spare.capacity() as u64).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert_eq!(evaluated, before);
        adopt_font_payload(&mut evaluated, 4096).unwrap();
        assert_eq!(font_payload_admission(&evaluated).unwrap(), 4096);
        assert_eq!(
            evaluated
                .composition_resources
                .as_ref()
                .unwrap()
                .font_payload_bytes,
            4096
        );
        assert_eq!(
            admit_caller_scene_clone(&evaluated).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    #[test]
    fn continuous_integer_clamps_preserve_collision_regions_and_subnormal_offsets() {
        let identity = RequestClock {
            offset: 0,
            low: 0,
            high: 9,
        };
        let late = identity.shifted(-5, 9).unwrap().shifted(5, 9).unwrap();
        let early = identity.shifted(5, 9).unwrap().shifted(-5, 9).unwrap();
        assert_ne!(identity, late);
        assert_ne!(identity, early);
        assert_ne!(late, early);
        for root in 0..10 {
            let values = [identity.at(root), late.at(root), early.at(root)];
            assert_eq!(values, [root, root.max(5), root.min(4)]);
            assert_eq!(
                values
                    .into_iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                2
            );
        }
        let mut p = project();
        p.tracks[0].items[1].visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: f64::from_bits(1),
            sample_count: 16,
        });
        let evaluated = scene(&p);
        assert_eq!(
            shutter_offsets(&evaluated.visual_layers[1]).unwrap(),
            [-1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
    #[test]
    fn continuous_duration_one_depth_32_preserves_weighted_samples_without_provenance_product() {
        let mut p = project();
        p.settings.width = 1;
        p.settings.height = 1;
        let template = serde_json::to_value(&p.tracks[0].items[0]).unwrap();
        let mut items = Vec::new();
        for index in 0..33 {
            let mut item = template.clone();
            item["id"] = json!(format!("leaf-{index}"));
            item["durationMs"] = json!(1);
            item["width"] = json!(1);
            item["height"] = json!(1);
            item["stackOrder"] = json!(index);
            item["matteOnly"] = json!(index != 32);
            item["motionBlur"] = json!({"shutterAngleDeg":360,"sampleCount":16});
            if index > 0 {
                item["matte"] = json!({"sourceId":format!("leaf-{}", index-1),"channel":"alpha"});
            }
            items.push(item);
        }
        p.tracks[0].items = serde_json::from_value(json!(items)).unwrap();
        let evaluated = scene(&p);
        let mut nodes = 0;
        let mut observations = 0;
        certify_continuous_requests(&evaluated, &mut nodes, |visits| {
            assert_eq!(visits, &[16; 33]);
            observations += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(observations, 1);
        assert!(nodes < 256, "canonical zero clocks must merge: {nodes}");
        let schedule = frame_schedule(&evaluated, 0).unwrap();
        assert_eq!(schedule.certificate.provider_requests, 32);
        assert_eq!(
            schedule
                .tasks
                .iter()
                .filter(|t| matches!(t, MatteTask::LeafSample { .. }))
                .count(),
            33 * 16
        );
        super::super::extended_certification::certify_scene(&evaluated, &p, &mut nodes).unwrap();
    }
    #[test]
    fn continuous_actual_graph_refines_excess_classes_at_integer_collision_regions() {
        let mut p = project();
        p.settings.width = 1;
        p.settings.height = 1;
        p.settings.fps = 50;
        for item in &mut p.tracks[0].items {
            if let crate::TimelineItem::Rectangle(r) = item {
                r.duration_ms = 10;
                r.width = 1;
                r.height = 1;
            }
        }
        p.tracks[0].items[0].visual_properties_mut().matte_only = false;
        let mut middle = p.tracks[0].items[1].clone();
        if let crate::TimelineItem::Rectangle(r) = &mut middle {
            r.id = "middle".into();
        }
        middle.visual_properties_mut().matte_only = true;
        middle.visual_properties_mut().stack_order = 1;
        middle.visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 360.,
            sample_count: 2,
        });
        p.tracks[0].items[1]
            .visual_properties_mut()
            .matte
            .as_mut()
            .unwrap()
            .source_id = "middle".into();
        p.tracks[0].items[1].visual_properties_mut().stack_order = 2;
        p.tracks[0].items[1].visual_properties_mut().motion_blur =
            middle.visual_properties().motion_blur;
        p.tracks[0].items.insert(1, middle);
        let evaluated = scene(&p);
        let mut nodes = 0;
        let mut excessive_upper = 0;
        let mut exact_regions = 0;
        let mut smallest = u64::MAX;
        let mut largest = 0;
        certify_continuous_requests(&evaluated, &mut nodes, |visits| {
            // Five terminal functions (including direct identity) have only
            // THREE or FOUR exact integer keys over roots0..9. This is an
            // actual graph, not a synthetic fractional-domain approximation.
            if visits[0] > 4 {
                excessive_upper += 1;
                return Err(invalid("test terminal work excess"));
            }
            assert!((3..=4).contains(&visits[0]));
            smallest = smallest.min(visits[0]);
            largest = largest.max(visits[0]);
            assert_eq!(visits[1], 4);
            assert_eq!(visits[2], 2);
            exact_regions += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(excessive_upper, 1);
        assert!(exact_regions > 1);
        assert_eq!((smallest, largest), (3, 4));
        let mut exhausted = 65535;
        let error =
            certify_continuous_requests(&evaluated, &mut exhausted, |_| Ok(())).unwrap_err();
        assert_eq!(exhausted, 65536);
        assert!(error.message.contains("maxCandidateAnalysisNodes"));
    }
    #[test]
    fn continuous_transitive_effect_work_joins_existing_owner_instead_of_resetting_per_provider() {
        let mut p = project();
        p.tracks[0].items[0].visual_properties_mut().effects =
            vec![crate::VisualEffect::GaussianBlur {
                id: "blur".into(),
                radius_px: 16.,
            }];
        p.tracks[0].items[0].visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 16,
        });
        let single = scene(&p);
        super::super::extended_certification::certify_scene(&single, &p, &mut 0).unwrap();
        frame_schedule(&single, 400).unwrap();
        p.tracks[0].items[1].visual_properties_mut().motion_blur = Some(crate::MotionBlur {
            shutter_angle_deg: 180.,
            sample_count: 16,
        });
        let transitive = scene(&p);
        let error = super::super::extended_certification::certify_scene(&transitive, &p, &mut 0)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(
            error.message.contains("maxPixelPassesPerSample"),
            "{error:?}"
        );
        let error = frame_schedule(&transitive, 400).unwrap_err();
        assert!(
            error.message.contains("maxPixelPassesPerSample"),
            "{error:?}"
        );
    }
    fn blend_project(count: usize, size: u32) -> Project {
        let mut p = project();
        p.settings.width = size;
        p.settings.height = size;
        let mut template = serde_json::to_value(&p.tracks[0].items[0]).unwrap();
        template.as_object_mut().unwrap().remove("matteOnly");
        template["width"] = json!(size);
        template["height"] = json!(size);
        template["blendMode"] = json!("multiply");
        let items = (0..count)
            .map(|index| {
                let mut item = template.clone();
                item["id"] = json!(format!("ordinary-{index}"));
                item["stackOrder"] = json!(index);
                item
            })
            .collect::<Vec<_>>();
        p.tracks[0].items = serde_json::from_value(json!(items)).unwrap();
        p
    }
    #[test]
    fn graph_free_blend_activation_has_real_resource_facts_and_exact_normal_bypass() {
        let p = blend_project(2, 1);
        let evaluated = scene(&p);
        assert!(evaluated.mattes.is_none());
        assert!(evaluated.composition_resources.is_some());
        let schedule = frame_schedule(&evaluated, 400).unwrap();
        assert_eq!(schedule.certificate.provider_requests, 0);
        assert_eq!(schedule.certificate.blend_work_units, 64);
        let mut normal = p.clone();
        for item in &mut normal.tracks[0].items {
            item.visual_properties_mut().blend_mode = crate::BlendMode::Normal;
        }
        let first = scene(&normal);
        assert!(first.mattes.is_none());
        assert!(first.composition_resources.is_none());
        let mut explicit = serde_json::to_value(&normal).unwrap();
        for item in explicit["tracks"][0]["items"].as_array_mut().unwrap() {
            item["blendMode"] = json!("normal");
        }
        assert_eq!(first, scene(&serde_json::from_value(explicit).unwrap()));
    }
    #[test]
    fn graph_free_4096_ordinary_occurrences_with_maximum16_shutters_have_zero_provider_requests() {
        let mut p = blend_project(4096, 1);
        for item in &mut p.tracks[0].items {
            item.visual_properties_mut().motion_blur = Some(crate::MotionBlur {
                shutter_angle_deg: 360.0,
                sample_count: 16,
            });
        }
        let evaluated = scene(&p);
        let schedule = frame_schedule(&evaluated, 400).unwrap();
        assert!(evaluated.mattes.is_none());
        assert_eq!(schedule.certificate.provider_requests, 0);
        assert_eq!(
            schedule
                .tasks
                .iter()
                .filter(|task| matches!(task, MatteTask::LeafSample { .. }))
                .count(),
            65536
        );
        assert_eq!(
            schedule
                .tasks
                .iter()
                .filter(|task| matches!(task, MatteTask::AverageCopy { .. }))
                .count(),
            4096
        );
        assert_eq!(schedule.direct_draw.len(), 4096);
        assert_eq!(schedule.certificate.blend_work_units, 4096 * 32);
        assert!(schedule.certificate.peak_live_bytes <= MAX_MATTE_LIVE_BYTES);
        for count in [17, 32] {
            let mut invalid = p.clone();
            invalid.tracks[0].items[0]
                .visual_properties_mut()
                .motion_blur
                .as_mut()
                .unwrap()
                .sample_count = count;
            assert_eq!(
                super::super::evaluate_project(&invalid, 1, 1, 10)
                    .unwrap_err()
                    .code,
                crate::ErrorCode::InvalidArgument
            );
        }
        let mut excessive = p.clone();
        let mut extra = p.tracks[0].items[0].clone();
        if let crate::TimelineItem::Rectangle(item) = &mut extra {
            item.id = "extra".into();
            item.visual_properties.stack_order = 4096;
        }
        excessive.tracks[0].items.push(extra);
        assert!(super::super::evaluate_project(&excessive, 1, 1, 10).is_err());
    }
    #[test]
    fn reachable_blend_work_exact128_full_footprints_and_one_clipped_pixel_excess() {
        let mut p = blend_project(128, 256);
        let evaluated = scene(&p);
        let schedule = frame_schedule(&evaluated, 400).unwrap();
        assert_eq!(schedule.certificate.blend_work_units, 268_435_456);
        assert_eq!(schedule.certificate.provider_requests, 0);
        assert!(schedule.certificate.peak_live_bytes <= MAX_MATTE_LIVE_BYTES);
        let mut extra = p.tracks[0].items[0].clone();
        if let crate::TimelineItem::Rectangle(item) = &mut extra {
            item.id = "extra-pixel".into();
            item.width = 1;
            item.height = 1;
            item.transform.position_x = 256.0;
            item.transform.position_y = 256.0;
            item.visual_properties.stack_order = 128;
        }
        p.tracks[0].items.push(extra);
        let excessive = scene(&p);
        let layer = excessive
            .visual_layers
            .iter()
            .find(|layer| layer.item_id == "extra-pixel")
            .unwrap();
        assert_eq!(
            super::super::extended_visual::certified_destination_visits(layer, 400, (256, 256))
                .unwrap(),
            1
        );
        assert_eq!(
            frame_schedule(&excessive, 400).unwrap_err().code,
            crate::ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn owning_average_charges_stationary_small_support_union_once() {
        let mut p = blend_project(129, 256);
        for item in &mut p.tracks[0].items {
            if let crate::TimelineItem::Rectangle(item) = item {
                item.width = 1;
                item.height = 1;
            }
            item.visual_properties_mut().motion_blur = Some(crate::MotionBlur {
                shutter_angle_deg: 360.0,
                sample_count: 16,
            });
        }
        let evaluated = scene(&p);
        let schedule = frame_schedule(&evaluated, 400).unwrap();
        assert_eq!(schedule.certificate.provider_requests, 0);
        assert!(
            schedule
                .direct_draw
                .iter()
                .all(|draw| draw.destination_visits == 4)
        );
        assert_eq!(schedule.certificate.blend_work_units, 129 * 4 * 32);
        assert_eq!(
            schedule
                .tasks
                .iter()
                .filter(|task| matches!(task, MatteTask::LeafSample { .. }))
                .count(),
            129 * 16
        );
    }
    #[test]
    fn normal_matte_metadata_exact_shared_limit_succeeds_and_excess_refuses_before_reservation() {
        let mut p = project();
        for item in &mut p.tracks[0].items {
            item.visual_properties_mut().matte_only = true;
        }
        let mut evaluated = scene(&p);
        let original = frame_schedule(&evaluated, 400).unwrap();
        assert_eq!(original.certificate.provider_requests, 0);
        assert_eq!(original.certificate.matte_work_units, 0);
        assert_eq!(original.certificate.blend_work_units, 0);
        assert!(original.tasks.is_empty());
        evaluated
            .composition_resources
            .as_mut()
            .unwrap()
            .resource_live_bytes += MAX_MATTE_LIVE_BYTES - original.certificate.peak_live_bytes;
        let exact = frame_schedule(&evaluated, 400).unwrap();
        assert_eq!(exact.certificate.peak_live_bytes, MAX_MATTE_LIVE_BYTES);
        evaluated
            .composition_resources
            .as_mut()
            .unwrap()
            .resource_live_bytes += 1;
        METADATA_RESERVATIONS.with(|counter| counter.set(0));
        assert_eq!(
            frame_schedule(&evaluated, 400).unwrap_err().code,
            crate::ErrorCode::InvalidArgument
        );
        assert_eq!(
            METADATA_RESERVATIONS.with(std::cell::Cell::get),
            0,
            "no owning vector reservation may precede memory refusal"
        );
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn private_provider_and_direct_relative_products_are_distinct_in_one_query() {
        let project: crate::Project = serde_json::from_value(json!({"schemaVersion":37,"id":"p","revision":0,"name":"Query","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":64,"height":64,"fps":10},"fonts":{},"markers":[],"assets":[],"components":[],
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":[
                {"type":"group","id":"g","startMs":0,"durationMs":1000,"clip":{"type":"composition_bounds"},"zIndex":0,"stackOrder":0},
                {"type":"rectangle","id":"provider","width":4,"height":4,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":1,
                    "parent":{"scope":"root","id":"g"},"motionBlur":{"shutterAngleDeg":180,"sampleCount":2}},
                {"type":"rectangle","id":"recipient","width":4,"height":4,"color":"#ff0000","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":2,
                    "parent":{"scope":"root","id":"g"},"matte":{"sourceId":"provider","channel":"alpha"}}
            ]}]})).unwrap();
        let mut scene = super::super::evaluate_project(&project, 64, 64, 10)
            .unwrap()
            .scene;
        super::super::finalize_affine_geometry(&mut scene, &Default::default()).unwrap();
        let frame = QueryFrame::new((8, 6), [-24., -18.], [[0., 1.], [-1., 0.]]).unwrap();
        assert_eq!(frame.matrix(), [0., 1., -1., 0., -24., -18.]);
        let scope = QueryScope {
            frame,
            owner: Some(0),
            local_origin: [0., 0.],
        };
        let mut budget = OutputFrameBudget::new(&scene).unwrap();
        let schedule = frame_schedule_in_domain(&scene, 400, Some(scope), &mut budget).unwrap();
        assert_eq!(schedule.query, Some(frame));
        assert_eq!(schedule.canvas, (8, 6));
        assert_eq!(schedule.direct_draw.len(), 2);
        let mut direct = Vec::new();
        let mut private = Vec::new();
        for task in &schedule.tasks {
            if let MatteTask::LeafSample {
                layer_index: 0,
                at_ms,
                relative_owner,
                ..
            } = task
            {
                if *relative_owner == Some(0) {
                    direct.push(*at_ms);
                } else {
                    private.push(*at_ms);
                }
            }
        }
        assert_eq!(direct.len(), 2);
        assert_eq!(private.len(), 2);
        assert_eq!(direct, private);
        assert_eq!(schedule.certificate.provider_requests, 1);
        assert_eq!(budget.requests, 1);
        let second = frame_schedule_in_domain(&scene, 400, Some(scope), &mut budget).unwrap();
        assert_eq!(second.certificate.provider_requests, 1);
        assert_eq!(budget.requests, 2);
        budget.requests = MAX_MATTE_REQUESTS;
        assert!(frame_schedule_in_domain(&scene, 400, Some(scope), &mut budget).is_err());
    }
    #[test]
    fn query_frame_admission_accepts_signed_and_singular_bases_but_rejects_invalid_surfaces() {
        assert!(QueryFrame::new((4, 3), [-500., -10.], [[0., 0.], [0., 0.]]).is_ok());
        assert!(QueryFrame::new((4, 3), [0., 0.], [[f64::MAX, 0.], [0., 1.]]).is_err());
        assert!(QueryFrame::new((4, 3), [f64::MAX, 0.], [[f64::MAX, 0.], [0., 1.]]).is_err());
        for size in [(0, 3), (16385, 1), (4097, 4096)] {
            assert!(QueryFrame::new(size, [0., 0.], [[1., 0.], [0., 1.]]).is_err());
        }
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(QueryFrame::new((4, 3), [invalid, 0.], [[1., 0.], [0., 1.]]).is_err());
            assert!(QueryFrame::new((4, 3), [0., 0.], [[invalid, 0.], [0., 1.]]).is_err());
        }
    }
}
