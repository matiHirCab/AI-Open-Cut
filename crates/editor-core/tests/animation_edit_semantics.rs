use opencut_editor_core::{
    AnimationClock, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
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
            [media],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Animation edits", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn keys() -> Value {
    json!([{"timeMs":100,"value":{"type":"scalar","value":0.0},"curve":{"type":"spring","mass":1.0,"stiffness":120.0,"damping":7.0,"initialVelocity":0.0}},{"timeMs":500,"value":{"type":"scalar","value":100.0},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":0.0},"curve":"hold"}])
}
fn add(core: &EditorCore, id: &str, track: &str) -> String {
    core.edit(id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":200,"durationMs":2400,"width":10,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone()
}
#[test]
fn clock_structural_contract_matches_canonical_cases() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../contracts/animation-channels-v1.json"
    ))
    .unwrap();
    for case in fixture["clockCases"].as_array().unwrap() {
        assert_eq!(
            serde_json::from_value::<AnimationClock>(case["clock"].clone()).is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["name"]
        );
    }
}
#[test]
fn split_trim_duplicate_preserve_exact_records_history_reopen_and_independent_replacement() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys(),"loop":{"mode":"ping_pong","iterations":1}}]}))).unwrap();
    let original = core.get_project(&id).unwrap();
    let split = core
        .edit(
            &id,
            2,
            op(json!({"operation":"split_item","itemId":item,"splitMs":550})),
        )
        .unwrap();
    let right = split.changed_ids[1].clone();
    let p = core.get_project(&id).unwrap();
    for (target, offset) in [(&item, 0), (&right, 350)] {
        let c = &p
            .find_item(target)
            .unwrap()
            .visual_properties()
            .animation_channels[0];
        assert_eq!(serde_json::to_value(&c.keyframes).unwrap(), keys());
        assert_eq!(
            c.clock,
            Some(AnimationClock {
                offset_ms: offset,
                source_duration_ms: 2400
            })
        );
        assert_eq!(
            c.r#loop,
            original
                .find_item(&item)
                .unwrap()
                .visual_properties()
                .animation_channels[0]
                .r#loop
        );
    }
    core.edit(
        &id,
        3,
        op(json!({"operation":"trim_item","itemId":right,"startMs":700,"durationMs":1500})),
    )
    .unwrap();
    core.edit(
        &id,
        4,
        op(json!({"operation":"trim_item","itemId":right,"startMs":650,"durationMs":1550})),
    )
    .unwrap();
    let copy = core
        .edit(
            &id,
            5,
            op(json!({"operation":"duplicate_items","itemIds":[right],"offsetMs":3000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let p = core.get_project(&id).unwrap();
    let retained = &p
        .find_item(&right)
        .unwrap()
        .visual_properties()
        .animation_channels;
    assert_eq!(
        retained[0].clock,
        Some(AnimationClock {
            offset_ms: 450,
            source_duration_ms: 2400
        })
    );
    assert_eq!(
        retained,
        &p.find_item(&copy)
            .unwrap()
            .visual_properties()
            .animation_channels
    );
    let saved = serde_json::to_value(&p).unwrap();
    core.undo(&id, 6).unwrap();
    core.redo(&id, 7).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    let mut actual = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
    actual["revision"] = saved["revision"].clone();
    actual["updatedAtMs"] = saved["updatedAtMs"].clone();
    assert_eq!(actual, saved);
    reopened.edit(&id,8,op(json!({"operation":"set_animation_channels","itemId":copy,"animationChannels":[{"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":5},"curve":"hold"}]}]}))).unwrap();
    let p = reopened.get_project(&id).unwrap();
    assert!(
        p.find_item(&copy)
            .unwrap()
            .visual_properties()
            .animation_channels[0]
            .clock
            .is_none()
    );
    assert_eq!(
        p.find_item(&right)
            .unwrap()
            .visual_properties()
            .animation_channels,
        *retained
    );
}
#[test]
fn clock_failures_and_edit_errors_are_atomic() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}))).unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for (operation, code) in [
        (
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","clock":{"offsetMs":9007199254740991_i64,"sourceDurationMs":1000},"keyframes":keys()}]}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","clock":{"offsetMs":0,"sourceDurationMs":100},"keyframes":keys()}]}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","clock":{"offsetMs":0,"sourceDurationMs":1000},"keyframes":[]}]}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"split_item","itemId":item,"splitMs":200}),
            ErrorCode::ValidationFailed,
        ),
        (
            json!({"operation":"trim_item","itemId":"missing","startMs":0,"durationMs":10}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(core.edit(&id, 2, op(operation)).unwrap_err().code, code);
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    assert_eq!(
        core.edit(
            &id,
            1,
            op(json!({"operation":"split_item","itemId":item,"splitMs":400}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let batch: Vec<opencut_editor_core::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"trim_item","itemId":item,"startMs":400,"durationMs":1000},
        {"operation":"split_item","itemId":item,"splitMs":400}
    ]))
    .unwrap();
    assert!(core.edit_batch(&id, 2, batch).is_err());
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        before
    );
}

