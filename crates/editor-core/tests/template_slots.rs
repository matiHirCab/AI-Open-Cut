use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, MediaProbeFacts, MediaType,
    PathPolicy, ProjectSettings, SlotValue, TemplateSlot,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/template-slots-v1.json")).unwrap()
}

#[test]
fn canonical_closed_slot_records_reject_unknown_own_fields() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!("../../../contracts/template-slots-v1.json"))
            .unwrap();
    for fixture in catalog["regressions"]["closedRecords"].as_array().unwrap() {
        let mut record = &fixture["slot"];
        for key in fixture["recordPath"].as_array().unwrap() {
            record = if let Some(index) = key.as_u64() {
                &record[index as usize]
            } else {
                &record[key.as_str().unwrap()]
            };
        }
        assert!(
            record
                .as_object()
                .unwrap()
                .contains_key(fixture["key"].as_str().unwrap()),
            "{}",
            fixture["id"]
        );
        assert!(
            serde_json::from_value::<TemplateSlot>(fixture["slot"].clone()).is_err(),
            "{}",
            fixture["id"]
        );
        assert!(
            serde_json::from_value::<EditOperation>(define("card", json!([fixture["slot"]])))
                .is_err(),
            "{}",
            fixture["id"]
        );
        if let Some(value) = fixture.get("override") {
            assert!(
                serde_json::from_value::<SlotValue>(value.clone()).is_err(),
                "{}",
                fixture["id"]
            );
            for id in catalog["regressions"]["specialKeys"].as_array().unwrap() {
                let mut operation = create(json!([]));
                operation["tracks"] = json!([track(vec![instance(
                    "leaf",
                    json!({id.as_str().unwrap(): value})
                )])]);
                assert!(
                    serde_json::from_value::<EditOperation>(operation).is_err(),
                    "{}",
                    fixture["id"]
                );
            }
        }
    }
}

#[test]
fn special_slot_keys_preserve_required_values_and_atomic_errors() {
    let regression = catalog()["regressions"].clone();
    for key in regression["specialKeys"].as_array().unwrap() {
        for default in [false, true] {
            let (_root, core, id) = setup();
            let mut s = slot("text");
            s["id"] = key.clone();
            if !default {
                s.as_object_mut().unwrap().remove("defaultValue");
            }
            let leaf = core
                .edit(&id, 0, op(create(json!([s]))))
                .unwrap()
                .changed_ids[0]
                .clone();
            let key = key.as_str().unwrap();
            let values = json!({key: regression["overrides"][key]});
            let mut outer = create(json!([]));
            outer["tracks"] = json!([track(vec![instance(&leaf, values.clone())])]);
            for invalid in regression["invalidValues"].as_array().unwrap() {
                let mut malformed = outer.clone();
                malformed["tracks"][0]["items"][0]["slotValues"][key] = invalid.clone();
                assert!(serde_json::from_value::<EditOperation>(malformed).is_err());
            }
            if !default {
                let mut missing = outer.clone();
                missing["tracks"][0]["items"][0]["slotValues"] = json!({});
                reject(&core, &id, 1, missing, ErrorCode::InvalidArgument);
            }
            let mut unknown = outer.clone();
            unknown["tracks"][0]["items"][0]["slotValues"]
                [regression["unknownSlotId"].as_str().unwrap()] =
                json!({"type":"text","value":"unknown"});
            reject(&core, &id, 1, unknown, ErrorCode::ItemNotFound);
            core.edit(&id, 1, op(outer)).unwrap();
            let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            assert_eq!(
                state["components"][1]["tracks"][0]["items"][0]["slotValues"],
                values
            );
            assert_eq!(
                state["components"][0]["tracks"][0]["items"][0]["text"],
                "Base"
            );
            let before = files(&core, &id);
            core.get_project(&id).unwrap();
            assert_eq!(files(&core, &id), before);
            core.undo(&id, 2).unwrap();
            assert_eq!(core.get_project(&id).unwrap().components.len(), 1);
            core.redo(&id, 3).unwrap();
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["components"],
                state["components"]
            );
        }
    }
}

