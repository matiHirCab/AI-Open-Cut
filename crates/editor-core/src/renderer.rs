use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CoreError, Easing, ErrorCode, Keyframe, KeyframeProperty, KeyframeValue, MediaType, Project,
    TimelineItem,
};

#[derive(Clone, Debug)]
pub struct Renderer {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
    default_font_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderArtifact {
    pub relative_path: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderProgress {
    pub progress: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ExportOptions<'a> {
    pub output: &'a Path,
    pub width: u32,
    pub height: u32,
    pub overwrite: bool,
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

impl Renderer {
    pub fn new(
        ffmpeg_path: impl Into<PathBuf>,
        ffprobe_path: impl Into<PathBuf>,
        default_font_path: Option<PathBuf>,
    ) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            ffprobe_path: ffprobe_path.into(),
            default_font_path,
        }
    }

    pub fn readiness(&self) -> Result<(), CoreError> {
        let output = Command::new(&self.ffmpeg_path)
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
        let status = Command::new(&self.ffprobe_path)
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

    pub fn probe(&self, path: &Path) -> Result<ProbeResult, CoreError> {
        let output = Command::new(&self.ffprobe_path)
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

    pub fn render_preview(
        &self,
        project: &Project,
        project_dir: &Path,
        time_ms: u64,
    ) -> Result<RenderArtifact, CoreError> {
        if time_ms > project.duration_ms() {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "preview time exceeds project duration",
            ));
        }
        let file_name = format!("preview-{}.png", Uuid::new_v4());
        let output = project_dir.join("previews").join(&file_name);
        let temporary = temporary_output(output.parent().unwrap_or(project_dir), "png");
        let mut command = self.build_command(
            project,
            project_dir,
            project.settings.width,
            project.settings.height,
        )?;
        command
            .args([
                "-ss",
                &seconds(time_ms),
                "-frames:v",
                "1",
                "-map",
                "[video]",
            ])
            .arg(&temporary)
            .args(["-map", "[audio]", "-f", "null"])
            .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
        if let Err(error) = run_to_completion(&mut command, project.duration_ms(), |_| {}) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        publish_output(&temporary, &output, false)?;
        artifact(&output, format!("previews/{file_name}"), "image/png")
    }

    pub fn export_video(
        &self,
        project: &Project,
        project_dir: &Path,
        options: ExportOptions<'_>,
        mut on_progress: impl FnMut(RenderProgress),
    ) -> Result<RenderArtifact, CoreError> {
        if options.output.exists() && !options.overwrite {
            return Err(CoreError::new(
                ErrorCode::ExportExists,
                "export already exists; pass overwrite=true only with explicit permission",
            ));
        }
        let temporary = temporary_output(
            options.output.parent().unwrap_or_else(|| Path::new(".")),
            "mp4",
        );
        let mut command =
            self.build_command(project, project_dir, options.width, options.height)?;
        command.args([
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
            &seconds(project.duration_ms()),
        ]);
        command.arg("-y").arg(&temporary);
        if let Err(error) = run_to_completion(&mut command, project.duration_ms(), |progress| {
            on_progress(progress)
        }) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        publish_output(&temporary, options.output, options.overwrite)?;
        artifact(
            options.output,
            options
                .output
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("export.mp4")
                .to_owned(),
            "video/mp4",
        )
    }

    fn build_command(
        &self,
        project: &Project,
        project_dir: &Path,
        width: u32,
        height: u32,
    ) -> Result<Command, CoreError> {
        let duration_ms = project.duration_ms().max(1);
        let mut command = Command::new(&self.ffmpeg_path);
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
                width,
                height,
                project.settings.fps,
                seconds(duration_ms)
            ),
            "-f",
            "lavfi",
            "-i",
            &format!("anullsrc=r=48000:cl=stereo:d={}", seconds(duration_ms)),
        ]);

        let asset_by_id = project
            .assets
            .iter()
            .map(|asset| (asset.id.as_str(), asset))
            .collect::<HashMap<_, _>>();
        let mut input_indexes = HashMap::new();
        let mut next_input = 2_usize;
        for item in project
            .tracks
            .iter()
            .filter(|track| !track.hidden)
            .flat_map(|track| &track.items)
            .filter(|item| !item.hidden())
        {
            let TimelineItem::Media(media) = item else {
                continue;
            };
            if input_indexes.contains_key(&media.id) {
                continue;
            }
            let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetNotFound,
                    "timeline references a missing asset",
                )
            })?;
            let path = resolve_project_asset(project_dir, Path::new(&asset.project_relative_path))?;
            if asset.media_type == MediaType::Image {
                command.args(["-loop", "1", "-t", &seconds(media.duration_ms), "-i"]);
            } else {
                command.args([
                    "-ss",
                    &seconds(media.source_in_ms),
                    "-t",
                    &seconds(media.duration_ms),
                    "-i",
                ]);
            }
            command.arg(path);
            input_indexes.insert(media.id.clone(), next_input);
            next_input += 1;
        }

        let filter_path = project_dir.join(format!("filter-{}.txt", Uuid::new_v4()));
        let filter = self.build_filter(project, &asset_by_id, &input_indexes, width, height)?;
        let mut file = File::create(&filter_path)
            .map_err(|error| CoreError::io("cannot create FFmpeg filter script", error))?;
        file.write_all(filter.as_bytes())
            .map_err(|error| CoreError::io("cannot write FFmpeg filter script", error))?;
        command.arg("-filter_complex_script").arg(filter_path);
        Ok(command)
    }

    fn build_filter(
        &self,
        project: &Project,
        asset_by_id: &HashMap<&str, &crate::Asset>,
        input_indexes: &HashMap<String, usize>,
        width: u32,
        height: u32,
    ) -> Result<String, CoreError> {
        let mut filters = vec!["[0:v]format=yuv420p[base0]".to_owned()];
        let mut current_video = "base0".to_owned();
        let mut visual_count = 0_usize;
        let mut audio_labels = vec!["[1:a]".to_owned()];
        let transitions = project
            .tracks
            .iter()
            .filter(|track| !track.hidden)
            .flat_map(|track| &track.items)
            .filter_map(|item| match item {
                TimelineItem::Transition(value) if !value.hidden => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();

        for track in &project.tracks {
            if track.hidden {
                continue;
            }
            for item in &track.items {
                if item.hidden() {
                    continue;
                }
                match item {
                    TimelineItem::Media(media) => {
                        let asset = asset_by_id.get(media.asset_id.as_str()).ok_or_else(|| {
                            CoreError::new(
                                ErrorCode::AssetNotFound,
                                "timeline references a missing asset",
                            )
                        })?;
                        let input = input_indexes.get(&media.id).ok_or_else(|| {
                            CoreError::new(
                                ErrorCode::InternalError,
                                "renderer input mapping is missing",
                            )
                        })?;
                        if asset.media_type != MediaType::Audio {
                            visual_count += 1;
                            let prepared = format!("visual{visual_count}");
                            let composited = format!("base{visual_count}");
                            let scale = scalar_expression(
                                &media.keyframes,
                                KeyframeProperty::Scale,
                                media.transform.scale,
                                media.start_ms,
                            );
                            let x = position_expression(
                                &media.keyframes,
                                true,
                                media.transform.position_x,
                                media.start_ms,
                            );
                            let y = position_expression(
                                &media.keyframes,
                                false,
                                media.transform.position_y,
                                media.start_ms,
                            );
                            let opacity = scalar_expression(
                                &media.keyframes,
                                KeyframeProperty::Opacity,
                                media.transform.opacity,
                                media.start_ms,
                            );
                            let fade = transition_filters(&media.id, &transitions, media.start_ms);
                            filters.push(format!(
                            "[{input}:v]setpts=PTS-STARTPTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,format=rgba,colorchannelmixer=aa='{opacity}'{fade}[{prepared}]",
                            seconds(media.start_ms)
                        ));
                            filters.push(format!(
                            "[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]",
                            seconds(media.start_ms),
                            seconds(media.start_ms.saturating_add(media.duration_ms))
                        ));
                            current_video = composited;
                        }
                        if asset.has_audio && !track.muted && !media.audio.muted {
                            let label = format!("audio{}", audio_labels.len());
                            let mut chain = format!(
                                "[{input}:a]atrim=duration={},asetpts=PTS-STARTPTS+{}/TB,volume={}",
                                seconds(media.duration_ms),
                                seconds(media.start_ms),
                                media.audio.volume
                            );
                            if media.audio.fade_in_ms > 0 {
                                chain.push_str(&format!(
                                    ",afade=t=in:st=0:d={}",
                                    seconds(media.audio.fade_in_ms)
                                ));
                            }
                            if media.audio.fade_out_ms > 0
                                && media.audio.fade_out_ms < media.duration_ms
                            {
                                chain.push_str(&format!(
                                    ",afade=t=out:st={}:d={}",
                                    seconds(media.duration_ms - media.audio.fade_out_ms),
                                    seconds(media.audio.fade_out_ms)
                                ));
                            }
                            chain.push_str(&format!("[{label}]"));
                            filters.push(chain);
                            audio_labels.push(format!("[{label}]"));
                        }
                    }
                    TimelineItem::Text(text) => {
                        visual_count += 1;
                        let composited = format!("base{visual_count}");
                        let x = position_expression(
                            &text.keyframes,
                            true,
                            text.transform.position_x,
                            text.start_ms,
                        );
                        let y = position_expression(
                            &text.keyframes,
                            false,
                            text.transform.position_y,
                            text.start_ms,
                        );
                        let font = self
                            .default_font_path
                            .as_ref()
                            .map(|path| {
                                format!("fontfile='{}':", escape_filter(&path.to_string_lossy()))
                            })
                            .unwrap_or_else(|| {
                                text.font_family
                                    .as_ref()
                                    .map(|family| format!("font='{}':", escape_filter(family)))
                                    .unwrap_or_default()
                            });
                        filters.push(format!(
                        "[{current_video}]drawtext={font}text='{}':fontsize={}:fontcolor={}:x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]",
                        escape_filter(&text.text),
                        text.font_size,
                        text.color,
                        seconds(text.start_ms),
                        seconds(text.start_ms.saturating_add(text.duration_ms))
                    ));
                        current_video = composited;
                    }
                    TimelineItem::Caption(caption) => {
                        visual_count += 1;
                        let composited = format!("base{visual_count}");
                        let font = self
                            .default_font_path
                            .as_ref()
                            .map(|path| {
                                format!("fontfile='{}':", escape_filter(&path.to_string_lossy()))
                            })
                            .unwrap_or_default();
                        filters.push(format!(
                        "[{current_video}]drawtext={font}text='{}':fontsize={}:fontcolor={}:box=1:boxcolor={}@0.75:boxborderw=12:x='(w-text_w)/2':y='h-text_h-{}':enable='between(t,{},{})'[{composited}]",
                        escape_filter(&caption.text),
                        caption.style.font_size,
                        caption.style.color,
                        caption.style.background_color,
                        caption.style.bottom_margin_px,
                        seconds(caption.start_ms),
                        seconds(caption.start_ms.saturating_add(caption.duration_ms))
                    ));
                        current_video = composited;
                    }
                    TimelineItem::Transition(_) => {}
                }
            }
        }
        filters.push(format!(
            "[{current_video}]scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]"
        ));
        filters.push(format!(
            "{}amix=inputs={}:duration=longest:normalize=0[audio]",
            audio_labels.join(""),
            audio_labels.len()
        ));
        Ok(filters.join(";\n"))
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

fn temporary_output(parent: &Path, extension: &str) -> PathBuf {
    let request_id = env::var("OPENCUT_REQUEST_ID")
        .ok()
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '-')
        })
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    parent.join(format!(".opencut-{request_id}.{extension}"))
}

