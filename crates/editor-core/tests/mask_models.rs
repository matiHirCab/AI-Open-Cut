use opencut_editor_core::{EditOperation, EditorCore, PathPolicy, ProjectSettings};
use serde_json::{Value, json};

fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).expect("supported typed edit")
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
        .create_project(
            "Mask models",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

fn rectangle(track: &str) -> Value {
    json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":600,
        "width":24,"height":16,"color":"#ff0000",
        "transform":{"positionX":20,"positionY":20,"scale":1,"opacity":0.75}})
}

fn mask(id: &str) -> Value {
    numeric_values(
        json!({"id":id,"source":{"type":"path","path":{"fillRule":"nonzero","commands":[
        {"type":"moveTo","to":{"x":0,"y":0}},
        {"type":"lineTo","to":{"x":32,"y":0}},
        {"type":"lineTo","to":{"x":32,"y":32}},
        {"type":"close"}]},"paint":{"type":"solid","color":{"r":0.2,"g":0.4,"b":0.8,"a":0.25}}},
        "channel":"luma","operation":"subtract","inverted":true,"featherPx":12,"expansionPx":-3,
        "transform":{"position":{"x":3,"y":4,"unit":"pixels"},"anchor":{"x":0.25,"y":0.75},
            "scaleX":1.5,"scaleY":0.5,"rotationDeg":30,"skewXDeg":5,"skewYDeg":-5,"opacity":0.5}}),
    )
}

#[test]
fn standalone_mask_model_is_accepted_and_observable() {
    let (_root, core, id, track) = setup();
    let added = core.edit(&id, 0, op(rectangle(&track))).unwrap();
    let item = &added.changed_ids[0];
    let masks = json!([mask("reveal")]);
    let edit = serde_json::from_value::<EditOperation>(
        json!({"operation":"update_item","itemId":item,"masks":masks}),
    );
    assert!(edit.is_ok(), "approved mask model must decode: {edit:?}");
    core.edit(&id, 1, edit.unwrap()).unwrap();
    let project = core.get_project(&id).unwrap();
    let actual = serde_json::to_value(project.find_item(item).unwrap()).unwrap();
    assert_eq!(actual["masks"], masks);
}

fn inventory(
    core: &EditorCore,
    id: &str,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        dir: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    visit(&core.paths().project_dir(id).unwrap(), &mut out);
    out
}

// JSON integer and decimal spellings represent the same typed finite number.
// Preserve every field and array order while comparing its semantic value.
fn numeric_values(mut value: Value) -> Value {
    match &mut value {
        Value::Number(n) => {
            *n = serde_json::Number::from_f64(n.as_f64().unwrap()).unwrap();
        }
        Value::Array(values) => {
            for child in values {
                *child = numeric_values(child.take());
            }
        }
        Value::Object(fields) => {
            for child in fields.values_mut() {
                *child = numeric_values(child.take());
            }
        }
        _ => {}
    }
    value
}

fn item_masks(core: &EditorCore, id: &str, item: &str) -> Value {
    let project = core.get_project(id).unwrap();
    serde_json::to_value(&project.find_item(item).unwrap().visual_properties().masks).unwrap()
}

fn update(item: &str, masks: Value) -> EditOperation {
    op(json!({"operation":"update_item","itemId":item,"masks":masks}))
}

