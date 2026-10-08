//! Independent fixed narration-envelope references, never generated from plans.
use super::*;
use serde_json::json;

fn fixture() -> Project {
    let mut p = fixture_project();
    p.schema_version = PROJECT_SCHEMA_VERSION;
    p.audio_buses = crate::default_audio_buses();
    p.tracks[0].items.clear();
    p.tracks[1].audio_bus_id = Some("music".into());
    let mut narration = p.tracks[1].clone();
    narration.id = "narration-track".into();
    narration.audio_role = AudioTrackRole::Voiceover;
    narration.audio_bus_id = Some("voiceover".into());
    if let TimelineItem::Media(item) = &mut narration.items[0] {
        item.id = "narration".into();
        item.start_ms = 200;
        item.duration_ms = 200;
        item.audio.volume = 0.2;
    }
    p.tracks.push(narration);
    p.audio_buses[1].ducking = Some(crate::AudioBusDucking {
        enabled: true,
        source_bus_id: "voiceover".into(),
        gain: 0.25,
        attack_ms: 100,
        release_ms: 200,
    });
    p
}

// A hand-authored continuous envelope: attack .1→.2, hold .2→.4,
// release .4→.6. This deliberately does not inspect evaluated facts/filters.
const BASIC: &str = "if(lt(t,0.1),1,if(lt(t,0.2),1-0.75*(t-0.1)/0.1,if(lt(t,0.4),0.25,if(lt(t,0.6),0.25+0.75*(t-0.4)/0.2,1))))";

#[test]
fn native_narration_bus_ducking_attack_release_routing_and_component_conformance() {
    verify_native_cases(&[
        "attack_release",
        "overlapping_ramps",
        "zero_times",
        "clipped_attack",
        "role_fallback",
        "upstream_source",
        "event_source",
        "component_source",
        "component_source_clipped",
        "dsp_order",
        "legacy_composition",
        "silent_source",
        "decoded_silence",
    ]);
}

#[test]
fn native_narration_bus_ducking_clipped_component_conformance() {
    verify_native_cases(&["component_source_clipped"]);
}

