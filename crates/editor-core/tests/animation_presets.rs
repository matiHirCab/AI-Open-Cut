use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PROJECT_SCHEMA_VERSION, PathPolicy,
    ProjectSettings,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/animation-presets-v1.json")).unwrap()
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    setup_canvas(64)
}
fn setup_canvas(size: u32) -> (tempfile::TempDir, EditorCore, String, String) {
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
    let project = core
        .create_project(
            "Presets",
            ProjectSettings {
                width: size,
                height: size,
                fps: 20,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let result=core.edit(&project,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    (root, core, project, result.changed_ids[0].clone())
}
fn request(item: &str) -> Value {
    let mut value = catalog()["examples"]["apply"].clone();
    value["itemId"] = json!(item);
    value
}
fn state(core: &EditorCore, project: &str) -> Value {
    serde_json::to_value(core.get_project(project).unwrap()).unwrap()
}
fn item(state: &Value) -> &Value {
    &state["tracks"][1]["items"][0]
}

#[test]
fn canonical_expansion_defaults_and_undo_reopen_are_exact() {
    let (_root, core, project, id) = setup();
    let before = state(&core, &project);
    core.edit(&project, 1, op(request(&id))).unwrap();
    let after = state(&core, &project);
    assert_eq!(after["schemaVersion"], PROJECT_SCHEMA_VERSION);
    assert_eq!(
        item(&after)["animationChannels"],
        json!([catalog()["examples"]["resolvedChannel"]])
    );
    assert_eq!(
        item(&after)["animationPresetProvenance"]["transform.opacity"],
        catalog()["examples"]["provenance"]
    );
    assert_eq!(state(&core, &project), after);
    core.undo(&project, 2).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&before));
    core.redo(&project, 3).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&after));
}

