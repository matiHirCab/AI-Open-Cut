//! FFmpeg and FFprobe process execution owner.

mod audio_analysis;

use std::{
    fmt::Debug,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use serde::{Deserialize, Serialize};

use crate::{
    CoreError, ErrorCode, MediaType,
    render_plan::{RenderIntent, RenderPlan, seconds},
};

pub(crate) const SPAWN_STAGE: &str = "spawn";
pub(crate) const RENDER_STAGE: &str = "render";
pub(crate) const STDERR_TAIL_BYTES: usize = 16_384;
pub(crate) const STDERR_EXCERPT_BYTES: usize = 4_096;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderProgress {
    pub progress: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub duration_ms: Option<u64>,
    pub has_video: bool,
    pub has_audio: bool,
    pub format_name: Option<String>,
    pub video_codec: Option<String>,
    pub video_width: Option<u32>,
    pub video_height: Option<u32>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<u32>,
    pub audio_sample_rate_hz: Option<u32>,
}

pub(crate) fn readiness(ffmpeg_path: &Path, ffprobe_path: &Path) -> Result<(), CoreError> {
    let output = Command::new(ffmpeg_path)
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|error| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("cannot start FFmpeg: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "FFmpeg readiness check failed",
        ));
    }
    let filters = String::from_utf8_lossy(&output.stdout);
    for required in [
        " overlay ",
        " drawtext ",
        " amix ",
        " geq ",
        " remap ",
        " blend ",
        " nullsrc ",
        " split ",
        " pad ",
        " crop ",
        " format ",
    ] {
        if !filters.contains(required) {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("FFmpeg is missing the {} filter", required.trim()),
            ));
        }
    }
    let status = Command::new(ffprobe_path)
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("cannot start FFprobe: {error}"),
            )
        })?;
    if !status.success() {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "FFprobe readiness check failed",
        ));
    }
    Ok(())
}

fn audio_bus_dsp_readiness(ffmpeg_path: &Path) -> Result<(), CoreError> {
    let output = Command::new(ffmpeg_path)
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|error| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("cannot start FFmpeg: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio bus DSP readiness check failed",
        ));
    }
    let filters = String::from_utf8_lossy(&output.stdout);
    for required in [
        " volume ",
        " aformat ",
        " pan ",
        " equalizer ",
        " acompressor ",
    ] {
        if !filters.contains(required) {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("FFmpeg is missing the {} filter", required.trim()),
            ));
        }
    }
    Ok(())
}

fn audio_bus_ducking_readiness(ffmpeg_path: &Path) -> Result<(), CoreError> {
    let output = Command::new(ffmpeg_path)
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|error| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("cannot start FFmpeg: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio bus ducking readiness check failed",
        ));
    }
    let filters = String::from_utf8_lossy(&output.stdout);
    for required in [" volume ", " aformat ", " pan "] {
        if !filters.contains(required) {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("FFmpeg is missing the {} filter", required.trim()),
            ));
        }
    }
    Ok(())
}

