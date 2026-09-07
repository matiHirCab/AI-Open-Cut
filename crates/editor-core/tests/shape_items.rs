use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, Paint, PathPolicy, ProjectSettings,
    ShapeGeometry, Stroke, TimelineItem, validate_shape,
};
use serde_json::{Value, json};
fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/shape-items-v1.json")).unwrap()
}
fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
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
            "Shapes",
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
fn create(track: &str) -> Value {
    let mut v = catalog()["valid"][0]["value"].clone();
    v["trackId"] = json!(track);
    v
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let p = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(p.join("project.json")).unwrap(),
        std::fs::read(p.join("history.json")).unwrap(),
    )
}

#[test]
fn canonical_shapes_roundtrip_and_reject_at_documented_stage() {
    assert!(
        serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":"shape","geometry":null})
        )
        .is_err()
    );
    let (_root, core, id, track) = setup();
    let mut revision = 0;
    for f in catalog()["valid"].as_array().unwrap() {
        let mut v = f["value"].clone();
        v["trackId"] = json!(track);
        let r = core.edit(&id, revision, op(v.clone())).unwrap();
        revision = r.revision;
        let p = core.get_project(&id).unwrap();
        let item = serde_json::to_value(p.find_item(&r.changed_ids[0]).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_value::<ShapeGeometry>(item["geometry"].clone()).unwrap(),
            serde_json::from_value::<ShapeGeometry>(v["geometry"].clone()).unwrap(),
            "{f}"
        );
        assert_eq!(
            serde_json::from_value::<Option<Paint>>(item["fill"].clone()).unwrap(),
            serde_json::from_value::<Option<Paint>>(v["fill"].clone()).unwrap()
        );
        assert_eq!(
            serde_json::from_value::<Option<Stroke>>(item["stroke"].clone()).unwrap(),
            serde_json::from_value::<Option<Stroke>>(v["stroke"].clone()).unwrap()
        );
        assert!(item["transform2d"].is_object());
        assert_eq!(item["type"], "shape");
        for key in ["geometry", "fill", "stroke"] {
            let mut missing = item.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<TimelineItem>(missing).is_err(),
                "missing {key}"
            );
        }
    }
    for f in catalog()["invalid"].as_array().unwrap() {
        let mut v = f["value"].clone();
        v["trackId"] = json!(track);
        let before = files(&core, &id);
        let decoded = serde_json::from_value::<EditOperation>(v);
        if f["stage"] == "structural" {
            assert!(decoded.is_err(), "{f}");
        } else {
            assert_eq!(
                core.edit(&id, revision, decoded.unwrap()).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{f}"
            );
        }
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn shape_numeric_and_collection_boundaries() {
    let mut g = catalog()["valid"][0]["value"]["geometry"].clone();
    for (v, valid) in [
        (f64::MIN_POSITIVE, true),
        (16384.0, true),
        (0.0, false),
        (16384.001, false),
        (-1.0, false),
    ] {
        g["width"] = json!(v);
        assert_eq!(
            serde_json::from_value::<ShapeGeometry>(g.clone())
                .unwrap()
                .validate()
                .is_ok(),
            valid
        );
    }
    let invalid = ShapeGeometry::Rectangle {
        width: f64::NAN,
        height: 1.0,
    };
    assert!(invalid.validate().is_err());
    for n in [2, 3, 4096, 4097] {
        let g = json!({"type":"polygon","points":vec![json!({"x":0,"y":0});n]});
        assert_eq!(
            serde_json::from_value::<ShapeGeometry>(g)
                .unwrap()
                .validate()
                .is_ok(),
            (3..=4096).contains(&n)
        );
    }
    for n in [2, 3, 2048, 2049] {
        let mut g = catalog()["valid"][5]["value"]["geometry"].clone();
        g["pointCount"] = json!(n);
        assert_eq!(
            serde_json::from_value::<ShapeGeometry>(g)
                .unwrap()
                .validate()
                .is_ok(),
            (3..=2048).contains(&n)
        );
    }
    let f = catalog()["valid"][0]["value"].clone();
    let geometry = serde_json::from_value(f["geometry"].clone()).unwrap();
    let fill: Option<Paint> = serde_json::from_value(f["fill"].clone()).unwrap();
    let mut stroke: Stroke = serde_json::from_value(f["stroke"].clone()).unwrap();
    stroke.width = f64::INFINITY;
    assert!(validate_shape(&geometry, &fill, &Some(stroke)).is_err());
}

#[test]
fn transactional_shape_aliases_failures_and_history() {
    let (_root, core, id, track) = setup();
    let mut shape = create(&track);
    shape["resultAlias"] = json!("shape");
    let operations = vec![
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"group"}),
        shape,
        json!({"operation":"item_set_parent","itemId":"@shape","parent":{"scope":"root","id":"@group"}}),
        json!({"operation":"item_set_z_index","itemId":"@shape","zIndex":-2}),
    ];
    let r = core
        .edit_batch(
            &id,
            0,
            operations
                .into_iter()
                .map(|v| serde_json::from_value::<BatchEditOperation>(v).unwrap())
                .collect(),
        )
        .unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(p.revision, 1);
    let shape_id = p.tracks[1].items[1].id().to_owned();
    let before = files(&core, &id);
    for (v, code) in [
        (
            json!({"operation":"update_item","itemId":shape_id,"geometry":{"type":"ellipse","width":0,"height":1}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":"missing","fill":null}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"item_set_parent","itemId":shape_id,"parent":{"scope":"root","id":"missing"}}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(core.edit(&id, r.revision, op(v)).unwrap_err().code, code);
        assert_eq!(files(&core, &id), before);
    }
    assert_eq!(
        core.edit(&id, 0, op(create(&track))).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let batch: Vec<BatchEditOperation> = vec![
        op(create(&track)).into(),
        op(json!({"operation":"delete_item","itemId":"missing"})).into(),
    ];
    assert!(core.edit_batch(&id, 1, batch).is_err());
    assert_eq!(files(&core, &id), before);
    let r = core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    let r = core.redo(&id, r.revision).unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(p.tracks[1].items[1].id(), shape_id);
    assert_eq!(p.revision, r.revision);
    let r=core.edit(&id,r.revision,op(json!({"operation":"update_item","itemId":shape_id,"geometry":{"type":"ellipse","width":25,"height":15},"stroke":null}))).unwrap();
    let r = core
        .edit(
            &id,
            r.revision,
            op(json!({"operation":"split_item","itemId":shape_id,"splitMs":500})),
        )
        .unwrap();
    assert_eq!(r.changed_ids.len(), 2);
    let r = core
        .edit(
            &id,
            r.revision,
            op(json!({"operation":"duplicate_items","itemIds":[shape_id],"offsetMs":0})),
        )
        .unwrap();
    assert_eq!(r.changed_ids.len(), 1);
    let r = core
        .edit(
            &id,
            r.revision,
            op(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    let before = files(&core, &id);
    assert_eq!(
        core.edit(&id, r.revision, op(create(&track)))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(&core, &id), before);
}

#[test]
fn schema14_migrates_current_history_and_rejects_old_shape_in_any_snapshot() {
    for version in 1..=13 {
        let (_root, core, id, _track) = setup();
        let p = core.paths().project_dir(&id).unwrap();
        let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        state["schemaVersion"] = json!(version);
        std::fs::write(p.join("project.json"), serde_json::to_vec(&state).unwrap()).unwrap();
        std::fs::write(
            p.join("history.json"),
            serde_json::to_vec(&json!({"undo":[state.clone()],"redo":[state]})).unwrap(),
        )
        .unwrap();
        assert_eq!(core.get_project(&id).unwrap().schema_version, 16);
        let before = files(&core, &id);
        core.get_project(&id).unwrap();
        assert_eq!(files(&core, &id), before);
        let h: Value = serde_json::from_slice(&before.1).unwrap();
        assert_eq!(h["undo"][0]["schemaVersion"], 16);
        assert_eq!(h["redo"][0]["schemaVersion"], 16);
    }
    for location in ["current", "undo", "redo"] {
        let (_root, core, id, track) = setup();
        core.edit(&id, 0, op(create(&track))).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut old = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        old["schemaVersion"] = json!(13);
        if location == "current" {
            std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
        } else {
            let h = if location == "undo" {
                json!({"undo":[old],"redo":[]})
            } else {
                json!({"undo":[],"redo":[old]})
            };
            std::fs::write(dir.join("history.json"), serde_json::to_vec(&h).unwrap()).unwrap();
        }
        let before = files(&core, &id);
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn shape_lifecycle_drafts_and_scoped_components_preserve_semantics() {
    let (_root, core, id, track) = setup();
    let r = core.edit(&id, 0, op(create(&track))).unwrap();
    let item = r.changed_ids[0].clone();
    let mut revision = r.revision;
    for value in [
        json!({"operation":"move_item","itemId":item,"trackId":track,"startMs":100}),
        json!({"operation":"trim_item","itemId":item,"startMs":150,"durationMs":500}),
        json!({"operation":"set_item_visibility","itemId":item,"hidden":true}),
    ] {
        revision = core.edit(&id, revision, op(value)).unwrap().revision;
    }
    let saved = core.get_project(&id).unwrap();
    let shape = saved.find_item(&item).unwrap();
    assert_eq!(shape.start_ms(), 150);
    assert_eq!(shape.duration_ms(), 500);
    assert!(shape.hidden());
    let before = files(&core, &id);
    let draft = core.create_draft(&id, revision, vec![op(json!({"operation":"set_item_visibility","itemId":item,"hidden":false})),op(json!({"operation":"update_item","itemId":item,"geometry":{"type":"ellipse","width":30,"height":15}}))],None).unwrap();
    let state = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert!(!state.find_item(&item).unwrap().hidden());
    assert_eq!(files(&core, &id), before);
    revision = core
        .commit_draft(&id, &draft.id, revision)
        .unwrap()
        .revision;
    assert!(
        !core
            .get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .hidden()
    );
    let mut local = serde_json::to_value(state.find_item(&item).unwrap()).unwrap();
    local["id"] = json!("local-shape");
    local["stackOrder"] = json!(0);
    let component = core.edit(&id,revision,op(json!({"operation":"component_create","name":"Shapes","width":160,"height":120,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[local]}]}))).unwrap();
    revision = component.revision;
    let component_id = &component.changed_ids[0];
    revision = core.edit(&id,revision,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component_id,"startMs":0,"durationMs":500,"trimStartMs":0,"timeScale":2}))).unwrap().revision;
    let p = core.get_project(&id).unwrap();
    assert!(matches!(
        p.components[0].tracks[0].items[0],
        TimelineItem::Shape(_)
    ));
    let before = files(&core, &id);
    assert!(core.edit(&id,revision,op(json!({"operation":"item_set_parent","itemId":item,"parent":{"scope":component_id,"id":"local-shape"}}))).is_err());
    assert_eq!(files(&core, &id), before);
    assert!(
        core.edit(
            &id,
            revision,
            op(json!({"operation":"set_audio","itemId":item,"audio":opencut_editor_core::AudioSettings::default()}))
        )
        .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let mut future = serde_json::to_value(p).unwrap();
    future["schemaVersion"] = json!(17);
    let dir = core.paths().project_dir(&id).unwrap();
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&future).unwrap(),
    )
    .unwrap();
    let before = files(&core, &id);
    assert!(core.get_project(&id).is_err());
    assert_eq!(files(&core, &id), before);
}

#[test]
fn old_schema_and_invalid_shapes_in_unused_hidden_definitions_never_rewrite() {
    for version in [13, 14] {
        let (_root, core, id, track) = setup();
        core.edit(&id, 0, op(create(&track))).unwrap();
        let mut p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let mut shape = p["tracks"][1]["items"][0].clone();
        shape["hidden"] = json!(true);
        if version == 14 {
            shape["geometry"]["width"] = json!(0);
        }
        p["tracks"][1]["items"] = json!([]);
        p["components"] = json!([{"id":"unused","name":"Unused","width":160,"height":120,"durationMs":1000,"slots":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","hidden":true,"items":[shape]}]}]);
        p["schemaVersion"] = json!(version);
        let dir = core.paths().project_dir(&id).unwrap();
        std::fs::write(dir.join("project.json"), serde_json::to_vec(&p).unwrap()).unwrap();
        let before = files(&core, &id);
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn shape_legacy_keyframes_and_mixed_migration_history() {
    let (_root, core, id, track) = setup();
    let r = core.edit(&id, 0, op(create(&track))).unwrap();
    let item = &r.changed_ids[0];
    let keys = json!([{"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},{"property":"position","timeMs":1000,"value":{"type":"position","x":20,"y":10},"easing":"linear"}]);
    let edit = op(json!({"operation":"set_keyframes","itemId":item,"keyframes":keys}));
    assert_eq!(
        core.edit(&id, 1, edit.clone()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    core.edit(&id, 2, edit).unwrap();
    let r = core
        .edit(
            &id,
            3,
            op(json!({"operation":"split_item","itemId":item,"splitMs":500})),
        )
        .unwrap();
    let p = core.get_project(&id).unwrap();
    let right = p.find_item(&r.changed_ids[1]).unwrap();
    assert_eq!(right.keyframes()[0].time_ms, 0);
    assert_eq!(
        right.keyframes()[0].value,
        opencut_editor_core::KeyframeValue::Position { x: 10.0, y: 5.0 }
    );

    let (_root, core, id, _track) = setup();
    let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    state["schemaVersion"] = json!(13);
    let snapshot = |version| {
        let mut v = state.clone();
        v["schemaVersion"] = json!(version);
        v
    };
    let dir = core.paths().project_dir(&id).unwrap();
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&state).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.join("history.json"),serde_json::to_vec(&json!({"undo":[snapshot(6),snapshot(12),snapshot(14)],"redo":[snapshot(9),snapshot(13)]})).unwrap()).unwrap();
    core.get_project(&id).unwrap();
    let (_, bytes) = files(&core, &id);
    let h: Value = serde_json::from_slice(&bytes).unwrap();
    for snapshot in h["undo"]
        .as_array()
        .unwrap()
        .iter()
        .chain(h["redo"].as_array().unwrap())
    {
        assert_eq!(snapshot["schemaVersion"], 16);
    }
}

#[test]
fn raw_shape_edits_reject_duplicate_vector_fields() {
    for color in [
        r#"{"r":1,"r":1,"g":0,"b":0,"a":1}"#,
        r#"{"r":2,"r":1,"g":0,"b":0,"a":1}"#,
    ] {
        let raw = format!(
            r#"{{"operation":"add_shape","trackId":"overlay","startMs":0,"durationMs":1000,"geometry":{{"type":"ellipse","width":2,"height":2}},"fill":{{"type":"solid","color":{color}}},"stroke":null}}"#
        );
        assert!(serde_json::from_str::<EditOperation>(&raw).is_err());
        assert!(serde_json::from_str::<BatchEditOperation>(&raw).is_err());
    }
}

#[test]
fn raw_shapes_reject_duplicates_in_current_and_retained_documents() {
    for target in ["current", "undo", "redo", "component"] {
        let (_root, core, id, track) = setup();
        core.edit(&id, 0, op(create(&track))).unwrap();
        let mut value = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        if target == "component" {
            let item = value["tracks"][1]["items"][0].clone();
            value["tracks"][1]["items"] = json!([]);
            value["components"] = json!([{"id":"local","name":"Local","width":160,"height":120,"durationMs":1000,"tracks":[{"id":"local-track","name":"Shapes","trackType":"overlay","items":[item]}],"slots":[]}]);
        }
        let raw = serde_json::to_string(&value).unwrap();
        let raw = raw.replacen("\"r\":", "\"r\":2,\"r\":", 1);
        assert!(
            serde_json::from_str::<opencut_editor_core::Project>(&raw).is_err(),
            "{target}"
        );
        let dir = core.paths().project_dir(&id).unwrap();
        if target == "undo" || target == "redo" {
            let history = format!(
                r#"{{"undo":{},"redo":{}}}"#,
                if target == "undo" {
                    format!("[{raw}]")
                } else {
                    "[]".into()
                },
                if target == "redo" {
                    format!("[{raw}]")
                } else {
                    "[]".into()
                }
            );
            std::fs::write(dir.join("history.json"), history).unwrap();
        } else {
            std::fs::write(dir.join("project.json"), raw).unwrap();
        }
        let before = files(&core, &id);
        assert!(core.get_project(&id).is_err(), "{target}");
        assert_eq!(before, files(&core, &id));
    }
}

#[test]
fn raw_shape_nested_records_and_update_fields_stay_strict() {
    let valid = create("overlay");
    let color = r#"{"r":1,"r":1,"g":0,"b":0,"a":1}"#;
    let point = r#"{"x":0,"x":0,"y":0}"#;
    let fills = [
        format!(r#"{{"type":"solid","color":{color}}}"#),
        format!(
            r#"{{"type":"linearGradient","start":{point},"end":{{"x":10,"y":0}},"stops":[{{"offset":0,"color":{{"r":1,"g":0,"b":0,"a":1}}}},{{"offset":1,"color":{{"r":1,"g":0,"b":0,"a":1}}}}]}}"#
        ),
        r#"{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":10,"y":0},"stops":[{"offset":0,"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":0,"b":0,"a":1}}]}"#.to_owned(),
    ];
    for fill in fills {
        for operation in ["add_shape", "update_item"] {
            let mut v = valid.clone();
            v["operation"] = json!(operation);
            if operation == "update_item" {
                v = json!({"operation":"update_item","itemId":"shape","fill":"PLACEHOLDER"});
            } else {
                v["fill"] = json!("PLACEHOLDER");
            }
            let raw = serde_json::to_string(&v)
                .unwrap()
                .replace("\"PLACEHOLDER\"", &fill);
            assert!(
                serde_json::from_str::<EditOperation>(&raw).is_err(),
                "{raw}"
            );
        }
    }
    for (field,value) in [("geometry",format!(r#"{{"type":"path","path":{{"fillRule":"nonzero","commands":[{{"type":"moveTo","to":{point}}}]}}}}"#)),("geometry",r#"{"type":"roundedRectangle","width":2,"height":2,"radii":{"topLeft":0,"topLeft":0,"topRight":0,"bottomLeft":0,"bottomRight":0}}"#.into()),("stroke",r#"{"paint":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"width":1,"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}"#.into())] {
        let raw=format!(r#"{{"operation":"update_item","itemId":"shape","{field}":{value}}}"#);
        assert!(serde_json::from_str::<EditOperation>(&raw).is_err(),"{raw}");
    }
}
