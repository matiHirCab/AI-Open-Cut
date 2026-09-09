//! Native preview/range/export parity for lazily evaluated repeater occurrences.
use super::*;
use crate::evaluated_scene::EvaluatedLayerOrder;
use serde_json::json;

fn fixture() -> Project {
    let mut project = fixture_project();
    project.schema_version = PROJECT_SCHEMA_VERSION;
    project.assets.clear();
    project.components.clear();
    project.tracks.truncate(1);
    project.tracks[0].items = serde_json::from_value(json!([
        {"type":"shape","id":"source","geometry":{"type":"rectangle","width":20,"height":20},
         "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
         "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0,
         "transform2d":{"position":{"x":10,"y":20,"unit":"pixels"},"anchor":{"x":0,"y":0},
         "scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},
        {"type":"repeater","id":"copies","startMs":100,"durationMs":800,"zIndex":0,"stackOrder":1,
         "repeater":{"source":{"scope":"root","id":"source"},"copies":3,
         "transformOffset":{"position":{"x":30,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
         "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.2}}
    ]))
    .unwrap();
    project.tracks[0]
        .items
        .extend(serde_json::from_value::<Vec<TimelineItem>>(json!([
            {"type":"group","id":"group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2,
             "transform2d":{"position":{"x":10,"y":45,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},
            {"type":"shape","id":"group-shape","geometry":{"type":"ellipse","width":16,"height":16},
             "fill":{"type":"solid","color":{"r":0,"g":1,"b":0,"a":1}},"stroke":null,
             "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":3,"parent":{"scope":"root","id":"group"}},
            {"type":"repeater","id":"group-copies","startMs":200,"durationMs":600,"zIndex":0,"stackOrder":4,
             "repeater":{"source":{"scope":"root","id":"group"},"copies":2,
             "transformOffset":{"position":{"x":25,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}},
            {"type":"component_instance","id":"component-source","componentId":"dot","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":5,
             "parent":{"scope":"root","id":"group"}},
            {"type":"repeater","id":"component-copies","startMs":100,"durationMs":800,"zIndex":0,"stackOrder":6,
             "repeater":{"source":{"scope":"root","id":"component-source"},"copies":2,
             "transformOffset":{"position":{"x":25,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":-0.25}}
        ]))
        .unwrap());
    project.components.push(serde_json::from_value(json!({
        "id":"dot","name":"Dot","width":160,"height":90,"durationMs":1000,"slots":[],
        "tracks":[{"id":"dot-track","name":"Dot","trackType":"overlay","items":[
            {"type":"shape","id":"dot-shape","geometry":{"type":"ellipse","width":12,"height":12},
             "fill":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"stroke":null,
             "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":0,
             "transform2d":{"position":{"x":10,"y":10,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}
        ]}]
    })).unwrap());
    for index in 1..=10 {
        project.components[0].tracks[0]
            .items
            .push(serde_json::from_value(json!({
                "type":"shape","id":format!("dot-shape-{index}"),
                "geometry":{"type":"ellipse","width":12,"height":12},
                "fill":{"type":"solid","color":{
                    "r":0,"g":if index == 10 { 1 } else { 0 },"b":if index == 10 { 0 } else { 1 },"a":1
                }},"stroke":null,"startMs":0,"durationMs":1000,"keyframes":[],
                "zIndex":0,"stackOrder":index,
                "transform2d":{"position":{"x":10,"y":10,"unit":"pixels"},"anchor":{"x":0,"y":0},
                "scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}
            })).unwrap());
    }
    project
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
    let mut project = fixture();
    let id = core
        .create_project("Repeater conformance", project.settings.clone())
        .unwrap()
        .project_id;
    project.id = id.clone();
    let directory = core.paths().project_dir(&id).unwrap();
    fs::write(
        directory.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let draft = core
        .create_draft(
            &id,
            project.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"set_item_visibility","itemId":"copies","hidden":false}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
    let normalize_internal_order = |mut scene: EvaluatedScene| {
        for (index, layer) in scene.visual_layers.iter_mut().enumerate() {
            layer.order = EvaluatedLayerOrder {
                track_index: index,
                item_index: 0,
            };
        }
        scene
    };
    assert_eq!(
        normalize_internal_order(
            evaluate_project(&project, WIDTH, HEIGHT, FPS)
                .unwrap()
                .scene
        ),
        normalize_internal_order(
            evaluate_project(&materialized, WIDTH, HEIGHT, FPS)
                .unwrap()
                .scene
        )
    );
    let range = renderer
        .render_preview_range(
            &project,
            &directory,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: WIDTH,
                height: HEIGHT,
                fps: FPS,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let range = directory.join(range.relative_path);
    let output = root.path().join("repeaters.mp4");
    renderer
        .export_video(
            &project,
            &directory,
            ExportOptions {
                output: &output,
                width: WIDTH,
                height: HEIGHT,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let range_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &range, 500);
    let export_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &output, 500);
    assert!(structural_similarity(&range_rgb, &export_rgb).unwrap() >= SSIM_MINIMUM);
    let topmost_component_pixel =
        &range_rgb[(61 * WIDTH as usize + 26) * 3..(61 * WIDTH as usize + 26) * 3 + 3];
    assert!(
        topmost_component_pixel[0] < 20
            && topmost_component_pixel[1] > 220
            && topmost_component_pixel[2] < 20,
        "the eleventh equal-z component sibling must remain topmost: {topmost_component_pixel:?}"
    );
    let frame = renderer
        .render_preview(&materialized, &directory, 500)
        .unwrap();
    let frame_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &directory.join(frame.relative_path), 0);
    assert!(structural_similarity(&range_rgb, &frame_rgb).unwrap() >= SSIM_MINIMUM);
}

#[test]
fn native_repeater_render_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    conformance(&tools);
}
