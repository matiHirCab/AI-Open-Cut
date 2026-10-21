use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PROJECT_SCHEMA_VERSION, PathPolicy,
    Project, ProjectSettings, SlotProperty, SlotValue, TemplateSlot, TrackType,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/solid-graphic-color-slots-v1.json"
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
        .create_project(
            "Graphic palette",
            ProjectSettings {
                width: 320,
                height: 180,
                fps: 10,
            },
        )
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
    let p = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(p.join("project.json")).unwrap(),
        std::fs::read(p.join("history.json")).unwrap(),
    )
}
fn content(core: &EditorCore, id: &str) -> Value {
    let mut p = serde_json::to_value(core.get_project(id).unwrap()).unwrap();
    p.as_object_mut().unwrap().remove("revision");
    p.as_object_mut().unwrap().remove("updatedAtMs");
    p
}
fn seed(core: &EditorCore, id: &str, track: &str) -> std::collections::BTreeMap<String, String> {
    let mut create = catalog()["definition"].clone();
    let slots = create["slots"].take();
    create["slots"] = json!([]);
    create["resultAlias"] = json!("card");
    let add = |alias: &str, x: u32, values: Value| json!({"operation":"add_component_instance","trackId":track,"componentId":"@card","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"resultAlias":alias,"transform":{"positionX":x,"positionY":0,"scale":1,"opacity":1},"slotValues":values});
    core.edit_batch::<BatchEditOperation>(id,0,serde_json::from_value(json!([create,{"operation":"component_define_slots","componentId":"@card","slots":slots},add("left",0,json!({})),add("right",160,catalog()["overrides"].clone())] )).unwrap()).unwrap().aliases
}
fn update(item: &str, component: &str, values: Value) -> Value {
    json!({"operation":"component_instance_update","itemId":item,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":values})
}
fn reject(core: &EditorCore, id: &str, revision: u64, value: Value, code: ErrorCode) {
    let before = files(core, id);
    let error = core.edit(id, revision, op(value)).unwrap_err();
    assert_eq!(error.code, code, "{error:?}");
    assert_eq!(files(core, id), before);
}

#[test]
fn canonical_closed_graphic_identifiers_and_values() {
    let c = catalog();
    assert_eq!(c["projectSchemaVersion"], PROJECT_SCHEMA_VERSION);
    for (property, slot) in c["properties"]
        .as_array()
        .unwrap()
        .iter()
        .zip(c["definition"]["slots"].as_array().unwrap())
    {
        assert!(serde_json::from_value::<SlotProperty>(property.clone()).is_ok());
        assert!(serde_json::from_value::<TemplateSlot>(slot.clone()).is_ok());
        for value in c["invalidValues"].as_array().unwrap() {
            let mut invalid = slot.clone();
            invalid["defaultValue"] = value.clone();
            if serde_json::from_value::<SlotValue>(value.clone()).is_err() {
                assert!(serde_json::from_value::<TemplateSlot>(invalid).is_err());
            }
        }
    }
    for property in [
        "shape.fill",
        "svg.paint.color",
        "grid.paint.color",
        "shape.fill.color.r",
    ] {
        assert!(serde_json::from_value::<SlotProperty>(json!(property)).is_err());
    }
}

#[test]
fn incompatible_targets_and_defaults_are_atomic_even_hidden_and_unused() {
    let (_root, core, id, _track) = setup();
    let base = catalog()["definition"].clone();
    let gradient = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":40,"y":0},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]});
    for index in 0..3 {
        let mut bad = base.clone();
        bad["slots"][index]["kind"] = json!("number");
        bad["slots"][index]["defaultValue"] = json!({"type":"number","value":1});
        reject(&core, &id, 0, bad, ErrorCode::InvalidArgument);
        let mut bad = base.clone();
        bad["slots"][index]["binding"]["targetLayerId"] = json!("foreign_root_only");
        reject(&core, &id, 0, bad, ErrorCode::ItemNotFound);
        let mut bad = base.clone();
        bad["slots"][index]["binding"]["targetLayerId"] =
            json!(if index == 2 { "shape" } else { "dots" });
        reject(&core, &id, 0, bad, ErrorCode::InvalidArgument);
        let mut bad = base.clone();
        match index {
            0 => bad["tracks"][0]["items"][0]["fill"] = gradient.clone(),
            1 => bad["tracks"][0]["items"][0]["stroke"]["paint"] = gradient.clone(),
            _ => bad["tracks"][0]["items"][1]["grid"]["pattern"]["paint"] = gradient.clone(),
        };
        bad["tracks"][0]["items"][0]["hidden"] = json!(true);
        bad["tracks"][0]["items"][1]["hidden"] = json!(true);
        reject(&core, &id, 0, bad, ErrorCode::InvalidArgument);
        for value in catalog()["invalidValues"].as_array().unwrap() {
            let mut bad = base.clone();
            bad["slots"][index]["defaultValue"] = value.clone();
            if let Ok(edit) = serde_json::from_value::<EditOperation>(bad) {
                let before = files(&core, &id);
                assert_eq!(
                    core.edit(&id, 0, edit).unwrap_err().code,
                    ErrorCode::InvalidArgument
                );
                assert_eq!(files(&core, &id), before);
            }
        }
    }
    for field in ["fill", "stroke"] {
        let mut bad = base.clone();
        bad["tracks"][0]["items"][0][field] = Value::Null;
        reject(&core, &id, 0, bad, ErrorCode::InvalidArgument);
    }
    let mut duplicate = base.clone();
    let mut slot = duplicate["slots"][0].clone();
    slot["id"] = json!("duplicate");
    duplicate["slots"].as_array_mut().unwrap().push(slot);
    reject(&core, &id, 0, duplicate, ErrorCode::InvalidArgument);

    let mut svg = base.clone();
    let item = svg["tracks"][0]["items"][0].as_object_mut().unwrap();
    item.remove("geometry");
    item.remove("fill");
    item.remove("stroke");
    item.insert("type".into(), json!("svg"));
    item.insert("document".into(), json!({"version":1,"width":40,"height":40,"viewBox":[0,0,40,40],"shapes":[{"geometry":{"type":"rectangle","width":40,"height":40},"offset":{"x":0,"y":0},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}]}));
    reject(&core, &id, 0, svg, ErrorCode::InvalidArgument);
    let mut linegrid = base.clone();
    linegrid["tracks"][0]["items"][1]["grid"]["pattern"] = json!({"type":"rectangular","spacingX":20,"spacingY":20,"stroke":base["tracks"][0]["items"][0]["stroke"]});
    reject(&core, &id, 0, linegrid, ErrorCode::InvalidArgument);
}

#[test]
fn graphic_targets_never_resolve_existing_root_or_foreign_identifiers() {
    let (_root, core, id, track) = setup();
    let root_item = core.edit(&id, 0, op(json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#000000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    let mut foreign = catalog()["definition"].clone();
    foreign["slots"] = json!([]);
    foreign["tracks"][0]["items"][0]["id"] = json!("foreign_shape");
    foreign["tracks"][0]["items"][1]["id"] = json!("foreign_dots");
    core.edit(&id, 1, op(foreign)).unwrap();
    for index in 0..3 {
        for target in [
            root_item.as_str(),
            if index == 2 {
                "foreign_dots"
            } else {
                "foreign_shape"
            },
        ] {
            let mut bad = catalog()["definition"].clone();
            bad["slots"][index]["binding"]["targetLayerId"] = json!(target);
            reject(&core, &id, 2, bad, ErrorCode::ItemNotFound);
        }
    }
}

#[test]
fn aliased_independent_instances_history_drafts_locks_and_rollback() {
    let (_root, core, id, track) = setup();
    let aliases = seed(&core, &id, &track);
    let card = &aliases["card"];
    let left = &aliases["left"];
    let right = &aliases["right"];
    let before = core.get_project(&id).unwrap();
    let values = json!({"fill":{"type":"color","value":"#12AbEf"}});
    reject(
        &core,
        &id,
        0,
        update(left, card, values.clone()),
        ErrorCode::RevisionConflict,
    );
    let saved = files(&core, &id);
    let edits = json!([update(left,card,values.clone()),{"operation":"component_delete","componentId":"missing"}]);
    assert_eq!(
        core.edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(edits).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(&core, &id), saved);
    let draft = core
        .create_draft(&id, 1, vec![op(update(left, card, values.clone()))], None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(files(&core, &id), saved);
    assert_eq!(
        serde_json::to_value(&candidate.components).unwrap(),
        serde_json::to_value(&before.components).unwrap()
    );
    assert_eq!(
        serde_json::to_value(candidate.find_item(right)).unwrap(),
        serde_json::to_value(before.find_item(right)).unwrap()
    );
    assert!(
        EditorCore::new(core.paths().clone())
            .get_draft_state(&id, &draft.id)
            .is_ok()
    );
    core.edit(&id, 1, op(update(left, card, values.clone())))
        .unwrap();
    let changed = content(&core, &id);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().components).unwrap(),
        serde_json::to_value(&before.components).unwrap()
    );
    assert_eq!(
        core.commit_draft(&id, &draft.id, 2).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    core.rebase_draft(&id, &draft.id, 2).unwrap();
    core.commit_draft(&id, &draft.id, 2).unwrap();
    assert_eq!(content(&core, &id), changed);
    core.undo(&id, 3).unwrap();
    core.undo(&id, 4).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap(),
        serde_json::to_value(&before.tracks).unwrap()
    );
    core.redo(&id, 5).unwrap();
    core.redo(&id, 6).unwrap();
    assert_eq!(content(&core, &id), changed);
    let discard = core
        .create_draft(&id, 7, vec![op(update(left, card, json!({})))], None)
        .unwrap();
    let persisted = files(&core, &id);
    core.discard_draft(&id, &discard.id).unwrap();
    assert_eq!(files(&core, &id), persisted);
    let p = core.paths().project_dir(&id).unwrap();
    let mut locked = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    locked["components"][0]["tracks"][0]["locked"] = json!(true);
    std::fs::write(p.join("project.json"), serde_json::to_vec(&locked).unwrap()).unwrap();
    let mut slots = catalog()["definition"]["slots"].clone();
    slots[0]["defaultValue"]["value"] = json!("#000000");
    reject(
        &core,
        &id,
        7,
        json!({"operation":"component_define_slots","componentId":card,"slots":slots}),
        ErrorCode::TrackLocked,
    );
    let mut locked = locked;
    locked["components"][0]["tracks"][0]["locked"] = json!(false);
    for t in locked["tracks"].as_array_mut().unwrap() {
        if t["id"] == track {
            t["locked"] = json!(true);
        }
    }
    std::fs::write(p.join("project.json"), serde_json::to_vec(&locked).unwrap()).unwrap();
    reject(
        &core,
        &id,
        7,
        update(left, card, values),
        ErrorCode::TrackLocked,
    );
}

#[test]
fn duplication_and_optional_required_graphic_values_remain_independent() {
    let (_root, core, id, track) = setup();
    let aliases = seed(&core, &id, &track);
    let before = core.get_project(&id).unwrap();
    let duplicate = core.edit(&id, 1, op(json!({"operation":"component_instance_duplicate","itemId":aliases["left"],"offsetMs":0,"slotValues":catalog()["overrides"]}))).unwrap();
    let saved = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&saved.components).unwrap(),
        serde_json::to_value(&before.components).unwrap()
    );
    for name in ["left", "right"] {
        assert_eq!(
            serde_json::to_value(saved.find_item(&aliases[name])).unwrap(),
            serde_json::to_value(before.find_item(&aliases[name])).unwrap()
        );
    }
    assert_eq!(
        serde_json::to_value(saved.find_item(&duplicate.changed_ids[0])).unwrap()["slotValues"],
        catalog()["overrides"]
    );
    let mut optional = catalog()["definition"]["slots"].clone();
    for slot in optional.as_array_mut().unwrap() {
        slot.as_object_mut().unwrap().remove("defaultValue");
        slot["required"] = json!(false);
    }
    core.edit(&id, 2, op(json!({"operation":"component_define_slots","componentId":aliases["card"],"slots":optional}))).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().components[0].tracks.clone()).unwrap(),
        serde_json::to_value(&before.components[0].tracks).unwrap()
    );
    let mut required = catalog()["definition"]["slots"].clone();
    for slot in required.as_array_mut().unwrap() {
        slot.as_object_mut().unwrap().remove("defaultValue");
    }
    reject(
        &core,
        &id,
        3,
        json!({"operation":"component_define_slots","componentId":aliases["card"],"slots":required}),
        ErrorCode::InvalidArgument,
    );
}

#[test]
fn schema44_adoption_and_future_or_mislabeled_history_are_atomic() {
    let (_root, core, id, _track) = setup();
    let mut existing = catalog()["definition"].clone();
    existing["slots"] = json!([{"id":"legacyOpacity","name":"Legacy opacity","kind":"number","required":true,"defaultValue":{"type":"number","value":0.75},"binding":{"targetLayerId":"shape","property":"visual.opacity"},"constraints":{}}]);
    core.edit(&id, 0, op(existing)).unwrap();
    let p = core.paths().project_dir(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut old = original.clone();
    old["schemaVersion"] = json!(44);
    old.as_object_mut().unwrap().remove("narrationManifests");
    std::fs::write(p.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(
        p.join("history.json"),
        serde_json::to_vec(&json!({"undo":[old.clone()],"redo":[old.clone()]})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        original
    );
    let migrated = files(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(files(&core, &id), migrated);
    let history: Value = serde_json::from_slice(&migrated.1).unwrap();
    assert_eq!(history["undo"][0], original);
    assert_eq!(history["redo"][0], original);
    let mut definition = catalog()["definition"].clone();
    definition.as_object_mut().unwrap().remove("operation");
    definition["id"] = json!("card");
    definition["markers"] = json!([]);
    let mut mislabeled = old.clone();
    mislabeled["components"] = json!([definition]);
    assert!(serde_json::from_value::<Project>(mislabeled.clone()).is_err());
    let mut future = old.clone();
    future["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION + 1);
    for bad in [mislabeled, future] {
        std::fs::write(p.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
        std::fs::write(
            p.join("history.json"),
            serde_json::to_vec(&json!({"undo":[old.clone()],"redo":[bad.clone()]})).unwrap(),
        )
        .unwrap();
        let before = files(&core, &id);
        assert!(core.get_project(&id).is_err());
        assert_eq!(files(&core, &id), before);
        std::fs::write(p.join("project.json"), serde_json::to_vec(&bad).unwrap()).unwrap();
        let before = files(&core, &id);
        assert!(core.get_project(&id).is_err());
        assert_eq!(files(&core, &id), before);
    }
}

// Genuine pre-graphic-slot data. Do not migrate with a read before exercising the
// rejected request: the persisted generation itself is the regression oracle.
fn legacy_slot_edit_fixture() -> (tempfile::TempDir, EditorCore, String, String, String) {
    let (root, core, id, track) = setup();
    let mut definition = catalog()["definition"].clone();
    definition["slots"] = json!([]);
    definition["resultAlias"] = json!("legacy_card");
    let seed: Vec<BatchEditOperation> = serde_json::from_value(json!([
        definition,
        {"operation":"add_component_instance","resultAlias":"legacy_instance",
         "trackId":track,"componentId":"@legacy_card","startMs":0,
         "trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{}}
    ]))
    .unwrap();
    let aliases = core.edit_batch(&id, 0, seed).unwrap().aliases;
    let dir = core.paths().project_dir(&id).unwrap();
    let mut project: Value = serde_json::from_slice(&files(&core, &id).0).unwrap();
    project["schemaVersion"] = json!(44);
    project
        .as_object_mut()
        .unwrap()
        .remove("narrationManifests");
    let mut retained = project.clone();
    retained["revision"] = json!(0);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({
            "undo":[retained.clone()],"redo":[retained]
        }))
        .unwrap(),
    )
    .unwrap();
    (
        root,
        core,
        id,
        aliases["legacy_card"].clone(),
        aliases["legacy_instance"].clone(),
    )
}

fn legacy_slot_definition(component: &str, invalid: bool) -> Value {
    json!({"operation":"component_define_slots","componentId":component,"slots":[{
        "id":"opacity","name":"Opacity","kind":"number","required":true,
        "defaultValue":{"type":"number","value":0.75},
        "binding":{"targetLayerId":if invalid {"missing_layer"} else {"shape"},"property":"visual.opacity"},
        "constraints":{}
    }]})
}

#[test]
fn rejected_schema44_slot_definition_preserves_project_and_history_bytes() {
    let (_root, core, id, component, _instance) = legacy_slot_edit_fixture();
    reject(
        &core,
        &id,
        1,
        legacy_slot_definition(&component, true),
        ErrorCode::ItemNotFound,
    );
    let persisted: Value = serde_json::from_slice(&files(&core, &id).0).unwrap();
    assert_eq!(persisted["schemaVersion"], 44);
    assert_eq!(persisted["revision"], 1);
}

#[test]
fn rejected_schema44_instance_update_preserves_project_and_history_bytes() {
    let (_root, core, id, component, instance) = legacy_slot_edit_fixture();
    reject(
        &core,
        &id,
        1,
        update(
            &instance,
            &component,
            json!({"missing_slot":{"type":"number","value":0.5}}),
        ),
        ErrorCode::ItemNotFound,
    );
    let persisted: Value = serde_json::from_slice(&files(&core, &id).0).unwrap();
    assert_eq!(persisted["schemaVersion"], 44);
    assert_eq!(persisted["revision"], 1);
}

#[test]
fn rejected_schema44_late_slot_batch_preserves_project_and_history_bytes() {
    let (_root, core, id, component, instance) = legacy_slot_edit_fixture();
    let mut valid = update(&instance, &component, json!({}));
    valid["startMs"] = json!(50);
    let before = files(&core, &id);
    let batch: Vec<BatchEditOperation> =
        serde_json::from_value(json!([valid, legacy_slot_definition(&component, true)])).unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, batch).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(&core, &id), before);
    let persisted: Value = serde_json::from_slice(&before.0).unwrap();
    assert_eq!(persisted["schemaVersion"], 44);
    assert_eq!(persisted["revision"], 1);
}

#[test]
fn valid_schema44_slot_edits_migrate_once_and_keep_history_reopen_and_revisions() {
    for route in 0..3 {
        let (_root, core, id, component, instance) = legacy_slot_edit_fixture();
        let before = files(&core, &id);
        let valid = if route == 0 {
            legacy_slot_definition(&component, false)
        } else {
            update(&instance, &component, json!({}))
        };
        assert_eq!(
            core.edit(&id, 0, op(valid.clone())).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(files(&core, &id), before);
        let result = if route == 2 {
            core.edit_batch(
                &id,
                1,
                vec![op(legacy_slot_definition(&component, false)), op(valid)],
            )
            .unwrap()
        } else {
            core.edit(&id, 1, op(valid)).unwrap()
        };
        assert_eq!(result.revision, 2);
        let dir = core.paths().project_dir(&id).unwrap();
        let (project, history) = files(&core, &id);
        let project: Value = serde_json::from_slice(&project).unwrap();
        let history: Value = serde_json::from_slice(&history).unwrap();
        assert_eq!(project["schemaVersion"], PROJECT_SCHEMA_VERSION);
        assert_eq!(project["revision"], 2);
        for snapshot in history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap())
        {
            assert_eq!(snapshot["schemaVersion"], PROJECT_SCHEMA_VERSION);
        }
        assert!(history["redo"].as_array().unwrap().is_empty());
        if route != 1 {
            assert_eq!(project["components"][0]["slots"][0]["id"], "opacity");
        }
        let reopened = EditorCore::new(core.paths().clone());
        let stable = files(&core, &id);
        reopened.get_project(&id).unwrap();
        reopened.get_project(&id).unwrap();
        assert_eq!(files(&core, &id), stable);
        reopened.undo(&id, 2).unwrap();
        reopened.redo(&id, 3).unwrap();
        let restored: Value = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
        assert_eq!(restored["components"], project["components"]);
        assert_eq!(restored["tracks"], project["tracks"]);
        assert!(!dir.join("transaction.json").exists());
    }
}
