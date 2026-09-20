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
    logical_box_pixel_conformance(tools);
    paint_regression_conformance(tools);
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

    for mode in 0..7 {
        let styled = mode > 0;
        if styled {
            let TimelineItem::Text(text) = &mut project.tracks[0].items[1] else {
                unreachable!()
            };
            text.document = serde_json::from_value(json!({"runs":[{"text":"café →\n", "color":"#ff2200"},{"text":"WWWW iiii","color":"#22ff00"}]})).unwrap();
            text.text = text.document.text();
            if mode == 2 {
                text.style.paint_layers = Some(serde_json::from_value(json!([
                    {"kind":"shadow","color":"#000000","opacity":0.7,"offsetXPx":-2.5,"offsetYPx":3,"blurSigmaPx":2},
                    {"kind":"shadow","color":"#112244","opacity":0.4,"offsetXPx":2,"offsetYPx":-1,"blurSigmaPx":0},
                    {"kind":"stroke","color":"#ffffff","opacity":0.8,"widthPx":6},
                    {"kind":"stroke","color":"#0000ff","opacity":1,"widthPx":3.5},
                    {"kind":"fill","color":"#22ff00","opacity":1}
                ])).unwrap());
                text.document.spans = Some(serde_json::from_value(json!([{"start":0,"end":4,"style":{"paintLayers":[
                    {"kind":"shadow","color":"#000000","opacity":0.5,"offsetXPx":2,"offsetYPx":-1,"blurSigmaPx":1.5},
                    {"kind":"stroke","color":"#ffffff","opacity":1,"widthPx":1.5},
                    {"kind":"fill","color":"#ff2200","opacity":1}
                ]}}])).unwrap());
            }
        }
        if mode == 2 {
            let mut title = serde_json::to_value(&project.tracks[0].items[1]).unwrap();
            title["id"] = json!("component-title");
            title["stackOrder"] = json!(0);
            let document = title["document"].clone();
            project.components.push(serde_json::from_value(json!({
                "id":"styled-card","name":"Styled card","width":WIDTH,"height":HEIGHT,
                "durationMs":DURATION_MS,
                "slots":[{"id":"title","name":"Title","kind":"rich_text","required":true,
                    "binding":{"targetLayerId":"component-title","property":"text.document"},"constraints":{}}],
                "tracks":[{"id":"card-overlay","name":"Card","trackType":"overlay","items":[title]}]
            })).unwrap());
            project.tracks[0].items.push(serde_json::from_value(json!({
                "type":"component_instance","id":"styled-instance","componentId":"styled-card",
                "startMs":0,"durationMs":DURATION_MS,"trimStartMs":0,"timeScale":1,"stackOrder":2,
                "slotValues":{"title":{"type":"rich_text","value":document}},
                "transform":{"positionX":20,"positionY":20,"scale":0.5,"opacity":0.8}
            })).unwrap());
            project.tracks[0].items.push(serde_json::from_value(json!({
                "type":"repeater","id":"styled-copies","startMs":0,"durationMs":DURATION_MS,"stackOrder":3,
                "repeater":{"source":{"scope":"root","id":"styled-instance"},"copies":1,
                    "transformOffset":{"position":{"x":40,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
                        "rotationDeg":10,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.1}
            })).unwrap());
        }
        if mode >= 3 {
            let layout = serde_json::from_value::<Box<crate::TextLayout>>(json!({
                "trackingPx":0.5,"lineHeightPx":30.5,"bounds":{"widthPx":220.5,"heightPx":100.5},
                "wrap":"word","verticalAlignment":"center","fit":(["none","shrink","fit_width","fit_box"][mode-3]),
                "backgroundCornerRadiusPx":8.5
            })).unwrap();
            for item in project
                .tracks
                .iter_mut()
                .chain(project.components.iter_mut().flat_map(|c| &mut c.tracks))
                .flat_map(|t| &mut t.items)
            {
                if let TimelineItem::Text(text) = item {
                    text.style.layout = Some(layout.clone());
                }
            }
        }
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&project).unwrap(),
        )
        .unwrap();
        let draft = core.create_draft(&id, project.revision, vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":"animated-text","hidden":false})).unwrap()], None).unwrap();
        project = core.get_project(&id).unwrap();
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
        let output = root.path().join(format!("rich-text-{mode}.mp4"));
        let export = renderer
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
        assert_eq!(export.text_layouts, range.text_layouts);
        if mode >= 3 {
            assert!(range.text_layouts.len() >= 3);
        }
        let rgb = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(&range.relative_path), 500);
        let similarity =
            structural_similarity(&rgb, &grids::decode_rgb_frame(&tools.ffmpeg, &output, 500))
                .unwrap();
        assert!(
            similarity >= 0.99,
            "range/export SSIM for mode {mode}: {similarity}"
        );
        let frame = renderer.render_preview(&project, &dir, 500).unwrap();
        assert_eq!(frame.text_layouts, range.text_layouts);
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
        let frame_similarity = structural_similarity(&pixels, &rgb).unwrap();
        assert!(
            frame_similarity >= 0.99,
            "frame/range SSIM mode {mode}: {frame_similarity}"
        );
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
    // Bound styles survive a renderer default with missing styled siblings.
    missing.render_preview(&project, &dir, 500).unwrap();
}