pub(crate) fn probe(ffprobe_path: &Path, path: &Path) -> Result<ProbeResult, CoreError> {
    let output = Command::new(ffprobe_path)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration,format_name:stream=codec_type,codec_name,width,height,channels,sample_rate",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .map_err(|error| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                format!("cannot start FFprobe: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(CoreError::new(
            ErrorCode::UnsupportedMedia,
            "FFprobe could not read the selected media",
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let duration_ms = value
        .get("format")
        .and_then(|format| format.get("duration"))
        .and_then(serde_json::Value::as_str)
        .and_then(|duration| duration.parse::<f64>().ok())
        .filter(|duration| duration.is_finite() && *duration >= 0.0)
        .map(|duration| (duration * 1_000.0).round() as u64);
    let streams = value
        .get("streams")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(ProbeResult {
        duration_ms,
        has_video: streams.iter().any(|stream| {
            stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("video")
        }),
        has_audio: streams.iter().any(|stream| {
            stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("audio")
        }),
        format_name: value
            .get("format")
            .and_then(|format| format.get("format_name"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        video_codec: stream_string(&streams, "video", "codec_name"),
        video_width: stream_u32(&streams, "video", "width"),
        video_height: stream_u32(&streams, "video", "height"),
        audio_codec: stream_string(&streams, "audio", "codec_name"),
        audio_channels: stream_u32(&streams, "audio", "channels"),
        audio_sample_rate_hz: stream_u32(&streams, "audio", "sample_rate"),
    })
}

// Render-only metadata never changes the persisted/public probe contract.
fn probe_render_geometry(
    ffprobe_path: &Path,
    path: &Path,
    kind: MediaType,
) -> Result<(u32, u32), CoreError> {
    let invalid = || {
        CoreError::new(
            ErrorCode::UnsupportedMedia,
            "unusable render source metadata",
        )
    };
    let mut command = Command::new(ffprobe_path);
    command.args(["-v", "error", "-select_streams", "v:0"]);
    if kind == MediaType::Image {
        command.args([
            "-read_intervals",
            "%+#1",
            "-show_entries",
            "frame=width,height:frame_side_data=side_data_type,displaymatrix",
        ]);
    } else {
        command.args([
            "-show_entries",
            "stream=width,height:stream_side_data=side_data_type,displaymatrix",
        ]);
    }
    let mut child = command
        .args(["-of", "json"])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                "cannot start render metadata probe",
            )
        })?;
    let read = read_geometry_metadata(child.stdout.take().expect("piped probe stdout"));
    let bytes = match read {
        Ok(bytes) => bytes,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    if !child.wait().map_err(|_| invalid())?.success() {
        return Err(invalid());
    }
    let metadata = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    oriented_geometry(&metadata, kind)
}

fn read_geometry_metadata(reader: impl Read) -> Result<Vec<u8>, CoreError> {
    const MAX_METADATA_BYTES: u64 = 65_536;
    let mut bytes = Vec::new();
    let read = reader.take(MAX_METADATA_BYTES + 1).read_to_end(&mut bytes);
    if read.is_err() || bytes.len() as u64 > MAX_METADATA_BYTES {
        return Err(CoreError::new(
            ErrorCode::UnsupportedMedia,
            "unusable render source metadata",
        ));
    }
    Ok(bytes)
}

fn oriented_geometry(
    metadata: &serde_json::Value,
    kind: MediaType,
) -> Result<(u32, u32), CoreError> {
    let invalid = || {
        CoreError::new(
            ErrorCode::UnsupportedMedia,
            "unusable render source metadata",
        )
    };
    let streams = metadata
        .get(if kind == MediaType::Image {
            "frames"
        } else {
            "streams"
        })
        .and_then(serde_json::Value::as_array)
        .ok_or_else(invalid)?;
    if streams.len() != 1 {
        return Err(invalid());
    }
    let stream = &streams[0];
    let dimension = |key| {
        stream
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or_else(invalid)
    };
    let (width, height) = (dimension("width")?, dimension("height")?);
    let mut rotation = None;
    if let Some(side_data) = stream.get("side_data_list") {
        for entry in side_data.as_array().ok_or_else(invalid)? {
            if entry
                .get("side_data_type")
                .and_then(serde_json::Value::as_str)
                == Some(if kind == MediaType::Image {
                    "3x3 displaymatrix"
                } else {
                    "Display Matrix"
                })
            {
                if rotation.is_some() {
                    return Err(invalid());
                }
                let matrix = entry
                    .get("displaymatrix")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(invalid)?;
                let mut values = Vec::with_capacity(9);
                for line in matrix.lines().filter(|line| !line.trim().is_empty()) {
                    let (_, row) = line.split_once(':').ok_or_else(invalid)?;
                    let row = row
                        .split_whitespace()
                        .map(str::parse::<i32>)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| invalid())?;
                    if row.len() != 3 || values.len() >= 9 {
                        return Err(invalid());
                    }
                    values.extend(row);
                }
                let m: [i32; 9] = values.try_into().map_err(|_| invalid())?;
                let sx = f64::from(m[0]).hypot(f64::from(m[3]));
                let sy = f64::from(m[1]).hypot(f64::from(m[4]));
                if sx == 0.0 || sy == 0.0 {
                    return Err(invalid());
                }
                // FFprobe's printed rotation truncates fractional degrees. Derive
                // the angle from the fixed-point matrix, then match FFmpeg's
                // get_rotation rounding before applying its transpose tolerance.
                rotation = Some(
                    (f64::from(m[1]) / sy)
                        .atan2(f64::from(m[0]) / sx)
                        .to_degrees()
                        .round(),
                );
            }
        }
    }
    // FFmpeg transposes only within a strict one-degree quarter-turn tolerance.
    // Its other autorotation/flip filters retain the input raster dimensions.
    let angle = rotation.unwrap_or(0.0).rem_euclid(360.0);
    if (angle - 90.0).abs() < 1.0 || (angle - 270.0).abs() < 1.0 {
        Ok((height, width))
    } else {
        Ok((width, height))
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn display_orientation_extents_match_backend() {
        for (angle, swapped) in [
            (0.0_f64, false),
            (90.0, true),
            (-90.0, true),
            (180.0, false),
            (270.0, true),
            (450.0, true),
            (45.0, false),
            (89.0, false),
            (89.5, true),
            (91.0, false),
        ] {
            let (sin, cos) = angle.to_radians().sin_cos();
            let (sin, cos) = ((sin * 65536.0) as i32, (cos * 65536.0) as i32);
            let matrix = format!(
                "00000000: {cos} {} 0\n00000001: {sin} {cos} 0\n00000002: 0 0 1073741824\n",
                -sin
            );
            let value = json!({"streams":[{"width":40,"height":20,"side_data_list":[{"side_data_type":"Display Matrix","displaymatrix":matrix}]}]});
            assert_eq!(
                oriented_geometry(&value, MediaType::Video).unwrap(),
                if swapped { (20, 40) } else { (40, 20) }
            );
        }
        assert_eq!(
            oriented_geometry(
                &json!({"streams":[{"width":40,"height":20}]}),
                MediaType::Video
            )
            .unwrap(),
            (40, 20)
        );
        // A flip does not change extent; only the display rotation is relevant here.
        assert_eq!(oriented_geometry(&json!({"streams":[{"width":40,"height":20,"side_data_list":[{"side_data_type":"Display Matrix","displaymatrix":"0: -65536 0 0\n1: 0 65536 0\n2: 0 0 1073741824"}]}]}), MediaType::Video).unwrap(), (40,20));
    }
    #[test]
    fn metadata_output_is_bounded_and_read_errors_fail_closed() {
        assert_eq!(
            read_geometry_metadata(&vec![b' '; 65_536][..])
                .unwrap()
                .len(),
            65_536
        );
        let mut excessive = std::io::Cursor::new(vec![b' '; 100_000]);
        assert_eq!(
            read_geometry_metadata(&mut excessive).unwrap_err().code,
            ErrorCode::UnsupportedMedia
        );
        assert_eq!(excessive.position(), 65_537);
        struct BrokenReader;
        impl Read for BrokenReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("injected read failure"))
            }
        }
        assert_eq!(
            read_geometry_metadata(BrokenReader).unwrap_err().code,
            ErrorCode::UnsupportedMedia
        );
    }

    #[test]
    fn image_frame_metadata_is_authoritative_and_fails_closed() {
        let base = json!({"width":40,"height":20});
        for matrix in [
            None,
            Some("0: -65536 0 0\n1: 0 65536 0\n2: 0 0 1073741824"),
            Some("0: 0 65536 0\n1: 65536 0 0\n2: 0 0 1073741824"),
        ] {
            let mut frame = base.clone();
            if let Some(matrix) = matrix {
                frame["side_data_list"] =
                    json!([{"side_data_type":"3x3 displaymatrix","displaymatrix":matrix}]);
            }
            let value = json!({"frames":[frame],"streams":[{"width":999,"height":999}]});
            assert_eq!(
                oriented_geometry(&value, MediaType::Image).unwrap(),
                if matrix.is_some_and(|m| m.starts_with("0: 0")) {
                    (20, 40)
                } else {
                    (40, 20)
                }
            );
        }
        for frames in [
            json!([]),
            json!([base.clone(), base.clone()]),
            json!([{"width":0,"height":20}]),
            json!([{"width":40,"height":-1}]),
            json!([{"width":40,"height":20,"side_data_list":[{"side_data_type":"3x3 displaymatrix","displaymatrix":"bad"}]}]),
            json!([{"width":40,"height":20,"side_data_list":[{"side_data_type":"3x3 displaymatrix"}]}]),
        ] {
            assert_eq!(
                oriented_geometry(&json!({"frames":frames,"streams":[base]}), MediaType::Image)
                    .unwrap_err()
                    .code,
                ErrorCode::UnsupportedMedia
            );
        }
        assert_eq!(
            oriented_geometry(&json!({"streams":[base]}), MediaType::Image)
                .unwrap_err()
                .code,
            ErrorCode::UnsupportedMedia
        );
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            probe_render_geometry(
                &root.path().join("missing-probe"),
                root.path(),
                MediaType::Image
            )
            .unwrap_err()
            .code,
            ErrorCode::DependencyUnavailable
        );
    }

    #[test]
    fn invalid_metadata_fails_closed() {
        for value in [
            json!({}),
            json!({"streams":[]}),
            json!({"streams":[{"width":0,"height":20}]}),
            json!({"streams":[{"width":40,"height":20,"side_data_list":[{"side_data_type":"Display Matrix","displaymatrix":"0: 0 0 0\n1: 0 0 0\n2: 0 0 0"}]}]}),
            json!({"streams":[{"width":40,"height":20,"side_data_list":[{"side_data_type":"Display Matrix"}]}]}),
            json!({"streams":[{"width":40,"height":20,"side_data_list":[{"side_data_type":"Display Matrix","rotation":"NaN"}]}]}),
        ] {
            assert_eq!(
                oriented_geometry(&value, MediaType::Video)
                    .unwrap_err()
                    .code,
                ErrorCode::UnsupportedMedia
            );
        }
    }
}

