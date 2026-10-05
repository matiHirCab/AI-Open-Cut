//! Process-local sampled visual facts; no paths, processes or persisted mutation.
use super::*;
use crate::{
    AnimationChannel, AnimationChannelProperty as P, AnimationChannelValue as V,
    AnimationTargetKind as K, MediaCrop, VisualEffect,
};

#[cfg(test)]
mod review_regressions;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExtendedVisual {
    pub crop: Option<MediaCrop>,
    pub motion_blur: Option<crate::MotionBlur>,
    pub frame_rate: u32,
    pub effects: Vec<VisualEffect>,
    pub channels: Vec<AnimationChannel>,
}

pub(crate) fn authored(item: &TimelineItem, frame_rate: u32) -> Option<ExtendedVisual> {
    let visual = item.visual_properties();
    let channels: Vec<_> = visual
        .animation_channels
        .iter()
        .filter(|c| c.property.extended() && !c.keyframes.is_empty())
        .cloned()
        .collect();
    (visual.crop.is_some()
        || !visual.effects.is_empty()
        || !channels.is_empty()
        || visual.motion_blur.is_some_and(crate::MotionBlur::enabled))
    .then(|| ExtendedVisual {
        crop: visual.crop,
        motion_blur: visual.motion_blur,
        frame_rate,
        effects: visual.effects.clone(),
        channels,
    })
}

