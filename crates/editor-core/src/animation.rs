use crate::{
    AnimationChannel, AnimationChannelValue, AnimationCurve, AnimationLoopIterations,
    AnimationLoopMode, Easing, Keyframe, KeyframeProperty, KeyframeValue,
    ParameterizedAnimationCurve, SimpleAnimationCurve,
};

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

pub(crate) fn sample_scalar_channel(channel: &AnimationChannel, time_ms: u64) -> Option<f64> {
    let first = channel.keyframes.first()?;
    let time_ms = map_loop_time(channel, time_ms)?;
    let scalar = |value: &AnimationChannelValue| match value {
        AnimationChannelValue::Scalar { value } => Some(*value),
        _ => None,
    };
    if time_ms <= first.time_ms {
        return scalar(&first.value);
    }
    for pair in channel.keyframes.windows(2) {
        let start = &pair[0];
        let end = &pair[1];
        if time_ms == end.time_ms {
            return scalar(&end.value);
        }
        if time_ms < end.time_ms {
            let start_value = scalar(&start.value)?;
            let end_value = scalar(&end.value)?;
            let progress = (time_ms - start.time_ms) as f64 / (end.time_ms - start.time_ms) as f64;
            let eased = match start.curve {
                AnimationCurve::Simple(SimpleAnimationCurve::Hold) => 0.0,
                AnimationCurve::Simple(SimpleAnimationCurve::Linear) => progress,
                AnimationCurve::Parameterized(curve) => {
                    parameterized_curve_progress(curve, progress)
                }
            };
            let value = start_value + (end_value - start_value) * eased;
            let value = if matches!(start.curve, AnimationCurve::Parameterized(_)) {
                match channel.property {
                    crate::AnimationChannelProperty::PositionX
                    | crate::AnimationChannelProperty::PositionY => {
                        value.clamp(-1_000_000.0, 1_000_000.0)
                    }
                    crate::AnimationChannelProperty::ScaleX
                    | crate::AnimationChannelProperty::ScaleY => value.clamp(0.000_001, 100.0),
                    crate::AnimationChannelProperty::Opacity => value.clamp(0.0, 1.0),
                    crate::AnimationChannelProperty::GainDb => value.clamp(-96.0, 12.0),
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

pub(crate) fn map_loop_time(channel: &AnimationChannel, time_ms: u64) -> Option<u64> {
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

#[cfg(test)]
mod curve_tests {
    use super::*;
    use crate::AnimationChannelProperty;
    use serde_json::json;

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
