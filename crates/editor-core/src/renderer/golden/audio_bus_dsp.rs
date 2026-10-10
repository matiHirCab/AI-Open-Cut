//! Independent fixed-filter references. Never derives an oracle from a plan.
use super::*;
use serde_json::json;

fn identity() -> serde_json::Value {
    json!({"gainDb":0,"pan":0,"eq":[],"compressor":null})
}
fn fixture() -> Project {
    let mut p = fixture_project();
    p.schema_version = PROJECT_SCHEMA_VERSION;
    p.audio_buses = crate::default_audio_buses();
    p.tracks[0].items.clear();
    p.tracks[1].audio_bus_id = Some("music".into());
    p
}
fn set(p: &mut Project, index: usize, value: serde_json::Value) {
    p.audio_buses[index].dsp = Some(serde_json::from_value(value).unwrap());
}

pub(super) fn stereo_channels(ffmpeg: &Path, path: &Path) -> [Vec<f32>; 2] {
    let output = Command::new(ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-map",
            "0:a:0",
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "-ar",
            "48000",
            "-ac",
            "2",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "stereo PCM decode");
    assert_eq!(output.stdout.len() % 8, 0);
    let mut channels: [Vec<f32>; 2] =
        std::array::from_fn(|_| Vec::with_capacity(output.stdout.len() / 8));
    for frame in output.stdout.as_chunks::<8>().0 {
        for index in 0..2 {
            channels[index].push(f32::from_le_bytes(
                frame[index * 4..index * 4 + 4].try_into().unwrap(),
            ));
        }
    }
    channels
}

