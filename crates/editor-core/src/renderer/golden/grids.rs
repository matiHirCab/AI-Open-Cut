//! Native output parity for grids; lattice and paint oracles live beside their owners.
use super::*;
use serde_json::json;

pub(super) fn decode_rgb_frame(ffmpeg: &Path, path: &Path, time_ms: u64) -> Vec<u8> {
    let output = std::process::Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{}", time_ms as f64 / 1000.0))
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
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn fixture() -> Project {
    let mut p = fixture_project();
    p.settings.width = 240;
    p.settings.height = 120;
    p.tracks[0].items.clear();
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../contracts/procedural-grids-v1.json"
    ))
    .unwrap();
    for i in 0..4 {
        let mut grid = catalog["valid"][i]["grid"].clone();
        grid["width"] = json!(40);
        grid["height"] = json!(40);
        let item = json!({"type":"grid","id":format!("grid-{i}"),"grid":grid,"startMs":0,"durationMs":1000,"keyframes":[],"stackOrder":i,"transform":{"positionX":10+i*55,"positionY":10,"scale":1,"opacity":1}});
        p.tracks[0]
            .items
            .push(serde_json::from_value(item).unwrap());
    }
    let mut local = serde_json::to_value(&p.tracks[0].items[2]).unwrap();
    local["id"] = json!("local-grid");
    local["stackOrder"] = json!(0);
    local["durationMs"] = json!(2000);
    local["transform"] = json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
    local["grid"] = catalog["valid"][6]["grid"].clone();
    p.components.push(serde_json::from_value(json!({"id":"grid-component","name":"Grid component","width":40,"height":40,"durationMs":2000,"tracks":[{"id":"local-track","name":"Grids","trackType":"overlay","items":[local]}],"slots":[]})).unwrap());
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"component_instance","id":"grid-instance","componentId":"grid-component","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":2,"stackOrder":4,"transform":{"positionX":60,"positionY":70,"scale":1,"opacity":1}})).unwrap());
    let mut edge = catalog["valid"][0]["grid"].clone();
    edge["width"] = json!(10.5);
    edge["height"] = json!(10);
    edge["pattern"]["spacingY"] = json!(20);
    edge["pattern"]["stroke"]["width"] = json!(1);
    edge["pattern"]["stroke"]["paint"]["color"] = json!({"r":1,"g":1,"b":1,"a":1});
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"grid","id":"fractional-grid","grid":edge,"startMs":0,"durationMs":1000,"keyframes":[],"stackOrder":5,"transform":{"positionX":180,"positionY":70,"scale":1,"opacity":1}})).unwrap());
    p
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
        .create_project("Grid conformance", p.settings.clone())
        .unwrap()
        .project_id;
    p.id = id.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    fs::create_dir_all(dir.join("assets")).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    fs::write(dir.join("project.json"), serde_json::to_vec(&p).unwrap()).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let draft = core
        .create_draft(
            &id,
            p.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"set_item_visibility","itemId":"grid-0","hidden":false}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
    let before = serde_json::to_vec(&p).unwrap();
    for (w, h) in [(240, 120), (160, 90)] {
        let scene = evaluate_project(&p, w, h, FPS).unwrap().scene;
        assert_eq!(scene, evaluate_project(&p, w, h, FPS).unwrap().scene);
        assert_eq!(
            scene,
            evaluate_project(&materialized, w, h, FPS).unwrap().scene
        );
        let range = renderer
            .render_preview_range(
                &p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: w,
                    height: h,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let range = dir.join(range.relative_path);
        let output = root.path().join(format!("grid-{w}.mp4"));
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &output,
                    width: w,
                    height: h,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let rgb = decode_rgb_frame(&tools.ffmpeg, &range, 500);
        let similarity =
            structural_similarity(&rgb, &decode_rgb_frame(&tools.ffmpeg, &output, 500)).unwrap();
        assert!(
            similarity >= 0.99,
            "grid range/export {w}x{h}: SSIM {similarity}"
        );
        let a = decode_mono_f32(&tools.ffmpeg, &range);
        let b = decode_mono_f32(&tools.ffmpeg, &output);
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
                .probe(&range)
                .unwrap()
                .duration_ms
                .unwrap()
                .abs_diff(1000)
                <= 1000 / u64::from(FPS)
        );
        if w == 240 {
            let frame = renderer.render_preview(&p, &dir, 500).unwrap();
            let frame_rgb = decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
            assert!(structural_similarity(&frame_rgb, &rgb).unwrap() >= 0.99);
            let frame = renderer.render_preview(&materialized, &dir, 500).unwrap();
            assert_eq!(
                frame_rgb,
                decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0)
            );
            // Interior of the rectangular lattice cell is transparent over black.
            assert_eq!(
                &frame_rgb[(15 * 240 + 15) * 3..(15 * 240 + 15) * 3 + 3],
                &[0, 0, 0]
            );
            assert!(frame_rgb[(20 * 240 + 20) * 3] > 70);
            for channel in &frame_rgb[(75 * 240 + 190) * 3..(75 * 240 + 190) * 3 + 3] {
                assert!(
                    channel.abs_diff(128) <= 1,
                    "fractional grid edge: {channel}"
                );
            }
            assert_eq!(
                &frame_rgb[(75 * 240 + 191) * 3..(75 * 240 + 191) * 3 + 3],
                &[0, 0, 0]
            );
        }
    }
    assert_eq!(serde_json::to_vec(&p).unwrap(), before);
    // Descriptor fits the 4096 mark limit, but expanded rounded coverage does
    // not fit the grid budget. Failure must precede export collision handling.
    let mut excessive = p.clone();
    if let TimelineItem::Grid(grid) = &mut excessive.tracks[0].items[0] {
        let mut value = serde_json::to_value(&grid.grid).unwrap();
        value["width"] = json!(4094);
        value["height"] = json!(1);
        value["pattern"]["spacingX"] = json!(1);
        value["pattern"]["spacingY"] = json!(16384);
        value["pattern"]["stroke"]["width"] = json!(2000);
        value["pattern"]["stroke"]["lineCap"] = json!("round");
        grid.grid = serde_json::from_value(value).unwrap();
        crate::validation::grid::validate_grid(&grid.grid).unwrap();
    }
    let collision = root.path().join("untouched.mp4");
    fs::write(&collision, b"untouched").unwrap();
    let project_bytes = fs::read(dir.join("project.json")).unwrap();
    let history_bytes = fs::read(dir.join("history.json")).unwrap();
    let error = renderer
        .export_video(
            &excessive,
            &dir,
            ExportOptions {
                output: &collision,
                width: 240,
                height: 120,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidArgument);
    assert_eq!(fs::read(collision).unwrap(), b"untouched");
    assert_eq!(fs::read(dir.join("project.json")).unwrap(), project_bytes);
    assert_eq!(fs::read(dir.join("history.json")).unwrap(), history_bytes);
    // Full hidden/unreachable occurrence preflight uses composed magnification.
    let mut invalid = p.clone();
    invalid.components[0].tracks[0].hidden = true;
    invalid.components[0].tracks[0].items[0]
        .visual_properties_mut()
        .transform
        .scale = 10000.;
    assert_eq!(
        evaluate_project(&invalid, 240, 120, FPS).unwrap_err().code,
        crate::ErrorCode::InvalidArgument
    );
    let mut animated = p.clone();
    if let TimelineItem::Grid(grid) = &mut animated.components[0].tracks[0].items[0] {
        grid.keyframes=serde_json::from_value(json!([{"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}])).unwrap();
    }
    let static_frame = renderer.render_preview(&p, &dir, 500).unwrap();
    let expected = decode_rgb_frame(&tools.ffmpeg, &dir.join(static_frame.relative_path), 0);
    let animated_frame = renderer.render_preview(&animated, &dir, 500).unwrap();
    assert_eq!(
        expected,
        decode_rgb_frame(&tools.ffmpeg, &dir.join(animated_frame.relative_path), 0)
    );
}

#[test]
fn native_grid_render_conformance() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}
