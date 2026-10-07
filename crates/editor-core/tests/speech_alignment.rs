//! Canonical producer provenance and persistence boundary conformance.
use opencut_editor_core::{SpeechAlignment, SpeechGeneration};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/speech-alignment-v1.json")).unwrap()
}
fn alignment() -> SpeechAlignment {
    serde_json::from_value(catalog()["valid"][0]["alignment"].clone()).unwrap()
}
fn generation() -> Value {
    json!({"request":{"text":"Hello", "language":"en", "voiceId":"af_heart", "speed":1.0,
        "textOptions":{"normalization":"basic","pronunciations":[],"chunking":"sentence","sentencePauseMs":120}},"providerId":"synthesis","modelId":"synthesis-model", "modelVersion":null,
        "sampleRateHz":24000,"generatedAtMs":1})
}

#[test]
fn canonical_alignment_cases_preserve_values_or_reject() {
    let cases = catalog();
    for case in cases["valid"].as_array().unwrap() {
        let value: SpeechAlignment = serde_json::from_value(case["alignment"].clone())
            .unwrap_or_else(|error| panic!("{}: {error}", case["id"]));
        value
            .validate_for_duration(case["durationMs"].as_u64())
            .unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            case["alignment"],
            "{}",
            case["id"]
        );
    }
    for case in cases["invalid"].as_array().unwrap() {
        let rejected = match serde_json::from_value::<SpeechAlignment>(case["alignment"].clone()) {
            Err(_) => true,
            Ok(value) => value
                .validate_for_duration(case["durationMs"].as_u64())
                .is_err(),
        };
        assert!(rejected, "{} must fail", case["id"]);
    }
}

#[test]
fn optional_alignment_is_omitted_but_null_and_missing_nullable_identity_are_rejected() {
    let original = generation();
    let value: SpeechGeneration = serde_json::from_value(original.clone()).unwrap();
    assert!(value.alignment.is_none());
    assert_eq!(serde_json::to_value(value).unwrap(), original);
    let mut with_null = original;
    with_null["alignment"] = Value::Null;
    assert!(serde_json::from_value::<SpeechGeneration>(with_null).is_err());
    for field in ["modelId", "modelVersion"] {
        let mut record = serde_json::to_value(alignment()).unwrap();
        record.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<SpeechAlignment>(record).is_err());
    }
    let mut record = serde_json::to_value(alignment()).unwrap();
    record["modelId"] = Value::Null;
    record["modelVersion"] = Value::Null;
    assert!(serde_json::from_value::<SpeechAlignment>(record).is_ok());
}

#[test]
fn independent_granularities_allow_gaps_without_nesting() {
    let mut value = alignment();
    let mut segment = value.sentences[0].clone();
    segment.start_ms = 1200;
    segment.end_ms = 1300;
    value.words = vec![segment.clone()];
    segment.start_ms = 1500;
    segment.end_ms = 1600;
    value.words.push(segment);
    assert!(value.validate_for_duration(Some(1600)).is_ok());
    assert!(value.validate_for_duration(Some(1599)).is_err());
    assert!(value.validate_for_duration(None).is_err());
    assert!(value.validate_for_duration(Some(0)).is_err());
}

#[test]
fn utf8_limits_and_total_text_budget_are_exact() {
    let mut value = alignment();
    value.sentences[0].text = "é".repeat(2048);
    assert!(value.validate().is_ok());
    value.sentences[0].text.push('a');
    assert!(value.validate().is_err());
    value.sentences[0].text = "a".repeat(4096);
    value.provider_id = "é".repeat(128);
    value.model_id = Some(value.provider_id.clone());
    value.model_version = Some(value.provider_id.clone());
    assert!(value.validate().is_ok());
    for field in 0..3 {
        let mut changed = value.clone();
        match field {
            0 => changed.provider_id.push('a'),
            1 => changed.model_id.as_mut().unwrap().push('a'),
            _ => changed.model_version.as_mut().unwrap().push('a'),
        }
        assert!(changed.validate().is_err());
    }
    let segment = value.sentences[0].clone();
    value.sentences = (0..256)
        .map(|index| {
            let mut segment = segment.clone();
            segment.start_ms = index;
            segment.end_ms = index + 1;
            segment
        })
        .collect();
    assert!(value.validate_for_duration(Some(256)).is_ok());
    let mut extra = segment;
    extra.text = "a".into();
    extra.start_ms = 256;
    extra.end_ms = 257;
    value.sentences.push(extra);
    assert!(value.validate().is_err());
}

