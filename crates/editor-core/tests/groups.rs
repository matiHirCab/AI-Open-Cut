use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
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
        .create_project("Groups", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn group(track: &str, alias: &str) -> Value {
    json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":alias})
}

#[test]
fn canonical_group_payloads_are_closed() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-parent-v1.json")).unwrap();
    for fixture in catalog["valid"].as_array().unwrap() {
        serde_json::from_value::<EditOperation>(fixture["value"].clone()).unwrap();
        if fixture["value"]["operation"] == "group_ungroup" {
            serde_json::from_value::<BatchEditOperation>(fixture["value"].clone()).unwrap();
        }
    }
    for fixture in catalog["invalid"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(fixture["value"].clone()).is_err(),
            "{fixture}"
        );
    }
    for fixture in catalog["invalid"].as_array().unwrap() {
        if fixture["value"]["operation"] == "group_ungroup" {
            assert!(
                serde_json::from_value::<BatchEditOperation>(fixture["value"].clone()).is_err(),
                "{fixture}"
            );
        }
    }
    let (_root, core, id, track) = setup();
    let mut creation = catalog["valid"][0]["value"].clone();
    creation["trackId"] = json!(track);
    let result = core.edit(&id, 0, op(creation)).unwrap();
    let state = core.get_project(&id).unwrap();
    let item = serde_json::to_value(state.find_item(&result.changed_ids[0]).unwrap()).unwrap();
    for (key, value) in catalog["defaults"].as_object().unwrap() {
        if key == "transform2d" {
            assert_eq!(
                serde_json::from_value::<opencut_editor_core::Transform2D>(item[key].clone())
                    .unwrap(),
                serde_json::from_value::<opencut_editor_core::Transform2D>(value.clone()).unwrap()
            );
        } else {
            assert_eq!(item[key], *value, "default {key}");
        }
    }
    assert!(item.as_object().unwrap().keys().all(|key| {
        catalog["groupShape"]["fields"]
            .as_array()
            .unwrap()
            .contains(&json!(key))
    }));
    let mut unknown = item.clone();
    unknown["children"] = json!([]);
    assert!(serde_json::from_value::<opencut_editor_core::TimelineItem>(unknown).is_err());
}

#[test]
fn persisted_graph_count_duplicate_and_reference_boundaries() {
    let (_root, core, id, track) = setup();
    core.edit(
        &id,
        0,
        op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let template = state["tracks"][1]["items"][0].clone();
    let items: Vec<Value> = (0..4096)
        .map(|index| {
            let mut v = template.clone();
            v["id"] = json!(format!("g{index}"));
            v["stackOrder"] = json!(index);
            v
        })
        .collect();
    state["tracks"][1]["items"] = json!(items);
    let save = |value: &Value| {
        std::fs::write(dir.join("project.json"), serde_json::to_vec(value).unwrap()).unwrap()
    };
    save(&state);
    assert_eq!(core.get_project(&id).unwrap().tracks[1].items.len(), 4096);
    let mut overflow = state.clone();
    let mut extra = template.clone();
    extra["id"] = json!("overflow");
    extra["stackOrder"] = json!(4096);
    overflow["tracks"][1]["items"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    save(&overflow);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut duplicate = state.clone();
    duplicate["tracks"][1]["items"][1]["id"] = json!("g0");
    save(&duplicate);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut hidden = state.clone();
    hidden["tracks"][1]["items"][0]["hidden"] = json!(true);
    hidden["tracks"][1]["items"][0]["parent"] = json!({"scope":"root","id":"g0"});
    save(&hidden);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn canonical_graph_failures_are_rejected_on_open_without_publication() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-parent-v1.json")).unwrap();
    for fixture in catalog["graphFailures"].as_array().unwrap() {
        let (_root, core, id, track) = setup();
        core.edit(
            &id,
            0,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let template = state["tracks"][1]["items"][0].clone();
        let mut items = Vec::new();
        let names: Vec<String> = if let Some(count) = fixture["count"].as_u64() {
            (0..count).map(|i| format!("g{i}")).collect()
        } else if let Some(depth) = fixture["depth"].as_u64() {
            (0..=depth).map(|i| format!("g{i}")).collect()
        } else {
            ["child", "group", "a", "b"].map(str::to_string).to_vec()
        };
        for (index, name) in names.iter().enumerate() {
            let mut item = template.clone();
            item["id"] = json!(name);
            item["stackOrder"] = json!(index);
            if fixture["depth"].is_number() && index > 0 {
                item["parent"] = json!({"scope":"root","id":names[index-1]});
            }
            if let Some(edges) = fixture["parents"].as_array() {
                for edge in edges {
                    if edge[0] == *name {
                        item["parent"] = json!({"scope":"root","id":edge[1]});
                    }
                }
            }
            if fixture["scope"].is_string() && index == 0 {
                item["parent"] = json!({"scope":fixture["scope"],"id":"group"});
            }
            items.push(item);
        }
        state["tracks"][1]["items"] = json!(items);
        let bytes = serde_json::to_vec(&state).unwrap();
        std::fs::write(dir.join("project.json"), &bytes).unwrap();
        let history = std::fs::read(dir.join("history.json")).unwrap();
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap_err().code).unwrap(),
            fixture["error"],
            "{fixture}"
        );
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
        let mut legacy = state.clone();
        legacy["schemaVersion"] = json!(9);
        legacy["tracks"][1]["items"] = json!([]);
        let legacy_bytes = serde_json::to_vec(&legacy).unwrap();
        let retained = serde_json::to_vec(&json!({"undo":[state],"redo":[]})).unwrap();
        std::fs::write(dir.join("project.json"), &legacy_bytes).unwrap();
        std::fs::write(dir.join("history.json"), &retained).unwrap();
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap_err().code).unwrap(),
            fixture["error"]
        );
        assert_eq!(
            std::fs::read(dir.join("project.json")).unwrap(),
            legacy_bytes
        );
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), retained);
    }
}

