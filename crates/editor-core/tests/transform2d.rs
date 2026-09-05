use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    Transform2D,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../contracts/transform2d-v1.json")).unwrap()
}
fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

#[test]
fn canonical_transform_payloads() {
    let fixture = fixture();
    for entry in fixture["valid"].as_array().unwrap() {
        let value: Transform2D = serde_json::from_value(entry["value"].clone()).unwrap();
        value.validate().unwrap();
        assert_eq!(
            serde_json::from_value::<Transform2D>(serde_json::to_value(value).unwrap()).unwrap(),
            value
        );
    }
    for entry in fixture["invalid"].as_array().unwrap() {
        let accepted = serde_json::from_value::<Transform2D>(entry["value"].clone())
            .ok()
            .is_some_and(|value| value.validate().is_ok());
        assert!(!accepted, "{}", entry["id"]);
    }
}

#[test]
fn every_numeric_bound_and_nonfinite_value() {
    let identity = fixture()["identity"].clone();
    for (field, low, high, open_low) in [
        ("/position/x", -1e6, 1e6, false),
        ("/position/y", -1e6, 1e6, false),
        ("/anchor/x", 0., 1., false),
        ("/anchor/y", 0., 1., false),
        ("/scaleX", 0., 100., true),
        ("/scaleY", 0., 100., true),
        ("/rotationDeg", -36000., 36000., false),
        ("/skewXDeg", -80., 80., false),
        ("/skewYDeg", -80., 80., false),
        ("/opacity", 0., 1., false),
    ] {
        for (v, valid) in [
            (low, !open_low),
            (high, true),
            (low - 0.01, false),
            (high + 0.01, false),
        ] {
            let mut value = identity.clone();
            *value.pointer_mut(field).unwrap() = json!(v);
            assert_eq!(
                serde_json::from_value::<Transform2D>(value)
                    .unwrap()
                    .validate()
                    .is_ok(),
                valid,
                "{field}={v}"
            );
        }
    }
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for field in 0..10 {
            let mut t = Transform2D::default();
            let target = match field {
                0 => &mut t.position.x,
                1 => &mut t.position.y,
                2 => &mut t.anchor.x,
                3 => &mut t.anchor.y,
                4 => &mut t.scale_x,
                5 => &mut t.scale_y,
                6 => &mut t.rotation_deg,
                7 => &mut t.skew_x_deg,
                8 => &mut t.skew_y_deg,
                _ => &mut t.opacity,
            };
            *target = v;
            assert_eq!(t.validate().unwrap_err().code, ErrorCode::InvalidArgument);
        }
    }
}

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
    let created = core
        .create_project("Transform", ProjectSettings::default())
        .unwrap();
    let track = core.get_project(&created.project_id).unwrap().tracks[1]
        .id
        .clone();
    (root, core, created.project_id, track)
}

#[test]
fn alias_batch_reset_legacy_switch_history_and_atomic_failures() {
    let (_root, core, id, track) = setup();
    let transform = fixture()["valid"][1]["value"].clone();
    let ops:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":4,"positionY":5,"scale":2,"opacity":0.8},"resultAlias":"box"},
        {"operation":"update_item","itemId":"@box","transform2d":transform}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, ops).unwrap();
    assert_eq!(result.revision, 1);
    let item = result.aliases["box"].clone();
    let get = || core.get_project(&id).unwrap();
    assert_eq!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform2d,
        Some(serde_json::from_value::<Transform2D>(transform.clone()).unwrap())
    );
    let before = serde_json::to_value(get()).unwrap();
    for value in [json!({"scaleX":0}), json!(null)] {
        let mut update = json!({"operation":"update_item","itemId":item,"transform2d":transform});
        if value.is_null() {
            update["transform"] = json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
        } else {
            update["transform2d"]["scaleX"] = json!(0);
        }
        assert_eq!(
            core.edit(&id, 1, operation(update)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(serde_json::to_value(get()).unwrap(), before);
    }
    let update = operation(json!({"operation":"update_item","itemId":item,"transform2d":null}));
    assert!(core.edit(&id, 0, update.clone()).is_err());
    core.edit(&id, 1, update).unwrap();
    assert!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform2d
            .is_none()
    );
    assert_eq!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform
            .position_x,
        4.
    );
    core.undo(&id, 2).unwrap();
    assert!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform2d
            .is_some()
    );
    core.redo(&id, 3).unwrap();
    assert!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform2d
            .is_none()
    );
    core.edit(
        &id,
        4,
        operation(json!({"operation":"update_item","itemId":item,"transform2d":transform})),
    )
    .unwrap();
    core.edit(&id,5,operation(json!({"operation":"update_item","itemId":item,"transform":{"positionX":8,"positionY":9,"scale":1,"opacity":1}}))).unwrap();
    assert!(
        get()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .transform2d
            .is_none()
    );
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(get()).unwrap()
    );
}

