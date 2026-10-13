use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, MediaProbeFacts, MediaType,
    PathPolicy, ProjectSettings, TrackType,
};
use serde_json::{Value, json};

fn recipe() -> Value {
    serde_json::from_str(include_str!("../../../contracts/native-role-team-v1.json")).unwrap()
}
fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String, String) {
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
        .create_project("Native role team", ProjectSettings::default())
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
    let source = media.join("original.bmp");
    std::fs::write(
        &source,
        b"Original synthetic image fixture for domain-only managed ownership",
    )
    .unwrap();
    let asset = core
        .import_asset(
            &id,
            0,
            &source,
            MediaType::Image,
            MediaProbeFacts::default(),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    (root, core, id, track, asset)
}
fn files(core: &EditorCore, id: &str) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        dir: &std::path::Path,
        result: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, result);
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    walk(&core.paths().project_dir(id).unwrap(), &mut result);
    result
}
fn content(core: &EditorCore, id: &str) -> Value {
    let mut p = serde_json::to_value(core.get_project(id).unwrap()).unwrap();
    p.as_object_mut().unwrap().remove("revision");
    p.as_object_mut().unwrap().remove("updatedAtMs");
    p
}
fn replace(v: &mut Value, track: &str, asset: &str) {
    match v {
        Value::String(s) => {
            if s == "$overlay" {
                *s = track.into()
            } else if s == "$artwork" {
                *s = asset.into()
            }
        }
        Value::Array(a) => {
            for x in a {
                replace(x, track, asset)
            }
        }
        Value::Object(o) => {
            for x in o.values_mut() {
                replace(x, track, asset)
            }
        }
        _ => {}
    }
}
fn edits(layout: &str, n: usize, track: &str, asset: &str) -> Value {
    let r = recipe();
    let mut a = vec![r["badgeCreate"].clone(), r["badgeDefineSlots"].clone()];
    a.extend(
        r["layouts"][layout][n.to_string()]
            .as_array()
            .unwrap()
            .iter()
            .cloned(),
    );
    let mut v = json!(a);
    replace(&mut v, track, asset);
    v
}
fn update(item: &str, component: &str, values: Value) -> Value {
    json!({"operation":"component_instance_update","itemId":item,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values})
}

