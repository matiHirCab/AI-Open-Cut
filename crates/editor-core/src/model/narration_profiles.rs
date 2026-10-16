use crate::{CoreError, ErrorCode, SpeechSynthesisRequest, SpeechTextOptions, SpeechVoiceId};
use serde::{Deserialize, Serialize};

pub const MAX_NARRATION_PROFILES: usize = 100;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationProfileSettings {
    pub name: String,
    pub language: String,
    pub voice_id: SpeechVoiceId,
    pub speed: f64,
    #[serde(default)]
    pub text_options: SpeechTextOptions,
}

impl NarrationProfileSettings {
    pub fn validate(&self) -> Result<(), CoreError> {
        validate_name(&self.name)?;
        for value in [&self.language, &self.voice_id.0] {
            if value.chars().count() > 256 || value.chars().any(char::is_control) {
                return Err(invalid(
                    "profile language/voice must be bounded identifiers",
                ));
            }
        }
        SpeechSynthesisRequest {
            text: "profile validation".into(),
            language: self.language.clone(),
            voice_id: self.voice_id.clone(),
            speed: self.speed,
            text_options: self.text_options.clone(),
            narration_profile: None,
        }
        .validate()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationProfile {
    pub id: String,
    pub version: u64,
    pub settings: NarrationProfileSettings,
}

impl NarrationProfile {
    pub fn validate(&self) -> Result<(), CoreError> {
        NarrationProfileAttribution {
            id: self.id.clone(),
            version: self.version,
            name: self.settings.name.clone(),
        }
        .validate()?;
        self.settings.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationProfileAttribution {
    pub id: String,
    pub version: u64,
    pub name: String,
}

impl NarrationProfileAttribution {
    pub fn validate(&self) -> Result<(), CoreError> {
        if uuid::Uuid::parse_str(&self.id).is_err()
            || self.version == 0
            || self.version > 9_007_199_254_740_991
        {
            return Err(invalid("invalid narration profile identity/version"));
        }
        validate_name(&self.name)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NarrationProfileApplication {
    pub profile_id: String,
    pub expected_version: u64,
    pub text: String,
    pub language: Option<String>,
    pub voice_id: Option<SpeechVoiceId>,
    pub speed: Option<f64>,
    pub text_options: Option<SpeechTextOptions>,
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.is_empty()
        || name != name.trim()
        || name.chars().count() > 80
        || name.chars().any(char::is_control)
    {
        return Err(invalid(
            "profile name must be trimmed, 1..80 Unicode scalars without controls",
        ));
    }
    Ok(())
}

pub(crate) fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

pub(super) fn deserialize_present_attribution<'de, D>(
    deserializer: D,
) -> Result<Option<NarrationProfileAttribution>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    NarrationProfileAttribution::deserialize(deserializer).map(Some)
}
