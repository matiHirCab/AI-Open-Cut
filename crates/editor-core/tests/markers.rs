use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PROJECT_SCHEMA_VERSION, PathPolicy,
    ProjectSettings,
};
use serde_json::{Value, json};

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
        .create_project("Markers", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

#[test]
fn marker_relative_start_tracks_marker_and_survives_history() {
    let (_root, core, id, track) = setup();
    let edits: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"marker_create","scope":"root","name":"impact","timeMs":300,"kind":"cue","resultAlias":"cue"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_item_start_time","scope":"root","itemId":"@box","time":{"type":"marker","markerName":"impact","offsetMs":-50}}
    ])).unwrap();
    let added = core.edit_batch(&id, 0, edits).unwrap();
    let item_id = added.aliases["box"].clone();
    let marker_id = added.aliases["cue"].clone();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item_id)
            .unwrap()
            .start_ms(),
        250
    );
    core.edit(&id, 1, op(json!({"operation":"marker_update","scope":"root","markerId":marker_id,"name":"impact","timeMs":500,"kind":"cue"}))).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item_id)
            .unwrap()
            .start_ms(),
        450
    );
    assert_eq!(
        core.edit(
            &id,
            1,
            op(json!({"operation":"marker_delete","scope":"root","markerId":marker_id}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    core.undo(&id, 2).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item_id)
            .unwrap()
            .start_ms(),
        250
    );
    core.redo(&id, 3).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item_id)
            .unwrap()
            .start_ms(),
        450
    );
}

#[test]
fn ambiguous_name_and_invalid_offset_roll_back() {
    let (_root, core, id, track) = setup();
    let added = core.edit(&id, 0, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let item_id = &added.changed_ids[0];
    core.edit(&id, 1, op(json!({"operation":"marker_create","scope":"root","name":"impact","timeMs":300,"kind":"cue"}))).unwrap();
    core.edit(&id, 2, op(json!({"operation":"marker_create","scope":"root","name":"impact","timeMs":400,"kind":"cue"}))).unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(core.edit(&id, 3, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item_id,"time":{"type":"marker","markerName":"impact","offsetMs":0}}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(
        before,
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
}

#[test]
fn duplication_shifts_marker_offset_and_numeric_move_clears_it() {
    let (_root, core, id, track) = setup();
    core.edit(&id, 0, op(json!({"operation":"marker_create","scope":"root","name":"impact","timeMs":300,"kind":"cue"}))).unwrap();
    let added = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let item_id = added.changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item_id,"time":{"type":"marker","markerName":"impact","offsetMs":-50}}))).unwrap();
    let duplicate = core
        .edit(
            &id,
            3,
            op(json!({"operation":"duplicate_items","itemIds":[item_id],"offsetMs":100})),
        )
        .unwrap();
    let copy = core
        .get_project(&id)
        .unwrap()
        .find_item(&duplicate.changed_ids[0])
        .unwrap()
        .clone();
    assert_eq!(copy.start_ms(), 350);
    assert_eq!(
        serde_json::to_value(&copy).unwrap()["startTime"]["offsetMs"],
        50
    );
    core.edit(
        &id,
        4,
        op(json!({"operation":"move_item","itemId":item_id,"trackId":track,"startMs":40})),
    )
    .unwrap();
    let original = core
        .get_project(&id)
        .unwrap()
        .find_item(&item_id)
        .unwrap()
        .clone();
    assert_eq!(original.start_ms(), 40);
    assert!(
        serde_json::to_value(&original)
            .unwrap()
            .get("startTime")
            .is_none()
    );
}

#[test]
fn deleting_a_referenced_marker_and_invalid_offsets_leave_revision_unchanged() {
    let (_root, core, id, track) = setup();
    let marker = core.edit(&id, 0, op(json!({"operation":"marker_create","scope":"root","name":"hit","timeMs":300,"kind":"cue"}))).unwrap().changed_ids[0].clone();
    let item = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"hit","offsetMs":-50}}))).unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(
        core.edit(
            &id,
            3,
            op(json!({"operation":"marker_delete","scope":"root","markerId":marker}))
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(core.edit(&id, 3, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"hit","offsetMs":-400}}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(core.edit(&id, 3, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"milliseconds","valueMs":9007199254740991_u64}}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(
        before,
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
}

#[test]
fn schema_23_migrates_current_and_history_and_rejects_backdated_markers() {
    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut old = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    old["schemaVersion"] = json!(23);
    old.as_object_mut().unwrap().remove("markers");
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[old.clone()],"redo":[old.clone()]})).unwrap(),
    )
    .unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    let migrated = reopened.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, PROJECT_SCHEMA_VERSION);
    assert!(migrated.markers.is_empty());
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(history["undo"][0]["schemaVersion"], PROJECT_SCHEMA_VERSION);
    assert_eq!(history["redo"][0]["markers"], json!([]));

    for location in ["current", "undo", "redo"] {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let mut bad = current.clone();
        bad["schemaVersion"] = json!(23);
        bad["markers"] = json!([{"id":"m1","name":"hit","scope":"root","timeMs":1,"kind":"cue"}]);
        let project = if location == "current" {
            &bad
        } else {
            &current
        };
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(project).unwrap(),
        )
        .unwrap();
        let history = match location {
            "undo" => json!({"undo":[bad],"redo":[]}),
            "redo" => json!({"undo":[],"redo":[bad]}),
            _ => json!({"undo":[],"redo":[]}),
        };
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&history).unwrap(),
        )
        .unwrap();
        let before = (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        );
        assert!(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .is_err()
        );
        assert_eq!(
            before,
            (
                std::fs::read(dir.join("project.json")).unwrap(),
                std::fs::read(dir.join("history.json")).unwrap()
            )
        );
    }
}

