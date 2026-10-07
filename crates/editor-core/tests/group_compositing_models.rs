use opencut_editor_core::{EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.file_name().unwrap() != ".lock" {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, &mut result);
    result
}
fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
#[test]
fn malformed_current_clips_reject_root_components_and_retained_history_without_writes() {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let policy = || {
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
    };
    let core = EditorCore::new(policy());
    let id = core
        .create_project(
            "Closed clips",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let group = core
        .edit(
            &id,
            0,
            operation(
                json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
            ),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(&id, 1, operation(json!({"operation":"component_create","name":"Unused","width":64,"height":64,"durationMs":1000,
        "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"group","id":"local-group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0}]}]}))).unwrap();
    core.edit(
        &id,
        2,
        operation(json!({"operation":"update_item","itemId":group,"zIndex":1})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let stable = inventory(&dir);
    let current: Value = serde_json::from_slice(&stable[&dir.join("project.json")]).unwrap();
    let history: Value = serde_json::from_slice(&stable[&dir.join("history.json")]).unwrap();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-compositing-v1.json")).unwrap();
    for case in catalog["clipCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["accepted"] == false)
    {
        let bad = &case["value"];
        for location in ["root", "component", "undo", "redo"] {
            for (path, bytes) in &stable {
                std::fs::write(path, bytes).unwrap();
            }
            let mut project = current.clone();
            let mut snapshots = history.clone();
            match location {
                "root" => project["tracks"][1]["items"][0]["clip"] = bad.clone(),
                "component" => {
                    project["components"][0]["tracks"][0]["items"][0]["clip"] = bad.clone()
                }
                key => {
                    let mut snapshot = current.clone();
                    snapshot["tracks"][1]["items"][0]["clip"] = bad.clone();
                    snapshots[key] = json!([snapshot]);
                }
            }
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(&project).unwrap(),
            )
            .unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&snapshots).unwrap(),
            )
            .unwrap();
            let before = inventory(&dir);
            let reopened = EditorCore::new(policy());
            let error = reopened.get_project(&id).unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "{location}: {bad}: {error}"
            );
            assert!(!error.retryable);
            assert_eq!(inventory(&dir), before, "{location}: {bad}");
        }
    }
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
            "Aggregate models",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn normalized_stack(value: Value) -> Value {
    serde_json::to_value(
        serde_json::from_value::<Vec<opencut_editor_core::VisualEffect>>(value).unwrap(),
    )
    .unwrap()
}
fn stack() -> Value {
    normalized_stack(json!([
        {"type":"screen_flash","id":"flash","startMs":50,"durationMs":900,"intensity":0.8,"color":{"r":1,"g":0.5,"b":0,"a":0.6}},
        {"type":"particle_overlay","id":"particles","count":4,"seed":173,"radiusPx":2,"speedPxPerSecond":10,"lifetimeMs":997,"color":{"r":1,"g":0,"b":0,"a":0.7}}
    ]))
}
fn saved_item(core: &EditorCore, id: &str, item: &str) -> Value {
    serde_json::to_value(core.get_project(id).unwrap().find_item(item).unwrap()).unwrap()
}
#[test]
fn controlled_duplicate_and_ungroup_keep_fields_scoped_and_restore_history() {
    let (_root, core, id, track) = setup();
    let stack = stack();
    let created=core.edit_batch(&id,0,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"owner"},
        {"operation":"update_item","itemId":"@owner","clip":{"type":"composition_bounds"},"effects":stack},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":3,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@owner"}}
    ])).unwrap()).unwrap();
    let owner = &created.aliases["owner"];
    let child = &created.aliases["child"];
    let original = saved_item(&core, &id, owner);
    let copied = core
        .edit(
            &id,
            1,
            operation(json!({"operation":"duplicate_items","itemIds":[owner,child],"offsetMs":0})),
        )
        .unwrap();
    let copy = &copied.changed_ids[0];
    let copied_child_id = &copied.changed_ids[1];
    core.edit_batch(&id,2,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"item_set_parent","itemId":copied_child_id,"parent":{"scope":"root","id":copy}},
        {"operation":"update_item","itemId":copy,"zIndex":1}
    ])).unwrap()).unwrap();
    let copy_state = saved_item(&core, &id, copy);
    assert_eq!(copy_state["clip"], original["clip"]);
    assert_eq!(copy_state["effects"], stack);
    let copied_child = core.get_project(&id).unwrap().tracks[1]
        .items
        .iter()
        .find(|item| {
            item.id() != child
                && item
                    .visual_properties()
                    .parent
                    .as_ref()
                    .is_some_and(|p| p.id == *copy)
        })
        .unwrap()
        .id()
        .to_owned();
    let child_state = saved_item(&core, &id, &copied_child);
    core.edit(
        &id,
        3,
        operation(json!({"operation":"group_ungroup","groupId":copy})),
    )
    .unwrap();
    assert!(core.get_project(&id).unwrap().find_item(copy).is_none());
    assert!(
        saved_item(&core, &id, &copied_child)
            .get("parent")
            .is_none()
    );
    assert_eq!(saved_item(&core, &id, owner), original);
    core.undo(&id, 4).unwrap();
    assert_eq!(saved_item(&core, &id, copy), copy_state);
    assert_eq!(saved_item(&core, &id, &copied_child), child_state);
    core.redo(&id, 5).unwrap();
    assert!(core.get_project(&id).unwrap().find_item(copy).is_none());
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
}
#[test]
fn clip_and_ordered_overlay_lifecycle_preserves_aliases_drafts_history_null_clear_and_reopen() {
    let (root, core, id, track) = setup();
    let stack = stack();
    let result=core.edit_batch(&id,0,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"owner"},
        {"operation":"update_item","itemId":"@owner","clip":{"type":"composition_bounds"},"effects":stack},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":3,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@owner"}}
    ])).unwrap()).unwrap();
    let owner = &result.aliases["owner"];
    let child = &result.aliases["child"];
    let initial = saved_item(&core, &id, owner);
    assert_eq!(initial["clip"], json!({"type":"composition_bounds"}));
    assert_eq!(initial["effects"], stack);
    let child_before = saved_item(&core, &id, child);
    core.edit(
        &id,
        1,
        operation(json!({"operation":"update_item","itemId":owner,"zIndex":2})),
    )
    .unwrap();
    assert_eq!(saved_item(&core, &id, owner)["clip"], initial["clip"]);
    assert_eq!(saved_item(&core, &id, owner)["effects"], stack);
    let reversed = normalized_stack(json!([stack[1], stack[0]]));
    let dir = core.paths().project_dir(&id).unwrap();
    let before = inventory(&dir);
    let draft = core
        .create_draft(
            &id,
            2,
            vec![operation(
                json!({"operation":"update_item","itemId":owner,"clip":null,"effects":reversed}),
            )],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap();
    let candidate = serde_json::to_value(materialized.project.find_item(owner).unwrap()).unwrap();
    assert!(candidate.get("clip").is_none());
    assert_eq!(candidate["effects"], reversed);
    assert_eq!(saved_item(&core, &id, owner)["effects"], stack);
    let after = inventory(&dir);
    for (path, bytes) in before {
        assert_eq!(after[&path], bytes);
    }
    core.commit_draft(&id, &draft.id, 2).unwrap();
    assert!(saved_item(&core, &id, owner).get("clip").is_none());
    assert_eq!(saved_item(&core, &id, owner)["effects"], reversed);
    core.undo(&id, 3).unwrap();
    assert_eq!(saved_item(&core, &id, owner)["clip"], initial["clip"]);
    assert_eq!(saved_item(&core, &id, owner)["effects"], stack);
    core.redo(&id, 4).unwrap();
    assert_eq!(saved_item(&core, &id, owner)["effects"], reversed);
    assert!(saved_item(&core, &id, owner).get("clip").is_none());
    core.edit(
        &id,
        5,
        operation(json!({"operation":"update_item","itemId":owner,"effects":[]})),
    )
    .unwrap();
    assert!(saved_item(&core, &id, owner).get("effects").is_none());
    assert_eq!(saved_item(&core, &id, child), child_before);
    let reopened = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
    assert_eq!(reopened.get_project(&id).unwrap().schema_version, 37);
}
#[test]
fn canonical_overlay_endpoints_closed_fields_and_atomic_later_batch_failures() {
    let (_root, core, id, track) = setup();
    let group = core
        .edit(
            &id,
            0,
            operation(
                json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
            ),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-compositing-v1.json")).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    for case in catalog["effectCases"].as_array().unwrap() {
        let revision = core.get_project(&id).unwrap().revision;
        let before = inventory(&dir);
        let parsed = serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":group,"effects":[case["value"]]}),
        );
        let accepted = parsed.is_ok_and(|edit| core.edit(&id, revision, edit).is_ok());
        assert_eq!(
            accepted,
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
        if !accepted {
            assert_eq!(inventory(&dir), before, "{}", case["id"]);
        }
    }
    let revision = core.get_project(&id).unwrap().revision;
    let before = inventory(&dir);
    let invalid = [
        json!([stack()[0], stack()[0]]),
        json!([{ "id":"unknown","type":"unknown"}]),
        json!([{"type":"screen_flash","id":"incomplete"}]),
        json!([{"type":"screen_flash","id":"closed","startMs":0,"durationMs":1,"intensity":0,"color":{"r":0,"g":0,"b":0,"a":1},"extra":1}]),
    ];
    for effects in invalid {
        let batch = serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
            {"operation":"update_item","itemId":group,"clip":{"type":"composition_bounds"}},
            {"operation":"update_item","itemId":group,"effects":effects}
        ]));
        if let Ok(batch) = batch {
            let error = core.edit_batch(&id, revision, batch).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(!error.retryable);
        }
        assert_eq!(inventory(&dir), before);
    }
    for edit in [
        json!({"operation":"update_item","itemId":"missing","clip":null}),
        json!({"operation":"update_item","itemId":group,"clip":null}),
    ] {
        let error = core
            .edit(
                &id,
                if edit["itemId"] == "missing" {
                    revision
                } else {
                    revision - 1
                },
                operation(edit.clone()),
            )
            .unwrap_err();
        assert_eq!(
            error.code,
            if edit["itemId"] == "missing" {
                ErrorCode::ItemNotFound
            } else {
                ErrorCode::RevisionConflict
            }
        );
        assert_eq!(inventory(&dir), before);
    }
}
#[test]
fn unsupported_owner_features_and_leaf_clip_reject_without_publication_but_clear_remains_valid() {
    let (_root, core, id, track) = setup();
    let result=core.edit_batch(&id,0,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"owner"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"}
    ])).unwrap()).unwrap();
    let owner = &result.aliases["owner"];
    let leaf = &result.aliases["leaf"];
    let dir = core.paths().project_dir(&id).unwrap();
    let before = inventory(&dir);
    for fields in [
        json!({"clip":{"type":"composition_bounds"}}),
        json!({"effects":stack(),"blendMode":"multiply"}),
        json!({"effects":stack(),"motionBlur":{"shutterAngleDeg":180,"sampleCount":2}}),
        json!({"effects":stack(),"matteOnly":true}),
        json!({"effects":stack(),"matte":{"sourceId":leaf,"channel":"alpha"}}),
    ] {
        let is_clip = fields.get("clip").is_some();
        let mut edit = fields;
        edit["operation"] = json!("update_item");
        edit["itemId"] = json!(if is_clip { leaf } else { owner });
        let error = core.edit(&id, 1, operation(edit)).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(inventory(&dir), before);
    }
    core.edit(
        &id,
        1,
        operation(json!({"operation":"update_item","itemId":leaf,"clip":null})),
    )
    .unwrap();
    assert!(saved_item(&core, &id, leaf).get("clip").is_none());
    core.edit(&id,2,operation(json!({"operation":"update_item","itemId":owner,"clip":{"type":"composition_bounds"},"effects":stack()}))).unwrap();
    let before = inventory(&dir);
    let error=core.edit(&id,3,operation(json!({"operation":"set_animation_channels","itemId":owner,"animationChannels":[{"property":"effect.glow_radius","target":{"kind":"effect","scope":"root","id":"particles"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"hold"}]}]}))).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(inventory(&dir), before);
}

