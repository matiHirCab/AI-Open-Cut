use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    TrackType,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/component-lifecycle-v1.json"
    ))
    .unwrap()
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Instances", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Overlay)
        .unwrap()
        .id
        .clone();
    (root, core, id, track)
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
fn definition() -> Value {
    json!({"operation":"component_create","name":"Leaf","width":320,"height":240,"durationMs":1000,"tracks":[]})
}

fn seed(core: &EditorCore, id: &str, track: &str) -> String {
    let mut leaf = definition();
    leaf["tracks"] = json!([{"id":"local","name":"Local","trackType":"overlay","items":[{"id":"box","type":"rectangle","startMs":0,"durationMs":1000,"width":20,"height":20,"color":"#FF0000","keyframes":[]}]}]);
    leaf["resultAlias"] = json!("leaf");
    let edits = json!([
        leaf,
        {"operation":"component_define_slots","componentId":"@leaf","slots":[{"id":"opacity","name":"Opacity","kind":"number","required":false,"defaultValue":{"type":"number","value":1},"binding":{"targetLayerId":"box","property":"visual.opacity"},"constraints":{"min":0,"max":1}}]},
        {"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":100,"trimStartMs":0,"durationMs":500,"timeScale":1.5,"slotValues":{"opacity":{"type":"number","value":0.5}},"zIndex":-2,"resultAlias":"instance"},
        {"operation":"component_instance_duplicate","itemId":"@instance","offsetMs":200,"slotValues":{"opacity":{"type":"number","value":0.25}},"resultAlias":"copy"},
        {"operation":"item_set_z_index","itemId":"@copy","zIndex":3}
    ]);
    core.edit_batch::<BatchEditOperation>(id, 0, serde_json::from_value(edits).unwrap())
        .unwrap()
        .aliases["instance"]
        .clone()
}

#[test]
fn canonical_lifecycle_operations_are_closed() {
    for value in catalog()["validOperations"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_ok(),
            "{value}"
        );
    }
    for value in catalog()["invalidOperations"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<EditOperation>(value.clone()).is_err(),
            "{value}"
        );
    }
    for value in catalog()["validBatch"].as_array().unwrap() {
        assert!(serde_json::from_value::<BatchEditOperation>(value.clone()).is_ok());
    }
}

#[test]
fn lifecycle_aliases_overrides_history_and_reopen() {
    let (_root, core, id, track) = setup();
    let instance = seed(&core, &id, &track);
    let seeded = core.get_project(&id).unwrap();
    let source = serde_json::to_value(seeded.find_item(&instance).unwrap()).unwrap();
    let components = serde_json::to_value(&seeded.components).unwrap();
    for (revision, replacement) in [
        (1, None),
        (2, Some(json!({}))),
        (3, Some(json!({"opacity":{"type":"number","value":0.0}}))),
    ] {
        let mut edit =
            json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":50});
        if let Some(values) = replacement.clone() {
            edit["slotValues"] = values;
        }
        let result = core.edit(&id, revision, op(edit)).unwrap();
        assert_eq!(result.changed_ids.len(), 1);
        let project = core.get_project(&id).unwrap();
        let mut expected = source.clone();
        expected["id"] = json!(result.changed_ids[0]);
        expected["startMs"] = json!(150);
        expected["stackOrder"] = json!(revision + 1);
        if let Some(values) = replacement {
            expected["slotValues"] = values;
        }
        assert_eq!(
            serde_json::to_value(project.find_item(&result.changed_ids[0]).unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::to_value(project.find_item(&instance).unwrap()).unwrap(),
            source
        );
        assert_eq!(
            serde_json::to_value(project.components).unwrap(),
            components
        );
    }
    let expected = serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap();
    core.undo(&id, 4).unwrap();
    core.redo(&id, 5).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(serde_json::to_value(reopened.tracks).unwrap(), expected);
    // The original four-step creation/definition/placement/duplication was one undo step.
    for revision in 6..10 {
        core.undo(&id, revision).unwrap();
    }
    assert!(core.get_project(&id).unwrap().components.is_empty());
    core.redo(&id, 10).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap(),
        serde_json::to_value(seeded.tracks).unwrap()
    );
}

