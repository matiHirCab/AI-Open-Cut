use opencut_editor_core::{
    AnimationChannelProperty, BatchEditOperation, EditOperation, EditorCore, ErrorCode,
    ExportOptions, PROJECT_SCHEMA_VERSION, PathPolicy, PreviewRangeOptions, ProjectSettings,
    Renderer,
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
        }
    }
    assert_eq!(
        contract["active"].as_object().unwrap().len()
            + contract["inactive"].as_object().unwrap().len(),
        29
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

fn red_at(ffmpeg: &std::path::Path, file: &std::path::Path, seek: Option<&str>, x: usize) -> bool {
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
    let at = (5 * 64 + x) * 3;
    output.stdout[at] > 180 && output.stdout[at + 1] < 60 && output.stdout[at + 2] < 60
}

#[test]
fn native_draft_still_range_and_export_sample_the_same_channel() {
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        return;
    };
    let ffmpeg = std::path::PathBuf::from(ffmpeg);
    let ffprobe = std::path::PathBuf::from(ffprobe);
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
