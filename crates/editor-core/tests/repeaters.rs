use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PROJECT_SCHEMA_VERSION, PathPolicy,
    ProjectSettings,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/repeaters-v1.json")).unwrap()
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
        .create_project("Repeaters", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

fn shape(track: &str) -> EditOperation {
    serde_json::from_value(
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
        "geometry":{"type":"rectangle","width":40,"height":20},
        "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}),
    )
    .unwrap()
}

fn retarget_batch(track: &str) -> Value {
    let mut source = serde_json::to_value(shape(track)).unwrap();
    source["resultAlias"] = json!("source");
    let mut replacement = source.clone();
    replacement["resultAlias"] = json!("replacement");
    let mut descriptor = catalog()["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!("@source");
    let add = json!({"operation":"add_repeater","trackId":track,"startMs":0,
        "durationMs":1000,"repeater":descriptor,"resultAlias":"copies"});
    descriptor["source"]["id"] = json!("@replacement");
    json!([source, add, replacement,
        {"operation":"update_item","itemId":"@copies","repeater":descriptor}])
}

fn authoritative_files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}

#[test]
fn effective_audio_batches_drafts_and_reopen_preserve_authoritative_state() {
    use opencut_editor_core::{MediaProbeFacts, MediaType};
    let (root, core, id, track) = setup();
    let mut assets = Vec::new();
    for (revision, audio) in [false, true].into_iter().enumerate() {
        let path = root.path().join(format!("media/video{revision}.mp4"));
        std::fs::write(&path, format!("fixture{revision}")).unwrap();
        assets.push(
            core.import_asset(
                &id,
                revision as u64,
                &path,
                MediaType::Video,
                MediaProbeFacts {
                    duration_ms: Some(1000),
                    has_audio: audio,
                    has_video: true,
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
                .clone(),
        );
    }
    let component = json!({"operation":"component_create","name":"Media","width":100,"height":100,"durationMs":1000,
        "resultAlias":"leaf","slots":[{"id":"asset","name":"Asset","kind":"asset","required":false,
        "binding":{"targetLayerId":"media","property":"media.asset"},"constraints":{}}],
        "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"media","id":"media",
        "assetId":assets[0],"startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},
        "keyframes":[],"zIndex":0,"stackOrder":0}]}]});
    let mut descriptor = catalog()["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!("@instance");
    let added = core.edit_batch::<BatchEditOperation>(&id,2,serde_json::from_value(json!([
        component,{"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":0,
        "durationMs":1000,"trimStartMs":0,"timeScale":1,"resultAlias":"instance"},
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor}
    ])).unwrap()).unwrap();
    let bad = json!({"operation":"component_instance_update","itemId":added.aliases["instance"],
        "componentId":added.aliases["leaf"],"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,
        "slotValues":{"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":assets[1]}}}});
    let good = json!({"operation":"component_instance_update","itemId":added.aliases["instance"],
        "componentId":added.aliases["leaf"],"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,
        "slotValues":{"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":assets[0]}}}});
    let before = authoritative_files(&core, &id);
    let draft = core
        .create_draft(
            &id,
            3,
            vec![serde_json::from_value(good.clone()).unwrap()],
            None,
        )
        .unwrap();
    core.get_draft_state(&id, &draft.id).unwrap();
    assert_eq!(
        core.create_draft(
            &id,
            3,
            vec![serde_json::from_value(bad.clone()).unwrap()],
            None
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let operations = vec![shape(&track), serde_json::from_value(bad.clone()).unwrap()];
    assert_eq!(
        core.edit_batch(&id, 3, operations.clone())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.edit_batch(&id, 0, operations).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(authoritative_files(&core, &id), before);
    core.edit(&id, 3, serde_json::from_value(good).unwrap())
        .unwrap();
    core.undo(&id, 4).unwrap();
    core.redo(&id, 5).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    let state = reopened.get_project(&id).unwrap();
    let mut corrupted = serde_json::to_value(&state).unwrap();
    corrupted["tracks"][1]["items"][0]["slotValues"] = bad["slotValues"].clone();
    let path = core.paths().project_dir(&id).unwrap().join("project.json");
    std::fs::write(path, serde_json::to_vec_pretty(&corrupted).unwrap()).unwrap();
    let corrupt_bytes = authoritative_files(&core, &id);
    assert_eq!(
        EditorCore::new(core.paths().clone())
            .get_project(&id)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(authoritative_files(&core, &id), corrupt_bytes);
}

#[test]
fn replacement_aliases_commit_once_and_preserve_literal_id_drafts() {
    let (_root, core, id, track) = setup();
    let result = core
        .edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(retarget_batch(&track)).unwrap(),
        )
        .unwrap();
    assert_eq!(result.revision, 1);
    let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(
        state["tracks"][1]["items"][1]["repeater"]["source"]["id"],
        result.aliases["replacement"]
    );
    let tracks = state["tracks"].clone();
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    core.redo(&id, 2).unwrap();
    let core = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["tracks"],
        tracks
    );
    let before = authoritative_files(&core, &id);
    let mut descriptor = state["tracks"][1]["items"][1]["repeater"].clone();
    descriptor["source"]["id"] = json!(result.aliases["source"]);
    let update =
        json!({"operation":"update_item","itemId":result.aliases["copies"],"repeater":descriptor});
    let draft = core
        .create_draft(
            &id,
            3,
            vec![serde_json::from_value(update.clone()).unwrap()],
            None,
        )
        .unwrap();
    let draft_state =
        serde_json::to_value(core.get_draft_state(&id, &draft.id).unwrap().project).unwrap();
    assert_eq!(
        draft_state["tracks"][1]["items"][1]["repeater"]["source"]["id"],
        result.aliases["source"]
    );
    assert_eq!(authoritative_files(&core, &id), before);
    let mut missing = update.clone();
    missing["repeater"]["source"]["id"] = json!("missing");
    assert_eq!(
        core.create_draft(&id, 3, vec![serde_json::from_value(missing).unwrap()], None)
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        core.edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(json!([update])).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(authoritative_files(&core, &id), before);
}

#[test]
fn replacement_alias_failures_roll_back_bytes_and_history() {
    for case in ["missing", "forward", "both", "trailing", "literal", "scope"] {
        let (_root, core, id, track) = setup();
        // Ensure both authoritative files exist before checking byte-level rollback.
        core.edit(&id, 0, shape(&track)).unwrap();
        let before = authoritative_files(&core, &id);
        let mut operations = retarget_batch(&track);
        let expected = match case {
            "missing" | "both" => {
                operations[3]["repeater"]["source"]["id"] = json!("@absent_source");
                if case == "both" {
                    operations[3]["itemId"] = json!("@absent_item");
                }
                ErrorCode::ValidationFailed
            }
            "forward" => {
                operations.as_array_mut().unwrap().swap(2, 3);
                ErrorCode::ValidationFailed
            }
            "trailing" => {
                operations
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"operation":"delete_item","itemId":"missing"}));
                ErrorCode::ItemNotFound
            }
            "scope" => {
                operations[3]["repeater"]["source"]["scope"] = json!("@source");
                ErrorCode::InvalidArgument
            }
            _ => {
                operations[3]["repeater"]["source"]["id"] = json!("missing");
                ErrorCode::ItemNotFound
            }
        };
        let error = core
            .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(operations).unwrap())
            .unwrap_err();
        assert_eq!(error.code, expected, "{case}");
        if case == "both" {
            assert!(error.message.contains("@absent_item"));
        }
        assert_eq!(authoritative_files(&core, &id), before, "{case}");
        assert_eq!(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap()
                .revision,
            1
        );
    }
}

