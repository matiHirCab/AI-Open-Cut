//! Stable narration orchestration through the ordinary staged project transaction.
use super::*;
use crate::validation::narration_blocks::{bounded, invalid, validate_plan, validate_prepared};
use crate::*;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitNarrationBlocksResult {
    pub write: WriteResult,
    pub manifest: NarrationManifest,
}

fn safe_add(start: u64, duration: u64) -> Result<u64, CoreError> {
    start
        .checked_add(duration)
        .filter(|end| *end <= 9_007_199_254_740_991)
        .ok_or_else(|| invalid("narration placement exceeds safe integer milliseconds"))
}

fn track_index(project: &Project, id: &str, kind: TrackType) -> Result<usize, CoreError> {
    let index = project
        .tracks
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| {
            CoreError::new(
                ErrorCode::TrackNotFound,
                "narration target track was not found",
            )
        })?;
    if project.tracks[index].locked {
        return Err(CoreError::new(
            ErrorCode::TrackLocked,
            "narration target track is locked",
        ));
    }
    if project.tracks[index].track_type != kind {
        return Err(invalid("narration target track has the wrong type"));
    }
    Ok(index)
}

fn caption_segments<'a>(
    input: &CommitNarrationBlocksInput,
    generated: &'a NarrationGeneratedBlock,
) -> Result<&'a [SpeechTimedText], CoreError> {
    let Some(options) = &input.caption else {
        return Ok(&[]);
    };
    let GeneratedAssetOrigin::SpeechSynthesis(generation) = &generated.origin;
    let alignment = generation
        .alignment
        .as_ref()
        .ok_or_else(|| invalid("requested narration captions require actual alignment"))?;
    if alignment.model_id.is_none() {
        return Err(invalid(
            "requested captions require an actual alignment modelId; cues and uncaptioned audio remain supported",
        ));
    }
    let segments = match options.granularity {
        NarrationCaptionGranularity::Sentence => &alignment.sentences,
        NarrationCaptionGranularity::Word => &alignment.words,
    };
    if segments.is_empty() {
        return Err(invalid(
            "requested caption alignment granularity is unavailable",
        ));
    }
    Ok(segments)
}

impl EditorCore {
    pub fn plan_narration_blocks(
        &self,
        input: NarrationPlanInput,
    ) -> Result<NarrationPlan, CoreError> {
        bounded(&input, MAX_NARRATION_MANIFEST_BYTES)?;
        if input.source_text.trim().is_empty()
            || input.source_text.len() > MAX_NARRATION_SOURCE_BYTES
            || !(1..=4096).contains(&input.max_text_characters)
        {
            return Err(invalid(
                "narration source or provider text budget is invalid",
            ));
        }
        let profile = self.get_narration_profile(&input.profile_id)?;
        if profile.version != input.expected_version {
            return Err(CoreError::new(
                ErrorCode::RevisionConflict,
                "narration profile changed; select an explicit current version",
            ));
        }
        let scalar_count = input.source_text.chars().count();
        let ends = if let Some(boundaries) = input.boundaries {
            boundaries
        } else {
            let mut boundaries = Vec::new();
            for (start, segment) in input.source_text.split_sentence_bound_indices() {
                if !segment.trim().is_empty() {
                    boundaries.push(input.source_text[..start + segment.len()].chars().count());
                }
            }
            if let Some(last) = boundaries.last_mut() {
                *last = scalar_count;
            }
            boundaries
        };
        if ends.is_empty()
            || ends.len() > MAX_NARRATION_BLOCKS
            || ends.last() != Some(&scalar_count)
        {
            return Err(invalid(
                "narration boundaries require 1..32 spans covering the source",
            ));
        }
        let mut scalar_bytes: Vec<usize> =
            input.source_text.char_indices().map(|(i, _)| i).collect();
        scalar_bytes.push(input.source_text.len());
        let mut blocks = Vec::new();
        let mut start = 0;
        for end in ends {
            if end <= start || end > scalar_count {
                return Err(invalid(
                    "narration boundaries must be strictly increasing Unicode scalar ends",
                ));
            }
            let display = input.source_text[scalar_bytes[start]..scalar_bytes[end]].to_owned();
            blocks.push(NarrationPlannedBlock {
                id: Uuid::new_v4().to_string(),
                start_scalar: start,
                end_scalar: end,
                paragraph: crate::validation::narration_blocks::paragraph(
                    &input.source_text[..scalar_bytes[start]],
                ),
                request: SpeechSynthesisRequest {
                    text: crate::validation::narration_blocks::trim_narration_request(&display)
                        .into(),
                    language: profile.settings.language.clone(),
                    voice_id: profile.settings.voice_id.clone(),
                    speed: profile.settings.speed,
                    text_options: profile.settings.text_options.clone(),
                    narration_profile: Some(NarrationProfileAttribution {
                        id: profile.id.clone(),
                        version: profile.version,
                        name: profile.settings.name.clone(),
                    }),
                },
                display_text: display,
            });
            start = end;
        }
        let plan = NarrationPlan {
            id: Uuid::new_v4().to_string(),
            source_text: input.source_text,
            max_text_characters: input.max_text_characters,
            blocks,
        };
        validate_plan(&plan, true)?;
        Ok(plan)
    }

