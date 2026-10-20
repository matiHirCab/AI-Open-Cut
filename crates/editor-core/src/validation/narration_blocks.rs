//! Canonical manifest identity, wording and native binding validation.
use crate::*;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{self, Write};

pub(crate) fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

struct Budget(usize);
impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("narration metadata byte budget exceeded"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn bounded(value: &impl Serialize, bytes: usize) -> Result<(), CoreError> {
    serde_json::to_writer(Budget(bytes), value)
        .map_err(|_| invalid("narration metadata exceeds its serialized byte budget"))
}

pub(crate) fn valid_id(id: &str) -> Result<(), CoreError> {
    uuid::Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| invalid("narration identity must be a UUID"))
}

pub(crate) fn native_id(id: &str) -> Result<(), CoreError> {
    if id.is_empty() || id.len() > 128 {
        return Err(invalid(
            "native narration reference must be a nonempty string of at most128 UTF8 bytes",
        ));
    }
    Ok(())
}

pub(crate) fn paragraph(prefix: &str) -> usize {
    let mut index = 0;
    let mut has_text = false;
    let mut was_blank = false;
    for line in prefix.lines() {
        let blank = line.trim().is_empty();
        if blank && !was_blank && has_text {
            index += 1;
        }
        has_text |= !blank;
        was_blank = blank;
    }
    index
}

fn text(value: &str, max_chars: usize) -> Result<(), CoreError> {
    if value.trim().is_empty()
        || value.len() > MAX_NARRATION_TEXT_BYTES
        || value.chars().count() > max_chars
    {
        return Err(invalid(
            "narration block text exceeds its UTF8/scalar budget or is blank; supply explicit manual boundaries",
        ));
    }
    Ok(())
}