#[test]
fn schema29_migrates_root_components_and_all_retained_history_atomically() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}))).unwrap();
    core.edit(
        &id,
        2,
        op(json!({"operation":"trim_item","itemId":item,"startMs":200,"durationMs":2400})),
    )
    .unwrap();
    core.undo(&id, 3).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let project_file = dir.join("project.json");
    let history_file = dir.join("history.json");
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(&project_file).unwrap()).unwrap();
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(&history_file).unwrap()).unwrap();
    let component = json!({"id":"source","name":"Source","width":64,"height":64,"durationMs":3000,"markers":[],"slots":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"rectangle","id":"local-rect","startMs":200,"durationMs":2400,"width":10,"height":10,"color":"#ff0000","keyframes":[],"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}]}]});
    let component = serde_json::to_value(
        serde_json::from_value::<opencut_editor_core::ComponentDefinition>(component).unwrap(),
    )
    .unwrap();
    fn legacy(snapshot: &mut Value, component: &Value) {
        snapshot["schemaVersion"] = json!(29);
        snapshot["components"] = json!([component]);
        for track in snapshot["tracks"].as_array_mut().unwrap() {
            for item in track["items"].as_array_mut().unwrap() {
                if let Some(channels) = item
                    .get_mut("animationChannels")
                    .and_then(Value::as_array_mut)
                {
                    for channel in channels {
                        channel.as_object_mut().unwrap().remove("clock");
                    }
                }
            }
        }
    }
    legacy(&mut current, &component);
    for name in ["undo", "redo"] {
        for snapshot in history[name].as_array_mut().unwrap() {
            legacy(snapshot, &component);
        }
    }
    std::fs::write(&project_file, serde_json::to_vec(&current).unwrap()).unwrap();
    std::fs::write(&history_file, serde_json::to_vec(&history).unwrap()).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    let migrated = reopened.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 31);
    assert_eq!(
        serde_json::to_value(
            &migrated
                .find_item(&item)
                .unwrap()
                .visual_properties()
                .animation_channels[0]
                .keyframes
        )
        .unwrap(),
        keys()
    );
    assert_eq!(
        migrated.components[0].tracks[0].items[0]
            .visual_properties()
            .animation_channels[0]
            .clock,
        None
    );
    let actual: Value = serde_json::from_slice(&std::fs::read(&history_file).unwrap()).unwrap();
    for name in ["undo", "redo"] {
        assert!(!actual[name].as_array().unwrap().is_empty());
        for snapshot in actual[name].as_array().unwrap() {
            assert_eq!(snapshot["schemaVersion"], 31);
            assert_eq!(snapshot["components"], json!([component]));
        }
    }
    let stable = (
        std::fs::read(&project_file).unwrap(),
        std::fs::read(&history_file).unwrap(),
    );
    reopened.get_project(&id).unwrap();
    assert_eq!(
        stable,
        (
            std::fs::read(&project_file).unwrap(),
            std::fs::read(&history_file).unwrap()
        )
    );
}

