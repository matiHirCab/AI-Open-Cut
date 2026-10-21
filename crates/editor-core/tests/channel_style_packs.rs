use opencut_editor_core::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/channel-style-packs-v1.json"
    ))
    .unwrap()
}

#[test]
fn canonical_portable_contract_is_consumed_by_the_actual_public_core() {
    let fixture = fixture();
    assert_eq!(fixture["limits"]["packBytes"], MAX_CHANNEL_PACK_BYTES);
    assert_eq!(fixture["limits"]["libraryBytes"], MAX_CHANNEL_LIBRARY_BYTES);
    assert_eq!(
        fixture["componentSchemaVersion"],
        CHANNEL_PACK_COMPONENT_SCHEMA_VERSION
    );
    let root = tempfile::tempdir().unwrap();
    let paths = PathPolicy::new(
        root.path().join("projects"),
        [root.path().to_owned()],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(paths);
    let pack = core
        .import_channel_pack(
            "Independent canonical consumer",
            &fixture["package"].to_string(),
        )
        .unwrap();
    let declared: ChannelPackPackage = serde_json::from_value(fixture["package"].clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&pack.package.recipes).unwrap(),
        serde_json::to_value(&declared.recipes).unwrap()
    );
    assert_eq!(core.get_channel_pack(&pack.id).unwrap().version, 1);
    assert_eq!(core.list_channel_packs().unwrap().packs[0].id, pack.id);
    assert_eq!(
        core.import_channel_pack(
            "Independent canonical consumer",
            &fixture["package"].to_string()
        )
        .unwrap_err()
        .code,
        ErrorCode::ValidationFailed
    );
}

#[test]
fn canonical_new_closed_enums_reject_execution_and_unknown_fields() {
    for mode in ["saved", "disabled"] {
        assert!(
            serde_json::from_value::<ChannelNarrationChoice>(serde_json::json!({"mode":mode}))
                .is_ok()
        );
        assert!(
            serde_json::from_value::<ChannelNarrationChoice>(
                serde_json::json!({"mode":mode,"script":"execute"})
            )
            .is_err()
        );
    }
    let mut package = fixture()["package"].clone();
    package["path"] = "/tmp/foreign".into();
    assert!(serde_json::from_value::<ChannelPackPackage>(package).is_err());
}

#[test]
fn cp1_component46_survives_project47_and_readonly_capture_of_both_compatible_sources() {
    use std::collections::BTreeMap;
    fn files(dir: &std::path::Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
        let mut result = BTreeMap::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                result.extend(files(&path));
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
        result
    }
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Compatible CP1 source", ProjectSettings::default())
        .unwrap()
        .project_id;
    let edit = core
        .edit(
            &id,
            0,
            EditOperation::ComponentCreate {
                name: "Independent native recipe".into(),
                width: 320,
                height: 180,
                duration_ms: 1000,
                tracks: vec![],
                slots: Some(vec![]),
            },
        )
        .unwrap();
    let source47 = core.get_project(&id).unwrap();
    assert_eq!(source47.schema_version, 47);
    let dir = core.paths().projects_root().join(&id);
    for schema in [46, 47] {
        let mut source = source47.clone();
        source.schema_version = schema;
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&source).unwrap(),
        )
        .unwrap();
        let before = files(&dir);
        let captured = core
            .capture_channel_pack(ChannelCaptureInput {
                project_id: id.clone(),
                expected_revision: 1,
                component_ids: vec![edit.changed_ids[0].clone()],
                name: format!("Compatible source{schema}"),
                caption_role: None,
                narration_profile: None,
            })
            .unwrap();
        assert_eq!(files(&dir), before);
        assert_eq!(captured.pack.package.component_schema_version, 46);
        let exported = core.get_channel_pack(&captured.pack.id).unwrap().package;
        let imported = core
            .import_channel_pack(
                &format!("Imported source{schema}"),
                &serde_json::to_string(&exported).unwrap(),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(imported.package.recipes).unwrap(),
            serde_json::to_value(&exported.recipes).unwrap()
        );
        let target = core
            .create_project(
                "Current project accepts component46",
                ProjectSettings::default(),
            )
            .unwrap()
            .project_id;
        let target_state = core.get_project(&target).unwrap();
        assert_eq!(target_state.schema_version, 47);
        let track = target_state
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Overlay)
            .unwrap();
        core.apply_channel_pack(ChannelApplyInput {
            project_id: target.clone(),
            expected_revision: 0,
            pack_id: imported.id.clone(),
            expected_version: 1,
            images: vec![],
            narration: ChannelNarrationChoice::Disabled {},
            application: ChannelApplyMode::Instantiate {
                placements: vec![ChannelPlacement {
                    recipe_key: exported.recipes[0].key.clone(),
                    track_id: track.id.clone(),
                    start_ms: 0,
                    trim_start_ms: 0,
                    duration_ms: 1000,
                    time_scale: 1.0,
                    stagger_ms: 0,
                    slot_values: Default::default(),
                    transform: Transform::default(),
                    transform2d: None,
                    hidden: false,
                    z_index: 0,
                    parent: None,
                }],
            },
        })
        .unwrap();
        let applied = core.get_project(&target).unwrap();
        assert_eq!(applied.schema_version, 47);
        assert_eq!(applied.components.len(), 1);
        assert!(applied.narration_manifests.is_empty());
        assert_eq!(
            core.get_channel_pack(&imported.id)
                .unwrap()
                .package
                .component_schema_version,
            46
        );
        let mut future = serde_json::to_value(&exported).unwrap();
        future["componentSchemaVersion"] = 47.into();
        assert!(
            core.import_channel_pack("Future portable must fail", &future.to_string())
                .is_err()
        );
        assert_eq!(files(&dir), before);
    }
    for schema in [45, 48] {
        let mut source = source47.clone();
        source.schema_version = schema;
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&source).unwrap(),
        )
        .unwrap();
        let before = files(&dir);
        assert!(
            core.capture_channel_pack(ChannelCaptureInput {
                project_id: id.clone(),
                expected_revision: 1,
                component_ids: vec![edit.changed_ids[0].clone()],
                name: format!("Unsupported source{schema}"),
                caption_role: None,
                narration_profile: None
            })
            .is_err()
        );
        assert_eq!(files(&dir), before);
    }
}