#[test]
fn group_alias_graph_failures_rollback_and_history() {
    let (_root, core, id, track) = setup();
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        group(&track,"a"), group(&track,"b"),
        {"operation":"item_set_parent","itemId":"@b","parent":{"scope":"root","id":"@a"}}
    ]))
    .unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    let a = &result.aliases["a"];
    let b = &result.aliases["b"];
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for (parent, code) in [
        (json!({"scope":"root","id":a}), ErrorCode::InvalidArgument),
        (json!({"scope":"root","id":b}), ErrorCode::InvalidArgument),
        (
            json!({"scope":"root","id":"absent"}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"scope":"component:other","id":b}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"scope":"root","id":"https://host/group"}),
            ErrorCode::InvalidArgument,
        ),
    ] {
        assert_eq!(
            core.edit(
                &id,
                1,
                op(json!({"operation":"item_set_parent","itemId":a,"parent":parent}))
            )
            .unwrap_err()
            .code,
            code
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    assert_eq!(
        core.edit(&id, 1, op(json!({"operation":"delete_item","itemId":a})))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.edit(
            &id,
            0,
            op(json!({"operation":"item_set_parent","itemId":b,"parent":null}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let invalid: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"item_set_parent","itemId":b,"parent":null},
        {"operation":"item_set_parent","itemId":a,"parent":{"scope":"root","id":"absent"}}
    ]))
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, invalid).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().find_item(a).is_none());
    core.redo(&id, 2).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(b)
            .unwrap()
            .visual_properties()
            .parent
            .as_ref()
            .unwrap()
            .id,
        *a
    );
    core.edit(
        &id,
        3,
        op(json!({"operation":"item_set_parent","itemId":b,"parent":null})),
    )
    .unwrap();
    core.edit(&id, 4, op(json!({"operation":"delete_item","itemId":a})))
        .unwrap();
}

#[test]
fn exact_parent_depth_boundary() {
    let (_root, core, id, track) = setup();
    let mut values = vec![group(&track, "g0")];
    for depth in 1..=32 {
        let mut value = group(&track, &format!("g{depth}"));
        value["parent"] = json!({"scope":"root","id":format!("@g{}",depth-1)});
        values.push(value);
    }
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!(values)).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    assert_eq!(result.aliases.len(), 33);
    let value = json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"parent":{"scope":"root","id":result.aliases["g32"]}});
    assert_eq!(
        core.edit(&id, 1, op(value)).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(core.get_project(&id).unwrap().revision, 1);
}

#[test]
fn cross_track_parenting_preserves_child_lifecycle_and_locked_parent() {
    let (_root, core, id, track) = setup();
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        group(&track,"parent"),
        {"operation":"create_track","trackType":"overlay","name":"Children","resultAlias":"children"},
        {"operation":"add_rectangle","trackId":"@children","startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":7,"positionY":9,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"update_track","trackId":track,"locked":true},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@parent"}}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    let child = &result.aliases["child"];
    let parent = &result.aliases["parent"];
    let original = core
        .get_project(&id)
        .unwrap()
        .find_item(child)
        .unwrap()
        .visual_properties()
        .transform
        .clone();
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"split_item","itemId":child,"splitMs":500},
        {"operation":"duplicate_items","itemIds":[child],"offsetMs":1000},
        {"operation":"move_item","itemId":child,"trackId":result.aliases["children"],"startMs":2000}
    ]))
    .unwrap();
    core.edit_batch(&id, 1, batch).unwrap();
    let state = core.get_project(&id).unwrap();
    for item in &state.tracks.last().unwrap().items {
        assert_eq!(
            item.visual_properties().parent.as_ref().unwrap().id,
            *parent
        );
        assert_eq!(item.visual_properties().transform, original);
    }
    assert_eq!(state.tracks.last().unwrap().items.len(), 3);
    let before = serde_json::to_value(&state).unwrap();
    assert_eq!(core.edit(&id,2,op(json!({"operation":"item_set_parent","itemId":child,"parent":{"scope":"root","id":child}}))).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
    core.edit(
        &id,
        2,
        op(json!({"operation":"item_set_parent","itemId":child,"parent":null})),
    )
    .unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .find_item(child)
            .unwrap()
            .visual_properties()
            .transform,
        original
    );
    let video = &state.tracks[0].id;
    assert_eq!(
        core.edit(
            &id,
            3,
            op(json!({"operation":"add_group","trackId":video,"startMs":0,"durationMs":1000}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn migration_preserves_every_supported_history_and_rejects_bad_graphs_atomically() {
    for version in 1..=9 {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        state["schemaVersion"] = json!(version);
        let mut oldest = state.clone();
        oldest["schemaVersion"] = json!(1);
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&json!({"undo":[oldest],"redo":[state]})).unwrap(),
        )
        .unwrap();
        assert_eq!(core.get_project(&id).unwrap().schema_version, 10);
        let history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        assert_eq!(history["undo"][0]["schemaVersion"], 10);
        assert_eq!(history["redo"][0]["schemaVersion"], 10);
        let once = std::fs::read(dir.join("project.json")).unwrap();
        core.get_project(&id).unwrap();
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), once);
    }
    let (_root, core, id, track) = setup();
    core.edit(
        &id,
        0,
        op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for scope in ["root", "component:other"] {
        let mut invalid = state.clone();
        invalid["tracks"][1]["items"][0]["parent"] = json!({"scope":scope,"id":"absent"});
        let current = serde_json::to_vec(&state).unwrap();
        let history = serde_json::to_vec(&json!({"undo":[invalid],"redo":[]})).unwrap();
        std::fs::write(dir.join("project.json"), &current).unwrap();
        std::fs::write(dir.join("history.json"), &history).unwrap();
        assert!(core.get_project(&id).is_err());
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), current);
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
    }
}