#[test]
fn group_opacity_defaults_overrides_history_and_failures() {
    let regression = catalog()["regressions"].clone();
    for group in regression["groupItems"].as_array().unwrap() {
        for opacity in regression["opacityValues"].as_array().unwrap() {
            let (_root, core, id) = setup();
            let mut s = slot("number");
            s["binding"]["targetLayerId"] = json!("group");
            s["defaultValue"]["value"] = opacity.clone();
            let mut leaf_input = create(json!([s.clone()]));
            leaf_input["tracks"] = json!([track(vec![group.clone()])]);
            let leaf = core
                .edit(&id, 0, op(leaf_input.clone()))
                .unwrap()
                .changed_ids[0]
                .clone();
            let base = core.get_project(&id).unwrap().components[0].tracks.clone();
            let mut revision = 1;
            for value in regression["opacityValues"].as_array().unwrap() {
                let mut outer = create(json!([]));
                outer["tracks"] = json!([track(vec![instance(
                    &leaf,
                    json!({"number":{"type":"number","value":value}})
                )])]);
                core.edit(&id, revision, op(outer)).unwrap();
                revision += 1;
            }
            let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            assert_eq!(
                state["components"][0]["tracks"],
                serde_json::to_value(&base).unwrap()
            );
            for bad in regression["invalidOpacityValues"].as_array().unwrap() {
                let mut invalid_slot = s.clone();
                invalid_slot["defaultValue"]["value"] = bad.clone();
                reject(
                    &core,
                    &id,
                    revision,
                    define(&leaf, json!([invalid_slot])),
                    ErrorCode::InvalidArgument,
                );
                let mut outer = create(json!([]));
                outer["tracks"] = json!([track(vec![instance(
                    &leaf,
                    json!({"number":{"type":"number","value":bad}})
                )])]);
                reject(&core, &id, revision, outer, ErrorCode::InvalidArgument);
            }
            let before = files(&core, &id);
            assert_eq!(
                core.edit(&id, revision - 1, op(define(&leaf, json!([s.clone()]))))
                    .unwrap_err()
                    .code,
                ErrorCode::RevisionConflict
            );
            assert_eq!(files(&core, &id), before);
            let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
                define(&leaf, json!([s.clone()])),
                define("missing", json!([]))
            ]))
            .unwrap();
            assert_eq!(
                core.edit_batch(&id, revision, batch).unwrap_err().code,
                ErrorCode::ItemNotFound
            );
            assert_eq!(files(&core, &id), before);
            core.undo(&id, revision).unwrap();
            core.redo(&id, revision + 1).unwrap();
            revision += 2;
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["components"],
                state["components"]
            );
            let mut locked = leaf_input;
            locked["operation"] = json!("component_update");
            locked["componentId"] = json!(leaf);
            locked["tracks"][0]["locked"] = json!(true);
            core.edit(&id, revision, op(locked)).unwrap();
            revision += 1;
            core.edit(&id, revision, op(define(&leaf, json!([s.clone()]))))
                .unwrap();
            revision += 1;
            s["defaultValue"]["value"] = json!(if opacity.as_f64().unwrap() == 0.5 {
                0.0
            } else {
                0.5
            });
            reject(
                &core,
                &id,
                revision,
                define(&leaf, json!([s])),
                ErrorCode::TrackLocked,
            );
        }
    }
}

