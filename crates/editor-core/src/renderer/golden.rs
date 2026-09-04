use super::*;
use crate::{
    AudioSettings, AudioTrackRole, Easing, Keyframe, KeyframeProperty, KeyframeValue, MediaItem,
    MediaType, PROJECT_SCHEMA_VERSION, ProjectSettings, SolidColorItem, TextItem, TextPadding,
    TextStyle, TimelineItem, Track, TrackType, Transform, evaluated_scene::evaluate_project,
    render_artifact::prepare_media_resources,
};
use fs2::FileExt;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tempfile::tempdir;
use uuid::Uuid;

const POINTER_VERSION: u32 = 1;
const MANIFEST_VERSION: u32 = 1;
const FIXTURE_ID: &str = "flat-scene-av-v1";
const FIXTURE_REVISION: u32 = 3;
const PERFORMANCE_SCHEMA_VERSION: u32 = 3;
const MIGRATION_FIXTURE_REVISION: u32 = 2;
const MIGRATION_PERFORMANCE_SCHEMA_VERSION: u32 = 2;
const STAGE_DEFINITION_VERSION: u32 = 1;
const WARMUP_SAMPLES: u32 = 1;
const MEASURED_SAMPLES: u32 = 3;
const MEMORY_SAMPLE_INTERVAL: Duration = Duration::from_millis(5);
const WIDTH: u32 = 160;
const HEIGHT: u32 = 90;
const FPS: u32 = 10;
const DURATION_MS: u64 = 1_000;
const SAMPLE_TIMESTAMPS_MS: [u64; 3] = [0, 500, 900];
const AUDIO_SAMPLE_RATE_HZ: u32 = 48_000;
const SSIM_MINIMUM: f64 = 0.99;
const PCM_RMS_MAXIMUM: f64 = 0.0001;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenPointer {
    schema_version: u32,
    generation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenManifest {
    schema_version: u32,
    fixture_id: String,
    fixture_revision: u32,
    canvas: GoldenCanvas,
    duration_ms: u64,
    sample_timestamps_ms: Vec<u64>,
    audio: GoldenAudio,
    tolerances: GoldenTolerances,
    environment: GoldenEnvironment,
    references: Vec<GoldenReference>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenCanvas {
    width: u32,
    height: u32,
    fps: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenAudio {
    sample_rate_hz: u32,
    channels: u32,
    sample_format: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenTolerances {
    minimum_ssim: f64,
    maximum_pcm_rms_error: f64,
    maximum_timing_frames: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GoldenEnvironment {
    reference_os: String,
    reference_arch: String,
    ffmpeg_version: String,
    ffprobe_version: String,
    font_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum GoldenReference {
    Frame {
        at_ms: u64,
        path: String,
        sha256: String,
    },
    Audio {
        path: String,
        sha256: String,
    },
    SemanticPlan {
        path: String,
        sha256: String,
    },
    FilterGraph {
        path: String,
        sha256: String,
    },
    PerformanceBaseline {
        path: String,
        sha256: String,
    },
}

impl GoldenReference {
    fn path(&self) -> &str {
        match self {
            Self::Frame { path, .. }
            | Self::Audio { path, .. }
            | Self::SemanticPlan { path, .. }
            | Self::FilterGraph { path, .. }
            | Self::PerformanceBaseline { path, .. } => path,
        }
    }

    fn sha256(&self) -> &str {
        match self {
            Self::Frame { sha256, .. }
            | Self::Audio { sha256, .. }
            | Self::SemanticPlan { sha256, .. }
            | Self::FilterGraph { sha256, .. }
            | Self::PerformanceBaseline { sha256, .. } => sha256,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PerformanceBaseline {
    schema_version: u32,
    fixture_id: String,
    fixture_revision: u32,
    #[serde(deserialize_with = "deserialize_present_git_revision")]
    git_revision: Option<String>,
    os: String,
    architecture: String,
    ffmpeg_version: String,
    ffprobe_version: String,
    font_sha256: String,
    units: BaselineUnits,
    warmup_samples: u32,
    measured_samples: u32,
    memory_scope: String,
    timing_aggregation: String,
    memory_aggregation: String,
    stage_definition_version: u32,
    non_additive_stage_timings: bool,
    intent_observations: Vec<IntentObservation>,
    peak_resident_working_set_bytes: u64,
    comparison_policy: String,
}

fn deserialize_present_git_revision<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyPerformanceBaseline {
    schema_version: u32,
    fixture_id: String,
    fixture_revision: u32,
    git_revision: serde_json::Value,
    os: String,
    architecture: String,
    ffmpeg_version: String,
    ffprobe_version: String,
    font_sha256: String,
    units: BaselineUnits,
    warmup_samples: u32,
    measured_samples: u32,
    memory_scope: String,
    timing_aggregation: String,
    memory_aggregation: String,
    timings_ms: LegacyPhaseTimings,
    peak_resident_working_set_bytes: u64,
    comparison_policy: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LegacyPhaseTimings {
    scene_evaluation: f64,
    filter_graph_construction: f64,
    frame_rendering: f64,
    audiovisual_range_rendering: f64,
    export_rendering: f64,
    total: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BaselineUnits {
    timing: String,
    memory: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageTimings {
    scene_evaluation: f64,
    rasterization: f64,
    filter_graph_construction: f64,
    decoding: f64,
    compositing: f64,
    encoding: f64,
    end_to_end: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageWork {
    decoded_inputs: u64,
    rasterized_layers: u64,
    composited_layers: u64,
    encoded_video_streams: u64,
    encoded_audio_streams: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct IntentObservation {
    intent: String,
    timings_ms: StageTimings,
    work: StageWork,
}

#[derive(Debug)]
struct NativeTools {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    font: PathBuf,
    ffmpeg_version: String,
    ffprobe_version: String,
    font_sha256: String,
}

#[derive(Clone, Debug)]
struct Capture {
    frames: BTreeMap<u64, Vec<u8>>,
    range_frames: BTreeMap<u64, Vec<u8>>,
    export_frames: BTreeMap<u64, Vec<u8>>,
    range_audio: Vec<f32>,
    export_audio: Vec<f32>,
    range_duration_ms: u64,
    export_duration_ms: u64,
    semantic_plan: String,
    filter_graph: String,
    intent_observations: Vec<IntentObservation>,
    peak_process_tree_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureMode {
    Conformance,
    Benchmark { sample_memory: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BenchmarkProbeStage {
    Decode,
    Composite,
}

trait BenchmarkProbeExecutor: std::fmt::Debug {
    fn execute(
        &self,
        stage: BenchmarkProbeStage,
        command: &mut Command,
        duration_ms: u64,
    ) -> Result<(), CoreError>;
}

#[derive(Debug)]
struct NativeBenchmarkProbeExecutor;

impl BenchmarkProbeExecutor for NativeBenchmarkProbeExecutor {
    fn execute(
        &self,
        _stage: BenchmarkProbeStage,
        command: &mut Command,
        duration_ms: u64,
    ) -> Result<(), CoreError> {
        run_to_completion(command, duration_ms, |_| {})
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PublishFault {
    #[default]
    None,
    BeforeInstall,
    GenerationContentSync,
    GenerationDirectorySync,
    GenerationDirectorySyncAndCleanup,
    InstallUnconfirmedAfterMove,
    BeforePointerReplace,
    AfterPointerReplace,
    Cleanup,
}

#[derive(Debug, Eq, PartialEq)]
enum GenerationInstallOutcome {
    Installed,
    Unconfirmed { error: String },
}

#[derive(Debug, Eq, PartialEq)]
enum PointerReplaceOutcome {
    Uncommitted { error: String },
    Committed { durability_pending: bool },
}

#[derive(Debug, Eq, PartialEq)]
struct PublishOutcome {
    generation: String,
    cleanup_pending: bool,
    pointer_durability_pending: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CleanupFault {
    #[default]
    None,
    Fail,
}

#[derive(Debug)]
struct ReconcileOutcome {
    selected: Option<(PathBuf, GoldenManifest)>,
    cleanup_pending: bool,
}

struct GoldenFixtureLock(fs::File);

impl GoldenFixtureLock {
    fn exclusive(container: &Path) -> Result<Self, String> {
        fs::create_dir_all(container).map_err(|_| "cannot create golden fixture root")?;
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(container.join(".golden.lock"))
            .map_err(|_| "cannot open golden fixture coordination file")?;
        file.lock_exclusive()
            .map_err(|_| "cannot lock golden fixture coordination file")?;
        Ok(Self(file))
    }
}

impl Drop for GoldenFixtureLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

pub(super) fn fixture_project() -> Project {
    Project {
        schema_version: PROJECT_SCHEMA_VERSION,
        id: "golden-render-fixture".into(),
        revision: 7,
        name: "Golden render fixture".into(),
        created_at_ms: 0,
        updated_at_ms: 0,
        settings: ProjectSettings {
            width: WIDTH,
            height: HEIGHT,
            fps: FPS,
        },
        assets: vec![crate::Asset {
            id: "tone".into(),
            media_type: MediaType::Audio,
            file_name: "tone.wav".into(),
            project_relative_path: "assets/tone.wav".into(),
            duration_ms: Some(DURATION_MS),
            has_audio: true,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        }],
        tracks: vec![
            Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![
                    TimelineItem::SolidColor(SolidColorItem {
                        id: "background".into(),
                        color: "#cc3311".into(),
                        start_ms: 0,
                        duration_ms: DURATION_MS,
                        visual_properties: crate::VisualProperties::new(
                            Transform {
                                opacity: 0.7,
                                ..Transform::default()
                            },
                            false,
                        ),
                        keyframes: vec![],
                    }),
                    TimelineItem::Text(TextItem {
                        id: "animated-text".into(),
                        text: "café →\nWWWW iiii".into(),
                        start_ms: 0,
                        duration_ms: DURATION_MS,
                        font_size: 20,
                        color: "#ffffff".into(),
                        font_family: None,
                        font_path: None,
                        style: TextStyle {
                            wrap_width_px: Some(100),
                            line_spacing_px: 3,
                            outline_width_px: 1,
                            background_opacity: 0.4,
                            padding: TextPadding {
                                top: 2,
                                right: 4,
                                bottom: 3,
                                left: 5,
                            },
                            anchor: crate::AnchorPoint::Center,
                            ..TextStyle::default()
                        },
                        visual_properties: crate::VisualProperties::new(
                            Transform {
                                position_x: 80.0,
                                position_y: 45.0,
                                scale: 1.0,
                                opacity: 0.8,
                            },
                            false,
                        ),
                        keyframes: vec![
                            Keyframe {
                                property: KeyframeProperty::Scale,
                                time_ms: 0,
                                value: KeyframeValue::Scalar { value: 0.8 },
                                easing: Easing::EaseInOut,
                            },
                            Keyframe {
                                property: KeyframeProperty::Scale,
                                time_ms: DURATION_MS,
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
                                time_ms: DURATION_MS,
                                value: KeyframeValue::Scalar { value: 1.0 },
                                easing: Easing::Linear,
                            },
                        ],
                    }),
                ],
            },
            Track {
                id: "audio".into(),
                name: "Audio".into(),
                track_type: TrackType::Audio,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![TimelineItem::Media(MediaItem {
                    id: "tone-item".into(),
                    asset_id: "tone".into(),
                    start_ms: 0,
                    duration_ms: DURATION_MS,
                    source_in_ms: 0,
                    visual_properties: crate::VisualProperties::default(),
                    audio: AudioSettings::default(),
                    keyframes: vec![],
                })],
            },
        ],
    }
}

fn fixture_container_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/render-golden")
}

fn generation_root(container: &Path, generation: &str) -> PathBuf {
    container.join("generations").join(generation)
}

fn load_pointer(container: &Path) -> Result<GoldenPointer, String> {
    let bytes = fs::read(container.join("CURRENT")).map_err(|_| "golden pointer is missing")?;
    let pointer: GoldenPointer =
        serde_json::from_slice(&bytes).map_err(|_| "golden pointer is malformed")?;
    if pointer.schema_version != POINTER_VERSION || validate_hash(&pointer.generation).is_err() {
        return Err("golden pointer has an unsupported version or invalid generation".into());
    }
    Ok(pointer)
}

fn selected_generation_root(container: &Path) -> Result<PathBuf, String> {
    let pointer = load_pointer(container)?;
    let root = generation_root(container, &pointer.generation);
    let manifest_bytes =
        fs::read(root.join("manifest.json")).map_err(|_| "selected golden manifest is missing")?;
    if hash_bytes(&manifest_bytes) != pointer.generation {
        return Err("selected golden generation digest does not match its manifest".into());
    }
    let manifest: GoldenManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| "selected golden manifest is malformed")?;
    validate_manifest(&root, &manifest, true)?;
    Ok(root)
}

fn configured_native_tools() -> Option<NativeTools> {
    let configured = [
        env::var_os("OPENCUT_FFMPEG_PATH"),
        env::var_os("OPENCUT_FFPROBE_PATH"),
        env::var_os("OPENCUT_TEST_FONT_PATH"),
    ];
    if configured.iter().all(Option::is_none) {
        let native_mode_requested = env::var("OPENCUT_GOLDEN_REQUIRED").as_deref() == Ok("1")
            || env::var("OPENCUT_UPDATE_GOLDENS").as_deref() == Ok("1")
            || env::var_os("OPENCUT_CAPTURE_GOLDENS_TO").is_some()
            || env::var_os("OPENCUT_GOLDEN_REPORT_PATH").is_some();
        assert!(
            !native_mode_requested,
            "golden verification, update, recapture, or reporting was requested but native tool variables are absent"
        );
        return None;
    }
    assert!(
        configured.iter().all(Option::is_some),
        "OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH, and OPENCUT_TEST_FONT_PATH must be configured together"
    );
    let ffmpeg = PathBuf::from(configured[0].clone().unwrap());
    let ffprobe = PathBuf::from(configured[1].clone().unwrap());
    let font = PathBuf::from(configured[2].clone().unwrap());
    assert!(
        font.is_file(),
        "configured golden font must be a readable file"
    );
    let ffmpeg_version = tool_identity(&ffmpeg);
    let ffprobe_version = tool_identity(&ffprobe);
    let font_sha256 = hash_bytes(&fs::read(&font).expect("read configured golden font"));
    Some(NativeTools {
        ffmpeg,
        ffprobe,
        font,
        ffmpeg_version,
        ffprobe_version,
        font_sha256,
    })
}

fn tool_identity(path: &Path) -> String {
    let output = Command::new(path)
        .arg("-version")
        .output()
        .unwrap_or_else(|error| panic!("cannot start configured tool {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "configured tool {} returned failure",
        path.display()
    );
    let identity = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    assert!(!identity.is_empty(), "configured tool identity is empty");
    identity
}

fn write_tone_wav(path: &Path) {
    let samples = AUDIO_SAMPLE_RATE_HZ;
    let data_bytes = samples * 2;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&AUDIO_SAMPLE_RATE_HZ.to_le_bytes());
    bytes.extend_from_slice(&(AUDIO_SAMPLE_RATE_HZ * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for index in 0..samples {
        let phase = (index * 440 * 4 / AUDIO_SAMPLE_RATE_HZ) % 4;
        let within = (index * 440 * 4) % AUDIO_SAMPLE_RATE_HZ;
        let fraction = i32::try_from(within * 16_000 / AUDIO_SAMPLE_RATE_HZ).unwrap();
        let sample = match phase {
            0 => fraction,
            1 => 16_000 - fraction,
            2 => -fraction,
            _ => -16_000 + fraction,
        } as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).expect("write deterministic tone fixture");
}

fn benchmark_intent(
    renderer: &Renderer,
    project: &Project,
    project_root: &Path,
    intent: RenderIntent,
    probe_executor: &dyn BenchmarkProbeExecutor,
) -> Result<IntentObservation, CoreError> {
    let end_to_end_started = Instant::now();
    let evaluation_started = Instant::now();
    let evaluated = evaluate_project(project, WIDTH, HEIGHT, FPS)?;
    let scene_evaluation = evaluation_started.elapsed();

    let filter_started = Instant::now();
    let media = prepare_media_resources(renderer.artifact_io.as_ref(), &evaluated, project_root)?;
    let built = renderer.prepare_render(&evaluated, media, project_root, intent)?;
    let filter_graph_construction = filter_started.elapsed();

    let extension = if matches!(intent, RenderIntent::Frame { .. }) {
        "png"
    } else {
        "mp4"
    };
    let encoded_output = temporary_output(renderer.artifact_io.as_ref(), project_root, extension);
    let encoding_started = Instant::now();
    if let Err(error) = renderer.process_executor.execute(
        &renderer.ffmpeg_path,
        &built.plan,
        &built.filter_path,
        &encoded_output,
        &mut |_| {},
    ) {
        let _ = renderer.artifact_io.remove(&encoded_output);
        return Err(error);
    }
    let encoding = encoding_started.elapsed();
    let end_to_end = end_to_end_started.elapsed();
    renderer
        .artifact_io
        .remove(&encoded_output)
        .map_err(|error| CoreError::io("remove encoded benchmark output", error))?;

    let decoding = if let Some(mut command) =
        build_decode_benchmark_command(&renderer.ffmpeg_path, &built.plan)
    {
        let started = Instant::now();
        probe_executor.execute(
            BenchmarkProbeStage::Decode,
            &mut command,
            built.plan.duration_ms,
        )?;
        started.elapsed()
    } else {
        Duration::ZERO
    };
    let mut composite =
        build_composite_benchmark_command(&renderer.ffmpeg_path, &built.plan, &built.filter_path);
    let composite_started = Instant::now();
    probe_executor.execute(
        BenchmarkProbeStage::Composite,
        &mut composite,
        built.plan.duration_ms,
    )?;
    let compositing = composite_started.elapsed();

    let encoded_audio_streams = match intent {
        RenderIntent::Frame { .. } => 0,
        RenderIntent::Range { include_audio, .. } => u64::from(include_audio),
        RenderIntent::Export => 1,
    };
    let intent_name = match intent {
        RenderIntent::Frame { .. } => "framePreview",
        RenderIntent::Range { .. } => "audiovisualRangePreview",
        RenderIntent::Export => "finalExport",
    };
    Ok(IntentObservation {
        intent: intent_name.into(),
        timings_ms: StageTimings {
            scene_evaluation: duration_ms(scene_evaluation),
            rasterization: 0.0,
            filter_graph_construction: duration_ms(filter_graph_construction),
            decoding: duration_ms(decoding),
            compositing: duration_ms(compositing),
            encoding: duration_ms(encoding),
            end_to_end: duration_ms(end_to_end),
        },
        work: StageWork {
            decoded_inputs: built.plan.media_inputs.len() as u64,
            rasterized_layers: 0,
            composited_layers: evaluated.scene.visual_layers.len() as u64,
            encoded_video_streams: 1,
            encoded_audio_streams,
        },
    })
}

fn benchmark_intent_observations(
    renderer: &Renderer,
    project: &Project,
    project_root: &Path,
    probe_executor: &dyn BenchmarkProbeExecutor,
) -> Result<Vec<IntentObservation>, CoreError> {
    [
        RenderIntent::Frame { at_ms: 500 },
        RenderIntent::Range {
            start_ms: 0,
            end_ms: DURATION_MS,
            include_audio: true,
        },
        RenderIntent::Export,
    ]
    .into_iter()
    .map(|intent| benchmark_intent(renderer, project, project_root, intent, probe_executor))
    .collect()
}

fn capture_intent_observations(
    mode: CaptureMode,
    renderer: &Renderer,
    project: &Project,
    project_root: &Path,
    probe_executor: &dyn BenchmarkProbeExecutor,
) -> Result<Vec<IntentObservation>, CoreError> {
    match mode {
        CaptureMode::Conformance => Ok(Vec::new()),
        CaptureMode::Benchmark { .. } => {
            benchmark_intent_observations(renderer, project, project_root, probe_executor)
        }
    }
}

fn capture(tools: &NativeTools, mode: CaptureMode) -> Capture {
    let memory_sampler = match mode {
        CaptureMode::Benchmark {
            sample_memory: true,
        } => Some(ProcessTreeSampler::start()),
        CaptureMode::Conformance
        | CaptureMode::Benchmark {
            sample_memory: false,
        } => None,
    };
    let root = tempdir().expect("create golden render root");
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let project = fixture_project();
    let project_before = serde_json::to_vec(&project).expect("serialize golden project");
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    renderer.readiness().expect("golden renderer readiness");

    let intent_observations = capture_intent_observations(
        mode,
        &renderer,
        &project,
        root.path(),
        &NativeBenchmarkProbeExecutor,
    )
    .expect("capture benchmark intent observations");

    let evaluated = evaluate_project(&project, WIDTH, HEIGHT, FPS).expect("evaluate golden scene");
    let semantic_plan = format!("{:#?}\n", evaluated.scene);

    let mut intent_graphs = Vec::new();
    for intent in [
        RenderIntent::Frame { at_ms: 500 },
        RenderIntent::Range {
            start_ms: 0,
            end_ms: DURATION_MS,
            include_audio: true,
        },
        RenderIntent::Export,
    ] {
        let media = prepare_media_resources(renderer.artifact_io.as_ref(), &evaluated, root.path())
            .expect("prepare golden media");
        let prepared = renderer
            .prepare_render(&evaluated, media, root.path(), intent)
            .expect("build golden filter graph");
        intent_graphs.push(normalize_filter_graph(
            &prepared.plan.filter_graph,
            root.path(),
            &tools.font,
        ));
    }
    assert!(
        intent_graphs
            .windows(2)
            .all(|graphs| graphs[0] == graphs[1]),
        "frame, range, and export filter graphs must share exact semantics"
    );
    let filter_graph = intent_graphs.pop().unwrap();

    let mut frames = BTreeMap::new();
    for at_ms in SAMPLE_TIMESTAMPS_MS {
        let artifact = renderer
            .render_preview(&project, root.path(), at_ms)
            .expect("render golden preview frame");
        frames.insert(
            at_ms,
            decode_rgb_frame(&tools.ffmpeg, &root.path().join(artifact.relative_path), 0),
        );
    }
    let repeated_evaluation =
        evaluate_project(&project, WIDTH, HEIGHT, FPS).expect("repeat golden scene evaluation");
    assert_eq!(
        repeated_evaluation, evaluated,
        "repeated golden evaluation must be exact"
    );
    let repeated_artifact = renderer
        .render_preview(&project, root.path(), 500)
        .expect("repeat golden preview frame");
    let repeated_frame = decode_rgb_frame(
        &tools.ffmpeg,
        &root.path().join(repeated_artifact.relative_path),
        0,
    );
    assert!(
        structural_similarity(frames.get(&500).unwrap(), &repeated_frame).unwrap() >= SSIM_MINIMUM,
        "repeated golden rendering drifted"
    );
    let range = renderer
        .render_preview_range(
            &project,
            root.path(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: WIDTH,
                height: HEIGHT,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .expect("render golden range");
    let range_path = root.path().join(range.relative_path);

    let export_path = root.path().join("export.mp4");
    renderer
        .export_video(
            &project,
            root.path(),
            ExportOptions {
                output: &export_path,
                width: WIDTH,
                height: HEIGHT,
                overwrite: false,
            },
            |_| {},
        )
        .expect("render golden export");

    let range_frames = SAMPLE_TIMESTAMPS_MS
        .into_iter()
        .map(|at_ms| (at_ms, decode_rgb_frame(&tools.ffmpeg, &range_path, at_ms)))
        .collect();
    let export_frames = SAMPLE_TIMESTAMPS_MS
        .into_iter()
        .map(|at_ms| (at_ms, decode_rgb_frame(&tools.ffmpeg, &export_path, at_ms)))
        .collect();
    let range_audio = decode_mono_f32(&tools.ffmpeg, &range_path);
    let export_audio = decode_mono_f32(&tools.ffmpeg, &export_path);
    let range_duration_ms = renderer
        .probe(&range_path)
        .expect("probe golden range")
        .duration_ms
        .expect("golden range duration");
    let export_duration_ms = renderer
        .probe(&export_path)
        .expect("probe golden export")
        .duration_ms
        .expect("golden export duration");
    assert_eq!(
        serde_json::to_vec(&project).expect("serialize rendered golden project"),
        project_before,
        "successful rendering mutated the fixture project"
    );

    let peak_process_tree_bytes = memory_sampler.map_or(0, ProcessTreeSampler::finish);
    Capture {
        frames,
        range_frames,
        export_frames,
        range_audio,
        export_audio,
        range_duration_ms,
        export_duration_ms,
        semantic_plan,
        filter_graph,
        intent_observations,
        peak_process_tree_bytes,
    }
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

struct ProcessTreeSampler {
    stop: Arc<AtomicBool>,
    peak: Arc<AtomicU64>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ProcessTreeSampler {
    fn start() -> Self {
        Self::start_with_exit_signal(None)
    }

    fn start_with_exit_signal(exit_signal: Option<Arc<AtomicBool>>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let peak = Arc::new(AtomicU64::new(0));
        let sampler_stop = Arc::clone(&stop);
        let sampler_peak = Arc::clone(&peak);
        let root = Pid::from_u32(std::process::id());
        let handle = thread::spawn(move || {
            let mut system = System::new();
            loop {
                system.refresh_processes(ProcessesToUpdate::All, true);
                let mut members = BTreeSet::from([root]);
                loop {
                    let before = members.len();
                    for (pid, process) in system.processes() {
                        if process.thread_kind().is_none()
                            && process
                                .parent()
                                .is_some_and(|parent| members.contains(&parent))
                        {
                            members.insert(*pid);
                        }
                    }
                    if members.len() == before {
                        break;
                    }
                }
                let resident = members
                    .iter()
                    .filter_map(|pid| system.process(*pid))
                    .filter(|process| process.thread_kind().is_none())
                    .map(sysinfo::Process::memory)
                    .sum();
                sampler_peak.fetch_max(resident, Ordering::Relaxed);
                if sampler_stop.load(Ordering::Acquire) {
                    break;
                }
                thread::sleep(MEMORY_SAMPLE_INTERVAL);
            }
            if let Some(exit_signal) = exit_signal {
                exit_signal.store(true, Ordering::Release);
            }
        });
        Self {
            stop,
            peak,
            handle: Some(handle),
        }
    }

    fn stop_and_join(&mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Release);
            handle.join()
        } else {
            Ok(())
        }
    }

    fn finish(mut self) -> u64 {
        self.stop_and_join()
            .expect("join process-tree memory sampler");
        self.peak.load(Ordering::Relaxed)
    }
}

impl Drop for ProcessTreeSampler {
    fn drop(&mut self) {
        let _ = self.stop_and_join();
    }
}

fn deterministic_capture_matches(left: &Capture, right: &Capture) -> bool {
    left.frames == right.frames
        && left.range_frames == right.range_frames
        && left.export_frames == right.export_frames
        && left.range_audio == right.range_audio
        && left.export_audio == right.export_audio
        && left.range_duration_ms == right.range_duration_ms
        && left.export_duration_ms == right.export_duration_ms
        && left.semantic_plan == right.semantic_plan
        && left.filter_graph == right.filter_graph
}

fn median(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty(), "median requires at least one sample");
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn aggregate_performance(tools: &NativeTools, captures: &[Capture]) -> PerformanceBaseline {
    assert_eq!(captures.len(), MEASURED_SAMPLES as usize);
    let mut intent_observations = Vec::new();
    for index in 0..captures[0].intent_observations.len() {
        let first = &captures[0].intent_observations[index];
        assert!(captures.iter().all(|capture| {
            capture.intent_observations[index].intent == first.intent
                && capture.intent_observations[index].work.decoded_inputs
                    == first.work.decoded_inputs
                && capture.intent_observations[index].work.rasterized_layers
                    == first.work.rasterized_layers
                && capture.intent_observations[index].work.composited_layers
                    == first.work.composited_layers
                && capture.intent_observations[index]
                    .work
                    .encoded_video_streams
                    == first.work.encoded_video_streams
                && capture.intent_observations[index]
                    .work
                    .encoded_audio_streams
                    == first.work.encoded_audio_streams
        }));
        let phase = |select: fn(&StageTimings) -> f64| {
            median(
                captures
                    .iter()
                    .map(|capture| select(&capture.intent_observations[index].timings_ms))
                    .collect(),
            )
        };
        intent_observations.push(IntentObservation {
            intent: first.intent.clone(),
            timings_ms: StageTimings {
                scene_evaluation: phase(|timings| timings.scene_evaluation),
                rasterization: phase(|timings| timings.rasterization),
                filter_graph_construction: phase(|timings| timings.filter_graph_construction),
                decoding: phase(|timings| timings.decoding),
                compositing: phase(|timings| timings.compositing),
                encoding: phase(|timings| timings.encoding),
                end_to_end: phase(|timings| timings.end_to_end),
            },
            work: first.work.clone(),
        });
    }
    PerformanceBaseline {
        schema_version: PERFORMANCE_SCHEMA_VERSION,
        fixture_id: FIXTURE_ID.into(),
        fixture_revision: FIXTURE_REVISION,
        git_revision: git_revision(),
        os: env::consts::OS.into(),
        architecture: env::consts::ARCH.into(),
        ffmpeg_version: tools.ffmpeg_version.clone(),
        ffprobe_version: tools.ffprobe_version.clone(),
        font_sha256: tools.font_sha256.clone(),
        units: BaselineUnits {
            timing: "milliseconds".into(),
            memory: "bytes".into(),
        },
        warmup_samples: WARMUP_SAMPLES,
        measured_samples: MEASURED_SAMPLES,
        memory_scope: "process_tree".into(),
        timing_aggregation: "median".into(),
        memory_aggregation: "maximum".into(),
        stage_definition_version: STAGE_DEFINITION_VERSION,
        non_additive_stage_timings: true,
        intent_observations,
        peak_resident_working_set_bytes: captures
            .iter()
            .map(|capture| capture.peak_process_tree_bytes)
            .max()
            .unwrap_or(0),
        comparison_policy: "report_only_compare_matching_environment_identity".into(),
    }
}

fn sampled_capture(tools: &NativeTools) -> (Capture, PerformanceBaseline) {
    for _ in 0..WARMUP_SAMPLES {
        let _ = capture(
            tools,
            CaptureMode::Benchmark {
                sample_memory: false,
            },
        );
    }
    let mut captures = (0..MEASURED_SAMPLES)
        .map(|_| {
            capture(
                tools,
                CaptureMode::Benchmark {
                    sample_memory: true,
                },
            )
        })
        .collect::<Vec<_>>();
    let first = captures.first().expect("measured golden capture");
    assert!(
        captures
            .iter()
            .skip(1)
            .all(|capture| deterministic_capture_matches(first, capture)),
        "measured golden captures produced different deterministic evidence"
    );
    let performance = aggregate_performance(tools, &captures);
    validate_performance_baseline(&performance).expect("validate aggregated performance report");
    (captures.remove(0), performance)
}

fn decode_rgb_frame(ffmpeg: &Path, path: &Path, time_ms: u64) -> Vec<u8> {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{:.3}", time_ms as f64 / 1_000.0))
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
        .expect("decode golden RGB frame");
    assert!(output.status.success(), "golden RGB decode failed");
    assert_eq!(output.stdout.len(), (WIDTH * HEIGHT * 3) as usize);
    output.stdout
}

fn decode_mono_f32(ffmpeg: &Path, path: &Path) -> Vec<f32> {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-map",
            "0:a:0",
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "-ar",
            "48000",
            "-ac",
            "1",
            "pipe:1",
        ])
        .output()
        .expect("decode golden audio");
    assert!(output.status.success(), "golden audio decode failed");
    let (samples, remainder) = output.stdout.as_chunks::<4>();
    assert!(remainder.is_empty(), "decoded audio has a partial sample");
    samples
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect()
}

fn normalize_filter_graph(graph: &str, project_root: &Path, font: &Path) -> String {
    let mut normalized = graph.replace('\\', "/");
    for (path, token) in [(project_root, "<PROJECT_ROOT>"), (font, "<FONT>")] {
        let path = path.to_string_lossy().replace('\\', "/");
        normalized = normalized.replace(&path, token);
        normalized = normalized.replace(&path.replace(':', "/:"), token);
    }
    normalized = normalize_textfile_arguments(&normalized);
    normalize_workspace_segments(&normalized)
}

fn normalize_textfile_arguments(value: &str) -> String {
    let mut remaining = value;
    let mut result = String::with_capacity(value.len());
    const PREFIX: &str = "textfile='";
    while let Some(start) = remaining.find(PREFIX) {
        let value_start = start + PREFIX.len();
        result.push_str(&remaining[..value_start]);
        let suffix = &remaining[value_start..];
        let Some(end) = suffix.find('\'') else {
            result.push_str(suffix);
            return result;
        };
        let path = &suffix[..end];
        let file_name = path.rsplit('/').next().unwrap_or(path);
        result.push_str("<WORKSPACE>/");
        result.push_str(file_name);
        remaining = &suffix[end..];
    }
    result.push_str(remaining);
    result
}

fn normalize_workspace_segments(value: &str) -> String {
    let mut remaining = value;
    let mut result = String::with_capacity(value.len());
    const PREFIX: &str = ".opencut-work-";
    while let Some(start) = remaining.find(PREFIX) {
        result.push_str(&remaining[..start]);
        result.push_str(".opencut-work-<REQUEST_ID>");
        let suffix = &remaining[start + PREFIX.len()..];
        let end = suffix.find(['/', '\'', ':', ',']).unwrap_or(suffix.len());
        remaining = &suffix[end..];
    }
    result.push_str(remaining);
    result
}

fn validate_manifest(
    root: &Path,
    manifest: &GoldenManifest,
    verify_hashes: bool,
) -> Result<(), String> {
    validate_manifest_with_performance(
        root,
        manifest,
        FIXTURE_REVISION,
        verify_hashes,
        validate_current_performance_bytes,
    )
}

fn validate_manifest_with_performance(
    root: &Path,
    manifest: &GoldenManifest,
    expected_fixture_revision: u32,
    verify_hashes: bool,
    validate_performance_bytes: fn(&[u8]) -> Result<(), String>,
) -> Result<(), String> {
    if manifest.schema_version != MANIFEST_VERSION {
        return Err("unknown golden manifest version".into());
    }
    if manifest.fixture_id != FIXTURE_ID || manifest.fixture_revision != expected_fixture_revision {
        return Err("golden fixture identity differs".into());
    }
    if (
        manifest.canvas.width,
        manifest.canvas.height,
        manifest.canvas.fps,
    ) != (WIDTH, HEIGHT, FPS)
        || manifest.duration_ms != DURATION_MS
        || manifest.audio.sample_rate_hz != AUDIO_SAMPLE_RATE_HZ
        || manifest.audio.channels != 1
        || manifest.audio.sample_format != "f32le"
    {
        return Err("golden media parameters differ".into());
    }
    if manifest.sample_timestamps_ms != SAMPLE_TIMESTAMPS_MS {
        return Err("golden sample timestamps differ or are not unique and ordered".into());
    }
    let tolerance_values = [
        manifest.tolerances.minimum_ssim,
        manifest.tolerances.maximum_pcm_rms_error,
        f64::from(manifest.tolerances.maximum_timing_frames),
    ];
    if tolerance_values.iter().any(|value| !value.is_finite())
        || manifest.tolerances.minimum_ssim != SSIM_MINIMUM
        || manifest.tolerances.maximum_pcm_rms_error != PCM_RMS_MAXIMUM
        || manifest.tolerances.maximum_timing_frames != 1
    {
        return Err("golden tolerances are non-finite or out of range".into());
    }
    for value in [
        &manifest.environment.reference_os,
        &manifest.environment.reference_arch,
        &manifest.environment.ffmpeg_version,
        &manifest.environment.ffprobe_version,
    ] {
        if value.trim().is_empty() {
            return Err("golden environment identity is incomplete".into());
        }
    }
    validate_hash(&manifest.environment.font_sha256)?;

    let mut paths = BTreeSet::new();
    let mut frame_times = BTreeSet::new();
    let mut audio = 0;
    let mut semantic = 0;
    let mut graph = 0;
    let mut performance = 0;
    for reference in &manifest.references {
        let path = safe_reference_path(root, reference.path())?;
        if !paths.insert(reference.path()) {
            return Err("duplicate golden reference path".into());
        }
        validate_hash(reference.sha256())?;
        match reference {
            GoldenReference::Frame { at_ms, .. } => {
                if !SAMPLE_TIMESTAMPS_MS.contains(at_ms) || !frame_times.insert(*at_ms) {
                    return Err("duplicate or unexpected golden frame timestamp".into());
                }
            }
            GoldenReference::Audio { .. } => audio += 1,
            GoldenReference::SemanticPlan { .. } => semantic += 1,
            GoldenReference::FilterGraph { .. } => graph += 1,
            GoldenReference::PerformanceBaseline { .. } => performance += 1,
        }
        if verify_hashes {
            let bytes = fs::read(&path).map_err(|_| "golden reference is missing")?;
            if hash_bytes(&bytes) != reference.sha256() {
                return Err("golden reference hash mismatch".into());
            }
            if matches!(reference, GoldenReference::PerformanceBaseline { .. }) {
                validate_performance_bytes(&bytes)?;
            }
        }
    }
    if frame_times != SAMPLE_TIMESTAMPS_MS.into_iter().collect()
        || (audio, semantic, graph, performance) != (1, 1, 1, 1)
    {
        return Err("golden reference set is incomplete".into());
    }
    Ok(())
}

fn validate_migration_source(root: &Path, manifest: &GoldenManifest) -> Result<(), String> {
    validate_manifest_with_performance(
        root,
        manifest,
        MIGRATION_FIXTURE_REVISION,
        true,
        validate_legacy_performance_bytes,
    )
}

fn validate_current_performance_bytes(bytes: &[u8]) -> Result<(), String> {
    let baseline: PerformanceBaseline =
        serde_json::from_slice(bytes).map_err(|_| "performance baseline is malformed")?;
    validate_performance_baseline(&baseline)
}

fn validate_legacy_performance_bytes(bytes: &[u8]) -> Result<(), String> {
    let baseline: LegacyPerformanceBaseline = serde_json::from_slice(bytes)
        .map_err(|_| "golden migration performance report is malformed")?;
    validate_legacy_performance_baseline(&baseline)
}

fn validate_legacy_performance_baseline(
    baseline: &LegacyPerformanceBaseline,
) -> Result<(), String> {
    let valid_git_revision = baseline.git_revision.is_null()
        || baseline
            .git_revision
            .as_str()
            .is_some_and(|revision| !revision.trim().is_empty());
    if baseline.schema_version != MIGRATION_PERFORMANCE_SCHEMA_VERSION
        || baseline.fixture_id != FIXTURE_ID
        || baseline.fixture_revision != MIGRATION_FIXTURE_REVISION
        || !valid_git_revision
        || baseline.os.trim().is_empty()
        || baseline.architecture.trim().is_empty()
        || baseline.ffmpeg_version.trim().is_empty()
        || baseline.ffprobe_version.trim().is_empty()
        || baseline.units.timing != "milliseconds"
        || baseline.units.memory != "bytes"
        || baseline.warmup_samples != WARMUP_SAMPLES
        || baseline.measured_samples != MEASURED_SAMPLES
        || baseline.memory_scope != "process_tree"
        || baseline.timing_aggregation != "median"
        || baseline.memory_aggregation != "maximum"
        || baseline.comparison_policy != "report_only_compare_matching_environment_identity"
    {
        return Err("golden migration performance report is incomplete or unsupported".into());
    }
    validate_hash(&baseline.font_sha256)?;
    let timings = &baseline.timings_ms;
    let values = [
        timings.scene_evaluation,
        timings.filter_graph_construction,
        timings.frame_rendering,
        timings.audiovisual_range_rendering,
        timings.export_rendering,
        timings.total,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("golden migration timings are non-finite or negative".into());
    }
    Ok(())
}

fn validate_performance_baseline(baseline: &PerformanceBaseline) -> Result<(), String> {
    let valid_git_revision = baseline
        .git_revision
        .as_ref()
        .is_none_or(|revision| !revision.trim().is_empty());
    if baseline.schema_version != PERFORMANCE_SCHEMA_VERSION
        || baseline.fixture_id != FIXTURE_ID
        || baseline.fixture_revision != FIXTURE_REVISION
        || !valid_git_revision
        || baseline.warmup_samples != WARMUP_SAMPLES
        || baseline.measured_samples != MEASURED_SAMPLES
        || baseline.memory_scope != "process_tree"
        || baseline.timing_aggregation != "median"
        || baseline.memory_aggregation != "maximum"
        || baseline.stage_definition_version != STAGE_DEFINITION_VERSION
        || !baseline.non_additive_stage_timings
        || baseline.os.trim().is_empty()
        || baseline.architecture.trim().is_empty()
        || baseline.ffmpeg_version.trim().is_empty()
        || baseline.ffprobe_version.trim().is_empty()
        || baseline.units.timing != "milliseconds"
        || baseline.units.memory != "bytes"
        || baseline.comparison_policy != "report_only_compare_matching_environment_identity"
    {
        return Err("performance baseline is incomplete, non-finite, or out of range".into());
    }
    validate_hash(&baseline.font_sha256)?;
    let mut intents = BTreeSet::new();
    for observation in &baseline.intent_observations {
        if !intents.insert(observation.intent.as_str()) {
            return Err("performance baseline contains a duplicate intent".into());
        }
        let expected_work = expected_stage_work(&observation.intent)?;
        let timings = &observation.timings_ms;
        let values = [
            timings.scene_evaluation,
            timings.rasterization,
            timings.filter_graph_construction,
            timings.decoding,
            timings.compositing,
            timings.encoding,
            timings.end_to_end,
        ];
        if values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
            || (observation.work.rasterized_layers == 0 && timings.rasterization != 0.0)
            || (observation.work.decoded_inputs == 0 && timings.decoding != 0.0)
        {
            return Err("performance stage observation is inconsistent".into());
        }
        if observation.work != expected_work {
            return Err("performance stage work differs from the canonical fixture".into());
        }
    }
    let expected = BTreeSet::from(["framePreview", "audiovisualRangePreview", "finalExport"]);
    if intents != expected {
        return Err("performance baseline intent set is incomplete".into());
    }
    Ok(())
}

fn expected_stage_work(intent: &str) -> Result<StageWork, String> {
    let encoded_audio_streams = match intent {
        "framePreview" => 0,
        "audiovisualRangePreview" | "finalExport" => 1,
        _ => return Err("performance baseline contains an unknown intent".into()),
    };
    Ok(StageWork {
        decoded_inputs: 1,
        rasterized_layers: 0,
        composited_layers: 2,
        encoded_video_streams: 1,
        encoded_audio_streams,
    })
}

fn validate_performance_report_file(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|_| "performance report is missing")?;
    let baseline: PerformanceBaseline = serde_json::from_slice(&bytes)
        .map_err(|_| "performance report is not strict schema-3 JSON")?;
    validate_performance_baseline(&baseline)
}

fn write_performance_report(path: &Path, baseline: &PerformanceBaseline) -> Result<(), String> {
    validate_performance_baseline(baseline)?;
    let bytes =
        serde_json::to_vec_pretty(baseline).map_err(|_| "cannot serialize performance report")?;
    fs::write(path, bytes).map_err(|_| "cannot write performance report")?;
    validate_performance_report_file(path)
}

fn safe_reference_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() || has_uri_scheme(relative) {
        return Err("golden reference path is empty or a URI".into());
    }
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("golden reference path is not a safe relative path".into());
    }
    let candidate = root.join(relative);
    if root.exists() && candidate.exists() {
        let canonical_root = root
            .canonicalize()
            .map_err(|_| "cannot canonicalize golden root")?;
        let canonical = candidate
            .canonicalize()
            .map_err(|_| "cannot canonicalize golden reference")?;
        if !canonical.starts_with(canonical_root) {
            return Err("golden reference escapes fixture root".into());
        }
    }
    Ok(candidate)
}

fn has_uri_scheme(value: &str) -> bool {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic()) {
        return false;
    }
    bytes
        .take_while(|byte| *byte != b'/' && *byte != b'\\')
        .try_fold(false, |_, byte| match byte {
            b':' => Err(true),
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.') => Ok(false),
            _ => Err(false),
        })
        .unwrap_or_else(|is_scheme| is_scheme)
}

fn validate_hash(value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("golden reference hash is not lowercase SHA-256".into());
    }
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load_manifest(root: &Path) -> Result<GoldenManifest, String> {
    let bytes = fs::read(root.join("manifest.json")).map_err(|_| "golden manifest is missing")?;
    let manifest: GoldenManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid golden manifest: {error}"))?;
    validate_manifest(root, &manifest, true)?;
    Ok(manifest)
}

fn reference_bytes(
    root: &Path,
    manifest: &GoldenManifest,
    kind: &str,
    at_ms: Option<u64>,
) -> Vec<u8> {
    let reference = manifest
        .references
        .iter()
        .find(|reference| match (kind, reference) {
            (
                "frame",
                GoldenReference::Frame {
                    at_ms: candidate, ..
                },
            ) => Some(*candidate) == at_ms,
            ("audio", GoldenReference::Audio { .. })
            | ("semantic", GoldenReference::SemanticPlan { .. })
            | ("graph", GoldenReference::FilterGraph { .. })
            | ("performance", GoldenReference::PerformanceBaseline { .. }) => true,
            _ => false,
        })
        .expect("validated reference must exist");
    fs::read(root.join(reference.path())).expect("read validated golden reference")
}

fn compare_capture(
    root: &Path,
    manifest: &GoldenManifest,
    capture: &Capture,
) -> Result<(), String> {
    for at_ms in SAMPLE_TIMESTAMPS_MS {
        let expected = reference_bytes(root, manifest, "frame", Some(at_ms));
        for (intent, frames) in [
            ("frame", &capture.frames),
            ("range", &capture.range_frames),
            ("export", &capture.export_frames),
        ] {
            let actual = frames.get(&at_ms).ok_or("captured frame is missing")?;
            let ssim = structural_similarity(&expected, actual)?;
            if ssim < manifest.tolerances.minimum_ssim {
                return Err(format!("{intent} frame at {at_ms}ms SSIM was {ssim}"));
            }
        }
    }
    let expected_audio = bytes_to_f32(&reference_bytes(root, manifest, "audio", None))?;
    for (intent, actual) in [
        ("range", &capture.range_audio),
        ("export", &capture.export_audio),
    ] {
        let rms = aligned_rms_error(
            &expected_audio,
            actual,
            (AUDIO_SAMPLE_RATE_HZ / FPS) as usize,
        )?;
        if rms > manifest.tolerances.maximum_pcm_rms_error {
            return Err(format!("{intent} decoded audio RMS error was {rms}"));
        }
    }
    let maximum_timing_ms =
        u64::from(manifest.tolerances.maximum_timing_frames) * 1_000 / u64::from(FPS);
    for (intent, duration) in [
        ("range", capture.range_duration_ms),
        ("export", capture.export_duration_ms),
    ] {
        if duration.abs_diff(DURATION_MS) > maximum_timing_ms {
            return Err(format!("{intent} timing differed by more than one frame"));
        }
    }
    let semantic = reference_bytes(root, manifest, "semantic", None);
    if semantic != capture.semantic_plan.as_bytes() {
        return Err("semantic plan differs from reviewed golden".into());
    }
    let graph = reference_bytes(root, manifest, "graph", None);
    if graph != capture.filter_graph.as_bytes() {
        return Err("filter graph differs from reviewed golden".into());
    }
    Ok(())
}

fn structural_similarity(left: &[u8], right: &[u8]) -> Result<f64, String> {
    if left.len() != right.len() || left.is_empty() {
        return Err("golden frame dimensions differ".into());
    }
    let count = left.len() as f64;
    let left_mean = left.iter().map(|value| f64::from(*value)).sum::<f64>() / count;
    let right_mean = right.iter().map(|value| f64::from(*value)).sum::<f64>() / count;
    let (left_variance, right_variance, covariance) =
        left.iter()
            .zip(right)
            .fold((0.0, 0.0, 0.0), |totals, (left, right)| {
                let left_delta = f64::from(*left) - left_mean;
                let right_delta = f64::from(*right) - right_mean;
                (
                    totals.0 + left_delta * left_delta,
                    totals.1 + right_delta * right_delta,
                    totals.2 + left_delta * right_delta,
                )
            });
    let c1 = (0.01_f64 * 255.0).powi(2);
    let c2 = (0.03_f64 * 255.0).powi(2);
    Ok(
        ((2.0 * left_mean * right_mean + c1) * (2.0 * covariance / count + c2))
            / ((left_mean.powi(2) + right_mean.powi(2) + c1)
                * ((left_variance + right_variance) / count + c2)),
    )
}

fn bytes_to_f32(bytes: &[u8]) -> Result<Vec<f32>, String> {
    let (samples, remainder) = bytes.as_chunks::<4>();
    if !remainder.is_empty() {
        return Err("golden audio has a partial float sample".into());
    }
    let values = samples
        .iter()
        .map(|sample| f32::from_le_bytes(*sample))
        .collect::<Vec<_>>();
    if values.iter().any(|value| !value.is_finite()) {
        return Err("golden audio contains a non-finite sample".into());
    }
    Ok(values)
}

fn aligned_rms_error(left: &[f32], right: &[f32], maximum_offset: usize) -> Result<f64, String> {
    if left.is_empty()
        || right.is_empty()
        || left.len().abs_diff(right.len()) > maximum_offset
        || left.iter().chain(right).any(|sample| !sample.is_finite())
    {
        return Err("decoded audio length differs by more than one frame".into());
    }
    let minimum_overlap = left.len().min(right.len()).saturating_sub(maximum_offset);
    if minimum_overlap == 0 {
        return Err("decoded audio has insufficient overlap".into());
    }
    let maximum_offset =
        isize::try_from(maximum_offset).map_err(|_| "audio offset is too large")?;
    (-maximum_offset..=maximum_offset)
        .filter_map(|offset| {
            let (left_start, right_start) = if offset < 0 {
                (offset.unsigned_abs(), 0)
            } else {
                (0, offset as usize)
            };
            let count = (left.len().saturating_sub(left_start))
                .min(right.len().saturating_sub(right_start));
            (count >= minimum_overlap).then(|| {
                (left[left_start..left_start + count]
                    .iter()
                    .zip(&right[right_start..right_start + count])
                    .map(|(left, right)| f64::from(left - right).powi(2))
                    .sum::<f64>()
                    / count as f64)
                    .sqrt()
            })
        })
        .min_by(f64::total_cmp)
        .ok_or_else(|| "decoded audio has no shared samples".into())
}

fn write_capture_set(
    root: &Path,
    tools: &NativeTools,
    capture: &Capture,
    performance: &PerformanceBaseline,
) -> GoldenManifest {
    fs::create_dir_all(root.join("frames")).expect("create golden frame directory");
    fs::create_dir_all(root.join("audio")).expect("create golden audio directory");
    let mut references = Vec::new();
    for (at_ms, bytes) in &capture.export_frames {
        let path = format!("frames/{at_ms:04}.rgb");
        fs::write(root.join(&path), bytes).expect("write golden frame");
        references.push(GoldenReference::Frame {
            at_ms: *at_ms,
            path,
            sha256: hash_bytes(bytes),
        });
    }
    let audio_bytes = capture
        .export_audio
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect::<Vec<_>>();
    let audio_path = "audio/reference.f32le".to_owned();
    fs::write(root.join(&audio_path), &audio_bytes).expect("write golden audio");
    references.push(GoldenReference::Audio {
        path: audio_path,
        sha256: hash_bytes(&audio_bytes),
    });
    for (kind, path, bytes) in [
        (
            "semantic",
            "semantic-plan.txt",
            capture.semantic_plan.as_bytes(),
        ),
        ("graph", "filter-graph.txt", capture.filter_graph.as_bytes()),
    ] {
        fs::write(root.join(path), bytes).expect("write golden text reference");
        let reference = match kind {
            "semantic" => GoldenReference::SemanticPlan {
                path: path.into(),
                sha256: hash_bytes(bytes),
            },
            _ => GoldenReference::FilterGraph {
                path: path.into(),
                sha256: hash_bytes(bytes),
            },
        };
        references.push(reference);
    }
    let performance_bytes = serde_json::to_vec_pretty(performance).unwrap();
    let performance_path = "performance-baseline.json".to_owned();
    fs::write(root.join(&performance_path), &performance_bytes)
        .expect("write performance baseline");
    references.push(GoldenReference::PerformanceBaseline {
        path: performance_path,
        sha256: hash_bytes(&performance_bytes),
    });
    let manifest = GoldenManifest {
        schema_version: MANIFEST_VERSION,
        fixture_id: FIXTURE_ID.into(),
        fixture_revision: FIXTURE_REVISION,
        canvas: GoldenCanvas {
            width: WIDTH,
            height: HEIGHT,
            fps: FPS,
        },
        duration_ms: DURATION_MS,
        sample_timestamps_ms: SAMPLE_TIMESTAMPS_MS.into(),
        audio: GoldenAudio {
            sample_rate_hz: AUDIO_SAMPLE_RATE_HZ,
            channels: 1,
            sample_format: "f32le".into(),
        },
        tolerances: GoldenTolerances {
            minimum_ssim: SSIM_MINIMUM,
            maximum_pcm_rms_error: PCM_RMS_MAXIMUM,
            maximum_timing_frames: 1,
        },
        environment: GoldenEnvironment {
            reference_os: env::consts::OS.into(),
            reference_arch: env::consts::ARCH.into(),
            ffmpeg_version: tools.ffmpeg_version.clone(),
            ffprobe_version: tools.ffprobe_version.clone(),
            font_sha256: tools.font_sha256.clone(),
        },
        references,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(root.join("manifest.json"), manifest_bytes).expect("write golden manifest");
    manifest
}

struct StageGuard {
    path: PathBuf,
    armed: bool,
}

impl StageGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for StageGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn pointer_bytes(generation: &str) -> Vec<u8> {
    serde_json::to_vec_pretty(&GoldenPointer {
        schema_version: POINTER_VERSION,
        generation: generation.into(),
    })
    .expect("serialize golden pointer")
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or("golden pointer has no parent")?;
    sync_directory(parent, "cannot sync golden pointer parent")
}

#[cfg(unix)]
fn sync_directory(path: &Path, error: &'static str) -> Result<(), String> {
    let directory = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|_| error)?;
    directory.sync_all().map_err(|_| error.into())
}

fn sync_generation_content(
    root: &Path,
    manifest: &GoldenManifest,
    fault: PublishFault,
) -> Result<(), String> {
    if fault == PublishFault::GenerationContentSync {
        return Err("injected failure while syncing golden generation content".into());
    }
    let mut retained = manifest
        .references
        .iter()
        .map(|reference| safe_reference_path(root, reference.path()))
        .collect::<Result<Vec<_>, _>>()?;
    retained.push(root.join("manifest.json"));
    for path in &retained {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|_| "cannot open golden generation content for synchronization")?
            .sync_all()
            .map_err(|_| "cannot sync golden generation content")?;
    }
    #[cfg(unix)]
    {
        for directory in generation_directories_deepest_first(root, &retained)? {
            sync_directory(&directory, "cannot sync golden generation directory")?;
        }
    }
    Ok(())
}

fn generation_directories_deepest_first(
    root: &Path,
    retained: &[PathBuf],
) -> Result<Vec<PathBuf>, String> {
    let mut directories = BTreeSet::new();
    for path in retained {
        let mut directory = path
            .parent()
            .ok_or("golden generation content has no parent")?;
        loop {
            directories.insert(directory.to_path_buf());
            if directory == root {
                break;
            }
            directory = directory
                .parent()
                .ok_or("golden generation directory does not reach its root")?;
        }
    }
    directories.insert(root.to_path_buf());
    let mut directories = directories.into_iter().collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    Ok(directories)
}

#[cfg(unix)]
fn install_generation(
    staged: &Path,
    installed: &Path,
    fault: PublishFault,
) -> GenerationInstallOutcome {
    if fs::rename(staged, installed).is_err() {
        return GenerationInstallOutcome::Unconfirmed {
            error: "cannot install golden generation".into(),
        };
    }
    if fault == PublishFault::InstallUnconfirmedAfterMove {
        return GenerationInstallOutcome::Unconfirmed {
            error: "injected unconfirmed golden generation installation".into(),
        };
    }
    GenerationInstallOutcome::Installed
}

#[cfg(windows)]
fn install_generation(
    staged: &Path,
    installed: &Path,
    fault: PublishFault,
) -> GenerationInstallOutcome {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW};

    let staged_wide = staged
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let installed_wide = installed
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            staged_wide.as_ptr(),
            installed_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return GenerationInstallOutcome::Unconfirmed {
            error: "cannot durably install golden generation".into(),
        };
    }
    if fault == PublishFault::InstallUnconfirmedAfterMove {
        return GenerationInstallOutcome::Unconfirmed {
            error: "injected unconfirmed golden generation installation".into(),
        };
    }
    GenerationInstallOutcome::Installed
}

#[cfg(not(any(unix, windows)))]
fn install_generation(
    _staged: &Path,
    _installed: &Path,
    _fault: PublishFault,
) -> GenerationInstallOutcome {
    GenerationInstallOutcome::Unconfirmed {
        error: "durable golden generation installation is unsupported".into(),
    }
}

fn sync_generation_install(
    container: &Path,
    generations: &Path,
    fault: PublishFault,
) -> Result<(), String> {
    if matches!(
        fault,
        PublishFault::GenerationDirectorySync | PublishFault::GenerationDirectorySyncAndCleanup
    ) {
        return Err("injected failure while syncing golden generation installation".into());
    }
    #[cfg(unix)]
    {
        sync_directory(generations, "cannot sync golden generations root")?;
        sync_directory(container, "cannot sync golden fixture root")?;
    }
    #[cfg(windows)]
    let _ = (container, generations);
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (container, generations);
        return Err("durable golden generation installation is unsupported".into());
    }
    Ok(())
}

#[cfg(unix)]
fn replace_pointer_file(
    temporary: &Path,
    current: &Path,
    generation: &str,
    fault: PublishFault,
) -> PointerReplaceOutcome {
    if fs::rename(temporary, current).is_err() {
        return confirm_failed_pointer_replace(
            current,
            generation,
            "cannot atomically replace golden pointer",
        );
    }
    if fault == PublishFault::AfterPointerReplace {
        return PointerReplaceOutcome::Committed {
            durability_pending: true,
        };
    }
    match sync_parent(current) {
        Ok(()) => PointerReplaceOutcome::Committed {
            durability_pending: false,
        },
        Err(_) => PointerReplaceOutcome::Committed {
            durability_pending: true,
        },
    }
}

#[cfg(windows)]
fn replace_pointer_file(
    temporary: &Path,
    current: &Path,
    generation: &str,
    fault: PublishFault,
) -> PointerReplaceOutcome {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let temporary_wide = temporary
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let current_wide = current
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            temporary_wide.as_ptr(),
            current_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return confirm_failed_pointer_replace(
            current,
            generation,
            "cannot atomically replace golden pointer",
        );
    }
    if fault == PublishFault::AfterPointerReplace {
        return PointerReplaceOutcome::Committed {
            durability_pending: true,
        };
    }
    PointerReplaceOutcome::Committed {
        durability_pending: false,
    }
}

#[cfg(not(any(unix, windows)))]
fn replace_pointer_file(
    temporary: &Path,
    current: &Path,
    generation: &str,
    fault: PublishFault,
) -> PointerReplaceOutcome {
    if current.exists() {
        return PointerReplaceOutcome::Uncommitted {
            error: "atomic golden pointer replacement is unsupported".into(),
        };
    }
    if fs::rename(temporary, current).is_err() {
        return confirm_failed_pointer_replace(current, generation, "cannot create golden pointer");
    }
    if fault == PublishFault::AfterPointerReplace {
        return PointerReplaceOutcome::Committed {
            durability_pending: true,
        };
    }
    PointerReplaceOutcome::Committed {
        durability_pending: false,
    }
}

fn confirm_failed_pointer_replace(
    current: &Path,
    generation: &str,
    error: impl Into<String>,
) -> PointerReplaceOutcome {
    let selected = current
        .parent()
        .and_then(|container| load_pointer(container).ok())
        .is_some_and(|pointer| pointer.generation == generation);
    if selected {
        PointerReplaceOutcome::Committed {
            durability_pending: true,
        }
    } else {
        PointerReplaceOutcome::Uncommitted {
            error: error.into(),
        }
    }
}

fn atomic_replace_pointer(
    container: &Path,
    generation: &str,
    fault: PublishFault,
) -> PointerReplaceOutcome {
    if let Err(error) = validate_hash(generation) {
        return PointerReplaceOutcome::Uncommitted { error };
    }
    let current = container.join("CURRENT");
    let temporary = container.join(format!(".CURRENT.tmp-{}", Uuid::new_v4()));
    let prepared = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "cannot create golden pointer temporary")?;
        file.write_all(&pointer_bytes(generation))
            .map_err(|_| "cannot write golden pointer temporary")?;
        file.sync_all()
            .map_err(|_| "cannot sync golden pointer temporary")?;
        Ok::<(), String>(())
    })();
    if let Err(error) = prepared {
        let _ = fs::remove_file(&temporary);
        return PointerReplaceOutcome::Uncommitted { error };
    }
    if fault == PublishFault::BeforePointerReplace {
        let _ = fs::remove_file(&temporary);
        return PointerReplaceOutcome::Uncommitted {
            error: "injected failure before golden pointer replacement".into(),
        };
    }
    let outcome = replace_pointer_file(&temporary, &current, generation, fault);
    let _ = fs::remove_file(&temporary);
    outcome
}

fn recognized_stage_name(name: &str) -> bool {
    name.strip_prefix(".stage-")
        .is_some_and(|value| Uuid::parse_str(value).is_ok())
}

fn recognized_pointer_temporary(name: &str) -> bool {
    name.strip_prefix(".CURRENT.tmp-")
        .is_some_and(|value| Uuid::parse_str(value).is_ok())
}

fn recognized_generation(root: &Path, name: &str) -> bool {
    validate_hash(name).is_ok()
        && fs::read(root.join("manifest.json")).is_ok_and(|bytes| hash_bytes(&bytes) == name)
        && fs::read(root.join("manifest.json")).is_ok_and(|bytes| {
            serde_json::from_slice::<GoldenManifest>(&bytes)
                .is_ok_and(|manifest| validate_recognized_generation(root, &manifest).is_ok())
        })
}

fn validate_recognized_generation(root: &Path, manifest: &GoldenManifest) -> Result<(), String> {
    if manifest.fixture_revision == FIXTURE_REVISION {
        validate_manifest(root, manifest, true)
    } else {
        validate_migration_source(root, manifest)
    }
}

fn cleanup_recognized_entries(container: &Path, active: Option<&str>) -> Result<(), String> {
    if !container.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(container).map_err(|_| "cannot inspect golden fixture root")? {
        let entry = entry.map_err(|_| "cannot inspect golden fixture entry")?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() && recognized_stage_name(&name) {
            fs::remove_dir_all(entry.path()).map_err(|_| "cannot remove stale golden stage")?;
        } else if entry.path().is_file() && recognized_pointer_temporary(&name) {
            fs::remove_file(entry.path())
                .map_err(|_| "cannot remove stale golden pointer temporary")?;
        }
    }
    let generations = container.join("generations");
    if let Some(active) = active.filter(|_| generations.exists()) {
        for entry in fs::read_dir(&generations).map_err(|_| "cannot inspect golden generations")? {
            let entry = entry.map_err(|_| "cannot inspect golden generation")?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != active
                && entry.path().is_dir()
                && recognized_generation(&entry.path(), &name)
            {
                fs::remove_dir_all(entry.path())
                    .map_err(|_| "cannot remove inactive golden generation")?;
            }
        }
    }
    Ok(())
}

fn reconcile_fixture_container(
    container: &Path,
    allow_missing_pointer: bool,
    fault: CleanupFault,
) -> Result<ReconcileOutcome, String> {
    let current = container.join("CURRENT");
    let selected = if current.exists() {
        let pointer = load_pointer(container)?;
        let root = generation_root(container, &pointer.generation);
        let manifest_bytes = fs::read(root.join("manifest.json"))
            .map_err(|_| "selected golden manifest is missing")?;
        if hash_bytes(&manifest_bytes) != pointer.generation {
            return Err("selected golden generation digest does not match its manifest".into());
        }
        let manifest: GoldenManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| "selected golden manifest is malformed")?;
        if allow_missing_pointer && manifest.fixture_revision == MIGRATION_FIXTURE_REVISION {
            validate_migration_source(&root, &manifest)?;
        } else {
            validate_manifest(&root, &manifest, true)?;
        }
        Some((pointer.generation, root, manifest))
    } else if allow_missing_pointer {
        None
    } else {
        return Err("golden pointer is missing".into());
    };
    let cleanup_pending = fault == CleanupFault::Fail
        || cleanup_recognized_entries(
            container,
            selected
                .as_ref()
                .map(|(generation, _, _)| generation.as_str()),
        )
        .is_err();
    Ok(ReconcileOutcome {
        selected: selected.map(|(_, root, manifest)| (root, manifest)),
        cleanup_pending,
    })
}

fn publish_staged_generation(
    container: &Path,
    staged: &Path,
    fault: PublishFault,
) -> Result<PublishOutcome, String> {
    let manifest_bytes =
        fs::read(staged.join("manifest.json")).map_err(|_| "staged manifest is missing")?;
    let generation = hash_bytes(&manifest_bytes);
    let manifest: GoldenManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|_| "staged manifest is malformed")?;
    validate_manifest(staged, &manifest, true)?;
    if fault == PublishFault::BeforeInstall {
        return Err("injected failure before generation installation".into());
    }
    let generations = container.join("generations");
    let installed = generation_root(container, &generation);
    let installed_now = if installed.exists() {
        if !recognized_generation(&installed, &generation) {
            return Err("existing golden generation is invalid".into());
        }
        let installed_manifest = load_manifest(&installed)?;
        sync_generation_content(&installed, &installed_manifest, fault)?;
        sync_generation_install(container, &generations, fault)?;
        fs::remove_dir_all(staged).map_err(|_| "cannot remove duplicate golden stage")?;
        false
    } else {
        sync_generation_content(staged, &manifest, fault)?;
        fs::create_dir_all(&generations).map_err(|_| "cannot create golden generations root")?;
        match install_generation(staged, &installed, fault) {
            GenerationInstallOutcome::Installed => {}
            GenerationInstallOutcome::Unconfirmed { error } => return Err(error),
        }
        if let Err(error) = sync_generation_install(container, &generations, fault) {
            if fault != PublishFault::GenerationDirectorySyncAndCleanup {
                let _ = fs::remove_dir_all(&installed);
            }
            return Err(error);
        }
        true
    };
    let pointer_durability_pending = match atomic_replace_pointer(container, &generation, fault) {
        PointerReplaceOutcome::Uncommitted { error } => {
            if installed_now {
                let _ = fs::remove_dir_all(&installed);
            }
            return Err(error);
        }
        PointerReplaceOutcome::Committed { durability_pending } => durability_pending,
    };
    let cleanup_pending = pointer_durability_pending
        || fault == PublishFault::Cleanup
        || cleanup_recognized_entries(container, Some(&generation)).is_err();
    Ok(PublishOutcome {
        generation,
        cleanup_pending,
        pointer_durability_pending,
    })
}

fn update_golden_set(tools: &NativeTools, capture: &Capture, performance: &PerformanceBaseline) {
    let container = fixture_container_root();
    fs::create_dir_all(&container).expect("create golden fixture root");
    let staged = container.join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&staged).expect("create golden staging directory");
    let mut guard = StageGuard::new(staged.clone());
    let manifest = write_capture_set(&staged, tools, capture, performance);
    validate_manifest(&staged, &manifest, true).expect("validate staged golden generation");
    let outcome = publish_staged_generation(&container, &staged, PublishFault::None)
        .expect("atomically update golden fixture generation");
    guard.disarm();
    if outcome.pointer_durability_pending {
        eprintln!(
            "golden generation committed; pointer durability is pending and both generations were retained"
        );
    } else if outcome.cleanup_pending {
        eprintln!("golden generation committed; inactive-generation cleanup remains pending");
    }
}

fn assert_deterministic_references_match(
    reviewed: &GoldenManifest,
    captured: &GoldenManifest,
) -> Result<(), String> {
    let deterministic = |manifest: &GoldenManifest| {
        manifest
            .references
            .iter()
            .filter(|reference| !matches!(reference, GoldenReference::PerformanceBaseline { .. }))
            .map(|reference| (reference.path().to_owned(), reference.sha256().to_owned()))
            .collect::<BTreeMap<_, _>>()
    };
    if deterministic(reviewed) != deterministic(captured) {
        return Err("recaptured deterministic references differ from the reviewed set".into());
    }
    Ok(())
}

#[test]
fn native_golden_render_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let update_requested = env::var("OPENCUT_UPDATE_GOLDENS").as_deref() == Ok("1");
    let fixture_container = fixture_container_root();
    let _fixture_lock = GoldenFixtureLock::exclusive(&fixture_container)
        .expect("acquire golden fixture coordination lock");
    let reconciliation =
        reconcile_fixture_container(&fixture_container, update_requested, CleanupFault::None)
            .expect("reconcile golden fixture generations");
    if reconciliation.cleanup_pending {
        eprintln!("golden startup cleanup remains pending");
    }
    let reviewed = if update_requested {
        None
    } else {
        let (root, manifest) = reconciliation
            .selected
            .expect("reconciled golden generation must be selected");
        assert_eq!(
            manifest.environment.font_sha256, tools.font_sha256,
            "configured font identity differs from the reviewed golden"
        );
        Some((root, manifest))
    };
    let sampled_requested = update_requested
        || env::var_os("OPENCUT_CAPTURE_GOLDENS_TO").is_some()
        || env::var_os("OPENCUT_GOLDEN_REPORT_PATH").is_some();
    let (capture, performance) = if sampled_requested {
        let (capture, performance) = sampled_capture(&tools);
        (capture, Some(performance))
    } else {
        (capture(&tools, CaptureMode::Conformance), None)
    };
    if update_requested {
        update_golden_set(
            &tools,
            &capture,
            performance.as_ref().expect("sampled update performance"),
        );
    } else {
        let (root, manifest) = reviewed.as_ref().expect("prevalidated reviewed generation");
        compare_capture(root, manifest, &capture)
            .expect("production render differs from reviewed golden");
    }
    if let Some(capture_path) = env::var_os("OPENCUT_CAPTURE_GOLDENS_TO") {
        let capture_path = PathBuf::from(capture_path);
        assert!(
            !capture_path.exists(),
            "explicit comparison capture path must not already exist"
        );
        fs::create_dir_all(&capture_path).expect("create explicit comparison capture path");
        let recaptured = write_capture_set(
            &capture_path,
            &tools,
            &capture,
            performance.as_ref().expect("sampled recapture performance"),
        );
        validate_manifest(&capture_path, &recaptured, true)
            .expect("validate explicit comparison capture");
        let reviewed_manifest = if let Some((_, manifest)) = &reviewed {
            manifest.clone()
        } else {
            let reviewed_root = selected_generation_root(&fixture_container_root())
                .expect("load selected golden generation");
            load_manifest(&reviewed_root).expect("load reviewed golden fixture")
        };
        assert_deterministic_references_match(&reviewed_manifest, &recaptured)
            .expect("explicit comparison capture is not reproducible");
    }
    if let Some(report_path) = env::var_os("OPENCUT_GOLDEN_REPORT_PATH") {
        write_performance_report(
            Path::new(&report_path),
            performance.as_ref().expect("sampled report performance"),
        )
        .expect("write and validate requested report-only golden baseline");
    }
}

#[test]
#[ignore = "CI helper validates an explicitly captured report-only artifact"]
fn validate_external_performance_report() {
    let path = env::var_os("OPENCUT_GOLDEN_REPORT_PATH")
        .expect("OPENCUT_GOLDEN_REPORT_PATH must identify the captured report");
    validate_performance_report_file(Path::new(&path))
        .expect("captured report-only golden baseline is invalid");
}

#[test]
fn manifest_rejects_invalid_metadata_before_rendering() {
    let root = tempdir().unwrap();
    let tools = test_tools();
    let capture = test_capture();
    let performance = test_performance();
    let mut manifest = write_capture_set(root.path(), &tools, &capture, &performance);
    assert!(validate_manifest(root.path(), &manifest, true).is_ok());
    let mut invalid_performance = performance.clone();
    invalid_performance.intent_observations[0]
        .timings_ms
        .encoding = f64::NAN;
    assert!(validate_performance_baseline(&invalid_performance).is_err());
    let mut additive = performance.clone();
    additive.non_additive_stage_timings = false;
    assert!(validate_performance_baseline(&additive).is_err());
    let mut duplicate_intent = performance.clone();
    duplicate_intent.intent_observations[1].intent = "framePreview".into();
    assert!(validate_performance_baseline(&duplicate_intent).is_err());
    let mut false_raster_work = performance.clone();
    false_raster_work.intent_observations[0]
        .timings_ms
        .rasterization = 1.0;
    assert!(validate_performance_baseline(&false_raster_work).is_err());

    manifest.schema_version = 2;
    assert!(
        validate_manifest(root.path(), &manifest, false)
            .unwrap_err()
            .contains("version")
    );
    manifest.schema_version = MANIFEST_VERSION;
    manifest.sample_timestamps_ms = vec![0, 0, 900];
    assert!(validate_manifest(root.path(), &manifest, false).is_err());
    manifest.sample_timestamps_ms = SAMPLE_TIMESTAMPS_MS.into();
    manifest.tolerances.minimum_ssim = f64::NAN;
    assert!(validate_manifest(root.path(), &manifest, false).is_err());
    manifest.tolerances.minimum_ssim = SSIM_MINIMUM;
    manifest.environment.ffmpeg_version.clear();
    assert!(validate_manifest(root.path(), &manifest, false).is_err());
    manifest.environment.ffmpeg_version = "ffmpeg fixture".into();
    match &mut manifest.references[0] {
        GoldenReference::Frame { path, .. } => *path = "../escape.rgb".into(),
        _ => unreachable!(),
    }
    assert!(validate_manifest(root.path(), &manifest, false).is_err());
    match &mut manifest.references[0] {
        GoldenReference::Frame { path, .. } => *path = "https://example.invalid/frame.rgb".into(),
        _ => unreachable!(),
    }
    assert!(validate_manifest(root.path(), &manifest, false).is_err());
}

#[test]
fn fixture_update_accepts_a_complete_schema_two_migration_source() {
    let root = tempdir().unwrap();
    let container = root.path().join("golden");
    let source = container.join("source");
    fs::create_dir_all(&source).unwrap();
    let tools = test_tools();
    let mut manifest = write_capture_set(&source, &tools, &test_capture(), &test_performance());
    manifest.fixture_revision = MIGRATION_FIXTURE_REVISION;
    assert!(validate_migration_source(&source, &manifest).is_err());
    let performance = manifest
        .references
        .iter_mut()
        .find(|reference| matches!(reference, GoldenReference::PerformanceBaseline { .. }))
        .unwrap();
    let legacy = serde_json::to_vec_pretty(&test_legacy_performance()).unwrap();
    fs::write(source.join(performance.path()), &legacy).unwrap();
    match performance {
        GoldenReference::PerformanceBaseline { sha256, .. } => *sha256 = hash_bytes(&legacy),
        _ => unreachable!(),
    }
    assert!(validate_migration_source(&source, &manifest).is_ok());
    assert!(validate_manifest(&source, &manifest, true).is_err());
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(source.join("manifest.json"), &manifest_bytes).unwrap();
    let generation = hash_bytes(&manifest_bytes);
    let installed = generation_root(&container, &generation);
    fs::create_dir_all(installed.parent().unwrap()).unwrap();
    fs::rename(&source, &installed).unwrap();
    fs::write(container.join("CURRENT"), pointer_bytes(&generation)).unwrap();

    let reconciliation = reconcile_fixture_container(&container, true, CleanupFault::None).unwrap();
    assert_eq!(
        reconciliation.selected.unwrap().1.fixture_revision,
        MIGRATION_FIXTURE_REVISION
    );
}

#[test]
fn legacy_performance_validation_is_strict_and_complete() {
    let legacy = test_legacy_performance();
    assert!(validate_legacy_performance_baseline(&legacy).is_ok());
    assert!(validate_legacy_performance_bytes(&serde_json::to_vec(&legacy).unwrap()).is_ok());

    let mut missing = serde_json::to_value(&legacy).unwrap();
    missing.as_object_mut().unwrap().remove("memoryScope");
    assert!(validate_legacy_performance_bytes(&serde_json::to_vec(&missing).unwrap()).is_err());

    let mut unknown = serde_json::to_value(&legacy).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), true.into());
    assert!(validate_legacy_performance_bytes(&serde_json::to_vec(&unknown).unwrap()).is_err());

    for (field, value) in [
        ("schemaVersion", serde_json::Value::from(3)),
        ("fixtureRevision", serde_json::Value::from(3)),
        ("fixtureId", serde_json::Value::from("other")),
        ("warmupSamples", serde_json::Value::from(2)),
        ("measuredSamples", serde_json::Value::from(2)),
        ("memoryScope", serde_json::Value::from("process")),
        ("timingAggregation", serde_json::Value::from("mean")),
        ("memoryAggregation", serde_json::Value::from("median")),
    ] {
        let mut invalid = serde_json::to_value(&legacy).unwrap();
        invalid[field] = value;
        assert!(
            validate_legacy_performance_bytes(&serde_json::to_vec(&invalid).unwrap()).is_err(),
            "accepted invalid legacy field {field}"
        );
    }

    let mut invalid_units = legacy.clone();
    invalid_units.units.timing = "seconds".into();
    assert!(validate_legacy_performance_baseline(&invalid_units).is_err());
    let mut negative = legacy.clone();
    negative.timings_ms.total = -1.0;
    assert!(validate_legacy_performance_baseline(&negative).is_err());
    let mut non_finite = legacy;
    non_finite.timings_ms.frame_rendering = f64::INFINITY;
    assert!(validate_legacy_performance_baseline(&non_finite).is_err());
}

#[test]
fn migration_manifest_requires_canonical_media_metadata_and_frame_set() {
    let root = tempdir().unwrap();
    let mut manifest = write_capture_set(
        root.path(),
        &test_tools(),
        &test_capture(),
        &test_performance(),
    );
    manifest.fixture_revision = MIGRATION_FIXTURE_REVISION;
    replace_performance_reference(root.path(), &mut manifest, &test_legacy_performance());
    assert!(validate_migration_source(root.path(), &manifest).is_ok());

    let mut invalid_media = manifest.clone();
    invalid_media.canvas.width += 1;
    assert!(validate_migration_source(root.path(), &invalid_media).is_err());
    let mut invalid_timestamps = manifest.clone();
    invalid_timestamps.sample_timestamps_ms = vec![0, 500, 901];
    assert!(validate_migration_source(root.path(), &invalid_timestamps).is_err());
    let mut invalid_frame = manifest;
    match invalid_frame
        .references
        .iter_mut()
        .find(|reference| matches!(reference, GoldenReference::Frame { at_ms: 900, .. }))
        .unwrap()
    {
        GoldenReference::Frame { at_ms, .. } => *at_ms = 901,
        _ => unreachable!(),
    }
    assert!(validate_migration_source(root.path(), &invalid_frame).is_err());
}

#[test]
fn strict_report_file_validation_rejects_unknown_and_inconsistent_data() {
    let root = tempdir().unwrap();
    let path = root.path().join("performance.json");
    let performance = test_performance();
    write_performance_report(&path, &performance).unwrap();

    let mut present_git_revision = performance.clone();
    present_git_revision.git_revision = Some("arbitrary-nonblank-git-identity".into());
    write_performance_report(&path, &present_git_revision).unwrap();

    let mut missing_git_revision = serde_json::to_value(&performance).unwrap();
    missing_git_revision
        .as_object_mut()
        .unwrap()
        .remove("gitRevision");
    fs::write(&path, serde_json::to_vec(&missing_git_revision).unwrap()).unwrap();
    assert!(validate_performance_report_file(&path).is_err());

    for invalid_git_revision in ["", " \t\r\n"] {
        let mut invalid = serde_json::to_value(&performance).unwrap();
        invalid["gitRevision"] = serde_json::Value::from(invalid_git_revision);
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(
            validate_performance_report_file(&path).is_err(),
            "accepted invalid gitRevision {invalid_git_revision:?}"
        );
    }

    let mut non_string_git_revision = serde_json::to_value(&performance).unwrap();
    non_string_git_revision["gitRevision"] = serde_json::Value::from(7);
    fs::write(&path, serde_json::to_vec(&non_string_git_revision).unwrap()).unwrap();
    assert!(validate_performance_report_file(&path).is_err());

    let mut unknown = serde_json::to_value(&performance).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpected".into(), serde_json::Value::Bool(true));
    fs::write(&path, serde_json::to_vec(&unknown).unwrap()).unwrap();
    assert!(validate_performance_report_file(&path).is_err());

    let mut inconsistent = serde_json::to_value(&performance).unwrap();
    inconsistent["warmupSamples"] = serde_json::Value::from(2);
    fs::write(&path, serde_json::to_vec(&inconsistent).unwrap()).unwrap();
    assert!(validate_performance_report_file(&path).is_err());

    let work_fields = [
        "decodedInputs",
        "rasterizedLayers",
        "compositedLayers",
        "encodedVideoStreams",
        "encodedAudioStreams",
    ];
    for (intent_index, observation) in performance.intent_observations.iter().enumerate() {
        let expected = [
            observation.work.decoded_inputs,
            observation.work.rasterized_layers,
            observation.work.composited_layers,
            observation.work.encoded_video_streams,
            observation.work.encoded_audio_streams,
        ];
        for (field, expected) in work_fields.into_iter().zip(expected) {
            let invalid_values = if expected == 0 {
                vec![1, 2]
            } else {
                vec![0, expected + 1]
            };
            for invalid_value in invalid_values {
                let mut invalid = serde_json::to_value(&performance).unwrap();
                invalid["intentObservations"][intent_index]["work"][field] =
                    serde_json::Value::from(invalid_value);
                fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
                assert!(
                    validate_performance_report_file(&path).is_err(),
                    "accepted {field}={invalid_value} for {}",
                    observation.intent
                );
            }
        }
    }
}

#[test]
fn update_reconciliation_rejects_a_malformed_current_generation() {
    let root = tempdir().unwrap();
    let container = root.path().join("golden");
    let staged = container.join("staged");
    fs::create_dir_all(&staged).unwrap();
    let mut manifest =
        write_capture_set(&staged, &test_tools(), &test_capture(), &test_performance());
    let performance = manifest
        .references
        .iter_mut()
        .find(|reference| matches!(reference, GoldenReference::PerformanceBaseline { .. }))
        .unwrap();
    let malformed = br#"{"schemaVersion":3}"#;
    fs::write(staged.join(performance.path()), malformed).unwrap();
    match performance {
        GoldenReference::PerformanceBaseline { sha256, .. } => *sha256 = hash_bytes(malformed),
        _ => unreachable!(),
    }
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(staged.join("manifest.json"), &manifest_bytes).unwrap();
    let generation = hash_bytes(&manifest_bytes);
    let installed = generation_root(&container, &generation);
    fs::create_dir_all(installed.parent().unwrap()).unwrap();
    fs::rename(&staged, &installed).unwrap();
    fs::write(container.join("CURRENT"), pointer_bytes(&generation)).unwrap();

    assert!(reconcile_fixture_container(&container, true, CleanupFault::None).is_err());
    assert!(installed.exists());
    assert!(container.join("CURRENT").exists());
}

#[test]
fn unsupported_inactive_generation_is_not_recognized_for_cleanup() {
    let root = tempdir().unwrap();
    let generation_root = root.path().join("generation");
    fs::create_dir(&generation_root).unwrap();
    let mut manifest = write_capture_set(
        &generation_root,
        &test_tools(),
        &test_capture(),
        &test_performance(),
    );
    manifest.fixture_revision = 1;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    fs::write(generation_root.join("manifest.json"), &manifest_bytes).unwrap();

    assert!(!recognized_generation(
        &generation_root,
        &hash_bytes(&manifest_bytes)
    ));
}

#[test]
fn malformed_legacy_inactive_generation_is_preserved_during_cleanup() {
    let root = tempdir().unwrap();
    let container = root.path().join("golden");
    let generations = container.join("generations");
    fs::create_dir_all(&generations).unwrap();

    let active_root = generations.join("active");
    fs::create_dir(&active_root).unwrap();
    let active_manifest = write_capture_set(
        &active_root,
        &test_tools(),
        &test_capture(),
        &test_performance(),
    );
    let active_bytes = serde_json::to_vec_pretty(&active_manifest).unwrap();
    fs::write(active_root.join("manifest.json"), &active_bytes).unwrap();
    let active = hash_bytes(&active_bytes);
    let canonical_active_root = generations.join(&active);
    fs::rename(&active_root, &canonical_active_root).unwrap();

    let malformed_root = generations.join("malformed");
    fs::create_dir(&malformed_root).unwrap();
    let mut malformed_manifest = write_capture_set(
        &malformed_root,
        &test_tools(),
        &test_capture(),
        &test_performance(),
    );
    malformed_manifest.fixture_revision = MIGRATION_FIXTURE_REVISION;
    let performance = malformed_manifest
        .references
        .iter_mut()
        .find(|reference| matches!(reference, GoldenReference::PerformanceBaseline { .. }))
        .unwrap();
    let malformed_report = br#"{"schemaVersion":2}"#;
    fs::write(malformed_root.join(performance.path()), malformed_report).unwrap();
    match performance {
        GoldenReference::PerformanceBaseline { sha256, .. } => {
            *sha256 = hash_bytes(malformed_report)
        }
        _ => unreachable!(),
    }
    let malformed_bytes = serde_json::to_vec_pretty(&malformed_manifest).unwrap();
    fs::write(malformed_root.join("manifest.json"), &malformed_bytes).unwrap();
    let malformed = hash_bytes(&malformed_bytes);
    let canonical_malformed_root = generations.join(&malformed);
    fs::rename(&malformed_root, &canonical_malformed_root).unwrap();

    assert!(!recognized_generation(
        &canonical_malformed_root,
        &malformed
    ));
    cleanup_recognized_entries(&container, Some(&active)).unwrap();
    assert!(canonical_active_root.exists());
    assert!(canonical_malformed_root.exists());
}

fn test_tools() -> NativeTools {
    NativeTools {
        ffmpeg: "ffmpeg".into(),
        ffprobe: "ffprobe".into(),
        font: "font.ttf".into(),
        ffmpeg_version: "ffmpeg fixture".into(),
        ffprobe_version: "ffprobe fixture".into(),
        font_sha256: "a".repeat(64),
    }
}

fn test_capture() -> Capture {
    Capture {
        frames: BTreeMap::new(),
        range_frames: BTreeMap::new(),
        export_frames: SAMPLE_TIMESTAMPS_MS
            .into_iter()
            .map(|at_ms| (at_ms, vec![0; (WIDTH * HEIGHT * 3) as usize]))
            .collect(),
        range_audio: vec![],
        export_audio: vec![0.0],
        range_duration_ms: DURATION_MS,
        export_duration_ms: DURATION_MS,
        semantic_plan: "scene\n".into(),
        filter_graph: "graph\n".into(),
        intent_observations: test_intent_observations(0.0),
        peak_process_tree_bytes: 0,
    }
}

fn test_intent_observations(value: f64) -> Vec<IntentObservation> {
    [
        ("framePreview", 0),
        ("audiovisualRangePreview", 1),
        ("finalExport", 1),
    ]
    .into_iter()
    .map(|(intent, encoded_audio_streams)| IntentObservation {
        intent: intent.into(),
        timings_ms: StageTimings {
            scene_evaluation: value,
            rasterization: 0.0,
            filter_graph_construction: value,
            decoding: value,
            compositing: value,
            encoding: value,
            end_to_end: value,
        },
        work: StageWork {
            decoded_inputs: 1,
            rasterized_layers: 0,
            composited_layers: 2,
            encoded_video_streams: 1,
            encoded_audio_streams,
        },
    })
    .collect()
}

fn test_performance() -> PerformanceBaseline {
    PerformanceBaseline {
        schema_version: PERFORMANCE_SCHEMA_VERSION,
        fixture_id: FIXTURE_ID.into(),
        fixture_revision: FIXTURE_REVISION,
        git_revision: None,
        os: "test".into(),
        architecture: "test".into(),
        ffmpeg_version: "test".into(),
        ffprobe_version: "test".into(),
        font_sha256: "a".repeat(64),
        units: BaselineUnits {
            timing: "milliseconds".into(),
            memory: "bytes".into(),
        },
        warmup_samples: WARMUP_SAMPLES,
        measured_samples: MEASURED_SAMPLES,
        memory_scope: "process_tree".into(),
        timing_aggregation: "median".into(),
        memory_aggregation: "maximum".into(),
        stage_definition_version: STAGE_DEFINITION_VERSION,
        non_additive_stage_timings: true,
        intent_observations: test_intent_observations(0.0),
        peak_resident_working_set_bytes: 0,
        comparison_policy: "report_only_compare_matching_environment_identity".into(),
    }
}

fn test_legacy_performance() -> LegacyPerformanceBaseline {
    LegacyPerformanceBaseline {
        schema_version: MIGRATION_PERFORMANCE_SCHEMA_VERSION,
        fixture_id: FIXTURE_ID.into(),
        fixture_revision: MIGRATION_FIXTURE_REVISION,
        git_revision: serde_json::Value::Null,
        os: "test".into(),
        architecture: "test".into(),
        ffmpeg_version: "test".into(),
        ffprobe_version: "test".into(),
        font_sha256: "a".repeat(64),
        units: BaselineUnits {
            timing: "milliseconds".into(),
            memory: "bytes".into(),
        },
        warmup_samples: WARMUP_SAMPLES,
        measured_samples: MEASURED_SAMPLES,
        memory_scope: "process_tree".into(),
        timing_aggregation: "median".into(),
        memory_aggregation: "maximum".into(),
        timings_ms: LegacyPhaseTimings {
            scene_evaluation: 1.0,
            filter_graph_construction: 2.0,
            frame_rendering: 3.0,
            audiovisual_range_rendering: 4.0,
            export_rendering: 5.0,
            total: 6.0,
        },
        peak_resident_working_set_bytes: 1,
        comparison_policy: "report_only_compare_matching_environment_identity".into(),
    }
}

fn replace_performance_reference(
    root: &Path,
    manifest: &mut GoldenManifest,
    baseline: &LegacyPerformanceBaseline,
) {
    let reference = manifest
        .references
        .iter_mut()
        .find(|reference| matches!(reference, GoldenReference::PerformanceBaseline { .. }))
        .unwrap();
    let bytes = serde_json::to_vec_pretty(baseline).unwrap();
    fs::write(root.join(reference.path()), &bytes).unwrap();
    match reference {
        GoldenReference::PerformanceBaseline { sha256, .. } => *sha256 = hash_bytes(&bytes),
        _ => unreachable!(),
    }
}

fn create_test_stage(container: &Path, label: &str) -> PathBuf {
    let staged = container.join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&staged).unwrap();
    let mut tools = test_tools();
    tools.ffmpeg_version = label.into();
    write_capture_set(&staged, &tools, &test_capture(), &test_performance());
    staged
}