#[test]
fn canonical_cases_and_ordered_stacks_use_the_public_owner_and_preserve_rejected_bytes() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    for family in ["cases", "stackCases"] {
        for case in catalog[family].as_array().unwrap() {
            let (_root, core, id, track) = setup();
            let item = core
                .edit(&id, 0, op(rectangle(&track)))
                .unwrap()
                .changed_ids[0]
                .clone();
            let value = numeric_values(if family == "cases" {
                json!([case["value"]])
            } else {
                case["value"].clone()
            });
            let before = inventory(&core, &id);
            let decoded = serde_json::from_value::<EditOperation>(
                json!({"operation":"update_item","itemId":item,"masks":value}),
            );
            if case["accepted"] == true {
                // Model acceptance remains independent of schema-33 rendering
                // certification. The canonical maximum feather/expansion vector
                // is structurally valid but its discrete Gaussian work exceeds
                // the unchanged output-frame rendering budget on this source.
                for authored in value.as_array().unwrap() {
                    serde_json::from_value::<opencut_editor_core::Mask>(authored.clone())
                        .unwrap()
                        .validate()
                        .unwrap();
                }
                if case["name"] == "maximum-parameters" {
                    let error = core.edit(&id, 1, decoded.unwrap()).unwrap_err();
                    assert_eq!(error.code, opencut_editor_core::ErrorCode::InvalidArgument);
                    assert!(!error.retryable);
                    assert!(error.message.contains("work exceeds"));
                    assert_eq!(inventory(&core, &id), before);
                } else {
                    core.edit(&id, 1, decoded.unwrap())
                        .unwrap_or_else(|error| panic!("{}: {error:?}", case["name"]));
                    assert_eq!(item_masks(&core, &id, &item), value, "{}", case["name"]);
                }
            } else {
                if let Ok(edit) = decoded {
                    assert!(core.edit(&id, 1, edit).is_err(), "{}", case["name"]);
                }
                assert_eq!(inventory(&core, &id), before, "{}", case["name"]);
            }
        }
    }
    for case in catalog["rawRejectedCases"].as_array().unwrap() {
        let raw = format!(
            "{{\"operation\":\"update_item\",\"itemId\":\"item\",\"masks\":[{}]}}",
            case["json"].as_str().unwrap()
        );
        assert!(
            serde_json::from_str::<EditOperation>(&raw).is_err(),
            "{}",
            case["name"]
        );
    }
}

#[test]
fn aliases_order_omission_clear_and_history_are_exact_after_reopen() {
    use opencut_editor_core::BatchEditOperation;
    let (_root, core, id, track) = setup();
    let mut add = rectangle(&track);
    add["resultAlias"] = json!("leaf");
    let original = json!([mask("first"), mask("second")]);
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([add,
        {"operation":"update_item","itemId":"@leaf","masks":original}]))
    .unwrap();
    let result = core.edit_batch(&id, 0, batch).unwrap();
    assert_eq!(result.revision, 1);
    let item = &result.aliases["leaf"];
    assert_eq!(item_masks(&core, &id, item), original);
    core.edit(
        &id,
        1,
        op(json!({"operation":"update_item","itemId":item,"hidden":true})),
    )
    .unwrap();
    assert_eq!(item_masks(&core, &id, item), original);
    let reversed = json!([mask("second"), mask("first")]);
    core.edit(&id, 2, update(item, reversed.clone())).unwrap();
    core.undo(&id, 3).unwrap();
    assert_eq!(item_masks(&core, &id, item), original);
    core.redo(&id, 4).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(item_masks(&reopened, &id, item), reversed);
    reopened.edit(&id, 5, update(item, json!([]))).unwrap();
    assert_eq!(item_masks(&reopened, &id, item), json!([]));
    reopened.undo(&id, 6).unwrap();
    assert_eq!(item_masks(&reopened, &id, item), reversed);
}

#[test]
fn invalid_later_batch_and_draft_edits_preserve_every_authoritative_and_resource_byte() {
    use opencut_editor_core::{BatchEditOperation, ErrorCode};
    let (_root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let valid = update(&item, json!([mask("valid")]));
    let draft = core
        .create_draft(&id, 1, vec![valid.clone()], None)
        .unwrap();
    let before = inventory(&core, &id);
    let invalid = update(&item, json!([mask("duplicate"), mask("duplicate")]));
    let batch: Vec<BatchEditOperation> = vec![valid.into(), invalid.clone().into()];
    assert_eq!(
        core.edit_batch(&id, 1, batch).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.update_draft(&id, &draft.id, 1, vec![invalid], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.commit_draft(&id, &draft.id, 0).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(&core, &id), before);
    assert_eq!(
        serde_json::to_value(
            core.get_draft_state(&id, &draft.id)
                .unwrap()
                .project
                .find_item(&item)
                .unwrap()
        )
        .unwrap()["masks"],
        json!([mask("valid")])
    );
    core.commit_draft(&id, &draft.id, 1).unwrap();
    assert_eq!(item_masks(&core, &id, &item), json!([mask("valid")]));
}

// Strip only the newly introduced field to produce a genuine immediate predecessor
// envelope. All other schema31 data remains exactly as authored by the public API.
fn source31(project: &mut Value) {
    project["schemaVersion"] = json!(31);
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("masks");
                for child in map.values_mut() {
                    strip(child);
                }
            }
            Value::Array(values) => {
                for child in values {
                    strip(child);
                }
            }
            _ => {}
        }
    }
    strip(project);
}

