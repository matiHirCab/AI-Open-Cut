use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use opencut_editor_core::{
    BatchEditOperation, CaptionStyle, CommitGeneratedAssetRequest, CommitTranscriptionRequest,
    CoreError, EditOperation, EditorCore, ExportOptions, GeneratedAssetOrigin, MediaProbeFacts,
    MediaType, PathPolicy, PreviewRangeOptions, ProjectSettings, Renderer,
    ReplaceGeneratedAssetRequest, TranscriptionSegment,
};
use serde::{Deserialize, Serialize};

const HEADLESS_PROTOCOL_VERSION: u32 = 1;
#[cfg(test)]
const HEADLESS_OPERATIONS: [&str; 26] = [
    "commit_draft",
    "commit_generated_asset",
    "commit_transcription",
    "create_draft",
    "create_project",
    "delete_asset",
    "discard_draft",
    "edit",
    "edit_batch",
    "export_video",
    "get_draft",
    "get_draft_state",
    "get_state",
    "import_asset",
    "list_projects",
    "open_project",
    "rebase_draft",
    "redo",
    "render_draft_preview",
    "render_preview",
    "render_preview_range",
    "replace_generated_asset",
    "resolve_asset_input",
    "status",
    "undo",
    "update_draft",
];

#[derive(Debug, Deserialize)]
#[serde(
    tag = "operation",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Request {
    Status {
        #[serde(default)]
        protocol_version: Option<u32>,
    },
    ListProjects {},
    CreateProject {
        name: String,
        #[serde(default)]
        settings: Option<ProjectSettings>,
    },
    OpenProject {
        project_id: String,
    },
    GetState {
        project_id: String,
        start_ms: Option<u64>,
        end_ms: Option<u64>,
    },
    ImportAsset {
        project_id: String,
        expected_revision: u64,
        path: PathBuf,
        media_type: MediaType,
    },
    DeleteAsset {
        project_id: String,
        expected_revision: u64,
        asset_id: String,
    },
    CommitGeneratedAsset {
        project_id: String,
        expected_revision: u64,
        path: PathBuf,
        track_id: String,
        start_ms: u64,
        display_name: String,
        origin: GeneratedAssetOrigin,
    },
    ReplaceGeneratedAsset {
        project_id: String,
        expected_revision: u64,
        path: PathBuf,
        item_id: String,
        origin: GeneratedAssetOrigin,
    },
    Edit {
        project_id: String,
        expected_revision: u64,
        edit: EditOperation,
    },
    EditBatch {
        project_id: String,
        expected_revision: u64,
        operations: Vec<BatchEditOperation>,
    },
    CreateDraft {
        project_id: String,
        expected_revision: u64,
        operations: Vec<EditOperation>,
        label: Option<String>,
    },
    GetDraft {
        project_id: String,
        draft_id: String,
    },
    UpdateDraft {
        project_id: String,
        draft_id: String,
        expected_revision: u64,
        operations: Vec<EditOperation>,
        label: Option<String>,
    },
    RebaseDraft {
        project_id: String,
        draft_id: String,
        expected_revision: u64,
    },
    GetDraftState {
        project_id: String,
        draft_id: String,
    },
    CommitDraft {
        project_id: String,
        draft_id: String,
        expected_revision: u64,
    },
    DiscardDraft {
        project_id: String,
        draft_id: String,
    },
    RenderDraftPreview {
        project_id: String,
        draft_id: String,
        time_ms: u64,
    },
    ResolveAssetInput {
        project_id: String,
        asset_id: String,
    },
    CommitTranscription {
        project_id: String,
        expected_revision: u64,
        asset_id: String,
        caption_track_id: Option<String>,
        provider_id: String,
        model_id: String,
        model_version: Option<String>,
        language: String,
        generated_at_ms: u64,
        segments: Vec<TranscriptionSegment>,
        style: CaptionStyle,
    },
    Undo {
        project_id: String,
        expected_revision: u64,
    },
    Redo {
        project_id: String,
        expected_revision: u64,
    },
    RenderPreview {
        project_id: String,
        expected_revision: u64,
        time_ms: u64,
    },
    RenderPreviewRange {
        project_id: String,
        expected_revision: u64,
        start_ms: u64,
        end_ms: u64,
        width: u32,
        height: u32,
        fps: u32,
        include_audio: bool,
    },
    ExportVideo {
        project_id: String,
        expected_revision: u64,
        relative_path: PathBuf,
        width: u32,
        height: u32,
        overwrite: bool,
    },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event<T: Serialize> {
    Progress { progress: f64 },
    Result { result: T },
    Error { error: ErrorBody },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    code: opencut_editor_core::ErrorCode,
    message: String,
    retryable: bool,
    failed_stage: Option<String>,
    ffmpeg_exit_code: Option<i32>,
    ffmpeg_stderr_excerpt: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubsystemStatus {
    ready: bool,
    capabilities: Vec<&'static str>,
    error: Option<ErrorBody>,
}

#[derive(Serialize)]
struct HeadlessSubsystems {
    editor: SubsystemStatus,
    rendering: SubsystemStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    ready: bool,
    version: &'static str,
    protocol_version: u32,
    capabilities: Vec<&'static str>,
    subsystems: HeadlessSubsystems,
}

fn main() {
    if let Err(error) = run() {
        let _ = emit(Event::<serde_json::Value>::Error {
            error: ErrorBody {
                code: error.code,
                message: error.message,
                retryable: error.retryable,
                failed_stage: error.failed_stage,
                ffmpeg_exit_code: error.ffmpeg_exit_code,
                ffmpeg_stderr_excerpt: error.ffmpeg_stderr_excerpt,
            },
        });
        std::process::exit(1);
    }
}

fn run() -> Result<(), CoreError> {
    let (core, renderer) = configured_services()?;
    if env::args().any(|argument| argument == "--health") {
        emit(Event::Result {
            result: status(&renderer),
        })?;
        return Ok(());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|error| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InvalidArgument,
            error.to_string(),
        )
    })?;
    let request: Request = serde_json::from_str(input.trim()).map_err(|error| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InvalidArgument,
            format!("invalid headless request: {error}"),
        )
    })?;
    match request {
        Request::Status { protocol_version } => {
            negotiate_protocol_version(protocol_version)?;
            emit_value(status(&renderer))
        }
        Request::ListProjects {} => emit_value(core.list_projects()?),
        Request::CreateProject { name, settings } => {
            emit_value(core.create_project(&name, settings.unwrap_or_default())?)
        }
        Request::OpenProject { project_id } => emit_value(core.get_state(&project_id, None)?),
        Request::GetState {
            project_id,
            start_ms,
            end_ms,
        } => {
            let range = match (start_ms, end_ms) {
                (None, None) => None,
                (Some(start), Some(end)) => Some((start, end)),
                _ => {
                    return Err(CoreError::new(
                        opencut_editor_core::ErrorCode::InvalidArgument,
                        "startMs and endMs must be supplied together",
                    ));
                }
            };
            emit_value(core.get_state(&project_id, range)?)
        }
        Request::ImportAsset {
            project_id,
            expected_revision,
            path,
            media_type,
        } => {
            let resolved = core.paths().import_path(&path)?;
            let probe = renderer.probe(&resolved)?;
            match media_type {
                MediaType::Image if !probe.has_video => {
                    return Err(CoreError::new(
                        opencut_editor_core::ErrorCode::UnsupportedMedia,
                        "selected file is not an image",
                    ));
                }
                MediaType::Video if !probe.has_video => {
                    return Err(CoreError::new(
                        opencut_editor_core::ErrorCode::UnsupportedMedia,
                        "selected file has no video stream",
                    ));
                }
                MediaType::Audio if !probe.has_audio => {
                    return Err(CoreError::new(
                        opencut_editor_core::ErrorCode::UnsupportedMedia,
                        "selected file has no audio stream",
                    ));
                }
                _ => {}
            }
            emit_value(core.import_asset(
                &project_id,
                expected_revision,
                resolved,
                media_type,
                MediaProbeFacts {
                    duration_ms: probe.duration_ms,
                    has_audio: probe.has_audio,
                    has_video: probe.has_video,
                    format_name: probe.format_name,
                    video_codec: probe.video_codec,
                    video_width: probe.video_width,
                    video_height: probe.video_height,
                    audio_codec: probe.audio_codec,
                    audio_channels: probe.audio_channels,
                    audio_sample_rate_hz: probe.audio_sample_rate_hz,
                },
            )?)
        }
        Request::DeleteAsset {
            project_id,
            expected_revision,
            asset_id,
        } => emit_value(core.delete_asset(&project_id, expected_revision, &asset_id)?),
        Request::CommitGeneratedAsset {
            project_id,
            expected_revision,
            path,
            track_id,
            start_ms,
            display_name,
            origin,
        } => {
            let resolved = core.paths().generated_media_path(&path)?;
            let probe = renderer.probe(&resolved)?;
            if !probe.has_audio {
                return Err(CoreError::new(
                    opencut_editor_core::ErrorCode::UnsupportedMedia,
                    "generated speech has no audio stream",
                ));
            }
            let duration_ms = probe
                .duration_ms
                .filter(|duration| *duration > 0)
                .ok_or_else(|| {
                    CoreError::new(
                        opencut_editor_core::ErrorCode::UnsupportedMedia,
                        "generated speech has no positive duration",
                    )
                })?;
            emit_value(core.commit_generated_asset(CommitGeneratedAssetRequest {
                project_id,
                expected_revision,
                path: resolved,
                track_id,
                start_ms,
                duration_ms,
                display_name,
                origin,
                probe: MediaProbeFacts {
                    duration_ms: probe.duration_ms,
                    has_audio: probe.has_audio,
                    has_video: probe.has_video,
                    format_name: probe.format_name,
                    video_codec: probe.video_codec,
                    video_width: probe.video_width,
                    video_height: probe.video_height,
                    audio_codec: probe.audio_codec,
                    audio_channels: probe.audio_channels,
                    audio_sample_rate_hz: probe.audio_sample_rate_hz,
                },
            })?)
        }
        Request::ReplaceGeneratedAsset {
            project_id,
            expected_revision,
            path,
            item_id,
            origin,
        } => {
            let resolved = core.paths().generated_media_path(&path)?;
            let probe = renderer.probe(&resolved)?;
            if !probe.has_audio {
                return Err(CoreError::new(
                    opencut_editor_core::ErrorCode::UnsupportedMedia,
                    "generated speech has no audio stream",
                ));
            }
            let duration_ms = probe
                .duration_ms
                .filter(|duration| *duration > 0)
                .ok_or_else(|| {
                    CoreError::new(
                        opencut_editor_core::ErrorCode::UnsupportedMedia,
                        "generated speech has no positive duration",
                    )
                })?;
            emit_value(core.replace_generated_asset(ReplaceGeneratedAssetRequest {
                project_id,
                expected_revision,
                path: resolved,
                item_id,
                duration_ms,
                origin,
                probe: MediaProbeFacts {
                    duration_ms: probe.duration_ms,
                    has_audio: probe.has_audio,
                    has_video: probe.has_video,
                    format_name: probe.format_name,
                    video_codec: probe.video_codec,
                    video_width: probe.video_width,
                    video_height: probe.video_height,
                    audio_codec: probe.audio_codec,
                    audio_channels: probe.audio_channels,
                    audio_sample_rate_hz: probe.audio_sample_rate_hz,
                },
            })?)
        }
        Request::Edit {
            project_id,
            expected_revision,
            edit,
        } => emit_value(core.edit(&project_id, expected_revision, edit)?),
        Request::EditBatch {
            project_id,
            expected_revision,
            operations,
        } => emit_value(core.edit_batch(&project_id, expected_revision, operations)?),
        Request::CreateDraft {
            project_id,
            expected_revision,
            operations,
            label,
        } => emit_value(core.create_draft(&project_id, expected_revision, operations, label)?),
        Request::GetDraft {
            project_id,
            draft_id,
        } => emit_value(core.get_draft(&project_id, &draft_id)?),
        Request::UpdateDraft {
            project_id,
            draft_id,
            expected_revision,
            operations,
            label,
        } => emit_value(core.update_draft(
            &project_id,
            &draft_id,
            expected_revision,
            operations,
            label,
        )?),
        Request::RebaseDraft {
            project_id,
            draft_id,
            expected_revision,
        } => emit_value(core.rebase_draft(&project_id, &draft_id, expected_revision)?),
        Request::GetDraftState {
            project_id,
            draft_id,
        } => emit_value(core.get_draft_state(&project_id, &draft_id)?),
        Request::CommitDraft {
            project_id,
            draft_id,
            expected_revision,
        } => emit_value(core.commit_draft(&project_id, &draft_id, expected_revision)?),
        Request::DiscardDraft {
            project_id,
            draft_id,
        } => emit_value(core.discard_draft(&project_id, &draft_id)?),
        Request::RenderDraftPreview {
            project_id,
            draft_id,
            time_ms,
        } => {
            let state = core.get_draft_state(&project_id, &draft_id)?;
            let dir = core.project_directory(&project_id)?;
            emit_value(renderer.render_preview(&state.project, &dir, time_ms)?)
        }
        Request::ResolveAssetInput {
            project_id,
            asset_id,
        } => emit_value(core.resolve_asset_input(&project_id, &asset_id)?),
        Request::CommitTranscription {
            project_id,
            expected_revision,
            asset_id,
            caption_track_id,
            provider_id,
            model_id,
            model_version,
            language,
            generated_at_ms,
            segments,
            style,
        } => emit_value(core.commit_transcription(CommitTranscriptionRequest {
            project_id,
            expected_revision,
            asset_id,
            caption_track_id,
            provider_id,
            model_id,
            model_version,
            language,
            generated_at_ms,
            segments,
            style,
        })?),
        Request::Undo {
            project_id,
            expected_revision,
        } => emit_value(core.undo(&project_id, expected_revision)?),
        Request::Redo {
            project_id,
            expected_revision,
        } => emit_value(core.redo(&project_id, expected_revision)?),
        Request::RenderPreview {
            project_id,
            expected_revision,
            time_ms,
        } => {
            let project = core.validate_revision(&project_id, expected_revision)?;
            let project_dir = core.paths().project_dir(&project_id)?;
            emit_value(renderer.render_preview(&project, &project_dir, time_ms)?)
        }
        Request::RenderPreviewRange {
            project_id,
            expected_revision,
            start_ms,
            end_ms,
            width,
            height,
            fps,
            include_audio,
        } => {
            let project = core.validate_revision(&project_id, expected_revision)?;
            let project_dir = core.paths().project_dir(&project_id)?;
            emit_value(renderer.render_preview_range(
                &project,
                &project_dir,
                PreviewRangeOptions {
                    start_ms,
                    end_ms,
                    width,
                    height,
                    fps,
                    include_audio,
                },
                |progress| {
                    let _ = emit(Event::<serde_json::Value>::Progress {
                        progress: progress.progress,
                    });
                },
            )?)
        }
        Request::ExportVideo {
            project_id,
            expected_revision,
            relative_path,
            width,
            height,
            overwrite,
        } => {
            let project = core.validate_revision(&project_id, expected_revision)?;
            let project_dir = core.paths().project_dir(&project_id)?;
            let output = core.paths().export_path(&relative_path)?;
            let result = renderer.export_video(
                &project,
                &project_dir,
                ExportOptions {
                    output: &output,
                    width,
                    height,
                    overwrite,
                },
                |progress| {
                    let _ = emit(Event::<serde_json::Value>::Progress {
                        progress: progress.progress,
                    });
                },
            )?;
            let relative = output
                .strip_prefix(core.paths().exports_root())
                .unwrap_or(Path::new("export.mp4"))
                .to_string_lossy()
                .replace('\\', "/");
            emit_value(opencut_editor_core::RenderArtifact {
                relative_path: relative,
                ..result
            })
        }
    }
}