fn stream<'a>(streams: &'a [serde_json::Value], kind: &str) -> Option<&'a serde_json::Value> {
    streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(serde_json::Value::as_str) == Some(kind))
}

fn stream_string(streams: &[serde_json::Value], kind: &str, field: &str) -> Option<String> {
    stream(streams, kind)?
        .get(field)?
        .as_str()
        .map(str::to_owned)
}

fn stream_u32(streams: &[serde_json::Value], kind: &str, field: &str) -> Option<u32> {
    let value = stream(streams, kind)?.get(field)?;
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| value.as_str()?.parse().ok())
}

pub(crate) trait ProcessExecutor: Debug + Send + Sync {
    fn audio_analysis_readiness(&self, _ffmpeg_path: &Path) -> Result<(), CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio analysis is unavailable",
        ))
    }
    fn analyze_audio(
        &self,
        _ffmpeg_path: &Path,
        _plan: &crate::render_plan::audio_analysis::AudioAnalysisPlan,
        _filter_path: &Path,
        _pcm_path: &Path,
        _on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<crate::render_plan::audio_analysis::AudioAnalysisDocument, CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio analysis execution is unavailable",
        ))
    }
    fn audio_bus_dsp_readiness(&self, _ffmpeg_path: &Path) -> Result<(), CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio bus DSP filter readiness is unavailable",
        ))
    }
    fn audio_bus_ducking_readiness(&self, _ffmpeg_path: &Path) -> Result<(), CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio bus ducking filter readiness is unavailable",
        ))
    }
    fn raster_source(
        &self,
        _ffmpeg: &Path,
        _source: &str,
        _size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "sampled source rasterization is unavailable",
        ))
    }
    fn prepare_visual_stream(
        &self,
        _ffmpeg: &Path,
        _output: &Path,
        _fps: u32,
        _frames: u64,
        _produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "lossless sampled visual preparation is unavailable",
        ))
    }
    fn decode_visual_frame_at(
        &self,
        ffmpeg: &Path,
        path: &Path,
        at_ms: f64,
        size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        if !at_ms.is_finite() || at_ms < 0.0 {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "invalid visual source time",
            ));
        }
        if at_ms.fract() != 0.0 {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                "fractional visual decoding is unavailable",
            ));
        }
        self.decode_visual_frame(ffmpeg, path, at_ms as u64, size)
    }
    fn decode_visual_frame(
        &self,
        _ffmpeg_path: &Path,
        _path: &Path,
        _at_ms: u64,
        _size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "sampled visual decoding is unavailable",
        ))
    }
    fn readiness(&self, ffmpeg_path: &Path, ffprobe_path: &Path) -> Result<(), CoreError>;
    fn probe(&self, ffprobe_path: &Path, path: &Path) -> Result<ProbeResult, CoreError>;
    fn probe_render_geometry(
        &self,
        _ffprobe_path: &Path,
        _path: &Path,
        _kind: MediaType,
    ) -> Result<(u32, u32), CoreError> {
        Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "render geometry probing is unavailable",
        ))
    }

    fn execute(
        &self,
        ffmpeg_path: &Path,
        plan: &RenderPlan,
        filter_path: &Path,
        output: &Path,
        on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SystemProcessExecutor;

impl ProcessExecutor for SystemProcessExecutor {
    fn audio_analysis_readiness(&self, ffmpeg_path: &Path) -> Result<(), CoreError> {
        audio_analysis::readiness(ffmpeg_path)
    }
    fn analyze_audio(
        &self,
        ffmpeg_path: &Path,
        plan: &crate::render_plan::audio_analysis::AudioAnalysisPlan,
        filter_path: &Path,
        pcm_path: &Path,
        on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<crate::render_plan::audio_analysis::AudioAnalysisDocument, CoreError> {
        audio_analysis::execute(ffmpeg_path, plan, filter_path, pcm_path, on_progress)
    }
    fn audio_bus_dsp_readiness(&self, ffmpeg_path: &Path) -> Result<(), CoreError> {
        audio_bus_dsp_readiness(ffmpeg_path)
    }
    fn audio_bus_ducking_readiness(&self, ffmpeg_path: &Path) -> Result<(), CoreError> {
        audio_bus_ducking_readiness(ffmpeg_path)
    }
    fn raster_source(
        &self,
        ffmpeg: &Path,
        source: &str,
        size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        // Independently bound process output; canonical source/scene preflight runs first.
        let expected = crate::render_plan::decoded_visual_rgba_bytes(size)?;
        let mut child = Command::new(ffmpeg)
            .args([
                "-v",
                "error",
                "-nostdin",
                "-f",
                "lavfi",
                "-i",
                source,
                "-frames:v",
                "1",
                "-threads",
                "1",
                "-pix_fmt",
                "rgba",
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| CoreError::render_failure("visual_prepare", None, None))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::render_failure("visual_prepare", None, None))?;
        let reader = thread::spawn(move || {
            let mut tail = Vec::new();
            let mut stream = stderr;
            let mut block = [0; 4096];
            while let Ok(n) = stream.read(&mut block) {
                if n == 0 {
                    break;
                }
                tail.extend_from_slice(&block[..n]);
                if tail.len() > STDERR_TAIL_BYTES {
                    tail.drain(..tail.len() - STDERR_TAIL_BYTES);
                }
            }
            tail
        });
        let result = child
            .stdout
            .take()
            .ok_or_else(|| CoreError::render_failure("visual_prepare", None, None))
            .and_then(|stdout| {
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(expected + 1).map_err(|_| {
                    CoreError::new(
                        ErrorCode::InvalidArgument,
                        "source raster allocation failed",
                    )
                })?;
                stdout
                    .take(expected as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| CoreError::render_failure("visual_prepare", None, None))?;
                Ok(bytes)
            });
        if !matches!(&result, Ok(bytes) if bytes.len() == expected) {
            let _ = child.kill();
        }
        let status = child
            .wait()
            .map_err(|_| CoreError::render_failure("visual_prepare", None, None));
        let tail = reader.join().unwrap_or_default();
        let bytes = result?;
        let status = status?;
        if !status.success() || bytes.len() != expected {
            return Err(CoreError::render_failure(
                "visual_prepare",
                status.code(),
                stderr_excerpt(&tail),
            ));
        }
        Ok(bytes)
    }
    fn prepare_visual_stream(
        &self,
        ffmpeg: &Path,
        output: &Path,
        fps: u32,
        frames: u64,
        produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        let mut child = Command::new(ffmpeg)
            .args([
                "-v",
                "error",
                "-nostdin",
                "-f",
                "image2pipe",
                "-vcodec",
                "pam",
                "-framerate",
                &fps.to_string(),
                "-i",
                "pipe:0",
                "-an",
                "-c:v",
                "ffv1",
                "-level",
                "3",
                "-pix_fmt",
                "bgra",
                "-fflags",
                "+bitexact",
                "-flags",
                "+bitexact",
                "-y",
            ])
            .arg(output)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| CoreError::render_failure("visual_prepare", None, None))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::render_failure("visual_prepare", None, None))?;
        let reader = thread::spawn(move || {
            let mut tail = Vec::new();
            let mut stream = stderr;
            let mut block = [0; 4096];
            while let Ok(n) = stream.read(&mut block) {
                if n == 0 {
                    break;
                }
                tail.extend_from_slice(&block[..n]);
                if tail.len() > STDERR_TAIL_BYTES {
                    tail.drain(..tail.len() - STDERR_TAIL_BYTES);
                }
            }
            tail
        });
        let result: Result<(), CoreError> = (|| {
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| CoreError::render_failure("visual_prepare", None, None))?;
            for frame in 0..frames {
                stdin
                    .write_all(&produce(frame)?)
                    .map_err(|_| CoreError::render_failure("visual_prepare", None, None))?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = child.kill();
        }
        let status = child
            .wait()
            .map_err(|_| CoreError::render_failure("visual_prepare", None, None));
        let tail = reader.join().unwrap_or_default();
        result?;
        let status = status?;
        if !status.success() {
            return Err(CoreError::render_failure(
                "visual_prepare",
                status.code(),
                stderr_excerpt(&tail),
            ));
        }
        Ok(())
    }
    fn decode_visual_frame(
        &self,
        ffmpeg: &Path,
        path: &Path,
        at_ms: u64,
        size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        self.decode_visual_frame_at(ffmpeg, path, at_ms as f64, size)
    }
    fn decode_visual_frame_at(
        &self,
        ffmpeg: &Path,
        path: &Path,
        at_ms: f64,
        size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        let expected = crate::render_plan::decoded_visual_rgba_bytes(size)?;
        if !at_ms.is_finite() || at_ms < 0.0 {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "invalid visual source time",
            ));
        }
        // Seek backward to the preceding keyframe without discarding held frames.
        // Compare eligibility in the original timebase, before any AVTB rounding.
        // The first future frame receives an EOF marker at least one second
        // beyond the target; trim therefore stops near the preceding GOP.
        // Its 2us margin admits eligible timestamps rounded up by AVTB and
        // FFmpeg's exclusive, integer-microsecond trim boundary. No future
        // frame can enter through that margin because classification came first.
        // Retain only two bounded raw frames, never the complete decoded sequence.
        let mut command = Command::new(ffmpeg);
        command.args(["-v", "error", "-nostdin"]);
        if at_ms > 0.0 {
            command.args([
                "-ss",
                &format!("{:.17}", at_ms / 1000.0),
                "-noaccurate_seek",
                "-copyts",
                "-start_at_zero",
            ]);
        }
        let mut child = command
            .arg("-i")
            .arg(path)
            .args([
                "-an",
                "-vf",
                &format!(
                    "{}setpts='if(lte(PTS*TB,{:.17}),PTS,ceil(({:.17}+1)/TB))',settb=AVTB,trim=end={:.17},scale={}:{},format=rgba",
                    if at_ms == 0.0 {
                        "setpts=PTS-STARTPTS,"
                    } else {
                        ""
                    },
                    at_ms / 1000.0,
                    at_ms / 1000.0,
                    at_ms / 1000.0 + 0.000002,
                    size.0,
                    size.1
                ),
                "-fps_mode",
                "passthrough",
                "-f",
                "rawvideo",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| CoreError::render_failure("visual_decode", None, None))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CoreError::render_failure("visual_decode", None, None))?;
        let reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
        let result = (|| {
            let mut output = child
                .stdout
                .take()
                .ok_or_else(|| CoreError::render_failure("visual_decode", None, None))?;
            let mut latest = vec![0; expected];
            let mut buffer = vec![0; expected];
            let mut saw_frame = false;
            loop {
                match output.read(&mut buffer[..1]) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => return Err(CoreError::render_failure("visual_decode", None, None)),
                }
                output
                    .read_exact(&mut buffer[1..])
                    .map_err(|_| CoreError::render_failure("visual_decode", None, None))?;
                std::mem::swap(&mut latest, &mut buffer);
                saw_frame = true;
            }
            if !saw_frame {
                return Err(CoreError::render_failure("visual_decode", None, None));
            }
            Ok(latest)
        })();
        if result.is_err() {
            let _ = child.kill();
        }
        let status = child
            .wait()
            .map_err(|_| CoreError::render_failure("visual_decode", None, None));
        let tail = reader.join().ok().and_then(Result::ok).unwrap_or_default();
        let bytes = result?;
        let status = status?;
        if !status.success() {
            return Err(CoreError::render_failure(
                "visual_decode",
                status.code(),
                stderr_excerpt(&tail),
            ));
        }
        Ok(bytes)
    }
    fn probe_render_geometry(
        &self,
        ffprobe_path: &Path,
        path: &Path,
        kind: MediaType,
    ) -> Result<(u32, u32), CoreError> {
        probe_render_geometry(ffprobe_path, path, kind)
    }

    fn readiness(&self, ffmpeg_path: &Path, ffprobe_path: &Path) -> Result<(), CoreError> {
        readiness(ffmpeg_path, ffprobe_path)
    }

    fn probe(&self, ffprobe_path: &Path, path: &Path) -> Result<ProbeResult, CoreError> {
        probe(ffprobe_path, path)
    }

    fn execute(
        &self,
        ffmpeg_path: &Path,
        plan: &RenderPlan,
        filter_path: &Path,
        output: &Path,
        on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        let mut command = build_render_command(ffmpeg_path, plan, filter_path, output);
        let progress_duration = match plan.intent {
            RenderIntent::Range {
                start_ms, end_ms, ..
            } => end_ms.saturating_sub(start_ms),
            RenderIntent::Frame { .. } | RenderIntent::Export => plan.duration_ms,
        };
        run_command(&mut command, progress_duration, on_progress)
    }
}