#[test]
fn every_supported_count_layout_has_exact_group_slots_motion_and_history() {
    for layout in ["row", "grid"] {
        for n in 1..=4 {
            let (_root, core, id, track, asset) = setup();
            let before = content(&core, &id);
            let result = core
                .edit_batch::<BatchEditOperation>(
                    &id,
                    1,
                    serde_json::from_value(edits(layout, n, &track, &asset)).unwrap(),
                )
                .unwrap();
            assert_eq!(result.revision, 2);
            assert_eq!(result.aliases.len(), n + 2);
            let saved = content(&core, &id);
            let def = &saved["components"][0];
            assert_eq!(def["tracks"][0]["items"].as_array().unwrap().len(), 9);
            assert_eq!(def["slots"].as_array().unwrap().len(), 10);
            let items = saved["tracks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == track)
                .unwrap()["items"]
                .as_array()
                .unwrap();
            assert_eq!(items.len(), n + 1);
            for i in 0..n {
                let member = &items[i + 1];
                assert_eq!(member["id"], result.aliases[&format!("member_{}", i + 1)]);
                assert_eq!(
                    member["parent"],
                    json!({"scope":"root","id":result.aliases["team"]})
                );
                let (x, y) = if layout == "row" {
                    (100 + 450 * i, 390)
                } else {
                    (480 + 560 * (i % 2), 160 + 400 * (i / 2))
                };
                assert_eq!(member["transform"]["positionX"].as_f64().unwrap(), x as f64);
                assert_eq!(member["transform"]["positionY"].as_f64().unwrap(), y as f64);
                assert_eq!(member["transform"]["scale"], 0.5);
                assert_eq!(
                    member["animationChannels"][0]["property"],
                    "transform.opacity"
                );
                assert_eq!(
                    member["animationChannels"][0]["keyframes"][0]["timeMs"],
                    80 * i
                );
                assert_eq!(
                    member["animationChannels"][0]["keyframes"][1]["timeMs"],
                    80 * i + 100
                );
            }
            core.undo(&id, 2).unwrap();
            assert_eq!(content(&core, &id), before);
            core.redo(&id, 3).unwrap();
            assert_eq!(content(&core, &id), saved);
            assert_eq!(content(&core, &id), saved);
        }
    }
}
#[test]
fn independent_overrides_reset_label_boundaries_and_atomic_failures() {
    let (root, core, id, track, asset) = setup();
    let a = core
        .edit_batch::<BatchEditOperation>(
            &id,
            1,
            serde_json::from_value(edits("grid", 4, &track, &asset)).unwrap(),
        )
        .unwrap()
        .aliases;
    let alternate = root.path().join("media/alternate.bmp");
    std::fs::write(&alternate, b"Independent managed image fixture").unwrap();
    let alternate = core
        .import_asset(
            &id,
            2,
            &alternate,
            MediaType::Image,
            MediaProbeFacts::default(),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let baseline = content(&core, &id);
    let values = json!({"role_label":{"type":"text","value":"😀".repeat(24)},"artwork":{"type":"asset","value":{"kind":"asset","scope":"project","id":alternate}},"background_color":{"type":"color","value":"#00ff00"},"rays_hidden":{"type":"boolean","value":true}});
    core.edit(
        &id,
        3,
        op(update(&a["member_1"], &a["badge"], values.clone())),
    )
    .unwrap();
    let changed = content(&core, &id);
    assert_eq!(changed["components"], baseline["components"]);
    let changed_member = changed["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["items"].as_array().unwrap())
        .find(|item| item["id"] == a["member_1"])
        .unwrap();
    assert_eq!(changed_member["slotValues"], values);

    assert_eq!(
        changed["tracks"][1]["items"][2],
        baseline["tracks"][1]["items"][2]
    );
    let mut revision = 4;
    for (value, code) in [
        (
            json!({"role_label":{"type":"text","value":""}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"role_label":{"type":"text","value":"X".repeat(25)}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"background_color":{"type":"color","value":"#nope00"}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"rays_hidden":{"type":"text","value":"true"}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"artwork":{"type":"asset","value":{"kind":"asset","scope":"project","id":"missing"}}}),
            ErrorCode::AssetNotFound,
        ),
    ] {
        let before = files(&core, &id);
        let error = core
            .edit(
                &id,
                revision,
                op(update(&a["member_1"], &a["badge"], value)),
            )
            .unwrap_err();
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert_eq!(files(&core, &id), before);
    }
    let before = files(&core, &id);
    let conflict = core
        .edit(
            &id,
            revision - 1,
            op(update(&a["member_1"], &a["badge"], json!({}))),
        )
        .unwrap_err();
    assert_eq!(conflict.code, ErrorCode::RevisionConflict);
    assert!(conflict.retryable);
    assert_eq!(files(&core, &id), before);
    core.edit(
        &id,
        revision,
        op(update(&a["member_1"], &a["badge"], json!({}))),
    )
    .unwrap();
    revision += 1;
    let reset = content(&core, &id);
    let member = reset["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["items"].as_array().unwrap())
        .find(|i| i["id"] == a["member_1"])
        .unwrap();
    assert_eq!(member["slotValues"], json!({}));
    assert_eq!(reset["components"], baseline["components"]);
    core.undo(&id, revision).unwrap();
    assert_eq!(content(&core, &id), changed);
    core.redo(&id, revision + 1).unwrap();
    assert_eq!(content(&core, &id), reset);
}
#[test]
fn full_construction_tail_alias_missing_asset_and_lock_fail_without_publication() {
    for failure in ["tail", "alias", "asset", "lock"] {
        let (_root, core, id, track, asset) = setup();
        let mut operations = edits("row", 4, &track, &asset);
        let code = match failure {
            "tail" => {
                operations
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"operation":"delete_item","itemId":"absent"}));
                ErrorCode::ItemNotFound
            }
            "alias" => {
                operations[3]["componentId"] = json!("@future_badge");
                ErrorCode::ValidationFailed
            }
            "asset" => {
                replace(&mut operations, &track, "absent");
                operations[0]["tracks"][0]["items"][5]["assetId"] = json!("absent");
                ErrorCode::ItemNotFound
            }
            _ => {
                let path = core.paths().project_dir(&id).unwrap().join("project.json");
                let mut p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
                for t in p["tracks"].as_array_mut().unwrap() {
                    if t["id"] == track {
                        t["locked"] = json!(true)
                    }
                }
                std::fs::write(path, serde_json::to_vec_pretty(&p).unwrap()).unwrap();
                ErrorCode::TrackLocked
            }
        };
        let before = files(&core, &id);
        let error = core
            .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(operations).unwrap())
            .unwrap_err();
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert_eq!(files(&core, &id), before);
    }
}

#[test]
fn minimum_label_and_parent_movement_preserve_exact_relative_members() {
    let (_root, core, id, track, asset) = setup();
    let result = core
        .edit_batch::<BatchEditOperation>(
            &id,
            1,
            serde_json::from_value(edits("grid", 4, &track, &asset)).unwrap(),
        )
        .unwrap();
    let a = result.aliases;
    core.edit(
        &id,
        2,
        op(update(
            &a["member_1"],
            &a["badge"],
            json!({"role_label":{"type":"text","value":"A"}}),
        )),
    )
    .unwrap();
    let before = content(&core, &id);
    core.edit(&id,3,op(json!({"operation":"update_item","itemId":a["team"],"transform2d":{"position":{"x":20,"y":30,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}}))).unwrap();
    let moved = content(&core, &id);
    assert_eq!(moved["components"], before["components"]);
    let root_items = |v: &Value| {
        v["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|t| t["items"].as_array().unwrap())
            .cloned()
            .collect::<Vec<Value>>()
    };
    for member in root_items(&before).iter().filter(|i| i["id"] != a["team"]) {
        assert!(root_items(&moved).contains(member));
    }
    let group = root_items(&moved)
        .into_iter()
        .find(|i| i["id"] == a["team"])
        .unwrap();
    assert_eq!(group["transform2d"]["position"]["x"].as_f64(), Some(20.0));
    assert_eq!(group["transform2d"]["position"]["y"].as_f64(), Some(30.0));
    core.undo(&id, 4).unwrap();
    assert_eq!(content(&core, &id), before);
    core.redo(&id, 5).unwrap();
    assert_eq!(content(&core, &id), moved);
}
