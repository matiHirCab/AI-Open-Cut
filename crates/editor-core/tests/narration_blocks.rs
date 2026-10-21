use opencut_editor_core::*;
use serde_json::{Value, json};

#[path = "support/narration_blocks.rs"]
mod support;
use support::*;

#[test]
fn consumes_independent_unicode_boundary_and_abbreviation_cases() {
    let (_root, core, profile) = setup();
    for case in fixture()["planningCases"].as_array().unwrap() {
        let boundaries = case
            .get("boundaries")
            .map(|v| serde_json::from_value(v.clone()).unwrap());
        let plan = core
            .plan_narration_blocks(input(&profile, case["text"].as_str().unwrap(), boundaries))
            .unwrap();
        assert_eq!(
            serde_json::to_value(
                plan.blocks
                    .iter()
                    .map(|b| &b.display_text)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            case["displays"],
            "{}",
            case["name"]
        );
        assert_eq!(
            serde_json::to_value(
                plan.blocks
                    .iter()
                    .map(|b| &b.request.text)
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            case["requests"]
        );
        assert_eq!(
            serde_json::to_value(plan.blocks.iter().map(|b| b.paragraph).collect::<Vec<_>>())
                .unwrap(),
            case["paragraphs"]
        );
        if let Some(ends) = case.get("ends") {
            assert_eq!(
                serde_json::to_value(plan.blocks.iter().map(|b| b.end_scalar).collect::<Vec<_>>())
                    .unwrap(),
                *ends
            );
        }
        assert!(
            plan.blocks
                .iter()
                .all(|b| b.request.narration_profile.as_ref().unwrap().version == 1)
        );
    }
}

#[test]
fn rejects_blank_invalid_boundaries_and_limits_before_project_mutation() {
    let (root, core, profile) = setup();
    let before = inventory(root.path());
    for boundaries in fixture()["invalidBoundaries"].as_array().unwrap() {
        assert_eq!(
            core.plan_narration_blocks(input(
                &profile,
                "A😀. B.",
                Some(serde_json::from_value(boundaries.clone()).unwrap())
            ))
            .unwrap_err()
            .code,
            ErrorCode::ValidationFailed
        );
    }
    for source in ["", "   ", &"x".repeat(4097)] {
        assert!(
            core.plan_narration_blocks(input(&profile, source, None))
                .is_err()
        );
    }
    let mut budget = input(&profile, "A.", None);
    budget.max_text_characters = 0;
    assert!(core.plan_narration_blocks(budget).is_err());
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn publishes_all_blocks_once_and_preserves_exact_history_without_library() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let ids = input
        .plan
        .blocks
        .iter()
        .map(|b| b.id.clone())
        .collect::<Vec<_>>();
    let result = core.commit_narration_blocks(input.clone()).unwrap();
    assert_eq!(result.write.revision, 1);
    assert_eq!(
        result
            .manifest
            .blocks
            .iter()
            .map(|b| b.binding.start_ms)
            .collect::<Vec<_>>(),
        [100, 500]
    );
    assert_eq!(
        result
            .manifest
            .blocks
            .iter()
            .map(|b| b.id.clone())
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        result.manifest.blocks[0]
            .request
            .narration_profile
            .as_ref()
            .unwrap()
            .version,
        1
    );
    let stale = core.commit_narration_blocks(input).unwrap_err();
    assert_eq!(stale.code, ErrorCode::RevisionConflict);
    std::fs::rename(
        root.path().join("projects/narration-profiles.json"),
        root.path().join("profile-preserved.json"),
    )
    .unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        reopened
            .get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests[0],
        result.manifest
    );
    reopened.undo(&result.write.project_id, 1).unwrap();
    assert!(
        reopened
            .get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests
            .is_empty()
    );
    reopened.redo(&result.write.project_id, 2).unwrap();
    assert_eq!(
        reopened
            .get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests[0],
        result.manifest
    );
}

#[test]
fn invalid_last_block_and_spoken_metadata_leave_all_bytes_unchanged() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let before = inventory(root.path());
    let mut invalid = input.clone();
    invalid.blocks[1].prepared.block.request.text = "unapproved".into();
    assert!(core.commit_narration_blocks(invalid).is_err());
    assert_eq!(inventory(root.path()), before);
    let mut invalid = input;
    invalid.blocks[1].prepared.spoken_text = "different declared segments".into();
    assert!(core.commit_narration_blocks(invalid).is_err());
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn detach_preserves_unrelated_blocks_and_allows_legacy_edit_only_afterward() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let result = core.commit_narration_blocks(input).unwrap();
    let first = &result.manifest.blocks[0];
    let second = result.manifest.blocks[1].clone();
    let before = inventory(root.path());
    let delete: EditOperation =
        serde_json::from_value(json!({"operation":"delete_item","itemId":first.binding.item_id}))
            .unwrap();
    assert_eq!(
        core.edit(&result.write.project_id, 1, delete.clone())
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    core.detach_narration_block(&result.write.project_id, 1, &result.manifest.id, &first.id)
        .unwrap();
    assert_eq!(
        core.get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests[0]
            .blocks,
        std::slice::from_ref(&second)
    );
    core.edit(&result.write.project_id, 2, delete).unwrap();
    assert_eq!(
        core.get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests[0]
            .blocks,
        [second]
    );
}

#[test]
fn closed_input_shapes_reject_null_and_unknown_fields() {
    let (_root, _core, profile) = setup();
    let source = serde_json::to_value(input(&profile, "A.", None)).unwrap();
    // Omitted boundaries is the absence form, explicit null is rejected.
    let mut valid = source.clone();
    valid.as_object_mut().unwrap().remove("boundaries");
    assert!(serde_json::from_value::<NarrationPlanInput>(valid.clone()).is_ok());
    valid["boundaries"] = Value::Null;
    assert!(serde_json::from_value::<NarrationPlanInput>(valid).is_err());
    let mut extra = source;
    extra["extra"] = json!(true);
    assert!(serde_json::from_value::<NarrationPlanInput>(extra).is_err());
}

#[test]
fn dedicated_legacy_regeneration_rejects_owned_audio_before_copy_and_works_after_detach() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let replacement = input.blocks[0].clone();
    let result = core.commit_narration_blocks(input).unwrap();
    let owned = &result.manifest.blocks[0];
    let other = result.manifest.blocks[1].clone();
    let request = ReplaceGeneratedAssetRequest {
        project_id: result.write.project_id.clone(),
        expected_revision: 1,
        path: replacement.path,
        item_id: owned.binding.item_id.clone(),
        duration_ms: replacement.duration_ms,
        origin: replacement.origin,
        probe: replacement.probe,
    };
    let before = inventory(root.path());
    assert_eq!(
        core.replace_generated_asset(request.clone())
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    core.detach_narration_block(&result.write.project_id, 1, &result.manifest.id, &owned.id)
        .unwrap();
    let mut request = request;
    request.expected_revision = 2;
    core.replace_generated_asset(request).unwrap();
    assert_eq!(
        core.get_project(&result.write.project_id)
            .unwrap()
            .narration_manifests[0]
            .blocks,
        [other]
    );
}

#[test]
fn schema46_current_history_adopt_empty_manifests_and_invalid_generations_never_publish() {
    let (root, core, _profile) = setup();
    let id = core
        .create_project("legacy46", ProjectSettings::default())
        .unwrap()
        .project_id;
    let dir = core.paths().projects_root().join(&id);
    let mut original = core.get_project(&id).unwrap();
    original.schema_version = 46;
    let legacy = serde_json::to_value(&original).unwrap();
    assert!(legacy.get("narrationManifests").is_none());
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[legacy.clone()],"redo":[legacy.clone()]})).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 47);
    assert!(migrated.narration_manifests.is_empty());
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for side in ["undo", "redo"] {
        assert_eq!(history[side][0]["schemaVersion"], 47);
        assert_eq!(history[side][0]["narrationManifests"], json!([]));
    }
    let current = serde_json::to_value(migrated).unwrap();
    for invalid in [
        {
            let mut v = legacy.clone();
            v["narrationManifests"] = json!([]);
            v
        },
        {
            let mut v = legacy.clone();
            v["narrationManifests"] = Value::Null;
            v
        },
        {
            let mut v = current.clone();
            v.as_object_mut().unwrap().remove("narrationManifests");
            v
        },
        {
            let mut v = current.clone();
            v["narrationManifests"] = Value::Null;
            v
        },
        {
            let mut v = current.clone();
            v["schemaVersion"] = json!(48);
            v
        },
    ] {
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        let before = inventory(root.path());
        assert!(core.get_project(&id).is_err());
        assert_eq!(inventory(root.path()), before);
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    let mut invalid_history = json!({"undo":[legacy.clone()],"redo":[legacy]});
    invalid_history["redo"][0]["narrationManifests"] = json!([]);
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&invalid_history).unwrap(),
    )
    .unwrap();
    let before = inventory(root.path());
    assert!(core.get_project(&id).is_err());
    assert_eq!(inventory(root.path()), before);
}