#[test]
fn native_audio_bus_dsp_gain_balance_eq_overlap_compression_and_nested_master_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    super::ordinary_audio_timing::verify_native(&tools);
    for case in [
        "gain",
        "role_fallback",
        "balance",
        "balance_positive",
        "eq",
        "chain",
        "compression",
        "nested",
        "master",
        "neutral",
        "unreachable",
        "zero_gain",
        "event_route",
        "component_clock",
    ] {
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
            .create_project("Native bus DSP", p.settings.clone())
            .unwrap()
            .project_id;
        p.revision = 0;
        let dir = core.paths().project_dir(&p.id).unwrap();
        write_tone_wav(&dir.join("assets/tone.wav"));
        write_tone_wav(&root.path().join("independent-tone.wav"));
        let mut dsp = identity();
        let processing = match case {
            "gain" => {
                dsp["gainDb"] = json!(-6);
                "volume=0.501187"
            }
            "role_fallback" => {
                p.tracks[1].audio_bus_id = None;
                p.tracks[1].audio_role = AudioTrackRole::Music;
                dsp["gainDb"] = json!(-6);
                "volume=0.501187"
            }
            "balance" => {
                dsp["pan"] = json!(-0.5);
                "pan=stereo|c0=c0|c1=0.5*c1"
            }
            "balance_positive" => {
                dsp["pan"] = json!(0.5);
                "pan=stereo|c0=0.5*c0|c1=c1"
            }
            "eq" => {
                dsp["eq"] = json!([{"frequencyHz":440,"q":1,"gainDb":-6}]);
                "equalizer=f=440:t=q:w=1:g=-6"
            }
            "chain" => {
                dsp = json!({"gainDb":-6,"pan":0.5,"eq":[{"frequencyHz":440,"q":1,"gainDb":6}],"compressor":{"thresholdDb":-18,"ratio":3,"attackMs":5,"releaseMs":80,"makeupGainDb":2}});
                "volume=0.501187,equalizer=f=440:t=q:w=1:g=6,acompressor=threshold=0.125893:ratio=3:attack=5:release=80:makeup=1.258925:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1,pan=stereo|c0=0.5*c0|c1=c1"
            }
            "compression" => {
                dsp["compressor"] = json!({"thresholdDb":-12,"ratio":4,"attackMs":10,"releaseMs":100,"makeupGainDb":0});
                let mut duplicate = p.tracks[1].items[0].clone();
                if let TimelineItem::Media(m) = &mut duplicate {
                    m.id = "second-tone".into();
                    m.visual_properties.stack_order = 1;
                }
                p.tracks[1].items.push(duplicate);
                "acompressor=threshold=0.251189:ratio=4:attack=10:release=100:makeup=1:mode=downward:detection=peak:link=maximum:knee=1:mix=1:level_in=1"
            }
            "nested" => {
                dsp["gainDb"] = json!(-6);
                p.audio_buses[1].output_bus_id = Some("sfx".into());
                let mut extra = identity();
                extra["gainDb"] = json!(-3);
                set(&mut p, 2, extra);
                "volume=0.501187,volume=0.707946"
            }
            "master" => {
                dsp["gainDb"] = json!(-6);
                set(&mut p, 3, dsp.clone());
                dsp = identity();
                "volume=0.501187"
            }
            "neutral" => "anull",
            "unreachable" => {
                dsp["gainDb"] = json!(-6);
                set(&mut p, 2, dsp.clone());
                dsp = identity();
                "anull"
            }
            "zero_gain" => {
                if let TimelineItem::Media(item) = &mut p.tracks[1].items[0] {
                    item.audio.volume = 0.0;
                }
                dsp["gainDb"] = json!(-6);
                "volume=0"
            }
            "event_route" => {
                p.sound_definitions = vec![crate::SoundEventDefinition {
                    event: "impact".into(),
                    variant_asset_ids: vec!["tone".into()],
                    default_gain_db: 0.0,
                    bus_id: "sfx".into(),
                    variant_seed: 0,
                }];
                let hash = crate::ContentHash {
                    algorithm: "sha256".into(),
                    digest: hash_bytes(&fs::read(dir.join("assets/tone.wav")).unwrap()),
                };
                p.assets[0].content_hash = Some(hash.clone());
                p.assets[0].size_bytes =
                    Some(fs::metadata(dir.join("assets/tone.wav")).unwrap().len());
                if let TimelineItem::Media(m) = &mut p.tracks[1].items[0] {
                    m.audio_event = Some(crate::AudioEventItem {
                        event: "impact".into(),
                        gain_db: 0.0,
                        default_gain_db: 0.0,
                        bus_id: "sfx".into(),
                        variant_seed: 0,
                        variant_index: 0,
                        content_hash: hash,
                    });
                }
                let mut extra = identity();
                extra["gainDb"] = json!(-6);
                set(&mut p, 2, extra);
                "volume=0.501187"
            }
            "component_clock" => {
                dsp["gainDb"] = json!(-6);
                p.components = vec![crate::ComponentDefinition {
                    id: "audio_component".into(),
                    name: "Audio component".into(),
                    width: WIDTH,
                    height: HEIGHT,
                    duration_ms: 1000,
                    tracks: vec![p.tracks[1].clone()],
                    slots: vec![],
                    markers: vec![],
                }];
                p.tracks[1].items.clear();
                p.tracks[0].items=serde_json::from_value(json!([{"type":"component_instance","id":"instance","componentId":"audio_component","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":0.5,"slotValues":{},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"zIndex":0,"stackOrder":0}])).unwrap();
                "atempo=0.5,volume=0.501187"
            }
            _ => unreachable!(),
        };
        set(&mut p, 1, dsp);
        fs::write(
            dir.join("project.json"),
            serde_json::to_vec_pretty(&p).unwrap(),
        )
        .unwrap();
        let retained = serde_json::to_value(&p.audio_buses[1].dsp).unwrap();
        let draft = core
            .create_draft(
                &p.id,
                0,
                vec![
                    serde_json::from_value(
                        json!({"operation":"audio_bus_set_dsp","busId":"music","dsp":retained}),
                    )
                    .unwrap(),
                ],
                None,
            )
            .unwrap();
        let materialized = core.get_draft_state(&p.id, &draft.id).unwrap().project;
        // Draft preparation adopts legacy media into content-addressed storage.
        // Render the actual canonical state, retaining independent source bytes.
        p = core.get_project(&p.id).unwrap();
        assert_eq!(
            fs::read(dir.join(&p.assets[0].project_relative_path)).unwrap(),
            fs::read(root.path().join("independent-tone.wav")).unwrap(),
            "{case} canonical source bytes"
        );
        let evaluated = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
        let draft_evaluated = evaluate_project(&materialized, WIDTH, HEIGHT, FPS).unwrap();
        assert_eq!(
            evaluated.scene, draft_evaluated.scene,
            "{case} draft changes scene"
        );
        let media = prepare_media_resources(&FileSystemArtifactIo, &evaluated, &dir).unwrap();
        build_render_plan(
            &evaluated.scene,
            &HashMap::new(),
            media.media_inputs,
            media.media_paths,
            None,
            RenderIntent::Export,
            &mut vec![],
        )
        .unwrap_or_else(|error| panic!("{case} original plan: {error:?}"));
        let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
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
        let mixed = if case == "compression" {
            format!(
                "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,asplit=2[a][b];[a][b]amix=inputs=2:normalize=0:duration=longest,{processing}[bus];[1:a][bus]amix=inputs=2:duration=longest:normalize=0[out]"
            )
        } else if case == "master" {
            format!(
                "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[a];[1:a][a]amix=inputs=2:duration=longest:normalize=0,{processing}[out]"
            )
        } else if case == "neutral" || case == "unreachable" {
            "[1:a][0:a]amix=inputs=2:duration=longest:normalize=0[out]".to_owned()
        } else {
            format!(
                "[0:a]aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,{processing}[bus];[1:a][bus]amix=inputs=2:duration=longest:normalize=0[out]"
            )
        };
        let result = Command::new(&tools.ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(root.path().join("independent-tone.wav"))
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=stereo:d=1",
                "-filter_complex",
                &mixed,
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
        let expected = decode_mono_f32(&tools.ffmpeg, &reference);
        let actual = decode_mono_f32(&tools.ffmpeg, &dir.join(&range.relative_path));
        let exported = decode_mono_f32(&tools.ffmpeg, &output);
        let error = aligned_rms_error(&actual, &expected, 1024).unwrap();
        assert!(
            error <= PCM_RMS_MAXIMUM,
            "{case} independent PCM RMS {error}"
        );
        assert!(
            aligned_rms_error(&actual, &exported, 1024).unwrap() <= PCM_RMS_MAXIMUM,
            "{case} range/export PCM"
        );
        let reference_channels = stereo_channels(&tools.ffmpeg, &reference);
        let actual_channels = stereo_channels(&tools.ffmpeg, &dir.join(&range.relative_path));
        for channel in 0..2 {
            assert!(
                aligned_rms_error(
                    &actual_channels[channel],
                    &reference_channels[channel],
                    1024
                )
                .unwrap()
                    <= PCM_RMS_MAXIMUM,
                "{case} channel {channel}"
            );
        }
        let preview_rgb =
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(preview.relative_path), 0);
        let exported_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &output, 500);
        assert!(
            structural_similarity(&preview_rgb, &exported_rgb).unwrap() >= SSIM_MINIMUM,
            "{case} RGB"
        );
        if case == "zero_gain" {
            assert!(
                actual.iter().all(|sample| *sample == 0.0),
                "known silent PCM"
            );
            assert!(
                exported.iter().all(|sample| *sample == 0.0),
                "known silent export PCM"
            );
        } else {
            assert!(
                actual.iter().any(|s| s.abs() > 0.01),
                "{case} cannot silently drop audio"
            );
        }
        let duration = renderer.probe(&output).unwrap().duration_ms.unwrap();
        assert!(duration.abs_diff(1000) <= 100, "{case} timing {duration}");
    }
}

