use serde::{Deserialize, Deserializer, Serialize};

pub const AUDIO_BUS_IDS: [&str; 4] = ["voiceover", "music", "sfx", "master"];
pub const MAX_AUDIO_BUSES: usize = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBus {
    pub id: String,
    #[serde(deserialize_with = "deserialize_nullable_bus_id")]
    pub output_bus_id: Option<String>,
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