#[test]
fn authentic_schema36_adoption_migrates_complete_history_without_enabling_controls_or_changing_identity()
 {
    let (root, core, id, track) = setup();
    let group = core
        .edit(
            &id,
            0,
            operation(
                json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
            ),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(&id,1,operation(json!({"operation":"component_create","name":"Unused","width":33,"height":19,"durationMs":1000,
        "tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[{"type":"group","id":"local-group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0}]}]}))).unwrap();
    core.edit(
        &id,
        2,
        operation(json!({"operation":"update_item","itemId":group,"zIndex":3})),
    )
    .unwrap();
    core.undo(&id, 3).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    current["schemaVersion"] = json!(36);
    for key in ["undo", "redo"] {
        for snapshot in history[key].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(36);
        }
    }
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
    let before = inventory(&dir);
    let reopened = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    reopened.get_project(&id).unwrap();
    let mut adopted: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    assert_eq!(adopted["schemaVersion"], 37);
    adopted["schemaVersion"] = json!(36);
    assert_eq!(adopted, current);
    let mut adopted_history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for key in ["undo", "redo"] {
        for snapshot in adopted_history[key].as_array_mut().unwrap() {
            assert_eq!(snapshot["schemaVersion"], 37);
            snapshot["schemaVersion"] = json!(36);
        }
    }
    assert_eq!(adopted_history, history);
    let after = inventory(&dir);
    for (path, bytes) in before {
        if ![dir.join("project.json"), dir.join("history.json")].contains(&path) {
            assert_eq!(after[&path], bytes);
        }
    }
    let stable = inventory(&dir);
    reopened.get_project(&id).unwrap();
    assert_eq!(inventory(&dir), stable);
}
#[test]
fn schema36_introduced_controls_reject_current_hidden_unused_and_history_before_any_rewrite() {
    let (_root, core, id, track) = setup();
    core.edit(
        &id,
        0,
        operation(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
    )
    .unwrap();
    core.edit(&id,1,operation(json!({"operation":"component_create","name":"Unused","width":33,"height":19,"durationMs":1000,
        "tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[{"type":"group","id":"local-group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":0}]}]}))).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let stable = inventory(&dir);
    let current: Value = serde_json::from_slice(&stable[&dir.join("project.json")]).unwrap();
    let history: Value = serde_json::from_slice(&stable[&dir.join("history.json")]).unwrap();
    for location in ["root", "hidden", "unused", "undo", "redo"] {
        for control in [
            json!({"clip":{"type":"composition_bounds"}}),
            json!({"effects":[{"id":"old-kind","type":"gaussian_blur","radiusPx":0}]}),
            json!({"effects":[stack()[0]]}),
            json!({"effects":[stack()[1]]}),
        ] {
            for (path, bytes) in &stable {
                std::fs::write(path, bytes).unwrap();
            }
            let mut project = current.clone();
            let mut snapshots = history.clone();
            let target = match location {
                "unused" => &mut project["components"][0]["tracks"][0]["items"][0],
                "undo" | "redo" => {
                    let mut snapshot = current.clone();
                    snapshot["schemaVersion"] = json!(36);
                    snapshots[location] = json!([snapshot]);
                    &mut snapshots[location][0]["tracks"][1]["items"][0]
                }
                _ => &mut project["tracks"][1]["items"][0],
            };
            for (key, value) in control.as_object().unwrap() {
                target[key] = value.clone();
            }
            if location == "hidden" {
                target["hidden"] = json!(true);
            }
            if !["undo", "redo"].contains(&location) {
                project["schemaVersion"] = json!(36);
            }
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(&project).unwrap(),
            )
            .unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&snapshots).unwrap(),
            )
            .unwrap();
            let before = inventory(&dir);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "{location}/{control}: {error}"
            );
            assert!(!error.retryable);
            assert_eq!(inventory(&dir), before, "{location}/{control}");
        }
    }
}