    /// Shared admission for transports that must probe actual generated audio.
    /// All semantic limits stay in core and run before any provider file probe.
    pub fn validate_narration_blocks_input(
        &self,
        input: &CommitNarrationBlocksInput,
    ) -> Result<Vec<PathBuf>, CoreError> {
        bounded(&input, MAX_NARRATION_INPUT_BYTES)?;
        validate_plan(&input.plan, true)?;
        if input.blocks.len() != input.plan.blocks.len() {
            return Err(invalid("complete narration preview set is required"));
        }
        let mut count = 0usize;
        let mut position = input.start_ms;
        let mut paths = Vec::new();
        for (generated, planned) in input.blocks.iter().zip(&input.plan.blocks) {
            if generated.prepared.block != *planned {
                return Err(invalid(
                    "narration generated block does not match the approved plan",
                ));
            }
            validate_prepared(&generated.prepared, input.plan.max_text_characters)?;
            validate_duration(generated.duration_ms)?;
            generated
                .origin
                .validate_for_duration(Some(generated.duration_ms))?;
            let GeneratedAssetOrigin::SpeechSynthesis(generation) = &generated.origin;
            if generation.request != planned.request {
                return Err(invalid(
                    "generated speech request does not match immutable planned intent",
                ));
            }
            let marker_count = match &input.marker_policy {
                SpeechMarkerPolicy::None {} => 0,
                SpeechMarkerPolicy::Sentence {} => generation
                    .alignment
                    .as_ref()
                    .map_or(0, |a| a.sentences.len()),
                SpeechMarkerPolicy::AllWord {} => {
                    generation.alignment.as_ref().map_or(0, |a| a.words.len())
                }
                SpeechMarkerPolicy::SelectedWord { indices } => indices.len(),
            };
            count = count
                .checked_add(marker_count)
                .and_then(|n| n.checked_add(caption_segments(input, generated).ok()?.len()))
                .ok_or_else(|| {
                    invalid("narration reference budget or requested alignment is invalid")
                })?;
            if count > 100 {
                return Err(invalid(
                    "narration creates at most100 combined caption/cue references",
                ));
            }
            position = safe_add(position, generated.duration_ms)?;
            if generated.path.as_os_str().len() > 4096 {
                return Err(invalid("generated narration path exceeds4096 bytes"));
            }
            paths.push(self.paths.generated_media_path(&generated.path)?);
        }
        Ok(paths)
    }