#[test]
fn combined_count_and_json_safe_integer_boundaries_are_exact() {
    let mut value = alignment();
    let segment = value.sentences[0].clone();
    value.sentences.clear();
    value.words = (0..100_000)
        .map(|index| {
            let mut segment = segment.clone();
            segment.text = "a".into();
            segment.start_ms = index;
            segment.end_ms = index + 1;
            segment
        })
        .collect();
    assert!(value.validate_for_duration(Some(100_000)).is_ok());
    value.phonemes.push(segment.clone());
    assert!(value.validate().is_err());
    value.words.clear();
    value.phonemes[0].start_ms = 9_007_199_254_740_990;
    value.phonemes[0].end_ms = 9_007_199_254_740_991;
    assert!(
        value
            .validate_for_duration(Some(9_007_199_254_740_991))
            .is_ok()
    );
    value.phonemes[0].end_ms += 1;
    assert!(value.validate().is_err());
}

use opencut_editor_core::{
    CommitGeneratedAssetRequest, EditorCore, GeneratedAssetOrigin, MediaProbeFacts, PathPolicy,
    ProjectSettings, ReplaceGeneratedAssetRequest,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, result);
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, &mut result);
    result
}
fn setup() -> (
    tempfile::TempDir,
    EditorCore,
    String,
    String,
    PathBuf,
    PathBuf,
) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    let generated = root.path().join("generated");
    std::fs::create_dir(&media).unwrap();
    std::fs::create_dir(&generated).unwrap();
    let source = generated.join("speech.wav");
    std::fs::write(&source, b"canonical speech media").unwrap();
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
        .create_project("alignment", ProjectSettings::default())
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
    let dir = root.path().join("projects").join(&id);
    (root, core, id, track, source, dir)
}
fn origin(with_alignment: bool) -> GeneratedAssetOrigin {
    let mut value = generation();
    if with_alignment {
        value["alignment"] = serde_json::to_value(alignment()).unwrap();
    }
    GeneratedAssetOrigin::SpeechSynthesis(serde_json::from_value(value).unwrap())
}
fn insert(
    id: &str,
    track: &str,
    source: &Path,
    provenance: GeneratedAssetOrigin,
) -> CommitGeneratedAssetRequest {
    CommitGeneratedAssetRequest {
        project_id: id.into(),
        expected_revision: 0,
        path: source.to_owned(),
        track_id: track.into(),
        start_ms: 0,
        duration_ms: 1000,
        display_name: "speech".into(),
        origin: provenance,
        probe: MediaProbeFacts::default(),
    }
}
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[test]
fn legacy_adoption_aligned_replacement_history_and_no_rewrite_reopen() {
    let (_root, core, id, track, source, dir) = setup();
    let project_file = dir.join("project.json");
    let mut legacy = read_json(&project_file);
    legacy["schemaVersion"] = json!(37);
    write_json(&project_file, &legacy);
    let original = origin(true);
    let committed = core
        .commit_generated_asset(insert(&id, &track, &source, original.clone()))
        .unwrap();
    let mut replacement = alignment();
    replacement.quality = opencut_editor_core::SpeechAlignmentQuality::Estimated;
    replacement.provider_id = "different-alignment-producer".into();
    replacement.model_id = None;
    let GeneratedAssetOrigin::SpeechSynthesis(mut next) = origin(false);
    next.alignment = Some(replacement);
    let next_origin = GeneratedAssetOrigin::SpeechSynthesis(next);
    let replacement_source = source.with_file_name("replacement.wav");
    std::fs::write(&replacement_source, b"replacement speech media").unwrap();
    let replaced = core
        .replace_generated_asset(ReplaceGeneratedAssetRequest {
            project_id: id.clone(),
            expected_revision: 1,
            item_id: committed.item_id.clone(),
            path: replacement_source,
            duration_ms: 1000,
            origin: next_origin.clone(),
            probe: MediaProbeFacts::default(),
        })
        .unwrap();
    assert_eq!(replaced.item_id, committed.item_id);
    assert_ne!(replaced.asset_id, committed.asset_id);
    assert_eq!(
        core.get_project(&id).unwrap().assets[0].origin.as_ref(),
        Some(&next_origin)
    );
    core.undo(&id, 2).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().assets[0].origin.as_ref(),
        Some(&original)
    );
    core.redo(&id, 3).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().assets[0].origin.as_ref(),
        Some(&next_origin)
    );
    let history = read_json(&dir.join("history.json"));
    for snapshot in history["undo"]
        .as_array()
        .unwrap()
        .iter()
        .chain(history["redo"].as_array().unwrap())
    {
        assert_eq!(snapshot["schemaVersion"], 38);
    }
    let before = inventory(&dir);
    core.get_project(&id).unwrap();
    assert_eq!(inventory(&dir), before);
}