#[test]
fn absent_dsp_preserves_historical_direct_render_inputs() {
    let mut historical = fixture();
    historical.schema_version = 18;
    for track in &mut historical.tracks {
        track.audio_bus_id = None;
    }
    let mut baseline = historical.clone();
    baseline.audio_buses.clear();
    let expected = evaluate_project(&baseline, WIDTH, HEIGHT, FPS).unwrap();
    // Existing native affine fixtures retain today's default bus records when
    // selecting their historical unbound-font render path. DSP omission must
    // preserve that direct renderer input rather than add a migration guard.
    let actual = evaluate_project(&historical, WIDTH, HEIGHT, FPS).unwrap();
    assert_eq!(actual.scene, expected.scene);
    assert_eq!(
        format!("{:?}", actual.scene),
        format!("{:?}", expected.scene)
    );
    set(&mut historical, 1, identity());
    let error = evaluate_project(&historical, WIDTH, HEIGHT, FPS).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert_eq!(error.message, "audio bus DSP requires schema 42");
}

#[test]
fn neutral_and_unreachable_dsp_preserve_exact_evaluated_scene_and_plan() {
    let p = fixture();
    let baseline = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
    for (index, dsp) in [
        (1, identity()),
        (2, json!({"gainDb":-6,"pan":0,"eq":[],"compressor":null})),
        (
            1,
            json!({"gainDb":0,"pan":0,"eq":[{"frequencyHz":1000,"q":1,"gainDb":0}],"compressor":null}),
        ),
    ] {
        let mut candidate = p.clone();
        set(&mut candidate, index, dsp);
        let current = evaluate_project(&candidate, WIDTH, HEIGHT, FPS).unwrap();
        assert_eq!(current.scene, baseline.scene);
        assert_eq!(
            format!("{:?}", current.scene),
            format!("{:?}", baseline.scene)
        );
        let plan = |scene: &EvaluatedScene| {
            build_render_plan(
                scene,
                &HashMap::new(),
                vec![crate::render_plan::MediaInputRequest {
                    item_id: "tone-item".into(),
                    asset_id: "tone".into(),
                    project_relative_path: PathBuf::from("assets/tone.wav"),
                    input_index: 2,
                    media_type: MediaType::Audio,
                    source_in_ms: 0,
                    duration_ms: 1000,
                }],
                vec![PathBuf::from("tone.wav")],
                None,
                RenderIntent::Export,
                &mut vec![],
            )
        };
        // The canonical scene equality is the complete planner input; check the
        // public renderer's pure prepare seam separately in readiness tests.
        assert_eq!(
            plan(&current.scene).unwrap(),
            plan(&baseline.scene).unwrap()
        );
    }
}

