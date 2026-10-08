use opencut_editor_core::{
    AudioBus, AudioTrackRole, BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy,
    Project, ProjectSettings, TrackType, default_audio_buses, resolve_audio_bus_route,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/audio-buses-v1.json")).unwrap()
}
fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
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
fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
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
        .create_project("audio buses", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Audio)
        .unwrap()
        .id
        .clone();
    (root, core, id, track)
}
fn route(track: &str, bus: Value) -> EditOperation {
    operation(json!({"operation":"audio_track_route","scope":"root","trackId":track,"busId":bus}))
}
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn legacy(mut value: Value, version: u32) -> Value {
    let object = value.as_object_mut().unwrap();
    object.insert("schemaVersion".into(), json!(version));
    object.remove("audioBuses");
    object.remove("soundDefinitions");
    if version < 19 {
        object.remove("fonts");
    }
    if version < 24 {
        object.remove("markers");
    }
    value
}

#[test]
fn canonical_defaults_fallback_and_maximal_route_are_core_owned() {
    let (_root, core, id, track_id) = setup();
    let mut project = core.get_project(&id).unwrap();
    assert_eq!(catalog()["projectSchemaVersion"], 39);
    assert_eq!(
        project.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        serde_json::to_value(&project.audio_buses).unwrap(),
        catalog()["defaultBuses"]
    );
    assert_eq!(project.audio_buses, default_audio_buses());
    for (role, key) in [
        (AudioTrackRole::Unassigned, "unassigned"),
        (AudioTrackRole::Voiceover, "voiceover"),
        (AudioTrackRole::Music, "music"),
        (AudioTrackRole::SoundEffects, "sound_effects"),
    ] {
        let mut track = project
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .unwrap()
            .clone();
        track.audio_role = role;
        let route = resolve_audio_bus_route(&project, &track).unwrap();
        assert_eq!(route[0], catalog()["roleFallback"][key]);
        assert_eq!(route.last().unwrap(), "master");
    }
    for name in ["stemViaStem", "maximalChain"] {
        let case = &catalog()["cases"][name];
        project.audio_buses = serde_json::from_value(case["buses"].clone()).unwrap();
        let mut track = project
            .tracks
            .iter()
            .find(|t| t.id == track_id)
            .unwrap()
            .clone();
        track.audio_bus_id = Some(case["trackBusId"].as_str().unwrap().into());
        assert_eq!(
            json!(resolve_audio_bus_route(&project, &track).unwrap()),
            case["expectedRoute"]
        );
    }
}

