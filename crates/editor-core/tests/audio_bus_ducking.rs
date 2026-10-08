use opencut_editor_core::{EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/audio-bus-ducking-v1.json")).unwrap()
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
        .create_project("Ducking", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
}

#[test]
fn typed_bus_ducking_round_trips_closed_independent_catalog() {
    let value = catalog()["input"].clone();
    let op: EditOperation = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(op).unwrap(), value);
    for key in ["enabled", "sourceBusId", "gain", "attackMs", "releaseMs"] {
        let mut missing = value.clone();
        missing["ducking"].as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<EditOperation>(missing).is_err());
    }
    for malformed in [json!(null), json!({"rawFilter":"volume=2"})] {
        let mut invalid = value.clone();
        invalid["ducking"] = malformed;
        assert!(serde_json::from_value::<EditOperation>(invalid).is_err());
    }
    let mut unknown = value;
    unknown["ducking"]["rawFilter"] = json!("volume=2");
    assert!(serde_json::from_value::<EditOperation>(unknown).is_err());
}

#[test]
fn bus_ducking_edits_retain_identity_and_history() {
    let (_root, core, id) = setup();
    let initial = core.get_project(&id).unwrap();
    assert_eq!(initial.schema_version, 43);
    assert!(
        serde_json::to_value(&initial).unwrap()["audioBuses"][0]
            .get("ducking")
            .is_none()
    );
    let op = serde_json::from_value(catalog()["input"].clone()).unwrap();
    core.edit(&id, initial.revision, op).unwrap();
    let changed = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&changed).unwrap()["audioBuses"][1]["ducking"],
        catalog()["input"]["ducking"]
    );
    core.undo(&id, changed.revision).unwrap();
    let undone = core.get_project(&id).unwrap();
    assert!(
        serde_json::to_value(&undone).unwrap()["audioBuses"][1]
            .get("ducking")
            .is_none()
    );
    core.redo(&id, undone.revision).unwrap();
    let redone = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&redone).unwrap()["audioBuses"][1]["ducking"],
        catalog()["input"]["ducking"]
    );
    let identity = json!({"operation":"audio_bus_set_ducking","busId":"music","ducking":catalog()["identity"]});
    core.edit(
        &id,
        redone.revision,
        serde_json::from_value(identity).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["audioBuses"][1]["ducking"],
        catalog()["identity"]
    );
}

#[test]
fn invalid_ducking_candidate_is_atomic_and_revision_conflict_takes_precedence() {
    let (_root, core, id) = setup();
    let initial = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for (key, invalid) in [
        ("gain", json!(-0.01)),
        ("gain", json!(1.01)),
        ("attackMs", json!(2001)),
        ("releaseMs", json!(9001)),
        ("sourceBusId", json!("music")),
        ("sourceBusId", json!("missing")),
    ] {
        let mut op = catalog()["input"].clone();
        op["ducking"][key] = invalid;
        let typed: EditOperation = serde_json::from_value(op).unwrap();
        assert_eq!(
            core.edit(&id, 99, typed.clone()).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            core.edit(&id, 0, typed).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            initial
        );
    }
}

fn inventory(path: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(inventory(&path));
        } else {
            files.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    files
}

#[test]
fn schema43_adopts_all42_sources_and_retained_history_without_inventing_ducking() {
    let (root, core, id) = setup();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = root.path().join("projects").join(&id);
    let mut sources = vec![];
    for version in 1..=42 {
        let mut source = original.clone();
        source["schemaVersion"] = json!(version);
        if version < 40 {
            source.as_object_mut().unwrap().remove("soundDefinitions");
        }
        if version < 39 {
            source.as_object_mut().unwrap().remove("audioBuses");
        }
        if version < 24 {
            source.as_object_mut().unwrap().remove("markers");
        }
        if version < 19 {
            source.as_object_mut().unwrap().remove("fonts");
        }
        sources.push(source);
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&sources[41]).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":sources,"redo":sources})).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 43);
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for list in ["undo", "redo"] {
        for (index, snapshot) in history[list].as_array().unwrap().iter().enumerate() {
            assert_eq!(snapshot["schemaVersion"], 43);
            assert_eq!(snapshot["assets"], sources[index]["assets"]);
            assert_eq!(snapshot["tracks"], sources[index]["tracks"]);
            assert!(
                snapshot["audioBuses"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|b| b.get("ducking").is_none())
            );
        }
    }
    let before = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn premature_null_malformed_retained_and_future_ducking_fail_without_any_rewrite() {
    for case in [
        "premature",
        "premature_null",
        "null",
        "unknown",
        "retained",
        "future",
    ] {
        let (root, core, id) = setup();
        let dir = root.path().join("projects").join(&id);
        let mut current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        match case {
            "premature" => {
                current["schemaVersion"] = json!(42);
                current["audioBuses"][1]["ducking"] = catalog()["identity"].clone();
            }
            "premature_null" => {
                current["schemaVersion"] = json!(42);
                current["audioBuses"][1]["ducking"] = Value::Null;
            }
            "null" => current["audioBuses"][1]["ducking"] = Value::Null,
            "unknown" => {
                current["audioBuses"][1]["ducking"] = catalog()["identity"].clone();
                current["audioBuses"][1]["ducking"]["rawFilter"] = json!("volume=2");
            }
            "retained" => {
                let mut bad = current.clone();
                bad["audioBuses"][1]["ducking"] = catalog()["identity"].clone();
                bad["audioBuses"][1]["ducking"]["gain"] = json!(1.01);
                std::fs::write(
                    dir.join("history.json"),
                    serde_json::to_vec_pretty(&json!({"undo":[bad],"redo":[]})).unwrap(),
                )
                .unwrap();
            }
            "future" => current["schemaVersion"] = json!(44),
            _ => unreachable!(),
        }
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec_pretty(&current).unwrap(),
        )
        .unwrap();
        let before = inventory(root.path());
        assert!(core.get_project(&id).is_err(), "{case}");
        assert_eq!(inventory(root.path()), before, "{case}");
    }
}

