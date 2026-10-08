//! Independent source/filter references for the private original PCM and public JSON.
use super::*;
use crate::render_plan::audio_analysis::{AudioAnalysisPlan, LoudnessMetrics};
use crate::render_process::ProcessExecutor;
use crate::{AudioAnalysisDocument, AudioAnalysisOptions};
use std::sync::Mutex;

struct RetainedNativeRoot(Option<tempfile::TempDir>);
impl RetainedNativeRoot {
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
    fn keep(mut self) -> PathBuf {
        self.0.take().unwrap().keep()
    }
}
impl Drop for RetainedNativeRoot {
    fn drop(&mut self) {
        if std::thread::panicking()
            && let Some(root) = self.0.take()
        {
            eprintln!(
                "failed analysis native fixtures retained at {}",
                root.keep().display()
            );
        }
    }
}

#[derive(Debug, Default)]
struct RecordingProcess {
    pcm: Mutex<Vec<f32>>,
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
    fn audio_bus_dsp_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.audio_bus_dsp_readiness(f)
    }
    fn audio_bus_ducking_readiness(&self, f: &Path) -> Result<(), CoreError> {
        SystemProcessExecutor.audio_bus_ducking_readiness(f)
    }
    fn execute(
        &self,
        f: &Path,
        p: &RenderPlan,
        s: &Path,
        o: &Path,
        g: &mut dyn FnMut(RenderProgress),
    ) -> Result<(), CoreError> {
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
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        Ok(result)
    }
}