#[test]
fn every_independent_invalid_model_case_rejects_before_publication() {
    for case in catalog()["cases"]["invalidModels"].as_array().unwrap() {
        let (root, core, id, _track) = setup();
        let path = root.path().join("projects").join(&id).join("project.json");
        let mut value = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        let buses = value["audioBuses"].as_array_mut().unwrap();
        let name = case["name"].as_str().unwrap();
        match name {
            "missing" => {
                buses.pop();
            }
            "duplicate" => buses[1]["id"] = json!("voiceover"),
            "extra" => buses.push(json!({"id":"custom","outputBusId":"master"})),
            "reordered" => buses.swap(0, 1),
            "self" => buses[1]["outputBusId"] = json!("music"),
            "cycle" => {
                buses[0]["outputBusId"] = json!("music");
                buses[1]["outputBusId"] = json!("voiceover");
            }
            "unknownOutput" => buses[1]["outputBusId"] = json!("absent"),
            "stemNull" => buses[0]["outputBusId"] = Value::Null,
            "masterOutput" => buses[3]["outputBusId"] = json!("music"),
            "missingField" => {
                buses[3].as_object_mut().unwrap().remove("outputBusId");
            }
            "unknownField" => buses[0]["expression"] = json!("unsafe"),
            _ => panic!("uncovered canonical case {name}"),
        }
        write_json(&path, &value);
        let before = inventory(root.path());
        let error = core.get_project(&id).unwrap_err();
        if !matches!(name, "missingField" | "unknownField") {
            assert_eq!(error.code, ErrorCode::InvalidArgument, "{name}");
        }
        assert!(!error.retryable, "{name}");
        assert_eq!(inventory(root.path()), before, "{name}");
    }
    assert!(serde_json::from_value::<AudioBus>(json!({"id":"master"})).is_err());
    assert!(
        serde_json::from_str::<AudioBus>(r#"{"id":"master","id":"music","outputBusId":null}"#)
            .is_err()
    );
}

#[test]
fn route_clear_history_and_reopen_keep_exact_records_and_old_roles() {
    let (root, core, id, track) = setup();
    core.edit(
        &id,
        0,
        operation(json!({"operation":"update_track","trackId":track,"audioRole":"music"})),
    )
    .unwrap();
    core.edit(
        &id,
        1,
        operation(json!({"operation":"audio_bus_set_route","busId":"music","outputBusId":"sfx"})),
    )
    .unwrap();
    core.edit(&id, 2, route(&track, json!("voiceover")))
        .unwrap();
    let routed = core.get_project(&id).unwrap();
    let t = routed.tracks.iter().find(|t| t.id == track).unwrap();
    assert_eq!(t.audio_role, AudioTrackRole::Music);
    assert_eq!(
        resolve_audio_bus_route(&routed, t).unwrap(),
        ["voiceover", "master"]
    );
    core.edit(&id, 3, route(&track, Value::Null)).unwrap();
    let cleared = core.get_project(&id).unwrap();
    let t = cleared.tracks.iter().find(|t| t.id == track).unwrap();
    assert!(t.audio_bus_id.is_none());
    assert_eq!(
        resolve_audio_bus_route(&cleared, t).unwrap(),
        ["music", "sfx", "master"]
    );
    core.undo(&id, 4).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .audio_bus_id,
        Some("voiceover".into())
    );
    core.redo(&id, 5).unwrap();
    let bytes = inventory(root.path());
    let reopened = core.get_project(&id).unwrap();
    assert_eq!(reopened.revision, 6);
    assert_eq!(reopened.audio_buses, cleared.audio_buses);
    assert_eq!(inventory(root.path()), bytes);
}