pub(crate) fn sample(
    layer: &EvaluatedVisualLayer,
    at_ms: u64,
) -> Result<(EvaluatedVisualLayer, Option<MediaCrop>, Vec<VisualEffect>), CoreError> {
    let mut sampled = layer.clone();
    let time = crate::animation::SampleTime::local(
        at_ms,
        layer.span.start_ms,
        layer.instance.map(|i| (i.rate, i.offset)),
    );
    let Some(authored) = &layer.extended else {
        return Ok((sampled, None, vec![]));
    };
    let mut crop = authored.crop;
    let mut effects = authored.effects.clone();
    let mut shape = if authored
        .channels
        .iter()
        .any(|c| matches!(c.property, P::PathPoints | P::PathTrim | P::GradientStops))
    {
        match &layer.source {
            EvaluatedVisualSource::Shape(shape) => Some((
                shape.geometry.clone(),
                shape.fill.clone(),
                shape.stroke.clone(),
                shape.density,
            )),
            _ => None,
        }
    } else {
        None
    };
    let mut trim = 1.0;
    for channel in &authored.channels {
        let value = time
            .sample(channel)
            .ok_or_else(|| invalid("non-finite extended sample"))?;
        match (channel.property, value) {
            (P::RotationDeg, V::Scalar { .. }) => {} // Applied with the legacy anchor after source measurement.
            (P::CropX | P::CropY | P::CropWidth | P::CropHeight, V::Scalar { value }) => {
                let crop = crop.get_or_insert_with(MediaCrop::default);
                match channel.property {
                    P::CropX => crop.x = value,
                    P::CropY => crop.y = value,
                    P::CropWidth => crop.width = value,
                    P::CropHeight => crop.height = value,
                    _ => unreachable!(),
                }
            }
            (P::PathPoints, V::PathPoints { points }) => {
                let Some((crate::ShapeGeometry::Path { path }, _, _, _)) = &mut shape else {
                    return Err(invalid("sampled path missing"));
                };
                let mut points = points.into_iter();
                let mut next = || {
                    points
                        .next()
                        .map(|p| crate::VectorPoint { x: p.x, y: p.y })
                        .ok_or_else(|| invalid("path sample topology mismatch"))
                };
                for command in &mut path.commands {
                    match command {
                        crate::PathCommand::MoveTo { to } | crate::PathCommand::LineTo { to } => {
                            *to = next()?
                        }
                        crate::PathCommand::QuadraticTo { control, to } => {
                            *control = next()?;
                            *to = next()?;
                        }
                        crate::PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => {
                            *control1 = next()?;
                            *control2 = next()?;
                            *to = next()?;
                        }
                        crate::PathCommand::Close {} => {}
                    }
                }
            }
            (P::PathTrim, V::Scalar { value }) => trim = value,
            (P::GradientStops, V::GradientStops { stops }) => {
                let Some((_, fill, stroke, _)) = &mut shape else {
                    return Err(invalid("sampled gradient missing"));
                };
                let paint = match channel.target.as_ref().map(|t| t.kind) {
                    Some(K::GraphicFill) => fill.as_mut(),
                    Some(K::GraphicStroke) => stroke.as_mut().map(|s| &mut s.paint),
                    _ => None,
                }
                .ok_or_else(|| invalid("sampled paint missing"))?;
                let values: Vec<_> = stops
                    .iter()
                    .map(|s| crate::GradientStop {
                        offset: s.offset,
                        color: crate::VectorColor {
                            r: s.color[0],
                            g: s.color[1],
                            b: s.color[2],
                            a: s.color[3],
                        },
                    })
                    .collect();
                if !values.windows(2).all(|v| v[0].offset < v[1].offset) {
                    return Err(invalid("sampled gradient order invalid"));
                }
                match paint {
                    crate::Paint::LinearGradient { stops, .. }
                    | crate::Paint::RadialGradient { stops, .. } => *stops = values,
                    _ => return Err(invalid("sampled paint not a gradient")),
                }
                paint.validate()?;
            }
            (property, value) => {
                let id = &channel
                    .target
                    .as_ref()
                    .ok_or_else(|| invalid("effect sample target missing"))?
                    .id;
                let effect = effects
                    .iter_mut()
                    .find(|e| e.id() == id)
                    .ok_or_else(|| invalid("effect sample missing"))?;
                match (property, value, effect) {
                    (
                        P::BlurRadius,
                        V::Scalar { value },
                        VisualEffect::GaussianBlur { radius_px, .. },
                    )
                    | (P::GlowRadius, V::Scalar { value }, VisualEffect::Glow { radius_px, .. }) => {
                        *radius_px = value
                    }
                    (
                        P::VignetteAmount,
                        V::Scalar { value },
                        VisualEffect::Vignette { amount, .. },
                    ) => *amount = value,
                    (
                        P::TintColor,
                        V::Rgba { r, g, b, a },
                        VisualEffect::ColorTint { color, .. },
                    ) => *color = crate::VectorColor { r, g, b, a },
                    _ => return Err(invalid("effect sample incompatible")),
                }
            }
        }
    }
    if let Some(crop) = crop {
        crate::validation::extended_visual::validate_crop(crop)?;
    }
    if let Some((geometry, fill, stroke, density)) = shape {
        geometry.validate()?;
        let mut evaluated = shapes::EvaluatedShape::new(geometry, fill, stroke, density)?;
        if trim < 1.0 {
            evaluated.fill = None;
            for contour in &mut evaluated.contours {
                let mut points = contour.points.clone();
                if contour.closed && !points.is_empty() {
                    points.push(points[0]);
                }
                let total: f64 = points
                    .windows(2)
                    .map(|p| (p[1].x - p[0].x).hypot(p[1].y - p[0].y))
                    .sum();
                let mut remaining = total * trim;
                let mut prefix = points.first().copied().into_iter().collect::<Vec<_>>();
                for pair in points.windows(2) {
                    let length = (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y);
                    if remaining >= length {
                        prefix.push(pair[1]);
                        remaining -= length;
                    } else {
                        if remaining > 0.0 && length > 0.0 {
                            let t = remaining / length;
                            prefix.push(crate::VectorPoint {
                                x: pair[0].x + (pair[1].x - pair[0].x) * t,
                                y: pair[0].y + (pair[1].y - pair[0].y) * t,
                            });
                        }
                        break;
                    }
                }
                contour.points = prefix;
                contour.closed = false;
            }
        }
        sampled.source = EvaluatedVisualSource::Shape(Box::new(evaluated));
    }
    Ok((sampled, crop, effects))
}

pub(crate) fn required(layer: &EvaluatedVisualLayer) -> bool {
    layer.extended.is_some()
        || layer.ancestor_stages.iter().any(|s| {
            s.animation.as_ref().is_some_and(|a| {
                a.channels
                    .iter()
                    .any(|c| c.property == P::RotationDeg && !c.keyframes.is_empty())
            })
        })
}

/// Keep codec/fidelity eligibility separate from requested-origin sampling.
pub(crate) fn sampling_required(_layer: &EvaluatedVisualLayer, _start: u64, _fps: u32) -> bool {
    true // All visual sources share the normative linear composition pipeline.
}

