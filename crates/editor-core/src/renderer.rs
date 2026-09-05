use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use uuid::Uuid;

use crate::{
    CoreError, ErrorCode, Project,
    evaluated_scene::{
        EvaluatedScene, EvaluatedSceneResult, EvaluatedVisualSource, evaluate_layer_affine,
        evaluate_project, finalize_affine_geometry,
    },
    render_artifact::{
        ArtifactIo, FileSystemArtifactIo, GRAPH_BUILD_STAGE, MeasuredText, PreparedMediaResources,
        RenderArtifact, RenderWorkspace, artifact_with, measure_evaluated_text_layers,
        prepare_media_resources, prepare_render_resources, publish_output_with, temporary_output,
        write_filter_script,
    },
    render_plan::{RenderIntent, RenderPlan, build_render_plan},
    render_process::{
        ProbeResult, ProcessExecutor, RenderProgress, SystemProcessExecutor, map_renderer_error,
    },
};

#[cfg(test)]
use crate::render_artifact::{
    PUBLISH_STAGE, media_input_requests, wrap_text, wrap_text_with_measure,
};
#[cfg(test)]
use crate::render_plan::{
    FilterContext, audible_voiceover_intervals, ducking_expression, ducking_gain_at, escape_filter,
    format_number, merge_intervals, piecewise_expression, position_expression, scalar_expression,
};
#[cfg(test)]
use crate::render_process::{
    RENDER_STAGE, SPAWN_STAGE, STDERR_EXCERPT_BYTES, STDERR_TAIL_BYTES,
    build_composite_benchmark_command, build_decode_benchmark_command, read_bounded_tail,
    run_to_completion, sanitize_stderr, stderr_excerpt,
};

#[cfg(test)]
use crate::{KeyframeProperty, KeyframeValue};

