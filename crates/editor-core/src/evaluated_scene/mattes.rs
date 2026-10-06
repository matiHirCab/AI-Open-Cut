//! Immutable scoped matte bindings and pre-certified exact frame tasks.
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
    pub resource_live_bytes: u64,
    pub font_payload_bytes: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MatteTask {
    LeafSample {
        layer_index: usize,
        at_ms: u64,
        provider: Option<(MatteTaskId, MatteChannel)>,
        source_live_bytes: u64,
    },
    AverageCopy {
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
    /// Caller destination/cache/facts/schedule and executor-slot metadata.
    pub fixed_live_bytes: u64,
    /// Actual schedule/nested Vec capacities plus128 bytes/task executor slots.
    pub descriptor_bytes: u64,
    /// Absolute shared peak including fixed storage, bound1073741824.
    pub peak_live_bytes: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MatteFrameSchedule {
    pub canvas: (u32, u32),
    pub tasks: Vec<MatteTask>,
    pub direct_draw: Vec<MatteTaskId>,
    /// Task indices followed by direct draws at tasks.len()+draw_index.
    pub last_uses: Vec<usize>,
    pub certificate: MatteFrameCertificate,
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
    let mut bytes = add(
        std::mem::size_of::<EvaluatedMatteGraph>() as u64,
        graph.resource_live_bytes,
    )?;
    bytes = add(bytes, capacity_bytes(&graph.groups)?)?;
    bytes = add(bytes, capacity_bytes(&graph.roles)?)?;
    bytes = add(bytes, capacity_bytes(&graph.provider_first)?)?;
    for group in &graph.groups {
        bytes = add(bytes, group.authored_item_id.capacity() as u64)?;
        bytes = add(bytes, capacity_bytes(&group.members)?)?;
    }
    Ok(bytes)
}

pub(crate) fn font_payload_admission(
    scene: &super::EvaluatedScene,
) -> Result<u64, crate::CoreError> {
    let graph = scene
        .mattes
        .as_ref()
        .ok_or_else(|| invalid("matte resource facts unavailable"))?;
    let base = add(
        add(
            MATTE_CACHE_RESERVATION,
            mul(
                mul(
                    u64::from(scene.canvas.width),
                    u64::from(scene.canvas.height),
                )?,
                20,
            )?,
        )?,
        add(scene_heap_bytes(scene)?, graph_heap_bytes(graph)?)?,
    )?;
    MAX_MATTE_LIVE_BYTES
        .checked_sub(
            base.checked_sub(graph.font_payload_bytes)
                .ok_or_else(|| invalid("matte font reservation invalid"))?,
        )
        .ok_or_else(|| invalid("matte resource memory exceeds shared bounds"))
}
pub(crate) fn adopt_font_payload(
    scene: &mut super::EvaluatedScene,
    bytes: u64,
) -> Result<(), crate::CoreError> {
    if scene.mattes.is_none() {
        return Ok(());
    }
    if bytes > font_payload_admission(scene)? {
        return Err(invalid("matte font payload exceeds shared memory bounds"));
    }
    let graph = scene.mattes.as_mut().unwrap();
    graph.resource_live_bytes = add(
        graph
            .resource_live_bytes
            .checked_sub(graph.font_payload_bytes)
            .ok_or_else(|| invalid("matte font reservation invalid"))?,
        bytes,
    )?;
    graph.font_payload_bytes = bytes;
    Ok(())
}
pub(crate) fn admit_caller_scene_clone(
    scene: &super::EvaluatedScene,
) -> Result<u64, crate::CoreError> {
    let graph = scene
        .mattes
        .as_ref()
        .ok_or_else(|| invalid("matte resource facts unavailable"))?;
    let heap = add(
        scene_heap_bytes(scene)?,
        graph_heap_bytes(graph)?
            .checked_sub(graph.resource_live_bytes)
            .ok_or_else(|| invalid("matte caller memory invalid"))?,
    )?;
    let available = font_payload_admission(scene)?;
    if add(heap, graph.font_payload_bytes)? > available {
        return Err(invalid(
            "matte caller scene clone exceeds shared memory bounds",
        ));
    }
    Ok(heap)
}

fn add(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_add(b)
        .ok_or_else(|| invalid("matte resource arithmetic overflow"))
}
fn mul(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_mul(b)
        .ok_or_else(|| invalid("matte resource arithmetic overflow"))
}
fn capacity_bytes<T>(items: &Vec<T>) -> Result<u64, crate::CoreError> {
    mul(items.capacity() as u64, std::mem::size_of::<T>() as u64)
}
pub(crate) fn channel_heap(
    channels: &Vec<crate::AnimationChannel>,
) -> Result<u64, crate::CoreError> {
    let mut bytes = capacity_bytes(channels)?;
    for channel in channels {
        bytes = add(bytes, capacity_bytes(&channel.keyframes)?)?;
        if let Some(target) = &channel.target {
            bytes = add(
                bytes,
                add(target.id.capacity() as u64, target.scope.capacity() as u64)?,
            )?;
        }
        for key in &channel.keyframes {
            bytes = add(
                bytes,
                match &key.value {
                    crate::AnimationChannelValue::PathPoints { points } => capacity_bytes(points)?,
                    crate::AnimationChannelValue::GradientStops { stops } => capacity_bytes(stops)?,
                    _ => 0,
                },
            )?;
        }
    }
    Ok(bytes)
}
fn paint_heap(paint: &crate::Paint) -> Result<u64, crate::CoreError> {
    match paint {
        crate::Paint::Solid { .. } => Ok(0),
        crate::Paint::LinearGradient { stops, .. } | crate::Paint::RadialGradient { stops, .. } => {
            capacity_bytes(stops)
        }
    }
}
fn stroke_heap(stroke: &crate::Stroke) -> Result<u64, crate::CoreError> {
    add(paint_heap(&stroke.paint)?, capacity_bytes(&stroke.dash)?)
}
fn geometry_heap(geometry: &crate::ShapeGeometry) -> Result<u64, crate::CoreError> {
    match geometry {
        crate::ShapeGeometry::Polygon { points } => capacity_bytes(points),
        crate::ShapeGeometry::Path { path } => capacity_bytes(&path.commands),
        _ => Ok(0),
    }
}
fn shape_heap(shape: &super::shapes::EvaluatedShape) -> Result<u64, crate::CoreError> {
    let mut bytes = add(
        geometry_heap(&shape.geometry)?,
        capacity_bytes(&shape.contours)?,
    )?;
    for contour in &shape.contours {
        bytes = add(bytes, capacity_bytes(&contour.points)?)?;
    }
    if let Some(fill) = &shape.fill {
        bytes = add(bytes, paint_heap(fill)?)?;
    }
    if let Some(stroke) = &shape.stroke {
        bytes = add(bytes, stroke_heap(stroke)?)?;
    }
    if let Some(grid) = &shape.grid_descriptor {
        bytes = add(
            bytes,
            match &grid.pattern {
                crate::GridPattern::Dot { paint, .. } => paint_heap(paint)?,
                crate::GridPattern::Rectangular { stroke, .. }
                | crate::GridPattern::Diagonal { stroke, .. }
                | crate::GridPattern::Isometric { stroke, .. } => stroke_heap(stroke)?,
            },
        )?;
    }
    if let Some(svg) = &shape.svg_document {
        bytes = add(bytes, capacity_bytes(&svg.shapes)?)?;
        for child in &svg.shapes {
            bytes = add(bytes, geometry_heap(&child.geometry)?)?;
            if let Some(fill) = &child.fill {
                bytes = add(bytes, paint_heap(fill)?)?;
            }
            if let Some(stroke) = &child.stroke {
                bytes = add(bytes, stroke_heap(stroke)?)?;
            }
        }
    }
    if let Some(children) = &shape.svg_children {
        bytes = add(bytes, capacity_bytes(children)?)?;
        for child in children {
            bytes = add(bytes, shape_heap(child)?)?;
        }
    }
    Ok(bytes)
}
pub(crate) fn text_paints_heap(
    paints: &Vec<crate::TextPaintLayer>,
) -> Result<u64, crate::CoreError> {
    let mut bytes = capacity_bytes(paints)?;
    for paint in paints {
        let (crate::TextPaintLayer::Fill { color, .. }
        | crate::TextPaintLayer::Stroke { color, .. }
        | crate::TextPaintLayer::Shadow { color, .. }) = paint;
        bytes = add(bytes, color.capacity() as u64)?;
    }
    Ok(bytes)
}
pub(crate) fn text_style_heap_bytes(
    style: &super::EvaluatedTextStyle,
) -> Result<u64, crate::CoreError> {
    let mut bytes = 0;
    for color in [
        &style.outline_color,
        &style.shadow.color,
        &style.background_color,
    ] {
        bytes = add(bytes, color.capacity() as u64)?;
    }
    if style.layout.is_some() {
        bytes = add(bytes, std::mem::size_of::<crate::TextLayout>() as u64)?;
    }
    if let Some(paints) = &style.paint_layers {
        bytes = add(bytes, text_paints_heap(paints)?)?;
    }
    Ok(bytes)
}
pub(crate) fn text_heap(text: &super::EvaluatedText) -> Result<u64, crate::CoreError> {
    let mut bytes = add(text.text.capacity() as u64, text.color.capacity() as u64)?;
    if let Some(id) = &text.font_resource_id {
        bytes = add(bytes, id.capacity() as u64)?;
    }
    bytes = add(bytes, text_style_heap_bytes(&text.style)?)?;
    if let Some(binding) = &text.font_binding {
        for value in [
            &binding.profile,
            &binding.regular,
            &binding.bold,
            &binding.italic,
            &binding.bold_italic,
        ] {
            bytes = add(bytes, value.capacity() as u64)?;
        }
        bytes = add(bytes, capacity_bytes(&binding.warnings)?)?;
        for warning in &binding.warnings {
            bytes = add(bytes, warning.capacity() as u64)?;
        }
    }
    if let Some(runs) = &text.rich_runs {
        bytes = add(bytes, capacity_bytes(runs)?)?;
        for run in runs {
            bytes = add(bytes, run.text.capacity() as u64)?;
            if let Some(color) = &run.color {
                bytes = add(bytes, color.capacity() as u64)?;
            }
        }
    }
    if let Some(spans) = &text.spans {
        bytes = add(
            bytes,
            mul(
                spans.len() as u64,
                std::mem::size_of::<crate::TextSpan>() as u64,
            )?,
        )?;
        for span in spans {
            if let Some(color) = &span.style.color {
                bytes = add(bytes, color.capacity() as u64)?;
            }
            if let Some(paints) = &span.style.paint_layers {
                bytes = add(bytes, text_paints_heap(paints)?)?;
            }
        }
    }
    if let Some(shaped) = &text.shaped {
        bytes = add(bytes, shaped_heap_bytes(shaped)?)?;
    }
    Ok(bytes)
}
pub(crate) fn shaped_heap_bytes(
    shaped: &crate::fonts::shaping::ShapedText,
) -> Result<u64, crate::CoreError> {
    let mut bytes = 0;
    bytes = add(
        bytes,
        add(
            capacity_bytes(&shaped.glyphs)?,
            add(
                capacity_bytes(&shaped.line_widths)?,
                capacity_bytes(&shaped.glyph_lines)?,
            )?,
        )?,
    )?;
    for glyph in &shaped.glyphs {
        bytes = add(
            bytes,
            add(glyph.face.capacity() as u64, glyph.color.capacity() as u64)?,
        )?;
        if let Some(paints) = &glyph.paint_layers {
            bytes = add(bytes, text_paints_heap(paints)?)?;
        }
    }
    Ok(bytes)
}
pub(crate) fn layer_heap_bytes(
    layer: &super::EvaluatedVisualLayer,
) -> Result<u64, crate::CoreError> {
    let mut bytes = std::mem::size_of::<super::EvaluatedVisualLayer>() as u64;
    bytes = add(bytes, layer.item_id.capacity() as u64)?;
    if let Some((id, _)) = &layer.sampled_input {
        bytes = add(bytes, id.capacity() as u64)?;
    }
    bytes = add(
        bytes,
        add(
            capacity_bytes(&layer.keyframes)?,
            capacity_bytes(&layer.transitions)?,
        )?,
    )?;
    if let Some(tiles) = &layer.sampling_tiles {
        bytes = add(bytes, capacity_bytes(tiles)?)?;
    }
    bytes = add(bytes, capacity_bytes(&layer.ancestor_stages)?)?;
    for stage in &layer.ancestor_stages {
        bytes = add(bytes, stage.item_id.capacity() as u64)?;
        if let Some(animation) = &stage.animation {
            bytes = add(bytes, channel_heap(&animation.channels)?)?;
            // Shared keys are conservatively charged per stage; no hidden
            // pointer-set allocation is required to claim a smaller bound.
            bytes = add(
                bytes,
                mul(
                    animation.keyframes.len() as u64,
                    std::mem::size_of::<super::EvaluatedKeyframe>() as u64,
                )?,
            )?;
        }
    }
    bytes = add(
        bytes,
        match &layer.source {
            super::EvaluatedVisualSource::Shape(shape) => add(
                std::mem::size_of::<super::shapes::EvaluatedShape>() as u64,
                shape_heap(shape)?,
            )?,
            super::EvaluatedVisualSource::Text(text) => add(
                std::mem::size_of::<super::EvaluatedText>() as u64,
                text_heap(text)?,
            )?,
            super::EvaluatedVisualSource::Media { asset_id, .. } => asset_id.capacity() as u64,
            super::EvaluatedVisualSource::SolidColor { color }
            | super::EvaluatedVisualSource::Rectangle { color, .. } => color.capacity() as u64,
            super::EvaluatedVisualSource::Caption(caption) => add(
                caption.text.capacity() as u64,
                add(
                    caption.color.capacity() as u64,
                    caption.background_color.capacity() as u64,
                )?,
            )?,
        },
    )?;
    if let Some(extended) = &layer.extended {
        bytes = add(bytes, capacity_bytes(&extended.effects)?)?;
        for effect in &extended.effects {
            bytes = add(
                bytes,
                match effect {
                    crate::VisualEffect::GaussianBlur { id, .. }
                    | crate::VisualEffect::Glow { id, .. }
                    | crate::VisualEffect::ColorTint { id, .. }
                    | crate::VisualEffect::Vignette { id, .. } => id.capacity() as u64,
                },
            )?;
        }
        bytes = add(bytes, channel_heap(&extended.channels)?)?;
    }
    Ok(bytes)
}

/// Heap payload actually retained by the complete scene while a matte schedule
/// and every sampled callback coexist. Capacity, rather than length, is charged.
pub(crate) fn scene_heap_bytes(scene: &super::EvaluatedScene) -> Result<u64, crate::CoreError> {
    let mut bytes = std::mem::size_of::<super::EvaluatedScene>() as u64;
    for value in [
        capacity_bytes(&scene.visual_layers)?,
        capacity_bytes(&scene.audio_layers)?,
        capacity_bytes(&scene.resources)?,
        capacity_bytes(&scene.voiceover_intervals)?,
    ] {
        bytes = add(bytes, value)?;
    }
    if let Some(intervals) = &scene.instance_voiceover_intervals {
        bytes = add(bytes, capacity_bytes(intervals)?)?;
    }
    for resource in &scene.resources {
        bytes = add(bytes, resource.asset_id.capacity() as u64)?;
    }
    for layer in &scene.audio_layers {
        bytes = add(
            bytes,
            add(
                layer.item_id.capacity() as u64,
                layer.asset_id.capacity() as u64,
            )?,
        )?;
        bytes = add(bytes, capacity_bytes(&layer.volume_keyframes)?)?;
    }
    for layer in &scene.visual_layers {
        bytes = add(bytes, layer_heap_bytes(layer)?)?;
    }
    add(bytes, super::masks::scene_program_bytes(scene)?)
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
fn shutter_offsets(layer: &super::EvaluatedVisualLayer) -> Result<Vec<i128>, crate::CoreError> {
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
    let graph = scene
        .mattes
        .as_ref()
        .ok_or_else(|| invalid("continuous matte graph absent"))?;
    let pixels = mul(
        u64::from(scene.canvas.width),
        u64::from(scene.canvas.height),
    )?;
    let base = add(
        add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?,
        add(scene_heap_bytes(scene)?, graph_heap_bytes(graph)?)?,
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
    use std::collections::HashSet;
    let graph = scene
        .mattes
        .as_ref()
        .ok_or_else(|| invalid("continuous matte graph absent"))?;
    let end = scene.duration_ms.saturating_sub(1);
    let pixels = mul(
        u64::from(scene.canvas.width),
        u64::from(scene.canvas.height),
    )?;
    let base = admit_continuous_metadata(scene)?;
    struct Classes<'a> {
        scene: &'a super::EvaluatedScene,
        graph: &'a EvaluatedMatteGraph,
        end: u64,
        base: u64,
        copies: HashSet<(usize, RequestClock)>,
        providers: HashSet<(MatteGroupId, RequestClock)>,
        upper_visits: Vec<u64>,
        upper_source: u64,
        upper_work: u64,
        pixels: u64,
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
            nodes: &mut usize,
        ) -> Result<(), crate::CoreError> {
            if self.providers.contains(&(group, clock)) {
                return Ok(());
            }
            self.admit(nodes)?;
            self.providers.insert((group, clock));
            self.upper_work = add(
                self.upper_work,
                mul(
                    mul(self.pixels, 4)?,
                    self.graph.groups[group.0].members.len() as u64,
                )?,
            )?;
            for slot in 0..self.graph.groups[group.0].members.len() {
                self.copy(self.graph.groups[group.0].members[slot], clock, nodes)?;
            }
            Ok(())
        }
        fn copy(
            &mut self,
            index: usize,
            clock: RequestClock,
            nodes: &mut usize,
        ) -> Result<(), crate::CoreError> {
            if self.copies.contains(&(index, clock)) {
                return Ok(());
            }
            self.admit(nodes)?;
            self.copies.insert((index, clock));
            let layer = &self.scene.visual_layers[index];
            let (visible_start, visible_end) = integer_visibility(layer);
            for delta in shutter_offsets(layer)? {
                self.upper_source = add(self.upper_source, self.pixels)?;
                let sample = clock.shifted(delta, self.end)?;
                // Hidden/zero-span leaves are still resource-admitted elsewhere.
                if sample.high < visible_start || sample.low >= visible_end {
                    continue;
                }
                self.upper_visits[index] = add(self.upper_visits[index], 1)?;
                if self.graph.roles[index].provider.is_some() {
                    self.upper_work = add(self.upper_work, mul(self.pixels, 5)?)?;
                }
                if let Some((provider, _)) = self.graph.roles[index].provider {
                    self.provider(provider, sample, nodes)?;
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
        end,
        base,
        copies: HashSet::new(),
        providers: HashSet::new(),
        upper_visits: vec![0; scene.visual_layers.len()],
        upper_source: 0,
        upper_work: 0,
        pixels,
    };
    for (index, role) in graph.roles.iter().enumerate() {
        if role.contributes && !role.matte_only {
            classes.copy(index, root_clock, nodes)?;
        }
    }
    // Function identity is a conservative upper bound. It is sufficient when
    // every shared owner admits it; otherwise resolve actual integer collisions.
    if classes.providers.len() as u64 <= MAX_MATTE_REQUESTS
        && classes.upper_source <= 268_435_456
        && classes.upper_work <= MAX_MATTE_WORK
        && verify(&classes.upper_visits).is_ok()
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
    for &(index, clock) in &classes.copies {
        point(i128::from(clock.low) - clock.offset)?;
        point(i128::from(clock.high) - clock.offset)?;
        let layer = &scene.visual_layers[index];
        let (visible_start, visible_end) = integer_visibility(layer);
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
        .map(|(g, c)| (true, g.0, *c))
        .chain(classes.copies.iter().map(|(i, c)| (false, *i, *c)))
        .collect();
    identities.sort_unstable();
    for group in identities.chunk_by(|a, b| a.0 == b.0 && a.1 == b.1) {
        for (_, _, first) in group {
            for (_, _, second) in group {
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
        graph: &'a EvaluatedMatteGraph,
        copies: HashSet<(usize, u64)>,
        providers: HashSet<(MatteGroupId, u64)>,
        visits: Vec<u64>,
        pixels: u64,
        leaf_work: u64,
        work: u64,
        nodes: &'a mut usize,
    }
    impl Frame<'_> {
        fn provider(&mut self, group: MatteGroupId, time: u64) -> Result<(), crate::CoreError> {
            if self.providers.contains(&(group, time)) {
                return Ok(());
            }
            super::extended_certification::charge(self.nodes)?;
            self.providers.insert((group, time));
            if self.providers.len() as u64 > MAX_MATTE_REQUESTS {
                return Err(invalid("continuous matte provider requests exceed bounds"));
            }
            self.work = add(
                self.work,
                mul(
                    mul(self.pixels, 4)?,
                    self.graph.groups[group.0].members.len() as u64,
                )?,
            )?;
            if self.work > MAX_MATTE_WORK {
                return Err(invalid("continuous matte work exceeds bounds"));
            }
            for slot in 0..self.graph.groups[group.0].members.len() {
                self.copy(self.graph.groups[group.0].members[slot], time)?;
            }
            Ok(())
        }
        fn copy(&mut self, index: usize, time: u64) -> Result<(), crate::CoreError> {
            if self.copies.contains(&(index, time)) {
                return Ok(());
            }
            super::extended_certification::charge(self.nodes)?;
            self.copies.insert((index, time));
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
            self.leaf_work = add(self.leaf_work, mul(self.pixels, times.len() as u64)?)?;
            if self.leaf_work > 268_435_456 {
                return Err(invalid("continuous matte source visits exceed bounds"));
            }
            for time in times {
                if !layer.visible_at(time) {
                    continue;
                }
                self.visits[index] = add(self.visits[index], 1)?;
                if let Some((provider, _)) = self.graph.roles[index].provider {
                    self.work = add(self.work, mul(self.pixels, 5)?)?;
                    if self.work > MAX_MATTE_WORK {
                        return Err(invalid("continuous matte work exceeds bounds"));
                    }
                    self.provider(provider, time)?;
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
            copies: HashSet::new(),
            providers: HashSet::new(),
            visits: vec![0; scene.visual_layers.len()],
            pixels,
            leaf_work: 0,
            work: 0,
            nodes,
        };
        for (index, role) in graph.roles.iter().enumerate() {
            if role.contributes && !role.matte_only {
                frame.copy(index, root)?;
            }
        }
        verify(&frame.visits)?;
    }
    Ok(())
}

pub(crate) fn frame_schedule(
    scene: &super::EvaluatedScene,
    at: u64,
) -> Result<MatteFrameSchedule, crate::CoreError> {
    struct Builder<'a> {
        scene: &'a super::EvaluatedScene,
        graph: &'a EvaluatedMatteGraph,
        tasks: Vec<MatteTask>,
        copies: std::collections::HashMap<(usize, u64), MatteTaskId>,
        providers: std::collections::HashMap<(MatteGroupId, u64), MatteTaskId>,
        requests: u64,
        work: u64,
        leaf_pixels: u64,
        pixels: u64,
        planning_fixed: u64,
        sampled_budget: super::extended_visual::SampledFrameBudget,
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
            if self.requests > MAX_MATTE_REQUESTS {
                return Err(invalid(
                    "matte provider requests exceed output-frame bounds",
                ));
            }
            let member_count = self.graph.groups[group.0].members.len();
            self.work = add(self.work, mul(mul(self.pixels, 4)?, member_count as u64)?)?;
            if self.work > MAX_MATTE_WORK {
                return Err(invalid("matte work exceeds output-frame bounds"));
            }
            self.admit_records(add(self.tasks.len() as u64, member_count as u64)?, 0)?;
            let mut copies = Vec::new();
            for offset in 0..member_count {
                let member = self.graph.groups[group.0].members[offset];
                copies.push(self.copy(member, at, depth)?);
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
        ) -> Result<MatteTaskId, crate::CoreError> {
            if let Some(id) = self.copies.get(&(index, at)) {
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
            if self.leaf_pixels > 268_435_456 {
                return Err(invalid(
                    "linear composition output-frame pixel work exceeds limits",
                ));
            }
            let provider = self.graph.roles[index].provider;

            let mut samples = Vec::new();
            for time in times {
                if !layer.visible_at(time) {
                    samples.push(self.push(MatteTask::LeafSample {
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
                if self.work > MAX_MATTE_WORK {
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
                let (base_memory, mask_memory) =
                    self.sampled_budget.certify(self.scene, index, time)?;
                let mut source_live_bytes = base_memory
                    .checked_sub(add(MATTE_CACHE_RESERVATION, mul(self.pixels, 40)?)?)
                    .ok_or_else(|| invalid("matte source memory certification invalid"))?;
                source_live_bytes = add(source_live_bytes, mul(layer_heap_bytes(layer)?, 3)?)?;
                source_live_bytes = add(source_live_bytes, mask_memory)?;
                samples.push(self.push(MatteTask::LeafSample {
                    layer_index: index,
                    at_ms: time,
                    provider,
                    source_live_bytes,
                })?);
            }
            let id = if samples.len() == 1 {
                samples[0]
            } else {
                self.push(MatteTask::AverageCopy { samples })?
            };
            self.copies.insert((index, at), id);
            Ok(id)
        }
    }
    let graph = scene
        .mattes
        .as_ref()
        .ok_or_else(|| invalid("matte frame requires derived bindings"))?;
    if graph.roles.len() != scene.visual_layers.len() {
        return Err(invalid("matte role count does not match scene"));
    }
    let pixels = mul(
        u64::from(scene.canvas.width),
        u64::from(scene.canvas.height),
    )?;
    let planning_fixed = add(
        add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?,
        add(scene_heap_bytes(scene)?, graph_heap_bytes(graph)?)?,
    )?;
    let mut builder = Builder {
        scene,
        graph,
        tasks: Vec::new(),
        copies: std::collections::HashMap::new(),
        providers: std::collections::HashMap::new(),
        requests: 0,
        work: 0,
        leaf_pixels: 0,
        pixels,
        planning_fixed,
        sampled_budget: super::extended_visual::SampledFrameBudget::new(scene)?,
    };
    let mut direct_draw = Vec::new();
    for (index, role) in graph.roles.iter().enumerate() {
        if role.contributes && !role.matte_only {
            direct_draw.push(builder.copy(index, at, 0)?);
        }
    }
    let mut last_uses = (0..builder.tasks.len()).collect::<Vec<_>>();
    for (index, task) in builder.tasks.iter().enumerate() {
        match task {
            MatteTask::LeafSample {
                provider: Some((id, _)),
                ..
            } => last_uses[id.0] = last_uses[id.0].max(index),
            MatteTask::AverageCopy { samples } => {
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
    for (index, id) in direct_draw.iter().enumerate() {
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
        bytes(direct_draw.capacity(), std::mem::size_of::<MatteTaskId>())?,
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
            MatteTask::AverageCopy { samples } => {
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
    let mut facts = add(
        std::mem::size_of::<EvaluatedMatteGraph>() as u64,
        graph.resource_live_bytes,
    )?;
    facts = add(
        facts,
        bytes(
            graph.groups.capacity(),
            std::mem::size_of::<MatteProviderGroup>(),
        )?,
    )?;
    facts = add(
        facts,
        bytes(
            graph.roles.capacity(),
            std::mem::size_of::<EvaluatedMatteRole>(),
        )?,
    )?;
    facts = add(
        facts,
        bytes(
            graph.provider_first.capacity(),
            std::mem::size_of::<MatteGroupId>(),
        )?,
    )?;
    for group in &graph.groups {
        facts = add(facts, group.authored_item_id.capacity() as u64)?;
        facts = add(
            facts,
            bytes(group.members.capacity(), std::mem::size_of::<usize>())?,
        )?;
    }
    let programs = scene_heap_bytes(scene)?;
    let fixed = add(
        add(
            add(add(MATTE_CACHE_RESERVATION, mul(pixels, 20)?)?, descriptor)?,
            facts,
        )?,
        programs,
    )?;
    let mut releases = vec![0u64; builder.tasks.len() + direct_draw.len()];
    for &last in &last_uses {
        releases[last] = add(releases[last], 1)?;
    }
    let memo_bytes = add(
        mul(
            builder.copies.capacity() as u64,
            2 * (std::mem::size_of::<((usize, u64), MatteTaskId)>() as u64 + 1),
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
    Ok(MatteFrameSchedule {
        canvas: (scene.canvas.width, scene.canvas.height),
        tasks: builder.tasks,
        direct_draw,
        last_uses,
        certificate: MatteFrameCertificate {
            provider_requests: builder.requests,
            matte_work_units: builder.work,
            fixed_live_bytes: fixed,
            descriptor_bytes: descriptor,
            peak_live_bytes: peak,
        },
    })
}

#[cfg(test)]
mod tests {
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
            MatteTask::AverageCopy { samples } => samples.iter().all(|s| s.0 < i),
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
        evaluated.mattes.as_mut().unwrap().resource_live_bytes += initial - 4096;
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
        assert_eq!(evaluated.mattes.as_ref().unwrap().font_payload_bytes, 4096);
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
}