fn align_all(input: &mut CommitNarrationBlocksInput, model: Option<&str>) {
    for block in &mut input.blocks {
        let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut block.origin;
        generation.alignment = Some(SpeechAlignment {
            sentences: vec![SpeechTimedText {
                text: generation.request.text.clone(),
                start_ms: 0,
                end_ms: 200,
            }],
            words: vec![],
            phonemes: vec![],
            quality: SpeechAlignmentQuality::Forced,
            provider_id: "independent-alignment-producer".into(),
            model_id: model.map(str::to_owned),
            model_version: None,
        });
    }
}

#[test]
fn optional_captions_and_prefixed_cues_use_only_actual_independent_alignment() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    let project = core.get_project(&input.project_id).unwrap();
    let caption_track = project
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Caption)
        .unwrap()
        .id
        .clone();
    input.caption = Some(NarrationCaptionOptions {
        track_id: caption_track,
        granularity: NarrationCaptionGranularity::Sentence,
    });
    input.marker_policy = SpeechMarkerPolicy::Sentence {};
    align_all(&mut input, Some("actual-alignment-model"));
    let result = core.commit_narration_blocks(input).unwrap();
    let state = core.get_project(&result.write.project_id).unwrap();
    for (index, block) in result.manifest.blocks.iter().enumerate() {
        assert_eq!(block.binding.caption_refs.len(), 1);
        assert_eq!(block.binding.cue_refs.len(), 1);
        assert!(block.binding.cue_refs[0].id.starts_with("m_"));
        assert_eq!(block.binding.cue_refs[0].time_ms, [100, 500][index]);
        let caption = state.find_item(&block.binding.caption_refs[0].id).unwrap();
        let TimelineItem::Caption(caption) = caption else {
            panic!("native caption missing")
        };
        assert_eq!(caption.source.provider_id, "independent-alignment-producer");
        assert_eq!(caption.source.model_id, "actual-alignment-model");
        assert_ne!(caption.source.provider_id, "synthesis-fixture");
        assert_eq!(caption.source.model_version, None);
    }
    let mut corrupt = serde_json::to_value(&state).unwrap();
    corrupt["narrationManifests"][0]["markerPolicy"] = json!({"type":"none"});
    let dir = core.paths().projects_root().join(&state.id);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&corrupt).unwrap(),
    )
    .unwrap();
    let before = inventory(root.path());
    assert!(core.get_project(&state.id).is_err());
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn caption_null_model_rejects_before_import_but_cues_preserve_nullable_actual_provenance() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    let project = core.get_project(&input.project_id).unwrap();
    let caption = project
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Caption)
        .unwrap()
        .id
        .clone();
    align_all(&mut input, None);
    input.marker_policy = SpeechMarkerPolicy::Sentence {};
    input.caption = Some(NarrationCaptionOptions {
        track_id: caption,
        granularity: NarrationCaptionGranularity::Sentence,
    });
    let before = inventory(root.path());
    assert_eq!(
        core.commit_narration_blocks(input.clone())
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    input.caption = None;
    let result = core.commit_narration_blocks(input).unwrap();
    assert_eq!(result.manifest.blocks[0].binding.cue_refs.len(), 1);
    let state = core.get_project(&result.write.project_id).unwrap();
    let GeneratedAssetOrigin::SpeechSynthesis(generation) =
        state.assets[0].origin.as_ref().unwrap();
    assert_eq!(generation.alignment.as_ref().unwrap().model_id, None);
}