fn configured_services() -> Result<(EditorCore, Renderer), CoreError> {
    let projects = required_path("OPENCUT_PROJECTS_DIR")?;
    let exports = required_path("OPENCUT_EXPORTS_DIR")?;
    let media = env::var_os("OPENCUT_ALLOWED_MEDIA_DIRS").ok_or_else(|| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InvalidArgument,
            "OPENCUT_ALLOWED_MEDIA_DIRS is required",
        )
    })?;
    let media = env::split_paths(&media).collect::<Vec<_>>();
    let mut policy = PathPolicy::new(projects, &media, exports)?;
    if let Some(generated) = env::var_os("OPENCUT_GENERATED_MEDIA_DIRS") {
        policy = policy.with_generated_media_roots(env::split_paths(&generated))?;
    }
    if let Some(generated) = env::var_os("OPENCUT_TTS_WORK_DIR") {
        policy = policy.with_generated_media_root(PathBuf::from(generated))?;
    }
    let ffmpeg = env::var_os("OPENCUT_FFMPEG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ffmpeg"));
    let ffprobe = env::var_os("OPENCUT_FFPROBE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ffprobe"));
    let font = env::var_os("OPENCUT_DEFAULT_FONT_PATH")
        .map(PathBuf::from)
        .or_else(discover_default_font);
    let font_roots = env::var_os("OPENCUT_ALLOWED_FONT_DIRS")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();
    Ok((
        EditorCore::new(policy),
        Renderer::new(ffmpeg, ffprobe, font).with_font_roots(font_roots),
    ))
}

