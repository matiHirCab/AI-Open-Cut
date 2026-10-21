use super::*;

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/channel-style-packs-v1.json"
    )))
    .unwrap()
}

fn package() -> ChannelPackPackage {
    serde_json::from_value(fixture()["package"].clone()).unwrap()
}

fn core() -> (EditorCore, tempfile::TempDir) {
    let temp = tempfile::tempdir().unwrap();
    for name in ["projects", "media", "exports"] {
        std::fs::create_dir(temp.path().join(name)).unwrap();
    }
    let policy = PathPolicy::new(
        temp.path().join("projects"),
        [temp.path().join("media")],
        temp.path().join("exports"),
    )
    .unwrap();
    (EditorCore::new(policy), temp)
}

#[test]
fn independent_canonical_package_and_limits() {
    let f = fixture();
    assert_eq!(f["limits"]["packBytes"], crate::MAX_CHANNEL_PACK_BYTES);
    assert_eq!(
        f["limits"]["libraryBytes"],
        crate::MAX_CHANNEL_LIBRARY_BYTES
    );
    assert_eq!(f["limits"]["recipes"], crate::MAX_CHANNEL_RECIPES);
    assert_eq!(f["limits"]["items"], crate::MAX_CHANNEL_ITEMS);
    assert_eq!(f["limits"]["markers"], crate::MAX_CHANNEL_MARKERS);
    assert_eq!(f["limits"]["fonts"], crate::MAX_CHANNEL_FONTS);
    assert_eq!(f["limits"]["images"], crate::MAX_CHANNEL_IMAGES);
    assert_eq!(f["limits"]["placements"], crate::MAX_CHANNEL_PLACEMENTS);
    assert_eq!(f["limits"]["jsonDepth"], crate::MAX_CHANNEL_JSON_DEPTH);
    assert_eq!(f["limits"]["maxVersion"], crate::MAX_CHANNEL_VERSION);
    let raw = serde_json::to_string(&f["package"]).unwrap();
    let parsed = portable::decode_package(&raw).unwrap();
    assert_eq!(
        super::apply::component_digest(&parsed.recipes[0].definition).unwrap(),
        f["definitionDigest"].as_str().unwrap()
    );
    assert_eq!(serde_json::to_value(parsed).unwrap(), f["package"]);
}

