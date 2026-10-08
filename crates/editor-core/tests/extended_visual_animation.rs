use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, MediaProbeFacts, MediaType, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};

// Limit both filter inputs: FFmpeg 6 may evaluate frames ahead of the output limit.
const SINGLE_FRAME_SSIM: &str = "[0:v]trim=end_frame=1,setpts=PTS-STARTPTS[reference];[1:v]trim=end_frame=1,setpts=PTS-STARTPTS[actual];[reference][actual]ssim";

fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let policy = PathPolicy::new(
        root.path().join("projects"),
        [&media],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(policy);
    let id = core
        .create_project("Extended visuals", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

#[test]
fn review_effects_only_scale_spring_rejects_before_publication() {
    for hidden in [false, true] {
        let (_root, core, id, track) = setup();
        let mut rectangle = authored_rectangle(&track);
        rectangle["width"] = json!(1000);
        rectangle["height"] = json!(1000);
        let item = core.edit(&id, 0, op(rectangle)).unwrap().changed_ids[0].clone();
        core.edit(&id, 1, op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"identity","type":"vignette","amount":0}]}))).unwrap();
        let mut revision = 2;
        if hidden {
            core.edit(
                &id,
                revision,
                op(json!({"operation":"update_track","trackId":track,"hidden":true})),
            )
            .unwrap();
            revision += 1;
        }
        let channels: Vec<_> = ["transform.scale_x", "transform.scale_y"].into_iter().map(|property|
            scalar_channel(property, 1.0, 3.0, json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0}))
        ).collect();
        let before = authoritative_files(&core, &id);
        let edit = op(
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}),
        );
        assert_eq!(
            core.edit(&id, revision, edit.clone()).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
        assert_eq!(
            core.create_draft(&id, revision, vec![edit], None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
    }
}

#[test]
fn review_effects_only_safe_scale_remains_accepted() {
    for curve in [
        json!("linear"),
        json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0}),
    ] {
        let (_root, core, id, track) = setup();
        let mut rectangle = authored_rectangle(&track);
        rectangle["width"] = json!(1000);
        rectangle["height"] = json!(1000);
        let item = core.edit(&id, 0, op(rectangle)).unwrap().changed_ids[0].clone();
        core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"identity","type":"vignette","amount":0}]}))).unwrap();
        let end = if curve.is_string() { 3.0 } else { 1.0 };
        let channels: Vec<_> = ["transform.scale_x", "transform.scale_y"]
            .into_iter()
            .map(|p| scalar_channel(p, 1.0, end, curve.clone()))
            .collect();
        assert_eq!(core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap().revision,3);
    }
}

#[test]
fn review_unused_component_scale_is_certified_without_rotation() {
    let (_root, core, id, _track) = setup();
    let component=core.edit(&id,0,op(json!({"operation":"component_create","name":"Retained","width":64,"height":64,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    let channels: Vec<_> = ["transform.scale_x", "transform.scale_y"]
        .into_iter()
        .map(|p| {
            scalar_channel(
                p,
                1.0,
                3.0,
                json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0}),
            )
        })
        .collect();
    let before = authoritative_files(&core, &id);
    let candidate = json!({"operation":"component_update","componentId":component,"name":"Retained","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"id":"rect","type":"rectangle","color":"#ff0000","width":1000,"height":1000,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"effects":[{"id":"identity","type":"vignette","amount":0}],"animationChannels":channels,"keyframes":[]}]}]});
    assert_eq!(
        core.edit(&id, 1, op(candidate)).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn native_review_identity_effect_preserves_offset_path_in_every_intent() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[
        {"type":"moveTo","to":{"x":10,"y":20}},{"type":"lineTo","to":{"x":30,"y":20}},
        {"type":"lineTo","to":{"x":30,"y":40}},{"type":"lineTo","to":{"x":10,"y":40}},{"type":"close"}
    ]}},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}))).unwrap().changed_ids[0].clone();
    core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let draft=core.create_draft(&id,2,vec![op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"identity","type":"vignette","amount":0}]}))],None).unwrap();
    let mut base = core.get_project(&id).unwrap();
    base.settings = ProjectSettings {
        width: 64,
        height: 64,
        fps: 10,
    };
    let mut sampled = core.get_draft_state(&id, &draft.id).unwrap().project;
    sampled.settings = base.settings.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let decode = |path: &std::path::Path| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
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
        assert_eq!(output.stdout.len(), 64 * 64 * 3);
        output.stdout
    };
    let extent = |pixels: &[u8]| {
        let mut points = pixels
            .as_chunks::<3>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p[0] > 180 && p[1] < 30 && p[2] < 30)
            .map(|(i, _)| (i % 64, i / 64));
        let (x, y) = points.next().expect("path disappeared");
        points.fold((x, x, y, y), |(minx, maxx, miny, maxy), (x, y)| {
            (minx.min(x), maxx.max(x), miny.min(y), maxy.max(y))
        })
    };
    let before = renderer.render_preview(&base, &dir, 0).unwrap();
    let baseline_png = root.path().join("baseline.png");
    std::fs::copy(dir.join(before.relative_path), &baseline_png).unwrap();
    let before = decode(&baseline_png);
    assert_eq!(extent(&before), (10, 29, 20, 39));
    assert!(
        before[(30 * 64 + 30) * 3] < 30,
        "transparent path border expanded"
    );
    let frame = renderer.render_preview(&sampled, &dir, 0).unwrap();
    let pixels = decode(&dir.join(&frame.relative_path));
    assert_eq!(extent(&pixels), (10, 29, 20, 39));
    let mse = before
        .iter()
        .zip(&pixels)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / pixels.len() as f64;
    assert!(mse < 30.0, "identity pipeline color drift: MSE={mse}");
    let sampled_png = root.path().join("sampled.png");
    std::fs::copy(dir.join(&frame.relative_path), &sampled_png).unwrap();
    let range = renderer
        .render_preview_range(
            &sampled,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let export = root.path().join("exports/identity.mp4");
    std::fs::create_dir_all(export.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &sampled,
            &dir,
            ExportOptions {
                output: &export,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    for path in [dir.join(range.relative_path), export] {
        let actual = decode(&path);
        assert_eq!(extent(&actual), (10, 29, 20, 39));
        let comparison = std::process::Command::new(&ffmpeg)
            .args(["-v", "info", "-i"])
            .arg(&sampled_png)
            .args(["-i"])
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-lavfi",
                SINGLE_FRAME_SSIM,
                "-f",
                "null",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            ])
            .output()
            .unwrap();
        assert!(comparison.status.success());
        let text = String::from_utf8_lossy(&comparison.stderr);
        let ssim: f64 = text
            .split("All:")
            .nth(1)
            .expect("SSIM missing")
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(ssim >= 0.99, "SSIM={ssim}");
    }
    core.commit_draft(&id, &draft.id, 2).unwrap();
    core.undo(&id, 3).unwrap();
    core.redo(&id, 4).unwrap();
    let mut reopened = core.get_project(&id).unwrap();
    reopened.settings = base.settings;
    let frame = renderer.render_preview(&reopened, &dir, 0).unwrap();
    assert!(
        decode(&dir.join(frame.relative_path)) == pixels,
        "reopen changed the identity-effect frame"
    );
}

