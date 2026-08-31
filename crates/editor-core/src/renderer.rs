use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use uuid::Uuid;

use crate::{
    CoreError, ErrorCode, MediaType, Project, TimelineItem,
    render_artifact::{
        GRAPH_BUILD_STAGE, RenderArtifact, RenderWorkspace, artifact, prepare_text_layers,
        publish_output, resolve_project_asset, temporary_output,
    },
    render_plan::{FilterContext, RenderIntent, SceneInput, build_render_plan, seconds},
    render_process::{
        ProbeResult, RenderProgress, map_renderer_error, probe, readiness, run_to_completion,
    },
};

#[cfg(test)]
use crate::render_artifact::{PUBLISH_STAGE, wrap_text, wrap_text_with_measure};
#[cfg(test)]
use crate::render_plan::{
    audible_voiceover_intervals, ducking_expression, ducking_gain_at, escape_filter, format_number,
    merge_intervals, piecewise_expression, position_expression, scalar_expression,
};
#[cfg(test)]
use crate::render_process::{
    RENDER_STAGE, SPAWN_STAGE, STDERR_EXCERPT_BYTES, STDERR_TAIL_BYTES, read_bounded_tail,
    sanitize_stderr, stderr_excerpt,
};

#[cfg(test)]
use crate::{KeyframeProperty, KeyframeValue};

#[derive(Clone, Debug)]
pub struct Renderer {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
    default_font_path: Option<PathBuf>,
    font_roots: Vec<PathBuf>,
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

struct BuiltCommand {
    command: Command,
    _workspace: RenderWorkspace,
    warnings: Vec<String>,
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
        readiness(&self.ffmpeg_path, &self.ffprobe_path)
    }

    pub fn probe(&self, path: &Path) -> Result<ProbeResult, CoreError> {
        probe(&self.ffprobe_path, path)
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
            RenderIntent::Frame { at_ms: time_ms },
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
            RenderIntent::Export,
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
            RenderIntent::Range {
                start_ms: options.start_ms,
                end_ms: options.end_ms,
                include_audio: options.include_audio,
            },
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
        intent: RenderIntent,
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
        let text_layers = prepare_text_layers(
            project,
            workspace.path(),
            self.default_font_path.as_deref(),
            &self.font_roots,
            &mut warnings,
        )?;
        let filter_path = workspace.path().join("filter.txt");
        let plan = build_render_plan(
            SceneInput { project, intent },
            FilterContext {
                asset_by_id: &asset_by_id,
                input_indexes: &input_indexes,
                text_layers: &text_layers,
                width,
                height,
                fps,
            },
            self.default_font_path.as_deref(),
            &mut warnings,
        )
        .map_err(|error| map_renderer_error(error, GRAPH_BUILD_STAGE))?;
        let mut file = File::create(&filter_path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        file.write_all(plan.filter_graph.as_bytes())
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        command.arg("-filter_complex_script").arg(&filter_path);
        Ok(BuiltCommand {
            command,
            _workspace: workspace,
            warnings,
        })
    }

    #[cfg(test)]
    fn build_filter(
        &self,
        project: &Project,
        context: FilterContext<'_>,
        warnings: &mut Vec<String>,
    ) -> Result<String, CoreError> {
        build_render_plan(
            SceneInput {
                project,
                intent: RenderIntent::Export,
            },
            context,
            self.default_font_path.as_deref(),
            warnings,
        )
        .map(|plan| plan.filter_graph)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaptionItem, CaptionSource, CaptionStyle, Easing, Keyframe, PROJECT_SCHEMA_VERSION,
        ProjectSettings, SolidColorItem, TimelineItem, Track, TrackType, Transform,
    };
    use std::env;
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
        let text_layers = prepare_text_layers(
            &project,
            root.path(),
            renderer.default_font_path.as_deref(),
            &renderer.font_roots,
            &mut warnings,
        )
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
                .build_command(&project, root.path(), 320, 180, 15, RenderIntent::Export,)
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