#[test]
fn catalog_edit_alias_rollback_and_reference_protection() {
    let (_root, core, id, track) = setup();
    let descriptor = catalog()["valid"][0]["repeater"].clone();
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
         "geometry":{"type":"rectangle","width":40,"height":20},
         "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"resultAlias":"source"},
        {"operation":"add_repeater","trackId":track,"startMs":100,"durationMs":800,
         "repeater":{"source":{"scope":"root","id":"@source"},"copies":descriptor["copies"],
         "transformOffset":descriptor["transformOffset"],"opacityOffset":descriptor["opacityOffset"]},"resultAlias":"copies"},
        {"operation":"item_set_z_index","itemId":"@copies","zIndex":3}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    let project = core.get_project(&id).unwrap();
    let source_id = project.tracks[1].items[0].id().to_owned();
    assert_eq!(project.tracks[1].items.len(), 2);
    let deletion: EditOperation =
        serde_json::from_value(json!({"operation":"delete_item","itemId":source_id})).unwrap();
    assert_eq!(
        core.edit(&id, result.revision, deletion).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let failing: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,
         "repeater":{"source":{"scope":"root","id":source_id},"copies":1,
         "transformOffset":descriptor["transformOffset"],"opacityOffset":0}},
        {"operation":"delete_item","itemId":"missing"}
    ]))
    .unwrap();
    assert!(core.edit_batch(&id, result.revision, failing).is_err());
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
}

