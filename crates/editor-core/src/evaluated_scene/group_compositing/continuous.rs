//! Simultaneous signed-domain proof over the immutable occurrence graph.
//! Shared translations cancel before bounding separation; unresolved intervals
//! refine left first under the canonical candidate-analysis quota.
use super::*;
use crate::animation::SampleTime;
use crate::{AnimationChannelProperty as P, AnimationChannelValue as V};

#[derive(Clone, Copy, Debug)]
struct Range(f64, f64);
impl Range {
    fn checked(low: f64, high: f64) -> Self {
        if low.is_finite() && high.is_finite() && low <= high {
            Self(low, high)
        } else {
            Self(f64::NEG_INFINITY, f64::INFINITY)
        }
    }
    fn finite(self) -> bool {
        self.0.is_finite() && self.1.is_finite() && self.0 <= self.1
    }
    fn exact(v: f64) -> Self {
        Self::checked(v, v)
    }
    fn plus(self, b: Self) -> Self {
        Self::checked((self.0 + b.0).next_down(), (self.1 + b.1).next_up())
    }
    fn minus(self, b: Self) -> Self {
        self.plus(Self(-b.1, -b.0))
    }
    fn times(self, b: Self) -> Self {
        if !self.finite() || !b.finite() {
            return Self::checked(f64::NEG_INFINITY, f64::INFINITY);
        }
        let products = [self.0 * b.0, self.0 * b.1, self.1 * b.0, self.1 * b.1];
        if products.iter().any(|value| !value.is_finite()) {
            return Self::checked(f64::NEG_INFINITY, f64::INFINITY);
        }
        Self::checked(
            products
                .into_iter()
                .fold(f64::INFINITY, f64::min)
                .next_down(),
            products
                .into_iter()
                .fold(f64::NEG_INFINITY, f64::max)
                .next_up(),
        )
    }
    fn magnitude(self) -> f64 {
        self.0.abs().max(self.1.abs())
    }
    fn reciprocal(self) -> Self {
        if !self.finite() || (self.0 <= 0. && self.1 >= 0.) {
            return Self::checked(f64::NEG_INFINITY, f64::INFINITY);
        }
        Self::checked((1. / self.1).next_down(), (1. / self.0).next_up())
    }
}
type Matrix = [Range; 6];
fn exact(matrix: [f64; 6]) -> Matrix {
    matrix.map(Range::exact)
}
fn product(l: Matrix, r: Matrix) -> Matrix {
    let [a, b, c, d, x, y] = l;
    let [e, f, g, h, u, v] = r;
    [
        a.times(e).plus(c.times(f)),
        b.times(e).plus(d.times(f)),
        a.times(g).plus(c.times(h)),
        b.times(g).plus(d.times(h)),
        a.times(u).plus(c.times(v)).plus(x),
        b.times(u).plus(d.times(v)).plus(y),
    ]
}
fn point(m: Matrix, p: [Range; 2]) -> [Range; 2] {
    [
        m[0].times(p[0]).plus(m[2].times(p[1])).plus(m[4]),
        m[1].times(p[0]).plus(m[3].times(p[1])).plus(m[5]),
    ]
}
fn trigonometric(range: Range, cosine: bool) -> Range {
    use std::f64::consts::{PI, TAU};
    if !range.finite() {
        return Range::checked(f64::NEG_INFINITY, f64::INFINITY);
    }
    let low = range.0.to_radians();
    let high = range.1.to_radians();
    if high - low >= TAU {
        return Range(-1., 1.);
    }
    let f = |v: f64| if cosine { v.cos() } else { v.sin() };
    let mut minimum = f(low).min(f(high));
    let mut maximum = f(low).max(f(high));
    let offset = if cosine { 0. } else { PI / 2. };
    let first = ((low - offset) / PI).ceil() as i64;
    let last = ((high - offset) / PI).floor() as i64;
    for index in first..=last {
        let value = if index % 2 == 0 { 1. } else { -1. };
        minimum = minimum.min(value);
        maximum = maximum.max(value);
    }
    Range(minimum.next_down(), maximum.next_up())
}
fn channel_range(
    channel: &crate::AnimationChannel,
    low: SampleTime,
    high: SampleTime,
) -> Result<Range, CoreError> {
    let same_sample = match (low, high) {
        (SampleTime::Integer(a), SampleTime::Integer(b)) => a == b,
        (SampleTime::Fractional(a), SampleTime::Fractional(b)) => a.to_bits() == b.to_bits(),
        (
            SampleTime::Split {
                whole: a,
                fraction: x,
            },
            SampleTime::Split {
                whole: b,
                fraction: y,
            },
        ) => a == b && x.to_bits() == y.to_bits(),
        _ => false,
    };
    if same_sample {
        return low
            .scalar(channel)
            .map(Range::exact)
            .ok_or_else(|| invalid("aggregate exact channel sample unavailable"));
    }
    // Loop bounds are supplied by their canonical owner, including clocks and
    // fractional derived instances. No sampled endpoint-only safety proof.
    let bounds = if channel.r#loop.is_some() {
        crate::animation::scalar_source_bounds(
            channel,
            SampleTime::Integer(0),
            SampleTime::Integer(
                channel
                    .keyframes
                    .last()
                    .ok_or_else(|| invalid("aggregate channel missing"))?
                    .time_ms,
            ),
        )
    } else {
        low.looped(channel)
            .zip(high.looped(channel))
            .and_then(|(low, high)| crate::animation::scalar_source_bounds(channel, low, high))
    };
    bounds
        .map(|(a, b)| Range(a, b))
        .ok_or_else(|| invalid("aggregate channel envelope unavailable"))
}
fn transform(
    base: crate::Transform2D,
    source: (f64, f64),
    canvas: (u32, u32),
    channels: &[crate::AnimationChannel],
    low: SampleTime,
    high: SampleTime,
) -> Result<Matrix, CoreError> {
    let bound = |property, fallback| {
        channels
            .iter()
            .find(|c| c.property == property)
            .map_or(Ok(Range::exact(fallback)), |c| channel_range(c, low, high))
    };
    let x = bound(P::PositionX, base.position.x)?;
    let y = bound(P::PositionY, base.position.y)?;
    let sx = bound(P::ScaleX, base.scale_x)?;
    let sy = bound(P::ScaleY, base.scale_y)?;
    let angle = bound(P::RotationDeg, base.rotation_deg)?;
    let sin = trigonometric(angle, false);
    let cos = trigonometric(angle, true);
    let kx = Range::exact(base.skew_x_deg.to_radians().tan());
    let ky = Range::exact(base.skew_y_deg.to_radians().tan());
    let a = cos.minus(sin.times(ky)).times(sx);
    let b = sin.plus(cos.times(ky)).times(sx);
    let c = cos
        .times(kx)
        .minus(sin.times(Range::exact(1.).plus(ky.times(kx))))
        .times(sy);
    let d = sin
        .times(kx)
        .plus(cos.times(Range::exact(1.).plus(ky.times(kx))))
        .times(sy);
    let factor = match base.position.unit {
        crate::PositionUnit::Pixels => (1., 1.),
        crate::PositionUnit::Normalized => (f64::from(canvas.0), f64::from(canvas.1)),
    };
    let ax = Range::exact(base.anchor.x * source.0);
    let ay = Range::exact(base.anchor.y * source.1);
    Ok([
        a,
        b,
        c,
        d,
        x.times(Range::exact(factor.0))
            .minus(a.times(ax))
            .minus(c.times(ay)),
        y.times(Range::exact(factor.1))
            .minus(b.times(ax))
            .minus(d.times(ay)),
    ])
}
fn root_window(low: u64, high: u64, delta: i128, end: u64) -> (u64, u64) {
    let shift = |time: u64| (i128::from(time) + delta).clamp(0, i128::from(end)) as u64;
    (shift(low), shift(high))
}
fn stage_matrix(stage: &EvaluatedAncestorStage, low: u64, high: u64) -> Result<Matrix, CoreError> {
    let Some(animation) = &stage.animation else {
        return Ok(exact(stage.matrix));
    };
    let time = |at| {
        SampleTime::local(
            at,
            animation.start_ms,
            Some((animation.clock.rate, animation.clock.offset)),
        )
    };
    transform(
        animation.base_transform,
        (
            f64::from(animation.source_canvas.0),
            f64::from(animation.source_canvas.1),
        ),
        animation.clock.canvas,
        &animation.channels,
        time(low),
        time(high),
    )
}