#[test]
fn zero_item_gain_preserves_legacy_scene_despite_volume_automation_and_bus_dsp() {
    let mut p = fixture();
    if let TimelineItem::Media(item) = &mut p.tracks[1].items[0] {
        item.audio.volume = 0.0;
        item.keyframes = serde_json::from_value(json!([
            {"property":"volume","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"},
            {"property":"volume","timeMs":500,"value":{"type":"scalar","value":2},"easing":"linear"}
        ]))
        .unwrap();
    }
    let baseline = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
    set(
        &mut p,
        1,
        json!({"gainDb":-6,"pan":0,"eq":[],"compressor":null}),
    );
    let current = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
    assert_eq!(current.scene, baseline.scene);
    assert_eq!(
        format!("{:?}", current.scene),
        format!("{:?}", baseline.scene)
    );
    let plan = |scene: &EvaluatedScene| {
        build_render_plan(
            scene,
            &HashMap::new(),
            vec![crate::render_plan::MediaInputRequest {
                item_id: "tone-item".into(),
                asset_id: "tone".into(),
                project_relative_path: PathBuf::from("assets/tone.wav"),
                input_index: 2,
                media_type: MediaType::Audio,
                source_in_ms: 0,
                duration_ms: 1000,
            }],
            vec![PathBuf::from("tone.wav")],
            None,
            RenderIntent::Export,
            &mut vec![],
        )
        .unwrap()
    };
    assert_eq!(plan(&current.scene), plan(&baseline.scene));
}

#[test]
fn audible_upstream_route_activates_dsp_and_keeps_zero_gain_stream_connected() {
    let mut p = fixture();
    let mut audible = p.tracks[1].clone();
    audible.id = "audible-sfx".into();
    audible.audio_bus_id = Some("sfx".into());
    if let TimelineItem::Media(item) = &mut audible.items[0] {
        item.id = "audible-tone".into();
    }
    if let TimelineItem::Media(item) = &mut p.tracks[1].items[0] {
        item.audio.volume = 0.0;
    }
    p.tracks.push(audible);
    set(
        &mut p,
        1,
        json!({"gainDb":-6,"pan":0,"eq":[],"compressor":null}),
    );
    // The audible signal bypasses the changed bus: preserve the legacy graph.
    let bypassed = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
    assert!(bypassed.scene.audio_bus_graph.is_none());
    assert!(
        bypassed
            .scene
            .audio_layers
            .iter()
            .all(|layer| layer.bus_index.is_none())
    );
    p.audio_buses[2].output_bus_id = Some("music".into());
    let routed = evaluate_project(&p, WIDTH, HEIGHT, FPS).unwrap();
    let graph = routed.scene.audio_bus_graph.as_ref().unwrap();
    assert_eq!(
        graph.buses.iter().map(|bus| bus.index).collect::<Vec<_>>(),
        vec![2, 1, 3]
    );
    assert_eq!(routed.scene.audio_layers.len(), 2);
    assert_eq!(routed.scene.audio_layers[0].bus_index, Some(1));
    assert_eq!(routed.scene.audio_layers[1].bus_index, Some(2));
}
