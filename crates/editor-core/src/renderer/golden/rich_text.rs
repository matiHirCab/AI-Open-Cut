//! Stored documents preserve legacy pixels and share all render entry points.
use super::*;
use serde_json::json;

#[test]
fn native_rich_text_render_conformance() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}

pub(super) fn conformance(tools: &NativeTools) {
    animation_conformance(tools);
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut project = fixture_project();
    let id = core
        .create_project("Rich text conformance", project.settings.clone())
        .unwrap()
        .project_id;
    project.id = id.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut legacy = serde_json::to_value(&project).unwrap();
    legacy["schemaVersion"] = json!(17);
    legacy["tracks"][0]["items"][1]
        .as_object_mut()
        .unwrap()
        .remove("document");
    let legacy: Project = serde_json::from_value(legacy).unwrap();
    assert_eq!(
        evaluate_project(&legacy, WIDTH, HEIGHT, FPS).unwrap().scene,
        evaluate_project(&project, WIDTH, HEIGHT, FPS)
            .unwrap()
            .scene
    );
    let frame = renderer.render_preview(&legacy, &dir, 500).unwrap();
    let old_pixels = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
    let frame = renderer.render_preview(&project, &dir, 500).unwrap();
    assert_eq!(
        old_pixels,
        grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0)
    );

    for styled in [false, true] {
        if styled {
            let TimelineItem::Text(text) = &mut project.tracks[0].items[1] else {
                unreachable!()
            };
            text.document = serde_json::from_value(json!({"runs":[{"text":"café →\n", "color":"#ff2200"},{"text":"WWWW iiii","color":"#22ff00"}]})).unwrap();
            text.text = text.document.text();
        }
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&project).unwrap(),
        )
        .unwrap();
        let draft = core.create_draft(&id, project.revision, vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":"animated-text","hidden":false})).unwrap()], None).unwrap();
        let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
        assert_eq!(
            evaluate_project(&project, WIDTH, HEIGHT, FPS)
                .unwrap()
                .scene,
            evaluate_project(&materialized, WIDTH, HEIGHT, FPS)
                .unwrap()
                .scene
        );
        let before = serde_json::to_vec(&project).unwrap();
        let range = renderer
            .render_preview_range(
                &project,
                &dir,
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
            .unwrap();
        let output = root.path().join(format!("rich-text-{styled}.mp4"));
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: WIDTH,
                    height: HEIGHT,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let rgb = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(&range.relative_path), 500);
        assert!(
            structural_similarity(&rgb, &grids::decode_rgb_frame(&tools.ffmpeg, &output, 500))
                .unwrap()
                >= 0.99
        );
        let frame = renderer.render_preview(&project, &dir, 500).unwrap();
        let pixels = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
        if styled {
            assert_ne!(pixels, old_pixels, "stored run colors were discarded");
            assert!(
                pixels
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .any(|rgb| rgb[0] > rgb[1].saturating_add(40))
            );
            assert!(
                pixels
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .any(|rgb| rgb[1] > rgb[0].saturating_add(40))
            );
        }
        assert!(structural_similarity(&pixels, &rgb).unwrap() >= 0.99);
        let frame = renderer.render_preview(&materialized, &dir, 500).unwrap();
        assert_eq!(
            pixels,
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0)
        );
        let a = decode_mono_f32(&tools.ffmpeg, &dir.join(range.relative_path));
        let b = decode_mono_f32(&tools.ffmpeg, &output);
        assert_eq!(a.len(), b.len());
        assert!(!a.is_empty());
        assert!(
            (a.iter()
                .zip(&b)
                .map(|(x, y)| f64::from(x - y).powi(2))
                .sum::<f64>()
                / a.len() as f64)
                .sqrt()
                <= 0.0001
        );
        assert_eq!(serde_json::to_vec(&project).unwrap(), before);
    }
    // A private font directory with no bold face deterministically exercises
    // the dependency error independently of host-installed fonts.
    let font_dir = root.path().join("font-only");
    fs::create_dir(&font_dir).unwrap();
    let font = font_dir.join("DejaVuSans.ttf");
    fs::copy(&tools.font, &font).unwrap();
    let TimelineItem::Text(text) = &mut project.tracks[0].items[1] else {
        unreachable!()
    };
    text.document.runs[0].bold = Some(true);
    let missing = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(font));
    assert_eq!(
        missing
            .render_preview(&project, &dir, 500)
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnavailable
    );
}

#[test]
fn native_styled_root_animation_conformance() {
    if let Some(tools) = configured_native_tools() {
        animation_conformance(&tools);
    }
}

fn animation_text(project: &mut Project) -> &mut crate::TextItem {
    let TimelineItem::Text(text) = &mut project.tracks[0].items[0] else {
        unreachable!()
    };
    text
}