fn wait_for_test_marker(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !path.exists() {
        assert!(Instant::now() < deadline, "timed out waiting for {path:?}");
        thread::sleep(Duration::from_millis(10));
    }
}

fn golden_helper_command(test_name: &str) -> Command {
    let mut command = Command::new(env::current_exe().unwrap());
    command.args(["--ignored", "--exact", test_name]);
    command
}

#[test]
fn golden_lock_blocks_reconciliation_until_the_owner_releases_it() {
    let root = tempdir().unwrap();
    let container = root.path().join("golden");
    let started = root.path().join("started");
    let acquired = root.path().join("acquired");
    let fixture_lock = GoldenFixtureLock::exclusive(&container).unwrap();
    let live_stage = create_test_stage(&container, "live-stage");
    let mut child = golden_helper_command("renderer::golden::golden_lock_reconcile_child_helper")
        .env("OPENCUT_GOLDEN_HELPER_CONTAINER", &container)
        .env("OPENCUT_GOLDEN_HELPER_STARTED", &started)
        .env("OPENCUT_GOLDEN_HELPER_ACQUIRED", &acquired)
        .spawn()
        .unwrap();

    wait_for_test_marker(&started, Duration::from_secs(5));
    let blocked_until = Instant::now() + Duration::from_millis(250);
    while Instant::now() < blocked_until {
        assert!(child.try_wait().unwrap().is_none());
        assert!(!acquired.exists());
        assert!(live_stage.exists());
        thread::sleep(Duration::from_millis(10));
    }

    drop(fixture_lock);
    assert!(child.wait().unwrap().success());
    assert!(acquired.exists());
    assert!(!live_stage.exists());
    assert!(container.join(".golden.lock").exists());
}

