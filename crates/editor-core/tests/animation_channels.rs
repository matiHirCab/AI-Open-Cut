use opencut_editor_core::{
    AnimationChannel, AnimationChannelProperty, AudioTrackRole, BatchEditOperation,
    DuckingSettings, EditOperation, EditorCore, ErrorCode, ExportOptions, MediaProbeFacts,
    MediaType, PROJECT_SCHEMA_VERSION, PathPolicy, PreviewRangeOptions, ProjectSettings, Renderer,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/animation-channels-v1.json"
    ))
    .unwrap()
}

fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let policy = PathPolicy::new(
        root.path().join("projects"),
        [&media],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(policy);
    let project_id = core
        .create_project("Channels", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track_id = core.get_project(&project_id).unwrap().tracks[1].id.clone();
    (root, core, project_id, track_id)
}

fn channel(property: &str, first: f64, last: f64) -> Value {
    json!({"property":property,"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":first},"curve":"linear"},
        {"timeMs":500,"value":{"type":"scalar","value":last},"curve":"hold"}
    ]})
}

#[test]
fn canonical_names_and_limits_match_rust_types() {
    let contract = fixture();
    assert_eq!(contract["projectSchemaVersion"], PROJECT_SCHEMA_VERSION);
    assert_eq!(contract["limits"]["maxChannelsPerItem"], 64);
    assert_eq!(contract["limits"]["maxKeyframesPerChannel"], 1000);
    for category in ["active", "inactive"] {
        for name in contract[category].as_object().unwrap().keys() {
            let property: AnimationChannelProperty = serde_json::from_value(json!(name)).unwrap();
            assert_eq!(serde_json::to_value(property).unwrap(), json!(name));
            let metadata = &contract[category][name];
            if category == "active" {
                let expected = match property {
                    AnimationChannelProperty::PositionX | AnimationChannelProperty::PositionY => {
                        json!({"valueType":"scalar","target":"visual_legacy","activation":"active","minimum":-1_000_000,"maximum":1_000_000})
                    }
                    AnimationChannelProperty::ScaleX | AnimationChannelProperty::ScaleY => {
                        json!({"valueType":"scalar","target":"visual_legacy","activation":"active","minimumExclusive":0,"maximum":100})
                    }
                    AnimationChannelProperty::Opacity => {
                        json!({"valueType":"scalar","target":"visual_legacy","activation":"active","minimum":0,"maximum":1})
                    }
                    AnimationChannelProperty::GainDb => {
                        json!({"valueType":"scalar","target":"media_audio","activation":"active","minimum":-96,"maximum":12})
                    }
                    _ => panic!("catalog activated an unsupported property: {name}"),
                };
                assert_eq!(*metadata, expected, "active catalog metadata for {name}");
            } else {
                let (value_type, target) = match property {
                    AnimationChannelProperty::RotationDeg
                    | AnimationChannelProperty::SkewXDeg
                    | AnimationChannelProperty::SkewYDeg
                    | AnimationChannelProperty::AnchorX
                    | AnimationChannelProperty::AnchorY => ("scalar", "visual"),
                    AnimationChannelProperty::CropX
                    | AnimationChannelProperty::CropY
                    | AnimationChannelProperty::CropWidth
                    | AnimationChannelProperty::CropHeight => ("scalar", "media_visual"),
                    AnimationChannelProperty::SourcePositionMs
                    | AnimationChannelProperty::PlaybackRate => ("scalar", "media_source"),
                    AnimationChannelProperty::PathPoints => ("path_points", "graphic_scoped"),
                    AnimationChannelProperty::GradientStops => ("gradient_stops", "graphic_scoped"),
                    AnimationChannelProperty::FillColor | AnimationChannelProperty::StrokeColor => {
                        ("rgba", "graphic_scoped")
                    }
                    AnimationChannelProperty::PathTrim | AnimationChannelProperty::StrokeWidth => {
                        ("scalar", "graphic_scoped")
                    }
                    AnimationChannelProperty::TintColor => ("rgba", "effect_scoped"),
                    AnimationChannelProperty::BlurRadius
                    | AnimationChannelProperty::GlowRadius
                    | AnimationChannelProperty::VignetteAmount
                    | AnimationChannelProperty::ParticleAmount => ("scalar", "effect_scoped"),
                    AnimationChannelProperty::Pan => ("scalar", "media_audio"),
                    _ => panic!("active property listed as inactive: {name}"),
                };
                assert_eq!(
                    *metadata,
                    json!({"valueType":value_type,"target":target,"activation":"inactive","bounds":"deferred"}),
                    "inactive catalog metadata for {name}"
                );
            }
        }
    }
    assert_eq!(
        contract["active"].as_object().unwrap().len()
            + contract["inactive"].as_object().unwrap().len(),
        29
    );
    assert!(
        serde_json::from_value::<AnimationChannel>(contract["examples"]["validChannel"].clone())
            .is_ok()
    );
    assert!(
        serde_json::from_value::<AnimationChannel>(contract["examples"]["invalidChannel"].clone())
            .is_err()
    );
    assert!(
        serde_json::from_value::<AnimationChannel>(contract["examples"]["inactiveChannel"].clone())
            .is_ok()
    );
}

