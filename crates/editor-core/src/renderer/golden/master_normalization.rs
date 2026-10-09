//! Independently authored source PCM and delivered-output EBU/crop references.
use super::*;
use crate::render_plan::{
    audio_analysis::AudioAnalysisPlan, master_normalization::PreparedMasterNormalization,
};
use crate::render_process::ProcessExecutor;
use crate::{AudioAnalysisDocument, AudioAnalysisOptions, MasterNormalization};
use std::sync::Mutex;

struct RetainedRoot(Option<tempfile::TempDir>);
impl RetainedRoot {
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
}
impl Drop for RetainedRoot {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Some(root) = self.0.take()
        {
            eprintln!(
                "failed master normalization native fixture retained at {}",
                root.keep().display()
            );
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct RecordingProcess {
    pcm: Mutex<Vec<f32>>,
    original: Mutex<Vec<f32>>,
    prepared: Mutex<Option<PreparedMasterNormalization>>,
    pub(super) rendered_pcm: Mutex<Vec<f32>>,
}
impl ProcessExecutor for RecordingProcess {
    fn readiness(&self, f: &Path, p: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.readiness(f, p)
    }
    fn probe(&self, p: &Path, a: &Path) -> Result<ProbeResult, CoreError> {
        SystemProcessExecutor.probe(p, a)
    }
    fn audio_analysis_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.audio_analysis_readiness(f)
    }
    fn master_normalization_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.master_normalization_readiness(f)
    }
    fn audio_bus_dsp_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.audio_bus_dsp_readiness(f)
    }
    fn audio_bus_ducking_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.audio_bus_ducking_readiness(f)
    }
    fn prepare_master_normalization(
        &self,
        f: &Path,
        p: &AudioAnalysisPlan,
        settings: &MasterNormalization,
        s: &Path,
        w: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<PreparedMasterNormalization, CoreError> {
        let prepared =
            SystemProcessExecutor.prepare_master_normalization(f, p, settings, s, w, g)?;
        *self.original.lock().unwrap() = fs::read(w.join("master-normalization-original.pcm"))
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        *self.prepared.lock().unwrap() = Some(prepared.clone());
        Ok(prepared)
    }
    fn execute(
        &self,
        f: &Path,
        p: &RenderPlan,
        s: &Path,
        o: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
        // Observe the actual ordinary-render graph before its final codec/crop.
        // This is compared to an independently metered full mix, never pinned
        // as a producer-generated golden fixture.
        let mut render = p.clone();
        render.filter_graph.push_str(&format!(";[video]nullsink;[audio]aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000,atrim=start_sample=0:end_sample={},asetpts=PTS-STARTPTS[analysis]",p.duration_ms*48));
        let plan = AudioAnalysisPlan {
            render,
            options: AudioAnalysisOptions {
                start_ms: 0,
                end_ms: p.duration_ms,
                waveform_bins: 1,
            },
            expected_frames: p.duration_ms * 48,
        };
        let filter = s.with_file_name("observed-ordinary-render-filter.txt");
        let pcm = s.with_file_name("observed-ordinary-render.pcm");
        fs::write(&filter, &plan.render.filter_graph).unwrap();
        SystemProcessExecutor.analyze_audio(f, &plan, &filter, &pcm, &mut |_| {})?;
        *self.rendered_pcm.lock().unwrap() = fs::read(pcm)
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        SystemProcessExecutor.execute(f, p, s, o, g)
    }
    fn analyze_audio(
        &self,
        f: &Path,
        p: &AudioAnalysisPlan,
        s: &Path,
        a: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<AudioAnalysisDocument, CoreError> {
        let result = SystemProcessExecutor.analyze_audio(f, p, s, a, g)?;
        *self.pcm.lock().unwrap() = fs::read(a)
            .unwrap()
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect();
        Ok(result)
    }
}

pub(super) fn source(
    path: &Path,
    duration_ms: u64,
    amplitude: f64,
    dynamic: bool,
    high_frequency: bool,
) {
    let frames = duration_ms * 48;
    let size = u32::try_from(frames * 8).unwrap();
    let mut writer = std::io::BufWriter::new(fs::File::create(path).unwrap());
    use std::io::Write;
    writer.write_all(b"RIFF").unwrap();
    writer.write_all(&(36 + size).to_le_bytes()).unwrap();
    writer.write_all(b"WAVEfmt ").unwrap();
    writer.write_all(&16u32.to_le_bytes()).unwrap();
    writer.write_all(&3u16.to_le_bytes()).unwrap();
    writer.write_all(&2u16.to_le_bytes()).unwrap();
    writer.write_all(&48000u32.to_le_bytes()).unwrap();
    writer.write_all(&384000u32.to_le_bytes()).unwrap();
    writer.write_all(&8u16.to_le_bytes()).unwrap();
    writer.write_all(&32u16.to_le_bytes()).unwrap();
    writer.write_all(b"data").unwrap();
    writer.write_all(&size.to_le_bytes()).unwrap();
    for frame in 0..frames {
        let seconds = frame as f64 / 48000.0;
        let level = if dynamic {
            if seconds < 2.0 {
                0.08
            } else if seconds < 5.0 {
                0.7
            } else if seconds < 7.0 {
                0.015
            } else if seconds < 10.0 {
                0.4
            } else {
                0.1
            }
        } else {
            amplitude
        };
        let frequencies = if high_frequency {
            [(20000.0, 1.0), (22000.0, 0.7)]
        } else {
            [(440.0, 1.0), (660.0, 0.7)]
        };
        for (frequency, balance) in frequencies {
            writer
                .write_all(
                    &((level * balance * (2.0 * std::f64::consts::PI * frequency * seconds).sin())
                        as f32)
                        .to_le_bytes(),
                )
                .unwrap();
        }
    }
    writer.flush().unwrap();
}

pub(super) fn independent_metrics(tools: &NativeTools, pcm: &[f32], path: &Path) -> (f64, f64) {
    fs::write(
        path,
        pcm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
    )
    .unwrap();
    let run = |filter: &str| {
        let output = Command::new(&tools.ffmpeg)
            .args([
                "-hide_banner",
                "-nostdin",
                "-f",
                "f32le",
                "-ar",
                "48000",
                "-ac",
                "2",
                "-i",
            ])
            .arg(path)
            .args(["-af", filter, "-f", "null", "-"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "independent meter: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    let ebu = run("ebur128=peak=true:metadata=1,ametadata=mode=print:key=lavfi.r128.I:file=-");
    let integrated = String::from_utf8(ebu.stdout)
        .unwrap()
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("lavfi.r128.I="))
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let fine = run("loudnorm=I=-24:TP=-2:LRA=7:print_format=json");
    let text = String::from_utf8(fine.stderr).unwrap();
    let report = serde_json::Deserializer::from_str(&text[text.rfind('{').unwrap()..])
        .into_iter::<serde_json::Value>()
        .next()
        .unwrap()
        .unwrap();
    let peak = report["input_tp"].as_str().unwrap().parse().unwrap();
    (integrated, peak)
}

fn independent_authored_mix(
    tools: &NativeTools,
    path: &Path,
    filter: &str,
    duration: u64,
) -> Vec<f32> {
    let output = Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args(["-f", "lavfi", "-i"])
        .arg(format!(
            "anullsrc=r=48000:cl=stereo:d={}",
            duration as f64 / 1000.0
        ))
        .args([
            "-filter_complex",
            filter,
            "-map",
            "[reference]",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-c:a",
            "pcm_f32le",
            "-f",
            "f32le",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent authored mix: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len() % 8, 0);
    output
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|bytes| f32::from_le_bytes(*bytes))
        .collect()
}

fn assert_pcm(reference: &[f32], actual: &[f32], context: &str) {
    assert_eq!(reference.len(), actual.len(), "{context}");
    let rms = (reference
        .iter()
        .zip(actual)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / reference.len() as f64)
        .sqrt();
    assert!(rms <= 0.0001, "{context}: RMS {rms}");
}

#[test]
fn native_two_pass_master_normalization_targets_ranges_drafts_and_failure_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let mut covered = std::collections::BTreeSet::new();
    for (case, duration, amplitude, dynamic) in [
        ("stereo_tone", 3000, 0.125, false),
        ("silence", 3000, 0.0, false),
        ("short_audible", 250, 0.125, false),
        ("very_quiet", 3000, 1e-10, false),
        ("tiny_finite", 3000, 1e-44, false),
        ("peak_limiting", 12000, 0.0, true),
        ("infeasible_target_failure", 3000, 0.125, false),
        ("dynamic_warm_root_ranges", 12000, 0.0, true),
        ("above_full_scale", 3000, 256.0, false),
        ("extreme_finite", 3000, 1e38, false),
        ("routed_dsp_ducking", 3000, 0.125, false),
        ("component_event_clocks", 3000, 0.125, false),
        ("selected_audio_resources", 3000, 0.125, false),
        ("draft_frame_range_export_analysis", 3000, 0.125, false),
        ("bounded_target_correction", 3000, 0.125, false),
        ("readiness_and_owned_failure_cleanup", 3000, 0.125, false),
    ] {
        let root = RetainedRoot(Some(tempdir().unwrap()));
        let mut project = fixture_project();
        project.schema_version = PROJECT_SCHEMA_VERSION;
        project.audio_buses = crate::default_audio_buses();
        project.tracks[0].items.clear();
        project.assets[0].duration_ms = Some(duration);
        let TimelineItem::Media(item) = &mut project.tracks[1].items[0] else {
            panic!("fixture media")
        };
        item.duration_ms = duration;
        item.audio = crate::AudioSettings::default();
        item.keyframes.clear();
        project.master_normalization = Some(MasterNormalization {
            enabled: true,
            target_integrated_lufs: -16.0,
            target_loudness_range_lu: if dynamic { 1.0 } else { 7.0 },
            target_true_peak_dbtp: -1.0,
        });
        if case == "infeasible_target_failure" {
            let controls = project.master_normalization.as_mut().unwrap();
            controls.target_integrated_lufs = -5.0;
            controls.target_true_peak_dbtp = -9.0;
        }
        let core = crate::EditorCore::new(
            crate::PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        project.id = core
            .create_project("Native normalization", project.settings.clone())
            .unwrap()
            .project_id;
        project.revision = 0;
        let dir = core.paths().project_dir(&project.id).unwrap();
        source(
            &dir.join("assets/tone.wav"),
            duration,
            amplitude,
            dynamic,
            case == "bounded_target_correction",
        );
        fs::copy(
            dir.join("assets/tone.wav"),
            root.path().join("independent-source.wav"),
        )
        .unwrap();
        let mut authored_reference = None;
        if case == "routed_dsp_ducking" {
            project.tracks[1].audio_bus_id = Some("music".into());
            let mut voice = project.tracks[1].clone();
            voice.id = "voice-track".into();
            voice.audio_bus_id = Some("voiceover".into());
            voice.audio_role = AudioTrackRole::Voiceover;
            let TimelineItem::Media(item) = &mut voice.items[0] else {
                unreachable!()
            };
            item.id = "voice".into();
            item.start_ms = 200;
            item.duration_ms = 200;
            item.audio.volume = 0.2;
            project.tracks.push(voice);
            project.audio_buses[1].dsp = Some(
                serde_json::from_value(
                    serde_json::json!({"gainDb":-6,"pan":-0.5,"eq":[],"compressor":null}),
                )
                .unwrap(),
            );
            project.audio_buses[1].ducking = Some(crate::AudioBusDucking {
                enabled: true,
                source_bus_id: "voiceover".into(),
                gain: 0.25,
                attack_ms: 100,
                release_ms: 200,
            });
            let envelope = "if(lt(t,0.1),1,if(lt(t,0.2),1-0.75*(t-0.1)/0.1,if(lt(t,0.4),0.25,if(lt(t,0.6),0.25+0.75*(t-0.4)/0.2,1))))";
            authored_reference = Some(format!(
                "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=2[m][v];[m]volume=0.501187,volume='{envelope}':eval=frame,pan=stereo|c0=c0|c1=0.5*c1[music];[v]atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.2,adelay=200:all=1[voice];[1:a][music][voice]amix=inputs=3:duration=longest:normalize=0,aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000,atrim=start_sample=0:end_sample=144000,asetpts=PTS-STARTPTS[reference]"
            ));
        }
        if case == "component_event_clocks" {
            let mut local = project.tracks[1].clone();
            local.id = "local-track".into();
            let TimelineItem::Media(local_item) = &mut local.items[0] else {
                unreachable!()
            };
            local_item.id = "local-audio".into();
            local_item.start_ms = 100;
            local_item.source_in_ms = 73;
            local_item.duration_ms = 200;
            local_item.audio.volume = 0.2;
            project.components = vec![crate::ComponentDefinition {
                id: "voice-component".into(),
                name: "Voice".into(),
                width: WIDTH,
                height: HEIGHT,
                duration_ms: 1000,
                tracks: vec![local],
                slots: vec![],
                markers: vec![],
            }];
            project.tracks[0].items=serde_json::from_value(serde_json::json!([{"type":"component_instance","id":"voice-instance","componentId":"voice-component","startMs":200,"trimStartMs":0,"durationMs":500,"timeScale":2,"slotValues":{},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"zIndex":0,"stackOrder":0}])).unwrap();
            let hash = crate::ContentHash {
                algorithm: "sha256".into(),
                digest: hash_bytes(&fs::read(dir.join("assets/tone.wav")).unwrap()),
            };
            project.assets[0].content_hash = Some(hash.clone());
            project.assets[0].size_bytes =
                Some(fs::metadata(dir.join("assets/tone.wav")).unwrap().len());
            project.sound_definitions = vec![crate::SoundEventDefinition {
                event: "cue".into(),
                variant_asset_ids: vec!["tone".into()],
                default_gain_db: -6.0,
                bus_id: "sfx".into(),
                variant_seed: 0,
            }];
            // The registered event is placed through the canonical timeline edit
            // so its durable content binding and occurrence clock are real.
            fs::write(
                dir.join("project.json"),
                serde_json::to_vec_pretty(&project).unwrap(),
            )
            .unwrap();
            core.edit(&project.id,0,serde_json::from_value(serde_json::json!({"operation":"timeline_add_audio_event","scope":"root","trackId":project.tracks[1].id,"event":"cue","at":{"type":"milliseconds","valueMs":1200},"durationMs":200,"gainDb":0,"variantSeed":0})).unwrap()).unwrap();
            project = core.get_project(&project.id).unwrap();
            authored_reference=Some("[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=3[bed][local][event];[local]atrim=start=0.073:duration=0.2,asetpts=PTS-STARTPTS,atempo=2,volume=0.2,adelay=250:all=1[voice];[event]atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.501187,adelay=1200:all=1[sfx];[1:a][bed][voice][sfx]amix=inputs=4:duration=longest:normalize=0,aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000,atrim=start_sample=0:end_sample=144000,asetpts=PTS-STARTPTS[reference]".to_owned());
        }
        if case == "selected_audio_resources" {
            let mut asset = project.assets[0].clone();
            asset.id = "missing-visual".into();
            asset.media_type = MediaType::Image;
            asset.has_audio = false;
            asset.project_relative_path = "assets/missing-visual.png".into();
            project.assets.push(asset);
            let TimelineItem::Media(mut image) = project.tracks[1].items[0].clone() else {
                unreachable!()
            };
            image.id = "unrelated-image".into();
            image.asset_id = "missing-visual".into();
            project.tracks[0].items.push(TimelineItem::Media(image));
        }
        let recording = Arc::new(RecordingProcess::default());
        let mut renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
        renderer.process_executor = recording.clone();
        if case == "readiness_and_owned_failure_cleanup" {
            let unavailable = Renderer::new(
                root.path().join("absent-ffmpeg"),
                &tools.ffprobe,
                Some(tools.font.clone()),
            );
            assert_eq!(
                unavailable
                    .master_normalization_readiness()
                    .unwrap_err()
                    .code,
                ErrorCode::DependencyUnavailable
            );
            renderer.master_normalization_readiness().unwrap();
            let before_project = fs::read(dir.join("project.json")).unwrap();
            let before_history = fs::read(dir.join("history.json")).unwrap();
            let published = dir.join("previews/unrelated.json");
            fs::write(&published, b"unrelated published bytes").unwrap();
            fs::write(dir.join("assets/tone.wav"), b"corrupt selected native WAV").unwrap();
            let failure = renderer
                .analyze_audio(
                    &project,
                    &dir,
                    AudioAnalysisOptions {
                        start_ms: 0,
                        end_ms: duration,
                        waveform_bins: 1,
                    },
                    |_| {},
                )
                .unwrap_err();
            assert_eq!(failure.code, ErrorCode::FfmpegFailed);
            assert_eq!(fs::read(&published).unwrap(), b"unrelated published bytes");
            assert_eq!(fs::read(dir.join("project.json")).unwrap(), before_project);
            assert_eq!(fs::read(dir.join("history.json")).unwrap(), before_history);
            assert_eq!(fs::read_dir(dir.join("previews")).unwrap().count(), 1);
            assert!(!fs::read_dir(&dir).unwrap().any(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".opencut-work-")
            }));
            fs::copy(
                root.path().join("independent-source.wav"),
                dir.join("assets/tone.wav"),
            )
            .unwrap();
        }
        let result = renderer.analyze_audio(
            &project,
            &dir,
            AudioAnalysisOptions {
                start_ms: 0,
                end_ms: duration,
                waveform_bins: 7,
            },
            |_| {},
        );
        if case == "infeasible_target_failure" {
            assert_eq!(result.unwrap_err().code, ErrorCode::FfmpegFailed);
            assert!(fs::read_dir(dir.join("previews")).unwrap().next().is_none());
            assert!(!fs::read_dir(&dir).unwrap().any(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".opencut-work")
            }));
            covered.insert(case);
            continue;
        }
        let result = result.unwrap_or_else(|error| panic!("{case}: {error:?}"));
        let full = recording.pcm.lock().unwrap().clone();
        if let Some(filter) = authored_reference {
            let reference = independent_authored_mix(
                &tools,
                &root.path().join("independent-source.wav"),
                &filter,
                duration,
            );
            assert_pcm(&reference, &recording.original.lock().unwrap(), case);
        }
        assert_eq!(full.len(), duration as usize * 96);
        if ["silence", "short_audible", "very_quiet", "tiny_finite"].contains(&case) {
            assert!(result.summary.integrated_lufs.is_none(), "{case}");
            if case == "silence" {
                assert!(full.iter().all(|sample| *sample == 0.0));
            } else {
                assert!(
                    full.iter().any(|sample| *sample != 0.0),
                    "{case} retains audible/subnormal samples"
                );
                assert!(result.summary.true_peak_dbtp.unwrap() <= -0.99);
                assert!(result.summary.linear_sample_peak <= f64::from(amplitude as f32));
            }
            assert!(!fs::read_dir(&dir).unwrap().any(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".opencut-work")
            }));
            covered.insert(case);
            continue;
        }
        let (integrated, peak) = independent_metrics(
            &tools,
            &full,
            &root.path().join("independent-normalized.pcm"),
        );
        assert!((integrated + 16.0).abs() <= 0.1, "{case}: {integrated}");
        assert!(peak <= -0.99, "{case}: {peak}");
        if case == "bounded_target_correction" {
            let prepared = recording.prepared.lock().unwrap();
            let Some(PreparedMasterNormalization::Measured { postgain_db, .. }) = prepared.as_ref()
            else {
                panic!("measured correction required")
            };
            assert!(
                *postgain_db > 0.1 && *postgain_db < 1.0,
                "independent high-frequency resampling case must exercise one actual correction: {postgain_db}"
            );
        }
        if case == "draft_frame_range_export_analysis" {
            fs::write(
                dir.join("project.json"),
                serde_json::to_vec_pretty(&project).unwrap(),
            )
            .unwrap();
            let before = fs::read(dir.join("project.json")).unwrap();
            renderer.render_preview(&project, &dir, 1234).unwrap();
            assert_pcm(
                &full,
                &recording.rendered_pcm.lock().unwrap(),
                "frame graph full-root master stage",
            );
            renderer
                .render_preview_range(
                    &project,
                    &dir,
                    crate::PreviewRangeOptions {
                        start_ms: 1003,
                        end_ms: 2017,
                        width: WIDTH,
                        height: HEIGHT,
                        fps: FPS,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap();
            assert_pcm(
                &full,
                &recording.rendered_pcm.lock().unwrap(),
                "range graph full-root master stage",
            );
            renderer
                .export_video(
                    &project,
                    &dir,
                    crate::ExportOptions {
                        output: &root.path().join("normalized-export.mp4"),
                        width: WIDTH,
                        height: HEIGHT,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            assert_pcm(
                &full,
                &recording.rendered_pcm.lock().unwrap(),
                "export graph full-root master stage",
            );
            assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
            let mut controls = project.master_normalization.clone().unwrap();
            controls.target_integrated_lufs = -18.0;
            controls.target_true_peak_dbtp = -2.0;
            let draft = core
                .create_draft(
                    &project.id,
                    0,
                    vec![crate::EditOperation::AudioMasterSetNormalization {
                        normalization: controls,
                    }],
                    None,
                )
                .unwrap();
            let state = core.get_draft_state(&project.id, &draft.id).unwrap();
            renderer
                .analyze_audio(
                    &state.project,
                    &dir,
                    AudioAnalysisOptions {
                        start_ms: 0,
                        end_ms: duration,
                        waveform_bins: 1,
                    },
                    |_| {},
                )
                .unwrap();
            let draft_pcm = recording.pcm.lock().unwrap().clone();
            let (draft_integrated, draft_peak) = independent_metrics(
                &tools,
                &draft_pcm,
                &root.path().join("independent-draft.pcm"),
            );
            assert!((draft_integrated + 18.0).abs() <= 0.1);
            assert!(draft_peak <= -1.99);
            renderer.render_preview(&state.project, &dir, 1234).unwrap();
            assert_pcm(
                &draft_pcm,
                &recording.rendered_pcm.lock().unwrap(),
                "draft frame graph shared stage",
            );
            assert_eq!(
                core.get_project(&project.id).unwrap().master_normalization,
                project.master_normalization
            );
        }
        if dynamic {
            for (start, length) in [(0, 250), (2003, 3017), (8959, 1013), (11407, 593)] {
                renderer
                    .analyze_audio(
                        &project,
                        &dir,
                        AudioAnalysisOptions {
                            start_ms: start,
                            end_ms: start + length,
                            waveform_bins: 3,
                        },
                        |_| {},
                    )
                    .unwrap();
                let selected = recording.pcm.lock().unwrap().clone();
                let reference = &full[start as usize * 96..(start + length) as usize * 96];
                assert_eq!(selected.len(), reference.len());
                let rms = (selected
                    .iter()
                    .zip(reference)
                    .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
                    .sum::<f64>()
                    / selected.len() as f64)
                    .sqrt();
                assert!(rms <= 0.0001, "warm range {start}/{length}: {rms}");
            }
        }
        assert!(!fs::read_dir(&dir).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".opencut-work")
        }));
        covered.insert(case);
    }
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../contracts/master-normalization-v1.json"
    ))
    .unwrap();
    let expected: std::collections::BTreeSet<_> = catalog["nativeCases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert!(
        expected.is_subset(&covered),
        "missing canonical native case: {:?}",
        expected.difference(&covered).collect::<Vec<_>>()
    );
}
