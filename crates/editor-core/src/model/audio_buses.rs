use serde::{Deserialize, Deserializer, Serialize};

pub const AUDIO_BUS_IDS: [&str; 4] = ["voiceover", "music", "sfx", "master"];
pub const MAX_AUDIO_BUSES: usize = 4;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBus {
    pub id: String,
    #[serde(deserialize_with = "deserialize_nullable_bus_id")]
    pub output_bus_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub dsp: Option<AudioBusDsp>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBusDsp {
    pub gain_db: f64,
    pub pan: f64,
    pub eq: Vec<AudioBusEqBand>,
    #[serde(deserialize_with = "deserialize_nullable_compressor")]
    pub compressor: Option<AudioBusCompressor>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBusEqBand {
    pub frequency_hz: f64,
    pub q: f64,
    pub gain_db: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBusCompressor {
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub makeup_gain_db: f64,
}

fn deserialize_nullable_compressor<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<AudioBusCompressor>, D::Error> {
    Option::<AudioBusCompressor>::deserialize(deserializer)
}

impl AudioBusDsp {
    pub(crate) fn is_identity(&self) -> bool {
        self.gain_db == 0.0
            && self.pan == 0.0
            && self.eq.iter().all(|band| band.gain_db == 0.0)
            && self.compressor.is_none()
    }

    pub(crate) fn validate(&self) -> Result<(), CoreError> {
        fn bounded(value: f64, min: f64, max: f64) -> Result<(), CoreError> {
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(invalid(
                    "audio bus DSP parameter is outside its finite bound",
                ));
            }
            Ok(())
        }
        bounded(self.gain_db, -120.0, 24.0)?;
        bounded(self.pan, -1.0, 1.0)?;
        if self.eq.len() > 8 {
            return Err(invalid("audio bus DSP permits at most eight EQ bands"));
        }
        for band in &self.eq {
            bounded(band.frequency_hz, 20.0, 20_000.0)?;
            bounded(band.q, 0.1, 10.0)?;
            bounded(band.gain_db, -24.0, 24.0)?;
        }
        if let Some(compressor) = &self.compressor {
            bounded(compressor.threshold_db, -60.0, 0.0)?;
            bounded(compressor.ratio, 1.0, 20.0)?;
            bounded(compressor.attack_ms, 0.01, 2000.0)?;
            bounded(compressor.release_ms, 0.01, 9000.0)?;
            bounded(compressor.makeup_gain_db, 0.0, 24.0)?;
        }
        Ok(())
    }
}

pub(super) fn deserialize_nullable_bus_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

pub fn default_audio_buses() -> Vec<AudioBus> {
    AUDIO_BUS_IDS
        .into_iter()
        .map(|id| AudioBus {
            id: id.to_owned(),
            output_bus_id: (id != "master").then(|| "master".to_owned()),
            dsp: None,
        })
        .collect()
}

impl super::AudioTrackRole {
    pub const fn default_audio_bus_id(self) -> &'static str {
        match self {
            Self::Unassigned => "master",
            Self::Voiceover => "voiceover",
            Self::Music => "music",
            Self::SoundEffects => "sfx",
        }
    }
}

use super::{Project, Track, TrackType};
use crate::error::{CoreError, ErrorCode};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

fn validate_buses(buses: &[AudioBus]) -> Result<(), CoreError> {
    if buses.len() != MAX_AUDIO_BUSES
        || buses
            .iter()
            .zip(AUDIO_BUS_IDS)
            .any(|(bus, id)| bus.id != id)
    {
        return Err(invalid("audio buses require four ordered built-in IDs"));
    }
    for bus in buses {
        if let Some(dsp) = &bus.dsp {
            dsp.validate()?;
        }
        if bus.id == "master" {
            if bus.output_bus_id.is_some() {
                return Err(invalid("master audio bus must be terminal"));
            }
        } else if bus.output_bus_id.is_none() {
            return Err(invalid("audio bus stems require an output"));
        }
        route(buses, &bus.id)?;
    }
    Ok(())
}

fn route(buses: &[AudioBus], start: &str) -> Result<Vec<String>, CoreError> {
    let mut result = Vec::new();
    let mut id = start;
    loop {
        if result.len() >= MAX_AUDIO_BUSES || result.iter().any(|previous| previous == id) {
            return Err(invalid("audio bus route must be bounded and acyclic"));
        }
        let bus = buses
            .iter()
            .find(|bus| bus.id == id)
            .ok_or_else(|| invalid("audio bus reference was not found"))?;
        result.push(id.to_owned());
        match bus.output_bus_id.as_deref() {
            Some(output) => id = output,
            None if id == "master" => return Ok(result),
            None => return Err(invalid("audio bus route must terminate at master")),
        }
    }
}

fn validate_project(project: &Project) -> Result<(), CoreError> {
    if project.schema_version < 42 && project.audio_buses.iter().any(|bus| bus.dsp.is_some()) {
        return Err(invalid("audio bus DSP requires schema 42"));
    }
    if project.schema_version < 39 {
        if !project.audio_buses.is_empty()
            || project
                .tracks
                .iter()
                .chain(project.components.iter().flat_map(|c| &c.tracks))
                .any(|track| track.audio_bus_id.is_some())
        {
            return Err(invalid("audio bus routing requires schema 39"));
        }
        return Ok(());
    }
    validate_buses(&project.audio_buses)?;
    for track in project
        .tracks
        .iter()
        .chain(project.components.iter().flat_map(|c| &c.tracks))
    {
        validate_track(track)?;
    }
    Ok(())
}

fn validate_track(track: &Track) -> Result<(), CoreError> {
    if let Some(id) = &track.audio_bus_id {
        if !matches!(track.track_type, TrackType::Audio | TrackType::Video) {
            return Err(invalid(
                "explicit audio routing requires an audio or video track",
            ));
        }
        if !AUDIO_BUS_IDS.contains(&id.as_str()) {
            return Err(invalid("audio bus reference was not found"));
        }
    }
    Ok(())
}

/// Resolve canonical persisted routing without changing role-based rendering.
fn resolve_audio_bus_route(project: &Project, track: &Track) -> Result<Vec<String>, CoreError> {
    validate_project(project)?;
    validate_track(track)?;
    let id = track
        .audio_bus_id
        .as_deref()
        .unwrap_or(track.audio_role.default_audio_bus_id());
    if project.schema_version < 39 {
        return Ok(if id == "master" {
            vec![id.to_owned()]
        } else {
            vec![id.to_owned(), "master".to_owned()]
        });
    }
    route(&project.audio_buses, id)
}

impl Project {
    pub(crate) fn validate_audio_bus_model(&self) -> Result<(), CoreError> {
        validate_project(self)
    }

    pub(crate) fn resolve_audio_bus_metadata(
        &self,
        track: &Track,
    ) -> Result<Vec<String>, CoreError> {
        resolve_audio_bus_route(self, track)
    }
}
