use base64::Engine;
use opencut_editor_core::*;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Cursor};

fn png_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 3, 3);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        // Original synthetic white border and opaque red center.
        let mut pixels = vec![255; 3 * 3 * 4];
        pixels[16..20].copy_from_slice(&[220, 20, 30, 255]);
        writer.write_image_data(&pixels).unwrap();
    }
    bytes
}
fn parameters() -> ArtworkParameters {
    serde_json::from_value(fixture()["parameters"].clone()).unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/artwork-preparation-v1.json"
    ))
    .unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String, Vec<u8>) {
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
        .create_project("Artwork", ProjectSettings::default())
        .unwrap()
        .project_id;
    let bytes = png_bytes();
    let path = root.path().join("original.png");
    fs::write(&path, &bytes).unwrap();
    let imported = core
        .import_asset(
            &id,
            0,
            &path,
            MediaType::Image,
            MediaProbeFacts {
                has_video: true,
                video_width: Some(3),
                video_height: Some(3),
                video_codec: Some("png".into()),
                ..MediaProbeFacts::default()
            },
        )
        .unwrap();
    (root, core, id, imported.changed_ids[0].clone(), bytes)
}
#[test]
fn canonical_manifest_and_derivation_are_strict_and_roundtrip() {
    let f = fixture();
    let m: ArtworkManifest = serde_json::from_value(f["manifest"].clone()).unwrap();
    assert_eq!(serde_json::to_value(m).unwrap(), f["manifest"]);
    let d: ArtworkDerivation = serde_json::from_value(f["derivation"].clone()).unwrap();
    assert_eq!(serde_json::to_value(d).unwrap(), f["derivation"]);
    let mut bad = f["manifest"].clone();
    bad["unknown"] = json!(true);
    assert!(serde_json::from_value::<ArtworkManifest>(bad).is_err());
    let mut bad = f["parameters"].clone();
    bad["cleanup"]["color"][0] = json!(256);
    assert!(serde_json::from_value::<ArtworkParameters>(bad).is_err());
    assert!(
        serde_json::from_value::<ArtworkParameters>(
            json!({"cleanup":null,"outline":{"radius":0,"color":[0,0,0,0]}})
        )
        .is_err()
    );
    let mut legacy_asset = json!({"id":"legacy","mediaType":"image","fileName":"a.png","projectRelativePath":"assets/a","durationMs":null});
    let legacy: Asset = serde_json::from_value(legacy_asset.clone()).unwrap();
    assert!(
        serde_json::to_value(legacy)
            .unwrap()
            .get("artworkDerivation")
            .is_none()
    );
    legacy_asset["artworkDerivation"] = Value::Null;
    assert!(serde_json::from_value::<Asset>(legacy_asset).is_err());
    assert_eq!(
        PROJECT_SCHEMA_VERSION,
        f["projectSchemaVersion"].as_u64().unwrap() as u32
    );
}
#[test]
fn preview_is_nonmutating_and_commit_reopen_history_preserve_exact_source_and_pixels() {
    let (root, core, id, source, source_bytes) = setup();
    let dir = root.path().join("projects").join(&id);
    let before = fs::read(dir.join("project.json")).unwrap();
    let history_before = fs::read(dir.join("history.json")).unwrap();
    let preview = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let second = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    assert_eq!(preview, second);
    assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
    assert_eq!(fs::read(dir.join("history.json")).unwrap(), history_before);
    assert_eq!(
        (
            preview.manifest.width,
            preview.manifest.height,
            preview.manifest.content_offset
        ),
        (5, 5, [1, 1])
    );
    for (bytes, hash) in [
        (
            &preview.files.result,
            &preview.manifest.review_hashes.result,
        ),
        (&preview.files.mask, &preview.manifest.review_hashes.mask),
        (&preview.files.light, &preview.manifest.review_hashes.light),
        (&preview.files.dark, &preview.manifest.review_hashes.dark),
    ] {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(bytes)
            .unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), *hash);
    }
    let expected = base64::engine::general_purpose::STANDARD
        .decode(&preview.files.result)
        .unwrap();
    let mut reader = png::Decoder::new(Cursor::new(&expected))
        .read_info()
        .unwrap();
    let mut decoded = vec![0; reader.output_buffer_size().unwrap()];
    reader.next_frame(&mut decoded).unwrap();
    // Center is original red; its immediate Chebyshev neighbors are selected opaque outline.
    assert_eq!(
        &decoded[(2 * 5 + 2) * 4..(2 * 5 + 2) * 4 + 4],
        &[220, 20, 30, 255]
    );
    assert_eq!(&decoded[24..28], &[20, 30, 40, 255]);
    let committed = core.commit_artwork(preview.manifest.clone()).unwrap();
    assert_eq!(committed.revision, 2);
    let project = core.get_project(&id).unwrap();
    assert_eq!(project.assets.len(), 2);
    assert!(project.tracks.iter().all(|t| t.items.is_empty()));
    let derivative = project
        .assets
        .iter()
        .find(|a| a.id == committed.changed_ids[0])
        .unwrap();
    assert!(derivative.origin.is_none());
    assert_eq!(
        fs::read(dir.join(&derivative.project_relative_path)).unwrap(),
        expected
    );
    assert_eq!(
        fs::read(dir.join(&project.assets[0].project_relative_path)).unwrap(),
        source_bytes
    );
    assert_eq!(
        derivative
            .artwork_derivation
            .as_ref()
            .unwrap()
            .source_asset_id,
        source
    );
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&project).unwrap()
    );
    core.undo(&id, 2).unwrap();
    assert_eq!(core.get_project(&id).unwrap().assets.len(), 1);
    core.redo(&id, 3).unwrap();
    let restored = core.get_project(&id).unwrap();
    assert_eq!(restored.assets.len(), 2);
    assert_eq!(
        fs::read(dir.join(&restored.assets[1].project_relative_path)).unwrap(),
        expected
    );
}
#[test]
fn conflicts_tampering_and_invalid_parameters_never_publish_assets() {
    let (root, core, id, source, _) = setup();
    let dir = root.path().join("projects").join(&id);
    let p = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let original = fs::read(dir.join("project.json")).unwrap();
    let history = fs::read(dir.join("history.json")).unwrap();
    for mutate in 0..5 {
        let mut m = p.manifest.clone();
        match mutate {
            0 => m.review_hashes.dark = "0".repeat(64),
            1 => m.source_hash = "0".repeat(64),
            2 => m.width += 1,
            3 => m.version += 1,
            _ => m.parameters.outline.radius = 17,
        };
        assert_eq!(
            core.commit_artwork(m).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(fs::read(dir.join("project.json")).unwrap(), original);
        assert_eq!(fs::read(dir.join("history.json")).unwrap(), history);
    }
    core.commit_artwork(p.manifest.clone()).unwrap();
    assert_eq!(
        core.commit_artwork(p.manifest).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(core.get_project(&id).unwrap().assets.len(), 2);
}
#[test]
fn derivative_source_delete_guard_and_history_keep_owned_bytes() {
    let (root, core, id, source, source_bytes) = setup();
    let p = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let result = core.commit_artwork(p.manifest).unwrap();
    let project = core.get_project(&id).unwrap();
    let path = root
        .path()
        .join("projects")
        .join(&id)
        .join(&project.assets[0].project_relative_path);
    let error = core.delete_asset(&id, 2, &source).unwrap_err();
    assert_eq!(error.code, ErrorCode::AssetInUse);
    assert!(error.message.contains("artwork source"));
    core.delete_asset(&id, 2, &result.changed_ids[0]).unwrap();
    core.delete_asset(&id, 3, &source).unwrap();
    assert_eq!(fs::read(&path).unwrap(), source_bytes);
    core.undo(&id, 4).unwrap();
    assert_eq!(core.get_project(&id).unwrap().assets.len(), 1);
    core.undo(&id, 5).unwrap();
    assert_eq!(core.get_project(&id).unwrap().assets.len(), 2);
}
#[test]
fn malformed_provenance_and_retained_cycles_fail_closed_without_rewrite() {
    for corruption in 0..6 {
        let (root, core, id, source, _) = setup();
        let p = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
        core.commit_artwork(p.manifest).unwrap();
        let path = root.path().join("projects").join(&id).join("project.json");
        let mut raw: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match corruption {
            0 => raw["assets"][1]["artworkDerivation"]["sourceAssetId"] = json!("missing"),
            1 => raw["assets"][1]["artworkDerivation"]["sourceHash"] = json!("0".repeat(64)),
            2 => {
                raw["assets"][1]["artworkDerivation"]["sourceAssetId"] =
                    raw["assets"][1]["id"].clone()
            }
            3 => raw["assets"][0]["mediaType"] = json!("audio"),
            4 => raw["assets"][1]["artworkDerivation"]["contentOffset"] = json!([0, 0]),
            _ => raw["assets"][1]["probe"]["videoWidth"] = json!(99),
        };
        let before = serde_json::to_vec_pretty(&raw).unwrap();
        fs::write(&path, &before).unwrap();
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::AssetIntegrityFailed
        );
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
#[test]
fn schema46_current_and_history_migrate_idempotently_and_future_or_premature_are_preserved() {
    let (root, core, id, _, _) = setup();
    let dir = root.path().join("projects").join(&id);
    let mut current: Value =
        serde_json::from_slice(&fs::read(dir.join("project.json")).unwrap()).unwrap();
    current["schemaVersion"] = json!(46);
    fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&current).unwrap(),
    )
    .unwrap();
    let mut history: Value =
        serde_json::from_slice(&fs::read(dir.join("history.json")).unwrap()).unwrap();
    for snapshot in history["undo"].as_array_mut().unwrap() {
        snapshot["schemaVersion"] = json!(46);
    }
    fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&history).unwrap(),
    )
    .unwrap();
    assert_eq!(core.get_project(&id).unwrap().schema_version, 47);
    let first = fs::read(dir.join("project.json")).unwrap();
    let history_first = fs::read(dir.join("history.json")).unwrap();
    core.get_project(&id).unwrap();
    assert_eq!(fs::read(dir.join("project.json")).unwrap(), first);
    assert_eq!(fs::read(dir.join("history.json")).unwrap(), history_first);
    assert!(
        !String::from_utf8(first.clone())
            .unwrap()
            .contains("artworkDerivation")
    );
    for (version, field) in [(48, false), (46, true)] {
        let mut raw: Value = serde_json::from_slice(&first).unwrap();
        raw["schemaVersion"] = json!(version);
        if field {
            raw["assets"][0]["artworkDerivation"] = Value::Null;
        }
        let bytes = serde_json::to_vec_pretty(&raw).unwrap();
        fs::write(dir.join("project.json"), &bytes).unwrap();
        assert!(core.get_project(&id).is_err());
        assert_eq!(fs::read(dir.join("project.json")).unwrap(), bytes);
    }
}
#[test]
fn retained_draft_keeps_derivative_and_transitive_source_logically_owned() {
    let (_root, core, id, source, _) = setup();
    let p = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let result = core.commit_artwork(p.manifest).unwrap();
    let project = core.get_project(&id).unwrap();
    let track = project
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Video)
        .unwrap()
        .id
        .clone();
    let operation:EditOperation=serde_json::from_value(json!({"operation":"add_media","trackId":track,"assetId":result.changed_ids[0],"sourceInMs":0,"startMs":0,"durationMs":1000})).unwrap();
    let draft = core
        .create_draft(&id, 2, vec![operation], Some("owned artwork".into()))
        .unwrap();
    assert_eq!(
        core.delete_asset(&id, 2, &result.changed_ids[0])
            .unwrap_err()
            .code,
        ErrorCode::AssetInUse
    );
    assert_eq!(
        core.delete_asset(&id, 2, &source).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    assert_eq!(core.get_draft(&id, &draft.id).unwrap().id, draft.id);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened.get_draft(&id, &draft.id).unwrap().operations.len(),
        1
    );
}
#[test]
fn two_asset_cycle_and_history_only_dangling_provenance_fail_closed() {
    let (root, core, id, _source, bytes) = setup();
    let path = root.path().join("same.png");
    fs::write(&path, &bytes).unwrap();
    core.import_asset(
        &id,
        1,
        &path,
        MediaType::Image,
        MediaProbeFacts {
            has_video: true,
            video_width: Some(3),
            video_height: Some(3),
            ..MediaProbeFacts::default()
        },
    )
    .unwrap();
    let dir = root.path().join("projects").join(&id);
    let file = dir.join("project.json");
    let mut raw: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    for index in 0..2 {
        let other = 1 - index;
        raw["assets"][index]["artworkDerivation"] = json!({"version":1,"sourceAssetId":raw["assets"][other]["id"],"sourceHash":raw["assets"][other]["contentHash"]["digest"],"parameters":{"outline":{"radius":0,"color":[0,0,0,0]}},"sourceWidth":3,"sourceHeight":3,"contentOffset":[0,0]});
    }
    let bad = serde_json::to_vec_pretty(&raw).unwrap();
    fs::write(&file, &bad).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(fs::read(&file).unwrap(), bad);
    let (root, core, id, _, _) = setup();
    let source = core.get_project(&id).unwrap().assets[0].id.clone();
    let preview = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let committed = core.commit_artwork(preview.manifest).unwrap();
    core.delete_asset(&id, 2, &committed.changed_ids[0])
        .unwrap();
    let dir = root.path().join("projects").join(&id);
    let file = dir.join("history.json");
    let mut raw: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    let snapshots = raw["undo"].as_array_mut().unwrap();
    let last = snapshots.last_mut().unwrap();
    last["assets"].as_array_mut().unwrap().remove(0);
    let bad = serde_json::to_vec_pretty(&raw).unwrap();
    fs::write(&file, &bad).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(fs::read(&file).unwrap(), bad);
}
#[test]
fn unreachable_derivative_and_original_collect_only_after_history_eviction() {
    let (root, core, id, source, _) = setup();
    let preview = core.prepare_artwork(&id, 1, &source, parameters()).unwrap();
    let committed = core.commit_artwork(preview.manifest).unwrap();
    let project = core.get_project(&id).unwrap();
    let dir = root.path().join("projects").join(&id);
    let original = dir.join(&project.assets[0].project_relative_path);
    let derivative = dir.join(&project.assets[1].project_relative_path);
    core.delete_asset(&id, 2, &committed.changed_ids[0])
        .unwrap();
    core.delete_asset(&id, 3, &source).unwrap();
    assert!(original.exists() && derivative.exists());
    let track = project.tracks[0].id.clone();
    for i in 0..101 {
        let op: EditOperation = serde_json::from_value(
            json!({"operation":"update_track","trackId":track,"name":format!("Retain {i}")}),
        )
        .unwrap();
        core.edit(&id, 4 + i, op).unwrap();
    }
    assert!(!original.exists());
    assert!(!derivative.exists());
    assert!(core.get_project(&id).unwrap().assets.is_empty());
}

