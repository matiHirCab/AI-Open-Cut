//! Bounded, deterministic certification of newly supported coupled channels.
//! Bounds are conservative; unresolved intervals subdivide left before right.
use super::*;
use crate::{
    AnimationChannel, AnimationChannelProperty as P, AnimationChannelValue as V, VisualEffect,
};

const MAX_NODES: usize = 65_536;
pub(crate) const MAX_EFFECT_PASSES: u64 = 268_435_456;

fn norm(matrix: [f64; 6]) -> f64 {
    let n = matrix[..4]
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0, f64::max);
    if n == 0.0 {
        return 0.0;
    }
    let [a, b, c, d] = std::array::from_fn(|i| matrix[i] / n);
    n * ((a + d).hypot(b - c) + (a - d).hypot(b + c)) * 0.5
}

pub(super) fn transform_magnification(
    layer: &EvaluatedVisualLayer,
    canvas: (u32, u32),
) -> Result<Option<f64>, CoreError> {
    if !super::extended_visual::required(layer) {
        return Ok(None);
    }
    let mut local = layer.transform2d.unwrap_or(crate::Transform2D {
        scale_x: layer.transform.scale,
        scale_y: layer.transform.scale,
        ..Default::default()
    });
    if layer.transform2d.is_none() {
        for property in [
            EvaluatedProperty::Scale,
            EvaluatedProperty::ScaleX,
            EvaluatedProperty::ScaleY,
        ] {
            let frames: Vec<_> = layer
                .keyframes
                .iter()
                .filter(|key| key.property == property)
                .collect();
            let scalar = |key: &EvaluatedKeyframe| match key.value {
                EvaluatedKeyframeValue::Scalar { value } => value,
                _ => 0.0,
            };
            let mut maximum = frames.iter().map(|key| scalar(key)).fold(0.0, f64::max);
            for pair in frames.windows(2) {
                let (low, high) = evaluated_easing_bounds(pair[0].easing);
                let first = scalar(pair[0]);
                let delta = scalar(pair[1]) - first;
                maximum = maximum
                    .max((first + delta * low).clamp(0.000001, 100.0))
                    .max((first + delta * high).clamp(0.000001, 100.0));
            }
            match property {
                EvaluatedProperty::Scale => {
                    local.scale_x = local.scale_x.max(maximum);
                    local.scale_y = local.scale_y.max(maximum);
                }
                EvaluatedProperty::ScaleX => local.scale_x = local.scale_x.max(maximum),
                EvaluatedProperty::ScaleY => local.scale_y = local.scale_y.max(maximum),
                _ => {}
            }
        }
    }
    let mut maximum = norm(transform_matrices(local, (1, 1), canvas)?.0);
    for stage in &layer.ancestor_stages {
        if let Some(animation) = &stage.animation {
            let mut transform = animation.base_transform;
            for channel in &animation.channels {
                if !matches!(channel.property, P::ScaleX | P::ScaleY) {
                    continue;
                }
                let end = channel
                    .keyframes
                    .last()
                    .ok_or_else(|| invalid("ancestor scale sample missing"))?
                    .time_ms;
                let scale = source_scalar_bounds(channel, end)
                    .ok_or_else(|| invalid("ancestor scale envelope missing"))?
                    .1
                    .clamp(0.000001, 100.0);
                match channel.property {
                    P::ScaleX => transform.scale_x = transform.scale_x.max(scale),
                    P::ScaleY => transform.scale_y = transform.scale_y.max(scale),
                    _ => {}
                }
            }
            maximum *= norm(
                transform_matrices(transform, animation.source_canvas, animation.clock.canvas)?.0,
            );
        } else {
            maximum *= norm(stage.matrix);
        }
    }
    if !maximum.is_finite() {
        return Err(invalid("non-finite transformed occurrence norm"));
    }
    Ok(Some(maximum))
}

fn charge(nodes: &mut usize) -> Result<(), CoreError> {
    if *nodes == MAX_NODES {
        return Err(invalid(
            "maxCandidateAnalysisNodes exhausted with unresolved extended safety",
        ));
    }
    *nodes += 1;
    Ok(())
}