pub(crate) fn build_render_command(
    ffmpeg_path: &Path,
    plan: &RenderPlan,
    filter_path: &Path,
    output: &Path,
) -> Command {
    let mut command = Command::new(ffmpeg_path);
    append_render_inputs(&mut command, plan);
    if plan.serial_bezier_filters {
        command.args(["-filter_complex_threads", "1"]);
    }
    command.arg("-filter_complex_script").arg(filter_path);
    match plan.intent {
        RenderIntent::Frame { at_ms } => {
            command
                .args(["-ss", &seconds(at_ms), "-frames:v", "1", "-map", "[video]"])
                .arg(output)
                .args(["-map", "[audio]", "-f", "null"])
                .arg(null_output());
        }
        RenderIntent::Range {
            start_ms,
            end_ms,
            include_audio,
        } => {
            let duration = seconds(end_ms.saturating_sub(start_ms));
            command.args(["-ss", &seconds(start_ms), "-map", "[video]"]);
            if include_audio {
                command.args(["-map", "[audio]", "-c:a", "aac"]);
            }
            command
                .args([
                    "-c:v",
                    "libx264",
                    "-preset",
                    if plan.detail_fidelity || plan.text_layout_fidelity {
                        "medium"
                    } else {
                        "veryfast"
                    },
                    "-crf",
                    if plan.text_layout_fidelity {
                        "18"
                    } else if plan.detail_fidelity {
                        "23"
                    } else {
                        "28"
                    },
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                    "-t",
                    &duration,
                    "-y",
                ])
                .arg(output);
            if !include_audio {
                command
                    .args(["-map", "[audio]", "-t", &duration, "-f", "null"])
                    .arg(null_output());
            }
        }
        RenderIntent::Export => {
            if plan.text_layout_fidelity {
                command.args(["-crf", "18", "-preset", "medium"]);
            }
            command
                .args([
                    "-map",
                    "[video]",
                    "-map",
                    "[audio]",
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                    "-movflags",
                    "+faststart",
                    "-t",
                    &seconds(plan.duration_ms),
                    "-y",
                ])
                .arg(output);
        }
    }
    command
}

