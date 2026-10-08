//! Audio-only selection through the existing evaluated binding/path owner.
use super::*;

pub(crate) struct PreparedAudioMedia {
    pub(crate) inputs: Vec<MediaInputRequest>,
    pub(crate) paths: Vec<PathBuf>,
}

pub(crate) fn prepare_audio_media(
    io: &dyn ArtifactIo,
    evaluated: &EvaluatedSceneResult,
    project_dir: &Path,
) -> Result<PreparedAudioMedia, CoreError> {
    let inputs = media_input_requests_for(evaluated, true)?;
    let paths = resolve_media_input_paths(io, evaluated, project_dir, &inputs)?;
    Ok(PreparedAudioMedia { inputs, paths })
}
