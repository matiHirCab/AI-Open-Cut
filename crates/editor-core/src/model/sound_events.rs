use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{Asset, MediaType, Project};
use crate::error::{CoreError, ErrorCode};

pub const MAX_SOUND_DEFINITIONS: usize = 512;
pub const MAX_SOUND_VARIANTS: usize = 32;
pub const MAX_SOUND_VARIANT_SEED: u64 = 9_007_199_254_740_991;
pub const MIN_SOUND_GAIN_DB: f64 = -120.0;
pub const MAX_SOUND_GAIN_DB: f64 = 24.0;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoundEventDefinition {
    pub event: String,
    pub variant_asset_ids: Vec<String>,
    pub default_gain_db: f64,
    pub bus_id: String,
    pub variant_seed: u64,
}

#[derive(Debug)]
pub struct ResolvedSoundVariant<'a> {
    pub asset: &'a Asset,
    pub default_gain_db: f64,
    pub bus_id: &'a str,
    pub variant_seed: u64,
    pub variant_index: usize,
}

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

fn bounded_id(value: &str, alphabetic_first: bool) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().next().is_some_and(|first| {
            first.is_ascii_alphabetic() || (!alphabetic_first && first.is_ascii_digit())
        })
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn validate_definition(
    project: &Project,
    definition: &SoundEventDefinition,
    missing_asset_code: ErrorCode,
) -> Result<(), CoreError> {
    if !bounded_id(&definition.event, true) {
        return Err(invalid(
            "sound event name has an invalid bounded identifier",
        ));
    }
    if definition.variant_asset_ids.is_empty()
        || definition.variant_asset_ids.len() > MAX_SOUND_VARIANTS
    {
        return Err(invalid("sound definitions require one to 32 variants"));
    }
    if !definition.default_gain_db.is_finite()
        || !(MIN_SOUND_GAIN_DB..=MAX_SOUND_GAIN_DB).contains(&definition.default_gain_db)
    {
        return Err(invalid(
            "sound definition gain must be finite within [-120, 24] dB",
        ));
    }
    if definition.variant_seed > MAX_SOUND_VARIANT_SEED {
        return Err(invalid(
            "sound variant seed must be a JavaScript-safe unsigned integer",
        ));
    }
    if !super::AUDIO_BUS_IDS.contains(&definition.bus_id.as_str())
        || !project
            .audio_buses
            .iter()
            .any(|bus| bus.id == definition.bus_id)
    {
        return Err(invalid("sound definition audio bus was not found"));
    }
    let mut ids = HashSet::new();
    let mut hashes = HashSet::new();
    for id in &definition.variant_asset_ids {
        if !bounded_id(id, false) || !ids.insert(id) {
            return Err(invalid(
                "sound variants require distinct bounded asset identifiers",
            ));
        }
        let asset = project
            .assets
            .iter()
            .find(|asset| &asset.id == id)
            .ok_or_else(|| {
                CoreError::new(
                    missing_asset_code,
                    format!(
                        "sound definition {} references missing variant asset {id}",
                        definition.event
                    ),
                )
            })?;
        if asset.media_type != MediaType::Audio
            && !(asset.media_type == MediaType::Video && asset.has_audio)
        {
            return Err(invalid("sound variants require audio-bearing media"));
        }
        let hash = asset.content_hash.as_ref().ok_or_else(|| {
            invalid("sound variants require canonical content-addressed asset metadata")
        })?;
        if hash.algorithm != "sha256"
            || hash.digest.len() != 64
            || !hash
                .digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || asset.size_bytes.is_none_or(|size| size == 0)
            || !hashes.insert(&hash.digest)
        {
            return Err(invalid(
                "sound variants require distinct canonical sha256 content and positive sizes",
            ));
        }
    }
    Ok(())
}

impl Project {
    pub(crate) fn validate_sound_definition_model(&self) -> Result<(), CoreError> {
        if self.schema_version < 40 {
            return if self.sound_definitions.is_empty() {
                Ok(())
            } else {
                Err(invalid("sound definitions require schema 40"))
            };
        }
        if self.sound_definitions.len() > MAX_SOUND_DEFINITIONS {
            return Err(invalid("sound definition registry exceeds 512 names"));
        }
        let mut names = HashSet::new();
        for definition in &self.sound_definitions {
            if !names.insert(&definition.event) {
                return Err(invalid("sound event names must be unique"));
            }
            validate_definition(self, definition, ErrorCode::AssetIntegrityFailed)?;
        }
        Ok(())
    }

    pub(crate) fn register_sound_definition(
        &mut self,
        definition: SoundEventDefinition,
    ) -> Result<(), CoreError> {
        validate_definition(self, &definition, ErrorCode::AssetNotFound)?;
        if let Some(index) = self
            .sound_definitions
            .iter()
            .position(|old| old.event == definition.event)
        {
            self.sound_definitions[index] = definition;
        } else {
            if self.sound_definitions.len() >= MAX_SOUND_DEFINITIONS {
                return Err(invalid("sound definition registry exceeds 512 names"));
            }
            self.sound_definitions.push(definition);
        }
        Ok(())
    }

    /// Metadata-only deterministic selection; timeline sound activation is separate.
    pub fn resolve_sound_event_variant(
        &self,
        event: &str,
        seed: Option<u64>,
    ) -> Result<ResolvedSoundVariant<'_>, CoreError> {
        self.validate_audio_bus_model()?;
        self.validate_sound_definition_model()?;
        let definition = self
            .sound_definitions
            .iter()
            .find(|definition| definition.event == event)
            .ok_or_else(|| invalid("sound event definition was not found"))?;
        let seed = seed.unwrap_or(definition.variant_seed);
        if seed > MAX_SOUND_VARIANT_SEED {
            return Err(invalid(
                "sound variant seed must be a JavaScript-safe unsigned integer",
            ));
        }
        // Validated nonempty length<=32 makes both integer conversions exact.
        let index = (seed % definition.variant_asset_ids.len() as u64) as usize;
        let asset = self
            .assets
            .iter()
            .find(|asset| asset.id == definition.variant_asset_ids[index])
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::AssetIntegrityFailed,
                    "sound variant asset was not found",
                )
            })?;
        Ok(ResolvedSoundVariant {
            asset,
            default_gain_db: definition.default_gain_db,
            bus_id: &definition.bus_id,
            variant_seed: seed,
            variant_index: index,
        })
    }
}