#[test]
fn review_correlated_crop_floor_rejects_both_axes_atomically() {
    for (origin, extent) in [
        ("media.crop_x", "media.crop_width"),
        ("media.crop_y", "media.crop_height"),
    ] {
        let (root, core, id, track) = setup();
        let source = root.path().join("media/image.png");
        std::fs::write(&source, b"typed image fixture").unwrap();
        let asset = core
            .import_asset(
                &id,
                0,
                &source,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    video_width: Some(40),
                    video_height: Some(20),
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let item = core.edit(&id, 1, op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
        let channels = vec![
            scalar_channel(origin, 0.9999999, 0.9999998, json!("linear")),
            scalar_channel(extent, 0.0000001, 0.0000002, json!("linear")),
        ];
        let before = authoritative_files(&core, &id);
        let edit = op(
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}),
        );
        assert_eq!(
            core.edit(&id, 2, edit.clone()).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
        assert_eq!(
            core.create_draft(&id, 2, vec![edit], None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
        let batch:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
            {"operation":"update_item","itemId":item,"transform":{"positionX":2,"positionY":3,"scale":1,"opacity":1}},
            {"operation":"set_animation_channels","itemId":item,"animationChannels":channels}
        ])).unwrap();
        assert_eq!(
            core.edit_batch(&id, 2, batch).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
    }
}

#[test]
fn review_tiny_correlated_hold_crop_remains_accepted() {
    for (origin, extent) in [
        ("media.crop_x", "media.crop_width"),
        ("media.crop_y", "media.crop_height"),
    ] {
        let (root, core, id, track) = setup();
        let source = root.path().join("media/image.png");
        std::fs::write(&source, b"typed image fixture").unwrap();
        let asset = core
            .import_asset(
                &id,
                0,
                &source,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    video_width: Some(40),
                    video_height: Some(20),
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let item = core.edit(&id, 1, op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
        let channels = vec![
            scalar_channel(origin, 0.9999999, 0.9999998, json!("hold")),
            scalar_channel(extent, 0.0000001, 0.0000002, json!("hold")),
        ];
        assert_eq!(core.edit(&id, 2, op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap().revision, 3);
    }
}

#[test]
fn review_schema26_legacy_drafts_validate_matching_retained_base() {
    for invalid in [false, true] {
        let (_root, core, id, track) = setup();
        let item = core
            .edit(&id, 0, op(authored_rectangle(&track)))
            .unwrap()
            .changed_ids[0]
            .clone();
        let draft = core
            .create_draft(
                &id,
                1,
                vec![op(
                    json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":1100}),
                )],
                None,
            )
            .unwrap();
        // The valid control's target exists only in its retained base. Replaying
        // onto current state would incorrectly reject this compatible stale draft.
        core.edit(
            &id,
            1,
            if invalid {
                op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":1200}))
            } else {
                op(json!({"operation":"delete_item","itemId":item}))
            },
        )
        .unwrap();
        for (path, bytes) in authoritative_files(&core, &id) {
            let mut value: Value = serde_json::from_slice(&bytes).unwrap();
            if path.file_name().unwrap() == "project.json" {
                value["schemaVersion"] = json!(26);
                value.as_object_mut().unwrap().remove("audioBuses");
            } else if path.file_name().unwrap() == "history.json" {
                for side in ["undo", "redo"] {
                    for snapshot in value[side].as_array_mut().unwrap() {
                        snapshot["schemaVersion"] = json!(26);
                        snapshot.as_object_mut().unwrap().remove("audioBuses");
                    }
                }
            } else if invalid {
                value["operations"][0]["itemId"] = json!("missing");
            } else {
                continue;
            }
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before = authoritative_files(&core, &id);
        if invalid {
            assert_eq!(
                core.get_project(&id).unwrap_err().code,
                ErrorCode::ItemNotFound
            );
            assert_eq!(authoritative_files(&core, &id), before);
        } else {
            assert_eq!(
                core.get_project(&id).unwrap().schema_version,
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
            assert_eq!(
                core.get_draft_state(&id, &draft.id).unwrap_err().code,
                ErrorCode::RevisionConflict
            );
            let draft_path = core
                .paths()
                .project_dir(&id)
                .unwrap()
                .join("drafts")
                .join(format!("{}.json", draft.id));
            assert_eq!(
                std::fs::read(&draft_path).unwrap(),
                before
                    .iter()
                    .find(|(path, _)| path == &draft_path)
                    .unwrap()
                    .1
            );
            let after = authoritative_files(&core, &id);
            core.get_project(&id).unwrap();
            assert_eq!(authoritative_files(&core, &id), after);
        }
    }
}

fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

#[test]
fn scoped_graphic_targets_resolve_creation_aliases_atomically() {
    use opencut_editor_core::BatchEditOperation;
    let (_root, core, id, track) = setup();
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_shape","trackId":track,"resultAlias":"path","startMs":0,"durationMs":1000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":null,"stroke":{"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},
        {"operation":"set_animation_channels","itemId":"@path","animationChannels":[{"property":"graphic.path_trim","target":{"kind":"graphic_geometry","scope":"root","id":"@path"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    let item = &result.aliases["path"];
    let project = core.get_project(&id).unwrap();
    assert_eq!(
        project
            .find_item(item)
            .unwrap()
            .visual_properties()
            .animation_channels[0]
            .target
            .as_ref()
            .unwrap()
            .id,
        *item
    );
    core.undo(&id, 1).unwrap();
    core.redo(&id, 2).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(item)
            .unwrap()
            .visual_properties()
            .animation_channels[0]
            .target
            .as_ref()
            .unwrap()
            .id,
        *item
    );
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"component_create","resultAlias":"component","name":"Local","width":16,"height":16,"durationMs":1000,"tracks":[]},
        {"operation":"component_update","componentId":"@component","name":"Local","width":16,"height":16,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"shape","id":"local-path","startMs":0,"durationMs":1000,"keyframes":[],"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,"animationChannels":[{"property":"graphic.path_trim","target":{"kind":"graphic_geometry","scope":"component:@component","id":"local-path"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}]}]}
    ])).unwrap();
    let component = core.edit_batch(&id, 3, batch).unwrap().aliases["component"].clone();
    assert_eq!(
        core.get_project(&id).unwrap().components[0].tracks[0].items[0]
            .visual_properties()
            .animation_channels[0]
            .target
            .as_ref()
            .unwrap()
            .scope,
        format!("component:{component}")
    );
}

#[test]
fn hidden_retained_effect_work_is_bounded_before_edit_publication() {
    let (_root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":1024,"height":1024,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(
        &id,
        1,
        op(json!({"operation":"update_item","itemId":item,"hidden":true})),
    )
    .unwrap();
    let before = authoritative_files(&core, &id);
    let error=core.edit(&id,2,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"b","type":"gaussian_blur","radiusPx":128}]}))).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn legacy_draft_edits_on_extended_bases_are_certified_before_reopen() {
    let (_root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":null,"stroke":{"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}))).unwrap().changed_ids[0].clone();
    let channels = json!([{"property":"graphic.path_points","target":{"kind":"graphic_geometry","scope":"root","id":item},"keyframes":[{"timeMs":0,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":10,"y":10}]},"curve":"linear"},{"timeMs":500,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":20,"y":10}]},"curve":"hold"}]}]);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap();
    let draft = core
        .create_draft(
            &id,
            2,
            vec![op(
                json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":1100}),
            )],
            None,
        )
        .unwrap();
    let path = core
        .paths()
        .project_dir(&id)
        .unwrap()
        .join("drafts")
        .join(format!("{}.json", draft.id));
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["operations"][0]["durationMs"] = json!(40000);
    std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let before = authoritative_files(&core, &id);
    let error = core.get_project(&id).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("maxCandidateAnalysisNodes"));
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn malformed_retained_generations_and_future_versions_never_publish() {
    for (which, version) in [
        ("current", 26),
        ("undo", 26),
        ("redo", 26),
        ("current", opencut_editor_core::PROJECT_SCHEMA_VERSION + 1),
    ] {
        let (_root, core, id, track) = setup();
        let item = core
            .edit(&id, 0, op(authored_rectangle(&track)))
            .unwrap()
            .changed_ids[0]
            .clone();
        core.edit(
            &id,
            1,
            op(json!({"operation":"update_item","itemId":item,"hidden":false})),
        )
        .unwrap();
        core.edit(
            &id,
            2,
            op(json!({"operation":"update_item","itemId":item,"hidden":true})),
        )
        .unwrap();
        core.undo(&id, 3).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let path = dir.join(if which == "current" {
            "project.json"
        } else {
            "history.json"
        });
        let mut data: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let state = if which == "current" {
            &mut data
        } else {
            let last = data[which].as_array().unwrap().len() - 1;
            &mut data[which][last]
        };
        state["schemaVersion"] = json!(version);
        if state["schemaVersion"]
            .as_u64()
            .is_some_and(|version| version < 39)
        {
            state.as_object_mut().unwrap().remove("audioBuses");
        }
        if version == 26 {
            state["tracks"][1]["items"][0]["effects"] = json!([]);
        }
        std::fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        let before = authoritative_files(&core, &id);
        assert!(core.get_project(&id).is_err());
        assert_eq!(
            authoritative_files(&core, &id),
            before,
            "{which} schema {version}"
        );
    }
}

