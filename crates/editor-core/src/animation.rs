use crate::{
    AnimationChannel, AnimationChannelValue, AnimationClock, AnimationCurve,
    AnimationLoopIterations, AnimationLoopMode, Easing, Keyframe, KeyframeProperty, KeyframeValue,
    ParameterizedAnimationCurve, SimpleAnimationCurve,
};

/// Preserve exact root clocks until segment/loop selection. Derived instance
/// clocks retain their fractional milliseconds rather than being quantized.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SampleTime {
    Integer(u64),
    Fractional(f64),
}

impl SampleTime {
    pub(crate) fn local(at: u64, start: u64, clock: Option<(f64, f64)>) -> Self {
        match clock {
            None | Some((1.0, 0.0)) => Self::Integer(at.saturating_sub(start)),
            Some((rate, offset)) => {
                Self::Fractional((rate * at as f64 + offset - start as f64).max(0.0))
            }
        }
    }
    pub(crate) fn sample(self, channel: &AnimationChannel) -> Option<AnimationChannelValue> {
        match self {
            Self::Integer(time) => sample_channel(channel, time),
            Self::Fractional(time) => sample_channel_at(channel, time),
        }
    }
    pub(crate) fn scalar(self, channel: &AnimationChannel) -> Option<f64> {
        match self.sample(channel)? {
            AnimationChannelValue::Scalar { value } => Some(value),
            _ => None,
        }
    }
    pub(crate) fn looped(self, channel: &AnimationChannel) -> Option<Self> {
        match self {
            Self::Integer(time) => map_loop_time(channel, time).map(Self::Integer),
            Self::Fractional(time) => map_loop_time_at(channel, time).map(Self::Fractional),
        }
    }
    pub(crate) fn compare(self, key: u64) -> Option<std::cmp::Ordering> {
        match self {
            Self::Integer(time) => Some(time.cmp(&key)),
            Self::Fractional(time) => time.partial_cmp(&(key as f64)),
        }
    }
    pub(crate) fn progress(self, start: u64, end: u64) -> f64 {
        let elapsed = match self {
            Self::Integer(time) => (time - start) as f64,
            Self::Fractional(time) => time - start as f64,
        };
        elapsed / (end - start) as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SpringCoefficients {
    Underdamped {
        decay: f64,
        frequency: f64,
        sine: f64,
    },
    Critical {
        decay: f64,
        linear: f64,
    },
    Overdamped {
        slow_root: f64,
        fast_root: f64,
        slow: f64,
        fast: f64,
    },
}

pub(crate) fn spring_coefficients(
    mass: f64,
    stiffness: f64,
    damping: f64,
    initial_velocity: f64,
) -> SpringCoefficients {
    let decay = damping / (2.0 * mass);
    let natural_squared = stiffness / mass;
    let discriminant = decay * decay - natural_squared;
    if discriminant < 0.0 {
        let frequency = (-discriminant).sqrt();
        SpringCoefficients::Underdamped {
            decay,
            frequency,
            sine: (initial_velocity - decay) / frequency,
        }
    } else if discriminant == 0.0 {
        SpringCoefficients::Critical {
            decay,
            linear: initial_velocity - decay,
        }
    } else {
        let root = discriminant.sqrt();
        let slow_root = -decay + root;
        let fast_root = -decay - root;
        let slow = (initial_velocity + fast_root) / (slow_root - fast_root);
        SpringCoefficients::Overdamped {
            slow_root,
            fast_root,
            slow,
            fast: -1.0 - slow,
        }
    }
}

pub(crate) fn parameterized_curve_progress(
    curve: ParameterizedAnimationCurve,
    progress: f64,
) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    if progress == 0.0 || progress == 1.0 {
        return progress;
    }
    match curve {
        ParameterizedAnimationCurve::CubicBezier { x1, y1, x2, y2 } => {
            let mut low = 0.0;
            let mut high = 1.0;
            for _ in 0..40 {
                let midpoint = (low + high) / 2.0;
                let x = cubic_bezier(midpoint, x1, x2);
                if x <= progress {
                    low = midpoint;
                } else {
                    high = midpoint;
                }
            }
            cubic_bezier((low + high) / 2.0, y1, y2)
        }
        ParameterizedAnimationCurve::Spring {
            mass,
            stiffness,
            damping,
            initial_velocity,
        } => {
            let displacement = match spring_coefficients(mass, stiffness, damping, initial_velocity)
            {
                SpringCoefficients::Underdamped {
                    decay,
                    frequency,
                    sine,
                } => {
                    (-decay * progress).exp()
                        * (-(frequency * progress).cos() + sine * (frequency * progress).sin())
                }
                SpringCoefficients::Critical { decay, linear } => {
                    (-decay * progress).exp() * (-1.0 + linear * progress)
                }
                SpringCoefficients::Overdamped {
                    slow_root,
                    fast_root,
                    slow,
                    fast,
                } => slow * (slow_root * progress).exp() + fast * (fast_root * progress).exp(),
            };
            1.0 + displacement
        }
    }
}

fn cubic_bezier(time: f64, first: f64, second: f64) -> f64 {
    let inverse = 1.0 - time;
    3.0 * inverse * inverse * time * first
        + 3.0 * inverse * time * time * second
        + time * time * time
}

/// Closed progress envelope, including all spring stationary points. Endpoints
/// retain the sampler's exact authored-value semantics.
pub(crate) fn curve_bounds(curve: AnimationCurve, low: f64, high: f64) -> (f64, f64) {
    let progress = |t| match curve {
        AnimationCurve::Simple(SimpleAnimationCurve::Hold) => 0.0,
        AnimationCurve::Simple(SimpleAnimationCurve::Linear) => t,
        AnimationCurve::Parameterized(c) => parameterized_curve_progress(c, t),
    };
    let mut values = vec![progress(low), progress(high)];
    if let AnimationCurve::Parameterized(ParameterizedAnimationCurve::Spring {
        mass,
        stiffness,
        damping,
        initial_velocity,
    }) = curve
    {
        // Evaluate the continuous extension at endpoints as well: spring curves
        // can jump to the exact authored final value at t=1.
        let continuous =
            |t: f64| match spring_coefficients(mass, stiffness, damping, initial_velocity) {
                SpringCoefficients::Underdamped {
                    decay,
                    frequency,
                    sine,
                } => {
                    1.0 + (-decay * t).exp()
                        * (-(frequency * t).cos() + sine * (frequency * t).sin())
                }
                SpringCoefficients::Critical { decay, linear } => {
                    1.0 + (-decay * t).exp() * (-1.0 + linear * t)
                }
                SpringCoefficients::Overdamped {
                    slow_root,
                    fast_root,
                    slow,
                    fast,
                } => 1.0 + slow * (slow_root * t).exp() + fast * (fast_root * t).exp(),
            };
        values.extend([continuous(low), continuous(high)]);
        let mut stationary = Vec::new();
        match spring_coefficients(mass, stiffness, damping, initial_velocity) {
            SpringCoefficients::Underdamped {
                decay,
                frequency,
                sine,
            } => {
                let phase = (-(decay + sine * frequency)).atan2(frequency - decay * sine);
                let first = ((frequency * low - phase) / std::f64::consts::PI).ceil() as i64;
                let last = ((frequency * high - phase) / std::f64::consts::PI).floor() as i64;
                for k in first..=last {
                    stationary.push((phase + k as f64 * std::f64::consts::PI) / frequency);
                }
            }
            SpringCoefficients::Critical { decay, linear } => {
                if decay * linear != 0.0 {
                    stationary.push((linear + decay) / (decay * linear));
                }
            }
            SpringCoefficients::Overdamped {
                slow_root,
                fast_root,
                slow,
                fast,
            } => {
                let ratio = -(fast * fast_root) / (slow * slow_root);
                if ratio > 0.0 {
                    stationary.push(ratio.ln() / (slow_root - fast_root));
                }
            }
        }
        values.extend(
            stationary
                .into_iter()
                .filter(|t| *t >= low && *t <= high)
                .map(continuous),
        );
    }
    (
        values.iter().copied().fold(f64::INFINITY, f64::min),
        values.into_iter().fold(f64::NEG_INFINITY, f64::max),
    )
}

/// Looping intervals use the entire authored cycle, a conservative enclosure.
pub(crate) fn scalar_bounds(channel: &AnimationChannel, low: u64, high: u64) -> Option<(f64, f64)> {
    scalar_bounds_at(channel, low as f64, high as f64)
}

pub(crate) fn scalar_bounds_at(
    channel: &AnimationChannel,
    mut low: f64,
    mut high: f64,
) -> Option<(f64, f64)> {
    if channel.clock.is_some() {
        low = map_source_time_at(channel.clock, low)?;
        high = map_source_time_at(channel.clock, high)?;
        let mut source = channel.clone();
        source.clock = None;
        return scalar_bounds_at(&source, low, high);
    }
    let scalar = |v: &AnimationChannelValue| {
        if let AnimationChannelValue::Scalar { value } = v {
            Some(*value)
        } else {
            None
        }
    };
    if low == high {
        let value = sample_scalar_channel_at(channel, low)?;
        return Some((value, value));
    }
    if channel.r#loop.is_some() {
        low = 0.0;
        high = channel.keyframes.last()?.time_ms as f64;
    }
    let mut bounds = (
        sample_scalar_channel_at(channel, low)?,
        sample_scalar_channel_at(channel, low)?,
    );
    let mut add = |value: f64| {
        bounds.0 = bounds.0.min(value);
        bounds.1 = bounds.1.max(value);
    };
    add(sample_scalar_channel_at(channel, high)?);
    for pair in channel.keyframes.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if high < a.time_ms as f64 || low > b.time_ms as f64 {
            continue;
        }
        let span = (b.time_ms - a.time_ms) as f64;
        let (p, q) = curve_bounds(
            a.curve,
            ((low - a.time_ms as f64).max(0.0) / span).clamp(0.0, 1.0),
            ((high - a.time_ms as f64).max(0.0) / span).clamp(0.0, 1.0),
        );
        let start = scalar(&a.value)?;
        let end = scalar(&b.value)?;
        let clamp = |value: f64| match channel.property {
            crate::AnimationChannelProperty::CropWidth
            | crate::AnimationChannelProperty::CropHeight => value.clamp(0.000001, 1.0),
            crate::AnimationChannelProperty::CropX
            | crate::AnimationChannelProperty::CropY
            | crate::AnimationChannelProperty::PathTrim
            | crate::AnimationChannelProperty::VignetteAmount => value.clamp(0.0, 1.0),
            crate::AnimationChannelProperty::BlurRadius
            | crate::AnimationChannelProperty::GlowRadius => value.clamp(0.0, 128.0),
            _ => value,
        };
        for t in [p, q] {
            add(clamp(start + (end - start) * t));
        }
        if (low..=high).contains(&(b.time_ms as f64)) {
            add(end);
        }
    }
    Some(bounds)
}