#[test]
fn persisted_marker_limits_and_duplicate_ids_fail_closed() {
    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let marker = |index: usize| json!({"id":format!("marker{index}"),"name":"beat","scope":"root","timeMs":100,"kind":"cue"});
    for markers in [vec![marker(0), marker(0)], (0..=4096).map(marker).collect()] {
        let mut bad = original.clone();
        bad["markers"] = json!(markers);
        let bytes = serde_json::to_vec(&bad).unwrap();
        std::fs::write(dir.join("project.json"), &bytes).unwrap();
        assert_eq!(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
    }
}

#[test]
fn component_markers_resolve_locally_and_respect_component_bounds() {
    let (_root, core, id, track) = setup();
    let root_item = core.edit(&id, 0, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let mut local_item = serde_json::to_value(
        core.get_project(&id)
            .unwrap()
            .find_item(&root_item)
            .unwrap(),
    )
    .unwrap();
    local_item["id"] = json!("local_item");
    let component = core.edit(&id, 1, op(json!({"operation":"component_create","name":"Local","width":100,"height":100,"durationMs":1000,"tracks":[{"id":"local_track","name":"Local","trackType":"overlay","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":[local_item]}]}))).unwrap().changed_ids[0].clone();
    let scope = format!("component:{component}");
    core.edit(&id, 2, op(json!({"operation":"marker_create","scope":"root","name":"beat","timeMs":50,"kind":"cue"}))).unwrap();
    core.edit(&id, 3, op(json!({"operation":"marker_create","scope":scope,"name":"beat","timeMs":300,"kind":"cue"}))).unwrap();
    core.edit(&id, 4, op(json!({"operation":"set_item_start_time","scope":scope,"itemId":"local_item","time":{"type":"marker","markerName":"beat","offsetMs":-50}}))).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().components[0].tracks[0].items[0].start_ms(),
        250
    );
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(core.edit(&id, 5, op(json!({"operation":"set_item_start_time","scope":scope,"itemId":"local_item","time":{"type":"marker","markerName":"beat","offsetMs":700}}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(
        before,
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
    let mut replacement =
        serde_json::to_value(&core.get_project(&id).unwrap().components[0]).unwrap();
    replacement.as_object_mut().unwrap().remove("id");
    replacement.as_object_mut().unwrap().remove("markers");
    replacement["componentId"] = json!(component);
    replacement["operation"] = json!("component_update");
    replacement["name"] = json!("Renamed");
    core.edit(&id, 5, op(replacement)).unwrap();
    let current = core.get_project(&id).unwrap();
    assert_eq!(current.components[0].markers.len(), 1);
    assert_eq!(current.components[0].tracks[0].items[0].start_ms(), 250);
}

#[test]
fn trim_retains_or_clears_expression_and_split_clears_both_halves() {
    let (_root, core, id, track) = setup();
    core.edit(&id, 0, op(json!({"operation":"marker_create","scope":"root","name":"hit","timeMs":300,"kind":"cue"}))).unwrap();
    let item = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":200,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"hit","offsetMs":-50}}))).unwrap();
    core.edit(
        &id,
        3,
        op(json!({"operation":"trim_item","itemId":item,"startMs":250,"durationMs":180})),
    )
    .unwrap();
    assert!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item).unwrap())
            .unwrap()
            .get("startTime")
            .is_some()
    );
    core.edit(
        &id,
        4,
        op(json!({"operation":"trim_item","itemId":item,"startMs":260,"durationMs":170})),
    )
    .unwrap();
    assert!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item).unwrap())
            .unwrap()
            .get("startTime")
            .is_none()
    );
    core.edit(&id, 5, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"hit","offsetMs":-40}}))).unwrap();
    let result = core
        .edit(
            &id,
            6,
            op(json!({"operation":"split_item","itemId":item,"splitMs":300})),
        )
        .unwrap();
    let project = core.get_project(&id).unwrap();
    for id in &result.changed_ids {
        assert!(
            serde_json::to_value(project.find_item(id).unwrap())
                .unwrap()
                .get("startTime")
                .is_none()
        );
    }
}