#[test]
fn rich_text_metadata_duration_and_override_bounds_cover_exact_endpoints() {
    let (_root, core, id) = setup();
    let mut s = slot("rich_text");
    s["id"] = json!("s".repeat(128));
    s["name"] = json!("é".repeat(128));
    s["defaultValue"]["value"] = json!({"runs":vec![json!({"text":"😀".repeat(16),"bold":true,"italic":false,"color":"#123ABC"});256]});
    s["constraints"] = json!({"minLength":4096,"maxLength":4096});
    let leaf = core
        .edit(&id, 0, op(create(json!([s.clone()]))))
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut bad = s.clone();
    bad["name"] = json!("é".repeat(129));
    reject(
        &core,
        &id,
        1,
        create(json!([bad])),
        ErrorCode::InvalidArgument,
    );
    let mut bad = s;
    bad["id"] = json!("s".repeat(129));
    reject(
        &core,
        &id,
        1,
        create(json!([bad])),
        ErrorCode::InvalidArgument,
    );
    let mut d = slot("duration");
    d["defaultValue"]["value"] = json!(9_007_199_254_740_991u64);
    d["constraints"] = json!({"min":0,"max":9_007_199_254_740_991u64});
    let mut c = create(json!([d]));
    c["durationMs"] = json!(9_007_199_254_740_991u64);
    core.edit(&id, 1, op(c)).unwrap();
    let values: serde_json::Map<_, _> = (0..129)
        .map(|i| (format!("s{i}"), json!({"type":"text","value":"x"})))
        .collect();
    let mut outer = create(json!([]));
    outer["tracks"] = json!([track(vec![instance(&leaf, Value::Object(values))])]);
    let before = files(&core, &id);
    let error = core.edit(&id, 2, op(outer)).unwrap_err();
    assert!(error.message.contains("instance slot count"), "{error:?}");
    assert_eq!(files(&core, &id), before);
}