fn paint_regression_conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    for explicit in [false, true] {
        let mut project = fixture_project();
        project.settings.width = 400;
        project.settings.height = 220;
        project.assets.clear();
        project.tracks.truncate(1);
        project.tracks[0]
            .items
            .retain(|item| matches!(item, TimelineItem::Text(_)));
        let TimelineItem::Text(text) = &mut project.tracks[0].items[0] else {
            unreachable!()
        };
        text.text = if explicit { "AV" } else { "AV " }.into();
        text.document = crate::RichTextDocument::plain(text.text.clone());
        text.font_size = 110;
        text.keyframes.clear();
        text.visual_properties = crate::VisualProperties::new(
            Transform {
                position_x: 10.0,
                position_y: 10.0,
                ..Transform::default()
            },
            false,
        );
        text.style = TextStyle::default();
        if explicit {
            text.style.paint_layers = Some(
                serde_json::from_value(json!([
                    {"kind":"stroke","color":"#ff0000","opacity":1,"widthPx":60},
                    {"kind":"fill","color":"#ffffff","opacity":1}
                ]))
                .unwrap(),
            );
        } else {
            text.style.outline_width_px = 8;
            text.style.outline_color = "#ff0000".into();
            text.style.shadow.color = "#00ff00".into();
            text.style.shadow.opacity = 0.5;
            text.style.shadow.offset_x = 3;
            text.style.shadow.offset_y = 3;
        }
        let frame = renderer.render_preview(&project, root.path(), 0).unwrap();
        let before =
            grids::decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0);
        let TimelineItem::Text(text) = &mut project.tracks[0].items[0] else {
            unreachable!()
        };
        text.document.spans = Some(
            serde_json::from_value(if explicit {
                json!([{"start":1,"end":2,"style":{"color":"#00ff00"}}])
            } else {
                json!([{"start":2,"end":3,"style":{"paintLayers":[]}}])
            })
            .unwrap(),
        );
        let frame = renderer.render_preview(&project, root.path(), 0).unwrap();
        let after =
            grids::decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0);
        assert!(
            before == after,
            "native paint regression: explicit={explicit}"
        );
    }
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
    for advanced in [false, true] {
        for property in ["position", "scale", "opacity"] {
            let text = animation_text(&mut project);
            if advanced {
                text.style.layout = Some(Box::new(crate::TextLayout {
                    bounds: Some(crate::TextBounds {
                        width_px: Some(40.5),
                        height_px: Some(40.5),
                    }),
                    ..Default::default()
                }));
                text.style.shadow.opacity = 1.0;
                text.style.shadow.offset_x = -20;
                text.style.outline_width_px = 1;
            }
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
            let output = root
                .path()
                .join(format!("animation-{advanced}-{property}.mp4"));
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
                let pixels =
                    grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
                // Independent oracle: explicit linear values, not evaluator helpers.
                let mut oracle = project.clone();
                let text = animation_text(&mut oracle);
                text.keyframes.clear();
                let fraction = time as f64 / 1000.0;
                match property {
                    "position" => {
                        text.visual_properties.transform.position_x = 40.0 + 40.0 * fraction
                    }
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
}

fn logical_box_pixel_conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Logical box",
            crate::ProjectSettings {
                width: 320,
                height: 240,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let item = core.edit(&id,0,serde_json::from_value(json!({"operation":"add_text","trackId":track,"text":"M","fontSize":30,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":50,"positionY":40,"scale":1,"opacity":1},"style":{"backgroundColor":"#ff0000","backgroundOpacity":1,"layout":{"bounds":{"widthPx":100,"heightPx":70},"wrap":"none"}}})).unwrap()).unwrap().changed_ids[0].clone();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let dir = core.paths().project_dir(&id).unwrap();
    for effect in [false, true] {
        if effect {
            core.edit(&id,1,serde_json::from_value(json!({"operation":"update_item","itemId":item,"style":{"backgroundColor":"#ff0000","backgroundOpacity":1,"layout":{"bounds":{"widthPx":100,"heightPx":70},"wrap":"none"},"paintLayers":[{"kind":"shadow","color":"#000000","opacity":1,"offsetXPx":-20,"offsetYPx":0,"blurSigmaPx":0},{"kind":"fill","color":"#ffffff","opacity":1}]}})).unwrap()).unwrap();
        }
        let project = core.get_project(&id).unwrap();
        let frame = renderer.render_preview(&project, &dir, 0).unwrap();
        let pixels = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
        let red: Vec<_> = pixels
            .chunks_exact(3)
            .enumerate()
            .filter(|(_, p)| p[0] > 240 && p[1] < 10 && p[2] < 10)
            .map(|(i, _)| (i % 320, i / 320))
            .collect();
        assert_eq!(
            (
                red.iter().map(|p| p.0).min(),
                red.iter().map(|p| p.1).min(),
                red.iter().map(|p| p.0).max(),
                red.iter().map(|p| p.1).max()
            ),
            (Some(50), Some(40), Some(149), Some(109)),
            "effect={effect}"
        );
    }
    core.edit(&id, 2, serde_json::from_value(json!({"operation":"update_item","itemId":item,"text":"\n","style":{"layout":{}},"transform2d":crate::Transform2D::default()})).unwrap()).unwrap();
    let blank = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 0)
        .unwrap();
    assert!(
        grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(blank.relative_path), 0)
            .iter()
            .all(|byte| *byte == 0)
    );
}