    pub fn commit_narration_blocks(
        &self,
        input: CommitNarrationBlocksInput,
    ) -> Result<CommitNarrationBlocksResult, CoreError> {
        let paths = self.validate_narration_blocks_input(&input)?;
        let (write, manifest) = self.edit_with_staged_change(
            &input.project_id,
            input.expected_revision,
            false,
            |prepared, dir, rollback| {
                let project = &mut prepared.project;
                if project
                    .narration_manifests
                    .iter()
                    .any(|m| m.id == input.plan.id)
                {
                    return Err(invalid(
                        "narration manifest already exists; inspect before retrying",
                    ));
                }
                let audio_track = track_index(project, &input.track_id, TrackType::Audio)?;
                let caption_track = input
                    .caption
                    .as_ref()
                    .map(|c| track_index(project, &c.track_id, TrackType::Caption))
                    .transpose()?;
                if project.narration_manifests.len() >= 32
                    || project
                        .narration_manifests
                        .iter()
                        .map(|m| m.blocks.len())
                        .sum::<usize>()
                        + input.blocks.len()
                        > 256
                {
                    return Err(invalid("project narration count budget exceeded"));
                }
                // Validate prospective metadata and aggregate byte budgets before any media import.
                let mut prospective = project.narration_manifests.clone();
                let mut manifest = NarrationManifest {
                    marker_policy: input.marker_policy.clone(),
                    caption: input.caption.clone(),
                    id: input.plan.id.clone(),
                    source_text: input.plan.source_text.clone(),
                    max_text_characters: input.plan.max_text_characters,
                    blocks: Vec::new(),
                };
                let mut start = input.start_ms;
                for generated in &input.blocks {
                    let block = &generated.prepared.block;
                    manifest.blocks.push(NarrationStoredBlock {
                        id: block.id.clone(),
                        start_scalar: block.start_scalar,
                        end_scalar: block.end_scalar,
                        paragraph: block.paragraph,
                        display_text: block.display_text.clone(),
                        request: block.request.clone(),
                        spoken_segments: generated.prepared.spoken_segments.clone(),
                        spoken_text: generated.prepared.spoken_text.clone(),
                        binding: NarrationBinding {
                            track_id: input.track_id.clone(),
                            item_id: Uuid::new_v4().to_string(),
                            asset_id: Uuid::new_v4().to_string(),
                            start_ms: start,
                            duration_ms: generated.duration_ms,
                            source_in_ms: 0,
                            caption_refs: vec![],
                            cue_refs: vec![],
                        },
                    });
                    start = safe_add(start, generated.duration_ms)?;
                }
                prospective.push(manifest.clone());
                bounded(&manifest, MAX_NARRATION_MANIFEST_BYTES)?;
                bounded(&prospective, MAX_NARRATION_PROJECT_BYTES)?;
                let mut changed = Vec::new();
                for (generated, block) in input.blocks.iter().zip(&mut manifest.blocks) {
                    let mut probe = generated.probe.clone();
                    probe.duration_ms = Some(generated.duration_ms);
                    probe.has_audio = true;
                    project.assets.push(Asset {
                        artwork_derivation: None,
                        id: block.binding.asset_id.clone(),
                        media_type: MediaType::Audio,
                        file_name: generated_display_name(&generated.origin),
                        project_relative_path: format!(
                            "assets/narration-pending-{}.wav",
                            block.binding.asset_id
                        ),
                        duration_ms: Some(generated.duration_ms),
                        has_audio: true,
                        origin: Some(generated.origin.clone()),
                        content_hash: None,
                        size_bytes: None,
                        probe: Some(probe),
                    });
                    project.tracks[audio_track]
                        .items
                        .push(TimelineItem::Media(MediaItem {
                            audio_event: None,
                            id: block.binding.item_id.clone(),
                            asset_id: block.binding.asset_id.clone(),
                            start_ms: block.binding.start_ms,
                            duration_ms: block.binding.duration_ms,
                            source_in_ms: 0,
                            visual_properties: VisualProperties::default(),
                            audio: AudioSettings::default(),
                            keyframes: vec![],
                        }));
                    changed.extend([
                        block.binding.asset_id.clone(),
                        block.binding.item_id.clone(),
                    ]);
                    let (marker_ids, _) = apply_operation(
                        project,
                        EditOperation::SpeechMarkersGenerate {
                            scope: "root".into(),
                            asset_id: block.binding.asset_id.clone(),
                            start_ms: block.binding.start_ms,
                            marker_policy: input.marker_policy.clone(),
                            alignment: None,
                        },
                    )?;
                    for id in marker_ids {
                        // Native results also include normalized item ordinals.
                        if let Some(cue) = project.markers.iter().find(|m| m.id == id) {
                            block.binding.cue_refs.push(NarrationCueRef {
                                id: cue.id.clone(),
                                name: cue.name.clone(),
                                time_ms: cue.time_ms,
                            });
                        }
                        if !changed.contains(&id) {
                            changed.push(id);
                        }
                    }
                    if let Some(index) = caption_track {
                        let GeneratedAssetOrigin::SpeechSynthesis(generation) = &generated.origin;
                        let alignment = generation
                            .alignment
                            .as_ref()
                            .expect("prevalidated caption alignment");
                        for segment in caption_segments(&input, generated)? {
                            let id = Uuid::new_v4().to_string();
                            let start_ms = safe_add(block.binding.start_ms, segment.start_ms)?;
                            let duration_ms = segment.end_ms - segment.start_ms;
                            project.tracks[index]
                                .items
                                .push(TimelineItem::Caption(CaptionItem {
                                    id: id.clone(),
                                    text: segment.text.clone(),
                                    start_ms,
                                    duration_ms,
                                    style: CaptionStyle::default(),
                                    source: CaptionSource {
                                        asset_id: block.binding.asset_id.clone(),
                                        provider_id: alignment.provider_id.clone(),
                                        model_id: alignment
                                            .model_id
                                            .clone()
                                            .expect("prevalidated actual caption alignment model"),
                                        model_version: alignment.model_version.clone(),
                                        language: block.request.language.clone(),
                                        generated_at_ms: generation.generated_at_ms,
                                        original_text: segment.text.clone(),
                                        confidence: None,
                                        words: vec![],
                                    },
                                    visual_properties: VisualProperties::default(),
                                }));
                            block.binding.caption_refs.push(NarrationCaptionRef {
                                id: id.clone(),
                                text: segment.text.clone(),
                                start_ms,
                                duration_ms,
                            });
                            changed.push(id);
                        }
                    }
                }
                // Include exact generated reference metadata in the pre-publication bounds.
                bounded(&manifest, MAX_NARRATION_MANIFEST_BYTES)?;
                project.narration_manifests.push(manifest.clone());
                bounded(&project.narration_manifests, MAX_NARRATION_PROJECT_BYTES)?;
                normalize_stack_order(project)?;
                crate::validation::validate_project_visual_properties(project)?;
                // All metadata, tracks, alignment and final bindings were validated
                // before importing even the first generated resource.
                for (path, block) in paths.iter().zip(&manifest.blocks) {
                    let stored = crate::assets::store_generated_content_tracked(
                        self.storage.as_ref(),
                        dir,
                        path,
                        rollback,
                    )?;
                    let asset = project
                        .assets
                        .iter_mut()
                        .find(|a| a.id == block.binding.asset_id)
                        .expect("validated narration asset");
                    asset.project_relative_path = stored.relative_path;
                    asset.content_hash = Some(stored.content_hash);
                    asset.size_bytes = Some(stored.size_bytes);
                }
                Ok((
                    AppliedBatch {
                        changed_ids: changed,
                        aliases: BTreeMap::new(),
                        summary: "Committed complete narration block group",
                    },
                    manifest,
                ))
            },
        )?;
        Ok(CommitNarrationBlocksResult { write, manifest })
    }

    pub fn detach_narration_block(
        &self,
        project_id: &str,
        expected_revision: u64,
        manifest_id: &str,
        block_id: &str,
    ) -> Result<WriteResult, CoreError> {
        self.edit_with_staged_change(
            project_id,
            expected_revision,
            false,
            |prepared, _dir, _rollback| {
                let manifests = &mut prepared.project.narration_manifests;
                let index = manifests
                    .iter()
                    .position(|m| m.id == manifest_id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "narration manifest not found")
                    })?;
                let block = manifests[index]
                    .blocks
                    .iter()
                    .position(|b| b.id == block_id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "narration block not found")
                    })?;
                manifests[index].blocks.remove(block);
                if manifests[index].blocks.is_empty() {
                    manifests.remove(index);
                }
                Ok((
                    AppliedBatch {
                        changed_ids: vec![block_id.into()],
                        aliases: BTreeMap::new(),
                        summary: "Detached narration block ownership",
                    },
                    (),
                ))
            },
        )
        .map(|(write, ())| write)
    }
}

#[cfg(test)]
mod tests;
