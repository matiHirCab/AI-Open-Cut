//! Renderer-neutral sampled mask facts and bounded certification.
#[cfg(test)]
mod tests;
use super::shapes::Contour;
use crate::{FillRule, MaskChannel, MaskOperation, Paint};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MaskOwnerBasis {
    pub size: (u32, u32),
    /// Source raster pixels per local project pixel; the local domain starts at0,0.
    pub density: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaskGrid {
    pub contours: Vec<Contour>,
    pub fill_rule: FillRule,
    pub analytic_bounds: [f64; 4],
    pub origin: (f64, f64),
    pub size: (u32, u32),
    pub density: f64,
    pub segments: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMask {
    pub id: String,
    pub grid: Option<MaskGrid>,
    pub paint: Paint,
    pub channel: MaskChannel,
    pub operation: MaskOperation,
    pub inverted: bool,
    /// Owner-local project coordinates to original path coordinates.
    pub inverse: [f64; 6],
    pub opacity: f64,
    pub expansion_grid: f64,
    pub feather_grid: f64,
    pub certified_scratch_bytes: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMaskStack {
    pub owner: MaskOwnerBasis,
    pub masks: Vec<EvaluatedMask>,
    pub work_units: u64,
    pub accumulated_bytes: u64,
    pub retained_fact_bytes: u64,
    pub peak_scratch_bytes: u64,
}

pub(super) fn sample_masks(
    authored: &[crate::Mask],
    channels: &[crate::AnimationChannel],
    time: crate::animation::SampleTime,
) -> Result<Vec<crate::Mask>, crate::CoreError> {
    use crate::{
        AnimationChannelProperty as P, AnimationChannelValue as V, GradientStop, MaskSource,
        PathCommand, VectorColor, VectorPoint,
    };
    let mut masks = authored.to_vec();
    for channel in channels
        .iter()
        .filter(|c| c.property.mask() && !c.keyframes.is_empty())
    {
        let target = channel
            .target
            .as_ref()
            .ok_or_else(|| super::invalid("sampled mask target missing"))?;
        let mask = masks
            .iter_mut()
            .find(|m| m.id == target.id)
            .ok_or_else(|| {
                crate::CoreError::new(
                    crate::ErrorCode::ItemNotFound,
                    "sampled mask target missing",
                )
            })?;
        let value = time
            .sample(channel)
            .ok_or_else(|| super::invalid("non-finite sampled mask value"))?;
        match (channel.property, value) {
            (P::MaskPathPoints, V::PathPoints { points }) => {
                let MaskSource::Path { path, .. } = &mut mask.source;
                let mut values = points.into_iter();
                let mut next = || {
                    values
                        .next()
                        .map(|p| VectorPoint { x: p.x, y: p.y })
                        .ok_or_else(|| super::invalid("mask path sample topology mismatch"))
                };
                for c in &mut path.commands {
                    match c {
                        PathCommand::MoveTo { to } | PathCommand::LineTo { to } => *to = next()?,
                        PathCommand::QuadraticTo { control, to } => {
                            *control = next()?;
                            *to = next()?;
                        }
                        PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => {
                            *control1 = next()?;
                            *control2 = next()?;
                            *to = next()?;
                        }
                        PathCommand::Close {} => {}
                    }
                }
                if values.next().is_some() {
                    return Err(super::invalid("mask path sample topology mismatch"));
                }
            }
            (P::MaskPaintColor, V::Rgba { r, g, b, a }) => {
                let MaskSource::Path { paint, .. } = &mut mask.source;
                let Paint::Solid { color } = paint else {
                    return Err(super::invalid("sampled mask solid paint missing"));
                };
                *color = VectorColor { r, g, b, a };
            }
            (P::MaskGradientStops, V::GradientStops { stops }) => {
                let MaskSource::Path { paint, .. } = &mut mask.source;
                let (Paint::LinearGradient { stops: target, .. }
                | Paint::RadialGradient { stops: target, .. }) = paint
                else {
                    return Err(super::invalid("sampled mask gradient missing"));
                };
                if stops.len() != target.len() {
                    return Err(super::invalid("mask gradient sample topology mismatch"));
                }
                *target = stops
                    .into_iter()
                    .map(|s| GradientStop {
                        offset: s.offset,
                        color: VectorColor {
                            r: s.color[0],
                            g: s.color[1],
                            b: s.color[2],
                            a: s.color[3],
                        },
                    })
                    .collect();
            }
            (p, V::Scalar { value }) => {
                // Exact endpoints/holds keep their stored positive scales. Interpolation
                // applies the documented lower scale clamp in the canonical sampler.
                let (low, high) = crate::validation::extended_visual::mask_scalar_bounds(
                    p,
                    mask.transform.position.unit,
                )
                .ok_or_else(|| super::invalid("sampled mask scalar property missing"))?;
                let value = if matches!(p, P::MaskTransformScaleX | P::MaskTransformScaleY) {
                    value.min(high)
                } else {
                    value.clamp(low, high)
                };
                match p {
                    P::MaskFeatherPx => mask.feather_px = value,
                    P::MaskExpansionPx => mask.expansion_px = value,
                    P::MaskTransformPositionX => mask.transform.position.x = value,
                    P::MaskTransformPositionY => mask.transform.position.y = value,
                    P::MaskTransformScaleX => mask.transform.scale_x = value,
                    P::MaskTransformScaleY => mask.transform.scale_y = value,
                    P::MaskTransformAnchorX => mask.transform.anchor.x = value,
                    P::MaskTransformAnchorY => mask.transform.anchor.y = value,
                    P::MaskTransformRotationDeg => mask.transform.rotation_deg = value,
                    P::MaskTransformSkewXDeg => mask.transform.skew_x_deg = value,
                    P::MaskTransformSkewYDeg => mask.transform.skew_y_deg = value,
                    P::MaskTransformOpacity => mask.transform.opacity = value,
                    _ => return Err(super::invalid("unsupported sampled mask scalar")),
                }
            }
            _ => return Err(super::invalid("sampled mask value kind mismatch")),
        }
        mask.validate()?;
    }
    Ok(masks)
}

pub(crate) const MAX_MASK_WORK: u64 = 268_435_456;
#[derive(Default, Debug)]
pub(crate) struct MaskFrameBudget {
    pub work: u64,
}
impl MaskFrameBudget {
    pub(crate) fn charge(&mut self, units: u64) -> Result<(), crate::CoreError> {
        self.work = self
            .work
            .checked_add(units)
            .filter(|w| *w <= MAX_MASK_WORK)
            .ok_or_else(|| super::invalid("output-frame mask work exceeds limits"))?;
        Ok(())
    }
}
fn add(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_add(b)
        .ok_or_else(|| super::invalid("mask accounting overflow"))
}
fn mul(a: u64, b: u64) -> Result<u64, crate::CoreError> {
    a.checked_mul(b)
        .ok_or_else(|| super::invalid("mask accounting overflow"))
}
fn pixels(size: (u32, u32)) -> Result<u64, crate::CoreError> {
    if size.0 == 0 || size.1 == 0 || size.0 > 16384 || size.1 > 16384 {
        return Err(super::invalid("mask surface dimensions exceed limits"));
    }
    mul(u64::from(size.0), u64::from(size.1)).and_then(|p| {
        if p <= 16777216 {
            Ok(p)
        } else {
            Err(super::invalid("mask surface pixels exceed limits"))
        }
    })
}
fn log2ceil(n: u64) -> u64 {
    if n <= 1 {
        0
    } else {
        u64::from(64 - (n - 1).leading_zeros())
    }
}
fn paint_heap(paint: &Paint) -> Result<u64, crate::CoreError> {
    match paint {
        Paint::Solid { .. } => Ok(0),
        Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. } => mul(
            stops.capacity() as u64,
            std::mem::size_of::<crate::GradientStop>() as u64,
        ),
    }
}
pub(crate) fn authored_mask_bytes(masks: &[crate::Mask]) -> Result<u64, crate::CoreError> {
    let mut total = mul(
        masks.len() as u64,
        std::mem::size_of::<crate::Mask>() as u64,
    )?;
    for mask in masks {
        let crate::MaskSource::Path { path, paint } = &mask.source;
        total = add(total, mask.id.capacity() as u64)?;
        total = add(
            total,
            mul(
                path.commands.capacity() as u64,
                std::mem::size_of::<crate::PathCommand>() as u64,
            )?,
        )?;
        total = add(total, paint_heap(paint)?)?;
    }
    Ok(total)
}
/// Pinned tiny-skia 0.11.4 line-only fill transient storage. Its Edge enum
/// reserves the cubic variant (bounded by 128 bytes), even for lines. Clipping
/// produces at most three edges per line. Vec growth/reallocation, two sentinels
/// and stable-sort auxiliary storage fit 1152 bytes per (segment+contour+64).
/// The additive 2048-byte reservation also covers simultaneous Path verbs/points,
/// clipped descriptors, headers and allocator growth. AA rows are separate.
/// This is ADDITIVE to retained contours/facts and the fixed pixel scratch.
pub(crate) fn fill_transient_bytes(
    segments: u64,
    contours: u64,
    size: (u32, u32),
) -> Result<u64, crate::CoreError> {
    add(
        mul(2048, add(add(segments, contours)?, 64)?)?,
        mul(32, add(u64::from(size.0), u64::from(size.1))?)?,
    )
}

pub(crate) fn owning_mask_bytes(masks: &Vec<crate::Mask>) -> Result<u64, crate::CoreError> {
    add(
        authored_mask_bytes(masks)?,
        mul(
            (masks.capacity() - masks.len()) as u64,
            std::mem::size_of::<crate::Mask>() as u64,
        )?,
    )
}
fn magnification(matrix: [f64; 6]) -> f64 {
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
fn affine(
    mask: &crate::Mask,
    bounds: [f64; 4],
    owner: MaskOwnerBasis,
) -> Result<([f64; 6], [f64; 6]), crate::CoreError> {
    let mut transform = mask.transform;
    if transform.position.unit == crate::PositionUnit::Normalized {
        transform.position.x *= f64::from(owner.size.0) / owner.density;
        transform.position.y *= f64::from(owner.size.1) / owner.density;
        transform.position.unit = crate::PositionUnit::Pixels;
    }
    // Normalize after resolving units; the canonical authored validator already
    // checked normalized coordinates, so derived pixels may exceed authored limits.
    let authored = transform.position;
    transform.position.x = 0.0;
    transform.position.y = 0.0;
    let (mut matrix, mut inverse) = super::transform_matrices_logical(
        transform,
        (bounds[2] - bounds[0], bounds[3] - bounds[1]),
        (1, 1),
    )?;
    matrix[4] += authored.x - matrix[0] * bounds[0] - matrix[2] * bounds[1];
    matrix[5] += authored.y - matrix[1] * bounds[0] - matrix[3] * bounds[1];
    let det = transform.scale_x * transform.scale_y;
    inverse[4] = (matrix[2] * matrix[5] - matrix[3] * matrix[4]) / det;
    inverse[5] = (matrix[1] * matrix[4] - matrix[0] * matrix[5]) / det;
    if matrix.iter().chain(&inverse).any(|v| !v.is_finite()) {
        return Err(super::invalid("mask inverse must be finite"));
    }
    Ok((matrix, inverse))
}
fn certify_precision(
    matrix: [f64; 6],
    inverse: [f64; 6],
    bounds: [f64; 4],
    owner: MaskOwnerBasis,
    rho: f64,
) -> Result<(), crate::CoreError> {
    for (x, y) in [
        (0.0, 0.0),
        (f64::from(owner.size.0) / owner.density, 0.0),
        (0.0, f64::from(owner.size.1) / owner.density),
        (
            f64::from(owner.size.0) / owner.density,
            f64::from(owner.size.1) / owner.density,
        ),
    ] {
        let error = (inverse[0] * x).abs()
            + (inverse[2] * y).abs()
            + inverse[4].abs()
            + (inverse[1] * x).abs()
            + (inverse[3] * y).abs()
            + inverse[5].abs();
        if !error.is_finite() || error * f64::EPSILON * 16.0 * rho > 0.25 {
            return Err(super::invalid(
                "mask inverse precision exceeds grid tolerance",
            ));
        }
    }
    for (x, y) in [
        (bounds[0], bounds[1]),
        (bounds[2], bounds[1]),
        (bounds[0], bounds[3]),
        (bounds[2], bounds[3]),
    ] {
        let px = matrix[0] * x + matrix[2] * y + matrix[4];
        let py = matrix[1] * x + matrix[3] * y + matrix[5];
        let qx = inverse[0] * px + inverse[2] * py + inverse[4];
        let qy = inverse[1] * px + inverse[3] * py + inverse[5];
        if !qx.is_finite() || !qy.is_finite() || (qx - x).hypot(qy - y) * rho > 0.25 {
            return Err(super::invalid(
                "mask affine roundtrip precision exceeds grid tolerance",
            ));
        }
    }
    Ok(())
}

pub(crate) fn certify_sampled_masks(
    sampled: &[crate::Mask],
    owner: MaskOwnerBasis,
    budget: &mut MaskFrameBudget,
    scene_segments: &mut usize,
) -> Result<EvaluatedMaskStack, crate::CoreError> {
    if !owner.density.is_finite() || owner.density <= 0.0 {
        return Err(super::invalid("mask owner density must be positive finite"));
    }
    let owner_pixels = pixels(owner.size)?;
    let before = budget.work;
    let mut result = EvaluatedMaskStack {
        owner,
        masks: Vec::new(),
        work_units: 0,
        accumulated_bytes: 0,
        retained_fact_bytes: 0,
        peak_scratch_bytes: 0,
    };
    if sampled.is_empty() {
        return Ok(result);
    }
    result
        .masks
        .try_reserve_exact(sampled.len())
        .map_err(|_| super::invalid("mask facts allocation failed"))?;
    result.accumulated_bytes = mul(owner_pixels, 4)?;
    result.retained_fact_bytes = add(
        std::mem::size_of::<EvaluatedMaskStack>() as u64,
        mul(
            result.masks.capacity() as u64,
            std::mem::size_of::<EvaluatedMask>() as u64,
        )?,
    )?;
    budget.charge(mul(owner_pixels, 4)?)?;
    for mask in sampled {
        mask.validate()?;
        let crate::MaskSource::Path { path, paint } = &mask.source;
        // Affine linear part does not depend on the analytic anchor. Compile at
        // final rho, then rebuild translation from sampled unexpanded bounds.
        let (linear, _) = affine(mask, [0.0; 4], owner)?;
        let rho = (owner.density * magnification(linear)).max(1.0);
        if !rho.is_finite() {
            return Err(super::invalid("mask density must be finite"));
        }
        let compiled = super::shapes::compile_mask_path(
            path,
            rho,
            super::shapes::MAX_SCENE_SEGMENTS.saturating_sub(*scene_segments),
        )?;
        *scene_segments = scene_segments
            .checked_add(compiled.segments)
            .filter(|s| *s <= super::shapes::MAX_SCENE_SEGMENTS)
            .ok_or_else(|| super::invalid("shared mask scene segment limit exceeded"))?;
        let (matrix, inverse) = affine(mask, compiled.analytic_bounds, owner)?;
        certify_precision(matrix, inverse, compiled.analytic_bounds, owner, rho)?;
        let expansion_grid = mask.expansion_px * rho;
        let feather_grid = mask.feather_px * rho;
        budget.charge(mul(owner_pixels, 5)?)?;
        let mut scratch = 0;
        let grid = if compiled.drawable_fill {
            let padding = expansion_grid.abs().ceil() + (3.0 * feather_grid).ceil() + 2.0;
            let [l, t, r, b] = compiled.analytic_bounds;
            let gx = (l * rho).floor() - padding;
            let gy = (t * rho).floor() - padding;
            let width = (r * rho).ceil() + padding - gx;
            let height = (b * rho).ceil() + padding - gy;
            if !width.is_finite()
                || !height.is_finite()
                || width < 1.0
                || height < 1.0
                || width > 16384.0
                || height > 16384.0
            {
                return Err(super::invalid("mask support exceeds surface dimensions"));
            }
            let size = (width as u32, height as u32);
            let p = pixels(size)?;
            let origin = (gx / rho, gy / rho);
            for point in compiled.contours.iter().flat_map(|c| &c.points) {
                let x = (point.x - origin.0) * rho;
                let y = (point.y - origin.1) * rho;
                if !x.is_finite()
                    || !y.is_finite()
                    || (x - f64::from(x as f32)).hypot(y - f64::from(y as f32)) > 0.25
                {
                    return Err(super::invalid(
                        "mask raster coordinate conversion exceeds grid tolerance",
                    ));
                }
            }
            let s = compiled.segments as u64;
            budget.charge(add(
                mul(p, 4)?,
                mul(
                    mul(mul(s, 4)?, u64::from(size.1))?,
                    add(1, log2ceil(s.max(1)))?,
                )?,
            )?)?;
            let stops = match paint {
                Paint::Solid { .. } => 1,
                Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. } => {
                    stops.len() as u64
                }
            };
            budget.charge(mul(p, add(1, log2ceil(stops))?)?)?;
            if expansion_grid != 0.0 {
                budget.charge(mul(p, 24)?)?;
            }
            scratch = add(
                mul(p, 64)?,
                mul(add(u64::from(size.0), u64::from(size.1))?, 16)?,
            )?;
            scratch = add(
                scratch,
                fill_transient_bytes(s, compiled.contours.len() as u64, size)?,
            )?;
            if feather_grid != 0.0 {
                let k = (3.0 * feather_grid).ceil();
                if !k.is_finite() || k > u64::MAX as f64 {
                    return Err(super::invalid("mask kernel radius overflow"));
                }
                let taps = add(mul(k as u64, 2)?, 1)?;
                budget.charge(mul(mul(p, 2)?, taps)?)?;
                scratch = add(scratch, mul(taps, 4)?)?;
            }
            result.retained_fact_bytes = add(result.retained_fact_bytes, mul(s, 64)?)?;
            result.retained_fact_bytes = add(
                result.retained_fact_bytes,
                mul(
                    compiled.contours.capacity() as u64,
                    std::mem::size_of::<Contour>() as u64,
                )?,
            )?;
            for contour in &compiled.contours {
                result.retained_fact_bytes = add(
                    result.retained_fact_bytes,
                    mul(
                        contour.points.capacity() as u64,
                        std::mem::size_of::<crate::VectorPoint>() as u64,
                    )?,
                )?;
            }
            Some(MaskGrid {
                contours: compiled.contours,
                fill_rule: compiled.fill_rule,
                analytic_bounds: compiled.analytic_bounds,
                origin,
                size,
                density: rho,
                segments: s,
            })
        } else {
            None
        };
        let id = mask.id.clone();
        let paint = paint.clone();
        result.retained_fact_bytes = add(
            result.retained_fact_bytes,
            add(id.capacity() as u64, paint_heap(&paint)?)?,
        )?;
        result.peak_scratch_bytes = result.peak_scratch_bytes.max(scratch);
        result.masks.push(EvaluatedMask {
            id,
            grid,
            paint,
            channel: mask.channel,
            operation: mask.operation,
            inverted: mask.inverted,
            inverse,
            opacity: mask.transform.opacity,
            expansion_grid,
            feather_grid,
            certified_scratch_bytes: scratch,
        });
    }
    result.work_units = budget.work - before;
    Ok(result)
}
impl EvaluatedMaskStack {
    pub(crate) fn additional_live_bytes(&self) -> Result<u64, crate::CoreError> {
        add(
            add(self.accumulated_bytes, self.retained_fact_bytes)?,
            self.peak_scratch_bytes,
        )
    }
}

/// Heap storage retained by authored channels, including compound keyframes.
fn channel_bytes(channels: &Vec<crate::AnimationChannel>) -> Result<u64, crate::CoreError> {
    use crate::AnimationChannelValue as V;
    let mut bytes = mul(
        channels.capacity() as u64,
        std::mem::size_of::<crate::AnimationChannel>() as u64,
    )?;
    for channel in channels {
        bytes = add(
            bytes,
            mul(
                channel.keyframes.capacity() as u64,
                std::mem::size_of::<crate::AnimationChannelKeyframe>() as u64,
            )?,
        )?;
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
                    V::PathPoints { points } => mul(
                        points.capacity() as u64,
                        std::mem::size_of::<crate::AnimationPoint>() as u64,
                    )?,
                    V::GradientStops { stops } => mul(
                        stops.capacity() as u64,
                        std::mem::size_of::<crate::AnimationGradientStop>() as u64,
                    )?,
                    _ => 0,
                },
            )?;
        }
    }
    Ok(bytes)
}

