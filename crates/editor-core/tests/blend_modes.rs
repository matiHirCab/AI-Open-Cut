use opencut_editor_core::{
    CoreError, EditOperation, EditorCore, ErrorCode, PathPolicy, Project, ProjectSettings,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../contracts/blend-modes-v1.json")).unwrap()
}

fn operation(value: Value) -> EditOperation {
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
            "Blend models",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let item = core.edit(&id,0,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":24,"height":16,"color":"#ff0000","transform":{"positionX":20,"positionY":20,"scale":1,"opacity":0.5}}))).unwrap().changed_ids[0].clone();
    (root, core, id, item)
}

fn mode(project: &Project, item: &str) -> Value {
    serde_json::to_value(
        project
            .find_item(item)
            .unwrap()
            .visual_properties()
            .blend_mode,
    )
    .unwrap()
}

#[test]
fn canonical_selections_omission_reset_history_and_reopen_use_public_owner() {
    let (_root, core, id, item) = setup();
    for case in fixture()["acceptedSelections"].as_array().unwrap() {
        if let Some(existing) = case.get("existingMode") {
            let p = core.get_project(&id).unwrap();
            core.edit(
                &id,
                p.revision,
                operation(json!({"operation":"update_item","itemId":item,"blendMode":existing})),
            )
            .unwrap();
        }
        let before = core.get_project(&id).unwrap();
        let previous = mode(&before, &item);
        let mut op = case["operation"].clone();
        op["itemId"] = json!(item);
        core.edit(&id, before.revision, operation(op)).unwrap();
        let after = core.get_project(&id).unwrap();
        let expected = case.get("expectedMode").unwrap_or(&case["name"]);
        assert_eq!(mode(&after, &item), *expected, "{}", case["name"]);
        assert_eq!(after.revision, before.revision + 1);
        core.undo(&id, after.revision).unwrap();
        let undone = core.get_project(&id).unwrap();
        assert_eq!(mode(&undone, &item), previous);
        core.redo(&id, undone.revision).unwrap();
        let redone = core.get_project(&id).unwrap();
        assert_eq!(mode(&redone, &item), *expected);
        let reopened = EditorCore::new(core.paths().clone())
            .get_project(&id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap(),
            serde_json::to_value(redone).unwrap()
        );
    }
}

#[test]
fn canonical_malformed_and_raw_duplicate_selections_reject_before_mutation() {
    let (_root, core, id, _item) = setup();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for case in fixture()["rejectedSelections"].as_array().unwrap() {
        let error = serde_json::from_value::<EditOperation>(case["operation"].clone()).unwrap_err();
        assert_eq!(
            CoreError::from(error).code,
            ErrorCode::InvalidArgument,
            "{}",
            case["name"]
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    for case in fixture()["rawDuplicateCases"].as_array().unwrap() {
        assert!(
            serde_json::from_str::<EditOperation>(case["rawOperation"].as_str().unwrap()).is_err()
        );
    }
    // An externally tagged Serde enum object must not broaden the string wire shape.
    assert!(
        serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":"visual","blendMode":{"multiply":null}})
        )
        .is_err()
    );
}

#[test]
fn pre35_raw_normal_null_and_non_normal_presence_reject_before_defaults() {
    let (_root, core, id, _item) = setup();
    let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for value in [json!("normal"), Value::Null, json!("multiply")] {
        let mut old = current.clone();
        old["schemaVersion"] = json!(34);
        old["tracks"][1]["items"][0]["blendMode"] = value;
        let error = serde_json::from_value::<Project>(old).unwrap_err();
        assert_eq!(CoreError::from(error).code, ErrorCode::InvalidArgument);
    }
    let mut old = current;
    old["schemaVersion"] = json!(34);
    let restored = serde_json::from_value::<Project>(old).unwrap();
    assert!(
        restored.tracks[1].items[0]
            .visual_properties()
            .blend_mode
            .is_normal()
    );
}

#[test]
fn missing_target_and_stale_revision_preserve_stable_errors_and_state() {
    let (_root, core, id, item) = setup();
    let before = core.get_project(&id).unwrap();
    let missing = core
        .edit(
            &id,
            before.revision,
            operation(fixture()["missingTargetCase"]["operation"].clone()),
        )
        .unwrap_err();
    assert_eq!(missing.code, ErrorCode::ItemNotFound);
    assert!(!missing.retryable);
    let stale = core
        .edit(
            &id,
            before.revision - 1,
            operation(json!({"operation":"update_item","itemId":item,"blendMode":"multiply"})),
        )
        .unwrap_err();
    assert_eq!(stale.code, ErrorCode::RevisionConflict);
    assert!(stale.retryable);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}

fn durable_bytes(dir: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        dir: &std::path::Path,
        files: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files)
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = Default::default();
    visit(dir, dir, &mut files);
    files
}
fn legacy34(dir: &std::path::Path) {
    fn downgrade(value: &mut Value) {
        match value {
            Value::Object(map) => {
                if map.contains_key("schemaVersion") {
                    map.insert("schemaVersion".into(), json!(34));
                }
                for child in map.values_mut() {
                    downgrade(child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    downgrade(child);
                }
            }
            _ => {}
        }
    }
    for name in ["project.json", "history.json"] {
        let path = dir.join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        downgrade(&mut value);
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
}
#[test]
fn component_explicit_normal_failed_batch_and_draft_do_not_adopt_legacy_generation() {
    for update in [false, true] {
        for draft in [false, true] {
            let (_root, core, id, _) = setup();
            let mut component = json!({"operation":"component_create","name":"Normal payload","width":64,"height":64,"durationMs":1000,"slots":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"rectangle","id":"local-rect","color":"#ffffff","width":8,"height":8,"startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0,"keyframes":[],"blendMode":"normal"}]}]});
            if update {
                let created = core.edit(&id, 1, operation(component.clone())).unwrap();
                component["operation"] = json!("component_update");
                component["componentId"] = json!(created.changed_ids[0]);
            }
            let revision = core.get_project(&id).unwrap().revision;
            let dir = core.paths().project_dir(&id).unwrap();
            legacy34(&dir);
            let before = durable_bytes(&dir);
            let operations = vec![
                operation(component),
                operation(json!({"operation":"delete_item","itemId":"missing"})),
            ];
            let error = if draft {
                core.create_draft(&id, revision, operations, None)
                    .unwrap_err()
            } else {
                core.edit_batch(&id, revision, operations).unwrap_err()
            };
            assert_eq!(error.code, ErrorCode::ItemNotFound);
            assert!(!error.retryable);
            assert_eq!(durable_bytes(&dir), before, "update={update},draft={draft}");
        }
    }
}
#[test]
fn current_and_retained_absence_migrate_once_and_pre35_history_presence_rejects_atomically() {
    for premature in [
        None,
        Some(json!("normal")),
        Some(Value::Null),
        Some(json!("screen")),
    ] {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        legacy34(&dir);
        if let Some(value) = premature {
            let path = dir.join("history.json");
            let mut history: Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            // Keep a real retained generation, not a fabricated current fallback.
            history["undo"][0]["tracks"][1]["items"] = json!([{ "type":"rectangle","id":"retained","color":"#ffffff","width":8,"height":8,"startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0,"keyframes":[],"blendMode":value}]);
            std::fs::write(path, serde_json::to_vec_pretty(&history).unwrap()).unwrap();
            let before = durable_bytes(&dir);
            assert_eq!(
                core.get_project(&id).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(durable_bytes(&dir), before);
        } else {
            let migrated = core.get_project(&id).unwrap();
            assert_eq!(
                migrated.schema_version,
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
            let adopted = durable_bytes(&dir);
            assert_eq!(core.get_project(&id).unwrap().revision, migrated.revision);
            assert_eq!(durable_bytes(&dir), adopted);
            core.undo(&id, migrated.revision).unwrap();
            assert_eq!(
                core.get_project(&id).unwrap().schema_version,
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
        }
    }
}

#[test]
fn stale_and_unavailable_blend_drafts_never_replay_against_current_and_remain_discardable() {
    for available in [true, false] {
        let (_root, core, id, item) = setup();
        let base = core.get_project(&id).unwrap();
        let draft = core
            .create_draft(
                &id,
                base.revision,
                vec![operation(
                    json!({"operation":"update_item","itemId":item,"blendMode":"multiply"}),
                )],
                None,
            )
            .unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let path = dir.join("drafts").join(format!("{}.json", draft.id));
        let revision = if available {
            core.edit(
                &id,
                base.revision,
                operation(json!({"operation":"delete_item","itemId":item})),
            )
            .unwrap()
            .revision
        } else {
            let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            raw["baseRevision"] = json!(999);
            std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
            base.revision
        };
        let before = durable_bytes(&dir);
        core.get_project(&id).unwrap();
        core.get_draft(&id, &draft.id).unwrap();
        assert_eq!(durable_bytes(&dir), before);
        for error in [
            core.get_draft_state(&id, &draft.id).unwrap_err(),
            core.commit_draft(&id, &draft.id, revision).unwrap_err(),
        ] {
            assert_eq!(error.code, ErrorCode::RevisionConflict);
            assert!(error.retryable);
        }
        assert_eq!(durable_bytes(&dir), before);
        let track = core.get_project(&id).unwrap().tracks[1].id.clone();
        core.edit(&id,revision,operation(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":8,"height":8,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            before[&path.strip_prefix(&dir).unwrap().to_path_buf()]
        );
        core.discard_draft(&id, &draft.id).unwrap();
        assert!(!path.exists());
    }
}
#[test]
fn legacy_template_discriminator_remains_strictly_unsupported_for_all_blend_selections() {
    for case in fixture()["unsupportedDiscriminatorCases"]
        .as_array()
        .unwrap()
    {
        let mut item = json!({"type":case["itemKind"],"id":"unsupported"});
        if !case["blendMode"].is_null() {
            item["blendMode"] = case["blendMode"].clone();
        }
        assert!(serde_json::from_value::<opencut_editor_core::TimelineItem>(item).is_err());
    }
}

#[test]
fn actual_ineligible_kinds_accept_normal_and_reject_non_normal_with_complete_batch_and_draft_rollback()
 {
    let (root, core, id, leaf) = setup();
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let edit = |value: Value| {
        core.edit(
            &id,
            core.get_project(&id).unwrap().revision,
            operation(value),
        )
        .unwrap()
        .changed_ids[0]
            .clone()
    };
    let group = edit(
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"transform2d":opencut_editor_core::Transform2D::default()}),
    );
    let shape = edit(
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":8,"height":8},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}),
    );
    let repeater = edit(
        json!({"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":shape},"copies":1,"timeOffsetMs":0,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}),
    );
    let local =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&leaf).unwrap()).unwrap();
    let component = edit(
        json!({"operation":"component_create","name":"Eligible leaf","width":64,"height":64,"durationMs":1000,"slots":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[local]}]}),
    );
    let instance = edit(
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{}}),
    );
    let transition = edit(
        json!({"operation":"add_transition","trackId":track,"fromItemId":leaf,"transitionType":"fade","startMs":0,"durationMs":100}),
    );
    let wav = root.path().join("media/audio.wav");
    let mut bytes = b"RIFF".to_vec();
    bytes.extend((36_u32 + 96000).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(96000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(96000_u32.to_le_bytes());
    bytes.extend(vec![0; 96000]);
    std::fs::write(&wav, bytes).unwrap();
    let asset = core
        .import_asset(
            &id,
            core.get_project(&id).unwrap().revision,
            &wav,
            opencut_editor_core::MediaType::Audio,
            opencut_editor_core::MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    let audio = edit(
        json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}),
    );
    let ids = std::collections::BTreeMap::from([
        ("audio", audio),
        ("group", group),
        ("repeater", repeater),
        ("component_instance", instance),
        ("transition", transition),
    ]);
    let dir = core.paths().project_dir(&id).unwrap();
    for case in fixture()["defaultUnsupportedCases"].as_array().unwrap() {
        let kind = case["itemKind"].as_str().unwrap();
        let item = &ids[kind];
        let mut value = case["operation"].clone();
        value["itemId"] = json!(item);
        edit(value);
        assert!(
            core.get_project(&id)
                .unwrap()
                .find_item(item)
                .unwrap()
                .visual_properties()
                .blend_mode
                .is_normal()
        );
    }
    for case in fixture()["unsupportedSelectionCases"].as_array().unwrap() {
        let kind = case["itemKind"].as_str().unwrap();
        let mut value = case["operation"].clone();
        value["itemId"] = json!(ids[kind]);
        let before = durable_bytes(&dir);
        let revision = core.get_project(&id).unwrap().revision;
        let first =
            operation(json!({"operation":"update_item","itemId":leaf,"blendMode":"screen"}));
        for error in [
            core.edit(&id, revision, operation(value.clone()))
                .unwrap_err(),
            core.edit_batch(&id, revision, vec![first.clone(), operation(value.clone())])
                .unwrap_err(),
            core.create_draft(&id, revision, vec![first, operation(value)], None)
                .unwrap_err(),
        ] {
            assert_eq!(error.code, ErrorCode::InvalidArgument, "{kind}");
            assert!(!error.retryable);
        }
        assert_eq!(
            durable_bytes(&dir),
            before,
            "{kind} complete resources/history/drafts unchanged"
        );
    }
}

#[test]
fn pre35_draft_presence_is_guarded_against_own_current_or_retained_base_before_adoption() {
    raw_draft_own_base_guards(false);
}
#[test]
fn pre35_component_payload_draft_presence_is_guarded_before_typed_normal_omission() {
    raw_draft_own_base_guards(true);
}
fn raw_draft_own_base_guards(component: bool) {
    for retained in [false, true] {
        for value in [json!("normal"), json!("multiply"), Value::Null] {
            let (_root, core, id, item) = setup();
            let base = core.get_project(&id).unwrap();
            let draft = core
                .create_draft(
                    &id,
                    base.revision,
                    vec![operation(
                        json!({"operation":"update_item","itemId":item,"blendMode":"normal"}),
                    )],
                    None,
                )
                .unwrap();
            if retained {
                core.edit(
                    &id,
                    base.revision,
                    operation(json!({"operation":"update_item","itemId":item,"color":"#00ff00"})),
                )
                .unwrap();
            }
            let dir = core.paths().project_dir(&id).unwrap();
            if retained {
                let path = dir.join("history.json");
                let mut raw: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                let source = raw["undo"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|source| source["revision"] == base.revision)
                    .unwrap();
                source["schemaVersion"] = json!(34);
                std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
            } else {
                legacy34(&dir);
            }
            let path = dir.join("drafts").join(format!("{}.json", draft.id));
            let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if component {
                let mut local = serde_json::to_value(base.find_item(&item).unwrap()).unwrap();
                local["id"] = json!("local");
                local["stackOrder"] = json!(0);
                local["blendMode"] = value;
                raw["operations"][0] = json!({"operation":"component_create","name":"Raw component","width":64,"height":64,"durationMs":1000,"slots":[],"tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[local]}]});
            } else {
                raw["operations"][0]["blendMode"] = value;
            }
            std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
            let before = durable_bytes(&dir);
            let error = EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "retained={retained}: {error:?}"
            );
            assert_eq!(durable_bytes(&dir), before);
        }
    }
}