#[test]
fn future_and_pre31_clocks_reject_current_or_history_without_disk_mutation() {
    for history_location in [false, true] {
        for case in ["old30_clock", "future", "old_clock", "unsafe_window"] {
            let (_root, core, id, track) = setup();
            let item = add(&core, &id, &track);
            core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}))).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let project_file = dir.join("project.json");
            let history_file = dir.join("history.json");
            let mut current: Value =
                serde_json::from_slice(&std::fs::read(&project_file).unwrap()).unwrap();
            let mut history: Value =
                serde_json::from_slice(&std::fs::read(&history_file).unwrap()).unwrap();
            let target = if history_location {
                &mut history["undo"][1]
            } else {
                &mut current
            };
            match case {
                "old30_clock" => {
                    target["schemaVersion"] = json!(30);
                    target["tracks"][1]["items"][0]["legacyAnimationClock"] =
                        json!({"offsetMs":0,"sourceDurationMs":2400});
                }
                "future" => target["schemaVersion"] = json!(32),
                "old_clock" => {
                    target["schemaVersion"] = json!(29);
                    target["tracks"][1]["items"][0]["legacyAnimationClock"] =
                        json!({"offsetMs":0,"sourceDurationMs":2400});
                }
                "unsafe_window" => {
                    target["tracks"][1]["items"][0]["animationChannels"] = json!([{"property":"transform.position_x","clock":{"offsetMs":9007199254740991_i64,"sourceDurationMs":2400},"keyframes":keys()}]);
                }
                _ => unreachable!(),
            }
            std::fs::write(&project_file, serde_json::to_vec(&current).unwrap()).unwrap();
            std::fs::write(&history_file, serde_json::to_vec(&history).unwrap()).unwrap();
            let before = (
                std::fs::read(&project_file).unwrap(),
                std::fs::read(&history_file).unwrap(),
            );
            assert!(
                EditorCore::new(core.paths().clone())
                    .get_project(&id)
                    .is_err(),
                "{case} history={history_location}"
            );
            assert_eq!(
                before,
                (
                    std::fs::read(&project_file).unwrap(),
                    std::fs::read(&history_file).unwrap()
                )
            );
        }
    }
}

#[test]
fn every_visual_item_retains_typed_and_legacy_source_records_and_group_restrictions() {
    let shapes: Value =
        serde_json::from_str(include_str!("../../../contracts/shape-items-v1.json")).unwrap();
    let grids: Value =
        serde_json::from_str(include_str!("../../../contracts/procedural-grids-v1.json")).unwrap();
    let svg: Value =
        serde_json::from_str(include_str!("../../../contracts/svg-ingestion-v1.json")).unwrap();
    for mut create in [
        json!({"operation":"add_rectangle","width":10,"height":10,"color":"#ff0000","transform":opencut_editor_core::Transform::default()}),
        json!({"operation":"add_solid_color","color":"#ff0000","transform":opencut_editor_core::Transform::default()}),
        json!({"operation":"add_text","text":"clock","fontSize":24,"color":"#ff0000","style":opencut_editor_core::TextStyle::default(),"transform":opencut_editor_core::Transform::default()}),
        shapes["valid"][0]["value"].clone(),
        json!({"operation":"add_svg","svg":svg["valid"][0]["svg"]}),
        json!({"operation":"add_grid","grid":grids["valid"][0]["grid"]}),
        json!({"operation":"add_group"}),
    ] {
        let (_root, core, id, track) = setup();
        create["trackId"] = json!(track);
        create["startMs"] = json!(200);
        create["durationMs"] = json!(2400);
        let kind = create["operation"].as_str().unwrap().to_owned();
        let item = core.edit(&id, 0, op(create)).unwrap().changed_ids[0].clone();
        core.edit(
            &id,
            1,
            op(json!({"operation":"update_item","itemId":item,"transform2d":null})),
        )
        .unwrap();
        core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}))).unwrap();
        let mut revision = 3;
        if kind != "add_group" {
            core.edit(&id,revision,op(json!({"operation":"set_keyframes","itemId":item,"keyframes":[{"property":"opacity","timeMs":0,"value":{"type":"scalar","value":0.0},"easing":"ease_in"},{"property":"opacity","timeMs":3000,"value":{"type":"scalar","value":1.0},"easing":"linear"}]}))).unwrap();
            revision += 1;
            let result = core
                .edit(
                    &id,
                    revision,
                    op(json!({"operation":"split_item","itemId":item,"splitMs":550})),
                )
                .unwrap();
            revision += 1;
            let right = core
                .get_project(&id)
                .unwrap()
                .find_item(&result.changed_ids[1])
                .unwrap()
                .clone();
            assert_eq!(
                right.visual_properties().animation_channels[0]
                    .clock
                    .unwrap()
                    .offset_ms,
                350,
                "{kind}"
            );
            assert_eq!(
                right.visual_properties().legacy_animation_clock,
                Some(AnimationClock {
                    offset_ms: 350,
                    source_duration_ms: 3000
                }),
                "{kind}"
            );
            assert_eq!(right.keyframes()[1].time_ms, 3000);
        } else {
            assert_eq!(
                core.edit(
                    &id,
                    revision,
                    op(json!({"operation":"split_item","itemId":item,"splitMs":550}))
                )
                .unwrap_err()
                .code,
                ErrorCode::InvalidArgument
            );
        }
        core.edit(
            &id,
            revision,
            op(json!({"operation":"trim_item","itemId":item,"startMs":250,"durationMs":200})),
        )
        .unwrap();
        revision += 1;
        let copy = core
            .edit(
                &id,
                revision,
                op(json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":3000})),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let project = core.get_project(&id).unwrap();
        let original = project.find_item(&item).unwrap();
        let copied = project.find_item(&copy).unwrap();
        assert_eq!(
            original.visual_properties().animation_channels,
            copied.visual_properties().animation_channels,
            "{kind}"
        );
        assert_eq!(
            original.visual_properties().animation_channels[0]
                .clock
                .unwrap()
                .offset_ms,
            50,
            "{kind}"
        );
        assert_eq!(
            original.visual_properties().legacy_animation_clock,
            copied.visual_properties().legacy_animation_clock,
            "{kind}"
        );
    }
}

