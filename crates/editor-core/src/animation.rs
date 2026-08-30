use crate::{Easing, Keyframe, KeyframeProperty, KeyframeValue};

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