fn verify_native_cases(cases: &[&str]) {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    for &case in cases {
        let root = tempdir().unwrap();
        let mut p = fixture();
        let core = crate::EditorCore::new(
            crate::PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        p.id = core
            .create_project("Narration bus ducking", p.settings.clone())
            .unwrap()
            .project_id;
        p.revision = 0;
        let dir = core.paths().project_dir(&p.id).unwrap();
        write_tone_wav(&dir.join("assets/tone.wav"));
        write_tone_wav(&root.path().join("independent-tone.wav"));
        let mut envelope = BASIC.to_owned();
        let mut source =
            "atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.2,adelay=200:all=1".to_owned();
        let mut source_extra = None;
        let mut before = "anull";
        let mut after = "anull";
        match case {
            "overlapping_ramps" => {
                let mut second = p.tracks[2].items[0].clone();
                if let TimelineItem::Media(item) = &mut second {
                    item.id = "narration-second".into();
                    item.start_ms = 500;
                    item.duration_ms = 100;
                    item.visual_properties.stack_order = 1;
                }
                p.tracks[2].items.push(second);
                envelope = format!(
                    "min({BASIC},if(lt(t,0.4),1,if(lt(t,0.5),1-0.75*(t-0.4)/0.1,if(lt(t,0.6),0.25,if(lt(t,0.8),0.25+0.75*(t-0.6)/0.2,1)))))"
                );
                source_extra =
                    Some("atrim=duration=0.1,asetpts=PTS-STARTPTS,volume=0.2,adelay=500:all=1");
            }
            "zero_times" => {
                let d = p.audio_buses[1].ducking.as_mut().unwrap();
                d.attack_ms = 0;
                d.release_ms = 0;
                envelope = "if(gte(t,0.2)*lt(t,0.4),0.25,1)".into();
            }
            "clipped_attack" => {
                if let TimelineItem::Media(item) = &mut p.tracks[2].items[0] {
                    item.start_ms = 50;
                    item.duration_ms = 50;
                }
                source =
                    "atrim=duration=0.05,asetpts=PTS-STARTPTS,volume=0.2,adelay=50:all=1".into();
                envelope="if(lt(t,0.05),1-0.75*(t+0.05)/0.1,if(lt(t,0.1),0.25,if(lt(t,0.3),0.25+0.75*(t-0.1)/0.2,1)))".into();
            }
            "role_fallback" => {
                p.tracks[1].audio_bus_id = None;
                p.tracks[1].audio_role = AudioTrackRole::Music;
                p.tracks[2].audio_bus_id = None;
            }
            "upstream_source" => {
                p.tracks[2].audio_bus_id = Some("sfx".into());
                p.audio_buses[2].output_bus_id = Some("voiceover".into());
            }
            "event_source" => {
                p.tracks[2].audio_bus_id = Some("sfx".into());
                let hash = crate::ContentHash {
                    algorithm: "sha256".into(),
                    digest: hash_bytes(&fs::read(dir.join("assets/tone.wav")).unwrap()),
                };
                p.assets[0].content_hash = Some(hash.clone());
                p.assets[0].size_bytes =
                    Some(fs::metadata(dir.join("assets/tone.wav")).unwrap().len());
                p.sound_definitions = vec![crate::SoundEventDefinition {
                    event: "narration-cue".into(),
                    variant_asset_ids: vec!["tone".into()],
                    default_gain_db: 0.0,
                    bus_id: "voiceover".into(),
                    variant_seed: 0,
                }];
                if let TimelineItem::Media(item) = &mut p.tracks[2].items[0] {
                    item.audio_event = Some(crate::AudioEventItem {
                        event: "narration-cue".into(),
                        gain_db: 0.0,
                        default_gain_db: 0.0,
                        bus_id: "voiceover".into(),
                        variant_seed: 0,
                        variant_index: 0,
                        content_hash: hash,
                    });
                }
            }
            "component_source" | "component_source_clipped" => {
                p.components = vec![crate::ComponentDefinition {
                    id: "narration_component".into(),
                    name: "Narration component".into(),
                    width: WIDTH,
                    height: HEIGHT,
                    duration_ms: 1000,
                    tracks: vec![p.tracks[2].clone()],
                    slots: vec![],
                    markers: vec![],
                }];
                p.tracks[2].items.clear();
                p.tracks[0].items=serde_json::from_value(json!([{"type":"component_instance","id":"narration-instance","componentId":"narration_component","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":0.5,"slotValues":{},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"zIndex":0,"stackOrder":0}])).unwrap();
                source="atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.2,atempo=0.5,adelay=400:all=1".into();
                envelope="if(lt(t,0.3),1,if(lt(t,0.4),1-0.75*(t-0.3)/0.1,if(lt(t,0.8),0.25,if(lt(t,1),0.25+0.75*(t-0.8)/0.2,1))))".into();
                if case == "component_source_clipped" {
                    let TimelineItem::ComponentInstance(instance) = &mut p.tracks[0].items[0]
                    else {
                        unreachable!()
                    };
                    instance.start_ms = 125;
                    instance.trim_start_ms = 324;
                    instance.duration_ms = 400;
                    instance.time_scale = 1.5;
                    // Preserve full-item tempo context, then select the clip:
                    // (324-200)/1.5*48=3968 through 200/1.5*48=6400 samples.
                    source="atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0.2,atempo=1.5,atrim=start_sample=3968:end_sample=6400,asetpts=PTS-STARTPTS,adelay=125:all=1".into();
                    envelope="if(lt(t,0.025),1,if(lt(t,0.125),1-0.75*(t-0.025)/0.1,if(lt(t,0.17566666666666667),0.25,if(lt(t,0.37566666666666667),0.25+0.75*(t-0.17566666666666667)/0.2,1))))".into();
                }
            }
            "dsp_order" => {
                p.audio_buses[1].dsp=Some(serde_json::from_value(json!({"gainDb":-6,"pan":0.5,"eq":[{"frequencyHz":440,"q":1,"gainDb":6}],"compressor":{"thresholdDb":-18,"ratio":3,"attackMs":5,"releaseMs":80,"makeupGainDb":2}})).unwrap());
                before = "volume=0.501187,equalizer=f=440:t=q:w=1:g=6,acompressor=threshold=0.125893:ratio=3:attack=5:release=80:makeup=1.258925:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1";
                after = "pan=stereo|c0=0.5*c0|c1=c1";
            }
            "legacy_composition" => {
                p.tracks[1].audio_role = AudioTrackRole::Music;
                p.tracks[1].ducking = Some(crate::DuckingSettings {
                    enabled: true,
                    gain: 0.5,
                    attack_ms: 0,
                    release_ms: 0,
                });
                // Retain the original inclusive legacy interval semantics.
                before = "volume='if(between(t,0.2,0.4),0.5,1)':eval=frame";
            }
            "silent_source" => {
                if let TimelineItem::Media(item) = &mut p.tracks[2].items[0] {
                    item.audio.volume = 0.0;
                }
                source = "atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0,adelay=200:all=1".into();
                envelope = "1".into();
            }
            "decoded_silence" => {
                let mut bytes = fs::read(dir.join("assets/tone.wav")).unwrap();
                bytes[44..].fill(0);
                fs::write(dir.join("assets/silent.wav"), bytes).unwrap();
                let mut asset = p.assets[0].clone();
                asset.id = "decoded-silence".into();
                asset.project_relative_path = "assets/silent.wav".into();
                p.assets.push(asset);
                if let TimelineItem::Media(item) = &mut p.tracks[2].items[0] {
                    item.asset_id = "decoded-silence".into();
                }
                // Positive authored gain still triggers BASIC for silent samples.
                source = "atrim=duration=0.2,asetpts=PTS-STARTPTS,volume=0,adelay=200:all=1".into();
            }
            "attack_release" => {}
            _ => unreachable!(),
        }
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec_pretty(&p).unwrap(),
        )
        .unwrap();
        let saved = serde_json::to_value(&p.audio_buses[1].ducking).unwrap();
        let draft=core.create_draft(&p.id,0,vec![serde_json::from_value(json!({"operation":"audio_bus_set_ducking","busId":"music","ducking":saved})).unwrap()],None).unwrap();
        let materialized = core.get_draft_state(&p.id, &draft.id).unwrap().project;
        p = core.get_project(&p.id).unwrap();
        assert_eq!(
            fs::read(dir.join(&p.assets[0].project_relative_path)).unwrap(),
            fs::read(root.path().join("independent-tone.wav")).unwrap(),
            "{case} source bytes"
        );
        assert_eq!(
            evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap().scene,
            evaluate_project(&materialized, WIDTH, HEIGHT, FPS)
                .unwrap()
                .scene,
            "{case} draft scene"
        );
        let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
        if case == "component_source_clipped" {
            let evaluated = evaluate_project(&materialized, WIDTH, HEIGHT, FPS).unwrap();
            let requests = media_input_requests(&evaluated).unwrap();
            let paths = requests
                .iter()
                .map(|request| request.project_relative_path.clone())
                .collect();
            let plan = build_render_plan(
                &evaluated.scene,
                &HashMap::new(),
                requests,
                paths,
                None,
                RenderIntent::Export,
                &mut vec![],
            )
            .unwrap();
            fs::write(
                root.path().join("evaluated-scene-and-plan.txt"),
                format!("{:#?}\n{:#?}", evaluated.scene, plan),
            )
            .unwrap();
        }
        let preview = renderer.render_preview(&materialized, &dir, 500).unwrap();
        let range = renderer
            .render_preview_range(
                &materialized,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: WIDTH,
                    height: HEIGHT,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let output = root.path().join("export.mp4");
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &output,
                    width: WIDTH,
                    height: HEIGHT,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let reference = root.path().join("reference.m4a");
        let split = if source_extra.is_some() {
            "asplit=3[m][v][w]"
        } else {
            "asplit=2[m][v]"
        };
        let extra = source_extra.map_or(String::new(), |filters| format!("[w]{filters}[voice2];"));
        let voices = if source_extra.is_some() {
            "[voice][voice2]"
        } else {
            "[voice]"
        };
        let count = if source_extra.is_some() { 4 } else { 3 };
        let fixed = format!(
            "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,{split};[m]{before},volume='{envelope}':eval=frame,{after}[music];[v]{source}[voice];{extra}[1:a][music]{voices}amix=inputs={count}:duration=longest:normalize=0[out]"
        );
        let result = Command::new(&tools.ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(root.path().join("independent-tone.wav"))
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=stereo:d=1",
                "-filter_complex",
                &fixed,
                "-map",
                "[out]",
                "-c:a",
                "aac",
                "-t",
                "1",
            ])
            .arg(&reference)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual = decode_mono_f32(&tools.ffmpeg, &dir.join(&range.relative_path));
        let expected = decode_mono_f32(&tools.ffmpeg, &reference);
        let exported = decode_mono_f32(&tools.ffmpeg, &output);
        let rms = aligned_rms_error(&actual, &expected, 1024).unwrap();
        if rms.is_nan() || rms > PCM_RMS_MAXIMUM {
            let retained = root.keep();
            panic!(
                "{case} independent PCM RMS {rms}; retained {}",
                retained.display()
            );
        }
        assert!(rms <= PCM_RMS_MAXIMUM, "{case} independent PCM RMS {rms}");
        assert!(
            aligned_rms_error(&actual, &exported, 1024).unwrap() <= PCM_RMS_MAXIMUM,
            "{case} range/export PCM"
        );
        let actual_channels =
            audio_bus_dsp::stereo_channels(&tools.ffmpeg, &dir.join(&range.relative_path));
        let expected_channels = audio_bus_dsp::stereo_channels(&tools.ffmpeg, &reference);
        for channel in 0..2 {
            assert!(
                aligned_rms_error(&actual_channels[channel], &expected_channels[channel], 1024)
                    .unwrap()
                    <= PCM_RMS_MAXIMUM,
                "{case} stereo {channel}"
            );
        }
        let preview_rgb =
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(preview.relative_path), 0);
        let exported_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &output, 500);
        assert!(
            structural_similarity(&preview_rgb, &exported_rgb).unwrap() >= SSIM_MINIMUM,
            "{case} RGB"
        );
        assert!(
            actual.iter().any(|v| v.abs() > 0.01),
            "{case} cannot drop audio"
        );
        assert!(
            renderer
                .probe(&output)
                .unwrap()
                .duration_ms
                .unwrap()
                .abs_diff(1000)
                <= 100,
            "{case} timing"
        );
    }
}

