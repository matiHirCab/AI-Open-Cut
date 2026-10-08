use opencut_editor_core::{
    Asset, BatchEditOperation, EditOperation, EditorCore, ErrorCode, MAX_SOUND_DEFINITIONS,
    MAX_SOUND_VARIANT_SEED, MAX_SOUND_VARIANTS, MediaProbeFacts, MediaType, PathPolicy, Project,
    ProjectSettings, SoundEventDefinition,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/semantic-sound-events-v1.json"
    ))
    .unwrap()
}
fn inventory(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(inventory(&path));
        } else {
            result.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    result
}
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
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
        .create_project("sound definitions", ProjectSettings::default())
        .unwrap()
        .project_id;
    (root, core, id)
}
fn setup_media() -> (tempfile::TempDir, EditorCore, String, Vec<String>) {
    let (root, core, id) = setup();
    let mut ids = vec![];
    // Core persistence accepts immutable facts from its probe port; real-media
    // render conformance remains covered by required native workflows.
    for (index, bytes) in [
        b"independent sound variant A".as_slice(),
        b"independent sound variant B",
        b"independent sound variant C",
    ]
    .into_iter()
    .enumerate()
    {
        let path = root
            .path()
            .join("media")
            .join(format!("variant{index}.wav"));
        std::fs::write(&path, bytes).unwrap();
        let result = core
            .import_asset(
                &id,
                index as u64,
                path,
                MediaType::Audio,
                MediaProbeFacts {
                    duration_ms: Some(500),
                    has_audio: true,
                    ..Default::default()
                },
            )
            .unwrap();
        ids.push(result.changed_ids[0].clone());
    }
    (root, core, id, ids)
}
fn register(event: &str, variants: &[String], gain: f64, bus: &str, seed: u64) -> EditOperation {
    EditOperation::SoundEventRegister {
        event: event.into(),
        variant_asset_ids: variants.into(),
        default_gain_db: gain,
        bus_id: bus.into(),
        variant_seed: seed,
    }
}
fn definitions(project: &Project) -> Value {
    serde_json::to_value(&project.sound_definitions).unwrap()
}
fn canonical_project() -> Project {
    let (_root, core, id) = setup();
    let mut project = core.get_project(&id).unwrap();
    project.assets = catalog()["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| {
            let mut value = asset.clone();
            value["fileName"] = json!("canonical.wav");
            value["durationMs"] = json!(500);
            serde_json::from_value::<Asset>(value).unwrap()
        })
        .collect();
    project.sound_definitions =
        vec![serde_json::from_value(catalog()["definition"].clone()).unwrap()];
    project
}
fn legacy(mut value: Value, version: u32) -> Value {
    value["schemaVersion"] = json!(version);
    value.as_object_mut().unwrap().remove("soundDefinitions");
    if version < 39 {
        value.as_object_mut().unwrap().remove("audioBuses");
        for track in value["tracks"].as_array_mut().unwrap() {
            track.as_object_mut().unwrap().remove("audioBusId");
        }
    }
    if version < 19 {
        value.as_object_mut().unwrap().remove("fonts");
    }
    if version < 24 {
        value.as_object_mut().unwrap().remove("markers");
    }
    value
}