fn append_render_inputs(command: &mut Command, plan: &RenderPlan) {
    append_render_inputs_mode(command, plan, true);
}

fn append_render_inputs_mode(command: &mut Command, plan: &RenderPlan, progress: bool) {
    command.args(["-hide_banner", "-loglevel", "error"]);
    if progress {
        command.args(["-progress", "pipe:1"]);
    }
    command.args([
        "-nostats",
        "-f",
        "lavfi",
        "-i",
        &format!(
            "color=c=black:s={}x{}:r={}:d={}",
            plan.width,
            plan.height,
            plan.fps,
            seconds(plan.duration_ms)
        ),
        "-f",
        "lavfi",
        "-i",
        &format!("anullsrc=r=48000:cl=stereo:d={}", seconds(plan.duration_ms)),
    ]);
    for (input, path) in plan.media_inputs.iter().zip(&plan.media_paths) {
        if input.media_type == MediaType::Image {
            command.args(["-loop", "1", "-t", &seconds(input.duration_ms), "-i"]);
        } else {
            command.args([
                "-ss",
                &seconds(input.source_in_ms),
                "-t",
                &seconds(input.duration_ms),
                "-i",
            ]);
        }
        command.arg(path);
    }
}

fn null_output() -> &'static str {
    if cfg!(windows) { "NUL" } else { "/dev/null" }
}

