//! Private preview cache orchestration and publication conformance.
use super::*;

#[derive(Debug)]
struct CacheProcess {
    executions: std::sync::atomic::AtomicUsize,
    ready: std::sync::atomic::AtomicBool,
}
impl ProcessExecutor for CacheProcess {
    fn preview_cache_identity(&self, _: &Path, _: &Path) -> Option<[u8; 32]> {
        Some([17; 32])
    }
    fn readiness(&self, _: &Path, _: &Path) -> Result<(), CoreError> {
        if self.ready.load(std::sync::atomic::Ordering::Relaxed) {
            Ok(())
        } else {
            Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                "injected readiness",
            ))
        }
    }
    fn prepare_visual_stream(
        &self,
        _: &Path,
        output: &Path,
        _: u32,
        frames: u64,
        produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        std::fs::write(output, produce(0)?).unwrap();
        if frames > 1 {
            let _ = produce(frames - 1)?;
        }
        Ok(())
    }
    fn probe(&self, _: &Path, _: &Path) -> Result<ProbeResult, CoreError> {
        unreachable!()
    }
    fn execute(
        &self,
        _: &Path,
        plan: &RenderPlan,
        _: &Path,
        output: &Path,
        progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        self.executions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::fs::write(output, format!("{:?}", plan.intent)).unwrap();
        progress(RenderProgress { progress: 1.0 });
        Ok(())
    }
}

