//! Shallow lowering of bounded, disjoint canonical envelope segments.
use crate::evaluated_scene::audio_bus_ducking::EnvelopeSegment;

pub(super) fn expression(segments: &[EnvelopeSegment]) -> String {
    let mut expression = "1".to_owned();
    for segment in segments {
        let start = segment.start_ms;
        let end = segment.end_ms;
        let duration = end - start;
        expression.push_str(&format!(
            "+if(gte(t*1000,{start:.17e})*lt(t*1000,{end:.17e}),({:.17e}-1)+({:.17e}-{:.17e})*(t*1000-{start:.17e})/{duration:.17e},0)",
            segment.start_gain, segment.end_gain, segment.start_gain,
        ));
    }
    expression
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_tiny_component_clock_never_lowers_to_zero_duration() {
        let filter = expression(&[EnvelopeSegment {
            start_ms: 1e-15,
            end_ms: 2e-15,
            start_gain: 0.25,
            end_gain: 1.0,
        }]);
        let duration = filter.rsplit_once('/').unwrap().1.trim_end_matches(",0)");
        let duration: f64 = duration.parse().unwrap();
        assert!(duration.is_finite() && duration > 0.0);
        assert!((duration - 1e-15).abs() < 1e-30);
        assert!(filter.contains("gte(t*1000,"));
        assert!(!filter.contains("NaN") && !filter.contains("inf"));
    }
}