#[test]
fn marker_alias_batch_retained_clock_history_and_later_failure_preserve_lifecycle() {
    let (_root, core, id, track) = setup();
    let ops:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"marker_create","scope":"root","name":"hit","timeMs":300,"kind":"cue","resultAlias":"cue"},
        {"operation":"add_rectangle","trackId":track,"startMs":200,"durationMs":2400,"width":10,"height":10,"color":"#ff0000","transform":opencut_editor_core::Transform::default(),"resultAlias":"box"},
        {"operation":"set_animation_channels","itemId":"@box","animationChannels":[{"property":"transform.position_x","keyframes":keys()}]},
        {"operation":"set_item_start_time","scope":"root","itemId":"@box","time":{"type":"marker","markerName":"hit","offsetMs":-100}},
        {"operation":"trim_item","itemId":"@box","startMs":200,"durationMs":1200}
    ])).unwrap();
    let result = core.edit_batch(&id, 0, ops.clone()).unwrap();
    let item = result.aliases["box"].clone();
    let project = core.get_project(&id).unwrap();
    let original = project.find_item(&item).unwrap();
    assert!(original.visual_properties().start_time.is_some());
    assert_eq!(
        original.visual_properties().animation_channels[0]
            .clock
            .unwrap()
            .offset_ms,
        0
    );
    let copy = core
        .edit(
            &id,
            1,
            op(json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":3000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&copy).unwrap()).unwrap()["startTime"]
            ["offsetMs"],
        2900
    );
    core.edit(
        &id,
        2,
        op(json!({"operation":"trim_item","itemId":item,"startMs":250,"durationMs":1000})),
    )
    .unwrap();
    assert!(
        core.get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .start_time
            .is_none()
    );
    let split = core
        .edit(
            &id,
            3,
            op(json!({"operation":"split_item","itemId":copy,"splitMs":3500})),
        )
        .unwrap();
    for id2 in split.changed_ids {
        assert!(
            core.get_project(&id)
                .unwrap()
                .find_item(&id2)
                .unwrap()
                .visual_properties()
                .start_time
                .is_none()
        );
    }
    core.undo(&id, 4).unwrap();
    core.redo(&id, 5).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened
            .get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .animation_channels[0]
            .clock
            .unwrap()
            .offset_ms,
        50
    );
    let before = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
    let failed:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track,"startMs":200,"durationMs":2400,"width":10,"height":10,"color":"#ff0000","transform":opencut_editor_core::Transform::default(),"resultAlias":"fresh"},
        {"operation":"set_animation_channels","itemId":"@fresh","animationChannels":[{"property":"transform.position_x","keyframes":keys()}]},
        {"operation":"trim_item","itemId":"@fresh","startMs":250,"durationMs":1000},
        {"operation":"split_item","itemId":"@fresh","splitMs":250}
    ])).unwrap();
    let dir = reopened.paths().project_dir(&id).unwrap();
    let disk = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    let error = reopened.edit_batch(&id, 6, failed).unwrap_err();
    assert_eq!(error.code, ErrorCode::ValidationFailed);
    assert!(error.message.contains("split time must be strictly inside"));
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        before
    );
    assert_eq!(
        disk,
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap()
        )
    );
}