fn publish_output(temporary: &Path, output: &Path, overwrite: bool) -> Result<(), CoreError> {
    if output.exists() {
        if !overwrite {
            let _ = std::fs::remove_file(temporary);
            return Err(CoreError::new(
                ErrorCode::ExportExists,
                "export already exists; pass overwrite=true only with explicit permission",
            ));
        }
        std::fs::remove_file(output)
            .map_err(|error| CoreError::io("cannot replace existing output", error))?;
    }
    std::fs::rename(temporary, output).map_err(|error| {
        let _ = std::fs::remove_file(temporary);
        CoreError::io("cannot publish rendered output", error)
    })
}

fn resolve_project_asset(project_dir: &Path, relative: &Path) -> Result<PathBuf, CoreError> {
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset path is not allowed",
        ));
    }
    let root = project_dir
        .canonicalize()
        .map_err(|error| CoreError::io("cannot resolve project directory", error))?;
    let resolved = project_dir
        .join(relative)
        .canonicalize()
        .map_err(|error| CoreError::io("cannot resolve project asset", error))?;
    if !resolved.starts_with(root) {
        return Err(CoreError::new(
            ErrorCode::PathNotAllowed,
            "project asset escapes the project directory",
        ));
    }
    Ok(resolved)
}

fn transition_filters(
    item_id: &str,
    transitions: &[&crate::TransitionItem],
    item_start_ms: u64,
) -> String {
    let mut result = String::new();
    for transition in transitions {
        if transition.from_item_id == item_id {
            let local_start = transition.start_ms.saturating_sub(item_start_ms);
            result.push_str(&format!(
                ",fade=t=out:st={}:d={}:alpha=1",
                seconds(local_start),
                seconds(transition.duration_ms)
            ));
        }
        if transition.to_item_id.as_deref() == Some(item_id) {
            let local_start = transition.start_ms.saturating_sub(item_start_ms);
            result.push_str(&format!(
                ",fade=t=in:st={}:d={}:alpha=1",
                seconds(local_start),
                seconds(transition.duration_ms)
            ));
        }
    }
    result
}