fn downgrade_current_and_history(core: &EditorCore, id: &str) {
    let dir = core.paths().project_dir(id).unwrap();
    for name in ["project.json", "history.json"] {
        let path = dir.join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if name == "project.json" {
            source31(&mut value);
        } else {
            for side in ["undo", "redo"] {
                for snapshot in value[side].as_array_mut().unwrap() {
                    source31(snapshot);
                }
            }
        }
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
}

#[test]
fn predecessor_migration_adopts_current_undo_redo_and_applicable_draft_without_rewriting_reopen() {
    let (_root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    core.edit(
        &id,
        1,
        op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":500})),
    )
    .unwrap();
    core.undo(&id, 2).unwrap();
    let draft = core
        .create_draft(
            &id,
            3,
            vec![op(
                json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":400}),
            )],
            None,
        )
        .unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    downgrade_current_and_history(&core, &id);
    let reopened = EditorCore::new(core.paths().clone());
    let current = serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap();
    assert_eq!(current, before);
    assert_eq!(
        current["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(item_masks(&reopened, &id, &item), json!([]));
    assert_eq!(
        reopened
            .get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .find_item(&item)
            .unwrap()
            .duration_ms(),
        400
    );
    let dir = core.paths().project_dir(&id).unwrap();
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for side in ["undo", "redo"] {
        for snapshot in history[side].as_array().unwrap() {
            assert_eq!(
                snapshot["schemaVersion"],
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
        }
    }
    let adopted = inventory(&reopened, &id);
    reopened.get_project(&id).unwrap();
    assert_eq!(inventory(&reopened, &id), adopted);
    reopened.redo(&id, 3).unwrap();
    assert_eq!(
        reopened
            .get_project(&id)
            .unwrap()
            .find_item(&item)
            .unwrap()
            .duration_ms(),
        500
    );
}

#[test]
fn premature_mask_field_in_current_or_retained_generation_rejects_without_partial_adoption() {
    for generation in ["current", "undo", "redo"] {
        for injection in [json!([]), Value::Null, json!([mask("premature")])] {
            let (_root, core, id, track) = setup();
            let item = core
                .edit(&id, 0, op(rectangle(&track)))
                .unwrap()
                .changed_ids[0]
                .clone();
            core.edit(
                &id,
                1,
                op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":500})),
            )
            .unwrap();
            core.edit(
                &id,
                2,
                op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":400})),
            )
            .unwrap();
            core.undo(&id, 3).unwrap();
            downgrade_current_and_history(&core, &id);
            let dir = core.paths().project_dir(&id).unwrap();
            let path = dir.join(if generation == "current" {
                "project.json"
            } else {
                "history.json"
            });
            let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let target = if generation == "current" {
                &mut value
            } else {
                value[generation]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|p| !p["tracks"][1]["items"].as_array().unwrap().is_empty())
                    .unwrap()
            };
            target["tracks"][1]["items"][0]["masks"] = injection;
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            let before = inventory(&core, &id);
            assert!(
                EditorCore::new(core.paths().clone())
                    .get_project(&id)
                    .is_err(),
                "{generation}"
            );
            assert_eq!(inventory(&core, &id), before);
        }
    }
}

#[test]
fn inclusive_hidden_and_unused_composition_and_project_budgets_are_owned_by_core() {
    for commands in [false, true] {
        for project_scope in [false, true] {
            let (_root, core, id, track) = setup();
            core.edit(&id, 0, op(rectangle(&track))).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let path = dir.join("project.json");
            let mut project: Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let prototype = project["tracks"][1]["items"][0].clone();
            let make_item = |index: usize, full: bool| {
                let mut item = prototype.clone();
                item["id"] = json!(format!("leaf{index}"));
                item["stackOrder"] = json!(index);
                item["hidden"] = json!(true);
                let count = if full { 16 } else { 1 };
                item["masks"] = json!(
                    (0..count)
                        .map(|i| {
                            let mut value = mask(&format!("m{i}"));
                            let n = if commands && full { 4096 } else { 1 };
                            value["source"]["path"]["commands"] =
                                json!(vec![json!({"type":"moveTo","to":{"x":0,"y":0}}); n]);
                            value
                        })
                        .collect::<Vec<_>>()
                );
                item
            };
            let full_items = if commands { 1 } else { 256 };
            let items = (0..full_items)
                .map(|i| make_item(i, true))
                .collect::<Vec<_>>();
            let local_track = |items: Vec<Value>| json!({"id":"local","name":"Hidden","trackType":"overlay","locked":false,"hidden":true,"muted":false,"audioRole":"unassigned","ducking":null,"items":items});
            if project_scope {
                project["tracks"][1]["items"] = json!([]);
                project["components"] = json!((0..4).map(|i| json!({"id":format!("unused{i}"),"name":"Unused","width":64,"height":64,"durationMs":600,"tracks":[local_track(items.clone())],"slots":[],"markers":[]})).collect::<Vec<_>>());
            } else {
                project["tracks"][1]["items"] = json!(items);
                project["tracks"][1]["hidden"] = json!(true);
            }
            std::fs::write(&path, serde_json::to_vec(&project).unwrap()).unwrap();
            assert!(
                {
                    let result = core.get_project(&id);
                    assert!(
                        result.is_ok(),
                        "inclusive commands={commands}, project={project_scope}: {result:?}"
                    );
                    true
                },
                "inclusive commands={commands}, project={project_scope}"
            );
            if project_scope {
                project["components"].as_array_mut().unwrap().push(json!({"id":"unused-extra","name":"Unused extra","width":64,"height":64,"durationMs":600,"tracks":[local_track(vec![make_item(0,false)])],"slots":[],"markers":[]}));
            } else {
                project["tracks"][1]["items"]
                    .as_array_mut()
                    .unwrap()
                    .push(make_item(full_items, false));
            }
            std::fs::write(&path, serde_json::to_vec(&project).unwrap()).unwrap();
            let before = inventory(&core, &id);
            assert!(
                core.get_project(&id).is_err(),
                "overflow commands={commands}, project={project_scope}"
            );
            assert_eq!(inventory(&core, &id), before);
        }
    }
}

#[test]
fn same_mask_ids_on_independent_items_and_per_item_limits_are_scoped() {
    let (_root, core, id, track) = setup();
    let a = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let b = core
        .edit(&id, 1, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let sixteen = json!((0..16).map(|i| mask(&format!("m{i}"))).collect::<Vec<_>>());
    core.edit(&id, 2, update(&a, sixteen.clone())).unwrap();
    core.edit(&id, 3, update(&b, sixteen.clone())).unwrap();
    assert_eq!(item_masks(&core, &id, &a), item_masks(&core, &id, &b));
    let mut overflow = sixteen;
    overflow.as_array_mut().unwrap().push(mask("extra"));
    let before = inventory(&core, &id);
    assert!(core.edit(&id, 4, update(&b, overflow)).is_err());
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn valid_unavailable_base_draft_is_preserved_but_cannot_be_previewed_or_committed() {
    use opencut_editor_core::ErrorCode;
    let (_root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let draft = core
        .create_draft(&id, 1, vec![update(&item, json!([mask("stale")]))], None)
        .unwrap();
    core.edit(
        &id,
        1,
        op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":500})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let history_path = dir.join("history.json");
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(&history_path).unwrap()).unwrap();
    history["undo"] = json!([]);
    history["redo"] = json!([]);
    std::fs::write(history_path, serde_json::to_vec(&history).unwrap()).unwrap();
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let bytes = std::fs::read(&draft_path).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(reopened.get_project(&id).unwrap().revision, 2);
    assert_eq!(
        reopened.get_draft_state(&id, &draft.id).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        reopened.commit_draft(&id, &draft.id, 2).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(std::fs::read(&draft_path).unwrap(), bytes);
    reopened
        .edit(&id, 2, update(&item, json!([mask("current")])))
        .unwrap();
    reopened.discard_draft(&id, &draft.id).unwrap();
    assert!(!draft_path.exists());
}

fn write_tone(path: &std::path::Path) {
    let samples = 28_800_u32;
    let data_len = samples * 2;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + data_len).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(48_000_u32.to_le_bytes());
    bytes.extend(96_000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(data_len.to_le_bytes());
    for i in 0..samples {
        let sample =
            (8_000.0 * (f64::from(i) * 440.0 * std::f64::consts::TAU / 48_000.0).sin()) as i16;
        bytes.extend(sample.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn predecessor_migration_preserves_managed_resource_provenance_and_revision() {
    use opencut_editor_core::{
        CommitGeneratedAssetRequest, GeneratedAssetOrigin, MediaProbeFacts, SpeechGeneration,
        SpeechSynthesisRequest, SpeechTextOptions, SpeechVoiceId,
    };
    let (root, core, id, track) = setup();
    let core = EditorCore::new(
        core.paths()
            .clone()
            .with_generated_media_root(root.path().join("media"))
            .unwrap(),
    );
    let wav = root.path().join("media/tone.wav");
    write_tone(&wav);
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    let origin = GeneratedAssetOrigin::SpeechSynthesis(SpeechGeneration {
        alignment: None,
        request: SpeechSynthesisRequest {
            text: "Retained provenance".into(),
            language: "en-US".into(),
            voice_id: SpeechVoiceId("af_heart".into()),
            speed: 1.0,
            text_options: SpeechTextOptions::default(),
        },
        provider_id: "fixture-provider".into(),
        model_id: "fixture-model".into(),
        model_version: Some("1".into()),
        sample_rate_hz: 48_000,
        generated_at_ms: 1_777_000_000_000,
    });
    core.commit_generated_asset(CommitGeneratedAssetRequest {
        project_id: id.clone(),
        expected_revision: 0,
        path: wav,
        track_id: audio_track,
        start_ms: 0,
        duration_ms: 600,
        display_name: "Retained provenance.wav".into(),
        origin: origin.clone(),
        probe: MediaProbeFacts {
            duration_ms: Some(600),
            has_audio: true,
            audio_sample_rate_hz: Some(48_000),
            ..Default::default()
        },
    })
    .unwrap();
    core.edit(&id, 1, op(rectangle(&track))).unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(before["assets"].as_array().unwrap().len(), 1);
    assert_eq!(
        before["assets"][0]["origin"],
        serde_json::to_value(origin).unwrap()
    );
    assert!(
        before["assets"][0]["contentHash"]["digest"]
            .as_str()
            .is_some_and(|s| s.len() == 64)
    );
    downgrade_current_and_history(&core, &id);
    let legacy_bytes = inventory(&core, &id);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        before
    );
    let adopted = inventory(&reopened, &id);
    for (path, bytes) in legacy_bytes {
        if path.file_name().unwrap() != "project.json"
            && path.file_name().unwrap() != "history.json"
        {
            assert_eq!(
                adopted[&path],
                bytes,
                "resource changed: {}",
                path.display()
            );
        }
    }
    reopened.get_project(&id).unwrap();
    assert_eq!(inventory(&reopened, &id), adopted);
}

#[test]
fn native_activated_masks_preserve_strict_frame_range_draft_export_witnesses() {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        return;
    };
    let font = std::env::var_os("OPENCUT_TEST_FONT_PATH").map(std::path::PathBuf::from);
    if std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref() == Ok("1") {
        assert!(
            font.as_ref().is_some_and(|p| p.is_file()),
            "mandatory native run requires explicit bundled font"
        );
    }
    let (root, core, id, track) = setup();
    let item = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let wav = root.path().join("media/tone.wav");
    write_tone(&wav);
    let asset = core
        .import_asset(
            &id,
            1,
            &wav,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(600),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let audio_track = core.get_project(&id).unwrap().tracks[2].id.clone();
    core.edit(&id,2,op(json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":600}))).unwrap();
    let baseline = core.get_project(&id).unwrap();
    let mut destructive = mask("black-control");
    destructive["source"]["paint"]["color"] = json!({"r":0,"g":0,"b":0,"a":0});
    destructive["operation"] = json!("intersect");
    destructive["inverted"] = json!(false);
    destructive["featherPx"] = json!(0);
    destructive["expansionPx"] = json!(0);
    let black_draft = core
        .create_draft(&id, 3, vec![update(&item, json!([destructive]))], None)
        .unwrap();
    let black_project = core.get_draft_state(&id, &black_draft.id).unwrap().project;
    let mut reveal = mask("reveal");
    reveal["source"]["paint"]["color"] = json!({"r":1,"g":1,"b":1,"a":0.5});
    reveal["operation"] = json!("add");
    reveal["inverted"] = json!(false);
    reveal["featherPx"] = json!(0);
    reveal["expansionPx"] = json!(0);
    reveal["transform"] =
        serde_json::to_value(opencut_editor_core::Transform2D::default()).unwrap();
    let operation = update(&item, json!([reveal]));
    let draft = core
        .create_draft(
            &id,
            3,
            vec![operation.clone()],
            Some("Mask metadata".into()),
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit(&id, 3, operation).unwrap();
    let committed = core.get_project(&id).unwrap();
    assert_ne!(baseline.revision, committed.revision);
    let dir = core.paths().project_dir(&id).unwrap();
    let authored_before = inventory(&core, &id);
    let renderer = Renderer::new(&ffmpeg, &ffprobe, font);
    let decode = |path: &std::path::Path, audio: bool| {
        let mut command = std::process::Command::new(&ffmpeg);
        command.args(["-v", "error", "-i"]).arg(path);
        if audio {
            command.args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "pipe:1",
            ]);
        } else {
            command.args([
                "-map", "0:v:0", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1",
            ]);
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    };
    let probe = |path: &std::path::Path| {
        let result=std::process::Command::new(&ffprobe).args(["-v","error","-show_entries","format=duration:stream=codec_type,time_base,duration,nb_frames,width,height,sample_rate,channels","-of","json"]).arg(path).output().unwrap();
        assert!(result.status.success());
        serde_json::from_slice::<Value>(&result.stdout).unwrap()
    };
    let black = renderer.render_preview(&black_project, &dir, 300).unwrap();
    let black_pixels = decode(&dir.join(black.relative_path), false);
    assert_eq!(black_pixels.len(), 64 * 64 * 3);
    assert!(
        black_pixels.iter().all(|v| *v == 0),
        "zero-alpha mask must suppress every RGB channel"
    );
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    let mut frame_oracle = None;
    let mut range_oracle: Option<(Vec<u8>, Vec<u8>, Value)> = None;
    let mut export_oracle: Option<(Vec<u8>, Vec<u8>, Value)> = None;
    for (index, project) in [&baseline, &candidate, &committed, &reopened]
        .into_iter()
        .enumerate()
    {
        let frame = renderer.render_preview(project, &dir, 300).unwrap();
        let pixels = decode(&dir.join(frame.relative_path), false);
        assert_eq!(pixels.len(), 64 * 64 * 3);
        assert!(
            pixels
                .as_chunks::<3>()
                .0
                .iter()
                .any(|p| p[0] > 100 && p[1] < 30 && p[2] < 30),
            "independent visible-red oracle failed"
        );
        if let Some(expected) = &frame_oracle {
            if index == 1 {
                assert_ne!(&pixels, expected, "mask must change final appearance");
                frame_oracle = Some(pixels);
            } else {
                assert_eq!(
                    &pixels, expected,
                    "same-mask frame/draft/commit/reopen identity"
                );
            }
        } else {
            frame_oracle = Some(pixels);
        }
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 100,
                    end_ms: 500,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let range_path = dir.join(range.relative_path);
        let range_pixels = decode(&range_path, false);
        assert_eq!(range_pixels.len(), 4 * 64 * 64 * 3);
        let range_audio = decode(&range_path, true);
        assert!(!range_audio.is_empty());
        assert!(
            range_audio
                .as_chunks::<4>()
                .0
                .iter()
                .any(|b| f32::from_le_bytes(*b).abs() > 0.01)
        );
        let range_result = (range_pixels, range_audio, probe(&range_path));
        if let Some(expected) = &range_oracle {
            if index == 1 {
                assert_ne!(range_result.0, expected.0, "mask must change range pixels");
                assert_eq!(range_result.1, expected.1, "mask must preserve range audio");
                assert_eq!(
                    range_result.2, expected.2,
                    "mask must preserve range timing"
                );
                range_oracle = Some(range_result);
            } else {
                assert_eq!(
                    &range_result, expected,
                    "same-mask range/draft/commit/reopen identity"
                );
            }
        } else {
            range_oracle = Some(range_result);
        }
        let export_path = root.path().join(format!("exports/mask-{index}.mp4"));
        std::fs::create_dir_all(export_path.parent().unwrap()).unwrap();
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &export_path,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let export_pixels = decode(&export_path, false);
        assert_eq!(export_pixels.len(), 6 * 64 * 64 * 3);
        let export_audio = decode(&export_path, true);
        assert!(!export_audio.is_empty());
        let export_result = (export_pixels, export_audio, probe(&export_path));
        if let Some(expected) = &export_oracle {
            if index == 1 {
                assert_ne!(
                    export_result.0, expected.0,
                    "mask must change export pixels"
                );
                assert_eq!(
                    export_result.1, expected.1,
                    "mask must preserve export audio"
                );
                assert_eq!(
                    export_result.2, expected.2,
                    "mask must preserve export timing"
                );
                export_oracle = Some(export_result);
            } else {
                assert_eq!(
                    &export_result, expected,
                    "same-mask export/draft/commit/reopen identity"
                );
            }
        } else {
            export_oracle = Some(export_result);
        }
    }
    // Artifacts/cache entries may be created. Every pre-existing authored/resource
    // file must retain its bytes; revision-scoped render cache identity may differ.
    let authored_after = inventory(&core, &id);
    for (path, bytes) in authored_before {
        assert_eq!(
            authored_after[&path],
            bytes,
            "render mutated {}",
            path.display()
        );
    }
}

#[test]
fn local_component_masks_roundtrip_update_history_and_reject_premature_component_sources() {
    let (_root, core, id, track) = setup();
    let leaf = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut local =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&leaf).unwrap()).unwrap();
    local["id"] = json!("local-leaf");
    local["masks"] = json!([mask("scoped")]);
    let tracks = json!([{"id":"local","name":"Local","trackType":"overlay","locked":false,"hidden":true,"muted":false,"audioRole":"unassigned","ducking":null,"items":[local]}]);
    let component=core.edit(&id,1,op(json!({"operation":"component_create","name":"Unused masked component","width":64,"height":64,"durationMs":600,"tracks":tracks}))).unwrap().changed_ids[0].clone();
    let masks = |core: &EditorCore| {
        serde_json::to_value(
            &core.get_project(&id).unwrap().components[0].tracks[0].items[0]
                .visual_properties()
                .masks,
        )
        .unwrap()
    };
    assert_eq!(masks(&core), json!([mask("scoped")]));
    let mut changed = tracks;
    changed[0]["items"][0]["masks"] = json!([mask("replacement"), mask("scoped")]);
    core.edit(&id,2,op(json!({"operation":"component_update","componentId":component,"name":"Changed","width":64,"height":64,"durationMs":600,"tracks":changed}))).unwrap();
    core.undo(&id, 3).unwrap();
    assert_eq!(masks(&core), json!([mask("scoped")]));
    core.redo(&id, 4).unwrap();
    assert_eq!(
        masks(&EditorCore::new(core.paths().clone())),
        json!([mask("replacement"), mask("scoped")])
    );
    // A component is not exempt from the source-envelope guard, including [].
    downgrade_current_and_history(&core, &id);
    let path = core.paths().project_dir(&id).unwrap().join("project.json");
    let mut source: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    source["components"][0]["tracks"][0]["items"][0]["masks"] = json!([]);
    std::fs::write(path, serde_json::to_vec(&source).unwrap()).unwrap();
    let before = inventory(&core, &id);
    assert!(core.get_project(&id).is_err());
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn stale_missing_and_unsupported_targets_keep_their_public_error_and_rollback_contracts() {
    use opencut_editor_core::ErrorCode;
    let (_root, core, id, track) = setup();
    let group = core
        .edit(
            &id,
            0,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":600})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let before = inventory(&core, &id);
    assert_eq!(
        core.edit(&id, 0, update(&group, json!([mask("x")])))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(&id, 1, update("missing", json!([mask("x")])))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        core.edit(&id, 1, update(&group, json!([mask("x")])))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&core, &id), before);
    assert!(
        serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":group,"masks":null})
        )
        .is_err()
    );
    core.edit(&id, 1, update(&group, json!([]))).unwrap();
}

#[test]
fn predecessor_drafts_are_validated_against_the_retained_base_before_any_migration_write() {
    use opencut_editor_core::ErrorCode;
    for invalid in [false, true] {
        let (_root, core, id, track) = setup();
        let leaf = core
            .edit(&id, 0, op(rectangle(&track)))
            .unwrap()
            .changed_ids[0]
            .clone();
        let draft = core
            .create_draft(
                &id,
                1,
                vec![op(
                    json!({"operation":"trim_item","itemId":leaf,"startMs":0,"durationMs":400}),
                )],
                None,
            )
            .unwrap();
        core.edit(&id, 1, op(json!({"operation":"delete_item","itemId":leaf})))
            .unwrap();
        downgrade_current_and_history(&core, &id);
        let draft_path = core
            .paths()
            .project_dir(&id)
            .unwrap()
            .join("drafts")
            .join(format!("{}.json", draft.id));
        if invalid {
            let mut value: Value =
                serde_json::from_slice(&std::fs::read(&draft_path).unwrap()).unwrap();
            value["operations"][0]["itemId"] = json!("missing");
            std::fs::write(&draft_path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before = inventory(&core, &id);
        let reopened = EditorCore::new(core.paths().clone());
        if invalid {
            assert_eq!(
                reopened.get_project(&id).unwrap_err().code,
                ErrorCode::ItemNotFound
            );
            assert_eq!(inventory(&core, &id), before);
        } else {
            assert_eq!(
                reopened.get_project(&id).unwrap().schema_version,
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
            assert_eq!(
                reopened.get_draft_state(&id, &draft.id).unwrap_err().code,
                ErrorCode::RevisionConflict
            );
            assert_eq!(std::fs::read(&draft_path).unwrap(), before[&draft_path]);
        }
    }
}

#[test]
fn predecessor_mask_bearing_draft_fields_reject_empty_null_and_nonempty_without_publication() {
    for masks in [json!([]), Value::Null, json!([mask("premature")])] {
        let (_root, core, id, track) = setup();
        let leaf = core
            .edit(&id, 0, op(rectangle(&track)))
            .unwrap()
            .changed_ids[0]
            .clone();
        let draft = core
            .create_draft(
                &id,
                1,
                vec![op(
                    json!({"operation":"trim_item","itemId":leaf,"startMs":0,"durationMs":400}),
                )],
                None,
            )
            .unwrap();
        downgrade_current_and_history(&core, &id);
        let path = core
            .paths()
            .project_dir(&id)
            .unwrap()
            .join("drafts")
            .join(format!("{}.json", draft.id));
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["operations"] = json!([{"operation":"update_item","itemId":leaf,"masks":masks}]);
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        let before = inventory(&core, &id);
        assert!(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .is_err()
        );
        assert_eq!(inventory(&core, &id), before);
    }
}

#[test]
fn utf8_id_path_command_and_gradient_stop_limits_are_inclusive_and_independent() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    let (_root, core, id, track) = setup();
    let leaf = core
        .edit(&id, 0, op(rectangle(&track)))
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut boundary = numeric_values(catalog["cases"][4]["value"].clone());
    boundary["id"] = json!("é".repeat(64));
    boundary["source"]["path"]["commands"] =
        json!(vec![json!({"type":"moveTo","to":{"x":0.0,"y":0.0}}); 4096]);
    boundary["source"]["paint"]["stops"] = json!(
        (0..64)
            .map(|i| json!({"offset":if i == 63 {1.0} else {f64::from(i)/64.0},"color":{"r":0.2,"g":0.4,"b":0.8,"a":0.25}}))
            .collect::<Vec<_>>()
    );
    core.edit(&id, 1, update(&leaf, json!([boundary.clone()])))
        .unwrap();
    assert_eq!(item_masks(&core, &id, &leaf), json!([boundary.clone()]));
    for property in ["id", "commands", "stops"] {
        let mut overflow = boundary.clone();
        match property {
            "id" => overflow["id"] = json!("é".repeat(65)),
            "commands" => overflow["source"]["path"]["commands"]
                .as_array_mut()
                .unwrap()
                .push(json!({"type":"moveTo","to":{"x":0,"y":0}})),
            "stops" => overflow["source"]["paint"]["stops"] = json!((0..65).map(|i|json!({"offset":f64::from(i)/64.0,"color":{"r":0.2,"g":0.4,"b":0.8,"a":0.25}})).collect::<Vec<_>>()),
            _ => unreachable!(),
        }
        let before = inventory(&core, &id);
        assert!(
            core.edit(&id, 2, update(&leaf, json!([overflow]))).is_err(),
            "{property}"
        );
        assert_eq!(inventory(&core, &id), before, "{property}");
    }
}