pub(crate) fn sample_scalar_channel(channel: &AnimationChannel, time_ms: u64) -> Option<f64> {
    match sample_channel(channel, time_ms)? {
        AnimationChannelValue::Scalar { value } => Some(value),
        _ => None,
    }
}

pub(crate) fn sample_scalar_channel_at(channel: &AnimationChannel, time_ms: f64) -> Option<f64> {
    let first = channel.keyframes.first()?;
    let time_ms = map_loop_time_at(channel, time_ms)?;
    let scalar = |value: &AnimationChannelValue| match value {
        AnimationChannelValue::Scalar { value } => Some(*value),
        _ => None,
    };
    if time_ms <= first.time_ms as f64 {
        return scalar(&first.value);
    }
    for pair in channel.keyframes.windows(2) {
        let start = &pair[0];
        let end = &pair[1];
        if time_ms == end.time_ms as f64 {
            return scalar(&end.value);
        }
        if time_ms < end.time_ms as f64 {
            if matches!(
                start.curve,
                AnimationCurve::Simple(SimpleAnimationCurve::Hold)
            ) {
                return scalar(&start.value);
            }
            let start_value = scalar(&start.value)?;
            let end_value = scalar(&end.value)?;
            let progress = (time_ms - start.time_ms as f64) / (end.time_ms - start.time_ms) as f64;
            let eased = match start.curve {
                AnimationCurve::Simple(SimpleAnimationCurve::Hold) => 0.0,
                AnimationCurve::Simple(SimpleAnimationCurve::Linear) => progress,
                AnimationCurve::Parameterized(curve) => {
                    parameterized_curve_progress(curve, progress)
                }
            };
            let value = start_value + (end_value - start_value) * eased;
            let value = if matches!(start.curve, AnimationCurve::Parameterized(_))
                || matches!(
                    channel.property,
                    crate::AnimationChannelProperty::CropWidth
                        | crate::AnimationChannelProperty::CropHeight
                ) {
                match channel.property {
                    crate::AnimationChannelProperty::PositionX
                    | crate::AnimationChannelProperty::PositionY => {
                        value.clamp(-1_000_000.0, 1_000_000.0)
                    }
                    crate::AnimationChannelProperty::ScaleX
                    | crate::AnimationChannelProperty::ScaleY => value.clamp(0.000_001, 100.0),
                    crate::AnimationChannelProperty::Opacity => value.clamp(0.0, 1.0),
                    crate::AnimationChannelProperty::GainDb => value.clamp(-96.0, 12.0),
                    crate::AnimationChannelProperty::RotationDeg => value.clamp(-36000.0, 36000.0),
                    crate::AnimationChannelProperty::CropWidth
                    | crate::AnimationChannelProperty::CropHeight => value.clamp(0.000_001, 1.0),
                    crate::AnimationChannelProperty::CropX
                    | crate::AnimationChannelProperty::CropY
                    | crate::AnimationChannelProperty::PathTrim
                    | crate::AnimationChannelProperty::VignetteAmount => value.clamp(0.0, 1.0),
                    crate::AnimationChannelProperty::BlurRadius
                    | crate::AnimationChannelProperty::GlowRadius => value.clamp(0.0, 128.0),
                    _ => value,
                }
            } else {
                value
            };
            return value.is_finite().then_some(value);
        }
    }
    scalar(&channel.keyframes.last()?.value)
}

