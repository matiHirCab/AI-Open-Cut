//! Audio-only selection through the existing evaluated binding/path owner.
use super::*;

pub(crate) struct PreparedAudioMedia {
    pub(crate) inputs: Vec<MediaInputRequest>,
    pub(crate) paths: Vec<PathBuf>,
}

pub(crate) struct AudioAnalysisOutputDirectory {
    pub(crate) path: PathBuf,
    canonical_root: PathBuf,
}
impl AudioAnalysisOutputDirectory {
    pub(crate) fn admit(io: &dyn ArtifactIo, project_dir: &Path) -> Result<Self, CoreError> {
        let directory = Self {
            path: project_dir.join("previews"),
            canonical_root: io
                .canonicalize_artifact_path(project_dir)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?,
        };
        directory.validate(io, false)?;
        Ok(directory)
    }

    pub(crate) fn validate(
        &self,
        io: &dyn ArtifactIo,
        require_existing: bool,
    ) -> Result<(), CoreError> {
        let unsafe_output = || {
            CoreError::new(
                ErrorCode::PathNotAllowed,
                "audio analysis output directory is unsafe",
            )
        };
        let root = io
            .canonicalize_artifact_path(self.path.parent().ok_or_else(unsafe_output)?)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        if root != self.canonical_root {
            return Err(unsafe_output());
        }
        match io.audio_analysis_output_kind(&self.path) {
            Ok(ArtifactEntryKind::Directory) => {}
            Ok(_) => return Err(unsafe_output()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !require_existing => {
                return Ok(());
            }
            Err(_) => return Err(CoreError::render_failure(GRAPH_BUILD_STAGE, None, None)),
        }
        let resolved = io
            .canonicalize_artifact_path(&self.path)
            .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        if resolved != self.canonical_root.join("previews") {
            return Err(unsafe_output());
        }
        Ok(())
    }
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