#[test]
fn invalid_current_and_retained_alignment_or_future_schema_never_adopt_bytes() {
    let (_root, core, id, track, source, dir) = setup();
    core.commit_generated_asset(insert(&id, &track, &source, origin(true)))
        .unwrap();
    let project_file = dir.join("project.json");
    let history_file = dir.join("history.json");
    let original_project = read_json(&project_file);
    let original_history = read_json(&history_file);
    for location in ["current", "undo", "redo"] {
        for fault in [
            "zero-span",
            "unknown-duration",
            "pre-introduction",
            "pre-introduction-null",
            "future",
        ] {
            let mut project = original_project.clone();
            let mut history = original_history.clone();
            let mut broken = original_project.clone();
            match fault {
                "zero-span" => {
                    broken["assets"][0]["origin"]["generation"]["alignment"]["sentences"][0]["endMs"] =
                        json!(0)
                }
                "unknown-duration" => broken["assets"][0]["durationMs"] = Value::Null,
                "pre-introduction" => broken["schemaVersion"] = json!(37),
                "pre-introduction-null" => {
                    broken["schemaVersion"] = json!(37);
                    broken["assets"][0]["origin"]["generation"]["alignment"] = Value::Null;
                }
                _ => broken["schemaVersion"] = json!(39),
            }
            if location == "current" {
                project = broken;
            } else {
                history[location] = json!([broken]);
            }
            write_json(&project_file, &project);
            write_json(&history_file, &history);
            let before = inventory(&dir);
            assert!(core.get_project(&id).is_err(), "{location}/{fault}");
            assert_eq!(inventory(&dir), before, "{location}/{fault}");
        }
    }
    write_json(&project_file, &original_project);
    write_json(&history_file, &original_history);
    assert!(core.get_project(&id).is_ok());
}

#[test]
fn mixed_legacy_history_preserves_genuine_unaligned_speech_without_inference() {
    let (_root, core, id, track, source, dir) = setup();
    core.commit_generated_asset(insert(&id, &track, &source, origin(false)))
        .unwrap();
    let project_file = dir.join("project.json");
    let history_file = dir.join("history.json");
    let mut project = read_json(&project_file);
    project["schemaVersion"] = json!(37);
    let mut history = read_json(&history_file);
    history["undo"][0]["schemaVersion"] = json!(36);
    history["redo"] = json!([project.clone()]);
    write_json(&project_file, &project);
    write_json(&history_file, &history);
    let adopted = core.get_project(&id).unwrap();
    assert_eq!(adopted.schema_version, 38);
    let GeneratedAssetOrigin::SpeechSynthesis(provenance) =
        adopted.assets[0].origin.as_ref().unwrap();
    assert!(provenance.alignment.is_none());
    assert_eq!(adopted.assets[0].origin.as_ref(), Some(&origin(false)));
    for snapshot in read_json(&history_file)["undo"]
        .as_array()
        .unwrap()
        .iter()
        .chain(read_json(&history_file)["redo"].as_array().unwrap())
    {
        assert_eq!(snapshot["schemaVersion"], 38);
    }
    assert!(
        read_json(&project_file)["assets"][0]["origin"]["generation"]
            .get("alignment")
            .is_none()
    );
}