fn inverse_matrix(matrix: Matrix, determinant: Range) -> Matrix {
    let [a, b, c, d, x, y] = matrix;
    let reciprocal = determinant.reciprocal();
    [
        d.times(reciprocal),
        Range::exact(0.).minus(b).times(reciprocal),
        Range::exact(0.).minus(c).times(reciprocal),
        a.times(reciprocal),
        c.times(y).minus(d.times(x)).times(reciprocal),
        b.times(x).minus(a.times(y)).times(reciprocal),
    ]
}
fn stage_inverse(stage: &EvaluatedAncestorStage, low: u64, high: u64) -> Result<Matrix, CoreError> {
    let Some(animation) = &stage.animation else {
        return Ok(exact(stage.inverse));
    };
    let time = |at| {
        SampleTime::local(
            at,
            animation.start_ms,
            Some((animation.clock.rate, animation.clock.offset)),
        )
    };
    let scale = |property, fallback| {
        animation
            .channels
            .iter()
            .find(|c| c.property == property)
            .map_or(Ok(Range::exact(fallback)), |c| {
                channel_range(c, time(low), time(high))
            })
    };
    let determinant = scale(P::ScaleX, animation.base_transform.scale_x)?
        .times(scale(P::ScaleY, animation.base_transform.scale_y)?);
    Ok(inverse_matrix(stage_matrix(stage, low, high)?, determinant))
}
fn finite_matrix(matrix: Matrix) -> Result<Matrix, CoreError> {
    if matrix.iter().all(|range| range.finite()) {
        Ok(matrix)
    } else {
        Err(invalid(
            "continuous composed inverse/query map must be finite",
        ))
    }
}
fn stage_pair(
    stages: &[EvaluatedAncestorStage],
    low: u64,
    high: u64,
) -> Result<(Matrix, Matrix), CoreError> {
    let mut forward = exact(IDENTITY_MATRIX);
    let mut inverse = forward;
    for stage in stages {
        forward = finite_matrix(product(forward, stage_matrix(stage, low, high)?))?;
        inverse = finite_matrix(product(stage_inverse(stage, low, high)?, inverse))?;
    }
    Ok((forward, inverse))
}
fn legacy_range(
    layer: &EvaluatedVisualLayer,
    property: EvaluatedProperty,
    fallback: f64,
    y_axis: bool,
    low: SampleTime,
    high: SampleTime,
) -> Result<Range, CoreError> {
    let mut result = None;
    // Preserve exact old sampling, including legacy quadratic easing. Its
    // monotone segments use endpoint progress bounds, parameterized segments
    // use the canonical analytic curve extrema.
    let frames: Vec<_> = layer
        .keyframes
        .iter()
        .filter(|k| k.property == property)
        .collect();
    if frames.is_empty() {
        return Ok(Range::exact(fallback));
    }
    let scalar = |k: &EvaluatedKeyframe| match k.value {
        EvaluatedKeyframeValue::Scalar { value } => value,
        EvaluatedKeyframeValue::Position { x, y } => {
            if y_axis {
                y
            } else {
                x
            }
        }
    };
    let channel = crate::AnimationChannel {
        property: P::PositionX,
        target: None,
        keyframes: frames
            .iter()
            .map(|k| crate::AnimationChannelKeyframe {
                time_ms: k.time_ms,
                value: V::Scalar { value: scalar(k) },
                curve: crate::AnimationCurve::Simple(crate::SimpleAnimationCurve::Linear),
            })
            .collect(),
        r#loop: frames[0].r#loop,
        clock: frames[0].clock,
    };
    let (low, high) = if channel.r#loop.is_some() {
        (
            SampleTime::Integer(0),
            SampleTime::Integer(frames.last().unwrap().time_ms),
        )
    } else {
        (
            low.looped(&channel)
                .ok_or_else(|| invalid("aggregate legacy clock"))?,
            high.looped(&channel)
                .ok_or_else(|| invalid("aggregate legacy clock"))?,
        )
    };
    let mut include = |v: f64| {
        result = Some(result.map_or(Range(v, v), |r: Range| Range(r.0.min(v), r.1.max(v))));
    };
    if low.compare(frames[0].time_ms) != Some(std::cmp::Ordering::Greater) {
        include(scalar(frames[0]));
    }
    if high.compare(frames.last().unwrap().time_ms) != Some(std::cmp::Ordering::Less) {
        include(scalar(frames.last().unwrap()));
    }
    for pair in frames.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if high.compare(a.time_ms) == Some(std::cmp::Ordering::Less)
            || low.compare(b.time_ms) == Some(std::cmp::Ordering::Greater)
        {
            continue;
        }
        let p = low.progress(a.time_ms, b.time_ms).clamp(0., 1.);
        let q = high.progress(a.time_ms, b.time_ms).clamp(0., 1.);
        let easing = |t: f64| match a.easing {
            EvaluatedEasing::Hold => 0.,
            EvaluatedEasing::EaseIn => t * t,
            EvaluatedEasing::EaseOut => 1. - (1. - t).powi(2),
            EvaluatedEasing::EaseInOut => {
                if t < 0.5 {
                    2. * t * t
                } else {
                    1. - (-2. * t + 2.).powi(2) / 2.
                }
            }
            _ => t,
        };
        let bounds = match a.easing {
            EvaluatedEasing::Spring {
                mass,
                stiffness,
                damping,
                initial_velocity,
            } => crate::animation::curve_bounds(
                crate::AnimationCurve::Parameterized(crate::ParameterizedAnimationCurve::Spring {
                    mass,
                    stiffness,
                    damping,
                    initial_velocity,
                }),
                p,
                q,
            ),
            EvaluatedEasing::CubicBezier { x1, y1, x2, y2 } => crate::animation::curve_bounds(
                crate::AnimationCurve::Parameterized(
                    crate::ParameterizedAnimationCurve::CubicBezier { x1, y1, x2, y2 },
                ),
                p,
                q,
            ),
            _ => (easing(p), easing(q)),
        };
        for progress in [bounds.0, bounds.1] {
            let v = scalar(a) + (scalar(b) - scalar(a)) * progress;
            include(
                if matches!(
                    property,
                    EvaluatedProperty::Scale
                        | EvaluatedProperty::ScaleX
                        | EvaluatedProperty::ScaleY
                ) {
                    v.clamp(0.000001, 100.)
                } else {
                    v
                },
            );
        }
    }
    result.ok_or_else(|| invalid("aggregate legacy envelope"))
}
#[derive(Clone, Copy)]
struct Support<'a> {
    stages: &'a [EvaluatedAncestorStage],
    layer: Option<&'a EvaluatedVisualLayer>,
    basis: (u32, u32),
    delta: i128,
    frozen: usize,
    // Intrinsic bound covers source, own masks/paint gutter and ordered FX.
    bounds: [f64; 4],
}
fn local_matrix(support: Support<'_>, low: u64, high: u64, end: u64) -> Result<Matrix, CoreError> {
    let Some(layer) = support.layer else {
        return Ok(exact(IDENTITY_MATRIX));
    };
    let (low, high) = root_window(low, high, support.delta, end);
    let time = |at| {
        SampleTime::local(
            at,
            layer.span.start_ms,
            layer.instance.map(|i| (i.rate, i.offset)),
        )
    };
    let canvas = layer.instance.map_or(support.basis, |i| i.canvas);
    let mut base = layer.transform2d.unwrap_or(crate::Transform2D {
        position: crate::TransformPosition {
            x: layer.transform.position_x,
            y: layer.transform.position_y,
            unit: crate::PositionUnit::Pixels,
        },
        scale_x: layer.transform.scale,
        scale_y: layer.transform.scale,
        ..Default::default()
    });
    let mut matrix = transform(
        base,
        (0., 0.),
        canvas,
        layer.extended.as_ref().map_or(&[], |e| e.channels.as_ref()),
        time(low),
        time(high),
    )?;
    if layer.transform2d.is_none() {
        matrix[4] = legacy_range(
            layer,
            EvaluatedProperty::PositionX,
            layer.transform.position_x,
            false,
            time(low),
            time(high),
        )?;
        matrix[5] = legacy_range(
            layer,
            EvaluatedProperty::PositionY,
            layer.transform.position_y,
            true,
            time(low),
            time(high),
        )?;
        let position = legacy_range(
            layer,
            EvaluatedProperty::Position,
            layer.transform.position_x,
            false,
            time(low),
            time(high),
        )?;
        if layer
            .keyframes
            .iter()
            .any(|k| k.property == EvaluatedProperty::Position)
            && !layer
                .keyframes
                .iter()
                .any(|k| k.property == EvaluatedProperty::PositionX)
        {
            matrix[4] = position;
        }
        if layer
            .keyframes
            .iter()
            .any(|k| k.property == EvaluatedProperty::Position)
            && !layer
                .keyframes
                .iter()
                .any(|k| k.property == EvaluatedProperty::PositionY)
        {
            matrix[5] = legacy_range(
                layer,
                EvaluatedProperty::Position,
                layer.transform.position_y,
                true,
                time(low),
                time(high),
            )?;
        }
        let scale = legacy_range(
            layer,
            EvaluatedProperty::Scale,
            base.scale_x,
            false,
            time(low),
            time(high),
        )?;
        let sx = legacy_range(
            layer,
            EvaluatedProperty::ScaleX,
            scale.0,
            false,
            time(low),
            time(high),
        )?;
        let sy = legacy_range(
            layer,
            EvaluatedProperty::ScaleY,
            scale.0,
            false,
            time(low),
            time(high),
        )?;
        let sx = if layer
            .keyframes
            .iter()
            .any(|k| k.property == EvaluatedProperty::ScaleX)
        {
            sx
        } else {
            scale
        };
        let sy = if layer
            .keyframes
            .iter()
            .any(|k| k.property == EvaluatedProperty::ScaleY)
        {
            sy
        } else {
            scale
        };
        base.scale_x = 1.;
        base.scale_y = 1.;
        let linear = transform(
            base,
            (0., 0.),
            canvas,
            layer.extended.as_ref().map_or(&[], |e| e.channels.as_ref()),
            time(low),
            time(high),
        )?;
        for index in 0..2 {
            matrix[index] = linear[index].times(sx);
            matrix[index + 2] = linear[index + 2].times(sy);
        }
    }
    let anchor = match &layer.source {
        EvaluatedVisualSource::Shape(shape) if layer.transform2d.is_some() => {
            if let Some(channel) = layer
                .extended
                .as_ref()
                .and_then(|e| e.channels.iter().find(|c| c.property == P::PathPoints))
            {
                let (_, _, bounds) = extended_certification::path_envelope_facts(channel, shape)?;
                // A sampled Path axis can collapse even when the global hull
                // does not. Runtime then uses a one-unit anchor extent.
                [
                    Range::checked(bounds[0], bounds[2] + base.anchor.x),
                    Range::checked(bounds[1], bounds[3] + base.anchor.y),
                ]
            } else {
                let extent = |axis| {
                    let extent = shape.bounds[axis + 2] - shape.bounds[axis];
                    if extent == 0. && matches!(shape.geometry, crate::ShapeGeometry::Path { .. }) {
                        1.
                    } else {
                        extent
                    }
                };
                [
                    Range::exact(shape.bounds[0] + base.anchor.x * extent(0)),
                    Range::exact(shape.bounds[1] + base.anchor.y * extent(1)),
                ]
            }
        }
        EvaluatedVisualSource::Shape(_) => [Range::exact(0.); 2],
        _ if layer.transform2d.is_none() => {
            let (x, y) = layer.legacy_anchor(layer.source_size.unwrap_or(support.basis));
            [Range::exact(x), Range::exact(y)]
        }
        _ => {
            if let Some([x, y, w, h]) = layer.text_logical_box() {
                [
                    Range::exact(x + base.anchor.x * w),
                    Range::exact(y + base.anchor.y * h),
                ]
            } else {
                let size = layer.source_size.unwrap_or(support.basis);
                [
                    Range::exact(base.anchor.x * f64::from(size.0)),
                    Range::exact(base.anchor.y * f64::from(size.1)),
                ]
            }
        }
    };
    matrix[4] = matrix[4]
        .minus(matrix[0].times(anchor[0]))
        .minus(matrix[2].times(anchor[1]));
    matrix[5] = matrix[5]
        .minus(matrix[1].times(anchor[0]))
        .minus(matrix[3].times(anchor[1]));
    Ok(matrix)
}
fn stage_window(support: Support<'_>, index: usize, low: u64, high: u64, end: u64) -> (u64, u64) {
    if index < support.frozen {
        (low, high)
    } else {
        root_window(low, high, support.delta, end)
    }
}
fn composed(
    support: Support<'_>,
    skip: usize,
    low: u64,
    high: u64,
    end: u64,
) -> Result<Matrix, CoreError> {
    let mut result = local_matrix(support, low, high, end)?;
    for index in (skip..support.stages.len()).rev() {
        let (a, b) = stage_window(support, index, low, high, end);
        result = product(stage_matrix(&support.stages[index], a, b)?, result);
    }
    Ok(result)
}
fn center(bounds: [f64; 4]) -> ([Range; 2], [f64; 2]) {
    (
        [
            Range::exact((bounds[0] + bounds[2]) / 2.),
            Range::exact((bounds[1] + bounds[3]) / 2.),
        ],
        [(bounds[2] - bounds[0]) / 2., (bounds[3] - bounds[1]) / 2.],
    )
}
fn radii(matrix: Matrix, extent: [f64; 2]) -> [f64; 2] {
    [
        matrix[0].times(Range::exact(extent[0])).magnitude()
            + matrix[2].times(Range::exact(extent[1])).magnitude(),
        matrix[1].times(Range::exact(extent[0])).magnitude()
            + matrix[3].times(Range::exact(extent[1])).magnitude(),
    ]
}
fn separation(
    first: Support<'_>,
    second: Support<'_>,
    low: u64,
    high: u64,
    end: u64,
) -> Result<[f64; 2], CoreError> {
    let mut common = 0;
    while common < first.stages.len().min(second.stages.len())
        && first.stages[common] == second.stages[common]
        && stage_window(first, common, low, high, end)
            == stage_window(second, common, low, high, end)
    {
        common += 1;
    }
    let mut prefix = exact(IDENTITY_MATRIX);
    for index in 0..common {
        let (a, b) = stage_window(first, index, low, high, end);
        prefix = product(prefix, stage_matrix(&first.stages[index], a, b)?);
    }
    prefix[4] = Range::exact(0.);
    prefix[5] = Range::exact(0.);
    let a = composed(first, common, low, high, end)?;
    let b = composed(second, common, low, high, end)?;
    let (ac, ar) = center(first.bounds);
    let (bc, br) = center(second.bounds);
    let mut av = point(a, ac);
    let mut bv = point(b, bc);
    // A surface's own translation cannot enlarge its simultaneous extent.
    if first.layer.map(|l| l as *const _) == second.layer.map(|l| l as *const _)
        && first.delta == second.delta
        && first.stages == second.stages
    {
        av = point(
            [a[0], a[1], a[2], a[3], Range::exact(0.), Range::exact(0.)],
            ac,
        );
        bv = point(
            [b[0], b[1], b[2], b[3], Range::exact(0.), Range::exact(0.)],
            bc,
        );
        if first.bounds == second.bounds {
            av = [Range::exact(0.); 2];
            bv = av;
        }
    }
    let difference = point(prefix, [av[0].minus(bv[0]), av[1].minus(bv[1])]);
    let ra = radii(product(prefix, a), ar);
    let rb = radii(product(prefix, b), br);
    Ok([
        difference[0].magnitude() + ra[0] + rb[0],
        difference[1].magnitude() + ra[1] + rb[1],
    ])
}
fn effect_padding(effects: &[crate::VisualEffect]) -> f64 {
    effects
        .iter()
        .map(|e| match e {
            crate::VisualEffect::GaussianBlur { radius_px, .. }
            | crate::VisualEffect::Glow { radius_px, .. } => (3. * radius_px).ceil(),
            crate::VisualEffect::ParticleOverlay { radius_px, .. } => radius_px.ceil(),
            _ => 0.,
        })
        .sum()
}