#[test]
fn raw_duplicate_keys_are_rejected_in_every_native_map() {
    for key in ["title", "__proto__", "constructor", "toString"] {
        let once = format!(r#"{{"recipes":[{{"slotValues":{{"{key}":1}}}}]}}"#);
        assert!(portable::decode_unique_json(&once).is_ok(), "literal {key}");
        let twice = format!(r#"{{"recipes":[{{"slotValues":{{"{key}":1,"{key}":2}}}}]}}"#);
        assert!(
            portable::decode_unique_json(&twice).is_err(),
            "repeated {key}"
        );
    }
    assert!(portable::decode_unique_json(r#"{"name":"a","\u006eame":"b"}"#).is_err());
    assert!(portable::decode_unique_json("{} {}").is_err());
    assert!(portable::decode_unique_json("1e999").is_err());
}

#[test]
fn raw_size_depth_and_required_nullable_boundaries() {
    assert!(portable::decode_unique_json(&" ".repeat(crate::MAX_CHANNEL_PACK_BYTES + 1)).is_err());
    let at = format!("{}0{}", "[".repeat(64), "]".repeat(64));
    assert!(portable::decode_unique_json(&at).is_ok());
    let over = format!("[{}]", at);
    assert!(portable::decode_unique_json(&over).is_err());
    for field in ["captionRole", "narrationProfile"] {
        let mut value = fixture()["package"].clone();
        value.as_object_mut().unwrap().remove(field);
        assert!(
            portable::decode_package(&value.to_string()).is_err(),
            "missing {field}"
        );
    }
}

#[test]
fn portable_unknown_unsafe_future_graph_and_dependencies_fail_closed() {
    for (field, value) in [
        ("script", serde_json::json!("exec")),
        ("path", serde_json::json!("/tmp/art")),
        ("formatVersion", serde_json::json!(2)),
        ("componentSchemaVersion", serde_json::json!(47)),
    ] {
        let mut package = fixture()["package"].clone();
        package[field] = value;
        assert!(
            portable::decode_package(&package.to_string()).is_err(),
            "{field}"
        );
    }
    let mut p = package();
    p.recipes.push(p.recipes[0].clone());
    assert!(portable::validate_package(&p).is_err());
    let mut p = package();
    p.recipes[0].definition.width = 0;
    assert!(portable::validate_package(&p).is_err());
    let mut p = package();
    p.fonts.push(crate::ChannelFontFace {
        sha256: "0".repeat(64),
        size_bytes: 3,
        face_index: 0,
        data_base64: "Zm9v".into(),
    });
    assert!(portable::validate_package(&p).is_err());
    let mut p = package();
    p.caption_role = Some(crate::ChannelCaptionRole {
        recipe_key: "cp1_card".into(),
        text_slot_id: "missing".into(),
    });
    assert!(portable::validate_package(&p).is_err());
}

#[test]
fn library_names_versions_and_existing_instances_are_independent() {
    let (core, _temp) = core();
    assert!(core.list_channel_packs().unwrap().packs.is_empty());
    let raw = serde_json::to_string(&package()).unwrap();
    let first = core.import_channel_pack(" First ñ 🎬 ", &raw).unwrap();
    assert_eq!(first.package.name, "First ñ 🎬");
    assert_eq!(first.version, 1);
    let before = std::fs::read(core.paths.projects_root().join(LIBRARY_FILE)).unwrap();
    assert!(core.import_channel_pack("First ñ 🎬", &raw).is_err());
    assert_eq!(
        before,
        std::fs::read(core.paths.projects_root().join(LIBRARY_FILE)).unwrap()
    );
    let copy = core.import_channel_pack("Independent", &raw).unwrap();
    assert_ne!(first.id, copy.id);
    let mut next = first.package.clone();
    next.recipes[0].definition.name = "Explicit new default".into();
    let second = core
        .publish_channel_pack(Some((&first.id, 1)), next.clone())
        .unwrap();
    assert_eq!(second.version, 2);
    let saved = std::fs::read(core.paths.projects_root().join(LIBRARY_FILE)).unwrap();
    assert_eq!(
        core.publish_channel_pack(Some((&first.id, 1)), next)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        saved,
        std::fs::read(core.paths.projects_root().join(LIBRARY_FILE)).unwrap()
    );
    assert_eq!(
        core.get_channel_pack(&copy.id).unwrap().package.recipes[0]
            .definition
            .name,
        "Original card"
    );
    assert_eq!(core.list_channel_packs().unwrap().packs.len(), 2);
}

#[test]
fn corrupt_future_duplicate_library_does_not_get_overwritten() {
    let (core, _temp) = core();
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    for raw in ["not json", r#"{"version":2,"packs":[]}"#] {
        std::fs::write(&path, raw).unwrap();
        assert!(core.list_channel_packs().is_err());
        assert!(
            core.import_channel_pack("new", &serde_json::to_string(&package()).unwrap())
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), raw);
    }
}

fn capture_input(
    project_id: String,
    expected_revision: u64,
    component_id: String,
) -> crate::ChannelCaptureInput {
    crate::ChannelCaptureInput {
        project_id,
        expected_revision,
        component_ids: vec![component_id],
        name: "Captured original".into(),
        caption_role: None,
        narration_profile: None,
    }
}

#[test]
fn capture_is_current_revision_readonly_and_rejects_pending_recovery() {
    let (core, _temp) = core();
    let created = core
        .create_project("Original source", ProjectSettings::default())
        .unwrap();
    let edited = core
        .edit(
            &created.project_id,
            0,
            EditOperation::ComponentCreate {
                name: "Original component".into(),
                width: 320,
                height: 180,
                duration_ms: 1000,
                tracks: vec![],
                slots: Some(vec![]),
            },
        )
        .unwrap();
    let dir = core.paths.project_dir(&created.project_id).unwrap();
    let project = std::fs::read(project_path(&dir)).unwrap();
    let history = std::fs::read(history_path(&dir)).unwrap();
    let input = capture_input(created.project_id.clone(), 1, edited.changed_ids[0].clone());
    let captured = core.capture_channel_pack(input.clone()).unwrap();
    assert_eq!(captured.pack.version, 1);
    assert_eq!(project, std::fs::read(project_path(&dir)).unwrap());
    assert_eq!(history, std::fs::read(history_path(&dir)).unwrap());
    let mut stale = input.clone();
    stale.expected_revision = 0;
    assert_eq!(
        core.capture_channel_pack(stale).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    std::fs::write(
        transaction_path(&dir),
        b"pending journal must remain untouched",
    )
    .unwrap();
    assert_eq!(
        core.capture_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::ProjectRecoveryFailed
    );
    assert_eq!(
        std::fs::read(transaction_path(&dir)).unwrap(),
        b"pending journal must remain untouched"
    );
    assert_eq!(project, std::fs::read(project_path(&dir)).unwrap());
    assert_eq!(history, std::fs::read(history_path(&dir)).unwrap());
    std::fs::remove_file(transaction_path(&dir)).unwrap();
    let mut legacy: serde_json::Value = serde_json::from_slice(&project).unwrap();
    legacy["schemaVersion"] = 45.into();
    legacy.as_object_mut().unwrap().remove("narrationManifests");
    let legacy = serde_json::to_vec_pretty(&legacy).unwrap();
    std::fs::write(project_path(&dir), &legacy).unwrap();
    assert_eq!(
        core.capture_channel_pack(input).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(legacy, std::fs::read(project_path(&dir)).unwrap());
    assert_eq!(history, std::fs::read(history_path(&dir)).unwrap());
}

#[cfg(unix)]
#[test]
fn dangling_journal_and_library_links_fail_without_following_or_mutating() {
    let (core, temp) = core();
    let created = core
        .create_project("Source", ProjectSettings::default())
        .unwrap();
    let dir = core.paths.project_dir(&created.project_id).unwrap();
    std::os::unix::fs::symlink(temp.path().join("missing"), transaction_path(&dir)).unwrap();
    let error = core
        .capture_channel_pack(capture_input(created.project_id, 0, "missing".into()))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ProjectRecoveryFailed);
    let outside = temp.path().join("outside.json");
    std::fs::write(&outside, b"outside remains unchanged").unwrap();
    std::os::unix::fs::symlink(&outside, core.paths.projects_root().join(LIBRARY_FILE)).unwrap();
    assert!(core.list_channel_packs().is_err());
    assert!(
        core.import_channel_pack("new", &serde_json::to_string(&package()).unwrap())
            .is_err()
    );
    assert_eq!(
        std::fs::read(&outside).unwrap(),
        b"outside remains unchanged"
    );
}

fn apply_input(
    core: &EditorCore,
    pack: &ChannelPack,
    project_id: String,
) -> crate::ChannelApplyInput {
    let project = core.get_project(&project_id).unwrap();
    crate::ChannelApplyInput {
        project_id,
        expected_revision: project.revision,
        pack_id: pack.id.clone(),
        expected_version: pack.version,
        images: vec![],
        narration: crate::ChannelNarrationChoice::Saved {},
        application: crate::ChannelApplyMode::Instantiate {
            placements: vec![crate::ChannelPlacement {
                recipe_key: pack.package.recipes[0].key.clone(),
                track_id: project
                    .tracks
                    .iter()
                    .find(|track| track.track_type == TrackType::Overlay)
                    .unwrap()
                    .id
                    .clone(),
                start_ms: 0,
                trim_start_ms: 0,
                duration_ms: 1000,
                time_scale: 1.0,
                stagger_ms: 0,
                slot_values: BTreeMap::new(),
                transform: crate::Transform::default(),
                transform2d: None,
                hidden: false,
                z_index: 0,
                parent: None,
            }],
        },
    }
}

#[test]
fn independent_materialization_reapply_history_and_reopen_without_library() {
    let (core, _temp) = core();
    let pack = core
        .import_channel_pack("Reusable", &serde_json::to_string(&package()).unwrap())
        .unwrap();
    let first = core
        .create_project("A", ProjectSettings::default())
        .unwrap();
    let second = core
        .create_project("B", ProjectSettings::default())
        .unwrap();
    let first_apply = core
        .apply_channel_pack(apply_input(&core, &pack, first.project_id.clone()))
        .unwrap();
    let second_apply = core
        .apply_channel_pack(apply_input(&core, &pack, second.project_id.clone()))
        .unwrap();
    assert_eq!(first_apply.write.revision, 1);
    assert_eq!(second_apply.write.revision, 1);
    assert_ne!(
        first_apply.components[0].component_id,
        second_apply.components[0].component_id
    );
    assert!(first_apply.components[0].changed);
    assert!(first_apply.components[0].before_digest.is_none());
    let a = core.get_project(&first.project_id).unwrap();
    let b = core.get_project(&second.project_id).unwrap();
    assert_eq!(a.components[0].name, b.components[0].name);
    let mut newer = pack.package.clone();
    newer.recipes[0].definition.name = "Explicit reapply".into();
    let next = core
        .publish_channel_pack(Some((&pack.id, 1)), newer)
        .unwrap();
    assert_eq!(
        core.get_project(&first.project_id).unwrap().components[0].name,
        "Original card"
    );
    let mut input = apply_input(&core, &next, first.project_id.clone());
    input.application = crate::ChannelApplyMode::Reapply {
        components: vec![crate::ChannelComponentMapping {
            recipe_key: "cp1_card".into(),
            component_id: first_apply.components[0].component_id.clone(),
        }],
    };
    let reapplied = core.apply_channel_pack(input.clone()).unwrap();
    assert_eq!(reapplied.write.revision, 2);
    assert!(reapplied.components[0].changed);
    assert_ne!(
        reapplied.components[0].before_digest.as_ref().unwrap(),
        &reapplied.components[0].after_digest
    );
    assert_eq!(
        core.apply_channel_pack(input).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.get_project(&second.project_id).unwrap().components[0].name,
        "Original card"
    );
    std::fs::rename(
        core.paths.projects_root().join(LIBRARY_FILE),
        core.paths.projects_root().join("hidden-library"),
    )
    .unwrap();
    assert_eq!(
        core.get_project(&first.project_id).unwrap().components[0].name,
        "Explicit reapply"
    );
    core.undo(&first.project_id, 2).unwrap();
    assert_eq!(
        core.get_project(&first.project_id).unwrap().components[0].name,
        "Original card"
    );
    core.redo(&first.project_id, 3).unwrap();
    assert_eq!(
        core.get_project(&first.project_id).unwrap().components[0].name,
        "Explicit reapply"
    );
}

#[test]
fn library_update_is_serialized_and_rejected_writers_do_not_publish() {
    let (core, _temp) = core();
    let pack = core
        .import_channel_pack("Versioned", &serde_json::to_string(&package()).unwrap())
        .unwrap();
    let core = Arc::new(core);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let handles: Vec<_> = (0..2)
        .map(|index| {
            let core = core.clone();
            let barrier = barrier.clone();
            let pack = pack.clone();
            std::thread::spawn(move || {
                let mut next = pack.package;
                next.recipes[0].definition.name = format!("Writer {index}");
                barrier.wait();
                core.publish_channel_pack(Some((&pack.id, 1)), next)
            })
        })
        .collect();
    barrier.wait();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| result
                .as_ref()
                .is_err_and(|error| error.code == ErrorCode::RevisionConflict))
            .count(),
        1
    );
    assert_eq!(core.get_channel_pack(&pack.id).unwrap().version, 2);
}

#[test]
fn exact_library_pack_count_boundary_and_duplicate_raw_library_keys() {
    let (core, _temp) = core();
    let raw = serde_json::to_string(&package()).unwrap();
    for index in 0..crate::MAX_CHANNEL_PACKS {
        core.import_channel_pack(&format!("Pack {index}"), &raw)
            .unwrap();
    }
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    let before = std::fs::read(&path).unwrap();
    assert!(core.import_channel_pack("overflow", &raw).is_err());
    assert_eq!(before, std::fs::read(&path).unwrap());
    let mut library = String::from_utf8(before).unwrap();
    library = library.replacen("\"slots\": []", "\"slots\": [], \"slots\": []", 1);
    std::fs::write(&path, &library).unwrap();
    assert!(core.list_channel_packs().is_err());
    assert_eq!(library, std::fs::read_to_string(&path).unwrap());
}

fn solid_package() -> ChannelPackPackage {
    let mut p = package();
    p.recipes[0].definition.tracks.push(serde_json::from_value(serde_json::json!({
        "id":"original_track", "name":"Original color", "trackType":"video",
        "items":[{"type":"solid_color","id":"original_color","color":"#17324D","startMs":0,"durationMs":1000,"keyframes":[]}]
    })).unwrap());
    p.recipes[0].definition.markers.push(crate::Marker {
        id: "original_marker".into(),
        name: "Cue".into(),
        scope: "component:cp1_card".into(),
        time_ms: 500,
        kind: crate::MarkerKind::Cue,
    });
    p
}

#[test]
fn equal_nonempty_reapply_preserves_scoped_ids_markers_and_native_undo_semantics() {
    let (core, _temp) = core();
    let pack = core
        .import_channel_pack(
            "Stable identities",
            &serde_json::to_string(&solid_package()).unwrap(),
        )
        .unwrap();
    let source = core
        .create_project("Target", ProjectSettings::default())
        .unwrap();
    let first = core
        .apply_channel_pack(apply_input(&core, &pack, source.project_id.clone()))
        .unwrap();
    let before = core.get_project(&source.project_id).unwrap().components[0].clone();
    assert!(first.write.changed_ids.contains(&before.markers[0].id));
    let mut again = apply_input(&core, &pack, source.project_id.clone());
    again.application = crate::ChannelApplyMode::Reapply {
        components: vec![crate::ChannelComponentMapping {
            recipe_key: "cp1_card".into(),
            component_id: before.id.clone(),
        }],
    };
    let reapplied = core.apply_channel_pack(again).unwrap();
    assert_eq!(reapplied.write.revision, 2);
    assert!(!reapplied.components[0].changed);
    assert_eq!(
        reapplied.components[0].before_digest.as_ref(),
        Some(&reapplied.components[0].after_digest)
    );
    let after = core.get_project(&source.project_id).unwrap().components[0].clone();
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after).unwrap()
    );
    core.undo(&source.project_id, 2).unwrap();
    assert_eq!(core.get_project(&source.project_id).unwrap().revision, 3);
    core.undo(&source.project_id, 3).unwrap();
    assert!(
        core.get_project(&source.project_id)
            .unwrap()
            .components
            .is_empty()
    );
}