fn map_source_time(clock: Option<AnimationClock>, time_ms: u64) -> Option<u64> {
    let Some(clock) = clock else {
        return Some(time_ms);
    };
    if clock.offset_ms < 0 {
        Some(time_ms.saturating_sub(clock.offset_ms.unsigned_abs()))
    } else {
        time_ms.checked_add(clock.offset_ms as u64)
    }
}

fn map_source_time_at(clock: Option<AnimationClock>, time_ms: f64) -> Option<f64> {
    if !time_ms.is_finite() || time_ms < 0.0 {
        return None;
    }
    let result = time_ms + clock.map_or(0.0, |clock| clock.offset_ms as f64);
    result.is_finite().then_some(result.max(0.0))
}

pub(crate) fn map_loop_time(channel: &AnimationChannel, time_ms: u64) -> Option<u64> {
    let time_ms = map_source_time(channel.clock, time_ms)?;
    map_loop_source_time(channel, time_ms)
}

fn map_loop_source_time(channel: &AnimationChannel, time_ms: u64) -> Option<u64> {
    let Some(loop_spec) = channel.r#loop else {
        return Some(time_ms);
    };
    let first = channel.keyframes.first()?.time_ms;
    let last = channel.keyframes.last()?.time_ms;
    let span = last.checked_sub(first)?;
    if span == 0 || time_ms < first {
        return Some(time_ms);
    }
    let period = u128::from(span)
        * if loop_spec.mode == AnimationLoopMode::PingPong {
            2
        } else {
            1
        };
    let elapsed = u128::from(time_ms - first);
    if let AnimationLoopIterations::Finite(count) = loop_spec.iterations
        && elapsed >= period * u128::from(count)
    {
        return Some(if loop_spec.mode == AnimationLoopMode::PingPong {
            first
        } else {
            last
        });
    }
    let phase = elapsed % period;
    let offset = if loop_spec.mode == AnimationLoopMode::PingPong && phase > u128::from(span) {
        period - phase
    } else {
        phase
    };
    first.checked_add(u64::try_from(offset).ok()?)
}

pub(crate) fn map_loop_time_at(channel: &AnimationChannel, time_ms: f64) -> Option<f64> {
    let time_ms = map_source_time_at(channel.clock, time_ms)?;
    if !time_ms.is_finite() || time_ms < 0.0 {
        return None;
    }
    if time_ms.fract() == 0.0 && time_ms < u64::MAX as f64 {
        return map_loop_source_time(channel, time_ms as u64).map(|t| t as f64);
    }
    let Some(spec) = channel.r#loop else {
        return Some(time_ms);
    };
    let first = channel.keyframes.first()?.time_ms as f64;
    let last = channel.keyframes.last()?.time_ms as f64;
    let span = last - first;
    if span <= 0.0 || time_ms < first {
        return Some(time_ms);
    }
    let period = span
        * if spec.mode == AnimationLoopMode::PingPong {
            2.0
        } else {
            1.0
        };
    let elapsed = time_ms - first;
    if let AnimationLoopIterations::Finite(count) = spec.iterations
        && elapsed >= period * f64::from(count)
    {
        return Some(if spec.mode == AnimationLoopMode::PingPong {
            first
        } else {
            last
        });
    }
    let phase = elapsed.rem_euclid(period);
    Some(
        first
            + if spec.mode == AnimationLoopMode::PingPong && phase > span {
                period - phase
            } else {
                phase
            },
    )
}

/// The same bounded sampler supplies compound scene facts for every output intent.
pub(crate) fn sample_channel(
    channel: &AnimationChannel,
    time_ms: u64,
) -> Option<AnimationChannelValue> {
    let time = map_loop_time(channel, time_ms)?;
    let first = channel.keyframes.first()?;
    if time <= first.time_ms {
        return Some(first.value.clone());
    }
    for pair in channel.keyframes.windows(2) {
        if time == pair[1].time_ms {
            return Some(pair[1].value.clone());
        }
        if time < pair[1].time_ms {
            if matches!(
                pair[0].curve,
                AnimationCurve::Simple(SimpleAnimationCurve::Hold)
            ) {
                return Some(pair[0].value.clone());
            }
            // Rebase only the selected segment. All scalar/compound interpolation
            // and clamps still come from the canonical fractional sampler.
            let origin = pair[0].time_ms;
            let mut start = pair[0].clone();
            let mut end = pair[1].clone();
            start.time_ms = 0;
            end.time_ms -= origin;
            let relative = AnimationChannel {
                clock: None,
                property: channel.property,
                target: channel.target.clone(),
                keyframes: vec![start, end],
                r#loop: None,
            };
            return sample_channel_at(&relative, (time - origin) as f64);
        }
    }
    Some(channel.keyframes.last()?.value.clone())
}