#[test]
#[ignore = "helper process for golden_lock_blocks_reconciliation_until_the_owner_releases_it"]
fn golden_lock_reconcile_child_helper() {
    let container = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_CONTAINER").unwrap());
    let started = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_STARTED").unwrap());
    let acquired = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_ACQUIRED").unwrap());
    fs::write(started, b"started").unwrap();
    let _fixture_lock = GoldenFixtureLock::exclusive(&container).unwrap();
    reconcile_fixture_container(&container, true, CleanupFault::None).unwrap();
    fs::write(acquired, b"acquired").unwrap();
}

#[test]
fn overlapping_same_digest_publications_are_serialized() {
    let root = tempdir().unwrap();
    let container = root.path().join("golden");
    let started = root.path().join("started");
    let completed = root.path().join("completed");
    let fixture_lock = GoldenFixtureLock::exclusive(&container).unwrap();
    let mut child = golden_helper_command("renderer::golden::golden_lock_publish_child_helper")
        .env("OPENCUT_GOLDEN_HELPER_CONTAINER", &container)
        .env("OPENCUT_GOLDEN_HELPER_STARTED", &started)
        .env("OPENCUT_GOLDEN_HELPER_ACQUIRED", &completed)
        .spawn()
        .unwrap();
    wait_for_test_marker(&started, Duration::from_secs(5));

    let first = publish_staged_generation(
        &container,
        &create_test_stage(&container, "same-digest"),
        PublishFault::None,
    )
    .unwrap();
    assert!(!completed.exists());
    drop(fixture_lock);

    assert!(child.wait().unwrap().success());
    assert!(completed.exists());
    assert_eq!(
        load_pointer(&container).unwrap().generation,
        first.generation
    );
    assert!(generation_root(&container, &first.generation).exists());
    let generations = fs::read_dir(container.join("generations"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .count();
    assert_eq!(generations, 1);
}

#[test]
#[ignore = "helper process for overlapping_same_digest_publications_are_serialized"]
fn golden_lock_publish_child_helper() {
    let container = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_CONTAINER").unwrap());
    let started = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_STARTED").unwrap());
    let completed = PathBuf::from(env::var_os("OPENCUT_GOLDEN_HELPER_ACQUIRED").unwrap());
    fs::write(started, b"started").unwrap();
    let _fixture_lock = GoldenFixtureLock::exclusive(&container).unwrap();
    let stage = create_test_stage(&container, "same-digest");
    publish_staged_generation(&container, &stage, PublishFault::None).unwrap();
    fs::write(completed, b"completed").unwrap();
}

#[test]
fn golden_lock_is_released_after_error_and_panic() {
    let container = tempdir().unwrap();
    let failed = (|| -> Result<(), String> {
        let _fixture_lock = GoldenFixtureLock::exclusive(container.path())?;
        Err("injected pre-commit failure".into())
    })();
    assert!(failed.is_err());
    drop(GoldenFixtureLock::exclusive(container.path()).unwrap());

    let panicked = std::panic::catch_unwind(|| {
        let _fixture_lock = GoldenFixtureLock::exclusive(container.path()).unwrap();
        panic!("controlled golden lock panic");
    });
    assert!(panicked.is_err());
    drop(GoldenFixtureLock::exclusive(container.path()).unwrap());
    assert!(container.path().join(".golden.lock").exists());
}

#[test]
fn generation_directory_collection_includes_all_ancestors_deepest_first() {
    let root = PathBuf::from("generation-root");
    let directories = generation_directories_deepest_first(
        &root,
        &[
            root.join("frames/nested/deeper/0000.rgb"),
            root.join("manifest.json"),
        ],
    )
    .unwrap();
    assert_eq!(
        directories,
        vec![
            root.join("frames/nested/deeper"),
            root.join("frames/nested"),
            root.join("frames"),
            root,
        ]
    );
}

#[test]
fn unconfirmed_install_preserves_the_destination_for_locked_reconciliation() {
    let container = tempdir().unwrap();
    let active = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "active"),
        PublishFault::None,
    )
    .unwrap();
    let stage = create_test_stage(container.path(), "unconfirmed");
    let generation = hash_bytes(&fs::read(stage.join("manifest.json")).unwrap());
    assert!(
        publish_staged_generation(
            container.path(),
            &stage,
            PublishFault::InstallUnconfirmedAfterMove,
        )
        .is_err()
    );
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        active.generation
    );
    assert!(generation_root(container.path(), &generation).exists());

    reconcile_fixture_container(container.path(), false, CleanupFault::None).unwrap();
    assert!(!generation_root(container.path(), &generation).exists());
    assert!(generation_root(container.path(), &active.generation).exists());
}

