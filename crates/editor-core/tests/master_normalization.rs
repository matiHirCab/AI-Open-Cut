use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/master-normalization-v1.json"
    ))
    .unwrap()
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
        .create_project("Normalization", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
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
fn closed_master_settings_and_operations_match_independent_catalog() {
    let input = catalog()["input"].clone();
    let edit: EditOperation = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(edit).unwrap(), input);
    let batch: BatchEditOperation = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(batch).unwrap(), input);
    for key in [
        "enabled",
        "targetIntegratedLufs",
        "targetLoudnessRangeLu",
        "targetTruePeakDbtp",
    ] {
        let mut missing = input.clone();
        missing["normalization"]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(serde_json::from_value::<EditOperation>(missing.clone()).is_err());
        assert!(serde_json::from_value::<BatchEditOperation>(missing).is_err());
    }
    for value in [json!(null), json!({"rawFilter":"loudnorm"}), json!(false)] {
        let mut invalid = input.clone();
        invalid["normalization"] = value;
        assert!(serde_json::from_value::<EditOperation>(invalid.clone()).is_err());
        assert!(serde_json::from_value::<BatchEditOperation>(invalid).is_err());
    }
    let mut unknown = input.clone();
    unknown["normalization"]["rawFilter"] = json!("loudnorm");
    assert!(serde_json::from_value::<EditOperation>(unknown).is_err());
    let mut alias = input;
    alias["resultAlias"] = json!("master");
    assert!(serde_json::from_value::<EditOperation>(alias.clone()).is_err());
    assert!(serde_json::from_value::<BatchEditOperation>(alias).is_err());
}

#[test]
fn empty_root_authoring_is_backend_independent_and_survives_history_and_reopen() {
    let (_root, core, id) = setup();
    let initial = core.get_project(&id).unwrap();
    assert_eq!(initial.schema_version, 44);
    assert!(
        serde_json::to_value(&initial)
            .unwrap()
            .get("masterNormalization")
            .is_none()
    );
    let result = core
        .edit(
            &id,
            0,
            serde_json::from_value(catalog()["input"].clone()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap()["changedIds"],
        catalog()["changedIds"]
    );
    core.undo(&id, 1).unwrap();
    assert!(
        serde_json::to_value(core.get_project(&id).unwrap())
            .unwrap()
            .get("masterNormalization")
            .is_none()
    );
    core.redo(&id, 2).unwrap();
    let mut disabled = catalog()["input"].clone();
    disabled["normalization"] = catalog()["disabledExample"].clone();
    core.edit(&id, 3, serde_json::from_value(disabled).unwrap())
        .unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap()["masterNormalization"],
        catalog()["disabledExample"]
    );
    reopened.undo(&id, 4).unwrap();
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap()["masterNormalization"],
        catalog()["settingsExample"]
    );
}

#[test]
fn draft_materialization_commit_and_stale_failure_preserve_root_and_source_generation() {
    let (root, core, id) = setup();
    let input: EditOperation = serde_json::from_value(catalog()["input"].clone()).unwrap();
    let draft = core
        .create_draft(&id, 0, vec![input.clone()], None)
        .unwrap();
    assert!(
        serde_json::to_value(core.get_project(&id).unwrap())
            .unwrap()
            .get("masterNormalization")
            .is_none()
    );
    assert_eq!(
        serde_json::to_value(core.get_draft_state(&id, &draft.id).unwrap()).unwrap()["project"]["masterNormalization"],
        catalog()["settingsExample"]
    );
    core.commit_draft(&id, &draft.id, 0).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap()["masterNormalization"],
        catalog()["settingsExample"]
    );
    let stale = core
        .create_draft(&id, 1, vec![input.clone()], None)
        .unwrap();
    core.edit(&id, 1, input).unwrap();
    let dir = root.path().join("projects").join(&id);
    let before = inventory(&dir);
    assert_eq!(
        core.commit_draft(&id, &stale.id, 1).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(&dir), before);
    assert!(core.get_draft_state(&id, &stale.id).is_err());
    assert_eq!(inventory(&dir), before);
}

