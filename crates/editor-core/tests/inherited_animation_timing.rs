use opencut_editor_core::{PROJECT_SCHEMA_VERSION, Project};
use serde_json::{Value, json};

fn project_with_timing(schema_version: u32) -> Value {
    json!({
        "schemaVersion": schema_version,
        "id": "timing-project",
        "revision": 0,
        "name": "Timing",
        "createdAtMs": 1,
        "updatedAtMs": 1,
        "settings": {"width": 100, "height": 100, "fps": 30},
        "assets": [],
        "fonts": {},
        "markers": [],
        "tracks": [{
            "id": "overlay", "name": "Overlay", "trackType": "overlay",
            "items": [
                {"type": "group", "id": "parent", "zIndex": 0, "stackOrder": 0, "startMs": 0, "durationMs": 1000, "staggerMs": 125},
                {"type": "repeater", "id": "copies", "zIndex": 0, "stackOrder": 1, "startMs": 0, "durationMs": 1000,
                    "repeater": {
                        "source": {"scope": "root", "id": "parent"},
                        "copies": 2,
                        "transformOffset": {
                            "position": {"x": 0, "y": 0, "unit": "pixels"},
                            "scaleX": 1, "scaleY": 1, "rotationDeg": 0,
                            "skewXDeg": 0, "skewYDeg": 0
                        },
                        "opacityOffset": 0,
                        "timeOffsetMs": -50
                    }
                }
            ]
        }],
        "components": []
    })
}

#[test]
fn canonical_schema_and_signed_timing_round_trip() {
    let contract: Value = serde_json::from_str(include_str!(
        "../../../contracts/inherited-animation-timing-v1.json"
    ))
    .unwrap();
    assert_eq!(contract["projectSchemaVersion"], PROJECT_SCHEMA_VERSION);
    assert_eq!(contract["fields"]["group.staggerMs"]["maximum"], 60_000);
    assert_eq!(
        contract["fields"]["repeater.timeOffsetMs"]["minimum"],
        -60_000
    );

    let project: Project =
        serde_json::from_value(project_with_timing(PROJECT_SCHEMA_VERSION)).unwrap();
    let value = serde_json::to_value(&project).unwrap();
    assert_eq!(value["tracks"][0]["items"][0]["staggerMs"], 125);
    assert_eq!(
        value["tracks"][0]["items"][1]["repeater"]["timeOffsetMs"],
        -50
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<Project>(value.clone()).unwrap()).unwrap(),
        value
    );
}

#[test]
fn older_source_rejects_new_fields_even_when_zero() {
    let mut value = project_with_timing(25);
    assert!(serde_json::from_value::<Project>(value.clone()).is_err());
    value["tracks"][0]["items"][0]["staggerMs"] = json!(0);
    value["tracks"][0]["items"][1]["repeater"]["timeOffsetMs"] = json!(0);
    assert!(serde_json::from_value::<Project>(value).is_err());
}

