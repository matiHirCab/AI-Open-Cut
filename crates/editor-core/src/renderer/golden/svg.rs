//! Independent integer-coordinate SVG oracle across native output intents and history.
use super::*;
use serde_json::json;

fn fixture() -> Project {
    let mut p = fixture_project();
    p.settings.width = 240;
    p.settings.height = 120;
    p.tracks[0].items.clear();
    let translucent =
        "<rect x=\"0\" y=\"0\" width=\"1000\" height=\"1000\" fill=\"#ff000001\"/>".repeat(500);
    let source = "<svg width=\"80\" height=\"20\" viewBox=\"0 0 10000 10000\"><rect width=\"10000\" height=\"10000\" fill=\"#f00\"/><path d=\"M2000 0 L4000 0 L4000 10000 L2000 10000 Z H4000 V10000 H2000 Z\" fill=\"#00f\"/></svg>";
    let document = crate::validation::svg::parse(source).unwrap();
    let alpha_doc = crate::validation::svg::parse(&format!(
        "<svg width=\"20\" height=\"20\" viewBox=\"0 0 1000 1000\">{translucent}</svg>"
    ))
    .unwrap();

    let item:TimelineItem=serde_json::from_value(json!({"type":"svg","id":"svg-root","document":document,"startMs":0,"durationMs":1000,"keyframes":[],"transform":{"positionX":10,"positionY":10,"scale":1,"opacity":1}})).unwrap();
    p.tracks[0].items.push(item.clone());
    let mut local = serde_json::to_value(&item).unwrap();
    local["id"] = json!("svg-local");
    local["durationMs"] = json!(2000);
    local["transform"] = serde_json::to_value(Transform {
        scale: 100.,
        ..Transform::default()
    })
    .unwrap();
    p.components.push(serde_json::from_value(json!({"id":"icon","name":"Icon","width":80,"height":20,"durationMs":2000,"tracks":[{"id":"local","name":"SVG","trackType":"overlay","items":[local]}],"slots":[]})).unwrap());
    let mut t = crate::Transform2D {
        scale_x: 0.01,
        scale_y: 0.01,
        ..Default::default()
    };
    t.position.x = 100.;
    t.position.y = 40.;
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"component_instance","id":"instance","componentId":"icon","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":2,"stackOrder":1,"transform2d":t})).unwrap());
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"svg","id":"svg-alpha","document":alpha_doc,"startMs":0,"durationMs":1000,"stackOrder":2,"keyframes":[],"transform":{"positionX":180,"positionY":70,"scale":1,"opacity":0.5}})).unwrap());
    p
}
fn oracle(moved: bool) -> Vec<u8> {
    let mut rgb = vec![0; 240 * 120 * 3];
    for (left, top) in [(40 + if moved { 10 } else { 0 }, 10), (130, 40)] {
        for y in top..top + 20 {
            for x in left..left + 20 {
                let i = (y * 240 + x) * 3;
                rgb[i + if (left + 4..left + 8).contains(&x) {
                    2
                } else {
                    0
                }] = 255;
            }
        }
    }
    // PAM alpha is rounded once; existing FFmpeg layer opacity then applies once.
    let alpha = ((1. - (254_f64 / 255.).powi(500)) * 255.).round();
    let red = (alpha * 0.5).floor() as u8;
    for y in 70..90 {
        for x in 180..200 {
            rgb[(y * 240 + x) * 3] = red;
        }
    }
    rgb
}
pub(super) fn conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut p = fixture();
    let id = core
        .create_project("SVG conformance", p.settings.clone())
        .unwrap()
        .project_id;
    p.id = id.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    fs::write(dir.join("project.json"), serde_json::to_vec(&p).unwrap()).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut revision = p.revision;
    for (step, moved) in [(0, false), (1, true), (2, false), (3, true), (4, true)] {
        if step == 1 {
            revision=core.edit(&id,revision,serde_json::from_value(json!({"operation":"update_item","itemId":"svg-root","transform":{"positionX":20,"positionY":10,"scale":1,"opacity":1}})).unwrap()).unwrap().revision;
        }
        if step == 2 {
            revision = core.undo(&id, revision).unwrap().revision;
        }
        if step == 3 {
            revision = core.redo(&id, revision).unwrap().revision;
        }
        let reopened = crate::EditorCore::new(core.paths().clone());
        let p = reopened.get_project(&id).unwrap();
        let before = serde_json::to_vec(&p).unwrap();
        let draft=core.create_draft(&id,revision,vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":"svg-root","hidden":false})).unwrap()],None).unwrap();
        let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
        assert_eq!(
            evaluate_project(&p, 240, 120, FPS).unwrap().scene,
            evaluate_project(&materialized, 240, 120, FPS)
                .unwrap()
                .scene
        );
        let draft_frame = renderer.render_preview(&materialized, &dir, 500).unwrap();
        assert!(
            structural_similarity(
                &oracle(moved),
                &shapes::decode_rgb_frame(&tools.ffmpeg, &dir.join(draft_frame.relative_path), 0)
            )
            .unwrap()
                >= 0.99
        );
        core.discard_draft(&id, &draft.id).unwrap();

        let scene = evaluate_project(&p, 240, 120, FPS).unwrap().scene;
        assert_eq!(scene, evaluate_project(&p, 240, 120, FPS).unwrap().scene);
        let frame = renderer.render_preview(&p, &dir, 500).unwrap();
        let rgb = shapes::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
        assert!(
            structural_similarity(&oracle(moved), &rgb).unwrap() >= 0.99,
            "independent SVG oracle, step {step}"
        );
        let range = renderer
            .render_preview_range(
                &p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 240,
                    height: 120,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let range = dir.join(range.relative_path);
        let output = root.path().join(format!("svg-{step}.mp4"));
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 240,
                    height: 120,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for path in [&range, &output] {
            assert!(
                structural_similarity(&rgb, &shapes::decode_rgb_frame(&tools.ffmpeg, path, 500))
                    .unwrap()
                    >= 0.99
            );
        }
        let a = decode_mono_f32(&tools.ffmpeg, &range);
        let b = decode_mono_f32(&tools.ffmpeg, &output);
        assert!(!a.is_empty());
        assert!(a.iter().any(|v| v.abs() > 0.001));
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
                .probe(&range)
                .unwrap()
                .duration_ms
                .unwrap()
                .abs_diff(renderer.probe(&output).unwrap().duration_ms.unwrap())
                <= 1000 / u64::from(FPS)
        );
        assert_eq!(
            before,
            serde_json::to_vec(&reopened.get_project(&id).unwrap()).unwrap()
        );
    }
}