#[cfg(test)]
pub(crate) fn build_decode_benchmark_command(
    ffmpeg_path: &Path,
    plan: &RenderPlan,
) -> Option<Command> {
    if plan.media_inputs.is_empty() {
        return None;
    }
    let mut command = Command::new(ffmpeg_path);
    append_render_inputs(&mut command, plan);
    for input in &plan.media_inputs {
        let stream = match input.media_type {
            MediaType::Image | MediaType::Video => "v",
            MediaType::Audio => "a",
        };
        command.args(["-map", &format!("{}:{stream}:0", input.input_index)]);
    }
    command
        .args(["-t", &seconds(intent_duration_ms(plan)), "-f", "null", "-y"])
        .arg(null_output());
    Some(command)
}

#[cfg(test)]
pub(crate) fn build_composite_benchmark_command(
    ffmpeg_path: &Path,
    plan: &RenderPlan,
    filter_path: &Path,
) -> Command {
    let mut command = Command::new(ffmpeg_path);
    append_render_inputs(&mut command, plan);
    if plan.serial_bezier_filters {
        command.args(["-filter_complex_threads", "1"]);
    }
    command.arg("-filter_complex_script").arg(filter_path);
    match plan.intent {
        RenderIntent::Frame { at_ms } => {
            command.args([
                "-ss",
                &seconds(at_ms),
                "-frames:v",
                "1",
                "-map",
                "[video]",
                "-map",
                "[audio]",
                "-t",
                &seconds(intent_duration_ms(plan)),
            ]);
        }
        RenderIntent::Range {
            start_ms,
            end_ms,
            include_audio,
        } => {
            let duration = seconds(end_ms.saturating_sub(start_ms));
            command.args(["-ss", &seconds(start_ms), "-map", "[video]"]);
            if include_audio {
                command.args(["-map", "[audio]"]);
            }
            command.args(["-t", &duration]);
        }
        RenderIntent::Export => {
            command.args([
                "-map",
                "[video]",
                "-map",
                "[audio]",
                "-t",
                &seconds(plan.duration_ms),
            ]);
        }
    }
    command.args(["-f", "null", "-y"]).arg(null_output());
    command
}

#[cfg(test)]
fn intent_duration_ms(plan: &RenderPlan) -> u64 {
    match plan.intent {
        RenderIntent::Frame { .. } => 1_000_u64.div_ceil(u64::from(plan.fps)),
        RenderIntent::Range {
            start_ms, end_ms, ..
        } => end_ms.saturating_sub(start_ms),
        RenderIntent::Export => plan.duration_ms,
    }
}

#[cfg(test)]
pub(crate) fn run_to_completion(
    command: &mut Command,
    duration_ms: u64,
    mut on_progress: impl FnMut(RenderProgress),
) -> Result<(), CoreError> {
    run_command(command, duration_ms, &mut on_progress)
}

fn run_command(
    command: &mut Command,
    duration_ms: u64,
    on_progress: &mut dyn FnMut(RenderProgress),
) -> Result<(), CoreError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| CoreError::render_failure(SPAWN_STAGE, None, None))?;
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(CoreError::render_failure(SPAWN_STAGE, None, None));
    };
    let Some(stderr) = child.stderr.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(CoreError::render_failure(SPAWN_STAGE, None, None));
    };
    let stderr_reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
    let mut progress_error = false;
    for line in BufReader::new(stdout).lines() {
        let Ok(line) = line else {
            progress_error = true;
            let _ = child.kill();
            break;
        };
        if let Some(value) = line.strip_prefix("out_time_ms=")
            && let Ok(microseconds) = value.parse::<u64>()
        {
            let denominator = duration_ms.max(1) as f64;
            on_progress(RenderProgress {
                progress: (microseconds as f64 / 1_000.0 / denominator).clamp(0.0, 1.0),
            });
        }
    }
    let status = child.wait();
    let stderr = stderr_reader.join();
    let stderr = match stderr {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(_)) | Err(_) => {
            return Err(CoreError::render_failure(RENDER_STAGE, None, None));
        }
    };
    let status = status
        .map_err(|_| CoreError::render_failure(RENDER_STAGE, None, stderr_excerpt(&stderr)))?;
    if progress_error || !status.success() {
        return Err(CoreError::render_failure(
            RENDER_STAGE,
            status.code(),
            stderr_excerpt(&stderr),
        ));
    }
    on_progress(RenderProgress { progress: 1.0 });
    Ok(())
}

pub(crate) fn read_bounded_tail(mut reader: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut tail = Vec::with_capacity(limit);
    let mut buffer = [0_u8; 4_096];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if count >= limit {
            tail.clear();
            tail.extend_from_slice(&buffer[count - limit..count]);
            continue;
        }
        let excess = tail.len().saturating_add(count).saturating_sub(limit);
        if excess > 0 {
            tail.drain(..excess);
        }
        tail.extend_from_slice(&buffer[..count]);
    }
    Ok(tail)
}

pub(crate) fn stderr_excerpt(stderr: &[u8]) -> Option<String> {
    let excerpt = sanitize_stderr(&String::from_utf8_lossy(stderr));
    (!excerpt.trim().is_empty()).then_some(excerpt)
}

pub(crate) fn map_renderer_error(error: CoreError, stage: &str) -> CoreError {
    match error.code {
        ErrorCode::InternalError | ErrorCode::ProjectRecoveryFailed | ErrorCode::JobFailed => {
            CoreError::render_failure(stage, None, None)
        }
        _ => error,
    }
}