#[test]
fn invalid_render_samples_fail_before_inspecting_or_replacing_destination() {
    use opencut_editor_core::{ExportOptions, Renderer, VisualEffect};
    let (root, core, id, track) = setup();
    core.edit(&id, 0, op(authored_rectangle(&track))).unwrap();
    let before = authoritative_files(&core, &id);
    let mut project = core.get_project(&id).unwrap();
    project.tracks[1].items[0].visual_properties_mut().effects = vec![VisualEffect::GaussianBlur {
        id: "b".into(),
        radius_px: 129.0,
    }];
    let output = root.path().join("exports/existing.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    std::fs::write(&output, b"preserve destination").unwrap();
    let error = Renderer::new("unavailable-ffmpeg", "unavailable-ffprobe", None)
        .export_video(
            &project,
            &core.paths().project_dir(&id).unwrap(),
            ExportOptions {
                output: &output,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(std::fs::read(output).unwrap(), b"preserve destination");
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn canonical_channel_fixtures_validate_topology_targets_and_numeric_bounds() {
    let (_root, core, id, track) = setup();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../contracts/extended-visual-animation-v1.json"
    ))
    .unwrap();
    let item=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":10,"y":10},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":0.5,"color":{"r":0,"g":1,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":1}}]},"stroke":null}))).unwrap().changed_ids[0].clone();
    let mut revision = 1;
    for case in fixture["channelCases"].as_array().unwrap() {
        let mut channel = case["channel"].clone();
        if let Some(target) = channel.get_mut("target") {
            target["id"] = json!(item);
        }
        let before = authoritative_files(&core, &id);
        let result=core.edit(&id,revision,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel]})));
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}: {result:?}",
            case["name"]
        );
        match result {
            Ok(_) => revision += 1,
            Err(error) => {
                assert_eq!(error.code, ErrorCode::InvalidArgument);
                assert_eq!(authoritative_files(&core, &id), before);
            }
        }
    }
    let mut gradient = fixture["channelCases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "valid-gradient")
        .unwrap()["channel"]
        .clone();
    gradient["target"]["id"] = json!(item);
    gradient["keyframes"][0]["curve"] =
        json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0});
    let mut final_key = gradient["keyframes"][0].clone();
    final_key["timeMs"] = json!(500);
    final_key["curve"] = json!("hold");
    final_key["value"]["stops"][1]["offset"] = json!(0.95);
    gradient["keyframes"]
        .as_array_mut()
        .unwrap()
        .push(final_key);
    let before = authoritative_files(&core, &id);
    assert_eq!(core.edit(&id,revision,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[gradient]}))).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn native_extended_draft_and_export_preserve_decoded_audio_and_timing() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(authored_rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let wav = root.path().join("media/tone.wav");
    let generated = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(&wav)
        .output()
        .unwrap();
    assert!(generated.status.success());
    let asset = core
        .import_asset(
            &id,
            1,
            &wav,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    core.edit(&id,2,op(json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap();
    let mut baseline = core.get_project(&id).unwrap();
    baseline.settings.width = 64;
    baseline.settings.height = 64;
    baseline.settings.fps = 10;
    let operation = op(
        json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[scalar_channel("transform.rotation_deg",0.0,90.0,json!("linear"))]}),
    );
    let draft = core
        .create_draft(&id, 3, vec![operation.clone()], Some("Animated".into()))
        .unwrap();
    let mut candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    candidate.settings = baseline.settings.clone();
    core.edit(&id, 3, operation).unwrap();
    let mut animated = core.get_project(&id).unwrap();
    animated.settings = baseline.settings.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let draft_frame = renderer.render_preview(&candidate, &dir, 500).unwrap();
    let expected_pixels = std::fs::read(dir.join(draft_frame.relative_path)).unwrap();
    let frame = renderer.render_preview(&animated, &dir, 500).unwrap();
    assert_eq!(
        std::fs::read(dir.join(frame.relative_path)).unwrap(),
        expected_pixels
    );
    let exports = root.path().join("exports");
    std::fs::create_dir_all(&exports).unwrap();
    let mut outputs = vec![];
    for (index, project) in [&baseline, &animated].into_iter().enumerate() {
        let path = exports.join(format!("audio{index}.mp4"));
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &path,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        outputs.push(path);
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 200,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let saved = exports.join(format!("range{index}.mp4"));
        std::fs::copy(dir.join(range.relative_path), &saved).unwrap();
        outputs.push(saved);
    }
    let decode = |path: &std::path::Path| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(path)
            .args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
            .stdout
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect::<Vec<_>>()
    };
    for (left, right) in [(0, 2), (1, 3)] {
        let a = decode(&outputs[left]);
        let b = decode(&outputs[right]);
        assert_eq!(a.len(), b.len());
        let rms = (a
            .iter()
            .zip(&b)
            .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
            .sum::<f64>()
            / a.len() as f64)
            .sqrt();
        assert!(rms <= 0.0001, "audio RMS {rms}");
        let durations = [&outputs[left], &outputs[right]].map(|path| {
            let probe = std::process::Command::new(&ffprobe)
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "format=duration",
                    "-of",
                    "default=noprint_wrappers=1:nokey=1",
                ])
                .arg(path)
                .output()
                .unwrap();
            assert!(probe.status.success());
            String::from_utf8(probe.stdout)
                .unwrap()
                .trim()
                .parse::<f64>()
                .unwrap()
        });
        assert!((durations[0] - durations[1]).abs() <= 0.1);
    }
}