/// Validate requested samples and cumulative scene work before output inspection
/// or workspace creation. This consumes the same facts as resource preparation.
/// Same source limits as the existing local raster allocation, before side effects.
pub(crate) fn validate_sampled_source_size(
    (width, height): (u32, u32),
) -> Result<usize, CoreError> {
    u64::from(width)
        .checked_mul(u64::from(height))
        .filter(|pixels| {
            width > 0 && height > 0 && width <= 16384 && height <= 16384 && *pixels <= 16_777_216
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or_else(|| invalid("sampled source raster exceeds limits"))
}

/// Populate strict intrinsic dimensions only when sampled paint consumes them.
/// Plain Caption paint already contains its local bottom-center placement.
pub(crate) fn finalize_intrinsic_sources(scene: &mut EvaluatedScene, start: u64) {
    let canvas = (scene.canvas.width, scene.canvas.height);
    let fps = scene.canvas.fps;
    for layer in &mut scene.visual_layers {
        if (matches!(layer.source, EvaluatedVisualSource::Caption(_))
            || (matches!(layer.source, EvaluatedVisualSource::SolidColor { .. })
                && layer.source_size.is_none()))
            && !layer.requires_affine()
            && sampling_required(layer, start, fps)
        {
            layer.source_size = Some(layer.instance.map_or(canvas, |i| i.canvas));
        }
    }
}

pub(crate) fn preflight_samples(
    scene: &EvaluatedScene,
    start: u64,
    end: u64,
    frame: bool,
) -> Result<(), CoreError> {
    let canvas = (scene.canvas.width, scene.canvas.height);
    if u64::from(canvas.0) * u64::from(canvas.1) > 16_777_216 {
        return Err(invalid("sampled output surface exceeds limits"));
    }
    certify_composition_memory(canvas, (1, 1), &[], 1.0)?;
    if scene.visual_layers.len() > 4096 {
        return Err(invalid("composition visual occurrence limit exceeded"));
    }
    for layer in scene
        .visual_layers
        .iter()
        .filter(|l| sampling_required(l, start, scene.canvas.fps))
    {
        let size = match &layer.source {
            EvaluatedVisualSource::Shape(shape) => shape.size,
            _ => layer
                .source_size
                .ok_or_else(|| invalid("sampled source measurement missing"))?,
        };
        validate_sampled_source_size(size)?;
    }
    let count = if frame {
        1
    } else {
        (end - start)
            .checked_mul(u64::from(scene.canvas.fps))
            .ok_or_else(|| invalid("sample count overflow"))?
            .div_ceil(1000)
            .max(1)
    };
    for index in 0..count {
        let time = start
            .checked_add(
                index
                    .checked_mul(1000)
                    .ok_or_else(|| invalid("sample time overflow"))?
                    / u64::from(scene.canvas.fps),
            )
            .ok_or_else(|| invalid("sample time overflow"))?;
        let mut work = 0;
        let mut segments = 0;
        let mut pixel_work = 0_u64;
        for layer in scene
            .visual_layers
            .iter()
            .filter(|l| sampling_required(l, start, scene.canvas.fps))
        {
            let times = layer
                .extended
                .as_ref()
                .and_then(|v| v.motion_blur)
                .map_or_else(
                    || Ok(vec![time]),
                    |settings| {
                        settings.sample_times(
                            time,
                            layer.extended.as_ref().unwrap().frame_rate,
                            scene.duration_ms,
                        )
                    },
                )?;
            {
                pixel_work = pixel_work
                    .checked_add(
                        u64::from(canvas.0)
                            .checked_mul(u64::from(canvas.1))
                            .and_then(|v| v.checked_mul(times.len() as u64))
                            .ok_or_else(|| invalid("motion blur pixel work overflow"))?,
                    )
                    .filter(|v| *v <= 268_435_456)
                    .ok_or_else(|| {
                        invalid("linear composition output-frame pixel work exceeds limits")
                    })?;
            }
            for time in times {
                if !layer.visible_at(time) {
                    continue;
                }
                if let EvaluatedVisualSource::Media {
                    asset_id,
                    source_in_ms,
                } = &layer.source
                    && !scene
                        .resources
                        .iter()
                        .any(|r| r.asset_id == *asset_id && r.kind == EvaluatedMediaKind::Image)
                {
                    certified_media_source_time(layer, time, *source_in_ms)?;
                }
                let (mut sampled, _, effects) = sample(layer, time)?;
                let (size, density) = if let EvaluatedVisualSource::Shape(shape) = &sampled.source {
                    segments += shape.segments();
                    if segments > shapes::MAX_SCENE_SEGMENTS {
                        return Err(invalid("sampled scene segment limit exceeded"));
                    }
                    (shape.size, shape.density)
                } else {
                    (
                        sampled
                            .source_size
                            .ok_or_else(|| invalid("sampled source measurement missing"))?,
                        1.0,
                    )
                };
                super::extended_certification::effect_budget(size, &effects, density, &mut work)?;
                certify_composition_memory(canvas, size, &effects, density)?;
                sample_transform(&mut sampled, time, size, canvas)?;
            }
        }
    }
    Ok(())
}

// Preserve authored validity while certifying the floating native timestamp
// boundary. TwoSum measures loss, including partial quantization of a fraction.
pub(crate) fn certified_media_source_time(
    layer: &EvaluatedVisualLayer,
    at: u64,
    source_in: u64,
) -> Result<f64, CoreError> {
    let local = layer
        .instance
        .map_or(at.saturating_sub(layer.span.start_ms) as f64, |i| {
            let relative = (i128::from(at) - i128::from(layer.span.start_ms)) as f64;
            i.rate.mul_add(
                relative,
                i.rate.mul_add(
                    layer.span.start_ms as f64,
                    i.offset - layer.span.start_ms as f64,
                ),
            )
        });
    if !local.is_finite() {
        return Err(invalid("invalid native media source clock"));
    }
    let local = local.max(0.0);
    let whole = source_in as f64;
    let sum = whole + local;
    if !sum.is_finite() {
        return Err(invalid("invalid native media source clock"));
    }
    if local.fract() != 0.0 {
        let virtual_local = sum - whole;
        let residual = (whole - (sum - virtual_local)) + (local - virtual_local);
        let seconds = sum / 1000.0;
        let recovered = seconds * 1000.0;
        let conversion = (recovered - sum) + seconds.mul_add(1000.0, -recovered);
        // Original PTS*TB comparison and native seconds use binary64. Bound
        // their rounding precision even when a seconds roundtrip is exact.
        let comparison = (seconds.next_up() - seconds) * 1000.0;
        if residual.abs() + conversion.abs() + comparison > 0.000001 {
            return Err(invalid(
                "fractional native media source clock exceeds precision limits",
            ));
        }
    }
    Ok(sum)
}

// Conservative peak, including rasterizer scratch, RGBA decode/serialization buffers,
// three output working rasters (scene, shutter average, layer), and the shared cache.
const MAX_COMPOSITION_BYTES: u64 = 1_073_741_824;
const CACHE_RESERVATION_BYTES: u64 = 67_108_864;
pub(crate) fn certify_composition_memory(
    canvas: (u32, u32),
    source: (u32, u32),
    effects: &[VisualEffect],
    density: f64,
) -> Result<u64, CoreError> {
    let pad = effects
        .iter()
        .map(|effect| match effect {
            VisualEffect::GaussianBlur { radius_px, .. } | VisualEffect::Glow { radius_px, .. } => {
                (3.0 * radius_px * density).ceil()
            }
            _ => 0.0,
        })
        .sum::<f64>();
    if !pad.is_finite() || !(0.0..=16384.0).contains(&pad) {
        return Err(invalid("composition effect support exceeds limits"));
    }
    let pad = (pad as u64)
        .checked_mul(2)
        .ok_or_else(|| invalid("composition support overflow"))?;
    let pixels = |size: (u32, u32)| u64::from(size.0).checked_mul(u64::from(size.1));
    let expanded = u64::from(source.0)
        .checked_add(pad)
        .and_then(|w| {
            u64::from(source.1)
                .checked_add(pad)
                .and_then(|h| w.checked_mul(h))
        })
        .ok_or_else(|| invalid("composition memory overflow"))?;
    // Empty effects return the existing raster without allocating padding or
    // convolution scratch. Reserve one f32 expanded raster in that case; active
    // effects reserve three f32 rasters plus serialization/coverage scratch.
    // 48/source pixel separately bounds source/crop rasters, decoder buffers
    // and SVG rasterization, including their simultaneous allocation phases.
    let expanded_bytes_per_pixel = if effects.is_empty() { 16 } else { 64 };
    let bytes = pixels(canvas)
        .and_then(|p| p.checked_mul(3 * 16 + 2 * 4))
        .and_then(|b| {
            expanded
                .checked_mul(expanded_bytes_per_pixel)
                .and_then(|v| b.checked_add(v))
        })
        .and_then(|b| {
            pixels(source)
                .and_then(|p| p.checked_mul(48))
                .and_then(|v| b.checked_add(v))
        })
        .and_then(|b| b.checked_add(CACHE_RESERVATION_BYTES + 1_048_576))
        .filter(|b| *b <= MAX_COMPOSITION_BYTES)
        .ok_or_else(|| invalid("linear composition live memory exceeds limits"))?;
    Ok(bytes)
}

pub(crate) fn sample_transform(
    layer: &mut EvaluatedVisualLayer,
    at_ms: u64,
    source: (u32, u32),
    canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    let legacy_basis = layer.transform2d.is_none();
    let time = crate::animation::SampleTime::local(
        at_ms,
        layer.span.start_ms,
        layer.instance.map(|i| (i.rate, i.offset)),
    );
    let scalar = |property, default| {
        sample_scalar(&layer.keyframes, property, time, false).unwrap_or(default)
    };
    let caption_position = if let EvaluatedVisualSource::Caption(caption) = &layer.source {
        if layer.transform2d.is_none() {
            Some(if layer.requires_affine() {
                caption_legacy_position(
                    caption,
                    source,
                    layer.instance.map_or(canvas, |i| i.canvas),
                )
            } else {
                (0.0, 0.0)
            })
        } else {
            None
        }
    } else {
        None
    };
    let mut transform = layer.transform2d.unwrap_or(crate::Transform2D {
        position: crate::TransformPosition {
            x: layer.transform.position_x,
            y: layer.transform.position_y,
            unit: crate::PositionUnit::Pixels,
        },
        scale_x: layer.transform.scale,
        scale_y: layer.transform.scale,
        opacity: layer.transform.opacity,
        ..Default::default()
    });
    if layer.transform2d.is_none() {
        let (ax, ay) = layer.legacy_anchor(source);
        let (ax, ay) = if let Some([bx, by, w, h]) = layer.text_logical_box() {
            ((ax - bx) / w, (ay - by) / h)
        } else {
            (ax / f64::from(source.0), ay / f64::from(source.1))
        };
        // Shapes account for their raster origin in shapes::affine.
        if !matches!(layer.source, EvaluatedVisualSource::Shape(_)) {
            transform.anchor = crate::TransformAnchor { x: ax, y: ay };
        }
        transform.position.x = scalar(
            EvaluatedProperty::PositionX,
            scalar(EvaluatedProperty::Position, layer.transform.position_x),
        );
        transform.position.y = scalar(
            EvaluatedProperty::PositionY,
            sample_scalar(&layer.keyframes, EvaluatedProperty::Position, time, true)
                .unwrap_or(layer.transform.position_y),
        );
        transform.scale_x = scalar(
            EvaluatedProperty::ScaleX,
            scalar(EvaluatedProperty::Scale, layer.transform.scale),
        );
        transform.scale_y = scalar(
            EvaluatedProperty::ScaleY,
            scalar(EvaluatedProperty::Scale, layer.transform.scale),
        );
        transform.opacity = scalar(EvaluatedProperty::Opacity, layer.transform.opacity);
    }
    if let Some((x, y)) = caption_position {
        transform.position.x = x;
        transform.position.y = y;
    }
    if let Some(channel) = layer.extended.as_ref().and_then(|e| {
        e.channels
            .iter()
            .find(|c| c.property == P::RotationDeg && !c.keyframes.is_empty())
    }) {
        transform.rotation_deg = time
            .scalar(channel)
            .ok_or_else(|| invalid("non-finite rotation sample"))?;
    }
    layer.transform2d = Some(transform);
    layer.keyframes.clear();
    let mut matrix = IDENTITY_MATRIX;
    let mut inverse = IDENTITY_MATRIX;
    let mut opacity = 1.0;
    for stage in &layer.ancestor_stages {
        let (stage_matrix, stage_inverse, stage_opacity) = if let Some(animation) = &stage.animation
        {
            let time = crate::animation::SampleTime::local(
                at_ms,
                animation.start_ms,
                Some((animation.clock.rate, animation.clock.offset)),
            );
            let mut transform = animation.base_transform;
            for channel in &animation.channels {
                let Some(value) = time.scalar(channel) else {
                    continue;
                };
                match channel.property {
                    P::PositionX => transform.position.x = value,
                    P::PositionY => transform.position.y = value,
                    P::ScaleX => transform.scale_x = value,
                    P::ScaleY => transform.scale_y = value,
                    P::Opacity => transform.opacity = value,
                    P::RotationDeg => transform.rotation_deg = value,
                    _ => {}
                }
            }
            let (matrix, inverse) =
                transform_matrices(transform, animation.source_canvas, animation.clock.canvas)?;
            (matrix, inverse, transform.opacity)
        } else {
            (stage.matrix, stage.inverse, stage.opacity)
        };
        matrix = multiply_matrix(matrix, stage_matrix);
        inverse = multiply_matrix(stage_inverse, inverse);
        opacity *= stage_opacity;
    }
    if !layer.ancestor_stages.is_empty() {
        layer.ancestors = Some(EvaluatedAncestors {
            matrix,
            inverse,
            opacity,
            clip: layer.ancestors.map(|a| a.clip).unwrap_or(layer.span),
        });
    }
    layer.ancestor_stages.clear();
    layer.sampling_tiles = None;
    if legacy_basis && let EvaluatedVisualSource::Shape(shape) = &layer.source {
        return shapes::affine_in_legacy_basis(layer, shape, canvas);
    }
    evaluate_layer_affine(layer, source, canvas)
}

fn sample_scalar(
    keyframes: &[EvaluatedKeyframe],
    property: EvaluatedProperty,
    time: crate::animation::SampleTime,
    y_axis: bool,
) -> Option<f64> {
    let frames: Vec<_> = keyframes
        .iter()
        .filter(|k| k.property == property)
        .collect();
    let first = *frames.first()?;
    let scalar = |frame: &EvaluatedKeyframe| match frame.value {
        EvaluatedKeyframeValue::Scalar { value } => Some(value),
        EvaluatedKeyframeValue::Position { x, y } => Some(if y_axis { y } else { x }),
    };
    let channel = AnimationChannel {
        property: P::PositionX,
        target: None,
        keyframes: frames
            .iter()
            .map(|k| crate::AnimationChannelKeyframe {
                time_ms: k.time_ms,
                value: V::Scalar {
                    value: scalar(k).unwrap_or(0.0),
                },
                curve: crate::AnimationCurve::Simple(crate::SimpleAnimationCurve::Linear),
            })
            .collect(),
        r#loop: first.r#loop,
        clock: first.clock,
    };
    let time = time.looped(&channel)?;
    if time.compare(first.time_ms)? != std::cmp::Ordering::Greater {
        return scalar(first);
    }
    for pair in frames.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if time.compare(b.time_ms)? == std::cmp::Ordering::Equal {
            return scalar(b);
        }
        if time.compare(b.time_ms)? == std::cmp::Ordering::Less {
            let t = time.progress(a.time_ms, b.time_ms);
            let t = match a.easing {
                EvaluatedEasing::Hold => 0.0,
                EvaluatedEasing::Linear => t,
                EvaluatedEasing::EaseIn => t * t,
                EvaluatedEasing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
                EvaluatedEasing::EaseInOut => {
                    if t < 0.5 {
                        2.0 * t * t
                    } else {
                        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                    }
                }
                EvaluatedEasing::CubicBezier { x1, y1, x2, y2 } => {
                    crate::animation::parameterized_curve_progress(
                        crate::ParameterizedAnimationCurve::CubicBezier { x1, y1, x2, y2 },
                        t,
                    )
                }
                EvaluatedEasing::Spring {
                    mass,
                    stiffness,
                    damping,
                    initial_velocity,
                } => crate::animation::parameterized_curve_progress(
                    crate::ParameterizedAnimationCurve::Spring {
                        mass,
                        stiffness,
                        damping,
                        initial_velocity,
                    },
                    t,
                ),
            };
            let value = scalar(a)? + (scalar(b)? - scalar(a)?) * t;
            return value.is_finite().then_some(match property {
                EvaluatedProperty::Scale
                | EvaluatedProperty::ScaleX
                | EvaluatedProperty::ScaleY => value.clamp(0.000001, 100.0),
                EvaluatedProperty::Opacity => value.clamp(0.0, 1.0),
                _ => value.clamp(-1_000_000.0, 1_000_000.0),
            });
        }
    }
    scalar(frames.last()?)
}