pub(crate) fn sanitize_stderr(stderr: &str) -> String {
    let sanitized = stderr
        .lines()
        .map(sanitize_stderr_line)
        .collect::<Vec<_>>()
        .join("\n");
    if sanitized.len() <= STDERR_EXCERPT_BYTES {
        return sanitized;
    }
    let mut start = sanitized.len() - STDERR_EXCERPT_BYTES;
    while !sanitized.is_char_boundary(start) {
        start += 1;
    }
    sanitized[start..].to_owned()
}

pub(crate) fn sanitize_stderr_line(line: &str) -> String {
    let mut sanitized = String::with_capacity(line.len());
    let mut cursor = 0;
    while let Some(relative_start) = find_absolute_path_start(&line[cursor..]) {
        let start = cursor + relative_start;
        sanitized.push_str(&line[cursor..start]);
        let quote = line[..start]
            .chars()
            .next_back()
            .filter(|character| matches!(character, '\'' | '"'));
        let path_tail = &line[start..];
        let end = quote
            .and_then(|quote| path_tail[1..].find(quote).map(|index| start + index + 1))
            .or_else(|| {
                path_tail[3.min(path_tail.len())..]
                    .find(": ")
                    .map(|index| start + 3.min(path_tail.len()) + index)
            })
            .unwrap_or(line.len());
        sanitized.push_str("[path]");
        cursor = end;
    }
    sanitized.push_str(&line[cursor..]);
    sanitized
}

pub(crate) fn find_absolute_path_start(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let windows = bytes.windows(3).position(|window| {
        window[0].is_ascii_alphabetic() && window[1] == b':' && matches!(window[2], b'\\' | b'/')
    });
    let posix = bytes.iter().enumerate().find_map(|(index, byte)| {
        if *byte != b'/' {
            return None;
        }
        let boundary = index == 0
            || bytes[index - 1].is_ascii_whitespace()
            || matches!(bytes[index - 1], b'=' | b'\'' | b'"' | b'(' | b'[');
        boundary.then_some(index)
    });
    windows.into_iter().chain(posix).min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_plan::MediaInputRequest;

    #[test]
    fn sampled_encoder_failure_has_safe_bounded_diagnostics_and_reaped_child() {
        let root = tempfile::tempdir().unwrap();
        let helper = root.path().join(if cfg!(windows) {
            "encoder.exe"
        } else {
            "encoder"
        });
        let compile = Command::new("rustc")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/failing_visual_encoder.rs"),
            )
            .arg("-o")
            .arg(&helper)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let output = root.path().join("partial.mkv");
        let mut produced = 0;
        let error = SystemProcessExecutor
            .prepare_visual_stream(&helper, &output, 20, 2, &mut |_| {
                produced += 1;
                Ok(vec![b'x'; 128])
            })
            .unwrap_err();
        assert_eq!(
            produced, 2,
            "fault must consume all input, not fail with BrokenPipe"
        );
        assert_eq!(error.failed_stage.as_deref(), Some("visual_prepare"));
        assert_eq!(error.ffmpeg_exit_code, Some(7));
        let excerpt = error.ffmpeg_stderr_excerpt.unwrap();
        assert!(
            excerpt.len() <= STDERR_EXCERPT_BYTES,
            "{} bytes",
            excerpt.len()
        );
        assert!(!excerpt.contains("private-review"), "{excerpt}");
        assert!(excerpt.contains("[path]"));
        assert!(excerpt.contains("é"));
        assert!(excerpt.ends_with("injected encoder failure"));
        // On Windows a live executable remains locked; successful removal also
        // verifies that the child was waited/reaped before returning the error.
        std::fs::remove_file(helper).unwrap();
    }

    #[derive(Debug)]
    struct FailingExecutor;
    impl ProcessExecutor for FailingExecutor {
        fn readiness(&self, _ffmpeg_path: &Path, _ffprobe_path: &Path) -> Result<(), CoreError> {
            Ok(())
        }
        fn probe(&self, _ffprobe_path: &Path, _path: &Path) -> Result<ProbeResult, CoreError> {
            Err(CoreError::new(ErrorCode::UnsupportedMedia, "injected"))
        }
        fn execute(
            &self,
            _ffmpeg_path: &Path,
            _plan: &RenderPlan,
            _filter_path: &Path,
            _output: &Path,
            _on_progress: &mut dyn FnMut(RenderProgress),
        ) -> Result<(), CoreError> {
            Err(CoreError::render_failure(SPAWN_STAGE, None, None))
        }
    }

    #[test]
    fn executor_outcomes_are_injectable_and_diagnostics_are_bounded() {
        let plan = RenderPlan {
            serial_bezier_filters: false,
            text_layout_fidelity: false,
            detail_fidelity: false,
            filter_graph: String::new(),
            width: 1,
            height: 1,
            fps: 1,
            duration_ms: 1,
            intent: RenderIntent::Export,
            media_inputs: vec![],
            media_paths: vec![],
        };
        let error = FailingExecutor
            .execute(
                Path::new("unused"),
                &plan,
                Path::new("filter"),
                Path::new("output"),
                &mut |_| {},
            )
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some(SPAWN_STAGE));

        let stderr = vec![b'x'; STDERR_TAIL_BYTES * 2];
        let tail = read_bounded_tail(std::io::Cursor::new(stderr), STDERR_TAIL_BYTES).unwrap();
        assert_eq!(tail.len(), STDERR_TAIL_BYTES);
        assert!(stderr_excerpt(&tail).unwrap().len() <= STDERR_EXCERPT_BYTES);
    }

    #[test]
    fn benchmark_commands_preserve_production_plan_inputs_bounds_and_graph() {
        let plan = RenderPlan {
            serial_bezier_filters: false,
            text_layout_fidelity: false,
            detail_fidelity: false,
            filter_graph: "[0:v]null[video];[1:a]anull[audio]".into(),
            width: 160,
            height: 90,
            fps: 10,
            duration_ms: 1_000,
            intent: RenderIntent::Range {
                start_ms: 100,
                end_ms: 900,
                include_audio: true,
            },
            media_inputs: vec![
                MediaInputRequest {
                    item_id: "image".into(),
                    asset_id: "asset-image".into(),
                    project_relative_path: "assets/image.png".into(),
                    media_type: MediaType::Image,
                    source_in_ms: 0,
                    duration_ms: 800,
                    input_index: 2,
                },
                MediaInputRequest {
                    item_id: "audio".into(),
                    asset_id: "asset-audio".into(),
                    project_relative_path: "assets/audio.wav".into(),
                    media_type: MediaType::Audio,
                    source_in_ms: 25,
                    duration_ms: 800,
                    input_index: 3,
                },
            ],
            media_paths: vec!["root/image.png".into(), "root/audio.wav".into()],
        };
        let filter_path = Path::new("work/filter.txt");
        let production = command_args(&build_render_command(
            Path::new("ffmpeg"),
            &plan,
            filter_path,
            Path::new("output.mp4"),
        ));
        let decode = command_args(
            &build_decode_benchmark_command(Path::new("ffmpeg"), &plan)
                .expect("media inputs require a decode workload"),
        );
        let composite = command_args(&build_composite_benchmark_command(
            Path::new("ffmpeg"),
            &plan,
            filter_path,
        ));

        for expected in ["root/image.png", "root/audio.wav", "0.025", "0.800"] {
            assert!(production.contains(&expected.to_owned()));
            assert!(decode.contains(&expected.to_owned()));
            assert!(composite.contains(&expected.to_owned()));
        }
        assert!(decode.windows(2).any(|pair| pair == ["-map", "2:v:0"]));
        assert!(decode.windows(2).any(|pair| pair == ["-map", "3:a:0"]));
        assert!(!decode.contains(&"-filter_complex_script".into()));
        assert!(
            composite
                .windows(2)
                .any(|pair| pair == ["-filter_complex_script", "work/filter.txt"])
        );
        assert!(composite.windows(2).any(|pair| pair == ["-ss", "0.100"]));
        assert!(composite.windows(2).any(|pair| pair == ["-t", "0.800"]));
        assert!(!composite.contains(&"libx264".into()));
    }

    fn command_args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn grid_range_fidelity_preserves_legacy_encoding() {
        let mut plan = RenderPlan {
            serial_bezier_filters: false,
            text_layout_fidelity: false,
            detail_fidelity: false,
            filter_graph: String::new(),
            width: 240,
            height: 120,
            fps: 24,
            duration_ms: 1000,
            intent: RenderIntent::Range {
                start_ms: 0,
                end_ms: 1000,
                include_audio: true,
            },
            media_inputs: vec![],
            media_paths: vec![],
        };
        let args = |p: &RenderPlan| {
            command_args(&build_render_command(
                Path::new("ffmpeg"),
                p,
                Path::new("filter"),
                Path::new("output"),
            ))
        };
        let legacy = args(&plan);
        assert!(!legacy.contains(&"-filter_complex_threads".to_owned()));
        for intent in [
            RenderIntent::Frame { at_ms: 250 },
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 1000,
                include_audio: true,
            },
            RenderIntent::Export,
        ] {
            plan.intent = intent;
            plan.serial_bezier_filters = true;
            assert!(
                args(&plan)
                    .windows(2)
                    .any(|pair| pair == ["-filter_complex_threads", "1"])
            );
        }
        plan.serial_bezier_filters = false;
        plan.intent = RenderIntent::Range {
            start_ms: 0,
            end_ms: 1000,
            include_audio: true,
        };
        assert!(legacy.windows(2).any(|v| v == ["-crf", "28"]));
        assert!(legacy.windows(2).any(|v| v == ["-preset", "veryfast"]));
        plan.detail_fidelity = true;
        let grid = args(&plan);
        assert!(grid.windows(2).any(|v| v == ["-crf", "23"]));
        assert!(grid.windows(2).any(|v| v == ["-preset", "medium"]));
        plan.intent = RenderIntent::Export;
        let grid_export = args(&plan);
        plan.detail_fidelity = false;
        assert_eq!(grid_export, args(&plan));
        plan.text_layout_fidelity = true;
        for intent in [
            RenderIntent::Export,
            RenderIntent::Range {
                start_ms: 0,
                end_ms: 1000,
                include_audio: true,
            },
        ] {
            plan.intent = intent;
            let advanced = args(&plan);
            assert!(advanced.windows(2).any(|v| v == ["-crf", "18"]));
            assert!(advanced.windows(2).any(|v| v == ["-preset", "medium"]));
        }
    }
}

