use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, ExportOptions, PathPolicy, PreviewRangeOptions, Project,
    ProjectSettings, Renderer,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
        .create_project(
            "Lifecycle",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 20,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn bytes(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
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
            let b = std::fs::read(&p).unwrap();
            (p, b)
        })
        .collect()
}

#[test]
fn lifecycle_evicted_draft_base_preserves_reads_edits_conflicts_and_discard() {
    for extended in [false, true] {
        let (_root, core, id, track) = setup();
        let add = || {
            op(
                json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":8,"height":8,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}),
            )
        };
        let first = core.edit(&id, 0, add()).unwrap().changed_ids[0].clone();
        let effect = if extended {
            json!([{"type":"vignette","id":"v","amount":0}])
        } else {
            json!([])
        };
        core.edit(
            &id,
            1,
            op(json!({"operation":"update_item","itemId":first,"effects":effect})),
        )
        .unwrap();
        let draft = core
            .create_draft(
                &id,
                2,
                vec![op(
                    json!({"operation":"trim_item","itemId":first,"startMs":0,"durationMs":1100}),
                )],
                None,
            )
            .unwrap();
        core.edit(
            &id,
            2,
            op(json!({"operation":"delete_item","itemId":first})),
        )
        .unwrap();
        let second = core.edit(&id, 3, add()).unwrap().changed_ids[0].clone();
        core.edit(
            &id,
            4,
            op(json!({"operation":"update_item","itemId":second,"effects":effect})),
        )
        .unwrap();
        let mut revision = 5;
        for n in 0..110 {
            revision=core.edit(&id,revision,op(json!({"operation":"update_item","itemId":second,"color":if n%2==0 {"#00ff00"} else {"#ff0000"}}))).unwrap_or_else(|e| panic!("extended={extended}, eviction edit at {revision}: {e:?}")).revision;
        }
        let dir = core.paths().project_dir(&id).unwrap();
        let history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        assert!(
            !history["undo"]
                .as_array()
                .unwrap()
                .iter()
                .chain(history["redo"].as_array().unwrap())
                .any(|p| p["revision"] == 2)
        );
        let before = bytes(&dir);
        assert_eq!(core.get_project(&id).unwrap().revision, 115);
        assert_eq!(
            core.get_draft_state(&id, &draft.id).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            core.commit_draft(&id, &draft.id, revision)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(bytes(&dir), before);
        // Reopen through a fresh facade rather than relying on process-local state.
        let reopened = EditorCore::new(core.paths().clone());
        assert_eq!(reopened.get_project(&id).unwrap().revision, revision);
        reopened.discard_draft(&id, &draft.id).unwrap();
        reopened
            .edit(
                &id,
                revision,
                op(json!({"operation":"trim_item","itemId":second,"startMs":0,"durationMs":1200})),
            )
            .unwrap();
    }
}

#[test]
fn lifecycle_schema26_missing_draft_base_preserves_draft_and_generation() {
    let (_root, core, id, track) = setup();
    let first=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":8,"height":8,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let draft = core
        .create_draft(
            &id,
            1,
            vec![op(
                json!({"operation":"trim_item","itemId":first,"startMs":0,"durationMs":1100}),
            )],
            None,
        )
        .unwrap();
    core.edit(
        &id,
        1,
        op(json!({"operation":"delete_item","itemId":first})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let current = core.get_project(&id).unwrap();
    for name in ["project.json", "history.json"] {
        let path = dir.join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if name == "project.json" {
            value["schemaVersion"] = json!(26);
        } else {
            // Both retained sides have an unrelated revision; the draft's valid base was evicted.
            for side in ["undo", "redo"] {
                value[side] = json!([current]);
                value[side][0]["schemaVersion"] = json!(26);
                value[side][0]["revision"] = json!(0);
            }
        }
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let draft_bytes = std::fs::read(&draft_path).unwrap();
    assert_eq!(core.get_project(&id).unwrap().schema_version, 27);
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for side in ["undo", "redo"] {
        assert!(
            history[side]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| p["schemaVersion"] == 27)
        );
    }
    assert_eq!(std::fs::read(&draft_path).unwrap(), draft_bytes);
    let after = bytes(&dir);
    assert_eq!(
        core.get_draft_state(&id, &draft.id).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.commit_draft(&id, &draft.id, 2).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    core.get_project(&id).unwrap();
    assert_eq!(bytes(&dir), after);
    core.discard_draft(&id, &draft.id).unwrap();
}

fn native_renderer() -> Option<Renderer> {
    match (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) {
        (Some(ff), Some(fp)) => Some(Renderer::new(ff, fp, None)),
        _ => {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "native lifecycle tests require FFmpeg/FFprobe"
            );
            None
        }
    }
}
fn transition_project() -> Value {
    let component = transition_component();
    json!({"schemaVersion":27,"id":"transition","revision":0,"name":"Transition","createdAtMs":1,"updatedAtMs":1,"settings":{"width":64,"height":64,"fps":20},"assets":[],"fonts":{},"markers":[],"tracks":[{"id":"root","name":"Root","trackType":"overlay","items":[{"type":"component_instance","id":"instance","componentId":"c","startMs":1000,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{},"stackOrder":0,"zIndex":0}]}],"components":[component]})
}
fn transition_component() -> Value {
    json!({"id":"c","name":"C","width":64,"height":64,"durationMs":1000,"slots":[],"markers":[],"tracks":[{"id":"child","name":"Child","trackType":"overlay","items":[{"type":"rectangle","id":"red","color":"#ff0000","width":32,"height":32,"startMs":0,"durationMs":1000,"transform":{"positionX":16,"positionY":16,"scale":1,"opacity":1},"keyframes":[],"stackOrder":0,"zIndex":0},{"type":"transition","id":"fade","transitionType":"fade","fromItemId":"red","toItemId":null,"startMs":0,"durationMs":500,"stackOrder":1,"zIndex":0}]}]})
}
fn center_red(path: &Path, seconds: f64) -> u8 {
    let decoded = Command::new(std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap())
        .args(["-v", "error", "-ss", &seconds.to_string(), "-i"])
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
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    assert_eq!(decoded.stdout.len(), 64 * 64 * 3);
    decoded.stdout[(32 * 64 + 32) * 3]
}
#[test]
fn native_lifecycle_offset_transition_preserves_analytic_gain_in_all_intents() {
    let Some(renderer) = native_renderer() else {
        return;
    };
    let (_root, core, id, _track) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let value = transition_project();
    // Author the component fixture through core persistence, then obtain the effect candidate through the real draft facade.
    let mut persisted = value.clone();
    persisted["id"] = json!(id);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&persisted).unwrap(),
    )
    .unwrap();
    let legacy = core.get_project(&id).unwrap();
    let mut component = transition_component();
    component["tracks"][0]["items"][0]["effects"] =
        json!([{"type":"vignette","id":"identity","amount":0}]);
    component["operation"] = json!("component_update");
    component["componentId"] = json!("c");
    component.as_object_mut().unwrap().remove("id");
    let draft = core
        .create_draft(&id, 0, vec![op(component)], None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    let mut ext = persisted;
    ext["components"][0]["tracks"][0]["items"][0]["effects"] =
        json!([{"type":"vignette","id":"identity","amount":0}]);
    let extended: Project = serde_json::from_value(ext).unwrap();
    let before = bytes(&dir);
    for (label, project) in [
        ("legacy", &legacy),
        ("extended", &extended),
        ("draft", &candidate),
    ] {
        let frame = renderer.render_preview(project, &dir, 1250).unwrap();
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 1000,
                    end_ms: 2000,
                    width: 64,
                    height: 64,
                    fps: 20,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let output = dir.join(format!("{label}.mp4"));
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
        for (intent, path, time) in [
            ("frame", dir.join(frame.relative_path), 0.0),
            ("range", dir.join(range.relative_path), 0.25),
            ("export", output, 1.25),
        ] {
            let red = center_red(&path, time);
            assert!(
                (i16::from(red) - 127).abs() <= 4,
                "{label} {intent}: analytic half gain expected red 127, got {red}"
            );
        }
    }
    assert_eq!(bytes(&dir), before);
}

#[test]
fn native_lifecycle_scaled_nested_and_repeated_transitions_match_legacy_and_analytic_gains() {
    let Some(renderer) = native_renderer() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("previews")).unwrap();
    for (label, at, expected) in [
        ("root", 250, 127),
        ("scaled", 1250, 159),
        ("nested", 1350, 197),
        ("repeated", 1450, 140),
    ] {
        let mut value = transition_project();
        match label {
            "root" => {
                value["tracks"][0]["items"] = value["components"][0]["tracks"][0]["items"].clone()
            }
            "scaled" => value["tracks"][0]["items"][0]["timeScale"] = json!(0.75),
            "nested" => {
                let inner = json!({"type":"component_instance","id":"inner","componentId":"c","startMs":100,"trimStartMs":0,"durationMs":600,"timeScale":1.5,"slotValues":{},"stackOrder":0,"zIndex":0});
                let outer = json!({"id":"outer","name":"Outer","width":64,"height":64,"durationMs":2000,"slots":[],"markers":[],"tracks":[{"id":"nested","name":"Nested","trackType":"overlay","items":[inner]}]});
                value["components"].as_array_mut().unwrap().push(outer);
                value["tracks"][0]["items"][0]["componentId"] = json!("outer");
                value["tracks"][0]["items"][0]["timeScale"] = json!(0.5);
                value["tracks"][0]["items"][0]["durationMs"] = json!(2000);
            }
            "repeated" => {
                value["tracks"][0]["items"][0]["parent"] = json!({"scope":"root","id":"group"});
                let items = value["tracks"][0]["items"].as_array_mut().unwrap();
                items.push(json!({"type":"group","id":"group","startMs":1000,"durationMs":1000,"stackOrder":1,"zIndex":0}));
                items.push(json!({"type":"repeater","id":"copies","startMs":1000,"durationMs":1200,"stackOrder":2,"zIndex":0,"repeater":{"source":{"scope":"root","id":"group"},"copies":1,"timeOffsetMs":200,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}));
            }
            _ => unreachable!(),
        }
        let legacy: Project = serde_json::from_value(value.clone()).unwrap();
        let target = if label == "root" {
            &mut value["tracks"][0]["items"][0]
        } else {
            &mut value["components"][0]["tracks"][0]["items"][0]
        };
        target["effects"] = json!([{"type":"vignette","id":"identity","amount":0}]);
        let extended: Project = serde_json::from_value(value).unwrap();
        for (variant, project) in [("legacy", &legacy), ("extended", &extended)] {
            let frame = renderer.render_preview(project, root.path(), at).unwrap();
            let red = center_red(&root.path().join(frame.relative_path), 0.0);
            assert!(
                (i16::from(red) - expected).abs() <= 5,
                "{label} {variant}: expected {expected}, got {red}"
            );
            let range = renderer
                .render_preview_range(
                    project,
                    root.path(),
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: project.duration_ms(),
                        width: 64,
                        height: 64,
                        fps: 20,
                        include_audio: false,
                    },
                    |_| {},
                )
                .unwrap();
            let output = root.path().join(format!("{label}-{variant}.mp4"));
            renderer
                .export_video(
                    project,
                    root.path(),
                    ExportOptions {
                        output: &output,
                        width: 64,
                        height: 64,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            for (intent, path) in [
                ("range", root.path().join(range.relative_path)),
                ("export", output),
            ] {
                let red = center_red(&path, at as f64 / 1000.0);
                assert!(
                    (i16::from(red) - expected).abs() <= 5,
                    "{label} {variant} {intent}: expected {expected}, got {red}"
                );
            }
        }
    }
}

#[test]
fn native_lifecycle_encoder_failure_cleans_workspace_and_preserves_destination_and_state() {
    if native_renderer().is_none() {
        return;
    }
    let (root, core, id, track) = setup();
    core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"color":"#ff0000","width":8,"height":8,"startMs":0,"durationMs":100,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let mut project = core.get_project(&id).unwrap();
    let mut value = serde_json::to_value(&project).unwrap();
    value["tracks"][1]["items"][0]["effects"] =
        json!([{"type":"vignette","id":"identity","amount":0}]);
    project = serde_json::from_value(value).unwrap();
    let helper = root.path().join(if cfg!(windows) {
        "encoder.exe"
    } else {
        "encoder"
    });
    let compiled = Command::new("rustc")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/failing_visual_encoder.rs"))
        .arg("-o")
        .arg(&helper)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let renderer = Renderer::new(
        &helper,
        std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap(),
        None,
    );
    let dir = core.paths().project_dir(&id).unwrap();
    let before = bytes(&dir);
    let output = root.path().join("existing.mp4");
    std::fs::write(&output, b"preserve destination").unwrap();
    let error = renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: 64,
                height: 64,
                overwrite: true,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::FfmpegFailed);
    assert_eq!(error.failed_stage.as_deref(), Some("visual_prepare"));
    assert_eq!(error.ffmpeg_exit_code, Some(7));
    let excerpt = error.ffmpeg_stderr_excerpt.unwrap();
    assert!(excerpt.len() <= 4096 && !excerpt.contains("private-review"));
    assert_eq!(std::fs::read(&output).unwrap(), b"preserve destination");
    assert_eq!(bytes(&dir), before);
    assert!(!std::fs::read_dir(&dir).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-work-")
    }));
    std::fs::remove_file(helper).unwrap();
}