#[test]
fn group_unsupported_edits_locks_and_node_only_duplication() {
    let (_root, core, id, track) = setup();
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([group(&track,"a"),group(&track,"b"),{"operation":"item_set_parent","itemId":"@b","parent":{"scope":"root","id":"@a"}}])).unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    let a = &result.aliases["a"];
    for value in [
        json!({"operation":"set_keyframes","itemId":a,"keyframes":[]}),
        json!({"operation":"split_item","itemId":a,"splitMs":500}),
        json!({"operation":"set_audio","itemId":a,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}}),
        json!({"operation":"update_item","itemId":a,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}),
        json!({"operation":"add_transition","trackId":track,"fromItemId":a,"startMs":0,"durationMs":100,"transitionType":"fade"}),
    ] {
        assert_eq!(
            core.edit(&id, 1, op(value)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let result2 = core
        .edit(
            &id,
            1,
            op(json!({"operation":"duplicate_items","itemIds":[a],"offsetMs":0})),
        )
        .unwrap();
    let state = core.get_project(&id).unwrap();
    assert_eq!(state.tracks[1].items.len(), 3);
    assert_ne!(result2.changed_ids[0], *a);
    assert_eq!(
        state
            .find_item(&result.aliases["b"])
            .unwrap()
            .visual_properties()
            .parent
            .as_ref()
            .unwrap()
            .id,
        *a
    );
    assert_eq!(
        core.edit(
            &id,
            2,
            op(json!({"operation":"delete_track","trackId":track}))
        )
        .unwrap_err()
        .code,
        ErrorCode::ValidationFailed
    );
    core.edit(
        &id,
        2,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    assert_eq!(
        core.edit(
            &id,
            3,
            op(json!({"operation":"item_set_parent","itemId":a,"parent":null}))
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
}

#[test]
fn group_static_edits_and_nonfinite_values_are_transactional() {
    let (_root, core, id, track) = setup();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut transform = opencut_editor_core::Transform2D::default();
        transform.position.x = value;
        assert_eq!(
            core.edit(
                &id,
                0,
                EditOperation::AddGroup {
                    track_id: track.clone(),
                    start_ms: 0,
                    duration_ms: 1000,
                    transform2d: Some(transform),
                    parent: None
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(core.get_project(&id).unwrap().revision, 0);
    }
    let result = core
        .edit(
            &id,
            0,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap();
    let group = &result.changed_ids[0];
    let transform = opencut_editor_core::Transform2D {
        rotation_deg: 23.0,
        ..Default::default()
    };
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"update_item","itemId":group,"transform2d":transform},
        {"operation":"set_item_visibility","itemId":group,"hidden":true},
        {"operation":"trim_item","itemId":group,"startMs":100,"durationMs":500},
        {"operation":"item_set_z_index","itemId":group,"zIndex":2}
    ]))
    .unwrap();
    core.edit_batch(&id, 1, operations).unwrap();
    let state = core.get_project(&id).unwrap();
    let item = state.find_item(group).unwrap();
    assert!(item.hidden());
    assert_eq!(item.start_ms(), 100);
    assert_eq!(item.duration_ms(), 500);
    assert_eq!(item.visual_properties().transform2d, Some(transform));
    assert_eq!(item.visual_properties().z_index, 2);
}

#[test]
fn native_grouped_animated_child_matches_preview_range_and_export() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer, Transform2D};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(
            std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
            Ok("1"),
            "native group test requires configured FFmpeg"
        );
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").expect("FFprobe required");
    let (_root, core, id, track) = setup();
    let mut transform = Transform2D::default();
    transform.position.x = 40.0;
    transform.position.y = 10.0;
    transform.rotation_deg = 90.0;
    let operations:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_group","trackId":track,"startMs":200,"durationMs":600,"transform2d":transform,"resultAlias":"group"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_keyframes","itemId":"@box","keyframes":[{"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},{"property":"position","timeMs":1000,"value":{"type":"position","x":10,"y":0},"easing":"linear"}]},
        {"operation":"item_set_parent","itemId":"@box","parent":{"scope":"root","id":"@group"}}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, operations).unwrap();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    let before = serde_json::to_value(&project).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let preview = renderer.render_preview(&project, &dir, 500).unwrap();
    let range = renderer
        .render_preview_range(
            &project,
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
    let export = dir.join("groups.mp4");
    renderer
        .export_video(
            &project,
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
    let decode = |path: &std::path::Path, time: &str| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", time, "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout.len(), 64 * 64 * 3);
        output.stdout
    };
    let reference = decode(&dir.join(preview.relative_path), "0");
    let draft=core.create_draft(&id,1,vec![op(json!({"operation":"item_set_parent","itemId":result.aliases["box"],"parent":{"scope":"root","id":result.aliases["group"]}}))],None).unwrap();
    let project_before = std::fs::read(dir.join("project.json")).unwrap();
    let history_before = std::fs::read(dir.join("history.json")).unwrap();
    let mut materialized = core.get_draft_state(&id, &draft.id).unwrap().project;
    materialized.settings = project.settings.clone();
    let draft_frame = renderer.render_preview(&materialized, &dir, 500).unwrap();
    assert_eq!(decode(&dir.join(draft_frame.relative_path), "0"), reference);
    assert_eq!(
        std::fs::read(dir.join("project.json")).unwrap(),
        project_before
    );
    assert_eq!(
        std::fs::read(dir.join("history.json")).unwrap(),
        history_before
    );
    let red = |bytes: &[u8], x: usize, y: usize| {
        let at = (y * 64 + x) * 3;
        bytes[at] > 180 && bytes[at + 1] < 60 && bytes[at + 2] < 60
    };
    assert!(red(&reference, 35, 20));
    assert!(!red(&reference, 45, 20));
    assert!(!red(&reference, 35, 5));
    for path in [dir.join(range.relative_path), export] {
        let frame = decode(&path, "0.5");
        assert!(red(&frame, 35, 20));
        assert!(!red(&frame, 45, 20));
        let outside = decode(&path, "0");
        assert!(!red(&outside, 35, 20));
    }
    assert_eq!(serde_json::to_value(&project).unwrap(), before);
}

#[test]
fn persisted_group_transition_endpoints_fail_closed_in_every_snapshot() {
    for endpoint in ["fromItemId", "toItemId"] {
        for hidden in [false, true] {
            for location in ["current", "undo", "redo"] {
                let (_root, core, id, track) = setup();
                let created=core.edit(&id,0,op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}))).unwrap();
                let group = &created.changed_ids[0];
                let dir = core.paths().project_dir(&id).unwrap();
                let valid = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
                let mut bad = valid.clone();
                bad["tracks"][1]["items"][0]["hidden"] = json!(hidden);
                let mut transition = json!({"type":"transition","id":"transition","startMs":0,"durationMs":100,"transitionType":"fade","fromItemId":"missing","toItemId":null,"hidden":hidden,"zIndex":0,"stackOrder":1});
                transition[endpoint] = json!(group);
                bad["tracks"][1]["items"]
                    .as_array_mut()
                    .unwrap()
                    .push(transition);
                let mut history = json!({"undo":[],"redo":[]});
                let current = if location == "current" {
                    bad.clone()
                } else {
                    history[location] = json!([bad]);
                    valid
                };
                let project_bytes = serde_json::to_vec(&current).unwrap();
                let history_bytes = serde_json::to_vec(&history).unwrap();
                std::fs::write(dir.join("project.json"), &project_bytes).unwrap();
                std::fs::write(dir.join("history.json"), &history_bytes).unwrap();
                assert_eq!(
                    core.get_project(&id).unwrap_err().code,
                    ErrorCode::InvalidArgument,
                    "{endpoint} {hidden} {location}"
                );
                assert_eq!(
                    std::fs::read(dir.join("project.json")).unwrap(),
                    project_bytes
                );
                assert_eq!(
                    std::fs::read(dir.join("history.json")).unwrap(),
                    history_bytes
                );
            }
        }
    }
}