#[test]
fn all_stable_failures_and_late_batch_failure_preserve_complete_bytes() {
    let (root, core, id, track) = setup();
    let before = inventory(root.path());
    for (edit, code) in [
        (route("absent", json!("music")), ErrorCode::TrackNotFound),
        (route(&track, json!("absent")), ErrorCode::InvalidArgument),
        (
            operation(
                json!({"operation":"audio_track_route","scope":"component:absent","trackId":track,"busId":"music"}),
            ),
            ErrorCode::InvalidArgument,
        ),
        (
            operation(
                json!({"operation":"audio_track_route","scope":"invalid","trackId":track,"busId":"music"}),
            ),
            ErrorCode::InvalidArgument,
        ),
        (
            operation(
                json!({"operation":"audio_bus_set_route","busId":"music","outputBusId":"music"}),
            ),
            ErrorCode::InvalidArgument,
        ),
        (
            operation(
                json!({"operation":"audio_bus_set_route","busId":"master","outputBusId":"sfx"}),
            ),
            ErrorCode::InvalidArgument,
        ),
    ] {
        assert_eq!(core.edit(&id, 0, edit).unwrap_err().code, code);
        assert_eq!(inventory(root.path()), before);
    }
    let error = core
        .edit(&id, 999, route(&track, json!("music")))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(error.retryable);
    assert_eq!(inventory(root.path()), before);
    let edits = vec![
        operation(json!({"operation":"audio_bus_set_route","busId":"music","outputBusId":"sfx"})),
        operation(json!({"operation":"audio_bus_set_route","busId":"sfx","outputBusId":"music"})),
    ];
    assert_eq!(
        core.edit_batch(&id, 0, edits).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    core.edit(
        &id,
        0,
        operation(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let locked = inventory(root.path());
    assert_eq!(
        core.edit(&id, 1, route(&track, json!("music")))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(inventory(root.path()), locked);
    for value in [
        json!({"operation":"audio_track_route","scope":"root","trackId":track}),
        json!({"operation":"audio_track_route","scope":"root","trackId":track,"busId":"music","expression":"bad"}),
        json!({"operation":"audio_bus_set_route","busId":"music","outputBusId":null}),
    ] {
        assert!(serde_json::from_value::<EditOperation>(value).is_err());
    }
}

#[test]
fn real_track_alias_and_materialized_draft_share_routing_validation() {
    let (_root, core, id, _track) = setup();
    let result = core
        .edit_batch(
            &id,
            0,
            vec![
                BatchEditOperation {
                    edit: operation(
                        json!({"operation":"create_track","name":"routed","trackType":"audio"}),
                    ),
                    result_alias: Some("new_track".into()),
                },
                BatchEditOperation {
                    edit: route("@new_track", json!("sfx")),
                    result_alias: None,
                },
            ],
        )
        .unwrap();
    let track = result.aliases["new_track"].clone();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .audio_bus_id,
        Some("sfx".into())
    );
    let draft = core
        .create_draft(&id, 1, vec![route(&track, json!("music"))], None)
        .unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .audio_bus_id,
        Some("sfx".into())
    );
    core.commit_draft(&id, &draft.id, 1).unwrap();
    assert_eq!(
        core.get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.id == track)
            .unwrap()
            .audio_bus_id,
        Some("music".into())
    );
}

#[test]
fn component_aliases_and_silent_tracks_use_complete_candidate_validation() {
    let (root, core, id, _) = setup();
    let result = core.edit_batch(&id, 0, vec![
        BatchEditOperation {
            edit: operation(json!({"operation":"component_create","name":"Silent bus component","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Silent audio","trackType":"audio","hidden":true,"muted":true,"items":[]}]})),
            result_alias: Some("card".into()),
        },
        BatchEditOperation {
            edit: operation(json!({"operation":"audio_track_route","scope":"component:@card","trackId":"local","busId":"voiceover"})),
            result_alias: None,
        },
    ]).unwrap();
    let project = core.get_project(&id).unwrap();
    let component = project
        .components
        .iter()
        .find(|component| component.id == result.aliases["card"])
        .unwrap();
    let track = &component.tracks[0];
    assert!(track.hidden && track.muted && track.items.is_empty());
    assert_eq!(track.audio_role, AudioTrackRole::Unassigned);
    assert_eq!(
        resolve_audio_bus_route(&project, track).unwrap(),
        ["voiceover", "master"]
    );
    let path = root.path().join("projects").join(&id).join("project.json");
    let baseline = serde_json::to_value(project).unwrap();
    for kind in ["audio", "video", "overlay", "caption"] {
        let mut malformed = baseline.clone();
        malformed["components"][0]["tracks"][0]["trackType"] = json!(kind);
        malformed["components"][0]["tracks"][0]["audioBusId"] = if matches!(kind, "audio" | "video")
        {
            json!("absent")
        } else {
            json!("music")
        };
        write_json(&path, &malformed);
        let before = inventory(root.path());
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::InvalidArgument,
            "{kind}"
        );
        assert_eq!(inventory(root.path()), before, "{kind}");
    }
}

#[test]
fn genuine_mixed_legacy_history_adopts_defaults_once_without_other_changes() {
    let (root, core, id, _track) = setup();
    let dir = root.path().join("projects").join(&id);
    let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let versions = catalog()["cases"]["migrationVersions"]
        .as_array()
        .unwrap()
        .clone();
    let sources: Vec<Value> = versions
        .iter()
        .map(|v| legacy(current.clone(), v.as_u64().unwrap() as u32))
        .collect();
    write_json(&dir.join("project.json"), sources.last().unwrap());
    write_json(
        &dir.join("history.json"),
        &json!({"undo":sources[..3],"redo":sources[3..]}),
    );
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.audio_buses, default_audio_buses());
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for (source, target) in sources.iter().zip(
        history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .chain(history["redo"].as_array().unwrap()),
    ) {
        assert_eq!(target["audioBuses"], catalog()["defaultBuses"]);
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
            assert_eq!(target[key], source[key], "{key}");
        }
    }
    let stable = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), stable);
}