#[test]
fn manifest_rejects_unknown_fields_hash_mismatch_and_duplicate_references() {
    let json = r#"{
      "schemaVersion":1,"fixtureId":"flat-scene-av-v1","fixtureRevision":1,
      "canvas":{"width":160,"height":90,"fps":10,"unknown":true},
      "durationMs":1000,"sampleTimestampsMs":[0,500,900],
      "audio":{"sampleRateHz":48000,"channels":1,"sampleFormat":"f32le"},
      "tolerances":{"minimumSsim":0.99,"maximumPcmRmsError":0.0001,"maximumTimingFrames":1},
      "environment":{"referenceOs":"test","referenceArch":"test","ffmpegVersion":"x","ffprobeVersion":"x","fontSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
      "references":[]
    }"#;
    assert!(serde_json::from_str::<GoldenManifest>(json).is_err());

    if let Ok(root) = selected_generation_root(&fixture_container_root()) {
        let mut manifest = load_manifest(&root).unwrap();
        let duplicate = serde_json::from_value::<GoldenReference>(
            serde_json::to_value(&manifest.references[0]).unwrap(),
        )
        .unwrap();
        manifest.references.push(duplicate);
        assert!(validate_manifest(&root, &manifest, false).is_err());
        match &mut manifest.references[0] {
            GoldenReference::Frame { sha256, .. } => *sha256 = "0".repeat(64),
            _ => unreachable!(),
        }
        assert!(validate_manifest(&root, &manifest, true).is_err());

        let mut absolute = load_manifest(&root).unwrap();
        match &mut absolute.references[0] {
            GoldenReference::Frame { path, .. } => {
                *path = root.join("frames/0000.rgb").display().to_string()
            }
            _ => unreachable!(),
        }
        assert!(validate_manifest(&root, &absolute, false).is_err());

        let temporary = tempdir().unwrap();
        fs::create_dir_all(temporary.path().join("frames")).unwrap();
        let mut missing = load_manifest(&root).unwrap();
        for reference in &mut missing.references {
            let source = root.join(reference.path());
            let target = temporary.path().join(reference.path());
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(source, target).unwrap();
        }
        fs::remove_file(temporary.path().join(missing.references[0].path())).unwrap();
        assert!(validate_manifest(temporary.path(), &missing, true).is_err());
    }
}