#[derive(Clone, Debug)]
pub struct Renderer {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
    default_font_path: Option<PathBuf>,
    font_roots: Vec<PathBuf>,
    process_executor: Arc<dyn ProcessExecutor>,
    artifact_io: Arc<dyn ArtifactIo>,
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

struct PreparedRender {
    plan: RenderPlan,
    filter_path: PathBuf,
    _workspace: RenderWorkspace,
    warnings: Vec<String>,
}

struct RenderPreflight {
    scene: EvaluatedScene,
    media: PreparedMediaResources,
    measured: HashMap<String, MeasuredText>,
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
            process_executor: Arc::new(SystemProcessExecutor),
            artifact_io: Arc::new(FileSystemArtifactIo),
        }
    }

    #[cfg(test)]
    fn with_adapters(
        mut self,
        process_executor: Arc<dyn ProcessExecutor>,
        artifact_io: Arc<dyn ArtifactIo>,
    ) -> Self {
        self.process_executor = process_executor;
        self.artifact_io = artifact_io;
        self
    }

    pub fn with_font_roots(mut self, roots: impl IntoIterator<Item = PathBuf>) -> Self {
        self.font_roots = roots.into_iter().collect();
        self
    }

    pub fn readiness(&self) -> Result<(), CoreError> {
        self.process_executor
            .readiness(&self.ffmpeg_path, &self.ffprobe_path)
    }

    pub fn probe(&self, path: &Path) -> Result<ProbeResult, CoreError> {
        self.process_executor.probe(&self.ffprobe_path, path)
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
        let evaluated = evaluate_project(
            project,
            project.settings.width,
            project.settings.height,
            project.settings.fps,
        )?;
        let media = prepare_media_resources(self.artifact_io.as_ref(), &evaluated, project_dir)?;
        let preflight = self.preflight_render(&evaluated, media)?;
        let file_name = format!("preview-{}.png", Uuid::new_v4());
        let output = project_dir.join("previews").join(&file_name);
        let temporary = temporary_output(
            self.artifact_io.as_ref(),
            output.parent().unwrap_or(project_dir),
            "png",
        );
        let built = self.materialize_render(
            preflight,
            project_dir,
            RenderIntent::Frame { at_ms: time_ms },
        )?;
        if let Err(error) = self.process_executor.execute(
            &self.ffmpeg_path,
            &built.plan,
            &built.filter_path,
            &temporary,
            &mut |_| {},
        ) {
            let _ = self.artifact_io.remove(&temporary);
            return Err(error);
        }
        publish_output_with(self.artifact_io.as_ref(), &temporary, &output, false)?;
        artifact_with(
            self.artifact_io.as_ref(),
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
        let evaluated =
            evaluate_project(project, options.width, options.height, project.settings.fps)?;
        let media = prepare_media_resources(self.artifact_io.as_ref(), &evaluated, project_dir)?;
        let preflight = self.preflight_render(&evaluated, media)?;
        if self.artifact_io.artifact_path_exists(options.output) && !options.overwrite {
            return Err(CoreError::new(
                ErrorCode::ExportExists,
                "export already exists; pass overwrite=true only with explicit permission",
            ));
        }
        let temporary = temporary_output(
            self.artifact_io.as_ref(),
            options.output.parent().unwrap_or_else(|| Path::new(".")),
            "mp4",
        );
        let built = self.materialize_render(preflight, project_dir, RenderIntent::Export)?;
        if let Err(error) = self.process_executor.execute(
            &self.ffmpeg_path,
            &built.plan,
            &built.filter_path,
            &temporary,
            &mut on_progress,
        ) {
            let _ = self.artifact_io.remove(&temporary);
            return Err(error);
        }
        publish_output_with(
            self.artifact_io.as_ref(),
            &temporary,
            options.output,
            options.overwrite,
        )?;
        artifact_with(
            self.artifact_io.as_ref(),
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
        let evaluated = evaluate_project(project, options.width, options.height, options.fps)?;
        let media = prepare_media_resources(self.artifact_io.as_ref(), &evaluated, project_dir)?;
        let preflight = self.preflight_render(&evaluated, media)?;
        let file_name = format!("preview-range-{}.mp4", Uuid::new_v4());
        let output = project_dir.join("previews").join(&file_name);
        let temporary = temporary_output(
            self.artifact_io.as_ref(),
            output.parent().unwrap_or(project_dir),
            "mp4",
        );
        let built = self.materialize_render(
            preflight,
            project_dir,
            RenderIntent::Range {
                start_ms: options.start_ms,
                end_ms: options.end_ms,
                include_audio: options.include_audio,
            },
        )?;
        let mut on_progress = on_progress;
        if let Err(error) = self.process_executor.execute(
            &self.ffmpeg_path,
            &built.plan,
            &built.filter_path,
            &temporary,
            &mut on_progress,
        ) {
            let _ = self.artifact_io.remove(&temporary);
            return Err(error);
        }
        publish_output_with(self.artifact_io.as_ref(), &temporary, &output, false)?;
        artifact_with(
            self.artifact_io.as_ref(),
            &output,
            format!("previews/{file_name}"),
            "video/mp4",
            built.warnings.clone(),
        )
    }

    #[cfg(test)]
    fn prepare_render(
        &self,
        evaluated: &EvaluatedSceneResult,
        media: PreparedMediaResources,
        project_dir: &Path,
        intent: RenderIntent,
    ) -> Result<PreparedRender, CoreError> {
        self.materialize_render(
            self.preflight_render(evaluated, media)?,
            project_dir,
            intent,
        )
    }

    fn preflight_render(
        &self,
        evaluated: &EvaluatedSceneResult,
        media: PreparedMediaResources,
    ) -> Result<RenderPreflight, CoreError> {
        let mut warnings = Vec::new();
        let mut measured = measure_evaluated_text_layers(
            self.artifact_io.as_ref(),
            evaluated,
            self.default_font_path.as_deref(),
            &self.font_roots,
            &mut warnings,
        )?;
        let mut finalized = evaluated.scene.clone();
        let mut measurements: HashMap<String, (u32, u32)> = measured
            .iter()
            .map(|(id, text)| {
                (
                    id.clone(),
                    (text.prepared.layer_width, text.prepared.layer_height),
                )
            })
            .collect();
        // Text geometry failures take precedence over any metadata process call.
        for layer in &finalized.visual_layers {
            if layer.requires_affine()
                && let Some(size) = measurements.get(&layer.item_id)
            {
                evaluate_layer_affine(
                    layer,
                    *size,
                    (finalized.canvas.width, finalized.canvas.height),
                )?;
            }
        }
        let mut asset_sizes = HashMap::new();
        for layer in &finalized.visual_layers {
            if !layer.requires_affine() {
                continue;
            }
            if let EvaluatedVisualSource::Media { asset_id, .. } = &layer.source {
                let size = if let Some(size) = asset_sizes.get(asset_id) {
                    *size
                } else {
                    let index = media
                        .media_inputs
                        .iter()
                        .position(|input| input.item_id == layer.item_id)
                        .ok_or_else(|| {
                            CoreError::new(
                                ErrorCode::InternalError,
                                "missing media geometry binding",
                            )
                        })?;
                    let size = self.process_executor.probe_render_geometry(
                        &self.ffprobe_path,
                        &media.media_paths[index],
                        media.media_inputs[index].media_type,
                    )?;
                    asset_sizes.insert(asset_id.clone(), size);
                    size
                };
                measurements.insert(layer.item_id.clone(), size);
            }
        }
        finalize_affine_geometry(&mut finalized, &measurements)?;
        measured.retain(|id, _| {
            finalized
                .visual_layers
                .iter()
                .any(|layer| &layer.item_id == id)
        });
        if finalized.visual_layers.iter().any(|layer| {
            layer.requires_affine()
                && layer
                    .source_size
                    .is_some_and(|(w, h)| w >= 65_535 || h >= 65_535)
        }) {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                "affine backend cannot address this source raster",
            ));
        }
        if finalized
            .visual_layers
            .iter()
            .any(|layer| layer.affine.is_some())
        {
            self.readiness()?;
        }
        Ok(RenderPreflight {
            scene: finalized,
            media,
            measured,
            warnings,
        })
    }

    fn materialize_render(
        &self,
        preflight: RenderPreflight,
        project_dir: &Path,
        intent: RenderIntent,
    ) -> Result<PreparedRender, CoreError> {
        let RenderPreflight {
            scene: finalized,
            media,
            measured,
            mut warnings,
        } = preflight;
        let workspace = RenderWorkspace::create(self.artifact_io.clone(), project_dir)?;
        let resources =
            prepare_render_resources(self.artifact_io.as_ref(), media, workspace.path(), measured)?;
        let filter_path = workspace.path().join("filter.txt");
        let plan = build_render_plan(
            &finalized,
            &resources.text_layers,
            resources.media_inputs,
            resources.media_paths,
            self.default_font_path.as_deref(),
            intent,
            &mut warnings,
        )
        .map_err(|error| map_renderer_error(error, GRAPH_BUILD_STAGE))?;
        write_filter_script(self.artifact_io.as_ref(), &filter_path, &plan.filter_graph)?;
        Ok(PreparedRender {
            plan,
            filter_path,
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
        let evaluated = evaluate_project(project, context.width, context.height, context.fps)?;
        let media_inputs = media_input_requests(&evaluated)?;
        build_render_plan(
            &evaluated.scene,
            context.text_layers,
            media_inputs,
            Vec::new(),
            self.default_font_path.as_deref(),
            RenderIntent::Export,
            warnings,
        )
        .map(|plan| plan.filter_graph)
    }
}

#[cfg(test)]
mod golden;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaptionItem, CaptionSource, CaptionStyle, Easing, Keyframe, MediaItem, MediaType,
        PROJECT_SCHEMA_VERSION, ProjectSettings, SolidColorItem, TimelineItem, Track, TrackType,
        Transform,
        render_artifact::{ArtifactEntryKind, artifact, prepare_text_layers, publish_output},
        render_plan::seconds,
        render_process::{build_render_command, run_to_completion},
    };
    use std::{
        collections::HashMap,
        env,
        fs::File,
        process::Command,
        sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };
    use tempfile::tempdir;

    #[derive(Clone, Copy, Debug)]
    enum FakeRunFailure {
        Spawn,
        Exit,
    }

    #[derive(Debug)]
    struct FakeProcess {
        readiness_error: bool,
        probe_error: bool,
        run_failure: Option<FakeRunFailure>,
        executions: Mutex<Vec<RenderIntent>>,
    }

    impl ProcessExecutor for FakeProcess {
        fn readiness(&self, _ffmpeg_path: &Path, _ffprobe_path: &Path) -> Result<(), CoreError> {
            if self.readiness_error {
                Err(CoreError::new(
                    ErrorCode::DependencyUnavailable,
                    "injected readiness",
                ))
            } else {
                Ok(())
            }
        }

        fn probe(&self, _ffprobe_path: &Path, _path: &Path) -> Result<ProbeResult, CoreError> {
            if self.probe_error {
                Err(CoreError::new(
                    ErrorCode::UnsupportedMedia,
                    "injected probe",
                ))
            } else {
                Ok(ProbeResult {
                    duration_ms: Some(1),
                    has_video: true,
                    has_audio: false,
                    format_name: None,
                    video_codec: None,
                    video_width: None,
                    video_height: None,
                    audio_codec: None,
                    audio_channels: None,
                    audio_sample_rate_hz: None,
                })
            }
        }

        fn execute(
            &self,
            _ffmpeg_path: &Path,
            plan: &RenderPlan,
            _filter_path: &Path,
            output: &Path,
            on_progress: &mut dyn FnMut(RenderProgress),
        ) -> Result<(), CoreError> {
            self.executions.lock().unwrap().push(plan.intent);
            match self.run_failure {
                Some(FakeRunFailure::Spawn) => {
                    return Err(CoreError::render_failure(SPAWN_STAGE, None, None));
                }
                Some(FakeRunFailure::Exit) => {
                    return Err(CoreError::render_failure(
                        RENDER_STAGE,
                        Some(7),
                        Some("injected diagnostic".into()),
                    ));
                }
                None => {}
            }
            std::fs::write(output, b"rendered").unwrap();
            on_progress(RenderProgress { progress: 1.0 });
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct FakeArtifactIo {
        fail_rename: bool,
        fail_size: bool,
        removed: Mutex<Vec<PathBuf>>,
    }

    impl ArtifactIo for FakeArtifactIo {
        fn request_id(&self) -> String {
            Uuid::new_v4().to_string()
        }

        fn create_dir(&self, path: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.create_dir(path)
        }

        fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove_dir_all(path)
        }

        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            FileSystemArtifactIo.read(path)
        }

        fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
            FileSystemArtifactIo.write(path, contents)
        }

        fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
            FileSystemArtifactIo.list(path)
        }

        fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind> {
            FileSystemArtifactIo.entry_kind(path)
        }

        fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf> {
            FileSystemArtifactIo.canonicalize_artifact_path(path)
        }

        fn artifact_path_exists(&self, path: &Path) -> bool {
            path.exists()
        }

        fn remove(&self, path: &Path) -> std::io::Result<()> {
            self.removed.lock().unwrap().push(path.to_owned());
            match std::fs::remove_file(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        }

        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            if self.fail_rename {
                Err(std::io::Error::other("injected publication failure"))
            } else {
                std::fs::rename(from, to)
            }
        }

        fn size(&self, path: &Path) -> std::io::Result<u64> {
            if self.fail_size {
                Err(std::io::Error::other("injected metadata failure"))
            } else {
                Ok(path.metadata()?.len())
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ArtifactFailure {
        CreateWorkspace,
        Write,
        Canonicalize,
        Read,
        List,
    }

    #[derive(Debug, Default)]
    struct LifecycleArtifactIo {
        next_failure: Mutex<Option<ArtifactFailure>>,
        canonical_escape: Mutex<Option<(PathBuf, PathBuf)>>,
        events: Mutex<Vec<&'static str>>,
        request_counter: AtomicUsize,
    }

    impl LifecycleArtifactIo {
        fn fail_next(&self, failure: ArtifactFailure) {
            *self.next_failure.lock().unwrap() = Some(failure);
        }

        fn take(&self, failure: ArtifactFailure) -> bool {
            let mut next = self.next_failure.lock().unwrap();
            if *next == Some(failure) {
                *next = None;
                true
            } else {
                false
            }
        }

        fn record(&self, event: &'static str) {
            self.events.lock().unwrap().push(event);
        }

        fn map_canonical_path(&self, input: PathBuf, output: PathBuf) {
            *self.canonical_escape.lock().unwrap() = Some((input, output));
        }

        fn clear_events(&self) {
            self.events.lock().unwrap().clear();
        }
    }

    impl ArtifactIo for LifecycleArtifactIo {
        fn request_id(&self) -> String {
            self.record("request_id");
            format!(
                "lifecycle-{}",
                self.request_counter.fetch_add(1, Ordering::SeqCst)
            )
        }

        fn create_dir(&self, path: &Path) -> std::io::Result<()> {
            self.record("create_dir");
            if self.take(ArtifactFailure::CreateWorkspace) {
                Err(std::io::Error::other("injected workspace failure"))
            } else {
                FileSystemArtifactIo.create_dir(path)
            }
        }

        fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
            self.record("remove_dir_all");
            FileSystemArtifactIo.remove_dir_all(path)
        }

        fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
            self.record("read");
            if self.take(ArtifactFailure::Read) {
                Err(std::io::Error::other("injected read failure"))
            } else {
                FileSystemArtifactIo.read(path)
            }
        }

        fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
            self.record("write");
            if self.take(ArtifactFailure::Write) {
                Err(std::io::Error::other("injected write failure"))
            } else {
                FileSystemArtifactIo.write(path, contents)
            }
        }

        fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
            self.record("list");
            if self.take(ArtifactFailure::List) {
                Err(std::io::Error::other("injected list failure"))
            } else {
                FileSystemArtifactIo.list(path)
            }
        }

        fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind> {
            self.record("entry_kind");
            FileSystemArtifactIo.entry_kind(path)
        }

        fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf> {
            self.record("canonicalize");
            if self.take(ArtifactFailure::Canonicalize) {
                Err(std::io::Error::other("injected canonicalize failure"))
            } else if let Some((input, output)) = self.canonical_escape.lock().unwrap().as_ref()
                && path == input
            {
                Ok(output.clone())
            } else {
                FileSystemArtifactIo.canonicalize_artifact_path(path)
            }
        }

        fn artifact_path_exists(&self, path: &Path) -> bool {
            self.record("exists");
            FileSystemArtifactIo.artifact_path_exists(path)
        }

        fn remove(&self, path: &Path) -> std::io::Result<()> {
            self.record("remove");
            match FileSystemArtifactIo.remove(path) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        }

        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            self.record("rename");
            FileSystemArtifactIo.rename(from, to)
        }

        fn size(&self, path: &Path) -> std::io::Result<u64> {
            self.record("size");
            FileSystemArtifactIo.size(path)
        }
    }

    fn empty_project() -> Project {
        Project {
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 0,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    fn visual_project() -> Project {
        let mut project = empty_project();
        project.settings.width = 320;
        project.settings.height = 180;
        project.settings.fps = 15;
        project.tracks.push(Track {
            id: "overlay".into(),
            name: "Overlay".into(),
            track_type: TrackType::Overlay,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: crate::AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::SolidColor(SolidColorItem {
                id: "background".into(),
                color: "#112233".into(),
                start_ms: 0,
                duration_ms: 1_000,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            })],
        });
        project
    }

    fn assert_no_render_side_effects(artifact_io: &LifecycleArtifactIo, process: &FakeProcess) {
        let events = artifact_io.events.lock().unwrap();
        assert!(
            events.iter().all(|event| *event == "canonicalize"),
            "unexpected mutating artifact event(s): {events:?}"
        );
        assert!(process.executions.lock().unwrap().is_empty());
    }

    fn assert_all_facades_reject_without_side_effects(
        renderer: &Renderer,
        artifact_io: &LifecycleArtifactIo,
        process: &FakeProcess,
        project: &Project,
        root: &Path,
        expected: ErrorCode,
    ) {
        let preview_error = renderer.render_preview(project, root, 0).unwrap_err();
        assert_eq!(preview_error.code, expected);
        assert_no_render_side_effects(artifact_io, process);

        artifact_io.clear_events();
        let range_error = renderer
            .render_preview_range(
                project,
                root,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: project.duration_ms().max(1),
                    width: 320,
                    height: 180,
                    fps: 15,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(range_error.code, expected);
        assert_no_render_side_effects(artifact_io, process);

        artifact_io.clear_events();
        let output = root.join("rejected.mp4");
        let export_error = renderer
            .export_video(
                project,
                root,
                ExportOptions {
                    output: &output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(export_error.code, expected);
        assert_no_render_side_effects(artifact_io, process);
        assert!(!output.exists());
        artifact_io.clear_events();
    }

    #[test]
    fn canonical_evaluation_failure_precedes_workspace_and_process_side_effects() {
        let mut project = empty_project();
        project.tracks.push(Track {
            id: "video".into(),
            name: "Video".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: crate::AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::Media(MediaItem {
                id: "missing".into(),
                asset_id: "missing-asset".into(),
                start_ms: 0,
                duration_ms: 1_000,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: crate::AudioSettings::default(),
                keyframes: vec![],
            })],
        });
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(LifecycleArtifactIo::default());
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(process.clone(), artifact_io.clone());
        let root = tempdir().unwrap();

        let error = renderer
            .render_preview(&project, root.path(), 0)
            .unwrap_err();

        assert_eq!(error.code, ErrorCode::AssetNotFound);
        assert!(artifact_io.events.lock().unwrap().is_empty());
        assert!(process.executions.lock().unwrap().is_empty());

        project.assets.push(crate::Asset {
            id: "missing-asset".into(),
            media_type: MediaType::Video,
            file_name: "video.mp4".into(),
            project_relative_path: "assets/video.mp4".into(),
            duration_ms: Some(1_000),
            has_audio: true,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        });
        let mut evaluated = evaluate_project(
            &project,
            project.settings.width,
            project.settings.height,
            project.settings.fps,
        )
        .unwrap();
        evaluated.resource_bindings.media.clear();
        let error = prepare_media_resources(artifact_io.as_ref(), &evaluated, root.path())
            .err()
            .expect("inconsistent bindings must fail");

        assert_eq!(error.code, ErrorCode::InternalError);
        assert!(artifact_io.events.lock().unwrap().is_empty());
        assert!(process.executions.lock().unwrap().is_empty());

        project.assets[0].project_relative_path = "../outside.mp4".into();
        let error = renderer
            .render_preview(&project, root.path(), 0)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert!(artifact_io.events.lock().unwrap().is_empty());
        assert!(process.executions.lock().unwrap().is_empty());
    }

    #[test]
    fn canonical_media_escape_precedes_collision_workspace_process_and_publication() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("assets")).unwrap();
        let media_path = root.path().join("assets/video.mp4");
        std::fs::write(&media_path, b"fixture").unwrap();
        let output = root.path().join("existing.mp4");
        std::fs::write(&output, b"existing").unwrap();
        let mut project = empty_project();
        project.assets.push(crate::Asset {
            id: "video".into(),
            media_type: MediaType::Video,
            file_name: "video.mp4".into(),
            project_relative_path: "assets/video.mp4".into(),
            duration_ms: Some(1_000),
            has_audio: true,
            origin: None,
            content_hash: None,
            size_bytes: None,
            probe: None,
        });
        project.tracks.push(Track {
            id: "video-track".into(),
            name: "Video".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: crate::AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::Media(MediaItem {
                id: "video-item".into(),
                asset_id: "video".into(),
                start_ms: 0,
                duration_ms: 1_000,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: crate::AudioSettings::default(),
                keyframes: vec![],
            })],
        });
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(LifecycleArtifactIo::default());
        artifact_io.map_canonical_path(
            media_path,
            root.path()
                .parent()
                .unwrap()
                .join("resolved-outside-project.mp4"),
        );
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(process.clone(), artifact_io.clone());

        let error = renderer
            .render_preview(&project, root.path(), 500)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert_no_render_side_effects(&artifact_io, &process);

        artifact_io.clear_events();
        let error = renderer
            .render_preview_range(
                &project,
                root.path(),
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1_000,
                    width: 320,
                    height: 180,
                    fps: 15,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert_no_render_side_effects(&artifact_io, &process);

        artifact_io.clear_events();
        let error = renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert_no_render_side_effects(&artifact_io, &process);
        assert_eq!(std::fs::read(output).unwrap(), b"existing");
    }

    #[test]
    fn malformed_stacking_is_rejected_without_side_effects_for_all_facades() {
        let root = tempdir().unwrap();
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let io = Arc::new(LifecycleArtifactIo::default());
        let renderer =
            Renderer::new("unused", "unused", None).with_adapters(process.clone(), io.clone());
        let mut valid = visual_project();
        let mut second = valid.tracks[0].items[0].clone();
        if let TimelineItem::SolidColor(item) = &mut second {
            item.id = "second".into();
        }
        second.visual_properties_mut().stack_order = 1;
        valid.tracks[0].items.push(second);
        let mut empty = valid.tracks[0].clone();
        empty.id = "empty".into();
        empty.items.clear();
        valid.tracks.push(empty);
        assert!(evaluate_project(&valid, 320, 180, 15).is_ok());
        assert!(evaluate_project(&empty_project(), 320, 180, 15).is_ok());
        for (orders, hidden_item, hidden_track, kind) in [
            ([7, 1], false, false, "visual"),
            ([0, 0], false, false, "visual"),
            ([1, 0], false, false, "visual"),
            ([0, 7], true, false, "visual"),
            ([0, 7], false, true, "visual"),
            ([0, 7], false, false, "audio"),
            ([0, 7], false, false, "transition"),
        ] {
            let mut project = valid.clone();
            if kind == "audio" {
                project.assets.push(crate::Asset {
                    id: "audio".into(),
                    media_type: MediaType::Audio,
                    file_name: "audio.wav".into(),
                    project_relative_path: "assets/audio.wav".into(),
                    duration_ms: Some(1000),
                    has_audio: true,
                    origin: None,
                    content_hash: None,
                    size_bytes: None,
                    probe: None,
                });
                project.tracks[0].items[1] = TimelineItem::Media(MediaItem {
                    id: "audio-item".into(),
                    asset_id: "audio".into(),
                    start_ms: 0,
                    duration_ms: 1000,
                    source_in_ms: 0,
                    visual_properties: crate::VisualProperties::default(),
                    audio: crate::AudioSettings::default(),
                    keyframes: vec![],
                });
            } else if kind == "transition" {
                project.tracks[0].items[1] = TimelineItem::Transition(crate::TransitionItem {
                    id: "transition".into(),
                    transition_type: crate::TransitionType::Fade,
                    from_item_id: "background".into(),
                    to_item_id: None,
                    start_ms: 0,
                    duration_ms: 100,
                    visual_properties: crate::VisualProperties::default(),
                });
            }
            for (item, order) in project.tracks[0].items.iter_mut().zip(orders) {
                item.visual_properties_mut().stack_order = order;
            }
            project.tracks[0].items[1].visual_properties_mut().hidden = hidden_item;
            project.tracks[0].hidden = hidden_track;
            // Exercise a current-schema document decoded outside the store facade.
            let before = serde_json::to_value(&project).unwrap();
            let project: Project = serde_json::from_value(before.clone()).unwrap();
            let error = evaluate_project(&project, 320, 180, 15).unwrap_err();
            assert_eq!(error.code, ErrorCode::ValidationFailed);
            assert_eq!(error.message, "stackOrder must match item array position");
            assert_all_facades_reject_without_side_effects(
                &renderer,
                &io,
                &process,
                &project,
                root.path(),
                ErrorCode::ValidationFailed,
            );
            assert!(io.events.lock().unwrap().is_empty());
            assert_eq!(serde_json::to_value(&project).unwrap(), before);
            assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }

    #[test]
    fn invalid_missing_non_finite_and_complex_scenes_are_side_effect_free_for_all_facades() {
        let root = tempdir().unwrap();
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(LifecycleArtifactIo::default());
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(process.clone(), artifact_io.clone());

        let mut missing = empty_project();
        missing.tracks.push(Track {
            id: "missing-track".into(),
            name: "Missing".into(),
            track_type: TrackType::Video,
            locked: false,
            hidden: false,
            muted: false,
            audio_role: crate::AudioTrackRole::Unassigned,
            ducking: None,
            items: vec![TimelineItem::Media(MediaItem {
                id: "missing-item".into(),
                asset_id: "missing-asset".into(),
                start_ms: 0,
                duration_ms: 1_000,
                source_in_ms: 0,
                visual_properties: crate::VisualProperties::default(),
                audio: crate::AudioSettings::default(),
                keyframes: vec![],
            })],
        });
        assert_all_facades_reject_without_side_effects(
            &renderer,
            &artifact_io,
            &process,
            &missing,
            root.path(),
            ErrorCode::AssetNotFound,
        );

        let mut invalid_timing = visual_project();
        let TimelineItem::SolidColor(item) = &mut invalid_timing.tracks[0].items[0] else {
            unreachable!()
        };
        item.duration_ms = 0;
        invalid_timing.tracks[0]
            .items
            .push(TimelineItem::SolidColor(SolidColorItem {
                id: "valid-duration-anchor".into(),
                color: "#000000".into(),
                start_ms: 0,
                duration_ms: 1,
                visual_properties: crate::VisualProperties::default(),
                keyframes: vec![],
            }));
        invalid_timing.tracks[0].items[1]
            .visual_properties_mut()
            .stack_order = 1;
        assert_all_facades_reject_without_side_effects(
            &renderer,
            &artifact_io,
            &process,
            &invalid_timing,
            root.path(),
            ErrorCode::InvalidArgument,
        );

        let mut non_finite = visual_project();
        let TimelineItem::SolidColor(item) = &mut non_finite.tracks[0].items[0] else {
            unreachable!()
        };
        item.transform.opacity = f64::NAN;
        assert_all_facades_reject_without_side_effects(
            &renderer,
            &artifact_io,
            &process,
            &non_finite,
            root.path(),
            ErrorCode::InvalidArgument,
        );

        let mut too_complex = visual_project();
        too_complex.tracks[0].items = (0..=crate::evaluated_scene::MAX_EVALUATED_VISUAL_LAYERS)
            .map(|index| {
                TimelineItem::SolidColor(SolidColorItem {
                    id: format!("layer-{index}"),
                    color: "#112233".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: vec![],
                })
            })
            .collect();
        assert_all_facades_reject_without_side_effects(
            &renderer,
            &artifact_io,
            &process,
            &too_complex,
            root.path(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn facade_delegates_readiness_probe_execution_and_cleanup_to_adapters() {
        let failing_process = Arc::new(FakeProcess {
            readiness_error: true,
            probe_error: true,
            run_failure: Some(FakeRunFailure::Spawn),
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(FakeArtifactIo::default());
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(failing_process.clone(), artifact_io.clone());
        assert_eq!(
            renderer.readiness().unwrap_err().code,
            ErrorCode::DependencyUnavailable
        );
        assert_eq!(
            renderer.probe(Path::new("media.mp4")).unwrap_err().code,
            ErrorCode::UnsupportedMedia
        );

        let root = tempdir().unwrap();
        let output = root.path().join("output.mp4");
        let error = renderer
            .export_video(
                &empty_project(),
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some(SPAWN_STAGE));
        assert_eq!(failing_process.executions.lock().unwrap().len(), 1);
        assert_eq!(artifact_io.removed.lock().unwrap().len(), 1);

        let exit_process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: Some(FakeRunFailure::Exit),
            executions: Mutex::new(vec![]),
        });
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(exit_process, artifact_io.clone());
        let error = renderer
            .export_video(
                &empty_project(),
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some(RENDER_STAGE));
        assert_eq!(error.ffmpeg_exit_code, Some(7));
        assert_eq!(
            error.ffmpeg_stderr_excerpt.as_deref(),
            Some("injected diagnostic")
        );
    }

    #[test]
    fn facade_delegates_publication_and_metadata_failures_to_artifact_adapter() {
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(FakeArtifactIo {
            fail_rename: false,
            fail_size: true,
            removed: Mutex::new(vec![]),
        });
        let renderer =
            Renderer::new("ffmpeg", "ffprobe", None).with_adapters(process, artifact_io.clone());
        let root = tempdir().unwrap();
        let output = root.path().join("output.mp4");
        let error = renderer
            .export_video(
                &empty_project(),
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some(PUBLISH_STAGE));
        assert!(!output.exists());
        assert!(artifact_io.removed.lock().unwrap().contains(&output));

        let publication_io = Arc::new(FakeArtifactIo {
            fail_rename: true,
            fail_size: false,
            removed: Mutex::new(vec![]),
        });
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let renderer =
            Renderer::new("ffmpeg", "ffprobe", None).with_adapters(process, publication_io.clone());
        let publication_output = root.path().join("publication.mp4");
        let error = renderer
            .export_video(
                &empty_project(),
                root.path(),
                ExportOptions {
                    output: &publication_output,
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some(PUBLISH_STAGE));
        assert_eq!(publication_io.removed.lock().unwrap().len(), 1);
    }

    #[test]
    fn facade_delegates_workspace_filter_paths_and_cleanup_for_every_render_intent() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let artifact_io = Arc::new(LifecycleArtifactIo::default());
        let renderer = Renderer::new("ffmpeg", "ffprobe", None)
            .with_adapters(process.clone(), artifact_io.clone());
        let project = visual_project();

        renderer.render_preview(&project, root.path(), 0).unwrap();
        renderer
            .render_preview_range(
                &project,
                root.path(),
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 500,
                    width: 320,
                    height: 180,
                    fps: 15,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &root.path().join("export.mp4"),
                    width: 320,
                    height: 180,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();

        assert_eq!(
            process.executions.lock().unwrap().as_slice(),
            &[
                RenderIntent::Frame { at_ms: 0 },
                RenderIntent::Range {
                    start_ms: 0,
                    end_ms: 500,
                    include_audio: false,
                },
                RenderIntent::Export,
            ]
        );
        let events = artifact_io.events.lock().unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|event| **event == "create_dir")
                .count(),
            3
        );
        assert_eq!(events.iter().filter(|event| **event == "write").count(), 3);
        assert_eq!(
            events
                .iter()
                .filter(|event| **event == "remove_dir_all")
                .count(),
            3
        );
        assert!(events.contains(&"rename"));
        assert!(events.contains(&"size"));
    }

    #[test]
    fn facade_maps_injected_workspace_and_filter_failures_and_still_cleans_up() {
        for failure in [ArtifactFailure::CreateWorkspace, ArtifactFailure::Write] {
            let root = tempdir().unwrap();
            let process = Arc::new(FakeProcess {
                readiness_error: false,
                probe_error: false,
                run_failure: None,
                executions: Mutex::new(vec![]),
            });
            let artifact_io = Arc::new(LifecycleArtifactIo::default());
            artifact_io.fail_next(failure);
            let renderer = Renderer::new("ffmpeg", "ffprobe", None)
                .with_adapters(process, artifact_io.clone());
            let error = renderer
                .export_video(
                    &visual_project(),
                    root.path(),
                    ExportOptions {
                        output: &root.path().join("export.mp4"),
                        width: 320,
                        height: 180,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap_err();
            assert_eq!(error.failed_stage.as_deref(), Some(GRAPH_BUILD_STAGE));
            let events = artifact_io.events.lock().unwrap();
            if failure == ArtifactFailure::Write {
                assert!(events.contains(&"remove_dir_all"));
            }
        }
    }

    #[test]
    fn facade_routes_font_path_listing_and_reads_through_artifact_io() {
        for failure in [
            ArtifactFailure::Canonicalize,
            ArtifactFailure::Read,
            ArtifactFailure::List,
        ] {
            let root = tempdir().unwrap();
            let fonts = root.path().join("fonts");
            std::fs::create_dir(&fonts).unwrap();
            std::fs::write(fonts.join("fixture.ttf"), b"not a real font").unwrap();
            let mut project = visual_project();
            project.tracks[0]
                .items
                .push(TimelineItem::Text(crate::TextItem {
                    id: "text".into(),
                    text: "artifact adapter".into(),
                    start_ms: 0,
                    duration_ms: 1_000,
                    font_size: 20,
                    color: "#ffffff".into(),
                    font_family: (failure == ArtifactFailure::List).then(|| "Missing".into()),
                    font_path: (failure != ArtifactFailure::List).then(|| "fixture.ttf".into()),
                    style: crate::TextStyle {
                        wrap_width_px: Some(120),
                        ..crate::TextStyle::default()
                    },
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: vec![],
                }));
            project.tracks[0].items[1]
                .visual_properties_mut()
                .stack_order = 1;
            let process = Arc::new(FakeProcess {
                readiness_error: false,
                probe_error: false,
                run_failure: None,
                executions: Mutex::new(vec![]),
            });
            let artifact_io = Arc::new(LifecycleArtifactIo::default());
            artifact_io.fail_next(failure);
            let renderer = Renderer::new("ffmpeg", "ffprobe", None)
                .with_font_roots([fonts])
                .with_adapters(process, artifact_io.clone());
            let artifact = renderer
                .export_video(
                    &project,
                    root.path(),
                    ExportOptions {
                        output: &root.path().join("export.mp4"),
                        width: 320,
                        height: 180,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            let events = artifact_io.events.lock().unwrap();
            match failure {
                ArtifactFailure::Canonicalize => {
                    assert!(events.contains(&"canonicalize"));
                    assert!(
                        artifact
                            .warnings
                            .iter()
                            .any(|warning| warning.contains("font path"))
                    );
                }
                ArtifactFailure::Read => assert!(events.contains(&"read")),
                ArtifactFailure::List => {
                    assert!(events.contains(&"list"));
                    assert!(
                        artifact
                            .warnings
                            .iter()
                            .any(|warning| warning.contains("font family"))
                    );
                }
                ArtifactFailure::CreateWorkspace | ArtifactFailure::Write => unreachable!(),
            }
        }
    }

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
        let command = build_render_command(
            Path::new("ffmpeg"),
            &RenderPlan {
                filter_graph: String::new(),
                width: 320,
                height: 180,
                fps: 15,
                duration_ms: 1_100,
                intent: RenderIntent::Range {
                    start_ms: 100,
                    end_ms: 1_100,
                    include_audio: false,
                },
                media_inputs: vec![],
                media_paths: vec![],
            },
            Path::new("filter.txt"),
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
                visual_properties: crate::VisualProperties::default(),
                audio: crate::AudioSettings {
                    volume,
                    ..crate::AudioSettings::default()
                },
                keyframes,
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
                    visual_properties: crate::VisualProperties::new(
                        Transform {
                            position_x: 100.0,
                            position_y: 200.0,
                            scale: 2.0,
                            opacity: 0.75,
                        },
                        false,
                    ),
                    keyframes: vec![],
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        let mut warnings = Vec::new();
        let text_resources = project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .filter_map(|item| match item {
                TimelineItem::Text(text) if !text.hidden => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>();
        let text_layers = prepare_text_layers(
            &FileSystemArtifactIo,
            &text_resources,
            root.path(),
            renderer.default_font_path.as_deref(),
            &renderer.font_roots,
            &mut warnings,
        )
        .unwrap();
        let filter = renderer
            .build_filter(
                &project,
                FilterContext {
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
        let graph_error =
            RenderWorkspace::create(Arc::new(FileSystemArtifactIo), &invalid_project_dir)
                .err()
                .unwrap();
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
                    visual_properties: crate::VisualProperties::default(),
                    keyframes: vec![],
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        let evaluated = evaluate_project(&project, 320, 180, 15).unwrap();
        let media = prepare_media_resources(renderer.artifact_io.as_ref(), &evaluated, root.path())
            .unwrap();
        assert!(
            renderer
                .prepare_render(&evaluated, media, root.path(), RenderIntent::Export)
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
            let workspace =
                RenderWorkspace::create(Arc::new(FileSystemArtifactIo), root.path()).unwrap();
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
        let ffmpeg_version = Command::new(&ffmpeg)
            .arg("-version")
            .output()
            .expect("configured OPENCUT_FFMPEG_PATH must be executable");
        assert!(
            ffmpeg_version.status.success(),
            "configured OPENCUT_FFMPEG_PATH must run successfully"
        );
        let ffprobe_version = Command::new(&ffprobe)
            .arg("-version")
            .output()
            .expect("configured OPENCUT_FFPROBE_PATH must be executable");
        assert!(
            ffprobe_version.status.success(),
            "configured OPENCUT_FFPROBE_PATH must run successfully"
        );
        let font_path = env::var_os("OPENCUT_TEST_FONT_PATH")
            .map(PathBuf::from)
            .expect("configured native parity requires OPENCUT_TEST_FONT_PATH");
        assert!(font_path.is_file(), "configured parity font must exist");

        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        std::fs::create_dir(root.path().join("assets")).unwrap();
        let tone_path = root.path().join("assets/tone.wav");
        let tone = Command::new(&ffmpeg)
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=1",
                "-y",
            ])
            .arg(&tone_path)
            .output()
            .unwrap();
        assert!(tone.status.success());
        let mut project = Project {
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
            assets: vec![crate::Asset {
                id: "tone".into(),
                media_type: MediaType::Audio,
                file_name: "tone.wav".into(),
                project_relative_path: "assets/tone.wav".into(),
                duration_ms: Some(1_000),
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
                    audio_role: crate::AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![
                        TimelineItem::SolidColor(SolidColorItem {
                            id: "background".into(),
                            color: "#cc3311".into(),
                            start_ms: 0,
                            duration_ms: 1_000,
                            visual_properties: crate::VisualProperties::new(
                                Transform {
                                    opacity: 0.7,
                                    ..Transform::default()
                                },
                                false,
                            ),
                            keyframes: vec![],
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
                    audio_role: crate::AudioTrackRole::Unassigned,
                    ducking: None,
                    items: vec![TimelineItem::Media(MediaItem {
                        id: "tone-item".into(),
                        asset_id: "tone".into(),
                        start_ms: 0,
                        duration_ms: 1_000,
                        source_in_ms: 0,
                        visual_properties: crate::VisualProperties::default(),
                        audio: crate::AudioSettings::default(),
                        keyframes: vec![],
                    })],
                },
            ],
        };
        for track in &mut project.tracks {
            for (index, item) in track.items.iter_mut().enumerate() {
                item.visual_properties_mut().stack_order = u32::try_from(index).unwrap();
            }
        }
        let renderer = Renderer::new(&ffmpeg, ffprobe, Some(font_path));
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
                    include_audio: true,
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
        let range_path = root.path().join(&range.relative_path);
        let range_frame = decode_rgb_frame(&ffmpeg, &range_path, 500);
        let export_frame = decode_rgb_frame(&ffmpeg, &export_path, 500);
        assert_frames_close(&preview_frame, &range_frame, 8.0);
        assert_frames_close(&preview_frame, &export_frame, 8.0);
        assert!(structural_similarity(&preview_frame, &range_frame) >= 0.99);
        assert!(structural_similarity(&preview_frame, &export_frame) >= 0.99);
        assert!(structural_similarity(&range_frame, &export_frame) >= 0.99);
        let range_audio = decode_mono_f32(&ffmpeg, &range_path);
        let export_audio = decode_mono_f32(&ffmpeg, &export_path);
        assert_eq!(range_audio.len(), export_audio.len());
        let rms_error = range_audio
            .iter()
            .zip(&export_audio)
            .map(|(left, right)| f64::from(left - right).powi(2))
            .sum::<f64>()
            / range_audio.len().max(1) as f64;
        let rms_error = rms_error.sqrt();
        assert!(rms_error <= 0.0001, "audio RMS error was {rms_error}");
        let range_duration = renderer.probe(&range_path).unwrap().duration_ms.unwrap();
        let export_duration = renderer.probe(&export_path).unwrap().duration_ms.unwrap();
        assert!(range_duration.abs_diff(export_duration) <= 100);
        assert!(range_duration.abs_diff(1_000) <= 100);
        assert!(export_duration.abs_diff(1_000) <= 100);
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
            visual_properties: crate::VisualProperties::default(),
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
        right.visual_properties.stack_order = 1;
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

    #[test]
    fn native_stacking_occlusion_and_render_intents_agree() {
        let (Ok(ffmpeg), Ok(ffprobe)) = (
            env::var("OPENCUT_FFMPEG_PATH"),
            env::var("OPENCUT_FFPROBE_PATH"),
        ) else {
            return;
        };
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        std::fs::create_dir(root.path().join("assets")).unwrap();
        let tone = Command::new(&ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=1",
                "-y",
            ])
            .arg(root.path().join("assets/tone.wav"))
            .output()
            .unwrap();
        assert!(tone.status.success());
        let font = env::var_os("OPENCUT_TEST_FONT_PATH")
            .map(PathBuf::from)
            .expect("native stacking requires configured font");
        let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
        let mut project = visual_project();
        project.settings = ProjectSettings {
            width: 160,
            height: 90,
            fps: 10,
        };
        project.tracks[0].items=serde_json::from_value(serde_json::json!([
            {"type":"solid_color","id":"red","color":"#ff0000","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":2,"stackOrder":0},
            {"type":"solid_color","id":"green","color":"#00ff00","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":1,"stackOrder":1},
            {"type":"rectangle","id":"blue","color":"#0000ff","width":80,"height":90,"startMs":0,"durationMs":1000,"keyframes":[],"zIndex":10,"stackOrder":2,"transform2d":{"position":{"x":80,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.5}},
            {"type":"solid_color","id":"hidden","color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[],"zIndex":99,"stackOrder":3,"hidden":true},
            {"type":"transition","id":"fade","transitionType":"fade","fromItemId":"red","toItemId":null,"startMs":900,"durationMs":100,"zIndex":0,"stackOrder":4}
        ])).unwrap();
        project.assets=serde_json::from_value(serde_json::json!([{"id":"tone","mediaType":"audio","fileName":"tone.wav","projectRelativePath":"assets/tone.wav","durationMs":1000,"hasAudio":true}])).unwrap();
        let mut audio = project.tracks[0].clone();
        audio.id = "audio".into();
        audio.track_type = TrackType::Audio;
        audio.items=serde_json::from_value(serde_json::json!([{"type":"media","id":"sound","assetId":"tone","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[],"zIndex":0,"stackOrder":0}])).unwrap();
        project.tracks.push(audio);
        let mut upper = project.tracks[0].clone();
        upper.id = "upper".into();
        upper.items=serde_json::from_value(serde_json::json!([{"type":"rectangle","id":"corner","color":"#ff00ff","width":20,"height":20,"startMs":0,"durationMs":1000,"keyframes":[],"zIndex":-2147483648,"stackOrder":0}])).unwrap();
        project.tracks.push(upper);
        let mut captions = project.tracks[0].clone();
        captions.id = "captions".into();
        captions.track_type = TrackType::Caption;
        captions.items=serde_json::from_value(serde_json::json!([{"type":"caption","id":"caption","text":"Stack","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0,"style":{"fontSize":10,"color":"#ffffff","backgroundColor":"#000000","bottomMarginPx":0},"source":{"assetId":"tone","providerId":"test","modelId":"test","modelVersion":null,"language":"en","generatedAtMs":1,"originalText":"Stack","confidence":null,"words":[]}}])).unwrap();
        project.tracks.push(captions);
        for mode in 0..3 {
            if mode == 1 {
                project.tracks[0].items[0].visual_properties_mut().z_index = 0;
                project.tracks[0].items[1].visual_properties_mut().z_index = 0;
            }
            if mode == 2 {
                project.tracks[0].items.swap(0, 1);
                for (index, item) in project.tracks[0].items.iter_mut().enumerate() {
                    item.visual_properties_mut().stack_order = index as u32;
                }
            }
            if mode == 2 {
                project.tracks.swap(0, 2);
            }
            let before = serde_json::to_value(&project).unwrap();
            let preview = renderer.render_preview(&project, root.path(), 500).unwrap();
            // Draft previews render the materialized candidate through this same facade.
            let materialized: Project = serde_json::from_value(before.clone()).unwrap();
            let draft = renderer
                .render_preview(&materialized, root.path(), 500)
                .unwrap();
            let range = renderer
                .render_preview_range(
                    &project,
                    root.path(),
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 1000,
                        width: 160,
                        height: 90,
                        fps: 10,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap();
            let export = root.path().join(format!("stacking-{mode}.mp4"));
            renderer
                .export_video(
                    &project,
                    root.path(),
                    ExportOptions {
                        output: &export,
                        width: 160,
                        height: 90,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            let frame = decode_rgb_frame(&ffmpeg, &root.path().join(preview.relative_path), 0);
            let corner = &frame[(10 * 160 + 10) * 3..(10 * 160 + 10) * 3 + 3];
            assert!(
                if mode == 2 {
                    corner[0] > 240 && corner[2] < 15
                } else {
                    corner[0] > 240 && corner[2] > 240
                },
                "track order: {corner:?}"
            );
            let left = &frame[(45 * 160 + 20) * 3..(45 * 160 + 20) * 3 + 3];
            let dominant = if mode == 1 { 1 } else { 0 };
            assert!(
                left[dominant] > 240 && left[2] < 15,
                "incorrect occlusion: {left:?}"
            );
            let right = &frame[(45 * 160 + 120) * 3..(45 * 160 + 120) * 3 + 3];
            assert!(
                right[2] > 90 && right[dominant] > 90,
                "transparent overlay missing: {right:?}"
            );
            for (path, time) in [
                (root.path().join(draft.relative_path), 0),
                (root.path().join(&range.relative_path), 500),
                (export.clone(), 500),
            ] {
                let compared = decode_rgb_frame(&ffmpeg, &path, time);
                assert!(structural_similarity(&frame, &compared) >= 0.99);
            }
            let range_path = root.path().join(range.relative_path);
            let a = decode_mono_f32(&ffmpeg, &range_path);
            let b = decode_mono_f32(&ffmpeg, &export);
            assert!(!a.is_empty());
            assert_eq!(a.len(), b.len());
            let rms = (a
                .iter()
                .zip(&b)
                .map(|(a, b)| f64::from(a - b).powi(2))
                .sum::<f64>()
                / a.len() as f64)
                .sqrt();
            assert!(rms <= 0.0001);
            assert!(
                renderer
                    .probe(&range_path)
                    .unwrap()
                    .duration_ms
                    .unwrap()
                    .abs_diff(renderer.probe(&export).unwrap().duration_ms.unwrap())
                    <= 100
            );
            assert_eq!(serde_json::to_value(&project).unwrap(), before);
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

    fn decode_mono_f32(ffmpeg: &str, path: &Path) -> Vec<f32> {
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
            .unwrap();
        assert!(output.status.success());
        output
            .stdout
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect()
    }

    fn structural_similarity(left: &[u8], right: &[u8]) -> f64 {
        assert_eq!(left.len(), right.len());
        let count = left.len().max(1) as f64;
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
        ((2.0 * left_mean * right_mean + c1) * (2.0 * covariance / count + c2))
            / ((left_mean.powi(2) + right_mean.powi(2) + c1)
                * ((left_variance + right_variance) / count + c2))
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
                    visual_properties: crate::VisualProperties::default(),
                })],
            }],
        };
        let renderer = Renderer::new("ffmpeg", "ffprobe", None);
        let text_layers = HashMap::new();
        let mut warnings = Vec::new();
        let filter = renderer
            .build_filter(
                &project,
                FilterContext {
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

    #[test]
    fn measured_transform_overflow_and_unavailable_backend_publish_nothing() {
        let root = tempdir().unwrap();
        for unavailable in [false, true] {
            let mut project = visual_project();
            if unavailable {
                project.tracks[0].items[0]
                    .visual_properties_mut()
                    .transform2d = Some(crate::Transform2D::default());
            } else {
                project.tracks[0].items = vec![TimelineItem::Text(crate::TextItem {
                    id: "text".into(),
                    text: "W".repeat(100),
                    font_size: 1000,
                    color: "#ffffff".into(),
                    start_ms: 0,
                    duration_ms: 1000,
                    font_family: None,
                    font_path: None,
                    style: Default::default(),
                    visual_properties: crate::VisualProperties {
                        transform2d: Some(crate::Transform2D::default()),
                        ..Default::default()
                    },
                    keyframes: vec![],
                })];
            }
            let process = Arc::new(FakeProcess {
                readiness_error: unavailable,
                probe_error: false,
                run_failure: None,
                executions: Mutex::new(vec![]),
            });
            let io = Arc::new(LifecycleArtifactIo::default());
            let renderer =
                Renderer::new("ffmpeg", "ffprobe", None).with_adapters(process.clone(), io.clone());
            let expected = if unavailable {
                ErrorCode::DependencyUnavailable
            } else {
                ErrorCode::InvalidArgument
            };
            assert_eq!(
                renderer
                    .render_preview(&project, root.path(), 0)
                    .unwrap_err()
                    .code,
                expected
            );
            let events = io.events.lock().unwrap();
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(*event, "create_dir" | "write" | "rename"))
            );
            assert!(process.executions.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn invalid_group_graph_and_composed_geometry_have_no_render_side_effects() {
        let root = tempdir().unwrap();
        let process = Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let io = Arc::new(LifecycleArtifactIo::default());
        let renderer =
            Renderer::new("unused", "unused", None).with_adapters(process.clone(), io.clone());
        for missing in [false, true] {
            let mut project = visual_project();
            project.tracks[0].track_type = TrackType::Overlay;
            project.tracks[0].items[0].visual_properties_mut().parent =
                Some(crate::ParentReference {
                    scope: "root".into(),
                    id: if missing { "absent" } else { "group" }.into(),
                });
            let transform = crate::Transform2D {
                scale_x: 100.0,
                scale_y: 100.0,
                ..Default::default()
            };
            project.tracks[0].items.push(serde_json::from_value(serde_json::json!({"type":"group","id":"group","startMs":0,"durationMs":1000,"stackOrder":1,"transform2d":transform})).unwrap());
            assert_all_facades_reject_without_side_effects(
                &renderer,
                &io,
                &process,
                &project,
                root.path(),
                if missing {
                    ErrorCode::ItemNotFound
                } else {
                    ErrorCode::InvalidArgument
                },
            );
        }
        let mut project = visual_project();
        project.tracks[0].track_type = TrackType::Overlay;
        project.tracks[0].items[0].visual_properties_mut().parent = Some(crate::ParentReference {
            scope: "root".into(),
            id: "group".into(),
        });
        project.tracks[0].items.push(serde_json::from_value(serde_json::json!({"type":"group","id":"group","startMs":0,"durationMs":1000,"stackOrder":1})).unwrap());
        for endpoint in ["fromItemId", "toItemId"] {
            let mut invalid = project.clone();
            let mut transition = serde_json::json!({"type":"transition","id":"invalid","startMs":0,"durationMs":100,"transitionType":"fade","fromItemId":"missing","toItemId":null,"hidden":true,"stackOrder":2});
            transition[endpoint] = serde_json::json!("group");
            invalid.tracks[0]
                .items
                .push(serde_json::from_value(transition).unwrap());
            assert_all_facades_reject_without_side_effects(
                &renderer,
                &io,
                &process,
                &invalid,
                root.path(),
                ErrorCode::InvalidArgument,
            );
        }
        let unavailable = Arc::new(FakeProcess {
            readiness_error: true,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        });
        let renderer =
            Renderer::new("unused", "unused", None).with_adapters(unavailable.clone(), io.clone());
        assert_all_facades_reject_without_side_effects(
            &renderer,
            &io,
            &unavailable,
            &project,
            root.path(),
            ErrorCode::DependencyUnavailable,
        );
    }

    #[test]
    fn transformed_caption_measurement_has_explicit_box_and_is_read_only() {
        let project = visual_project();
        let mut evaluated = evaluate_project(&project, 320, 180, 15).unwrap();
        let layer = &mut evaluated.scene.visual_layers[0];
        layer.transform2d = Some(crate::Transform2D::default());
        layer.source = crate::evaluated_scene::EvaluatedVisualSource::Caption(
            crate::evaluated_scene::EvaluatedCaption {
                text: "abcd".into(),
                font_size: 20,
                color: "#ffffff".into(),
                background_color: "#000000".into(),
                bottom_margin_px: 64,
            },
        );
        let io = LifecycleArtifactIo::default();
        let measured =
            measure_evaluated_text_layers(&io, &evaluated, None, &[], &mut vec![]).unwrap();
        let text = &measured["background"];
        assert_eq!(
            (text.prepared.layer_width, text.prepared.layer_height),
            (72, 48)
        );
        assert_eq!((text.prepared.text_x, text.prepared.text_y), (12, 12));
        assert!(io.events.lock().unwrap().is_empty());
    }

    #[test]
    fn selected_font_metrics_determine_affine_anchor_before_writes() {
        let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts");
        let fonts = (
            fixture_root.join("DejaVuSans.ttf"),
            fixture_root.join("DejaVuSerif.ttf"),
        );
        let mut project = visual_project();
        project.tracks[0].items = vec![TimelineItem::Text(crate::TextItem {
            id: "font-test".into(),
            text: "WWiii".into(),
            font_size: 40,
            color: "#ffffff".into(),
            start_ms: 0,
            duration_ms: 1000,
            font_family: None,
            font_path: None,
            style: Default::default(),
            visual_properties: crate::VisualProperties {
                transform2d: Some(crate::Transform2D {
                    anchor: crate::TransformAnchor { x: 0.5, y: 0.5 },
                    ..Default::default()
                }),
                ..Default::default()
            },
            keyframes: vec![],
        })];
        let io = LifecycleArtifactIo::default();
        let mut anchors = vec![];
        for font in [fonts.0, fonts.1] {
            assert!(
                font.is_file(),
                "native font fixture unavailable: {}",
                font.display()
            );
            let mut evaluated = evaluate_project(&project, 320, 180, 15).unwrap();
            let measurements =
                measure_evaluated_text_layers(&io, &evaluated, Some(&font), &[], &mut vec![])
                    .unwrap();
            let text = &measurements["font-test"];
            assert_eq!(text.prepared.font_path.as_ref(), Some(&font));
            finalize_affine_geometry(
                &mut evaluated.scene,
                &HashMap::from([(
                    "font-test".into(),
                    (text.prepared.layer_width, text.prepared.layer_height),
                )]),
            )
            .unwrap();
            anchors.push(evaluated.scene.visual_layers[0].affine.unwrap().matrix[4]);
        }
        assert_ne!(anchors[0], anchors[1]);
        assert!(
            !io.events
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(*event, "create_dir" | "write" | "rename"))
        );
    }
    #[derive(Debug, Default)]
    struct GeometryProcess {
        calls: Mutex<Vec<(PathBuf, crate::MediaType)>>,
        fail: bool,
    }
    impl ProcessExecutor for GeometryProcess {
        fn readiness(&self, _: &Path, _: &Path) -> Result<(), CoreError> {
            Ok(())
        }
        fn probe(&self, _: &Path, _: &Path) -> Result<ProbeResult, CoreError> {
            panic!("public probe is not the geometry port")
        }
        fn probe_render_geometry(
            &self,
            _: &Path,
            path: &Path,
            kind: crate::MediaType,
        ) -> Result<(u32, u32), CoreError> {
            self.calls.lock().unwrap().push((path.to_owned(), kind));
            if self.fail {
                Err(CoreError::new(
                    ErrorCode::UnsupportedMedia,
                    "injected geometry failure",
                ))
            } else {
                Ok((20, 40))
            }
        }
        fn execute(
            &self,
            _: &Path,
            _: &RenderPlan,
            _: &Path,
            _: &Path,
            _: &mut dyn FnMut(RenderProgress),
        ) -> Result<(), CoreError> {
            panic!("preflight must not render")
        }
    }

    #[test]
    fn oriented_geometry_is_probed_once_and_reused_without_writes() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("assets")).unwrap();
        std::fs::write(root.path().join("assets/source.mp4"), b"metadata fixture").unwrap();
        let mut project = visual_project();
        project.assets = vec![serde_json::from_value(serde_json::json!({"id":"asset","mediaType":"video","fileName":"source.mp4","projectRelativePath":"assets/source.mp4","durationMs":1000})).unwrap()];
        project.tracks[0].items = ["first", "second"].iter().map(|id| serde_json::from_value(serde_json::json!({"type":"media","id":id,"assetId":"asset","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[],"transform2d":crate::Transform2D::default()})).unwrap()).collect();
        project.tracks[0].items[1]
            .visual_properties_mut()
            .stack_order = 1;
        for kind in [crate::MediaType::Image, crate::MediaType::Video] {
            project.assets[0].media_type = kind;
            let before = serde_json::to_value(&project).unwrap();
            for fail in [false, true] {
                let process = Arc::new(GeometryProcess {
                    fail,
                    ..Default::default()
                });
                let io = Arc::new(LifecycleArtifactIo::default());
                let renderer = Renderer::new("unused", "configured-probe", None)
                    .with_adapters(process.clone(), io.clone());
                let evaluated = evaluate_project(&project, 320, 180, 15).unwrap();
                assert!(
                    evaluated
                        .scene
                        .visual_layers
                        .iter()
                        .all(|v| v.affine.is_none())
                );
                let media = prepare_media_resources(io.as_ref(), &evaluated, root.path()).unwrap();
                let result = renderer.preflight_render(&evaluated, media);
                if fail {
                    assert_eq!(result.err().unwrap().code, ErrorCode::UnsupportedMedia);
                } else {
                    let ready = result.unwrap();
                    assert!(
                        ready
                            .scene
                            .visual_layers
                            .iter()
                            .all(|v| v.source_size == Some((20, 40)))
                    );
                    let RenderPreflight {
                        scene,
                        media,
                        measured,
                        warnings,
                    } = ready;
                    // Materialization may write, but must not probe again.
                    renderer
                        .materialize_render(
                            RenderPreflight {
                                scene,
                                media,
                                measured,
                                warnings,
                            },
                            root.path(),
                            RenderIntent::Export,
                        )
                        .unwrap();
                }
                let calls = process.calls.lock().unwrap();
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].1, kind);
                assert_eq!(
                    calls[0].0,
                    root.path()
                        .join("assets/source.mp4")
                        .canonicalize()
                        .unwrap()
                );
                if fail {
                    assert!(
                        !io.events.lock().unwrap().iter().any(|e| matches!(
                            *e,
                            "request_id" | "exists" | "create_dir" | "write"
                        ))
                    );
                }
            }
            assert_eq!(serde_json::to_value(&project).unwrap(), before);
        }
    }

    #[test]
    fn measured_overflow_precedes_collision_and_metadata_probes() {
        let root = tempdir().unwrap();
        let mut project = visual_project();
        project.tracks[0].items = vec![serde_json::from_value(serde_json::json!({"type":"text","id":"text","text":"W".repeat(100),"fontSize":1000,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[],"transform2d":crate::Transform2D::default()})).unwrap()];
        std::fs::create_dir(root.path().join("assets")).unwrap();
        std::fs::write(root.path().join("assets/source.mp4"), b"metadata fixture").unwrap();
        project.assets = vec![serde_json::from_value(serde_json::json!({"id":"asset","mediaType":"image","fileName":"source.mp4","projectRelativePath":"assets/source.mp4","durationMs":1000})).unwrap()];
        project.tracks[0].items.push(serde_json::from_value(serde_json::json!({"type":"media","id":"media","assetId":"asset","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[],"transform2d":crate::Transform2D::default()})).unwrap());
        project.tracks[0].items[1]
            .visual_properties_mut()
            .stack_order = 1;
        let output = root.path().join("exists.mp4");
        let process = Arc::new(GeometryProcess::default());
        let io = Arc::new(LifecycleArtifactIo::default());
        let renderer =
            Renderer::new("unused", "unused", None).with_adapters(process.clone(), io.clone());
        for exists in [false, true] {
            if exists {
                std::fs::write(&output, b"original").unwrap();
            }
            io.clear_events();
            assert_eq!(
                renderer
                    .export_video(
                        &project,
                        root.path(),
                        ExportOptions {
                            output: &output,
                            width: 320,
                            height: 180,
                            overwrite: false
                        },
                        |_| {}
                    )
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidArgument
            );
            assert!(
                !io.events
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|e| matches!(*e, "exists" | "request_id" | "create_dir" | "write"))
            );
        }
        assert_eq!(
            renderer
                .render_preview(&project, root.path(), 0)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            renderer
                .render_preview_range(
                    &project,
                    root.path(),
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 1000,
                        width: 320,
                        height: 180,
                        fps: 15,
                        include_audio: true
                    },
                    |_| {}
                )
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        let mut missing = project.clone();
        missing.assets.clear();
        io.clear_events();
        assert_eq!(
            renderer
                .render_preview(&missing, root.path(), 0)
                .unwrap_err()
                .code,
            ErrorCode::AssetNotFound
        );
        assert!(io.events.lock().unwrap().is_empty());
        assert!(process.calls.lock().unwrap().is_empty());
        assert_eq!(std::fs::read(&output).unwrap(), b"original");
        io.clear_events();
        assert_eq!(
            renderer
                .export_video(
                    &visual_project(),
                    root.path(),
                    ExportOptions {
                        output: &output,
                        width: 320,
                        height: 180,
                        overwrite: false
                    },
                    |_| {}
                )
                .unwrap_err()
                .code,
            ErrorCode::ExportExists
        );
        assert_eq!(*io.events.lock().unwrap(), vec!["exists"]);
    }
}