#[test]
fn preview_cache_fresh_publication_clones_revision_intent_and_preflight() {
    use std::sync::atomic::Ordering;
    let root = tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    let process = Arc::new(CacheProcess {
        executions: Default::default(),
        ready: true.into(),
    });
    let renderer = Renderer::new("unused", "unused", None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let project = visual_project();
    let first = renderer.render_preview(&project, root.path(), 100).unwrap();
    let bytes = std::fs::read(root.path().join(&first.relative_path)).unwrap();
    std::fs::write(
        root.path().join(&first.relative_path),
        b"corrupt published output",
    )
    .unwrap();
    let warm = renderer
        .clone()
        .with_request_id("cache-request-2")
        .unwrap()
        .render_preview(&project, root.path(), 100)
        .unwrap();
    assert_ne!(first.relative_path, warm.relative_path);
    assert_eq!(
        std::fs::read(root.path().join(&warm.relative_path)).unwrap(),
        bytes
    );
    assert_eq!(process.executions.load(Ordering::Relaxed), 1);
    process.ready.store(false, Ordering::Relaxed);
    let before = std::fs::read_dir(root.path().join("previews"))
        .unwrap()
        .count();
    assert_eq!(
        renderer
            .render_preview(&project, root.path(), 100)
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnavailable
    );
    assert_eq!(
        std::fs::read_dir(root.path().join("previews"))
            .unwrap()
            .count(),
        before
    );
    process.ready.store(true, Ordering::Relaxed);
    renderer.render_preview(&project, root.path(), 101).unwrap();
    let mut changed = project.clone();
    changed.revision += 1;
    renderer.render_preview(&changed, root.path(), 100).unwrap();
    changed = project.clone();
    if let TimelineItem::SolidColor(item) = &mut changed.tracks[0].items[0] {
        item.color = "#abcdef".into();
    }
    renderer.render_preview(&changed, root.path(), 100).unwrap();
    assert_eq!(process.executions.load(Ordering::Relaxed), 4);
    let options = PreviewRangeOptions {
        start_ms: 0,
        end_ms: 1000,
        width: 320,
        height: 180,
        fps: 15,
        include_audio: false,
    };
    let cold = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    let mut progress = vec![];
    let hit = renderer
        .render_preview_range(&project, root.path(), options, |value| {
            progress.push(value.progress)
        })
        .unwrap();
    assert_ne!(cold.relative_path, hit.relative_path);
    assert_eq!(progress, vec![1.0]);
    assert_eq!(process.executions.load(Ordering::Relaxed), 5);
    assert_eq!(renderer.preview_cache.counts(), (2, 5));
    for variant in [
        PreviewRangeOptions {
            width: 322,
            ..options
        },
        PreviewRangeOptions {
            height: 182,
            ..options
        },
        PreviewRangeOptions { fps: 16, ..options },
        PreviewRangeOptions {
            include_audio: true,
            ..options
        },
        PreviewRangeOptions {
            start_ms: 100,
            ..options
        },
        PreviewRangeOptions {
            end_ms: 900,
            ..options
        },
    ] {
        let before = process.executions.load(Ordering::Relaxed);
        let candidate = renderer
            .render_preview_range(&project, root.path(), variant, |_| {})
            .unwrap();
        assert_eq!(process.executions.load(Ordering::Relaxed), before + 1);
        let warm = renderer
            .render_preview_range(&project, root.path(), variant, |_| {})
            .unwrap();
        assert_eq!(process.executions.load(Ordering::Relaxed), before + 1);
        let cold_process = Arc::new(CacheProcess {
            executions: Default::default(),
            ready: true.into(),
        });
        let cold = Renderer::new("unused", "unused", None)
            .with_adapters(cold_process, Arc::new(FileSystemArtifactIo))
            .render_preview_range(&project, root.path(), variant, |_| {})
            .unwrap();
        for result in [&warm, &cold] {
            assert_eq!(
                std::fs::read(root.path().join(&candidate.relative_path)).unwrap(),
                std::fs::read(root.path().join(&result.relative_path)).unwrap()
            );
            assert_eq!(candidate.warnings, result.warnings);
            assert_eq!(candidate.text_layouts, result.text_layouts);
        }
    }
    assert_eq!(renderer.preview_cache.counts(), (8, 11));
    let mut other = project.clone();
    other.id = "other-project".into();
    renderer.render_preview(&other, root.path(), 100).unwrap();
    renderer.render_preview(&other, root.path(), 100).unwrap();
    assert_eq!(process.executions.load(Ordering::Relaxed), 12);
    assert_eq!(renderer.preview_cache.counts(), (9, 12));
    let replaced = renderer
        .clone()
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    replaced.render_preview(&project, root.path(), 100).unwrap();
    assert_eq!(process.executions.load(Ordering::Relaxed), 13);
    assert_eq!(replaced.preview_cache.counts(), (0, 1));
    let reconfigured = renderer.clone().with_font_roots([]);
    reconfigured
        .render_preview(&project, root.path(), 100)
        .unwrap();
    assert_eq!(process.executions.load(Ordering::Relaxed), 14);
    assert_eq!(reconfigured.preview_cache.counts(), (0, 1));
}

#[derive(Debug, Default)]
struct PreviewFaultIo(std::sync::atomic::AtomicU8);
impl ArtifactIo for PreviewFaultIo {
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
    fn read_preview_payload(&self, path: &Path, capacity: usize) -> std::io::Result<Vec<u8>> {
        match self.0.load(Ordering::Relaxed) {
            3 => Err(std::io::Error::other("optional read failure")),
            4 => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "optional read unsupported",
            )),
            _ => FileSystemArtifactIo.read_preview_payload(path, capacity),
        }
    }
    fn write(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        if self.0.load(Ordering::Relaxed) == 2
            && path.extension().is_some_and(|value| value == "png")
        {
            // Simulate a failed partial cache-hit write; normal cleanup must own it.
            std::fs::write(path, b"partial")?;
            Err(std::io::Error::other("hit write failure"))
        } else {
            FileSystemArtifactIo.write(path, bytes)
        }
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
        FileSystemArtifactIo.artifact_path_exists(path)
    }
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        FileSystemArtifactIo.remove(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        if self.0.load(Ordering::Relaxed) == 1 {
            Err(std::io::Error::other("publish failure"))
        } else {
            FileSystemArtifactIo.rename(from, to)
        }
    }
    fn size(&self, path: &Path) -> std::io::Result<u64> {
        if self.0.load(Ordering::Relaxed) == 5 {
            Err(std::io::Error::other("published metadata failure"))
        } else {
            FileSystemArtifactIo.size(path)
        }
    }
}