fn discover_default_font() -> Option<PathBuf> {
    [
        PathBuf::from(r"C:\Windows\Fonts\arial.ttf"),
        PathBuf::from(r"C:\Windows\Fonts\segoeui.ttf"),
        PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
        PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf"),
    ]
    .into_iter()
    .find(|path| path.is_file())
}

fn required_path(name: &str) -> Result<PathBuf, CoreError> {
    env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InvalidArgument,
            format!("{name} is required"),
        )
    })
}

fn editor_capabilities() -> Vec<&'static str> {
    vec![
        "projects",
        "assets",
        "timeline",
        "text",
        "text_styling",
        "solid_color",
        "rectangle",
        "keyframes",
        "transitions",
        "audio",
        "audio_roles",
        "audio_ducking",
        "undo_redo",
    ]
}

fn render_capabilities() -> Vec<&'static str> {
    vec!["preview", "preview_range", "mp4_export"]
}

fn negotiate_protocol_version(requested: Option<u32>) -> Result<u32, CoreError> {
    let requested = requested.unwrap_or(HEADLESS_PROTOCOL_VERSION);
    if requested == HEADLESS_PROTOCOL_VERSION {
        Ok(requested)
    } else {
        Err(CoreError::new(
            opencut_editor_core::ErrorCode::InvalidArgument,
            format!(
                "unsupported headless protocol version {requested}; supported version is {HEADLESS_PROTOCOL_VERSION}"
            ),
        ))
    }
}

