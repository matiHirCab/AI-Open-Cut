use opencut_editor_core as audit_core;
use opencut_editor_core::{EditorCore, ErrorCode, PROJECT_SCHEMA_VERSION};
use serde_json::{Value, json};
#[path = "historical_animation_seed.rs"]
mod historical_animation_seed;
use historical_animation_seed::{historical, inventory, seed, slot_generation, write};
#[test]
fn historical_slot_collisions_migrate_complete_generations_and_restore_history() {
    for version in [21, 23, 25, 26, 27] {
        for historical_current in [true, false] {
            let (_root, core, id, current) = seed();
            let undo = slot_generation(current.clone(), 0.2);
            let redo = slot_generation(current.clone(), 0.9);
            let dir = core.paths().project_dir(&id).unwrap();
            write(
                &dir,
                &historical(
                    current.clone(),
                    if historical_current { version } else { 31 },
                ),
                &historical(undo.clone(), version),
                &historical(redo.clone(), version),
            );
            let reopened = EditorCore::new(core.paths().clone());
            let actual = reopened.get_project(&id).unwrap();
            let actual = serde_json::to_value(actual).unwrap();
            assert_eq!(
                actual, current,
                "version={version} historical_current={historical_current}"
            );
            let history: Value =
                serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
            assert_eq!(history, json!({"undo":[undo],"redo":[redo]}));
            let before = inventory(&dir);
            reopened.get_project(&id).unwrap();
            assert_eq!(inventory(&dir), before);
            reopened
                .undo(&id, actual["revision"].as_u64().unwrap())
                .unwrap();
            let restored = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
            assert_eq!(restored["tracks"], undo["tracks"]);
            assert_eq!(restored["components"], undo["components"]);
            reopened
                .redo(&id, restored["revision"].as_u64().unwrap())
                .unwrap();
            let restored = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
            assert_eq!(restored["tracks"], current["tracks"]);
            assert_eq!(restored["components"], current["components"]);
            assert_eq!(restored["schemaVersion"], PROJECT_SCHEMA_VERSION);
        }
    }
}
#[test]
fn actual_premature_fields_reject_by_presence_in_every_generation() {
    for (version, field, message) in [
        (21, "animationChannels", "schema 22"),
        (23, "startTime", "schema 24"),
        (25, "staggerMs", "schema 26"),
        (25, "timeOffsetMs", "schema 26"),
        (26, "crop", "schema 27"),
        (26, "effects", "schema 27"),
        (27, "motionBlur", "schema 28"),
    ] {
        for component in [false, true] {
            for location in ["current", "undo", "redo"] {
                for presence in [json!(0), Value::Null, json!([])] {
                    let (_root, core, id, current) = seed();
                    let dir = core.paths().project_dir(&id).unwrap();
                    let mut bad = historical(current.clone(), version);
                    let item = if component {
                        &mut bad["components"][0]["tracks"][0]["items"][0]
                    } else {
                        &mut bad["tracks"][1]["items"][0]
                    };
                    if field == "timeOffsetMs" {
                        item["repeater"] = json!({"timeOffsetMs":presence});
                    } else {
                        item[field] = presence;
                    }
                    let source = if location == "current" {
                        &bad
                    } else {
                        &current
                    };
                    write(
                        &dir,
                        source,
                        if location == "undo" { &bad } else { &current },
                        if location == "redo" { &bad } else { &current },
                    );
                    let before = inventory(&dir);
                    let error = EditorCore::new(core.paths().clone())
                        .get_project(&id)
                        .unwrap_err();
                    assert_eq!(error.code, ErrorCode::InternalError);
                    assert!(
                        error.message.contains(message),
                        "{field} {component} {location}: {}",
                        error.message
                    );
                    assert_eq!(inventory(&dir), before);
                }
            }
        }
    }
}
#[test]
fn actual_extended_channel_properties_and_other_source_guards_remain_closed() {
    for component in [false, true] {
        for (version, field, value, message) in [
            (
                26,
                "animationChannels",
                json!([{"property":"transform.rotation_deg","keyframes":[]}]),
                "extended animation requires schema 27",
            ),
            (
                22,
                "animationChannels",
                json!([{"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":{"type":"spring","mass":1,"stiffness":100,"damping":2,"initialVelocity":0}}]}]),
                "schema 23",
            ),
            (
                24,
                "animationChannels",
                json!([{"property":"transform.position_x","loop":null,"keyframes":[]}]),
                "schema 25",
            ),
            (
                30,
                "legacyAnimationClock",
                json!({"offsetMs":0,"sourceDurationMs":1000}),
                "schema 31",
            ),
            (28, "animationPresetProvenance", json!({}), "schema 29"),
            (
                29,
                "animationPresetProvenance",
                json!({"transform.position_x":{"parameters":{"kind":"builtin"}}}),
                "schema 30",
            ),
        ] {
            let (_root, core, id, current) = seed();
            let dir = core.paths().project_dir(&id).unwrap();
            let mut bad = historical(current.clone(), version);
            let item = if component {
                &mut bad["components"][0]["tracks"][0]["items"][0]
            } else {
                &mut bad["tracks"][1]["items"][0]
            };
            item[field] = value;
            write(&dir, &current, &bad, &current);
            let before = inventory(&dir);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(error.code, ErrorCode::InternalError);
            assert!(error.message.contains(message), "{}", error.message);
            assert_eq!(inventory(&dir), before);
        }
    }
}