#[test]
fn package_depth_budget_is_unchanged_by_the_three_library_containers() {
    use serde::de::DeserializeSeed;
    let at = format!("{}0{}", "[".repeat(64), "]".repeat(64));
    let wrapped = format!(r#"{{"packs":[{{"package":{at}}}]}}"#);
    assert!(portable::decode_unique_json(&at).is_ok());
    let mut decoder = serde_json::Deserializer::from_str(&wrapped);
    assert!(
        portable::UniqueValue(0, crate::MAX_CHANNEL_JSON_DEPTH + 3)
            .deserialize(&mut decoder)
            .is_ok()
    );
    let over = format!(r#"{{"packs":[{{"package":[{at}]}}]}}"#);
    let mut decoder = serde_json::Deserializer::from_str(&over);
    assert!(
        portable::UniqueValue(0, crate::MAX_CHANNEL_JSON_DEPTH + 3)
            .deserialize(&mut decoder)
            .is_err()
    );
}

#[test]
fn all_new_tagged_variants_reject_unknown_fields() {
    for mode in ["saved", "disabled"] {
        let valid = serde_json::json!({"mode":mode});
        assert!(serde_json::from_value::<crate::ChannelNarrationChoice>(valid).is_ok());
        let invalid = serde_json::json!({"mode":mode,"script":"forbidden"});
        assert!(
            serde_json::from_value::<crate::ChannelNarrationChoice>(invalid).is_err(),
            "extra in {mode}"
        );
    }
    let extra = serde_json::json!({"mode":"replacement","reference":{"profileId":"00000000-0000-4000-8000-000000000207","expectedVersion":1},"script":"forbidden"});
    assert!(serde_json::from_value::<crate::ChannelNarrationChoice>(extra).is_err());
    for (mode, field) in [("instantiate", "placements"), ("reapply", "components")] {
        let mut extra = serde_json::json!({"mode":mode,"script":"forbidden"});
        extra[field] = serde_json::json!([]);
        assert!(serde_json::from_value::<crate::ChannelApplyMode>(extra).is_err());
    }
}

fn captured_text_pack(core: &EditorCore, locked: bool) -> ChannelPack {
    let created = core
        .create_project("Pinned source", ProjectSettings::default())
        .unwrap();
    let text = crate::TextItem {
        id: "caption_text".into(),
        text: "MontiPKM original".into(),
        document: crate::RichTextDocument::plain("MontiPKM original".into()),
        font_binding: None,
        start_ms: 0,
        duration_ms: 1000,
        font_size: 24,
        color: "#FFFFFF".into(),
        font_family: None,
        font_path: None,
        style: crate::TextStyle::default(),
        visual_properties: crate::VisualProperties::default(),
        keyframes: vec![],
    };
    let track = Track {
        id: "caption_track".into(),
        name: "Original caption".into(),
        track_type: TrackType::Overlay,
        locked,
        hidden: false,
        muted: false,
        audio_role: AudioTrackRole::Unassigned,
        audio_bus_id: None,
        ducking: None,
        items: vec![TimelineItem::Text(text)],
    };
    let slot = crate::TemplateSlot {
        id: "caption".into(),
        name: "Caption".into(),
        kind: crate::SlotKind::Text,
        required: false,
        default_value: Some(crate::SlotValue::Text("MontiPKM original".into())),
        binding: crate::SlotBinding {
            target_layer_id: "caption_text".into(),
            property: crate::SlotProperty::TextDocument,
        },
        constraints: crate::SlotConstraints::default(),
    };
    let edited = core
        .edit(
            &created.project_id,
            0,
            EditOperation::ComponentCreate {
                name: "Caption recipe".into(),
                width: 320,
                height: 180,
                duration_ms: 1000,
                tracks: vec![track],
                slots: Some(vec![slot]),
            },
        )
        .unwrap();
    let dir = core.project_directory(&created.project_id).unwrap();
    let before_project = std::fs::read(project_path(&dir)).unwrap();
    let before_history = std::fs::read(history_path(&dir)).unwrap();
    let mut capture = capture_input(created.project_id, 1, edited.changed_ids[0].clone());
    capture.caption_role = Some(crate::ChannelCaptionRole {
        recipe_key: edited.changed_ids[0].clone(),
        text_slot_id: "caption".into(),
    });
    let pack = core.capture_channel_pack(capture).unwrap().pack;
    assert_eq!(before_project, std::fs::read(project_path(&dir)).unwrap());
    assert_eq!(before_history, std::fs::read(history_path(&dir)).unwrap());
    let oracle = fixture()["independentRenderOracle"]["fonts"]
        .as_array()
        .unwrap()
        .clone();
    for font in &pack.package.fonts {
        assert!(
            oracle
                .iter()
                .any(|face| face["sha256"] == font.sha256 && face["sizeBytes"] == font.size_bytes)
        );
    }
    pack
}

#[test]
fn pinned_font_apply_never_uses_target_default_selectors_and_reapply_preserves_overrides() {
    let (core, _temp) = core();
    let pack = captured_text_pack(&core, false);
    let target = core
        .create_project("Independent target", ProjectSettings::default())
        .unwrap();
    let mut input = apply_input(&core, &pack, target.project_id.clone());
    if let crate::ChannelApplyMode::Instantiate { placements } = &mut input.application {
        placements[0].slot_values.insert(
            "caption".into(),
            crate::SlotValue::Text("Personal override".into()),
        );
    }
    let isolated = EditorCore::new(core.paths().clone()).with_font_config(crate::FontConfig {
        roots: vec![],
        default_path: Some(
            core.paths
                .projects_root()
                .join("MISSING-font-must-never-be-read.ttf"),
        ),
    });
    let first = isolated.apply_channel_pack(input).unwrap();
    assert!(first.caption_role.is_some());
    let before = isolated.get_project(&target.project_id).unwrap();
    assert!(!before.fonts.is_empty());
    let mut next = pack.package.clone();
    next.recipes[0].definition.slots[0].default_value =
        Some(crate::SlotValue::Text("New default".into()));
    let next = core
        .publish_channel_pack(Some((&pack.id, 1)), next)
        .unwrap();
    let mut again = apply_input(&isolated, &next, target.project_id.clone());
    again.application = crate::ChannelApplyMode::Reapply {
        components: vec![crate::ChannelComponentMapping {
            recipe_key: next.package.recipes[0].key.clone(),
            component_id: first.components[0].component_id.clone(),
        }],
    };
    let reapplied = isolated.apply_channel_pack(again.clone()).unwrap();
    assert!(reapplied.components[0].changed);
    let project = isolated.get_project(&target.project_id).unwrap();
    let instance = project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .find_map(|item| match item {
            TimelineItem::ComponentInstance(instance) => Some(instance),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        instance.slot_values["caption"],
        crate::SlotValue::Text("Personal override".into())
    );
    let mut incompatible = next.package.clone();
    incompatible.recipes[0].definition.slots.clear();
    incompatible.caption_role = None;
    let incompatible = core
        .publish_channel_pack(Some((&next.id, 2)), incompatible)
        .unwrap();
    again.expected_revision = 2;
    again.expected_version = incompatible.version;
    let dir = core.project_directory(&target.project_id).unwrap();
    let bytes = std::fs::read(project_path(&dir)).unwrap();
    assert!(isolated.apply_channel_pack(again).is_err());
    assert_eq!(bytes, std::fs::read(project_path(&dir)).unwrap());
}

#[test]
fn unchanged_locked_pinned_track_reapplies_but_a_trusted_font_only_change_is_rejected() {
    let (core, _temp) = core();
    let pack = captured_text_pack(&core, true);
    let target = core
        .create_project("Locked target", ProjectSettings::default())
        .unwrap();
    let first = core
        .apply_channel_pack(apply_input(&core, &pack, target.project_id.clone()))
        .unwrap();
    let mut again = apply_input(&core, &pack, target.project_id.clone());
    again.application = crate::ChannelApplyMode::Reapply {
        components: vec![crate::ChannelComponentMapping {
            recipe_key: pack.package.recipes[0].key.clone(),
            component_id: first.components[0].component_id.clone(),
        }],
    };
    let same = core.apply_channel_pack(again.clone()).unwrap();
    assert!(!same.components[0].changed);
    assert_eq!(same.write.revision, 2);
    let mut next = pack.package.clone();
    let TimelineItem::Text(text) = &mut next.recipes[0].definition.tracks[0].items[0] else {
        unreachable!()
    };
    let binding = text.font_binding.as_mut().unwrap();
    let removed = binding.regular.clone();
    binding.regular = binding.bold.clone();
    next.fonts.retain(|font| font.sha256 != removed);
    let next = core
        .publish_channel_pack(Some((&pack.id, 1)), next)
        .unwrap();
    again.expected_revision = 2;
    again.expected_version = next.version;
    let dir = core.project_directory(&target.project_id).unwrap();
    let bytes = std::fs::read(project_path(&dir)).unwrap();
    assert_eq!(
        core.apply_channel_pack(again).unwrap_err().code,
        ErrorCode::TrackLocked
    );
    assert_eq!(bytes, std::fs::read(project_path(&dir)).unwrap());
}

#[test]
fn optional_profile_resolution_is_explicit_and_never_runs_inference() {
    let (core, _temp) = core();
    let mut p = package();
    p.narration_profile = Some(crate::ChannelProfileRef {
        profile_id: "00000000-0000-4000-8000-000000000207".into(),
        expected_version: 1,
    });
    let pack = core.publish_channel_pack(None, p).unwrap();
    let target = core
        .create_project("Profile target", ProjectSettings::default())
        .unwrap();
    let mut input = apply_input(&core, &pack, target.project_id.clone());
    assert_eq!(
        core.apply_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(core.get_project(&target.project_id).unwrap().revision, 0);
    let profile_fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/narration-profiles-v1.json"
    )))
    .unwrap();
    let settings: crate::NarrationProfileSettings =
        serde_json::from_value(profile_fixture["settings"].clone()).unwrap();
    let profile = core.create_narration_profile(settings.clone()).unwrap();
    input.narration = crate::ChannelNarrationChoice::Replacement {
        reference: crate::ChannelProfileRef {
            profile_id: profile.id.clone(),
            expected_version: 1,
        },
    };
    let first = core.apply_channel_pack(input.clone()).unwrap();
    assert_eq!(first.narration_profile.unwrap().settings, settings);
    let mut next = settings;
    next.speed = 1.3;
    core.update_narration_profile(&profile.id, 1, next).unwrap();
    input.expected_revision = 1;
    assert_eq!(
        core.apply_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(core.get_project(&target.project_id).unwrap().revision, 1);
    input.narration = crate::ChannelNarrationChoice::Disabled {};
    let disabled = core.apply_channel_pack(input).unwrap();
    assert!(disabled.narration_profile.is_none());
}

#[test]
fn explicit_artwork_bindings_reject_missing_wrong_and_corrupt_target_content() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let (core, temp) = core();
    let raw = STANDARD
        .decode(
            fixture()["independentRenderOracle"]["image"]["pngBase64"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
    let file = temp.path().join("media/original.png");
    std::fs::write(&file, &raw).unwrap();
    let probe = MediaProbeFacts {
        has_video: true,
        video_width: Some(16),
        video_height: Some(16),
        format_name: Some("png_pipe".into()),
        video_codec: Some("png".into()),
        ..Default::default()
    };
    let source = core
        .create_project("Artwork source", ProjectSettings::default())
        .unwrap();
    let artwork = core
        .import_asset(
            &source.project_id,
            0,
            &file,
            MediaType::Image,
            probe.clone(),
        )
        .unwrap();
    let track: Track = serde_json::from_value(serde_json::json!({"id":"image_track","name":"Original artwork","trackType":"video","items":[{"type":"media","id":"image_item","assetId":artwork.changed_ids[0],"startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}]})).unwrap();
    let definition = core
        .edit(
            &source.project_id,
            1,
            EditOperation::ComponentCreate {
                name: "Artwork card".into(),
                width: 320,
                height: 180,
                duration_ms: 1000,
                tracks: vec![track],
                slots: Some(vec![]),
            },
        )
        .unwrap();
    let pack = core
        .capture_channel_pack(capture_input(
            source.project_id,
            2,
            definition.changed_ids[0].clone(),
        ))
        .unwrap()
        .pack;
    assert_eq!(pack.package.images.len(), 1);
    let target = core
        .create_project("Artwork target", ProjectSettings::default())
        .unwrap();
    let imported = core
        .import_asset(
            &target.project_id,
            0,
            &file,
            MediaType::Image,
            probe.clone(),
        )
        .unwrap();
    let mut input = apply_input(&core, &pack, target.project_id.clone());
    let dir = core.project_directory(&target.project_id).unwrap();
    let before = std::fs::read(project_path(&dir)).unwrap();
    assert_eq!(
        core.apply_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::DependencyUnavailable
    );
    input.images = vec![crate::ChannelImageBinding {
        dependency_key: pack.package.images[0].key.clone(),
        asset_id: "missing".into(),
    }];
    assert_eq!(
        core.apply_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::AssetNotFound
    );
    assert_eq!(before, std::fs::read(project_path(&dir)).unwrap());
    let wrong = temp.path().join("media/wrong.png");
    let mut wrong_bytes = raw;
    wrong_bytes.extend(b"different-content");
    std::fs::write(&wrong, wrong_bytes).unwrap();
    let wrong_asset = core
        .import_asset(&target.project_id, 1, &wrong, MediaType::Image, probe)
        .unwrap();
    input.expected_revision = 2;
    input.images[0].asset_id = wrong_asset.changed_ids[0].clone();
    assert_eq!(
        core.apply_channel_pack(input.clone()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    input.images[0].asset_id = imported.changed_ids[0].clone();
    let project = core.get_project(&target.project_id).unwrap();
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.id == imported.changed_ids[0])
        .unwrap();
    std::fs::write(dir.join(&asset.project_relative_path), b"corrupted").unwrap();
    assert_eq!(
        core.apply_channel_pack(input).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(
        std::fs::read(project_path(&dir)).unwrap(),
        serde_json::to_vec_pretty(&project).unwrap()
    );
}

#[test]
fn font_publication_faults_preserve_whole_generations_and_owned_resource_recovery() {
    use crate::persistence::PersistencePhase;
    for phase in [
        PersistencePhase::BeforeFontPublish,
        PersistencePhase::AfterFontPublish,
        PersistencePhase::BeforeJournal,
        PersistencePhase::AfterJournal,
        PersistencePhase::AfterProject,
        PersistencePhase::AfterHistory,
        PersistencePhase::AfterDraftUpdates,
        PersistencePhase::AfterDraftCleanup,
        PersistencePhase::AfterJournalCleanup,
        PersistencePhase::GarbageCollection,
    ] {
        let (core, _temp) = core();
        let pack = captured_text_pack(&core, false);
        let target = core
            .create_project("Fault target", ProjectSettings::default())
            .unwrap();
        let input = apply_input(&core, &pack, target.project_id.clone());
        let dir = core.project_directory(&target.project_id).unwrap();
        let before_project = std::fs::read(project_path(&dir)).unwrap();
        let before_history = std::fs::read(history_path(&dir)).unwrap();
        core.persistence_faults.inject(phase);
        let result = core.apply_channel_pack(input);
        if matches!(
            phase,
            PersistencePhase::BeforeFontPublish
                | PersistencePhase::AfterFontPublish
                | PersistencePhase::BeforeJournal
        ) {
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::InternalError,
                "{phase:?}"
            );
            assert_eq!(
                before_project,
                std::fs::read(project_path(&dir)).unwrap(),
                "{phase:?}"
            );
            assert_eq!(before_history, std::fs::read(history_path(&dir)).unwrap());
            assert!(!transaction_path(&dir).exists());
            assert!(
                std::fs::read_dir(dir.join("fonts"))
                    .map(|mut entries| entries.next().is_none())
                    .unwrap_or(true)
            );
        } else {
            let result = result.unwrap();
            assert_eq!(result.write.revision, 1);
            if !matches!(
                phase,
                PersistencePhase::AfterJournalCleanup | PersistencePhase::GarbageCollection
            ) {
                assert!(
                    result
                        .write
                        .warnings
                        .contains(&PERSISTENCE_RECOVERY_PENDING.into()),
                    "{phase:?}"
                );
            }
            let recovered = core.get_project(&target.project_id).unwrap();
            assert_eq!(recovered.revision, 1);
            assert_eq!(recovered.components.len(), 1);
            assert!(!recovered.fonts.is_empty());
            assert!(!transaction_path(&dir).exists());
            core.undo(&target.project_id, 1).unwrap();
            assert!(
                core.get_project(&target.project_id)
                    .unwrap()
                    .components
                    .is_empty()
            );
        }
    }
}

#[test]
fn duplicate_visible_marker_tuples_keep_distinct_identity_through_capture_import_apply_and_reapply()
{
    let (core, _temp) = core();
    let mut p = solid_package();
    let mut duplicate = p.recipes[0].definition.markers[0].clone();
    duplicate.id = "second_original_marker".into();
    p.recipes[0].definition.markers.push(duplicate);
    let pack = core.publish_channel_pack(None, p).unwrap();
    let source = core
        .create_project("Marker source", ProjectSettings::default())
        .unwrap();
    let first = core
        .apply_channel_pack(apply_input(&core, &pack, source.project_id.clone()))
        .unwrap();
    let captured = core
        .capture_channel_pack(capture_input(
            source.project_id,
            1,
            first.components[0].component_id.clone(),
        ))
        .unwrap()
        .pack;
    let imported = core
        .import_channel_pack(
            "Independent marker copy",
            &serde_json::to_string(&captured.package).unwrap(),
        )
        .unwrap();
    let target = core
        .create_project("Marker target", ProjectSettings::default())
        .unwrap();
    let applied = core
        .apply_channel_pack(apply_input(&core, &imported, target.project_id.clone()))
        .unwrap();
    let before = core.get_project(&target.project_id).unwrap().components[0].clone();
    assert_eq!(before.markers.len(), 2);
    assert_ne!(before.markers[0].id, before.markers[1].id);
    assert_eq!(before.markers[0].name, before.markers[1].name);
    assert_eq!(before.markers[0].time_ms, before.markers[1].time_ms);
    for marker in &before.markers {
        assert!(applied.write.changed_ids.contains(&marker.id));
    }
    let mut input = apply_input(&core, &imported, target.project_id.clone());
    input.application = crate::ChannelApplyMode::Reapply {
        components: vec![crate::ChannelComponentMapping {
            recipe_key: imported.package.recipes[0].key.clone(),
            component_id: before.id.clone(),
        }],
    };
    let again = core.apply_channel_pack(input).unwrap();
    assert!(!again.components[0].changed);
    assert_eq!(again.write.revision, 2);
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(&core.get_project(&target.project_id).unwrap().components[0]).unwrap()
    );
}

#[test]
fn nested_native_fields_remain_closed_without_rejecting_literal_slot_keys() {
    let original = serde_json::to_value(solid_package()).unwrap();
    for path in [
        vec!["recipes", "0", "definition"],
        vec!["recipes", "0", "definition", "tracks", "0"],
        vec!["recipes", "0", "definition", "tracks", "0", "items", "0"],
        vec!["recipes", "0", "definition", "markers", "0"],
    ] {
        let mut value = original.clone();
        let mut selected = &mut value;
        for part in path {
            selected = if let Ok(index) = part.parse::<usize>() {
                &mut selected[index]
            } else {
                &mut selected[part]
            };
        }
        selected["script"] = "execute".into();
        assert!(portable::decode_package(&value.to_string()).is_err());
    }
}

#[test]
fn legacy_target_migration_joins_the_same_revision_history_and_font_transaction() {
    use crate::persistence::PersistencePhase;
    let (core, _temp) = core();
    let pack = captured_text_pack(&core, false);
    for schema in [18, 45] {
        for fault in [None, Some(PersistencePhase::AfterFontPublish)] {
            let target = core
                .create_project("Legacy target", ProjectSettings::default())
                .unwrap();
            let mut input = apply_input(&core, &pack, target.project_id.clone());
            let dir = core.project_directory(&target.project_id).unwrap();
            let mut old = core.get_project(&target.project_id).unwrap();
            old.schema_version = schema;
            let old_bytes = serde_json::to_vec_pretty(&old).unwrap();
            std::fs::write(project_path(&dir), &old_bytes).unwrap();
            let old_history = std::fs::read(history_path(&dir)).unwrap();
            input.expected_revision = 1;
            assert_eq!(
                core.apply_channel_pack(input.clone()).unwrap_err().code,
                ErrorCode::RevisionConflict
            );
            assert_eq!(old_bytes, std::fs::read(project_path(&dir)).unwrap());
            input.expected_revision = 0;
            if let Some(phase) = fault {
                core.persistence_faults.inject(phase);
                assert_eq!(
                    core.apply_channel_pack(input).unwrap_err().code,
                    ErrorCode::InternalError
                );
                assert_eq!(old_bytes, std::fs::read(project_path(&dir)).unwrap());
                assert_eq!(old_history, std::fs::read(history_path(&dir)).unwrap());
                assert!(!transaction_path(&dir).exists());
            } else {
                assert_eq!(core.apply_channel_pack(input).unwrap().write.revision, 1);
                let current = core.get_project(&target.project_id).unwrap();
                assert_eq!(current.schema_version, PROJECT_SCHEMA_VERSION);
                assert!(!current.fonts.is_empty());
                let history: History =
                    read_json(core.storage.as_ref(), &history_path(&dir)).unwrap();
                assert_eq!(history.undo.len(), 1);
                assert_eq!(history.undo[0].schema_version, PROJECT_SCHEMA_VERSION);
                core.undo(&target.project_id, 1).unwrap();
                assert_eq!(
                    core.get_project(&target.project_id).unwrap().schema_version,
                    PROJECT_SCHEMA_VERSION
                );
            }
        }
    }
}

fn nested_literal_package() -> ChannelPackPackage {
    let mut p = package();
    let mut child = solid_package().recipes.remove(0);
    child.key = "child".into();
    child.definition.id = "child".into();
    child.definition.markers.clear();
    let original = child.definition.tracks[0].items[0].clone();
    child.definition.tracks[0].items.clear();
    for (index, key) in ["__proto__", "constructor", "toString"]
        .into_iter()
        .enumerate()
    {
        let mut item = original.clone();
        if let TimelineItem::SolidColor(value) = &mut item {
            value.id = format!("paint_{index}");
            value.visual_properties.stack_order = index as u32;
        }
        child.definition.tracks[0].items.push(item);
        child.definition.slots.push(crate::TemplateSlot {
            id: key.into(),
            name: format!("Visible {index}"),
            kind: crate::SlotKind::Boolean,
            required: false,
            default_value: Some(crate::SlotValue::Boolean(false)),
            binding: crate::SlotBinding {
                target_layer_id: format!("paint_{index}"),
                property: crate::SlotProperty::VisualHidden,
            },
            constraints: crate::SlotConstraints::default(),
        });
    }
    p.recipes[0].key = "parent".into();
    p.recipes[0].definition.id = "parent".into();
    p.recipes[0].definition.tracks.push(serde_json::from_value(serde_json::json!({"id":"nested_track","name":"Nested scene","trackType":"overlay","items":[{"type":"component_instance","id":"nested_instance","componentId":"child","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1.0,"slotValues":{"__proto__":{"type":"boolean","value":true},"constructor":{"type":"boolean","value":false},"toString":{"type":"boolean","value":true}}}]})).unwrap());
    // Parent deliberately precedes child: portable recipe array order is not a dependency order.
    p.recipes.push(child);
    p
}

#[test]
fn native_nested_capture_closure_and_literal_slots_roundtrip_without_partial_mapping_effects() {
    let (core, _temp) = core();
    let portable = nested_literal_package();
    let pack = core
        .import_channel_pack(
            "Nested original",
            &serde_json::to_string(&portable).unwrap(),
        )
        .unwrap();
    let source = core
        .create_project("Nested source", ProjectSettings::default())
        .unwrap();
    let applied = core
        .apply_channel_pack(apply_input(&core, &pack, source.project_id.clone()))
        .unwrap();
    let parent = applied
        .components
        .iter()
        .find(|v| v.recipe_key == "parent")
        .unwrap();
    let child = applied
        .components
        .iter()
        .find(|v| v.recipe_key == "child")
        .unwrap();
    let dir = core.project_directory(&source.project_id).unwrap();
    let before = (
        std::fs::read(project_path(&dir)).unwrap(),
        std::fs::read(history_path(&dir)).unwrap(),
    );
    let captured = core
        .capture_channel_pack(capture_input(
            source.project_id.clone(),
            1,
            parent.component_id.clone(),
        ))
        .unwrap()
        .pack;
    assert_eq!(captured.package.recipes.len(), 2);
    assert!(
        captured
            .package
            .recipes
            .iter()
            .any(|v| v.key == child.component_id)
    );
    assert_eq!(
        before,
        (
            std::fs::read(project_path(&dir)).unwrap(),
            std::fs::read(history_path(&dir)).unwrap()
        )
    );
    let copied = core
        .import_channel_pack(
            "Nested independent copy",
            &serde_json::to_string(&captured.package).unwrap(),
        )
        .unwrap();
    let target = core
        .create_project("Independent nested target", ProjectSettings::default())
        .unwrap();
    let mut request = apply_input(&core, &copied, target.project_id.clone());
    if let crate::ChannelApplyMode::Instantiate { placements } = &mut request.application {
        placements[0].recipe_key = parent.component_id.clone();
    }
    let second = core.apply_channel_pack(request).unwrap();
    let current = core.get_project(&target.project_id).unwrap();
    let parent_id = second
        .components
        .iter()
        .find(|v| v.recipe_key == parent.component_id)
        .unwrap()
        .component_id
        .clone();
    let child_id = second
        .components
        .iter()
        .find(|v| v.recipe_key == child.component_id)
        .unwrap()
        .component_id
        .clone();
    assert_ne!(parent_id, parent.component_id);
    assert_ne!(child_id, child.component_id);
    let nested = current
        .components
        .iter()
        .find(|v| v.id == parent_id)
        .unwrap()
        .tracks[0]
        .items
        .iter()
        .find_map(|v| {
            if let TimelineItem::ComponentInstance(v) = v {
                Some(v)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(nested.component_id, child_id);
    for (key, expected) in [
        ("__proto__", true),
        ("constructor", false),
        ("toString", true),
    ] {
        assert_eq!(
            nested.slot_values.get(key),
            Some(&crate::SlotValue::Boolean(expected))
        );
    }
    let mappings: Vec<_> = second
        .components
        .iter()
        .map(|v| crate::ChannelComponentMapping {
            recipe_key: v.recipe_key.clone(),
            component_id: v.component_id.clone(),
        })
        .collect();
    let mut request = apply_input(&core, &copied, target.project_id.clone());
    request.application = crate::ChannelApplyMode::Reapply {
        components: mappings.clone(),
    };
    let again = core.apply_channel_pack(request.clone()).unwrap();
    assert!(again.components.iter().all(|v| !v.changed));
    let target_dir = core.project_directory(&target.project_id).unwrap();
    let before = (
        std::fs::read(project_path(&target_dir)).unwrap(),
        std::fs::read(history_path(&target_dir)).unwrap(),
    );
    request.expected_revision = 2;
    request.application = crate::ChannelApplyMode::Reapply {
        components: mappings[..1].to_vec(),
    };
    assert!(core.apply_channel_pack(request).is_err());
    assert_eq!(
        before,
        (
            std::fs::read(project_path(&target_dir)).unwrap(),
            std::fs::read(history_path(&target_dir)).unwrap()
        )
    );
}

fn empty_recipes(count: usize) -> ChannelPackPackage {
    let mut p = package();
    let base = p.recipes[0].clone();
    p.recipes = (0..count)
        .map(|i| {
            let mut r = base.clone();
            r.key = format!("recipe_{i}");
            r.definition.id = r.key.clone();
            r
        })
        .collect();
    p
}

fn reject_import_unchanged(core: &EditorCore, p: &ChannelPackPackage) -> CoreError {
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    let before = std::fs::read(&path).ok();
    let error = core
        .import_channel_pack("Rejected", &serde_json::to_string(p).unwrap())
        .unwrap_err();
    assert_eq!(before, std::fs::read(path).ok());
    error
}

#[test]
fn portable_recipe_item_marker_and_raw_byte_boundaries_are_behavioral() {
    let (core, _temp) = core();
    let pack = core
        .import_channel_pack(
            "Sixteen",
            &serde_json::to_string(&empty_recipes(16)).unwrap(),
        )
        .unwrap();
    assert_eq!(pack.package.recipes.len(), 16);
    reject_import_unchanged(&core, &empty_recipes(17));
    let mut p = solid_package();
    p.recipes[0].definition.markers.clear();
    let original = p.recipes[0].definition.tracks[0].items[0].clone();
    p.recipes[0].definition.tracks[0].items = (0..1024)
        .map(|index| {
            let mut item = original.clone();
            if let TimelineItem::SolidColor(v) = &mut item {
                v.id = format!("item_{index}");
                v.visual_properties.stack_order = index as u32;
            }
            item
        })
        .collect();
    let accepted = core
        .import_channel_pack("1024 original paints", &serde_json::to_string(&p).unwrap())
        .unwrap();
    assert_eq!(
        accepted.package.recipes[0].definition.tracks[0].items.len(),
        1024
    );
    let mut extra = original;
    if let TimelineItem::SolidColor(v) = &mut extra {
        v.id = "item_1024".into();
        v.visual_properties.stack_order = 1024;
    }
    p.recipes[0].definition.tracks[0].items.push(extra);
    reject_import_unchanged(&core, &p);
    let mut p = solid_package();
    let marker = p.recipes[0].definition.markers[0].clone();
    p.recipes[0].definition.markers = (0..32)
        .map(|index| {
            let mut v = marker.clone();
            v.id = format!("marker_{index}");
            v
        })
        .collect();
    core.import_channel_pack("32 cues", &serde_json::to_string(&p).unwrap())
        .unwrap();
    let mut extra = marker;
    extra.id = "marker_32".into();
    p.recipes[0].definition.markers.push(extra);
    reject_import_unchanged(&core, &p);
    let mut exact = serde_json::to_string(&package()).unwrap();
    exact.extend(std::iter::repeat_n(' ', 8 * 1024 * 1024 - exact.len()));
    core.import_channel_pack("Exact raw byte bound", &exact)
        .unwrap();
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    let before = std::fs::read(&path).unwrap();
    exact.push(' ');
    assert!(
        core.import_channel_pack("Too many raw bytes", &exact)
            .is_err()
    );
    assert_eq!(before, std::fs::read(path).unwrap());
}

#[test]
fn placement_and_generated_native_operation_bounds_preserve_authoritative_generations() {
    let (core, _temp) = core();
    let p = package();
    let pack = core
        .import_channel_pack("Placement bound", &serde_json::to_string(&p).unwrap())
        .unwrap();
    let target = core
        .create_project("32 placements", ProjectSettings::default())
        .unwrap();
    let mut input = apply_input(&core, &pack, target.project_id.clone());
    if let crate::ChannelApplyMode::Instantiate { placements } = &mut input.application {
        placements.resize(32, placements[0].clone());
    }
    core.apply_channel_pack(input.clone()).unwrap();
    let dir = core.project_directory(&target.project_id).unwrap();
    let before = (
        std::fs::read(project_path(&dir)).unwrap(),
        std::fs::read(history_path(&dir)).unwrap(),
    );
    input.expected_revision = 1;
    if let crate::ChannelApplyMode::Instantiate { placements } = &mut input.application {
        placements.push(placements[0].clone());
    }
    assert!(core.apply_channel_pack(input).is_err());
    assert_eq!(
        before,
        (
            std::fs::read(project_path(&dir)).unwrap(),
            std::fs::read(history_path(&dir)).unwrap()
        )
    );
    let mut p = solid_package();
    let marker = p.recipes[0].definition.markers[0].clone();
    p.recipes[0].definition.markers = (0..32)
        .map(|i| {
            let mut m = marker.clone();
            m.id = format!("cue_{i}");
            m
        })
        .collect();
    let pack = core
        .import_channel_pack("32 marker reapply", &serde_json::to_string(&p).unwrap())
        .unwrap();
    for added in [35, 36] {
        let target = core
            .create_project("Native generated limit", ProjectSettings::default())
            .unwrap();
        let first = core
            .apply_channel_pack(apply_input(&core, &pack, target.project_id.clone()))
            .unwrap();
        let component = first.components[0].component_id.clone();
        core.edit_batch(
            &target.project_id,
            1,
            (0..added)
                .map(|i| EditOperation::MarkerCreate {
                    scope: format!("component:{component}"),
                    name: format!("Independent_cue_{i}"),
                    time_ms: 500,
                    kind: crate::MarkerKind::Cue,
                })
                .collect(),
        )
        .unwrap();
        let mut input = apply_input(&core, &pack, target.project_id.clone());
        input.application = crate::ChannelApplyMode::Reapply {
            components: vec![crate::ChannelComponentMapping {
                recipe_key: pack.package.recipes[0].key.clone(),
                component_id: component,
            }],
        };
        let dir = core.project_directory(&target.project_id).unwrap();
        let before = (
            std::fs::read(project_path(&dir)).unwrap(),
            std::fs::read(history_path(&dir)).unwrap(),
        );
        let result = core.apply_channel_pack(input);
        if added == 35 {
            assert_eq!(result.unwrap().write.revision, 3);
        } else {
            assert!(result.is_err());
            assert_eq!(
                before,
                (
                    std::fs::read(project_path(&dir)).unwrap(),
                    std::fs::read(history_path(&dir)).unwrap()
                )
            );
        }
    }
}

#[test]
fn library_byte_limit_unicode_names_and_version_exhaustion_fail_without_overwrite() {
    let (core, _temp) = core();
    let raw = serde_json::to_string(&package()).unwrap();
    let pack = core.import_channel_pack(&"🎬".repeat(80), &raw).unwrap();
    for invalid in ["🎬".repeat(81), "invalid\nname".into()] {
        let path = core.paths.projects_root().join(LIBRARY_FILE);
        let before = std::fs::read(&path).unwrap();
        assert!(core.import_channel_pack(&invalid, &raw).is_err());
        assert_eq!(before, std::fs::read(path).unwrap());
    }
    let mut last = pack.clone();
    last.version = crate::MAX_CHANNEL_VERSION;
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    let library = Library {
        version: 1,
        packs: vec![last],
    };
    let raw = serde_json::to_vec(&library).unwrap();
    std::fs::write(&path, &raw).unwrap();
    assert_eq!(
        core.get_channel_pack(&pack.id).unwrap().version,
        crate::MAX_CHANNEL_VERSION
    );
    assert!(
        core.publish_channel_pack(
            Some((&pack.id, crate::MAX_CHANNEL_VERSION)),
            pack.package.clone()
        )
        .is_err()
    );
    assert_eq!(raw, std::fs::read(&path).unwrap());
    let mut exact = raw;
    exact.resize(crate::MAX_CHANNEL_LIBRARY_BYTES, b' ');
    std::fs::write(&path, &exact).unwrap();
    assert_eq!(core.list_channel_packs().unwrap().packs.len(), 1);
    exact.push(b' ');
    std::fs::write(&path, &exact).unwrap();
    assert!(core.list_channel_packs().is_err());
    assert!(
        core.import_channel_pack(
            "Oversized root unchanged",
            &serde_json::to_string(&package()).unwrap()
        )
        .is_err()
    );
    assert_eq!(exact, std::fs::read(&path).unwrap());
}

#[test]
fn actual_managed_image_dependency_limit_accepts_32_and_rejects_33_atomically() {
    let (core, _temp) = core();
    let mut p = package();
    let oracle = fixture()["independentRenderOracle"]["image"].clone();
    let mut track: Track = serde_json::from_value(serde_json::json!({"id":"art_track","name":"Authorized original image","trackType":"video","items":[]})).unwrap();
    for index in 0..33 {
        let key = format!("art_{index}");
        p.images.push(crate::ChannelImageDependency {
            key: key.clone(),
            content_hash: ContentHash {
                algorithm: "sha256".into(),
                digest: oracle["sha256"].as_str().unwrap().into(),
            },
            size_bytes: oracle["sizeBytes"].as_u64().unwrap(),
            probe: MediaProbeFacts {
                has_video: true,
                video_width: Some(16),
                video_height: Some(16),
                format_name: Some("png_pipe".into()),
                video_codec: Some("png".into()),
                ..Default::default()
            },
        });
        let mut item: TimelineItem=serde_json::from_value(serde_json::json!({"type":"media","id":format!("image_{index}"),"assetId":key,"startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]})).unwrap();
        item.visual_properties_mut().stack_order = index as u32;
        track.items.push(item);
    }
    let extra_image = p.images.pop().unwrap();
    let extra_item = track.items.pop().unwrap();
    p.recipes[0].definition.tracks.push(track);
    assert_eq!(
        core.import_channel_pack(
            "32 artwork declarations",
            &serde_json::to_string(&p).unwrap()
        )
        .unwrap()
        .package
        .images
        .len(),
        32
    );
    p.images.push(extra_image);
    p.recipes[0].definition.tracks[0].items.push(extra_item);
    reject_import_unchanged(&core, &p);
}

#[test]
fn actual_pinned_font_limit_accepts_32_valid_referenced_faces_and_rejects_33_atomically() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let original = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/OpenCutBoundaryTest.ttf"
    ));
    let oracle = fixture()["independentBoundaryFont"].clone();
    assert_eq!(crate::fonts::content_hash(original), oracle["sha256"]);
    assert_eq!(original.len() as u64, oracle["sizeBytes"]);
    let (core, _temp) = core();
    let mut p = package();
    let mut track: Track=serde_json::from_value(serde_json::json!({"id":"pinned_track","name":"Pinned glyph A","trackType":"overlay","items":[]})).unwrap();
    for index in 0..33 {
        let mut bytes = original.to_vec();
        bytes.push(index as u8);
        let record = crate::fonts::record(&bytes).unwrap();
        let hash = record.sha256;
        p.fonts.push(crate::ChannelFontFace {
            sha256: hash.clone(),
            size_bytes: bytes.len() as u64,
            face_index: 0,
            data_base64: STANDARD.encode(bytes),
        });
        track.items.push(TimelineItem::Text(crate::TextItem {
            id: format!("glyph_{index}"),
            text: "A".into(),
            document: crate::RichTextDocument::plain("A".into()),
            font_binding: Some(crate::FontBinding {
                profile: crate::TEXT_LAYOUT_PROFILE.into(),
                regular: hash.clone(),
                bold: hash.clone(),
                italic: hash.clone(),
                bold_italic: hash,
                warnings: vec![],
            }),
            start_ms: 0,
            duration_ms: 1000,
            font_size: 24,
            color: "#FFFFFF".into(),
            font_family: None,
            font_path: None,
            style: crate::TextStyle::default(),
            visual_properties: crate::VisualProperties {
                stack_order: index as u32,
                ..Default::default()
            },
            keyframes: vec![],
        }));
    }
    let extra_font = p.fonts.pop().unwrap();
    let extra_text = track.items.pop().unwrap();
    p.recipes[0].definition.tracks.push(track);
    let pack = core
        .import_channel_pack("32 exact pinned faces", &serde_json::to_string(&p).unwrap())
        .unwrap();
    assert_eq!(pack.package.fonts.len(), 32);
    let target = core
        .create_project(
            "Pinned boundary materialization",
            ProjectSettings::default(),
        )
        .unwrap();
    core.apply_channel_pack(apply_input(&core, &pack, target.project_id.clone()))
        .unwrap();
    assert_eq!(
        core.get_project(&target.project_id).unwrap().fonts.len(),
        32
    );
    p.fonts.push(extra_font);
    p.recipes[0].definition.tracks[0].items.push(extra_text);
    reject_import_unchanged(&core, &p);
}

#[test]
fn changed_nested_reapply_populates_new_child_slots_before_parent_overrides() {
    let (core, _temp) = core();
    let mut p = nested_literal_package();
    let opacity_slot = |id: &str, target: &str| crate::TemplateSlot {
        id: id.into(),
        name: id.into(),
        kind: crate::SlotKind::Number,
        required: false,
        default_value: Some(crate::SlotValue::Number(0.75)),
        binding: crate::SlotBinding {
            target_layer_id: target.into(),
            property: crate::SlotProperty::VisualOpacity,
        },
        constraints: crate::SlotConstraints::default(),
    };
    p.recipes[0]
        .definition
        .slots
        .push(opacity_slot("root_opacity", "nested_instance"));
    let pack = core.publish_channel_pack(None, p).unwrap();
    let target = core
        .create_project("Changed nested reapply", ProjectSettings::default())
        .unwrap();
    let mut input = apply_input(&core, &pack, target.project_id.clone());
    if let crate::ChannelApplyMode::Instantiate { placements } = &mut input.application {
        placements[0]
            .slot_values
            .insert("root_opacity".into(), crate::SlotValue::Number(0.6));
    }
    let first = core.apply_channel_pack(input).unwrap();
    let mut next = pack.package.clone();
    next.recipes[1]
        .definition
        .slots
        .push(opacity_slot("new_opacity", "paint_0"));
    let TimelineItem::ComponentInstance(nested) =
        &mut next.recipes[0].definition.tracks[0].items[0]
    else {
        panic!("nested instance")
    };
    nested
        .slot_values
        .insert("new_opacity".into(), crate::SlotValue::Number(0.5));
    let next = core
        .publish_channel_pack(Some((&pack.id, 1)), next)
        .unwrap();
    let mut input = apply_input(&core, &next, target.project_id.clone());
    input.application = crate::ChannelApplyMode::Reapply {
        components: first
            .components
            .iter()
            .map(|v| crate::ChannelComponentMapping {
                recipe_key: v.recipe_key.clone(),
                component_id: v.component_id.clone(),
            })
            .collect(),
    };
    let second = core.apply_channel_pack(input).unwrap();
    assert_eq!(second.write.revision, 2);
    assert!(second.components.iter().all(|v| v.changed));
    let project = core.get_project(&target.project_id).unwrap();
    let root = project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find_map(|v| {
            if let TimelineItem::ComponentInstance(v) = v {
                Some(v)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        root.slot_values["root_opacity"],
        crate::SlotValue::Number(0.6)
    );
    let parent = project
        .components
        .iter()
        .find(|v| v.id == root.component_id)
        .unwrap();
    let TimelineItem::ComponentInstance(nested) = &parent.tracks[0].items[0] else {
        panic!("nested instance")
    };
    assert_eq!(
        nested.slot_values["new_opacity"],
        crate::SlotValue::Number(0.5)
    );
    assert_eq!(
        nested.slot_values["__proto__"],
        crate::SlotValue::Boolean(true)
    );
    assert_eq!(
        nested.slot_values["constructor"],
        crate::SlotValue::Boolean(false)
    );
    assert_eq!(
        nested.slot_values["toString"],
        crate::SlotValue::Boolean(true)
    );
}

fn valid_padded_font_package(size: usize) -> ChannelPackPackage {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let mut bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/OpenCutBoundaryTest.ttf"
    ))
    .to_vec();
    bytes.resize(size, 0);
    let record = crate::fonts::record(&bytes).unwrap();
    let hash = record.sha256;
    let mut p = package();
    p.fonts.push(crate::ChannelFontFace {
        sha256: hash.clone(),
        size_bytes: bytes.len() as u64,
        face_index: 0,
        data_base64: STANDARD.encode(bytes),
    });
    p.recipes[0].definition.tracks.push(serde_json::from_value(serde_json::json!({
        "id":"padded_track", "name":"Padded pinned glyph", "trackType":"overlay", "items":[{
            "type":"text", "id":"padded_glyph", "text":"A", "document":crate::RichTextDocument::plain("A".into()),
            "fontBinding": {"profile":crate::TEXT_LAYOUT_PROFILE,"regular":hash,"bold":hash,"italic":hash,"boldItalic":hash,"warnings":[]},
            "startMs":0,"durationMs":1000,"fontSize":24,"color":"#FFFFFF","keyframes":[]
        }]
    })).unwrap());
    p
}

#[test]
fn canonical_package_and_candidate_library_publication_limits_are_atomic() {
    let (core, _temp) = core();
    let target = core
        .create_project("Publication isolation", ProjectSettings::default())
        .unwrap();
    let dir = core.project_directory(&target.project_id).unwrap();
    let snapshot = || {
        (
            std::fs::read(project_path(&dir)).unwrap(),
            std::fs::read(history_path(&dir)).unwrap(),
            core.get_project(&target.project_id).unwrap().fonts,
        )
    };
    let original_project = snapshot();
    let path = core.paths.projects_root().join(LIBRARY_FILE);
    let mut valid = valid_padded_font_package(5 * 1024 * 1024);
    assert!(serde_json::to_vec(&valid).unwrap().len() < crate::MAX_CHANNEL_PACK_BYTES);
    let pack = core.publish_channel_pack(None, valid.clone()).unwrap();
    assert_eq!(pack.package.fonts.len(), 1);
    let before = std::fs::read(&path).unwrap();
    let oversized = valid_padded_font_package(6 * 1024 * 1024);
    assert!(serde_json::to_vec(&oversized).unwrap().len() > crate::MAX_CHANNEL_PACK_BYTES);
    let error = core.publish_channel_pack(None, oversized).unwrap_err();
    assert!(error.message.contains("package byte limit"), "{error:?}");
    assert_eq!(before, std::fs::read(&path).unwrap());
    assert_eq!(original_project, snapshot());
    let mut rejected = false;
    for index in 1..crate::MAX_CHANNEL_PACKS {
        valid.name = format!("Bounded publication {index}");
        let before = std::fs::read(&path).unwrap();
        match core.publish_channel_pack(None, valid.clone()) {
            Ok(_) => assert!(
                std::fs::metadata(&path).unwrap().len() <= crate::MAX_CHANNEL_LIBRARY_BYTES as u64
            ),
            Err(error) => {
                assert!(error.message.contains("library byte limit"), "{error:?}");
                assert_eq!(before, std::fs::read(&path).unwrap());
                rejected = true;
                break;
            }
        }
    }
    assert!(
        rejected,
        "candidate serialization must exceed 64MiB before pack-count quota"
    );
    assert_eq!(original_project, snapshot());
}