#[test]
fn strict_and_semantic_descriptor_failures() {
    let (_root, core, id, track) = setup();
    let source = core.edit(&id, 0, shape(&track)).unwrap();
    let source_id = source.changed_ids[0].clone();
    let mut base = catalog()["valid"][0]["repeater"].clone();
    base["source"]["id"] = json!(source_id);
    for fixture in catalog()["invalid"].as_array().unwrap() {
        let mut descriptor = fixture
            .get("repeater")
            .cloned()
            .unwrap_or_else(|| base.clone());
        if let Some(path) = fixture.get("path").and_then(Value::as_str) {
            let parts = path.split('.').collect::<Vec<_>>();
            let mut target = &mut descriptor;
            for part in &parts[..parts.len() - 1] {
                target = &mut target[*part];
            }
            target[parts[parts.len() - 1]] = fixture["value"].clone();
        }
        if let Some(path) = fixture.get("remove").and_then(Value::as_str) {
            let parts = path.split('.').collect::<Vec<_>>();
            let mut target = &mut descriptor;
            for part in &parts[..parts.len() - 1] {
                target = &mut target[*part];
            }
            target
                .as_object_mut()
                .unwrap()
                .remove(parts[parts.len() - 1]);
        }
        let decoded = serde_json::from_value::<EditOperation>(json!({
            "operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor
        }));
        if let Ok(edit) = decoded {
            assert_eq!(
                core.edit(&id, source.revision, edit).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{}",
                fixture["id"]
            );
        }
    }
    let raw = format!(
        r#"{{"operation":"add_repeater","trackId":"{track}","startMs":0,"durationMs":1000,"repeater":{{"source":{{"scope":"root","id":"{source_id}"}},"copies":1,"copies":2,"transformOffset":{{"position":{{"x":0,"y":0,"unit":"pixels"}},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}},"opacityOffset":0}}}}"#
    );
    assert!(serde_json::from_str::<EditOperation>(&raw).is_err());
}

#[test]
fn schema_16_current_and_history_migrate_atomically() {
    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    state["schemaVersion"] = json!(16);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[state.clone()],"redo":[state]})).unwrap(),
    )
    .unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened.get_project(&id).unwrap().schema_version,
        PROJECT_SCHEMA_VERSION
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(
        history["undo"][0]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        history["redo"][0]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    let migrated = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    let reopened_again = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened_again.get_project(&id).unwrap().schema_version,
        PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        migrated,
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap()
        )
    );
}

#[test]
fn pre_17_repeaters_and_future_versions_fail_without_rewrite() {
    for location in ["current", "undo", "redo"] {
        let (_root, core, id, track) = setup();
        let source = core.edit(&id, 0, shape(&track)).unwrap();
        let source_id = source.changed_ids[0].clone();
        let mut descriptor = catalog()["valid"][0]["repeater"].clone();
        descriptor["source"]["id"] = json!(source_id);
        let add: EditOperation = serde_json::from_value(json!({
            "operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor
        })).unwrap();
        core.edit(&id, source.revision, add).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let mut bad = state.clone();
        bad["schemaVersion"] = json!(16);
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(if location == "current" { &bad } else { &state }).unwrap(),
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

    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut future = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    future["schemaVersion"] = json!(opencut_editor_core::PROJECT_SCHEMA_VERSION + 1);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&future).unwrap(),
    )
    .unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    assert!(
        EditorCore::new(core.paths().clone())
            .get_project(&id)
            .is_err()
    );
    assert_eq!(before, std::fs::read(dir.join("project.json")).unwrap());
}