#[test]
fn closed_persisted_marker_variants_fail_before_document_rewrite_or_journal_replay() {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../../contracts/motion-graphics-v1.json"
    ))
    .unwrap();
    for fixture in catalog["timeExpressionCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["accept"] == false && v["value"]["type"] == "marker")
    {
        for component in [false, true] {
            for location in ["current", "undo", "redo"] {
                for journal in [false, true] {
                    let (_root, core, id, current) = seed();
                    let dir = core.paths().project_dir(&id).unwrap();
                    let mut bad = current.clone();
                    let item = if component {
                        &mut bad["components"][0]["tracks"][0]["items"][0]
                    } else {
                        &mut bad["tracks"][1]["items"][0]
                    };
                    item["startTime"] = fixture["value"].clone();
                    let source = if location == "current" {
                        &bad
                    } else {
                        &current
                    };
                    let undo = if location == "undo" { &bad } else { &current };
                    let redo = if location == "redo" { &bad } else { &current };
                    if journal {
                        std::fs::write(dir.join(".project-transaction.json"),serde_json::to_vec(&json!({"version":1,"project":source,"history":{"undo":[undo],"redo":[redo]},"committedDraftId":null})).unwrap()).unwrap();
                    } else {
                        write(&dir, source, undo, redo);
                    }
                    let before = inventory(&dir);
                    let error = core.get_project(&id).unwrap_err();
                    assert_eq!(
                        error.code,
                        if journal {
                            ErrorCode::ProjectRecoveryFailed
                        } else {
                            ErrorCode::InternalError
                        },
                        "{} component={component} location={location} journal={journal}: {}",
                        fixture["id"],
                        error.message
                    );
                    assert!(error.message.contains("unknown field"), "{}", error.message);
                    assert!(!error.retryable);
                    assert_eq!(inventory(&dir), before);
                }
            }
        }
    }
}
#[test]
fn legal_colliding_slots_recover_committed_historical_generations_before_atomic_migration() {
    for version in [21, 23, 25, 26, 27] {
        let (_root, core, id, current) = seed();
        let dir = core.paths().project_dir(&id).unwrap();
        let generation = slot_generation(current.clone(), 0.6);
        let undo = slot_generation(current.clone(), 0.4);
        let redo = slot_generation(current, 0.8);
        std::fs::write(dir.join(".project-transaction.json"),serde_json::to_vec(&json!({"version":1,"project":historical(generation.clone(),version),"history":{"undo":[historical(undo.clone(),version)],"redo":[historical(redo.clone(),version)]},"committedDraftId":null})).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            generation
        );
        let history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        assert_eq!(history, json!({"undo":[undo],"redo":[redo]}));
        assert!(!dir.join(".project-transaction.json").exists());
        let before = inventory(&dir);
        core.get_project(&id).unwrap();
        assert_eq!(inventory(&dir), before);
    }
}

#[test]
fn colliding_dictionary_names_do_not_bypass_slot_types_or_future_schema() {
    for component in [false, true] {
        for location in ["current", "undo", "redo"] {
            for future in [false, true] {
                let (_root, core, id, current) = seed();
                let dir = core.paths().project_dir(&id).unwrap();
                let mut bad = historical(
                    current.clone(),
                    if future {
                        PROJECT_SCHEMA_VERSION + 1
                    } else {
                        21
                    },
                );
                if !future {
                    let item = if component {
                        &mut bad["components"][1]["tracks"][0]["items"][0]
                    } else {
                        &mut bad["tracks"][1]["items"][0]
                    };
                    item["slotValues"]["animationChannels"]["value"] = json!("not a number");
                }
                write(
                    &dir,
                    if location == "current" {
                        &bad
                    } else {
                        &current
                    },
                    if location == "undo" { &bad } else { &current },
                    if location == "redo" { &bad } else { &current },
                );
                let before = inventory(&dir);
                let error = core.get_project(&id).unwrap_err();
                assert_eq!(error.code, ErrorCode::InternalError);
                assert!(!error.retryable);
                assert_eq!(inventory(&dir), before);
            }
        }
    }
}