pub(crate) fn sample_channel_at(
    channel: &AnimationChannel,
    time_ms: f64,
) -> Option<AnimationChannelValue> {
    if matches!(
        channel.keyframes.first()?.value,
        AnimationChannelValue::Scalar { .. }
    ) {
        return sample_scalar_channel_at(channel, time_ms)
            .map(|value| AnimationChannelValue::Scalar { value });
    }
    let time = map_loop_time_at(channel, time_ms)?;
    let first = channel.keyframes.first()?;
    if time <= first.time_ms as f64 {
        return Some(first.value.clone());
    }
    for pair in channel.keyframes.windows(2) {
        let (start, end) = (&pair[0], &pair[1]);
        if time == end.time_ms as f64 {
            return Some(end.value.clone());
        }
        if time < end.time_ms as f64 {
            let progress = (time - start.time_ms as f64) / (end.time_ms - start.time_ms) as f64;
            let t = match start.curve {
                AnimationCurve::Simple(SimpleAnimationCurve::Hold) => {
                    return Some(start.value.clone());
                }
                AnimationCurve::Simple(SimpleAnimationCurve::Linear) => progress,
                AnimationCurve::Parameterized(curve) => {
                    parameterized_curve_progress(curve, progress)
                }
            };
            let mix = |a: f64, b: f64, low: f64, high: f64| (a + (b - a) * t).clamp(low, high);
            return match (&start.value, &end.value) {
                (
                    AnimationChannelValue::Rgba {
                        r: ar,
                        g: ag,
                        b: ab,
                        a: aa,
                    },
                    AnimationChannelValue::Rgba {
                        r: br,
                        g: bg,
                        b: bb,
                        a: ba,
                    },
                ) => {
                    let [r, g, b, a] =
                        interpolate_rgba([*ar, *ag, *ab, *aa], [*br, *bg, *bb, *ba], t);
                    Some(AnimationChannelValue::Rgba { r, g, b, a })
                }
                (
                    AnimationChannelValue::PathPoints { points: a },
                    AnimationChannelValue::PathPoints { points: b },
                ) if a.len() == b.len() => Some(AnimationChannelValue::PathPoints {
                    points: a
                        .iter()
                        .zip(b)
                        .map(|(a, b)| crate::AnimationPoint {
                            x: mix(a.x, b.x, -1_000_000.0, 1_000_000.0),
                            y: mix(a.y, b.y, -1_000_000.0, 1_000_000.0),
                        })
                        .collect(),
                }),
                (
                    AnimationChannelValue::GradientStops { stops: a },
                    AnimationChannelValue::GradientStops { stops: b },
                ) if a.len() == b.len() => Some(AnimationChannelValue::GradientStops {
                    stops: a
                        .iter()
                        .zip(b)
                        .map(|(a, b)| crate::AnimationGradientStop {
                            offset: mix(a.offset, b.offset, 0.0, 1.0),
                            color: interpolate_rgba(a.color, b.color, t),
                        })
                        .collect(),
                }),
                _ => None,
            };
        }
    }
    Some(channel.keyframes.last()?.value.clone())
}

