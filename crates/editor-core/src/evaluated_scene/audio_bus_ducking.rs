//! Bounded narration clip activity and renderer-neutral minimum envelopes.
use super::{EvaluatedScene, invalid};
use crate::{AudioBusDucking, CoreError, Project};

pub(crate) const MAX_MERGED_INTERVALS: usize = 64;
pub(crate) const MAX_ENVELOPE_SEGMENTS: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EnvelopeSegment {
    pub(crate) start_ms: f64,
    pub(crate) end_ms: f64,
    pub(crate) start_gain: f64,
    pub(crate) end_gain: f64,
}

fn reaches(project: &Project, mut index: usize, destination: usize) -> bool {
    for _ in 0..4 {
        if index == destination {
            return true;
        }
        let Some(output) = project.audio_buses[index].output_bus_id.as_deref() else {
            return false;
        };
        // The caller has certified the complete fixed routing model.
        index = crate::AUDIO_BUS_IDS
            .iter()
            .position(|id| *id == output)
            .unwrap();
    }
    false
}

pub(super) fn prepare(
    project: &Project,
    scene: &EvaluatedScene,
    audible: &[bool; 4],
) -> Result<[Vec<EnvelopeSegment>; 4], CoreError> {
    let mut result = std::array::from_fn(|_| Vec::new());
    for (target, bus) in project.audio_buses.iter().enumerate() {
        let Some(settings) = bus.ducking.as_ref().filter(|d| !d.is_identity()) else {
            continue;
        };
        if !audible[target] {
            continue;
        }
        let source = crate::AUDIO_BUS_IDS
            .iter()
            .position(|id| *id == settings.source_bus_id)
            .unwrap();
        let mut intervals = Vec::new();
        for layer in &scene.audio_layers {
            if layer.volume <= 0.0 || !layer.bus_index.is_some_and(|i| reaches(project, i, source))
            {
                continue;
            }
            let (start, end) = layer.instance.map_or(
                (layer.span.start_ms as f64, layer.span.end_ms as f64),
                |clock| (clock.start_ms, clock.end_ms),
            );
            let start = start.max(0.0);
            let end = end.min(scene.duration_ms as f64);
            if start < end {
                if intervals.len() >= super::MAX_EVALUATED_AUDIO_LAYERS {
                    return Err(invalid(
                        "audio bus ducking source activity exceeds layer bounds",
                    ));
                }
                intervals.push((start, end));
            }
        }
        result[target] = envelope(intervals, settings, scene.duration_ms as f64)?;
    }
    Ok(result)
}

// Endpoint containment identifies one linear branch without constructing a
// midpoint that can round to the half-open end for adjacent finite clocks.
fn branch(time: f64, segment: (f64, f64), span: (f64, f64), settings: &AudioBusDucking) -> f64 {
    let (start, end) = span;
    let (left, right) = segment;
    let attack = settings.attack_ms as f64;
    let release = settings.release_ms as f64;
    let gain = settings.gain;
    if left >= start && right <= end {
        gain
    } else if attack > 0.0 && left >= start - attack && right <= start {
        1.0 - (1.0 - gain) * ((time - (start - attack)) / attack)
    } else if release > 0.0 && left >= end && right <= end + release {
        gain + (1.0 - gain) * ((time - end) / release)
    } else {
        1.0
    }
}