#[cfg(unix)]
#[test]
fn manifest_rejects_a_symlink_escape_from_the_fixture_root() {
    use std::os::unix::fs::symlink;

    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    fs::write(outside.path().join("frame.rgb"), b"outside").unwrap();
    symlink(outside.path(), root.path().join("escaped")).unwrap();
    assert!(
        safe_reference_path(root.path(), "escaped/frame.rgb")
            .unwrap_err()
            .contains("escapes")
    );
}

#[test]
fn normalization_only_replaces_declared_paths_and_workspace_identity() {
    let root = Path::new("C:/fixture/project");
    let font = Path::new("C:/fonts/golden.ttf");
    let first = "drawtext=textfile='C\\:/fixture/project/.opencut-work-abc/text.txt':fontfile='C\\:/fonts/golden.ttf':x=10";
    let second = "drawtext=textfile='C\\:/fixture/project/.opencut-work-def/text.txt':fontfile='C\\:/fonts/golden.ttf':x=10";
    assert_eq!(
        normalize_filter_graph(first, root, font),
        "drawtext=textfile='<WORKSPACE>/text.txt':fontfile='<FONT>':x=10"
    );
    assert_eq!(
        normalize_filter_graph(first, root, font),
        normalize_filter_graph(second, root, font)
    );
    assert_ne!(
        normalize_filter_graph(first, root, font),
        normalize_filter_graph(&second.replace("x=10", "x=11"), root, font)
    );
}

