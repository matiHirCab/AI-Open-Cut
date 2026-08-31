use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CoreError, Easing, ErrorCode, Keyframe, KeyframeProperty, KeyframeValue, MediaType, Project,
    TimelineItem, animation::positive_scalar_ranges,
};

const GRAPH_BUILD_STAGE: &str = "graph_build";
const SPAWN_STAGE: &str = "spawn";
const RENDER_STAGE: &str = "render";
const PUBLISH_STAGE: &str = "publish";
const STDERR_TAIL_BYTES: usize = 16_384;
const STDERR_EXCERPT_BYTES: usize = 4_096;

#[derive(Clone, Debug)]
pub struct Renderer {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
    default_font_path: Option<PathBuf>,
    font_roots: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderArtifact {
    pub relative_path: String,
    pub mime_type: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
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

#[derive(Clone, Copy, Debug)]
pub struct PreviewRangeOptions {
    pub start_ms: u64,
    pub end_ms: u64,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub include_audio: bool,
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

struct BuiltCommand {
    command: Command,
    _workspace: RenderWorkspace,
    warnings: Vec<String>,
}

struct RenderWorkspace {
    path: PathBuf,
}

struct PreparedText {
    file_path: PathBuf,
    font_path: Option<PathBuf>,
    layer_width: u32,
    layer_height: u32,
    canvas_width: u32,
    canvas_height: u32,
    text_x: u32,
    text_y: u32,
}

struct FilterContext<'a> {
    asset_by_id: &'a HashMap<&'a str, &'a crate::Asset>,
    input_indexes: &'a HashMap<String, usize>,
    text_layers: &'a HashMap<String, PreparedText>,
    width: u32,
    height: u32,
    fps: u32,
}

impl Drop for RenderWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
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
            font_roots: vec![],
        }
    }

    pub fn with_font_roots(mut self, roots: impl IntoIterator<Item = PathBuf>) -> Self {
        self.font_roots = roots.into_iter().collect();
        self
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
        let mut built = self.build_command(
            project,
            project_dir,
            project.settings.width,
            project.settings.height,
            project.settings.fps,
        )?;
        built
            .command
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
        if let Err(error) = run_to_completion(&mut built.command, project.duration_ms(), |_| {}) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        publish_output(&temporary, &output, false)?;
        artifact(
            &output,
            format!("previews/{file_name}"),
            "image/png",
            built.warnings.clone(),
        )
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
        let mut built = self.build_command(
            project,
            project_dir,
            options.width,
            options.height,
            project.settings.fps,
        )?;
        built.command.args([
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
        built.command.arg("-y").arg(&temporary);
        if let Err(error) =
            run_to_completion(&mut built.command, project.duration_ms(), |progress| {
                on_progress(progress)
            })
        {
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
            built.warnings.clone(),
        )
    }

    pub fn render_preview_range(
        &self,
        project: &Project,
        project_dir: &Path,
        options: PreviewRangeOptions,
        on_progress: impl FnMut(RenderProgress),
    ) -> Result<RenderArtifact, CoreError> {
        if options.start_ms >= options.end_ms
            || options.end_ms > project.duration_ms()
            || options.width == 0
            || options.height == 0
            || !(1..=120).contains(&options.fps)
        {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "preview range options are invalid",
            ));
        }
        let file_name = format!("preview-range-{}.mp4", Uuid::new_v4());
        let output = project_dir.join("previews").join(&file_name);
        let temporary = temporary_output(output.parent().unwrap_or(project_dir), "mp4");
        let mut built = self.build_command(
            project,
            project_dir,
            options.width,
            options.height,
            options.fps,
        )?;
        configure_preview_range_outputs(&mut built.command, options, &temporary);
        if let Err(error) = run_to_completion(
            &mut built.command,
            options.end_ms - options.start_ms,
            on_progress,
        ) {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        publish_output(&temporary, &output, false)?;
        artifact(
            &output,
            format!("previews/{file_name}"),
            "video/mp4",
            built.warnings.clone(),
        )
    }

    fn build_command(
        &self,
        project: &Project,
        project_dir: &Path,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<BuiltCommand, CoreError> {
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
                fps,
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

        let workspace = RenderWorkspace::create(project_dir)?;
        let mut warnings = Vec::new();
        let text_layers = self.prepare_text_layers(project, workspace.path(), &mut warnings)?;
        let filter_path = workspace.path().join("filter.txt");
        let filter = self
            .build_filter(
                project,
                FilterContext {
                    asset_by_id: &asset_by_id,
                    input_indexes: &input_indexes,
                    text_layers: &text_layers,
                    width,
                    height,
                    fps,
                },
                &mut warnings,
            )
            .map_err(|error| map_renderer_error(error, GRAPH_BUILD_STAGE))?;
        let mut file = File::create(&filter_path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        file.write_all(filter.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        command.arg("-filter_complex_script").arg(&filter_path);
        Ok(BuiltCommand {
            command,
            _workspace: workspace,
            warnings,
        })
    }

    fn prepare_text_layers(
        &self,
        project: &Project,
        workspace: &Path,
        warnings: &mut Vec<String>,
    ) -> Result<HashMap<String, PreparedText>, CoreError> {
        let mut result = HashMap::new();
        for text in project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .filter_map(|item| match item {
                TimelineItem::Text(text) if !text.hidden => Some(text),
                _ => None,
            })
        {
            let path = workspace.join(format!("text-{}.txt", text.id));
            let font_path = self.resolve_text_font(text, warnings);
            let content = wrap_text(
                &text.text,
                text.style.wrap_width_px,
                text.font_size,
                font_path.as_deref(),
            );
            std::fs::write(&path, content.as_bytes())
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            let metrics = measure_text_block(&content, text.font_size, font_path.as_deref());
            let outline = text.style.outline_width_px;
            let shadow_left = text.style.shadow.offset_x.unsigned_abs()
                * u32::from(text.style.shadow.offset_x < 0);
            let shadow_right = text.style.shadow.offset_x.max(0) as u32;
            let shadow_top = text.style.shadow.offset_y.unsigned_abs()
                * u32::from(text.style.shadow.offset_y < 0);
            let shadow_bottom = text.style.shadow.offset_y.max(0) as u32;
            let text_x = text
                .style
                .padding
                .left
                .saturating_add(outline)
                .saturating_add(shadow_left);
            let text_y = text
                .style
                .padding
                .top
                .saturating_add(outline)
                .saturating_add(shadow_top);
            let layer_width = text_x
                .saturating_add(metrics.width.ceil() as u32)
                .saturating_add(text.style.padding.right)
                .saturating_add(outline)
                .saturating_add(shadow_right)
                .saturating_add(2)
                .max(1);
            let line_spacing = text
                .style
                .line_spacing_px
                .saturating_mul(metrics.line_count.saturating_sub(1) as i32);
            let text_height = (metrics.height + f64::from(line_spacing)).max(1.0);
            let layer_height = text_y
                .saturating_add(text_height.ceil() as u32)
                .saturating_add(text.style.padding.bottom)
                .saturating_add(outline)
                .saturating_add(shadow_bottom)
                .saturating_add(2)
                .max(1);
            let maximum_scale = text
                .keyframes
                .iter()
                .filter_map(|keyframe| match (keyframe.property, &keyframe.value) {
                    (KeyframeProperty::Scale, KeyframeValue::Scalar { value }) => Some(*value),
                    _ => None,
                })
                .fold(text.transform.scale, f64::max)
                .max(0.01);
            let canvas_width = ((f64::from(layer_width) * maximum_scale).ceil() as u32)
                .saturating_add(2)
                .max(1);
            let canvas_height = ((f64::from(layer_height) * maximum_scale).ceil() as u32)
                .saturating_add(2)
                .max(1);
            result.insert(
                text.id.clone(),
                PreparedText {
                    file_path: path,
                    font_path,
                    layer_width,
                    layer_height,
                    canvas_width,
                    canvas_height,
                    text_x,
                    text_y,
                },
            );
        }
        Ok(result)
    }

    fn resolve_text_font(
        &self,
        text: &crate::TextItem,
        warnings: &mut Vec<String>,
    ) -> Option<PathBuf> {
        if let Some(requested) = text.font_path.as_deref() {
            let requested = PathBuf::from(requested);
            let candidates = if requested.is_absolute() {
                vec![requested]
            } else {
                self.font_roots
                    .iter()
                    .map(|root| root.join(&requested))
                    .collect()
            };
            for candidate in candidates {
                if let Ok(resolved) = candidate.canonicalize()
                    && resolved.is_file()
                    && self
                        .font_roots
                        .iter()
                        .filter_map(|root| root.canonicalize().ok())
                        .any(|root| resolved.starts_with(root))
                {
                    return Some(resolved);
                }
            }
            warnings.push(format!(
                "Text item {} requested a font path that could not be resolved; using fallback",
                text.id
            ));
        }
        if let Some(family) = text.font_family.as_deref() {
            let needle = family.to_lowercase().replace([' ', '-', '_'], "");
            for root in &self.font_roots {
                if let Some(path) = find_font_file(root, &needle) {
                    return Some(path);
                }
            }
            warnings.push(format!("Text item {} requested font family {family:?} that could not be resolved; using fallback", text.id));
        }
        self.default_font_path.clone()
    }

    fn build_filter(
        &self,
        project: &Project,
        context: FilterContext<'_>,
        _warnings: &mut Vec<String>,
    ) -> Result<String, CoreError> {
        let FilterContext {
            asset_by_id,
            input_indexes,
            text_layers,
            width,
            height,
            fps,
        } = context;
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
        let voiceover_intervals = audible_voiceover_intervals(project, asset_by_id);

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
                            let opacity = scalar_expression_for(
                                &media.keyframes,
                                KeyframeProperty::Opacity,
                                media.transform.opacity,
                                media.start_ms,
                                "T",
                            );
                            let fade = transition_filters(&media.id, &transitions, media.start_ms);
                            filters.push(format!(
                            "[{input}:v]setpts=PTS-STARTPTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{fade}[{prepared}]",
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
                            let automation = scalar_expression(
                                &media.keyframes,
                                KeyframeProperty::Volume,
                                1.0,
                                0,
                            );
                            let volume =
                                format!("({})*({automation})", format_number(media.audio.volume));
                            let ducking = ducking_expression(track, &voiceover_intervals);
                            let mut chain = format!(
                                "[{input}:a]atrim=duration={},asetpts=PTS-STARTPTS,volume='{volume}':eval=frame",
                                seconds(media.duration_ms),
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
                            chain.push_str(&format!(
                                ",asetpts=PTS+{}/TB,volume='{ducking}':eval=frame",
                                seconds(media.start_ms)
                            ));
                            chain.push_str(&format!("[{label}]"));
                            filters.push(chain);
                            audio_labels.push(format!("[{label}]"));
                        }
                    }
                    TimelineItem::Text(text) => {
                        visual_count += 1;
                        let prepared = format!("visual{visual_count}");
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
                        let scale = scalar_expression(
                            &text.keyframes,
                            KeyframeProperty::Scale,
                            text.transform.scale,
                            text.start_ms,
                        );
                        let opacity = scalar_expression_for(
                            &text.keyframes,
                            KeyframeProperty::Opacity,
                            text.transform.opacity,
                            text.start_ms,
                            "T",
                        );
                        let prepared_text = text_layers.get(&text.id).ok_or_else(|| {
                            CoreError::new(
                                ErrorCode::InternalError,
                                "renderer text file is missing",
                            )
                        })?;
                        let font = prepared_text
                            .font_path
                            .as_ref()
                            .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
                            .unwrap_or_default();
                        let (x, y) = anchored_layer_position(&x, &y, text.style.anchor);
                        let alignment = match text.style.alignment {
                            crate::TextAlignment::Left => "L",
                            crate::TextAlignment::Center => "C",
                            crate::TextAlignment::Right => "R",
                        };
                        let padding = &text.style.padding;
                        let (pad_x, pad_y) = text_layer_padding(text.style.anchor);
                        let transition = transition_filters(&text.id, &transitions, text.start_ms);
                        filters.push(format!(
                            "color=c=black@0.0:s={}x{}:r={fps}:d={},format=rgba,drawtext={font}textfile='{}':expansion=none:fontsize={}:fontcolor={}:borderw={}:bordercolor={}:shadowx={}:shadowy={}:shadowcolor={}@{}:box=1:boxcolor={}@{}:boxborderw={}|{}|{}|{}:line_spacing={}:text_align={alignment}:x={}:y={},scale=w='iw*({scale})':h='ih*({scale})':eval=frame,pad={}:{}:{pad_x}:{pad_y}:color=black@0:eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]",
                            prepared_text.layer_width,
                            prepared_text.layer_height,
                            seconds(project.duration_ms().max(1)),
                            escape_filter_path(&prepared_text.file_path),
                            text.font_size,
                            text.color,
                            text.style.outline_width_px,
                            text.style.outline_color,
                            text.style.shadow.offset_x,
                            text.style.shadow.offset_y,
                            text.style.shadow.color,
                            text.style.shadow.opacity,
                            text.style.background_color,
                            text.style.background_opacity,
                            padding.top, padding.right, padding.bottom, padding.left,
                            text.style.line_spacing_px,
                            prepared_text.text_x,
                            prepared_text.text_y,
                            prepared_text.canvas_width,
                            prepared_text.canvas_height,
                        ));
                        filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(text.start_ms), seconds(text.start_ms.saturating_add(text.duration_ms))));
                        current_video = composited;
                    }
                    TimelineItem::SolidColor(shape) => {
                        visual_count += 1;
                        let prepared = format!("visual{visual_count}");
                        let composited = format!("base{visual_count}");
                        let scale = scalar_expression(
                            &shape.keyframes,
                            KeyframeProperty::Scale,
                            shape.transform.scale,
                            shape.start_ms,
                        );
                        let opacity = scalar_expression_for(
                            &shape.keyframes,
                            KeyframeProperty::Opacity,
                            shape.transform.opacity,
                            shape.start_ms,
                            "T",
                        );
                        let x = position_expression(
                            &shape.keyframes,
                            true,
                            shape.transform.position_x,
                            shape.start_ms,
                        );
                        let y = position_expression(
                            &shape.keyframes,
                            false,
                            shape.transform.position_y,
                            shape.start_ms,
                        );
                        let transition =
                            transition_filters(&shape.id, &transitions, shape.start_ms);
                        filters.push(format!("color=c={}:s={width}x{height}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(&shape.color), seconds(shape.duration_ms), seconds(shape.start_ms)));
                        filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(shape.start_ms), seconds(shape.start_ms.saturating_add(shape.duration_ms))));
                        current_video = composited;
                    }
                    TimelineItem::Rectangle(shape) => {
                        visual_count += 1;
                        let prepared = format!("visual{visual_count}");
                        let composited = format!("base{visual_count}");
                        let scale = scalar_expression(
                            &shape.keyframes,
                            KeyframeProperty::Scale,
                            shape.transform.scale,
                            shape.start_ms,
                        );
                        let opacity = scalar_expression_for(
                            &shape.keyframes,
                            KeyframeProperty::Opacity,
                            shape.transform.opacity,
                            shape.start_ms,
                            "T",
                        );
                        let x = position_expression(
                            &shape.keyframes,
                            true,
                            shape.transform.position_x,
                            shape.start_ms,
                        );
                        let y = position_expression(
                            &shape.keyframes,
                            false,
                            shape.transform.position_y,
                            shape.start_ms,
                        );
                        let transition =
                            transition_filters(&shape.id, &transitions, shape.start_ms);
                        filters.push(format!("color=c={}:s={}x{}:r={fps}:d={},format=rgba,setpts=PTS+{}/TB,scale=w='iw*({scale})':h='ih*({scale})':eval=frame,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*({opacity})'{transition}[{prepared}]", ffmpeg_color(&shape.color), shape.width, shape.height, seconds(shape.duration_ms), seconds(shape.start_ms)));
                        filters.push(format!("[{current_video}][{prepared}]overlay=x='{x}':y='{y}':enable='between(t,{},{})'[{composited}]", seconds(shape.start_ms), seconds(shape.start_ms.saturating_add(shape.duration_ms))));
                        current_video = composited;
                    }
                    TimelineItem::Caption(caption) => {
                        visual_count += 1;
                        let composited = format!("base{visual_count}");
                        let font = self
                            .default_font_path
                            .as_ref()
                            .map(|path| format!("fontfile='{}':", escape_filter_path(path)))
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

struct TextMetrics {
    width: f64,
    height: f64,
    line_count: usize,
}

fn wrap_text(
    text: &str,
    width_px: Option<u32>,
    font_size: u32,
    font_path: Option<&Path>,
) -> String {
    let Some(width_px) = width_px else {
        return text.to_owned();
    };
    let font_data = font_path.and_then(|path| std::fs::read(path).ok());
    let face = font_data
        .as_deref()
        .and_then(|data| ttf_parser::Face::parse(data, 0).ok());
    wrap_text_with_measure(text, f64::from(width_px), |value| {
        measure_text_run(value, font_size, face.as_ref())
    })
}

fn wrap_text_with_measure(text: &str, maximum_width: f64, measure: impl Fn(&str) -> f64) -> String {
    text.split('\n')
        .map(|line| {
            let mut lines = Vec::new();
            let mut current = String::new();
            for word in line.split_whitespace() {
                let proposed = if current.is_empty() {
                    word.to_owned()
                } else {
                    format!("{current} {word}")
                };
                if measure(&proposed) <= maximum_width {
                    current = proposed;
                    continue;
                }
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                if measure(word) <= maximum_width {
                    current.push_str(word);
                    continue;
                }
                for character in word.chars() {
                    let mut candidate = current.clone();
                    candidate.push(character);
                    if !current.is_empty() && measure(&candidate) > maximum_width {
                        lines.push(std::mem::take(&mut current));
                    }
                    current.push(character);
                }
            }
            lines.push(current);
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn measure_text_block(text: &str, font_size: u32, font_path: Option<&Path>) -> TextMetrics {
    let font_data = font_path.and_then(|path| std::fs::read(path).ok());
    let face = font_data
        .as_deref()
        .and_then(|data| ttf_parser::Face::parse(data, 0).ok());
    let line_count = text.split('\n').count().max(1);
    let width = text
        .split('\n')
        .map(|line| measure_text_run(line, font_size, face.as_ref()))
        .fold(0.0_f64, f64::max);
    let line_height = face.as_ref().map_or(f64::from(font_size) * 1.2, |face| {
        let units = f64::from(face.units_per_em());
        let height = f64::from(face.ascender() - face.descender() + face.line_gap());
        (height / units * f64::from(font_size)).max(1.0)
    });
    TextMetrics {
        width,
        height: line_height * line_count as f64,
        line_count,
    }
}

fn measure_text_run(text: &str, font_size: u32, face: Option<&ttf_parser::Face<'_>>) -> f64 {
    let Some(face) = face else {
        return text.chars().count() as f64 * f64::from(font_size) * 0.6;
    };
    let units = f64::from(face.units_per_em());
    let fallback = face
        .glyph_index('\u{fffd}')
        .or_else(|| face.glyph_index('?'));
    let advance = text
        .chars()
        .map(|character| {
            face.glyph_index(character)
                .or(fallback)
                .and_then(|glyph| face.glyph_hor_advance(glyph))
                .map_or(units * 0.6, f64::from)
        })
        .sum::<f64>();
    advance / units * f64::from(font_size)
}

fn ffmpeg_color(color: &str) -> String {
    format!("0x{}", color.trim_start_matches('#'))
}

fn find_font_file(root: &Path, normalized_family: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_font_file(&path, normalized_family) {
                return Some(found);
            }
        } else if matches!(
            path.extension()
                .and_then(|value| value.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("ttf" | "otf" | "ttc")
        ) {
            let stem = path
                .file_stem()?
                .to_string_lossy()
                .to_lowercase()
                .replace([' ', '-', '_'], "");
            if stem.contains(normalized_family) {
                return path.canonicalize().ok();
            }
        }
    }
    None
}

fn anchored_layer_position(x: &str, y: &str, anchor: crate::AnchorPoint) -> (String, String) {
    use crate::AnchorPoint::*;
    let anchored_x = match anchor {
        TopCenter | Center | BottomCenter => format!("({x})-(overlay_w/2)"),
        TopRight | CenterRight | BottomRight => format!("({x})-overlay_w"),
        _ => x.into(),
    };
    let anchored_y = match anchor {
        CenterLeft | Center | CenterRight => format!("({y})-(overlay_h/2)"),
        BottomLeft | BottomCenter | BottomRight => format!("({y})-overlay_h"),
        _ => y.into(),
    };
    (anchored_x, anchored_y)
}

fn text_layer_padding(anchor: crate::AnchorPoint) -> (&'static str, &'static str) {
    use crate::AnchorPoint::*;
    let x = match anchor {
        TopCenter | Center | BottomCenter => "(ow-iw)/2",
        TopRight | CenterRight | BottomRight => "ow-iw",
        _ => "0",
    };
    let y = match anchor {
        CenterLeft | Center | CenterRight => "(oh-ih)/2",
        BottomLeft | BottomCenter | BottomRight => "oh-ih",
        _ => "0",
    };
    (x, y)
}

fn ducking_expression(track: &crate::Track, intervals: &[(u64, u64)]) -> String {
    let Some(settings) = track.ducking.as_ref().filter(|settings| settings.enabled) else {
        return "1".into();
    };
    if track.audio_role != crate::AudioTrackRole::Music || intervals.is_empty() {
        return "1".into();
    }
    let mut expression = "1".to_owned();
    for (start, end) in intervals {
        let attack_start = start.saturating_sub(settings.attack_ms);
        let release_end = end.saturating_add(settings.release_ms);
        let attack = seconds(settings.attack_ms.max(1));
        let release = seconds(settings.release_ms.max(1));
        let gain = format_number(settings.gain);
        let envelope = format!(
            "if(between(t,{},{}),1-(1-({gain}))*((t-{})/{attack}),if(between(t,{},{}),({gain}),if(between(t,{},{}),({gain})+(1-({gain}))*((t-{})/{release}),1)))",
            seconds(attack_start),
            seconds(*start),
            seconds(attack_start),
            seconds(*start),
            seconds(*end),
            seconds(*end),
            seconds(release_end),
            seconds(*end),
        );
        expression = format!("min({expression},{envelope})");
    }
    expression
}

fn audible_voiceover_intervals(
    project: &Project,
    asset_by_id: &HashMap<&str, &crate::Asset>,
) -> Vec<(u64, u64)> {
    merge_intervals(
        project
            .tracks
            .iter()
            .filter(|track| {
                !track.hidden
                    && !track.muted
                    && track.audio_role == crate::AudioTrackRole::Voiceover
            })
            .flat_map(|track| track.items.iter())
            .flat_map(|item| {
                let TimelineItem::Media(media) = item else {
                    return vec![];
                };
                if media.hidden
                    || media.audio.muted
                    || media.audio.volume == 0.0
                    || !asset_by_id
                        .get(media.asset_id.as_str())
                        .is_some_and(|asset| asset.has_audio)
                {
                    return vec![];
                }
                positive_scalar_ranges(
                    &media.keyframes,
                    KeyframeProperty::Volume,
                    media.duration_ms,
                )
                .into_iter()
                .map(|(start, end)| {
                    (
                        media.start_ms.saturating_add(start),
                        media.start_ms.saturating_add(end),
                    )
                })
                .collect()
            })
            .collect(),
    )
}

fn merge_intervals(mut intervals: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    intervals.retain(|(start, end)| start < end);
    intervals.sort_unstable_by_key(|(start, end)| (*start, *end));
    let mut merged: Vec<(u64, u64)> = Vec::with_capacity(intervals.len());
    for (start, end) in intervals {
        if let Some((_, previous_end)) = merged.last_mut()
            && start <= *previous_end
        {
            *previous_end = (*previous_end).max(end);
        } else {
            merged.push((start, end));
        }
    }
    merged
}

#[cfg(test)]
fn ducking_gain_at(
    settings: &crate::DuckingSettings,
    intervals: &[(u64, u64)],
    time_ms: u64,
) -> f64 {
    intervals.iter().fold(1.0, |gain, (start, end)| {
        let attack_start = start.saturating_sub(settings.attack_ms);
        let release_end = end.saturating_add(settings.release_ms);
        let envelope = if (attack_start..*start).contains(&time_ms) {
            let progress = (time_ms - attack_start) as f64 / settings.attack_ms.max(1) as f64;
            1.0 - (1.0 - settings.gain) * progress
        } else if (*start..=*end).contains(&time_ms) {
            settings.gain
        } else if (*end < time_ms) && time_ms <= release_end {
            let progress = (time_ms - end) as f64 / settings.release_ms.max(1) as f64;
            settings.gain + (1.0 - settings.gain) * progress
        } else {
            1.0
        };
        gain.min(envelope)
    })
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

fn configure_preview_range_outputs(
    command: &mut Command,
    options: PreviewRangeOptions,
    temporary: &Path,
) {
    let duration = seconds(options.end_ms - options.start_ms);
    command.args(["-ss", &seconds(options.start_ms), "-map", "[video]"]);
    if options.include_audio {
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
        .arg(temporary);
    if !options.include_audio {
        command
            .args(["-map", "[audio]", "-t", &duration, "-f", "null"])
            .arg(if cfg!(windows) { "NUL" } else { "/dev/null" });
    }
}

impl RenderWorkspace {
    fn create(project_dir: &Path) -> Result<Self, CoreError> {
        let request_id = env::var("OPENCUT_REQUEST_ID")
            .ok()
            .filter(|value| {
                !value.is_empty()
                    && value
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '-')
            })
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let path = project_dir.join(format!(".opencut-work-{request_id}"));
        std::fs::create_dir(&path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
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
        if std::fs::remove_file(output).is_err() {
            let _ = std::fs::remove_file(temporary);
            return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
        }
    }
    std::fs::rename(temporary, output).map_err(|_| {
        let _ = std::fs::remove_file(temporary);
        CoreError::render_failure(PUBLISH_STAGE, None, None)
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
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
    let resolved = project_dir
        .join(relative)
        .canonicalize()
        .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
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
    _item_start_ms: u64,
) -> String {
    let mut result = String::new();
    for transition in transitions {
        if transition.from_item_id == item_id {
            result.push_str(&format!(
                ",fade=t=out:st={}:d={}:alpha=1",
                seconds(transition.start_ms),
                seconds(transition.duration_ms)
            ));
        }
        if transition.to_item_id.as_deref() == Some(item_id) {
            result.push_str(&format!(
                ",fade=t=in:st={}:d={}:alpha=1",
                seconds(transition.start_ms),
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
    scalar_expression_for(keyframes, property, default, item_start_ms, "t")
}

fn scalar_expression_for(
    keyframes: &[Keyframe],
    property: KeyframeProperty,
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
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
    piecewise_expression_for(&values, default, item_start_ms, time_variable)
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
    piecewise_expression_for(values, default, item_start_ms, "t")
}

fn piecewise_expression_for(
    values: &[(u64, f64, Easing)],
    default: f64,
    item_start_ms: u64,
    time_variable: &str,
) -> String {
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
        let progress = format!("(({time_variable})-({global_start}))/({span})");
        let eased = easing_expression(&progress, easing);
        let interpolated = format!(
            "({})+(({})-({}))*({eased})",
            format_number(start_value),
            format_number(end_value),
            format_number(start_value)
        );
        expression =
            format!("if(lt(({time_variable}),({global_end})),{interpolated},{expression})");
    }
    let first_time = seconds(item_start_ms.saturating_add(values[0].0));
    format!(
        "if(lt(({time_variable}),({first_time})),({}),{expression})",
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

fn escape_filter_path(path: &Path) -> String {
    escape_filter(&path.to_string_lossy().replace('\\', "/"))
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
    warnings: Vec<String>,
) -> Result<RenderArtifact, CoreError> {
    let size_bytes = match path.metadata() {
        Ok(metadata) => metadata.len(),
        Err(_) => {
            let _ = std::fs::remove_file(path);
            return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
        }
    };
    if size_bytes == 0 {
        let _ = std::fs::remove_file(path);
        return Err(CoreError::render_failure(PUBLISH_STAGE, None, None));
    }
    Ok(RenderArtifact {
        relative_path,
        mime_type: mime_type.into(),
        size_bytes,
        warnings,
    })
}

fn run_to_completion(
    command: &mut Command,
    duration_ms: u64,
    mut on_progress: impl FnMut(RenderProgress),
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

fn read_bounded_tail(mut reader: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
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

fn stderr_excerpt(stderr: &[u8]) -> Option<String> {
    let excerpt = sanitize_stderr(&String::from_utf8_lossy(stderr));
    (!excerpt.trim().is_empty()).then_some(excerpt)
}

fn map_renderer_error(error: CoreError, stage: &str) -> CoreError {
    match error.code {
        ErrorCode::InternalError | ErrorCode::ProjectRecoveryFailed | ErrorCode::JobFailed => {
            CoreError::render_failure(stage, None, None)
        }
        _ => error,
    }
}

fn sanitize_stderr(stderr: &str) -> String {
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

fn sanitize_stderr_line(line: &str) -> String {
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

fn find_absolute_path_start(value: &str) -> Option<usize> {
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
    use crate::{
        CaptionItem, CaptionSource, CaptionStyle, PROJECT_SCHEMA_VERSION, ProjectSettings,
        SolidColorItem, TimelineItem, Track, TrackType, Transform,
    };
    use tempfile::tempdir;

    #[test]
    fn keyframe_expression_contains_no_shell_syntax() {
        let expression = piecewise_expression(
            &[(0, 1.0, Easing::Linear), (1_000, 2.0, Easing::EaseOut)],
            1.0,
            500,
        );
        assert!(expression.contains("if(lt((t),"));
        assert!(!expression.contains('$'));
        assert!(!expression.contains('`'));
    }

    #[test]
    fn preview_range_without_audio_consumes_the_labeled_audio_output() {
        let mut command = Command::new("ffmpeg");
        configure_preview_range_outputs(
            &mut command,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 1_100,
                width: 320,
                height: 180,
                fps: 15,
                include_audio: false,
            },
            Path::new("preview.mp4"),
        );
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let audio_map = arguments
            .windows(2)
            .filter(|window| *window == ["-map", "[audio]"])
            .count();
        assert_eq!(audio_map, 1);
        assert!(arguments.windows(2).any(|window| window == ["-f", "null"]));
        assert!(!arguments.windows(2).any(|window| window == ["-c:a", "aac"]));
    }

    #[test]
    fn keyframe_interpolation_parenthesizes_every_sign_and_easing_combination() {
        let easings = [
            Easing::Hold,
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
        ];
        for easing in easings {
            for (start, end) in [
                (288.0, 250.0),
                (288.0, -250.0),
                (-288.0, 250.0),
                (-288.0, -250.0),
            ] {
                for property in [
                    KeyframeProperty::Position,
                    KeyframeProperty::Scale,
                    KeyframeProperty::Opacity,
                    KeyframeProperty::Volume,
                ] {
                    let values = vec![
                        Keyframe {
                            property,
                            time_ms: 0,
                            value: if property == KeyframeProperty::Position {
                                KeyframeValue::Position { x: start, y: start }
                            } else {
                                KeyframeValue::Scalar { value: start }
                            },
                            easing,
                        },
                        Keyframe {
                            property,
                            time_ms: 1_000,
                            value: if property == KeyframeProperty::Position {
                                KeyframeValue::Position { x: end, y: end }
                            } else {
                                KeyframeValue::Scalar { value: end }
                            },
                            easing: Easing::Linear,
                        },
                    ];
                    let expression = if property == KeyframeProperty::Position {
                        position_expression(&values, true, start, 0)
                    } else {
                        scalar_expression(&values, property, start, 0)
                    };
                    assert!(expression.contains(&format!("({:.6})", start)));
                    assert!(expression.contains(&format!("({:.6})", end)));
                    assert!(!expression.contains("-288.000000--250.000000"));
                    assert!(!expression.contains("288.000000--250.000000"));
                }
            }
        }
    }

    #[test]
    fn wrapping_preserves_newlines_and_unicode_without_filter_escaping() {
        let value = "It's: 100% \\ café →\nsecond line";
        assert_eq!(wrap_text(value, None, 48, None), value);
        let wrapped = wrap_text("one two three", Some(80), 20, None);
        assert!(wrapped.contains('\n'));
    }

    #[test]
    fn wrapping_uses_proportional_and_unicode_glyph_measurements() {
        let measure = |value: &str| {
            value
                .chars()
                .map(|character| match character {
                    'W' => 10.0,
                    'i' => 2.0,
                    'é' => 7.0,
                    '→' => 9.0,
                    ' ' => 3.0,
                    _ => 5.0,
                })
                .sum()
        };
        assert_eq!(
            wrap_text_with_measure("iiii WWWW", 20.0, measure),
            "iiii\nWW\nWW"
        );
        assert_eq!(wrap_text_with_measure("café →", 30.0, measure), "café\n→");
    }

    #[test]
    fn overlapping_voiceovers_remain_ducked_and_adjacent_envelopes_take_minimum_gain() {
        let settings = crate::DuckingSettings {
            enabled: true,
            gain: 0.2,
            attack_ms: 200,
            release_ms: 200,
        };
        let merged = merge_intervals(vec![(900, 2_000), (0, 1_000)]);
        assert_eq!(merged, vec![(0, 2_000)]);
        assert_eq!(ducking_gain_at(&settings, &merged, 1_100), settings.gain);

        let adjacent = vec![(0, 1_000), (1_100, 2_000)];
        assert!(ducking_gain_at(&settings, &adjacent, 1_050) < 0.5);
        let track = Track {
            id: "music".into(),
            name: "Music".into(),
            track_type: TrackType::Audio,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: crate::AudioTrackRole::Music,
            ducking: Some(settings),
            items: vec![],
        };
        assert!(
            ducking_expression(&track, &adjacent)
                .matches("min(")
                .count()
                >= 2
        );
    }

    #[test]
    fn voiceover_activity_excludes_manual_and_automated_silence() {
        let assets = [crate::Asset {
            id: "voice".into(),
            media_type: MediaType::Audio,
            file_name: "voice.wav".into(),
            project_relative_path: "assets/voice.wav".into(),
            duration_ms: Some(3_000),
            has_audio: true,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        }];
        let media = |id: &str, start_ms: u64, volume: f64, keyframes: Vec<Keyframe>| {
            TimelineItem::Media(crate::MediaItem {
                id: id.into(),
                asset_id: "voice".into(),
                start_ms,
                duration_ms: 1_000,
                source_in_ms: 0,
                transform: Transform::default(),
                audio: crate::AudioSettings {
                    volume,
                    ..crate::AudioSettings::default()
                },
                keyframes,
                hidden: false,
            })
        };
        let volume = |time_ms: u64, value: f64, easing: Easing| Keyframe {
            property: KeyframeProperty::Volume,
            time_ms,
            value: KeyframeValue::Scalar { value },
            easing,
        };
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "voiceover-activity".into(),
            revision: 0,
            name: "voiceover activity".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings::default(),
            assets: assets.to_vec(),
            tracks: vec![Track {
                id: "voiceover".into(),
                name: "Voiceover".into(),
                track_type: TrackType::Audio,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Voiceover,
                ducking: None,
                items: vec![
                    media("manual-silent", 0, 0.0, vec![]),
                    media(
                        "automated-silent",
                        1_000,
                        1.0,
                        vec![
                            volume(0, 0.0, Easing::Hold),
                            volume(1_000, 0.0, Easing::Linear),
                        ],
                    ),
                    media(
                        "mixed",
                        2_000,
                        0.5,
                        vec![
                            volume(0, 0.0, Easing::Hold),
                            volume(500, 1.0, Easing::Linear),
                            volume(1_000, 0.0, Easing::Linear),
                        ],
                    ),
                ],
            }],
        };
        let asset_by_id = assets
            .iter()
            .map(|asset| (asset.id.as_str(), asset))
            .collect::<HashMap<_, _>>();
        assert_eq!(
            audible_voiceover_intervals(&project, &asset_by_id),
            vec![(2_500, 3_000)]
        );
    }

    #[test]
    fn manual_volume_multiplies_the_automation_expression() {
        let automation = scalar_expression(
            &[
                Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: 0,
                    value: KeyframeValue::Scalar { value: 0.5 },
                    easing: Easing::Linear,
                },
                Keyframe {
                    property: KeyframeProperty::Volume,
                    time_ms: 1_000,
                    value: KeyframeValue::Scalar { value: 1.0 },
                    easing: Easing::Linear,
                },
            ],
            KeyframeProperty::Volume,
            1.0,
            0,
        );
        let effective = format!("({})*({automation})", format_number(0.25));
        assert!(effective.starts_with("(0.250000)*("));
        assert!(effective.contains("(0.500000)"));
        assert!(effective.contains("(1.000000)"));
    }

    #[test]
    fn text_scale_applies_to_the_complete_isolated_styled_layer() {
        let root = tempdir().unwrap();
        let style = crate::TextStyle {
            outline_width_px: 3,
            shadow: crate::TextShadow {
                offset_x: 4,
                offset_y: 5,
                ..crate::TextShadow::default()
            },
            background_opacity: 0.6,
            padding: crate::TextPadding {
                top: 13,
                right: 12,
                bottom: 14,
                left: 11,
            },
            line_spacing_px: 7,
            anchor: crate::AnchorPoint::Center,
            ..crate::TextStyle::default()
        };
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "text scale".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![TimelineItem::Text(crate::TextItem {
                    id: "text".into(),
                    text: "Styled\ntext".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 40,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style,
                    transform: Transform {
                        position_x: 100.0,
                        position_y: 200.0,
                        scale: 2.0,
                        opacity: 0.75,
                    },
                    keyframes: vec![],
                    hidden: false,
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        let mut warnings = Vec::new();
        let text_layers = renderer
            .prepare_text_layers(&project, root.path(), &mut warnings)
            .unwrap();
        let asset_by_id = HashMap::new();
        let input_indexes = HashMap::new();
        let filter = renderer
            .build_filter(
                &project,
                FilterContext {
                    asset_by_id: &asset_by_id,
                    input_indexes: &input_indexes,
                    text_layers: &text_layers,
                    width: 1920,
                    height: 1080,
                    fps: 30,
                },
                &mut warnings,
            )
            .unwrap();
        assert!(filter.contains("fontsize=40"));
        assert!(!filter.contains("fontsize='40*"));
        assert!(filter.contains("borderw=3"));
        assert!(filter.contains("shadowx=4:shadowy=5"));
        assert!(filter.contains("boxborderw=13|12|14|11"));
        assert!(filter.contains("line_spacing=7"));
        assert!(filter.contains("scale=w='iw*(2.000000)':h='ih*(2.000000)':eval=frame"));
        assert!(filter.contains("pad="));
        assert!(filter.contains(":(ow-iw)/2:(oh-ih)/2:color=black@0:eval=frame"));
        assert!(filter.contains("overlay_w/2"));
        assert!(filter.contains("overlay_h/2"));
    }

    #[test]
    fn filter_text_is_escaped() {
        assert_eq!(escape_filter("it's: 100%"), "it\\'s\\: 100\\%");
    }

    #[test]
    fn ffmpeg_stderr_is_path_redacted_and_tail_bounded() {
        let stderr = format!(
            "{} input=C:\\Users\\Jane Doe\\private clip.mov: Invalid argument\nsource='/var/private/project clip.mov': Permission denied\nfinal diagnostic",
            "x".repeat(5_000)
        );
        let sanitized = sanitize_stderr(&stderr);
        assert!(sanitized.len() <= STDERR_EXCERPT_BYTES);
        assert!(!sanitized.contains("C:\\Users"));
        assert!(!sanitized.contains("Jane Doe"));
        assert!(!sanitized.contains("private clip.mov"));
        assert!(!sanitized.contains("/var/private"));
        assert!(sanitized.contains("[path]"));
        assert!(sanitized.contains("Invalid argument"));
        assert!(sanitized.contains("Permission denied"));
        assert!(sanitized.ends_with("final diagnostic"));
    }

    #[test]
    fn bounded_stderr_tail_handles_unicode_without_splitting_panics() {
        let stderr = format!(
            "{}\nC:\\Users\\Private\\clip.mov: entrée rejetée → final diagnostic",
            "é→".repeat(8_192)
        );
        let tail =
            read_bounded_tail(std::io::Cursor::new(stderr.as_bytes()), STDERR_TAIL_BYTES).unwrap();
        let excerpt = stderr_excerpt(&tail).unwrap();
        assert!(excerpt.len() <= STDERR_EXCERPT_BYTES);
        assert!(!excerpt.contains("C:\\Users"));
        assert!(excerpt.contains("[path]"));
        assert!(excerpt.ends_with("entrée rejetée → final diagnostic"));
    }

    #[test]
    fn renderer_setup_and_publication_failures_are_structured() {
        let root = tempdir().unwrap();
        let invalid_project_dir = root.path().join("project-file");
        std::fs::write(&invalid_project_dir, b"not a directory").unwrap();
        let graph_error = RenderWorkspace::create(&invalid_project_dir).err().unwrap();
        assert_eq!(graph_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(graph_error.failed_stage.as_deref(), Some(GRAPH_BUILD_STAGE));
        assert_eq!(graph_error.ffmpeg_exit_code, None);
        assert_eq!(graph_error.ffmpeg_stderr_excerpt, None);

        let temporary = root.path().join("temporary.mp4");
        std::fs::write(&temporary, b"partial").unwrap();
        let output_directory = root.path().join("output.mp4");
        std::fs::create_dir(&output_directory).unwrap();
        let publish_error = publish_output(&temporary, &output_directory, true).unwrap_err();
        assert_eq!(publish_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(publish_error.failed_stage.as_deref(), Some(PUBLISH_STAGE));
        assert_eq!(publish_error.ffmpeg_exit_code, None);
        assert_eq!(publish_error.ffmpeg_stderr_excerpt, None);
        assert!(!temporary.exists());

        let inspection_error = artifact(
            &root.path().join("missing.mp4"),
            "missing.mp4".into(),
            "video/mp4",
            vec![],
        )
        .unwrap_err();
        assert_eq!(inspection_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(
            inspection_error.failed_stage.as_deref(),
            Some(PUBLISH_STAGE)
        );

        let empty = root.path().join("empty.mp4");
        File::create(&empty).unwrap();
        let empty_error = artifact(&empty, "empty.mp4".into(), "video/mp4", vec![]).unwrap_err();
        assert_eq!(empty_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(empty_error.failed_stage.as_deref(), Some(PUBLISH_STAGE));
        assert!(!empty.exists());
    }

    #[test]
    fn process_spawn_and_exit_failures_are_structured() {
        let mut missing = Command::new("definitely-missing-opencut-ffmpeg");
        let spawn_error = run_to_completion(&mut missing, 1_000, |_| {}).unwrap_err();
        assert_eq!(spawn_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(spawn_error.failed_stage.as_deref(), Some(SPAWN_STAGE));
        assert_eq!(spawn_error.ffmpeg_exit_code, None);
        assert_eq!(spawn_error.ffmpeg_stderr_excerpt, None);

        #[cfg(windows)]
        let mut failing = {
            let mut command = Command::new("cmd");
            command.args(["/C", "echo final diagnostic 1>&2 & exit /b 7"]);
            command
        };
        #[cfg(not(windows))]
        let mut failing = {
            let mut command = Command::new("sh");
            command.args(["-c", "echo final diagnostic >&2; exit 7"]);
            command
        };
        let render_error = run_to_completion(&mut failing, 1_000, |_| {}).unwrap_err();
        assert_eq!(render_error.code, ErrorCode::FfmpegFailed);
        assert_eq!(render_error.failed_stage.as_deref(), Some(RENDER_STAGE));
        assert_eq!(render_error.ffmpeg_exit_code, Some(7));
        assert!(
            render_error
                .ffmpeg_stderr_excerpt
                .as_deref()
                .is_some_and(|excerpt| excerpt.contains("final diagnostic"))
        );
    }

    #[test]
    fn render_workspace_is_removed_when_text_preparation_fails() {
        let root = tempdir().unwrap();
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "cleanup".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![TimelineItem::Text(crate::TextItem {
                    id: "missing/parent".into(),
                    text: "failure".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 40,
                    color: "#ffffff".into(),
                    font_family: None,
                    font_path: None,
                    style: crate::TextStyle::default(),
                    transform: Transform::default(),
                    keyframes: vec![],
                    hidden: false,
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        assert!(
            renderer
                .build_command(&project, root.path(), 320, 180, 15)
                .is_err()
        );
        assert!(std::fs::read_dir(root.path()).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".opencut-work-")
        }));
    }

    #[test]
    fn render_workspace_guard_removes_completed_preparation_files() {
        let root = tempdir().unwrap();
        let workspace_path;
        {
            let workspace = RenderWorkspace::create(root.path()).unwrap();
            workspace_path = workspace.path().to_owned();
            std::fs::write(workspace.path().join("filter.txt"), b"fixture").unwrap();
            assert!(workspace_path.exists());
        }
        assert!(!workspace_path.exists());
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
    fn native_preview_range_and_export_frames_are_consistent_when_ffmpeg_is_available() {
        let (Ok(ffmpeg), Ok(ffprobe)) = (
            env::var("OPENCUT_FFMPEG_PATH"),
            env::var("OPENCUT_FFPROBE_PATH"),
        ) else {
            return;
        };
        if !Command::new(&ffmpeg)
            .arg("-version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return;
        }

        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "renderer-consistency".into(),
            revision: 7,
            name: "renderer consistency".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings {
                width: 160,
                height: 90,
                fps: 10,
            },
            assets: vec![],
            tracks: vec![Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![
                    TimelineItem::SolidColor(SolidColorItem {
                        id: "background".into(),
                        color: "#cc3311".into(),
                        start_ms: 0,
                        duration_ms: 1_000,
                        transform: Transform {
                            opacity: 0.7,
                            ..Transform::default()
                        },
                        keyframes: vec![],
                        hidden: false,
                    }),
                    TimelineItem::Text(crate::TextItem {
                        id: "animated-text".into(),
                        text: "café →\nWWWW iiii".into(),
                        start_ms: 0,
                        duration_ms: 1_000,
                        font_size: 20,
                        color: "#ffffff".into(),
                        font_family: None,
                        font_path: None,
                        style: crate::TextStyle {
                            wrap_width_px: Some(100),
                            line_spacing_px: 3,
                            outline_width_px: 1,
                            background_opacity: 0.4,
                            padding: crate::TextPadding {
                                top: 2,
                                right: 4,
                                bottom: 3,
                                left: 5,
                            },
                            anchor: crate::AnchorPoint::Center,
                            ..crate::TextStyle::default()
                        },
                        transform: Transform {
                            position_x: 80.0,
                            position_y: 45.0,
                            scale: 1.0,
                            opacity: 0.8,
                        },
                        keyframes: vec![
                            Keyframe {
                                property: KeyframeProperty::Scale,
                                time_ms: 0,
                                value: KeyframeValue::Scalar { value: 0.8 },
                                easing: Easing::EaseInOut,
                            },
                            Keyframe {
                                property: KeyframeProperty::Scale,
                                time_ms: 1_000,
                                value: KeyframeValue::Scalar { value: 1.2 },
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                property: KeyframeProperty::Opacity,
                                time_ms: 0,
                                value: KeyframeValue::Scalar { value: 0.4 },
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                property: KeyframeProperty::Opacity,
                                time_ms: 1_000,
                                value: KeyframeValue::Scalar { value: 1.0 },
                                easing: Easing::Linear,
                            },
                        ],
                        hidden: false,
                    }),
                ],
            }],
        };
        let renderer = Renderer::new(
            &ffmpeg,
            ffprobe,
            env::var_os("OPENCUT_TEST_FONT_PATH").map(PathBuf::from),
        );
        let preview = renderer.render_preview(&project, root.path(), 500).unwrap();
        let range = renderer
            .render_preview_range(
                &project,
                root.path(),
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1_000,
                    width: 160,
                    height: 90,
                    fps: 10,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let export_path = root.path().join("export.mp4");
        renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &export_path,
                    width: 160,
                    height: 90,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();

        let preview_frame = decode_rgb_frame(&ffmpeg, &root.path().join(preview.relative_path), 0);
        let range_frame = decode_rgb_frame(&ffmpeg, &root.path().join(range.relative_path), 500);
        let export_frame = decode_rgb_frame(&ffmpeg, &export_path, 500);
        assert_frames_close(&preview_frame, &range_frame, 8.0);
        assert_frames_close(&preview_frame, &export_frame, 8.0);
    }

    #[test]
    fn native_split_linear_keyframes_match_the_unsplit_render_when_ffmpeg_is_available() {
        let (Ok(ffmpeg), Ok(ffprobe)) = (
            env::var("OPENCUT_FFMPEG_PATH"),
            env::var("OPENCUT_FFPROBE_PATH"),
        ) else {
            return;
        };
        if !Command::new(&ffmpeg)
            .arg("-version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return;
        }

        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let rectangle = crate::RectangleItem {
            id: "animated".into(),
            color: "#44aaee".into(),
            width: 40,
            height: 30,
            start_ms: 0,
            duration_ms: 1_000,
            transform: Transform::default(),
            keyframes: vec![
                Keyframe {
                    property: KeyframeProperty::Position,
                    time_ms: 0,
                    value: KeyframeValue::Position { x: 10.0, y: 10.0 },
                    easing: Easing::Linear,
                },
                Keyframe {
                    property: KeyframeProperty::Position,
                    time_ms: 1_000,
                    value: KeyframeValue::Position { x: 90.0, y: 45.0 },
                    easing: Easing::Linear,
                },
                Keyframe {
                    property: KeyframeProperty::Opacity,
                    time_ms: 0,
                    value: KeyframeValue::Scalar { value: 0.25 },
                    easing: Easing::Linear,
                },
                Keyframe {
                    property: KeyframeProperty::Opacity,
                    time_ms: 1_000,
                    value: KeyframeValue::Scalar { value: 1.0 },
                    easing: Easing::Linear,
                },
            ],
            hidden: false,
        };
        let project = Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "unsplit".into(),
            revision: 0,
            name: "unsplit".into(),
            created_at_ms: 0,
            updated_at_ms: 0,
            settings: ProjectSettings {
                width: 160,
                height: 90,
                fps: 20,
            },
            assets: vec![],
            tracks: vec![Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![TimelineItem::Rectangle(rectangle.clone())],
            }],
        };
        let mut split_project = project.clone();
        let (left_keyframes, right_keyframes) =
            crate::animation::split_keyframes(&rectangle.keyframes, 500, 1_000);
        let mut left = rectangle.clone();
        left.duration_ms = 500;
        left.keyframes = left_keyframes;
        let mut right = rectangle;
        right.id = "animated-right".into();
        right.start_ms = 500;
        right.duration_ms = 500;
        right.keyframes = right_keyframes;
        split_project.tracks[0].items = vec![
            TimelineItem::Rectangle(left),
            TimelineItem::Rectangle(right),
        ];

        let renderer = Renderer::new(ffmpeg.clone(), ffprobe, None);
        for time_ms in [250, 750] {
            let unsplit = renderer
                .render_preview(&project, root.path(), time_ms)
                .unwrap();
            let split = renderer
                .render_preview(&split_project, root.path(), time_ms)
                .unwrap();
            let unsplit = decode_rgb_frame(&ffmpeg, &root.path().join(unsplit.relative_path), 0);
            let split = decode_rgb_frame(&ffmpeg, &root.path().join(split.relative_path), 0);
            assert_frames_close(&unsplit, &split, 1.0);
        }
    }

    fn decode_rgb_frame(ffmpeg: &str, path: &Path, time_ms: u64) -> Vec<u8> {
        let output = Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-ss"])
            .arg(seconds(time_ms))
            .arg("-i")
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());
        output.stdout
    }

    fn assert_frames_close(left: &[u8], right: &[u8], tolerance: f64) {
        assert_eq!(left.len(), right.len());
        let mean_error = left
            .iter()
            .zip(right)
            .map(|(left, right)| f64::from(left.abs_diff(*right)))
            .sum::<f64>()
            / left.len() as f64;
        assert!(mean_error <= tolerance, "mean pixel error was {mean_error}");
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
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
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
        let text_layers = HashMap::new();
        let asset_by_id = HashMap::new();
        let input_indexes = HashMap::new();
        let mut warnings = Vec::new();
        let filter = renderer
            .build_filter(
                &project,
                FilterContext {
                    asset_by_id: &asset_by_id,
                    input_indexes: &input_indexes,
                    text_layers: &text_layers,
                    width: 1920,
                    height: 1080,
                    fps: 30,
                },
                &mut warnings,
            )
            .unwrap();
        assert!(filter.contains("x='(w-text_w)/2':y='h-text_h-64'"));
        assert!(filter.contains("It\\'s\\: 100\\%"));
        project.tracks[0].hidden = true;
        let hidden = renderer
            .build_filter(
                &project,
                FilterContext {
                    asset_by_id: &asset_by_id,
                    input_indexes: &input_indexes,
                    text_layers: &text_layers,
                    width: 1920,
                    height: 1080,
                    fps: 30,
                },
                &mut warnings,
            )
            .unwrap();
        assert!(!hidden.contains("drawtext"));
    }
}