#[test]
fn exhausted_candidate_analysis_preserves_batch_and_draft_generations() {
    use opencut_editor_core::BatchEditOperation;
    let (_root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":40000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":null,"stroke":{"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}))).unwrap().changed_ids[0].clone();
    let draft = core
        .create_draft(
            &id,
            1,
            vec![op(
                json!({"operation":"update_item","itemId":item,"hidden":false}),
            )],
            Some("Retain".into()),
        )
        .unwrap();
    let before = authoritative_files(&core, &id);
    let channels = json!([{"property":"graphic.path_points","target":{"kind":"graphic_geometry","scope":"root","id":item},"keyframes":[{"timeMs":0,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":10,"y":10}]},"curve":"linear"},{"timeMs":39999,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":20,"y":10}]},"curve":"hold"}]}]);
    let update =
        json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels});
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"color":"#ffffff","width":1,"height":1,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"new"},update])).unwrap();
    let error = core.edit_batch(&id, 1, batch).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(
        error.message.contains("maxCandidateAnalysisNodes"),
        "{}",
        error.message
    );
    assert_eq!(authoritative_files(&core, &id), before);
    assert_eq!(
        core.update_draft(&id, &draft.id, 1, vec![op(update)], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn stack_eligibility_topology_and_clear_preserve_static_state() {
    let (_root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let before = authoritative_files(&core, &id);
    for effects in [
        json!([{"id":"x","type":"vignette","amount":0},{"id":"x","type":"vignette","amount":0}]),
        Value::Array(
            (0..17)
                .map(|i| json!({"id":i.to_string(),"type":"vignette","amount":0}))
                .collect(),
        ),
    ] {
        assert_eq!(
            core.edit(
                &id,
                1,
                op(json!({"operation":"update_item","itemId":item,"effects":effects}))
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(authoritative_files(&core, &id), before);
    }
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[scalar_channel("transform.rotation_deg",0.0,90.0,json!("linear"))]}))).unwrap();
    let static_before =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item).unwrap()).unwrap();
    core.edit(
        &id,
        2,
        op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[]})),
    )
    .unwrap();
    let mut static_after =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item).unwrap()).unwrap();
    static_after["animationChannels"] = static_before["animationChannels"].clone();
    assert_eq!(static_after, static_before);
    let group = core
        .edit(
            &id,
            3,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(&id,4,op(json!({"operation":"update_item","itemId":group,"effects":[{"id":"x","type":"vignette","amount":0}]}))).unwrap();
    assert_eq!(
        serde_json::to_value(
            core.get_project(&id)
                .unwrap()
                .find_item(&group)
                .unwrap()
                .visual_properties()
                .effects
                .clone()
        )
        .unwrap(),
        json!([{"id":"x","type":"vignette","amount":0.0}])
    );
    let before = authoritative_files(&core, &id);
    assert_eq!(core.edit(&id,5,op(json!({"operation":"set_animation_channels","itemId":group,"animationChannels":[{"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"x"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"hold"}]}]}))).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn nested_rotation_clocks_and_scoped_effects_render_at_loop_turns() {
    use opencut_editor_core::{BatchEditOperation, PreviewRangeOptions, Renderer};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (_root, core, id, track) = setup();
    let component = core.edit(&id,0,op(json!({"operation":"component_create","name":"Nested","width":64,"height":64,"durationMs":2000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    let mut rotation = scalar_channel("transform.rotation_deg", 0.0, 90.0, json!("linear"));
    rotation["keyframes"][1]["timeMs"] = json!(200);
    rotation["loop"] = json!({"mode":"ping_pong","iterations":"infinite"});
    let mut tint = scalar_channel("effect.vignette_amount", 0.0, 0.8, json!("linear"));
    tint["target"] = json!({"kind":"effect","scope":format!("component:{component}"),"id":"edge"});
    core.edit(&id,1,op(json!({"operation":"component_update","componentId":component,"name":"Nested","width":64,"height":64,"durationMs":2000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"rectangle","id":"leaf","startMs":0,"durationMs":2000,"width":16,"height":8,"color":"#dddddd","transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1},"effects":[{"id":"edge","type":"vignette","amount":0}],"animationChannels":[rotation,tint],"keyframes":[],"zIndex":0,"stackOrder":0}]}]}))).unwrap();
    let edits: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"staggerMs":50,"resultAlias":"group"},
        {"operation":"set_animation_channels","itemId":"@group","animationChannels":[scalar_channel("transform.rotation_deg",0.0,20.0,json!("linear"))]},
        {"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":2,"resultAlias":"instance"},
        {"operation":"item_set_parent","itemId":"@instance","parent":{"scope":"root","id":"@group"}},
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"@group"},"copies":1,"timeOffsetMs":-100,"opacityOffset":0,"transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}
    ])).unwrap();
    core.edit_batch(&id, 2, edits).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    project.settings.fps = 20;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 200,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 20,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let mut frames = Vec::new();
    for time in [200, 250, 400, 450, 900] {
        let frame = renderer.render_preview(&project, &dir, time).unwrap();
        frames.push(std::fs::read(dir.join(&frame.relative_path)).unwrap());
        let comparison = std::process::Command::new(&ffmpeg)
            .current_dir(_root.path())
            .args(["-v", "info", "-i"])
            .arg(dir.join(frame.relative_path))
            .args(["-ss", &format!("{:.3}", (time - 200) as f64 / 1000.0), "-i"])
            .arg(dir.join(&range.relative_path))
            .args([
                "-frames:v",
                "1",
                "-lavfi",
                &format!("{SINGLE_FRAME_SSIM}=stats_file=single-frame-ssim.txt"),
                "-f",
                "null",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            ])
            .output()
            .unwrap();
        assert!(
            comparison.status.success(),
            "{}",
            String::from_utf8_lossy(&comparison.stderr)
        );
        let stats = std::fs::read_to_string(_root.path().join("single-frame-ssim.txt")).unwrap();
        assert_eq!(
            stats.lines().count(),
            1,
            "comparison evaluated later animation frames"
        );
        let log = String::from_utf8_lossy(&comparison.stderr);
        let ssim = log
            .split("All:")
            .nth(1)
            .and_then(|v| v.split_whitespace().next())
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(ssim >= 0.99, "nested sample {time}: {ssim}");
    }
    assert!(frames.windows(2).any(|pair| pair[0] != pair[1]));
}

#[test]
fn extended_rotation_and_effects_are_atomic_and_undoable() {
    let (_root, core, project, track) = setup();
    let added = core
        .edit(
            &project,
            0,
            op(json!({"operation":"add_rectangle", "trackId":track,
        "color":"#ff0000", "width":40, "height":20, "startMs":0, "durationMs":1000,
        "transform":{"positionX":60,"positionY":40,"scale":1,"opacity":1}})),
        )
        .unwrap();
    let item = &added.changed_ids[0];
    let effect = json!({"id":"soft", "type":"gaussian_blur", "radiusPx":1});
    core.edit(
        &project,
        1,
        op(json!({"operation":"update_item", "itemId":item, "effects":[effect]})),
    )
    .unwrap();
    let channels = json!([
        {"property":"transform.rotation_deg", "keyframes":[
            {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
            {"timeMs":500,"value":{"type":"scalar","value":180},"curve":"hold"}]},
        {"property":"effect.blur_radius", "target":{"kind":"effect","scope":"root","id":"soft"}, "keyframes":[
            {"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},
            {"timeMs":500,"value":{"type":"scalar","value":2},"curve":"hold"}]}]);
    core.edit(&project, 2, op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap();
    let snapshot = serde_json::to_value(core.get_project(&project).unwrap()).unwrap();
    let invalid = op(
        json!({"operation":"update_item","itemId":item,"effects":[{"id":"soft","type":"gaussian_blur","radiusPx":129}]}),
    );
    assert_eq!(
        core.edit(&project, 3, invalid).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&project).unwrap()).unwrap(),
        snapshot
    );
    core.undo(&project, 3).unwrap();
    core.redo(&project, 4).unwrap();
    let reopened = core.get_project(&project).unwrap();
    assert_eq!(
        reopened.tracks[1].items[0]
            .visual_properties()
            .animation_channels
            .len(),
        2
    );
}

fn scalar_channel(property: &str, first: f64, last: f64, curve: Value) -> Value {
    json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first},"curve":curve},{"timeMs":500,"value":{"type":"scalar","value":last},"curve":"hold"}]})
}

fn authored_rectangle(track: &str) -> Value {
    json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":24,"height":8,"startMs":0,"durationMs":1000,"transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1}})
}

fn authoritative_files(core: &EditorCore, id: &str) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let dir = core.paths().project_dir(id).unwrap();
    let mut files = vec![dir.join("project.json"), dir.join("history.json")];
    if dir.join("drafts").exists() {
        files.extend(
            std::fs::read_dir(dir.join("drafts"))
                .unwrap()
                .map(|e| e.unwrap().path()),
        );
    }
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let bytes = std::fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect()
}

#[test]
fn schema_27_migrates_current_history_and_drafts_and_reopen_does_not_rewrite() {
    let (_root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(authored_rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(
        &id,
        1,
        op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":1200})),
    )
    .unwrap();
    core.undo(&id, 2).unwrap();
    let draft = core
        .create_draft(
            &id,
            3,
            vec![op(
                json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":1100}),
            )],
            None,
        )
        .unwrap();
    let before = core.get_project(&id).unwrap();
    for (path, bytes) in authoritative_files(&core, &id) {
        if path.file_name().unwrap() == "project.json" {
            let mut project: Value = serde_json::from_slice(&bytes).unwrap();
            project["schemaVersion"] = json!(26);
            project.as_object_mut().unwrap().remove("audioBuses");
            std::fs::write(path, serde_json::to_vec(&project).unwrap()).unwrap();
        } else if path.file_name().unwrap() == "history.json" {
            let mut history: Value = serde_json::from_slice(&bytes).unwrap();
            for side in ["undo", "redo"] {
                for project in history[side].as_array_mut().unwrap() {
                    project["schemaVersion"] = json!(26);
                    project.as_object_mut().unwrap().remove("audioBuses");
                }
            }
            std::fs::write(path, serde_json::to_vec(&history).unwrap()).unwrap();
        }
    }
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(
        migrated.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(migrated.id, before.id);
    assert_eq!(migrated.revision, before.revision);
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .find_item(&item)
            .unwrap()
            .duration_ms(),
        1100
    );
    let files = authoritative_files(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(authoritative_files(&core, &id), files);
    core.redo(&id, 3).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .duration_ms(),
        1200
    );
}

#[test]
fn premature_extended_draft_fields_reject_migration_without_touching_any_authoritative_bytes() {
    let (_root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(authored_rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    core.create_draft(&id,1,vec![op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[scalar_channel("transform.rotation_deg",0.0,90.0,json!("linear"))]}))],None).unwrap();
    let path = core.paths().project_dir(&id).unwrap().join("project.json");
    let mut source: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    source["schemaVersion"] = json!(26);
    source.as_object_mut().unwrap().remove("audioBuses");
    std::fs::write(path, serde_json::to_vec(&source).unwrap()).unwrap();
    let before = authoritative_files(&core, &id);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn aliased_extended_batch_and_failed_draft_preserve_one_revision_and_bytes() {
    use opencut_editor_core::BatchEditOperation;
    let (_root, core, id, track) = setup();
    let mut added = authored_rectangle(&track);
    added["resultAlias"] = json!("box");
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([added,{"operation":"update_item","itemId":"@box","effects":[{"id":"v","type":"vignette","amount":0.5}]},{"operation":"set_animation_channels","itemId":"@box","animationChannels":[scalar_channel("transform.rotation_deg",0.0,90.0,json!("linear"))]}])).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    assert_eq!(result.revision, 1);
    let item = &result.aliases["box"];
    let draft=core.create_draft(&id,1,vec![op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[scalar_channel("transform.rotation_deg",0.0,180.0,json!("linear"))]}))],None).unwrap();
    let before = authoritative_files(&core, &id);
    let invalid = op(
        json!({"operation":"update_item","itemId":item,"effects":[{"id":"v","type":"vignette","amount":1.01}]}),
    );
    assert_eq!(
        core.update_draft(&id, &draft.id, 1, vec![invalid.clone()], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.edit_batch(&id, 1, vec![invalid]).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(authoritative_files(&core, &id), before);
    assert_eq!(
        core.commit_draft(&id, &draft.id, 0).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    core.commit_draft(&id, &draft.id, 1).unwrap();
    assert_eq!(core.get_project(&id).unwrap().revision, 2);
}

#[test]
fn crop_certification_rejects_spring_overshoot_without_publication_and_accepts_correlated_linear_crop()
 {
    let (root, core, id, track) = setup();
    let source = root.path().join("media/image.png");
    std::fs::write(&source, b"typed media fixture").unwrap();
    let asset = core
        .import_asset(
            &id,
            0,
            &source,
            MediaType::Image,
            MediaProbeFacts {
                has_video: true,
                video_width: Some(40),
                video_height: Some(20),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let item=core.edit(&id,1,op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
    core.edit(&id,2,op(json!({"operation":"update_item","itemId":item,"crop":{"x":0,"y":0,"width":0.5,"height":1}}))).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let channels = json!([scalar_channel(
        "media.crop_x",
        0.0,
        0.5,
        json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0})
    )]);
    let error=core.edit(&id,3,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    assert_eq!(core.get_project(&id).unwrap().revision, 3);
    let channels = json!([
        scalar_channel("media.crop_x", 0.0, 0.5, json!("linear")),
        scalar_channel("media.crop_width", 1.0, 0.5, json!("linear"))
    ]);
    core.edit(&id,3,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap();
    assert_eq!(core.get_project(&id).unwrap().revision, 4);
}

#[test]
fn effect_ids_are_scoped_and_duplicate_property_targets_remain_distinct() {
    let (_root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":10,"height":10,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"a","type":"gaussian_blur","radiusPx":0},{"id":"b","type":"gaussian_blur","radiusPx":0}]}))).unwrap();
    let channel = |id: &str, scope: &str| {
        let mut c = scalar_channel("effect.blur_radius", 0.0, 1.0, json!("linear"));
        c["target"] = json!({"kind":"effect","scope":scope,"id":id});
        c
    };
    for (target, scope, expected) in [
        ("missing", "root", ErrorCode::ItemNotFound),
        ("a", "component:missing", ErrorCode::InvalidArgument),
    ] {
        assert_eq!(core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel(target,scope)]}))).unwrap_err().code,expected);
        assert_eq!(core.get_project(&id).unwrap().revision, 2);
    }
    core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel("a","root"),channel("b","root")]}))).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().tracks[1].items[0]
            .visual_properties()
            .animation_channels
            .len(),
        2
    );
}