fn local_inverse(support: Support<'_>, low: u64, high: u64, end: u64) -> Result<Matrix, CoreError> {
    let Some(layer) = support.layer else {
        return Ok(exact(IDENTITY_MATRIX));
    };
    let (a, b) = root_window(low, high, support.delta, end);
    let time = |at| {
        SampleTime::local(
            at,
            layer.span.start_ms,
            layer.instance.map(|i| (i.rate, i.offset)),
        )
    };
    let channels = layer
        .extended
        .as_ref()
        .map_or(&[][..], |e| e.channels.as_ref());
    let scale = |property, legacy, fallback| -> Result<Range, CoreError> {
        if let Some(channel) = channels.iter().find(|c| c.property == property) {
            return channel_range(channel, time(a), time(b));
        }
        if layer.transform2d.is_some() {
            return Ok(Range::exact(fallback));
        }
        if layer.keyframes.iter().any(|k| k.property == legacy) {
            legacy_range(layer, legacy, fallback, false, time(a), time(b))
        } else {
            legacy_range(
                layer,
                EvaluatedProperty::Scale,
                fallback,
                false,
                time(a),
                time(b),
            )
        }
    };
    let determinant = scale(
        P::ScaleX,
        EvaluatedProperty::ScaleX,
        layer
            .transform2d
            .map_or(layer.transform.scale, |t| t.scale_x),
    )?
    .times(scale(
        P::ScaleY,
        EvaluatedProperty::ScaleY,
        layer
            .transform2d
            .map_or(layer.transform.scale, |t| t.scale_y),
    )?);
    finite_matrix(inverse_matrix(
        local_matrix(support, low, high, end)?,
        determinant,
    ))
}
fn support_pair(
    support: Support<'_>,
    low: u64,
    high: u64,
    end: u64,
) -> Result<(Matrix, Matrix), CoreError> {
    let mut ancestor = exact(IDENTITY_MATRIX);
    let mut ancestor_inverse = ancestor;
    for (index, stage) in support.stages.iter().enumerate() {
        let (a, b) = stage_window(support, index, low, high, end);
        ancestor = finite_matrix(product(ancestor, stage_matrix(stage, a, b)?))?;
        ancestor_inverse = finite_matrix(product(stage_inverse(stage, a, b)?, ancestor_inverse))?;
    }
    let local = local_matrix(support, low, high, end)?;
    let inverse = local_inverse(support, low, high, end)?;
    let forward = finite_matrix(product(ancestor, local))?;
    let mut inverse = finite_matrix(product(inverse, ancestor_inverse))?;
    if let Some(layer) = support.layer
        && let EvaluatedVisualSource::Shape(shape) = &layer.source
    {
        let density = Range::exact(shape.density);
        // The full authored/path/paint/FX support encloses every signed sampled
        // raster origin; inversion converts logical positions to raster pixels.
        let bounds = leaf_bounds(layer)?;
        let origin_x = Range::checked(bounds[0], bounds[2]);
        let origin_y = Range::checked(bounds[1], bounds[3]);
        let conversion = [
            density.reciprocal(),
            Range::exact(0.),
            Range::exact(0.),
            density.reciprocal(),
            origin_x,
            origin_y,
        ];
        // Shape runtime inverts the complete raster forward matrix using its
        // computed determinant. A finite analytic inverse alone cannot prove
        // that this determinant avoids floating-point underflow/cancellation.
        let raster = finite_matrix(product(forward, conversion))?;
        let determinant = raster[0].times(raster[3]).minus(raster[1].times(raster[2]));
        inverse = finite_matrix(inverse_matrix(raster, determinant))?;
    }
    Ok((forward, inverse))
}
fn mapped_ranges(matrix: Matrix, rectangle: [Range; 2]) -> Result<[Range; 2], CoreError> {
    let mapped = point(matrix, rectangle);
    if mapped.iter().all(|r| r.finite()) {
        Ok(mapped)
    } else {
        Err(invalid(
            "continuous mapped query coordinates must be finite",
        ))
    }
}
fn support_rectangle(support: Support<'_>) -> [Range; 2] {
    [
        Range::checked(support.bounds[0], support.bounds[2]),
        Range::checked(support.bounds[1], support.bounds[3]),
    ]
}
fn local_rectangle(
    scene: &EvaluatedScene,
    owner: usize,
    supports: &[Support<'_>],
    low: u64,
    high: u64,
) -> Result<[Range; 2], CoreError> {
    let node = &scene.aggregates.as_ref().unwrap().nodes[owner];
    let mut rectangle = if node.clip {
        [
            Range(0., f64::from(node.basis.0)),
            Range(0., f64::from(node.basis.1)),
        ]
    } else {
        let mut bounds: Option<[Range; 2]> = None;
        for support in supports {
            let matrix = composed(*support, 0, low, high, scene.duration_ms.saturating_sub(1))?;
            let mapped = mapped_ranges(matrix, support_rectangle(*support))?;
            bounds = Some(bounds.map_or(mapped, |old| {
                [
                    Range::checked(old[0].0.min(mapped[0].0), old[0].1.max(mapped[0].1)),
                    Range::checked(old[1].0.min(mapped[1].0), old[1].1.max(mapped[1].1)),
                ]
            }));
        }
        bounds.unwrap_or([Range(0., 1.); 2])
    };
    // Nested output padding and both signed raster rounds are conservative;
    // this absolute-origin proof never determines the simultaneous dimensions.
    let mut padding = [effect_padding(&node.effects) + 4.; 2];
    for (index, child) in scene.aggregates.as_ref().unwrap().nodes.iter().enumerate() {
        if index == owner || blocked(scene.aggregates.as_ref().unwrap(), child.parent, owner) {
            continue;
        }
        let (matrix, _) = stage_pair(
            &child.stages[owner_boundary(&child.stages, owner)?..],
            low,
            high,
        )?;
        let child_pad = effect_padding(&child.effects) + 2.;
        let expanded = radii(matrix, [child_pad; 2]);
        for axis in 0..2 {
            padding[axis] += expanded[axis];
        }
    }
    for (r, pad) in rectangle.iter_mut().zip(padding) {
        *r = r.plus(Range(-pad, pad));
    }
    Ok(rectangle)
}
fn certify_query_maps(
    scene: &EvaluatedScene,
    low: u64,
    high: u64,
    nodes: &mut usize,
) -> Result<(), CoreError> {
    let graph = scene.aggregates.as_ref().unwrap();
    let end = scene.duration_ms.saturating_sub(1);
    let root = [
        Range(0., f64::from(scene.canvas.width)),
        Range(0., f64::from(scene.canvas.height)),
    ];
    certify_private_query(scene, low, high, nodes, exact(IDENTITY_MATRIX), root, true)?;
    for owner in 0..graph.nodes.len() {
        extended_certification::charge(nodes)?;
        let support = supports(scene, owner, nodes)?;
        let rectangle = local_rectangle(scene, owner, &support, low, high)?;
        let node = &graph.nodes[owner];
        let (world, _) = stage_pair(&node.stages, low, high)?;
        mapped_ranges(world, rectangle)?;
        let (_, paint_inverse) = stage_pair(outward_stages(node)?, low, high)?;
        let destination = if let Some(parent) = node.parent {
            let parent_support = supports(scene, parent, nodes)?;
            local_rectangle(scene, parent, &parent_support, low, high)?
        } else {
            root
        };
        mapped_ranges(paint_inverse, destination)?;
        for source in support.iter().filter(|s| s.layer.is_some()) {
            extended_certification::charge(nodes)?;
            let (_, inverse) = support_pair(*source, low, high, end)?;
            mapped_ranges(inverse, rectangle)?;
        }
        // Every named provider may be requested transitively on this grid.
        // Full-world ancestry and its own shutter remain independent of controls.
        certify_private_query(scene, low, high, nodes, world, rectangle, false)?;
    }
    Ok(())
}
fn certify_private_query(
    scene: &EvaluatedScene,
    low: u64,
    high: u64,
    nodes: &mut usize,
    world: Matrix,
    rectangle: [Range; 2],
    root: bool,
) -> Result<(), CoreError> {
    let end = scene.duration_ms.saturating_sub(1);
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        let provider = scene.mattes.as_ref().is_some_and(|graph| {
            graph.roles[index].group.is_some_and(|group| {
                graph
                    .groups
                    .iter()
                    .any(|node| node.provider.is_some_and(|(id, _)| id == group))
            })
        });
        let root_direct = root
            && leaf_owner(layer).is_none()
            && scene
                .mattes
                .as_ref()
                .is_none_or(|g| g.roles[index].contributes && !g.roles[index].matte_only);
        if !provider && !root_direct {
            continue;
        }
        if layer.source_size.is_none() && matches!(layer.source, EvaluatedVisualSource::Text(_)) {
            continue;
        }
        for delta in mattes::shutter_offsets(layer)? {
            extended_certification::charge(nodes)?;
            let provider = Support {
                stages: &layer.ancestor_stages,
                layer: Some(layer),
                basis: (scene.canvas.width, scene.canvas.height),
                delta,
                frozen: 0,
                bounds: leaf_bounds(layer)?,
            };
            let (_, inverse) = support_pair(provider, low, high, end)?;
            mapped_ranges(finite_matrix(product(inverse, world))?, rectangle)?;
        }
    }
    Ok(())
}
fn leaf_bounds(layer: &EvaluatedVisualLayer) -> Result<[f64; 4], CoreError> {
    let (mut bounds, density) = match &layer.source {
        EvaluatedVisualSource::Shape(shape) => (
            [
                shape.origin.0,
                shape.origin.1,
                shape.origin.0 + f64::from(shape.size.0) / shape.density,
                shape.origin.1 + f64::from(shape.size.1) / shape.density,
            ],
            shape.density,
        ),
        _ => {
            let (w, h) = layer
                .source_size
                .ok_or_else(|| invalid("aggregate source geometry unavailable"))?;
            ([0., 0., f64::from(w), f64::from(h)], 1.)
        }
    };
    if let EvaluatedVisualSource::Shape(shape) = &layer.source
        && let Some(channel) = layer
            .extended
            .as_ref()
            .and_then(|e| e.channels.iter().find(|c| c.property == P::PathPoints))
    {
        let stroke = shape
            .stroke
            .as_ref()
            .map_or(0., |s| s.width * 0.5 * s.miter_limit.max(2.));
        let (_, _, envelope) = extended_certification::path_envelope_facts(channel, shape)?;
        bounds[0] = bounds[0].min(envelope[0] - stroke - 2. / density);
        bounds[1] = bounds[1].min(envelope[1] - stroke - 2. / density);
        bounds[2] = bounds[2].max(envelope[2] + stroke + 2. / density);
        bounds[3] = bounds[3].max(envelope[3] + stroke + 2. / density);
    }
    let mut pad = 0.;
    if let Some(extended) = &layer.extended {
        for effect in &extended.effects {
            let (radius, property) = match effect {
                crate::VisualEffect::GaussianBlur { radius_px, .. } => {
                    (*radius_px, Some(P::BlurRadius))
                }
                crate::VisualEffect::Glow { radius_px, .. } => (*radius_px, Some(P::GlowRadius)),
                crate::VisualEffect::ParticleOverlay { radius_px, .. } => {
                    pad += (*radius_px * density).ceil() / density;
                    continue;
                }
                _ => continue,
            };
            let maximum = extended
                .channels
                .iter()
                .find(|c| {
                    Some(c.property) == property
                        && c.target.as_ref().is_some_and(|t| t.id == effect.id())
                })
                .map_or(Ok(radius), |c| {
                    channel_range(
                        c,
                        SampleTime::Integer(0),
                        SampleTime::Integer(
                            c.keyframes
                                .last()
                                .ok_or_else(|| invalid("effect channel empty"))?
                                .time_ms,
                        ),
                    )
                    .map(|r| r.1)
                })?;
            pad += (3. * maximum * density).ceil() / density;
        }
    }
    Ok([
        bounds[0] - pad - 1. / density,
        bounds[1] - pad - 1. / density,
        bounds[2] + pad + 1. / density,
        bounds[3] + pad + 1. / density,
    ])
}
fn owner_boundary(stages: &[EvaluatedAncestorStage], owner: usize) -> Result<usize, CoreError> {
    stages
        .iter()
        .rposition(|s| s.aggregate == Some(owner))
        .map(|i| i + 1)
        .ok_or_else(|| invalid("aggregate domain owner boundary missing"))
}
fn blocked(graph: &AggregateGraph, mut node: Option<usize>, owner: usize) -> bool {
    while let Some(index) = node {
        if index == owner {
            return false;
        }
        if graph.nodes[index].clip {
            return true;
        }
        node = graph.nodes[index].parent;
    }
    true
}
fn supports<'a>(
    scene: &'a EvaluatedScene,
    owner: usize,
    nodes: &mut usize,
) -> Result<Vec<Support<'a>>, CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate graph absent"))?;
    let maximum = scene
        .visual_layers
        .len()
        .checked_mul(16)
        .and_then(|n| n.checked_add(graph.nodes.len()))
        .ok_or_else(|| invalid("continuous support count overflow"))?;
    let reservation = mul(maximum as u64, std::mem::size_of::<Support<'_>>() as u64)?;
    if add(
        add(
            composition_resources::composition_heap_bytes(scene)?,
            mattes::MATTE_CACHE_RESERVATION,
        )?,
        mul(reservation, 4)?,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("continuous support metadata exceeds memory"));
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(maximum)
        .map_err(|_| invalid("continuous support allocation failed"))?;
    for (index, node) in graph.nodes.iter().enumerate() {
        if index == owner || blocked(graph, node.parent, owner) {
            continue;
        }
        if node.clip
            || node
                .effects
                .iter()
                .any(|e| matches!(e, crate::VisualEffect::ParticleOverlay { .. }))
        {
            extended_certification::charge(nodes)?;
            let stages = &node.stages[owner_boundary(&node.stages, owner)?..];
            let pad = effect_padding(&node.effects);
            result.push(Support {
                stages,
                layer: None,
                basis: node.basis,
                delta: 0,
                frozen: stages.len(),
                bounds: [
                    -pad,
                    -pad,
                    f64::from(node.basis.0) + pad,
                    f64::from(node.basis.1) + pad,
                ],
            });
        }
    }
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        // The transport-free pass cannot measure font bytes. The existing
        // staged-resource pass supplies this geometry before publication.
        if layer.source_size.is_none() && matches!(layer.source, EvaluatedVisualSource::Text(_)) {
            continue;
        }
        if blocked(graph, leaf_owner(layer), owner)
            || scene
                .mattes
                .as_ref()
                .is_some_and(|g| !g.roles[index].contributes || g.roles[index].matte_only)
        {
            continue;
        }
        let stages = &layer.ancestor_stages[owner_boundary(&layer.ancestor_stages, owner)?..];
        let frozen = stages
            .iter()
            .rposition(|s| s.aggregate.is_some())
            .map_or(0, |i| i + 1);
        let bounds = leaf_bounds(layer)?;
        for delta in mattes::shutter_offsets(layer)? {
            extended_certification::charge(nodes)?;
            result.push(Support {
                stages,
                layer: Some(layer),
                basis: graph.nodes[owner].basis,
                delta,
                frozen,
                bounds,
            });
        }
    }
    let node = &graph.nodes[owner];
    if node
        .effects
        .iter()
        .any(|e| matches!(e, crate::VisualEffect::ParticleOverlay { .. }))
        || result.is_empty()
    {
        result.push(Support {
            stages: &[],
            layer: None,
            basis: node.basis,
            delta: 0,
            frozen: 0,
            bounds: if node
                .effects
                .iter()
                .any(|e| matches!(e, crate::VisualEffect::ParticleOverlay { .. }))
            {
                [0., 0., f64::from(node.basis.0), f64::from(node.basis.1)]
            } else {
                [0., 0., 1., 1.]
            },
        });
    }
    Ok(result)
}
/// Bound width/height at the same output sample, never by whole-time travel.
fn domain_size(
    scene: &EvaluatedScene,
    owner: usize,
    supports: &[Support<'_>],
    low: u64,
    high: u64,
    nodes: &mut usize,
) -> Result<(u32, u32), CoreError> {
    let graph = scene.aggregates.as_ref().unwrap();
    let node = &graph.nodes[owner];
    if node.clip {
        return Ok(node.basis);
    }
    let mut extent = [1f64; 2];
    for (index, first) in supports.iter().enumerate() {
        for second in &supports[index..] {
            extended_certification::charge(nodes)?;
            let difference = separation(
                *first,
                *second,
                low,
                high,
                scene.duration_ms.saturating_sub(1),
            )?;
            for axis in 0..2 {
                extent[axis] = extent[axis].max(difference[axis]);
            }
        }
    }
    // Every inward child-controlled output's effect support remains part of
    // this owner's union. Bound its magnified halo independently of translation.
    let mut nested_pad = [0f64; 2];
    for (index, child) in graph.nodes.iter().enumerate() {
        if index == owner || blocked(graph, child.parent, owner) {
            continue;
        }
        let pad = effect_padding(&child.effects) + 2.;
        let mut matrix = exact(IDENTITY_MATRIX);
        for stage in &child.stages[owner_boundary(&child.stages, owner)?..] {
            matrix = product(matrix, stage_matrix(stage, low, high)?);
        }
        let radii = radii(matrix, [pad, pad]);
        for axis in 0..2 {
            nested_pad[axis] += radii[axis];
        }
    }
    // sampled_support_bounds first rounds each source and the aggregate then
    // rounds the resulting union. Four pixels enclose both signed edge rounds.
    let width = (extent[0] + 2. * nested_pad[0] + 4.).ceil();
    let height = (extent[1] + 2. * nested_pad[1] + 4.).ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width > 16384.
        || height > 16384.
        || width * height > 16_777_216.
    {
        return Err(invalid(
            "continuous signed aggregate surface exceeds bounds",
        ));
    }
    Ok((width as u32, height as u32))
}
/// Certify owner-domain geometry and charge owner effects into the same output
/// ledger as every direct/private query visit. Legacy continuous owners remain
/// responsible for source/mask/geometry/full-world provider certification.
pub(in crate::evaluated_scene) fn certify(
    scene: &EvaluatedScene,
    nodes: &mut usize,
    peak_source_memory: u64,
    mut verify: impl FnMut(&[u64], u64) -> Result<(), CoreError>,
) -> Result<(), CoreError> {
    let graph = scene
        .aggregates
        .as_ref()
        .ok_or_else(|| invalid("aggregate graph absent"))?;
    let descriptors = graph
        .nodes
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(scene.visual_layers.len() + 1))
        .filter(|n| *n <= MAX_EVALUATED_VISUAL_LAYERS)
        .ok_or_else(|| invalid("aggregate/query descriptor limit exceeded"))?;
    if add(
        add(
            composition_resources::composition_heap_bytes(scene)?,
            mattes::MATTE_CACHE_RESERVATION,
        )?,
        mul(descriptors as u64, 8192)?,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid("continuous aggregate metadata exceeds memory"));
    }
    let clone_reserve = scene
        .visual_layers
        .iter()
        .try_fold(0u64, |maximum, layer| {
            Ok::<_, CoreError>(
                maximum.max(mul(composition_resources::layer_heap_bytes(layer)?, 3)?),
            )
        })?;
    if add(
        add(
            composition_resources::composition_heap_bytes(scene)?,
            mattes::MATTE_CACHE_RESERVATION,
        )?,
        add(mul(descriptors as u64, 8192)?, clone_reserve)?,
    )? > mattes::MAX_MATTE_LIVE_BYTES
    {
        return Err(invalid(
            "continuous query/source clone admission exceeds shared memory",
        ));
    }
    let mut pending = vec![(0, scene.duration_ms.saturating_sub(1))];
    while let Some((low, high)) = pending.pop() {
        extended_certification::charge(nodes)?;
        let attempt = (|| {
            certify_query_maps(scene, low, high, nodes)?;
            let mut queries = Vec::new();
            queries
                .try_reserve_exact(graph.nodes.len() + 1)
                .map_err(|_| invalid("continuous query allocation failed"))?;
            let mut owner_work = 0;
            let mut retained = mul(
                mul(
                    u64::from(scene.canvas.width),
                    u64::from(scene.canvas.height),
                )?,
                16,
            )?;
            let root_bytes = mul(
                mul(
                    u64::from(scene.canvas.width),
                    u64::from(scene.canvas.height),
                )?,
                56,
            )?;
            let source_extra = peak_source_memory
                .saturating_sub(add(mattes::MATTE_CACHE_RESERVATION, root_bytes)?);
            let mut transient = source_extra;

            for owner in 0..graph.nodes.len() {
                let support = supports(scene, owner, nodes)?;
                let size = domain_size(scene, owner, &support, low, high, nodes)?;
                extended_certification::effect_budget(
                    size,
                    &graph.nodes[owner].effects,
                    1.,
                    &mut owner_work,
                )?;
                let pad = effect_padding(&graph.nodes[owner].effects) as u64;
                let output_width = add(u64::from(size.0), mul(pad, 2)?)?;
                let output_height = add(u64::from(size.1), mul(pad, 2)?)?;
                retained = add(retained, mul(mul(output_width, output_height)?, 16)?)?;
                transient = transient.max(
                    extended_visual::certify_composition_memory(
                        (scene.canvas.width, scene.canvas.height),
                        size,
                        &graph.nodes[owner].effects,
                        1.,
                    )?
                    .checked_sub(mattes::MATTE_CACHE_RESERVATION)
                    .ok_or_else(|| invalid("owner memory certificate invalid"))?,
                );
                queries.push(mattes::ContinuousQuery {
                    owner: Some(owner),
                    size,
                });
            }
            queries.push(mattes::ContinuousQuery {
                owner: None,
                size: (scene.canvas.width, scene.canvas.height),
            });
            let fixed = add(
                add(
                    composition_resources::composition_heap_bytes(scene)?,
                    mattes::MATTE_CACHE_RESERVATION,
                )?,
                add(retained, clone_reserve)?,
            )?;
            // Outputs remain retained during every query/source/owner-effect
            // transient. Certify their entire overlap before request expansion.
            if add(fixed, transient)? > mattes::MAX_MATTE_LIVE_BYTES {
                return Err(invalid(
                    "continuous aggregate retained/source overlap exceeds bounds",
                ));
            }
            mattes::certify_continuous_queries(scene, nodes, &queries, true, |visits, memory| {
                if add(
                    add(fixed, memory.descriptors)?,
                    add(memory.query_peak, source_extra)?.max(transient),
                )? > mattes::MAX_MATTE_LIVE_BYTES
                {
                    return Err(invalid(
                        "continuous aggregate/query live overlap exceeds bounds",
                    ));
                }
                verify(visits, owner_work)
            })
        })();
        match attempt {
            Ok(()) => {}
            Err(error) => {
                if low == high || error.message.contains("maxCandidateAnalysisNodes") {
                    return Err(error);
                }
                let middle = low + (high - low) / 2;
                pending.push((middle + 1, high));
                pending.push((low, middle));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn interval_overflow_cannot_disappear_in_extrema_or_zero_products() {
        let overflow = Range::exact(f64::MAX).plus(Range::exact(f64::MAX));
        assert!(!overflow.finite());
        assert_eq!(overflow.times(Range::exact(0.)).magnitude(), f64::INFINITY);
        assert_eq!(Range::exact(0.).times(overflow).magnitude(), f64::INFINITY);
        assert_eq!(Range::checked(f64::NAN, 1.).magnitude(), f64::INFINITY);
        assert_eq!(Range::checked(2., 1.).magnitude(), f64::INFINITY);
        let matrix = product(exact([f64::MAX; 6]), exact([2.; 6]));
        assert!(
            point(matrix, [Range::exact(0.); 2])
                .iter()
                .all(|r| !r.finite())
        );
        assert_eq!(radii(matrix, [0.; 2]), [f64::INFINITY; 2]);
    }
    #[test]
    fn retained_channel_clock_bounds_follow_canonical_sampling() {
        let channel: crate::AnimationChannel = serde_json::from_value(json!({
            "property":"transform.position_x",
            "clock":{"offsetMs":700,"sourceDurationMs":1000},
            "keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
                         {"timeMs":1000,"value":{"type":"scalar","value":10000},"curve":"hold"}]
        }))
        .unwrap();
        for (low, high) in [
            (SampleTime::Integer(0), SampleTime::Integer(100)),
            (SampleTime::Fractional(0.5), SampleTime::Fractional(100.5)),
        ] {
            let bound = channel_range(&channel, low, high).unwrap();
            assert!(bound.0 >= 7000.);
            for at in [low, high] {
                let value = at.scalar(&channel).unwrap();
                assert!(
                    value >= bound.0 && value <= bound.1,
                    "{value} outside {bound:?}"
                );
            }
        }
    }
    #[test]
    fn future_deep_shrinking_owner_rejects_nonfinite_inverse_before_publication() {
        let mut items = Vec::new();
        for i in 0..12 {
            let mut item = json!({"type":"group","id":format!("g{i}"),"startMs":0,"durationMs":1000,
                "zIndex":0,"stackOrder":i,
                "animationChannels":[
                    {"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":999,"value":{"type":"scalar","value":1e-30},"curve":"hold"}]},
                    {"property":"transform.scale_y","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":999,"value":{"type":"scalar","value":1e-30},"curve":"hold"}]}]});
            if i > 0 {
                item["parent"] = json!({"scope":"root","id":format!("g{}",i-1)});
            }
            if i == 11 {
                item["clip"] = json!({"type":"composition_bounds"});
            }
            items.push(item);
        }
        let scene = scene(json!(items), 64, 64);
        assert!(frame::frame_program(&scene, 0).is_ok());
        assert!(frame::frame_program(&scene, 999).is_err());
        let error = certify(&scene, &mut 0, 0, |_, _| Ok(())).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
        assert!(error.message.contains("finite"), "{error}");
    }
    #[test]
    fn complete_shape_raster_determinant_underflow_rejects_finite_analytic_inverse() {
        let mut items = Vec::new();
        for i in 0..6 {
            let mut item = json!({"type":"group","id":format!("g{i}"),"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":i,
                "animationChannels":[
                    {"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":999,"value":{"type":"scalar","value":1e-30},"curve":"hold"}]},
                    {"property":"transform.scale_y","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":999,"value":{"type":"scalar","value":1e-30},"curve":"hold"}]}]});
            if i > 0 {
                item["parent"] = json!({"scope":"root","id":format!("g{}",i-1)});
            }
            if i == 5 {
                item["clip"] = json!({"type":"composition_bounds"});
            }
            items.push(item);
        }
        items.push(json!({"type":"shape","id":"leaf","keyframes":[],"matteOnly":true,"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":6,
            "parent":{"scope":"root","id":"g5"},"geometry":{"type":"rectangle","width":8,"height":8},
            "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
            "transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}));
        items.push(json!({"type":"rectangle","id":"recipient","keyframes":[],"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":7,
            "width":8,"height":8,"color":"#00ff00","matte":{"sourceId":"leaf","channel":"alpha"}}));
        let scene = scene(json!(items), 64, 64);
        let graph = scene.aggregates.as_ref().unwrap();
        assert!(
            stage_pair(&graph.nodes[0].stages, 999, 999).is_ok(),
            "analytic inverse remains finite"
        );
        assert!(frame::frame_program(&scene, 0).is_ok());
        assert!(
            frame::frame_program(&scene, 999).is_err(),
            "actual Shape determinant underflows"
        );
        let error = certify(&scene, &mut 0, 0, |_, _| Ok(())).unwrap_err();
        assert!(error.message.contains("finite"), "{error}");
    }
    #[test]
    fn zero_extent_stroked_path_keeps_runtime_one_unit_noncentral_anchor() {
        for horizontal in [false, true] {
            let end = if horizontal {
                json!({"x":22,"y":10})
            } else {
                json!({"x":10,"y":22})
            };
            let scene = scene(
                json!([
                    {"type":"group","id":"owner","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"clip":{"type":"composition_bounds"}},
                    {"type":"shape","id":"path","keyframes":[],"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"owner"},
                    "geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":10,"y":10}},{"type":"lineTo","to":end}]}},
                    "fill":{"type":"solid","color":{"r":0,"g":0,"b":0,"a":0}},
                    "stroke":{"paint":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"width":2,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4},
                    "transform2d":{"position":{"x":20,"y":20,"unit":"pixels"},"anchor":{"x":0.75,"y":0.75},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}
                ]),
                64,
                64,
            );
            let support = supports(&scene, 0, &mut 0).unwrap();
            let leaf = support.iter().find(|s| s.layer.is_some()).unwrap();
            let matrix = local_matrix(*leaf, 0, 0, 999).unwrap();
            let position = point(matrix, [Range::exact(10.); 2]);
            let axis = usize::from(horizontal);
            assert!((position[axis].0 - 19.25).abs() < 1e-9, "{position:?}");
            certify(&scene, &mut 0, 0, |_, _| Ok(())).unwrap();
            assert!(frame::frame_program(&scene, 500).is_ok());
        }
    }
    fn scene(items: serde_json::Value, width: u32, height: u32) -> EvaluatedScene {
        let project:Project=serde_json::from_value(json!({"schemaVersion":37,"id":"p","revision":0,"name":"Continuous","createdAtMs":1,"updatedAtMs":1,
            "settings":{"width":width,"height":height,"fps":10},"assets":[],"components":[],"fonts":{},"markers":[],
            "tracks":[{"id":"t","name":"T","trackType":"overlay","items":items}]})).unwrap();
        let mut scene = evaluate_project(&project, width, height, 10).unwrap().scene;
        finalize_affine_geometry(&mut scene, &Default::default()).unwrap();
        scene
    }
    #[test]
    fn common_ancestor_travel_cancels_before_simultaneous_domain_certification() {
        let scene = scene(
            json!([
                {"type":"group","id":"owner","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
                 "effects":[{"id":"identity","type":"color_adjustment","exposureStops":0,"contrast":1,"saturation":1}]},
                {"type":"group","id":"travel","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":1,"parent":{"scope":"root","id":"owner"},
                 "animationChannels":[{"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":-100000},"curve":"linear"},{"timeMs":999,"value":{"type":"scalar","value":100000},"curve":"hold"}]}]},
                {"type":"rectangle","id":"a","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2,"width":4,"height":3,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"travel"}},
                {"type":"rectangle","id":"b","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":3,"width":4,"height":3,"color":"#00ff00","keyframes":[],"parent":{"scope":"root","id":"travel"},"transform":{"positionX":10,"positionY":0,"scale":1,"opacity":1}}
            ]),
            64,
            64,
        );
        let support = supports(&scene, 0, &mut 0).unwrap();
        let size = domain_size(&scene, 0, &support, 0, 999, &mut 0).unwrap();
        assert!(size.0 < 64 && size.1 < 64, "{size:?}");
        certify(&scene, &mut 0, 0, |visits, owner_work| {
            assert_eq!(visits, &[1, 1]);
            assert!(owner_work > 0);
            Ok(())
        })
        .unwrap();
        for at in [0, 1, 137, 499, 500, 833, 999] {
            let actual = frame::frame_program(&scene, at).unwrap().aggregates[0]
                .domain
                .size;
            assert!(
                actual.0 <= size.0 && actual.1 <= size.1,
                "{at}: actual{actual:?}, certificate{size:?}"
            );
        }
    }
    #[test]
    fn empty_particle_owner_uses_full_fixed_basis_and_identity_work_is_charged() {
        let scene = scene(
            json!([{ "type":"group","id":"owner","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,
            "effects":[{"id":"particles","type":"particle_overlay","count":0,"seed":17,"radiusPx":0,"speedPxPerSecond":0,"lifetimeMs":997,"color":{"r":1,"g":0,"b":0,"a":0}}]}]),
            512,
            384,
        );
        let support = supports(&scene, 0, &mut 0).unwrap();
        assert_eq!(support.len(), 1);
        assert_eq!(support[0].bounds, [0., 0., 512., 384.]);
        certify(&scene, &mut 0, 0, |visits, work| {
            assert!(visits.is_empty());
            assert!(work >= 512 * 384 * 4);
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn retained_empty_controlled_planes_reject_shared_live_overflow_despite_safe_effect_work() {
        let owners=(0..24).map(|i|json!({"type":"group","id":format!("owner-{i}"),"startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":i,"clip":{"type":"composition_bounds"},
            "effects":[{"id":"identity","type":"color_adjustment","exposureStops":0,"contrast":1,"saturation":1}]})).collect::<Vec<_>>();
        let scene = scene(json!(owners), 1536, 1536);
        // 24 * 3 * 1536² = 169869312 < the unchanged effect budget.
        assert!(
            scene.aggregates.as_ref().unwrap().nodes.len() as u64
                * 3
                * u64::from(scene.canvas.width)
                * u64::from(scene.canvas.height)
                < extended_certification::MAX_EFFECT_PASSES
        );
        let error = certify(&scene, &mut 0, 0, |_, _| Ok(())).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
        assert!(error.message.contains("overlap"), "{error}");
        assert!(frame::frame_program(&scene, 400).is_err());
    }
}