fn intervals(
    low: u64,
    high: u64,
    nodes: &mut usize,
    mut certify: impl FnMut(u64, u64) -> Result<bool, CoreError>,
) -> Result<(), CoreError> {
    let mut pending = vec![(low, high)];
    while let Some((low, high)) = pending.pop() {
        charge(nodes)?;
        if certify(low, high)? {
            continue;
        }
        if low == high {
            return Err(invalid("unsafe extended sample geometry"));
        }
        let middle = low + (high - low) / 2;
        pending.push((middle + 1, high));
        pending.push((low, middle));
    }
    Ok(())
}

pub(crate) fn effect_budget(
    size: (u32, u32),
    effects: &[VisualEffect],
    density: f64,
    budget: &mut u64,
) -> Result<(), CoreError> {
    let pad: f64 = effects
        .iter()
        .map(|e| match e {
            VisualEffect::GaussianBlur { radius_px, .. } | VisualEffect::Glow { radius_px, .. } => {
                (3.0 * radius_px * density).ceil()
            }
            _ => 0.0,
        })
        .sum();
    let w = f64::from(size.0) + 2.0 * pad;
    let h = f64::from(size.1) + 2.0 * pad;
    if !w.is_finite() || !h.is_finite() || w > 16384.0 || h > 16384.0 || w * h > 16_777_216.0 {
        return Err(invalid("expanded effect surface exceeds limits"));
    }
    let pixels = (w * h) as u64;
    for effect in effects {
        let passes = match effect {
            VisualEffect::GaussianBlur { radius_px, .. } | VisualEffect::Glow { radius_px, .. }
                if *radius_px > 0.0 =>
            {
                2 * (2 * (3.0 * radius_px * density).ceil() as u64 + 1)
            }
            _ => 1,
        };
        *budget = budget
            .checked_add(
                pixels
                    .checked_mul(passes)
                    .ok_or_else(|| invalid("effect work overflow"))?,
            )
            .filter(|v| *v <= MAX_EFFECT_PASSES)
            .ok_or_else(|| invalid("maxPixelPassesPerSample exceeded"))?;
    }
    Ok(())
}

pub(crate) fn certify_project(project: &Project) -> Result<usize, CoreError> {
    let mut nodes = 0;
    for item in project
        .tracks
        .iter()
        .chain(project.components.iter().flat_map(|c| &c.tracks))
        .flat_map(|t| &t.items)
    {
        let Some(extended) = super::extended_visual::authored(item, project.settings.fps) else {
            continue;
        };
        let end = extended
            .channels
            .iter()
            .filter_map(|c| c.keyframes.last().map(|k| k.time_ms))
            .max()
            .unwrap_or(0)
            .max(item.duration_ms().saturating_sub(1));
        certify_fractional_crop(&extended, item.duration_ms(), &mut nodes)?;
        intervals(0, end, &mut nodes, |low, high| {
            certify_interval(&extended, item, low, high)
        })?;
    }
    Ok(nodes)
}

fn certify_fractional_crop(
    extended: &super::extended_visual::ExtendedVisual,
    duration: u64,
    nodes: &mut usize,
) -> Result<(), CoreError> {
    if !extended.channels.iter().any(|c| {
        matches!(
            c.property,
            P::CropX | P::CropY | P::CropWidth | P::CropHeight
        )
    }) {
        return Ok(());
    }
    let crop = extended.crop.unwrap_or_default();
    let mut pending = vec![(0.0, duration as f64)];
    while let Some((low, high)) = pending.pop() {
        charge(nodes)?;
        let mut safe = true;
        for (position, size, p, w) in [
            (P::CropX, P::CropWidth, crop.x, crop.width),
            (P::CropY, P::CropHeight, crop.y, crop.height),
        ] {
            let a = extended.channels.iter().find(|c| c.property == position);
            let b = extended.channels.iter().find(|c| c.property == size);
            let x = a
                .map_or(Some((p, p)), |c| {
                    crate::animation::scalar_bounds_at(c, low, high)
                })
                .ok_or_else(|| invalid("crop envelope missing"))?;
            let width = b
                .map_or(Some((w, w)), |c| {
                    crate::animation::scalar_bounds_at(c, low, high)
                })
                .ok_or_else(|| invalid("crop envelope missing"))?;
            if x.0 + width.0 > 1.0 || width.1 <= 0.0 {
                return Err(invalid("unsafe fractional crop envelope"));
            }
            if x.1 + width.1 > 1.0
                && a.zip(b)
                    .and_then(|(a, b)| correlated_sum(a, b))
                    .is_none_or(|sum| sum > 1.0)
            {
                safe = false;
            }
        }
        if safe {
            continue;
        }
        let middle = low + (high - low) * 0.5;
        if middle == low || middle == high {
            return Err(invalid("unresolved fractional crop safety"));
        }
        pending.push((middle, high));
        pending.push((low, middle));
    }
    Ok(())
}