#[test]
fn native_long_distance_motion_tiles_reentry_and_draft_agree() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (_root, core, id, track) = setup();
    let keys:Vec<Value>=[(0,20.0),(250,4090.0),(500,20000.0),(750,4090.0),(1000,20.0)].into_iter().map(|(time,x)|json!({"property":"position","timeMs":time,"value":{"type":"position","x":x,"y":0},"easing":"linear"})).collect();
    let operations:Vec<BatchEditOperation>=serde_json::from_value(json!([
        group(&track,"g"),
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":20,"positionY":0,"scale":1,"opacity":0.5},"resultAlias":"r"},
        {"operation":"set_keyframes","itemId":"@r","keyframes":keys},
        {"operation":"item_set_parent","itemId":"@r","parent":{"scope":"root","id":"@g"}}
    ])).unwrap();
    let created = core.edit_batch(&id, 0, operations).unwrap();
    let mut p = core.get_project(&id).unwrap();
    p.settings.width = 4200;
    p.settings.height = 32;
    p.settings.fps = 20;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let decode = |path: &std::path::Path, time: &str| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", time, "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout.len(), 4200 * 32 * 3);
        output.stdout
    };
    let frame = renderer.render_preview(&p, &dir, 250).unwrap();
    let reference = decode(&dir.join(&frame.relative_path), "0");
    let check_seam = |pixels: &[u8], tolerance: u8| {
        let red = |x: usize| pixels[(5 * 4200 + x) * 3];
        assert!((100..155).contains(&red(4095)));
        assert!((100..155).contains(&red(4096)));
        assert!(
            red(4095).abs_diff(red(4096)) <= tolerance,
            "seam: {} / {}",
            red(4095),
            red(4096)
        );
        assert!(red(4085) < 10);
        assert!(red(4115) < 10);
    };
    // Lossless pixels must be continuous; encoded video permits codec rounding.
    check_seam(&reference, 3);
    let range = renderer
        .render_preview_range(
            &p,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 4200,
                height: 32,
                fps: 20,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let export = dir.join("travel.mp4");
    renderer
        .export_video(
            &p,
            &dir,
            ExportOptions {
                output: &export,
                width: 4200,
                height: 32,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    for path in [dir.join(range.relative_path), export] {
        check_seam(&decode(&path, "0.25"), 15);
        check_seam(&decode(&path, "0.75"), 15);
        assert!(decode(&path, "0.5").iter().all(|v| *v < 10));
        let comparison=std::process::Command::new(&ffmpeg).args(["-v","info","-i"]).arg(dir.join(&frame.relative_path)).args(["-ss","0.25","-i"]).arg(&path).args(["-lavfi","[0:v]scale=in_range=auto:out_range=tv,format=yuv420p[a];[1:v]scale=in_range=auto:out_range=tv,format=yuv420p[b];[a][b]ssim","-frames:v","1","-f","null","-"]).output().unwrap();
        assert!(comparison.status.success());
        let stderr = String::from_utf8_lossy(&comparison.stderr);
        let score: f64 = stderr
            .split("All:")
            .last()
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(score >= 0.99, "motion SSIM {score}");
    }
    let draft=core.create_draft(&id,1,vec![op(json!({"operation":"item_set_parent","itemId":created.aliases["r"],"parent":{"scope":"root","id":created.aliases["g"]}}))],None).unwrap();
    let project_bytes = std::fs::read(dir.join("project.json")).unwrap();
    let history_bytes = std::fs::read(dir.join("history.json")).unwrap();
    let mut draft_state = core.get_draft_state(&id, &draft.id).unwrap().project;
    draft_state.settings = p.settings.clone();
    let draft_frame = renderer.render_preview(&draft_state, &dir, 250).unwrap();
    assert_eq!(decode(&dir.join(draft_frame.relative_path), "0"), reference);
    assert_eq!(
        std::fs::read(dir.join("project.json")).unwrap(),
        project_bytes
    );
    assert_eq!(
        std::fs::read(dir.join("history.json")).unwrap(),
        history_bytes
    );
}

#[test]
fn native_identity_parent_preserves_every_styled_text_anchor() {
    use opencut_editor_core::Renderer;
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let font = std::env::var_os("OPENCUT_TEST_FONT_PATH").unwrap();
    let (_root, core, id, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font.into()));
    let mut p = core.get_project(&id).unwrap();
    p.settings.width = 320;
    p.settings.height = 180;
    p.settings.fps = 20;
    let bounds = |path: &std::path::Path| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let mut b = (320i32, 180i32, 0i32, 0i32);
        for (i, pixel) in output.stdout.as_chunks::<3>().0.iter().enumerate() {
            if pixel.iter().all(|v| *v > 128) {
                let x = (i % 320) as i32;
                let y = (i / 320) as i32;
                b = (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y));
            }
        }
        b
    };
    for anchor in [
        "top_left",
        "top_center",
        "top_right",
        "center_left",
        "center",
        "center_right",
        "bottom_left",
        "bottom_center",
        "bottom_right",
    ] {
        for animated in [false, true] {
            let keys = if animated {
                json!([
                {"property":"position","timeMs":0,"value":{"type":"position","x":150,"y":80},"easing":"linear"},
                {"property":"position","timeMs":1000,"value":{"type":"position","x":170,"y":100},"easing":"linear"},
                {"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"},
                {"property":"scale","timeMs":1000,"value":{"type":"scalar","value":1.5},"easing":"linear"}])
            } else {
                json!([])
            };
            p.tracks[1].items=serde_json::from_value(json!([
                {"type":"text","id":"text","text":"ANCHOR","fontSize":18,"color":"#ffffff","startMs":0,"durationMs":1000,"stackOrder":0,"style":{"anchor":anchor,"padding":{"left":2,"right":4,"top":2,"bottom":4},"outlineWidthPx":1,"outlineColor":"#224466"},"transform":{"positionX":160,"positionY":90,"scale":1.25,"opacity":1},"keyframes":keys}
            ])).unwrap();
            let legacy = renderer.render_preview(&p, &dir, 500).unwrap();
            let expected = bounds(&dir.join(legacy.relative_path));
            p.tracks[1].items[0].visual_properties_mut().parent =
                Some(opencut_editor_core::ParentReference {
                    scope: "root".into(),
                    id: "g".into(),
                });
            p.tracks[1].items.push(
                serde_json::from_value(
                    json!({"type":"group","id":"g","startMs":0,"durationMs":1000,"stackOrder":1}),
                )
                .unwrap(),
            );
            let grouped = renderer.render_preview(&p, &dir, 500).unwrap();
            let actual = bounds(&dir.join(grouped.relative_path));
            // Legacy YUV overlay snaps to a chroma grid; affine bilinear sampling
            // preserves subpixel coordinates. Allow one 2-pixel chroma cell in glyph bounds.
            for (a, b) in [
                (actual.0, expected.0),
                (actual.1, expected.1),
                (actual.2, expected.2),
                (actual.3, expected.3),
            ] {
                assert!(
                    (a - b).abs() <= 2,
                    "anchor {anchor} animated {animated}: expected {expected:?}, actual {actual:?}"
                );
            }
        }
    }
}