fn animation_conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut project = fixture_project();
    let id = core
        .create_project("Animation", project.settings.clone())
        .unwrap()
        .project_id;
    project.id = id.clone();
    project.tracks[0].items.remove(0);
    let text = animation_text(&mut project);
    text.visual_properties.stack_order = 0;
    text.text = "WW".into();
    // A neutral override selects styled preparation without chroma-subsampling
    // noise obscuring the independent animation oracle. Colored runs are
    // covered by the existing conformance fixture above.
    text.document =
        serde_json::from_value(json!({"runs":[{"text":"WW","color":"#cccccc"}]})).unwrap();
    text.font_size = 18;
    text.style = crate::TextStyle::default();
    text.style.anchor = crate::AnchorPoint::Center;
    text.visual_properties.transform = crate::Transform {
        position_x: 40.0,
        position_y: 40.0,
        scale: 1.0,
        opacity: 1.0,
    };
    let dir = core.paths().project_dir(&id).unwrap();
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    for property in ["position", "scale", "opacity"] {
        let text = animation_text(&mut project);
        text.keyframes = [0, 1000]
            .into_iter()
            .map(|time| {
                let value = match property {
                    "position" => {
                        json!({"type":"position","x":if time==0 {40.0} else {80.0},"y":40.0})
                    }
                    "scale" => json!({"type":"scalar","value":if time==0 {1.0} else {1.5}}),
                    _ => json!({"type":"scalar","value":if time==0 {0.2} else {1.0}}),
                };
                serde_json::from_value(
                    json!({"property":property,"timeMs":time,"value":value,"easing":"linear"}),
                )
                .unwrap()
            })
            .collect();
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&project).unwrap(),
        )
        .unwrap();
        // Normalize fixture media ownership before asserting preview isolation.
        project = core.get_state(&id, None).unwrap().project;
        let authoritative = fs::read(dir.join("project.json")).unwrap();
        let draft = core.create_draft(&id,project.revision,vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":"animated-text","hidden":false})).unwrap()],None).unwrap();
        let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
        let range = renderer
            .render_preview_range(
                &project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: WIDTH,
                    height: HEIGHT,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let output = root.path().join(format!("animation-{property}.mp4"));
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: WIDTH,
                    height: HEIGHT,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let mut metrics = Vec::new();
        for time in [0, 400, 800] {
            let frame = renderer.render_preview(&project, &dir, time).unwrap();
            let pixels = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
            // Independent oracle: explicit linear values, not evaluator helpers.
            let mut oracle = project.clone();
            let text = animation_text(&mut oracle);
            text.keyframes.clear();
            let fraction = time as f64 / 1000.0;
            match property {
                "position" => text.visual_properties.transform.position_x = 40.0 + 40.0 * fraction,
                "scale" => text.visual_properties.transform.scale = 1.0 + 0.5 * fraction,
                _ => text.visual_properties.transform.opacity = 0.2 + 0.8 * fraction,
            }
            let expected = renderer.render_preview(&oracle, &dir, time).unwrap();
            let expected =
                grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(expected.relative_path), 0);
            assert!(
                structural_similarity(&pixels, &expected).unwrap() >= 0.99,
                "{property} static oracle at {time}"
            );
            for path in [&dir.join(&range.relative_path), &output] {
                assert!(
                    structural_similarity(
                        &pixels,
                        &grids::decode_rgb_frame(&tools.ffmpeg, path, time)
                    )
                    .unwrap()
                        >= 0.99,
                    "{property} parity at {time}: {} for {}",
                    structural_similarity(
                        &pixels,
                        &grids::decode_rgb_frame(&tools.ffmpeg, path, time)
                    )
                    .unwrap(),
                    path.display()
                );
            }
            let draft_frame = renderer.render_preview(&materialized, &dir, time).unwrap();
            assert_eq!(
                pixels,
                grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(draft_frame.relative_path), 0)
            );
            let visible: Vec<_> = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, rgb)| rgb[0] > 8)
                .map(|(i, _)| (i % WIDTH as usize) as i32)
                .collect();
            assert!(!visible.is_empty());
            let left = *visible.iter().min().unwrap();
            let right = *visible.iter().max().unwrap();
            let brightness: u64 = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .map(|rgb| u64::from(rgb[0]))
                .sum();
            metrics.push((left, right - left + 1, brightness));
        }
        for pair in metrics.windows(2) {
            match property {
                "position" => assert!(
                    (pair[1].0 - pair[0].0 - 16).abs() <= 1,
                    "position must advance 16 pixels"
                ),
                "scale" => assert!(pair[1].1 > pair[0].1, "visible text bounds must grow"),
                _ => assert!(pair[1].2 > pair[0].2, "text brightness must increase"),
            }
        }
        assert_eq!(fs::read(dir.join("project.json")).unwrap(), authoritative);
    }
}