/// Certify expanded occurrence surfaces and cumulative worst-case scene work.
/// Retained analysis and expanded occurrences share the same candidate quota.
pub(crate) fn certify_scene(
    scene: &EvaluatedScene,
    project: &Project,
    nodes: &mut usize,
) -> Result<(), CoreError> {
    let mut pixel_work = 0_u64;
    let has_blur = scene.visual_layers.iter().any(|l| {
        l.extended
            .as_ref()
            .and_then(|v| v.motion_blur)
            .is_some_and(crate::MotionBlur::enabled)
    });
    for layer in scene
        .visual_layers
        .iter()
        .filter(|l| has_blur && super::extended_visual::required(l))
    {
        let count = layer
            .extended
            .as_ref()
            .and_then(|v| v.motion_blur)
            .filter(|v| v.enabled())
            .map_or(1, |v| v.sample_count);
        pixel_work = pixel_work
            .checked_add(
                u64::from(scene.canvas.width)
                    .checked_mul(u64::from(scene.canvas.height))
                    .and_then(|v| v.checked_mul(u64::from(count)))
                    .ok_or_else(|| invalid("motion blur pixel work overflow"))?,
            )
            .filter(|v| *v <= crate::MotionBlur::MAX_PIXEL_WORK)
            .ok_or_else(|| invalid("motion blur scene pixel work exceeds limits"))?;
    }
    let mut mask_work = super::masks::MaskFrameBudget::default();
    let scene_program_bytes = super::masks::scene_program_bytes(scene)?;
    let mut mask_retained = 0_u64;
    let mut work = 0;
    let mut segments: usize = scene
        .visual_layers
        .iter()
        .filter(|l| !super::extended_visual::required(l))
        .filter_map(|l| {
            if let EvaluatedVisualSource::Shape(s) = &l.source {
                Some(s.segments())
            } else {
                None
            }
        })
        .sum();
    for layer in scene
        .visual_layers
        .iter()
        .filter(|l| super::extended_visual::required(l))
    {
        charge(nodes)?;
        let size = match &layer.source {
            EvaluatedVisualSource::Shape(shape) => shape.size,
            EvaluatedVisualSource::Media { asset_id, .. } => project
                .assets
                .iter()
                .find(|a| &a.id == asset_id)
                .and_then(|a| a.probe.as_ref())
                .and_then(|p| p.video_width.zip(p.video_height))
                .ok_or_else(|| invalid("extended media requires retained source geometry"))?,
            // Store resolves these facts with verified staged/managed font bytes
            // before publication; the initial transport-free pass has no bytes.
            EvaluatedVisualSource::Text(_) => {
                if let Some(size) = layer.source_size {
                    size
                } else {
                    continue;
                }
            }
            _ => layer
                .source_size
                .unwrap_or((scene.canvas.width, scene.canvas.height)),
        };
        let (mut size, density) = if let EvaluatedVisualSource::Shape(shape) = &layer.source {
            (shape.size, shape.density)
        } else {
            (size, 1.0)
        };
        let mut effects = layer
            .extended
            .as_ref()
            .map_or(vec![], |e| e.effects.clone());
        if let EvaluatedVisualSource::Shape(shape) = &layer.source {
            let mut maximum_segments = shape.segments();
            if let Some(channel) = layer
                .extended
                .as_ref()
                .and_then(|e| e.channels.iter().find(|c| c.property == P::PathPoints))
            {
                let (envelope_size, envelope_segments) = path_envelope(channel, shape)?;
                size.0 = size.0.max(envelope_size.0);
                size.1 = size.1.max(envelope_size.1);
                maximum_segments = maximum_segments.max(envelope_segments);
                let mut local = layer.clone();
                local.instance = None;
                local.span.start_ms = 0;
                let end = channel
                    .keyframes
                    .last()
                    .ok_or_else(|| invalid("path sample missing"))?
                    .time_ms;
                intervals(0, end, nodes, |low, high| {
                    if low != high
                        && channel
                            .keyframes
                            .windows(2)
                            .any(|p| p[0].value != p[1].value)
                    {
                        return Ok(false);
                    }
                    let (sampled, _, _) = super::extended_visual::sample(&local, low)?;
                    if let EvaluatedVisualSource::Shape(shape) = sampled.source {
                        size.0 = size.0.max(shape.size.0);
                        size.1 = size.1.max(shape.size.1);
                        maximum_segments = maximum_segments.max(shape.segments());
                    }
                    Ok(true)
                })?;
            }
            segments = segments
                .checked_add(
                    maximum_segments
                        .checked_mul(
                            layer
                                .extended
                                .as_ref()
                                .and_then(|v| v.motion_blur)
                                .filter(|v| v.enabled())
                                .map_or(1, |v| v.sample_count as usize),
                        )
                        .ok_or_else(|| invalid("motion blur geometry work overflow"))?,
                )
                .filter(|n| *n <= shapes::MAX_SCENE_SEGMENTS)
                .ok_or_else(|| invalid("extended scene segment envelope exceeded"))?;
        }
        if let Some(extended) = &layer.extended {
            let end = extended
                .channels
                .iter()
                .filter_map(|c| c.keyframes.last().map(|k| k.time_ms))
                .max()
                .unwrap_or(0);
            for effect in &mut effects {
                let property = match effect {
                    VisualEffect::GaussianBlur { .. } => P::BlurRadius,
                    VisualEffect::Glow { .. } => P::GlowRadius,
                    _ => continue,
                };
                if let Some(channel) = extended.channels.iter().find(|c| {
                    c.property == property && c.target.as_ref().is_some_and(|t| t.id == effect.id())
                }) {
                    let maximum = source_scalar_bounds(channel, end)
                        .ok_or_else(|| invalid("effect envelope missing"))?
                        .1;
                    match effect {
                        VisualEffect::GaussianBlur { radius_px, .. }
                        | VisualEffect::Glow { radius_px, .. } => *radius_px = maximum,
                        _ => {}
                    }
                }
            }
        }
        let samples = layer
            .extended
            .as_ref()
            .and_then(|v| v.motion_blur)
            .filter(|v| v.enabled())
            .map_or(1, |v| v.sample_count);
        for _ in 0..samples {
            effect_budget(size, &effects, density, &mut work)?;
        }
        if let Some(extended) = &layer.extended
            && !extended.masks.is_empty()
        {
            let mut current_scratch = 0;
            let mut current_retained = 0_u64;
            for mask in extended.masks.iter() {
                let envelope = super::masks::continuous_mask_envelope(
                    mask,
                    &extended.channels,
                    super::masks::MaskOwnerBasis { size, density },
                    nodes,
                )?;
                for _ in 0..samples {
                    mask_work.charge(envelope.work)?;
                }
                segments = segments
                    .checked_add(envelope.segments)
                    .filter(|v| *v <= shapes::MAX_SCENE_SEGMENTS)
                    .ok_or_else(|| invalid("ordinary and mask scene segments exceed limits"))?;
                current_retained = current_retained
                    .checked_add(envelope.retained)
                    .ok_or_else(|| invalid("mask retained facts overflow"))?;
                current_scratch = current_scratch.max(envelope.scratch);
            }
            for _ in 0..samples {
                mask_work.charge(u64::from(size.0) * u64::from(size.1) * 4)?;
            }
            mask_retained = mask_retained
                .checked_add(current_retained)
                .ok_or_else(|| invalid("mask retained scene facts overflow"))?;
            super::extended_visual::certify_composition_memory(
                (scene.canvas.width, scene.canvas.height),
                size,
                &effects,
                density,
            )?
            .checked_add(scene_program_bytes)
            .and_then(|v| v.checked_add(mask_retained))
            .and_then(|v| v.checked_add(current_scratch))
            .and_then(|v| v.checked_add(u64::from(size.0) * u64::from(size.1) * 4))
            .and_then(|v| {
                v.checked_add(
                    super::masks::authored_mask_bytes(&extended.masks)
                        .ok()?
                        .checked_mul(3)?,
                )
            })
            .filter(|v| *v <= super::extended_visual::MAX_COMPOSITION_BYTES)
            .ok_or_else(|| invalid("continuous mask composition memory exceeds limits"))?;
        }
        // A rotation changes direction, never the operator norm. Bounding each
        // ancestor's norm separately covers every combination of inherited clocks.
        if let Some(maximum) =
            transform_magnification(layer, (scene.canvas.width, scene.canvas.height))?
        {
            let axis_aligned = layer.transform2d.is_none_or(|t| {
                t.rotation_deg == 0.0 && t.skew_x_deg == 0.0 && t.skew_y_deg == 0.0
            }) && layer
                .extended
                .as_ref()
                .is_none_or(|e| !e.channels.iter().any(|c| c.property == P::RotationDeg))
                && layer.ancestor_stages.iter().all(|stage| {
                    stage.matrix[1] == 0.0
                        && stage.matrix[2] == 0.0
                        && stage.animation.as_ref().is_none_or(|a| {
                            !a.channels.iter().any(|c| c.property == P::RotationDeg)
                        })
                });
            let (width, height) = if axis_aligned {
                (
                    (f64::from(size.0) * maximum / density).ceil(),
                    (f64::from(size.1) * maximum / density).ceil(),
                )
            } else {
                let diameter =
                    (f64::from(size.0).hypot(f64::from(size.1)) * maximum / density).ceil();
                (diameter, diameter)
            };
            if !width.is_finite()
                || !height.is_finite()
                || width > 16384.0
                || height > 16384.0
                || width * height > 16_777_216.0
            {
                return Err(invalid(
                    "transformed occurrence envelope exceeds raster limits",
                ));
            }
        }
    }
    Ok(())
}

