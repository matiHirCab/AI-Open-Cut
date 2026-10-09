//! Warm encoded artifacts checked against the unchanged reviewed media references.
use super::*;
use crate::render_process::ProcessExecutor;

#[test]
fn native_preview_cache_preserves_frozen_av_references_dependencies_and_metadata() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let container = fixture_container_root();
    let _lock = GoldenFixtureLock::exclusive(&container).unwrap();
    let references = selected_generation_root(&container).unwrap();
    let manifest = load_manifest(&references).unwrap();
    validate_manifest(&references, &manifest, true).unwrap();
    assert_eq!(manifest.environment.font_sha256, tools.font_sha256);
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let project = fixture_project();
    let before = serde_json::to_vec(&project).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    for time in SAMPLE_TIMESTAMPS_MS {
        let cold = renderer
            .render_preview(&project, root.path(), time)
            .unwrap();
        let warm = renderer
            .clone()
            .with_request_id(&format!("frame-{time}"))
            .unwrap()
            .render_preview(&project, root.path(), time)
            .unwrap();
        assert_ne!(cold.relative_path, warm.relative_path);
        assert_eq!(cold.warnings, warm.warnings);
        assert_eq!(cold.text_layouts, warm.text_layouts);
        let expected = reference_bytes(&references, &manifest, "frame", Some(time));
        for artifact in [&cold, &warm] {
            let pixels =
                decode_rgb_frame(&tools.ffmpeg, &root.path().join(&artifact.relative_path), 0);
            assert!(structural_similarity(&expected, &pixels).unwrap() >= SSIM_MINIMUM);
        }
    }
    assert_eq!(renderer.preview_cache.counts(), (3, 3));
    let options = PreviewRangeOptions {
        start_ms: 0,
        end_ms: DURATION_MS,
        width: WIDTH,
        height: HEIGHT,
        fps: FPS,
        include_audio: true,
    };
    let cold = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    let cold_path = root.path().join(&cold.relative_path);
    let encoded = fs::read(&cold_path).unwrap();
    fs::remove_file(&cold_path).unwrap();
    let mut progress = vec![];
    let warm = renderer
        .render_preview_range(&project, root.path(), options, |p| {
            progress.push(p.progress)
        })
        .unwrap();
    assert_ne!(cold.relative_path, warm.relative_path);
    assert_eq!(cold.warnings, warm.warnings);
    assert_eq!(cold.text_layouts, warm.text_layouts);
    assert_eq!(progress, vec![1.0]);
    let path = root.path().join(&warm.relative_path);
    assert_eq!(fs::read(&path).unwrap(), encoded);
    for time in SAMPLE_TIMESTAMPS_MS {
        let expected = reference_bytes(&references, &manifest, "frame", Some(time));
        assert!(
            structural_similarity(&expected, &decode_rgb_frame(&tools.ffmpeg, &path, time))
                .unwrap()
                >= SSIM_MINIMUM
        );
    }
    let expected_audio =
        bytes_to_f32(&reference_bytes(&references, &manifest, "audio", None)).unwrap();
    let actual_audio = decode_mono_f32(&tools.ffmpeg, &path);
    assert!(actual_audio.iter().any(|value| value.abs() > 0.001));
    assert!(
        aligned_rms_error(
            &expected_audio,
            &actual_audio,
            (AUDIO_SAMPLE_RATE_HZ / FPS) as usize
        )
        .unwrap()
            <= PCM_RMS_MAXIMUM
    );
    assert!(
        renderer
            .probe(&path)
            .unwrap()
            .duration_ms
            .unwrap()
            .abs_diff(DURATION_MS)
            <= 1000 / u64::from(FPS)
    );
    assert_eq!(renderer.preview_cache.counts(), (4, 4));
    assert_eq!(serde_json::to_vec(&project).unwrap(), before);
    // Same revision, changed actual managed input: content must separate even
    // when authored binding paths and the immutable Project are identical.
    let media = root.path().join("assets/tone.wav");
    let mut samples = fs::read(&media).unwrap();
    for sample in samples[44..].chunks_exact_mut(2) {
        let value = i16::from_le_bytes([sample[0], sample[1]]) / 2;
        sample.copy_from_slice(&value.to_le_bytes());
    }
    fs::write(&media, &samples).unwrap();
    let changed = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    let fresh = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let independent_cold = fresh
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap();
    let changed_audio = decode_mono_f32(&tools.ffmpeg, &root.path().join(&changed.relative_path));
    assert!(aligned_rms_error(&actual_audio, &changed_audio, 0).unwrap() > PCM_RMS_MAXIMUM);
    assert!(
        aligned_rms_error(
            &changed_audio,
            &decode_mono_f32(
                &tools.ffmpeg,
                &root.path().join(independent_cold.relative_path)
            ),
            0
        )
        .unwrap()
            <= PCM_RMS_MAXIMUM
    );
    assert_eq!(renderer.preview_cache.counts(), (4, 5));
    fs::remove_file(&media).unwrap();
    let count = fs::read_dir(root.path().join("previews")).unwrap().count();
    let warm_error = renderer
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap_err();
    let cold_error = fresh
        .render_preview_range(&project, root.path(), options, |_| {})
        .unwrap_err();
    assert_eq!(warm_error.code, cold_error.code);
    assert_eq!(warm_error.failed_stage, cold_error.failed_stage);
    assert_eq!(
        fs::read_dir(root.path().join("previews")).unwrap().count(),
        count
    );
    assert_eq!(
        renderer.preview_cache.counts(),
        (4, 5),
        "missing input must precede lookup"
    );
}