#[test]
fn omitted_fields_preserve_zero_defaults() {
    let mut value = project_with_timing(PROJECT_SCHEMA_VERSION);
    value["tracks"][0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("staggerMs");
    value["tracks"][0]["items"][1]["repeater"]
        .as_object_mut()
        .unwrap()
        .remove("timeOffsetMs");
    let project: Project = serde_json::from_value(value).unwrap();
    let serialized = serde_json::to_value(project).unwrap();
    assert!(
        serialized["tracks"][0]["items"][0]
            .get("staggerMs")
            .is_none()
    );
    assert!(
        serialized["tracks"][0]["items"][1]["repeater"]
            .get("timeOffsetMs")
            .is_none()
    );
}

#[test]
fn timing_edits_are_alias_aware_atomic_and_reversible() {
    use opencut_editor_core::{
        BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    };
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Timing", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let channels = json!([{"property":"transform.position_x","loop":{"mode":"ping_pong","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":50.0},"curve":"hold"}]}]);
    let edits=serde_json::from_value::<Vec<BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"staggerMs":125,"resultAlias":"parent"},
        {"operation":"update_item","itemId":"@parent","transform2d":null},
        {"operation":"set_animation_channels","itemId":"@parent","animationChannels":channels},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@parent"}},
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{
            "source":{"scope":"root","id":"@parent"},"copies":1,"timeOffsetMs":-100,"opacityOffset":0,
            "transformOffset":{"position":{"x":20,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}
        },"resultAlias":"copies"}
    ])).unwrap();
    let changed = core.edit_batch(&id, 0, edits).unwrap();
    let parent = &changed.aliases["parent"];
    let state = core.get_project(&id).unwrap();
    let value = serde_json::to_value(state.find_item(parent).unwrap()).unwrap();
    assert_eq!(value["staggerMs"], 125);
    assert_eq!(value["animationChannels"], channels);
    assert_eq!(
        serde_json::to_value(state.find_item(&changed.aliases["copies"]).unwrap()).unwrap()["repeater"]
            ["timeOffsetMs"],
        -100
    );
    assert!(
        core.edit(
            &id,
            1,
            edit_operation(
                json!({"operation":"update_item","itemId":changed.aliases["child"],"staggerMs":5})
            )
        )
        .is_err()
    );

    let edit = |v| serde_json::from_value::<EditOperation>(v).unwrap();
    let failure = serde_json::from_value::<Vec<BatchEditOperation>>(json!([
        {"operation":"update_item","itemId":parent,"staggerMs":60000},
        {"operation":"update_item","itemId":"missing","staggerMs":5}
    ]))
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, failure).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&state).unwrap()
    );
    assert_eq!(
        core.edit(
            &id,
            0,
            edit(json!({"operation":"update_item","itemId":parent,"staggerMs":5}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert!(
        core.edit(
            &id,
            1,
            edit(json!({"operation":"update_item","itemId":parent,"staggerMs":60001}))
        )
        .is_err()
    );
    let draft = core
        .create_draft(
            &id,
            1,
            vec![edit(
                json!({"operation":"update_item","itemId":parent,"staggerMs":250}),
            )],
            None,
        )
        .unwrap();
    let draft_state = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(
        serde_json::to_value(draft_state.find_item(parent).unwrap()).unwrap()["staggerMs"],
        250
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&state).unwrap()
    );
    core.undo(&id, 1).unwrap();
    assert!(core.get_project(&id).unwrap().find_item(parent).is_none());
    core.redo(&id, 2).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(parent).unwrap()).unwrap(),
        value
    );
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()
    );
}