fn interpolate_rgba(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    let linear = |v: f64| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let srgb = |v: f64| {
        if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    let alpha = (a[3] + (b[3] - a[3]) * t).clamp(0.0, 1.0);
    if alpha == 0.0 {
        return [0.0; 4];
    }
    let mut result = [0.0; 4];
    for i in 0..3 {
        let left = linear(a[i]) * a[3];
        let right = linear(b[i]) * b[3];
        result[i] = srgb(((left + (right - left) * t) / alpha).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    }
    result[3] = alpha;
    result
}

#[cfg(test)]
mod curve_tests {
    use super::*;
    use crate::AnimationChannelProperty;
    use serde_json::json;

    #[test]
    fn legacy_integer_keyframes_above_f64_precision_keep_exact_selection() {
        let mut value = channel(json!("linear"), 0.0, 100.0);
        let origin = 9_007_199_254_740_993;
        value.keyframes[0].time_ms = origin;
        value.keyframes[1].time_ms = origin + 4;
        assert_eq!(sample_scalar_channel(&value, origin), Some(0.0));
        assert_eq!(sample_scalar_channel(&value, origin + 1), Some(25.0));
        assert_eq!(sample_scalar_channel(&value, origin + 3), Some(75.0));
        assert_eq!(sample_scalar_channel(&value, origin + 4), Some(100.0));
        value.keyframes[0].curve = AnimationCurve::Simple(SimpleAnimationCurve::Hold);
        assert_eq!(sample_scalar_channel(&value, origin + 3), Some(0.0));
        assert_eq!(sample_scalar_channel(&value, origin + 4), Some(100.0));
    }

    fn channel(curve: serde_json::Value, first: f64, last: f64) -> AnimationChannel {
        serde_json::from_value(json!({
            "property":"transform.position_x",
            "keyframes":[
                {"timeMs":0,"value":{"type":"scalar","value":first},"curve":curve},
                {"timeMs":1000,"value":{"type":"scalar","value":last},"curve":"hold"}
            ]
        }))
        .unwrap()
    }

    #[test]
    fn bezier_fixed_samples_and_exact_boundaries() {
        let value = channel(
            json!({"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}),
            0.0,
            100.0,
        );
        assert_eq!(value.property, AnimationChannelProperty::PositionX);
        assert_eq!(sample_scalar_channel(&value, 0), Some(0.0));
        assert_eq!(sample_scalar_channel(&value, 1000), Some(100.0));
        assert!((sample_scalar_channel(&value, 250).unwrap() - 40.851_059_135_527_1).abs() < 1e-9);
        assert!((sample_scalar_channel(&value, 500).unwrap() - 80.240_338_758_508_51).abs() < 1e-9);
    }

    #[test]
    fn spring_regimes_have_fixed_scalar_results() {
        for (damping, expected) in [
            (10.0, 107.459_056_659_503_33),
            (20.0, 95.957_231_800_548_71),
            (30.0, 82.659_534_975_953_59),
        ] {
            let value = channel(
                json!({"type":"spring","mass":1,"stiffness":100,"damping":damping,"initialVelocity":0}),
                0.0,
                100.0,
            );
            assert_eq!(sample_scalar_channel(&value, 0), Some(0.0));
            assert_eq!(sample_scalar_channel(&value, 1000), Some(100.0));
            assert!((sample_scalar_channel(&value, 500).unwrap() - expected).abs() < 1e-9);
        }
        let value = channel(
            json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0}),
            0.0,
            100.0,
        );
        assert!(sample_scalar_channel(&value, 300).unwrap() > 100.0);
    }

    #[test]
    fn loop_phases_are_item_local_and_exact_at_seams() {
        let repeat: AnimationChannel = serde_json::from_value(json!({
            "property":"transform.position_x",
            "keyframes":[
                {"timeMs":100,"value":{"type":"scalar","value":0},"curve":"linear"},
                {"timeMs":200,"value":{"type":"scalar","value":100},"curve":"linear"},
                {"timeMs":300,"value":{"type":"scalar","value":0},"curve":"hold"}
            ],
            "loop":{"mode":"repeat","iterations":3}
        }))
        .unwrap();
        assert_eq!(sample_scalar_channel(&repeat, 99), Some(0.0));
        assert_eq!(sample_scalar_channel(&repeat, 200), Some(100.0));
        assert_eq!(sample_scalar_channel(&repeat, 299), Some(1.0));
        assert_eq!(sample_scalar_channel(&repeat, 300), Some(0.0));
        assert_eq!(sample_scalar_channel(&repeat, 301), Some(1.0));
        assert_eq!(sample_scalar_channel(&repeat, 450), Some(50.0));
        assert_eq!(sample_scalar_channel(&repeat, 699), Some(1.0));
        assert_eq!(sample_scalar_channel(&repeat, 700), Some(0.0));
        let ping_pong: AnimationChannel = serde_json::from_value(json!({
            "property":"transform.position_x",
            "keyframes":[
                {"timeMs":100,"value":{"type":"scalar","value":0},"curve":"linear"},
                {"timeMs":300,"value":{"type":"scalar","value":100},"curve":"hold"}
            ],
            "loop":{"mode":"ping_pong","iterations":1}
        }))
        .unwrap();
        assert_eq!(sample_scalar_channel(&ping_pong, 300), Some(100.0));
        assert_eq!(sample_scalar_channel(&ping_pong, 299), Some(99.5));
        assert_eq!(sample_scalar_channel(&ping_pong, 301), Some(99.5));
        assert_eq!(sample_scalar_channel(&ping_pong, 400), Some(50.0));
        assert_eq!(sample_scalar_channel(&ping_pong, 499), Some(0.5));
        assert_eq!(sample_scalar_channel(&ping_pong, 500), Some(0.0));
        assert_eq!(sample_scalar_channel(&ping_pong, 700), Some(0.0));
        let spring: AnimationChannel = serde_json::from_value(json!({
            "property":"transform.position_x",
            "keyframes":[
                {"timeMs":100,"value":{"type":"scalar","value":0},
                    "curve":{"type":"spring","mass":1,"stiffness":100,"damping":20,"initialVelocity":0}},
                {"timeMs":300,"value":{"type":"scalar","value":100},"curve":"hold"}
            ],
            "loop":{"mode":"ping_pong","iterations":"infinite"}
        })).unwrap();
        assert_eq!(
            sample_scalar_channel(&spring, 150),
            sample_scalar_channel(&spring, 450)
        );
        assert_eq!(
            sample_scalar_channel(&spring, 299),
            sample_scalar_channel(&spring, 301)
        );
        assert_eq!(sample_scalar_channel(&spring, 500), Some(0.0));
        assert_eq!(sample_scalar_channel(&spring, 900), Some(0.0));
        assert!(
            sample_scalar_channel(&spring, u64::MAX)
                .unwrap()
                .is_finite()
        );
        let bezier: AnimationChannel = serde_json::from_value(json!({
            "property":"transform.position_x",
            "keyframes":[
                {"timeMs":100,"value":{"type":"scalar","value":0},
                    "curve":{"type":"cubic_bezier","x1":0.2,"y1":0.1,"x2":0.8,"y2":0.9}},
                {"timeMs":300,"value":{"type":"scalar","value":100},"curve":"hold"}
            ],
            "loop":{"mode":"ping_pong","iterations":"infinite"}
        }))
        .unwrap();
        assert_eq!(
            sample_scalar_channel(&bezier, 175),
            sample_scalar_channel(&bezier, 425)
        );
    }
}

#[cfg(test)]
pub(crate) fn easing_progress(easing: Easing, progress: f64) -> f64 {
    let progress = progress.clamp(0.0, 1.0);
    match easing {
        Easing::Hold => 0.0,
        Easing::Linear => progress,
        Easing::EaseIn => progress * progress,
        Easing::EaseOut => 1.0 - (1.0 - progress) * (1.0 - progress),
        Easing::EaseInOut if progress < 0.5 => 2.0 * progress * progress,
        Easing::EaseInOut => 1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0,
    }
}

#[cfg(test)]
pub(crate) fn evaluate_keyframe_value(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    time_ms: u64,
) -> Option<(KeyframeValue, Easing)> {
    let values = keyframes
        .iter()
        .filter(|keyframe| keyframe.property == property)
        .collect::<Vec<_>>();
    let first = *values.first()?;
    if time_ms <= first.time_ms {
        return Some((first.value.clone(), first.easing));
    }
    for pair in values.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if time_ms < end.time_ms {
            let span = end.time_ms.saturating_sub(start.time_ms).max(1);
            let progress = (time_ms.saturating_sub(start.time_ms)) as f64 / span as f64;
            return Some((
                interpolate_value(
                    &start.value,
                    &end.value,
                    easing_progress(start.easing, progress),
                ),
                start.easing,
            ));
        }
        if time_ms == end.time_ms {
            return Some((end.value.clone(), end.easing));
        }
    }
    let last = *values.last()?;
    Some((last.value.clone(), last.easing))
}

#[cfg(test)]
pub(crate) fn split_keyframes(
    keyframes: &[Keyframe],
    split_offset_ms: u64,
    original_duration_ms: u64,
) -> (Vec<Keyframe>, Vec<Keyframe>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for property in [
        KeyframeProperty::Position,
        KeyframeProperty::Scale,
        KeyframeProperty::Opacity,
        KeyframeProperty::Volume,
    ] {
        let Some((boundary_value, boundary_easing)) =
            evaluate_keyframe_value(keyframes, property, split_offset_ms)
        else {
            continue;
        };
        left.extend(
            keyframes
                .iter()
                .filter(|keyframe| {
                    keyframe.property == property && keyframe.time_ms < split_offset_ms
                })
                .cloned(),
        );
        left.push(Keyframe {
            property,
            time_ms: split_offset_ms,
            value: boundary_value.clone(),
            easing: boundary_easing,
        });
        right.push(Keyframe {
            property,
            time_ms: 0,
            value: boundary_value,
            easing: boundary_easing,
        });
        right.extend(
            keyframes
                .iter()
                .filter(|keyframe| {
                    keyframe.property == property
                        && keyframe.time_ms > split_offset_ms
                        && keyframe.time_ms <= original_duration_ms
                })
                .cloned()
                .map(|mut keyframe| {
                    keyframe.time_ms -= split_offset_ms;
                    keyframe
                }),
        );
    }
    (left, right)
}

/// Map conservative source activity intervals into the edited item window.
pub(crate) fn positive_scalar_ranges_at(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    duration_ms: u64,
    clock: Option<AnimationClock>,
) -> Vec<(u64, u64)> {
    let Some(clock) = clock else {
        return positive_scalar_ranges(keyframes, property, duration_ms);
    };
    let source_end = (i128::from(clock.offset_ms) + i128::from(duration_ms)).max(0) as u64;
    let source_ranges = positive_scalar_ranges(keyframes, property, source_end);
    let mut ranges = Vec::new();
    // A left extension holds the first source value (or the static default).
    if clock.offset_ms < 0 {
        let first = keyframes.iter().find(|key| key.property == property);
        let positive = first
            .is_none_or(|key| matches!(key.value, KeyframeValue::Scalar { value } if value > 0.0));
        push_positive_range(
            &mut ranges,
            0,
            clock.offset_ms.unsigned_abs().min(duration_ms),
            positive,
        );
    }
    for (start, end) in source_ranges {
        let local = |source: u64| {
            (i128::from(source) - i128::from(clock.offset_ms)).clamp(0, i128::from(duration_ms))
                as u64
        };
        push_positive_range(&mut ranges, local(start), local(end), true);
    }
    ranges
}

pub(crate) fn positive_scalar_ranges(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    duration_ms: u64,
) -> Vec<(u64, u64)> {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| {
            if keyframe.property != property {
                return None;
            }
            let KeyframeValue::Scalar { value } = keyframe.value else {
                return None;
            };
            Some((keyframe.time_ms.min(duration_ms), value, keyframe.easing))
        })
        .collect::<Vec<_>>();
    if duration_ms == 0 {
        return vec![];
    }
    if values.is_empty() {
        return vec![(0, duration_ms)];
    }

    let mut ranges = Vec::new();
    let first = values[0];
    push_positive_range(&mut ranges, 0, first.0, first.1 > 0.0);
    for pair in values.windows(2) {
        let (start_time, start_value, easing) = pair[0];
        let (end_time, end_value, _) = pair[1];
        let positive = match easing {
            Easing::Hold => start_value > 0.0,
            _ => start_value > 0.0 || end_value > 0.0,
        };
        push_positive_range(&mut ranges, start_time, end_time, positive);
    }
    let last = values[values.len() - 1];
    push_positive_range(&mut ranges, last.0, duration_ms, last.1 > 0.0);
    ranges
}