#[derive(Debug, Default)]
struct NormalizationProcess {
    inner: super::master_normalization::RecordingProcess,
    executions: std::sync::atomic::AtomicUsize,
}
impl ProcessExecutor for NormalizationProcess {
    fn preview_cache_identity(&self, f: &Path, p: &Path) -> Option<[u8; 32]> {
        SystemProcessExecutor.preview_cache_identity(f, p)
    }
    fn readiness(&self, f: &Path, p: &Path) -> Result<(), CoreError> {
        self.inner.readiness(f, p)
    }
    fn probe(&self, p: &Path, a: &Path) -> Result<ProbeResult, CoreError> {
        self.inner.probe(p, a)
    }
    fn master_normalization_readiness(&self, f: &Path) -> Result<(), CoreError> {
        self.inner.master_normalization_readiness(f)
    }
    fn audio_bus_dsp_readiness(&self, f: &Path) -> Result<(), CoreError> {
        self.inner.audio_bus_dsp_readiness(f)
    }
    fn audio_bus_ducking_readiness(&self, f: &Path) -> Result<(), CoreError> {
        self.inner.audio_bus_ducking_readiness(f)
    }
    fn raster_source(
        &self,
        ffmpeg: &Path,
        source: &str,
        size: (u32, u32),
    ) -> Result<Vec<u8>, CoreError> {
        SystemProcessExecutor.raster_source(ffmpeg, source, size)
    }
    fn prepare_visual_stream(
        &self,
        f: &Path,
        o: &Path,
        fps: u32,
        frames: u64,
        produce: &mut dyn FnMut(u64) -> Result<Vec<u8>, CoreError>,
    ) -> Result<(), CoreError> {
        SystemProcessExecutor.prepare_visual_stream(f, o, fps, frames, produce)
    }
    fn prepare_master_normalization(
        &self,
        f: &Path,
        p: &crate::render_plan::audio_analysis::AudioAnalysisPlan,
        settings: &crate::MasterNormalization,
        s: &Path,
        w: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<crate::render_plan::master_normalization::PreparedMasterNormalization, CoreError>
    {
        self.inner
            .prepare_master_normalization(f, p, settings, s, w, g)
    }
    fn execute(
        &self,
        f: &Path,
        p: &RenderPlan,
        s: &Path,
        o: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        self.executions.fetch_add(1, Ordering::Relaxed);
        self.inner.execute(f, p, s, o, g)
    }
}

#[test]
fn native_preview_cache_preserves_complete_root_normalization_on_warm_ranges_and_crops() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    super::master_normalization::source(
        &root.path().join("assets/tone.wav"),
        3000,
        0.125,
        false,
        false,
    );
    let mut project = fixture_project();
    // Match the existing normalization recorder's audio-only fixture; the
    // separate frozen-AV test covers animated typography and compositing.
    project.tracks[0].items.clear();
    project.schema_version = PROJECT_SCHEMA_VERSION;
    project.audio_buses = crate::default_audio_buses();
    project.assets[0].duration_ms = Some(3000);
    let TimelineItem::Media(item) = &mut project.tracks[1].items[0] else {
        panic!("fixture media")
    };
    item.duration_ms = 3000;
    item.audio = crate::AudioSettings::default();
    item.keyframes.clear();
    let settings: crate::MasterNormalization = serde_json::from_value(
        serde_json::from_str::<serde_json::Value>(include_str!(
            "../../../../../contracts/master-normalization-v1.json"
        ))
        .unwrap()["settingsExample"]
            .clone(),
    )
    .unwrap();
    project.master_normalization = Some(settings.clone());
    let process = Arc::new(NormalizationProcess::default());
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()))
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let full = PreviewRangeOptions {
        start_ms: 0,
        end_ms: 3000,
        width: WIDTH,
        height: HEIGHT,
        fps: FPS,
        include_audio: true,
    };
    let cold = renderer
        .render_preview_range(&project, root.path(), full, |_| {})
        .unwrap();
    let pcm = process.inner.rendered_pcm.lock().unwrap().clone();
    let (loudness, peak) = super::master_normalization::independent_metrics(
        &tools,
        &pcm,
        &root.path().join("full.f32"),
    );
    assert!((loudness - settings.target_integrated_lufs).abs() <= 0.1);
    assert!(peak <= settings.target_true_peak_dbtp + 0.01);
    let warm = renderer
        .render_preview_range(&project, root.path(), full, |_| {})
        .unwrap();
    assert_eq!(
        fs::read(root.path().join(cold.relative_path)).unwrap(),
        fs::read(root.path().join(warm.relative_path)).unwrap()
    );
    assert_eq!(process.executions.load(Ordering::Relaxed), 1);
    let crop = PreviewRangeOptions {
        start_ms: 600,
        end_ms: 2400,
        ..full
    };
    let cold_crop = renderer
        .render_preview_range(&project, root.path(), crop, |_| {})
        .unwrap();
    assert_eq!(
        *process.inner.rendered_pcm.lock().unwrap(),
        pcm,
        "crop retains the complete-root precodec decision"
    );
    let warm_crop = renderer
        .render_preview_range(&project, root.path(), crop, |_| {})
        .unwrap();
    assert_eq!(
        fs::read(root.path().join(cold_crop.relative_path)).unwrap(),
        fs::read(root.path().join(warm_crop.relative_path)).unwrap()
    );
    assert_eq!(process.executions.load(Ordering::Relaxed), 2);
    assert_eq!(renderer.preview_cache.counts(), (2, 2));
}