#[test]
fn typed_nonfinite_settings_fail_even_when_disabled_and_boundary_controls_remain_valid() {
    let (_root, core, id) = setup();
    for enabled in [true, false] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for field in 0..3 {
                let mut normalization: opencut_editor_core::MasterNormalization =
                    serde_json::from_value(catalog()["settingsExample"].clone()).unwrap();
                normalization.enabled = enabled;
                match field {
                    0 => normalization.target_integrated_lufs = value,
                    1 => normalization.target_loudness_range_lu = value,
                    _ => normalization.target_true_peak_dbtp = value,
                }
                assert_eq!(
                    core.edit(
                        &id,
                        0,
                        EditOperation::AudioMasterSetNormalization { normalization }
                    )
                    .unwrap_err()
                    .code,
                    ErrorCode::InvalidArgument
                );
                assert_eq!(core.get_project(&id).unwrap().revision, 0);
            }
        }
    }
    for (index, (integrated, lra, peak)) in [(-70.0, 1.0, -9.0), (-5.0, 50.0, 0.0)]
        .into_iter()
        .enumerate()
    {
        core.edit(
            &id,
            index as u64,
            EditOperation::AudioMasterSetNormalization {
                normalization: opencut_editor_core::MasterNormalization {
                    enabled: false,
                    target_integrated_lufs: integrated,
                    target_loudness_range_lu: lra,
                    target_true_peak_dbtp: peak,
                },
            },
        )
        .unwrap();
    }
}

#[test]
fn bounds_validate_disabled_controls_with_stale_revision_priority_and_atomic_batch() {
    let (root, core, id) = setup();
    let dir = root.path().join("projects").join(&id);
    let before = inventory(&dir);
    for enabled in [true, false] {
        for (key, invalid) in [
            ("targetIntegratedLufs", -70.01),
            ("targetIntegratedLufs", -4.99),
            ("targetLoudnessRangeLu", 0.99),
            ("targetLoudnessRangeLu", 50.01),
            ("targetTruePeakDbtp", -9.01),
            ("targetTruePeakDbtp", 0.01),
        ] {
            let mut input = catalog()["input"].clone();
            input["normalization"]["enabled"] = json!(enabled);
            input["normalization"][key] = json!(invalid);
            let op: EditOperation = serde_json::from_value(input).unwrap();
            assert_eq!(
                core.edit(&id, 99, op.clone()).unwrap_err().code,
                ErrorCode::RevisionConflict
            );
            assert_eq!(
                core.edit(&id, 0, op).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(inventory(&dir), before);
        }
    }
    let first: BatchEditOperation = serde_json::from_value(catalog()["input"].clone()).unwrap();
    let second: BatchEditOperation = serde_json::from_value(
        json!({"operation":"marker_delete","scope":"root","markerId":"missing"}),
    )
    .unwrap();
    assert!(core.edit_batch(&id, 0, vec![first, second]).is_err());
    assert_eq!(inventory(&dir), before);
    let op = BatchEditOperation {
        edit: serde_json::from_value(catalog()["input"].clone()).unwrap(),
        result_alias: Some("master".into()),
    };
    assert_eq!(
        core.edit_batch(&id, 0, vec![op]).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(&dir), before);
}

#[test]
fn all43_sources_and_retained_history_migrate_without_inventing_controls() {
    let (root, core, id) = setup();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = root.path().join("projects").join(&id);
    let sources: Vec<_> = (1..=43)
        .map(|version| {
            let mut source = original.clone();
            source["schemaVersion"] = json!(version);
            for (introduced, field) in [
                (40, "soundDefinitions"),
                (39, "audioBuses"),
                (24, "markers"),
                (19, "fonts"),
            ] {
                if version < introduced {
                    source.as_object_mut().unwrap().remove(field);
                }
            }
            source
        })
        .collect();
    for source in &sources {
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec_pretty(source).unwrap(),
        )
        .unwrap();
        let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        assert_eq!(migrated["schemaVersion"], 44);
        assert!(migrated.get("masterNormalization").is_none());
        for field in ["assets", "tracks", "createdAtMs", "updatedAtMs"] {
            assert_eq!(migrated[field], source[field]);
        }
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&sources[42]).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":sources,"redo":sources})).unwrap(),
    )
    .unwrap();
    assert_eq!(core.get_project(&id).unwrap().schema_version, 44);
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for list in ["undo", "redo"] {
        for (index, snapshot) in history[list].as_array().unwrap().iter().enumerate() {
            assert_eq!(snapshot["schemaVersion"], 44);
            assert!(snapshot.get("masterNormalization").is_none());
            for field in ["assets", "tracks", "createdAtMs", "updatedAtMs"] {
                assert_eq!(snapshot[field], sources[index][field]);
            }
        }
    }
}