#[test]
fn preview_cache_failed_publication_hit_cleanup_and_optional_read_bypass() {
    let project = visual_project();
    for failure in [1, 5] {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let io = Arc::new(PreviewFaultIo(failure.into()));
        let process = Arc::new(CacheProcess {
            executions: Default::default(),
            ready: true.into(),
        });
        let renderer =
            Renderer::new("unused", "unused", None).with_adapters(process.clone(), io.clone());
        let error = renderer
            .render_preview(&project, root.path(), 0)
            .unwrap_err();
        assert_eq!(error.failed_stage.as_deref(), Some("publish"));
        assert_eq!(
            std::fs::read_dir(root.path().join("previews"))
                .unwrap()
                .count(),
            0
        );
        assert_eq!(renderer.preview_cache.counts(), (0, 1));
        io.0.store(0, Ordering::Relaxed);
        let cold = renderer.render_preview(&project, root.path(), 0).unwrap();
        assert_eq!(process.executions.load(Ordering::Relaxed), 2);
        let bytes = std::fs::read(root.path().join(&cold.relative_path)).unwrap();
        io.0.store(2, Ordering::Relaxed);
        assert_eq!(
            renderer
                .render_preview(&project, root.path(), 0)
                .unwrap_err()
                .failed_stage
                .as_deref(),
            Some("publish")
        );
        assert_eq!(
            std::fs::read_dir(root.path().join("previews"))
                .unwrap()
                .count(),
            1
        );
        assert_eq!(
            std::fs::read_dir(root.path()).unwrap().count(),
            1,
            "workspace cleanup"
        );
        io.0.store(0, Ordering::Relaxed);
        let warm = renderer.render_preview(&project, root.path(), 0).unwrap();
        assert_eq!(
            std::fs::read(root.path().join(&warm.relative_path)).unwrap(),
            bytes
        );
        assert_eq!(process.executions.load(Ordering::Relaxed), 2);
    }
    for failure in [3, 4] {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let io = Arc::new(PreviewFaultIo(failure.into()));
        let process = Arc::new(CacheProcess {
            executions: Default::default(),
            ready: true.into(),
        });
        let renderer = Renderer::new("unused", "unused", None).with_adapters(process.clone(), io);
        renderer.render_preview(&project, root.path(), 0).unwrap();
        renderer.render_preview(&project, root.path(), 0).unwrap();
        assert_eq!(process.executions.load(Ordering::Relaxed), 2);
        assert_eq!(renderer.preview_cache.counts(), (0, 2));
    }
}

#[derive(Debug, Default)]
struct NativeCacheProcess(std::sync::atomic::AtomicUsize);
impl ProcessExecutor for NativeCacheProcess {
    fn preview_cache_identity(&self, ffmpeg: &Path, ffprobe: &Path) -> Option<[u8; 32]> {
        SystemProcessExecutor.preview_cache_identity(ffmpeg, ffprobe)
    }
    fn readiness(&self, ffmpeg: &Path, ffprobe: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.readiness(ffmpeg, ffprobe)
    }
    fn probe(&self, ffprobe: &Path, path: &Path) -> Result<ProbeResult, CoreError> {
        SystemProcessExecutor.probe(ffprobe, path)
    }
    fn prepare_visual_stream(
        &self,
        ffmpeg: &Path,
        output: &Path,
        fps: u32,
        frames: u64,
        produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        SystemProcessExecutor.prepare_visual_stream(ffmpeg, output, fps, frames, produce)
    }
    fn execute(
        &self,
        ffmpeg: &Path,
        plan: &RenderPlan,
        filter: &Path,
        output: &Path,
        progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        SystemProcessExecutor.execute(ffmpeg, plan, filter, output, progress)
    }
}