#[test]
fn native_rotation_effects_share_frame_range_and_export_samples() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (root, core, id, track) = setup();
    let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":24,"height":8,"startMs":0,"durationMs":1000,"transform":{"positionX":32,"positionY":32,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"tint","type":"color_tint","color":{"r":0,"g":1,"b":0,"a":1}}]}))).unwrap();
    core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[scalar_channel("transform.rotation_deg",0.0,90.0,json!("linear"))]}))).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    project.settings.fps = 10;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let frame = renderer.render_preview(&project, &dir, 500).unwrap();
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("exports/extended.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let decode = |path: &std::path::Path, time: Option<&str>| {
        let mut command = std::process::Command::new(&ffmpeg);
        command.args(["-v", "error"]);
        if let Some(time) = time {
            command.args(["-ss", time]);
        }
        let result = command
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
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout.len(), 64 * 64 * 3);
        result.stdout
    };
    let pixels = decode(&dir.join(&frame.relative_path), None);
    let at = (40 * 64 + 28) * 3;
    assert!(pixels[at + 1] > 180, "rotated/tinted rectangle missing");
    assert!(pixels[at] < 40);
    for encoded in [dir.join(range.relative_path), output] {
        let candidate = decode(&encoded, Some("0.5"));
        let mse = pixels
            .iter()
            .zip(candidate)
            .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
            .sum::<f64>()
            / pixels.len() as f64;
        assert!(mse < 30.0, "frame/encoded sample drift: MSE {mse}");
        let comparison = std::process::Command::new(&ffmpeg)
            .args(["-v", "info", "-i"])
            .arg(dir.join(&frame.relative_path))
            .args(["-ss", "0.5", "-i"])
            .arg(&encoded)
            .args([
                "-frames:v",
                "1",
                "-lavfi",
                SINGLE_FRAME_SSIM,
                "-f",
                "null",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            ])
            .output()
            .unwrap();
        assert!(
            comparison.status.success(),
            "{}",
            String::from_utf8_lossy(&comparison.stderr)
        );
        let log = String::from_utf8_lossy(&comparison.stderr);
        let ssim = log
            .split("All:")
            .nth(1)
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok())
            .expect("SSIM result");
        assert!(ssim >= 0.99, "sample SSIM {ssim}");
    }
}