#[test]
fn svg_hidden_and_unused_content_cannot_bypass_mapped_limits() {
    let mut p = fixture();
    let TimelineItem::Svg(item) = &mut p.tracks[0].items[0] else {
        panic!()
    };
    item.hidden = true;
    item.document.width = 5000.;
    item.document.height = 5000.;
    assert!(
        evaluate_project(&p, 240, 120, FPS)
            .unwrap_err()
            .message
            .contains("SVG raster bounds")
    );
    let mut p = fixture();
    p.tracks[0]
        .items
        .retain(|i| !matches!(i, TimelineItem::ComponentInstance(_)));
    let TimelineItem::Svg(item) = &mut p.components[0].tracks[0].items[0] else {
        panic!()
    };
    item.document.width = 5000.;
    item.document.height = 5000.;
    assert!(
        evaluate_project(&p, 240, 120, FPS)
            .unwrap_err()
            .message
            .contains("SVG raster bounds")
    );
}

#[test]
fn svg_invalid_mapping_publishes_no_artifacts() {
    for source in [
        "<svg width=\"100\" height=\"100\" viewBox=\"0 0 0.0001 0.0001\"><rect x=\"-5000\" y=\"-5000\" width=\"10000\" height=\"10000\"/></svg>",
        "<svg width=\"100\" height=\"100\" viewBox=\"0 0 .001 .001\"><polygon points=\"-5000,-5000 5000,5000 5000,5000.0001 -5000,-4999.9999\" fill=\"#f00\"/></svg>",
    ] {
        assert_svg_invalid_mapping(source);
    }
}

fn assert_svg_invalid_mapping(source: &str) {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let created = core
        .create_project(
            "Invalid SVG",
            crate::ProjectSettings {
                width: 100,
                height: 100,
                fps: FPS,
            },
        )
        .unwrap();
    let p = core.get_project(&created.project_id).unwrap();
    let track = p
        .tracks
        .iter()
        .find(|t| t.track_type == crate::TrackType::Overlay)
        .unwrap();
    let edit = serde_json::from_value(json!({"operation":"add_svg","trackId":track.id,"startMs":0,"durationMs":1000,"svg":source})).unwrap();
    core.edit(&p.id, p.revision, edit).unwrap();
    let p = core.get_project(&p.id).unwrap();
    let dir = core.paths().project_dir(&p.id).unwrap();
    fn snapshot(dir: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        let mut result = std::collections::BTreeMap::new();
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                result.extend(snapshot(&path));
            } else {
                result.insert(path.clone(), fs::read(path).unwrap());
            }
        }
        result
    }
    let before = snapshot(root.path());
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let output = root.path().join("invalid.mp4");
    for error in [
        renderer.render_preview(&p, &dir, 0).unwrap_err(),
        renderer
            .render_preview_range(
                &p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 100,
                    height: 100,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap_err(),
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 100,
                    height: 100,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap_err(),
    ] {
        assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
        assert!(!error.retryable);
    }
    assert_eq!(snapshot(root.path()), before);
    let svg_id = p
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, TimelineItem::Svg(_)))
        .unwrap()
        .id();
    let draft = core
        .create_draft(
            &p.id,
            p.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"set_item_visibility","itemId":svg_id,"hidden":false}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&p.id, &draft.id).unwrap().project;
    let before_draft = snapshot(root.path());
    assert_eq!(
        renderer
            .render_preview(&materialized, &dir, 0)
            .unwrap_err()
            .code,
        crate::ErrorCode::InvalidArgument
    );
    assert_eq!(snapshot(root.path()), before_draft);
    assert_eq!(
        serde_json::to_value(core.get_project(&p.id).unwrap()).unwrap(),
        serde_json::to_value(p).unwrap()
    );
}

#[test]
fn svg_precision_control_native() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    let mut p = fixture();
    p.components.clear();
    p.assets.clear();
    p.tracks.truncate(1);
    p.tracks[0].items.truncate(1);
    let TimelineItem::Svg(item) = &mut p.tracks[0].items[0] else {
        panic!()
    };
    item.transform = Transform::default();
    item.document = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 .001 .001\"><polygon points=\"-.005,-.005 .005,.005 .005,.0051 -.005,-.0049\" fill=\"#f00\"/></svg>").unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let preview = renderer.render_preview(&p, root.path(), 0).unwrap();
    let rgb = shapes::decode_rgb_frame(&tools.ffmpeg, &root.path().join(preview.relative_path), 0);
    for (y, expected) in [(45, [0_u8; 3]), (55, [255, 0, 0]), (65, [0; 3])] {
        let offset = (y * 240 + 50) * 3;
        assert!(
            rgb[offset..offset + 3]
                .iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(b) <= 2)
        );
    }
}

#[test]
fn svg_native_conformance() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}