/// Run before evaluated programs are copied. Scene occurrences subsequently share
/// immutable programs; sampling owns at most one mask copy at a time.
pub(crate) fn certify_authored_program_memory(
    project: &crate::Project,
) -> Result<(), crate::CoreError> {
    let mut bytes = 0;
    for item in project
        .tracks
        .iter()
        .chain(project.components.iter().flat_map(|c| &c.tracks))
        .flat_map(|t| &t.items)
    {
        let visual = item.visual_properties();
        if visual.masks.is_empty() {
            continue;
        }
        bytes = add(bytes, mul(owning_mask_bytes(&visual.masks)?, 4)?)?;
        bytes = add(bytes, mul(channel_bytes(&visual.animation_channels)?, 3)?)?;
    }
    if bytes > super::extended_visual::MAX_COMPOSITION_BYTES {
        return Err(super::invalid(
            "authored mask programs exceed live memory limits",
        ));
    }
    Ok(())
}

pub(crate) fn scene_program_bytes(scene: &super::EvaluatedScene) -> Result<u64, crate::CoreError> {
    let mut seen_masks = std::collections::HashSet::new();
    let mut seen_channels = std::collections::HashSet::new();
    let mut bytes = 0;
    for layer in &scene.visual_layers {
        let Some(extended) = &layer.extended else {
            continue;
        };
        if extended.masks.is_empty() {
            continue;
        }
        bytes = add(
            bytes,
            std::mem::size_of::<super::extended_visual::ExtendedVisual>() as u64,
        )?;
        if seen_masks.insert(extended.masks.identity()) {
            bytes = add(bytes, owning_mask_bytes(&extended.masks)?)?;
        }
        if seen_channels.insert(extended.channels.identity()) {
            bytes = add(bytes, channel_bytes(&extended.channels)?)?;
        }
    }
    Ok(bytes)
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MaskEnvelope {
    pub work: u64,
    pub retained: u64,
    pub scratch: u64,
    pub segments: usize,
}

/// A continuous source-curve interval, with integer whole clocks kept separate
/// from fractions. Left and right children share the midpoint: no time gap.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Boundary {
    whole: u64,
    fraction: f64,
}
impl Boundary {
    fn time(self) -> crate::animation::SampleTime {
        crate::animation::SampleTime::Split {
            whole: self.whole,
            fraction: self.fraction,
        }
    }
    fn midpoint(self, high: Self) -> Self {
        let span = high.whole - self.whole;
        let fraction = (span % 2) as f64 * 0.5 + (self.fraction + high.fraction) * 0.5;
        Self {
            whole: self.whole + span / 2 + fraction.floor() as u64,
            fraction: fraction.fract(),
        }
    }
}