/// Convex control hulls bound every fractional sample, including spring extrema.
/// Subdivision curvature contracts by four at each bisection; the hull diameter
/// therefore supplies a conservative segment/depth bound without sampling.
fn path_envelope(
    channel: &AnimationChannel,
    shape: &shapes::EvaluatedShape,
) -> Result<((u32, u32), usize), CoreError> {
    let mut bounds = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let mut include = |x: f64, y: f64| {
        bounds[0] = bounds[0].min(x);
        bounds[1] = bounds[1].min(y);
        bounds[2] = bounds[2].max(x);
        bounds[3] = bounds[3].max(y);
    };
    for key in &channel.keyframes {
        let V::PathPoints { points } = &key.value else {
            return Err(invalid("path envelope topology missing"));
        };
        for point in points {
            include(point.x, point.y);
        }
    }
    for pair in channel.keyframes.windows(2) {
        let (V::PathPoints { points: a }, V::PathPoints { points: b }) =
            (&pair[0].value, &pair[1].value)
        else {
            return Err(invalid("path envelope topology missing"));
        };
        let (low, high) = crate::animation::curve_bounds(pair[0].curve, 0.0, 1.0);
        for (a, b) in a.iter().zip(b) {
            for t in [low, high] {
                include(
                    (a.x + (b.x - a.x) * t).clamp(-1_000_000.0, 1_000_000.0),
                    (a.y + (b.y - a.y) * t).clamp(-1_000_000.0, 1_000_000.0),
                );
            }
        }
    }
    let diameter = (bounds[2] - bounds[0]).hypot(bounds[3] - bounds[1]);
    let depth =
        ((diameter * shape.density / 0.25).max(1.0).log(4.0).ceil() as u32).saturating_add(1);
    if depth > 16 {
        return Err(invalid(
            "fractional path subdivision envelope exceeds limits",
        ));
    }
    let crate::ShapeGeometry::Path { path } = &shape.geometry else {
        return Err(invalid("path envelope geometry missing"));
    };
    let mut segments = 0usize;
    for command in &path.commands {
        segments = segments
            .checked_add(match command {
                crate::PathCommand::CubicTo { .. } | crate::PathCommand::QuadraticTo { .. } => {
                    1usize << depth
                }
                _ => 1,
            })
            .ok_or_else(|| invalid("path segment envelope overflow"))?;
    }
    if let Some(stroke) = &shape.stroke
        && let Some(shortest) = stroke.dash.iter().copied().reduce(f64::min)
    {
        segments = segments
            .checked_add((segments as f64 * diameter / shortest).ceil() as usize)
            .ok_or_else(|| invalid("path dash envelope overflow"))?;
    }
    if segments > shapes::MAX_SEGMENTS {
        return Err(invalid("fractional path segment envelope exceeds limits"));
    }
    let pad = shape.stroke.as_ref().map_or(1.0 / shape.density, |s| {
        s.width
            * 0.5
            * if s.line_join == crate::LineJoin::Miter {
                s.miter_limit
            } else {
                2.0_f64.sqrt()
            }
            + 1.0 / shape.density
    });
    let width = ((bounds[2] - bounds[0] + 2.0 * pad) * shape.density).ceil() + 2.0;
    let height = ((bounds[3] - bounds[1] + 2.0 * pad) * shape.density).ceil() + 2.0;
    if !width.is_finite()
        || !height.is_finite()
        || width > 16384.0
        || height > 16384.0
        || width * height > 16_777_216.0
    {
        return Err(invalid("fractional path raster envelope exceeds limits"));
    }
    Ok(((width as u32, height as u32), segments))
}