#[test]
fn group_endpoints_are_rejected_before_draft_publication() {
    let (_root, core, id, track) = setup();
    let group = core
        .edit(
            &id,
            0,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    for endpoint in ["fromItemId", "toItemId"] {
        let mut transition = json!({"operation":"add_transition","trackId":track,"startMs":0,"durationMs":100,"transitionType":"fade","fromItemId":group});
        transition[endpoint] = json!(group);
        assert_eq!(
            core.create_draft(&id, 1, vec![op(transition)], None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
    assert!(!dir.join("drafts").exists());
}

fn ungroup_fixture(track: &str) -> Vec<BatchEditOperation> {
    serde_json::from_value(json!([
        {"operation":"create_track","trackType":"overlay","name":"Children","resultAlias":"children"},
        {"operation":"create_track","trackType":"overlay","name":"Read only","resultAlias":"readonly"},
        group("@readonly", "ancestor"),
        {"operation":"add_group","trackId":track,"startMs":200,"durationMs":500,"parent":{"scope":"root","id":"@ancestor"},"resultAlias":"group"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":7,"positionY":9,"scale":1,"opacity":0.5},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@group"}},
        {"operation":"add_group","trackId":"@children","startMs":0,"durationMs":1000,"parent":{"scope":"root","id":"@group"},"resultAlias":"nested"},
        {"operation":"add_group","trackId":"@readonly","startMs":0,"durationMs":1000,"parent":{"scope":"root","id":"@nested"},"resultAlias":"deep"},
        {"operation":"item_set_z_index","itemId":"@child","zIndex":-5},
        {"operation":"set_item_visibility","itemId":"@nested","hidden":true}
    ])).unwrap()
}

fn persisted_bytes(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}

#[test]
fn ungroup_promotes_immediate_children_preserves_local_state_and_exact_history() {
    for root_group in [false, true] {
        let (_root, core, id, track) = setup();
        let created = core.edit_batch(&id, 0, ungroup_fixture(&track)).unwrap();
        let a = &created.aliases;
        if root_group {
            core.edit(
                &id,
                1,
                op(json!({"operation":"item_set_parent","itemId":a["group"],"parent":null})),
            )
            .unwrap();
        }
        let before = core.get_project(&id).unwrap();
        let mut expected = serde_json::to_value(&before).unwrap();
        let replacement = before
            .find_item(&a["group"])
            .unwrap()
            .visual_properties()
            .parent
            .clone();
        // Independent state oracle: promote direct references, remove only the node,
        // and rebuild ordinals without using mutation helpers.
        for track in expected["tracks"].as_array_mut().unwrap() {
            let items = track["items"].as_array_mut().unwrap();
            items.retain(|item| item["id"] != a["group"]);
            for (index, item) in items.iter_mut().enumerate() {
                if item["parent"]["id"] == a["group"] {
                    if let Some(parent) = &replacement {
                        item["parent"] = json!(parent);
                    } else {
                        item.as_object_mut().unwrap().remove("parent");
                    }
                }
                item["stackOrder"] = json!(index);
            }
        }
        let result = core
            .edit(
                &id,
                before.revision,
                op(json!({"operation":"group_ungroup","groupId":a["group"]})),
            )
            .unwrap();
        assert_eq!(
            result.changed_ids,
            vec![a["group"].clone(), a["child"].clone(), a["nested"].clone()]
        );
        let after = core.get_project(&id).unwrap();
        let after_json = serde_json::to_value(&after).unwrap();
        expected["revision"] = after_json["revision"].clone();
        expected["updatedAtMs"] = after_json["updatedAtMs"].clone();
        assert_eq!(after_json, expected);
        let bytes = persisted_bytes(&core, &id);
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            after_json
        );
        assert_eq!(persisted_bytes(&core, &id), bytes);
        core.undo(&id, result.revision).unwrap();
        assert_eq!(
            json!(core.get_project(&id).unwrap().tracks),
            json!(before.tracks)
        );
        core.redo(&id, result.revision + 1).unwrap();
        assert_eq!(
            json!(core.get_project(&id).unwrap().tracks),
            after_json["tracks"]
        );
    }
}

#[test]
fn ungroup_checks_all_affected_tracks_but_allows_read_only_locks() {
    for locked in ["group", "children", "readonly"] {
        let (_root, core, id, track) = setup();
        let created = core.edit_batch(&id, 0, ungroup_fixture(&track)).unwrap();
        let a = &created.aliases;
        let locked_track = if locked == "group" {
            &track
        } else {
            &a[locked]
        };
        core.edit(
            &id,
            1,
            op(json!({"operation":"update_track","trackId":locked_track,"locked":true})),
        )
        .unwrap();
        let before = persisted_bytes(&core, &id);
        let edit = op(json!({"operation":"group_ungroup","groupId":a["group"]}));
        let result = core.edit(&id, 2, edit);
        if locked == "readonly" {
            assert_eq!(result.unwrap().revision, 3);
            assert_eq!(
                core.get_project(&id)
                    .unwrap()
                    .find_item(&a["deep"])
                    .unwrap()
                    .visual_properties()
                    .parent
                    .as_ref()
                    .unwrap()
                    .id,
                a["nested"]
            );
        } else {
            assert_eq!(result.unwrap_err().code, ErrorCode::TrackLocked);
            assert_eq!(persisted_bytes(&core, &id), before);
        }
    }
}

#[test]
fn ungroup_canonical_target_failures_and_stale_revision_leave_files_untouched() {
    let (_root, core, id, track) = setup();
    let created = core.edit_batch(&id, 0, ungroup_fixture(&track)).unwrap();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-parent-v1.json")).unwrap();
    let before = persisted_bytes(&core, &id);
    for fixture in catalog["ungroupFailures"].as_array().unwrap() {
        let reference = fixture["groupId"].as_str().unwrap();
        let target = created
            .aliases
            .get(reference)
            .map(String::as_str)
            .unwrap_or(reference);
        let error = core
            .edit(
                &id,
                1,
                op(json!({"operation":"group_ungroup","groupId":target})),
            )
            .unwrap_err();
        assert_eq!(json!(error.code), fixture["error"]);
        assert_eq!(persisted_bytes(&core, &id), before);
    }
    assert_eq!(
        core.edit(
            &id,
            0,
            op(json!({"operation":"group_ungroup","groupId":"missing"}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(persisted_bytes(&core, &id), before);
}

#[test]
fn ungroup_alias_creation_lifetime_and_late_failure_are_atomic() {
    let (_root, core, id, track) = setup();
    let initial = persisted_bytes(&core, &id);
    let mut operations = ungroup_fixture(&track);
    operations.push(op(json!({"operation":"group_ungroup","groupId":"@group"})).into());
    let success = core.edit_batch(&id, 0, operations.clone()).unwrap();
    assert!(success.aliases.contains_key("group"));
    assert!(
        core.get_project(&id)
            .unwrap()
            .find_item(&success.aliases["group"])
            .is_none()
    );
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    core.redo(&id, 2).unwrap();
    let after = persisted_bytes(&core, &id);
    assert_ne!(initial, after);
    for (extra, code) in [
        (
            json!({"operation":"group_ungroup","groupId":"@group"}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"group_ungroup","groupId":"@missing"}),
            ErrorCode::ValidationFailed,
        ),
        (
            json!({"operation":"item_set_z_index","itemId":"absent","zIndex":0}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        let mut failed = operations.clone();
        failed.push(op(extra).into());
        assert_eq!(core.edit_batch(&id, 3, failed).unwrap_err().code, code);
        assert_eq!(persisted_bytes(&core, &id), after);
    }
    let forward: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"group_ungroup","groupId":"@later"}, group(&track,"later")
    ]))
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, 3, forward).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    let forbidden = BatchEditOperation {
        edit: op(json!({"operation":"group_ungroup","groupId":success.aliases["nested"]})),
        result_alias: Some("removed".into()),
    };
    assert_eq!(
        core.edit_batch(&id, 3, vec![forbidden]).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(persisted_bytes(&core, &id), after);
}

#[test]
fn ungroup_empty_group_normalizes_other_ordinals_and_is_undoable() {
    let (_root, core, id, track) = setup();
    let created = core
        .edit_batch(
            &id,
            0,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!([
                group(&track, "empty"),
                group(&track, "other")
            ]))
            .unwrap(),
        )
        .unwrap();
    let before = core.get_project(&id).unwrap();
    let result = core
        .edit(
            &id,
            1,
            op(json!({"operation":"group_ungroup","groupId":created.aliases["empty"]})),
        )
        .unwrap();
    assert_eq!(
        result.changed_ids,
        vec![
            created.aliases["empty"].clone(),
            created.aliases["other"].clone()
        ]
    );
    assert_eq!(
        core.get_project(&id).unwrap().tracks[1].items[0]
            .visual_properties()
            .stack_order,
        0
    );
    core.undo(&id, 2).unwrap();
    assert_eq!(
        json!(core.get_project(&id).unwrap().tracks),
        json!(before.tracks)
    );
}

#[test]
fn ungroup_preserves_every_visual_kind_media_integrity_and_caption_provenance() {
    use opencut_editor_core::{
        CaptionStyle, CommitTranscriptionRequest, MediaProbeFacts, MediaType, TranscriptionSegment,
    };
    let (root, core, id, track) = setup();
    let path = root.path().join("media/video.mp4");
    std::fs::write(&path, b"trusted probe fixture").unwrap();
    let imported = core
        .import_asset(
            &id,
            0,
            &path,
            MediaType::Video,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap();
    let asset = &imported.changed_ids[0];
    core.commit_transcription(CommitTranscriptionRequest {
        project_id: id.clone(),
        expected_revision: 1,
        asset_id: asset.clone(),
        caption_track_id: None,
        provider_id: "test-provider".into(),
        model_id: "test-model".into(),
        model_version: Some("1".into()),
        language: "en".into(),
        generated_at_ms: 1,
        segments: vec![TranscriptionSegment {
            text: "Caption source".into(),
            start_ms: 0,
            end_ms: 500,
            confidence: Some(0.9),
            words: vec![],
        }],
        style: CaptionStyle::default(),
    })
    .unwrap();
    let caption = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, opencut_editor_core::TimelineItem::Caption(_)))
        .unwrap()
        .id()
        .to_owned();
    let operations:Vec<BatchEditOperation>=serde_json::from_value(json!([
        group(&track,"group"),group(&track,"nested"),
        {"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0,"resultAlias":"media"},
        {"operation":"add_text","trackId":track,"text":"Preserve","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":2,"positionY":3,"scale":1,"opacity":1},"resultAlias":"text"},
        {"operation":"add_solid_color","trackId":track,"color":"#abcdef","startMs":0,"durationMs":1000,"transform":{"positionX":2,"positionY":3,"scale":1,"opacity":0.5},"resultAlias":"solid"},
        {"operation":"add_rectangle","trackId":track,"width":20,"height":10,"color":"#123456","startMs":0,"durationMs":1000,"transform":{"positionX":2,"positionY":3,"scale":1,"opacity":1},"resultAlias":"rectangle"},
        {"operation":"item_set_parent","itemId":"@nested","parent":{"scope":"root","id":"@group"}},
        {"operation":"item_set_parent","itemId":"@media","parent":{"scope":"root","id":"@group"}},
        {"operation":"item_set_parent","itemId":"@text","parent":{"scope":"root","id":"@group"}},
        {"operation":"item_set_parent","itemId":"@solid","parent":{"scope":"root","id":"@group"}},
        {"operation":"item_set_parent","itemId":"@rectangle","parent":{"scope":"root","id":"@group"}},
        {"operation":"item_set_parent","itemId":caption,"parent":{"scope":"root","id":"@group"}}
    ])).unwrap();
    let created = core.edit_batch(&id, 2, operations).unwrap();
    let before = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let media = dir.join(&before.assets[0].project_relative_path);
    let media_bytes = std::fs::read(&media).unwrap();
    let removed = core
        .edit(
            &id,
            3,
            op(json!({"operation":"group_ungroup","groupId":created.aliases["group"]})),
        )
        .unwrap();
    assert_eq!(removed.changed_ids.len(), 7);
    let after = core.get_project(&id).unwrap();
    assert_eq!(json!(before.assets), json!(after.assets));
    for item in after.tracks.iter().flat_map(|t| &t.items) {
        let mut expected = json!(before.find_item(item.id()).unwrap());
        expected.as_object_mut().unwrap().remove("parent");
        expected["stackOrder"] = json!(item.visual_properties().stack_order);
        assert_eq!(json!(item), expected);
    }
    core.undo(&id, 4).unwrap();
    assert_eq!(
        json!(core.get_project(&id).unwrap().tracks),
        json!(before.tracks)
    );
    core.redo(&id, 5).unwrap();
    assert_eq!(
        json!(core.get_project(&id).unwrap().tracks),
        json!(after.tracks)
    );
    assert_eq!(std::fs::read(media).unwrap(), media_bytes);
}

#[test]
fn native_ungroup_matches_explicit_reparent_delete_in_frame_range_and_export() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer, Transform2D};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (_root, core, id, track) = setup();
    let mut values = ungroup_fixture(&track);
    let mut transform = Transform2D::default();
    transform.position.x = 30.0;
    transform.rotation_deg = 30.0;
    transform.opacity = 0.4;
    values.push(
        op(json!({"operation":"update_item","itemId":"@group","transform2d":transform})).into(),
    );
    values.push(
        op(json!({"operation":"set_item_visibility","itemId":"@group","hidden":true})).into(),
    );
    let created = core.edit_batch(&id, 0, values).unwrap();
    let a = &created.aliases;
    let draft = core
        .create_draft(
            &id,
            1,
            vec![op(
                json!({"operation":"group_ungroup","groupId":a["group"]}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit(
        &id,
        1,
        op(json!({"operation":"group_ungroup","groupId":a["group"]})),
    )
    .unwrap();
    let mut actual = core.get_project(&id).unwrap();
    assert_eq!(json!(candidate.tracks), json!(actual.tracks));
    core.undo(&id, 2).unwrap();
    let equivalent:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"item_set_parent","itemId":a["child"],"parent":{"scope":"root","id":a["ancestor"]}},
        {"operation":"item_set_parent","itemId":a["nested"],"parent":{"scope":"root","id":a["ancestor"]}},
        {"operation":"delete_item","itemId":a["group"]}
    ])).unwrap();
    core.edit_batch(&id, 3, equivalent).unwrap();
    let mut oracle = core.get_project(&id).unwrap();
    assert_eq!(json!(actual.tracks), json!(oracle.tracks));
    actual.settings.width = 64;
    actual.settings.height = 64;
    oracle.settings = actual.settings.clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let decode = |path: &std::path::Path, time: &str| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", time, "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout.len(), 64 * 64 * 3);
        output.stdout
    };
    let mut references = Vec::new();
    for project in [&actual, &oracle] {
        let mut frames = Vec::new();
        for time in [0, 500, 900] {
            let preview = renderer.render_preview(project, &dir, time).unwrap();
            frames.push(decode(&dir.join(preview.relative_path), "0"));
        }
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
        let export = dir.join(format!("ungroup-{}.mp4", project.revision));
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
        for (index, time) in ["0", "0.5", "0.9"].iter().enumerate() {
            for output in [&dir.join(&range.relative_path), &export] {
                let decoded = decode(output, time);
                let mean_error = decoded
                    .iter()
                    .zip(&frames[index])
                    .map(|(a, b)| f64::from(a.abs_diff(*b)))
                    .sum::<f64>()
                    / decoded.len() as f64;
                assert!(mean_error < 3.0, "mean RGB error {mean_error}");
            }
        }
        references.push(frames);
    }
    assert_eq!(references[0], references[1]);
    assert!(
        references[0][0].iter().any(|value| *value > 80),
        "ungroup must reveal the previously hidden visual"
    );
}