fn status(renderer: &Renderer) -> Status {
    let rendering = match renderer.readiness() {
        Ok(()) => SubsystemStatus {
            ready: true,
            capabilities: render_capabilities(),
            error: None,
        },
        Err(error) => SubsystemStatus {
            ready: false,
            capabilities: vec![],
            error: Some(ErrorBody {
                code: error.code,
                message: error.message,
                retryable: error.retryable,
                failed_stage: error.failed_stage,
                ffmpeg_exit_code: error.ffmpeg_exit_code,
                ffmpeg_stderr_excerpt: error.ffmpeg_stderr_excerpt,
            }),
        },
    };
    let mut capabilities = editor_capabilities();
    capabilities.extend(rendering.capabilities.iter().copied());
    Status {
        ready: true,
        version: env!("CARGO_PKG_VERSION"),
        protocol_version: HEADLESS_PROTOCOL_VERSION,
        capabilities,
        subsystems: HeadlessSubsystems {
            editor: SubsystemStatus {
                ready: true,
                capabilities: editor_capabilities(),
                error: None,
            },
            rendering,
        },
    }
}

fn emit_value(value: impl Serialize) -> Result<(), CoreError> {
    emit(Event::Result { result: value })
}

fn emit(event: impl Serialize) -> Result<(), CoreError> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &event)?;
    stdout.write_all(b"\n").map_err(|error| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InternalError,
            error.to_string(),
        )
    })?;
    stdout.flush().map_err(|error| {
        CoreError::new(
            opencut_editor_core::ErrorCode::InternalError,
            error.to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_sets_match_the_canonical_headless_contract() {
        let contract: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/headless-protocol-v1.json"))
                .unwrap();
        assert_eq!(
            serde_json::to_value(editor_capabilities()).unwrap(),
            contract["status"]["editorCapabilities"]
        );
        assert_eq!(
            serde_json::to_value(render_capabilities()).unwrap(),
            contract["status"]["renderingCapabilities"]
        );
        assert_eq!(
            HEADLESS_PROTOCOL_VERSION,
            contract["version"].as_u64().unwrap() as u32
        );
        assert_eq!(
            serde_json::to_value(HEADLESS_OPERATIONS).unwrap(),
            contract["operations"]
        );
    }

    #[test]
    fn generated_asset_command_deserializes_provider_neutral_provenance() {
        let request: Request = serde_json::from_value(serde_json::json!({
            "operation": "commit_generated_asset",
            "projectId": "project",
            "expectedRevision": 3,
            "path": "generated.wav",
            "trackId": "audio",
            "startMs": 250,
            "displayName": "speech.wav",
            "origin": {
                "type": "speech_synthesis",
                "generation": {
                    "request": {
                        "text": "Hello",
                        "language": "en-US",
                        "voiceId": "voice",
                        "speed": 1.0
                    },
                    "providerId": "provider",
                    "modelId": "model",
                    "modelVersion": null,
                    "sampleRateHz": 24_000,
                    "generatedAtMs": 1
                }
            }
        }))
        .unwrap();

        match request {
            Request::CommitGeneratedAsset {
                project_id,
                expected_revision,
                origin: GeneratedAssetOrigin::SpeechSynthesis(generation),
                ..
            } => {
                assert_eq!(project_id, "project");
                assert_eq!(expected_revision, 3);
                assert_eq!(generation.provider_id, "provider");
                assert_eq!(generation.request.voice_id.0, "voice");
                assert_eq!(generation.sample_rate_hz, 24_000);
            }
            _ => panic!("unexpected request variant"),
        }
    }
}
