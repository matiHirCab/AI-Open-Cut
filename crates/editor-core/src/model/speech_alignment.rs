//! Provider-neutral, reference-free speech timing and producer provenance.
use super::{Asset, GeneratedAssetOrigin, SpeechGeneration};
use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Deserializer, Serialize};

pub const MAX_SPEECH_ALIGNMENT_SEGMENTS: usize = 100_000;
pub const MAX_SPEECH_ALIGNMENT_TEXT_BYTES: usize = 4096;
pub const MAX_SPEECH_ALIGNMENT_TOTAL_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_SPEECH_ALIGNMENT_ID_BYTES: usize = 256;
pub const MAX_SPEECH_ALIGNMENT_TIME_MS: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpeechMarkerPolicy {
    None {},
    Sentence {},
    SelectedWord { indices: Vec<u64> },
    AllWord {},
}

impl Default for SpeechMarkerPolicy {
    fn default() -> Self {
        Self::None {}
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechAlignmentQuality {
    Native,
    Forced,
    Estimated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTimedText {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechAlignment {
    pub sentences: Vec<SpeechTimedText>,
    pub words: Vec<SpeechTimedText>,
    pub phonemes: Vec<SpeechTimedText>,
    pub quality: SpeechAlignmentQuality,
    pub provider_id: String,
    #[serde(deserialize_with = "required_nullable_string")]
    pub model_id: Option<String>,
    #[serde(deserialize_with = "required_nullable_string")]
    pub model_version: Option<String>,
}

fn required_nullable_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

pub(super) fn deserialize_present_alignment<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SpeechAlignment>, D::Error> {
    SpeechAlignment::deserialize(deserializer).map(Some)
}

fn invalid(message: impl Into<String>) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

fn validate_identity(value: &str, label: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() || value.len() > MAX_SPEECH_ALIGNMENT_ID_BYTES {
        return Err(invalid(format!(
            "{label} must be nonblank and at most {MAX_SPEECH_ALIGNMENT_ID_BYTES} UTF-8 bytes"
        )));
    }
    Ok(())
}

impl SpeechAlignment {
    /// Intrinsic provenance checks; duration is supplied by the owning asset.
    pub fn validate(&self) -> Result<(), CoreError> {
        let count = self
            .sentences
            .len()
            .checked_add(self.words.len())
            .and_then(|count| count.checked_add(self.phonemes.len()));
        if !count.is_some_and(|count| (1..=MAX_SPEECH_ALIGNMENT_SEGMENTS).contains(&count)) {
            return Err(invalid(format!(
                "speech alignment requires 1 to {MAX_SPEECH_ALIGNMENT_SEGMENTS} total segments"
            )));
        }
        validate_identity(&self.provider_id, "alignment provider ID")?;
        if let Some(model_id) = &self.model_id {
            validate_identity(model_id, "alignment model ID")?;
        }
        if let Some(version) = &self.model_version {
            validate_identity(version, "alignment model version")?;
        }
        let mut text_bytes = 0_usize;
        for segments in [&self.sentences, &self.words, &self.phonemes] {
            let mut previous_end = 0;
            for segment in segments {
                if segment.start_ms > MAX_SPEECH_ALIGNMENT_TIME_MS
                    || segment.end_ms > MAX_SPEECH_ALIGNMENT_TIME_MS
                    || segment.end_ms <= segment.start_ms
                    || segment.start_ms < previous_end
                {
                    return Err(invalid(
                        "speech alignment requires safe integer positive spans ordered without overlap within each granularity",
                    ));
                }
                if segment.text.trim().is_empty()
                    || segment.text.len() > MAX_SPEECH_ALIGNMENT_TEXT_BYTES
                {
                    return Err(invalid(format!(
                        "speech alignment text must be nonblank and at most {MAX_SPEECH_ALIGNMENT_TEXT_BYTES} UTF-8 bytes"
                    )));
                }
                text_bytes = text_bytes
                    .checked_add(segment.text.len())
                    .filter(|count| *count <= MAX_SPEECH_ALIGNMENT_TOTAL_TEXT_BYTES)
                    .ok_or_else(|| invalid("speech alignment timing text exceeds 1 MiB"))?;
                previous_end = segment.end_ms;
            }
        }
        Ok(())
    }

    fn validate_duration(&self, duration_ms: Option<u64>) -> Result<(), CoreError> {
        let duration = duration_ms
            .filter(|duration| *duration > 0)
            .ok_or_else(|| invalid("aligned speech requires a known positive asset duration"))?;
        if self
            .sentences
            .iter()
            .chain(&self.words)
            .chain(&self.phonemes)
            .any(|segment| segment.end_ms > duration)
        {
            return Err(invalid(
                "speech alignment extends beyond its asset duration",
            ));
        }
        Ok(())
    }

    pub fn validate_for_duration(&self, duration_ms: Option<u64>) -> Result<(), CoreError> {
        self.validate()?;
        self.validate_duration(duration_ms)
    }
}

impl SpeechGeneration {
    pub fn validate_for_duration(&self, duration_ms: Option<u64>) -> Result<(), CoreError> {
        self.validate()?;
        if let Some(alignment) = &self.alignment {
            alignment.validate_duration(duration_ms)?;
        }
        Ok(())
    }
}

impl GeneratedAssetOrigin {
    pub fn validate_for_duration(&self, duration_ms: Option<u64>) -> Result<(), CoreError> {
        match self {
            Self::SpeechSynthesis(generation) => generation.validate_for_duration(duration_ms),
        }
    }
}

impl Asset {
    pub(crate) fn validate_speech_provenance(&self) -> Result<(), CoreError> {
        if let Some(origin) = &self.origin {
            origin.validate_for_duration(self.duration_ms)?;
        }
        Ok(())
    }
}
