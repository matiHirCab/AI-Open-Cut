use opencut_editor_core::{EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/audio-bus-dsp-v1.json")).unwrap()
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
        .create_project("DSP", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
}

#[test]
fn typed_bus_dsp_round_trips_closed_independent_catalog() {
    let value = catalog()["input"].clone();
    let op: EditOperation = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(op).unwrap(), value);
    for key in ["gainDb", "pan", "eq", "compressor"] {
        let mut missing = value.clone();
        missing["dsp"].as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<EditOperation>(missing).is_err());
    }
    for malformed in [json!(null), json!({"rawFilter":"volume=2"})] {
        let mut invalid = value.clone();
        invalid["dsp"] = malformed;
        assert!(serde_json::from_value::<EditOperation>(invalid).is_err());
    }
    let mut unknown = value;
    unknown["dsp"]["rawFilter"] = json!("volume=2");
    assert!(serde_json::from_value::<EditOperation>(unknown).is_err());
}

#[test]
fn bus_dsp_edits_retain_identity_and_history() {
    let (_root, core, id) = setup();
    let initial = core.get_project(&id).unwrap();
    assert_eq!(initial.schema_version, 42);
    assert!(
        serde_json::to_value(&initial).unwrap()["audioBuses"][0]
            .get("dsp")
            .is_none()
    );
    let op = serde_json::from_value(catalog()["input"].clone()).unwrap();
    core.edit(&id, initial.revision, op).unwrap();
    let changed = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&changed).unwrap()["audioBuses"][1]["dsp"],
        catalog()["input"]["dsp"]
    );
    core.undo(&id, changed.revision).unwrap();
    let undone = core.get_project(&id).unwrap();
    assert!(
        serde_json::to_value(&undone).unwrap()["audioBuses"][1]
            .get("dsp")
            .is_none()
    );
    core.redo(&id, undone.revision).unwrap();
    let redone = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&redone).unwrap()["audioBuses"][1]["dsp"],
        catalog()["input"]["dsp"]
    );
    let identity =
        json!({"operation":"audio_bus_set_dsp","busId":"music","dsp":catalog()["identity"]});
    core.edit(
        &id,
        redone.revision,
        serde_json::from_value(identity).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["audioBuses"][1]["dsp"],
        catalog()["identity"]
    );
}

#[test]
fn invalid_dsp_candidate_is_atomic_and_revision_conflict_takes_precedence() {
    let (_root, core, id) = setup();
    let initial = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for (key, invalid) in [
        ("gainDb", json!(24.01)),
        ("gainDb", json!(-120.01)),
        ("pan", json!(1.01)),
        ("pan", json!(-1.01)),
        (
            "eq",
            json!(vec![json!({"frequencyHz":1000,"q":1,"gainDb":0}); 9]),
        ),
    ] {
        let mut op = catalog()["input"].clone();
        op["dsp"][key] = invalid;
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
fn every_compressor_and_eq_bound_rejects_atomically_including_nonfinite_rust_values() {
    let (_root, core, id) = setup();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for (section, key, value) in [
        ("compressor", "thresholdDb", -60.01),
        ("compressor", "thresholdDb", 0.01),
        ("compressor", "ratio", 0.99),
        ("compressor", "ratio", 20.01),
        ("compressor", "attackMs", 0.009),
        ("compressor", "attackMs", 2000.01),
        ("compressor", "releaseMs", 0.009),
        ("compressor", "releaseMs", 9000.01),
        ("compressor", "makeupGainDb", -0.01),
        ("compressor", "makeupGainDb", 24.01),
        ("eq", "frequencyHz", 19.99),
        ("eq", "frequencyHz", 20000.01),
        ("eq", "q", 0.099),
        ("eq", "q", 10.01),
        ("eq", "gainDb", -24.01),
        ("eq", "gainDb", 24.01),
    ] {
        let mut op = catalog()["input"].clone();
        if section == "eq" {
            op["dsp"]["eq"][0][key] = json!(value);
        } else {
            op["dsp"][section][key] = json!(value);
        }
        assert_eq!(
            core.edit(&id, 0, serde_json::from_value(op).unwrap())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument,
            "{section}.{key}/{value}"
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut dsp: opencut_editor_core::AudioBusDsp =
            serde_json::from_value(catalog()["identity"].clone()).unwrap();
        dsp.gain_db = value;
        assert_eq!(
            core.edit(
                &id,
                0,
                EditOperation::AudioBusSetDsp {
                    bus_id: "music".into(),
                    dsp
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn schema42_adopts_all41_sources_and_retained_history_without_inventing_dsp() {
    let (root, core, id) = setup();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = root.path().join("projects").join(&id);
    let mut sources = vec![];
    for version in 1..=41 {
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
        serde_json::to_vec_pretty(&sources[40]).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":sources,"redo":sources})).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 42);
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for list in ["undo", "redo"] {
        for (index, snapshot) in history[list].as_array().unwrap().iter().enumerate() {
            assert_eq!(snapshot["schemaVersion"], 42);
            assert_eq!(snapshot["assets"], sources[index]["assets"]);
            assert_eq!(snapshot["tracks"], sources[index]["tracks"]);
            assert!(
                snapshot["audioBuses"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|b| b.get("dsp").is_none())
            );
        }
    }
    let before = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn premature_null_malformed_retained_and_future_dsp_fail_without_any_rewrite() {
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
                current["schemaVersion"] = json!(41);
                current["audioBuses"][1]["dsp"] = catalog()["identity"].clone();
            }
            "premature_null" => {
                current["schemaVersion"] = json!(41);
                current["audioBuses"][1]["dsp"] = Value::Null;
            }
            "null" => current["audioBuses"][1]["dsp"] = Value::Null,
            "unknown" => {
                current["audioBuses"][1]["dsp"] = catalog()["identity"].clone();
                current["audioBuses"][1]["dsp"]["rawFilter"] = json!("volume=2");
            }
            "retained" => {
                let mut bad = current.clone();
                bad["audioBuses"][1]["dsp"] = catalog()["identity"].clone();
                bad["audioBuses"][1]["dsp"]["gainDb"] = json!(25);
                std::fs::write(
                    dir.join("history.json"),
                    serde_json::to_vec_pretty(&json!({"undo":[bad],"redo":[]})).unwrap(),
                )
                .unwrap();
            }
            "future" => current["schemaVersion"] = json!(43),
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
    source["schemaVersion"] = json!(41);
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
    assert_eq!(current.schema_version, 42);
    assert_eq!(current.revision, 1);
    assert_eq!(
        serde_json::to_value(&current.audio_buses[1].dsp).unwrap(),
        catalog()["input"]["dsp"]
    );
}