#[test]
fn ungroup_batch_enforces_exact_operation_count_limits() {
    let (_root, core, id, track) = setup();
    let mut values = Vec::new();
    for index in 0..50 {
        let alias = format!("g{index}");
        values.push(group(&track, &alias));
        values.push(json!({"operation":"group_ungroup","groupId":format!("@{alias}")}));
    }
    let before = persisted_bytes(&core, &id);
    let mut overflow = values.clone();
    overflow.push(group(&track, "extra"));
    for invalid in [json!([]), json!(overflow)] {
        let operations: Vec<BatchEditOperation> = serde_json::from_value(invalid).unwrap();
        assert_eq!(
            core.edit_batch(&id, 0, operations).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(persisted_bytes(&core, &id), before);
    }
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!(values)).unwrap();
    let result = core.edit_batch(&id, 0, operations).unwrap();
    assert_eq!(result.revision, 1);
    assert_eq!(result.aliases.len(), 50);
    assert_eq!(result.changed_ids.len(), 50);
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    let created = core
        .edit(
            &id,
            1,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap();
    core.edit_batch(
        &id,
        2,
        vec![BatchEditOperation::from(op(
            json!({"operation":"group_ungroup","groupId":created.changed_ids[0]}),
        ))],
    )
    .unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
}

#[test]
fn batch_alias_presence_preserves_other_operations_and_duplicate_rejection() {
    for operation in [
        json!({"operation":"add_group","trackId":"overlay","startMs":0,"durationMs":1000}),
        json!({"operation":"item_set_parent","itemId":"item","parent":null}),
    ] {
        for alias in [None, Some(Value::Null), Some(json!("created"))] {
            let mut input = operation.clone();
            if let Some(value) = &alias {
                input["resultAlias"] = value.clone();
            }
            let batch: BatchEditOperation = serde_json::from_value(input).unwrap();
            let expected = alias.as_ref().and_then(Value::as_str);
            assert_eq!(batch.result_alias.as_deref(), expected);
            let serialized = json!(batch);
            assert_eq!(
                serialized.get("resultAlias").and_then(Value::as_str),
                expected
            );
            let round_trip: BatchEditOperation =
                serde_json::from_value(serialized.clone()).unwrap();
            assert_eq!(json!(round_trip), serialized);
        }
        let mut wrong = operation.clone();
        wrong["resultAlias"] = json!(42);
        assert!(serde_json::from_value::<BatchEditOperation>(wrong).is_err());
        let prefix = serde_json::to_string(&operation).unwrap();
        let prefix = prefix.strip_suffix('}').unwrap();
        for fields in [
            r#", "resultAlias":null,"resultAlias":null}"#,
            r#", "resultAlias":null,"resultAlias":"alias"}"#,
            r#", "resultAlias":"alias","resultAlias":null}"#,
        ] {
            assert!(
                serde_json::from_str::<BatchEditOperation>(&format!("{prefix}{fields}")).is_err()
            );
        }
    }
}