fn scalar_expression(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    default: f64,
    item_start_ms: u64,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| {
            if keyframe.property != property {
                return None;
            }
            let KeyframeValue::Scalar { value } = keyframe.value else {
                return None;
            };
            Some((keyframe.time_ms, value, keyframe.easing))
        })
        .collect::<Vec<_>>();
    piecewise_expression(&values, default, item_start_ms)
}

fn position_expression(
    keyframes: &[Keyframe],
    x_axis: bool,
    default: f64,
    item_start_ms: u64,
) -> String {
    let values = keyframes
        .iter()
        .filter_map(|keyframe| {
            if keyframe.property != KeyframeProperty::Position {
                return None;
            }
            let KeyframeValue::Position { x, y } = keyframe.value else {
                return None;
            };
            Some((
                keyframe.time_ms,
                if x_axis { x } else { y },
                keyframe.easing,
            ))
        })
        .collect::<Vec<_>>();
    piecewise_expression(&values, default, item_start_ms)
}

fn piecewise_expression(values: &[(u64, f64, Easing)], default: f64, item_start_ms: u64) -> String {
    if values.is_empty() {
        return format_number(default);
    }
    let mut expression = format_number(values.last().map_or(default, |value| value.1));
    for pair in values.windows(2).rev() {
        let (start_time, start_value, easing) = pair[0];
        let (end_time, end_value, _) = pair[1];
        let global_start = seconds(item_start_ms.saturating_add(start_time));
        let global_end = seconds(item_start_ms.saturating_add(end_time));
        let span = seconds(end_time.saturating_sub(start_time).max(1));
        let progress = format!("(t-{global_start})/{span}");
        let eased = easing_expression(&progress, easing);
        let interpolated = format!(
            "{}+({}-{})*({eased})",
            format_number(start_value),
            format_number(end_value),
            format_number(start_value)
        );
        expression = format!("if(lt(t,{global_end}),{interpolated},{expression})");
    }
    let first_time = seconds(item_start_ms.saturating_add(values[0].0));
    format!(
        "if(lt(t,{first_time}),{},{expression})",
        format_number(values[0].1)
    )
}

