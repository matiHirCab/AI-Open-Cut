use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};
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
            "Grid",
            ProjectSettings {
                width: 160,
                height: 120,
                fps: 24,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn edit(track: &str, grid: &Value) -> EditOperation {
    serde_json::from_value(
        json!({"operation":"add_grid","trackId":track,"startMs":0,"durationMs":1000,"grid":grid}),
    )
    .unwrap()
}
fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/procedural-grids-v1.json")).unwrap()
}

#[test]
fn canonical_grid_fixtures_and_atomic_failures() {
    let (_root, core, id, track) = setup();
    let mut revision = 0;
    for fixture in catalog()["valid"].as_array().unwrap() {
        let result = core
            .edit(&id, revision, edit(&track, &fixture["grid"]))
            .unwrap();
        revision = result.revision;
        let p = core.get_project(&id).unwrap();
        let value = serde_json::to_value(p.find_item(&result.changed_ids[0]).unwrap()).unwrap();
        assert_eq!(value["type"], "grid");
        assert_eq!(
            value["grid"],
            serde_json::to_value(
                serde_json::from_value::<opencut_editor_core::GridDescriptor>(
                    fixture["grid"].clone()
                )
                .unwrap()
            )
            .unwrap()
        );
        assert!(p.assets.is_empty());
    }
    let before = files(&core, &id);
    for fixture in catalog()["invalid"].as_array().unwrap() {
        let decoded = serde_json::from_value::<EditOperation>(
            json!({"operation":"add_grid","grid":fixture["grid"],"trackId":track,"startMs":0,"durationMs":1000}),
        );
        if fixture["stage"] == "structure" {
            assert!(decoded.is_err(), "{}", fixture["id"]);
        } else {
            assert_eq!(
                core.edit(&id, revision, decoded.unwrap()).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{}",
                fixture["id"]
            );
        }
        assert_eq!(files(&core, &id), before);
    }
    let grid = catalog()["valid"][0]["grid"].clone();
    assert_eq!(
        core.edit(&id, 0, edit(&track, &grid)).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(&id, revision, edit("missing", &grid))
            .unwrap_err()
            .code,
        ErrorCode::TrackNotFound
    );
    for tail in [
        json!({"operation":"delete_item","itemId":"missing"}),
        json!({"operation":"item_set_parent","itemId":"@grid","parent":{"scope":"root","id":"missing"}}),
    ] {
        let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
            {"operation":"add_grid","grid":grid,"trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"grid"},tail
        ])).unwrap();
        assert!(core.edit_batch(&id, revision, batch).is_err());
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn grid_replacement_definitions_drafts_and_strict_raw_decoding() {
    let (_root, core, id, track) = setup();
    let grid = catalog()["valid"][0]["grid"].clone();
    let mut r = core.edit(&id, 0, edit(&track, &grid)).unwrap();
    let item_id = r.changed_ids[0].clone();
    for replacement in catalog()["valid"].as_array().unwrap() {
        r = core
            .edit(
                &id,
                r.revision,
                serde_json::from_value(
                    json!({"operation":"update_item","itemId":item_id,"grid":replacement["grid"]}),
                )
                .unwrap(),
            )
            .unwrap();
    }
    let value =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item_id).unwrap()).unwrap();
    let before = files(&core, &id);
    for patch in [
        json!({"fill":null}),
        json!({"geometry":{"type":"rectangle","width":10,"height":10}}),
        json!({"grid":{"width":0,"height":20,"pattern":grid["pattern"]}}),
    ] {
        let mut input = json!({"operation":"update_item","itemId":item_id});
        input
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        assert_eq!(
            core.edit(&id, r.revision, serde_json::from_value(input).unwrap())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    assert!(
        serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":item_id,"grid":null})
        )
        .is_err()
    );
    let draft = core
        .create_draft(&id, r.revision, vec![edit(&track, &grid)], None)
        .unwrap();
    assert_eq!(
        core.get_draft_state(&id, &draft.id).unwrap().project.tracks[1]
            .items
            .len(),
        2
    );
    assert_eq!(files(&core, &id), before);
    let mut local = value.clone();
    local["id"] = json!("local");
    local["stackOrder"] = json!(0);
    let component = json!({"operation":"component_create","name":"Grids","width":160,"height":120,"durationMs":1000,"tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[local]}]});
    let mut bad = component.clone();
    bad["tracks"][0]["hidden"] = json!(true);
    bad["tracks"][0]["items"][0]["grid"]["width"] = json!(0);
    assert!(
        core.edit(&id, r.revision, serde_json::from_value(bad).unwrap())
            .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let raw = serde_json::to_string(&component).unwrap();
    for (from, to) in [
        ("\"width\":20.0", "\"width\":0,\"width\":20.0"),
        ("\"r\":1.0", "\"r\":0,\"r\":1.0"),
    ] {
        let forged = raw.replace(from, to);
        assert_ne!(forged, raw);
        assert!(serde_json::from_str::<EditOperation>(&forged).is_err());
    }
    r = core
        .edit(&id, r.revision, serde_json::from_value(component).unwrap())
        .unwrap();
    r = core
        .edit(
            &id,
            r.revision,
            serde_json::from_value(
                json!({"operation":"update_track","trackId":track,"locked":true}),
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        core.edit(&id, r.revision, edit(&track, &grid))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
}

#[test]
fn grid_exact_mark_boundaries() {
    let (_root, core, id, track) = setup();
    let mut dot = catalog()["valid"][2]["grid"].clone();
    dot["width"] = json!(63);
    dot["height"] = json!(63);
    dot["pattern"]["spacingX"] = json!(1);
    dot["pattern"]["spacingY"] = json!(1);
    dot["pattern"]["radius"] = json!(0.25);
    let r = core.edit(&id, 0, edit(&track, &dot)).unwrap();
    let before = files(&core, &id);
    dot["width"] = json!(64);
    assert_eq!(
        core.edit(&id, r.revision, edit(&track, &dot))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
    let mut lines = catalog()["valid"][0]["grid"].clone();
    lines["width"] = json!(2047);
    lines["height"] = json!(2047);
    lines["pattern"]["spacingX"] = json!(1);
    lines["pattern"]["spacingY"] = json!(1);
    let r = core.edit(&id, r.revision, edit(&track, &lines)).unwrap();
    lines["width"] = json!(2048);
    assert_eq!(
        core.edit(&id, r.revision, edit(&track, &lines))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn grid_track_parent_aliases_visual_lifecycle_and_failures() {
    let (_root, core, id, original_track) = setup();
    let grid = catalog()["valid"][0]["grid"].clone();
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"create_track","name":"Grid track","trackType":"overlay","resultAlias":"track"},
        {"operation":"add_group","trackId":"@track","startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"add_grid","trackId":"@track","grid":grid,"startMs":0,"durationMs":1000,"parent":{"scope":"root","id":"@parent"},"resultAlias":"grid"},
        {"operation":"item_reorder","itemId":"@grid","index":0}
    ])).unwrap();
    let mut r = core.edit_batch(&id, 0, batch).unwrap();
    let p = core.get_project(&id).unwrap();
    let item = p
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, opencut_editor_core::TimelineItem::Grid(_)))
        .unwrap();
    assert!(item.visual_properties().parent.is_some());
    let item_id = item.id().to_owned();
    let keys = json!([{"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}]);
    let before = files(&core, &id);
    for input in [
        json!({"operation":"set_keyframes","itemId":item_id,"keyframes":keys}),
        json!({"operation":"set_audio","itemId":item_id,"audio":opencut_editor_core::AudioSettings::default()}),
        json!({"operation":"update_item","itemId":"missing","grid":grid}),
    ] {
        assert!(
            core.edit(&id, r.revision, serde_json::from_value(input).unwrap())
                .is_err()
        );
        assert_eq!(files(&core, &id), before);
    }
    for input in [
        json!({"operation":"update_item","itemId":item_id,"transform":{"positionX":5,"positionY":3,"scale":1,"opacity":0.8}}),
        json!({"operation":"set_keyframes","itemId":item_id,"keyframes":keys}),
        json!({"operation":"move_item","itemId":item_id,"trackId":original_track,"startMs":100}),
        json!({"operation":"trim_item","itemId":item_id,"startMs":100,"durationMs":700}),
        json!({"operation":"set_item_visibility","itemId":item_id,"hidden":true}),
        json!({"operation":"item_set_parent","itemId":item_id,"parent":null}),
    ] {
        r = core
            .edit(&id, r.revision, serde_json::from_value(input).unwrap())
            .unwrap();
    }
    let p = core.get_project(&id).unwrap();
    let item = p.find_item(&item_id).unwrap();
    assert_eq!(item.start_ms(), 100);
    assert_eq!(item.duration_ms(), 700);
    assert!(item.hidden());
    let raw = serde_json::to_value(item).unwrap();
    r = core
        .edit(
            &id,
            r.revision,
            serde_json::from_value(json!({"operation":"delete_item","itemId":item_id})).unwrap(),
        )
        .unwrap();
    r = core.undo(&id, r.revision).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item_id).unwrap()).unwrap(),
        raw
    );
    let before = files(&core, &id);
    for alias in ["@missing", "@later"] {
        let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
            {"operation":"add_grid","trackId":alias,"grid":grid,"startMs":0,"durationMs":1000},
            {"operation":"create_track","name":"Later","trackType":"overlay","resultAlias":"later"}
        ]))
        .unwrap();
        assert!(core.edit_batch(&id, r.revision, batch).is_err());
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn duplicate_grid_records_fail_in_every_persisted_container() {
    let (_root, core, id, track) = setup();
    let grid = catalog()["valid"][0]["grid"].clone();
    let edit = edit(&track, &grid);
    core.edit(&id, 0, edit.clone()).unwrap();
    let project = core.get_project(&id).unwrap();
    let item = &project.tracks[1].items[0];
    let corrupt = |text: String| {
        let out = text.replace("\"width\":20.0", "\"width\":0,\"width\":20.0");
        assert_ne!(out, text);
        out
    };
    assert!(
        serde_json::from_str::<EditOperation>(&corrupt(serde_json::to_string(&edit).unwrap()))
            .is_err()
    );
    assert!(
        serde_json::from_str::<Vec<BatchEditOperation>>(&corrupt(
            serde_json::to_string(&vec![BatchEditOperation::from(edit)]).unwrap()
        ))
        .is_err()
    );
    assert!(
        serde_json::from_str::<opencut_editor_core::TimelineItem>(&corrupt(
            serde_json::to_string(item).unwrap()
        ))
        .is_err()
    );
    assert!(
        serde_json::from_str::<opencut_editor_core::Project>(&corrupt(
            serde_json::to_string(&project).unwrap()
        ))
        .is_err()
    );
    let history = json!({"undo":[project],"redo":[]});
    assert!(
        serde_json::from_str::<opencut_editor_core::History>(&corrupt(
            serde_json::to_string(&history).unwrap()
        ))
        .is_err()
    );
}

fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
#[test]
fn grid_alias_lifecycle_and_reopen() {
    let (_root, core, id, track) = setup();
    let source = catalog()["valid"][0]["grid"].clone();
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_grid","trackId":track,"grid":source,"startMs":0,"durationMs":1000,"resultAlias":"icon"},
        {"operation":"item_set_z_index","itemId":"@icon","zIndex":2}
    ])).unwrap();
    let r = core.edit_batch(&id, 0, batch).unwrap();
    let p = core.get_project(&id).unwrap();
    let item = p.tracks[1].items[0].clone();
    let r = core.undo(&id, r.revision).unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    let mut r = core.redo(&id, r.revision).unwrap();
    assert_eq!(
        serde_json::to_value(&core.get_project(&id).unwrap().tracks[1].items[0]).unwrap(),
        serde_json::to_value(&item).unwrap()
    );
    for value in [
        json!({"operation":"split_item","itemId":item.id(),"splitMs":500}),
        json!({"operation":"duplicate_items","itemIds":[item.id()],"offsetMs":1000}),
        json!({"operation":"update_item","itemId":item.id(),"transform":{"positionX":12,"positionY":4,"scale":1,"opacity":0.5}}),
    ] {
        r = core
            .edit(&id, r.revision, serde_json::from_value(value).unwrap())
            .unwrap();
    }
    let before = files(&core, &id);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(reopened.get_project(&id).unwrap().revision, r.revision);
    assert_eq!(files(&core, &id), before);
    let value = json!({"operation":"update_item","itemId":item.id(),"fill":null});
    assert_eq!(
        core.edit(&id, r.revision, serde_json::from_value(value).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn grid_migration_current_history_and_future_rejection() {
    for version in 1..=15 {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        state["schemaVersion"] = json!(version);
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
        assert_eq!(core.get_project(&id).unwrap().schema_version, 17);
        let h: Value = serde_json::from_slice(&files(&core, &id).1).unwrap();
        assert_eq!(h["undo"][0]["schemaVersion"], 17);
        assert_eq!(h["redo"][0]["schemaVersion"], 17);
    }
    for location in ["current", "undo", "redo"] {
        let (_root, core, id, track) = setup();
        core.edit(&id, 0, edit(&track, &catalog()["valid"][0]["grid"]))
            .unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        for version in [15, 18] {
            let mut bad = state.clone();
            bad["schemaVersion"] = json!(version);
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(if location == "current" { &bad } else { &state }).unwrap(),
            )
            .unwrap();
            let h = if location == "undo" {
                json!({"undo":[bad],"redo":[]})
            } else if location == "redo" {
                json!({"undo":[],"redo":[bad]})
            } else {
                json!({"undo":[],"redo":[]})
            };
            std::fs::write(dir.join("history.json"), serde_json::to_vec(&h).unwrap()).unwrap();
            let before = files(&core, &id);
            assert!(core.get_project(&id).is_err());
            assert_eq!(files(&core, &id), before);
        }
    }
}