#[test]
fn alias_batch_history_reopen_and_atomic_failures() {
    let (_root, core, project_id, track_id) = setup();
    let channels = vec![
        channel("transform.position_x", 0.0, 50.0),
        channel("transform.scale_y", 1.0, 2.0),
        channel("transform.opacity", 1.0, 0.5),
    ];
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track_id,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_animation_channels","itemId":"@box","animationChannels":channels}
    ])).unwrap();
    let result = core.edit_batch(&project_id, 0, operations).unwrap();
    assert_eq!(result.revision, 1);
    let item_id = result.aliases["box"].clone();
    let read = || core.get_project(&project_id).unwrap();
    assert_eq!(
        read()
            .find_item(&item_id)
            .unwrap()
            .visual_properties()
            .animation_channels
            .len(),
        3
    );

    let before = serde_json::to_value(read()).unwrap();
    for invalid in [
        vec![channel("transform.scale_x", 0.0, 2.0)],
        vec![channel("transform.rotation_deg", 0.0, 10.0)],
        vec![
            channel("transform.position_x", 0.0, 2.0),
            channel("transform.position_x", 0.0, 3.0),
        ],
        vec![json!({"property":"transform.position_y","keyframes":[
            {"timeMs":500,"value":{"type":"scalar","value":0},"curve":"linear"},
            {"timeMs":500,"value":{"type":"scalar","value":10},"curve":"hold"}
        ]})],
        vec![json!({"property":"transform.position_y","keyframes":[
            {"timeMs":1000,"value":{"type":"scalar","value":0},"curve":"hold"}
        ]})],
        vec![json!({"property":"transform.opacity","keyframes":[
            {"timeMs":0,"value":{"type":"point","x":0,"y":1},"curve":"hold"}
        ]})],
        vec![json!({"property":"transform.opacity","keyframes":[
            {"timeMs":0,"value":{"type":"scalar","value":1.1},"curve":"hold"}
        ]})],
        vec![
            json!({"property":"transform.position_y","target":{"scope":"root","id":"missing"},"keyframes":[
                {"timeMs":0,"value":{"type":"scalar","value":1},"curve":"hold"}
            ]}),
        ],
        vec![channel("transform.opacity", 1.0, 1.0); 65],
        vec![
            json!({"property":"transform.opacity","keyframes":(0..1001).map(|time_ms| {
            json!({"timeMs":time_ms,"value":{"type":"scalar","value":1},"curve":"hold"})
        }).collect::<Vec<_>>()}),
        ],
    ] {
        let error = core.edit(&project_id, 1, operation(json!({
            "operation":"set_animation_channels", "itemId":item_id, "animationChannels":invalid
        }))).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert_eq!(serde_json::to_value(read()).unwrap(), before);
    }
    assert_eq!(
        core.edit(
            &project_id,
            0,
            operation(json!({
                "operation":"set_animation_channels", "itemId":item_id, "animationChannels":[]
            }))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(
            &project_id,
            1,
            operation(json!({
                "operation":"set_animation_channels", "itemId":"missing", "animationChannels":[]
            }))
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );

    core.undo(&project_id, 1).unwrap();
    assert!(read().find_item(&item_id).is_none());
    core.redo(&project_id, 2).unwrap();
    assert_eq!(
        read()
            .find_item(&item_id)
            .unwrap()
            .visual_properties()
            .animation_channels
            .len(),
        3
    );
    // The core instance reopens the durable project on every read.
    assert_eq!(read().schema_version, 22);

    core.edit(
        &project_id,
        3,
        operation(json!({"operation":"update_track","trackId":track_id,"locked":true})),
    )
    .unwrap();
    assert_eq!(
        core.edit(
            &project_id,
            4,
            operation(json!({
                "operation":"set_animation_channels", "itemId":item_id, "animationChannels":[]
            }))
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
}

#[test]
fn later_transform_and_legacy_edits_cannot_conflict_with_channels() {
    let (_root, core, project_id, track_id) = setup();
    let added = core
        .edit(
            &project_id,
            0,
            operation(json!({"operation":"add_rectangle","trackId":track_id,
                "startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})),
        )
        .unwrap();
    let item_id = &added.changed_ids[0];
    let set_channels = |revision, channels: Vec<Value>| {
        core.edit(
            &project_id,
            revision,
            operation(
                json!({"operation":"set_animation_channels","itemId":item_id,
                "animationChannels":channels}),
            ),
        )
    };
    let set_legacy = |revision, keyframes: Vec<Value>| {
        core.edit(
            &project_id,
            revision,
            operation(json!({"operation":"set_keyframes","itemId":item_id,
                "keyframes":keyframes})),
        )
    };
    let legacy = vec![json!({"property":"position","timeMs":0,
        "value":{"type":"position","x":0,"y":0},"easing":"linear"})];
    let typed = vec![channel("transform.position_x", 0.0, 20.0)];
    set_channels(1, typed.clone()).unwrap();
    let before = serde_json::to_value(core.get_project(&project_id).unwrap()).unwrap();
    assert_eq!(
        core.edit(
            &project_id,
            2,
            operation(json!({"operation":"update_item","itemId":item_id,
                "transform2d":opencut_editor_core::Transform2D::default()})),
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        set_legacy(2, legacy.clone()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&project_id).unwrap()).unwrap(),
        before
    );
    set_channels(2, vec![]).unwrap();
    set_legacy(3, legacy).unwrap();
    assert_eq!(
        set_channels(4, typed.clone()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    set_legacy(4, vec![]).unwrap();
    core.edit(
        &project_id,
        5,
        operation(json!({"operation":"update_item","itemId":item_id,
            "transform2d":opencut_editor_core::Transform2D::default()})),
    )
    .unwrap();
    assert_eq!(
        set_channels(6, typed).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn incompatible_audio_target_and_persisted_inactive_channel_fail_before_render() {
    let (_root, core, project_id, track_id) = setup();
    let added = core
        .edit(
            &project_id,
            0,
            operation(json!({"operation":"add_rectangle","trackId":track_id,
                "startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})),
        )
        .unwrap();
    let item_id = &added.changed_ids[0];
    let before = serde_json::to_value(core.get_project(&project_id).unwrap()).unwrap();
    assert_eq!(
        core.edit(
            &project_id,
            1,
            operation(
                json!({"operation":"set_animation_channels","itemId":item_id,
                "animationChannels":[channel("audio.gain_db", -6.0, 0.0)]})
            ),
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&project_id).unwrap()).unwrap(),
        before
    );

    let mut externally_edited = core.get_project(&project_id).unwrap();
    externally_edited.tracks[1].items[0]
        .visual_properties_mut()
        .animation_channels =
        serde_json::from_value(json!([channel("transform.rotation_deg", 0.0, 90.0)])).unwrap();
    let project_dir = core.paths().project_dir(&project_id).unwrap();
    let preview_count = std::fs::read_dir(project_dir.join("previews"))
        .unwrap()
        .count();
    let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
    assert_eq!(
        renderer
            .render_preview(&externally_edited, &project_dir, 0)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        std::fs::read_dir(project_dir.join("previews"))
            .unwrap()
            .count(),
        preview_count
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&project_id).unwrap()).unwrap(),
        before
    );
}

#[test]
fn audio_gain_endpoints_are_inclusive_and_first_outside_values_are_rejected() {
    let (root, core, id, _) = setup();
    let source = root.path().join("media/tone.wav");
    std::fs::write(&source, b"audio fixture").unwrap();
    let asset = core
        .import_asset(
            &id,
            0,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let track_id = core.get_project(&id).unwrap().tracks[2].id.clone();
    let item_id = core
        .edit(
            &id,
            1,
            operation(json!({
                "operation":"add_media","trackId":track_id,"assetId":asset,
                "startMs":0,"sourceInMs":0,"durationMs":1000
            })),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    for (revision, value) in [(2, -96.0), (3, 12.0)] {
        core.edit(
            &id,
            revision,
            operation(json!({
                "operation":"set_animation_channels","itemId":item_id,
                "animationChannels":[channel("audio.gain_db", value, value)]
            })),
        )
        .unwrap();
    }
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for value in [-96.0001, 12.0001] {
        let error = core
            .edit(
                &id,
                4,
                operation(json!({
                    "operation":"set_animation_channels","itemId":item_id,
                    "animationChannels":[channel("audio.gain_db", value, value)]
                })),
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert_eq!(
            serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
}

fn rgb_at(
    ffmpeg: &std::path::Path,
    file: &std::path::Path,
    seek: Option<&str>,
    x: usize,
    y: usize,
) -> [u8; 3] {
    let mut command = std::process::Command::new(ffmpeg);
    command.args(["-v", "error"]);
    if let Some(time) = seek {
        command.args(["-ss", time]);
    }
    let output = command
        .arg("-i")
        .arg(file)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let at = (y * 64 + x) * 3;
    output.stdout[at..at + 3].try_into().unwrap()
}

fn red_at(ffmpeg: &std::path::Path, file: &std::path::Path, seek: Option<&str>, x: usize) -> bool {
    let pixel = rgb_at(ffmpeg, file, seek, x, 5);
    pixel[0] > 180 && pixel[1] < 60 && pixel[2] < 60
}

fn native_tools() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    match (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) {
        (Some(ffmpeg), Some(ffprobe)) => Some((ffmpeg.into(), ffprobe.into())),
        _ => {
            assert_ne!(
                std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
                Ok("1"),
                "native animation-channel rendering requires FFmpeg and FFprobe"
            );
            None
        }
    }
}

#[test]
fn native_each_visual_channel_changes_output_and_static_values_survive() {
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    let (_root, core, project_id, track_id) = setup();
    let added = core
        .edit(
            &project_id,
            0,
            operation(json!({
                "operation":"add_rectangle","trackId":track_id,"startMs":0,"durationMs":1000,
                "width":10,"height":10,"color":"#ff0000",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}
            })),
        )
        .unwrap();
    let _item_id = &added.changed_ids[0];
    let mut project = core.get_project(&project_id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    let dir = core.paths().project_dir(&project_id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let mut render = |property: Option<(&str, f64)>| {
        project.tracks[1].items[0]
            .visual_properties_mut()
            .animation_channels = property
            .map(|(name, value)| {
                serde_json::from_value(json!([channel(name, value, value)])).unwrap()
            })
            .unwrap_or_default();
        let result = renderer.render_preview(&project, &dir, 500).unwrap();
        dir.join(result.relative_path)
    };
    let baseline = render(None);
    assert!(red_at(&ffmpeg, &baseline, None, 5));
    assert!(!red_at(&ffmpeg, &baseline, None, 15));
    let position_x = render(Some(("transform.position_x", 20.0)));
    assert!(red_at(&ffmpeg, &position_x, None, 25));
    assert!(!red_at(&ffmpeg, &position_x, None, 5));
    let position_y = render(Some(("transform.position_y", 20.0)));
    assert!(rgb_at(&ffmpeg, &position_y, None, 5, 25)[0] > 180);
    assert!(rgb_at(&ffmpeg, &position_y, None, 5, 5)[0] < 60);
    let scale_x = render(Some(("transform.scale_x", 2.0)));
    assert!(red_at(&ffmpeg, &scale_x, None, 15));
    assert!(rgb_at(&ffmpeg, &scale_x, None, 5, 15)[0] < 60);
    let scale_y = render(Some(("transform.scale_y", 2.0)));
    assert!(rgb_at(&ffmpeg, &scale_y, None, 5, 15)[0] > 180);
    assert!(!red_at(&ffmpeg, &scale_y, None, 15));
    let opacity = render(Some(("transform.opacity", 0.5)));
    let half = rgb_at(&ffmpeg, &opacity, None, 5, 5)[0];
    assert!(
        (70..180).contains(&half),
        "half-opacity red component {half}"
    );
    let restored = render(None);
    assert!(red_at(&ffmpeg, &restored, None, 5));
}

#[test]
fn native_audio_gain_multiplies_volume_mute_fades_and_ducking() {
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    let (root, core, id, _) = setup();
    let source = root.path().join("media/tone.wav");
    let generated = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
        ])
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let asset = core
        .import_asset(
            &id,
            0,
            &source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                audio_sample_rate_hz: Some(48000),
                audio_channels: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let track_id = core.get_project(&id).unwrap().tracks[2].id.clone();
    let _item_id = core
        .edit(
            &id,
            1,
            operation(json!({
                "operation":"add_media", "trackId":track_id, "assetId":asset,
                "startMs":0,"sourceInMs":0,"durationMs":1000
            })),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let silence_source = root.path().join("media/silence.wav");
    let generated_silence = std::process::Command::new(&ffmpeg)
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=48000:cl=mono",
            "-t",
            "1",
        ])
        .arg(&silence_source)
        .output()
        .unwrap();
    assert!(generated_silence.status.success());
    let silence_asset = core
        .import_asset(
            &id,
            2,
            &silence_source,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                audio_sample_rate_hz: Some(48000),
                audio_channels: Some(1),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 64;
    project.settings.height = 64;
    project.tracks[2].audio_role = AudioTrackRole::Music;
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let render = |project: &opencut_editor_core::Project| {
        let output = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let decoded = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(dir.join(output.relative_path))
            .args(["-vn", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1"])
            .output()
            .unwrap();
        assert!(
            decoded.status.success(),
            "{}",
            String::from_utf8_lossy(&decoded.stderr)
        );
        decoded
            .stdout
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes) as f64)
            .collect::<Vec<_>>()
    };
    let rms = |samples: &[f64], start_ms: usize, end_ms: usize| {
        let samples = &samples[start_ms * 48..end_ms * 48];
        (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
    };
    let baseline = render(&project);
    let baseline_rms = rms(&baseline, 100, 200);
    assert!(baseline_rms > 0.01);
    project.tracks[2].items[0]
        .visual_properties_mut()
        .animation_channels =
        serde_json::from_value(json!([channel("audio.gain_db", -6.0, -6.0)])).unwrap();
    if let opencut_editor_core::TimelineItem::Media(item) = &mut project.tracks[2].items[0] {
        item.audio.volume = 0.5;
    }
    let gained = render(&project);
    let expected = 0.5 * 10_f64.powf(-6.0 / 20.0);
    let actual = rms(&gained, 100, 200) / baseline_rms;
    assert!(
        (actual - expected).abs() < 0.04,
        "gain times base volume: {actual} vs {expected}"
    );
    if let opencut_editor_core::TimelineItem::Media(item) = &mut project.tracks[2].items[0] {
        item.audio.fade_in_ms = 200;
        item.audio.fade_out_ms = 200;
    }
    let faded = render(&project);
    assert!(rms(&faded, 0, 50) < rms(&faded, 300, 400) * 0.4);
    assert!(rms(&faded, 950, 1000) < rms(&faded, 600, 700) * 0.4);
    if let opencut_editor_core::TimelineItem::Media(item) = &mut project.tracks[2].items[0] {
        item.audio.muted = true;
    }
    let muted = render(&project);
    assert!(rms(&muted, 300, 400) < 0.001);
    if let opencut_editor_core::TimelineItem::Media(item) = &mut project.tracks[2].items[0] {
        item.audio.muted = false;
        item.audio.fade_in_ms = 0;
        item.audio.fade_out_ms = 0;
    }
    project.tracks[2].ducking = Some(DuckingSettings {
        enabled: true,
        gain: 0.25,
        attack_ms: 0,
        release_ms: 0,
    });
    let mut voice = project.tracks[2].clone();
    voice.id = "silent-voice".into();
    voice.audio_role = AudioTrackRole::Voiceover;
    voice.ducking = None;
    if let opencut_editor_core::TimelineItem::Media(item) = &mut voice.items[0] {
        item.id = "voice-trigger".into();
        item.start_ms = 300;
        item.duration_ms = 400;
        item.asset_id = silence_asset;
        item.audio.volume = 1.0;
        item.visual_properties.animation_channels.clear();
    }
    project.tracks.push(voice);
    let ducked = render(&project);
    let before_duck = rms(&ducked, 100, 200);
    let during_duck = rms(&ducked, 400, 500);
    assert!(
        (during_duck / before_duck - 0.25).abs() < 0.05,
        "ducking should multiply gain-adjusted volume: {during_duck} / {before_duck}"
    );
}

#[test]
fn native_draft_still_range_and_export_sample_the_same_channel() {
    let Some((ffmpeg, ffprobe)) = native_tools() else {
        return;
    };
    let (root, core, project_id, track_id) = setup();
    let added = core
        .edit(
            &project_id,
            0,
            operation(json!({"operation":"add_rectangle","trackId":track_id,
                "startMs":0,"durationMs":1000,"width":10,"height":10,"color":"#ff0000",
                "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})),
        )
        .unwrap();
    let item_id = &added.changed_ids[0];
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    renderer.readiness().unwrap();
    let dir = core.paths().project_dir(&project_id).unwrap();
    let configure = |mut project: opencut_editor_core::Project| {
        project.settings.width = 64;
        project.settings.height = 64;
        project.settings.fps = 10;
        project
    };
    let baseline = configure(core.get_project(&project_id).unwrap());
    let baseline_still = renderer.render_preview(&baseline, &dir, 500).unwrap();
    assert!(!red_at(
        &ffmpeg,
        &dir.join(baseline_still.relative_path),
        None,
        25
    ));

    let draft = core
        .create_draft(
            &project_id,
            1,
            vec![operation(
                json!({"operation":"set_animation_channels","itemId":item_id,
                "animationChannels":[{"property":"transform.position_x","keyframes":[
                    {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
                    {"timeMs":500,"value":{"type":"scalar","value":20},"curve":"hold"}
                ]}]}),
            )],
            None,
        )
        .unwrap();
    let draft_project = configure(
        core.get_draft_state(&project_id, &draft.id)
            .unwrap()
            .project,
    );
    let draft_still = renderer.render_preview(&draft_project, &dir, 500).unwrap();
    assert!(red_at(
        &ffmpeg,
        &dir.join(draft_still.relative_path),
        None,
        25
    ));

    core.commit_draft(&project_id, &draft.id, 1).unwrap();
    let project = configure(core.get_project(&project_id).unwrap());
    for (time_ms, x) in [(0, 5), (250, 15), (750, 25)] {
        let sampled = renderer.render_preview(&project, &dir, time_ms).unwrap();
        assert!(red_at(&ffmpeg, &dir.join(sampled.relative_path), None, x));
    }
    let still = renderer.render_preview(&project, &dir, 500).unwrap();
    assert!(red_at(&ffmpeg, &dir.join(still.relative_path), None, 25));
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    assert!(red_at(
        &ffmpeg,
        &dir.join(range.relative_path),
        Some("0.5"),
        25
    ));
    let export = root.path().join("exports/animated.mp4");
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &export,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    assert!(red_at(&ffmpeg, &export, Some("0.5"), 25));
}