#[cfg(unix)]
#[test]
fn symlinked_source_and_destination_preserve_foreign_files_and_project_state() {
    use std::os::unix::fs::symlink;
    let (root, core, id, source, source_bytes) = setup();
    let project = core.get_project(&id).unwrap();
    let dir = root.path().join("projects").join(&id);
    let original = dir.join(&project.assets[0].project_relative_path);
    let external = root.path().join("foreign.png");
    fs::write(&external, &source_bytes).unwrap();
    fs::remove_file(&original).unwrap();
    symlink(&external, &original).unwrap();
    let before = fs::read(dir.join("project.json")).unwrap();
    assert_eq!(
        core.prepare_artwork(&id, 1, &source, parameters())
            .unwrap_err()
            .code,
        ErrorCode::PathNotAllowed
    );
    assert_eq!(fs::read(&external).unwrap(), source_bytes);
    assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
    fs::remove_file(&original).unwrap();
    fs::write(&original, &source_bytes).unwrap();
    let mut selected = None;
    for red in 0..=255 {
        let mut params = parameters();
        params.outline.color[0] = red;
        let preview = core.prepare_artwork(&id, 1, &source, params).unwrap();
        let prefix = dir
            .join("assets/sha256")
            .join(&preview.manifest.result_hash[..2]);
        if !prefix.exists() {
            selected = Some((preview, prefix));
            break;
        }
    }
    let (preview, prefix) = selected.unwrap();
    let foreign = root.path().join("foreign-directory");
    fs::create_dir(&foreign).unwrap();
    fs::write(foreign.join("sentinel"), b"retain").unwrap();
    symlink(&foreign, &prefix).unwrap();
    assert_eq!(
        core.commit_artwork(preview.manifest).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(fs::read(foreign.join("sentinel")).unwrap(), b"retain");
    assert_eq!(fs::read_dir(&foreign).unwrap().count(), 1);
    assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
}

#[test]
fn missing_legacy_dimension_facts_guide_reimport_without_claiming_damage_or_blocking_existing_edits()
 {
    for missing in ["videoWidth", "videoHeight"] {
        let (root, core, id, source, source_bytes) = setup();
        let dir = root.path().join("projects").join(&id);
        let project_path = dir.join("project.json");
        let mut raw: Value = serde_json::from_slice(&fs::read(&project_path).unwrap()).unwrap();
        raw["assets"][0]["probe"][missing] = Value::Null;
        fs::write(&project_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
        let legacy = core.get_project(&id).unwrap();
        let before = fs::read(&project_path).unwrap();
        let history_before = fs::read(dir.join("history.json")).unwrap();
        let error = core
            .prepare_artwork(&id, 1, &source, parameters())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
        assert!(error.message.contains("missing required dimension facts"));
        assert!(error.message.contains("reimport"));
        assert!(!error.message.contains("mismatch"));
        assert!(!error.message.contains("damaged"));
        assert_eq!(fs::read(&project_path).unwrap(), before);
        assert_eq!(fs::read(dir.join("history.json")).unwrap(), history_before);
        assert_eq!(
            fs::read(dir.join(&legacy.assets[0].project_relative_path)).unwrap(),
            source_bytes
        );
        let operation: EditOperation = serde_json::from_value(json!({"operation":"update_track","trackId":legacy.tracks[0].id,"name":"Legacy remains editable"})).unwrap();
        let edited = core.edit(&id, 1, operation).unwrap();
        assert_eq!(edited.revision, 2);
        assert_eq!(core.get_project(&id).unwrap().assets[0].id, source);
    }
    let (root, core, id, source, _) = setup();
    let file = root.path().join("projects").join(&id).join("project.json");
    let mut raw: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    raw["assets"][0]["probe"]["videoWidth"] = json!(42);
    let before = serde_json::to_vec_pretty(&raw).unwrap();
    fs::write(&file, &before).unwrap();
    let error = core
        .prepare_artwork(&id, 1, &source, parameters())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::AssetIntegrityFailed);
    assert!(error.message.contains("dimension facts mismatch"));
    assert!(!error.message.contains("missing"));
    assert_eq!(fs::read(file).unwrap(), before);
}
