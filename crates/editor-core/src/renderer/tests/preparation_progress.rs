use super::*;
use std::sync::atomic::AtomicBool;

fn witness() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../../contracts/render-preparation-progress-v1.json"
    ))
    .unwrap()
}

#[derive(Debug)]
struct SlowPreparationProcess {
    inner: FakeProcess,
    started: AtomicBool,
    fail_preparation: bool,
    frames: Mutex<Vec<Vec<u8>>>,
}

impl SlowPreparationProcess {
    fn new(fail_preparation: bool, fail_encoding: bool) -> Self {
        Self {
            inner: FakeProcess {
                readiness_error: false,
                probe_error: false,
                run_failure: fail_encoding.then_some(FakeRunFailure::Exit),
                executions: Mutex::new(vec![]),
            },
            started: AtomicBool::new(false),
            fail_preparation,
            frames: Mutex::new(vec![]),
        }
    }
}

impl ProcessExecutor for SlowPreparationProcess {
    fn preview_cache_identity(&self, _: &Path, _: &Path) -> Option<[u8; 32]> {
        Some([179; 32])
    }
    fn readiness(&self, ffmpeg: &Path, probe: &Path) -> Result<(), CoreError> {
        self.inner.readiness(ffmpeg, probe)
    }
    fn probe(&self, ffprobe: &Path, path: &Path) -> Result<ProbeResult, CoreError> {
        self.inner.probe(ffprobe, path)
    }
    fn prepare_visual_stream(
        &self,
        _: &Path,
        output: &Path,
        _: u32,
        count: u64,
        produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        for n in 0..count {
            assert!(!self.started.load(Ordering::SeqCst));
            std::thread::sleep(std::time::Duration::from_millis(5));
            if self.fail_preparation && n == 1 {
                return Err(CoreError::render_failure(GRAPH_BUILD_STAGE, None, None));
            }
            self.frames.lock().unwrap().push(produce(n)?);
        }
        std::fs::write(output, &self.frames.lock().unwrap()[0]).unwrap();
        Ok(())
    }
    fn execute(
        &self,
        ffmpeg: &Path,
        plan: &RenderPlan,
        filter: &Path,
        output: &Path,
        on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        self.started.store(true, Ordering::SeqCst);
        for value in witness()["encodingFractions"].as_array().unwrap() {
            on_progress(RenderProgress {
                progress: value.as_f64().unwrap(),
            });
        }
        self.inner
            .execute(ffmpeg, plan, filter, output, on_progress)
    }
}

fn small_project() -> Project {
    let mut project = visual_project();
    project.settings.width = 8;
    project.settings.height = 8;
    project.settings.fps = 4;
    project
}

fn range() -> PreviewRangeOptions {
    PreviewRangeOptions {
        start_ms: 0,
        end_ms: 1000,
        width: 8,
        height: 8,
        fps: 4,
        include_audio: false,
    }
}

#[test]
fn slow_native_preparation_is_visible_before_execution_and_one_follows_publication() {
    for export in [false, true] {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join("previews")).unwrap();
        let output = root.path().join("output.mp4");
        let process = Arc::new(SlowPreparationProcess::new(false, false));
        let renderer = Renderer::new("unused", "unused", None)
            .with_adapters(process.clone(), Arc::new(FakeArtifactIo::default()));
        let project = small_project();
        let original = serde_json::to_vec(&project).unwrap();
        let mut events = Vec::new();
        let mut observe = |event: RenderProgress| {
            if event.progress == 1.0 {
                if export {
                    assert_eq!(std::fs::read(&output).unwrap(), b"rendered");
                } else {
                    assert!(
                        std::fs::read_dir(root.path().join("previews"))
                            .unwrap()
                            .any(|entry| entry
                                .unwrap()
                                .file_name()
                                .to_string_lossy()
                                .starts_with("preview-range-"))
                    );
                }
            }
            events.push((event.progress, process.started.load(Ordering::SeqCst)));
        };
        let artifact = if export {
            renderer.export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 8,
                    height: 8,
                    overwrite: false,
                },
                &mut observe,
            )
        } else {
            renderer.render_preview_range(&project, root.path(), range(), &mut observe)
        }
        .unwrap();
        assert!(artifact.size_bytes > 0);
        let expected: Vec<_> = witness()["sampledProgress"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        assert_eq!(events.len(), expected.len());
        for ((actual, started), expected) in events.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-12);
            if *actual <= 0.5 {
                assert!(!started);
            }
        }
        assert!(events.windows(2).all(|v| v[0].0 <= v[1].0));
        let frames = process.frames.lock().unwrap();
        assert_eq!(
            frames.len(),
            witness()["sampledFrames"].as_u64().unwrap() as usize
        );
        assert!(frames.windows(2).all(|v| v[0] == v[1]));
        assert_eq!(serde_json::to_vec(&project).unwrap(), original);
        assert!(!std::fs::read_dir(root.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".opencut-work-")
        }));
    }
}

