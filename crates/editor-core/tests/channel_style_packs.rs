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
    assert_eq!(fixture["componentSchemaVersion"], PROJECT_SCHEMA_VERSION);
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