#[test]
fn batch_alias_and_failed_legacy_draft_edits_preserve_every_source_byte() {
    let (root, core, id) = setup();
    let valid: EditOperation = serde_json::from_value(catalog()["input"].clone()).unwrap();
    let retained = core
        .create_draft(&id, 0, vec![valid.clone()], None)
        .unwrap();
    let dir = root.path().join("projects").join(&id);
    let mut source = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    source["schemaVersion"] = json!(42);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&source).unwrap(),
    )
    .unwrap();
    let before = inventory(root.path());
    let alias = opencut_editor_core::BatchEditOperation {
        edit: valid.clone(),
        result_alias: Some("bus".into()),
    };
    assert_eq!(
        core.edit_batch(&id, 0, vec![alias]).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    let mut bad = catalog()["input"].clone();
    bad["busId"] = json!("missing");
    assert_eq!(
        core.edit_batch(
            &id,
            0,
            vec![valid.clone(), serde_json::from_value(bad.clone()).unwrap()]
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    assert_eq!(
        core.create_draft(&id, 0, vec![serde_json::from_value(bad).unwrap()], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    assert!(core.get_draft(&id, &retained.id).is_ok());
    core.commit_draft(&id, &retained.id, 0).unwrap();
    let current = core.get_project(&id).unwrap();
    assert_eq!(current.schema_version, 43);
    assert_eq!(current.revision, 1);
    assert_eq!(
        serde_json::to_value(&current.audio_buses[1].ducking).unwrap(),
        catalog()["input"]["ducking"]
    );
}

#[test]
fn nonfinite_rust_gain_and_integer_input_guards_are_fail_closed() {
    let (root, core, id) = setup();
    let before = inventory(root.path());
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut ducking: opencut_editor_core::AudioBusDucking =
            serde_json::from_value(catalog()["identity"].clone()).unwrap();
        ducking.gain = value;
        assert_eq!(
            core.edit(
                &id,
                0,
                EditOperation::AudioBusSetDucking {
                    bus_id: "music".into(),
                    ducking
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(inventory(root.path()), before);
    }
    for (key, value) in [
        ("attackMs", json!(-1)),
        ("releaseMs", json!(-1)),
        ("attackMs", json!(0.5)),
        ("releaseMs", json!(0.5)),
    ] {
        let mut value_op = catalog()["input"].clone();
        value_op["ducking"][key] = value;
        assert!(serde_json::from_value::<EditOperation>(value_op).is_err());
    }
}

#[test]
fn schema42_existing_dsp_and_role_settings_migrate_byte_semantics_in_every_snapshot() {
    let (root, core, id) = setup();
    let dir = root.path().join("projects").join(&id);
    let mut source = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    source["schemaVersion"] = json!(42);
    source["audioBuses"][1]["dsp"] = json!({"gainDb":-6.0,"pan":0.5,"eq":[{"frequencyHz":440.0,"q":1.0,"gainDb":6.0}],"compressor":null});
    source["tracks"][0]["audioRole"] = json!("music");
    source["tracks"][0]["ducking"] =
        json!({"enabled":true,"gain":0.5,"attackMs":100,"releaseMs":200});
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&source).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":[source.clone()],"redo":[source.clone()]}))
            .unwrap(),
    )
    .unwrap();
    let mut expected = source.clone();
    expected["schemaVersion"] = json!(43);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        expected
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(
        history,
        json!({"undo":[expected.clone()],"redo":[expected]})
    );
    let stable = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), stable);
}
