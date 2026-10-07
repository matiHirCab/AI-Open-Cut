use opencut_editor_core::{
    BatchEditOperation, CommitGeneratedAssetRequest, EditOperation, EditorCore, ErrorCode,
    GeneratedAssetOrigin, MediaProbeFacts, PathPolicy, ProjectSettings, SpeechMarkerPolicy,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/speech-alignment-markers-v1.json"
    ))
    .unwrap()
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn inventory(root: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    fn visit(path: &Path, files: &mut BTreeMap<std::path::PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, files);
            } else {
                files.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    visit(root, &mut files);
    files
}
fn setup(
    aligned: bool,
    policy: SpeechMarkerPolicy,
) -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    let generated = root.path().join("generated");
    std::fs::create_dir(&media).unwrap();
    std::fs::create_dir(&generated).unwrap();
    let source = generated.join("speech.wav");
    std::fs::write(&source, b"speech marker fixture media").unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap(),
    );
    let id = core
        .create_project("speech markers", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Audio)
        .unwrap()
        .id
        .clone();
    let mut generation = json!({"request":{"text":"EVERY. SINGLE. ONE.","language":"en","voiceId":"fixture","speed":1.0},"providerId":"fixture","modelId":"fixture","modelVersion":null,"sampleRateHz":24000,"generatedAtMs":1});
    if aligned {
        generation["alignment"] = fixture()["alignment"].clone();
    }
    let inserted = core
        .commit_generated_asset(CommitGeneratedAssetRequest {
            marker_policy: policy,
            project_id: id.clone(),
            expected_revision: 0,
            path: source,
            track_id: track,
            start_ms: 1000,
            duration_ms: 1000,
            display_name: "speech".into(),
            origin: GeneratedAssetOrigin::SpeechSynthesis(
                serde_json::from_value(generation).unwrap(),
            ),
            probe: MediaProbeFacts::default(),
        })
        .unwrap();
    (root, core, id, inserted.asset_id)
}
fn edit(asset: &str, policy: Value) -> Value {
    json!({"operation":"speech_markers_generate","assetId":asset,"scope":"root","startMs":1000,"markerPolicy":policy})
}
fn cues(core: &EditorCore, id: &str) -> Value {
    Value::Array(
        core.get_project(id)
            .unwrap()
            .markers
            .iter()
            .map(|marker| json!({"name":marker.name,"timeMs":marker.time_ms}))
            .collect(),
    )
}

