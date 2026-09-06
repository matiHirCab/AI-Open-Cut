use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, Project, ProjectSettings,
    TrackType,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/component-evaluation-v1.json"
    ))
    .unwrap()
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Instances", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Overlay)
        .unwrap()
        .id
        .clone();
    (root, core, id, track)
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
fn definition() -> Value {
    json!({"operation":"component_create","name":"Leaf","width":320,"height":240,"durationMs":1000,"tracks":[]})
}

#[test]
fn canonical_instance_operations_are_closed() {
    for value in catalog()["validOperations"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_ok(),
            "{value}"
        );
    }
    for value in catalog()["invalidOperations"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn root_instances_aliases_atomic_failures_and_history() {
    let (_root, core, id, track) = setup();
    let mut leaf = definition();
    leaf["resultAlias"] = json!("leaf");
    let mut add = catalog()["validOperations"][0].clone();
    add["trackId"] = json!(track);
    add["componentId"] = json!("@leaf");
    add["resultAlias"] = json!("instance");
    let mut update = catalog()["validOperations"][1].clone();
    update["componentId"] = json!("@leaf");
    update["itemId"] = json!("@instance");
    let result = core
        .edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(json!([leaf, add, update])).unwrap(),
        )
        .unwrap();
    let instance = &result.aliases["instance"];
    let component = &result.aliases["leaf"];
    let before = files(&core, &id);
    for fixture in catalog()["semanticFailures"].as_array().unwrap() {
        let mut edit = fixture["edit"].clone();
        edit["trackId"] = json!(track);
        if edit["componentId"] == "leaf" {
            edit["componentId"] = json!(component);
        }
        let error = core.edit(&id, 1, op(edit)).unwrap_err();
        assert_eq!(serde_json::to_value(error.code).unwrap(), fixture["code"]);
        assert_eq!(files(&core, &id), before);
    }
    assert_eq!(
        core.edit(
            &id,
            1,
            op(json!({"operation":"component_delete","componentId":component}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.edit(
            &id,
            0,
            op(json!({"operation":"delete_item","itemId":instance}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let edits = json!([{"operation":"move_item","itemId":instance,"trackId":track,"startMs":100}, {"operation":"component_delete","componentId":"missing"}]);
    assert!(
        core.edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(edits).unwrap())
            .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let expected = serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap();
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().find_item(instance).is_none());
    core.redo(&id, 2).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap(),
        expected
    );
    assert_eq!(
        serde_json::to_value(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap()
                .tracks
        )
        .unwrap(),
        expected
    );
}

#[test]
fn old_schema_cannot_smuggle_root_instances() {
    let (_root, core, id, track) = setup();
    let component = core.edit(&id, 0, op(definition())).unwrap().changed_ids[0].clone();
    let mut edit = catalog()["validOperations"][0].clone();
    edit["trackId"] = json!(track);
    edit["componentId"] = json!(component);
    core.edit(&id, 1, op(edit)).unwrap();
    let mut value = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert!(serde_json::from_value::<Project>(value.clone()).is_ok());
    value["schemaVersion"] = json!(12);
    assert!(serde_json::from_value::<Project>(value).is_err());
}

#[test]
fn nested_preview_range_and_export_share_pixels() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (_root, core, id, track) = setup();
    let mut leaf = definition();
    leaf["tracks"] = catalog()["renderFixture"]["leafTracks"].clone();
    let component = core.edit(&id, 0, op(leaf)).unwrap().changed_ids[0].clone();
    let mut outer = definition();
    outer["tracks"] = json!([{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"component_instance","id":"nested","componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"transform":{"positionX":4,"positionY":4,"scale":1,"opacity":1}}]}]);
    let outer = core.edit(&id, 1, op(outer)).unwrap().changed_ids[0].clone();
    core.edit(&id,2,op(json!({"operation":"add_component_instance","trackId":track,"componentId":outer,"startMs":100,"trimStartMs":50,"durationMs":600,"timeScale":1.5}))).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    let before = files(&core, &id);
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let frame = renderer.render_preview(&project, &dir, 300).unwrap();
    let instance_id = project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, opencut_editor_core::TimelineItem::ComponentInstance(_)))
        .unwrap()
        .id();
    let draft=core.create_draft(&id,3,vec![op(json!({"operation":"component_instance_update","itemId":instance_id,"componentId":outer,"startMs":100,"trimStartMs":50,"durationMs":600,"timeScale":1.5}))],None).unwrap();
    let mut draft_project = core.get_draft_state(&id, &draft.id).unwrap().project;
    draft_project.settings = project.settings.clone();
    let draft_frame = renderer.render_preview(&draft_project, &dir, 300).unwrap();

    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 300,
                end_ms: 600,
                width: 64,
                height: 64,
                fps: 30,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let export = dir.join("instances.mp4");
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
    let decode = |path: &std::path::Path, time: &str| {
        let out = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", time, "-i"])
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
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout.len(), 64 * 64 * 3);
        out.stdout
    };
    let preview = decode(&dir.join(frame.relative_path), "0");
    let draft_pixels = decode(&dir.join(draft_frame.relative_path), "0");
    assert_eq!(preview, draft_pixels, "materialized draft preview differs");
    let range = decode(&dir.join(range.relative_path), "0");
    let exported = decode(&export, "0.3");
    for pixels in [&preview, &range, &exported] {
        let inside = (12 * 64 + 20) * 3;
        assert!(
            pixels[inside] > 220 && pixels[inside + 1] < 25,
            "missing transformed red rectangle"
        );
        assert!(pixels[0] < 25, "unexpected background");
    }
    for pixels in [&range, &exported] {
        let n = preview.len() as f64;
        let mean_a = preview.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
        let mean_b = pixels.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
        let (va, vb, cov) =
            preview
                .iter()
                .zip(pixels)
                .fold((0.0, 0.0, 0.0), |(va, vb, cov), (a, b)| {
                    let a = f64::from(*a) - mean_a;
                    let b = f64::from(*b) - mean_b;
                    (va + a * a, vb + b * b, cov + a * b)
                });
        let c1 = (0.01_f64 * 255.0).powi(2);
        let c2 = (0.03_f64 * 255.0).powi(2);
        let ssim = ((2.0 * mean_a * mean_b + c1) * (2.0 * cov / n + c2))
            / ((mean_a.powi(2) + mean_b.powi(2) + c1) * ((va + vb) / n + c2));
        assert!(ssim >= 0.99, "preview/export SSIM {ssim}");
    }
    assert_eq!(files(&core, &id), before);
}

#[test]
fn retimed_audio_preserves_pitch_and_preview_export_pcm() {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (root, core, id, track) = setup();
    let source = root.path().join("media/tone.wav");
    let generated = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=2",
        ])
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let asset = core
        .import_asset(
            &id,
            0,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(2000),
                has_audio: true,
                audio_sample_rate_hz: Some(48000),
                audio_channels: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut leaf = definition();
    leaf["durationMs"] = json!(2000);
    leaf["tracks"] = json!([{"id":"audio","name":"Audio","trackType":"audio","items":[{"type":"media","id":"tone","assetId":asset,"startMs":0,"durationMs":2000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}]}]);
    let component = core.edit(&id, 1, op(leaf)).unwrap().changed_ids[0].clone();
    let mut placement = catalog()["audioFixture"]["instanceEdit"].clone();
    placement["trackId"] = json!(track);
    placement["componentId"] = json!(component);
    core.edit(&id, 2, op(placement)).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 600,
                width: 64,
                height: 64,
                fps: 30,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let export = dir.join("audio.mp4");
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
    let decode = |path: &std::path::Path| {
        let out = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(path)
            .args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        out.stdout
            .chunks_exact(4)
            .map(|v| f32::from_le_bytes(v.try_into().unwrap()) as f64)
            .collect::<Vec<_>>()
    };
    let a = decode(&dir.join(range.relative_path));
    let b = decode(&export);
    assert!(a.len() >= 28_800 && b.len() >= 28_800);
    let rms = (a
        .iter()
        .zip(&b)
        .take(28_800)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / 28_800.0)
        .sqrt();
    assert!(rms <= 0.0001, "preview/export PCM RMS {rms}");
    assert!(
        a[..3840].iter().all(|v| v.abs() < 0.001),
        "audio starts before instance"
    );
    let steady = &a[9600..24000];
    let crossings = steady
        .windows(2)
        .filter(|v| v[0] <= 0.0 && v[1] > 0.0)
        .count();
    let frequency = crossings as f64 / 0.3;
    assert!(
        (frequency - 440.0).abs() < 5.0,
        "pitch changed to {frequency} Hz"
    );
}

#[test]
fn generic_edits_locks_drafts_and_unsupported_operations() {
    let (_root, core, id, track) = setup();
    let component = core.edit(&id, 0, op(definition())).unwrap().changed_ids[0].clone();
    let add = json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1});
    let before = files(&core, &id);
    let draft = core
        .create_draft(&id, 1, vec![op(add.clone())], None)
        .unwrap();
    assert_eq!(files(&core, &id), before);
    let materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
    let instance = core.edit(&id, 1, op(add)).unwrap().changed_ids[0].clone();
    let actual = core.get_project(&id).unwrap();
    let expected = materialized
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, opencut_editor_core::TimelineItem::ComponentInstance(_)))
        .unwrap();
    let mut a = serde_json::to_value(expected).unwrap();
    let mut b = serde_json::to_value(actual.find_item(&instance).unwrap()).unwrap();
    a.as_object_mut().unwrap().remove("id");
    b.as_object_mut().unwrap().remove("id");
    assert_eq!(a, b);
    for edit in [
        json!({"operation":"split_item","itemId":instance,"splitMs":500}),
        json!({"operation":"set_keyframes","itemId":instance,"keyframes":[]}),
        json!({"operation":"add_transition","trackId":track,"fromItemId":instance,"startMs":0,"durationMs":100,"transitionType":"fade"}),
    ] {
        let before = files(&core, &id);
        assert_eq!(
            core.edit(&id, 2, op(edit)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    let duplicate = core
        .edit(
            &id,
            2,
            op(json!({"operation":"duplicate_items","itemIds":[instance],"offsetMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(
        &id,
        3,
        op(json!({"operation":"move_item","itemId":duplicate,"trackId":track,"startMs":2000})),
    )
    .unwrap();
    core.edit(&id,4,op(json!({"operation":"update_item","itemId":duplicate,"transform":{"positionX":12,"positionY":7,"scale":2,"opacity":0.5},"hidden":true}))).unwrap();
    core.edit(
        &id,
        5,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let before = files(&core, &id);
    assert_eq!(core.edit(&id,6,op(json!({"operation":"component_instance_update","itemId":duplicate,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1}))).unwrap_err().code,ErrorCode::TrackLocked);
    assert_eq!(files(&core, &id), before);
    core.edit(
        &id,
        6,
        op(json!({"operation":"update_track","trackId":track,"locked":false})),
    )
    .unwrap();
    core.edit(
        &id,
        7,
        op(json!({"operation":"delete_item","itemId":duplicate})),
    )
    .unwrap();
    assert!(
        core.get_project(&id)
            .unwrap()
            .find_item(&instance)
            .is_some()
    );
}

#[test]
fn native_rich_text_runs_render_colors_and_missing_style_fails_cleanly() {
    use opencut_editor_core::Renderer;
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let font = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fonts/DejaVuSans.ttf");
    let (_root, core, id, track) = setup();
    let mut leaf = definition();
    leaf["tracks"] = json!([{"id":"local","name":"Text","trackType":"overlay","items":[{"type":"text","id":"text","text":"Base","fontSize":20,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]}]}]);
    leaf["slots"] = json!([{"id":"title","name":"Title","kind":"rich_text","required":true,"defaultValue":{"type":"rich_text","value":{"runs":[{"text":"Red","color":"#ff0000"},{"text":"Blue","color":"#0000ff"}]}},"binding":{"targetLayerId":"text","property":"text.document"},"constraints":{}}]);
    let component = core.edit(&id, 0, op(leaf)).unwrap().changed_ids[0].clone();
    let instance=core.edit(&id,1,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1}))).unwrap().changed_ids[0].clone();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 128;
    project.settings.height = 64;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    let frame = renderer.render_preview(&project, &dir, 0).unwrap();
    let pixels = std::process::Command::new(&ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(dir.join(frame.relative_path))
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"])
        .output()
        .unwrap();
    assert!(pixels.status.success());
    assert!(
        pixels
            .stdout
            .chunks_exact(3)
            .filter(|p| p[0] > 150 && p[1] < 50 && p[2] < 50)
            .count()
            > 20
    );
    assert!(
        pixels
            .stdout
            .chunks_exact(3)
            .filter(|p| p[2] > 150 && p[1] < 50 && p[0] < 50)
            .count()
            > 20
    );
    core.edit(&id,2,op(json!({"operation":"component_instance_update","itemId":instance,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{"title":{"type":"rich_text","value":{"runs":[{"text":"Bold","bold":true}]}}}}))).unwrap();
    let before = files(&core, &id);
    assert_eq!(
        renderer
            .render_preview(&core.get_project(&id).unwrap(), &dir, 0)
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnavailable
    );
    assert_eq!(files(&core, &id), before);
    assert!(std::fs::read_dir(&dir).unwrap().all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-work-")
    }));
}

fn migration_component_document(
    core: &EditorCore,
    id: &str,
    version: u32,
    transform: Value,
    hidden: bool,
) -> Value {
    let mut document = serde_json::to_value(core.get_project(id).unwrap()).unwrap();
    document["schemaVersion"] = json!(version);
    document["components"] = json!([
        {"id":"leaf","name":"Leaf","width":320,"height":240,"durationMs":1000,"tracks":[],"slots":[]},
        {"id":"unused","name":"Unused","width":320,"height":240,"durationMs":1000,"slots":[],"tracks":[
            {"id":"local","name":"Local","trackType":"overlay","hidden":hidden,"items":[
                {"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":0,"hidden":hidden,"transform":transform}
            ]}
        ]}
    ]);
    document
}

#[test]
fn source_schema_transform_rejection_preserves_current_and_history() {
    for version in [11, 12] {
        for location in ["current", "undo", "redo"] {
            for (field, value) in [
                ("positionX", 1.0),
                ("positionY", 1.0),
                ("scale", 2.0),
                ("opacity", 0.5),
            ] {
                for hidden in [false, true] {
                    let (_root, core, id, _) = setup();
                    let default = json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
                    let valid =
                        migration_component_document(&core, &id, 12, default.clone(), false);
                    let mut transform = default;
                    transform[field] = json!(value);
                    let invalid =
                        migration_component_document(&core, &id, version, transform, hidden);
                    let mut current = valid.clone();
                    let mut history = json!({"undo":[valid.clone()],"redo":[valid]});
                    if location == "current" {
                        current = invalid;
                    } else {
                        history[location][0] = invalid;
                    }
                    let dir = core.paths().project_dir(&id).unwrap();
                    std::fs::write(
                        dir.join("project.json"),
                        serde_json::to_vec(&current).unwrap(),
                    )
                    .unwrap();
                    std::fs::write(
                        dir.join("history.json"),
                        serde_json::to_vec(&history).unwrap(),
                    )
                    .unwrap();
                    let before = files(&core, &id);
                    let error = core.get_project(&id).unwrap_err();
                    assert_eq!(
                        error.code,
                        ErrorCode::InvalidArgument,
                        "{version} {location} {field} {hidden}"
                    );
                    assert!(!error.retryable);
                    assert_eq!(files(&core, &id), before);
                }
            }
        }
    }
}

#[test]
fn source_schema_valid_transforms_preserve_content_and_reopen() {
    for version in [11, 12, 13] {
        for use_transform2d in [false, true] {
            let (_root, core, id, _) = setup();
            let mut transform = json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
            if version == 13 && !use_transform2d {
                transform = json!({"positionX":4,"positionY":5,"scale":2,"opacity":0.5});
            }
            let mut document = migration_component_document(&core, &id, version, transform, true);
            if use_transform2d {
                let t = opencut_editor_core::Transform2D {
                    scale_x: 2.0,
                    ..Default::default()
                };
                document["components"][1]["tracks"][0]["items"][0]["transform2d"] =
                    serde_json::to_value(t).unwrap();
            }
            // Canonicalize historical defaults before comparing the version-only migration.
            let typed: Project = serde_json::from_value(document).unwrap();
            let mut expected = serde_json::to_value(&typed).unwrap();
            expected["schemaVersion"] = json!(13);
            let mut older = expected.clone();
            older["schemaVersion"] = json!(11);
            older["components"][1]["tracks"][0]["items"][0]["transform"] =
                json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
            let older: Project = serde_json::from_value(older).unwrap();
            let mut expected_older = serde_json::to_value(&older).unwrap();
            expected_older["schemaVersion"] = json!(13);
            let dir = core.paths().project_dir(&id).unwrap();
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(&typed).unwrap(),
            )
            .unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&json!({"undo":[older],"redo":[typed]})).unwrap(),
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
                expected
            );
            let persisted: Value = serde_json::from_slice(&files(&core, &id).1).unwrap();
            assert_eq!(
                persisted,
                json!({"undo":[expected_older],"redo":[expected]})
            );
            let before = files(&core, &id);
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
                expected
            );
            assert_eq!(files(&core, &id), before);
        }
    }
}