#[test]
fn native_compound_path_gradient_crop_and_effect_channels_share_nonzero_range_samples() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let (root, core, id, track) = setup();
    let points = json!([{"x":0,"y":0},{"x":24,"y":0},{"x":24,"y":16},{"x":0,"y":16}]);
    let geometry = json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":24,"y":0}},{"type":"lineTo","to":{"x":24,"y":16}},{"type":"lineTo","to":{"x":0,"y":16}},{"type":"close"}]}});
    let color = |value: f64| json!({"r":value,"g":value,"b":value,"a":1});
    let paint = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":24,"y":0},"stops":[{"offset":0,"color":color(0.3)},{"offset":1,"color":color(1.0)}]});
    let shape=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":geometry,"fill":paint,"stroke":{"paint":paint,"width":2,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4},"transform2d":{"position":{"x":32,"y":32,"unit":"pixels"},"anchor":{"x":0.25,"y":0.25},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let effects = json!([{"id":"blur","type":"gaussian_blur","radiusPx":0},{"id":"glow","type":"glow","radiusPx":0,"intensity":0.5,"color":color(1.0)},{"id":"tint","type":"color_tint","color":color(1.0)},{"id":"vignette","type":"vignette","amount":0}]);
    core.edit(
        &id,
        1,
        op(json!({"operation":"update_item","itemId":shape,"effects":effects})),
    )
    .unwrap();
    let compound = |property: &str, target: Value, a: Value, b: Value| json!({"property":property,"target":target,"keyframes":[{"timeMs":0,"value":a,"curve":"linear"},{"timeMs":500,"value":b,"curve":"hold"}]});
    let geometry_target = json!({"kind":"graphic_geometry","scope":"root","id":shape});
    let mut channels = vec![
        scalar_channel("transform.rotation_deg", 0.0, 20.0, json!("linear")),
        compound(
            "graphic.path_points",
            geometry_target.clone(),
            json!({"type":"path_points","points":points}),
            json!({"type":"path_points","points":[{"x":0,"y":0},{"x":32,"y":0},{"x":24,"y":24},{"x":0,"y":16}]}),
        ),
    ];
    let mut trim = scalar_channel("graphic.path_trim", 1.0, 0.8, json!("linear"));
    trim["target"] = geometry_target;
    channels.push(trim);
    channels.push(compound("graphic.gradient_stops",json!({"kind":"graphic_stroke","scope":"root","id":shape}),json!({"type":"gradient_stops","stops":[{"offset":0,"color":[0.3,0.3,0.3,1]},{"offset":1,"color":[1,1,1,1]}]}),json!({"type":"gradient_stops","stops":[{"offset":0,"color":[1,1,1,1]},{"offset":1,"color":[0.3,0.3,0.3,1]}]})));
    for (property, effect, last) in [
        ("effect.blur_radius", "blur", 0.4),
        ("effect.glow_radius", "glow", 0.6),
        ("effect.vignette_amount", "vignette", 0.5),
    ] {
        let mut c = scalar_channel(property, 0.0, last, json!("linear"));
        c["target"] = json!({"kind":"effect","scope":"root","id":effect});
        channels.push(c);
    }
    channels.push(compound(
        "effect.tint_color",
        json!({"kind":"effect","scope":"root","id":"tint"}),
        json!({"type":"rgba","r":1,"g":1,"b":1,"a":0}),
        json!({"type":"rgba","r":0.8,"g":0.8,"b":0.8,"a":0.5}),
    ));
    for channel in &mut channels {
        channel["loop"] = json!({"mode":"ping_pong","iterations":"infinite"});
    }
    core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":shape,"animationChannels":channels}))).unwrap();
    let source = root.path().join("media/asymmetric.pam");
    let mut bytes =
        b"P7\nWIDTH 16\nHEIGHT 16\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
    for y in 0..16 {
        for x in 0..16 {
            let gray = if x < 8 {
                50
            } else if y < 8 {
                150
            } else {
                250
            };
            bytes.extend([gray, gray, gray, 255]);
        }
    }
    std::fs::write(&source, bytes).unwrap();
    let png = source.with_extension("png");
    let conversion = std::process::Command::new(&ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(&source)
        .args(["-frames:v", "1"])
        .arg(&png)
        .output()
        .unwrap();
    assert!(
        conversion.status.success(),
        "{}",
        String::from_utf8_lossy(&conversion.stderr)
    );
    let asset = core
        .import_asset(
            &id,
            3,
            &png,
            MediaType::Image,
            MediaProbeFacts {
                has_video: true,
                video_width: Some(16),
                video_height: Some(16),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let media=core.edit(&id,4,op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
    let mut crop_channels = vec![
        scalar_channel("media.crop_x", 0.0, 0.25, json!("linear")),
        scalar_channel("media.crop_y", 0.0, 0.25, json!("linear")),
        scalar_channel("media.crop_width", 1.0, 0.5, json!("linear")),
        scalar_channel("media.crop_height", 1.0, 0.5, json!("linear")),
    ];
    for channel in &mut crop_channels {
        channel["loop"] = json!({"mode":"ping_pong","iterations":"infinite"});
    }
    core.edit(&id,5,op(json!({"operation":"set_animation_channels","itemId":media,"animationChannels":crop_channels}))).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    project.settings.fps = 10;
    let draft = core
        .create_draft(
            &id,
            6,
            vec![op(
                json!({"operation":"update_item","itemId":shape,"hidden":false}),
            )],
            Some("All channel preview".into()),
        )
        .unwrap();
    let mut draft_project = core.get_draft_state(&id, &draft.id).unwrap().project;
    draft_project.settings = project.settings.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    // Isolate each channel so an inert implementation cannot be hidden by other
    // animated properties or an opaque sibling in the intent-parity fixture.
    let isolated_channels: Vec<_> = project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .flat_map(|item| {
            item.visual_properties()
                .animation_channels
                .iter()
                .map(move |channel| (item.id().to_owned(), channel.property))
        })
        .collect();
    assert_eq!(isolated_channels.len(), 12);
    for (item_id, property) in isolated_channels {
        let mut isolated = project.clone();
        for item in isolated.tracks.iter_mut().flat_map(|t| &mut t.items) {
            let selected = item.id() == item_id;
            let visual = item.visual_properties_mut();
            visual.hidden = !selected;
            visual
                .animation_channels
                .retain(|c| selected && c.property == property);
            if matches!(
                property,
                opencut_editor_core::AnimationChannelProperty::BlurRadius
                    | opencut_editor_core::AnimationChannelProperty::GlowRadius
            ) {
                // Use a radius large enough to survive 8-bit output quantization.
                for channel in &mut visual.animation_channels {
                    if let Some(key) = channel.keyframes.last_mut()
                        && let opencut_editor_core::AnimationChannelValue::Scalar { value } =
                            &mut key.value
                    {
                        *value = 4.0;
                    }
                }
            }
            if property != opencut_editor_core::AnimationChannelProperty::TintColor {
                // The fixture's static opaque white tint would hide a sampled
                // gradient when its tint channel is removed for isolation.
                visual.effects.retain(|effect| {
                    !matches!(effect, opencut_editor_core::VisualEffect::ColorTint { .. })
                });
            }
            if matches!(
                property,
                opencut_editor_core::AnimationChannelProperty::CropX
                    | opencut_editor_core::AnimationChannelProperty::CropY
            ) && selected
            {
                visual.crop = Some(opencut_editor_core::MediaCrop {
                    x: 0.0,
                    y: 0.0,
                    width: if property == opencut_editor_core::AnimationChannelProperty::CropX {
                        0.5
                    } else {
                        1.0
                    },
                    height: if property == opencut_editor_core::AnimationChannelProperty::CropY {
                        0.5
                    } else {
                        1.0
                    },
                });
            }
        }
        let first = renderer.render_preview(&isolated, &dir, 0).unwrap();
        let first_pixels = std::fs::read(dir.join(first.relative_path)).unwrap();
        let interior = renderer.render_preview(&isolated, &dir, 200).unwrap();
        assert!(
            first_pixels != std::fs::read(dir.join(interior.relative_path)).unwrap(),
            "isolated {property:?} must visibly change intermediate output"
        );
    }
    let compare = |project: &opencut_editor_core::Project,
                   draft_project: &opencut_editor_core::Project,
                   label: &str| {
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 200,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let output = root.path().join(format!("exports/compound-{label}.mp4"));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for time in [0, 200, 500, 900] {
            let draft_frame = renderer.render_preview(draft_project, &dir, time).unwrap();
            let draft_pixels = std::fs::read(dir.join(draft_frame.relative_path)).unwrap();
            let frame = renderer.render_preview(project, &dir, time).unwrap();
            assert_eq!(
                std::fs::read(dir.join(&frame.relative_path)).unwrap(),
                draft_pixels
            );
            let mut encoded_samples = vec![(output.clone(), time)];
            if time >= 200 {
                encoded_samples.push((dir.join(&range.relative_path), time - 200));
            }
            for (encoded, offset) in encoded_samples {
                let comparison = std::process::Command::new(&ffmpeg)
                    .args(["-v", "info", "-i"])
                    .arg(dir.join(&frame.relative_path))
                    .args(["-ss", &format!("{:.3}", offset as f64 / 1000.0), "-i"])
                    .arg(encoded)
                    .args([
                        "-frames:v",
                        "1",
                        "-lavfi",
                        SINGLE_FRAME_SSIM,
                        "-f",
                        "null",
                        if cfg!(windows) { "NUL" } else { "/dev/null" },
                    ])
                    .output()
                    .unwrap();
                assert!(
                    comparison.status.success(),
                    "{}",
                    String::from_utf8_lossy(&comparison.stderr)
                );
                let log = String::from_utf8_lossy(&comparison.stderr);
                let ssim = log
                    .split("All:")
                    .nth(1)
                    .and_then(|v| v.split_whitespace().next())
                    .and_then(|v| v.parse::<f64>().ok())
                    .expect("SSIM result");
                assert!(ssim >= 0.99, "compound sample {time}: SSIM {ssim}");
            }
        }
    };
    compare(&project, &draft_project, "root");
    let component=core.edit(&id,6,op(json!({"operation":"component_create","name":"Every extended channel","width":64,"height":64,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    let mut local_track = serde_json::to_value(&project.tracks[1]).unwrap();
    for item in local_track["items"].as_array_mut().unwrap() {
        if let Some(channels) = item
            .get_mut("animationChannels")
            .and_then(Value::as_array_mut)
        {
            for channel in channels {
                if let Some(target) = channel.get_mut("target") {
                    target["scope"] = json!(format!("component:{component}"));
                }
            }
        }
    }
    core.edit(&id,7,op(json!({"operation":"component_update","componentId":component,"name":"Every extended channel","width":64,"height":64,"durationMs":1000,"tracks":[local_track]}))).unwrap();
    let batch:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"update_item","itemId":shape,"hidden":true},
        {"operation":"update_item","itemId":media,"hidden":true},
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"set_animation_channels","itemId":"@parent","animationChannels":[scalar_channel("transform.rotation_deg",0.0,10.0,json!("linear"))]},
        {"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":25,"timeScale":0.75,"resultAlias":"nested"},
        {"operation":"item_set_parent","itemId":"@nested","parent":{"scope":"root","id":"@parent"}}
    ])).unwrap();
    let nested = core.edit_batch(&id, 8, batch).unwrap().aliases["nested"].clone();
    let mut nested_project = core.get_project(&id).unwrap();
    nested_project.settings = project.settings.clone();
    let draft = core
        .create_draft(
            &id,
            9,
            vec![op(
                json!({"operation":"update_item","itemId":nested,"hidden":false}),
            )],
            None,
        )
        .unwrap();
    let mut nested_draft = core.get_draft_state(&id, &draft.id).unwrap().project;
    nested_draft.settings = project.settings.clone();
    compare(&nested_project, &nested_draft, "nested");
}