#[test]
fn inactive_controls_preserve_exact_previous_scene_debug_and_plan() {
    for case in [
        "absent",
        "disabled",
        "identity",
        "no_source",
        "zero_source",
        "muted_source",
        "hidden_source",
        "zero_target",
        "with_dsp",
    ] {
        let mut original = fixture();
        original.audio_buses[1].ducking = None;
        if case == "no_source" {
            original.tracks[2].items.clear();
        }
        if case == "zero_source"
            && let TimelineItem::Media(item) = &mut original.tracks[2].items[0]
        {
            item.audio.volume = 0.0;
        }
        if case == "muted_source" {
            original.tracks[2].muted = true;
        }
        if case == "hidden_source" {
            original.tracks[2].hidden = true;
        }
        if case == "zero_target"
            && let TimelineItem::Media(item) = &mut original.tracks[1].items[0]
        {
            item.audio.volume = 0.0;
        }
        if case == "with_dsp" {
            original.audio_buses[1].dsp = Some(
                serde_json::from_value(json!({"gainDb":-6,"pan":0,"eq":[],"compressor":null}))
                    .unwrap(),
            );
        }
        let mut changed = original.clone();
        let mut controls = fixture().audio_buses[1].ducking.take().unwrap();
        if matches!(case, "disabled" | "with_dsp") {
            controls.enabled = false;
        }
        if case == "identity" {
            controls.gain = 1.0;
        }
        if case != "absent" {
            changed.audio_buses[1].ducking = Some(controls);
        }
        let a = evaluate_project(&original, WIDTH, HEIGHT, FPS).unwrap();
        let b = evaluate_project(&changed, WIDTH, HEIGHT, FPS).unwrap();
        assert_eq!(a.scene, b.scene, "{case}");
        assert_eq!(
            format!("{:?}", a.scene),
            format!("{:?}", b.scene),
            "{case} Debug"
        );
        let plan = |evaluated: &EvaluatedSceneResult| {
            let requests = media_input_requests(evaluated).unwrap();
            let paths = requests
                .iter()
                .map(|request| request.project_relative_path.clone())
                .collect();
            build_render_plan(
                &evaluated.scene,
                &HashMap::new(),
                requests,
                paths,
                None,
                RenderIntent::Export,
                &mut vec![],
            )
            .unwrap()
        };
        assert_eq!(plan(&a), plan(&b), "{case} plan");
    }
}