#[test]
fn component_instance_duplicate_shifts_marker_offset() {
    let (_root, core, id, track) = setup();
    let component = core.edit(&id, 0, op(json!({"operation":"component_create","name":"Empty","width":64,"height":64,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 1, op(json!({"operation":"marker_create","scope":"root","name":"beat","timeMs":300,"kind":"cue"}))).unwrap();
    let instance = core.edit(&id, 2, op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":500,"timeScale":1}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 3, op(json!({"operation":"set_item_start_time","scope":"root","itemId":instance,"time":{"type":"marker","markerName":"beat","offsetMs":-50}}))).unwrap();
    let duplicate = core.edit(&id, 4, op(json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":100}))).unwrap();
    let copy = core
        .get_project(&id)
        .unwrap()
        .find_item(&duplicate.changed_ids[0])
        .unwrap()
        .clone();
    assert_eq!(copy.start_ms(), 350);
    assert_eq!(
        serde_json::to_value(copy).unwrap()["startTime"]["offsetMs"],
        50
    );
}

#[test]
fn moved_marker_changes_preview_draft_range_and_export_pixels() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let ffmpeg = std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into());
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap_or_else(|| "ffprobe".into());
    let Ok(help) = std::process::Command::new(&ffmpeg)
        .args(["-h", "full"])
        .output()
    else {
        return;
    };
    if !String::from_utf8_lossy(&help.stdout).contains("-filter_complex_script") {
        return;
    }
    let (_root, core, id, track) = setup();
    let marker = core.edit(&id, 0, op(json!({"operation":"marker_create","scope":"root","name":"hit","timeMs":300,"kind":"cue"}))).unwrap().changed_ids[0].clone();
    let item = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":64,"height":64,"color":"#ff0000","transform":{"positionX":32,"positionY":32,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"hit","offsetMs":-50}}))).unwrap();
    let mut before = core.get_project(&id).unwrap();
    before.settings.width = 64;
    before.settings.height = 64;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let visible = renderer.render_preview(&before, &dir, 300).unwrap();

    let move_marker = op(
        json!({"operation":"marker_update","scope":"root","markerId":marker,"name":"hit","timeMs":500,"kind":"cue"}),
    );
    let draft = core
        .create_draft(&id, 3, vec![move_marker.clone()], None)
        .unwrap();
    let mut draft_state = core.get_draft_state(&id, &draft.id).unwrap().project;
    draft_state.settings.width = 64;
    draft_state.settings.height = 64;
    let draft_empty = renderer.render_preview(&draft_state, &dir, 300).unwrap();
    assert_ne!(
        std::fs::read(dir.join(&visible.relative_path)).unwrap(),
        std::fs::read(dir.join(&draft_empty.relative_path)).unwrap()
    );
    core.discard_draft(&id, &draft.id).unwrap();
    core.edit(&id, 3, move_marker).unwrap();
    let mut after = core.get_project(&id).unwrap();
    after.settings.width = 64;
    after.settings.height = 64;
    let empty = renderer.render_preview(&after, &dir, 300).unwrap();
    let moved_visible = renderer.render_preview(&after, &dir, 500).unwrap();
    assert_eq!(
        std::fs::read(dir.join(&draft_empty.relative_path)).unwrap(),
        std::fs::read(dir.join(&empty.relative_path)).unwrap()
    );
    assert_eq!(
        std::fs::read(dir.join(&visible.relative_path)).unwrap(),
        std::fs::read(dir.join(&moved_visible.relative_path)).unwrap()
    );

    let range = renderer
        .render_preview_range(
            &after,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 550,
                width: 64,
                height: 64,
                fps: 30,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let export_path = dir.join("marker-export.mp4");
    renderer
        .export_video(
            &after,
            &dir,
            ExportOptions {
                output: &export_path,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let center_red = |path: &std::path::Path, second: &str| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", second, "-i"])
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
        output.stdout[(32 * 64 + 32) * 3]
    };
    for path in [dir.join(&range.relative_path), export_path] {
        assert!(center_red(&path, "0.30") < 60);
        assert!(center_red(&path, "0.50") > 180);
    }
}

#[test]
fn malformed_marker_timing_fails_before_render_output_inspection() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let (_root, core, id, track) = setup();
    core.edit(&id, 0, op(json!({"operation":"marker_create","scope":"root","name":"beat","timeMs":300,"kind":"cue"}))).unwrap();
    let item = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"set_item_start_time","scope":"root","itemId":item,"time":{"type":"marker","markerName":"beat","offsetMs":-50}}))).unwrap();
    let mut malformed = core.get_project(&id).unwrap();
    malformed.markers.clear();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
    assert_eq!(
        renderer
            .render_preview(&malformed, &dir, 250)
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        renderer
            .render_preview_range(
                &malformed,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 300,
                    width: 64,
                    height: 64,
                    fps: 30,
                    include_audio: false
                },
                |_| {}
            )
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    let destination = dir.join("existing.mp4");
    std::fs::write(&destination, b"existing output").unwrap();
    assert_eq!(
        renderer
            .export_video(
                &malformed,
                &dir,
                ExportOptions {
                    output: &destination,
                    width: 64,
                    height: 64,
                    overwrite: false
                },
                |_| {}
            )
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(std::fs::read(destination).unwrap(), b"existing output");
}
