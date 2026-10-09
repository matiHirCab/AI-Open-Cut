//! Immutable audio-only analysis orchestration through the existing owners.
use super::*;
use crate::{
    render_artifact::audio_analysis::{AudioAnalysisOutputDirectory, prepare_audio_media},
    render_plan::audio_analysis::{
        AudioAnalysisOptions, AudioAnalysisSummary, build_audio_analysis_plan,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudioAnalysisResult {
    pub artifact: RenderArtifact,
    pub summary: AudioAnalysisSummary,
}

impl Renderer {
    pub fn audio_analysis_readiness(&self) -> Result<(), CoreError> {
        self.readiness()?;
        self.process_executor
            .audio_analysis_readiness(&self.ffmpeg_path)
    }

    pub fn analyze_audio(
        &self,
        project: &Project,
        project_dir: &Path,
        options: AudioAnalysisOptions,
        mut on_progress: impl FnMut(RenderProgress),
    ) -> Result<AudioAnalysisResult, CoreError> {
        options.admit(project.duration_ms())?;
        let evaluated = evaluate_project(
            project,
            project.settings.width,
            project.settings.height,
            project.settings.fps,
        )?;
        let media = prepare_audio_media(self.artifact_io.as_ref(), &evaluated, project_dir)?;
        let output_directory =
            AudioAnalysisOutputDirectory::admit(self.artifact_io.as_ref(), project_dir)?;
        let plan = build_audio_analysis_plan(&evaluated.scene, media.inputs, media.paths, options)?;
        // Complete semantic/resource/graph admission precedes dependency probes
        // and any request workspace, selected PCM, or published JSON work.
        self.audio_analysis_readiness()?;
        self.audio_bus_processing_readiness(&evaluated.scene)?;
        let workspace = RenderWorkspace::create(self.artifact_io.clone(), project_dir)?;
        let filter_path = workspace.path().join("audio-analysis-filter.txt");
        write_filter_script(
            self.artifact_io.as_ref(),
            &filter_path,
            &plan.render.filter_graph,
        )?;
        let pcm_path = workspace.path().join("audio-analysis-original.pcm");
        let document = self.process_executor.analyze_audio(
            &self.ffmpeg_path,
            &plan,
            &filter_path,
            &pcm_path,
            &mut on_progress,
        )?;
        let bytes = document.bounded_json(options)?;
        let file_name = format!("audio-analysis-{}.json", Uuid::new_v4());
        let directory = &output_directory.path;
        output_directory.validate(self.artifact_io.as_ref(), false)?;
        if !self.artifact_io.artifact_path_exists(directory) {
            self.artifact_io
                .create_dir(directory)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
        }
        output_directory.validate(self.artifact_io.as_ref(), true)?;
        let output = directory.join(&file_name);
        let temporary = temporary_output(self.artifact_io.as_ref(), directory, "json");
        let mut published = false;
        let result = (|| {
            self.artifact_io
                .write(&temporary, &bytes)
                .map_err(|_| CoreError::render_failure(GRAPH_BUILD_STAGE, None, None))?;
            publish_output_with(self.artifact_io.as_ref(), &temporary, &output, false)?;
            published = true;
            let artifact = artifact_with(
                self.artifact_io.as_ref(),
                &output,
                format!("previews/{file_name}"),
                "application/json",
                vec![],
            )?;
            Ok(AudioAnalysisResult {
                artifact,
                summary: document.summary,
            })
        })();
        if result.is_err() {
            let _ = self.artifact_io.remove(&temporary);
            if published {
                let _ = self.artifact_io.remove(&output);
            }
        } else {
            on_progress(RenderProgress { progress: 1.0 });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_artifact::ArtifactEntryKind;
    use crate::render_plan::audio_analysis::AudioAnalysisPlan;
    use crate::{
        AudioAnalysisDocument, AudioChannelStatistics, AudioWaveformBin, EditorCore, PathPolicy,
        ProjectSettings,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct AnalysisProcess {
        mode: &'static str,
        calls: AtomicUsize,
    }
    impl ProcessExecutor for AnalysisProcess {
        fn readiness(&self, _: &Path, _: &Path) -> Result<(), CoreError> {
            Ok(())
        }
        fn probe(&self, _: &Path, _: &Path) -> Result<ProbeResult, CoreError> {
            unreachable!()
        }
        fn execute(
            &self,
            _: &Path,
            _: &RenderPlan,
            _: &Path,
            _: &Path,
            _: &mut dyn FnMut(RenderProgress),
        ) -> Result<(), CoreError> {
            unreachable!()
        }
        fn audio_analysis_readiness(&self, _: &Path) -> Result<(), CoreError> {
            if self.mode == "unsupported" {
                Err(CoreError::new(
                    ErrorCode::DependencyUnavailable,
                    "unsupported analysis",
                ))
            } else {
                Ok(())
            }
        }
        fn analyze_audio(
            &self,
            _: &Path,
            plan: &AudioAnalysisPlan,
            _: &Path,
            pcm: &Path,
            _: &mut dyn FnMut(RenderProgress),
        ) -> Result<AudioAnalysisDocument, CoreError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            std::fs::write(pcm, b"private original PCM").unwrap();
            if self.mode == "process" {
                return Err(CoreError::render_failure("render", Some(2), None));
            }
            assert!(
                plan.render
                    .filter_graph
                    .contains("atrim=start_sample=0:end_sample=48000")
            );
            let stats = AudioChannelStatistics {
                min: 0.,
                max: 0.,
                rms: 0.,
            };
            // Independently specified silence; no accumulator or producer fixture.
            let mut document = AudioAnalysisDocument {
                version: 1,
                summary: AudioAnalysisSummary {
                    start_ms: 0,
                    end_ms: 1000,
                    sample_rate_hz: 48000,
                    channels: 2,
                    frame_count: 48000,
                    actual_bin_count: 2,
                    linear_sample_peak: 0.,
                    sample_peak_dbfs: None,
                    integrated_lufs: None,
                    true_peak_dbtp: None,
                    loudness_range_lu: 0.,
                    threshold_lufs: -70.,
                },
                bins: vec![
                    AudioWaveformBin {
                        start_frame: 0,
                        end_frame: 24000,
                        left: stats,
                        right: stats,
                    },
                    AudioWaveformBin {
                        start_frame: 24000,
                        end_frame: 48000,
                        left: stats,
                        right: stats,
                    },
                ],
            };
            match self.mode {
                "frames" => document.summary.frame_count = 47999,
                "nonfinite" => document.bins[0].left.rms = f64::NAN,
                "partition" => document.bins[1].start_frame = 23999,
                _ => {}
            }
            Ok(document)
        }
    }
    #[derive(Debug)]
    struct AnalysisIo {
        fault: &'static str,
        writes: AtomicUsize,
    }
    impl ArtifactIo for AnalysisIo {
        fn request_id(&self) -> String {
            "analysis-owned-test".into()
        }
        fn create_dir(&self, p: &Path) -> std::io::Result<()> {
            if self.fault == "workspace" {
                Err(std::io::Error::other("workspace fault"))
            } else {
                FileSystemArtifactIo.create_dir(p)
            }
        }
        fn remove_dir_all(&self, p: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove_dir_all(p)
        }
        fn read(&self, _: &Path) -> std::io::Result<Vec<u8>> {
            panic!("analysis must not read visual/font data")
        }
        fn write(&self, p: &Path, b: &[u8]) -> std::io::Result<()> {
            let count = self.writes.fetch_add(1, Ordering::SeqCst);
            if (self.fault == "filter" && count == 0) || (self.fault == "json" && count == 1) {
                Err(std::io::Error::other("write fault"))
            } else {
                FileSystemArtifactIo.write(p, b)
            }
        }
        fn list(&self, _: &Path) -> std::io::Result<Vec<PathBuf>> {
            panic!("analysis must not enumerate fonts")
        }
        fn entry_kind(&self, p: &Path) -> std::io::Result<ArtifactEntryKind> {
            FileSystemArtifactIo.entry_kind(p)
        }
        fn audio_analysis_output_kind(&self, p: &Path) -> std::io::Result<ArtifactEntryKind> {
            if self.fault == "output-metadata" {
                return Err(std::io::Error::other("output metadata fault"));
            }
            if self.fault == "output-unsupported" {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "output metadata unsupported",
                ));
            }
            FileSystemArtifactIo.audio_analysis_output_kind(p)
        }
        fn canonicalize_artifact_path(&self, p: &Path) -> std::io::Result<PathBuf> {
            if p.file_name().is_some_and(|name| name == "previews")
                && (self.fault == "unsafe-output"
                    || (self.fault == "changed-output" && self.writes.load(Ordering::SeqCst) > 0))
            {
                return Ok(p.parent().unwrap().join("outside"));
            }
            FileSystemArtifactIo.canonicalize_artifact_path(p)
        }
        fn artifact_path_exists(&self, p: &Path) -> bool {
            FileSystemArtifactIo.artifact_path_exists(p)
        }
        fn remove(&self, p: &Path) -> std::io::Result<()> {
            FileSystemArtifactIo.remove(p)
        }
        fn rename(&self, p: &Path, q: &Path) -> std::io::Result<()> {
            if self.fault == "publish" {
                Err(std::io::Error::other("publish fault"))
            } else {
                FileSystemArtifactIo.rename(p, q)
            }
        }
        fn size(&self, p: &Path) -> std::io::Result<u64> {
            if self.fault == "metadata" {
                Err(std::io::Error::other("metadata fault"))
            } else {
                FileSystemArtifactIo.size(p)
            }
        }
    }
    #[test]
    fn audio_analysis_injected_failures_clean_private_pcm_partial_and_published_json_without_state_changes()
     {
        for fault in [
            "unsupported",
            "workspace",
            "filter",
            "process",
            "frames",
            "nonfinite",
            "partition",
            "json",
            "publish",
            "metadata",
            "unsafe-output",
            "changed-output",
            "output-metadata",
            "output-unsupported",
            "success",
        ] {
            let root = tempfile::tempdir().unwrap();
            let core = EditorCore::new(
                PathPolicy::new(
                    root.path().join("projects"),
                    [root.path()],
                    root.path().join("exports"),
                )
                .unwrap(),
            );
            let id = core
                .create_project("Analysis ports", ProjectSettings::default())
                .unwrap()
                .project_id;
            let initial = core.get_project(&id).unwrap();
            core.edit(&id,0,serde_json::from_value(serde_json::json!({"operation":"add_solid_color","trackId":initial.tracks[1].id,"startMs":0,"durationMs":1000,"color":"#123456","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap()).unwrap();
            let project = core.get_project(&id).unwrap();
            let dir = root.path().join("projects").join(id);
            let before = std::fs::read(dir.join("project.json")).unwrap();
            let history = std::fs::read(dir.join("history.json")).unwrap();
            let process = Arc::new(AnalysisProcess {
                mode: fault,
                calls: AtomicUsize::new(0),
            });
            let io = Arc::new(AnalysisIo {
                fault,
                writes: AtomicUsize::new(0),
            });
            let renderer =
                Renderer::new("unused", "unused", None).with_adapters(process.clone(), io.clone());
            let result = renderer.analyze_audio(
                &project,
                &dir,
                AudioAnalysisOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    waveform_bins: 2,
                },
                |_| {},
            );
            if fault == "success" {
                assert_eq!(result.unwrap().summary.frame_count, 48000);
            } else {
                assert_eq!(
                    result.unwrap_err().code,
                    if matches!(fault, "unsafe-output" | "changed-output") {
                        ErrorCode::PathNotAllowed
                    } else if fault == "unsupported" {
                        ErrorCode::DependencyUnavailable
                    } else {
                        ErrorCode::FfmpegFailed
                    },
                    "{fault}"
                );
            }
            assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
            assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
            for e in std::fs::read_dir(&dir).unwrap() {
                assert!(
                    !e.unwrap()
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".opencut-"),
                    "{fault}"
                );
            }
            let outputs = std::fs::read_dir(dir.join("previews"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect::<Vec<_>>();
            assert_eq!(outputs.len(), usize::from(fault == "success"), "{fault}");
            assert_eq!(
                process.calls.load(Ordering::SeqCst),
                usize::from(!matches!(
                    fault,
                    "unsupported"
                        | "workspace"
                        | "filter"
                        | "unsafe-output"
                        | "output-metadata"
                        | "output-unsupported"
                )),
                "{fault}"
            );
            if matches!(
                fault,
                "unsafe-output" | "changed-output" | "output-metadata" | "output-unsupported"
            ) {
                assert_eq!(
                    io.writes.load(Ordering::SeqCst),
                    usize::from(fault == "changed-output"),
                    "{fault} must never write destination JSON"
                );
            }
        }
    }
}
