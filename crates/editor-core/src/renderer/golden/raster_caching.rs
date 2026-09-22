use super::*;
use serde_json::json;

#[test]
fn native_raster_cache_all_routes_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let (root, core, mut project) = crate::render_artifact::raster_cache::tests::fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let track = project.tracks[1].id.clone();
    let position = |x, y| {
        let mut t = crate::Transform2D::default();
        t.position.x = x;
        t.position.y = y;
        t
    };
    let ops:Vec<crate::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_shape","trackId":track,"resultAlias":"box","startMs":0,"durationMs":1000,"stroke":null,"geometry":{"type":"rectangle","width":30,"height":20},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"transform2d":position(30.0,40.0)},
        {"operation":"update_item","itemId":"@box","transform2d":position(40.0,40.0)},
        {"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":"<svg width=\"15\" height=\"15\"><rect width=\"15\" height=\"15\" fill=\"#00f\"/></svg>","transform2d":position(100.0,40.0)}
    ])).unwrap();
    core.edit_batch(&project.id, project.revision, ops).unwrap();
    project = core.get_project(&project.id).unwrap();
    // Add the established deterministic audio fixture to this immutable snapshot.
    let audio = fixture_project();
    project.assets = audio.assets;
    project.tracks.push(audio.tracks[1].clone());
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&project).unwrap(),
    )
    .unwrap();
    // Adopt the injected legacy audio fixture into managed content before the
    // read-only render/draft assertions begin.
    project = core.get_project(&project.id).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut baseline = None;
    for round in 0..2 {
        let frame = renderer.render_preview(&project, &dir, 500).unwrap();
        let rgb = decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
        // Independent interior pixels for both vector paths.
        let pixel = |x: usize, y: usize| &rgb[(y * 160 + x) * 3..(y * 160 + x) * 3 + 3];
        // The composed frame traverses FFmpeg's YUV conversion. Raw PAM pixel
        // oracles are exact in the cache unit suite; allow conversion rounding here.
        assert!(
            pixel(50, 50)
                .iter()
                .zip([255, 0, 0])
                .all(|(a, b)| a.abs_diff(b) <= 3)
        );
        assert!(
            pixel(105, 45)
                .iter()
                .zip([0, 0, 255])
                .all(|(a, b)| a.abs_diff(b) <= 3)
        );
        if let Some(baseline) = &baseline {
            assert_eq!(&rgb, baseline);
        } else {
            baseline = Some(rgb.clone());
        }
        let range = renderer
            .render_preview_range(
                &project,
                &dir,
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
        let range = dir.join(range.relative_path);
        let output = root.path().join(format!("cache-{round}.mp4"));
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 160,
                    height: 90,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for path in [&range, &output] {
            assert!(
                structural_similarity(&rgb, &decode_rgb_frame(&tools.ffmpeg, path, 500)).unwrap()
                    >= SSIM_MINIMUM
            );
        }
        let a = decode_mono_f32(&tools.ffmpeg, &range);
        let b = decode_mono_f32(&tools.ffmpeg, &output);
        assert!(a.iter().any(|v| v.abs() > 0.001));
        assert_eq!(a.len(), b.len());
        assert!(aligned_rms_error(&a, &b, 0).unwrap() <= PCM_RMS_MAXIMUM);
        let fresh = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
        let cold = fresh.render_preview(&project, &dir, 500).unwrap();
        assert_eq!(
            rgb,
            decode_rgb_frame(&tools.ffmpeg, &dir.join(cold.relative_path), 0)
        );
        assert_eq!(frame.warnings, cold.warnings);
        assert_eq!(frame.text_layouts, cold.text_layouts);
        let fresh_range = fresh
            .render_preview_range(
                &project,
                &dir,
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
        let fresh_output = root.path().join(format!("fresh-cache-{round}.mp4"));
        fresh
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &fresh_output,
                    width: 160,
                    height: 90,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for (cached, fresh_path) in [
            (&range, dir.join(fresh_range.relative_path)),
            (&output, fresh_output),
        ] {
            assert_eq!(
                decode_rgb_frame(&tools.ffmpeg, cached, 500),
                decode_rgb_frame(&tools.ffmpeg, &fresh_path, 500)
            );
            assert_eq!(
                decode_mono_f32(&tools.ffmpeg, cached),
                decode_mono_f32(&tools.ffmpeg, &fresh_path)
            );
        }
    }
    assert_eq!(renderer.raster_cache.misses.load(Ordering::Relaxed), 3);
    assert!(renderer.raster_cache.hits.load(Ordering::Relaxed) >= 15);
    let before = fs::read(dir.join("project.json")).unwrap();
    let item = project.tracks[1].items[0].id();
    let draft = core
        .create_draft(
            &project.id,
            project.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"set_item_visibility","itemId":item,"hidden":false}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let snapshot = core
        .get_draft_state(&project.id, &draft.id)
        .unwrap()
        .project;
    let frame = renderer.render_preview(&snapshot, &dir, 500).unwrap();
    assert_eq!(
        baseline.unwrap(),
        decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0)
    );
    assert_eq!(before, fs::read(dir.join("project.json")).unwrap());
    assert_eq!(renderer.raster_cache.misses.load(Ordering::Relaxed), 3);
}
