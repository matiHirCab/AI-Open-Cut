//! FFmpeg and FFprobe process execution owner.

use std::{
    fmt::Debug,
    io::{BufRead, BufReader, Read},
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
    for required in [" overlay ", " drawtext ", " amix "] {
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
    fn readiness(&self, ffmpeg_path: &Path, ffprobe_path: &Path) -> Result<(), CoreError>;
    fn probe(&self, ffprobe_path: &Path, path: &Path) -> Result<ProbeResult, CoreError>;
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
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-progress",
        "pipe:1",
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
    command.arg("-filter_complex_script").arg(filter_path);
    match plan.intent {
        RenderIntent::Frame { at_ms } => {
            command
                .args(["-ss", &seconds(at_ms), "-frames:v", "1", "-map", "[video]"])
                .arg(output)
                .args(["-map", "[audio]", "-f", "null"])
                .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
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
                    "veryfast",
                    "-crf",
                    "28",
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
                    .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
            }
        }
        RenderIntent::Export => {
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
}