#[test]
fn raw_duplicate_alignment_fields_remain_rejected_after_legacy_inspection() {
    let (_root, core, id, track, source, dir) = setup();
    core.commit_generated_asset(insert(&id, &track, &source, origin(true)))
        .unwrap();
    let project_file = dir.join("project.json");
    let original = std::fs::read_to_string(&project_file).unwrap();
    for duplicate in [
        "\"quality\":\"native\",\"quality\":\"forced\"",
        "\"providerId\":\"x\",\"providerId\":\"y\"",
    ] {
        let changed = if duplicate.contains("quality") {
            original.replace("\"quality\": \"native\"", duplicate)
        } else {
            original.replace("\"providerId\": \"speech-provider\"", duplicate)
        };
        assert_ne!(changed, original);
        std::fs::write(&project_file, changed).unwrap();
        let before = inventory(&dir);
        assert!(core.get_project(&id).is_err());
        assert_eq!(inventory(&dir), before);
    }
}

#[test]
fn current_catalog_markers_preserve_independently_captured_schema37_bytes() {
    use sha2::{Digest, Sha256};
    let pins: Value = serde_json::from_str(include_str!(
        "../../../apps/agent-bridge/tests/fixtures/speech-alignment-predecessor-pins.json"
    ))
    .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts");
    for name in pins["currentMarkerCatalogs"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        let current = std::fs::read_to_string(root.join(name)).unwrap();
        assert_eq!(
            current.matches("\"projectSchemaVersion\": 38").count(),
            1,
            "{name}"
        );
        let prior = current.replacen(
            "\"projectSchemaVersion\": 38",
            "\"projectSchemaVersion\": 37",
            1,
        );
        let digest = format!("{:x}", Sha256::digest(prior.as_bytes()));
        assert_eq!(
            digest,
            pins["catalogs"][name]["rawSha256"].as_str().unwrap(),
            "{name}"
        );
    }
}

#[test]
fn aligned_request_failures_preserve_current_and_legacy_files_and_exact_codes() {
    use opencut_editor_core::ErrorCode;
    for legacy in [false, true] {
        let (_root, core, id, track, source, dir) = setup();
        let committed = core
            .commit_generated_asset(insert(&id, &track, &source, origin(false)))
            .unwrap();
        if legacy {
            let path = dir.join("project.json");
            let mut raw = read_json(&path);
            raw["schemaVersion"] = json!(37);
            write_json(&path, &raw);
        }
        let before = inventory(&dir);
        for fault in ["stale", "missing", "duration"] {
            let mut request = insert(&id, &track, &source, origin(true));
            request.expected_revision = 1;
            let expected = match fault {
                "stale" => {
                    request.expected_revision = 0;
                    ErrorCode::RevisionConflict
                }
                "missing" => {
                    request.track_id = "missing".into();
                    ErrorCode::ValidationFailed
                }
                _ => {
                    request.duration_ms = 999;
                    ErrorCode::ValidationFailed
                }
            };
            assert_eq!(
                core.commit_generated_asset(request).unwrap_err().code,
                expected
            );
            assert_eq!(inventory(&dir), before, "insert/legacy={legacy}/{fault}");
            let mut request = ReplaceGeneratedAssetRequest {
                project_id: id.clone(),
                expected_revision: 1,
                item_id: committed.item_id.clone(),
                path: source.clone(),
                duration_ms: 1000,
                origin: origin(true),
                probe: MediaProbeFacts::default(),
            };
            let expected = match fault {
                "stale" => {
                    request.expected_revision = 0;
                    ErrorCode::RevisionConflict
                }
                "missing" => {
                    request.item_id = "missing".into();
                    ErrorCode::ItemNotFound
                }
                _ => {
                    request.duration_ms = 999;
                    ErrorCode::ValidationFailed
                }
            };
            assert_eq!(
                core.replace_generated_asset(request).unwrap_err().code,
                expected
            );
            assert_eq!(inventory(&dir), before, "replace/legacy={legacy}/{fault}");
        }
    }
}