fn reference_pcm(tools: &NativeTools, source: &Path, filter: &str) -> Vec<f32> {
    let result = Command::new(&tools.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(source)
        .args([
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=48000:cl=stereo:d=1",
            "-filter_complex",
            filter,
            "-map",
            "[out]",
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
        result.status.success(),
        "independent reference: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout.len() % 8, 0);
    result
        .stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect()
}
fn reference_metrics(tools: &NativeTools, pcm: &[f32], path: &Path) -> LoudnessMetrics {
    fs::write(
        path,
        pcm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
    )
    .unwrap();
    let result = Command::new(&tools.ffmpeg)
        .args([
            "-hide_banner",
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-i",
        ])
        .arg(path)
        .args([
            "-af",
            "loudnorm=I=-24:TP=-2:LRA=7:print_format=json",
            "-f",
            "null",
            "-",
        ])
        .output()
        .unwrap();
    assert!(result.status.success());
    let text = String::from_utf8(result.stderr).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&text[text.rfind('{').unwrap()..=text.rfind('}').unwrap()]).unwrap();
    let metric = |key: &str| {
        let s = value[key].as_str().unwrap();
        if s == "-inf" {
            None
        } else {
            Some(s.parse::<f64>().unwrap())
        }
    };
    LoudnessMetrics {
        integrated_lufs: metric("input_i"),
        true_peak_dbtp: metric("input_tp"),
        loudness_range_lu: metric("input_lra").unwrap(),
        threshold_lufs: metric("input_thresh").unwrap(),
    }
}
fn near_nullable(actual: Option<f64>, expected: Option<f64>) {
    match (actual, expected) {
        (None, None) => {}
        (Some(a), Some(e)) => assert!((a - e).abs() <= 0.05, "loudness {a} versus independent {e}"),
        _ => panic!("nullable metric differs"),
    }
}

fn independent_stereo_source(path: &Path) {
    // Manually specified distinct channels; no renderer/accumulator-generated
    // input or expected waveform. References independently consume this WAV.
    let frames = 48_000_u32;
    let data_bytes = frames * 4;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&frames.to_le_bytes());
    bytes.extend_from_slice(&(frames * 4).to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for frame in 0..frames {
        for (frequency, amplitude) in [(440, 16000), (660, 9000)] {
            let phase = (frame * frequency * 4 / frames) % 4;
            let fraction =
                i32::try_from((frame * frequency * 4) % frames * amplitude / frames).unwrap();
            let amplitude = i32::try_from(amplitude).unwrap();
            let sample = match phase {
                0 => fraction,
                1 => amplitude - fraction,
                2 => -fraction,
                _ => -amplitude + fraction,
            };
            bytes.extend_from_slice(&i16::try_from(sample).unwrap().to_le_bytes());
        }
    }
    fs::write(path, bytes).unwrap();
}

#[test]
fn native_audio_analysis_original_pcm_statistics_loudness_warm_routes_components_and_resources() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../contracts/audio-analysis-v1.json"
    ))
    .unwrap();
    let cases: [&str; 9] = [
        "stereo_tone",
        "silence",
        "short_audible",
        "above_full_scale",
        "root_warm_compression",
        "routed_dsp_and_ducking",
        "component_and_event_clocks",
        "missing_unrelated_visual",
        "missing_selected_audio",
    ];
    assert_eq!(serde_json::to_value(cases).unwrap(), catalog["nativeCases"]);
    for case in cases {
        let root = RetainedNativeRoot(Some(tempdir().unwrap()));
        let mut p = fixture_project();
        p.schema_version = PROJECT_SCHEMA_VERSION;
        p.audio_buses = crate::default_audio_buses();
        p.tracks[0].items.clear();
        p.tracks[1].audio_bus_id = Some("music".into());
        let core = crate::EditorCore::new(
            crate::PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        p.id = core
            .create_project("Analysis native", p.settings.clone())
            .unwrap()
            .project_id;
        p.revision = 0;
        let dir = core.paths().project_dir(&p.id).unwrap();
        independent_stereo_source(&dir.join("assets/tone.wav"));
        independent_stereo_source(&root.path().join("independent-tone.wav"));
        let mut options = AudioAnalysisOptions {
            start_ms: 0,
            end_ms: 1000,
            waveform_bins: 7,
        };
        let mut source="[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[m];[1:a][m]amix=inputs=2:duration=longest:normalize=0[mix]".to_owned();
        if case == "silence" {
            if let TimelineItem::Media(item) = &mut p.tracks[1].items[0] {
                item.audio.volume = 0.0;
            }
            source="[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,volume=0[m];[1:a][m]amix=inputs=2:duration=longest:normalize=0[mix]".into();
        }
        if case == "above_full_scale" {
            if let TimelineItem::Media(item) = &mut p.tracks[1].items[0] {
                item.audio.volume = 4.0;
            }
            source="[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,volume=4[m];[1:a][m]amix=inputs=2:duration=longest:normalize=0[mix]".into();
        }
        if case == "short_audible" {
            options.end_ms = 100;
        }
        if case == "root_warm_compression" {
            p.audio_buses[1].dsp=Some(serde_json::from_value(serde_json::json!({"gainDb":0,"pan":0,"eq":[],"compressor":{"thresholdDb":-18,"ratio":4,"attackMs":100,"releaseMs":300,"makeupGainDb":0}})).unwrap());
            options.start_ms = 500;
            options.end_ms = 900;
            source="[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,acompressor=threshold=0.125893:ratio=4:attack=100:release=300:makeup=1:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1[m];[1:a][m]amix=inputs=2:duration=longest:normalize=0[mix]".into();
        }
        if case == "routed_dsp_and_ducking" || case == "component_and_event_clocks" {
            let mut voice = p.tracks[1].clone();
            voice.id = "voice".into();
            voice.audio_role = AudioTrackRole::Voiceover;
            voice.audio_bus_id = Some("voiceover".into());
            if let TimelineItem::Media(item) = &mut voice.items[0] {
                item.id = "voice-item".into();
                item.start_ms = 200;
                item.duration_ms = 200;
                item.audio.volume = 0.2;
            }
            p.audio_buses[1].ducking = Some(crate::AudioBusDucking {
                enabled: true,
                source_bus_id: "voiceover".into(),
                gain: 0.25,
                attack_ms: 100,
                release_ms: 200,
            });
            let basic = "if(lt(t,0.1),1,if(lt(t,0.2),1-0.75*(t-0.1)/0.1,if(lt(t,0.4),0.25,if(lt(t,0.6),0.25+0.75*(t-0.4)/0.2,1))))";
            if case == "routed_dsp_and_ducking" {
                p.audio_buses[1].dsp = Some(
                    serde_json::from_value(
                        serde_json::json!({"gainDb":-6,"pan":-0.5,"eq":[],"compressor":null}),
                    )
                    .unwrap(),
                );
                source = format!(
                    "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=2[m][v];[m]volume=0.501187,volume='{basic}':eval=frame,pan=stereo|c0=c0|c1=0.5*c1[music];[v]atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.2,adelay=200:all=1[voice];[1:a][music][voice]amix=inputs=3:duration=longest:normalize=0[mix]"
                );
                p.tracks.push(voice);
            } else {
                if let TimelineItem::Media(item) = &mut voice.items[0] {
                    item.start_ms = 100;
                    item.source_in_ms = 73;
                }
                p.components = vec![crate::ComponentDefinition {
                    id: "voice_component".into(),
                    name: "Voice".into(),
                    width: WIDTH,
                    height: HEIGHT,
                    duration_ms: 1000,
                    tracks: vec![voice.clone()],
                    slots: vec![],
                    markers: vec![],
                }];
                p.tracks[0].items=serde_json::from_value(serde_json::json!([{"type":"component_instance","id":"voice-instance","componentId":"voice_component","startMs":200,"trimStartMs":0,"durationMs":500,"timeScale":2,"slotValues":{},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"zIndex":0,"stackOrder":0}])).unwrap();
                let hash = crate::ContentHash {
                    algorithm: "sha256".into(),
                    digest: hash_bytes(&fs::read(dir.join("assets/tone.wav")).unwrap()),
                };
                p.assets[0].content_hash = Some(hash.clone());
                p.assets[0].size_bytes =
                    Some(fs::metadata(dir.join("assets/tone.wav")).unwrap().len());
                p.sound_definitions = vec![crate::SoundEventDefinition {
                    event: "voice-cue".into(),
                    variant_asset_ids: vec!["tone".into()],
                    default_gain_db: 0.0,
                    bus_id: "voiceover".into(),
                    variant_seed: 0,
                }];
                voice.id = "event-track".into();
                if let TimelineItem::Media(item) = &mut voice.items[0] {
                    item.id = "event-item".into();
                    item.start_ms = 600;
                    item.duration_ms = 100;
                    item.source_in_ms = 0;
                    item.audio.volume = 0.1;
                    item.audio_event = Some(crate::AudioEventItem {
                        event: "voice-cue".into(),
                        gain_db: 0.0,
                        default_gain_db: 0.0,
                        bus_id: "voiceover".into(),
                        variant_seed: 0,
                        variant_index: 0,
                        content_hash: hash,
                    });
                }
                p.tracks.push(voice);
                let envelope = "min(if(lt(t,0.15),1,if(lt(t,0.25),1-0.75*(t-0.15)/0.1,if(lt(t,0.35),0.25,if(lt(t,0.55),0.25+0.75*(t-0.35)/0.2,1)))),if(lt(t,0.5),1,if(lt(t,0.6),1-0.75*(t-0.5)/0.1,if(lt(t,0.7),0.25,if(lt(t,0.9),0.25+0.75*(t-0.7)/0.2,1)))))";
                source = format!(
                    "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=3[m][v][e];[m]volume='{envelope}':eval=frame[music];[v]atrim=start_sample=3504:end_sample=13104,asetpts=PTS-STARTPTS,volume=0.2,atempo=2,adelay=250:all=1[voice];[e]atrim=duration=0.1,asetpts=PTS-STARTPTS,volume=0.1,adelay=600:all=1[event];[1:a][music][voice][event]amix=inputs=4:duration=longest:normalize=0[mix]"
                );
            }
        }
        if case == "missing_unrelated_visual" {
            let mut asset = p.assets[0].clone();
            asset.id = "missing-image".into();
            asset.media_type = MediaType::Image;
            asset.has_audio = false;
            asset.project_relative_path = "assets/missing.png".into();
            p.assets.push(asset);
            let mut item = p.tracks[1].items[0].clone();
            if let TimelineItem::Media(item) = &mut item {
                item.id = "visual-item".into();
                item.asset_id = "missing-image".into();
            }
            p.tracks[0].track_type = TrackType::Video;
            p.tracks[0].items = vec![item];
        }
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec_pretty(&p).unwrap(),
        )
        .unwrap();
        let before = fs::read(dir.join("project.json")).unwrap();
        let recorder = Arc::new(RecordingProcess::default());
        let mut renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
        renderer.process_executor = recorder.clone();
        if case == "missing_selected_audio" {
            fs::remove_file(dir.join("assets/tone.wav")).unwrap();
            assert_eq!(
                renderer
                    .analyze_audio(&p, &dir, options, |_| {})
                    .unwrap_err()
                    .code,
                ErrorCode::FfmpegFailed
            );
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(
                    root.path().join("independent-tone.wav"),
                    dir.join("assets/tone.wav"),
                )
                .unwrap();
                assert_eq!(
                    renderer
                        .analyze_audio(&p, &dir, options, |_| {})
                        .unwrap_err()
                        .code,
                    ErrorCode::PathNotAllowed
                );
            }
            assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
            assert!(recorder.pcm.lock().unwrap().is_empty());
            assert!(!fs::read_dir(&dir).unwrap().any(|e| {
                e.unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".opencut-")
            }));
            continue;
        }
        let result = match renderer.analyze_audio(&p, &dir, options, |p| {
            assert!((0.0..=1.0).contains(&p.progress))
        }) {
            Ok(result) => result,
            Err(e) => {
                let retained = root.keep();
                panic!(
                    "{case}: {e:?}; failed fixtures retained at {}",
                    retained.display()
                )
            }
        };
        assert_eq!(fs::read(dir.join("project.json")).unwrap(), before);
        assert_eq!(result.artifact.mime_type, "application/json");
        let document: AudioAnalysisDocument =
            serde_json::from_slice(&fs::read(dir.join(&result.artifact.relative_path)).unwrap())
                .unwrap();
        assert_eq!(document.summary, result.summary);
        assert_eq!(
            result.summary.frame_count,
            (options.end_ms - options.start_ms) * 48
        );
        let filter = format!(
            "{source};[mix]aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000,atrim=start_sample={}:end_sample={},asetpts=PTS-STARTPTS[out]",
            options.start_ms * 48,
            options.end_ms * 48
        );
        let expected = reference_pcm(&tools, &root.path().join("independent-tone.wav"), &filter);
        if case == "stereo_tone" {
            let distinct = (expected
                .chunks_exact(2)
                .map(|frame| (f64::from(frame[0]) - f64::from(frame[1])).powi(2))
                .sum::<f64>()
                / (expected.len() / 2) as f64)
                .sqrt();
            assert!(
                distinct > PCM_RMS_MAXIMUM,
                "stereo reference must distinguish channel swapping"
            );
        }
        let actual = recorder.pcm.lock().unwrap();
        if case == "root_warm_compression" {
            // Independently crop the full warm response, then contrast with a
            // source cropped BEFORE compression. Equal outputs would mean this
            // fixture cannot detect a cold-start range implementation.
            let full = reference_pcm(
                &tools,
                &root.path().join("independent-tone.wav"),
                &format!(
                    "{source};[mix]aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000[out]"
                ),
            );
            let crop = &full[options.start_ms as usize * 48 * 2..options.end_ms as usize * 48 * 2];
            assert_eq!(crop, expected.as_slice());
            let cold = reference_pcm(
                &tools,
                &root.path().join("independent-tone.wav"),
                "[0:a]atrim=start_sample=24000:end_sample=43200,asetpts=PTS-STARTPTS,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,acompressor=threshold=0.125893:ratio=4:attack=100:release=300:makeup=1:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1,aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo,aresample=48000[out]",
            );
            assert_eq!(cold.len(), expected.len());
            let cold_rms = (cold
                .iter()
                .zip(&expected)
                .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
                .sum::<f64>()
                / cold.len() as f64)
                .sqrt();
            assert!(
                cold_rms > PCM_RMS_MAXIMUM,
                "warm fixture must distinguish a cold range: {cold_rms}"
            );
        }
        assert_eq!(
            actual.len(),
            expected.len(),
            "{case} exact reference frames"
        );
        let rms = (actual
            .iter()
            .zip(&expected)
            .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
            .sum::<f64>()
            / actual.len() as f64)
            .sqrt();
        if rms > PCM_RMS_MAXIMUM {
            let retained = root.keep();
            panic!(
                "{case} independent original PCM RMS{rms}; retained {}",
                retained.display()
            );
        }
        assert!(rms <= PCM_RMS_MAXIMUM);
        let peak = expected
            .iter()
            .map(|v| f64::from(v.abs()))
            .fold(0.0_f64, f64::max);
        assert!((result.summary.linear_sample_peak - peak).abs() <= PCM_RMS_MAXIMUM);
        for bin in &document.bins {
            for (channel, stats) in [(0, bin.left), (1, bin.right)] {
                let samples = (bin.start_frame..bin.end_frame)
                    .map(|frame| f64::from(expected[frame as usize * 2 + channel]))
                    .collect::<Vec<_>>();
                assert!(
                    (stats.min - samples.iter().copied().fold(f64::INFINITY, f64::min)).abs()
                        <= PCM_RMS_MAXIMUM
                );
                assert!(
                    (stats.max - samples.iter().copied().fold(f64::NEG_INFINITY, f64::max)).abs()
                        <= PCM_RMS_MAXIMUM
                );
                let expected_rms =
                    (samples.iter().map(|s| s * s).sum::<f64>() / samples.len() as f64).sqrt();
                assert!((stats.rms - expected_rms).abs() <= PCM_RMS_MAXIMUM);
            }
        }
        let metrics = reference_metrics(
            &tools,
            &expected,
            &root.path().join("independent-original.pcm"),
        );
        near_nullable(result.summary.integrated_lufs, metrics.integrated_lufs);
        near_nullable(result.summary.true_peak_dbtp, metrics.true_peak_dbtp);
        assert!((result.summary.loudness_range_lu - metrics.loudness_range_lu).abs() <= 0.05);
        assert!((result.summary.threshold_lufs - metrics.threshold_lufs).abs() <= 0.05);
        if case == "silence" {
            assert_eq!(peak, 0.0);
            assert!(result.summary.sample_peak_dbfs.is_none());
            assert!(result.summary.integrated_lufs.is_none());
            assert!(result.summary.true_peak_dbtp.is_none());
        }
        if case == "short_audible" {
            assert!(result.summary.integrated_lufs.is_none());
            assert!(result.summary.true_peak_dbtp.is_some());
            assert!(peak > 0.0);
        }
        if case == "above_full_scale" {
            assert!(peak > 1.0);
        }
        assert!(!fs::read_dir(&dir).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".opencut-")
        }));
    }
}