#[test]
fn conservative_activity_ignores_automation_fades_and_preprocessor_control_cycles() {
    let original = fixture();
    let baseline = evaluate_project(&original, WIDTH, HEIGHT, FPS).unwrap();
    let mut changed = original.clone();
    let TimelineItem::Media(source) = &mut changed.tracks[2].items[0] else {
        unreachable!()
    };
    source.audio.fade_in_ms = 200;
    source.audio.fade_out_ms = 200;
    source.keyframes = vec![crate::Keyframe {
        property: crate::KeyframeProperty::Volume,
        time_ms: 0,
        value: crate::KeyframeValue::Scalar { value: 0.0 },
        easing: crate::Easing::Linear,
    }];
    let automated = evaluate_project(&changed, WIDTH, HEIGHT, FPS).unwrap();
    assert_eq!(
        baseline.scene.audio_bus_graph,
        automated.scene.audio_bus_graph
    );

    // Control references use preprocessor activity and never become signal edges.
    changed.audio_buses[0].ducking = Some(crate::AudioBusDucking {
        source_bus_id: "music".into(),
        ..original.audio_buses[1].ducking.clone().unwrap()
    });
    changed.validate_audio_bus_model().unwrap();
    let mutual = evaluate_project(&changed, WIDTH, HEIGHT, FPS).unwrap();
    assert_eq!(
        mutual.scene.audio_bus_graph.as_ref().unwrap().buses.len(),
        3
    );
    let master = mutual
        .scene
        .audio_bus_graph
        .as_ref()
        .unwrap()
        .buses
        .last()
        .unwrap();
    assert_eq!(master.index, 3);
    assert!(master.ducking.is_none());
    changed.audio_buses[3].ducking = original.audio_buses[1].ducking.clone();
    let shared = evaluate_project(&changed, WIDTH, HEIGHT, FPS).unwrap();
    let buses = &shared.scene.audio_bus_graph.as_ref().unwrap().buses;
    assert_eq!(buses.iter().filter(|bus| bus.index == 3).count(), 1);
    assert!(buses.last().unwrap().ducking.is_some());
}
