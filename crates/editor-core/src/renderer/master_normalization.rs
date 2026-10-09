//! Orchestrate root-only preparation through the existing owner seams.
use super::*;
use crate::{
    render_artifact::audio_analysis::PreparedAudioMedia,
    render_plan::audio_analysis::{
        AudioAnalysisOptions, AudioAnalysisPlan, build_audio_analysis_plan,
    },
};

pub(crate) fn capture_plan(
    scene: &EvaluatedScene,
    media: PreparedAudioMedia,
) -> Result<AudioAnalysisPlan, CoreError> {
    crate::evaluated_scene::master_normalization::admit_duration(scene.duration_ms)?;
    let mut bypass = scene.clone();
    bypass.master_normalization = None;
    build_audio_analysis_plan(
        &bypass,
        media.inputs,
        media.paths,
        AudioAnalysisOptions {
            start_ms: 0,
            end_ms: scene.duration_ms,
            waveform_bins: 1,
        },
    )
}

impl Renderer {
    pub fn master_normalization_readiness(&self) -> Result<(), CoreError> {
        self.readiness()?;
        self.process_executor
            .master_normalization_readiness(&self.ffmpeg_path)
    }

    pub(crate) fn prepare_master_normalization(
        &self,
        scene: &mut EvaluatedScene,
        plan: &AudioAnalysisPlan,
        workspace: &RenderWorkspace,
        on_progress: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        let normalization = scene.master_normalization.as_mut().ok_or_else(|| {
            CoreError::new(
                ErrorCode::InternalError,
                "missing active master normalization",
            )
        })?;
        let filter_path = workspace
            .path()
            .join("master-normalization-capture-filter.txt");
        write_filter_script(
            self.artifact_io.as_ref(),
            &filter_path,
            &plan.render.filter_graph,
        )?;
        normalization.prepared = Some(self.process_executor.prepare_master_normalization(
            &self.ffmpeg_path,
            plan,
            &normalization.settings,
            &filter_path,
            workspace.path(),
            on_progress,
        )?);
        Ok(())
    }
}