#[test]
fn generation_pointer_commit_preserves_a_complete_selected_set() {
    let container = tempdir().unwrap();

    let first = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "first"),
        PublishFault::None,
    )
    .unwrap();
    assert!(!first.cleanup_pending);
    assert!(!first.pointer_durability_pending);
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        first.generation
    );
    for fault in [
        PublishFault::BeforeInstall,
        PublishFault::BeforePointerReplace,
    ] {
        let staged = create_test_stage(container.path(), &format!("failure-{fault:?}"));
        assert!(publish_staged_generation(container.path(), &staged, fault).is_err());
        assert_eq!(
            load_pointer(container.path()).unwrap().generation,
            first.generation
        );
        let selected = selected_generation_root(container.path()).unwrap();
        assert_eq!(
            load_manifest(&selected).unwrap().environment.ffmpeg_version,
            "first"
        );
        let _ = fs::remove_dir_all(staged);
    }

    let second = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "second"),
        PublishFault::Cleanup,
    )
    .unwrap();
    assert!(second.cleanup_pending);
    assert!(!second.pointer_durability_pending);
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        second.generation
    );
    assert!(generation_root(container.path(), &first.generation).exists());
    cleanup_recognized_entries(container.path(), Some(&second.generation)).unwrap();
    assert!(!generation_root(container.path(), &first.generation).exists());
}

