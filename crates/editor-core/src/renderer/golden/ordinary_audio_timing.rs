//! Literal authored clocks and frequency witnesses, independent of render plans.
use super::*;
use serde_json::json;

fn source(path: &Path) {
    let samples: Vec<i16> = (0..48_000)
        .map(|n| {
            let frequency = if n < 24_000 { 440.0 } else { 880.0 };
            (0.2 * i16::MAX as f64
                * (std::f64::consts::TAU * frequency * n as f64 / 48_000.0).sin())
                as i16
        })
        .collect();
    let bytes = (samples.len() * 2) as u32;
    let mut wav = Vec::new();
    wav.extend(b"RIFF");
    wav.extend((36 + bytes).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16_u32.to_le_bytes());
    wav.extend(1_u16.to_le_bytes());
    wav.extend(1_u16.to_le_bytes());
    wav.extend(48_000_u32.to_le_bytes());
    wav.extend(96_000_u32.to_le_bytes());
    wav.extend(2_u16.to_le_bytes());
    wav.extend(16_u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend(bytes.to_le_bytes());
    for sample in samples {
        wav.extend(sample.to_le_bytes());
    }
    fs::write(path, wav).unwrap();
}

fn rms(pcm: &[f32], start: usize, end: usize) -> f64 {
    let window = &pcm[start * 48..end * 48];
    (window.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / window.len() as f64).sqrt()
}

fn witness(pcm: &[f32], start: usize, end: usize, frequency: f64) -> f64 {
    let window = &pcm[start * 48..end * 48];
    let (mut sin, mut cos) = (0.0, 0.0);
    for (n, sample) in window.iter().enumerate() {
        let phase = std::f64::consts::TAU * frequency * n as f64 / 48_000.0;
        sin += f64::from(*sample) * phase.sin();
        cos += f64::from(*sample) * phase.cos();
    }
    2.0 * sin.hypot(cos) / window.len() as f64
}

fn verify(pcm: &[f32], offset: usize, multiple: bool, case: &str) {
    if !multiple {
        let onset = pcm
            .iter()
            .position(|sample| sample.abs() > 0.02)
            .expect("audible onset");
        let expected = 2000_usize.saturating_sub(offset) * 48;
        assert!(
            onset.abs_diff(expected) <= 4800,
            "{case}: onset {onset} vs {expected}, beyond one frame"
        );
        if offset == 0 {
            assert!(rms(pcm, 100, 300) < 0.0001, "{case}: premature audio");
        }
        assert!(
            witness(pcm, 2100 - offset, 2300 - offset, 440.0) > 0.04,
            "{case}: missing delayed tone"
        );
        return;
    }
    if offset <= 525 {
        assert!(
            witness(pcm, 525 - offset, 575 - offset, 440.0) < 0.005,
            "{case}: wrong source trim"
        );
    }
    for (start, end) in [(300, 400), (850, 950), (1500, 1700), (1800, 1900)] {
        if start >= offset {
            assert!(
                rms(pcm, start - offset, end - offset) < 0.0001,
                "{case}: gap {start}"
            );
        }
    }
    for (start, end, frequency) in [
        (50, 150, 440.0),
        (525, 575, 880.0),
        (625, 675, 440.0),
        (625, 675, 880.0),
        (725, 775, 440.0),
        (1250, 1350, 440.0),
        (2100, 2300, 880.0),
    ] {
        if start >= offset {
            assert!(
                witness(pcm, start - offset, end - offset, frequency) > 0.04,
                "{case}: {frequency}Hz at {start}"
            );
        }
    }
    // Source trim selects the880Hz half; fade/volume automation remains local.
    if offset <= 1025 {
        let early = witness(pcm, 1025 - offset, 1075 - offset, 880.0);
        let late = witness(pcm, 1125 - offset, 1175 - offset, 880.0);
        assert!(
            early > 0.005 && late > early * 1.5,
            "{case}: local fade/automation {early}/{late}"
        );
    }
}

#[test]
fn native_ordinary_audio_timing_dsp_routes_gaps_overlap_trim_and_preview_crops() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    verify_native(&tools);
}