#[test]
fn hidden_unused_component_repeaters_are_version_gated() {
    let (_root, core, id, track) = setup();
    let source = core.edit(&id, 0, shape(&track)).unwrap();
    let source_id = source.changed_ids[0].clone();
    let mut descriptor = catalog()["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!(source_id);
    core.edit(&id, source.revision, serde_json::from_value(json!({
        "operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor
    })).unwrap()).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut items = state["tracks"][1]["items"].take();
    items[1]["repeater"]["source"]["scope"] = json!("component:hidden");
    state["tracks"][1]["items"] = json!([]);
    state["components"] = json!([{"id":"hidden","name":"Hidden","width":100,"height":100,"durationMs":1000,"slots":[],
        "tracks":[{"id":"local","name":"Local","trackType":"overlay","hidden":true,"items":items}]}]);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    assert!(
        EditorCore::new(core.paths().clone())
            .get_project(&id)
            .is_ok()
    );
    state["schemaVersion"] = json!(16);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    assert!(
        EditorCore::new(core.paths().clone())
            .get_project(&id)
            .is_err()
    );
}

#[test]
fn repeater_lifecycle_update_and_unsupported_edits_are_transactional() {
    let (_root, core, id, track) = setup();
    let source = core.edit(&id, 0, shape(&track)).unwrap();
    let source_id = source.changed_ids[0].clone();
    let mut descriptor = catalog()["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!(source_id);
    let added = core.edit(&id, source.revision, serde_json::from_value(json!({
        "operation":"add_repeater","trackId":track,"startMs":100,"durationMs":800,"repeater":descriptor
    })).unwrap()).unwrap();
    let repeater_id = added.changed_ids[0].clone();
    let mut replacement = catalog()["valid"][0]["repeater"].clone();
    replacement["source"]["id"] = json!(source_id);
    replacement["copies"] = json!(256);
    replacement["opacityOffset"] = json!(-1);
    let updated = core
        .edit(
            &id,
            added.revision,
            serde_json::from_value(json!({
                "operation":"update_item","itemId":repeater_id,"repeater":replacement
            }))
            .unwrap(),
        )
        .unwrap();
    let moved = core
        .edit(
            &id,
            updated.revision,
            serde_json::from_value(json!({
                "operation":"move_item","itemId":repeater_id,"trackId":track,"startMs":200
            }))
            .unwrap(),
        )
        .unwrap();
    let trimmed = core
        .edit(
            &id,
            moved.revision,
            serde_json::from_value(json!({
                "operation":"trim_item","itemId":repeater_id,"startMs":250,"durationMs":500
            }))
            .unwrap(),
        )
        .unwrap();
    let split = core
        .edit(
            &id,
            trimmed.revision,
            serde_json::from_value(json!({
                "operation":"split_item","itemId":repeater_id,"splitMs":500
            }))
            .unwrap(),
        )
        .unwrap();
    assert_eq!(split.changed_ids.len(), 2);
    let duplicate = core
        .edit(
            &id,
            split.revision,
            serde_json::from_value(json!({
                "operation":"duplicate_items","itemIds":[repeater_id],"offsetMs":1000
            }))
            .unwrap(),
        )
        .unwrap();
    assert_eq!(duplicate.changed_ids.len(), 1);
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for operation in [
        json!({"operation":"set_keyframes","itemId":repeater_id,"keyframes":[]}),
        json!({"operation":"set_audio","itemId":repeater_id,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}}),
        json!({"operation":"update_item","itemId":repeater_id,"transform2d":null}),
        json!({"operation":"add_transition","trackId":track,"fromItemId":repeater_id,"startMs":250,"durationMs":100,"transitionType":"fade"}),
    ] {
        let edit: EditOperation = serde_json::from_value(operation).unwrap();
        assert_eq!(
            core.edit(&id, duplicate.revision, edit).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    core.undo(&id, duplicate.revision).unwrap();
    let redone = core.redo(&id, duplicate.revision + 1).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
    assert_eq!(redone.revision, duplicate.revision + 2);
}