#[test]
fn compound_self_targets_remap_on_split_duplicate_and_effect_targets_stay_local() {
    let (_root, core, id, track) = setup();
    let geometry = json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}});
    let paint = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":20,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":1,"b":0,"a":1}}]});
    let item=core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1800,"geometry":geometry,"fill":paint,"stroke":null}))).unwrap().changed_ids[0].clone();
    core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"tint","type":"color_tint","color":{"r":1,"g":1,"b":1,"a":1}}]}))).unwrap();
    let channel = |property: &str, kind: &str, target: &str, a: Value, b: Value| json!({"property":property,"target":{"kind":kind,"scope":"root","id":target},"keyframes":[{"timeMs":0,"value":a,"curve":{"type":"cubic_bezier","x1":0.2,"y1":0.8,"x2":0.7,"y2":0.9}},{"timeMs":1000,"value":b,"curve":"hold"}]});
    let channels = json!([
        channel(
            "graphic.path_points",
            "graphic_geometry",
            &item,
            json!({"type":"path_points","points":[{"x":0,"y":0},{"x":10,"y":10}]}),
            json!({"type":"path_points","points":[{"x":0,"y":0},{"x":20,"y":15}]})
        ),
        channel(
            "graphic.path_trim",
            "graphic_geometry",
            &item,
            json!({"type":"scalar","value":0.2}),
            json!({"type":"scalar","value":0.9})
        ),
        channel(
            "graphic.gradient_stops",
            "graphic_fill",
            &item,
            json!({"type":"gradient_stops","stops":[{"offset":0,"color":[1,0,0,1]},{"offset":1,"color":[0,1,0,1]}]}),
            json!({"type":"gradient_stops","stops":[{"offset":0,"color":[0,0,1,1]},{"offset":1,"color":[1,0,0,1]}]})
        ),
        channel(
            "effect.tint_color",
            "effect",
            "tint",
            json!({"type":"rgba","r":1,"g":0,"b":0,"a":1}),
            json!({"type":"rgba","r":0,"g":1,"b":0,"a":1})
        )
    ]);
    core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap();
    let source = core
        .get_project(&id)
        .unwrap()
        .find_item(&item)
        .unwrap()
        .visual_properties()
        .animation_channels
        .clone();
    let right = core
        .edit(
            &id,
            3,
            op(json!({"operation":"split_item","itemId":item,"splitMs":275})),
        )
        .unwrap()
        .changed_ids[1]
        .clone();
    let copy = core
        .edit(
            &id,
            4,
            op(json!({"operation":"duplicate_items","itemIds":[right],"offsetMs":2000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(
        &id,
        5,
        op(json!({"operation":"trim_item","itemId":copy,"startMs":2300,"durationMs":1400})),
    )
    .unwrap();
    let p = core.get_project(&id).unwrap();
    for (owner, offset) in [(&right, 275), (&copy, 300)] {
        for (index, c) in p
            .find_item(owner)
            .unwrap()
            .visual_properties()
            .animation_channels
            .iter()
            .enumerate()
        {
            assert_eq!(c.keyframes, source[index].keyframes);
            assert_eq!(
                c.clock,
                Some(AnimationClock {
                    offset_ms: offset,
                    source_duration_ms: 1800
                })
            );
            let target = c.target.as_ref().unwrap();
            assert_eq!(target.scope, "root");
            assert_eq!(target.id, if index == 3 { "tint" } else { owner });
        }
    }
}

#[test]
fn replacing_one_preset_property_keeps_other_retained_typed_and_legacy_clocks() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    let mut opacity = keys();
    for key in opacity.as_array_mut().unwrap() {
        key["value"]["value"] = json!(key["value"]["value"].as_f64().unwrap() / 100.0);
    }
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()},{"property":"transform.opacity","keyframes":opacity}]}))).unwrap();
    core.edit(&id,2,op(json!({"operation":"set_keyframes","itemId":item,"keyframes":[{"property":"scale","timeMs":0,"value":{"type":"scalar","value":1.0},"easing":"ease_in"},{"property":"scale","timeMs":2200,"value":{"type":"scalar","value":2.0},"easing":"linear"}]}))).unwrap();
    core.edit(
        &id,
        3,
        op(json!({"operation":"trim_item","itemId":item,"startMs":500,"durationMs":1000})),
    )
    .unwrap();
    let before = core
        .get_project(&id)
        .unwrap()
        .find_item(&item)
        .unwrap()
        .visual_properties()
        .clone();
    core.edit(&id,4,op(json!({"operation":"apply_animation_preset","itemId":item,"presetId":"scalar_tween","presetVersion":1,"collisionPolicy":"replace","parameters":{"property":"transform.opacity","startMs":0,"durationMs":500,"from":0.0,"to":1.0}}))).unwrap();
    let p = core.get_project(&id).unwrap();
    let after = p.find_item(&item).unwrap().visual_properties();
    assert_eq!(after.legacy_animation_clock, before.legacy_animation_clock);
    assert_eq!(
        after
            .animation_channels
            .iter()
            .find(|c| c.property == opencut_editor_core::AnimationChannelProperty::PositionX),
        before
            .animation_channels
            .iter()
            .find(|c| c.property == opencut_editor_core::AnimationChannelProperty::PositionX)
    );
    assert!(
        after
            .animation_channels
            .iter()
            .find(|c| c.property == opencut_editor_core::AnimationChannelProperty::Opacity)
            .unwrap()
            .clock
            .is_none()
    );
}