#[test]
fn native_preview_cache_fingerprints_changed_default_font_content() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let original = fs::read(&tools.font).unwrap();
    let font = root.path().join("default.ttf");
    fs::write(&font, &original).unwrap();
    let project = fixture_project();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(font.clone()));
    renderer.render_preview(&project, root.path(), 500).unwrap();
    renderer.render_preview(&project, root.path(), 500).unwrap();
    assert_eq!(renderer.preview_cache.counts(), (1, 1));
    // A valid sfnt permits trailing bytes. Same path and glyphs, changed verified
    // bytes: dependency identity must miss even when this frame looks identical.
    let mut changed = original.clone();
    changed.extend_from_slice(b"preview-cache-font-dependency");
    fs::write(&font, changed).unwrap();
    let candidate = renderer.render_preview(&project, root.path(), 500).unwrap();
    assert_eq!(renderer.preview_cache.counts(), (1, 2));
    let cold = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(font.clone()))
        .render_preview(&project, root.path(), 500)
        .unwrap();
    assert_eq!(
        fs::read(root.path().join(candidate.relative_path)).unwrap(),
        fs::read(root.path().join(cold.relative_path)).unwrap()
    );
    fs::write(&font, original).unwrap();
    renderer.render_preview(&project, root.path(), 500).unwrap();
    assert_eq!(renderer.preview_cache.counts(), (2, 2));
}
