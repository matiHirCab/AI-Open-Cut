use crate::{
    AUDIO_BUS_IDS, AudioBus, CoreError, ErrorCode, MAX_AUDIO_BUSES, Project, Track, TrackType,
};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) fn validate_buses(buses: &[AudioBus]) -> Result<(), CoreError> {
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

pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
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
pub fn resolve_audio_bus_route(project: &Project, track: &Track) -> Result<Vec<String>, CoreError> {
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
