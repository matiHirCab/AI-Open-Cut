//! Shared immutable admission facts for bounded composition, independent of matte bindings.
use super::mattes::{MATTE_CACHE_RESERVATION, MAX_MATTE_LIVE_BYTES};
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CompositionResourceFacts {
    pub resource_live_bytes: u64,
    pub font_payload_bytes: u64,
}
fn invalid(message: &str) -> crate::CoreError {
    crate::CoreError::new(crate::ErrorCode::InvalidArgument, message)
}
pub(crate) fn composition_heap_bytes(
    scene: &super::EvaluatedScene,
) -> Result<u64, crate::CoreError> {
    let mut bytes = scene_heap_bytes(scene)?;
    if let Some(graph) = &scene.mattes {
        bytes = add(bytes, super::mattes::graph_heap_bytes(graph)?)?;
    }
    if let Some(resources) = &scene.composition_resources {
        bytes = add(bytes, resources.resource_live_bytes)?;
    }
    Ok(bytes)
}
pub(crate) fn font_payload_admission(
    scene: &super::EvaluatedScene,
) -> Result<u64, crate::CoreError> {
    let graph = scene
        .composition_resources
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
        composition_heap_bytes(scene)?,
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
    if scene.composition_resources.is_none() {
        return Ok(());
    }
    if bytes > font_payload_admission(scene)? {
        return Err(invalid("matte font payload exceeds shared memory bounds"));
    }
    let graph = scene.composition_resources.as_mut().unwrap();
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
        .composition_resources
        .as_ref()
        .ok_or_else(|| invalid("matte resource facts unavailable"))?;
    let heap = composition_heap_bytes(scene)?
        .checked_sub(graph.resource_live_bytes)
        .ok_or_else(|| invalid("matte caller memory invalid"))?;
    let available = font_payload_admission(scene)?;
    if add(heap, graph.font_payload_bytes)? > available {
        return Err(invalid(
            "matte caller scene clone exceeds shared memory bounds",
        ));
    }
    Ok(heap)
}

pub(super) fn add(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_add(b)
        .ok_or_else(|| invalid("matte resource arithmetic overflow"))
}
pub(super) fn mul(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_mul(b)
        .ok_or_else(|| invalid("matte resource arithmetic overflow"))
}
pub(super) fn capacity_bytes<T>(items: &Vec<T>) -> Result<u64, crate::CoreError> {
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
