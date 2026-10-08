use crate::{CoreError, Project, Track};

pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
    project.validate_audio_bus_model()
}

/// Resolve canonical persisted routing without changing role-based rendering.
pub fn resolve_audio_bus_route(project: &Project, track: &Track) -> Result<Vec<String>, CoreError> {
    project.resolve_audio_bus_metadata(track)
}