#[test]
fn failures_preserve_project_and_history_bytes() {
    let (_root, core, id, track) = setup();
    let instance = seed(&core, &id, &track);
    let before = files(&core, &id);
    for fixture in catalog()["semanticFailures"].as_array().unwrap() {
        let mut edit = fixture["edit"].clone();
        if edit["itemId"] == "instance" {
            edit["itemId"] = json!(instance);
        }
        let error = core.edit(&id, 1, op(edit)).unwrap_err();
        assert_eq!(serde_json::to_value(error.code).unwrap(), fixture["code"]);
        assert_eq!(files(&core, &id), before);
    }
    let duplicate =
        json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":0});
    assert_eq!(
        core.edit(&id, 0, op(duplicate.clone())).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    for edits in [
        json!([duplicate, {"operation":"delete_item","itemId":"missing"}]),
        json!([{"operation":"component_instance_duplicate","itemId":"@later","offsetMs":0,"resultAlias":"later"}]),
        json!([{"operation":"component_instance_duplicate","itemId":instance,"offsetMs":0,"resultAlias":"copy"},{"operation":"component_instance_duplicate","itemId":"@copy","offsetMs":0,"resultAlias":"copy"}]),
    ] {
        assert!(
            core.edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(edits).unwrap())
                .is_err()
        );
        assert_eq!(files(&core, &id), before);
    }
    core.edit(
        &id,
        1,
        op(json!({"operation":"update_track","trackId":track,"locked":true})),
    )
    .unwrap();
    let locked = files(&core, &id);
    assert_eq!(
        core.edit(&id, 2, op(duplicate)).unwrap_err().code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(&core, &id), locked);
}