#[test]
fn canonical_policy_results_and_exact_lifecycle() {
    let contract = fixture();
    for policy in contract["policies"].as_array().unwrap() {
        let (_root, core, id, asset) = setup(true, SpeechMarkerPolicy::None {});
        let result = core.edit(&id, 1, op(edit(&asset, policy.clone()))).unwrap();
        assert_eq!(result.revision, 2);
        assert_eq!(
            cues(&core, &id),
            contract["expected"][policy["type"].as_str().unwrap()]
        );
        let markers = core.get_project(&id).unwrap().markers;
        let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        assert_eq!(
            core.edit(&id, 1, op(edit(&asset, policy.clone())))
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
        core.undo(&id, 2).unwrap();
        assert!(core.get_project(&id).unwrap().markers.is_empty());
        core.redo(&id, 3).unwrap();
        assert_eq!(core.get_project(&id).unwrap().markers, markers);
        let reopened = EditorCore::new(core.paths().clone());
        assert_eq!(reopened.get_project(&id).unwrap().markers, markers);
    }
}

#[test]
fn closed_policies_requests_and_semantic_failures_preserve_bytes() {
    let (root, core, id, asset) = setup(false, SpeechMarkerPolicy::None {});
    for policy in fixture()["invalidPolicies"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<SpeechMarkerPolicy>(policy.clone()).is_err(),
            "{policy}"
        );
    }
    let before = inventory(root.path());
    for selection in fixture()["invalidSelections"].as_array().unwrap() {
        let mut request = edit(&asset, json!({"type":"selected_word","indices":selection}));
        request["alignment"] = fixture()["alignment"].clone();
        assert_eq!(
            core.edit(&id, 1, op(request)).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    for (patch, code) in [
        (json!({"assetId":"missing"}), ErrorCode::AssetNotFound),
        (
            json!({"scope":"component:missing"}),
            ErrorCode::ItemNotFound,
        ),
        (json!({}), ErrorCode::ValidationFailed),
        (
            json!({"alignment":{"sentences":[],"words":[],"phonemes":[],"quality":"forced","providerId":"test","modelId":null,"modelVersion":null}}),
            ErrorCode::ValidationFailed,
        ),
        (
            json!({"startMs":9007199254740991_u64,"alignment":fixture()["alignment"]}),
            ErrorCode::ValidationFailed,
        ),
    ] {
        let mut request = edit(&asset, json!({"type":"all_word"}));
        for (key, value) in patch.as_object().unwrap() {
            request[key] = value.clone();
        }
        assert_eq!(core.edit(&id, 1, op(request)).unwrap_err().code, code);
        assert_eq!(inventory(root.path()), before);
    }
    let mut missing_words = fixture()["alignment"].clone();
    missing_words["words"] = json!([]);
    let mut out_of_duration = fixture()["alignment"].clone();
    out_of_duration["words"][2]["endMs"] = json!(1001);
    for alignment in [missing_words, out_of_duration] {
        let mut request = edit(&asset, json!({"type":"all_word"}));
        request["alignment"] = alignment;
        assert_eq!(
            core.edit(&id, 1, op(request)).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    let mut too_many_indices = edit(
        &asset,
        json!({"type":"selected_word","indices":vec![0;4097]}),
    );
    too_many_indices["alignment"] = fixture()["alignment"].clone();
    assert_eq!(
        core.edit(&id, 1, op(too_many_indices)).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    for patch in [json!({"alignment":null}), json!({"unknown":true})] {
        let mut request = edit(&asset, json!({"type":"none"}));
        for (key, value) in patch.as_object().unwrap() {
            request[key] = value.clone();
        }
        assert!(serde_json::from_value::<EditOperation>(request).is_err());
    }
    core.edit(&id, 1, op(edit(&asset, json!({"type":"none"}))))
        .unwrap();
    assert!(core.get_project(&id).unwrap().markers.is_empty());
}

#[test]
fn canonical_naming_collision_and_long_text() {
    let (_root, core, id, asset) = setup(false, SpeechMarkerPolicy::None {});
    for case in fixture()["naming"].as_array().unwrap() {
        let granularity = case["granularity"].as_str().unwrap();
        let ordinal = case["ordinal"].as_u64().unwrap() as usize;
        let mut alignment = fixture()["alignment"].clone();
        let segments: Vec<Value> = (0..ordinal).map(|index| json!({"text":if index+1==ordinal {case["text"].as_str().unwrap()}else{"prefix"},"startMs":index*10,"endMs":index*10+5})).collect();
        alignment[if granularity == "word" {
            "words"
        } else {
            "sentences"
        }] = json!(segments);
        let mut request = edit(
            &asset,
            if granularity == "word" {
                json!({"type":"selected_word","indices":[ordinal-1]})
            } else {
                json!({"type":"sentence"})
            },
        );
        request["alignment"] = alignment;
        let revision = core.get_project(&id).unwrap().revision;
        core.edit(&id, revision, op(request)).unwrap();
        assert_eq!(
            core.get_project(&id).unwrap().markers.last().unwrap().name,
            case["expected"].as_str().unwrap()
        );
    }
    for name in fixture()["collision"]["existingNames"].as_array().unwrap() {
        let revision = core.get_project(&id).unwrap().revision;
        core.edit(&id,revision,op(json!({"operation":"marker_create","scope":"root","name":name,"timeMs":0,"kind":"cue"}))).unwrap();
    }
    let mut alignment = fixture()["alignment"].clone();
    alignment["words"] =
        json!([{"text":"EVERY","startMs":0,"endMs":5},{"text":"EVERY","startMs":10,"endMs":15}]);
    let mut request = edit(&asset, json!({"type":"all_word"}));
    request["alignment"] = alignment;
    let revision = core.get_project(&id).unwrap().revision;
    core.edit(&id, revision, op(request.clone())).unwrap();
    let markers = core.get_project(&id).unwrap().markers;
    // Earlier naming case already reserved EVERY, in addition to deliberately duplicated EVERY.
    assert_eq!(markers[markers.len() - 2].name, "EVERY_3");
    assert_eq!(markers[markers.len() - 1].name, "EVERY_5");
    request["alignment"]["words"] = json!([{"text":"a".repeat(4096),"startMs":0,"endMs":5}]);
    core.edit(&id, revision + 1, op(request.clone())).unwrap();
    core.edit(&id, revision + 2, op(request)).unwrap();
    let markers = core.get_project(&id).unwrap().markers;
    assert_eq!(markers[markers.len() - 2].name.len(), 120);
    assert_eq!(
        markers.last().unwrap().name,
        format!("{}_2", "a".repeat(120))
    );
}

#[test]
fn component_batch_alias_success_and_full_rollback() {
    let (root, core, id, asset) = setup(true, SpeechMarkerPolicy::None {});
    let operations:Vec<BatchEditOperation>=serde_json::from_value(json!([
      {"operation":"component_create","name":"Local","width":64,"height":64,"durationMs":2000,"tracks":[],"resultAlias":"local"},
      {"operation":"speech_markers_generate","scope":"component:@local","assetId":asset,"startMs":1000,"markerPolicy":{"type":"selected_word","indices":[0]},"resultAlias":"cue"},
      {"operation":"marker_update","scope":"component:@local","markerId":"@cue","name":"impact","timeMs":1200,"kind":"cue"}
    ])).unwrap();
    let result = core.edit_batch(&id, 1, operations).unwrap();
    assert_eq!(result.revision, 2);
    let marker = core.get_project(&id).unwrap().components[0].markers[0].clone();
    assert_eq!(marker.id, result.aliases["cue"]);
    assert_eq!(marker.name, "impact");
    let before = inventory(root.path());
    for policy in [json!({"type":"none"}), json!({"type":"all_word"})] {
        let mut request = edit(&asset, policy);
        request["resultAlias"] = json!("bad");
        let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([request])).unwrap();
        assert_eq!(
            core.edit_batch(&id, 2, batch).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    let mut request = edit(&asset, json!({"type":"all_word"}));
    request["scope"] = json!("component:@forward");
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([request])).unwrap();
    assert_eq!(
        core.edit_batch(&id, 2, batch).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    let mut request = edit(&asset, json!({"type":"all_word"}));
    request["scope"] = json!(marker.scope);
    request["startMs"] = json!(1900);
    assert_eq!(
        core.edit(&id, 2, op(request)).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    core.undo(&id, 2).unwrap();
    assert!(core.get_project(&id).unwrap().components.is_empty());
    core.redo(&id, 3).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().components[0].markers[0],
        marker
    );
}

#[test]
fn generated_speech_policy_is_one_atomic_history_entry() {
    let (_root, core, id, _) = setup(
        true,
        SpeechMarkerPolicy::SelectedWord {
            indices: vec![2, 0],
        },
    );
    assert_eq!(cues(&core, &id), fixture()["expected"]["selected_word"]);
    let project = core.get_project(&id).unwrap();
    assert_eq!(project.revision, 1);
    core.undo(&id, 1).unwrap();
    let undone = core.get_project(&id).unwrap();
    assert!(undone.assets.is_empty());
    assert!(undone.markers.is_empty());
    core.redo(&id, 2).unwrap();
    assert_eq!(core.get_project(&id).unwrap().markers, project.markers);
}

#[test]
fn source_semantics_count_boundary_and_late_batch_failure_preserve_bytes() {
    let (root, core, id, asset) = setup(false, SpeechMarkerPolicy::None {});
    let file = root.path().join("projects").join(&id).join("project.json");
    let original: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    for (field, value, code) in [
        ("hasAudio", json!(false), ErrorCode::UnsupportedMedia),
        ("durationMs", Value::Null, ErrorCode::ValidationFailed),
    ] {
        let mut project = original.clone();
        project["assets"][0][field] = value.clone();
        project["assets"][0]["probe"][field] = value;
        std::fs::write(&file, serde_json::to_vec_pretty(&project).unwrap()).unwrap();
        let before = inventory(root.path());
        assert_eq!(
            core.edit(&id, 1, op(edit(&asset, json!({"type":"none"}))))
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(inventory(root.path()), before);
    }
    let mut project = original;
    project["markers"]=json!((0..4095).map(|index|json!({"id":format!("m_{index}"),"name":format!("existing_{index}"),"scope":"root","timeMs":0,"kind":"cue"})).collect::<Vec<_>>());
    std::fs::write(&file, serde_json::to_vec_pretty(&project).unwrap()).unwrap();
    let mut request = edit(&asset, json!({"type":"all_word"}));
    request["alignment"] = fixture()["alignment"].clone();
    let before = inventory(root.path());
    assert_eq!(
        core.edit(&id, 1, op(request.clone())).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    request["markerPolicy"] = json!({"type":"selected_word","indices":[0]});
    let batch: Vec<BatchEditOperation> = serde_json::from_value(
        json!([request.clone(),{"operation":"marker_delete","scope":"root","markerId":"missing"}]),
    )
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, batch).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(inventory(root.path()), before);
    core.edit(&id, 1, op(request.clone())).unwrap();
    assert_eq!(core.get_project(&id).unwrap().markers.len(), 4096);
    let before = inventory(root.path());
    assert_eq!(
        core.edit(&id, 2, op(request)).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn generated_resource_semantic_failure_rolls_back_every_byte() {
    let (root, core, id, asset) = setup(false, SpeechMarkerPolicy::None {});
    let project = core.get_project(&id).unwrap();
    let source = root.path().join("generated").join("new.wav");
    std::fs::write(&source, b"distinct candidate resource").unwrap();
    let original = project
        .assets
        .iter()
        .find(|record| record.id == asset)
        .unwrap()
        .origin
        .clone()
        .unwrap();
    let request = CommitGeneratedAssetRequest {
        marker_policy: SpeechMarkerPolicy::AllWord {},
        project_id: id.clone(),
        expected_revision: 1,
        path: source,
        track_id: project
            .tracks
            .iter()
            .find(|track| track.track_type == opencut_editor_core::TrackType::Audio)
            .unwrap()
            .id
            .clone(),
        start_ms: 0,
        duration_ms: 1000,
        display_name: "new".into(),
        origin: original,
        probe: MediaProbeFacts::default(),
    };
    let before = inventory(root.path());
    assert_eq!(
        core.commit_generated_asset(request.clone())
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    let mut conflict = request;
    conflict.expected_revision = 0;
    assert_eq!(
        core.commit_generated_asset(conflict).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(root.path()), before);
}