fn easing_expression(progress: &str, easing: Easing) -> String {
    match easing {
        Easing::Hold => "0".into(),
        Easing::Linear => progress.into(),
        Easing::EaseIn => format!("({progress})*({progress})"),
        Easing::EaseOut => format!("1-(1-({progress}))*(1-({progress}))"),
        Easing::EaseInOut => format!(
            "if(lt(({progress}),0.5),2*({progress})*({progress}),1-pow(-2*({progress})+2,2)/2)"
        ),
    }
}

fn escape_filter(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "\\'")
        .replace('%', "\\%")
        .replace('\n', "\\n")
}

fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1_000.0)
}

fn format_number(value: f64) -> String {
    format!("{value:.6}")
}

fn artifact(
    path: &Path,
    relative_path: String,
    mime_type: &str,
) -> Result<RenderArtifact, CoreError> {
    let size_bytes = path
        .metadata()
        .map_err(|error| CoreError::io("cannot inspect render output", error))?
        .len();
    if size_bytes == 0 {
        return Err(CoreError::new(
            ErrorCode::JobFailed,
            "renderer produced an empty output",
        ));
    }
    Ok(RenderArtifact {
        relative_path,
        mime_type: mime_type.into(),
        size_bytes,
    })
}

fn run_to_completion(
    command: &mut Command,
    duration_ms: u64,
    mut on_progress: impl FnMut(RenderProgress),
) -> Result<(), CoreError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        CoreError::new(
            ErrorCode::DependencyUnavailable,
            format!("cannot start FFmpeg: {error}"),
        )
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "cannot read FFmpeg progress"))?;
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|error| CoreError::io("cannot read FFmpeg progress", error))?;
        if let Some(value) = line.strip_prefix("out_time_ms=")
            && let Ok(microseconds) = value.parse::<u64>()
        {
            let denominator = duration_ms.max(1) as f64;
            on_progress(RenderProgress {
                progress: (microseconds as f64 / 1_000.0 / denominator).clamp(0.0, 1.0),
            });
        }
    }
    let output = child
        .wait_with_output()
        .map_err(|error| CoreError::io("cannot wait for FFmpeg", error))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let summary = stderr.lines().last().unwrap_or("FFmpeg failed");
        return Err(CoreError::new(
            ErrorCode::JobFailed,
            format!("FFmpeg render failed: {summary}"),
        ));
    }
    on_progress(RenderProgress { progress: 1.0 });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaptionItem, CaptionSource, CaptionStyle, PROJECT_SCHEMA_VERSION, ProjectSettings,
        TimelineItem, Track, TrackType,
    };
    use tempfile::tempdir;

    #[test]
    fn keyframe_expression_contains_no_shell_syntax() {
        let expression = piecewise_expression(
            &[(0, 1.0, Easing::Linear), (1_000, 2.0, Easing::EaseOut)],
            1.0,
            500,
        );
        assert!(expression.contains("if(lt(t,"));
        assert!(!expression.contains('$'));
        assert!(!expression.contains('`'));
    }

    #[test]
    fn filter_text_is_escaped() {
        assert_eq!(escape_filter("it's: 100%"), "it\\'s\\: 100\\%");
    }

    #[test]
    fn export_collision_requires_explicit_overwrite() {
        let root = tempdir().unwrap();
        let output = root.path().join("existing.mp4");
        std::fs::write(&output, b"existing").unwrap();
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "test".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        };
        let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
        let error = renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 1_920,
                    height: 1_080,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::ExportExists);
        assert_eq!(std::fs::read(&output).unwrap(), b"existing");
    }

    #[test]
    fn captions_render_bottom_centered_and_hidden_tracks_are_excluded() {
        let mut project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "captions".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![Track {
                id: "caption-track".into(),
                name: "Captions".into(),
                track_type: TrackType::Caption,
                locked: false,
                hidden: false,
                muted: false,
                items: vec![TimelineItem::Caption(CaptionItem {
                    id: "caption".into(),
                    text: "It's: 100%".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    style: CaptionStyle::default(),
                    source: CaptionSource {
                        asset_id: "asset".into(),
                        provider_id: "test".into(),
                        model_id: "small".into(),
                        model_version: None,
                        language: "en".into(),
                        generated_at_ms: 1,
                        original_text: "It's: 100%".into(),
                        confidence: None,
                        words: vec![],
                    },
                    hidden: false,
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        let filter = renderer
            .build_filter(&project, &HashMap::new(), &HashMap::new(), 1920, 1080)
            .unwrap();
        assert!(filter.contains("x='(w-text_w)/2':y='h-text_h-64'"));
        assert!(filter.contains("It\\'s\\: 100\\%"));
        project.tracks[0].hidden = true;
        let hidden = renderer
            .build_filter(&project, &HashMap::new(), &HashMap::new(), 1920, 1080)
            .unwrap();
        assert!(!hidden.contains("drawtext"));
    }
}
