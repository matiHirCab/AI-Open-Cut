use opencut_editor_core::*;
use serde_json::{Value, json};

fn settings() -> NarrationProfileSettings {
    let contract: Value = serde_json::from_str(include_str!(
        "../../../contracts/narration-profiles-v1.json"
    ))
    .unwrap();
    serde_json::from_value(contract["settings"].clone()).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore) {
    let root = tempfile::tempdir().unwrap();
    let paths = PathPolicy::new(
        root.path().join("projects"),
        vec![root.path().to_path_buf()],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(paths);
    (root, core)
}
fn application(p: &NarrationProfile) -> NarrationProfileApplication {
    NarrationProfileApplication {
        profile_id: p.id.clone(),
        expected_version: p.version,
        text: "AI Open ñ 🎙".into(),
        language: None,
        voice_id: None,
        speed: None,
        text_options: None,
    }
}
#[test]
fn shared_versioned_profiles_are_explicit_and_snapshots_are_independent() {
    let (_root, core) = setup();
    let a = core
        .create_project("A", ProjectSettings::default())
        .unwrap();
    let b = core
        .create_project("B", ProjectSettings::default())
        .unwrap();
    let profile = core.create_narration_profile(settings()).unwrap();
    let first = core
        .resolve_narration_profile(application(&profile))
        .unwrap();
    let second = core
        .resolve_narration_profile(application(&profile))
        .unwrap();
    assert_eq!(first, second);
    let mut changed = settings();
    changed.speed = 1.3;
    let updated = core
        .update_narration_profile(&profile.id, 1, changed)
        .unwrap();
    assert_eq!(updated.version, 2);
    assert_eq!(first.speed, 1.1);
    assert_eq!(first.narration_profile.unwrap().version, 1);
    assert_eq!(
        core.resolve_narration_profile(application(&profile))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let reloaded = EditorCore::new(core.paths().clone());
    assert_eq!(
        reloaded.get_narration_profile(&profile.id).unwrap(),
        updated
    );
    assert_eq!(
        core.get_state(&a.project_id, None)
            .unwrap()
            .project
            .revision,
        a.revision
    );
    assert_eq!(
        core.get_state(&b.project_id, None)
            .unwrap()
            .project
            .revision,
        b.revision
    );
}
#[test]
fn explicit_overrides_replace_whole_options_including_empty_entries() {
    let (_root, core) = setup();
    let profile = core.create_narration_profile(settings()).unwrap();
    let mut apply = application(&profile);
    apply.speed = Some(1.2);
    apply.text_options = Some(SpeechTextOptions::default());
    let resolved = core.resolve_narration_profile(apply).unwrap();
    assert_eq!(resolved.speed, 1.2);
    assert_eq!(resolved.language, profile.settings.language);
    assert_eq!(resolved.voice_id, profile.settings.voice_id);
    assert!(resolved.text_options.pronunciations.is_empty());
    assert_eq!(profile.settings.text_options.pronunciations.len(), 2);
}
#[test]
fn stale_update_duplicate_names_and_invalid_values_preserve_library_bytes() {
    let (_root, core) = setup();
    let profile = core.create_narration_profile(settings()).unwrap();
    let path = core.paths().projects_root().join("narration-profiles.json");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(
        core.update_narration_profile(&profile.id, 0, settings())
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert!(core.create_narration_profile(settings()).is_err());
    for bad in ["", "\n", &"🎙".repeat(81)] {
        let mut value = settings();
        value.name = bad.into();
        assert!(
            core.update_narration_profile(&profile.id, 1, value)
                .is_err()
        );
    }
    for speed in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut value = settings();
        value.speed = speed;
        assert!(
            core.update_narration_profile(&profile.id, 1, value)
                .is_err()
        );
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn ordered_overlaps_unicode_limits_and_trimmed_duplicates_are_deterministic() {
    let (_root, core) = setup();
    let mut value = settings();
    value.name = "  🎙  ".into();
    value.text_options.pronunciations = vec![SpeechPronunciation {
        term: "🎙".repeat(128),
        spoken: "ñ".repeat(256),
    }];
    let valid = core.create_narration_profile(value.clone()).unwrap();
    assert_eq!(valid.settings.name, "🎙");
    value.name = "other".into();
    value.text_options.pronunciations[0].term.push('🎙');
    assert!(core.create_narration_profile(value).is_err());
    let mut duplicate = settings();
    duplicate
        .text_options
        .pronunciations
        .push(SpeechPronunciation {
            term: " AI ".into(),
            spoken: "dup".into(),
        });
    assert!(core.create_narration_profile(duplicate).is_err());
    let overlap = core.create_narration_profile(settings()).unwrap();
    assert_eq!(
        overlap.settings.text_options.pronunciations,
        settings().text_options.pronunciations
    );
}
#[test]
fn profile_count_and_pronunciation_bounds_fail_without_partial_publication() {
    let (_root, core) = setup();
    for i in 0..MAX_NARRATION_PROFILES {
        let mut value = settings();
        value.name = format!("profile {i}");
        core.create_narration_profile(value).unwrap();
    }
    assert!(core.create_narration_profile(settings()).is_err());
    assert_eq!(
        core.list_narration_profiles().unwrap().len(),
        MAX_NARRATION_PROFILES
    );
    let mut value = settings();
    value.text_options.sentence_pause_ms = 5001;
    assert!(value.validate().is_err());
    value = settings();
    value.text_options.pronunciations = (0..101)
        .map(|i| SpeechPronunciation {
            term: format!("t{i}"),
            spoken: "x".into(),
        })
        .collect();
    assert!(value.validate().is_err());
}
#[test]
fn corrupt_future_duplicate_and_oversized_libraries_fail_closed() {
    let (_root, core) = setup();
    let profile = core.create_narration_profile(settings()).unwrap();
    let path = core.paths().projects_root().join("narration-profiles.json");
    for bytes in [
        b"{".to_vec(),
        serde_json::to_vec(&json!({"version":2,"profiles":[]})).unwrap(),
        serde_json::to_vec(&json!({"version":1,"profiles":[profile.clone(),profile]})).unwrap(),
    ] {
        std::fs::write(&path, &bytes).unwrap();
        assert!(core.list_narration_profiles().is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(64 * 1024 * 1024 + 1).unwrap();
    assert!(core.list_narration_profiles().is_err());
    assert_eq!(
        std::fs::metadata(&path).unwrap().len(),
        64 * 1024 * 1024 + 1
    );
}
#[test]
fn canonical_attribution_and_legacy_request_round_trip() {
    let contract: Value = serde_json::from_str(include_str!(
        "../../../contracts/narration-profiles-v1.json"
    ))
    .unwrap();
    let attribution: NarrationProfileAttribution =
        serde_json::from_value(contract["attribution"].clone()).unwrap();
    attribution.validate().unwrap();
    assert_eq!(
        serde_json::to_value(attribution).unwrap(),
        contract["attribution"]
    );
    let legacy = json!({"text":"Hello", "language":"en", "voiceId":"voice", "speed":1.0, "textOptions":{"normalization":"basic","pronunciations":[],"chunking":"sentence","sentencePauseMs":120}});
    let request: SpeechSynthesisRequest = serde_json::from_value(legacy.clone()).unwrap();
    assert!(request.narration_profile.is_none());
    assert_eq!(serde_json::to_value(request).unwrap(), legacy);
}

#[test]
fn profiled_generation_reopen_history_and_stale_project_revision_are_exact() {
    let (root, initial) = setup();
    let core = EditorCore::new(
        initial
            .paths()
            .clone()
            .with_generated_media_root(root.path())
            .unwrap(),
    );
    let id = core
        .create_project("profiled", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .into_iter()
        .find(|t| t.track_type == TrackType::Audio)
        .unwrap()
        .id;
    let profile = core.create_narration_profile(settings()).unwrap();
    let request = core
        .resolve_narration_profile(application(&profile))
        .unwrap();
    let source = root.path().join("original.wav");
    std::fs::write(&source, b"synthetic original fixture").unwrap();
    let origin = GeneratedAssetOrigin::SpeechSynthesis(SpeechGeneration {
        request: request.clone(),
        provider_id: "fake".into(),
        model_id: "fake".into(),
        model_version: None,
        sample_rate_hz: 24000,
        generated_at_ms: 1,
        alignment: None,
    });
    let insert = CommitGeneratedAssetRequest {
        marker_policy: Default::default(),
        project_id: id.clone(),
        expected_revision: 0,
        path: source,
        track_id: track,
        start_ms: 0,
        duration_ms: 1000,
        display_name: "profiled".into(),
        origin: origin.clone(),
        probe: MediaProbeFacts::default(),
    };
    core.commit_generated_asset(insert.clone()).unwrap();
    let before = core.get_state(&id, None).unwrap();
    assert_eq!(
        core.commit_generated_asset(insert).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        serde_json::to_value(core.get_state(&id, None).unwrap().project).unwrap(),
        serde_json::to_value(before.project).unwrap()
    );
    core.update_narration_profile(&profile.id, 1, {
        let mut s = settings();
        s.speed = 1.4;
        s
    })
    .unwrap();
    std::fs::remove_file(core.paths().projects_root().join("narration-profiles.json")).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened.get_project(&id).unwrap().assets[0].origin,
        Some(origin.clone())
    );
    reopened.undo(&id, 1).unwrap();
    reopened.redo(&id, 2).unwrap();
    assert_eq!(
        reopened.get_project(&id).unwrap().assets[0].origin,
        Some(origin)
    );
    let dir = core.paths().projects_root().join(&id);
    let current = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    for retained in [false, true] {
        for attribution in [
            Value::Null,
            serde_json::to_value(&request.narration_profile).unwrap(),
        ] {
            let mut forged: Value = serde_json::from_slice(&current).unwrap();
            forged["schemaVersion"] = json!(45);
            forged["assets"][0]["origin"]["generation"]["request"]["narrationProfile"] =
                attribution;
            let (bad_current, bad_history) = if retained {
                (
                    current.clone(),
                    serde_json::to_vec(&json!({"undo":[forged],"redo":[]})).unwrap(),
                )
            } else {
                (serde_json::to_vec(&forged).unwrap(), history.clone())
            };
            std::fs::write(dir.join("project.json"), &bad_current).unwrap();
            std::fs::write(dir.join("history.json"), &bad_history).unwrap();
            assert!(reopened.get_project(&id).is_err());
            assert_eq!(
                std::fs::read(dir.join("project.json")).unwrap(),
                bad_current
            );
            assert_eq!(
                std::fs::read(dir.join("history.json")).unwrap(),
                bad_history
            );
        }
    }
}

#[test]
fn legacy_current_and_retained_history_migrate_atomically_and_future_versions_fail() {
    let (_root, core) = setup();
    let id = core
        .create_project("legacy", ProjectSettings::default())
        .unwrap()
        .project_id;
    let original = core.get_project(&id).unwrap();
    let dir = core.paths().projects_root().join(&id);
    let mut legacy = serde_json::to_value(&original).unwrap();
    legacy["schemaVersion"] = json!(45);
    let history = json!({"undo":[legacy.clone()],"redo":[legacy.clone()]});
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&history).unwrap(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(history["undo"][0]["schemaVersion"], PROJECT_SCHEMA_VERSION);
    assert_eq!(history["redo"][0]["schemaVersion"], PROJECT_SCHEMA_VERSION);
    let mut future = serde_json::to_value(&original).unwrap();
    future["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION + 1);
    let bytes = serde_json::to_vec(&future).unwrap();
    std::fs::write(dir.join("project.json"), &bytes).unwrap();
    let prior_history = std::fs::read(dir.join("history.json")).unwrap();
    assert!(core.get_project(&id).is_err());
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
    assert_eq!(
        std::fs::read(dir.join("history.json")).unwrap(),
        prior_history
    );
}

#[test]
fn invalid_or_null_attribution_rejects_without_legacy_fallback() {
    let (_root, core) = setup();
    let profile = core.create_narration_profile(settings()).unwrap();
    let resolved = core
        .resolve_narration_profile(application(&profile))
        .unwrap();
    let mut request = serde_json::to_value(&resolved).unwrap();
    request["narrationProfile"] = Value::Null;
    assert!(serde_json::from_value::<SpeechSynthesisRequest>(request).is_err());
    let mut bad = resolved;
    bad.narration_profile.as_mut().unwrap().version = 0;
    assert!(bad.validate().is_err());
}

#[test]
fn profile_entries_snapshot_the_exact_trimmed_effective_literals() {
    let (_root, core) = setup();
    let mut value = settings();
    value.text_options.pronunciations[0].term = " AI ".into();
    value.text_options.pronunciations[0].spoken = " a i ".into();
    let profile = core.create_narration_profile(value).unwrap();
    assert_eq!(profile.settings.text_options.pronunciations[0].term, "AI");
    assert_eq!(
        profile.settings.text_options.pronunciations[0].spoken,
        "a i"
    );
    let mut apply = application(&profile);
    let mut options = SpeechTextOptions::default();
    options.pronunciations.push(SpeechPronunciation {
        term: " Open ".into(),
        spoken: " editor ".into(),
    });
    apply.text_options = Some(options);
    let resolved = core.resolve_narration_profile(apply).unwrap();
    assert_eq!(resolved.text_options.pronunciations[0].term, "Open");
    assert_eq!(resolved.text_options.pronunciations[0].spoken, "editor");
    assert_eq!(resolved.narration_profile.unwrap().version, 1);
}

#[test]
fn profile_application_identifier_overrides_obey_the_profile_bounds() {
    let (_root, core) = setup();
    let profile = core.create_narration_profile(settings()).unwrap();
    for language in ["x".repeat(257), "es\n".into(), String::new()] {
        let mut apply = application(&profile);
        apply.language = Some(language);
        assert!(core.resolve_narration_profile(apply).is_err());
    }
    for voice in ["x".repeat(257), "voice\n".into(), String::new()] {
        let mut apply = application(&profile);
        apply.voice_id = Some(SpeechVoiceId(voice));
        assert!(core.resolve_narration_profile(apply).is_err());
    }
}

#[cfg(unix)]
#[test]
fn symlinked_profile_library_is_rejected_without_touching_its_target() {
    let (root, core) = setup();
    let target = root.path().join("outside-library.json");
    let bytes = b"{\"version\":1,\"profiles\":[]}";
    std::fs::write(&target, bytes).unwrap();
    std::fs::create_dir_all(core.paths().projects_root()).unwrap();
    let library = core.paths().projects_root().join("narration-profiles.json");
    std::os::unix::fs::symlink(&target, &library).unwrap();
    assert!(core.list_narration_profiles().is_err());
    assert!(core.create_narration_profile(settings()).is_err());
    assert_eq!(std::fs::read(target).unwrap(), bytes);
    assert!(
        std::fs::symlink_metadata(library)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
