use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, Project, ProjectSettings,
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
        .create_project("Stacking", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn rectangle(track: &str, alias: &str) -> Value {
    json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":alias})
}
fn assert_ordinals(project: &Project) {
    for track in &project.tracks {
        for (index, item) in track.items.iter().enumerate() {
            assert_eq!(item.visual_properties().stack_order as usize, index);
        }
    }
}

#[test]
fn canonical_stacking_payloads_are_strict() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/stacking-v1.json")).unwrap();
    for value in catalog["valid"].as_array().unwrap() {
        assert_eq!(serde_json::to_value(op(value.clone())).unwrap(), *value);
    }
    for value in catalog["invalid"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_err(),
            "{value}"
        );
    }
}

#[test]
fn created_track_alias_move_and_draft_preserve_stacking() {
    let (_root, core, id, original_track) = setup();
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"create_track","name":"New overlay","trackType":"overlay","resultAlias":"track"},
        rectangle("@track","a"), rectangle("@track","b"),
        {"operation":"track_reorder","trackId":"@track","index":0},
        {"operation":"item_set_z_index","itemId":"@a","zIndex":7}
    ])).unwrap();
    let added = core.edit_batch(&id, 0, operations).unwrap();
    let a = &added.aliases["a"];
    let b = &added.aliases["b"];
    let moved = core
        .edit(
            &id,
            1,
            op(json!({"operation":"move_item","itemId":a,"trackId":original_track,"startMs":250})),
        )
        .unwrap();
    assert!(moved.changed_ids.contains(b));
    let project = core.get_project(&id).unwrap();
    assert_ordinals(&project);
    assert_eq!(project.find_item(a).unwrap().visual_properties().z_index, 7);
    let before = serde_json::to_value(&project).unwrap();
    let draft = core
        .create_draft(
            &id,
            2,
            vec![
                op(json!({"operation":"item_set_z_index","itemId":a,"zIndex":-3})),
                op(json!({"operation":"track_reorder","trackId":original_track,"index":0})),
            ],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap();
    assert_eq!(
        materialized
            .project
            .find_item(a)
            .unwrap()
            .visual_properties()
            .z_index,
        -3
    );
    assert_ordinals(&materialized.project);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
    core.commit_draft(&id, &draft.id, 2).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(a)
            .unwrap()
            .visual_properties()
            .z_index,
        -3
    );
}