#[test]
fn legacy_dynamic_slot_keys_matching_bus_fields_remain_user_values() {
    let (root, core, id, _) = setup();
    let mut slots: Value =
        serde_json::from_str(include_str!("../../../contracts/template-slots-v1.json")).unwrap();
    let base = slots["valid"]
        .as_array_mut()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "number")
        .unwrap()["slot"]
        .clone();
    let bindings: Vec<Value> = ["audioBuses", "audioBusId"]
        .iter()
        .map(|key| {
            let mut value = base.clone();
            value["id"] = json!(key);
            value["binding"]["targetLayerId"] = json!(key);
            value
        })
        .collect();
    let items: Vec<Value> = ["audioBuses", "audioBusId"].iter().enumerate().map(|(order, key)| json!({"id":key,"type":"rectangle","keyframes":[],"startMs":0,"durationMs":1000,"stackOrder":order,"zIndex":order,"color":"#ffffff","width":32,"height":32,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).collect();
    let component = core.edit(&id, 0, operation(json!({"operation":"component_create","name":"Dynamic keys","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"visual","name":"Visual","trackType":"overlay","items":items}],"slots":bindings}))).unwrap().changed_ids[0].clone();
    let values = json!({"audioBuses":{"type":"number","value":0.25},"audioBusId":{"type":"number","value":0.75}});
    let outer = core.edit(&id, 1, operation(json!({"operation":"component_create","name":"Outer","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"outer","name":"Outer","trackType":"overlay","items":[{"type":"component_instance","id":"nested","componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":values}]}]}))).unwrap().changed_ids[0].clone();
    let source = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let dir = root.path().join("projects").join(&id);
    write_json(&dir.join("project.json"), &legacy(source.clone(), 38));
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(migrated["components"], source["components"]);
    let component = migrated["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["id"] == outer)
        .unwrap();
    assert_eq!(component["tracks"][0]["items"][0]["slotValues"], values);
    assert_eq!(migrated["audioBuses"], catalog()["defaultBuses"]);
    let stable = inventory(root.path());
    core.get_project(&id).unwrap();
    assert_eq!(inventory(root.path()), stable);
}

#[test]
fn premature_fields_malformed_retained_routes_and_failed_legacy_edits_never_publish() {
    let (root, core, id, track) = setup();
    let dir = root.path().join("projects").join(&id);
    let current = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let old = legacy(current.clone(), 38);
    let mut missing = current.clone();
    missing.as_object_mut().unwrap().remove("audioBuses");
    assert!(serde_json::from_value::<Project>(missing).is_err());
    for field in [Value::Null, catalog()["defaultBuses"].clone()] {
        let mut bad = old.clone();
        bad["audioBuses"] = field;
        assert!(serde_json::from_value::<Project>(bad).is_err());
    }
    let mut premature = old.clone();
    premature["tracks"][0]["audioBusId"] = Value::Null;
    assert!(serde_json::from_value::<Project>(premature).is_err());
    for malformed in [Value::Null, json!([])] {
        let mut bad = current.clone();
        bad["audioBuses"] = malformed;
        write_json(&dir.join("project.json"), &bad);
        let before = inventory(root.path());
        assert!(core.get_project(&id).is_err());
        assert_eq!(inventory(root.path()), before);
    }
    write_json(&dir.join("project.json"), &old);
    let before = inventory(root.path());
    assert_eq!(
        core.edit(&id, 0, route(&track, json!("absent")))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before);
    assert_eq!(
        core.edit(&id, 9, route(&track, json!("music")))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(root.path()), before);
    assert_eq!(
        core.edit(&id, 0, route("absent", json!("music")))
            .unwrap_err()
            .code,
        ErrorCode::TrackNotFound
    );
    assert_eq!(inventory(root.path()), before);
    let mut locked_legacy = old.clone();
    locked_legacy["tracks"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|value| value["id"] == track)
        .unwrap()["locked"] = json!(true);
    write_json(&dir.join("project.json"), &locked_legacy);
    let locked_before = inventory(root.path());
    assert_eq!(
        core.edit(&id, 0, route(&track, json!("music")))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(inventory(root.path()), locked_before);
    write_json(&dir.join("project.json"), &old);
    let mut malformed = current.clone();
    malformed["tracks"][0]["audioBusId"] = json!("absent");
    write_json(
        &dir.join("history.json"),
        &json!({"undo":[],"redo":[malformed]}),
    );
    let before = inventory(root.path());
    assert!(core.get_project(&id).is_err());
    assert_eq!(inventory(root.path()), before);
    let mut future = old;
    future["schemaVersion"] = json!(opencut_editor_core::PROJECT_SCHEMA_VERSION + 1);
    future["audioBuses"] = catalog()["defaultBuses"].clone();
    write_json(&dir.join("project.json"), &future);
    let before = inventory(root.path());
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InternalError
    );
    assert_eq!(inventory(root.path()), before);
}
