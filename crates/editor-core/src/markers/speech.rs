//! Reference-free cue expansion using canonical asset and alignment bounds.
use std::collections::BTreeSet;

use crate::{
    CoreError, ErrorCode, GeneratedAssetOrigin, MAX_MARKERS_PER_COMPOSITION, Marker, MarkerKind,
    Project, SpeechAlignment, SpeechMarkerPolicy,
};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

fn base_name(text: &str, granularity: &str, ordinal: usize) -> String {
    let mut name = String::new();
    let mut separator = false;
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() {
            if separator && !name.is_empty() {
                name.push('_');
            }
            name.push(char::from(byte));
            separator = false;
        } else {
            separator = true;
        }
    }
    if name.is_empty() {
        name = format!("{granularity}_{ordinal}");
    } else if !name.as_bytes()[0].is_ascii_alphabetic() {
        name = format!("speech_{name}");
    }
    name.truncate(120);
    name
}

pub(crate) fn generate_speech_markers(
    project: &mut Project,
    scope: &str,
    asset_id: &str,
    start_ms: u64,
    policy: &SpeechMarkerPolicy,
    supplied_alignment: Option<&SpeechAlignment>,
) -> Result<Vec<String>, CoreError> {
    let existing = super::markers_for_scope(project, scope)?;
    if start_ms > super::MAX_SAFE {
        return Err(invalid("speech marker offset exceeds safe milliseconds"));
    }
    let scope_duration = scope.strip_prefix("component:").map(|id| {
        project
            .components
            .iter()
            .find(|component| component.id == id)
            .expect("validated component scope")
            .duration_ms
    });
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .ok_or_else(|| {
            CoreError::new(
                ErrorCode::AssetNotFound,
                "speech marker asset was not found",
            )
        })?;
    if !asset.has_audio {
        return Err(CoreError::new(
            ErrorCode::UnsupportedMedia,
            "speech marker asset requires audio",
        ));
    }
    let duration = asset
        .duration_ms
        .filter(|duration| *duration > 0)
        .ok_or_else(|| invalid("speech markers require known positive source duration"))?;
    if matches!(policy, SpeechMarkerPolicy::None {}) {
        if let Some(alignment) = supplied_alignment {
            alignment.validate_for_duration(Some(duration))?;
        }
        return Ok(Vec::new());
    }
    let alignment = supplied_alignment
        .or(match &asset.origin {
            Some(GeneratedAssetOrigin::SpeechSynthesis(generation)) => {
                generation.alignment.as_ref()
            }
            None => None,
        })
        .ok_or_else(|| invalid("speech markers require alignment"))?;
    alignment.validate_for_duration(Some(duration))?;
    let (segments, granularity) = match policy {
        SpeechMarkerPolicy::Sentence {} => (&alignment.sentences, "sentence"),
        _ => (&alignment.words, "word"),
    };
    if segments.is_empty() {
        return Err(invalid(
            "speech alignment does not contain requested marker granularity",
        ));
    }
    let indices: Vec<usize> = match policy {
        SpeechMarkerPolicy::SelectedWord { indices } => {
            if indices.is_empty() || indices.len() > MAX_MARKERS_PER_COMPOSITION {
                return Err(invalid(
                    "selected word indices must contain 1 to 4096 entries",
                ));
            }
            let mut selected = BTreeSet::new();
            for index in indices {
                let index =
                    usize::try_from(*index).map_err(|_| invalid("word index out of range"))?;
                if index >= segments.len() || !selected.insert(index) {
                    return Err(invalid("selected word indices must be valid and unique"));
                }
            }
            selected.into_iter().collect()
        }
        _ => {
            if segments.len() > MAX_MARKERS_PER_COMPOSITION {
                return Err(invalid("speech marker selection exceeds composition limit"));
            }
            (0..segments.len()).collect()
        }
    };
    if existing.len() + indices.len() > MAX_MARKERS_PER_COMPOSITION {
        return Err(invalid("speech markers exceed composition marker limit"));
    }
    let mut names: BTreeSet<String> = existing.iter().map(|marker| marker.name.clone()).collect();
    let mut output = Vec::with_capacity(indices.len());
    for index in indices {
        let segment = &segments[index];
        let end = start_ms
            .checked_add(segment.end_ms)
            .filter(|end| *end <= super::MAX_SAFE)
            .ok_or_else(|| invalid("speech marker interval exceeds safe milliseconds"))?;
        if scope_duration.is_some_and(|duration| end > duration) {
            return Err(invalid("speech marker interval exceeds component duration"));
        }
        let base = base_name(&segment.text, granularity, index + 1);
        let mut name = base.clone();
        let mut suffix = 2;
        while !names.insert(name.clone()) {
            name = format!("{base}_{suffix}");
            suffix += 1;
        }
        output.push(Marker {
            id: format!("m_{}", uuid::Uuid::new_v4()),
            name,
            scope: scope.into(),
            time_ms: start_ms + segment.start_ms,
            kind: MarkerKind::Cue,
        });
    }
    let ids = output.iter().map(|marker| marker.id.clone()).collect();
    super::markers_for_scope_mut(project, scope)?.extend(output);
    Ok(ids)
}