fn bound(
    extended: &super::extended_visual::ExtendedVisual,
    property: P,
    fallback: f64,
    low: u64,
    high: u64,
) -> Result<(f64, f64), CoreError> {
    extended
        .channels
        .iter()
        .find(|c| c.property == property)
        .map_or(Ok((fallback, fallback)), |c| {
            crate::animation::scalar_bounds(c, low, high)
                .ok_or_else(|| invalid("non-finite extended envelope"))
        })
}

// Resource envelopes cover every retained source key, independently of the
// edited window's offset. Applying the item clock here could omit late values
// after a left extension and understate magnification or effect work.
fn source_scalar_bounds(channel: &AnimationChannel, end: u64) -> Option<(f64, f64)> {
    let mut source = channel.clone();
    source.clock = None;
    crate::animation::scalar_bounds(&source, 0, end)
}

// Equal clocks/curves preserve component correlation for non-overshooting
// interpolation. Independent envelopes remain valid for all other combinations.
fn correlated_sum(left: &AnimationChannel, right: &AnimationChannel) -> Option<f64> {
    let held = left.keyframes.windows(2).all(|pair| {
        matches!(
            pair[0].curve,
            crate::AnimationCurve::Simple(crate::SimpleAnimationCurve::Hold)
        )
    });
    if left.r#loop != right.r#loop
        || left.clock != right.clock
        || left.keyframes.len() != right.keyframes.len()
    {
        return None;
    }
    let mut maximum: f64 = 0.0;
    for (a, b) in left.keyframes.iter().zip(&right.keyframes) {
        if a.time_ms != b.time_ms || a.curve != b.curve {
            return None;
        }
        if !matches!(
            a.curve,
            crate::AnimationCurve::Simple(_)
                | crate::AnimationCurve::Parameterized(
                    crate::ParameterizedAnimationCurve::CubicBezier { .. }
                )
        ) {
            return None;
        }
        let (V::Scalar { value: x }, V::Scalar { value: y }) = (&a.value, &b.value) else {
            return None;
        };
        // Exact endpoints may be smaller than the interpolation floor. Equal
        // curves no longer imply correlation when either extent is clamped.
        if [(left.property, *x), (right.property, *y)]
            .iter()
            .any(|(property, value)| {
                !held && matches!(property, P::CropWidth | P::CropHeight) && *value < 0.000001
            })
        {
            return None;
        }
        maximum = maximum.max(x + y);
    }
    Some(maximum)
}