#[test]
fn generation_content_sync_failure_never_reaches_pointer_commit() {
    let first_publication = tempdir().unwrap();
    let first_stage = create_test_stage(first_publication.path(), "first");
    assert!(
        publish_staged_generation(
            first_publication.path(),
            &first_stage,
            PublishFault::GenerationContentSync,
        )
        .is_err()
    );
    assert!(!first_publication.path().join("CURRENT").exists());
    assert!(!first_publication.path().join("generations").exists());

    let container = tempdir().unwrap();
    let active = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "active"),
        PublishFault::None,
    )
    .unwrap();
    let failed_stage = create_test_stage(container.path(), "failed");
    let failed_generation = hash_bytes(&fs::read(failed_stage.join("manifest.json")).unwrap());
    assert!(
        publish_staged_generation(
            container.path(),
            &failed_stage,
            PublishFault::GenerationContentSync,
        )
        .is_err()
    );
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        active.generation
    );
    assert!(!generation_root(container.path(), &failed_generation).exists());
}

#[test]
fn generation_install_sync_failure_rolls_back_or_reconciles_before_pointer_commit() {
    let container = tempdir().unwrap();
    let active = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "active"),
        PublishFault::None,
    )
    .unwrap();

    let rollback_stage = create_test_stage(container.path(), "rollback");
    let rollback_generation = hash_bytes(&fs::read(rollback_stage.join("manifest.json")).unwrap());
    assert!(
        publish_staged_generation(
            container.path(),
            &rollback_stage,
            PublishFault::GenerationDirectorySync,
        )
        .is_err()
    );
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        active.generation
    );
    assert!(!generation_root(container.path(), &rollback_generation).exists());

    let orphan_stage = create_test_stage(container.path(), "orphan");
    let orphan_generation = hash_bytes(&fs::read(orphan_stage.join("manifest.json")).unwrap());
    assert!(
        publish_staged_generation(
            container.path(),
            &orphan_stage,
            PublishFault::GenerationDirectorySyncAndCleanup,
        )
        .is_err()
    );
    let orphan = generation_root(container.path(), &orphan_generation);
    assert!(orphan.exists());
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        active.generation
    );
    reconcile_fixture_container(container.path(), false, CleanupFault::None).unwrap();
    assert!(!orphan.exists());
    assert!(generation_root(container.path(), &active.generation).exists());
}

#[test]
fn existing_generation_is_resynchronized_and_never_rolled_back() {
    let container = tempdir().unwrap();
    let active = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "same"),
        PublishFault::None,
    )
    .unwrap();
    let duplicate_stage = create_test_stage(container.path(), "same");
    assert!(
        publish_staged_generation(
            container.path(),
            &duplicate_stage,
            PublishFault::GenerationContentSync,
        )
        .is_err()
    );
    assert_eq!(
        load_pointer(container.path()).unwrap().generation,
        active.generation
    );
    assert!(generation_root(container.path(), &active.generation).exists());

    let duplicate_stage = create_test_stage(container.path(), "same");
    assert!(
        publish_staged_generation(
            container.path(),
            &duplicate_stage,
            PublishFault::GenerationDirectorySync,
        )
        .is_err()
    );
    assert!(generation_root(container.path(), &active.generation).exists());
}

#[test]
fn post_replace_failure_preserves_both_generations_until_reopen() {
    for persist_new_pointer in [true, false] {
        let container = tempdir().unwrap();
        let first = publish_staged_generation(
            container.path(),
            &create_test_stage(container.path(), "first"),
            PublishFault::None,
        )
        .unwrap();
        let second = publish_staged_generation(
            container.path(),
            &create_test_stage(container.path(), "second"),
            PublishFault::AfterPointerReplace,
        )
        .unwrap();
        assert!(second.pointer_durability_pending);
        assert!(second.cleanup_pending);
        assert_eq!(
            load_pointer(container.path()).unwrap().generation,
            second.generation
        );
        assert!(generation_root(container.path(), &first.generation).exists());
        assert!(generation_root(container.path(), &second.generation).exists());

        if !persist_new_pointer {
            fs::write(
                container.path().join("CURRENT"),
                pointer_bytes(&first.generation),
            )
            .unwrap();
        }
        let reopened =
            reconcile_fixture_container(container.path(), false, CleanupFault::None).unwrap();
        assert!(!reopened.cleanup_pending);
        let expected = if persist_new_pointer {
            &second.generation
        } else {
            &first.generation
        };
        let inactive = if persist_new_pointer {
            &first.generation
        } else {
            &second.generation
        };
        assert_eq!(
            load_pointer(container.path()).unwrap().generation,
            *expected
        );
        assert!(generation_root(container.path(), expected).exists());
        assert!(!generation_root(container.path(), inactive).exists());
    }
}

#[test]
fn cleanup_removes_only_recognized_inactive_entries() {
    let container = tempdir().unwrap();
    let active_stage = container.path().join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&active_stage).unwrap();
    write_capture_set(
        &active_stage,
        &test_tools(),
        &test_capture(),
        &test_performance(),
    );
    let active =
        publish_staged_generation(container.path(), &active_stage, PublishFault::None).unwrap();
    let orphan_stage = create_test_stage(container.path(), "orphan");
    let orphan_manifest = fs::read(orphan_stage.join("manifest.json")).unwrap();
    let orphan_generation = hash_bytes(&orphan_manifest);
    let orphan_root = generation_root(container.path(), &orphan_generation);
    fs::rename(orphan_stage, &orphan_root).unwrap();
    let stale_stage = container.path().join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&stale_stage).unwrap();
    let stale_pointer = container
        .path()
        .join(format!(".CURRENT.tmp-{}", Uuid::new_v4()));
    fs::write(&stale_pointer, b"partial").unwrap();
    let unknown_stage = container.path().join(".stage-not-a-uuid");
    fs::create_dir(&unknown_stage).unwrap();
    let unknown_generation = container.path().join("generations/not-a-digest");
    fs::create_dir(&unknown_generation).unwrap();

    let reconciled =
        reconcile_fixture_container(container.path(), false, CleanupFault::None).unwrap();
    assert!(!reconciled.cleanup_pending);
    assert!(reconciled.selected.is_some());
    assert!(!stale_stage.exists());
    assert!(!stale_pointer.exists());
    assert!(unknown_stage.exists());
    assert!(unknown_generation.exists());
    assert!(generation_root(container.path(), &active.generation).exists());
    assert!(!orphan_root.exists());
}

#[test]
fn reconciliation_without_a_pointer_cleans_only_recognized_temporaries() {
    let container = tempdir().unwrap();
    let orphan_stage = create_test_stage(container.path(), "orphan");
    let orphan_manifest = fs::read(orphan_stage.join("manifest.json")).unwrap();
    let orphan_generation = hash_bytes(&orphan_manifest);
    let orphan_root = generation_root(container.path(), &orphan_generation);
    fs::create_dir_all(orphan_root.parent().unwrap()).unwrap();
    fs::rename(&orphan_stage, &orphan_root).unwrap();
    let stale_stage = container.path().join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&stale_stage).unwrap();
    let stale_pointer = container
        .path()
        .join(format!(".CURRENT.tmp-{}", Uuid::new_v4()));
    fs::write(&stale_pointer, b"partial").unwrap();

    let reconciled =
        reconcile_fixture_container(container.path(), true, CleanupFault::None).unwrap();
    assert!(reconciled.selected.is_none());
    assert!(!reconciled.cleanup_pending);
    assert!(!stale_stage.exists());
    assert!(!stale_pointer.exists());
    assert!(orphan_root.exists());
}

#[test]
fn failed_startup_cleanup_is_non_fatal_and_keeps_selected_generation() {
    let container = tempdir().unwrap();
    let active = publish_staged_generation(
        container.path(),
        &create_test_stage(container.path(), "active"),
        PublishFault::None,
    )
    .unwrap();
    let stale_stage = container.path().join(format!(".stage-{}", Uuid::new_v4()));
    fs::create_dir(&stale_stage).unwrap();

    let reconciled =
        reconcile_fixture_container(container.path(), false, CleanupFault::Fail).unwrap();
    assert!(reconciled.cleanup_pending);
    assert!(reconciled.selected.is_some());
    assert!(stale_stage.exists());
    assert!(generation_root(container.path(), &active.generation).exists());
}

#[test]
fn checked_in_stage_observability_generation_digest_is_unchanged() {
    assert_eq!(
        load_pointer(&fixture_container_root()).unwrap().generation,
        "259e3521eddf4d1ed46113dee7d9e0c5bd0ed218948755b2d43c9c6eaaa4c708"
    );
}