#[test]
fn independent_catalog_selection_is_exact_and_metadata_only() {
    let project = canonical_project();
    assert_eq!(MAX_SOUND_DEFINITIONS, catalog()["maxDefinitions"]);
    assert_eq!(MAX_SOUND_VARIANTS, catalog()["maxVariantsPerDefinition"]);
    assert_eq!(MAX_SOUND_VARIANT_SEED, catalog()["maximumVariantSeed"]);
    let before = serde_json::to_value(&project).unwrap();
    for case in catalog()["selectionCases"].as_array().unwrap() {
        let chosen = project
            .resolve_sound_event_variant("impact", case["seed"].as_u64())
            .unwrap();
        assert_eq!(chosen.variant_index, case["expectedIndex"]);
        assert_eq!(chosen.asset.id, case["expectedAssetId"]);
        assert_eq!(chosen.default_gain_db, -3.0);
        assert_eq!(chosen.bus_id, "sfx");
        assert_eq!(
            chosen.asset.content_hash.as_ref().unwrap().digest,
            catalog()["assets"][chosen.variant_index]["contentHash"]["digest"]
        );
    }
    assert_eq!(serde_json::to_value(&project).unwrap(), before);
    assert_eq!(
        project
            .resolve_sound_event_variant("absent", None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        project
            .resolve_sound_event_variant("impact", Some(MAX_SOUND_VARIANT_SEED + 1))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    for gain in catalog()["gainBoundaryInputs"].as_array().unwrap() {
        let mut candidate = project.clone();
        candidate.sound_definitions[0].default_gain_db = gain.as_f64().unwrap();
        candidate
            .resolve_sound_event_variant("impact", None)
            .unwrap();
    }
}

#[test]
fn whole_registry_bounds_identity_hash_type_and_closed_record_guards() {
    let base = canonical_project();
    let mut project = base.clone();
    project.sound_definitions = (0..512)
        .map(|i| {
            let mut definition = base.sound_definitions[0].clone();
            definition.event = format!("event{i}");
            definition
        })
        .collect();
    project
        .resolve_sound_event_variant("event511", None)
        .unwrap();
    let mut extra = base.sound_definitions[0].clone();
    extra.event = "event512".into();
    project.sound_definitions.push(extra);
    assert_eq!(
        project
            .resolve_sound_event_variant("event0", None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    for invalid in catalog()["invalidModelCases"].as_array().unwrap() {
        let name = invalid.as_str().unwrap();
        if matches!(
            name,
            "513_definitions"
                | "33_variants"
                | "missing_record_field"
                | "unknown_record_field"
                | "duplicate_record_field"
        ) {
            continue;
        }
        let mut candidate = base.clone();
        match name {
            "duplicate_name" => candidate
                .sound_definitions
                .push(candidate.sound_definitions[0].clone()),
            "duplicate_content_hash" => {
                candidate.assets[1].content_hash = candidate.assets[0].content_hash.clone()
            }
            "silent_video" => {
                candidate.assets[0].media_type = MediaType::Video;
                candidate.assets[0].has_audio = false;
            }
            "non_audio_asset" => candidate.assets[0].media_type = MediaType::Image,
            "missing_content_hash" => candidate.assets[0].content_hash = None,
            "wrong_hash_algorithm" => {
                candidate.assets[0].content_hash.as_mut().unwrap().algorithm = "md5".into()
            }
            "malformed_hash" => {
                candidate.assets[0].content_hash.as_mut().unwrap().digest = "A".repeat(64)
            }
            "missing_size" => candidate.assets[0].size_bytes = None,
            "zero_size" => candidate.assets[0].size_bytes = Some(0),
            "non_finite_gain" => candidate.sound_definitions[0].default_gain_db = f64::NAN,
            _ => panic!("uncovered canonical model case {name}"),
        }
        assert_eq!(
            candidate
                .resolve_sound_event_variant("impact", None)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument,
            "{name}"
        );
    }
    for field in [
        "event",
        "variantAssetIds",
        "defaultGainDb",
        "busId",
        "variantSeed",
    ] {
        let mut value = catalog()["definition"].clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<SoundEventDefinition>(value).is_err(),
            "{field}"
        );
    }
    let mut value = catalog()["definition"].clone();
    value["expression"] = json!("raw");
    assert!(serde_json::from_value::<SoundEventDefinition>(value).is_err());
    assert!(serde_json::from_str::<SoundEventDefinition>(r#"{"event":"impact","event":"duplicate","variantAssetIds":["a"],"defaultGainDb":0,"busId":"sfx","variantSeed":0}"#).is_err());
    for seed in catalog()["invalidSeedInputs"].as_array().unwrap() {
        let mut value = catalog()["definition"].clone();
        value["variantSeed"] = seed.clone();
        let rejected = match serde_json::from_value::<SoundEventDefinition>(value) {
            Err(_) => true,
            Ok(def) => {
                let mut candidate = base.clone();
                candidate.sound_definitions = vec![def];
                candidate
                    .resolve_sound_event_variant("impact", None)
                    .is_err()
            }
        };
        assert!(rejected, "{seed}");
    }
    let mut many = base.clone();
    many.assets = (0..33)
        .map(|index| {
            let mut asset = base.assets[0].clone();
            asset.id = format!("variant{index}");
            asset.content_hash.as_mut().unwrap().digest = format!("{index:064x}");
            asset
        })
        .collect();
    many.sound_definitions[0].variant_asset_ids = many.assets[..32]
        .iter()
        .map(|asset| asset.id.clone())
        .collect();
    many.resolve_sound_event_variant("impact", None).unwrap();
    many.sound_definitions[0]
        .variant_asset_ids
        .push("variant32".into());
    assert_eq!(
        many.resolve_sound_event_variant("impact", None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn real_named_alias_replacement_history_and_reopen_are_exact() {
    let (root, core, id, assets) = setup_media();
    let result = core
        .edit_batch(
            &id,
            3,
            vec![
                BatchEditOperation {
                    edit: register("impact", &assets[..1], -3.0, "sfx", 42),
                    result_alias: Some("hit".into()),
                },
                BatchEditOperation {
                    edit: register("@hit", &assets[1..2], -6.0, "music", 1),
                    result_alias: None,
                },
            ],
        )
        .unwrap();
    assert_eq!(result.revision, 4);
    assert_eq!(result.aliases["hit"], "impact");
    let first = definitions(&core.get_project(&id).unwrap());
    assert_eq!(first.as_array().unwrap().len(), 1);
    core.edit(&id, 4, register("other", &assets[2..], 0.0, "master", 0))
        .unwrap();
    let before = core.get_project(&id).unwrap();
    core.edit(
        &id,
        5,
        register(
            "impact",
            &assets[..1],
            -120.0,
            "voiceover",
            MAX_SOUND_VARIANT_SEED,
        ),
    )
    .unwrap();
    let replaced = core.get_project(&id).unwrap();
    assert_eq!(replaced.sound_definitions[0].event, "impact");
    assert_eq!(replaced.sound_definitions[1].event, "other");
    assert_eq!(
        serde_json::to_value(&replaced.assets).unwrap(),
        serde_json::to_value(&before.assets).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&replaced.tracks).unwrap(),
        serde_json::to_value(&before.tracks).unwrap()
    );
    core.undo(&id, 6).unwrap();
    assert_eq!(
        definitions(&core.get_project(&id).unwrap()),
        definitions(&before)
    );
    core.redo(&id, 7).unwrap();
    assert_eq!(
        definitions(&core.get_project(&id).unwrap()),
        definitions(&replaced)
    );
    let reopened_core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let reopened = reopened_core.get_project(&id).unwrap();
    assert_eq!(definitions(&reopened), definitions(&replaced));
    assert_eq!(
        reopened
            .resolve_sound_event_variant("impact", None)
            .unwrap()
            .asset
            .id,
        assets[0]
    );
}

#[test]
fn all_invalid_registration_and_late_batch_failures_preserve_complete_bytes() {
    let (root, core, id, assets) = setup_media();
    for case in catalog()["invalidDefinitionChanges"].as_array().unwrap() {
        let mut input = catalog()["registrationInput"].clone();
        input["variantAssetIds"] = json!(assets);
        input[case["field"].as_str().unwrap()] = case["value"].clone();
        if let Some(variants) = input["variantAssetIds"].as_array_mut() {
            for variant in variants {
                if let Some(index) = catalog()["definition"]["variantAssetIds"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .position(|v| v == variant)
                {
                    *variant = json!(assets[index]);
                }
            }
        }
        let before = inventory(root.path());
        let op: EditOperation = serde_json::from_value(input).unwrap();
        let error = core.edit(&id, 3, op).unwrap_err();
        assert_eq!(
            serde_json::to_value(error.code).unwrap(),
            case["expectedCode"],
            "{case}"
        );
        assert!(!error.retryable);
        assert_eq!(inventory(root.path()), before, "{case}");
    }
    let before = inventory(root.path());
    let error = core
        .edit(
            &id,
            2,
            register(
                "bad/name",
                &["missing".into()],
                f64::INFINITY,
                "missing",
                u64::MAX,
            ),
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(error.retryable);
    assert_eq!(inventory(root.path()), before);
    let error = core
        .edit_batch(
            &id,
            3,
            vec![
                register("impact", &assets[..1], 0.0, "sfx", 0),
                register("other", &["missing".into()], 0.0, "sfx", 0),
            ],
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::AssetNotFound);
    assert_eq!(inventory(root.path()), before);
    let mut value = catalog()["registrationInput"].clone();
    value["unexpected"] = json!(true);
    assert!(serde_json::from_value::<EditOperation>(value).is_err());
    for field in [
        "event",
        "variantAssetIds",
        "defaultGainDb",
        "busId",
        "variantSeed",
    ] {
        let mut value = catalog()["registrationInput"].clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<EditOperation>(value).is_err());
    }
}

#[test]
fn pending_draft_roots_update_rebase_preview_commit_and_conflict_are_atomic() {
    let (root, core, id, assets) = setup_media();
    let draft = core
        .create_draft(
            &id,
            3,
            vec![register("impact", &assets[..1], -3.0, "sfx", 1)],
            None,
        )
        .unwrap();
    assert!(core.get_project(&id).unwrap().sound_definitions.is_empty());
    let before = inventory(root.path());
    let error = core.delete_asset(&id, 3, &assets[0]).unwrap_err();
    assert_eq!(error.code, ErrorCode::AssetInUse);
    assert!(error.message.contains("draft operation"));
    assert_eq!(inventory(root.path()), before);
    core.update_draft(
        &id,
        &draft.id,
        3,
        vec![register("impact", &assets[1..2], -6.0, "music", 0)],
        None,
    )
    .unwrap();
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .sound_definitions[0]
            .variant_asset_ids,
        [assets[1].clone()]
    );
    core.delete_asset(&id, 3, &assets[0]).unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.commit_draft(&id, &draft.id, 4).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(root.path()), before);
    core.rebase_draft(&id, &draft.id, 4).unwrap();
    core.commit_draft(&id, &draft.id, 4).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().sound_definitions[0].variant_asset_ids,
        [assets[1].clone()]
    );
    assert_eq!(
        core.get_draft(&id, &draft.id).unwrap_err().code,
        ErrorCode::DraftNotFound
    );
}

#[test]
fn sound_roots_protect_deletion_and_history_owns_replaced_variant_bytes() {
    let (root, core, id, assets) = setup_media();
    core.edit(&id, 3, register("impact", &assets[..1], 0.0, "sfx", 0))
        .unwrap();
    let original = core.get_project(&id).unwrap();
    let old_path = root
        .path()
        .join("projects")
        .join(&id)
        .join(&original.assets[0].project_relative_path);
    let before = inventory(root.path());
    let error = core.delete_asset(&id, 4, &assets[0]).unwrap_err();
    assert_eq!(error.code, ErrorCode::AssetInUse);
    assert!(error.message.contains("sound definition"));
    assert_eq!(inventory(root.path()), before);
    core.edit(&id, 4, register("impact", &assets[1..2], 0.0, "sfx", 0))
        .unwrap();
    core.delete_asset(&id, 5, &assets[0]).unwrap();
    assert!(old_path.exists());
    core.undo(&id, 6).unwrap();
    core.undo(&id, 7).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .resolve_sound_event_variant("impact", None)
            .unwrap()
            .asset
            .id,
        assets[0]
    );
    core.redo(&id, 8).unwrap();
    core.redo(&id, 9).unwrap();
    for revision in 10..115 {
        core.edit(
            &id,
            revision,
            register("impact", &assets[1..2], 0.0, "sfx", revision),
        )
        .unwrap();
    }
    assert!(
        !old_path.exists(),
        "only after current/draft/history roots expire"
    );
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .resolve_sound_event_variant("impact", None)
            .unwrap()
            .asset
            .id,
        assets[1]
    );
}

#[test]
fn every_supported_source_and_mixed_history_adopt_only_empty_registry() {
    for version in 1..=39 {
        let (root, core, id) = setup();
        let dir = root.path().join("projects").join(&id);
        let mut current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        current["audioBuses"][0]["outputBusId"] = json!("music");
        let audio = current["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|t| t["trackType"] == "audio")
            .unwrap();
        audio["audioBusId"] = json!("sfx");
        let source = legacy(current, version);
        write_json(&dir.join("project.json"), &source);
        write_json(
            &dir.join("history.json"),
            &json!({"undo":[source.clone()],"redo":[source.clone()]}),
        );
        let target = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        assert_eq!(catalog()["projectSchemaVersion"], 40);
        assert_eq!(
            target["schemaVersion"],
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        assert_eq!(target["soundDefinitions"], json!([]));
        for key in [
            "id",
            "revision",
            "name",
            "createdAtMs",
            "updatedAtMs",
            "settings",
            "assets",
            "tracks",
        ] {
            assert_eq!(target[key], source[key], "source{version} {key}");
        }
        if version == 39 {
            assert_eq!(target["audioBuses"], source["audioBuses"]);
        }
        let history: Value =
            serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
        for snapshot in history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap())
        {
            assert_eq!(catalog()["projectSchemaVersion"], 40);
            assert_eq!(
                snapshot["schemaVersion"],
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
            assert_eq!(snapshot["soundDefinitions"], json!([]));
            if version == 39 {
                assert_eq!(snapshot["audioBuses"], source["audioBuses"]);
                assert_eq!(snapshot["tracks"], source["tracks"]);
            }
        }
        let stable = inventory(root.path());
        core.get_project(&id).unwrap();
        assert_eq!(inventory(root.path()), stable);
    }
    let (root, core, id) = setup();
    let dir = root.path().join("projects").join(&id);
    let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let sources: Vec<Value> = [1, 7, 19, 24, 37, 38, 39]
        .into_iter()
        .map(|version| legacy(current.clone(), version))
        .collect();
    write_json(&dir.join("project.json"), &sources[6]);
    write_json(
        &dir.join("history.json"),
        &json!({"undo":sources[..3],"redo":sources[3..]}),
    );
    core.get_project(&id).unwrap();
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for snapshot in history["undo"]
        .as_array()
        .unwrap()
        .iter()
        .chain(history["redo"].as_array().unwrap())
    {
        assert_eq!(catalog()["projectSchemaVersion"], 40);
        assert_eq!(
            snapshot["schemaVersion"],
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        assert_eq!(snapshot["soundDefinitions"], json!([]));
    }
}

#[test]
fn premature_malformed_retained_future_and_failed_legacy_edits_never_publish() {
    for case in [
        "premature_empty",
        "premature_null",
        "missing_current",
        "null_current",
        "malformed_retained",
        "future",
    ] {
        let (root, core, id) = setup();
        let dir = root.path().join("projects").join(&id);
        let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let mut value = current.clone();
        match case {
            "premature_empty" | "premature_null" => {
                value = legacy(current.clone(), 39);
                value["soundDefinitions"] = if case == "premature_null" {
                    Value::Null
                } else {
                    json!([])
                };
            }
            "missing_current" => {
                value.as_object_mut().unwrap().remove("soundDefinitions");
            }
            "null_current" => value["soundDefinitions"] = Value::Null,
            "future" => {
                // Retain the frozen predecessor's version boundary while the
                // live rejection probe follows the current schema successor.
                assert_eq!(catalog()["projectSchemaVersion"], 40);
                assert_eq!(catalog()["projectSchemaVersion"].as_u64().unwrap() + 1, 41);
                value["schemaVersion"] = json!(opencut_editor_core::PROJECT_SCHEMA_VERSION + 1);
            }
            "malformed_retained" => {
                let mut snapshot = current.clone();
                snapshot["soundDefinitions"] = json!([catalog()["definition"].clone()]);
                write_json(
                    &dir.join("history.json"),
                    &json!({"undo":[snapshot],"redo":[]}),
                );
            }
            _ => unreachable!(),
        }
        write_json(&dir.join("project.json"), &value);
        let before = inventory(root.path());
        let error = core.get_project(&id).unwrap_err();
        assert!(!error.retryable);
        if case == "future" {
            assert_eq!(error.code, ErrorCode::InternalError);
        }
        if case == "malformed_retained" {
            assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        }
        assert_eq!(inventory(root.path()), before, "{case}");
    }
    for version in [1, 7, 19, 24, 37, 38, 39] {
        let (root, core, id) = setup();
        let dir = root.path().join("projects").join(&id);
        let source = legacy(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            version,
        );
        write_json(&dir.join("project.json"), &source);
        write_json(
            &dir.join("history.json"),
            &json!({"undo":[source.clone()],"redo":[source]}),
        );
        for (revision, op, expected) in [
            (
                1,
                register("impact", &["absent".into()], 0.0, "sfx", 0),
                ErrorCode::RevisionConflict,
            ),
            (
                0,
                register("impact", &["absent".into()], 0.0, "sfx", 0),
                ErrorCode::AssetNotFound,
            ),
            (
                0,
                register("bad/name", &["absent".into()], 0.0, "sfx", 0),
                ErrorCode::InvalidArgument,
            ),
        ] {
            let before = inventory(root.path());
            assert_eq!(core.edit(&id, revision, op).unwrap_err().code, expected);
            assert_eq!(inventory(root.path()), before, "source{version}");
        }
        let before = inventory(root.path());
        assert_eq!(
            core.create_draft(
                &id,
                0,
                vec![register("impact", &["absent".into()], 0.0, "sfx", 0)],
                None
            )
            .unwrap_err()
            .code,
            ErrorCode::AssetNotFound
        );
        assert_eq!(inventory(root.path()), before);
    }
}

#[test]
fn corrupted_or_dangling_registered_media_rejects_without_state_changes() {
    for corrupt in [false, true] {
        let (root, core, id, assets) = setup_media();
        core.edit(&id, 3, register("impact", &assets[..1], 0.0, "sfx", 0))
            .unwrap();
        let project = core.get_project(&id).unwrap();
        let dir = root.path().join("projects").join(&id);
        if corrupt {
            std::fs::write(
                dir.join(&project.assets[0].project_relative_path),
                b"changed registered bytes",
            )
            .unwrap();
        } else {
            let mut value = serde_json::to_value(project).unwrap();
            value["soundDefinitions"][0]["variantAssetIds"] = json!(["absent"]);
            write_json(&dir.join("project.json"), &value);
        }
        let before = inventory(root.path());
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::AssetIntegrityFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
}

fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

#[test]
fn legacy_dynamic_sound_definition_slot_keys_remain_user_values() {
    let (root, core, id) = setup();
    let mut slots: Value =
        serde_json::from_str(include_str!("../../../contracts/template-slots-v1.json")).unwrap();
    let base = slots["valid"]
        .as_array_mut()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "number")
        .unwrap()["slot"]
        .clone();
    let bindings: Vec<Value> = ["soundDefinitions", "variantSeed"]
        .iter()
        .map(|key| {
            let mut value = base.clone();
            value["id"] = json!(key);
            value["binding"]["targetLayerId"] = json!(key);
            value
        })
        .collect();
    let items: Vec<Value> = ["soundDefinitions", "variantSeed"].iter().enumerate().map(|(order, key)| json!({"id":key,"type":"rectangle","keyframes":[],"startMs":0,"durationMs":1000,"stackOrder":order,"zIndex":order,"color":"#ffffff","width":32,"height":32,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).collect();
    let component = core.edit(&id, 0, operation(json!({"operation":"component_create","name":"Dynamic keys","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"visual","name":"Visual","trackType":"overlay","items":items}],"slots":bindings}))).unwrap().changed_ids[0].clone();
    let values = json!({"soundDefinitions":{"type":"number","value":0.25},"variantSeed":{"type":"number","value":0.75}});
    let outer = core.edit(&id, 1, operation(json!({"operation":"component_create","name":"Outer","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"outer","name":"Outer","trackType":"overlay","items":[{"type":"component_instance","id":"nested","componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values}]}]}))).unwrap().changed_ids[0].clone();
    let source = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = root.path().join("projects").join(&id);
    write_json(&dir.join("project.json"), &legacy(source.clone(), 39));
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(migrated["components"], source["components"]);
    let component = migrated["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["id"] == outer)
        .unwrap();
    assert_eq!(component["tracks"][0]["items"][0]["slotValues"], values);
    assert_eq!(migrated["soundDefinitions"], json!([]));
    assert_eq!(migrated["audioBuses"], source["audioBuses"]);
    let stable = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), stable);
}

#[test]
fn unresolved_forward_and_duplicate_batch_aliases_preserve_all_bytes() {
    let (root, core, id, assets) = setup_media();
    let before = inventory(root.path());
    for operations in [
        vec![
            BatchEditOperation {
                edit: register("@future", &assets[..1], 0.0, "sfx", 0),
                result_alias: None,
            },
            BatchEditOperation {
                edit: register("impact", &assets[..1], 0.0, "sfx", 0),
                result_alias: Some("future".into()),
            },
        ],
        vec![BatchEditOperation {
            edit: register("impact", &["@missing".into()], 0.0, "sfx", 0),
            result_alias: None,
        }],
        vec![
            BatchEditOperation {
                edit: register("impact", &assets[..1], 0.0, "sfx", 0),
                result_alias: Some("same".into()),
            },
            BatchEditOperation {
                edit: register("other", &assets[..1], 0.0, "sfx", 0),
                result_alias: Some("same".into()),
            },
        ],
    ] {
        assert_eq!(
            core.edit_batch(&id, 3, operations).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
}