pub(super) fn verify_native(tools: &NativeTools) {
    let mut baselines = HashMap::new();
    for mode in ["absent", "identity", "gain", "master", "unreachable"] {
        for multiple in [false, true] {
            let root = tempdir().unwrap();
            let mut p = fixture_project();
            p.schema_version = PROJECT_SCHEMA_VERSION;
            p.audio_buses = crate::default_audio_buses();
            p.tracks[0].items.clear();
            p.tracks[1].audio_role = AudioTrackRole::Voiceover;
            p.tracks[1].audio_bus_id = if mode == "absent" {
                None
            } else {
                Some("voiceover".into())
            };
            let template = p.tracks[1].items[0].clone();
            p.tracks[1].items.clear();
            let clips = if multiple {
                vec![
                    (0, 200, 0),
                    (500, 200, 500),
                    (600, 200, 0),
                    (1000, 200, 500),
                    (1200, 200, 0),
                    (2000, 500, 500),
                ]
            } else {
                vec![(2000, 500, 0)]
            };
            for (index, (start, duration, trim)) in clips.into_iter().enumerate() {
                let mut item = template.clone();
                if let TimelineItem::Media(m) = &mut item {
                    m.id = format!("tone-{index}");
                    m.start_ms = start;
                    m.duration_ms = duration;
                    m.source_in_ms = trim;
                    m.visual_properties.stack_order = index as u32;
                    if multiple && start == 1000 {
                        m.audio.fade_in_ms = 100;
                        m.audio.volume = 0.5;
                        m.keyframes = vec![
                            Keyframe {
                                property: KeyframeProperty::Volume,
                                time_ms: 0,
                                value: KeyframeValue::Scalar { value: 0.5 },
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                property: KeyframeProperty::Volume,
                                time_ms: 200,
                                value: KeyframeValue::Scalar { value: 1.0 },
                                easing: Easing::Linear,
                            },
                        ];
                    }
                }
                p.tracks[1].items.push(item);
            }
            if mode != "absent" {
                let bus = match mode {
                    "master" => 3,
                    "unreachable" => 2,
                    _ => 0,
                };
                p.audio_buses[bus].dsp=Some(serde_json::from_value(json!({"gainDb":if mode=="identity" {0}else{-1},"pan":0,"eq":[],"compressor":null})).unwrap());
            }
            let directory = root.path().join("project");
            fs::create_dir_all(directory.join("assets")).unwrap();
            fs::create_dir_all(directory.join("previews")).unwrap();
            source(&directory.join("assets/tone.wav"));
            let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
            let output = root.path().join("export.mp4");
            renderer
                .export_video(
                    &p,
                    &directory,
                    ExportOptions {
                        output: &output,
                        width: WIDTH,
                        height: HEIGHT,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            let pcm = decode_mono_f32(&tools.ffmpeg, &output);
            verify(&pcm, 0, multiple, mode);
            if mode == "absent" {
                baselines.insert(multiple, pcm.clone());
            } else if mode == "identity" || mode == "unreachable" {
                assert!(
                    aligned_rms_error(&baselines[&multiple], &pcm, 0).unwrap() <= PCM_RMS_MAXIMUM,
                    "{mode}: neutral output drift"
                );
            } else {
                let frequency = if multiple { 880.0 } else { 440.0 };
                let before = witness(&baselines[&multiple], 2100, 2300, frequency);
                let after = witness(&pcm, 2100, 2300, frequency);
                assert!(
                    (after / before - 10_f64.powf(-1.0 / 20.0)).abs() < 0.005,
                    "{mode}: intended gain ratio"
                );
            }
            for start in [0, 900, 2050] {
                let range = renderer
                    .render_preview_range(
                        &p,
                        &directory,
                        PreviewRangeOptions {
                            start_ms: start,
                            end_ms: 2500,
                            width: WIDTH,
                            height: HEIGHT,
                            fps: FPS,
                            include_audio: true,
                        },
                        |_| {},
                    )
                    .unwrap();
                let path = directory.join(&range.relative_path);
                let preview = decode_mono_f32(&tools.ffmpeg, &path);
                verify(&preview, start as usize, multiple, mode);
                if start == 0 {
                    assert!(
                        aligned_rms_error(&pcm, &preview, 0).unwrap() <= PCM_RMS_MAXIMUM,
                        "{mode}: preview/export PCM"
                    );
                }
                fs::remove_file(path).unwrap();
            }
        }
    }
    verify_global_role_ducking(tools);
}

fn verify_global_role_ducking(tools: &NativeTools) {
    for gain in [None, Some(0), Some(-1)] {
        let root = tempdir().unwrap();
        let mut p = fixture_project();
        p.schema_version = PROJECT_SCHEMA_VERSION;
        p.audio_buses = crate::default_audio_buses();
        p.tracks[0].items.clear();
        p.tracks[1].audio_role = AudioTrackRole::Music;
        p.tracks[1].ducking = Some(crate::DuckingSettings {
            enabled: true,
            gain: 0.25,
            attack_ms: 0,
            release_ms: 0,
        });
        if let TimelineItem::Media(m) = &mut p.tracks[1].items[0] {
            m.start_ms = 1000;
        }
        let mut voice = p.tracks[1].clone();
        voice.id = "voice-track".into();
        voice.audio_role = AudioTrackRole::Voiceover;
        voice.ducking = None;
        if let TimelineItem::Media(m) = &mut voice.items[0] {
            m.id = "voice".into();
            m.start_ms = 1500;
            m.duration_ms = 200;
        }
        p.tracks.push(voice);
        if let Some(gain_db) = gain {
            p.audio_buses[0].dsp = Some(
                serde_json::from_value(json!({"gainDb":gain_db,"pan":0,"eq":[],"compressor":null}))
                    .unwrap(),
            );
        }
        let directory = root.path().join("project");
        fs::create_dir_all(directory.join("assets")).unwrap();
        source(&directory.join("assets/tone.wav"));
        let output = root.path().join("ducking.mp4");
        Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()))
            .export_video(
                &p,
                &directory,
                ExportOptions {
                    output: &output,
                    width: WIDTH,
                    height: HEIGHT,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let pcm = decode_mono_f32(&tools.ffmpeg, &output);
        assert!(
            rms(&pcm, 100, 300) < 0.0001,
            "{gain:?}: premature routed content"
        );
        let ducked = witness(&pcm, 1550, 1650, 880.0);
        let released = witness(&pcm, 1800, 1900, 880.0);
        assert!(
            (ducked / released - 0.25).abs() < 0.02,
            "{gain:?}: global ducking ratio {ducked}/{released}"
        );
        assert!(
            witness(&pcm, 1550, 1650, 440.0) > 0.04,
            "{gain:?}: narration activity"
        );
    }
}