#[test]
fn corrupt_slot_values_in_current_history_and_drafts_never_publish() {
    let (_root, core, id) = setup();
    let mut invalid = slot("text");
    invalid["defaultValue"] = json!({"type":"boolean","value":true});
    let before = files(&core, &id);
    assert_eq!(
        core.create_draft(&id, 0, vec![op(create(json!([invalid.clone()])))], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
    core.edit(&id, 0, op(create(json!([slot("text")]))))
        .unwrap();
    let valid = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut bad = valid.clone();
    bad["components"][0]["slots"] = json!([invalid]);
    let dir = core.paths().project_dir(&id).unwrap();
    for location in ["current", "undo", "redo"] {
        let mut history = json!({"undo":[],"redo":[]});
        if location != "current" {
            history[location] = json!([bad.clone()]);
        }
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(if location == "current" { &bad } else { &valid }).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&history).unwrap(),
        )
        .unwrap();
        let before = files(&core, &id);
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn invalid_unused_slots_fail_before_render_output_or_process_execution() {
    use opencut_editor_core::Renderer;
    let (_root, core, id) = setup();
    let root_track = core.get_project(&id).unwrap().tracks[0].id.clone();
    core.edit(&id,0,op(json!({"operation":"add_solid_color","trackId":root_track,"startMs":0,"durationMs":1000,"color":"#000000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    core.edit(&id, 1, op(create(json!([slot("text")]))))
        .unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.components[0].slots[0].default_value = Some(SlotValue::Number(3.0));
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
    let before = files(&core, &id);
    assert_eq!(
        renderer.render_preview(&project, &dir, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
}

#[test]
fn aggregate_slot_and_text_limits_are_inclusive() {
    let (_root, core, id) = setup();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let load = |definitions: Value| {
        let mut state = original.clone();
        state["components"] = definitions;
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        core.get_project(&id)
    };
    let mut definition = create(json!([]));
    definition.as_object_mut().unwrap().remove("operation");
    definition["tracks"][0]["items"] = json!(
        (0..64)
            .map(|i| title(&format!("t{i}"), i))
            .collect::<Vec<_>>()
    );
    definition["slots"] = json!(
        (0..128)
            .map(|i| {
                let mut s = slot(if i % 2 == 0 { "number" } else { "boolean" });
                s["id"] = json!(format!("s{i}"));
                s["binding"]["targetLayerId"] = json!(format!("t{}", i / 2));
                s
            })
            .collect::<Vec<_>>()
    );
    let mut definitions: Vec<_> = (0..32)
        .map(|i| {
            let mut d = definition.clone();
            d["id"] = json!(format!("d{i}"));
            d
        })
        .collect();
    assert!(load(json!(definitions)).is_ok());
    let mut extra = definition.clone();
    extra["id"] = json!("extra");
    extra["slots"].as_array_mut().unwrap().truncate(1);
    definitions.push(extra);
    assert!(
        load(json!(definitions))
            .unwrap_err()
            .message
            .contains("slot collection")
    );
    definition["tracks"][0]["items"] = json!(
        (0..128)
            .map(|i| title(&format!("t{i}"), i))
            .collect::<Vec<_>>()
    );
    definition["slots"] = json!(
        (0..128)
            .map(|i| {
                let mut s = slot("text");
                s["id"] = json!(format!("s{i}"));
                s["binding"]["targetLayerId"] = json!(format!("t{i}"));
                s["defaultValue"]["value"] = json!("x".repeat(4096));
                s
            })
            .collect::<Vec<_>>()
    );
    let mut definitions: Vec<_> = (0..2)
        .map(|i| {
            let mut d = definition.clone();
            d["id"] = json!(format!("d{i}"));
            d
        })
        .collect();
    assert!(load(json!(definitions)).is_ok());
    let mut extra = definition;
    extra["id"] = json!("extra");
    extra["slots"].as_array_mut().unwrap().truncate(1);
    extra["slots"][0]["defaultValue"]["value"] = json!("x");
    definitions.push(extra);
    assert!(
        load(json!(definitions))
            .unwrap_err()
            .message
            .contains("snapshot slot text")
    );
}

#[test]
fn effective_asset_and_duration_are_validated_together_and_defaults_remain_validated() {
    let (root, core, id) = setup();
    let mut assets = Vec::new();
    for (i, duration) in [1000, 300].iter().enumerate() {
        let path = root.path().join(format!("media/video{i}.mp4"));
        std::fs::write(&path, format!("video{i}")).unwrap();
        assets.push(
            core.import_asset(
                &id,
                i as u64,
                &path,
                MediaType::Video,
                MediaProbeFacts {
                    duration_ms: Some(*duration),
                    has_video: true,
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
                .clone(),
        );
    }
    let mut asset = slot("asset");
    asset["defaultValue"]["value"]["id"] = json!(assets[0]);
    let mut duration = slot("duration");
    duration["binding"]["targetLayerId"] = json!("media");
    duration["defaultValue"]["value"] = json!(300);
    let mut c = create(json!([asset.clone(), duration]));
    c["tracks"] = json!([track(vec![
        json!({"type":"media","id":"media","assetId":assets[0],"startMs":0,"durationMs":1000,"sourceInMs":0,"keyframes":[],"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}})
    ])]);
    let leaf = core.edit(&id, 2, op(c.clone())).unwrap().changed_ids[0].clone();
    let values = json!({"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":assets[1]}},"duration":{"type":"duration","value":300}});
    let mut outer = create(json!([]));
    outer["tracks"] = json!([track(vec![instance(&leaf, values)])]);
    let mut bad = outer.clone();
    bad["tracks"][0]["items"][0]["slotValues"]["duration"]["value"] = json!(301);
    reject(&core, &id, 3, bad, ErrorCode::InvalidArgument);
    core.edit(&id, 3, op(outer)).unwrap();
    c["operation"] = json!("component_update");
    c["componentId"] = json!(leaf);
    c["slots"][1]["defaultValue"]["value"] = json!(1001);
    reject(&core, &id, 4, c, ErrorCode::InvalidArgument);
}

#[test]
fn slot_only_assets_are_retained_in_current_drafts_and_history() {
    let (root, core, id) = setup();
    let mut assets = Vec::new();
    for n in 0..3 {
        let path = root.path().join(format!("media/image{n}.png"));
        std::fs::write(&path, format!("image {n}")).unwrap();
        assets.push(
            core.import_asset(&id, n, &path, MediaType::Image, MediaProbeFacts::default())
                .unwrap()
                .changed_ids[0]
                .clone(),
        );
    }
    let mut s = slot("asset");
    s["defaultValue"]["value"]["id"] = json!(assets[1]);
    let mut c = create(json!([s.clone()]));
    c["tracks"] = json!([track(vec![
        json!({"type":"media","id":"media","assetId":assets[0],"startMs":0,"durationMs":1000,"sourceInMs":0,"keyframes":[],"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}})
    ])]);
    let draft = core
        .create_draft(&id, 3, vec![op(c.clone())], None)
        .unwrap();
    assert_eq!(
        core.delete_asset(&id, 3, &assets[1]).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    core.get_draft_state(&id, &draft.id).unwrap();
    core.discard_draft(&id, &draft.id).unwrap();
    for (asset, error) in [
        ("missing", ErrorCode::AssetNotFound),
        ("../escape", ErrorCode::InvalidArgument),
        ("https://example.org", ErrorCode::InvalidArgument),
    ] {
        let mut bad = c.clone();
        bad["slots"][0]["defaultValue"]["value"]["id"] = json!(asset);
        reject(&core, &id, 3, bad, error);
    }
    let leaf = core.edit(&id, 3, op(c)).unwrap().changed_ids[0].clone();
    let mut outer = create(json!([]));
    outer["tracks"] = json!([track(vec![instance(
        &leaf,
        json!({"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":assets[2]}}})
    )])]);
    let draft = core
        .create_draft(&id, 4, vec![op(outer.clone())], None)
        .unwrap();
    assert_eq!(
        core.delete_asset(&id, 4, &assets[2]).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    core.discard_draft(&id, &draft.id).unwrap();
    let parent = core.edit(&id, 4, op(outer)).unwrap().changed_ids[0].clone();
    for asset in &assets[1..] {
        assert_eq!(
            core.delete_asset(&id, 5, asset).unwrap_err().code,
            ErrorCode::AssetInUse
        );
    }
    let state = core.get_project(&id).unwrap();
    let stored = core
        .paths()
        .project_dir(&id)
        .unwrap()
        .join(&state.assets[1].project_relative_path);
    core.edit_batch(
        &id,
        5,
        vec![
            op(json!({"operation":"component_delete","componentId":parent})),
            op(json!({"operation":"component_delete","componentId":leaf})),
        ],
    )
    .unwrap();
    core.delete_asset(&id, 6, &assets[1]).unwrap();
    assert!(stored.exists());
    core.undo(&id, 7).unwrap();
    core.undo(&id, 8).unwrap();
    assert_eq!(core.get_project(&id).unwrap().components.len(), 2);
    assert!(stored.exists());
}
fn setup() -> (tempfile::TempDir, EditorCore, String) {
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
        .create_project("Slots", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
}
fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
}
fn title(id: &str, order: usize) -> Value {
    json!({"type":"text","id":id,"text":"Base","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[],"stackOrder":order})
}
fn track(items: Vec<Value>) -> Value {
    json!({"id":"local","name":"Local","trackType":"overlay","items":items})
}
fn create(slots: Value) -> Value {
    json!({"operation":"component_create","name":"Card","width":320,"height":240,"durationMs":1000,"tracks":[track(vec![title("title",0)])],"slots":slots})
}
fn slot(kind: &str) -> Value {
    catalog()["valid"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == kind)
        .unwrap()["slot"]
        .clone()
}
fn define(component: &str, slots: Value) -> Value {
    json!({"operation":"component_define_slots","componentId":component,"slots":slots})
}
fn instance(target: &str, values: Value) -> Value {
    json!({"type":"component_instance","id":"nested","componentId":target,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values})
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
fn reject(core: &EditorCore, id: &str, rev: u64, value: Value, code: ErrorCode) {
    let before = files(core, id);
    assert_eq!(core.edit(id, rev, op(value)).unwrap_err().code, code);
    assert_eq!(files(core, id), before);
}

#[test]
fn canonical_all_kinds_roundtrip_defaults_overrides_history_and_reopen() {
    for fixture in catalog()["valid"].as_array().unwrap() {
        let (root, core, id) = setup();
        let mut s = fixture["slot"].clone();
        let mut value = create(json!([s.clone()]));
        let mut revision = 0;
        if fixture["id"] == "asset" {
            let path = root.path().join("media/image.png");
            std::fs::write(&path, b"trusted image").unwrap();
            let asset = core
                .import_asset(&id, 0, &path, MediaType::Image, MediaProbeFacts::default())
                .unwrap()
                .changed_ids[0]
                .clone();
            revision = 1;
            s["defaultValue"]["value"]["id"] = json!(asset);
            value["slots"] = json!([s.clone()]);
            value["tracks"] = json!([track(vec![
                json!({"type":"media","id":"media","assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0,"keyframes":[],"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}})
            ])]);
        }
        let mut ops = vec![value];
        ops[0]["resultAlias"] = json!("card");
        let mut outer = create(json!([]));
        let mut values = serde_json::Map::new();
        values.insert(s["id"].as_str().unwrap().into(), s["defaultValue"].clone());
        outer["tracks"] = json!([track(vec![instance(
            "@card",
            Value::Object(values.clone())
        )])]);
        ops.push(outer);
        let result = core
            .edit_batch(
                &id,
                revision,
                serde_json::from_value::<Vec<BatchEditOperation>>(json!(ops)).unwrap(),
            )
            .unwrap();
        let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        assert_eq!(state["components"][0]["slots"], json!([s]));
        assert_eq!(
            state["components"][1]["tracks"][0]["items"][0]["slotValues"],
            json!(values)
        );
        assert_eq!(state["schemaVersion"], 12);
        let persisted = files(&core, &id);
        core.get_project(&id).unwrap();
        assert_eq!(files(&core, &id), persisted);
        core.undo(&id, result.revision).unwrap();
        assert!(core.get_project(&id).unwrap().components.is_empty());
        core.redo(&id, result.revision + 1).unwrap();
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["components"],
            state["components"]
        );
    }
}

#[test]
fn canonical_invalid_and_closed_wire_records() {
    for fixture in catalog()["invalid"].as_array().unwrap() {
        let s = fixture["slot"].clone();
        if fixture["stage"] == "structural" {
            assert!(serde_json::from_value::<TemplateSlot>(s).is_err());
            continue;
        }
        let (_root, core, id) = setup();
        let error = if fixture["error"] == "ITEM_NOT_FOUND" {
            ErrorCode::ItemNotFound
        } else {
            ErrorCode::InvalidArgument
        };
        reject(&core, &id, 0, create(json!([s])), error);
    }
    for field in ["kind", "required", "binding", "constraints", "id", "name"] {
        let mut s = slot("text");
        s.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<TemplateSlot>(s).is_err());
    }
    for value in [
        json!(null),
        json!({"type":"text","value":true}),
        json!({"type":"text","value":"x","extra":1}),
        json!({"type":"duration","value":0.5}),
        json!({"type":"rich_text","value":{"runs":[{"text":"x","href":"https://example.org"}]}}),
    ] {
        assert!(serde_json::from_value::<SlotValue>(value).is_err());
    }
    assert!(serde_json::from_str::<SlotValue>(r#"{"type":"text","value":"\ud800"}"#).is_err());
    let mut s = slot("text");
    s["defaultValue"] = Value::Null;
    assert!(serde_json::from_value::<TemplateSlot>(s).is_err());
}

#[test]
fn required_optional_unknown_values_and_definition_replacement_are_atomic() {
    let (_root, core, id) = setup();
    let mut s = slot("text");
    s.as_object_mut().unwrap().remove("defaultValue");
    let leaf = core
        .edit(&id, 0, op(create(json!([s.clone()]))))
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut outer = create(json!([]));
    outer["tracks"] = json!([track(vec![instance(&leaf, json!({}))])]);
    reject(&core, &id, 1, outer.clone(), ErrorCode::InvalidArgument);
    s["required"] = json!(false);
    core.edit(&id, 1, op(define(&leaf, json!([s.clone()]))))
        .unwrap();
    outer["tracks"][0]["items"][0]["slotValues"] =
        json!({"unknown":{"type":"text","value":"Hello"}});
    reject(&core, &id, 2, outer.clone(), ErrorCode::ItemNotFound);
    outer["tracks"][0]["items"][0]["slotValues"] =
        json!({"text":{"type":"text","value":"Override"}});
    core.edit(&id, 2, op(outer)).unwrap();
    reject(
        &core,
        &id,
        3,
        define(&leaf, json!([])),
        ErrorCode::ItemNotFound,
    );
    reject(
        &core,
        &id,
        2,
        define(&leaf, json!([s.clone()])),
        ErrorCode::RevisionConflict,
    );
    reject(
        &core,
        &id,
        3,
        define("missing", json!([])),
        ErrorCode::ItemNotFound,
    );
    let before = files(&core, &id);
    let batch = json!([define(&leaf, json!([s])), define("missing", json!([]))]);
    assert_eq!(
        core.edit_batch(
            &id,
            3,
            serde_json::from_value::<Vec<BatchEditOperation>>(batch).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(&core, &id), before);
    let state = core.get_project(&id).unwrap();
    let mut update = create(json!([]));
    update.as_object_mut().unwrap().remove("slots");
    update["operation"] = json!("component_update");
    update["componentId"] = json!(leaf);
    core.edit(&id, 3, op(update.clone())).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().components[0].slots,
        state.components[0].slots
    );
    update["tracks"] = json!([]);
    reject(&core, &id, 4, update, ErrorCode::ItemNotFound);
}

#[test]
fn locks_scope_duplicate_writers_and_effective_domain_rules() {
    let (_root, core, id) = setup();
    let mut c = create(json!([slot("text")]));
    c["tracks"][0]["locked"] = json!(true);
    let leaf = core.edit(&id, 0, op(c)).unwrap().changed_ids[0].clone();
    core.edit(&id, 1, op(define(&leaf, json!([slot("text")]))))
        .unwrap();
    reject(
        &core,
        &id,
        2,
        define(&leaf, json!([])),
        ErrorCode::TrackLocked,
    );
    let mut s = slot("text");
    s["defaultValue"]["value"] = json!("Changed");
    reject(
        &core,
        &id,
        2,
        define(&leaf, json!([s])),
        ErrorCode::TrackLocked,
    );
    for (s, error) in [
        (slot("duration"), ErrorCode::InvalidArgument),
        (slot("number"), ErrorCode::InvalidArgument),
        (slot("rich_text"), ErrorCode::InvalidArgument),
        (slot("text"), ErrorCode::InvalidArgument),
    ] {
        let mut bad = s;
        match bad["kind"].as_str().unwrap() {
            "duration" => bad["defaultValue"]["value"] = json!(1001),
            "number" => bad["defaultValue"]["value"] = json!(1.1),
            "rich_text" => bad["binding"]["property"] = json!("visual.opacity"),
            _ => bad["binding"]["targetLayerId"] = json!("root:title"),
        }
        reject(&core, &id, 2, create(json!([bad])), error);
    }
    let mut duplicate = slot("text");
    duplicate["id"] = json!("other");
    reject(
        &core,
        &id,
        2,
        create(json!([slot("text"), duplicate])),
        ErrorCode::InvalidArgument,
    );
    // Equal local IDs in independent definitions are valid and never target the locked definition.
    core.edit(&id, 2, op(create(json!([slot("text")]))))
        .unwrap();
}

#[test]
fn bounds_constraints_unicode_and_nonfinite_typed_values() {
    let (_root, core, id) = setup();
    let mut s = slot("text");
    s["defaultValue"]["value"] = json!("😀é");
    s["constraints"] = json!({"minLength":3,"maxLength":3});
    core.edit(&id, 0, op(create(json!([s.clone()])))).unwrap();
    s["constraints"]["maxLength"] = json!(2);
    reject(
        &core,
        &id,
        1,
        create(json!([s])),
        ErrorCode::InvalidArgument,
    );
    for (kind, value) in [
        ("text", json!("x".repeat(4097))),
        ("rich_text", json!({"runs":vec![json!({"text":"x"});257]})),
        ("duration", json!(9_007_199_254_740_992u64)),
        ("enum", json!("missing")),
        ("color", json!("red")),
    ] {
        let mut s = slot(kind);
        s["defaultValue"]["value"] = value;
        reject(
            &core,
            &id,
            1,
            create(json!([s])),
            ErrorCode::InvalidArgument,
        );
    }
    for constraints in [
        json!({"min":1}),
        json!({"minLength":4,"maxLength":3}),
        json!({"minLength":4097}),
    ] {
        let mut s = slot("text");
        s["constraints"] = constraints;
        reject(
            &core,
            &id,
            1,
            create(json!([s])),
            ErrorCode::InvalidArgument,
        );
    }
    for choices in [json!([]), json!(["left", "left"]), json!(["unknown"])] {
        let mut s = slot("enum");
        s["constraints"]["choices"] = choices;
        reject(
            &core,
            &id,
            1,
            create(json!([s])),
            ErrorCode::InvalidArgument,
        );
    }
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut edit = op(create(json!([slot("number")])));
        let EditOperation::ComponentCreate {
            slots: Some(slots), ..
        } = &mut edit
        else {
            unreachable!()
        };
        slots[0].default_value = Some(SlotValue::Number(number));
        assert_eq!(
            core.edit(&id, 1, edit).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut many = create(json!([]));
    many["tracks"][0]["items"] = json!(
        (0..129)
            .map(|i| title(&format!("title{i}"), i))
            .collect::<Vec<_>>()
    );
    let slots: Vec<_> = (0..129)
        .map(|i| {
            let mut s = slot("text");
            s["id"] = json!(format!("s{i}"));
            s["binding"]["targetLayerId"] = json!(format!("title{i}"));
            s
        })
        .collect();
    many["slots"] = json!(&slots[..128]);
    core.edit(&id, 1, op(many.clone())).unwrap();
    many["slots"] = json!(slots);
    reject(&core, &id, 2, many, ErrorCode::InvalidArgument);
}

#[test]
fn schema11_nested_history_migrates_and_schema12_fields_are_required() {
    let (_root, core, id) = setup();
    let leaf = core
        .edit(&id, 0, op(create(json!([]))))
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut outer = create(json!([]));
    outer["tracks"] = json!([track(vec![instance(&leaf, json!({}))])]);
    core.edit(&id, 1, op(outer)).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut old = original.clone();
    old["schemaVersion"] = json!(11);
    for c in old["components"].as_array_mut().unwrap() {
        c.as_object_mut().unwrap().remove("slots");
        for t in c["tracks"].as_array_mut().unwrap() {
            for i in t["items"].as_array_mut().unwrap() {
                i.as_object_mut().unwrap().remove("slotValues");
            }
        }
    }
    let dir = core.paths().project_dir(&id).unwrap();
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[old.clone()],"redo":[old]})).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        original
    );
    let bytes = files(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(files(&core, &id), bytes);
    let history: Value = serde_json::from_slice(&bytes.1).unwrap();
    assert_eq!(history["undo"][0], original);
    assert_eq!(history["redo"][0], original);
    for field in ["slots", "slotValues"] {
        let mut bad = original.clone();
        if field == "slots" {
            bad["components"][0].as_object_mut().unwrap().remove(field);
        } else {
            bad["components"][1]["tracks"][0]["items"][0]
                .as_object_mut()
                .unwrap()
                .remove(field);
        }
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&json!({"undo":[bad],"redo":[]})).unwrap(),
        )
        .unwrap();
        let before = files(&core, &id);
        assert!(core.get_project(&id).is_err());
        assert_eq!(files(&core, &id), before);
    }
}