#[cfg(all(test, unix))]
mod raster_source_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn executable(root: &Path, body: &str) -> std::path::PathBuf {
        let path = root.join("raster-port.sh");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }
    #[test]
    fn epic6_raster_port_requires_exact_rgba_and_propagates_exit_status() {
        let root = tempfile::tempdir().unwrap();
        let executor = SystemProcessExecutor;
        let path = executable(root.path(), "printf '\\001\\002\\003\\004'");
        assert_eq!(
            executor.raster_source(&path, "source", (1, 1)).unwrap(),
            [1, 2, 3, 4]
        );
        for body in [
            "printf '\\001\\002\\003'",
            "printf '\\001\\002\\003\\004\\005'",
            "printf '\\001\\002\\003\\004'; printf 'invalid raster source' >&2; exit 7",
        ] {
            let path = executable(root.path(), body);
            let error = executor.raster_source(&path, "source", (1, 1)).unwrap_err();
            assert_eq!(error.code, ErrorCode::FfmpegFailed);
        }
        // A very large producer is stopped after exactly one overflow byte;
        // neither stdout nor stderr may accumulate the complete child output.
        let path = executable(
            root.path(),
            "dd if=/dev/zero bs=65536 count=32 2>/dev/null; printf 'bad source\\n' >&2",
        );
        assert_eq!(
            executor
                .raster_source(&path, "source", (1, 1))
                .unwrap_err()
                .code,
            ErrorCode::FfmpegFailed
        );
    }
}