#[test]
fn migrate_every_supported_version_and_mixed_history() {
    for version in 1..=7 {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut value = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        value["schemaVersion"] = json!(version);
        let mut old = value.clone();
        old["schemaVersion"] = json!(1);
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&json!({"undo":[old],"redo":[value]})).unwrap(),
        )
        .unwrap();
        assert_eq!(
            core.get_project(&id).unwrap().schema_version,
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        let history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        for entry in history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap())
        {
            assert_eq!(
                entry["schemaVersion"],
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
        }
    }
}

#[test]
fn rendered_affine_rectangle_has_expected_position_and_rotated_extent() {
    let Some(tools) = native_tools() else {
        return;
    };
    use opencut_editor_core::{ExportOptions, Renderer};
    let (_root, core, id, track) = setup();
    let mut transform = Transform2D::default();
    transform.position.x = 40.;
    transform.position.y = 10.;
    transform.rotation_deg = 90.;
    core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let project = core.get_project(&id).unwrap();
    let item = project.tracks[1].items[0].id().to_owned();
    core.edit(
        &id,
        1,
        operation(json!({"operation":"update_item","itemId":item,"transform2d":transform})),
    )
    .unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    renderer.readiness().unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let preview = renderer.render_preview(&project, &dir, 0).unwrap();
    let _ = preview;
    let image = std::fs::read_dir(dir.join("previews"))
        .unwrap()
        .map(|v| v.unwrap().path())
        .find(|v| v.extension().is_some_and(|v| v == "png"))
        .unwrap();
    let output = std::process::Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(image)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let red = |x: usize, y: usize| {
        let at = (y * 64 + x) * 3;
        output.stdout[at] > 180 && output.stdout[at + 1] < 60 && output.stdout[at + 2] < 60
    };
    assert!(red(35, 20));
    assert!(!red(25, 20));
    assert!(!red(45, 20));
    assert!(!red(35, 35));
    let export = dir.join("out.mp4");
    renderer
        .export_video(
            &project,
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
    assert!(export.exists());
}

#[test]
fn incompatible_animation_and_failed_batch_preserve_state() {
    let (_root, core, id, track) = setup();
    let result=core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let item = &result.changed_ids[0];
    let keyframes = json!([{"property":"scale","timeMs":0,"value":{"type":"scalar","value":2},"easing":"linear"}]);
    core.edit(
        &id,
        1,
        operation(json!({"operation":"set_keyframes","itemId":item,"keyframes":keyframes})),
    )
    .unwrap();
    let set = operation(
        json!({"operation":"update_item","itemId":item,"transform2d":fixture()["identity"]}),
    );
    assert_eq!(
        core.edit(&id, 2, set.clone()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    core.edit(
        &id,
        2,
        operation(json!({"operation":"set_keyframes","itemId":item,"keyframes":[]})),
    )
    .unwrap();
    core.edit(&id, 3, set).unwrap();
    assert_eq!(
        core.edit(
            &id,
            4,
            operation(json!({"operation":"set_keyframes","itemId":item,"keyframes":keyframes}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let ops = vec![
        operation(json!({"operation":"update_item","itemId":item,"transform2d":null})),
        operation(
            json!({"operation":"update_item","itemId":"missing","transform2d":fixture()["identity"]}),
        ),
    ];
    assert!(core.edit_batch(&id, 4, ops).is_err());
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
}

#[test]
fn invalid_transform_in_retained_history_is_never_published() {
    let (_root, core, id, track) = setup();
    core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    state["schemaVersion"] = json!(7);
    let mut invalid = state.clone();
    invalid["tracks"][1]["items"][0]["transform2d"] = fixture()["identity"].clone();
    invalid["tracks"][1]["items"][0]["transform2d"]["scaleX"] = json!(0);
    let state_bytes = serde_json::to_vec(&state).unwrap();
    let history_bytes = serde_json::to_vec(&json!({"undo":[invalid],"redo":[]})).unwrap();
    std::fs::write(dir.join("project.json"), &state_bytes).unwrap();
    std::fs::write(dir.join("history.json"), &history_bytes).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        std::fs::read(dir.join("project.json")).unwrap(),
        state_bytes
    );
    assert_eq!(
        std::fs::read(dir.join("history.json")).unwrap(),
        history_bytes
    );
}

#[derive(Debug)]
struct NativeTools {
    ffmpeg: std::path::PathBuf,
    ffprobe: std::path::PathBuf,
    font: std::path::PathBuf,
}

fn native_configuration(
    configured: [Option<std::ffi::OsString>; 3],
    required: bool,
) -> Result<Option<NativeTools>, &'static str> {
    if configured.iter().all(Option::is_none) {
        return if required {
            Err("required native tool configuration is missing")
        } else {
            Ok(None)
        };
    }
    let [Some(ffmpeg), Some(ffprobe), Some(font)] = configured else {
        return Err(
            "OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH, and OPENCUT_TEST_FONT_PATH must be configured together",
        );
    };
    Ok(Some(NativeTools {
        ffmpeg: ffmpeg.into(),
        ffprobe: ffprobe.into(),
        font: font.into(),
    }))
}

fn native_tools() -> Option<NativeTools> {
    let tools = native_configuration(
        [
            std::env::var_os("OPENCUT_FFMPEG_PATH"),
            std::env::var_os("OPENCUT_FFPROBE_PATH"),
            std::env::var_os("OPENCUT_TEST_FONT_PATH"),
        ],
        std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref() == Ok("1"),
    )
    .expect("native Transform2D configuration")?;
    assert!(
        std::fs::read(&tools.font).is_ok(),
        "configured native font must be readable"
    );
    opencut_editor_core::Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()))
        .readiness()
        .expect("configured native tools must be usable");
    Some(tools)
}

#[test]
fn native_configuration_is_explicit_and_required_mode_fails_closed() {
    assert!(
        native_configuration([None, None, None], false)
            .unwrap()
            .is_none()
    );
    assert!(native_configuration([None, None, None], true).is_err());
    for mask in 1..7 {
        let values = std::array::from_fn(|i| {
            (mask & (1 << i) != 0).then(|| std::ffi::OsString::from("configured"))
        });
        assert!(native_configuration(values, false).is_err());
    }
    let values = ["/tools/ffmpeg", "/tools/ffprobe", "/fonts/fixture.ttf"].map(|v| Some(v.into()));
    let tools = native_configuration(values, true).unwrap().unwrap();
    assert_eq!(tools.ffmpeg, std::path::PathBuf::from("/tools/ffmpeg"));
    assert_eq!(tools.ffprobe, std::path::PathBuf::from("/tools/ffprobe"));
}

#[test]
fn all_visual_sources_share_affine_preview_range_and_export() {
    let Some(tools) = native_tools() else {
        return;
    };
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 128;
    project.settings.height = 96;
    project.settings.fps = 10;
    let mut ppm = b"P6\n20 12\n255\n".to_vec();
    for y in 0..12 {
        for x in 0..20 {
            ppm.extend_from_slice(&[
                if x < 8 { 255 } else { 40 },
                if y < 5 { 180 } else { 20 },
                70,
            ]);
        }
    }
    std::fs::write(dir.join("assets/source.ppm"), ppm).unwrap();
    project.assets=serde_json::from_value(json!([{"id":"source","mediaType":"image","fileName":"source.ppm","projectRelativePath":"assets/source.ppm","durationMs":null,
        "probe":{"durationMs":null,"hasAudio":false,"hasVideo":true,"formatName":null,"videoCodec":null,"videoWidth":20,"videoHeight":12,"audioCodec":null,"audioChannels":null,"audioSampleRateHz":null}}])).unwrap();
    let tone = dir.join("assets/tone.wav");
    let generated = std::process::Command::new(&tools.ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-y",
        ])
        .arg(&tone)
        .output()
        .unwrap();
    assert!(generated.status.success());
    project.assets.push(serde_json::from_value(json!({"id":"tone","hasAudio":true,"mediaType":"audio","fileName":"tone.wav","projectRelativePath":"assets/tone.wav","durationMs":1000,
        "probe":{"durationMs":1000,"hasAudio":true,"hasVideo":false,"formatName":"wav","videoCodec":null,"videoWidth":null,"videoHeight":null,"audioCodec":"pcm_s16le","audioChannels":1,"audioSampleRateHz":48000}})).unwrap());
    project.tracks[2].items = vec![serde_json::from_value(json!({"type":"media","id":"audio","assetId":"tone","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]})).unwrap()];
    let cases = [
        json!({"type":"rectangle","color":"#ff0000","width":30,"height":20,"keyframes":[]}),
        json!({"type":"solid_color","color":"#00aaee","keyframes":[]}),
        json!({"type":"text","text":"Affine W","fontSize":16,"color":"#ffffff","fontFamily":null,"keyframes":[]}),
        json!({"type":"media","assetId":"source","sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}),
        json!({"type":"caption","text":"Caption","style":{"fontSize":14,"color":"#ffffff","backgroundColor":"#445566","bottomMarginPx":10},
            "source":{"assetId":"source","providerId":"fixture","modelId":"fixture","modelVersion":null,"language":"en","generatedAtMs":1,"originalText":"Caption","confidence":null,"words":[]}}),
    ];
    for (index, (mode, mut item)) in (0..3)
        .flat_map(|mode| cases.iter().cloned().map(move |item| (mode, item)))
        .enumerate()
    {
        item["id"] = json!("visual");
        item["startMs"] = json!(0);
        item["durationMs"] = json!(1000);
        let mut transform = fixture()["valid"][1]["value"].clone();
        transform["scaleX"] = json!(0.8);
        transform["scaleY"] = json!(0.65);
        item["transform2d"] = transform;
        if mode > 0 {
            item["parent"] = json!({"scope":"root","id":"inner"});
            if mode == 2 {
                item.as_object_mut().unwrap().remove("transform2d");
            }
        }
        project.tracks[1].items = vec![serde_json::from_value(item).unwrap()];
        if mode > 0 {
            let mut ancestor = Transform2D::default();
            ancestor.position.x = 5.0;
            ancestor.position.y = 3.0;
            ancestor.scale_x = 0.8;
            ancestor.scale_y = 0.9;
            ancestor.opacity = 0.8;
            project.tracks[1].items.extend([
                serde_json::from_value(json!({"type":"group","id":"outer","startMs":100,"durationMs":800,"stackOrder":1,"transform2d":ancestor})).unwrap(),
                serde_json::from_value(json!({"type":"group","id":"inner","startMs":200,"durationMs":600,"stackOrder":2,"parent":{"scope":"root","id":"outer"},"transform2d":Transform2D::default()})).unwrap()
            ]);
        }
        let preview = renderer.render_preview(&project, &dir, 500).unwrap();
        let export = dir.join(format!("source-{index}.mp4"));
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &export,
                    width: 128,
                    height: 96,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let range = renderer
            .render_preview_range(
                &project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 128,
                    height: 96,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let decode_audio = |path: &std::path::Path| {
            let result = std::process::Command::new(&tools.ffmpeg)
                .args(["-v", "error", "-i"])
                .arg(path)
                .args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "-"])
                .output()
                .unwrap();
            assert!(result.status.success());
            result
                .stdout
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| f32::from_le_bytes(*v))
                .collect::<Vec<_>>()
        };
        let exported_audio = decode_audio(&export);
        let range_audio = decode_audio(&dir.join(&range.relative_path));
        assert!(!exported_audio.is_empty());
        assert!(exported_audio.iter().any(|v| v.abs() > 0.01));
        assert!(exported_audio.len().abs_diff(48000) <= 4800);
        assert!(exported_audio.len().abs_diff(range_audio.len()) <= 4800);
        let count = exported_audio.len().min(range_audio.len());
        let rms = (exported_audio
            .iter()
            .zip(&range_audio)
            .map(|(a, b)| f64::from(a - b).powi(2))
            .sum::<f64>()
            / count as f64)
            .sqrt();
        assert!(rms <= 0.0001, "source {index}: audio RMS {rms}");
        // Bound frames at the filter inputs: the output frame limit alone lets
        // FFmpeg 6 framesync score later frames outside the group visibility window.
        for video in [&export, &dir.join(&range.relative_path)] {
            let result = std::process::Command::new(&tools.ffmpeg)
                .args(["-v", "info", "-i"])
                .arg(dir.join(&preview.relative_path))
                .args([
                    "-ss",
                    if video.extension().is_some_and(|e| e == "png") {
                        "0"
                    } else {
                        "0.5"
                    },
                    "-i",
                ])
                .arg(video)
                .args([
                    "-lavfi",
                    "[0:v]trim=end_frame=1,setpts=PTS-STARTPTS,scale=in_range=auto:out_range=tv,format=yuv420p[a];[1:v]trim=end_frame=1,setpts=PTS-STARTPTS,scale=in_range=auto:out_range=tv,format=yuv420p[b];[a][b]ssim",
                    "-frames:v",
                    "1",
                    "-f",
                    "null",
                    "-",
                ])
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let stderr = String::from_utf8_lossy(&result.stderr);
            let score: f64 = stderr
                .split("All:")
                .last()
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert!(
                score >= 0.99,
                "source {index}, {}: SSIM {score}",
                video.display()
            );
        }
    }
}

#[test]
fn split_duplicate_and_lock_preserve_transform_rules() {
    let (_root, core, id, track) = setup();
    let created=core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let item = &created.changed_ids[0];
    core.edit(
        &id,
        1,
        operation(
            json!({"operation":"update_item","itemId":item,"transform2d":fixture()["identity"]}),
        ),
    )
    .unwrap();
    core.edit(
        &id,
        2,
        operation(json!({"operation":"split_item","itemId":item,"splitMs":500})),
    )
    .unwrap();
    let project = core.get_project(&id).unwrap();
    assert_eq!(project.tracks[1].items.len(), 2);
    assert!(
        project.tracks[1]
            .items
            .iter()
            .all(|v| v.visual_properties().transform2d.is_some())
    );
    core.edit(
        &id,
        3,
        operation(json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":1000})),
    )
    .unwrap();
    assert_eq!(core.get_project(&id).unwrap().tracks[1].items.len(), 3);
    core.edit(
        &id,
        4,
        operation(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert!(
        core.edit(
            &id,
            5,
            operation(json!({"operation":"update_item","itemId":item,"transform2d":null}))
        )
        .is_err()
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
}

#[test]
fn transition_and_audio_only_updates_are_rejected() {
    use opencut_editor_core::{MediaProbeFacts, MediaType};
    let (root, core, id, track) = setup();
    let created=core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let transition=core.edit(&id,1,operation(json!({"operation":"add_transition","trackId":track,"transitionType":"fade","fromItemId":created.changed_ids[0],"toItemId":null,"startMs":0,"durationMs":100}))).unwrap();
    assert_eq!(core.edit(&id,2,operation(json!({"operation":"update_item","itemId":transition.changed_ids[0],"transform2d":fixture()["identity"]}))).unwrap_err().code,ErrorCode::InvalidArgument);
    let source = root.path().join("media/audio.wav");
    std::fs::write(&source, b"fixture audio").unwrap();
    let imported = core
        .import_asset(
            &id,
            2,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(core.edit(&id,3,operation(json!({"operation":"item_set_z_index","itemId":transition.changed_ids[0],"zIndex":1}))).unwrap_err().code,ErrorCode::InvalidArgument);
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    let added=core.edit(&id,3,operation(json!({"operation":"add_media","trackId":audio_track,"assetId":imported.changed_ids[0],"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap();
    assert_eq!(core.edit(&id,4,operation(json!({"operation":"update_item","itemId":added.changed_ids[0],"transform2d":fixture()["identity"]}))).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(
        core.edit(
            &id,
            4,
            operation(
                json!({"operation":"item_set_z_index","itemId":added.changed_ids[0],"zIndex":1})
            )
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn display_rotated_video_preserves_extent_and_all_render_intents() {
    oriented_media_preserves_extent_and_all_render_intents(false);
}

#[test]
fn exif_images_preserve_extent_and_all_render_intents() {
    oriented_media_preserves_extent_and_all_render_intents(true);
}

// A minimal little-endian TIFF IFD containing only EXIF orientation.
fn jpeg_with_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    assert_eq!(&jpeg[..2], &[0xff, 0xd8]);
    if orientation == 0 {
        return jpeg.to_vec();
    }
    let mut exif = b"Exif\0\0II".to_vec();
    exif.extend(42_u16.to_le_bytes());
    exif.extend(8_u32.to_le_bytes());
    exif.extend(1_u16.to_le_bytes());
    exif.extend(274_u16.to_le_bytes());
    exif.extend(3_u16.to_le_bytes());
    exif.extend(1_u32.to_le_bytes());
    exif.extend(orientation.to_le_bytes());
    exif.extend([0; 6]);
    let mut result = jpeg[..2].to_vec();
    result.extend([0xff, 0xe1]);
    result.extend(u16::try_from(exif.len() + 2).unwrap().to_be_bytes());
    result.extend(exif);
    result.extend(&jpeg[2..]);
    result
}

fn oriented_media_preserves_extent_and_all_render_intents(image: bool) {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let Some(tools) = native_tools() else {
        return;
    };
    let (root, core, _, _) = setup();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let source = root.path().join(if image {
        "media/original.jpg"
    } else {
        "media/original.mp4"
    });
    let generated = if image {
        std::process::Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=red:s=40x20",
                "-vf",
                "drawbox=x=0:y=0:w=8:h=6:color=yellow:t=fill",
                "-frames:v",
                "1",
            ])
            .arg(&source)
            .output()
            .unwrap()
    } else {
        std::process::Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=red:s=40x20:r=10:d=1",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=1",
                "-vf",
                "drawbox=x=0:y=0:w=8:h=6:color=yellow:t=fill",
                "-c:v",
                "libx264",
                "-c:a",
                "aac",
                "-t",
                "1",
            ])
            .arg(&source)
            .output()
            .unwrap()
    };
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let decode = |path: &std::path::Path, audio: bool| {
        let mut command = std::process::Command::new(&tools.ffmpeg);
        command.args(["-v", "error", "-i"]).arg(path);
        if audio {
            command.args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "-"]);
        } else {
            command.args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    };
    let ssim = |frame: &std::path::Path, video: &std::path::Path| {
        let result = std::process::Command::new(&tools.ffmpeg)
            .args(["-v", "info", "-i"])
            .arg(frame)
            .args([
                "-ss",
                if video.extension().is_some_and(|e| e == "png") {
                    "0"
                } else {
                    "0.5"
                },
                "-i",
            ])
            .arg(video)
            .args([
                "-lavfi",
                "[0:v]trim=end_frame=1,setpts=PTS-STARTPTS,scale=in_range=auto:out_range=tv,format=yuv420p[a];[1:v]trim=end_frame=1,setpts=PTS-STARTPTS,scale=in_range=auto:out_range=tv,format=yuv420p[b];[a][b]ssim",
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let log = String::from_utf8_lossy(&result.stderr);
        let score: f64 = log
            .split("All:")
            .last()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            score >= 0.99,
            "SSIM {score}: {} vs {}",
            frame.display(),
            video.display()
        );
    };
    let rotations: &[f64] = if image {
        &[0., 1., 2., 3., 4., 5., 6., 7., 8.]
    } else {
        &[0., 90., 180., 270., 89.5, 45.]
    };
    for &rotation in rotations {
        let rotated = root.path().join(format!(
            "media/rotated-{rotation}.{}",
            if image { "jpg" } else { "mp4" }
        ));
        if image {
            std::fs::write(
                &rotated,
                jpeg_with_orientation(&std::fs::read(&source).unwrap(), rotation as u16),
            )
            .unwrap();
        } else {
            let result = std::process::Command::new(&tools.ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-display_rotation",
                    &rotation.to_string(),
                    "-i",
                ])
                .arg(&source)
                .args(["-c", "copy"])
                .arg(&rotated)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        let id = core
            .create_project(
                "Rotated",
                ProjectSettings {
                    width: 128,
                    height: 96,
                    fps: 10,
                },
            )
            .unwrap()
            .project_id;
        let dir = core.paths().project_dir(&id).unwrap();
        let track = core.get_project(&id).unwrap().tracks[0].id.clone();
        let facts: MediaProbeFacts = serde_json::from_value(
            serde_json::to_value(renderer.probe(&rotated).unwrap()).unwrap(),
        )
        .unwrap();
        let asset = core
            .import_asset(
                &id,
                0,
                &rotated,
                if image {
                    MediaType::Image
                } else {
                    MediaType::Video
                },
                facts,
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let item = core.edit(&id,1,operation(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0}))).unwrap().changed_ids[0].clone();
        let legacy = renderer
            .render_preview(&core.get_project(&id).unwrap(), &dir, 500)
            .unwrap();
        let bounds = |pixels: &[u8]| {
            let points: Vec<_> = pixels
                .as_chunks::<3>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, p)| p.iter().any(|v| *v > 100))
                .map(|(i, _)| (i % 128, i / 128))
                .collect();
            (
                points.iter().map(|p| p.0).max().unwrap() + 1,
                points.iter().map(|p| p.1).max().unwrap() + 1,
                points.len(),
            )
        };
        core.edit(&id,2,operation(json!({"operation":"update_item","itemId":item,"transform2d":fixture()["identity"]}))).unwrap();
        let identity = renderer
            .render_preview(&core.get_project(&id).unwrap(), &dir, 500)
            .unwrap();
        let legacy_pixels = decode(&dir.join(&legacy.relative_path), false);
        let identity_pixels = decode(&dir.join(&identity.relative_path), false);
        assert_eq!(bounds(&legacy_pixels), bounds(&identity_pixels));
        if image {
            // The legacy YUV and affine RGBA paths may round color conversion
            // differently. Require identical colored-pixel placement rather than byte equality across those paths.
            let classify = |pixels: &[u8]| {
                pixels
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|p| (p[0] > 100, p[1] > 100, p[2] > 100))
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                classify(&legacy_pixels),
                classify(&identity_pixels),
                "EXIF {rotation}: content placement"
            );
        }
        if rotation != 45.0 {
            assert_eq!(
                bounds(&identity_pixels),
                if (image && rotation >= 5.0)
                    || (!image && (rotation == 90.0 || rotation == 270.0 || rotation == 89.5))
                {
                    (20, 40, 800)
                } else {
                    (40, 20, 800)
                }
            );
        }
        let transform = fixture()["valid"][1]["value"].clone();
        let draft = core
            .create_draft(
                &id,
                3,
                vec![operation(
                    json!({"operation":"update_item","itemId":item,"transform2d":transform}),
                )],
                None,
            )
            .unwrap();
        let project_before = std::fs::read(dir.join("project.json")).unwrap();
        let history_before = std::fs::read(dir.join("history.json")).unwrap();
        let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
        let draft_frame = renderer.render_preview(&materialized, &dir, 500).unwrap();
        assert_eq!(
            std::fs::read(dir.join("project.json")).unwrap(),
            project_before
        );
        assert_eq!(
            std::fs::read(dir.join("history.json")).unwrap(),
            history_before
        );
        core.edit(
            &id,
            3,
            operation(json!({"operation":"update_item","itemId":item,"transform2d":transform})),
        )
        .unwrap();
        let project = core.get_project(&id).unwrap();
        let frame = renderer.render_preview(&project, &dir, 500).unwrap();
        assert_eq!(
            decode(&dir.join(&draft_frame.relative_path), false),
            decode(&dir.join(&frame.relative_path), false)
        );
        let output = dir.join("rotated.mp4");
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 128,
                    height: 96,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let range = renderer
            .render_preview_range(
                &project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 128,
                    height: 96,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        for video in [&output, &dir.join(&range.relative_path)] {
            ssim(&dir.join(&frame.relative_path), video);
            assert!(
                renderer
                    .probe(video)
                    .unwrap()
                    .duration_ms
                    .unwrap()
                    .abs_diff(1000)
                    <= 100
            );
        }
        if image {
            continue;
        }
        let a = decode(&output, true);
        let b = decode(&dir.join(&range.relative_path), true);
        let samples = |bytes: Vec<u8>| {
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| f32::from_le_bytes(*v))
                .collect::<Vec<_>>()
        };
        let (a, b) = (samples(a), samples(b));
        assert!(a.iter().any(|v| v.abs() > 0.01));
        assert!(a.len().abs_diff(b.len()) <= 4800);
        let rms = (a
            .iter()
            .zip(&b)
            .map(|(a, b)| f64::from(a - b).powi(2))
            .sum::<f64>()
            / a.len().min(b.len()) as f64)
            .sqrt();
        assert!(rms <= 0.0001, "rotation {rotation}: RMS {rms}");
    }
}
