use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/component-definitions-v1.json"
    ))
    .unwrap()
}

#[test]
fn definitions_and_durable_drafts_retain_media_through_history() {
    use opencut_editor_core::{MediaProbeFacts, MediaType};
    let (root, core, id) = setup();
    let path = root.path().join("media/source.mp4");
    std::fs::write(&path, b"trusted media fixture").unwrap();
    let asset = core
        .import_asset(
            &id,
            0,
            &path,
            MediaType::Video,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_video: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut definition = create();
    definition["tracks"] = json!([track(vec![
        json!({"type":"media","id":"local_media","assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0,"stackOrder":0,"zIndex":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]})
    ])]);
    for invalid_reference in [
        "missing",
        "../../outside.mp4",
        "https://example.invalid/media",
    ] {
        let mut bad = definition.clone();
        bad["tracks"][0]["items"][0]["assetId"] = json!(invalid_reference);
        let unchanged = files(&core, &id);
        assert_eq!(
            core.edit(&id, 1, op(bad)).unwrap_err().code,
            ErrorCode::ItemNotFound
        );
        assert_eq!(files(&core, &id), unchanged);
    }
    let draft = core
        .create_draft(&id, 1, vec![op(definition.clone())], None)
        .unwrap();
    assert_eq!(
        core.delete_asset(&id, 1, &asset).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .components
            .len(),
        1
    );
    core.discard_draft(&id, &draft.id).unwrap();
    let component = core.edit(&id, 1, op(definition)).unwrap().changed_ids[0].clone();
    let before = files(&core, &id);
    assert_eq!(
        core.delete_asset(&id, 2, &asset).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    assert_eq!(files(&core, &id), before);
    let state = core.get_project(&id).unwrap();
    let stored = core
        .paths()
        .project_dir(&id)
        .unwrap()
        .join(&state.assets[0].project_relative_path);
    core.edit(
        &id,
        2,
        op(json!({"operation":"component_delete","componentId":component})),
    )
    .unwrap();
    core.delete_asset(&id, 3, &asset).unwrap();
    assert_eq!(std::fs::read(&stored).unwrap(), b"trusted media fixture");
    core.undo(&id, 4).unwrap();
    core.undo(&id, 5).unwrap();
    assert_eq!(core.get_project(&id).unwrap().components[0].id, component);
    assert_eq!(std::fs::read(&stored).unwrap(), b"trusted media fixture");
}

#[test]
fn native_unused_definitions_preserve_frame_range_export_and_draft_output() {
    let create = || {
        let mut value = create();
        value["tracks"] = json!([track(vec![
            json!({"type":"text","id":"title","text":"Base","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]})
        ])]);
        value["slots"] = json!([{"id":"title","name":"Title","kind":"text","required":true,"defaultValue":{"type":"text","value":"Slot text"},"binding":{"targetLayerId":"title","property":"text.document"},"constraints":{}}]);
        value
    };
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (_root, core, id) = setup();
    let root_track = core.get_project(&id).unwrap().tracks[0].id.clone();
    core.edit(&id,0,op(json!({"operation":"add_solid_color","trackId":root_track,"startMs":0,"durationMs":1000,"color":"#FF0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    let mut before = core.get_project(&id).unwrap();
    before.settings.width = 64;
    before.settings.height = 64;
    let draft = core.create_draft(&id, 1, vec![op(create())], None).unwrap();
    let mut materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
    materialized.settings = before.settings.clone();
    core.edit(&id, 1, op(create())).unwrap();
    let mut after = core.get_project(&id).unwrap();
    after.settings = before.settings.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let decode = |path: &std::path::Path| {
        let out = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .unwrap();
        assert!(out.status.success());
        assert_eq!(out.stdout.len(), 64 * 64 * 3);
        out.stdout
    };
    let mut rendered = Vec::new();
    for (n, project) in [&before, &after, &materialized].iter().enumerate() {
        assert_eq!(project.duration_ms(), 1000);
        let frame = renderer.render_preview(project, &dir, 500).unwrap();
        let pixels = decode(&dir.join(frame.relative_path));
        assert!(pixels[0] > 240);
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 30,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let export = dir.join(format!("component-output-{n}.mp4"));
        renderer
            .export_video(
                project,
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
        rendered.push((
            pixels,
            decode(&dir.join(range.relative_path)),
            decode(&export),
        ));
    }
    assert_eq!(rendered[0], rendered[1]);
    assert_eq!(rendered[0], rendered[2]);
    after.components[0].duration_ms = 0;
    assert_eq!(
        renderer.render_preview(&after, &dir, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
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
        .create_project("Components", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
}
fn create() -> Value {
    catalog()["validOperations"][0].clone()
}
fn track(items: Vec<Value>) -> Value {
    json!({"id":"local","name":"Local","trackType":"overlay","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":items})
}
fn instance(id: &str, target: &str, order: u32) -> Value {
    json!({"slotValues":{},"id":id,"type":"component_instance","componentId":target,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"zIndex":0,"stackOrder":order,"hidden":false,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}

#[test]
fn canonical_component_operations_are_closed() {
    for fixture in catalog()["semanticFixtures"].as_array().unwrap() {
        let (_root, core, id) = setup();
        let result = load_definitions(&core, &id, fixture["components"].clone());
        match fixture["error"].as_str() {
            None => {
                result.unwrap();
            }
            Some(code) => {
                assert_eq!(
                    serde_json::to_value(result.unwrap_err().code).unwrap(),
                    json!(code),
                    "{}",
                    fixture["id"]
                );
            }
        }
    }
    for value in catalog()["validOperations"].as_array().unwrap() {
        serde_json::from_value::<EditOperation>(value.clone()).unwrap();
        serde_json::from_value::<BatchEditOperation>(value.clone()).unwrap();
    }
    for value in catalog()["invalidOperations"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_err(),
            "{value}"
        );
        assert!(
            serde_json::from_value::<BatchEditOperation>(value.clone()).is_err(),
            "{value}"
        );
    }
    for operation in ["component_update", "component_delete"] {
        for alias in [Value::Null, json!("alias"), json!(42)] {
            let mut value = if operation == "component_update" {
                catalog()["validOperations"][1].clone()
            } else {
                catalog()["validOperations"][2].clone()
            };
            value["resultAlias"] = alias;
            assert!(serde_json::from_value::<BatchEditOperation>(value).is_err());
        }
    }
}

#[test]
fn component_item_defaults_closed_fields_and_unsupported_root_placement() {
    let (_root, core, id) = setup();
    let mut create_group = create();
    create_group["tracks"] = json!([track(vec![
        json!({"type":"group","id":"g","startMs":0,"durationMs":0,"stackOrder":0,"zIndex":0})
    ])]);
    let before = files(&core, &id);
    assert_eq!(
        core.edit(&id, 0, op(create_group.clone()))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
    create_group["tracks"][0]["items"][0]["durationMs"] = json!(1000);
    for field in ["children", "expression", "url"] {
        let mut bad = create_group.clone();
        bad["tracks"][0]["items"][0][field] = json!("unsafe");
        assert!(serde_json::from_value::<EditOperation>(bad).is_err());
    }
    let dir = core.paths().project_dir(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for components in [None, Some(Value::Null)] {
        let mut bad = original.clone();
        bad.as_object_mut().unwrap().remove("components");
        if let Some(value) = components {
            bad["components"] = value;
        }
        assert!(serde_json::from_value::<opencut_editor_core::Project>(bad).is_err());
    }
    let mut bad = original;
    bad["tracks"][1]["items"] = json!([instance("root_instance", "absent", 0)]);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&bad).unwrap()).unwrap();
    let root_before = files(&core, &id);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), root_before);
}

#[test]
fn aliased_lifecycle_incoming_durations_locks_and_rollback() {
    let (_root, core, id) = setup();
    let mut leaf = create();
    leaf["resultAlias"] = json!("leaf");
    let mut outer = create();
    outer["resultAlias"] = json!("outer");
    outer["tracks"] = json!([track(vec![
        instance("one", "@leaf", 0),
        instance("two", "@leaf", 1)
    ])]);
    let result = core
        .edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(json!([leaf, outer])).unwrap(),
        )
        .unwrap();
    let leaf = &result.aliases["leaf"];
    let outer = &result.aliases["outer"];
    assert_eq!(result.changed_ids, vec![leaf.clone(), outer.clone()]);
    let before = files(&core, &id);
    let mut shorten = create();
    shorten["operation"] = json!("component_update");
    shorten["componentId"] = json!(leaf);
    shorten["durationMs"] = json!(999);
    let mut cycle = create();
    cycle["operation"] = json!("component_update");
    cycle["componentId"] = json!(leaf);
    cycle["tracks"] = json!([track(vec![instance("cycle", outer, 0)])]);
    for edit in [
        shorten,
        cycle,
        json!({"operation":"component_delete","componentId":leaf}),
    ] {
        assert_eq!(
            core.edit(&id, 1, op(edit)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    assert_eq!(
        core.edit(&id, 0, op(create())).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(
            &id,
            1,
            op(json!({"operation":"component_delete","componentId":"missing"}))
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );
    assert!(
        core.edit_batch::<BatchEditOperation>(
            &id,
            1,
            serde_json::from_value(
                json!([create(),{"operation":"component_delete","componentId":"missing"}])
            )
            .unwrap()
        )
        .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let expected = serde_json::to_value(core.get_project(&id).unwrap().components).unwrap();
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().components.is_empty());
    core.redo(&id, 2).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().components).unwrap(),
        expected
    );
    let mut locked = create();
    locked["operation"] = json!("component_update");
    locked["componentId"] = json!(outer);
    let mut t = track(vec![]);
    t["locked"] = json!(true);
    locked["tracks"] = json!([t]);
    core.edit(&id, 3, op(locked.clone())).unwrap();
    let locked_before = files(&core, &id);
    assert_eq!(
        core.edit(
            &id,
            4,
            op(json!({"operation":"component_delete","componentId":outer}))
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
    locked["tracks"] = json!([]);
    assert_eq!(
        core.edit(&id, 4, op(locked)).unwrap_err().code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(&core, &id), locked_before);
}

#[test]
fn numeric_boundaries_and_missing_references_publish_nothing() {
    let (_root, core, id) = setup();
    let leaf = core.edit(&id, 0, op(create())).unwrap().changed_ids[0].clone();
    let before = files(&core, &id);
    for (key, value) in [
        ("durationMs", json!(0)),
        ("durationMs", json!(9_007_199_254_740_992u64)),
        ("width", json!(0)),
        ("height", json!(4321)),
    ] {
        let mut v = create();
        v[key] = value;
        assert_eq!(
            core.edit(&id, 1, op(v)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    for (key, value) in [
        ("timeScale", json!(0)),
        ("timeScale", json!(-1)),
        ("timeScale", json!(1.001)),
        ("startMs", json!(1)),
        ("trimStartMs", json!(1)),
        ("durationMs", json!(0)),
    ] {
        let mut item = instance("nested", &leaf, 0);
        item[key] = value;
        let mut v = create();
        v["tracks"] = json!([track(vec![item])]);
        assert_eq!(
            core.edit(&id, 1, op(v)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    let mut missing = create();
    missing["tracks"] = json!([track(vec![instance("nested", "absent", 0)])]);
    assert_eq!(
        core.edit(&id, 1, op(missing)).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut valid = create();
        valid["tracks"] = json!([track(vec![instance("nested", &leaf, 0)])]);
        let EditOperation::ComponentCreate {
            mut tracks,
            name,
            width,
            height,
            duration_ms,
            slots,
        } = op(valid)
        else {
            panic!()
        };
        let opencut_editor_core::TimelineItem::ComponentInstance(v) = &mut tracks[0].items[0]
        else {
            panic!()
        };
        v.time_scale = number;
        assert_eq!(
            core.edit(
                &id,
                1,
                EditOperation::ComponentCreate {
                    name,
                    width,
                    height,
                    duration_ms,
                    tracks,
                    slots,
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(files(&core, &id), before);
}

fn load_definitions(
    core: &EditorCore,
    id: &str,
    definitions: Value,
) -> Result<opencut_editor_core::Project, opencut_editor_core::CoreError> {
    let dir = core.paths().project_dir(id).unwrap();
    let mut project: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    project["components"] = definitions;
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    core.get_project(id)
}

#[test]
fn full_graph_scope_identity_and_longest_path_boundaries() {
    let (_root, core, id) = setup();
    let defs: Vec<_> = (0..18)
        .map(|i| {
            let mut d = catalog()["definition"].clone();
            d["id"] = json!(format!("c{i}"));
            if i > 0 {
                d["tracks"] = json!([track(vec![instance("nested", &format!("c{}", i - 1), 0)])]);
            }
            d
        })
        .collect();
    assert!(load_definitions(&core, &id, json!(&defs[..17])).is_ok());
    assert_eq!(
        load_definitions(&core, &id, json!(defs)).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut a = catalog()["definition"].clone();
    a["id"] = json!("a");
    a["tracks"] = json!([track(vec![instance("local", "b", 0)])]);
    let mut b = a.clone();
    b["id"] = json!("b");
    b["tracks"][0]["items"][0]["componentId"] = json!("a");
    assert_eq!(
        load_definitions(&core, &id, json!([a.clone(), b]))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        load_definitions(&core, &id, json!([a.clone(), a]))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let mut d = catalog()["definition"].clone();
    let group = json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0});
    let mut child = group.clone();
    child["id"] = json!("child");
    child["stackOrder"] = json!(1);
    child["parent"] = json!({"scope":"component:leaf","id":"parent"});
    d["tracks"] = json!([track(vec![group, child])]);
    assert!(load_definitions(&core, &id, json!([d.clone()])).is_ok());
    d["tracks"][0]["items"][1]["parent"]["scope"] = json!("root");
    assert_eq!(
        load_definitions(&core, &id, json!([d])).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn canonical_aggregate_count_limits_are_inclusive() {
    let (_root, core, id) = setup();
    let defs: Vec<_> = (0..513)
        .map(|i| {
            let mut d = catalog()["definition"].clone();
            d["id"] = json!(format!("c{i}"));
            d
        })
        .collect();
    assert!(load_definitions(&core, &id, json!(&defs[..512])).is_ok());
    assert!(load_definitions(&core, &id, json!(defs)).is_err());
    let mut d = catalog()["definition"].clone();
    let tracks: Vec<_> = (0..4097)
        .map(|i| {
            let mut t = track(vec![]);
            t["id"] = json!(format!("t{i}"));
            t
        })
        .collect();
    d["tracks"] = json!(&tracks[..4096]);
    assert!(load_definitions(&core, &id, json!([d.clone()])).is_ok());
    d["tracks"] = json!(tracks);
    assert!(load_definitions(&core, &id, json!([d.clone()])).is_err());
    let items:Vec<_>=(0..4097).map(|i|json!({"type":"group","id":format!("g{i}"),"startMs":0,"durationMs":1000,"stackOrder":i,"zIndex":0})).collect();
    d["tracks"] = json!([track(items[..4096].to_vec())]);
    assert!(load_definitions(&core, &id, json!([d.clone()])).is_ok());
    d["tracks"] = json!([track(items)]);
    assert!(load_definitions(&core, &id, json!([d])).is_err());
}

#[test]
fn all_supported_current_and_mixed_history_migrate_atomically() {
    for version in catalog()["migration"]["sourceVersions"].as_array().unwrap() {
        let (_root, core, id) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        project["schemaVersion"] = version.clone();
        project.as_object_mut().unwrap().remove("components");
        let snapshots = |key: &str| {
            catalog()["migration"][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let mut s = project.clone();
                    s["schemaVersion"] = v.clone();
                    s
                })
                .collect::<Vec<_>>()
        };
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&project).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(
                &json!({"undo":snapshots("undoVersions"),"redo":snapshots("redoVersions")}),
            )
            .unwrap(),
        )
        .unwrap();
        let migrated = core.get_project(&id).unwrap();
        assert_eq!(migrated.schema_version, 12);
        assert!(migrated.components.is_empty());
        let before = files(&core, &id);
        core.get_project(&id).unwrap();
        assert_eq!(files(&core, &id), before);
        let history: Value = serde_json::from_slice(&before.1).unwrap();
        for s in history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap())
        {
            assert_eq!(s["schemaVersion"], 12);
            assert_eq!(s["components"], json!([]));
        }
    }
}

#[test]
fn invalid_current_and_retained_components_never_rewrite() {
    let (_root, core, id) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for history_case in [false, true] {
        for version in [0, 12, 13] {
            let mut bad = original.clone();
            bad["schemaVersion"] = json!(version);
            if version == 12 {
                let mut def = catalog()["definition"].clone();
                def["durationMs"] = json!(0);
                bad["components"] = json!([def]);
            }
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(if history_case { &original } else { &bad }).unwrap(),
            )
            .unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&if history_case {
                    json!({"undo":[bad],"redo":[]})
                } else {
                    json!({"undo":[],"redo":[]})
                })
                .unwrap(),
            )
            .unwrap();
            let before = files(&core, &id);
            assert!(core.get_project(&id).is_err());
            assert_eq!(files(&core, &id), before);
        }
    }
}

#[test]
fn canonical_component_item_validation_is_atomic_at_every_core_boundary() {
    use opencut_editor_core::{MediaProbeFacts, MediaType, Project, Renderer};
    for fixture in catalog()["itemValidationFixtures"].as_array().unwrap() {
        let label = fixture["id"].as_str().unwrap();
        let value = fixture["operation"].clone();
        let decodes = fixture["rustDecode"].as_bool().unwrap();
        assert_eq!(
            serde_json::from_value::<EditOperation>(value.clone()).is_ok(),
            decodes,
            "{label}"
        );
        assert_eq!(
            serde_json::from_value::<BatchEditOperation>(value.clone()).is_ok(),
            decodes,
            "{label}"
        );
        if !decodes {
            continue;
        }
        let (root, core, id) = setup();
        let path = root.path().join("media/source.mp4");
        std::fs::write(&path, b"trusted component fixture").unwrap();
        let asset = core
            .import_asset(
                &id,
                0,
                &path,
                MediaType::Video,
                MediaProbeFacts {
                    duration_ms: Some(1000),
                    has_video: true,
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let value: Value = serde_json::from_str(
            &serde_json::to_string(&value)
                .unwrap()
                .replace("component-fixture-asset", &asset),
        )
        .unwrap();
        let component_id = core.edit(&id, 1, op(create())).unwrap().changed_ids[0].clone();
        let mut update = value.clone();
        update["operation"] = json!("component_update");
        update["componentId"] = json!(component_id);
        if fixture["valid"] == true {
            let draft = core
                .create_draft(&id, 2, vec![op(update.clone())], None)
                .unwrap();
            let expected = serde_json::to_value(
                core.get_draft_state(&id, &draft.id)
                    .unwrap()
                    .project
                    .components,
            )
            .unwrap();
            core.discard_draft(&id, &draft.id).unwrap();
            core.edit(&id, 2, op(update)).unwrap();
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap().components).unwrap(),
                expected,
                "{label}"
            );
            core.undo(&id, 3).unwrap();
            core.redo(&id, 4).unwrap();
            assert_eq!(
                serde_json::to_value(core.get_project(&id).unwrap().components).unwrap(),
                expected,
                "{label}"
            );
            core.edit(&id, 5, op(value)).unwrap();
            continue;
        }
        let before = files(&core, &id);
        for edit in [value.clone(), update.clone()] {
            assert_eq!(
                core.edit(&id, 2, op(edit.clone())).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{label}"
            );
            assert_eq!(
                core.edit_batch::<BatchEditOperation>(
                    &id,
                    2,
                    serde_json::from_value(json!([create(), edit.clone()])).unwrap()
                )
                .unwrap_err()
                .code,
                ErrorCode::InvalidArgument,
                "{label}"
            );
            assert_eq!(
                core.create_draft(&id, 2, vec![op(edit)], None)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidArgument,
                "{label}"
            );
            assert_eq!(files(&core, &id), before, "{label}");
        }
        let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let mut bad = original.clone();
        bad["components"][0]["tracks"] = value["tracks"].clone();
        let project: Project = serde_json::from_value(bad.clone()).unwrap();
        let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
        let output = root.path().join("uncreated-render-dir");
        assert_eq!(
            renderer
                .render_preview(&project, &output, 0)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument,
            "{label}"
        );
        assert!(!output.exists(), "{label}");
        let dir = core.paths().project_dir(&id).unwrap();
        for location in ["current", "undo", "redo"] {
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(if location == "current" {
                    &bad
                } else {
                    &original
                })
                .unwrap(),
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
            let unchanged = files(&core, &id);
            assert_eq!(
                core.get_project(&id).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{label}: {location}"
            );
            assert_eq!(files(&core, &id), unchanged, "{label}: {location}");
        }
    }
}

#[test]
fn typed_nonfinite_component_caption_confidence_is_rejected() {
    use opencut_editor_core::{MediaProbeFacts, MediaType, TimelineItem};
    let (root, core, id) = setup();
    let path = root.path().join("media/source.mp4");
    std::fs::write(&path, b"trusted fixture").unwrap();
    let asset = core
        .import_asset(
            &id,
            0,
            &path,
            MediaType::Video,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_video: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let catalog = catalog();
    let fixture = catalog["itemValidationFixtures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == "caption-moved-words")
        .unwrap();
    let value = serde_json::to_string(&fixture["operation"])
        .unwrap()
        .replace("component-fixture-asset", &asset);
    let before = files(&core, &id);
    for confidence in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for word in [false, true] {
            let mut edit: EditOperation = serde_json::from_str(&value).unwrap();
            let EditOperation::ComponentCreate { tracks, .. } = &mut edit else {
                unreachable!()
            };
            let TimelineItem::Caption(caption) = &mut tracks[0].items[0] else {
                unreachable!()
            };
            if word {
                caption.source.words[0].confidence = Some(confidence);
            } else {
                caption.source.confidence = Some(confidence);
            }
            assert_eq!(
                core.edit(&id, 1, edit).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(files(&core, &id), before);
        }
    }
}
