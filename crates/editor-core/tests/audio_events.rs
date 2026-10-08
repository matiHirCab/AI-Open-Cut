use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, MediaProbeFacts, MediaType, PROJECT_SCHEMA_VERSION,
    PathPolicy, ProjectSettings, TimelineItem,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/timeline-audio-events-v1.json"
    ))
    .unwrap()
}
fn inventory(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in std::fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            result.extend(inventory(&p));
        } else {
            result.insert(p.clone(), std::fs::read(p).unwrap());
        }
    }
    result
}
fn setup() -> (tempfile::TempDir, EditorCore, String, Vec<String>, String) {
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
        .create_project("audio events", ProjectSettings::default())
        .unwrap()
        .project_id;
    let mut assets = vec![];
    for (index, bytes) in [b"immutable event A".as_slice(), b"immutable event B"]
        .into_iter()
        .enumerate()
    {
        let path = media.join(format!("variant{index}.wav"));
        std::fs::write(&path, bytes).unwrap();
        assets.push(
            core.import_asset(
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
            .unwrap()
            .changed_ids[0]
                .clone(),
        );
    }
    core.edit(&id,2,serde_json::from_value(json!({"operation":"sound_event_register","event":"impact","variantAssetIds":assets,"defaultGainDb":-6,"busId":"sfx","variantSeed":1})).unwrap()).unwrap();
    let track = core
        .edit(
            &id,
            3,
            serde_json::from_value(
                json!({"operation":"create_track","name":"events","trackType":"audio"}),
            )
            .unwrap(),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    (root, core, id, assets, track)
}
fn input(track: &str) -> Value {
    let mut value = catalog()["input"].clone();
    value["trackId"] = json!(track);
    value
}
fn media(core: &EditorCore, id: &str, track: &str) -> opencut_editor_core::MediaItem {
    let project = core.get_project(id).unwrap();
    let item = &project.tracks.iter().find(|t| t.id == track).unwrap().items[0];
    let TimelineItem::Media(media) = item else {
        panic!("expected media")
    };
    media.clone()
}
#[test]
fn deterministic_selection_snapshot_replacement_history_and_reopen() {
    let (root, core, id, assets, track) = setup();
    let result = core
        .edit(
            &id,
            4,
            serde_json::from_value::<EditOperation>(input(&track)).unwrap(),
        )
        .unwrap();
    let saved = media(&core, &id, &track);
    assert_eq!(saved.asset_id, assets[1]);
    assert_eq!(
        (saved.start_ms, saved.duration_ms, saved.source_in_ms),
        (250, 300, 0)
    );
    let meta = saved.audio_event.as_ref().unwrap();
    assert_eq!(
        (
            meta.gain_db,
            meta.default_gain_db,
            meta.variant_seed,
            meta.variant_index
        ),
        (-3.0, -6.0, 1, 1)
    );
    assert!((meta.linear_gain() - 10_f64.powf(-9.0 / 20.0)).abs() < 1e-15);
    core.edit(&id,5,serde_json::from_value(json!({"operation":"sound_event_register","event":"impact","variantAssetIds":[assets[0]],"defaultGainDb":12,"busId":"music","variantSeed":0})).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(media(&core, &id, &track)).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    let before = inventory(root.path());
    assert_eq!(
        core.delete_asset(&id, 6, &assets[1]).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    assert_eq!(inventory(root.path()), before);
    core.undo(&id, 6).unwrap();
    core.undo(&id, 7).unwrap();
    assert!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .items
            .is_empty()
    );
    core.redo(&id, 8).unwrap();
    core.get_project(&id).unwrap();
    assert_eq!(media(&core, &id, &track).id, result.changed_ids[0]);
    assert_eq!(
        serde_json::to_value(media(&core, &id, &track)).unwrap(),
        serde_json::to_value(saved).unwrap()
    );
}
#[test]
fn malformed_missing_stale_and_late_batch_failures_preserve_bytes() {
    let (root, core, id, _, track) = setup();
    for case in catalog()["invalidInputChanges"].as_array().unwrap() {
        let mut v = input(&track);
        v[case["field"].as_str().unwrap()] = case["value"].clone();
        let before = inventory(root.path());
        assert!(
            core.edit(&id, 4, serde_json::from_value(v).unwrap())
                .is_err(),
            "{case}"
        );
        assert_eq!(inventory(root.path()), before, "{case}");
    }
    let before = inventory(root.path());
    assert_eq!(
        core.edit(
            &id,
            3,
            serde_json::from_value::<EditOperation>(input(&track)).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(root.path()), before);
    let mut missing = input(&track);
    missing["event"] = json!("missing");
    assert_eq!(
        core.edit_batch(
            &id,
            4,
            vec![
                serde_json::from_value::<EditOperation>(input(&track)).unwrap(),
                serde_json::from_value::<EditOperation>(missing).unwrap()
            ]
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    for field in ["at", "gainDb", "variantSeed", "durationMs"] {
        let mut v = input(&track);
        v[field] = Value::Null;
        assert!(serde_json::from_value::<EditOperation>(v).is_err());
    }
    let mut v = input(&track);
    v["at"]["extra"] = json!(1);
    assert!(serde_json::from_value::<EditOperation>(v).is_err());
    let mut v = input(&track);
    v["extra"] = json!(1);
    assert!(serde_json::from_value::<EditOperation>(v).is_err());
}
#[test]
fn marker_move_and_draft_old_selected_root_are_atomic() {
    let (root, core, id, assets, track) = setup();
    core.edit(&id,4,serde_json::from_value(json!({"operation":"marker_create","scope":"root","name":"impact","timeMs":400,"kind":"cue"})).unwrap()).unwrap();
    let mut v = input(&track);
    v["at"] = json!({"type":"marker","markerName":"impact","offsetMs":-50});
    let draft = core
        .create_draft(&id, 5, vec![serde_json::from_value(v).unwrap()], None)
        .unwrap();
    assert_eq!(
        draft.audio_event_asset_ids.as_ref().unwrap(),
        &[assets[1].clone()]
    );
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .items[0]
            .start_ms(),
        350
    );
    core.edit(&id,5,serde_json::from_value(json!({"operation":"sound_event_register","event":"impact","variantAssetIds":[assets[0]],"defaultGainDb":0,"busId":"sfx","variantSeed":0})).unwrap()).unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.delete_asset(&id, 6, &assets[1]).unwrap_err().code,
        ErrorCode::AssetInUse
    );
    assert_eq!(inventory(root.path()), before);
    let rebased = core.rebase_draft(&id, &draft.id, 6).unwrap();
    assert_eq!(
        rebased.audio_event_asset_ids.as_ref().unwrap(),
        &[assets[0].clone()]
    );
    core.commit_draft(&id, &draft.id, 6).unwrap();
    assert_eq!(media(&core, &id, &track).asset_id, assets[0]);
    let marker = core.get_project(&id).unwrap().markers[0].clone();
    core.edit(&id,7,serde_json::from_value(json!({"operation":"marker_update","scope":"root","markerId":marker.id,"name":"impact","timeMs":500,"kind":"cue"})).unwrap()).unwrap();
    assert_eq!(media(&core, &id, &track).start_ms, 450);
    assert_eq!(
        media(&core, &id, &track)
            .visual_properties
            .start_time
            .unwrap(),
        serde_json::from_value(json!({"type":"marker","markerName":"impact","offsetMs":-50}))
            .unwrap()
    );
}
#[test]
fn schema40_populated_library_adopts_without_erasure_and_legacy_forgery_fails() {
    let (root, core, id, assets, track) = setup();
    let path = root.path().join("projects").join(&id).join("project.json");
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["schemaVersion"] = json!(40);
    std::fs::write(&path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    let original = raw["soundDefinitions"].clone();
    core.get_project(&id).unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(p.schema_version, PROJECT_SCHEMA_VERSION);
    assert_eq!(
        serde_json::to_value(&p.sound_definitions).unwrap(),
        original
    );
    core.edit(
        &id,
        4,
        serde_json::from_value::<EditOperation>(input(&track)).unwrap(),
    )
    .unwrap();
    assert_eq!(media(&core, &id, &track).asset_id, assets[1]);
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    raw["schemaVersion"] = json!(40);
    std::fs::write(&path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    let before = inventory(root.path());
    assert!(core.get_project(&id).is_err());
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn aliases_component_scope_ambiguity_and_locked_tracks_are_canonical() {
    let (root, core, id, assets, track) = setup();
    let operations:Vec<opencut_editor_core::BatchEditOperation>=serde_json::from_value(json!([
 {"operation":"component_create","name":"event_component","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Audio","trackType":"audio","items":[]}],"resultAlias":"comp"},
 {"operation":"marker_create","scope":"component:@comp","name":"impact","timeMs":300,"kind":"cue","resultAlias":"cue"},
 {"operation":"sound_event_register","event":"component_hit","variantAssetIds":[assets[0]],"defaultGainDb":0,"busId":"sfx","variantSeed":0,"resultAlias":"sound"},
 {"operation":"timeline_add_audio_event","scope":"component:@comp","trackId":"local","event":"@sound","at":{"type":"marker","markerName":"impact","offsetMs":-50},"durationMs":300,"resultAlias":"hit"}
 ])).unwrap();
    let saved = core.edit_batch(&id, 4, operations).unwrap();
    assert_eq!(saved.revision, 5);
    let p = core.get_project(&id).unwrap();
    let c = &p.components[0];
    assert_eq!(c.id, saved.aliases["comp"]);
    assert_eq!(c.tracks[0].items[0].id(), saved.aliases["hit"]);
    assert_eq!(c.tracks[0].items[0].start_ms(), 250);
    let before = inventory(root.path());
    let error=core.edit(&id,5,serde_json::from_value(json!({"operation":"marker_create","scope":format!("component:{}",c.id),"name":"impact","timeMs":400,"kind":"cue"})).unwrap()).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(inventory(root.path()), before);
    let mut op = input(&track);
    op["scope"] = json!(format!("component:{}", c.id));
    op["trackId"] = json!("local");
    op["at"] = json!({"type":"milliseconds","valueMs":900});
    assert_eq!(
        core.edit(&id, 5, serde_json::from_value(op).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    core.edit(
        &id,
        5,
        serde_json::from_value(json!({"operation":"update_track","trackId":track,"locked":true}))
            .unwrap(),
    )
    .unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.edit(
            &id,
            6,
            serde_json::from_value::<EditOperation>(input(&track)).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(inventory(root.path()), before);
    let mut op = input("absent");
    assert_eq!(
        core.edit(&id, 6, serde_json::from_value(op.clone()).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::TrackNotFound
    );
    op["scope"] = json!("component:absent");
    assert_eq!(
        core.edit(&id, 6, serde_json::from_value(op).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(inventory(root.path()), before);
}
#[test]
fn all_source_versions_mixed_history_preserve_registered40_and_routed39_generations() {
    let (root, core, id, _, _) = setup();
    let dir = root.path().join("projects").join(&id);
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut sources = vec![];
    for version in 1..=40 {
        let mut raw = original.clone();
        raw["schemaVersion"] = json!(version);
        if version < 40 {
            raw.as_object_mut().unwrap().remove("soundDefinitions");
        }
        if version < 39 {
            raw.as_object_mut().unwrap().remove("audioBuses");
            for t in raw["tracks"].as_array_mut().unwrap() {
                t.as_object_mut().unwrap().remove("audioBusId");
            }
        }
        if version < 24 {
            raw.as_object_mut().unwrap().remove("markers");
        }
        if version < 19 {
            raw.as_object_mut().unwrap().remove("fonts");
        }
        sources.push(raw);
    }
    let current = sources[39].clone();
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec_pretty(&current).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec_pretty(&json!({"undo":sources,"redo":sources})).unwrap(),
    )
    .unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&p.sound_definitions).unwrap(),
        original["soundDefinitions"]
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for list in ["undo", "redo"] {
        for (index, snapshot) in history[list].as_array().unwrap().iter().enumerate() {
            assert_eq!(snapshot["schemaVersion"], 41);
            assert_eq!(
                snapshot["soundDefinitions"],
                if index == 39 {
                    original["soundDefinitions"].clone()
                } else {
                    json!([])
                }
            );
            assert_eq!(snapshot["assets"], sources[index]["assets"]);
            assert_eq!(snapshot["tracks"], sources[index]["tracks"]);
            if index >= 38 {
                assert_eq!(snapshot["audioBuses"], sources[index]["audioBuses"]);
            }
        }
    }
    let stable = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), stable);
}
#[test]
fn persisted_provenance_content_and_draft_root_forgery_fail_without_rewrite() {
    let (root, core, id, assets, track) = setup();
    let draft = core
        .create_draft(
            &id,
            4,
            vec![serde_json::from_value::<EditOperation>(input(&track)).unwrap()],
            None,
        )
        .unwrap();
    let draft_path = root
        .path()
        .join("projects")
        .join(&id)
        .join("drafts")
        .join(format!("{}.json", draft.id));
    let original = std::fs::read(&draft_path).unwrap();
    let mut raw: Value = serde_json::from_slice(&original).unwrap();
    raw["audioEventAssetIds"] = json!([assets[0]]);
    std::fs::write(&draft_path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.commit_draft(&id, &draft.id, 4).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(inventory(root.path()), before);
    std::fs::write(&draft_path, &original).unwrap();
    core.commit_draft(&id, &draft.id, 4).unwrap();
    let path = root.path().join("projects").join(&id).join("project.json");
    let original = std::fs::read(&path).unwrap();
    let raw: Value = serde_json::from_slice(&original).unwrap();
    let index = raw["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["id"] == track)
        .unwrap();
    for mutation in ["hash", "gain", "null", "extra"] {
        let mut changed = raw.clone();
        let meta = &mut changed["tracks"][index]["items"][0]["audioEvent"];
        match mutation {
            "hash" => meta["contentHash"]["digest"] = json!("0".repeat(64)),
            "gain" => meta["gainDb"] = json!(25),
            "null" => *meta = Value::Null,
            _ => meta["extra"] = json!(1),
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&changed).unwrap()).unwrap();
        let before = inventory(root.path());
        assert!(core.get_project(&id).is_err(), "{mutation}");
        assert_eq!(inventory(root.path()), before);
    }
    std::fs::write(&path, original).unwrap();
    core.get_project(&id).unwrap();
}

#[test]
fn component_marker_moves_ignore_root_names_and_keep_selection_through_timing_edits() {
    let (root, core, id, assets, _) = setup();
    let batch = core.edit_batch(&id, 4, serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"marker_create","scope":"root","name":"impact","timeMs":900,"kind":"cue"},
        {"operation":"component_create","name":"event_component","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Audio","trackType":"audio","items":[]}],"resultAlias":"comp"},
        {"operation":"marker_create","scope":"component:@comp","name":"impact","timeMs":300,"kind":"cue","resultAlias":"cue"},
        {"operation":"timeline_add_audio_event","scope":"component:@comp","trackId":"local","event":"impact","at":{"type":"marker","markerName":"impact","offsetMs":-50},"durationMs":300,"resultAlias":"hit"}
    ])).unwrap()).unwrap();
    let read = || core.get_project(&id).unwrap().components[0].tracks[0].items[0].clone();
    let initial = read();
    assert_eq!(initial.start_ms(), 250);
    let scope = format!("component:{}", batch.aliases["comp"]);
    core.edit(&id, 5, serde_json::from_value(json!({"operation":"marker_update","scope":scope,"markerId":batch.aliases["cue"],"name":"impact","timeMs":500,"kind":"cue"})).unwrap()).unwrap();
    assert_eq!(read().start_ms(), 450);
    let TimelineItem::Media(before) = initial else {
        panic!("event")
    };
    let TimelineItem::Media(after) = read() else {
        panic!("event")
    };
    assert_eq!(after.asset_id, assets[1]);
    assert_eq!(after.audio_event, before.audio_event);
    let bytes = inventory(root.path());
    let error = core.edit(&id, 6, serde_json::from_value(json!({"operation":"marker_update","scope":scope,"markerId":batch.aliases["cue"],"name":"impact","timeMs":900,"kind":"cue"})).unwrap()).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(inventory(root.path()), bytes);
    core.undo(&id, 6).unwrap();
    assert_eq!(read().start_ms(), 250);
    core.redo(&id, 7).unwrap();
    assert_eq!(read().start_ms(), 450);
}
