//! Canonical audio-analysis admission, output and original-PCM statistics.
use serde::{Deserialize, Serialize};

use super::{MediaInputRequest, RenderIntent, RenderPlan, append_audio_layer, audio_bus_dsp};
use crate::evaluated_scene::{EvaluatedEasing, EvaluatedScene};
use crate::{CoreError, ErrorCode};
use std::{collections::HashMap, path::PathBuf};

pub(crate) const SAMPLE_RATE: u32 = 48_000;
pub(crate) const MAX_PROJECT_MS: u64 = 600_000;
pub(crate) const MAX_FRAMES: u64 = 28_800_000;
pub(crate) const MAX_BINS: u32 = 4096;
pub(crate) const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct AudioAnalysisPlan {
    pub(crate) render: RenderPlan,
    pub(crate) options: AudioAnalysisOptions,
    pub(crate) expected_frames: u64,
}

pub(crate) fn build_audio_analysis_plan(
    scene: &EvaluatedScene,
    inputs: Vec<MediaInputRequest>,
    paths: Vec<PathBuf>,
    options: AudioAnalysisOptions,
) -> Result<AudioAnalysisPlan, CoreError> {
    let expected_frames = options.admit(scene.duration_ms)?;
    let indexes = inputs
        .iter()
        .map(|input| (input.item_id.as_str(), input.input_index))
        .collect::<HashMap<_, _>>();
    let mut filters = vec!["[0:v]nullsink".to_owned()];
    let mut labels = vec!["[1:a]".to_owned()];
    for audio in &scene.audio_layers {
        append_audio_layer(
            &mut filters,
            &mut labels,
            audio,
            &scene.voiceover_intervals,
            scene.instance_voiceover_intervals.as_deref(),
            &indexes,
        )?;
    }
    if let Some(graph) = &scene.audio_bus_graph {
        audio_bus_dsp::compile(&mut filters, &labels, scene, graph)?;
    } else {
        filters.push(format!(
            "{}amix=inputs={}:duration=longest:normalize=0[audio]",
            labels.join(""),
            labels.len()
        ));
    }
    filters.push(format!("[audio]aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000,atrim=start_sample={}:end_sample={},asetpts=PTS-STARTPTS[analysis]",options.start_ms*48,options.end_ms*48));
    Ok(AudioAnalysisPlan {
        expected_frames,
        options,
        render: RenderPlan {
            text_layout_fidelity: false,
            serial_bezier_filters: scene.audio_layers.iter().any(|audio| {
                audio
                    .volume_keyframes
                    .iter()
                    .any(|key| matches!(key.easing, EvaluatedEasing::CubicBezier { .. }))
            }),
            detail_fidelity: false,
            filter_graph: filters.join(";\n"),
            width: 2,
            height: 2,
            fps: scene.canvas.fps,
            duration_ms: scene.duration_ms,
            intent: RenderIntent::Export,
            media_inputs: inputs,
            media_paths: paths,
        },
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioAnalysisOptions {
    pub start_ms: u64,
    pub end_ms: u64,
    pub waveform_bins: u32,
}
impl AudioAnalysisOptions {
    pub(crate) fn admit(self, project_duration_ms: u64) -> Result<u64, CoreError> {
        if project_duration_ms > MAX_PROJECT_MS
            || self.start_ms >= self.end_ms
            || self.end_ms > project_duration_ms
            || !(1..=MAX_BINS).contains(&self.waveform_bins)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "audio analysis options exceed supported bounds",
            ));
        }
        (self.end_ms - self.start_ms)
            .checked_mul(u64::from(SAMPLE_RATE) / 1000)
            .filter(|frames| (1..=MAX_FRAMES).contains(frames))
            .ok_or_else(|| {
                CoreError::new(ErrorCode::InvalidArgument, "audio analysis frame overflow")
            })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioAnalysisSummary {
    pub start_ms: u64,
    pub end_ms: u64,
    pub sample_rate_hz: u32,
    pub channels: u32,
    pub frame_count: u64,
    pub actual_bin_count: u32,
    pub linear_sample_peak: f64,
    #[serde(deserialize_with = "required_nullable_metric")]
    pub sample_peak_dbfs: Option<f64>,
    #[serde(deserialize_with = "required_nullable_metric")]
    pub integrated_lufs: Option<f64>,
    #[serde(deserialize_with = "required_nullable_metric")]
    pub true_peak_dbtp: Option<f64>,
    pub loudness_range_lu: f64,
    pub threshold_lufs: f64,
}
fn required_nullable_metric<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    Option::<f64>::deserialize(deserializer)
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AudioChannelStatistics {
    pub min: f64,
    pub max: f64,
    pub rms: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioWaveformBin {
    pub start_frame: u64,
    pub end_frame: u64,
    pub left: AudioChannelStatistics,
    pub right: AudioChannelStatistics,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AudioAnalysisDocument {
    pub version: u32,
    pub summary: AudioAnalysisSummary,
    pub bins: Vec<AudioWaveformBin>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LoudnessMetrics {
    pub integrated_lufs: Option<f64>,
    pub true_peak_dbtp: Option<f64>,
    pub loudness_range_lu: f64,
    pub threshold_lufs: f64,
}
pub(crate) fn invalid_output() -> CoreError {
    CoreError::render_failure("render", None, None)
}
impl AudioAnalysisDocument {
    pub(crate) fn validate(&self, options: AudioAnalysisOptions) -> Result<(), CoreError> {
        let expected = options.admit(options.end_ms)?;
        let summary = &self.summary;
        let bin_count = u64::from(options.waveform_bins).min(expected) as u32;
        let peak = summary.linear_sample_peak;
        if self.version != 1
            || summary.start_ms != options.start_ms
            || summary.end_ms != options.end_ms
            || summary.sample_rate_hz != SAMPLE_RATE
            || summary.channels != 2
            || summary.frame_count != expected
            || summary.actual_bin_count != bin_count
            || self.bins.len() != bin_count as usize
            || !peak.is_finite()
            || !(0.0..=f64::from(f32::MAX)).contains(&peak)
            || !summary.loudness_range_lu.is_finite()
            || summary.loudness_range_lu < 0.0
            || !summary.threshold_lufs.is_finite()
            || [
                summary.integrated_lufs,
                summary.true_peak_dbtp,
                summary.sample_peak_dbfs,
            ]
            .into_iter()
            .flatten()
            .any(|value| !value.is_finite())
        {
            return Err(invalid_output());
        }
        let expected_db = (peak > 0.0).then(|| 20.0 * peak.log10());
        if match (expected_db, summary.sample_peak_dbfs) {
            (None, None) => false,
            (Some(expected), Some(actual)) => (expected - actual).abs() > 1e-9,
            _ => true,
        } || (peak > 0.0 && summary.true_peak_dbtp.is_none())
            || (peak == 0.0
                && (summary.true_peak_dbtp.is_some() || summary.integrated_lufs.is_some()))
        {
            return Err(invalid_output());
        }
        let mut bin_peak = 0.0_f64;
        for (index, bin) in self.bins.iter().enumerate() {
            let index = index as u64;
            if bin.start_frame != index * expected / u64::from(bin_count)
                || bin.end_frame != (index + 1) * expected / u64::from(bin_count)
                || bin.start_frame >= bin.end_frame
            {
                return Err(invalid_output());
            }
            for channel in [bin.left, bin.right] {
                let maximum = channel.min.abs().max(channel.max.abs());
                if ![channel.min, channel.max, channel.rms]
                    .iter()
                    .all(|v| v.is_finite())
                    || channel.min > channel.max
                    || channel.rms < 0.0
                    || maximum > f64::from(f32::MAX)
                    || channel.rms > maximum + 1e-9
                {
                    return Err(invalid_output());
                }
                bin_peak = bin_peak.max(maximum);
            }
        }
        if bin_peak != peak {
            return Err(invalid_output());
        }
        Ok(())
    }

    pub(crate) fn bounded_json(&self, options: AudioAnalysisOptions) -> Result<Vec<u8>, CoreError> {
        self.validate(options)?;
        // Admission bounds bins and all scalar representation before serialization.
        let bytes = serde_json::to_vec(self).map_err(|_| invalid_output())?;
        if bytes.len() > MAX_JSON_BYTES {
            return Err(invalid_output());
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug)]
struct ChannelAccumulator {
    min: f64,
    max: f64,
    squares: f64,
}
impl Default for ChannelAccumulator {
    fn default() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            squares: 0.0,
        }
    }
}
impl ChannelAccumulator {
    fn push(&mut self, value: f64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.squares += value * value;
    }
    fn finish(self, count: u64) -> AudioChannelStatistics {
        AudioChannelStatistics {
            min: self.min,
            max: self.max,
            rms: (self.squares / count as f64).sqrt(),
        }
    }
}
#[derive(Debug)]
pub(crate) struct PcmAccumulator {
    expected: u64,
    frames: u64,
    bins: Vec<(u64, u64, [ChannelAccumulator; 2])>,
    current: usize,
    peak: f64,
}
impl PcmAccumulator {
    pub(crate) fn new(expected: u64, requested_bins: u32) -> Result<Self, CoreError> {
        if !(1..=MAX_FRAMES).contains(&expected) || !(1..=MAX_BINS).contains(&requested_bins) {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "invalid PCM analysis admission",
            ));
        }
        let count = expected.min(u64::from(requested_bins));
        Ok(Self {
            expected,
            frames: 0,
            bins: (0..count)
                .map(|k| {
                    (
                        k * expected / count,
                        (k + 1) * expected / count,
                        [ChannelAccumulator::default(); 2],
                    )
                })
                .collect(),
            current: 0,
            peak: 0.0,
        })
    }
    pub(crate) fn push(&mut self, frame: [f32; 2]) -> Result<(), CoreError> {
        if self.frames >= self.expected || !frame.iter().all(|value| value.is_finite()) {
            return Err(invalid_output());
        }
        if self.frames >= self.bins[self.current].1 {
            self.current += 1;
        }
        for (channel, value) in self.bins[self.current].2.iter_mut().zip(frame) {
            let value = f64::from(value);
            channel.push(value);
            self.peak = self.peak.max(value.abs());
        }
        self.frames += 1;
        Ok(())
    }
    pub(crate) fn frames(&self) -> u64 {
        self.frames
    }
    pub(crate) fn finish(
        self,
        options: AudioAnalysisOptions,
        metrics: LoudnessMetrics,
    ) -> Result<AudioAnalysisDocument, CoreError> {
        if self.frames != self.expected {
            return Err(invalid_output());
        }
        let actual_bin_count = self.bins.len() as u32;
        let bins = self
            .bins
            .into_iter()
            .map(|(start, end, channels)| AudioWaveformBin {
                start_frame: start,
                end_frame: end,
                left: channels[0].finish(end - start),
                right: channels[1].finish(end - start),
            })
            .collect();
        Ok(AudioAnalysisDocument {
            version: 1,
            summary: AudioAnalysisSummary {
                start_ms: options.start_ms,
                end_ms: options.end_ms,
                sample_rate_hz: SAMPLE_RATE,
                channels: 2,
                frame_count: self.frames,
                actual_bin_count,
                linear_sample_peak: self.peak,
                sample_peak_dbfs: (self.peak > 0.0).then(|| 20.0 * self.peak.log10()),
                integrated_lufs: metrics.integrated_lufs,
                true_peak_dbtp: metrics.true_peak_dbtp,
                loudness_range_lu: metrics.loudness_range_lu,
                threshold_lufs: metrics.threshold_lufs,
            },
            bins,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn metrics() -> LoudnessMetrics {
        LoudnessMetrics {
            integrated_lufs: None,
            true_peak_dbtp: Some(12.04),
            loudness_range_lu: 0.0,
            threshold_lufs: -70.0,
        }
    }
    #[test]
    fn independent_five_frame_floor_partition_preserves_boundary_and_stereo_statistics() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../../contracts/audio-analysis-v1.json"))
                .unwrap();
        let mut pcm = PcmAccumulator::new(5, 2).unwrap();
        for frame in [
            [0.0, 1.0],
            [-2.0, 0.0],
            [3.0, -1.0],
            [0.0, 4.0],
            [-1.0, 0.0],
        ] {
            pcm.push(frame).unwrap();
        }
        let result = pcm
            .finish(
                AudioAnalysisOptions {
                    start_ms: 0,
                    end_ms: 1,
                    waveform_bins: 2,
                },
                metrics(),
            )
            .unwrap();
        let expected: Vec<AudioWaveformBin> =
            serde_json::from_value(fixture["independentPcm"]["bins"].clone()).unwrap();
        for (actual, expected) in result.bins.iter().zip(expected) {
            assert_eq!(
                (actual.start_frame, actual.end_frame),
                (expected.start_frame, expected.end_frame)
            );
            for (actual, expected) in [(actual.left, expected.left), (actual.right, expected.right)]
            {
                assert_eq!(actual.min, expected.min);
                assert_eq!(actual.max, expected.max);
                assert!((actual.rms - expected.rms).abs() < 1e-12);
            }
        }
        assert_eq!(result.summary.linear_sample_peak, 4.0);
        assert!((result.summary.sample_peak_dbfs.unwrap() - 12.041199826559248).abs() < 1e-12);
    }
    #[test]
    fn bin_count_short_nonfinite_and_oversized_streams_are_bounded() {
        let mut pcm = PcmAccumulator::new(2, 4096).unwrap();
        pcm.push([-1.0, 0.0]).unwrap();
        assert!(pcm.push([f32::NAN, 0.0]).is_err());
        pcm.push([0.0, -2.0]).unwrap();
        assert!(pcm.push([0.0, 0.0]).is_err());
        let result = pcm
            .finish(
                AudioAnalysisOptions {
                    start_ms: 0,
                    end_ms: 1,
                    waveform_bins: 4096,
                },
                metrics(),
            )
            .unwrap();
        assert_eq!(result.bins.len(), 2);
        assert_eq!(result.bins[0].left.max, -1.0);
        assert_eq!(result.summary.linear_sample_peak, 2.0);
        assert!(
            PcmAccumulator::new(2, 1)
                .unwrap()
                .finish(
                    AudioAnalysisOptions {
                        start_ms: 0,
                        end_ms: 1,
                        waveform_bins: 1
                    },
                    metrics()
                )
                .is_err()
        );
        for (frames, bins) in [(0, 1), (MAX_FRAMES + 1, 1), (1, 0), (1, MAX_BINS + 1)] {
            assert!(PcmAccumulator::new(frames, bins).is_err());
        }
    }
    #[test]
    fn valid_silence_json_and_injected_poison_are_checked_before_publication() {
        let options = AudioAnalysisOptions {
            start_ms: 0,
            end_ms: 1,
            waveform_bins: 4096,
        };
        let mut pcm = PcmAccumulator::new(48, 4096).unwrap();
        for _ in 0..48 {
            pcm.push([0.0, 0.0]).unwrap();
        }
        let result = pcm
            .finish(
                options,
                LoudnessMetrics {
                    true_peak_dbtp: None,
                    integrated_lufs: None,
                    ..metrics()
                },
            )
            .unwrap();
        assert_eq!(result.summary.actual_bin_count, 48);
        assert!(result.summary.sample_peak_dbfs.is_none());
        assert!(result.bounded_json(options).unwrap().len() < MAX_JSON_BYTES);
        let mut poison = result.clone();
        poison.bins[1].start_frame = 0;
        assert!(poison.validate(options).is_err());
        let mut poison = result.clone();
        poison.summary.threshold_lufs = f64::NAN;
        assert!(poison.validate(options).is_err());
        let mut poison = result.clone();
        poison.bins[0].left.rms = 1.0;
        assert!(poison.validate(options).is_err());
        let mut poison = result;
        poison.summary.integrated_lufs = Some(-24.0);
        assert!(poison.validate(options).is_err());
    }
    #[test]
    fn analysis_limits_do_not_overflow_or_accept_empty_ranges() {
        let options = AudioAnalysisOptions {
            start_ms: 0,
            end_ms: MAX_PROJECT_MS,
            waveform_bins: 1,
        };
        assert_eq!(options.admit(MAX_PROJECT_MS).unwrap(), MAX_FRAMES);
        for options in [
            AudioAnalysisOptions {
                end_ms: u64::MAX,
                ..options
            },
            AudioAnalysisOptions {
                start_ms: 1,
                end_ms: 1,
                ..options
            },
            AudioAnalysisOptions {
                waveform_bins: 0,
                ..options
            },
        ] {
            assert_eq!(
                options.admit(MAX_PROJECT_MS).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        assert!(options.admit(MAX_PROJECT_MS + 1).is_err());
    }
}