#[test]
fn locked_clock_edits_preserve_existing_track_error_and_history() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys()}]}))).unwrap();
    core.edit(
        &id,
        2,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for operation in [
        json!({"operation":"trim_item","itemId":item,"startMs":250,"durationMs":100}),
        json!({"operation":"split_item","itemId":item,"splitMs":500}),
        json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":3000}),
    ] {
        assert_eq!(
            core.edit(&id, 3, op(operation)).unwrap_err().code,
            ErrorCode::TrackLocked
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
}

#[test]
fn standalone_duplicate_materializes_equivalent_implicit_clock_only_on_copy() {
    let (_root, core, id, track) = setup();
    let item = add(&core, &id, &track);
    core.edit(&id,1,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":keys(),"loop":{"mode":"ping_pong","iterations":1}}]}))).unwrap();
    let before = core
        .get_project(&id)
        .unwrap()
        .find_item(&item)
        .unwrap()
        .visual_properties()
        .animation_channels
        .clone();
    let copy = core
        .edit(
            &id,
            2,
            op(json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":3000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let p = core.get_project(&id).unwrap();
    assert_eq!(
        p.find_item(&item)
            .unwrap()
            .visual_properties()
            .animation_channels,
        before
    );
    let copied = &p
        .find_item(&copy)
        .unwrap()
        .visual_properties()
        .animation_channels[0];
    assert_eq!(
        copied.clock,
        Some(AnimationClock {
            offset_ms: 0,
            source_duration_ms: 2400
        })
    );
    assert_eq!(copied.keyframes, before[0].keyframes);
    assert_eq!(copied.r#loop, before[0].r#loop);
}