#[test]
fn component_timing_updates_duplicate_and_locked_tracks_are_atomic() {
    use opencut_editor_core::{
        BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    };
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Timing", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let operations=serde_json::from_value::<Vec<BatchEditOperation>>(json!([
        {"operation":"component_create","name":"Definition","width":100,"height":100,"durationMs":1000,"tracks":[],"resultAlias":"definition"},
        {"operation":"add_component_instance","trackId":track,"componentId":"@definition","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"staggerMs":60000,"resultAlias":"instance"},
        {"operation":"component_instance_update","itemId":"@instance","componentId":"@definition","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"staggerMs":125}
    ])).unwrap();
    let changed = core.edit_batch(&id, 0, operations).unwrap();
    let item = &changed.aliases["instance"];
    let edit = |v| serde_json::from_value::<EditOperation>(v).unwrap();
    let duplicate = core
        .edit(
            &id,
            1,
            edit(json!({"operation":"component_instance_duplicate","itemId":item,"offsetMs":1000})),
        )
        .unwrap();
    let state = core.get_project(&id).unwrap();
    for target in [item, &duplicate.changed_ids[0]] {
        assert_eq!(
            serde_json::to_value(state.find_item(target).unwrap()).unwrap()["staggerMs"],
            125
        );
    }
    core.edit(
        &id,
        2,
        edit(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(
        core.edit(
            &id,
            3,
            edit(json!({"operation":"update_item","itemId":item,"staggerMs":0}))
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
}

fn edit_operation(value: Value) -> opencut_editor_core::EditOperation {
    serde_json::from_value(value).unwrap()
}

fn inventory(dir: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        base: &std::path::Path,
        dir: &std::path::Path,
        result: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, result);
            } else if path.file_name().unwrap() != "project.lock" {
                result.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(dir, dir, &mut result);
    result
}

#[test]
fn inherited_bounds_reject_every_publication_boundary_without_resource_changes() {
    use opencut_editor_core::{
        BatchEditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    };
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let fonts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/fonts");
    let core = core.with_font_config(opencut_editor_core::FontConfig {
        roots: vec![fonts.clone()],
        default_path: Some(fonts.join("DejaVuSans.ttf")),
    });
    let id = core
        .create_project("Bounds", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let setup = json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"update_item","itemId":"@parent","transform2d":null},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":500,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@parent"}}
    ]);
    let changed = core
        .edit_batch(
            &id,
            0,
            serde_json::from_value::<Vec<BatchEditOperation>>(setup.clone()).unwrap(),
        )
        .unwrap();
    let parent = &changed.aliases["parent"];
    let unsafe_edit = json!({"operation":"set_animation_channels","itemId":parent,"animationChannels":[{"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":100},"curve":"hold"}]}]});
    let staged_text = json!({"operation":"add_text","trackId":track,"text":"Staged font","fontFamily":"DejaVu Sans","fontSize":20,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}});
    let dir = core.paths().project_dir(&id).unwrap();
    let before = inventory(&dir);
    let e = core
        .edit(&id, 1, edit_operation(unsafe_edit.clone()))
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArgument);
    assert!(!e.retryable);
    assert!(e.message.contains("raster bounds"));
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.edit_batch(
            &id,
            1,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!([unsafe_edit.clone()]))
                .unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.create_draft(
            &id,
            1,
            vec![
                edit_operation(staged_text.clone()),
                edit_operation(unsafe_edit.clone())
            ],
            None
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    let draft = core
        .create_draft(
            &id,
            1,
            vec![edit_operation(
                json!({"operation":"set_item_visibility","itemId":parent,"hidden":false}),
            )],
            None,
        )
        .unwrap();
    let before = inventory(&dir);
    assert_eq!(
        core.update_draft(
            &id,
            &draft.id,
            1,
            vec![
                edit_operation(staged_text.clone()),
                edit_operation(unsafe_edit.clone())
            ],
            None
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    // A persisted candidate must be checked again at rebase and commit, even if
    // an old writer accepted it. This bypasses create/update solely as a fixture.
    let path = dir.join("drafts").join(format!("{}.json", draft.id));
    let mut value = serde_json::to_value(&draft).unwrap();
    value["operations"] = json!([unsafe_edit.clone()]);
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let before = inventory(&dir);
    assert_eq!(
        core.rebase_draft(&id, &draft.id, 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.commit_draft(&id, &draft.id, 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    // Unsafe intermediate channels are allowed when the final batch clears them.
    core.edit_batch(&id, 1, serde_json::from_value::<Vec<BatchEditOperation>>(json!([unsafe_edit, {"operation":"set_animation_channels","itemId":parent,"animationChannels":[]}])).unwrap()).unwrap();
    assert_eq!(core.get_project(&id).unwrap().revision, 2);
    // Alias-aware construction must fail atomically before aliases are returned.
    let id2 = core
        .create_project("Alias bounds", ProjectSettings::default())
        .unwrap()
        .project_id;
    let t2 = core.get_project(&id2).unwrap().tracks[1].id.clone();
    let mut ops = setup.as_array().unwrap().clone();
    for op in &mut ops {
        if op.get("trackId").is_some() {
            op["trackId"] = json!(t2);
        }
    }
    let mut channel = value["operations"][0].clone();
    channel["itemId"] = json!("@parent");
    let mut text = staged_text.clone();
    text["trackId"] = json!(t2);
    ops.push(text);
    ops.push(channel);
    let dir2 = core.paths().project_dir(&id2).unwrap();
    let before = inventory(&dir2);
    assert_eq!(
        core.edit_batch(
            &id2,
            0,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!(ops)).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir2), before);
}

#[test]
fn retained_unsafe_current_and_history_fail_before_asset_migration() {
    use opencut_editor_core::{EditorCore, ErrorCode, PathPolicy, ProjectSettings};
    for retained in ["current", "undo", "redo"] {
        for kind in ["overflow", "scale"] {
            let root = tempfile::tempdir().unwrap();
            let core = EditorCore::new(
                PathPolicy::new(
                    root.path().join("projects"),
                    [root.path()],
                    root.path().join("exports"),
                )
                .unwrap(),
            );
            let id = core
                .create_project("Overflow", ProjectSettings::default())
                .unwrap()
                .project_id;
            let mut p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            p["tracks"][1]["items"] = json!([
                {"type":"group","id":"parent","startMs":0,"durationMs":u64::MAX,"staggerMs":1,"hidden":true,"zIndex":0,"stackOrder":0},
                {"type":"rectangle","id":"first","startMs":0,"durationMs":1,"width":10,"height":10,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"parent"},"zIndex":0,"stackOrder":1},
                {"type":"rectangle","id":"last","startMs":u64::MAX-1,"durationMs":1,"width":10,"height":10,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"parent"},"zIndex":0,"stackOrder":2}
            ]);
            if kind == "scale" {
                p["tracks"][1]["items"] = json!([
                    {"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0,"animationChannels":[{"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":100},"curve":"hold"}]}]},
                    {"type":"rectangle","id":"child","startMs":0,"durationMs":1000,"width":500,"height":10,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"parent"},"stackOrder":1,"zIndex":0}
                ]);
            }
            p["assets"] = json!([{"id":"legacy","mediaType":"image","fileName":"legacy.ppm","projectRelativePath":"assets/legacy.ppm","durationMs":null,"hasAudio":false}]);
            let dir = core.paths().project_dir(&id).unwrap();
            let path = dir.join(if retained != "current" {
                "history.json"
            } else {
                "project.json"
            });
            let value = if retained != "current" {
                if retained == "undo" {
                    json!({"undo":[p],"redo":[]})
                } else {
                    json!({"undo":[],"redo":[p]})
                }
            } else {
                p
            };
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            std::fs::write(dir.join("assets/legacy.ppm"), b"P6\n1 1\n255\n\xff\x00\x00").unwrap();
            let before = inventory(&dir);
            let e = core.get_project(&id).unwrap_err();
            assert_eq!(e.code, ErrorCode::InvalidArgument, "{e:?}");
            assert!(
                e.message.contains(if kind == "overflow" {
                    "stagger clock overflow"
                } else {
                    "raster bounds"
                }),
                "{e:?}"
            );
            assert_eq!(inventory(&dir), before);
        }
    }
}

#[test]
fn hidden_stagger_overflow_rejects_edits_batches_and_all_draft_boundaries() {
    use opencut_editor_core::{
        BatchEditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    };
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Overflow edit", ProjectSettings::default())
        .unwrap()
        .project_id;
    let mut p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    p["tracks"][1]["items"] = json!([
        {"type":"group","id":"parent","startMs":0,"durationMs":u64::MAX,"hidden":true,"stackOrder":0,"zIndex":0},
        {"type":"rectangle","id":"first","startMs":0,"durationMs":1,"width":10,"height":10,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"parent"},"stackOrder":1,"zIndex":0},
        {"type":"rectangle","id":"last","startMs":u64::MAX-1,"durationMs":1,"width":10,"height":10,"color":"#ff0000","keyframes":[],"parent":{"scope":"root","id":"parent"},"stackOrder":2,"zIndex":0}
    ]);
    let dir = core.paths().project_dir(&id).unwrap();
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&p).unwrap()).unwrap();
    let op = json!({"operation":"update_item","itemId":"parent","staggerMs":1});
    let before = inventory(&dir);
    assert_eq!(
        core.edit(&id, 0, edit_operation(op.clone()))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.edit_batch(
            &id,
            0,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!([op.clone()])).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.create_draft(&id, 0, vec![edit_operation(op.clone())], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    let draft = core
        .create_draft(
            &id,
            0,
            vec![edit_operation(
                json!({"operation":"set_item_visibility","itemId":"parent","hidden":true}),
            )],
            None,
        )
        .unwrap();
    let before = inventory(&dir);
    assert_eq!(
        core.update_draft(&id, &draft.id, 0, vec![edit_operation(op.clone())], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    let mut d = serde_json::to_value(&draft).unwrap();
    d["operations"] = json!([op]);
    std::fs::write(
        dir.join("drafts").join(format!("{}.json", draft.id)),
        serde_json::to_vec(&d).unwrap(),
    )
    .unwrap();
    let before = inventory(&dir);
    assert_eq!(
        core.rebase_draft(&id, &draft.id, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(
        core.commit_draft(&id, &draft.id, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&dir), before);
    assert_eq!(core.get_project(&id).unwrap().revision, 0);
}