#[test]
fn pointer_validation_rejects_unknown_fields_and_digest_mismatch() {
    let container = tempdir().unwrap();
    fs::write(
        container.path().join("CURRENT"),
        r#"{"schemaVersion":1,"generation":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","unknown":true}"#,
    )
    .unwrap();
    assert!(load_pointer(container.path()).is_err());
    fs::write(
        container.path().join("CURRENT"),
        pointer_bytes(&"a".repeat(64)),
    )
    .unwrap();
    fs::create_dir_all(generation_root(container.path(), &"a".repeat(64))).unwrap();
    fs::write(
        generation_root(container.path(), &"a".repeat(64)).join("manifest.json"),
        b"{}",
    )
    .unwrap();
    assert!(selected_generation_root(container.path()).is_err());
}

#[test]
fn ambiguous_replace_error_uses_the_strict_pointer_to_classify_commit() {
    let container = tempdir().unwrap();
    let generation = "a".repeat(64);
    let current = container.path().join("CURRENT");
    fs::write(&current, pointer_bytes(&generation)).unwrap();
    assert_eq!(
        confirm_failed_pointer_replace(&current, &generation, "ambiguous replacement failure"),
        PointerReplaceOutcome::Committed {
            durability_pending: true
        }
    );

    fs::write(
        &current,
        format!(r#"{{"schemaVersion":1,"generation":"{generation}","unknown":true}}"#),
    )
    .unwrap();
    assert!(matches!(
        confirm_failed_pointer_replace(&current, &generation, "ambiguous replacement failure"),
        PointerReplaceOutcome::Uncommitted { .. }
    ));
}

#[test]
fn comparisons_reject_semantic_and_decoded_drift() {
    assert_eq!(
        structural_similarity(&[0, 10, 20], &[0, 10, 20]).unwrap(),
        1.0
    );
    assert!(structural_similarity(&[0, 10, 20], &[255, 255, 255]).unwrap() < SSIM_MINIMUM);
    assert!(aligned_rms_error(&[0.0, 0.0], &[0.1, 0.1], 1).unwrap() > PCM_RMS_MAXIMUM);
    assert!(bytes_to_f32(&f32::NAN.to_le_bytes()).is_err());
}

#[test]
fn reference_paths_reject_rfc3986_schemes_and_allow_portable_relatives() {
    for value in [
        "file:frame.rgb",
        "data:text/plain,frame",
        "HtTpS:frame.rgb",
        "git+ssh:frame.rgb",
        "a-b.c:frame.rgb",
    ] {
        assert!(
            safe_reference_path(Path::new("fixture"), value).is_err(),
            "{value}"
        );
    }
    for value in ["frames/0000.rgb", "audio/reference.f32le", "frame_name.rgb"] {
        assert!(
            safe_reference_path(Path::new("fixture"), value).is_ok(),
            "{value}"
        );
    }
}

#[test]
fn pcm_alignment_searches_both_directions_and_enforces_bounds() {
    let signal = [0.13, -0.27, 0.44, 0.08, -0.51, 0.36];
    assert_eq!(aligned_rms_error(&signal, &signal, 2).unwrap(), 0.0);
    assert_eq!(
        aligned_rms_error(&signal, &[9.0, 0.13, -0.27, 0.44, 0.08, -0.51], 1).unwrap(),
        0.0
    );
    assert_eq!(
        aligned_rms_error(&[9.0, 0.13, -0.27, 0.44, 0.08, -0.51], &signal, 1).unwrap(),
        0.0
    );
    assert_eq!(
        aligned_rms_error(&signal, &[8.0, 7.0, 0.13, -0.27, 0.44, 0.08], 2).unwrap(),
        0.0
    );
    assert!(aligned_rms_error(&signal, &[0.0; 3], 2).is_err());
    assert!(aligned_rms_error(&[1.0], &[1.0], 1).is_err());
    assert!(aligned_rms_error(&[f32::NAN, 1.0], &[0.0, 1.0], 1).is_err());
    assert!(aligned_rms_error(&signal, &[1.0; 6], 2).unwrap() > PCM_RMS_MAXIMUM);
}

#[test]
fn performance_aggregation_uses_medians_and_process_tree_maximum() {
    let mut captures = vec![test_capture(), test_capture(), test_capture()];
    for (capture, value) in captures.iter_mut().zip([3.0, 1.0, 2.0]) {
        capture.intent_observations = test_intent_observations(value);
        capture.peak_process_tree_bytes = (value * 100.0) as u64;
    }
    let baseline = aggregate_performance(&test_tools(), &captures);
    assert_eq!(baseline.warmup_samples, 1);
    assert_eq!(baseline.measured_samples, 3);
    assert_eq!(baseline.intent_observations.len(), 3);
    assert_eq!(baseline.intent_observations[2].timings_ms.end_to_end, 2.0);
    assert!(baseline.non_additive_stage_timings);
    assert_eq!(baseline.peak_resident_working_set_bytes, 300);
    assert_eq!(baseline.memory_scope, "process_tree");
    assert_eq!(baseline.timing_aggregation, "median");
    assert_eq!(baseline.memory_aggregation, "maximum");
}

#[test]
fn process_tree_sampler_observes_a_child_allocation() {
    let status = Command::new(env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "renderer::golden::process_tree_sampler_isolated_helper",
        ])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
#[ignore = "isolates sampler baseline from parallel test helper processes"]
fn process_tree_sampler_isolated_helper() {
    let baseline_sampler = ProcessTreeSampler::start();
    thread::sleep(Duration::from_millis(100));
    let baseline = baseline_sampler.finish();
    let sampler = ProcessTreeSampler::start();
    let status = Command::new(env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "renderer::golden::process_tree_memory_child_helper",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let with_child = sampler.finish();
    assert!(with_child >= baseline.saturating_add(32 * 1024 * 1024));
}

#[test]
fn process_tree_sampler_finish_signals_and_joins_worker() {
    let exited = Arc::new(AtomicBool::new(false));
    let sampler = ProcessTreeSampler::start_with_exit_signal(Some(Arc::clone(&exited)));
    sampler.finish();
    assert!(exited.load(Ordering::Acquire));
}

#[test]
fn process_tree_sampler_finish_surfaces_worker_panic_without_drop_double_panic() {
    let sampler = ProcessTreeSampler {
        stop: Arc::new(AtomicBool::new(false)),
        peak: Arc::new(AtomicU64::new(0)),
        handle: Some(thread::spawn(|| panic!("injected sampler worker panic"))),
    };
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sampler.finish())).is_err());
}

#[test]
fn process_tree_sampler_drop_signals_and_joins_during_unwind() {
    let exited = Arc::new(AtomicBool::new(false));
    let stop = Arc::new(std::sync::Mutex::new(None));
    let result = std::panic::catch_unwind({
        let exited = Arc::clone(&exited);
        let stop = Arc::clone(&stop);
        move || {
            let sampler = ProcessTreeSampler::start_with_exit_signal(Some(Arc::clone(&exited)));
            *stop.lock().unwrap() = Some(Arc::clone(&sampler.stop));
            panic!("exercise sampler cleanup during unwinding");
        }
    });
    assert!(result.is_err());
    assert!(exited.load(Ordering::Acquire));
    assert!(
        stop.lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .load(Ordering::Acquire)
    );
}

#[test]
#[ignore = "helper process for process_tree_sampler_observes_a_child_allocation"]
fn process_tree_memory_child_helper() {
    let allocation = vec![0x5a_u8; 64 * 1024 * 1024];
    std::hint::black_box(&allocation);
    thread::sleep(Duration::from_millis(300));
}

#[test]
fn coordinated_output_drift_still_fails_the_reviewed_reference() {
    let root = selected_generation_root(&fixture_container_root()).unwrap();
    let manifest = load_manifest(&root).unwrap();
    let frames = SAMPLE_TIMESTAMPS_MS
        .into_iter()
        .map(|at_ms| {
            (
                at_ms,
                reference_bytes(&root, &manifest, "frame", Some(at_ms)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let audio = bytes_to_f32(&reference_bytes(&root, &manifest, "audio", None)).unwrap();
    let mut capture = Capture {
        frames: frames.clone(),
        range_frames: frames.clone(),
        export_frames: frames,
        range_audio: audio.clone(),
        export_audio: audio,
        range_duration_ms: DURATION_MS,
        export_duration_ms: DURATION_MS,
        semantic_plan: String::from_utf8(reference_bytes(&root, &manifest, "semantic", None))
            .unwrap(),
        filter_graph: String::from_utf8(reference_bytes(&root, &manifest, "graph", None)).unwrap(),
        intent_observations: test_intent_observations(0.0),
        peak_process_tree_bytes: 0,
    };
    for frames in [
        &mut capture.frames,
        &mut capture.range_frames,
        &mut capture.export_frames,
    ] {
        for value in frames.get_mut(&500).unwrap() {
            *value = 255_u8.wrapping_sub(*value);
        }
    }
    assert!(
        compare_capture(&root, &manifest, &capture)
            .unwrap_err()
            .contains("SSIM")
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InjectedBenchmarkFailure {
    Encoding,
    Decode,
    Composite,
}

#[derive(Debug)]
struct RecordingBenchmarkProcess {
    calls: Arc<std::sync::Mutex<Vec<&'static str>>>,
    failure: InjectedBenchmarkFailure,
}

impl ProcessExecutor for RecordingBenchmarkProcess {
    fn readiness(&self, _ffmpeg_path: &Path, _ffprobe_path: &Path) -> Result<(), CoreError> {
        Ok(())
    }

    fn probe(&self, _ffprobe_path: &Path, _path: &Path) -> Result<ProbeResult, CoreError> {
        unreachable!("benchmark intent does not probe encoded output")
    }

    fn execute(
        &self,
        _ffmpeg_path: &Path,
        _plan: &RenderPlan,
        _filter_path: &Path,
        output: &Path,
        _on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        self.calls.lock().unwrap().push("encoding");
        fs::write(output, b"partial benchmark output").unwrap();
        if self.failure == InjectedBenchmarkFailure::Encoding {
            Err(CoreError::render_failure(RENDER_STAGE, Some(1), None))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
struct RecordingBenchmarkProbes {
    calls: Arc<std::sync::Mutex<Vec<&'static str>>>,
    failure: Option<InjectedBenchmarkFailure>,
}

impl BenchmarkProbeExecutor for RecordingBenchmarkProbes {
    fn execute(
        &self,
        stage: BenchmarkProbeStage,
        _command: &mut Command,
        _duration_ms: u64,
    ) -> Result<(), CoreError> {
        let (name, failure) = match stage {
            BenchmarkProbeStage::Decode => ("decode", InjectedBenchmarkFailure::Decode),
            BenchmarkProbeStage::Composite => ("composite", InjectedBenchmarkFailure::Composite),
        };
        self.calls.lock().unwrap().push(name);
        if self.failure == Some(failure) {
            Err(CoreError::render_failure(name, Some(1), None))
        } else {
            Ok(())
        }
    }
}

#[test]
fn ordinary_conformance_collects_no_benchmark_observations() {
    let renderer = Renderer::new("must-not-run-ffmpeg", "must-not-run-ffprobe", None);
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let probes = RecordingBenchmarkProbes {
        calls: Arc::clone(&calls),
        failure: None,
    };
    let root = tempdir().unwrap();

    let observations = capture_intent_observations(
        CaptureMode::Conformance,
        &renderer,
        &fixture_project(),
        root.path(),
        &probes,
    )
    .unwrap();

    assert!(observations.is_empty());
    assert!(calls.lock().unwrap().is_empty());
}

#[test]
fn benchmark_mode_collects_every_intent_and_probe_in_order() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut renderer = Renderer::new("unused-ffmpeg", "unused-ffprobe", None);
    renderer.process_executor = Arc::new(RecordingBenchmarkProcess {
        calls: Arc::clone(&calls),
        failure: InjectedBenchmarkFailure::Decode,
    });
    let probes = RecordingBenchmarkProbes {
        calls: Arc::clone(&calls),
        failure: None,
    };

    let observations = capture_intent_observations(
        CaptureMode::Benchmark {
            sample_memory: false,
        },
        &renderer,
        &fixture_project(),
        root.path(),
        &probes,
    )
    .unwrap();

    assert_eq!(
        observations
            .iter()
            .map(|observation| observation.intent.as_str())
            .collect::<Vec<_>>(),
        vec!["framePreview", "audiovisualRangePreview", "finalExport"]
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec![
            "encoding",
            "decode",
            "composite",
            "encoding",
            "decode",
            "composite",
            "encoding",
            "decode",
            "composite"
        ]
    );
}

#[test]
fn each_benchmark_process_failure_stops_and_cleans_up() {
    for (failure, expected_calls) in [
        (InjectedBenchmarkFailure::Encoding, vec!["encoding"]),
        (InjectedBenchmarkFailure::Decode, vec!["encoding", "decode"]),
        (
            InjectedBenchmarkFailure::Composite,
            vec!["encoding", "decode", "composite"],
        ),
    ] {
        let root = tempdir().unwrap();
        fs::create_dir(root.path().join("assets")).unwrap();
        write_tone_wav(&root.path().join("assets/tone.wav"));
        fs::write(root.path().join("CURRENT"), b"selected-generation").unwrap();
        let project = fixture_project();
        let project_before = serde_json::to_vec(&project).unwrap();
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut renderer = Renderer::new("unused-ffmpeg", "unused-ffprobe", None);
        renderer.process_executor = Arc::new(RecordingBenchmarkProcess {
            calls: Arc::clone(&calls),
            failure,
        });
        let probes = RecordingBenchmarkProbes {
            calls: Arc::clone(&calls),
            failure: Some(failure),
        };

        let result = benchmark_intent(
            &renderer,
            &project,
            root.path(),
            RenderIntent::Export,
            &probes,
        );

        assert!(result.is_err(), "{failure:?} unexpectedly succeeded");
        assert_eq!(*calls.lock().unwrap(), expected_calls);
        assert_eq!(serde_json::to_vec(&project).unwrap(), project_before);
        assert_eq!(
            fs::read(root.path().join("CURRENT")).unwrap(),
            b"selected-generation"
        );
        let residue = fs::read_dir(root.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(".opencut-"))
            .collect::<Vec<_>>();
        assert!(
            residue.is_empty(),
            "benchmark residue remained after {failure:?}: {residue:?}"
        );
    }
}

#[test]
fn invalid_render_work_preserves_project_and_files() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    let renderer = Renderer::new("must-not-run-ffmpeg", "must-not-run-ffprobe", None);

    let mut missing = fixture_project();
    missing.assets.clear();
    let before = serde_json::to_vec(&missing).unwrap();
    let files_before = directory_entries(root.path());
    assert_eq!(
        renderer
            .render_preview(&missing, root.path(), 0)
            .unwrap_err()
            .code,
        ErrorCode::AssetNotFound
    );
    assert_eq!(serde_json::to_vec(&missing).unwrap(), before);
    assert_eq!(directory_entries(root.path()), files_before);

    let mut invalid = fixture_project();
    let asset = &mut invalid.assets[0];
    asset.duration_ms = Some(DURATION_MS);
    let TimelineItem::Media(item) = &mut invalid.tracks[1].items[0] else {
        unreachable!()
    };
    item.source_in_ms = 1;
    let before = serde_json::to_vec(&invalid).unwrap();
    assert_eq!(
        renderer
            .render_preview(&invalid, root.path(), 0)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(serde_json::to_vec(&invalid).unwrap(), before);
    assert_eq!(directory_entries(root.path()), files_before);
}

fn directory_entries(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, current: &Path, entries: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(current).unwrap() {
            let path = entry.unwrap().path();
            entries.push(path.strip_prefix(root).unwrap().to_owned());
            if path.is_dir() {
                visit(root, &path, entries);
            }
        }
    }
    let mut entries = Vec::new();
    visit(root, root, &mut entries);
    entries.sort();
    entries
}