#[test]
fn all_slot_kinds_and_special_keys_duplicate_without_materializing_definitions() {
    use opencut_editor_core::{MediaProbeFacts, MediaType, SlotValue};
    let slots: Value =
        serde_json::from_slice(include_bytes!("../../../contracts/template-slots-v1.json"))
            .unwrap();
    let mut fixtures = slots["valid"].as_array().unwrap().clone();
    for key in ["__proto__", "constructor", "toString"] {
        let mut fixture = fixtures[0].clone();
        fixture["slot"]["id"] = json!(key);
        fixture["slot"]["defaultValue"]["value"] = json!("@literal");
        fixtures.push(fixture);
    }
    for fixture in fixtures {
        let (root, core, id, track) = setup();
        let mut slot = fixture["slot"].clone();
        let mut leaf = definition();
        leaf["tracks"] = json!([{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"text","id":"title","text":"Base","document":{"runs":[{"text":"Base"}]},"fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]}]}]);
        let mut revision = 0;
        if fixture["id"] == "asset" {
            let path = root.path().join("media/image.png");
            std::fs::write(&path, b"trusted managed image fixture").unwrap();
            let asset = core
                .import_asset(&id, 0, &path, MediaType::Image, MediaProbeFacts::default())
                .unwrap()
                .changed_ids[0]
                .clone();
            revision = 1;
            slot["defaultValue"]["value"]["id"] = json!(asset);
            leaf["tracks"][0]["items"] = json!([{"type":"media","id":"media","assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0,"keyframes":[],"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}}]);
        }
        let values = json!({slot["id"].as_str().unwrap():slot["defaultValue"]});
        // Required with no default proves that omitted copy overrides are inherited.
        slot["required"] = json!(true);
        slot.as_object_mut().unwrap().remove("defaultValue");
        leaf["slots"] = json!([slot]);
        leaf["resultAlias"] = json!("leaf");
        let created = core.edit_batch::<BatchEditOperation>(&id, revision, serde_json::from_value(json!([
            leaf,
            {"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":0,"trimStartMs":0,"durationMs":500,"timeScale":1,"slotValues":values,"resultAlias":"source"},
            {"operation":"component_instance_duplicate","itemId":"@source","offsetMs":0,"resultAlias":"copy"},
            {"operation":"component_instance_duplicate","itemId":"@copy","offsetMs":100,"slotValues":values,"resultAlias":"replacement"}
        ])).unwrap()).unwrap();
        let project = core.get_project(&id).unwrap();
        let expected: std::collections::BTreeMap<String, SlotValue> =
            serde_json::from_value(values).unwrap();
        for alias in ["source", "copy", "replacement"] {
            let opencut_editor_core::TimelineItem::ComponentInstance(item) =
                project.find_item(&created.aliases[alias]).unwrap()
            else {
                panic!("expected instance")
            };
            assert_eq!(
                serde_json::to_value(&item.slot_values).unwrap(),
                serde_json::to_value(&expected).unwrap()
            );
        }
        let before = files(&core, &id);
        assert_eq!(core.edit(&id, created.revision, op(json!({"operation":"component_instance_duplicate","itemId":created.aliases["source"],"offsetMs":0,"slotValues":{}}))).unwrap_err().code, ErrorCode::InvalidArgument);
        assert_eq!(files(&core, &id), before);
        core.undo(&id, created.revision).unwrap();
        core.redo(&id, created.revision + 1).unwrap();
        let reopened = EditorCore::new(core.paths().clone())
            .get_project(&id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(reopened.tracks).unwrap(),
            serde_json::to_value(project.tracks).unwrap()
        );
        assert_eq!(
            serde_json::to_value(reopened.components).unwrap(),
            serde_json::to_value(project.components).unwrap()
        );
    }
}

#[test]
fn safe_time_end_boundary_and_wrong_source_type_are_atomic() {
    let (_root, core, id, track) = setup();
    let instance = seed(&core, &id, &track);
    let duplicate = |offset| {
        op(json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":offset}))
    };
    core.edit(&id, 1, duplicate(9_007_199_254_740_991u64 - 600))
        .unwrap();
    let before = files(&core, &id);
    assert_eq!(
        core.edit(&id, 2, duplicate(9_007_199_254_740_991u64 - 599))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
    let group = core
        .edit(
            &id,
            2,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let before = files(&core, &id);
    assert_eq!(
        core.edit(
            &id,
            3,
            op(json!({"operation":"component_instance_duplicate","itemId":group,"offsetMs":0}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
}

#[test]
fn duplicate_matches_explicit_placement_in_preview_range_draft_and_export() {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions, Renderer};
    let Some(ffmpeg) = std::env::var_os("OPENCUT_FFMPEG_PATH") else {
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    let (_root, core, id, track) = setup();
    let source = seed(&core, &id, &track);
    let component = core.get_project(&id).unwrap().components[0].id.clone();
    let duplicate = op(
        json!({"operation":"component_instance_duplicate","itemId":source,"offsetMs":700,"slotValues":{"opacity":{"type":"number","value":0.75}}}),
    );
    let draft = core
        .create_draft(&id, 1, vec![duplicate.clone()], None)
        .unwrap();
    let explicit = core.create_draft(&id, 1, vec![op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":800,"trimStartMs":0,"durationMs":500,"timeScale":1.5,"zIndex":-2,"slotValues":{"opacity":{"type":"number","value":0.75}}}))], None).unwrap();
    let mut draft_project = core.get_draft_state(&id, &draft.id).unwrap().project;
    let mut explicit_project = core.get_draft_state(&id, &explicit.id).unwrap().project;
    core.edit(&id, 1, duplicate).unwrap();
    let mut committed = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    for project in [&mut draft_project, &mut explicit_project, &mut committed] {
        project.settings.width = 64;
        project.settings.height = 64;
    }
    let before = files(&core, &id);
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
    let decode = |path: &std::path::Path, time: &str| {
        let output = std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-ss", time, "-i"])
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout.len(), 64 * 64 * 3);
        output.stdout
    };
    let mut frames = vec![];
    let mut ranges = vec![];
    let mut exports = vec![];
    for (index, project) in [&draft_project, &explicit_project, &committed]
        .iter()
        .enumerate()
    {
        let frame = renderer.render_preview(project, &dir, 900).unwrap();
        frames.push(decode(&dir.join(frame.relative_path), "0"));
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 900,
                    end_ms: 1200,
                    width: 64,
                    height: 64,
                    fps: 30,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        ranges.push(decode(&dir.join(range.relative_path), "0"));
        let output = dir.join(format!("lifecycle-{index}.mp4"));
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        exports.push(decode(&output, "0.9"));
    }
    assert!(
        frames[0].iter().any(|value| *value > 100),
        "the fixture must emit visible content"
    );
    for outputs in [frames, ranges, exports] {
        assert_eq!(outputs[0], outputs[1]);
        assert_eq!(outputs[1], outputs[2]);
    }
    assert_eq!(files(&core, &id), before);
}

#[test]
fn duplication_enforces_aggregate_text_at_the_inclusive_boundary() {
    let (_root, core, id, track) = setup();
    let items: Vec<_> = (0..128).map(|i| json!({"type":"text","id":format!("t{i}"),"stackOrder":i,"text":"Base","document":{"runs":[{"text":"Base"}]},"fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]})).collect();
    let slots: Vec<_> = (0..128).map(|i| json!({"id":format!("s{i}"),"name":"Text","kind":"text","required":true,"binding":{"targetLayerId":format!("t{i}"),"property":"text.document"},"constraints":{}})).collect();
    let values: serde_json::Map<String, Value> = (0..128)
        .map(|i| {
            (
                format!("s{i}"),
                json!({"type":"text","value":"x".repeat(4096)}),
            )
        })
        .collect();
    let created = core.edit_batch::<BatchEditOperation>(&id, 0, serde_json::from_value(json!([
        {"operation":"component_create","name":"Large","width":64,"height":64,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":items}],"slots":slots,"resultAlias":"leaf"},
        {"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":values,"resultAlias":"source"}
    ])).unwrap()).unwrap();
    let edit = op(
        json!({"operation":"component_instance_duplicate","itemId":created.aliases["source"],"offsetMs":0}),
    );
    core.edit(&id, 1, edit.clone()).unwrap(); // 2 * 128 * 4096 = 1048576 scalars.
    let before = files(&core, &id);
    assert_eq!(
        core.edit(&id, 2, edit).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(files(&core, &id), before);
    let mut oversized = Value::Object(values);
    oversized["s0"]["value"] = json!("x".repeat(4097));
    assert_eq!(core.edit(&id, 2, op(json!({"operation":"component_instance_duplicate","itemId":created.aliases["source"],"offsetMs":0,"slotValues":oversized}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(files(&core, &id), before);
}

#[test]
fn duplicated_expansion_fails_before_render_artifacts() {
    use opencut_editor_core::Renderer;
    let (_root, core, id, track) = setup();
    let mut operations = vec![];
    let mut leaf = definition();
    leaf["resultAlias"] = json!("level0");
    operations.push(leaf);
    for level in 1..=15 {
        let mut outer = definition();
        outer["resultAlias"] = json!(format!("level{level}"));
        outer["tracks"] = json!([{"id":"local","name":"Local","trackType":"overlay","items":(0..2).map(|i| json!({"type":"component_instance","id":format!("i{i}"),"stackOrder":i,"componentId":format!("@level{}",level-1),"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"hidden":true})).collect::<Vec<_>>()}]);
        operations.push(outer);
    }
    operations.push(json!({"operation":"add_component_instance","trackId":track,"componentId":"@level15","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"resultAlias":"source"}));
    operations.push(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}));
    let created = core
        .edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(json!(operations)).unwrap(),
        )
        .unwrap();
    let directory = core.paths().project_dir(&id).unwrap();
    // 65536 occurrences fit the inclusive work bound, including hidden content.
    let ffmpeg = std::env::var_os("OPENCUT_FFMPEG_PATH");
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH");
    let renderer = Renderer::new(
        ffmpeg
            .as_deref()
            .unwrap_or(std::ffi::OsStr::new("missing-ffmpeg")),
        ffprobe
            .as_deref()
            .unwrap_or(std::ffi::OsStr::new("missing-ffprobe")),
        None,
    );
    let bounded = renderer.render_preview(&core.get_project(&id).unwrap(), &directory, 0);
    if ffmpeg.is_some() {
        bounded.unwrap();
    } else {
        let error = bounded.unwrap_err();
        assert!(
            error.code == ErrorCode::DependencyUnavailable
                || error.failed_stage.as_deref() == Some("spawn"),
            "{error:?}"
        );
    }
    core.edit(&id, 1, op(json!({"operation":"component_instance_duplicate","itemId":created.aliases["source"],"offsetMs":0}))).unwrap();
    let before = files(&core, &id);
    let entries = || {
        let mut names = std::fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        names.sort();
        names
    };
    let before_entries = entries();
    let error = renderer
        .render_preview(&core.get_project(&id).unwrap(), &directory, 0)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("maxExpandedOccurrences"));
    assert_eq!(entries(), before_entries);
    assert_eq!(files(&core, &id), before);
}