pub(crate) fn continuous_mask_envelope(
    mask: &crate::Mask,
    channels: &[crate::AnimationChannel],
    owner: MaskOwnerBasis,
    nodes: &mut usize,
) -> Result<MaskEnvelope, crate::CoreError> {
    use crate::{
        AnimationChannelProperty as P, AnimationChannelValue as V, MaskSource, PathCommand,
    };
    let channels: Vec<_> = channels
        .iter()
        .filter(|c| {
            c.property.mask()
                && c.target.as_ref().is_some_and(|t| t.id == mask.id)
                && !c.keyframes.is_empty()
        })
        .collect();
    let MaskSource::Path { path, .. } = &mask.source;
    let no_drawable_topology = !path.commands.iter().any(|c| {
        matches!(
            c,
            PathCommand::LineTo { .. }
                | PathCommand::QuadraticTo { .. }
                | PathCommand::CubicTo { .. }
        )
    });
    let end = channels
        .iter()
        .filter_map(|c| c.keyframes.last().map(|k| k.time_ms))
        .max()
        .unwrap_or(0);
    // Equal source clocks share the same source axis, even with loops/retained
    // windows. Different clocks use independent full source envelopes.
    let correlated = channels
        .windows(2)
        .all(|c| c[0].clock == c[1].clock && c[0].r#loop == c[1].r#loop);
    let mut pending = vec![(
        Boundary {
            whole: 0,
            fraction: 0.0,
        },
        Boundary {
            whole: end,
            fraction: 0.0,
        },
    )];
    let mut result = MaskEnvelope::default();
    while let Some((low, high)) = pending.pop() {
        if *nodes == 65_536 {
            return Err(super::invalid(
                "maxCandidateAnalysisNodes exhausted with unresolved mask safety",
            ));
        }
        *nodes += 1;
        #[cfg(test)]
        tests::record_interval(low, high);
        let scalar = |p, fallback| -> Result<(f64, f64), crate::CoreError> {
            let Some(channel) = channels.iter().find(|c| c.property == p) else {
                return Ok((fallback, fallback));
            };
            let mut source = (*channel).clone();
            source.clock = None;
            source.r#loop = None;
            let (a, b) = if correlated {
                (low.time(), high.time())
            } else {
                (
                    crate::animation::SampleTime::Integer(0),
                    crate::animation::SampleTime::Integer(source.keyframes.last().unwrap().time_ms),
                )
            };
            crate::animation::scalar_source_bounds(&source, a, b)
                .ok_or_else(|| super::invalid("mask scalar envelope missing"))
        };
        let sx = scalar(P::MaskTransformScaleX, mask.transform.scale_x)?;
        let sy = scalar(P::MaskTransformScaleY, mask.transform.scale_y)?;
        let kx = scalar(P::MaskTransformSkewXDeg, mask.transform.skew_x_deg)?;
        let ky = scalar(P::MaskTransformSkewYDeg, mask.transform.skew_y_deg)?;
        let mut rho = 1.0_f64;
        let mut inverse_norm = 0.0_f64;
        for x in [sx.0, sx.1] {
            for y in [sy.0, sy.1] {
                for a in [kx.0, kx.1] {
                    for b in [ky.0, ky.1] {
                        let mut transform = mask.transform;
                        transform.position.x = 0.0;
                        transform.position.y = 0.0;
                        transform.scale_x = x;
                        transform.scale_y = y;
                        transform.skew_x_deg = a;
                        transform.skew_y_deg = b;
                        transform.rotation_deg = 0.0;
                        let (matrix, inverse) =
                            super::transform_matrices_logical(transform, (1.0, 1.0), (1, 1))?;
                        rho = rho.max(owner.density * magnification(matrix));
                        inverse_norm = inverse_norm.max(magnification(inverse));
                    }
                }
            }
        }
        // Sequential shears/scales are multiaffine; operator norm is convex in
        // each independent coefficient, hence the endpoint-box corner maximum.
        let MaskSource::Path { path, paint } = &mask.source;
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
        for command in &path.commands {
            match command {
                PathCommand::MoveTo { to } | PathCommand::LineTo { to } => include(to.x, to.y),
                PathCommand::QuadraticTo { control, to } => {
                    include(control.x, control.y);
                    include(to.x, to.y);
                }
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => {
                    include(control1.x, control1.y);
                    include(control2.x, control2.y);
                    include(to.x, to.y);
                }
                PathCommand::Close {} => {}
            }
        }
        if let Some(channel) = channels.iter().find(|c| c.property == P::MaskPathPoints) {
            for key in &channel.keyframes {
                let V::PathPoints { points } = &key.value else {
                    return Err(super::invalid("mask path envelope missing"));
                };
                for p in points {
                    include(p.x, p.y);
                }
            }
            for pair in channel.keyframes.windows(2) {
                let (V::PathPoints { points: a }, V::PathPoints { points: b }) =
                    (&pair[0].value, &pair[1].value)
                else {
                    return Err(super::invalid("mask path envelope topology"));
                };
                let (p, q) = crate::animation::curve_bounds(pair[0].curve, 0.0, 1.0);
                for (a, b) in a.iter().zip(b) {
                    for t in [p, q] {
                        include(
                            (a.x + (b.x - a.x) * t).clamp(-1_000_000.0, 1_000_000.0),
                            (a.y + (b.y - a.y) * t).clamp(-1_000_000.0, 1_000_000.0),
                        );
                    }
                }
            }
        }
        if !bounds[0].is_finite() {
            bounds = [0.0; 4];
        }
        let feather = scalar(P::MaskFeatherPx, mask.feather_px)?.1;
        let expansion = scalar(P::MaskExpansionPx, mask.expansion_px)?;
        let expansion = expansion.0.abs().max(expansion.1.abs());
        let pad = (expansion * rho).ceil() + (3.0 * feather * rho).ceil() + 2.0;
        let w = (bounds[2] * rho).ceil() - (bounds[0] * rho).floor() + 2.0 * pad;
        let h = (bounds[3] * rho).ceil() - (bounds[1] * rho).floor() + 2.0 * pad;
        let px = scalar(P::MaskTransformPositionX, mask.transform.position.x)?;
        let py = scalar(P::MaskTransformPositionY, mask.transform.position.y)?;
        let position_factor = if mask.transform.position.unit == crate::PositionUnit::Normalized {
            (
                f64::from(owner.size.0) / owner.density,
                f64::from(owner.size.1) / owner.density,
            )
        } else {
            (1.0, 1.0)
        };
        let extent = bounds.into_iter().map(f64::abs).fold(0.0, f64::max)
            + px.0.abs().max(px.1.abs()) * position_factor.0
            + py.0.abs().max(py.1.abs()) * position_factor.1
            + f64::from(owner.size.0.max(owner.size.1)) / owner.density
            + 1.0;
        let precise = rho.is_finite()
            && inverse_norm.is_finite()
            && 64.0 * f64::EPSILON * extent * (1.0 + rho) * (1.0 + inverse_norm) * rho <= 0.25;
        let safe = precise
            && w.is_finite()
            && h.is_finite()
            && (no_drawable_topology || (w <= 16384.0 && h <= 16384.0 && w * h <= 16_777_216.0));
        if !safe {
            let middle = low.midpoint(high);
            if low == high || middle == low || middle == high {
                return Err(super::invalid(
                    "unsafe continuous mask support or precision",
                ));
            }
            pending.push((middle, high));
            pending.push((low, middle));
            continue;
        }
        if no_drawable_topology {
            result.work = result.work.max(mul(5, pixels(owner.size)?)?);
            result.retained = result
                .retained
                .max(authored_mask_bytes(std::slice::from_ref(mask))?);
            continue;
        }
        let diameter = (bounds[2] - bounds[0]).hypot(bounds[3] - bounds[1]);
        let depth = ((diameter * rho / 0.25).max(1.0).log(4.0).ceil() as u32).saturating_add(1);
        let mut segments = 0_usize;
        for command in &path.commands {
            let n = match command {
                PathCommand::CubicTo { .. } | PathCommand::QuadraticTo { .. } => {
                    1_usize.checked_shl(depth).unwrap_or(usize::MAX)
                }
                PathCommand::MoveTo { .. } => 0,
                _ => 1,
            };
            segments = segments
                .checked_add(n)
                .filter(|v| *v <= super::shapes::MAX_SEGMENTS)
                .ok_or_else(|| super::invalid("continuous mask subdivision limit exceeded"))?;
        }
        let closures = path
            .commands
            .iter()
            .filter(|c| matches!(c, PathCommand::MoveTo { .. }))
            .count();
        segments = segments
            .checked_add(closures)
            .filter(|v| *v <= super::shapes::MAX_SEGMENTS)
            .ok_or_else(|| super::invalid("continuous mask closure limit exceeded"))?;
        let p = (w * h) as u64;
        let s = segments as u64;
        let owner_pixels = pixels(owner.size)?;
        let radius = (3.0 * feather * rho).ceil();
        if !radius.is_finite() || radius > u64::MAX as f64 {
            return Err(super::invalid("continuous mask kernel overflow"));
        }
        let taps = add(mul(2, radius as u64)?, 1)?;
        let stops = match paint {
            crate::Paint::Solid { .. } => 1,
            crate::Paint::LinearGradient { stops, .. }
            | crate::Paint::RadialGradient { stops, .. } => stops.len() as u64,
        };
        let mut work = add(
            mul(4, p)?,
            mul(mul(mul(4, s)?, h as u64)?, 1 + log2ceil(s.max(1)))?,
        )?;
        work = add(work, mul(p, 1 + log2ceil(stops))?)?;
        if expansion != 0.0 {
            work = add(work, mul(24, p)?)?;
        }
        if feather != 0.0 {
            work = add(work, mul(mul(2, p)?, taps)?)?;
        }
        work = add(work, mul(5, owner_pixels)?)?;
        result.work = result.work.max(work);
        result.retained = result.retained.max(add(
            mul(64, s)?,
            authored_mask_bytes(std::slice::from_ref(mask))?,
        )?);
        result.scratch = result.scratch.max(add(
            add(
                add(mul(64, p)?, mul(16, w as u64 + h as u64)?)?,
                mul(4, taps)?,
            )?,
            fill_transient_bytes(s, path.commands.len() as u64, (w as u32, h as u32))?,
        )?);
        result.segments = result.segments.max(segments);
    }
    Ok(result)
}