#[test]
fn premature_null_invalid_and_future_current_or_history_are_rejected_without_rewrite() {
    for retained in [false, true] {
        for (version, value) in [
            (43, json!(null)),
            (43, catalog()["disabledExample"].clone()),
            (44, json!(null)),
            (44, json!({"enabled":false})),
            (45, catalog()["settingsExample"].clone()),
        ] {
            let (root, core, id) = setup();
            let dir = root.path().join("projects").join(&id);
            let mut invalid = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            invalid["schemaVersion"] = json!(version);
            invalid["masterNormalization"] = value;
            let (file, document) = if retained {
                ("history.json", json!({"undo":[invalid],"redo":[]}))
            } else {
                ("project.json", invalid)
            };
            std::fs::write(
                dir.join(file),
                serde_json::to_vec_pretty(&document).unwrap(),
            )
            .unwrap();
            let before = inventory(&dir);
            assert!(core.get_project(&id).is_err(), "{retained}/{version}");
            assert_eq!(inventory(&dir), before);
        }
    }
}

#[test]
fn schema43_managed_speech_provenance_and_history_survive_normalization_adoption() {
    use opencut_editor_core::{CommitGeneratedAssetRequest, GeneratedAssetOrigin, MediaProbeFacts};
    let (root, _, id) = setup();
    let generated = root.path().join("generated");
    std::fs::create_dir(&generated).unwrap();
    let source = generated.join("speech.wav");
    std::fs::write(
        &source,
        b"independently authored managed speech transport fixture",
    )
    .unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap(),
    );
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Audio)
        .unwrap()
        .id
        .clone();
    let generation = json!({"request":{"text":"Preserve the saved origin.","language":"en","voiceId":"fixture","speed":1.0},"providerId":"fixture","modelId":"fixture","modelVersion":null,"sampleRateHz":24000,"generatedAtMs":1});
    core.commit_generated_asset(CommitGeneratedAssetRequest {
        marker_policy: Default::default(),
        project_id: id.clone(),
        expected_revision: 0,
        path: source,
        track_id: track,
        start_ms: 0,
        duration_ms: 1000,
        display_name: "managed speech".into(),
        origin: GeneratedAssetOrigin::SpeechSynthesis(serde_json::from_value(generation).unwrap()),
        probe: MediaProbeFacts::default(),
    })
    .unwrap();
    let dir = root.path().join("projects").join(&id);
    let mut predecessor = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    predecessor["schemaVersion"] = json!(43);
    assert!(predecessor.get("masterNormalization").is_none());
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&predecessor).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":[predecessor],"redo":[predecessor]})).unwrap(),
    )
    .unwrap();
    let managed_before = inventory(&dir.join("assets"));
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(migrated["schemaVersion"], 44);
    for field in ["assets", "tracks", "fonts", "createdAtMs", "updatedAtMs"] {
        assert_eq!(migrated[field], predecessor[field]);
    }
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for list in ["undo", "redo"] {
        assert_eq!(history[list][0]["assets"], predecessor["assets"]);
        assert_eq!(history[list][0]["tracks"], predecessor["tracks"]);
        assert_eq!(history[list][0]["schemaVersion"], 44);
        assert!(history[list][0].get("masterNormalization").is_none());
    }
    let write = core
        .edit(
            &id,
            1,
            serde_json::from_value(catalog()["input"].clone()).unwrap(),
        )
        .unwrap();
    assert_eq!(inventory(&dir.join("assets")), managed_before);
    core.undo(&id, write.revision).unwrap();
    let reopened = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(reopened["assets"], predecessor["assets"]);
    assert!(reopened.get("masterNormalization").is_none());
    assert_eq!(inventory(&dir.join("assets")), managed_before);
}