#[test]
fn preview_cache_native_avoids_final_execution_and_keeps_export_uncached() {
    use std::sync::atomic::Ordering;
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(
            std::env::var("OPENCUT_PREVIEW_CACHE_TESTS_REQUIRED").as_deref(),
            Ok("1"),
            "required native backend is absent"
        );
        eprintln!("native preview cache evidence not configured");
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").expect("configured ffprobe");
    let root = tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    let process = Arc::new(NativeCacheProcess::default());
    let renderer = Renderer::new(ffmpeg.clone(), ffprobe, None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let mut project = visual_project();
    project.settings.width = 32;
    project.settings.height = 32;
    project.settings.fps = 10;
    let frame = renderer.render_preview(&project, root.path(), 0).unwrap();
    let warm = renderer
        .clone()
        .with_request_id("native-cache-2")
        .unwrap()
        .render_preview(&project, root.path(), 0)
        .unwrap();
    let decode = |artifact: &RenderArtifact| {
        let result = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(root.path().join(&artifact.relative_path))
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
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout.len(), 32 * 32 * 3);
        result.stdout
    };
    let cold_pixels = decode(&frame);
    assert_eq!(decode(&warm), cold_pixels);
    for pixel in cold_pixels.chunks_exact(3) {
        for (actual, expected) in pixel.iter().zip([17i16, 34, 51]) {
            assert!((*actual as i16 - expected).abs() <= 1);
        }
    }
    assert_eq!(process.0.load(Ordering::Relaxed), 1);
    let options = PreviewRangeOptions {
        start_ms: 0,
        end_ms: 1000,
        width: 32,
        height: 32,
        fps: 10,
        include_audio: true,
    };
    let range = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    let hit = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    assert_eq!(
        std::fs::read(root.path().join(&range.relative_path)).unwrap(),
        std::fs::read(root.path().join(&hit.relative_path)).unwrap()
    );
    assert_eq!(decode(&range), decode(&hit));
    assert_eq!(process.0.load(Ordering::Relaxed), 2);
    renderer
        .export_video(
            &project,
            root.path(),
            ExportOptions {
                output: &root.path().join("export.mp4"),
                width: 32,
                height: 32,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    assert_eq!(process.0.load(Ordering::Relaxed), 3);
    assert_eq!(renderer.preview_cache.counts(), (2, 2));
    let mut limited = renderer.clone();
    limited.preview_cache =
        Arc::new(crate::render_artifact::preview_cache::PreviewCache::test_with_limits(1, 32));
    let uncached = limited.render_preview(&project, root.path(), 0).unwrap();
    let repeat = limited.render_preview(&project, root.path(), 0).unwrap();
    assert_eq!(decode(&uncached), decode(&repeat));
    assert_eq!(process.0.load(Ordering::Relaxed), 5);
    assert_eq!(limited.preview_cache.counts(), (0, 2));
}

#[derive(Debug)]
struct FailingProcess {
    inner: CacheProcess,
    fail: std::sync::atomic::AtomicBool,
}
impl ProcessExecutor for FailingProcess {
    fn preview_cache_identity(&self, ffmpeg: &Path, ffprobe: &Path) -> Option<[u8; 32]> {
        self.inner.preview_cache_identity(ffmpeg, ffprobe)
    }
    fn readiness(&self, ffmpeg: &Path, ffprobe: &Path) -> Result<(), CoreError> {
        self.inner.readiness(ffmpeg, ffprobe)
    }
    fn probe(&self, _: &Path, _: &Path) -> Result<ProbeResult, CoreError> {
        unreachable!()
    }
    fn execute(
        &self,
        ffmpeg: &Path,
        plan: &RenderPlan,
        filter: &Path,
        output: &Path,
        progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        if self.fail.swap(false, Ordering::Relaxed) {
            self.inner.executions.fetch_add(1, Ordering::Relaxed);
            std::fs::write(output, b"partial").unwrap();
            Err(CoreError::render_failure(
                "render",
                Some(7),
                Some("injected exit".into()),
            ))
        } else {
            self.inner.execute(ffmpeg, plan, filter, output, progress)
        }
    }
}

#[test]
fn preview_cache_failed_execution_is_not_retained_and_retries_cleanly() {
    let root = tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    let process = Arc::new(FailingProcess {
        inner: CacheProcess {
            executions: Default::default(),
            ready: true.into(),
        },
        fail: true.into(),
    });
    let renderer = Renderer::new("unused", "unused", None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let project = visual_project();
    let error = renderer
        .render_preview(&project, root.path(), 0)
        .unwrap_err();
    assert_eq!(error.failed_stage.as_deref(), Some("render"));
    assert_eq!(error.ffmpeg_exit_code, Some(7));
    assert_eq!(
        std::fs::read_dir(root.path().join("previews"))
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        std::fs::read_dir(root.path()).unwrap().count(),
        1,
        "failed render workspace cleaned"
    );
    let cold = renderer.render_preview(&project, root.path(), 0).unwrap();
    let warm = renderer.render_preview(&project, root.path(), 0).unwrap();
    assert_eq!(
        std::fs::read(root.path().join(cold.relative_path)).unwrap(),
        std::fs::read(root.path().join(warm.relative_path)).unwrap()
    );
    assert_eq!(process.inner.executions.load(Ordering::Relaxed), 2);
    assert_eq!(renderer.preview_cache.counts(), (1, 2));
}

#[test]
fn preview_cache_warm_input_and_glyph_admission_reject_before_lookup_or_output() {
    let root = tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    let process = Arc::new(CacheProcess {
        executions: Default::default(),
        ready: true.into(),
    });
    let renderer = Renderer::new("unused", "unused", None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let mut project = visual_project();
    project.schema_version = crate::PROJECT_SCHEMA_VERSION;
    project.audio_buses = crate::default_audio_buses();
    renderer.render_preview(&project, root.path(), 0).unwrap();
    let fresh = Renderer::new("unused", "unused", None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let invalid = PreviewRangeOptions {
        start_ms: 0,
        end_ms: 1000,
        width: 0,
        height: 180,
        fps: 15,
        include_audio: true,
    };
    for (warm, cold) in [
        (
            renderer
                .render_preview(&project, root.path(), 1001)
                .unwrap_err(),
            fresh
                .render_preview(&project, root.path(), 1001)
                .unwrap_err(),
        ),
        (
            renderer
                .render_preview_range(&project, root.path(), invalid, |_| {})
                .unwrap_err(),
            fresh
                .render_preview_range(&project, root.path(), invalid, |_| {})
                .unwrap_err(),
        ),
    ] {
        assert_eq!(warm.code, cold.code);
        assert_eq!(warm.retryable, cold.retryable);
        assert_eq!(warm.failed_stage, cold.failed_stage);
    }
    let (text, faces) =
        crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
    std::fs::create_dir(root.path().join("fonts")).unwrap();
    for (hash, bytes) in &faces {
        let record = crate::fonts::record(bytes).unwrap();
        std::fs::write(root.path().join(&record.relative_path), bytes).unwrap();
        project.fonts.insert(hash.clone(), record);
    }
    project.tracks[0].items.push(serde_json::from_value(serde_json::json!({"type":"text","id":"title","text":"MM","document":{"runs":[{"text":"MM"}]},"fontSize":30,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[],"style":{"layout":{}},"fontBinding":text.font_binding})).unwrap());
    project.tracks[0]
        .items
        .last_mut()
        .unwrap()
        .visual_properties_mut()
        .stack_order = 1;
    let before = serde_json::to_vec(&project).unwrap();
    let mut warmed_limit = renderer.clone();
    warmed_limit.text_glyph_limit = Some(0);
    let mut cold_limit = fresh.clone();
    cold_limit.text_glyph_limit = Some(0);
    let warm = warmed_limit
        .render_preview(&project, root.path(), 0)
        .unwrap_err();
    let cold = cold_limit
        .render_preview(&project, root.path(), 0)
        .unwrap_err();
    assert_eq!(warm.code, ErrorCode::InvalidArgument, "{warm:?}");
    assert_eq!(warm.code, cold.code);
    assert_eq!(warm.retryable, cold.retryable);
    assert_eq!(warm.failed_stage, cold.failed_stage);
    assert_eq!(serde_json::to_vec(&project).unwrap(), before);
    assert_eq!(
        std::fs::read_dir(root.path().join("previews"))
            .unwrap()
            .count(),
        1
    );
    assert_eq!(renderer.preview_cache.counts(), (0, 1));
    assert_eq!(fresh.preview_cache.counts(), (0, 0));
    assert_eq!(process.executions.load(Ordering::Relaxed), 1);
}