#[test]
fn all_visual_seed_properties_and_curves_have_fixed_keys() {
    for (property, from, to) in [
        ("transform.position_x", -20.0, 30.0),
        ("transform.position_y", 10.0, -10.0),
        ("transform.scale_x", 0.5, 2.0),
        ("transform.scale_y", 1.0, 0.5),
        ("transform.opacity", 0.0, 1.0),
    ] {
        for curve in [
            json!("hold"),
            json!("linear"),
            json!({"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1.0}),
            json!({"type":"spring","mass":1.0,"stiffness":100.0,"damping":20.0,"initialVelocity":0.0}),
        ] {
            let (_root, core, project, id) = setup();
            let mut input = request(&id);
            input["parameters"] = json!({"property":property,"startMs":50,"durationMs":400,"from":from,"to":to,"curve":curve});
            core.edit(&project, 1, op(input.clone())).unwrap();
            let result = state(&core, &project);
            assert_eq!(
                item(&result)["animationChannels"],
                json!([{"property":property,"keyframes":[{"timeMs":50,"value":{"type":"scalar","value":from},"curve":curve},{"timeMs":450,"value":{"type":"scalar","value":to},"curve":"hold"}]}])
            );
            assert_eq!(
                item(&result)["animationPresetProvenance"][property]["parameters"],
                input["parameters"]
            );
        }
    }
}

#[test]
fn invalid_catalog_cases_and_half_open_timing_publish_nothing() {
    let (root, core, project, id) = setup();
    let before = state(&core, &project);
    let dir = root.path().join("projects").join(&project);
    let files = || {
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        )
    };
    let bytes = files();
    for case in catalog()["invalidApplications"].as_array().unwrap() {
        let mut input = request(&id);
        let path = case["field"].as_str().unwrap();
        if let Some(key) = path.strip_prefix("parameters.") {
            input["parameters"][key] = case["value"].clone();
        } else {
            input[path] = case["value"].clone();
        }
        assert_eq!(
            core.edit(&project, 1, op(input)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(state(&core, &project), before);
        assert_eq!(files(), bytes);
    }
    for (start, duration) in [
        (0, 1000),
        (900, 100),
        (u64::MAX, 1),
        (9_007_199_254_740_991, 1),
    ] {
        let mut input = request(&id);
        input["parameters"]["startMs"] = json!(start);
        input["parameters"]["durationMs"] = json!(duration);
        assert_eq!(
            core.edit(&project, 1, op(input)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(), bytes);
    }
    let mut valid = request(&id);
    valid["parameters"]["durationMs"] = json!(999);
    core.edit(&project, 1, op(valid)).unwrap();
}

#[test]
fn collision_replace_order_and_raw_clear_are_explicit() {
    let (_root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    let before = state(&core, &project);
    let mut disjoint = request(&id);
    disjoint["parameters"]["startMs"] = json!(600);
    disjoint["parameters"]["durationMs"] = json!(100);
    assert_eq!(
        core.edit(&project, 2, op(disjoint.clone()))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(state(&core, &project), before);
    disjoint["collisionPolicy"] = json!("replace");
    core.edit(&project, 2, op(disjoint)).unwrap();
    let replaced = state(&core, &project);
    assert_eq!(
        item(&replaced)["animationChannels"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let raw = op(
        json!({"operation":"set_animation_channels","itemId":id,"animationChannels":item(&replaced)["animationChannels"]}),
    );
    core.edit(&project, 3, raw).unwrap();
    assert!(
        item(&state(&core, &project))
            .get("animationPresetProvenance")
            .is_none()
    );
    core.undo(&project, 4).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&replaced));
    core.redo(&project, 5).unwrap();
    assert!(
        item(&state(&core, &project))
            .get("animationPresetProvenance")
            .is_none()
    );
}

#[test]
fn batch_alias_success_and_later_failure_are_atomic() {
    let (_root, core, project, _id) = setup();
    let before = state(&core, &project);
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let create = json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#00ff00","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"new"});
    let edits: Vec<BatchEditOperation> =
        serde_json::from_value(json!([create, request("@new")])).unwrap();
    let result = core.edit_batch(&project, 1, edits.clone()).unwrap();
    assert!(result.aliases.contains_key("new"));
    core.undo(&project, 2).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&before));
    let mut failing = edits;
    failing.push(serde_json::from_value(request("@new")).unwrap());
    let undo_state = state(&core, &project);
    assert_eq!(
        core.edit_batch(&project, 3, failing).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(state(&core, &project), undo_state);
    assert_eq!(
        core.edit(&project, 2, op(request("missing")))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(&project, 3, op(request("missing")))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
}

#[test]
fn drafts_reject_intents_and_preview_saved_channels_in_isolation() {
    let (root, core, project, id) = setup();
    let before = state(&core, &project);
    assert_eq!(
        core.create_draft(&project, 1, vec![op(request(&id))], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(state(&core, &project), before);
    assert!(
        !root
            .path()
            .join("projects")
            .join(&project)
            .join("drafts")
            .exists()
    );
    core.edit(&project, 1, op(request(&id))).unwrap();
    let saved = state(&core, &project);
    let draft = core
        .create_draft(
            &project,
            2,
            vec![op(
                json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[]}),
            )],
            None,
        )
        .unwrap();
    assert_eq!(
        core.update_draft(&project, &draft.id, 2, vec![op(request(&id))], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let candidate =
        serde_json::to_value(core.get_draft_state(&project, &draft.id).unwrap().project).unwrap();
    assert!(item(&candidate).get("animationPresetProvenance").is_none());
    assert_eq!(state(&core, &project), saved);
    core.commit_draft(&project, &draft.id, 2).unwrap();
    assert!(
        item(&state(&core, &project))
            .get("animationPresetProvenance")
            .is_none()
    );
    core.undo(&project, 3).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&saved));
}

#[test]
fn retired_source_reopens_without_dispatch_and_future_schema_fails_closed() {
    let (root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    let path = root
        .path()
        .join("projects")
        .join(&project)
        .join("project.json");
    let mut saved = state(&core, &project);
    saved["tracks"][1]["items"][0]["animationPresetProvenance"]["transform.opacity"] =
        catalog()["examples"]["retiredProvenance"].clone();
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(state(&core, &project), saved);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    saved["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION + 1);
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        core.get_project(&project).unwrap_err().code,
        ErrorCode::InternalError
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn provenance_null_missing_curve_or_premature_field_is_not_defaulted() {
    let (_root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    let saved = state(&core, &project);
    let mut invalid = saved.clone();
    invalid["tracks"][1]["items"][0]["animationPresetProvenance"] = Value::Null;
    assert!(serde_json::from_value::<opencut_editor_core::Project>(invalid).is_err());
    let mut invalid = saved.clone();
    invalid["tracks"][1]["items"][0]["animationPresetProvenance"]["transform.opacity"]["parameters"].as_object_mut().unwrap().remove("curve");
    assert!(serde_json::from_value::<opencut_editor_core::Project>(invalid).is_err());
    let mut invalid = saved;
    invalid["schemaVersion"] = json!(28);
    invalid["tracks"][1]["items"][0]["animationPresetProvenance"] = json!({});
    assert!(serde_json::from_value::<opencut_editor_core::Project>(invalid).is_err());
}

#[test]
fn strict_preset_records_reject_arrays_and_duplicate_raw_fields_without_publication() {
    use opencut_editor_core::{AnimationPresetParameters, AnimationPresetProvenance};
    let parameter_cases = [
        r#"["transform.opacity",0,500,0.0,1.0]"#,
        r#"["transform.opacity",0,500,0.0,1.0,"linear"]"#,
        r#"{"property":"transform.opacity","startMs":0,"durationMs":500,"from":99.0,"from":0.0,"to":1.0,"curve":"linear"}"#,
        r#"{"property":"transform.opacity","startMs":0,"durationMs":500,"from":0.0,"to":1.0,"curve":{"type":"cubic_bezier","x1":99.0,"x1":0.0,"y1":0.0,"x2":1.0,"y2":1.0}}"#,
    ];
    for raw in parameter_cases {
        assert!(
            serde_json::from_str::<AnimationPresetParameters>(raw).is_err(),
            "{raw}"
        );
        let mut request = catalog()["examples"]["apply"].clone();
        request["parameters"] = json!("PARAMETERS_TOKEN");
        let request = serde_json::to_string(&request)
            .unwrap()
            .replace("\"PARAMETERS_TOKEN\"", raw);
        assert!(
            serde_json::from_str::<EditOperation>(&request).is_err(),
            "{request}"
        );
    }
    let source_cases = [
        r#"["scalar_tween",1,1,{"property":"transform.opacity","startMs":0,"durationMs":500,"from":0.0,"to":1.0,"curve":"linear"}]"#,
        r#"{"presetId":"scalar_tween","presetVersion":1,"compilerVersion":1,"parameters":{"property":"transform.opacity","startMs":0,"durationMs":500,"from":99.0,"from":0.0,"to":1.0,"curve":"linear"}}"#,
        r#"{"presetId":"scalar_tween","presetVersion":1,"compilerVersion":1,"parameters":{"property":"transform.opacity","startMs":0,"durationMs":500,"from":0.0,"to":1.0,"curve":{"type":"cubic_bezier","x1":99.0,"x1":0.0,"y1":0.0,"x2":1.0,"y2":1.0}}}"#,
    ];
    for raw in source_cases {
        assert!(
            serde_json::from_str::<AnimationPresetProvenance>(raw).is_err(),
            "{raw}"
        );
    }
    let mut source_maps: Vec<String> = source_cases
        .into_iter()
        .map(|raw| format!(r#"{{"transform.opacity":{raw}}}"#))
        .collect();
    let valid = serde_json::to_string(&catalog()["examples"]["provenance"]).unwrap();
    for invalid in [
        valid.replace("scalar_tween", "Bad/ID"),
        {
            let mut source = catalog()["examples"]["provenance"].clone();
            source["parameters"]["from"] = json!(99.0);
            serde_json::to_string(&source).unwrap()
        },
        valid.clone(),
    ] {
        source_maps.push(format!(
            r#"{{"transform.opacity":{invalid},"transform.opacity":{valid}}}"#
        ));
    }
    for raw in source_maps {
        for target in ["current", "component", "undo", "redo"] {
            let (_root, core, project, id) = setup();
            core.edit(&project, 1, op(request(&id))).unwrap();
            for (revision, color) in [(2, "#00ff00"), (3, "#0000ff")] {
                core.edit(
                    &project,
                    revision,
                    op(json!({"operation":"update_item","itemId":id,"color":color})),
                )
                .unwrap();
            }
            core.undo(&project, 4).unwrap();
            let dir = core.paths().project_dir(&project).unwrap();
            let mut document: Value = if matches!(target, "current" | "component") {
                state(&core, &project)
            } else {
                serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap()
            };
            let candidate = if matches!(target, "current" | "component") {
                &mut document
            } else {
                document[target]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|snapshot| {
                        snapshot["tracks"][1]["items"][0]
                            .get("animationPresetProvenance")
                            .is_some()
                    })
                    .unwrap()
            };
            if target == "component" {
                candidate["components"] = json!([{
                    "id":"source-component","name":"Source component","width":64,
                    "height":64,"durationMs":1000,"tracks":[candidate["tracks"][1]],"slots":[],"markers":[]
                }]);
                serde_json::from_value::<opencut_editor_core::Project>(candidate.clone()).unwrap();
                candidate["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"] =
                    json!("SOURCE_TOKEN");
            } else {
                candidate["tracks"][1]["items"][0]["animationPresetProvenance"] =
                    json!("SOURCE_TOKEN");
            }
            let bytes = serde_json::to_string(&document)
                .unwrap()
                .replace("\"SOURCE_TOKEN\"", &raw);
            std::fs::write(
                dir.join(if matches!(target, "current" | "component") {
                    "project.json"
                } else {
                    "history.json"
                }),
                bytes,
            )
            .unwrap();
            let before = files(&core, &project);
            assert_eq!(
                core.get_project(&project).unwrap_err().code,
                ErrorCode::InternalError,
                "{target} {raw}"
            );
            assert_eq!(files(&core, &project), before, "{target} {raw}");
        }
    }
}

fn files(core: &EditorCore, project: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(project).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}

#[test]
fn audio_gain_curves_use_existing_media_compatibility() {
    use opencut_editor_core::{MediaProbeFacts, MediaType};
    let (root, core, project, rectangle) = setup();
    let source = root.path().join("media/tone.wav");
    std::fs::write(&source, b"fixture").unwrap();
    let asset = core
        .import_asset(
            &project,
            1,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let track = core.get_project(&project).unwrap().tracks[2].id.clone();
    let audio = core.edit(&project, 2, op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
    let before = files(&core, &project);
    let mut gain = request(&rectangle);
    gain["parameters"] =
        json!({"property":"audio.gain_db","startMs":0,"durationMs":500,"from":-96.0,"to":12.0});
    assert_eq!(
        core.edit(&project, 3, op(gain.clone())).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &project), before);
    assert_eq!(
        core.edit(&project, 3, op(request(&audio)))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    gain["itemId"] = json!(audio);
    for (index, curve) in [
        json!("hold"),
        json!("linear"),
        json!({"type":"cubic_bezier","x1":0.0,"y1":0.0,"x2":1.0,"y2":1.0}),
        json!({"type":"spring","mass":1.0,"stiffness":100.0,"damping":20.0,"initialVelocity":0.0}),
    ]
    .into_iter()
    .enumerate()
    {
        gain["parameters"]["curve"] = curve.clone();
        gain["collisionPolicy"] = json!("replace");
        core.edit(&project, 3 + index as u64, op(gain.clone()))
            .unwrap();
        let saved = state(&core, &project);
        assert_eq!(
            saved["tracks"][2]["items"][0]["animationChannels"],
            json!([{"property":"audio.gain_db","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":-96.0},"curve":curve},{"timeMs":500,"value":{"type":"scalar","value":12.0},"curve":"hold"}]}])
        );
    }
}

#[test]
fn locks_alias_misuse_and_batch_limits_publish_nothing() {
    let (_root, core, project, id) = setup();
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    core.edit(
        &project,
        1,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let before = files(&core, &project);
    assert_eq!(
        core.edit(&project, 2, op(request(&id))).unwrap_err().code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(&core, &project), before);
    core.edit(
        &project,
        2,
        op(json!({"operation":"update_track","trackId":track,"locked":false})),
    )
    .unwrap();
    let before = files(&core, &project);
    for operations in [
        json!([request("@unknown")]),
        {
            let mut input = request(&id);
            input["resultAlias"] = json!("source");
            json!([input])
        },
        json!(vec![request(&id); 101]),
    ] {
        let edits: Vec<BatchEditOperation> = serde_json::from_value(operations).unwrap();
        assert_eq!(
            core.edit_batch(&project, 3, edits).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(files(&core, &project), before);
    }
}

#[test]
fn replacement_preserves_order_static_state_and_legacy_collisions() {
    let (_root, core, project, id) = setup();
    let channel = |property: &str, value: f64| json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":value},"curve":"hold"}]});
    let original = json!([
        channel("transform.position_x", 20.0),
        channel("transform.opacity", 0.5),
        channel("transform.position_y", 30.0)
    ]);
    core.edit(
        &project,
        1,
        op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":original})),
    )
    .unwrap();
    let before = state(&core, &project);
    let mut input = request(&id);
    input["collisionPolicy"] = json!("replace");
    core.edit(&project, 2, op(input)).unwrap();
    let after = state(&core, &project);
    assert_eq!(item(&after)["animationChannels"][0], original[0]);
    assert_eq!(item(&after)["animationChannels"][2], original[2]);
    assert_eq!(item(&after)["transform"], item(&before)["transform"]);
    assert_eq!(
        item(&after)["animationChannels"][1],
        catalog()["examples"]["resolvedChannel"]
    );
    core.edit(
        &project,
        3,
        op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[]})),
    )
    .unwrap();
    core.edit(&project, 4, op(json!({"operation":"set_keyframes","itemId":id,"keyframes":[{"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"}]}))).unwrap();
    let before = files(&core, &project);
    for policy in ["reject", "replace"] {
        let mut input = request(&id);
        input["parameters"]["property"] = json!("transform.position_x");
        input["parameters"]["from"] = json!(0);
        input["parameters"]["to"] = json!(20);
        input["collisionPolicy"] = json!(policy);
        assert_eq!(
            core.edit(&project, 5, op(input)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &project), before);
    }
}

#[test]
fn unrelated_edits_copies_and_duration_failures_preserve_known_source() {
    let (_root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    let saved = state(&core, &project);
    let source = item(&saved)["animationPresetProvenance"].clone();
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let operations = [
        json!({"operation":"update_item","itemId":id,"color":"#00ff00"}),
        json!({"operation":"move_item","itemId":id,"trackId":track,"startMs":100}),
        json!({"operation":"set_item_visibility","itemId":id,"hidden":true}),
        json!({"operation":"trim_item","itemId":id,"startMs":100,"durationMs":1200}),
    ];
    for (index, input) in operations.into_iter().enumerate() {
        core.edit(&project, 2 + index as u64, op(input)).unwrap();
        assert_eq!(
            item(&state(&core, &project))["animationPresetProvenance"],
            source
        );
    }
    let before = files(&core, &project);
    for input in [
        json!({"operation":"trim_item","itemId":id,"startMs":100,"durationMs":500}),
        json!({"operation":"split_item","itemId":id,"splitMs":400}),
    ] {
        assert_eq!(
            core.edit(&project, 6, op(input)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &project), before);
    }
    let copies = core
        .edit(
            &project,
            6,
            op(json!({"operation":"duplicate_items","itemIds":[id],"offsetMs":1500})),
        )
        .unwrap();
    let copy = core
        .get_project(&project)
        .unwrap()
        .find_item(&copies.changed_ids[0])
        .unwrap()
        .visual_properties()
        .animation_preset_provenance
        .clone();
    assert_eq!(serde_json::to_value(copy).unwrap(), source);
    core.edit(
        &project,
        7,
        op(json!({"operation":"delete_item","itemId":id})),
    )
    .unwrap();
    core.undo(&project, 8).unwrap();
    assert_eq!(
        item(&state(&core, &project))["animationPresetProvenance"],
        source
    );
}

#[test]
fn raw_component_replacement_strips_forgery_and_preserves_only_known_exact_channels() {
    let (root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    let saved = state(&core, &project);
    let tracks = json!([saved["tracks"][1].clone()]);
    let created=core.edit(&project,2,op(json!({"operation":"component_create","name":"Seed","width":64,"height":64,"durationMs":1000,"tracks":tracks}))).unwrap();
    let component = &created.changed_ids[0];
    assert!(
        state(&core, &project)["components"][0]["tracks"][0]["items"][0]
            .get("animationPresetProvenance")
            .is_none()
    );
    // Model a previously persisted, valid source in a definition. Root authoring stays separate.
    let path = root
        .path()
        .join("projects")
        .join(&project)
        .join("project.json");
    let mut saved = state(&core, &project);
    saved["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"] =
        item(&saved)["animationPresetProvenance"].clone();
    std::fs::write(path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let mut unchanged = saved["components"][0]["tracks"].clone();
    unchanged[0]["items"][0]["animationPresetProvenance"]["transform.opacity"]["presetId"] =
        json!("forged");
    core.edit(&project,3,op(json!({"operation":"component_update","componentId":component,"name":"Seed","width":64,"height":64,"durationMs":1000,"tracks":unchanged}))).unwrap();
    let known = state(&core, &project);
    assert_eq!(
        known["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"],
        item(&saved)["animationPresetProvenance"]
    );
    let mut changed = saved["components"][0]["tracks"].clone();
    changed[0]["items"][0]["animationChannels"][0]["keyframes"][0]["value"]["value"] = json!(0.25);
    core.edit(&project,4,op(json!({"operation":"component_update","componentId":component,"name":"Seed","width":64,"height":64,"durationMs":1000,"tracks":changed}))).unwrap();
    assert!(
        state(&core, &project)["components"][0]["tracks"][0]["items"][0]
            .get("animationPresetProvenance")
            .is_none()
    );
    core.undo(&project, 5).unwrap();
    assert_eq!(
        state(&core, &project)["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"],
        item(&saved)["animationPresetProvenance"]
    );
    assert_eq!(
        core.edit(&project, 6, op(request("definition_only")))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    let mut signed_zero = saved["components"][0]["tracks"].clone();
    signed_zero[0]["items"][0]["animationChannels"][0]["keyframes"][0]["value"]["value"] =
        json!(-0.0);
    core.edit(&project,6,op(json!({"operation":"component_update","componentId":component,"name":"Seed","width":64,"height":64,"durationMs":1000,"tracks":signed_zero}))).unwrap();
    let rewritten = state(&core, &project);
    let rewritten_item = &rewritten["components"][0]["tracks"][0]["items"][0];
    assert!(rewritten_item.get("animationPresetProvenance").is_none());
    assert_eq!(
        rewritten_item["animationChannels"][0]["keyframes"][0]["value"]["value"]
            .as_f64()
            .unwrap()
            .to_bits(),
        (-0.0_f64).to_bits()
    );
    core.undo(&project, 7).unwrap();
    let restored = state(&core, &project);
    assert_eq!(
        serde_json::to_vec(&restored["components"][0]["tracks"][0]["items"][0]).unwrap(),
        serde_json::to_vec(&known["components"][0]["tracks"][0]["items"][0]).unwrap()
    );
}

#[test]
fn schema_28_migrates_current_components_and_all_history_once() {
    let (root, core, project, id) = setup();
    let saved = state(&core, &project);
    let tracks = json!([saved["tracks"][1].clone()]);
    core.edit(&project,1,op(json!({"operation":"component_create","name":"Legacy","width":64,"height":64,"durationMs":1000,"tracks":tracks}))).unwrap();
    core.edit(
        &project,
        2,
        op(json!({"operation":"update_item","itemId":id,"color":"#00ff00"})),
    )
    .unwrap();
    core.undo(&project, 3).unwrap();
    let dir = root.path().join("projects").join(&project);
    let mut current = state(&core, &project);
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    current["schemaVersion"] = json!(28);
    for snapshots in ["undo", "redo"] {
        for snapshot in history[snapshots].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(28);
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
    current["schemaVersion"] = json!(29);
    for snapshots in ["undo", "redo"] {
        for snapshot in history[snapshots].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(29);
        }
    }
    assert_eq!(state(&core, &project), current);
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(dir.join("history.json")).unwrap()).unwrap(),
        history
    );
    let bytes = files(&core, &project);
    assert_eq!(state(&core, &project), current);
    assert_eq!(files(&core, &project), bytes);
}

#[test]
fn malformed_retained_provenance_rejects_complete_generation_without_writes() {
    for target in ["current", "undo", "redo"] {
        let (_root, core, project, id) = setup();
        core.edit(&project, 1, op(request(&id))).unwrap();
        core.edit(
            &project,
            2,
            op(json!({"operation":"update_item","itemId":id,"color":"#00ff00"})),
        )
        .unwrap();
        core.edit(
            &project,
            3,
            op(json!({"operation":"update_item","itemId":id,"color":"#0000ff"})),
        )
        .unwrap();
        core.undo(&project, 4).unwrap();
        let dir = core.paths().project_dir(&project).unwrap();
        let mut current = state(&core, &project);
        let mut history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        let candidate = if target == "current" {
            &mut current
        } else {
            history[target]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|p| {
                    p["tracks"][1]["items"][0]
                        .get("animationPresetProvenance")
                        .is_some()
                })
                .unwrap()
        };
        candidate["tracks"][1]["items"][0]["animationPresetProvenance"]["transform.opacity"]["parameters"]
            ["property"] = json!("transform.position_x");
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
        let before = files(&core, &project);
        assert!(core.get_project(&project).is_err(), "{target}");
        assert_eq!(files(&core, &project), before, "{target}");
    }
}

#[test]
fn injected_draft_intent_is_rejected_on_every_materialization_without_writes() {
    let (_root, core, project, id) = setup();
    let draft = core
        .create_draft(
            &project,
            1,
            vec![op(
                json!({"operation":"update_item","itemId":id,"color":"#00ff00"}),
            )],
            None,
        )
        .unwrap();
    let dir = core.paths().project_dir(&project).unwrap();
    let path = dir.join("drafts").join(format!("{}.json", draft.id));
    let mut injected: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    injected["operations"] = json!([request(&id)]);
    std::fs::write(&path, serde_json::to_vec(&injected).unwrap()).unwrap();
    let before = files(&core, &project);
    let draft_bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        core.get_draft(&project, &draft.id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.get_draft_state(&project, &draft.id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.rebase_draft(&project, &draft.id, 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.commit_draft(&project, &draft.id, 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &project), before);
    assert_eq!(std::fs::read(path).unwrap(), draft_bytes);
}

fn native_tools() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    match (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) {
        (Some(ff), Some(fp)) => Some((ff.into(), fp.into())),
        _ => {
            assert_ne!(
                std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
                Ok("1"),
                "preset native evidence requires FFmpeg/FFprobe"
            );
            None
        }
    }
}
fn decode(ffmpeg: &std::path::Path, path: &std::path::Path, audio: bool) -> Vec<u8> {
    let mut command = std::process::Command::new(ffmpeg);
    command.args(["-v", "error", "-i"]).arg(path);
    if audio {
        command.args([
            "-map", "0:a:0", "-f", "f32le", "-ac", "1", "-ar", "48000", "-",
        ]);
    } else {
        command.args(["-map", "0:v:0", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn native_preset_matches_independent_primitives_frames_range_draft_export_and_retired_source() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Project, Renderer};
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    let (root, core, id, item_id) = setup();
    core.edit(&id, 1, op(request(&item_id))).unwrap();
    let compiled = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let mut manual = compiled.clone();
    let visual = manual.tracks[1].items[0].visual_properties_mut();
    visual.animation_preset_provenance.clear();
    visual.animation_channels=serde_json::from_value(json!([{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":1.0},"curve":"hold"}]}])).unwrap();
    let frame = |project: &Project, time| {
        let result = renderer.render_preview(project, &dir, time).unwrap();
        decode(&ffmpeg, &dir.join(result.relative_path), false)
    };
    for time in [0, 250, 500, 750, 950] {
        assert_eq!(
            frame(&compiled, time),
            frame(&manual, time),
            "frame at {time}"
        );
    }
    let draft = core
        .create_draft(
            &id,
            2,
            vec![op(
                json!({"operation":"update_item","itemId":item_id,"color":"#ff0000"}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(frame(&candidate, 250), frame(&manual, 250));
    core.discard_draft(&id, &draft.id).unwrap();
    assert_eq!(core.get_project(&id).unwrap().revision, 2);
    let range = |project: &Project| {
        let result = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 20,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        decode(&ffmpeg, &dir.join(result.relative_path), false)
    };
    assert_eq!(range(&compiled), range(&manual));
    let export = |project: &Project, name: &str| {
        let output = root.path().join(name);
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        decode(&ffmpeg, &output, false)
    };
    assert_eq!(
        export(&compiled, "preset.mp4"),
        export(&manual, "manual.mp4")
    );
    let mut retired = compiled.clone();
    let source = retired.tracks[1].items[0]
        .visual_properties_mut()
        .animation_preset_provenance
        .values_mut()
        .next()
        .unwrap();
    source.preset_id = "retired_tween".into();
    source.preset_version = 77;
    source.compiler_version = 55;
    assert_eq!(frame(&retired, 250), frame(&manual, 250));
    // Same independently authored channel inside a fractional inherited composition clock.
    let nested = |project: &Project| {
        let mut saved = serde_json::to_value(project).unwrap();
        let child = saved["tracks"][1].clone();
        saved["components"] = json!([{"id":"nested","name":"Nested","width":64,"height":64,"durationMs":1000,"tracks":[child],"slots":[],"markers":[]}]);
        saved["tracks"][1]["items"] = json!([{"type":"component_instance","id":"instance","componentId":"nested","startMs":100,"durationMs":500,"trimStartMs":25,"timeScale":0.75,"slotValues":{},"stackOrder":0,"zIndex":0}]);
        serde_json::from_value::<Project>(saved).unwrap()
    };
    for time in [100, 101, 333, 550] {
        assert_eq!(
            frame(&nested(&compiled), time),
            frame(&nested(&manual), time),
            "fractional nested frame {time}"
        );
    }
}

#[test]
fn native_gain_preset_matches_independent_audio_range_and_export() {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    let (root, core, id, _) = setup();
    let source = root.path().join("media/tone.wav");
    let generated = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
        ])
        .arg(&source)
        .output()
        .unwrap();
    assert!(generated.status.success());
    let asset = core
        .import_asset(
            &id,
            1,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                audio_sample_rate_hz: Some(48000),
                audio_channels: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let track = core.get_project(&id).unwrap().tracks[2].id.clone();
    let audio=core.edit(&id,2,op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
    let mut input = request(&audio);
    input["parameters"] =
        json!({"property":"audio.gain_db","startMs":0,"durationMs":500,"from":0.0,"to":-12.0});
    core.edit(&id, 3, op(input)).unwrap();
    let compiled = core.get_project(&id).unwrap();
    let mut manual = compiled.clone();
    let visual = manual.tracks[2].items[0].visual_properties_mut();
    visual.animation_preset_provenance.clear();
    visual.animation_channels=serde_json::from_value(json!([{"property":"audio.gain_db","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":-12.0},"curve":"hold"}]}])).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let range = |project: &opencut_editor_core::Project| {
        let result = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 20,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        decode(&ffmpeg, &dir.join(result.relative_path), true)
    };
    assert_eq!(range(&compiled), range(&manual));
    let export = |project: &opencut_editor_core::Project, name: &str| {
        let output = root.path().join(name);
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        decode(&ffmpeg, &output, true)
    };
    assert_eq!(
        export(&compiled, "gain.mp4"),
        export(&manual, "raw-gain.mp4")
    );
}

#[test]
fn malformed_identity_or_orphan_source_and_premature_retained_fields_fail_closed() {
    for mode in [
        "id",
        "version",
        "compiler",
        "orphan",
        "unknown_parameter",
        "premature_current",
        "premature_undo",
        "premature_redo",
        "future_undo",
        "future_redo",
    ] {
        let (_root, core, project, id) = setup();
        core.edit(&project, 1, op(request(&id))).unwrap();
        core.edit(
            &project,
            2,
            op(json!({"operation":"update_item","itemId":id,"color":"#00ff00"})),
        )
        .unwrap();
        core.edit(
            &project,
            3,
            op(json!({"operation":"update_item","itemId":id,"color":"#0000ff"})),
        )
        .unwrap();
        core.undo(&project, 4).unwrap();
        let dir = core.paths().project_dir(&project).unwrap();
        let mut current = state(&core, &project);
        let mut history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        if let Some(target) = mode.strip_prefix("premature_") {
            let candidate = if target == "current" {
                &mut current
            } else {
                history[target]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|p| !p["tracks"][1]["items"].as_array().unwrap().is_empty())
                    .unwrap()
            };
            candidate["schemaVersion"] = json!(28);
            candidate["tracks"][1]["items"][0]["animationPresetProvenance"] = json!({});
        } else if let Some(target) = mode.strip_prefix("future_") {
            history[target][0]["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION + 1);
        } else {
            let visual = &mut current["tracks"][1]["items"][0];
            match mode {
                "id" => {
                    visual["animationPresetProvenance"]["transform.opacity"]["presetId"] =
                        json!("Bad/ID")
                }
                "version" => {
                    visual["animationPresetProvenance"]["transform.opacity"]["presetVersion"] =
                        json!(0)
                }
                "compiler" => {
                    visual["animationPresetProvenance"]["transform.opacity"]["compilerVersion"] =
                        json!(0)
                }
                "orphan" => visual["animationChannels"] = json!([]),
                "unknown_parameter" => {
                    visual["animationPresetProvenance"]["transform.opacity"]["parameters"]["expression"] =
                        json!("t+1")
                }
                _ => unreachable!(),
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
        let before = files(&core, &project);
        let error = core.get_project(&project).unwrap_err();
        if mode.starts_with("future_") {
            assert_eq!(error.code, ErrorCode::InternalError);
        }
        assert_eq!(files(&core, &project), before, "{mode}");
    }
}

#[test]
fn successful_split_preserves_only_exact_local_channels_and_labels() {
    let (_root, core, project, id) = setup();
    let mut input = request(&id);
    input["parameters"]["durationMs"] = json!(200);
    core.edit(&project, 1, op(input)).unwrap();
    let before = state(&core, &project);
    let split = core
        .edit(
            &project,
            2,
            op(json!({"operation":"split_item","itemId":id,"splitMs":500})),
        )
        .unwrap();
    let after = core.get_project(&project).unwrap();
    for id in &split.changed_ids {
        let visual = after.find_item(id).unwrap().visual_properties();
        assert_eq!(
            serde_json::to_value(&visual.animation_channels).unwrap(),
            item(&before)["animationChannels"]
        );
        assert_eq!(
            serde_json::to_value(&visual.animation_preset_provenance).unwrap(),
            item(&before)["animationPresetProvenance"]
        );
    }
    core.undo(&project, 3).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&before));
}

#[test]
fn parameter_and_deferred_shape_boundaries_publish_no_accepted_prefix() {
    let (_root, core, project, id) = setup();
    let before = files(&core, &project);
    for (property, from, to) in [
        ("transform.position_x", -1_000_000.1, 0.0),
        ("transform.position_y", 0.0, 1_000_000.1),
        ("transform.scale_x", 0.0, 1.0),
        ("transform.scale_y", 1.0, 100.1),
        ("transform.opacity", -0.1, 1.0),
        ("audio.gain_db", -96.1, 0.0),
    ] {
        let mut input = request(&id);
        input["parameters"]["property"] = json!(property);
        input["parameters"]["from"] = json!(from);
        input["parameters"]["to"] = json!(to);
        assert_eq!(
            core.edit(&project, 1, op(input)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &project), before);
    }
    for extra in [
        json!({"target":"effect-id"}),
        json!({"expression":"time"}),
        json!({"loop":{"mode":"repeat"}}),
        json!({"url":"https://example.com/preset"}),
    ] {
        let mut input = request(&id);
        input["parameters"]
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert!(serde_json::from_value::<EditOperation>(input).is_err());
    }
    for curve in [
        json!({"type":"cubic_bezier","x1":-0.1,"y1":0.0,"x2":1.0,"y2":1.0}),
        json!({"type":"spring","mass":0.0,"stiffness":100.0,"damping":20.0,"initialVelocity":0.0}),
    ] {
        let mut input = request(&id);
        input["parameters"]["curve"] = curve;
        let edits: Vec<BatchEditOperation> = serde_json::from_value(
            json!([{"operation":"update_item","itemId":id,"color":"#00ff00"},input]),
        )
        .unwrap();
        assert_eq!(
            core.edit_batch(&project, 1, edits).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &project), before);
    }
}

#[test]
fn replacing_loop_is_whole_channel_and_undo_restores_original_loop() {
    let (_root, core, project, id) = setup();
    let original = json!([{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},{"timeMs":250,"value":{"type":"scalar","value":1.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":0.0},"curve":"hold"}],"loop":{"mode":"repeat","iterations":3}}]);
    core.edit(
        &project,
        1,
        op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":original})),
    )
    .unwrap();
    let before = state(&core, &project);
    let mut input = request(&id);
    input["collisionPolicy"] = json!("replace");
    core.edit(&project, 2, op(input)).unwrap();
    assert!(
        item(&state(&core, &project))["animationChannels"][0]
            .get("loop")
            .is_none()
    );
    core.undo(&project, 3).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&before));
}

#[test]
fn final_scene_budget_failure_rolls_back_preset_alias_and_existing_draft() {
    for version in [28, 29] {
        let (_root, core, project, id) = setup();
        let track = core.get_project(&project).unwrap().tracks[1].id.clone();
        let shape=core.edit(&project,1,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":40000,"geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":10,"y":10}}]}},"fill":null,"stroke":{"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}))).unwrap().changed_ids[0].clone();
        let draft = core
            .create_draft(
                &project,
                2,
                vec![op(
                    json!({"operation":"update_item","itemId":id,"color":"#00ff00"}),
                )],
                None,
            )
            .unwrap();
        let dir = core.paths().project_dir(&project).unwrap();
        let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
        let draft_bytes = std::fs::read(&draft_path).unwrap();
        if version == 28 {
            for name in ["project.json", "history.json"] {
                let path = dir.join(name);
                let mut value: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                if name == "project.json" {
                    value["schemaVersion"] = json!(28);
                } else {
                    for kind in ["undo", "redo"] {
                        for snapshot in value[kind].as_array_mut().unwrap() {
                            snapshot["schemaVersion"] = json!(28);
                        }
                    }
                }
                std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
        }
        let before = files(&core, &project);
        let channels = json!([{"property":"graphic.path_points","target":{"kind":"graphic_geometry","scope":"root","id":shape},"keyframes":[{"timeMs":0,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":10,"y":10}]},"curve":"linear"},{"timeMs":39999,"value":{"type":"path_points","points":[{"x":0,"y":0},{"x":20,"y":10}]},"curve":"hold"}]}]);
        let operations:Vec<BatchEditOperation>=serde_json::from_value(json!([{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":8,"height":8,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"new"},request("@new"),{"operation":"set_animation_channels","itemId":shape,"animationChannels":channels}])).unwrap();
        let error = core.edit_batch(&project, 2, operations).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(
            error.message.contains("maxCandidateAnalysisNodes"),
            "{}",
            error.message
        );
        assert_eq!(files(&core, &project), before);
        assert_eq!(std::fs::read(draft_path).unwrap(), draft_bytes);
    }
}

#[test]
fn root_group_component_and_parenting_use_existing_compatibility() {
    let (_root, core, project, id) = setup();
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let group = core
        .edit(
            &project,
            1,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit_batch(
        &project,
        2,
        vec![
            op(json!({"operation":"update_item","itemId":group,"transform2d":null})),
            op(request(&group)),
        ],
    )
    .unwrap();
    core.edit(&project, 3, op(request(&id))).unwrap();
    let source = item(&state(&core, &project))["animationPresetProvenance"].clone();
    core.edit(
        &project,
        4,
        op(json!({"operation":"item_set_parent","itemId":id,"parent":{"scope":"root","id":group}})),
    )
    .unwrap();
    assert_eq!(
        item(&state(&core, &project))["animationPresetProvenance"],
        source
    );
    let component=core.edit(&project,5,op(json!({"operation":"component_create","name":"Empty","width":64,"height":64,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
    let instance=core.edit(&project,6,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1}))).unwrap().changed_ids[0].clone();
    core.edit(&project, 7, op(request(&instance))).unwrap();
    // Transform2D remains incompatible with these legacy scalar axes.
    let before = files(&core, &project);
    let edits:Vec<BatchEditOperation>=serde_json::from_value(json!([{"operation":"set_animation_channels","itemId":id,"animationChannels":[]},{"operation":"update_item","itemId":id,"transform2d":opencut_editor_core::Transform2D::default()},request(&id)])).unwrap();
    assert_eq!(
        core.edit_batch(&project, 8, edits).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &project), before);
}

#[test]
fn canonical_collision_outcomes_are_independent_of_live_compiler() {
    for case in catalog()["collisionCases"].as_array().unwrap() {
        let (_root, core, project, id) = setup();
        core.edit(&project, 1, op(request(&id))).unwrap();
        let before = files(&core, &project);
        let mut input = request(&id);
        input["parameters"]["startMs"] = case["startMs"].clone();
        input["parameters"]["durationMs"] = case["durationMs"].clone();
        if let Some(policy) = case.get("collisionPolicy") {
            input["collisionPolicy"] = policy.clone();
        }
        let result = core.edit(&project, 2, op(input));
        if case["accepted"] == true {
            assert_eq!(result.unwrap().revision, 3);
            assert_eq!(
                item(&state(&core, &project))["animationChannels"]
                    .as_array()
                    .unwrap()
                    .len(),
                case["channelCount"].as_u64().unwrap() as usize
            );
        } else {
            assert_eq!(
                serde_json::to_value(result.unwrap_err().code).unwrap(),
                case["error"]
            );
            assert_eq!(files(&core, &project), before);
        }
    }
}

#[test]
fn legacy_slot_named_animation_preset_provenance_migrates_without_false_rejection() {
    let (_root, core, project, _) = setup();
    let saved = state(&core, &project);
    let child_id = item(&saved)["id"].clone();
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let component=core.edit(&project,1,op(json!({"operation":"component_create","name":"Named slot","width":64,"height":64,"durationMs":1000,"tracks":[saved["tracks"][1]],"slots":[{"id":"animationPresetProvenance","name":"Opacity override","kind":"number","required":false,"defaultValue":{"type":"number","value":1},"binding":{"targetLayerId":child_id,"property":"visual.opacity"},"constraints":{}}]}))).unwrap().changed_ids[0].clone();
    core.edit(&project,2,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{"animationPresetProvenance":{"type":"number","value":0.5}}}))).unwrap();
    let dir = core.paths().project_dir(&project).unwrap();
    let mut current = state(&core, &project);
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    current["schemaVersion"] = json!(28);
    for list in ["undo", "redo"] {
        for snapshot in history[list].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(28);
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
    current["schemaVersion"] = json!(29);
    assert_eq!(state(&core, &project), current);
    let bytes = files(&core, &project);
    assert_eq!(state(&core, &project), current);
    assert_eq!(files(&core, &project), bytes);
}

#[test]
fn legacy_invalid_preset_does_not_publish_schema_migration() {
    let (_root, core, project, id) = setup();
    let dir = core.paths().project_dir(&project).unwrap();
    let mut doc = state(&core, &project);
    doc["schemaVersion"] = json!(28);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&doc).unwrap()).unwrap();
    let mut hist: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for k in ["undo", "redo"] {
        for snap in hist[k].as_array_mut().unwrap() {
            snap["schemaVersion"] = json!(28);
        }
    }
    std::fs::write(dir.join("history.json"), serde_json::to_vec(&hist).unwrap()).unwrap();
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    let mut req = request(&id);
    req["presetVersion"] = json!(2);
    let err = core.edit(&project, 1, op(req)).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
    let after = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    assert!(
        before == after,
        "INVALID_ARGUMENT changed schema to {}",
        serde_json::from_slice::<Value>(&after.0).unwrap()["schemaVersion"]
    );
}

#[test]
fn legacy_failed_alias_batch_does_not_publish_schema_migration() {
    let (_root, core, project, id) = setup();
    let dir = core.paths().project_dir(&project).unwrap();
    let mut doc = state(&core, &project);
    doc["schemaVersion"] = json!(28);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&doc).unwrap()).unwrap();
    let mut hist: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for k in ["undo", "redo"] {
        for snap in hist[k].as_array_mut().unwrap() {
            snap["schemaVersion"] = json!(28);
        }
    }
    std::fs::write(dir.join("history.json"), serde_json::to_vec(&hist).unwrap()).unwrap();
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    let mut req = request(&id);
    req["presetVersion"] = json!(2);
    let track = doc["tracks"][1]["id"].clone();
    req["itemId"] = json!("@seed");
    let ops:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
 {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#00ff00","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"seed"},req])).unwrap();
    let err = core.edit_batch(&project, 1, ops).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
    let after = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    assert!(
        before == after,
        "INVALID_ARGUMENT changed schema to {}",
        serde_json::from_slice::<Value>(&after.0).unwrap()["schemaVersion"]
    );
}

fn pack_files(
    core: &EditorCore,
    project: &str,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        dir: &std::path::Path,
        files: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, files);
            } else {
                files.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    visit(&core.paths().project_dir(project).unwrap(), &mut files);
    files
}

fn pack_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/initial-motion-preset-pack-v1.json"
    ))
    .unwrap()
}
fn pack_request(id: &str, entry: &Value) -> Value {
    json!({"operation":"apply_animation_preset","itemId":id,"presetId":entry["id"],"presetVersion":1,"parameters":entry["parameters"]})
}
fn pack_channels(entry: &Value) -> Value {
    Value::Array(entry["expected"]["channels"].as_array().unwrap().iter().map(|channel| {
        let times=channel["times"].as_array().unwrap();
        let keys:Vec<_>=times.iter().zip(channel["values"].as_array().unwrap()).enumerate().map(|(i,(time,value))| json!({"timeMs":time,"value":{"type":"scalar","value":value},"curve":if i+1 == times.len() { "hold" } else { "linear" }})).collect();
        let mut value=json!({"property":channel["property"],"keyframes":keys});
        if !entry["expected"]["loop"].is_null() { value["loop"]=entry["expected"]["loop"].clone(); }
        value
    }).collect())
}

#[test]
fn motion_pack_canonical_primitives_complete_sources_and_history_are_exact() {
    for entry in pack_fixture()["presets"].as_array().unwrap() {
        let (_root, core, project, id) = setup();
        let before = state(&core, &project);
        core.edit(&project, 1, op(pack_request(&id, entry)))
            .unwrap();
        let after = state(&core, &project);
        assert_eq!(
            item(&after)["animationChannels"],
            pack_channels(entry),
            "{}",
            entry["id"]
        );
        assert_eq!(
            item(&after)["animationPresetProvenance"],
            entry["expectedProvenance"],
            "{}",
            entry["id"]
        );
        assert_eq!(
            item(&after).get("motionBlur"),
            if entry["expected"]["motionBlur"].is_null() {
                None
            } else {
                Some(&entry["expected"]["motionBlur"])
            }
        );
        assert_eq!(after["schemaVersion"], 30);
        core.undo(&project, 2).unwrap();
        assert_eq!(item(&state(&core, &project)), item(&before));
        core.redo(&project, 3).unwrap();
        assert_eq!(item(&state(&core, &project)), item(&after));
        let reopened = EditorCore::new(core.paths().clone());
        reopened.get_project(&project).unwrap();
        assert_eq!(item(&state(&core, &project)), item(&after));
    }
}

#[test]
fn motion_pack_phase_minima_and_odd_times_use_fixed_oracles() {
    let fixture = pack_fixture();
    for case in fixture["phaseCases"].as_array().unwrap() {
        let (_root, core, project, id) = setup();
        let entry = fixture["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == case["id"])
            .unwrap();
        let mut request = pack_request(&id, entry);
        request["parameters"]["startMs"] = case["startMs"].clone();
        request["parameters"]["durationMs"] = case["durationMs"].clone();
        core.edit(&project, 1, op(request)).unwrap();
        let after = state(&core, &project);
        for channel in item(&after)["animationChannels"].as_array().unwrap() {
            let times: Vec<_> = channel["keyframes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| k["timeMs"].clone())
                .collect();
            assert_eq!(
                json!(times),
                case["timesByProperty"][channel["property"].as_str().unwrap()]
            );
        }
    }
}

#[test]
fn motion_pack_raw_shape_errors_preserve_original_duplicate_fields() {
    use opencut_editor_core::AnimationPresetParameters;
    for raw in pack_fixture()["invalidRawParameters"].as_array().unwrap() {
        let raw = raw.as_str().unwrap();
        assert!(
            serde_json::from_str::<AnimationPresetParameters>(raw).is_err(),
            "accepted {raw}"
        );
        let edit = format!(
            "{{\"operation\":\"apply_animation_preset\",\"itemId\":\"item\",\"presetId\":\"impact_slam\",\"presetVersion\":1,\"parameters\":{raw}}}"
        );
        assert!(
            serde_json::from_str::<EditOperation>(&edit).is_err(),
            "accepted {edit}"
        );
    }
}

#[test]
fn motion_pack_invalid_bounds_and_identity_leave_documents_and_resources_unchanged() {
    let fixture = pack_fixture();
    let (_root, core, project, id) = setup();
    let before = pack_files(&core, &project);
    for case in fixture["invalidApplications"].as_array().unwrap() {
        let entry = fixture["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == case["id"])
            .unwrap();
        let mut request = pack_request(&id, entry);
        let mut target = &mut request["parameters"];
        let parts: Vec<_> = case["field"].as_str().unwrap().split('.').collect();
        for part in &parts[..parts.len() - 1] {
            target = &mut target[*part];
        }
        target[parts[parts.len() - 1]] = case["value"].clone();
        let error = match serde_json::from_value::<EditOperation>(request) {
            Ok(request) => core.edit(&project, 1, request).unwrap_err(),
            Err(_) => {
                assert_eq!(pack_files(&core, &project), before);
                continue;
            }
        };
        assert_eq!(error.code, ErrorCode::InvalidArgument, "{case}");
        assert!(!error.retryable);
        assert_eq!(pack_files(&core, &project), before, "{case}");
    }
    for entry in fixture["presets"].as_array().unwrap() {
        for field in ["presetId", "presetVersion"] {
            let mut request = pack_request(&id, entry);
            request[field] = if field == "presetId" {
                json!("retired_pack")
            } else {
                json!(2)
            };
            assert_eq!(
                core.edit(&project, 1, op(request)).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(pack_files(&core, &project), before);
        }
    }
}

#[test]
fn motion_pack_impact_blur_reject_replace_retired_labels_and_undo_are_exact() {
    let fixture = pack_fixture();
    let entry = &fixture["presets"][0];
    let (_root, core, project, id) = setup();
    core.edit(&project,1,op(json!({"operation":"update_item","itemId":id,"motionBlur":{"shutterAngleDeg":0,"sampleCount":1}}))).unwrap();
    let before = files(&core, &project);
    assert_eq!(
        core.edit(&project, 2, op(pack_request(&id, entry)))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &project), before);
    let mut request = pack_request(&id, entry);
    request["collisionPolicy"] = json!("replace");
    core.edit(&project, 2, op(request)).unwrap();
    let saved = state(&core, &project);
    core.edit(&project,3,op(json!({"operation":"update_item","itemId":id,"motionBlur":{"shutterAngleDeg":180,"sampleCount":3}}))).unwrap();
    assert_eq!(
        item(&state(&core, &project))["animationPresetProvenance"],
        item(&saved)["animationPresetProvenance"]
    );
    core.edit(&project,4,op(json!({"operation":"update_item","itemId":id,"motionBlur":{"shutterAngleDeg":90,"sampleCount":2}}))).unwrap();
    let after = state(&core, &project);
    assert!(item(&after).get("animationPresetProvenance").is_none());
    assert_eq!(
        item(&after)["animationChannels"],
        item(&saved)["animationChannels"]
    );
    core.undo(&project, 5).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&saved));
    core.redo(&project, 6).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&after));
}

#[test]
fn motion_pack_and_scalar_preserve_unrelated_scoped_channel_targets() {
    let fixture = pack_fixture();
    let mut entries = fixture["presets"].as_array().unwrap().clone();
    entries.push(json!({"id":"scalar_tween","parameters":{"property":"transform.opacity","startMs":10,"durationMs":80,"from":0.0,"to":1.0}}));
    for entry in entries {
        let (_root, core, project, id) = setup();
        core.edit(&project,1,op(json!({"operation":"update_item","itemId":id,"effects":[{"id":"soft","type":"gaussian_blur","radiusPx":1.0}]}))).unwrap();
        let channel = json!({"property":"effect.blur_radius","target":{"kind":"effect","scope":"root","id":"soft"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":2.0},"curve":"hold"}]});
        core.edit(&project,2,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[channel]}))).unwrap();
        let before = state(&core, &project);
        core.edit(&project, 3, op(pack_request(&id, &entry)))
            .unwrap();
        let after = state(&core, &project);
        assert_eq!(
            item(&after)["animationChannels"][0],
            channel,
            "{}",
            entry["id"]
        );
        assert_eq!(item(&after)["effects"], item(&before)["effects"]);
        core.undo(&project, 4).unwrap();
        assert_eq!(item(&state(&core, &project)), item(&before));
    }
}

#[test]
fn motion_pack_descriptive_retirement_membership_and_premature_generations_fail_closed() {
    let fixture = pack_fixture();
    for entry in fixture["presets"].as_array().unwrap() {
        let (_root, core, project, id) = setup();
        core.edit(&project, 1, op(pack_request(&id, entry)))
            .unwrap();
        let dir = core.paths().project_dir(&project).unwrap();
        let path = dir.join("project.json");
        let mut saved = state(&core, &project);
        let visual = &mut saved["tracks"][1]["items"][0];
        let property = visual["animationChannels"][0]["property"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut source = visual["animationPresetProvenance"][&property].clone();
        source["presetId"] = json!("retired_artistic_pack");
        source["presetVersion"] = json!(99);
        source["compilerVersion"] = json!(88);
        visual["animationChannels"] = json!([visual["animationChannels"][0]]);
        visual["animationPresetProvenance"] = json!({property.clone():source});
        std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
        let bytes = files(&core, &project);
        assert_eq!(state(&core, &project), saved);
        assert_eq!(files(&core, &project), bytes);
        if ["scan", "pulse", "radar_expand"].contains(&entry["id"].as_str().unwrap()) {
            let mut missing = saved.clone();
            missing["tracks"][1]["items"][0]["animationPresetProvenance"][&property]["parameters"]
                .as_object_mut()
                .unwrap()
                .remove("iterations");
            std::fs::write(&path, serde_json::to_vec(&missing).unwrap()).unwrap();
            let before = files(&core, &project);
            assert!(core.get_project(&project).is_err());
            assert_eq!(files(&core, &project), before);
        }
        let mut premature = saved.clone();
        premature["schemaVersion"] = json!(29);
        std::fs::write(&path, serde_json::to_vec(&premature).unwrap()).unwrap();
        let before = files(&core, &project);
        assert!(core.get_project(&project).is_err());
        assert_eq!(files(&core, &project), before);
        let mut mismatched = saved.clone();
        mismatched["tracks"][1]["items"][0]["animationPresetProvenance"] =
            json!({"audio.gain_db":source});
        std::fs::write(&path, serde_json::to_vec(&mismatched).unwrap()).unwrap();
        let before = files(&core, &project);
        assert!(core.get_project(&project).is_err());
        assert_eq!(files(&core, &project), before);
    }
}

#[test]
fn motion_pack_schema29_scalar_current_components_and_history_migrate_once_without_relabeling() {
    let (_root, core, project, id) = setup();
    core.edit(&project, 1, op(request(&id))).unwrap();
    core.edit(
        &project,
        2,
        op(json!({"operation":"update_item","itemId":id,"color":"#00ff00"})),
    )
    .unwrap();
    core.edit(
        &project,
        3,
        op(json!({"operation":"update_item","itemId":id,"color":"#0000ff"})),
    )
    .unwrap();
    core.undo(&project, 4).unwrap();
    let dir = core.paths().project_dir(&project).unwrap();
    let mut current = state(&core, &project);
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for document in std::iter::once(&mut current).chain(
        history
            .as_object_mut()
            .unwrap()
            .values_mut()
            .filter_map(Value::as_array_mut)
            .flat_map(|snapshots| snapshots.iter_mut()),
    ) {
        document["schemaVersion"] = json!(29);
        if !document["tracks"][1]["items"]
            .as_array()
            .unwrap()
            .is_empty()
        {
            let tracks = json!([document["tracks"][1]]);
            document["components"] = json!([{"id":"local","name":"Local","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[],"markers":[]}]);
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
    let before = json!([current, history]);
    let reopened = EditorCore::new(core.paths().clone());
    let migrated = serde_json::to_value(reopened.get_project(&project).unwrap()).unwrap();
    let migrated_history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    let mut expected = before;
    expected[0]["schemaVersion"] = json!(30);
    for stack in ["undo", "redo"] {
        for snapshot in expected[1][stack].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(30);
        }
    }
    assert_eq!(json!([migrated, migrated_history]), expected);
    let bytes = files(&core, &project);
    reopened.get_project(&project).unwrap();
    assert_eq!(files(&core, &project), bytes);
}

#[test]
fn native_motion_pack_matches_fixed_primitives_frames_range_draft_export_and_fractional_clocks() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Project, Renderer};
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    for entry in pack_fixture()["presets"].as_array().unwrap() {
        let (root, core, id, item_id) = setup_canvas(512);
        core.edit(&id, 1, op(pack_request(&item_id, entry)))
            .unwrap();
        let authoritative = files(&core, &id);
        let compiled = core.get_project(&id).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
        let mut manual = compiled.clone();
        let visual = manual.tracks[1].items[0].visual_properties_mut();
        visual.animation_preset_provenance.clear();
        visual.animation_channels = serde_json::from_value(pack_channels(entry)).unwrap();
        visual.motion_blur =
            serde_json::from_value(entry["expected"]["motionBlur"].clone()).unwrap();
        let frame = |project: &Project, time| {
            let result = renderer.render_preview(project, &dir, time).unwrap();
            decode(&ffmpeg, &dir.join(result.relative_path), false)
        };
        for time in [
            0, 9, 10, 25, 50, 55, 60, 65, 70, 75, 80, 89, 90, 91, 170, 950,
        ] {
            assert_eq!(
                frame(&compiled, time),
                frame(&manual, time),
                "frame at {time}"
            );
        }
        let draft = core
            .create_draft(
                &id,
                2,
                vec![op(
                    json!({"operation":"update_item","itemId":item_id,"color":"#ff0000"}),
                )],
                None,
            )
            .unwrap();
        let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
        assert_eq!(frame(&candidate, 55), frame(&manual, 55));
        core.discard_draft(&id, &draft.id).unwrap();
        assert_eq!(core.get_project(&id).unwrap().revision, 2);
        let range = |project: &Project| {
            let result = renderer
                .render_preview_range(
                    project,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 1000,
                        width: 512,
                        height: 512,
                        fps: 20,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap();
            (
                decode(&ffmpeg, &dir.join(&result.relative_path), false),
                decode(&ffmpeg, &dir.join(result.relative_path), true),
            )
        };
        assert_eq!(range(&compiled), range(&manual));
        let export = |project: &Project, name: &str| {
            let output = root.path().join(name);
            renderer
                .export_video(
                    project,
                    &dir,
                    ExportOptions {
                        output: &output,
                        width: 512,
                        height: 512,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            decode(&ffmpeg, &output, false)
        };
        assert_eq!(
            export(&compiled, "preset.mp4"),
            export(&manual, "manual.mp4")
        );
        let mut retired = compiled.clone();
        let source = retired.tracks[1].items[0]
            .visual_properties_mut()
            .animation_preset_provenance
            .values_mut()
            .next()
            .unwrap();
        source.preset_id = "retired_tween".into();
        source.preset_version = 77;
        source.compiler_version = 55;
        assert_eq!(frame(&retired, 55), frame(&manual, 55));
        // Same independently authored channel inside a fractional inherited composition clock.
        let nested = |project: &Project| {
            let mut saved = serde_json::to_value(project).unwrap();
            let child = saved["tracks"][1].clone();
            saved["components"] = json!([{"id":"nested","name":"Nested","width":512,"height":512,"durationMs":1000,"tracks":[child],"slots":[],"markers":[]}]);
            saved["tracks"][1]["items"] = json!([{"type":"component_instance","id":"instance","componentId":"nested","startMs":100,"durationMs":500,"trimStartMs":25,"timeScale":0.75,"slotValues":{},"stackOrder":0,"zIndex":0}]);
            serde_json::from_value::<Project>(saved).unwrap()
        };
        for time in [100, 101, 140, 141, 186, 187, 333, 550] {
            assert_eq!(
                frame(&nested(&compiled), time),
                frame(&nested(&manual), time),
                "fractional nested frame {time}"
            );
        }
        assert_eq!(files(&core, &id), authoritative);
    }
}

#[test]
fn motion_pack_partial_collisions_replace_order_and_nonimpact_blur_preservation() {
    let fixture = pack_fixture();
    let pulse = &fixture["presets"][3];
    let (_root, core, project, id) = setup();
    let unrelated = json!({"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]});
    let colliding = json!({"property":"transform.scale_y","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":2.0},"curve":"hold"}]});
    core.edit(&project,1,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[unrelated,colliding]}))).unwrap();
    core.edit(&project,2,op(json!({"operation":"update_item","itemId":id,"motionBlur":{"shutterAngleDeg":90.0,"sampleCount":2}}))).unwrap();
    let before = state(&core, &project);
    let bytes = pack_files(&core, &project);
    assert_eq!(
        core.edit(&project, 3, op(pack_request(&id, pulse)))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(pack_files(&core, &project), bytes);
    let mut replace = pack_request(&id, pulse);
    replace["collisionPolicy"] = json!("replace");
    core.edit(&project, 3, op(replace)).unwrap();
    let after = state(&core, &project);
    let expected = pack_channels(pulse);
    assert_eq!(
        item(&after)["animationChannels"],
        json!([unrelated, expected[1], expected[0]])
    );
    assert_eq!(item(&after)["motionBlur"], item(&before)["motionBlur"]);
    core.undo(&project, 4).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&before));
    core.redo(&project, 5).unwrap();
    assert_eq!(item(&state(&core, &project)), item(&after));
}

#[test]
fn motion_pack_component_replacement_clears_only_blur_dependent_sources() {
    let fixture = pack_fixture();
    let (_root, core, project, id) = setup();
    core.edit(&project, 1, op(pack_request(&id, &fixture["presets"][0])))
        .unwrap();
    let before = state(&core, &project);
    let tracks = json!([before["tracks"][1]]);
    let component=core.edit(&project,2,op(json!({"operation":"component_create","name":"Motion","width":64,"height":64,"durationMs":1000,"tracks":tracks}))).unwrap().changed_ids[0].clone();
    let path = core
        .paths()
        .project_dir(&project)
        .unwrap()
        .join("project.json");
    let mut saved = state(&core, &project);
    let mut sources = item(&before)["animationPresetProvenance"].clone();
    for source in sources.as_object_mut().unwrap().values_mut() {
        source["presetId"] = json!("retired_impact");
        source["compilerVersion"] = json!(99);
    }
    sources["transform.opacity"] = catalog()["examples"]["retiredProvenance"].clone();
    saved["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"] = sources.clone();
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let unchanged = saved["components"][0]["tracks"].clone();
    core.edit(&project,3,op(json!({"operation":"component_update","componentId":component,"name":"Motion","width":64,"height":64,"durationMs":1000,"tracks":unchanged}))).unwrap();
    let known = state(&core, &project);
    assert_eq!(
        known["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"],
        sources
    );
    let mut removed = unchanged;
    removed[0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("motionBlur");
    core.edit(&project,4,op(json!({"operation":"component_update","componentId":component,"name":"Motion","width":64,"height":64,"durationMs":1000,"tracks":removed}))).unwrap();
    let after = state(&core, &project);
    assert_eq!(
        after["components"][0]["tracks"][0]["items"][0]["animationPresetProvenance"],
        json!({"transform.opacity":sources["transform.opacity"]})
    );
    assert_eq!(
        after["components"][0]["tracks"][0]["items"][0]["animationChannels"],
        known["components"][0]["tracks"][0]["items"][0]["animationChannels"]
    );
    core.undo(&project, 5).unwrap();
    assert_eq!(state(&core, &project)["components"], known["components"]);
    core.redo(&project, 6).unwrap();
    assert_eq!(state(&core, &project)["components"], after["components"]);
}

#[test]
fn motion_pack_raster_budget_failure_rolls_back_aliased_prefix_and_existing_draft() {
    let fixture = pack_fixture();
    let (_root, core, project, id) = setup_canvas(4096);
    let mut first = pack_request(&id, &fixture["presets"][0]);
    first["parameters"]["motionBlur"]["sampleCount"] = json!(16);
    core.edit(&project, 1, op(first.clone())).unwrap();
    let draft = core
        .create_draft(
            &project,
            2,
            vec![op(
                json!({"operation":"update_item","itemId":id,"color":"#00ff00"}),
            )],
            None,
        )
        .unwrap();
    let track = core.get_project(&project).unwrap().tracks[1].id.clone();
    let before = pack_files(&core, &project);
    first["itemId"] = json!("@motion");
    let edits:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"update_item","itemId":id,"color":"#0000ff"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"motion"},first])).unwrap();
    let error = core.edit_batch(&project, 2, edits).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("pixel work"), "{}", error.message);
    assert_eq!(pack_files(&core, &project), before);
    assert_eq!(
        core.get_draft(&project, &draft.id)
            .unwrap()
            .operations
            .len(),
        1
    );
}

#[test]
fn motion_pack_groups_instances_duration_edits_copies_and_channel_clear_follow_existing_owners() {
    let fixture = pack_fixture();
    for entry in fixture["presets"].as_array().unwrap() {
        let (_root, core, project, id) = setup();
        core.edit(&project, 1, op(pack_request(&id, entry)))
            .unwrap();
        let saved = state(&core, &project);
        let source = item(&saved)["animationPresetProvenance"].clone();
        core.edit_batch(
            &project,
            2,
            vec![
                op(json!({"operation":"update_item","itemId":id,"color":"#00ff00"})),
                op(json!({"operation":"trim_item","itemId":id,"startMs":0,"durationMs":900})),
            ],
        )
        .unwrap();
        assert_eq!(
            item(&state(&core, &project))["animationPresetProvenance"],
            source
        );
        let before = pack_files(&core, &project);
        assert_eq!(
            core.edit(
                &project,
                3,
                op(json!({"operation":"trim_item","itemId":id,"startMs":0,"durationMs":90}))
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(pack_files(&core, &project), before);
        let copies = core
            .edit(
                &project,
                3,
                op(json!({"operation":"duplicate_items","itemIds":[id],"offsetMs":1500})),
            )
            .unwrap();
        let copy = core.get_project(&project).unwrap();
        assert_eq!(
            serde_json::to_value(
                &copy
                    .find_item(&copies.changed_ids[0])
                    .unwrap()
                    .visual_properties()
                    .animation_preset_provenance
            )
            .unwrap(),
            source
        );
        core.edit(&project,4,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":item(&saved)["animationChannels"]}))).unwrap();
        assert!(
            item(&state(&core, &project))
                .get("animationPresetProvenance")
                .is_none()
        );
        core.undo(&project, 5).unwrap();
        assert_eq!(
            item(&state(&core, &project))["animationPresetProvenance"],
            source
        );
    }
    for entry in fixture["presets"].as_array().unwrap() {
        for instance in [false, true] {
            let (_root, core, project, _id) = setup();
            let track = core.get_project(&project).unwrap().tracks[1].id.clone();
            let target = if instance {
                let component=core.edit(&project,1,op(json!({"operation":"component_create","name":"Empty","width":64,"height":64,"durationMs":1000,"tracks":[]}))).unwrap().changed_ids[0].clone();
                core.edit(&project,2,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1}))).unwrap().changed_ids[0].clone()
            } else {
                core.edit(&project,1,op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone()
            };
            let revision = core.get_project(&project).unwrap().revision;
            if !instance {
                core.edit(
                    &project,
                    revision,
                    op(json!({"operation":"update_item","itemId":target,"transform2d":null})),
                )
                .unwrap();
            }
            let revision = core.get_project(&project).unwrap().revision;
            let before = pack_files(&core, &project);
            let applied = core.edit(&project, revision, op(pack_request(&target, entry)));
            if entry["id"] == "impact_slam" {
                assert_eq!(applied.unwrap_err().code, ErrorCode::InvalidArgument);
                assert_eq!(pack_files(&core, &project), before);
            } else {
                applied.unwrap();
            }
        }
    }
}