fn persist_draft_operations(core: &EditorCore, id: &str, current: &Value, operations: Value) {
    let item = current["tracks"][1]["items"][0]["id"].as_str().unwrap();
    let draft = core
        .create_draft(
            id,
            current["revision"].as_u64().unwrap(),
            vec![historical_animation_seed::op(
                json!({"operation":"update_item","itemId":item,"hidden":false}),
            )],
            None,
        )
        .unwrap();
    let path = core
        .paths()
        .project_dir(id)
        .unwrap()
        .join("drafts")
        .join(format!("{}.json", draft.id));
    let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["operations"] = operations;
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

fn rotation_channel() -> Value {
    json!({"property":"transform.rotation_deg","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"hold"}]})
}

#[test]
fn typed_draft_owners_reject_premature_fields_without_scanning_slot_dictionaries() {
    for version in [26, 27] {
        for owner in ["update", "create", "component_update", "channels"] {
            if version == 27 && owner == "channels" {
                continue;
            }
            let (_root, core, id, current) = seed();
            let item = &current["tracks"][1]["items"][0]["id"];
            let mut operation = match owner {
                "update" => json!({"operation":"update_item","itemId":item}),
                "channels" => {
                    json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[rotation_channel()]})
                }
                _ => {
                    json!({"operation":if owner=="create" {"component_create"} else {"component_update"},"componentId":current["components"][0]["id"],"name":"Draft leaf","width":64,"height":64,"durationMs":1000,"tracks":current["components"][0]["tracks"],"slots":current["components"][0]["slots"]})
                }
            };
            if owner == "create" {
                operation.as_object_mut().unwrap().remove("componentId");
            }
            if owner != "channels" {
                let envelope = if owner == "update" {
                    &mut operation
                } else {
                    &mut operation["tracks"][0]["items"][0]
                };
                if version == 27 {
                    envelope["motionBlur"] = json!({"shutterAngleDeg":180,"sampleCount":4});
                } else if owner == "update" {
                    envelope["effects"] = json!([]);
                } else {
                    envelope["animationChannels"] = json!([rotation_channel()]);
                }
            }
            // The persisted operation must itself decode; the historical owner
            // guard, rather than an unrelated shape error, rejects it on open.
            historical_animation_seed::op(operation.clone());
            persist_draft_operations(&core, &id, &current, json!([operation]));
            let dir = core.paths().project_dir(&id).unwrap();
            let old = historical(current.clone(), version);
            write(&dir, &old, &old, &old);
            let before = inventory(&dir);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "{version} {owner}: {error:?}"
            );
            assert!(
                error.message.contains(if version == 27 {
                    "schema 28"
                } else {
                    "schema 27"
                }),
                "{error:?}"
            );
            assert_eq!(inventory(&dir), before);
        }
        for owner in ["component_create", "component_update"] {
            let (_root, core, id, current) = seed();
            let mut operation = json!({"operation":owner,"componentId":current["components"][1]["id"],"name":"Draft outer","width":64,"height":64,"durationMs":1000,"tracks":current["components"][1]["tracks"]});
            if owner == "component_create" {
                operation.as_object_mut().unwrap().remove("componentId");
            }
            persist_draft_operations(&core, &id, &current, json!([operation]));
            let dir = core.paths().project_dir(&id).unwrap();
            let old = historical(current.clone(), version);
            write(&dir, &old, &old, &old);
            assert_eq!(
                core.get_project(&id).unwrap().schema_version,
                PROJECT_SCHEMA_VERSION
            );
            let before = inventory(&dir);
            core.get_project(&id).unwrap();
            assert_eq!(inventory(&dir), before);
        }
    }
}

#[test]
fn current_draft_candidates_and_component_only_extended_bases_require_replay_validation() {
    for extended_base in [false, true] {
        let (_root, core, id, mut current) = seed();
        let mut operation =
            json!({"operation":"update_item","itemId":"missing-draft-target","hidden":false});
        if extended_base {
            current["components"][0]["tracks"][0]["items"][0]["animationChannels"] =
                json!([rotation_channel()]);
        } else {
            // An explicitly present empty list still belongs to the extended
            // UpdateItem envelope, unlike a dictionary entry named effects.
            operation["effects"] = json!([]);
        }
        persist_draft_operations(&core, &id, &current, json!([operation]));
        let dir = core.paths().project_dir(&id).unwrap();
        write(&dir, &current, &current, &current);
        let before = inventory(&dir);
        let error = core.get_project(&id).unwrap_err();
        assert_eq!(
            error.code,
            ErrorCode::ItemNotFound,
            "extended_base={extended_base}: {error:?}"
        );
        assert_eq!(inventory(&dir), before);
    }
}