#[test]
fn preparation_encoding_and_publication_failures_never_report_completion() {
    for failure in 0..4 {
        let root = tempdir().unwrap();
        let output = root.path().join("output.mp4");
        let process = Arc::new(SlowPreparationProcess::new(failure == 0, failure == 1));
        let io = Arc::new(FakeArtifactIo {
            fail_rename: failure == 2,
            fail_size: failure == 3,
            ..Default::default()
        });
        let renderer = Renderer::new("unused", "unused", None).with_adapters(process.clone(), io);
        let project = small_project();
        let original = serde_json::to_vec(&project).unwrap();
        let mut events = vec![];
        let error = renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 8,
                    height: 8,
                    overwrite: false,
                },
                |p| events.push(p.progress),
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::FfmpegFailed);
        assert!(
            events
                .iter()
                .all(|p| p.is_finite() && (0.0..1.0).contains(p))
        );
        assert!(events.windows(2).all(|v| v[0] <= v[1]));
        assert_eq!(serde_json::to_vec(&project).unwrap(), original);
        if failure == 0 {
            assert!(!process.started.load(Ordering::SeqCst));
            assert_eq!(events, vec![0.125]);
        }
        if failure < 3 {
            assert!(!output.exists());
        }
    }
}

#[test]
fn empty_and_single_frame_work_remain_bounded_and_invalid_ranges_emit_nothing() {
    for sampled in [false, true] {
        let root = tempdir().unwrap();
        let process = Arc::new(SlowPreparationProcess::new(false, false));
        let renderer = Renderer::new("unused", "unused", None)
            .with_adapters(process.clone(), Arc::new(FakeArtifactIo::default()));
        let mut project = if sampled {
            small_project()
        } else {
            empty_project()
        };
        project.settings.width = 8;
        project.settings.height = 8;
        project.settings.fps = 1;
        let output = root.path().join("output.mp4");
        let mut events = vec![];
        renderer
            .export_video(
                &project,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 8,
                    height: 8,
                    overwrite: false,
                },
                |p| events.push(p.progress),
            )
            .unwrap();
        assert_eq!(events.last(), Some(&1.0));
        assert!(
            events
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        );
        assert!(events.windows(2).all(|v| v[0] <= v[1]));
        assert_eq!(process.frames.lock().unwrap().len(), usize::from(sampled));
        assert_eq!(events[0], if sampled { 0.5 } else { 0.2475 });
        if !sampled {
            let expected: Vec<f64> = witness()["unsampledProgress"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap())
                .collect();
            assert_eq!(events, expected);
        }
        let mut invalid = range();
        invalid.start_ms = invalid.end_ms;
        let mut rejected = vec![];
        assert_eq!(
            renderer
                .render_preview_range(&project, root.path(), invalid, |p| rejected
                    .push(p.progress))
                .unwrap_err()
                .code,
            ErrorCode::ValidationFailed
        );
        assert!(rejected.is_empty());
    }
}

#[test]
fn cached_range_still_requires_successful_publication_before_one() {
    let root = tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    let process = Arc::new(SlowPreparationProcess::new(false, false));
    let renderer = Renderer::new("unused", "unused", None)
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let project = small_project();
    let first = renderer
        .render_preview_range(&project, root.path(), range(), |_| {})
        .unwrap();
    assert_eq!(process.inner.executions.lock().unwrap().len(), 1);
    process.started.store(false, Ordering::SeqCst);
    let mut warm_events = vec![];
    let warm = renderer
        .render_preview_range(&project, root.path(), range(), |p| {
            warm_events.push(p.progress);
            if p.progress == 1.0 {
                assert_eq!(
                    std::fs::read_dir(root.path().join("previews"))
                        .unwrap()
                        .count(),
                    2
                );
            }
        })
        .unwrap();
    assert_ne!(first.relative_path, warm.relative_path);
    assert_eq!(warm_events, vec![0.125, 0.25, 0.375, 0.5, 1.0]);
    assert_eq!(process.inner.executions.lock().unwrap().len(), 1);
    let mut cached = renderer.clone();
    cached.artifact_io = Arc::new(FakeArtifactIo {
        fail_rename: true,
        ..Default::default()
    });
    process.started.store(false, Ordering::SeqCst);
    let before = std::fs::read(root.path().join(&first.relative_path)).unwrap();
    let mut events = vec![];
    let error = cached
        .render_preview_range(&project, root.path(), range(), |p| events.push(p.progress))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FfmpegFailed);
    assert_eq!(error.failed_stage.as_deref(), Some(PUBLISH_STAGE));
    assert_eq!(
        process.inner.executions.lock().unwrap().len(),
        1,
        "must exercise cache branch"
    );
    assert!(!process.started.load(Ordering::SeqCst));
    assert_eq!(events.last(), Some(&0.5));
    assert!(!events.contains(&1.0));
    assert_eq!(
        std::fs::read(root.path().join(&first.relative_path)).unwrap(),
        before
    );
}