#[test]
fn stacking_aliases_revisions_rollback_and_history() {
    let (_root, core, id, track) = setup();
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!([
        rectangle(&track,"a"),rectangle(&track,"b"),rectangle(&track,"c"),
        {"operation":"item_set_z_index","itemId":"@a","zIndex":-2147483648},
        {"operation":"item_reorder","itemId":"@c","index":0},
        {"operation":"track_reorder","trackId":track,"index":0}
    ]))
    .unwrap();
    let result = core.edit_batch(&id, 0, operations).unwrap();
    assert_eq!(result.revision, 1);
    assert_eq!(result.changed_ids.len(), 4);
    let p = core.get_project(&id).unwrap();
    assert_ordinals(&p);
    assert_eq!(
        p.tracks[0].items.iter().map(|v| v.id()).collect::<Vec<_>>(),
        vec![
            result.aliases["c"].as_str(),
            result.aliases["a"].as_str(),
            result.aliases["b"].as_str()
        ]
    );
    assert_eq!(
        p.find_item(&result.aliases["a"])
            .unwrap()
            .visual_properties()
            .z_index,
        i32::MIN
    );
    let before = serde_json::to_value(&p).unwrap();
    let bad = vec![
        op(json!({"operation":"item_set_z_index","itemId":result.aliases["a"],"zIndex":7})),
        op(json!({"operation":"item_reorder","itemId":result.aliases["b"],"index":99})),
    ];
    assert_eq!(
        core.edit_batch(&id, 1, bad).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
    assert_eq!(
        core.edit(
            &id,
            0,
            op(json!({"operation":"item_reorder","itemId":result.aliases["a"],"index":0}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    core.undo(&id, 1).unwrap();
    assert!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .all(|t| t.items.is_empty())
    );
    core.redo(&id, 2).unwrap();
    let reopened = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened.tracks).unwrap(),
        serde_json::to_value(&p.tracks).unwrap()
    );
    assert_ordinals(&reopened);
    for (edit, error) in [
        (
            json!({"operation":"item_reorder","itemId":"missing","index":0}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"track_reorder","trackId":"missing","index":0}),
            ErrorCode::TrackNotFound,
        ),
        (
            json!({"operation":"track_reorder","trackId":track,"index":99}),
            ErrorCode::ValidationFailed,
        ),
    ] {
        assert_eq!(core.edit(&id, 3, op(edit)).unwrap_err().code, error);
    }
    core.edit(
        &id,
        3,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    for edit in [
        json!({"operation":"item_set_z_index","itemId":result.aliases["a"],"zIndex":0}),
        json!({"operation":"item_reorder","itemId":result.aliases["a"],"index":0}),
        json!({"operation":"track_reorder","trackId":track,"index":1}),
    ] {
        assert_eq!(
            core.edit(&id, 4, op(edit)).unwrap_err().code,
            ErrorCode::TrackLocked
        );
    }
}

#[test]
fn stacking_lifecycle_and_track_reorder_compatibility() {
    let (_root, core, id, track) = setup();
    let added = core
        .edit_batch(
            &id,
            0,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!([
                rectangle(&track, "a"),
                rectangle(&track, "b")
            ]))
            .unwrap(),
        )
        .unwrap();
    let a = &added.aliases["a"];
    core.edit(
        &id,
        1,
        op(json!({"operation":"item_set_z_index","itemId":a,"zIndex":2147483647})),
    )
    .unwrap();
    let split = core
        .edit(
            &id,
            2,
            op(json!({"operation":"split_item","itemId":a,"splitMs":500})),
        )
        .unwrap();
    assert!(split.changed_ids.contains(&added.aliases["b"]));
    let p = core.get_project(&id).unwrap();
    assert_ordinals(&p);
    assert_eq!(p.tracks[1].items[1].visual_properties().z_index, i32::MAX);
    let duplicated = core
        .edit(
            &id,
            3,
            op(json!({"operation":"duplicate_items","itemIds":[a],"offsetMs":1000})),
        )
        .unwrap();
    let p = core.get_project(&id).unwrap();
    assert_ordinals(&p);
    assert_eq!(
        p.find_item(&duplicated.changed_ids[0])
            .unwrap()
            .visual_properties()
            .z_index,
        i32::MAX
    );
    core.edit(&id, 4, op(json!({"operation":"delete_item","itemId":a})))
        .unwrap();
    assert_ordinals(&core.get_project(&id).unwrap());
    core.edit(
        &id,
        5,
        op(json!({"operation":"track_reorder","trackId":track,"index":0})),
    )
    .unwrap();
    let explicit = core.get_project(&id).unwrap().tracks;
    core.undo(&id, 6).unwrap();
    core.edit(
        &id,
        7,
        op(json!({"operation":"update_track","trackId":track,"index":0})),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(explicit).unwrap(),
        serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap()
    );
}

#[test]
fn schema_nine_requires_explicit_valid_order_and_migrates_mixed_history() {
    let (root, core, id, track) = setup();
    core.edit_batch(
        &id,
        0,
        serde_json::from_value::<Vec<BatchEditOperation>>(json!([
            rectangle(&track, "a"),
            rectangle(&track, "b")
        ]))
        .unwrap(),
    )
    .unwrap();
    let path = root.path().join("projects").join(&id).join("project.json");
    let history_path = path.with_file_name("history.json");
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for value in [
        json!(null),
        json!(-1),
        json!(0.5),
        json!(4294967296_u64),
        json!(0),
    ] {
        let mut bad = original.clone();
        bad["tracks"][1]["items"][1]["stackOrder"] = value;
        std::fs::write(&path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(core.get_project(&id).is_err());
        assert_eq!(
            serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(),
            bad
        );
    }
    for field in ["zIndex", "stackOrder"] {
        let mut bad = original.clone();
        bad["tracks"][1]["items"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(serde_json::from_value::<Project>(bad).is_err());
    }
    let mut old = original.clone();
    old["schemaVersion"] = json!(8);
    for item in old["tracks"][1]["items"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("zIndex");
        item.as_object_mut().unwrap().remove("stackOrder");
    }
    let mut oldest = old.clone();
    oldest["schemaVersion"] = json!(1);
    std::fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(
        &history_path,
        serde_json::to_vec(&json!({"undo":[oldest],"redo":[old]})).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(
        migrated.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_ordinals(&migrated);
    assert_eq!(
        serde_json::to_value(&migrated.tracks).unwrap(),
        original["tracks"]
    );
    let once = std::fs::read(&path).unwrap();
    core.get_project(&id).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), once);
    core.undo(&id, 1).unwrap();
    assert_ordinals(&core.get_project(&id).unwrap());
    core.redo(&id, 2).unwrap();
    assert_ordinals(&core.get_project(&id).unwrap());
}