#[test]
fn planning_byte_scalar_source_and_block_boundaries_are_explicit_and_nontruncating() {
    let (root, core, profile) = setup();
    let before = inventory(root.path());
    let exact = "😀".repeat(1024);
    let plan = core
        .plan_narration_blocks(input(&profile, &exact, Some(vec![1024])))
        .unwrap();
    assert_eq!(plan.blocks[0].display_text, exact);
    assert!(
        core.plan_narration_blocks(input(&profile, &"😀".repeat(1025), None))
            .is_err()
    );
    let mut short = input(&profile, "A😀.", None);
    short.max_text_characters = 3;
    assert!(core.plan_narration_blocks(short.clone()).is_ok());
    short.max_text_characters = 2;
    assert!(core.plan_narration_blocks(short).is_err());
    let source = ("x".repeat(4095) + " ").repeat(8);
    let plan = core
        .plan_narration_blocks(input(
            &profile,
            &source,
            Some((1..=8).map(|i| i * 4096).collect()),
        ))
        .unwrap();
    assert_eq!(plan.source_text.len(), 32768);
    assert!(
        core.plan_narration_blocks(input(&profile, &(source + "x"), None))
            .is_err()
    );
    for (count, valid) in [(32, true), (33, false)] {
        let source = "A. ".repeat(count);
        assert_eq!(
            core.plan_narration_blocks(input(
                &profile,
                &source,
                Some((1..=count).map(|i| i * 3).collect())
            ))
            .is_ok(),
            valid
        );
    }
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn declared_spoken_segment_limit_is_atomic_at32_and_rejects33_or_byte_expansion() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    input.blocks[0].prepared.spoken_segments = vec!["x".into(); 32];
    input.blocks[0].prepared.spoken_text = input.blocks[0].prepared.spoken_segments.join("\n");
    let before = inventory(root.path());
    for segments in [vec!["x".into(); 33], vec!["😀".repeat(1025)]] {
        let mut invalid = input.clone();
        invalid.blocks[0].prepared.spoken_text = segments.join("\n");
        invalid.blocks[0].prepared.spoken_segments = segments;
        assert_eq!(
            core.commit_narration_blocks(invalid).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    assert!(core.commit_narration_blocks(input).is_ok());
}

#[test]
fn request_and_whole_plan_budgets_refuse_large_pinned_profiles_before_publication() {
    let (root, core, profile) = setup();
    for (count, unicode, blocks) in [(100, true, 1), (100, false, 16)] {
        let mut settings = profile.settings.clone();
        settings.name = format!("Budget {unicode}");
        let c = if unicode { "😀" } else { "x" };
        settings.text_options.pronunciations = (0..count)
            .map(|index| SpeechPronunciation {
                term: format!("{index:03}{}", c.repeat(125)),
                spoken: c.repeat(256),
            })
            .collect();
        let large = core.create_narration_profile(settings).unwrap();
        let before = inventory(root.path());
        let source = "A. ".repeat(blocks);
        assert!(
            core.plan_narration_blocks(input(
                &large,
                &source,
                Some((1..=blocks).map(|i| i * 3).collect())
            ))
            .is_err()
        );
        assert_eq!(inventory(root.path()), before);
    }
}

#[test]
fn project_live_block_and_manifest_caps_reject_the_next_atomic_publication() {
    for (groups, blocks) in [(8, 32), (32, 1)] {
        let (root, core, profile) = setup();
        let project = core
            .create_project("Project narration caps", ProjectSettings::default())
            .unwrap()
            .project_id;
        let mut last = None;
        for revision in 0..groups {
            let state = core.get_project(&project).unwrap();
            let track = state
                .tracks
                .iter()
                .find(|track| track.track_type == TrackType::Audio)
                .unwrap();
            let source = "A. ".repeat(blocks);
            let plan = core
                .plan_narration_blocks(input(
                    &profile,
                    &source,
                    Some((1..=blocks).map(|i| i * 3).collect()),
                ))
                .unwrap();
            let input = CommitNarrationBlocksInput {
                project_id: project.clone(),
                expected_revision: revision as u64,
                track_id: track.id.clone(),
                start_ms: revision as u64 * 1000,
                blocks: plan
                    .blocks
                    .iter()
                    .map(|block| generated(root.path(), block, 1))
                    .collect(),
                plan,
                marker_policy: SpeechMarkerPolicy::default(),
                caption: None,
            };
            last = Some(core.commit_narration_blocks(input).unwrap());
        }
        let state = core.get_project(&project).unwrap();
        assert_eq!(state.narration_manifests.len(), groups);
        assert_eq!(
            state
                .narration_manifests
                .iter()
                .map(|m| m.blocks.len())
                .sum::<usize>(),
            groups * blocks
        );
        let plan = core
            .plan_narration_blocks(input(&profile, "A.", None))
            .unwrap();
        let input = CommitNarrationBlocksInput {
            project_id: project.clone(),
            expected_revision: groups as u64,
            track_id: last.unwrap().manifest.blocks[0].binding.track_id.clone(),
            start_ms: groups as u64 * 1000,
            blocks: plan
                .blocks
                .iter()
                .map(|block| generated(root.path(), block, 1))
                .collect(),
            plan,
            marker_policy: SpeechMarkerPolicy::default(),
            caption: None,
        };
        let before = inventory(root.path());
        assert_eq!(
            core.commit_narration_blocks(input).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
}

#[test]
fn combined_caption_cue_limit_accepts100_and_rejects101_before_managed_import() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    let state = core.get_project(&input.project_id).unwrap();
    input.caption = Some(NarrationCaptionOptions {
        track_id: state
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Caption)
            .unwrap()
            .id
            .clone(),
        granularity: NarrationCaptionGranularity::Sentence,
    });
    input.marker_policy = SpeechMarkerPolicy::AllWord {};
    align_all(&mut input, Some("alignment-fixture"));
    for block in &mut input.blocks {
        let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut block.origin;
        generation.alignment.as_mut().unwrap().words = (0..49)
            .map(|index| SpeechTimedText {
                text: format!("word{index}"),
                start_ms: index * 2,
                end_ms: index * 2 + 1,
            })
            .collect();
    }
    let mut excessive = input.clone();
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut excessive.blocks[0].origin;
    generation
        .alignment
        .as_mut()
        .unwrap()
        .words
        .push(SpeechTimedText {
            text: "word49".into(),
            start_ms: 98,
            end_ms: 99,
        });
    let before = inventory(root.path());
    assert_eq!(
        core.commit_narration_blocks(excessive).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    let result = core.commit_narration_blocks(input).unwrap();
    assert_eq!(
        result
            .manifest
            .blocks
            .iter()
            .map(|block| block.binding.caption_refs.len() + block.binding.cue_refs.len())
            .sum::<usize>(),
        100
    );
}

#[test]
fn canonical_wire_examples_and_single_field_closed_negatives_deserialize_without_resource_access() {
    fn parse(target: &str, value: Value) -> Result<(), serde_json::Error> {
        match target {
            "planInput" => serde_json::from_value::<NarrationPlanInput>(value).map(|_| ()),
            "plan" => serde_json::from_value::<NarrationPlan>(value).map(|_| ()),
            "preparedBlock" => serde_json::from_value::<NarrationPreparedBlock>(value).map(|_| ()),
            "manifest" => serde_json::from_value::<NarrationManifest>(value).map(|_| ()),
            "commitInput" => {
                serde_json::from_value::<CommitNarrationBlocksInput>(value).map(|_| ())
            }
            _ => panic!("Unknown independently authored wire fixture target"),
        }
    }
    let canonical = fixture();
    for (target, value) in canonical["wire"].as_object().unwrap() {
        parse(target, value.clone()).unwrap();
    }
    for case in canonical["wireInvalidCases"].as_array().unwrap() {
        let target = case["target"].as_str().unwrap();
        let mut value = canonical["wire"][target].clone();
        let path = case["path"].as_array().unwrap();
        let mut parent = &mut value;
        for part in &path[..path.len() - 1] {
            parent = if let Some(key) = part.as_str() {
                parent.get_mut(key).unwrap()
            } else {
                parent.get_mut(part.as_u64().unwrap() as usize).unwrap()
            };
        }
        let field = path.last().unwrap().as_str().unwrap();
        parent[field] = case["value"].clone();
        let error = parse(target, value).unwrap_err();
        assert!(error.is_data(), "{target}.{field}: {error}");
        assert!(
            error.to_string().contains(if case["value"].is_null() {
                "null"
            } else {
                field
            }),
            "{target}.{field}: {error}"
        );
    }
}

#[test]
fn edit_draft_v2_replays_migrated_bases_and_cannot_invalidate_owned_bindings() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let committed = core.commit_narration_blocks(input).unwrap();
    let id = &committed.write.project_id;
    let operation: EditOperation = serde_json::from_value(
        json!({"operation":"delete_item","itemId":committed.manifest.blocks[0].binding.item_id}),
    )
    .unwrap();
    let before = inventory(root.path());
    match core.create_draft(id, 1, vec![operation], None) {
        Err(error) => {
            assert_eq!(error.code, ErrorCode::ValidationFailed);
            assert_eq!(inventory(root.path()), before);
        }
        Ok(draft) => {
            let before = inventory(root.path());
            assert_eq!(
                core.get_draft_state(id, &draft.id).unwrap_err().code,
                ErrorCode::ValidationFailed
            );
            assert_eq!(
                core.commit_draft(id, &draft.id, 1).unwrap_err().code,
                ErrorCode::ValidationFailed
            );
            assert_eq!(inventory(root.path()), before);
        }
    }
    let update:EditOperation=serde_json::from_value(json!({"operation":"update_track","trackId":committed.manifest.blocks[0].binding.track_id,"name":"Unrelated draft edit"})).unwrap();
    let draft = core.create_draft(id, 1, vec![update], None).unwrap();
    assert_eq!(draft.version, 2);
    assert_eq!(
        core.get_draft_state(id, &draft.id)
            .unwrap()
            .project
            .narration_manifests,
        std::slice::from_ref(&committed.manifest)
    );
    core.commit_draft(id, &draft.id, 1).unwrap();
    assert_eq!(
        core.get_project(id).unwrap().narration_manifests,
        [committed.manifest]
    );

    let legacy_id = core
        .create_project("Legacy draft base", ProjectSettings::default())
        .unwrap()
        .project_id;
    let mut legacy = core.get_project(&legacy_id).unwrap();
    let operation: EditOperation = serde_json::from_value(
        json!({"operation":"update_track","trackId":legacy.tracks[0].id,"name":"Migrated draft"}),
    )
    .unwrap();
    let draft = core
        .create_draft(&legacy_id, 0, vec![operation], None)
        .unwrap();
    let dir = core.paths().projects_root().join(&legacy_id);
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let before = std::fs::read(&draft_path).unwrap();
    legacy.schema_version = 46;
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    let preview = core.get_draft_state(&legacy_id, &draft.id).unwrap();
    assert_eq!(preview.project.schema_version, 47);
    assert!(preview.project.narration_manifests.is_empty());
    assert_eq!(std::fs::read(&draft_path).unwrap(), before);
    let wire: Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(wire["version"], 2);
    assert!(wire.get("schemaVersion").is_none() && wire.get("narrationManifests").is_none());
    core.commit_draft(&legacy_id, &draft.id, 0).unwrap();
    assert_eq!(core.get_project(&legacy_id).unwrap().schema_version, 47);
}

#[test]
fn frozen_audio_rejects_move_trim_split_and_late_batch_but_duplicates_as_unowned_native_audio() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let result = core.commit_narration_blocks(input).unwrap();
    let id = &result.write.project_id;
    let first = &result.manifest.blocks[0];
    let track = &first.binding.track_id;
    let item = &first.binding.item_id;
    for operation in [
        json!({"operation":"move_item","itemId":item,"trackId":track,"startMs":200}),
        json!({"operation":"trim_item","itemId":item,"startMs":100,"durationMs":300}),
        json!({"operation":"split_item","itemId":item,"splitMs":300}),
    ] {
        let before = inventory(root.path());
        let operation: EditOperation = serde_json::from_value(operation).unwrap();
        assert_eq!(
            core.edit(id, 1, operation).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    let operations:Vec<EditOperation>=serde_json::from_value(json!([{"operation":"update_track","trackId":track,"name":"must roll back"},{"operation":"move_item","itemId":item,"trackId":track,"startMs":200}])).unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.edit_batch(id, 1, operations).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    let duplicate = serde_json::from_value(
        json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":1000}),
    )
    .unwrap();
    core.edit(id, 1, duplicate).unwrap();
    let state = core.get_project(id).unwrap();
    assert_eq!(
        state.narration_manifests,
        std::slice::from_ref(&result.manifest)
    );
    assert_eq!(
        state
            .tracks
            .iter()
            .find(|candidate| candidate.id == *track)
            .unwrap()
            .items
            .len(),
        3
    );
    assert_eq!(state.assets.len(), 2);
}

#[test]
fn new_block_effective_trim_matches_pinned_ecmascript_set_and_keeps_legacy_blank_rejection() {
    let (_root, core, profile) = setup();
    for character in [
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{00a0}',
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}',
        '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
    ] {
        let source = format!("{character}A.{character}");
        let plan = core
            .plan_narration_blocks(input(&profile, &source, Some(vec![4])))
            .unwrap();
        assert_eq!(plan.blocks[0].display_text, source);
        assert_eq!(plan.blocks[0].request.text, "A.");
    }
    for character in ['\u{0085}', '\u{180e}', '\u{200b}'] {
        let source = format!("{character}A.{character}");
        let plan = core
            .plan_narration_blocks(input(&profile, &source, Some(vec![4])))
            .unwrap();
        assert_eq!(plan.blocks[0].request.text, source);
    }
    for source in ["\u{0085}", "\u{feff}", " \t\r\n"] {
        assert_eq!(
            core.plan_narration_blocks(input(&profile, source, None))
                .unwrap_err()
                .code,
            ErrorCode::ValidationFailed
        );
    }
}

fn expansion_profile(
    core: &EditorCore,
    profile: &NarrationProfile,
    unused: usize,
) -> NarrationProfile {
    let mut settings = profile.settings.clone();
    settings.name = format!("Expansion budget {unused}");
    settings.text_options.pronunciations = std::iter::once(SpeechPronunciation {
        term: "A".into(),
        spoken: "x".repeat(128),
    })
    .chain((0..unused).map(|index| SpeechPronunciation {
        term: format!("unused-{index}"),
        spoken: "x".repeat(128),
    }))
    .collect();
    core.create_narration_profile(settings).unwrap()
}
fn expanded_input(
    root: &std::path::Path,
    core: &EditorCore,
    profile: &NarrationProfile,
    project_id: &str,
    revision: u64,
    count: usize,
) -> CommitNarrationBlocksInput {
    let source = "A".repeat(count * 32);
    let plan = core
        .plan_narration_blocks(input(
            profile,
            &source,
            Some((1..=count).map(|index| index * 32).collect()),
        ))
        .unwrap();
    assert!(serde_json::to_vec(&plan).unwrap().len() < MAX_NARRATION_MANIFEST_BYTES);
    let state = core.get_project(project_id).unwrap();
    let blocks = plan
        .blocks
        .iter()
        .map(|block| {
            let mut generated = generated(root, block, 1);
            generated.prepared.spoken_text = "x".repeat(4096);
            generated.prepared.spoken_segments = vec![generated.prepared.spoken_text.clone()];
            generated
        })
        .collect();
    CommitNarrationBlocksInput {
        project_id: project_id.into(),
        expected_revision: revision,
        track_id: state
            .tracks
            .iter()
            .find(|track| track.track_type == TrackType::Audio)
            .unwrap()
            .id
            .clone(),
        start_ms: revision * 1000,
        plan,
        blocks,
        marker_policy: SpeechMarkerPolicy::default(),
        caption: None,
    }
}

#[test]
fn stored_manifest_bytes_reject_a_valid_plan_and_prepared_set_before_any_managed_import() {
    let (root, core, profile) = setup();
    let profile = expansion_profile(&core, &profile, 46);
    let id = core
        .create_project("Stored manifest budget", ProjectSettings::default())
        .unwrap()
        .project_id;
    let input = expanded_input(root.path(), &core, &profile, &id, 0, 32);
    let prepared: Vec<_> = input.blocks.iter().map(|block| &block.prepared).collect();
    let prepared_envelope = json!({"id":input.plan.id,"sourceText":input.plan.source_text,"maxTextCharacters":input.plan.max_text_characters,"blocks":prepared});
    let prepared_bytes = serde_json::to_vec(&prepared_envelope).unwrap().len();
    assert!(
        prepared_bytes <= MAX_NARRATION_MANIFEST_BYTES,
        "prepared bytes {prepared_bytes}"
    );
    assert!(serde_json::to_vec(&input).unwrap().len() < MAX_NARRATION_INPUT_BYTES);
    let before = inventory(root.path());
    let error = core.commit_narration_blocks(input).unwrap_err();
    assert_eq!(error.code, ErrorCode::ValidationFailed);
    assert!(
        error.message.contains("serialized byte budget"),
        "{}",
        error.message
    );
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn project_manifest_bytes_reject_below_count_caps_with_each_manifest_individually_valid() {
    let (root, core, profile) = setup();
    let profile = expansion_profile(&core, &profile, 56);
    let id = core
        .create_project("Project metadata budget", ProjectSettings::default())
        .unwrap()
        .project_id;
    for revision in 0..8 {
        let input = expanded_input(root.path(), &core, &profile, &id, revision, 28);
        let result = core.commit_narration_blocks(input).unwrap();
        let bytes = serde_json::to_vec(&result.manifest).unwrap().len();
        assert!(
            bytes > MAX_NARRATION_PROJECT_BYTES / 9 && bytes <= MAX_NARRATION_MANIFEST_BYTES,
            "fixture manifest bytes {bytes}"
        );
    }
    let state = core.get_project(&id).unwrap();
    assert_eq!(state.narration_manifests.len(), 8);
    assert_eq!(
        state
            .narration_manifests
            .iter()
            .map(|manifest| manifest.blocks.len())
            .sum::<usize>(),
        224
    );
    assert!(
        serde_json::to_vec(&state.narration_manifests)
            .unwrap()
            .len()
            <= MAX_NARRATION_PROJECT_BYTES
    );
    let input = expanded_input(root.path(), &core, &profile, &id, 8, 28);
    let before = inventory(root.path());
    let error = core.commit_narration_blocks(input).unwrap_err();
    assert_eq!(error.code, ErrorCode::ValidationFailed);
    assert!(
        error.message.contains("serialized byte budget"),
        "{}",
        error.message
    );
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn native_commit_input_byte_budget_runs_before_probe_or_resource_import() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut input.blocks[0].origin;
    generation.model_id = "m".repeat(MAX_NARRATION_INPUT_BYTES - 20000);
    assert!(serde_json::to_vec(&input).unwrap().len() < MAX_NARRATION_INPUT_BYTES);
    let before = inventory(root.path());
    assert!(core.validate_narration_blocks_input(&input).is_ok());
    assert_eq!(inventory(root.path()), before);
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = &mut input.blocks[0].origin;
    generation.model_id = "m".repeat(MAX_NARRATION_INPUT_BYTES);
    let error = core.commit_narration_blocks(input).unwrap_err();
    assert_eq!(error.code, ErrorCode::ValidationFailed);
    assert!(
        error.message.contains("serialized byte budget"),
        "{}",
        error.message
    );
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn locked_audio_and_caption_tracks_reject_complete_publication_without_import() {
    for lock_caption in [false, true] {
        let (root, core, profile) = setup();
        let mut input = commit_input(root.path(), &core, &profile);
        let state = core.get_project(&input.project_id).unwrap();
        let caption_track = state
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Caption)
            .unwrap()
            .id
            .clone();
        input.caption = Some(NarrationCaptionOptions {
            track_id: caption_track.clone(),
            granularity: NarrationCaptionGranularity::Sentence,
        });
        align_all(&mut input, Some("actual-alignment-model"));
        let target = if lock_caption {
            caption_track
        } else {
            input.track_id.clone()
        };
        core.edit(
            &input.project_id,
            0,
            serde_json::from_value(
                json!({"operation":"update_track","trackId":target,"locked":true}),
            )
            .unwrap(),
        )
        .unwrap();
        input.expected_revision = 1;
        let before = inventory(root.path());
        assert_eq!(
            core.commit_narration_blocks(input).unwrap_err().code,
            ErrorCode::TrackLocked
        );
        assert_eq!(inventory(root.path()), before);
    }
}

#[test]
fn unsafe_placement_and_absent_requested_alignment_granularity_reject_before_import() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    for start in [9_007_199_254_740_491, u64::MAX - 100] {
        let mut invalid = input.clone();
        invalid.start_ms = start;
        let before = inventory(root.path());
        assert_eq!(
            core.commit_narration_blocks(invalid).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
        assert_eq!(inventory(root.path()), before);
    }
    let mut invalid = input.clone();
    align_all(&mut invalid, Some("actual-alignment-model"));
    let state = core.get_project(&invalid.project_id).unwrap();
    invalid.caption = Some(NarrationCaptionOptions {
        track_id: state
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Caption)
            .unwrap()
            .id
            .clone(),
        granularity: NarrationCaptionGranularity::Word,
    });
    let before = inventory(root.path());
    assert_eq!(
        core.commit_narration_blocks(invalid).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(root.path()), before);
    assert_eq!(
        core.commit_narration_blocks(input).unwrap().write.revision,
        1
    );
}

#[test]
fn owned_caption_cue_edits_fail_but_presentation_and_last_detach_preserve_native_content() {
    let (root, core, profile) = setup();
    let mut input = commit_input(root.path(), &core, &profile);
    let state = core.get_project(&input.project_id).unwrap();
    input.caption = Some(NarrationCaptionOptions {
        track_id: state
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Caption)
            .unwrap()
            .id
            .clone(),
        granularity: NarrationCaptionGranularity::Sentence,
    });
    input.marker_policy = SpeechMarkerPolicy::Sentence {};
    align_all(&mut input, Some("actual-alignment-model"));
    let committed = core.commit_narration_blocks(input).unwrap();
    let id = &committed.write.project_id;
    let block = &committed.manifest.blocks[0];
    let caption = &block.binding.caption_refs[0].id;
    let cue = &block.binding.cue_refs[0];
    for operation in [
        json!({"operation":"update_item","itemId":caption,"text":"Changed owned wording"}),
        json!({"operation":"trim_item","itemId":caption,"startMs":101,"durationMs":199}),
        json!({"operation":"delete_item","itemId":caption}),
        json!({"operation":"marker_update","scope":"root","markerId":cue.id,"name":"changed_owned_cue","timeMs":cue.time_ms,"kind":"cue"}),
        json!({"operation":"marker_delete","scope":"root","markerId":cue.id}),
    ] {
        let before = inventory(root.path());
        let label = operation.to_string();
        let error = core
            .edit(id, 1, serde_json::from_value(operation).unwrap())
            .unwrap_err();
        assert_eq!(
            error.code,
            ErrorCode::ValidationFailed,
            "{label}: {}",
            error.message
        );
        assert_eq!(inventory(root.path()), before);
    }
    let operations: Vec<EditOperation> = serde_json::from_value(json!([
        {"operation":"set_audio","itemId":block.binding.item_id,"audio":{"volume":0.5,"muted":false,"fadeInMs":0,"fadeOutMs":0}},
        {"operation":"set_item_visibility","itemId":caption,"hidden":true}
    ])).unwrap();
    core.edit_batch(id, 1, operations).unwrap();
    let state = core.get_project(id).unwrap();
    assert_eq!(
        state.narration_manifests,
        std::slice::from_ref(&committed.manifest)
    );
    // Caption styling has no dedicated edit operation: exercise persisted-domain validation.
    let mut styled = serde_json::to_value(&state).unwrap();
    for track in styled["tracks"].as_array_mut().unwrap() {
        for item in track["items"].as_array_mut().unwrap() {
            if item["id"] == *caption {
                item["style"]["color"] = json!("#112233");
            }
        }
    }
    std::fs::write(
        core.paths().projects_root().join(id).join("project.json"),
        serde_json::to_vec(&styled).unwrap(),
    )
    .unwrap();
    let state = core.get_project(id).unwrap();
    assert_eq!(
        state.narration_manifests,
        std::slice::from_ref(&committed.manifest)
    );
    let tracks = serde_json::to_value(&state.tracks).unwrap();
    let assets = serde_json::to_value(&state.assets).unwrap();
    core.detach_narration_block(id, 2, &committed.manifest.id, &block.id)
        .unwrap();
    core.detach_narration_block(
        id,
        3,
        &committed.manifest.id,
        &committed.manifest.blocks[1].id,
    )
    .unwrap();
    let detached = core.get_project(id).unwrap();
    assert!(detached.narration_manifests.is_empty());
    assert_eq!(serde_json::to_value(&detached.tracks).unwrap(), tracks);
    assert_eq!(serde_json::to_value(&detached.assets).unwrap(), assets);
    core.undo(id, 4).unwrap();
    let restored = core.get_project(id).unwrap();
    assert_eq!(
        restored.narration_manifests[0].blocks,
        std::slice::from_ref(&committed.manifest.blocks[1])
    );
    core.redo(id, 5).unwrap();
    assert!(
        EditorCore::new(core.paths().clone())
            .get_project(id)
            .unwrap()
            .narration_manifests
            .is_empty()
    );
}