// Effective wording for new blocks must match the existing ECMAScript
// String.trim resolver. Keep legacy speech/blank preparation unchanged.
pub(crate) fn trim_narration_request(value: &str) -> &str {
    value.trim_matches(|character|matches!(character,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}'))
}

pub(crate) fn validate_plan(plan: &NarrationPlan, complete: bool) -> Result<(), CoreError> {
    bounded(plan, MAX_NARRATION_MANIFEST_BYTES)?;
    valid_id(&plan.id)?;
    if plan.source_text.trim().is_empty()
        || plan.source_text.len() > MAX_NARRATION_SOURCE_BYTES
        || !(1..=4096).contains(&plan.max_text_characters)
        || plan.blocks.is_empty()
        || plan.blocks.len() > MAX_NARRATION_BLOCKS
    {
        return Err(invalid("invalid narration source, budget or block count"));
    }
    let mut bytes: Vec<usize> = plan.source_text.char_indices().map(|(i, _)| i).collect();
    bytes.push(plan.source_text.len());
    let mut previous = 0;
    let mut ids = BTreeSet::new();
    let mut settings: Option<SpeechSynthesisRequest> = None;
    for block in &plan.blocks {
        valid_id(&block.id)?;
        if !ids.insert(&block.id)
            || block.start_scalar < previous
            || (complete && block.start_scalar != previous)
            || block.end_scalar <= block.start_scalar
            || block.end_scalar >= bytes.len()
        {
            return Err(invalid(
                "narration spans must be unique ordered nonempty scalar intervals",
            ));
        }
        let start = bytes[block.start_scalar];
        let end = bytes[block.end_scalar];
        if block.display_text != plan.source_text[start..end]
            || block.request.text != trim_narration_request(&block.display_text)
            || block.paragraph != paragraph(&plan.source_text[..start])
        {
            return Err(invalid("narration source/display/request span mismatch"));
        }
        text(&block.display_text, plan.max_text_characters)?;
        block.request.validate()?;
        bounded(&block.request, 131_072)?;
        if block.request.narration_profile.is_none() {
            return Err(invalid(
                "narration blocks require a pinned profile snapshot",
            ));
        }
        let mut effective = block.request.clone();
        effective.text.clear();
        if settings.as_ref().is_some_and(|prior| prior != &effective) {
            return Err(invalid(
                "narration plan profile/settings must be identical across blocks",
            ));
        }
        settings = Some(effective);
        previous = block.end_scalar;
    }
    if complete && previous + 1 != bytes.len() {
        return Err(invalid("narration plan must cover the complete source"));
    }
    Ok(())
}

pub(crate) fn validate_prepared(
    prepared: &NarrationPreparedBlock,
    max_chars: usize,
) -> Result<(), CoreError> {
    if prepared.spoken_segments.is_empty() || prepared.spoken_segments.len() > 32 {
        return Err(invalid("narration requires 1..32 declared spoken segments"));
    }
    text(&prepared.spoken_text, max_chars)?;
    let mut remaining = MAX_NARRATION_TEXT_BYTES;
    for segment in &prepared.spoken_segments {
        text(segment, max_chars)?;
        remaining = remaining
            .checked_sub(segment.len())
            .ok_or_else(|| invalid("spoken segment byte budget exceeded"))?;
    }
    if prepared.spoken_segments.join("\n") != prepared.spoken_text {
        return Err(invalid(
            "declared spoken text must equal LF-joined prepared segments",
        ));
    }
    Ok(())
}

pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
    if project.schema_version < 47 {
        return if project.narration_manifests.is_empty() {
            Ok(())
        } else {
            Err(invalid("narration manifests require schema47"))
        };
    }
    bounded(&project.narration_manifests, MAX_NARRATION_PROJECT_BYTES)?;
    if project.narration_manifests.len() > 32
        || project
            .narration_manifests
            .iter()
            .map(|m| m.blocks.len())
            .sum::<usize>()
            > 256
    {
        return Err(invalid("project narration manifest/block count exceeded"));
    }
    let mut manifest_ids = BTreeSet::new();
    let mut block_ids = BTreeSet::new();
    let mut native_ids = BTreeSet::new();
    let mut asset_ids = BTreeSet::new();
    for manifest in &project.narration_manifests {
        bounded(manifest, MAX_NARRATION_MANIFEST_BYTES)?;
        if !manifest_ids.insert(&manifest.id) {
            return Err(invalid("duplicate narration manifest identity"));
        }
        let plan = NarrationPlan {
            id: manifest.id.clone(),
            source_text: manifest.source_text.clone(),
            max_text_characters: manifest.max_text_characters,
            blocks: manifest
                .blocks
                .iter()
                .map(|b| NarrationPlannedBlock {
                    id: b.id.clone(),
                    start_scalar: b.start_scalar,
                    end_scalar: b.end_scalar,
                    paragraph: b.paragraph,
                    display_text: b.display_text.clone(),
                    request: b.request.clone(),
                })
                .collect(),
        };
        validate_plan(&plan, false)?;
        for block in &manifest.blocks {
            if !block_ids.insert(&block.id) {
                return Err(invalid(
                    "block identity is already owned by another manifest",
                ));
            }
            validate_prepared(
                &NarrationPreparedBlock {
                    block: plan
                        .blocks
                        .iter()
                        .find(|b| b.id == block.id)
                        .expect("validated plan block")
                        .clone(),
                    spoken_segments: block.spoken_segments.clone(),
                    spoken_text: block.spoken_text.clone(),
                },
                manifest.max_text_characters,
            )?;
            let binding = &block.binding;
            for id in [&binding.track_id, &binding.item_id, &binding.asset_id] {
                native_id(id)?;
            }
            if !native_ids.insert(&binding.item_id)
                || !asset_ids.insert(&binding.asset_id)
                || binding.caption_refs.len() + binding.cue_refs.len() > 100
            {
                return Err(invalid(
                    "narration native ownership is duplicated or oversized",
                ));
            }
            let track = project
                .tracks
                .iter()
                .find(|t| t.id == binding.track_id)
                .ok_or_else(|| {
                    invalid("owned narration track is missing; detach before editing")
                })?;
            let item = track.items.iter().find(|i| i.id() == binding.item_id);
            let Some(TimelineItem::Media(item)) = item else {
                return Err(invalid(
                    "owned narration audio item is missing; detach before editing",
                ));
            };
            if track.track_type != TrackType::Audio
                || item.asset_id != binding.asset_id
                || item.start_ms != binding.start_ms
                || item.duration_ms != binding.duration_ms
                || item.source_in_ms != binding.source_in_ms
            {
                return Err(invalid(
                    "owned narration audio binding changed; detach before editing",
                ));
            }
            let asset = project
                .assets
                .iter()
                .find(|a| a.id == binding.asset_id)
                .ok_or_else(|| invalid("owned narration asset is missing"))?;
            let Some(GeneratedAssetOrigin::SpeechSynthesis(generation)) = &asset.origin else {
                return Err(invalid(
                    "owned narration requires generated speech provenance",
                ));
            };
            if asset.media_type != MediaType::Audio || generation.request != block.request {
                return Err(invalid(
                    "owned narration request/provenance changed; detach before editing",
                ));
            }
            if binding.source_in_ms != 0 || asset.duration_ms != Some(binding.duration_ms) {
                return Err(invalid(
                    "narration binding must retain its complete original audio interval",
                ));
            }
            asset.validate_speech_provenance()?;
            let (selected_cues, _) = crate::markers::speech_marker_selection(
                generation.alignment.as_ref(),
                &manifest.marker_policy,
            )?;
            if selected_cues.len() != binding.cue_refs.len()
                || selected_cues
                    .iter()
                    .zip(&binding.cue_refs)
                    .any(|((_, segment), reference)| {
                        binding.start_ms.checked_add(segment.start_ms) != Some(reference.time_ms)
                    })
            {
                return Err(invalid(
                    "narration cue ownership does not match its exact selected alignment",
                ));
            }
            let caption_segments = match &manifest.caption {
                None => &[][..],
                Some(options) => {
                    let alignment = generation.alignment.as_ref().ok_or_else(|| {
                        invalid("owned narration captions require actual alignment")
                    })?;
                    if alignment.model_id.is_none() {
                        return Err(invalid(
                            "owned captions require actual alignment model identity",
                        ));
                    }
                    let segments = match options.granularity {
                        NarrationCaptionGranularity::Sentence => &alignment.sentences,
                        NarrationCaptionGranularity::Word => &alignment.words,
                    };
                    if segments.is_empty() {
                        return Err(invalid(
                            "owned captions require the selected actual alignment granularity",
                        ));
                    }
                    &segments[..]
                }
            };
            if caption_segments.len() != binding.caption_refs.len()
                || caption_segments
                    .iter()
                    .zip(&binding.caption_refs)
                    .any(|(segment, reference)| {
                        reference.text != segment.text
                            || binding.start_ms.checked_add(segment.start_ms)
                                != Some(reference.start_ms)
                            || reference.duration_ms != segment.end_ms - segment.start_ms
                    })
            {
                return Err(invalid(
                    "owned captions do not match selected alignment text and timing",
                ));
            }
            for reference in &binding.caption_refs {
                native_id(&reference.id)?;
                if !native_ids.insert(&reference.id) {
                    return Err(invalid("duplicate narration caption ownership"));
                }
                let caption = project
                    .tracks
                    .iter()
                    .filter(|t| {
                        t.track_type == TrackType::Caption
                            && manifest
                                .caption
                                .as_ref()
                                .is_some_and(|options| options.track_id == t.id)
                    })
                    .flat_map(|t| &t.items)
                    .find(|i| i.id() == reference.id);
                if !matches!(caption,Some(TimelineItem::Caption(c)) if c.text==reference.text && c.start_ms==reference.start_ms && c.duration_ms==reference.duration_ms)
                {
                    return Err(invalid(
                        "owned caption text/timing changed; detach before editing",
                    ));
                }
            }
            for reference in &binding.caption_refs {
                let caption = project
                    .tracks
                    .iter()
                    .flat_map(|track| &track.items)
                    .find_map(|item| match item {
                        TimelineItem::Caption(c) if c.id == reference.id => Some(c),
                        _ => None,
                    })
                    .ok_or_else(|| invalid("owned caption is missing"))?;
                let alignment = generation
                    .alignment
                    .as_ref()
                    .ok_or_else(|| invalid("caption alignment is missing"))?;
                if caption.source.asset_id != binding.asset_id
                    || caption.source.provider_id != alignment.provider_id
                    || Some(&caption.source.model_id) != alignment.model_id.as_ref()
                    || caption.source.model_version != alignment.model_version
                    || caption.source.original_text != reference.text
                {
                    return Err(invalid("owned caption producer/source provenance changed"));
                }
            }
            for reference in &binding.cue_refs {
                native_id(&reference.id)?;
                if !native_ids.insert(&reference.id)
                    || !project.markers.iter().any(|m| {
                        m.id == reference.id
                            && m.scope == "root"
                            && m.name == reference.name
                            && m.time_ms == reference.time_ms
                    })
                {
                    return Err(invalid(
                        "owned cue name/timing changed; detach before editing",
                    ));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_legacy_replacement(
    project: &Project,
    item_id: &str,
) -> Result<(), CoreError> {
    if project
        .narration_manifests
        .iter()
        .flat_map(|m| &m.blocks)
        .any(|b| b.binding.item_id == item_id)
    {
        return Err(invalid(
            "owned narration speech cannot be regenerated before explicit detachment",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn exact_json_byte_limits_reject_the_next_byte_without_truncation() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../contracts/narration-blocks-v1.json"
        )))
        .unwrap();
        for key in ["requestBytes", "manifestBytes", "projectManifestBytes"] {
            let bytes = fixture["limits"][key].as_u64().unwrap() as usize;
            assert!(bounded(&"x".repeat(bytes - 2), bytes).is_ok());
            assert!(bounded(&"x".repeat(bytes - 1), bytes).is_err());
        }
    }
}