#[cfg(test)]
fn interpolate_value(start: &KeyframeValue, end: &KeyframeValue, progress: f64) -> KeyframeValue {
    match (start, end) {
        (
            KeyframeValue::Position {
                x: start_x,
                y: start_y,
            },
            KeyframeValue::Position { x: end_x, y: end_y },
        ) => KeyframeValue::Position {
            x: start_x + (end_x - start_x) * progress,
            y: start_y + (end_y - start_y) * progress,
        },
        (KeyframeValue::Scalar { value: start }, KeyframeValue::Scalar { value: end }) => {
            KeyframeValue::Scalar {
                value: start + (end - start) * progress,
            }
        }
        _ => start.clone(),
    }
}

fn push_positive_range(ranges: &mut Vec<(u64, u64)>, start_ms: u64, end_ms: u64, positive: bool) {
    if !positive || start_ms >= end_ms {
        return;
    }
    if let Some((_, previous_end)) = ranges.last_mut()
        && start_ms <= *previous_end
    {
        *previous_end = (*previous_end).max(end_ms);
    } else {
        ranges.push((start_ms, end_ms));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_fractional_clocks_preserve_loop_phase_and_compound_values() {
        let channel: AnimationChannel = serde_json::from_value(serde_json::json!({"property":"transform.rotation_deg","loop":{"mode":"ping_pong","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},{"timeMs":10,"value":{"type":"scalar","value":100},"curve":"hold"}]})).unwrap();
        assert_eq!(sample_scalar_channel_at(&channel, 9.5), Some(95.0));
        assert_eq!(sample_scalar_channel_at(&channel, 10.5), Some(95.0));
        assert_eq!(sample_scalar_channel_at(&channel, 20.5), Some(5.0));
        let mut path: AnimationChannel = serde_json::from_value(serde_json::json!({"property":"graphic.path_points","target":{"kind":"graphic_geometry","scope":"root","id":"p"},"keyframes":[{"timeMs":0,"value":{"type":"path_points","points":[{"x":0,"y":0}]},"curve":"linear"},{"timeMs":10,"value":{"type":"path_points","points":[{"x":10,"y":20}]},"curve":"hold"}]})).unwrap();
        assert_eq!(
            serde_json::to_value(sample_channel_at(&path, 0.5).unwrap()).unwrap(),
            serde_json::json!({"type":"path_points","points":[{"x":0.5,"y":1.0}]})
        );
        path.keyframes[0].curve = AnimationCurve::Simple(SimpleAnimationCurve::Hold);
        assert_eq!(
            sample_channel_at(&path, 0.5),
            Some(path.keyframes[0].value.clone())
        );
    }

    #[test]
    fn canonical_extended_samples_match_independent_values() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/extended-visual-animation-v1.json"
        ))
        .unwrap();
        fn compare(actual: &serde_json::Value, expected: &serde_json::Value) {
            match (actual, expected) {
                (serde_json::Value::Number(a), serde_json::Value::Number(b)) => {
                    assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-9)
                }
                (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                    assert_eq!(a.len(), b.len());
                    for (a, b) in a.iter().zip(b) {
                        compare(a, b);
                    }
                }
                (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                    assert_eq!(a.len(), b.len());
                    for (key, value) in b {
                        compare(&a[key], value);
                    }
                }
                _ => assert_eq!(actual, expected),
            }
        }
        for case in fixture["sampleCases"].as_array().unwrap() {
            let channel: AnimationChannel =
                serde_json::from_value(case["channel"].clone()).unwrap();
            let sampled = sample_channel(&channel, case["timeMs"].as_u64().unwrap()).unwrap();
            compare(&serde_json::to_value(sampled).unwrap(), &case["expected"]);
        }
    }

    #[test]
    fn spring_envelopes_contain_all_stationary_points_and_endpoint_jumps() {
        for damping in [0.01, 1.0, 20.0, 100.0] {
            for velocity in [-10.0, 0.0, 10.0] {
                let curve = ParameterizedAnimationCurve::Spring {
                    mass: 1.0,
                    stiffness: 100.0,
                    damping,
                    initial_velocity: velocity,
                };
                for (low, high) in [(0.0, 1.0), (0.1, 0.2), (0.9, 1.0)] {
                    let (min, max) = curve_bounds(AnimationCurve::Parameterized(curve), low, high);
                    for i in 0..=1000 {
                        let value = parameterized_curve_progress(
                            curve,
                            low + (high - low) * f64::from(i) / 1000.0,
                        );
                        assert!(
                            value >= min - 1e-12 && value <= max + 1e-12,
                            "{curve:?}: {value} outside [{min},{max}]"
                        );
                    }
                }
            }
        }
    }

    fn scalar(time_ms: u64, value: f64, easing: Easing) -> Keyframe {
        Keyframe {
            property: KeyframeProperty::Opacity,
            time_ms,
            value: KeyframeValue::Scalar { value },
            easing,
        }
    }

    fn value_for(property: KeyframeProperty, value: f64) -> KeyframeValue {
        if property == KeyframeProperty::Position {
            KeyframeValue::Position {
                x: value,
                y: -value,
            }
        } else {
            KeyframeValue::Scalar { value }
        }
    }

    #[test]
    fn evaluates_every_easing_at_the_split_boundary() {
        let expected = [
            (Easing::Hold, 0.0),
            (Easing::Linear, 0.25),
            (Easing::EaseIn, 0.0625),
            (Easing::EaseOut, 0.4375),
            (Easing::EaseInOut, 0.125),
        ];
        for (easing, expected) in expected {
            let keyframes = vec![scalar(0, 0.0, easing), scalar(1_000, 1.0, Easing::Linear)];
            let (KeyframeValue::Scalar { value }, _) =
                evaluate_keyframe_value(&keyframes, KeyframeProperty::Opacity, 250).unwrap()
            else {
                panic!("expected scalar value")
            };
            assert!((value - expected).abs() < 1e-9, "{easing:?}: {value}");
        }
    }

    #[test]
    fn positive_ranges_preserve_hold_silence_and_interpolated_activity() {
        let keyframes = vec![
            scalar(0, 0.0, Easing::Hold),
            scalar(500, 1.0, Easing::Linear),
            scalar(1_000, 0.0, Easing::Linear),
        ];
        assert_eq!(
            positive_scalar_ranges(&keyframes, KeyframeProperty::Opacity, 1_500),
            vec![(500, 1_000)]
        );
    }

    #[test]
    fn split_partitions_and_rebases_every_property_and_easing() {
        for property in [
            KeyframeProperty::Position,
            KeyframeProperty::Scale,
            KeyframeProperty::Opacity,
            KeyframeProperty::Volume,
        ] {
            for easing in [
                Easing::Hold,
                Easing::Linear,
                Easing::EaseIn,
                Easing::EaseOut,
                Easing::EaseInOut,
            ] {
                let keyframes = vec![
                    Keyframe {
                        property,
                        time_ms: 100,
                        value: value_for(property, 0.0),
                        easing,
                    },
                    Keyframe {
                        property,
                        time_ms: 900,
                        value: value_for(property, 1.0),
                        easing: Easing::Linear,
                    },
                ];
                let expected = evaluate_keyframe_value(&keyframes, property, 500)
                    .unwrap()
                    .0;
                let (left, right) = split_keyframes(&keyframes, 500, 1_000);
                assert!(left.iter().all(|keyframe| keyframe.time_ms <= 500));
                assert!(right.iter().all(|keyframe| keyframe.time_ms <= 500));
                assert_eq!(left.last().unwrap().value, expected);
                assert_eq!(right.first().unwrap().time_ms, 0);
                assert_eq!(right.first().unwrap().value, expected);
                assert_eq!(right.last().unwrap().time_ms, 400);
            }
        }
    }

    #[test]
    fn split_on_an_existing_keyframe_does_not_duplicate_timestamps() {
        let keyframes = vec![
            scalar(0, 0.0, Easing::Linear),
            scalar(500, 0.5, Easing::EaseOut),
            scalar(1_000, 1.0, Easing::Linear),
        ];
        let (left, right) = split_keyframes(&keyframes, 500, 1_000);
        assert_eq!(
            left.iter()
                .map(|keyframe| keyframe.time_ms)
                .collect::<Vec<_>>(),
            vec![0, 500]
        );
        assert_eq!(
            right
                .iter()
                .map(|keyframe| keyframe.time_ms)
                .collect::<Vec<_>>(),
            vec![0, 500]
        );
        assert_eq!(right[0].easing, Easing::EaseOut);
    }

    #[test]
    fn split_before_and_exactly_on_keyframes_is_continuous_for_every_property_and_easing() {
        for property in [
            KeyframeProperty::Position,
            KeyframeProperty::Scale,
            KeyframeProperty::Opacity,
            KeyframeProperty::Volume,
        ] {
            for easing in [
                Easing::Hold,
                Easing::Linear,
                Easing::EaseIn,
                Easing::EaseOut,
                Easing::EaseInOut,
            ] {
                let keyframes = vec![
                    Keyframe {
                        property,
                        time_ms: 100,
                        value: value_for(property, 0.25),
                        easing,
                    },
                    Keyframe {
                        property,
                        time_ms: 900,
                        value: value_for(property, 0.75),
                        easing: Easing::EaseOut,
                    },
                ];
                for split_at in [50, 100, 900] {
                    let expected = evaluate_keyframe_value(&keyframes, property, split_at)
                        .unwrap()
                        .0;
                    let (left, right) = split_keyframes(&keyframes, split_at, 1_000);
                    assert!(left.iter().all(|keyframe| keyframe.time_ms <= split_at));
                    assert!(
                        right
                            .iter()
                            .all(|keyframe| keyframe.time_ms <= 1_000 - split_at)
                    );
                    assert_eq!(left.last().unwrap().value, expected);
                    assert_eq!(right.first().unwrap().time_ms, 0);
                    assert_eq!(right.first().unwrap().value, expected);
                    assert_eq!(
                        left.iter()
                            .filter(|keyframe| keyframe.time_ms == split_at)
                            .count(),
                        1
                    );
                    assert_eq!(
                        right
                            .iter()
                            .filter(|keyframe| keyframe.time_ms == 0)
                            .count(),
                        1
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod retained_clock_tests {
    use super::*;
    use serde_json::json;

    fn source(curve: serde_json::Value, looping: Option<serde_json::Value>) -> AnimationChannel {
        let mut value = json!({"property":"transform.position_x","keyframes":[
            {"timeMs":100,"value":{"type":"scalar","value":0},"curve":curve},
            {"timeMs":250,"value":{"type":"scalar","value":100},"curve":"linear"},
            {"timeMs":400,"value":{"type":"scalar","value":0},"curve":"hold"}
        ]});
        if let Some(looping) = looping {
            value["loop"] = looping;
        }
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn retained_clock_preserves_arbitrary_curves_finite_phases_and_fractional_interiors() {
        for curve in [
            json!("linear"),
            json!("hold"),
            json!({"type":"cubic_bezier","x1":0.2,"y1":0.8,"x2":0.7,"y2":0.95}),
            json!({"type":"spring","mass":1,"stiffness":120,"damping":7,"initialVelocity":-2}),
        ] {
            for looping in [
                None,
                Some(json!({"mode":"repeat","iterations":2})),
                Some(json!({"mode":"ping_pong","iterations":2})),
                Some(json!({"mode":"ping_pong","iterations":"infinite"})),
            ] {
                let original = source(curve.clone(), looping);
                for offset in [175, 350, 700] {
                    let mut retained = original.clone();
                    retained.clock = Some(AnimationClock {
                        offset_ms: offset,
                        source_duration_ms: 1800,
                    });
                    for time in [
                        0.0, 0.5, 25.0, 49.5, 50.0, 50.5, 100.0, 224.5, 225.0, 225.5, 500.0, 1100.0,
                    ] {
                        assert_eq!(
                            sample_channel_at(&retained, time),
                            sample_channel_at(&original, time + offset as f64),
                            "offset={offset} time={time}"
                        );
                    }
                    for time in [0, 1, 25, 50, 100, 225, 500, 1100] {
                        assert_eq!(
                            sample_channel(&retained, time),
                            sample_channel(&original, time + offset as u64)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn negative_retained_clock_holds_source_first_before_exact_start() {
        let original = source(
            json!("linear"),
            Some(json!({"mode":"ping_pong","iterations":1})),
        );
        let mut retained = original.clone();
        retained.clock = Some(AnimationClock {
            offset_ms: -200,
            source_duration_ms: 1000,
        });
        for (time, expected) in [
            (0.0, 0.0),
            (299.5, 0.0),
            (300.0, 0.0),
            (375.0, 50.0),
            (500.0, 200.0 / 3.0),
            (900.0, 0.0),
        ] {
            let AnimationChannelValue::Scalar { value } =
                sample_channel_at(&retained, time).unwrap()
            else {
                panic!()
            };
            assert!((value - expected).abs() < 1e-9, "{time}: {value}");
        }
    }

    #[test]
    fn retained_clocks_keep_compound_segments_and_bounds() {
        for (property, a, b) in [
            (
                "graphic.fill_color",
                json!({"type":"rgba","r":1,"g":0,"b":0,"a":1}),
                json!({"type":"rgba","r":0,"g":1,"b":0,"a":0.5}),
            ),
            (
                "graphic.path_points",
                json!({"type":"path_points","points":[{"x":0,"y":10}]}),
                json!({"type":"path_points","points":[{"x":100,"y":20}]}),
            ),
            (
                "graphic.gradient_stops",
                json!({"type":"gradient_stops","stops":[{"offset":0,"color":[1,0,0,1]}]}),
                json!({"type":"gradient_stops","stops":[{"offset":1,"color":[0,1,0,1]}]}),
            ),
        ] {
            let original: AnimationChannel = serde_json::from_value(json!({"property":property,"keyframes":[{"timeMs":0,"value":a,"curve":"linear"},{"timeMs":1000,"value":b,"curve":"hold"}]})).unwrap();
            let mut retained = original.clone();
            retained.clock = Some(AnimationClock {
                offset_ms: 275,
                source_duration_ms: 1200,
            });
            for time in [0.0, 0.5, 100.5, 725.0] {
                assert_eq!(
                    sample_channel_at(&retained, time),
                    sample_channel_at(&original, time + 275.0)
                );
            }
        }
        let original = source(json!("linear"), None);
        let mut retained = original.clone();
        retained.clock = Some(AnimationClock {
            offset_ms: 175,
            source_duration_ms: 1000,
        });
        let (low, high) = scalar_bounds_at(&retained, 0.0, 25.0).unwrap();
        assert!((low - 50.0).abs() < 1e-9 && (high - 200.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn retained_legacy_voice_activity_maps_source_intervals_without_restart() {
        let keys: Vec<Keyframe> = serde_json::from_value(json!([
            {"property":"volume","timeMs":0,"value":{"type":"scalar","value":0},"easing":"hold"},
            {"property":"volume","timeMs":500,"value":{"type":"scalar","value":1},"easing":"hold"},
            {"property":"volume","timeMs":1000,"value":{"type":"scalar","value":0},"easing":"hold"}
        ]))
        .unwrap();
        assert_eq!(
            positive_scalar_ranges_at(
                &keys,
                KeyframeProperty::Volume,
                800,
                Some(AnimationClock {
                    offset_ms: 300,
                    source_duration_ms: 1500
                })
            ),
            vec![(200, 700)]
        );
        assert_eq!(
            positive_scalar_ranges_at(
                &keys,
                KeyframeProperty::Volume,
                1300,
                Some(AnimationClock {
                    offset_ms: -200,
                    source_duration_ms: 1500
                })
            ),
            vec![(700, 1200)]
        );
    }
}