fn certify_interval(
    extended: &super::extended_visual::ExtendedVisual,
    item: &TimelineItem,
    low: u64,
    high: u64,
) -> Result<bool, CoreError> {
    let crop = extended.crop.unwrap_or_default();
    for (position, size, p, w) in [
        (P::CropX, P::CropWidth, crop.x, crop.width),
        (P::CropY, P::CropHeight, crop.y, crop.height),
    ] {
        let a = bound(extended, position, p, low, high)?;
        let b = bound(extended, size, w, low, high)?;
        let correlated = extended
            .channels
            .iter()
            .find(|c| c.property == position)
            .zip(extended.channels.iter().find(|c| c.property == size))
            .and_then(|(a, b)| correlated_sum(a, b));
        if a.0 + b.0 > 1.0 || b.1 <= 0.0 {
            return Err(invalid("unsafe crop envelope"));
        }
        if a.1 + b.1 > 1.0 && correlated.is_none_or(|v| v > 1.0) {
            return Ok(false);
        }
    }
    for channel in extended.channels.iter() {
        if !matches!(channel.property, P::GradientStops | P::MaskGradientStops) {
            continue;
        }
        if low == high {
            let Some(V::GradientStops { stops }) = crate::animation::sample_channel(channel, low)
            else {
                return Err(invalid("gradient sample missing"));
            };
            if !stops.windows(2).all(|p| p[0].offset < p[1].offset) {
                return Err(invalid("unsafe gradient sample"));
            }
        } else {
            for pair in channel.keyframes.windows(2) {
                let (V::GradientStops { stops: a }, V::GradientStops { stops: b }) =
                    (&pair[0].value, &pair[1].value)
                else {
                    return Err(invalid("gradient topology missing"));
                };
                let (p, q) = crate::animation::curve_bounds(pair[0].curve, 0.0, 1.0);
                // Clamped offsets are piecewise linear in curve progress. Their
                // endpoints and clamp breakpoints prove ordering continuously,
                // including fractional inherited clocks between integer samples.
                if !matches!(
                    pair[0].curve,
                    crate::AnimationCurve::Simple(crate::SimpleAnimationCurve::Hold)
                ) {
                    let mut critical = vec![p, q];
                    for (a, b) in a.iter().zip(b) {
                        let delta = b.offset - a.offset;
                        if delta != 0.0 {
                            for boundary in [0.0, 1.0] {
                                let t = (boundary - a.offset) / delta;
                                if (p..=q).contains(&t) {
                                    critical.push(t);
                                }
                            }
                        }
                    }
                    for t in critical {
                        let offsets: Vec<_> = a
                            .iter()
                            .zip(b)
                            .map(|(a, b)| (a.offset + (b.offset - a.offset) * t).clamp(0.0, 1.0))
                            .collect();
                        if !offsets.windows(2).all(|p| p[0] < p[1]) {
                            return Err(invalid("unsafe fractional gradient ordering"));
                        }
                    }
                }
                for (x, y) in a.windows(2).zip(b.windows(2)) {
                    let gap = x[1].offset - x[0].offset;
                    let delta = (y[1].offset - y[0].offset) - gap;
                    if (gap + delta * p).min(gap + delta * q) <= 0.0 {
                        return Ok(false);
                    }
                }
                for (a, b) in a.iter().zip(b) {
                    let min = (a.offset + (b.offset - a.offset) * p)
                        .min(a.offset + (b.offset - a.offset) * q);
                    let max = (a.offset + (b.offset - a.offset) * p)
                        .max(a.offset + (b.offset - a.offset) * q);
                    if min < 0.0 || max > 1.0 {
                        return Ok(false);
                    }
                }
            }
        }
    }
    // Dynamic path geometry is compiled at singleton intervals, preserving the
    // canonical analytic bounds and subdivision/segment limits.
    if let Some(channel) = extended
        .channels
        .iter()
        .find(|c| c.property == P::PathPoints)
    {
        if low != high
            && channel
                .keyframes
                .windows(2)
                .any(|p| p[0].value != p[1].value)
        {
            return Ok(false);
        }
        let TimelineItem::Shape(shape) = item else {
            return Err(invalid("path geometry missing"));
        };
        let layer = EvaluatedVisualLayer {
            extended: Some(extended.clone()),
            sampled_input: None,
            instance: None,
            item_id: shape.id.clone(),
            order: EvaluatedLayerOrder {
                track_index: 0,
                item_index: 0,
            },
            span: checked_span(shape.start_ms, shape.duration_ms)?,
            transform: evaluate_transform(&shape.transform)?,
            transform2d: shape.transform2d,
            affine: None,
            sampling_tiles: None,
            ancestors: None,
            ancestor_stages: vec![],
            source_size: None,
            keyframes: vec![],
            transitions: vec![],
            source: EvaluatedVisualSource::Shape(Box::new(shapes::EvaluatedShape::new(
                shape.geometry.clone(),
                shape.fill.clone(),
                shape.stroke.clone(),
                1.0,
            )?)),
        };
        let (sampled, _, _) = super::extended_visual::sample(
            &layer,
            low.checked_add(shape.start_ms)
                .ok_or_else(|| invalid("sample time overflow"))?,
        )?;
        if let EvaluatedVisualSource::Shape(shape) = sampled.source {
            effect_budget(shape.size, &extended.effects, shape.density, &mut 0)?;
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_retained_clocks_cannot_hide_late_ancestor_or_effect_work() {
        use serde_json::json;
        let channel = |property: &str, first: f64, last: f64| {
            json!({
                "property":property,"clock":{"offsetMs":-500,"sourceDurationMs":1100},
                "keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first},"curve":"linear"},{"timeMs":1000,"value":{"type":"scalar","value":last},"curve":"hold"}]
            })
        };
        let project = |items: serde_json::Value| -> Project {
            serde_json::from_value(json!({
            "schemaVersion":crate::PROJECT_SCHEMA_VERSION,"id":"budget","revision":0,"name":"Budget","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":64,"height":64,"fps":10},"assets":[],"markers":[],"components":[],"fonts":{},
            "tracks":[{"id":"overlay","name":"Overlay","trackType":"overlay","items":items}]
        })).unwrap()
        };
        let ancestor = project(json!([
            {"type":"group","id":"parent","startMs":0,"durationMs":1500,"zIndex":0,"stackOrder":0,"animationChannels":[channel("transform.scale_x",1.0,100.0),channel("transform.scale_y",1.0,100.0)]},
            {"type":"rectangle","id":"child","startMs":0,"durationMs":1500,"zIndex":0,"stackOrder":1,"width":60,"height":60,"color":"#ffffff","parent":{"scope":"root","id":"parent"},"keyframes":[],"effects":[{"id":"identity","type":"vignette","amount":0}]}
        ]));
        let mut small = ancestor.clone();
        if let TimelineItem::Rectangle(rect) = &mut small.tracks[0].items[1] {
            rect.width = 1;
            rect.height = 1;
        }
        let evaluated = evaluate_project(&small, 64, 64, 10).unwrap();
        let maximum = transform_magnification(&evaluated.scene.visual_layers[0], (64, 64))
            .unwrap()
            .unwrap();
        assert_eq!(
            maximum, 100.0,
            "retained source last scale must remain in the ancestor envelope"
        );
        let error = evaluate_project(&ancestor, 64, 64, 10).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(
            error.message.contains("bounds") || error.message.contains("envelope"),
            "{}",
            error.message
        );
        for (kind, property) in [
            ("gaussian_blur", "effect.blur_radius"),
            ("glow", "effect.glow_radius"),
        ] {
            let mut effect = json!({"id":"radius","type":kind,"radiusPx":0});
            if kind == "glow" {
                effect["intensity"] = json!(0.5);
                effect["color"] = json!({"r":1,"g":0,"b":0,"a":1});
            }
            let mut radius = channel(property, 0.0, 64.0);
            radius["target"] = json!({"scope":"root","kind":"effect","id":"radius"});
            let p = project(
                json!([{ "type":"rectangle","id":"effect","startMs":0,"durationMs":1500,"zIndex":0,"stackOrder":0,"width":300,"height":300,"color":"#ffffff","keyframes":[],"effects":[effect],"animationChannels":[radius]}]),
            );
            let error = preflight_inherited_project(&p).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(
                error.message.contains("effect") || error.message.contains("PixelPasses"),
                "{}",
                error.message
            );
        }
    }

    #[test]
    fn node_quota_is_inclusive_and_left_subdivision_is_deterministic() {
        let mut nodes = 0;
        let mut visited = vec![];
        intervals(0, 3, &mut nodes, |low, high| {
            visited.push((low, high));
            Ok(low == high)
        })
        .unwrap();
        assert_eq!(
            visited,
            [(0, 3), (0, 1), (0, 0), (1, 1), (2, 3), (2, 2), (3, 3)]
        );
        let mut nodes = MAX_NODES - 1;
        intervals(0, 0, &mut nodes, |_, _| Ok(true)).unwrap();
        assert_eq!(nodes, MAX_NODES);
        assert_eq!(
            intervals(0, 0, &mut nodes, |_, _| Ok(true))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        let mut nodes = MAX_NODES - 1;
        assert_eq!(
            intervals(0, u64::MAX, &mut nodes, |_, _| Ok(false))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(nodes, MAX_NODES);
    }
    #[test]
    fn scene_effect_work_is_cumulative_and_overflow_fails_closed() {
        let effect = VisualEffect::ColorTint {
            id: "tint".into(),
            color: crate::VectorColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            },
        };
        let mut work = MAX_EFFECT_PASSES - 1;
        effect_budget((1, 1), std::slice::from_ref(&effect), 1.0, &mut work).unwrap();
        assert_eq!(work, MAX_EFFECT_PASSES);
        assert_eq!(
            effect_budget((1, 1), &[effect], 1.0, &mut work)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            effect_budget((u32::MAX, u32::MAX), &[], 1.0, &mut 0)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
}