fn envelope(
    mut intervals: Vec<(f64, f64)>,
    settings: &AudioBusDucking,
    duration: f64,
) -> Result<Vec<EnvelopeSegment>, CoreError> {
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (start, end) in intervals {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
        } else {
            if merged.len() == MAX_MERGED_INTERVALS {
                return Err(invalid(
                    "audio bus ducking exceeds merged source interval limit",
                ));
            }
            merged.push((start, end));
        }
    }
    if merged.is_empty() {
        return Ok(Vec::new());
    }
    let attack = settings.attack_ms as f64;
    let release = settings.release_ms as f64;
    let mut points = Vec::with_capacity(merged.len() * 5 + 2);
    points.extend([0.0, duration]);
    for &(start, end) in &merged {
        points.extend([start - attack, start, end, end + release].map(|t| t.clamp(0.0, duration)));
    }
    if attack > 0.0 && release > 0.0 {
        for pair in merged.windows(2) {
            let previous_end = pair[0].1;
            let next_start = pair[1].0;
            let time = (release * next_start + attack * previous_end) / (attack + release);
            if time > previous_end.max(next_start - attack)
                && time < (previous_end + release).min(next_start)
            {
                points.push(time.clamp(0.0, duration));
            }
        }
    }
    points.sort_by(f64::total_cmp);
    points.dedup();
    let mut segments = Vec::new();
    for pair in points.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let segment = (start, end);
        // Every branch is linear throughout this disjoint segment. Average
        // endpoint gains evaluate its mathematical interior even when no
        // representable clock lies strictly between the endpoints.
        let mean = |span| {
            (branch(start, segment, span, settings) + branch(end, segment, span, settings)) * 0.5
        };
        let source = merged
            .iter()
            .copied()
            .min_by(|a, b| mean(*a).total_cmp(&mean(*b)))
            .unwrap();
        let start_gain = branch(start, segment, source, settings).clamp(0.0, 1.0);
        let end_gain = branch(end, segment, source, settings).clamp(0.0, 1.0);
        if start_gain == 1.0 && end_gain == 1.0 {
            continue;
        }
        if segments.len() == MAX_ENVELOPE_SEGMENTS {
            return Err(invalid("audio bus ducking exceeds envelope segment limit"));
        }
        if [start, end, start_gain, end_gain]
            .iter()
            .any(|v| !v.is_finite())
            || start >= end
        {
            return Err(invalid(
                "audio bus ducking envelope must be finite and ordered",
            ));
        }
        segments.push(EnvelopeSegment {
            start_ms: start,
            end_ms: end,
            start_gain,
            end_gain,
        });
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings(attack_ms: u64, release_ms: u64) -> AudioBusDucking {
        AudioBusDucking {
            enabled: true,
            source_bus_id: "voiceover".into(),
            gain: 0.25,
            attack_ms,
            release_ms,
        }
    }
    fn at(segments: &[EnvelopeSegment], time: f64) -> f64 {
        segments
            .iter()
            .find(|s| time >= s.start_ms && time < s.end_ms)
            .map_or(1.0, |s| {
                s.start_gain
                    + (s.end_gain - s.start_gain) * ((time - s.start_ms) / (s.end_ms - s.start_ms))
            })
    }
    #[test]
    fn independent_attack_hold_release_and_adjacent_minimum() {
        let segments = envelope(
            vec![(500.0, 600.0), (200.0, 400.0)],
            &settings(100, 200),
            1000.0,
        )
        .unwrap();
        for (time, expected) in [
            (0.0, 1.0),
            (150.0, 0.625),
            (200.0, 0.25),
            (400.0, 0.25),
            (450.0, 0.4375),
            (480.0, 0.4),
            (500.0, 0.25),
            (700.0, 0.625),
            (800.0, 1.0),
        ] {
            assert!((at(&segments, time) - expected).abs() < 1e-12, "{time}");
        }
    }
    #[test]
    fn anticipatory_clipping_and_instantaneous_changes_are_exact() {
        let clipped = envelope(vec![(50.0, 100.0)], &settings(100, 200), 500.0).unwrap();
        assert_eq!(at(&clipped, 0.0), 0.625);
        let instantaneous = envelope(vec![(50.0, 100.0)], &settings(0, 0), 500.0).unwrap();
        assert_eq!(at(&instantaneous, 49.999), 1.0);
        assert_eq!(at(&instantaneous, 50.0), 0.25);
        assert_eq!(at(&instantaneous, 100.0), 1.0);
    }
    #[test]
    fn adjacent_finite_endpoints_preserve_positive_instantaneous_hold() {
        let start = f64::from_bits(1.0_f64.to_bits() + 1);
        let end = f64::from_bits(1.0_f64.to_bits() + 2);
        assert!(start < end);
        let segments = envelope(vec![(start, end)], &settings(0, 0), 2.0).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start_ms, start);
        assert_eq!(segments[0].end_ms, end);
        assert_eq!(at(&segments, start), 0.25);
        assert_eq!(at(&segments, end), 1.0);
    }
    #[test]
    fn merged_interval_admission_preserves_the_boundary() {
        let spans = (0..64)
            .map(|i| (i as f64 * 1000.0, i as f64 * 1000.0 + 100.0))
            .collect::<Vec<_>>();
        let admitted = envelope(spans.clone(), &settings(100, 200), 65_000.0).unwrap();
        assert!(admitted.len() <= MAX_ENVELOPE_SEGMENTS);
        let mut excess = spans;
        excess.push((64_000.0, 64_100.0));
        assert_eq!(
            envelope(excess, &settings(100, 200), 65_000.0)
                .unwrap_err()
                .code,
            crate::ErrorCode::InvalidArgument
        );
        assert!(envelope(vec![(0.0, 100.0); 4096], &settings(100, 200), 1000.0).is_ok());
    }
}
