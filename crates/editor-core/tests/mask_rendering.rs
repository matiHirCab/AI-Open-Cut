use opencut_editor_core::{EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings};
use serde_json::{Value, json};

fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
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
        .create_project(
            "Active masks",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":24,"height":16,"color":"#ff0000","transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
    (root, core, id, item)
}
fn mask() -> Value {
    json!({"id":"reveal","source":{"type":"path","path":{"fillRule":"nonzero","commands":[
        {"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":24,"y":0}},
        {"type":"lineTo","to":{"x":24,"y":16}},{"type":"lineTo","to":{"x":0,"y":16}},{"type":"close"}]},
        "paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}},"channel":"alpha","operation":"add","inverted":false,"featherPx":0,"expansionPx":0,
        "transform":serde_json::to_value(opencut_editor_core::Transform2D::default()).unwrap()})
}
fn channel(property: &str, first: Value, last: Value) -> Value {
    json!({"property":property,"target":{"kind":"mask","scope":"root","id":"reveal"},"keyframes":[
        {"timeMs":0,"value":first,"curve":"linear"},{"timeMs":999,"value":last,"curve":"linear"}]})
}
fn scalar(property: &str, a: f64, b: f64) -> Value {
    channel(
        property,
        json!({"type":"scalar","value":a}),
        json!({"type":"scalar","value":b}),
    )
}
fn inventory(
    core: &EditorCore,
    id: &str,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        p: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(p).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(&p, out)
            } else {
                out.insert(p.clone(), std::fs::read(p).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(&core.paths().project_dir(id).unwrap(), &mut out);
    out
}
fn install(core: &EditorCore, id: &str, item: &str, value: Value) {
    core.edit(
        id,
        1,
        op(json!({"operation":"update_item","itemId":item,"masks":[value]})),
    )
    .unwrap();
}
#[test]
fn mask_channels_enforce_local_ids_scope_owner_topology_and_atomic_rollback() {
    let (_root, core, id, item) = setup();
    install(&core, &id, &item, mask());
    for (target, code) in [
        (
            json!({"kind":"mask","scope":"root","id":""}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"kind":"mask","scope":"root","id":"x".repeat(129)}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"kind":"mask","scope":"root","id":"missing"}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"kind":"mask","scope":"component:missing","id":"reveal"}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"kind":"effect","scope":"root","id":"reveal"}),
            ErrorCode::InvalidArgument,
        ),
    ] {
        let before = inventory(&core, &id);
        let mut c = scalar("mask.transform.opacity", 1.0, 0.5);
        c["target"] = target;
        let error=core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}))).unwrap_err();
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert_eq!(inventory(&core, &id), before);
    }
    let before = inventory(&core, &id);
    let c = channel(
        "mask.path_points",
        json!({"type":"path_points","points":[{"x":0,"y":0}]}),
        json!({"type":"path_points","points":[{"x":1,"y":1}]}),
    );
    assert_eq!(
        core.edit(
            &id,
            2,
            op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn coupled_mask_safe_endpoints_do_not_hide_unsafe_continuous_interior() {
    // Independent hand case: endpoints have grids604x5 and5855x14; the
    // midpoint requires21259x40, exceeding the unchanged16384 axis cap.
    for (scale_y, skew_x) in [(0.01, -75.0), (24.0, 12.0)] {
        let (_root, core, id, item) = setup();
        let mut m = mask();
        m["source"]["path"]["commands"] = json!([
            {"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":600,"y":0}},
            {"type":"lineTo","to":{"x":600,"y":1}},{"type":"lineTo","to":{"x":0,"y":1}},{"type":"close"}]);
        m["transform"]["scaleX"] = json!(0.2);
        m["transform"]["scaleY"] = json!(scale_y);
        m["transform"]["skewXDeg"] = json!(skew_x);
        m["transform"]["skewYDeg"] = json!(-72.0);
        install(&core, &id, &item, m);
    }
    let (_root, core, id, item) = setup();
    let mut m = mask();
    m["source"]["path"]["commands"] = json!([
        {"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":600,"y":0}},
        {"type":"lineTo","to":{"x":600,"y":1}},{"type":"lineTo","to":{"x":0,"y":1}},{"type":"close"}]);
    m["transform"]["scaleX"] = json!(0.2);
    m["transform"]["scaleY"] = json!(0.01);
    m["transform"]["skewXDeg"] = json!(-75.0);
    m["transform"]["skewYDeg"] = json!(-72.0);
    install(&core, &id, &item, m);
    let before = inventory(&core, &id);
    let error=core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[
        scalar("mask.transform.scale_y",0.01,24.0),scalar("mask.transform.skew_x_deg",-75.0,12.0)]}))).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn source32_channels_reject_even_empty_keys_without_migration_writes() {
    for field in ["property", "target"] {
        let (_root, core, id, item) = setup();
        install(&core, &id, &item, mask());
        let path = core.paths().project_dir(&id).unwrap().join("project.json");
        let mut document: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        document["schemaVersion"] = json!(32);
        document.as_object_mut().unwrap().remove("audioBuses");
        let target = document["tracks"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .flat_map(|t| t["items"].as_array_mut().unwrap())
            .find(|i| i["id"] == item)
            .unwrap();
        target["animationChannels"] = json!([{ "property":if field=="property" {"mask.transform.opacity"}else{"transform.opacity"},"target":{"kind":"mask","scope":"root","id":"reveal"},"keyframes":[]}]);
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let before = inventory(&core, &id);
        assert_eq!(
            core.get_project(&id).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(inventory(&core, &id), before);
    }
}

#[test]
fn literal_at_unicode_mask_ids_survive_single_batch_draft_and_missing_controls() {
    let (_root, core, id, item) = setup();
    let mut m = mask();
    m["id"] = json!("@paint-é");
    install(&core, &id, &item, m);
    let mut c = scalar("mask.transform.opacity", 1.0, 0.5);
    c["target"]["id"] = json!("@paint-é");
    let operation =
        op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}));
    core.edit(&id, 2, operation.clone()).unwrap();
    core.edit_batch(&id, 3, vec![operation.clone()]).unwrap();
    let draft = core.create_draft(&id, 4, vec![operation], None).unwrap();
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .animation_channels[0]
            .target
            .as_ref()
            .unwrap()
            .id,
        "@paint-é"
    );
    let before = inventory(&core, &id);
    let mut missing = scalar("mask.transform.opacity", 1.0, 0.5);
    missing["target"]["id"] = json!("@missing-é");
    assert_eq!(core.edit_batch(&id,4,vec![op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[missing]}))]).unwrap_err().code,ErrorCode::ItemNotFound);
    assert_eq!(inventory(&core, &id), before);
    let mut wrong = scalar("mask.transform.opacity", 1.0, 0.5);
    wrong["target"] = json!({"kind":"graphic_geometry","scope":"root","id":"@paint-é"});
    assert_eq!(core.edit_batch(&id,4,vec![op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[wrong]}))]).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(
        inventory(&core, &id),
        before,
        "wrong mask kind never resolves a creation alias"
    );
}

#[test]
fn original_extreme_transparent_intersect_is_model_valid_but_work_rejects_atomically() {
    let (_root, core, id, item) = setup();
    let mut extreme = mask();
    extreme["source"]["paint"]["color"]["a"] = json!(0.0);
    extreme["operation"] = json!("intersect");
    extreme["featherPx"] = json!(128.0);
    extreme["expansionPx"] = json!(-128.0);
    serde_json::from_value::<opencut_editor_core::Mask>(extreme.clone())
        .unwrap()
        .validate()
        .unwrap();
    let before = inventory(&core, &id);
    let error = core
        .edit(
            &id,
            1,
            op(json!({"operation":"update_item","itemId":item,"masks":[extreme]})),
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert!(error.message.contains("work exceeds"));
    assert_eq!(
        inventory(&core, &id),
        before,
        "no project/history/draft/resource/artifact publication"
    );
}

#[test]
fn exact_large_no_clock_keys_survive_current_and_unavailable_base_draft_validation() {
    let (_root, core, id, item) = setup();
    let start = (1_u64 << 53) + 1;
    core.edit(
        &id,
        1,
        op(json!({"operation":"trim_item","itemId":item,"startMs":0,"durationMs":start+101})),
    )
    .unwrap();
    core.edit(
        &id,
        2,
        op(json!({"operation":"update_item","itemId":item,"masks":[mask()]})),
    )
    .unwrap();
    let mut c = scalar("mask.transform.opacity", 0.5, 0.5);
    c["keyframes"][0]["timeMs"] = json!(start);
    c["keyframes"][1]["timeMs"] = json!(start + 100);
    let operation =
        op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}));
    core.edit(&id, 3, operation.clone()).unwrap();
    let draft = core.create_draft(&id, 4, vec![operation], None).unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    let channels = &candidate
        .find_item(&item)
        .unwrap()
        .visual_properties()
        .animation_channels;
    assert_eq!(channels[0].keyframes[0].time_ms, start);
    assert_eq!(channels[0].keyframes[1].time_ms, start + 100);
    core.edit(
        &id,
        4,
        op(json!({"operation":"update_item","itemId":item,"color":"#00ff00"})),
    )
    .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let history_path = dir.join("history.json");
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(&history_path).unwrap()).unwrap();
    history["undo"] = json!([]);
    history["redo"] = json!([]);
    std::fs::write(&history_path, serde_json::to_vec(&history).unwrap()).unwrap();
    let before = inventory(&core, &id);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(reopened.get_project(&id).unwrap().revision, 5);
    assert_eq!(
        reopened.get_draft_state(&id, &draft.id).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        inventory(&core, &id),
        before,
        "exact-u64 known records remain valid without borrowing an evicted owner"
    );
}

#[test]
fn mask_gradient_order_is_certified_between_safe_spring_endpoints() {
    let (_root, core, id, item) = setup();
    let mut m = mask();
    let paint_stops = |middle: f64, next: f64| {
        json!([
        {"offset":0,"color":{"r":1,"g":1,"b":1,"a":1}},
        {"offset":middle,"color":{"r":1,"g":1,"b":1,"a":1}},
        {"offset":next,"color":{"r":1,"g":1,"b":1,"a":1}},
        {"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}])
    };
    m["source"]["paint"] = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":24,"y":0},"stops":paint_stops(0.25,0.5)});
    install(&core, &id, &item, m);
    let before = inventory(&core, &id);
    let stops = |a: f64, b: f64| {
        json!({"type":"gradient_stops","stops":[
        {"offset":0,"color":[1,1,1,1]},{"offset":a,"color":[1,1,1,1]},
        {"offset":b,"color":[1,1,1,1]},{"offset":1,"color":[1,1,1,1]}]})
    };
    let mut c = channel("mask.gradient_stops", stops(0.25, 0.5), stops(0.5, 0.51));
    c["keyframes"][0]["curve"] =
        json!({"type":"spring","mass":1,"stiffness":100,"damping":1,"initialVelocity":0});
    assert_eq!(
        core.edit(
            &id,
            2,
            op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn schema32_static_masks_adopt_all_generations_but_newly_unsafe_masks_roll_back() {
    fn version(value: &mut Value, to: u64) {
        match value {
            Value::Object(fields) => {
                if fields.contains_key("schemaVersion") {
                    fields.insert("schemaVersion".into(), json!(to));
                    if to < 39 && fields.contains_key("tracks") && fields.contains_key("assets") {
                        fields.remove("audioBuses");
                    }
                }
                for v in fields.values_mut() {
                    version(v, to)
                }
            }
            Value::Array(values) => {
                for v in values {
                    version(v, to)
                }
            }
            _ => {}
        }
    }
    let (_root, core, id, item) = setup();
    install(&core, &id, &item, mask());
    core.edit(
        &id,
        2,
        op(json!({"operation":"set_item_visibility","itemId":item,"hidden":true})),
    )
    .unwrap();
    core.undo(&id, 3).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut draft_mask = mask();
    draft_mask["transform"]["opacity"] = json!(0.5);
    let draft = core
        .create_draft(
            &id,
            4,
            vec![op(
                json!({"operation":"update_item","itemId":item,"masks":[draft_mask]}),
            )],
            None,
        )
        .unwrap();
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let draft_bytes = std::fs::read(&draft_path).unwrap();
    for name in ["project.json", "history.json"] {
        let p = dir.join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        version(&mut value, 32);
        std::fs::write(p, serde_json::to_vec(&value).unwrap()).unwrap();
    }
    let before = core.get_project(&id).unwrap();
    assert_eq!(
        before.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        before
            .find_item(&item)
            .unwrap()
            .visual_properties()
            .masks
            .len(),
        1
    );
    let adopted = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(
        adopted.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        adopted.find_item(&item).unwrap().visual_properties().masks[0]
            .transform
            .opacity,
        0.5
    );
    assert_eq!(
        std::fs::read(&draft_path).unwrap(),
        draft_bytes,
        "source32 own-base metadata draft remains byte-identical"
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for side in ["undo", "redo"] {
        for snapshot in history[side].as_array().unwrap() {
            assert_eq!(
                snapshot["schemaVersion"],
                opencut_editor_core::PROJECT_SCHEMA_VERSION
            );
        }
    }
    core.redo(&id, 4).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    let (_root, core, id, item) = setup();
    install(&core, &id, &item, mask());
    let p = core.paths().project_dir(&id).unwrap().join("project.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
    value["schemaVersion"] = json!(32);
    value.as_object_mut().unwrap().remove("audioBuses");
    let leaf = value["tracks"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .flat_map(|t| t["items"].as_array_mut().unwrap())
        .find(|i| i["id"] == item)
        .unwrap();
    leaf["masks"][0]["source"]["path"]["commands"][1]["to"]["x"] = json!(60000);
    leaf["masks"][0]["source"]["path"]["commands"][2]["to"]["x"] = json!(60000);
    std::fs::write(p, serde_json::to_vec(&value).unwrap()).unwrap();
    let inventory_before = inventory(&core, &id);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&core, &id), inventory_before);
}

#[test]
fn source32_retained_draft_mask_channel_tags_reject_even_empty_keys_atomically() {
    for property in ["mask.transform.opacity", "transform.opacity"] {
        let (_root, core, id, item) = setup();
        install(&core, &id, &item, mask());
        let draft = core
            .create_draft(
                &id,
                2,
                vec![op(
                    json!({"operation":"update_item","itemId":item,"color":"#00ff00"}),
                )],
                None,
            )
            .unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let project_path = dir.join("project.json");
        let mut project: Value =
            serde_json::from_slice(&std::fs::read(&project_path).unwrap()).unwrap();
        project["schemaVersion"] = json!(32);
        project.as_object_mut().unwrap().remove("audioBuses");
        std::fs::write(project_path, serde_json::to_vec(&project).unwrap()).unwrap();
        let path = dir.join("drafts").join(format!("{}.json", draft.id));
        let mut raw: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        raw["operations"] = json!([{"operation":"set_animation_channels","itemId":item,"animationChannels":[{
            "property":property,"target":{"kind":"mask","scope":"root","id":"reveal"},"keyframes":[]}]}]);
        std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
        let before = inventory(&core, &id);
        let error = EditorCore::new(core.paths().clone())
            .get_project(&id)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(!error.retryable);
        assert_eq!(inventory(&core, &id), before);
    }
}

#[test]
fn native_animated_mask_family_board_preserves_all_intents_and_independent_linear_colors() {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let font =
        std::env::var_os("OPENCUT_TEST_FONT_PATH").expect("native board requires bundled font");
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let font_path = std::path::PathBuf::from(&font);
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        let source = if name == "DejaVuSans.ttf" {
            font_path.clone()
        } else {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/fonts")
                .join(name)
        };
        std::fs::copy(source, media.join(name)).unwrap();
    }
    let image = media.join("red.pam");
    let mut pam =
        b"P7\nWIDTH 24\nHEIGHT 20\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
    pam.extend([255, 0, 0, 255].repeat(24 * 20));
    std::fs::write(&image, pam).unwrap();
    let png = media.join("red.png");
    assert!(
        std::process::Command::new(&ffmpeg)
            .args(["-v", "error", "-i"])
            .arg(&image)
            .args(["-frames:v", "1", "-y"])
            .arg(&png)
            .status()
            .unwrap()
            .success()
    );
    let image = png;
    let wav = media.join("tone.wav");
    let count = 48000_u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + count * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(96000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count * 2).to_le_bytes());
    for _ in 0..count {
        bytes.extend(8192_i16.to_le_bytes());
    }
    std::fs::write(&wav, bytes).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap(),
    )
    .with_font_config(opencut_editor_core::FontConfig {
        roots: vec![media.clone()],
        default_path: None,
    });
    let id = core
        .create_project(
            "Animated mask family board",
            ProjectSettings {
                width: 240,
                height: 120,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let tracks = core.get_project(&id).unwrap().tracks;
    let track = tracks[1].id.clone();
    let audio_track = tracks[2].id.clone();
    let mut revision = 0;
    let mut items = Vec::new();
    let transform =
        |x: f64, y: f64, scale: f64| json!({"positionX":x,"positionY":y,"scale":scale,"opacity":1});
    let mut add = |v: Value| {
        let result = core.edit(&id, revision, op(v)).unwrap();
        revision += 1;
        items.push(result.changed_ids[0].clone());
    };
    add(
        json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":transform(5.0,5.0,0.1)}),
    );
    add(
        json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":24,"height":20,"color":"#ff0000","transform":transform(40.0,5.0,1.0)}),
    );
    let position = |x: f64, y: f64| {
        let mut t = serde_json::to_value(opencut_editor_core::Transform2D::default()).unwrap();
        t["position"]["x"] = json!(x);
        t["position"]["y"] = json!(y);
        t
    };
    add(
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":24,"height":20},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"transform2d":position(75.0,5.0)}),
    );
    add(
        json!({"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":"<svg width=\"24\" height=\"20\"><rect width=\"24\" height=\"20\" fill=\"#ff0000\"/></svg>","transform2d":position(110.0,5.0)}),
    );
    add(
        json!({"operation":"add_grid","trackId":track,"startMs":0,"durationMs":1000,"grid":{"width":24,"height":20,"pattern":{"type":"dot","spacingX":16,"spacingY":16,"radius":6,"paint":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}}}},"transform2d":position(145.0,5.0)}),
    );
    add(
        json!({"operation":"add_text","trackId":track,"startMs":0,"durationMs":1000,"text":"HH","fontPath":media.join("DejaVuSans.ttf"),"fontSize":40,"color":"#ff0000","transform":transform(10.0,60.0,1.0)}),
    );
    let asset = core
        .import_asset(
            &id,
            revision,
            &image,
            MediaType::Image,
            MediaProbeFacts {
                duration_ms: Some(1000),
                video_width: Some(24),
                video_height: Some(20),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    revision += 1;
    let image_item=core.edit(&id,revision,op(json!({"operation":"add_media","trackId":track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
    revision += 1;
    core.edit(&id,revision,op(json!({"operation":"update_item","itemId":image_item,"transform":transform(180.0,5.0,1.0)}))).unwrap();
    revision += 1;
    items.push(image_item);
    let asset = core
        .import_asset(
            &id,
            revision,
            &wav,
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
    revision += 1;
    core.edit(&id,revision,op(json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}))).unwrap();
    revision += 1;
    let baseline = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font.into()));
    let decode = |path: &std::path::Path, audio: bool, seek: Option<&str>| {
        let mut cmd = std::process::Command::new(&ffmpeg);
        cmd.args(["-v", "error", "-i"]).arg(path);
        if let Some(t) = seek {
            cmd.args(["-ss", t]);
        }
        if audio {
            cmd.args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "pipe:1",
            ]);
        } else {
            cmd.args([
                "-map",
                "0:v:0",
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ]);
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    };
    let base_frame = renderer.render_preview(&baseline, &dir, 400).unwrap();
    let base = decode(&dir.join(base_frame.relative_path), false, None);
    // Fixed known opaque interiors: rectangle/solid/vector cell centers and the
    // bundled DejaVu Sans H's vertical stem. Baseline verifies fixture placement;
    // expected mask colors below come solely from authored opaque #ff0000.
    let points = [
        (16, 11),
        (50, 15),
        (87, 15),
        (122, 15),
        (161, 21),
        (15, 80),
        (192, 15),
    ];
    let mut operations = Vec::new();
    for item in &items {
        let mut m = mask();
        m["source"]["path"]["commands"] = json!([{ "type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":240,"y":0}},{"type":"lineTo","to":{"x":240,"y":120}},{"type":"lineTo","to":{"x":0,"y":120}},{"type":"close"}]);
        operations.push(op(
            json!({"operation":"update_item","itemId":item,"masks":[m]}),
        ));
        let mut c = scalar("mask.transform.opacity", 0.25, 0.75);
        c["keyframes"][1]["timeMs"] = json!(800);
        operations.push(op(
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[c]}),
        ));
    }
    let draft = core
        .create_draft(&id, revision, operations.clone(), None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit_batch(&id, revision, operations).unwrap();
    let committed = core.get_project(&id).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    let mut reference = None;
    for project in [&candidate, &committed, &reopened] {
        let frame = renderer.render_preview(project, &dir, 400).unwrap();
        let pixels = decode(&dir.join(frame.relative_path), false, None);
        assert_eq!(pixels.len(), 240 * 120 * 3);
        assert_ne!(pixels, base);
        for (family, (x, y)) in points.iter().enumerate() {
            let start = (y * 240 + x) * 3;
            assert!(
                base[start] > 100 && base[start + 1] < 30 && base[start + 2] < 30,
                "independent visible-red fixture interior family{family}"
            );
            // Authored linear red1 * sampled mask opacity(.25 + .5*(400/800))
            // = .5; encode only once, independently yielding sRGB188.
            assert!(
                (i32::from(pixels[start]) - 188).abs() <= 1,
                "family{family} red"
            );
            assert!(
                pixels[start + 1] <= 1 && pixels[start + 2] <= 1,
                "family{family} off channels"
            );
        }
        if let Some(expected) = &reference {
            assert_eq!(&pixels, expected, "same-mask draft/commit/reopen frame");
        } else {
            reference = Some(pixels);
        }
    }
    let baseline_range = renderer
        .render_preview_range(
            &baseline,
            &dir,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 240,
                height: 120,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let baseline_export = root.path().join("exports/baseline.mp4");
    std::fs::create_dir_all(baseline_export.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &baseline,
            &dir,
            ExportOptions {
                output: &baseline_export,
                width: 240,
                height: 120,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let probe = |path: &std::path::Path| {
        let result=std::process::Command::new(&ffprobe).args(["-v","error","-show_entries","format=duration:stream=codec_type,time_base,duration,nb_frames,width,height,sample_rate,channels","-of","json"]).arg(path).output().unwrap();
        assert!(result.status.success());
        serde_json::from_slice::<Value>(&result.stdout).unwrap()
    };
    let range = renderer
        .render_preview_range(
            &committed,
            &dir,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 240,
                height: 120,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("exports/board.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &committed,
            &dir,
            ExportOptions {
                output: &output,
                width: 240,
                height: 120,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    // Keep the existing native SSIM gate. Independent per-family raw equations
    // above establish the color target; this catches intent/clock alignment drift.
    let expected = reference.unwrap();
    for (path, baseline_path, seek, frames, duration) in [
        (
            dir.join(range.relative_path),
            dir.join(baseline_range.relative_path),
            "0.3",
            8,
            0.8,
        ),
        (output, baseline_export, "0.4", 10, 1.0),
    ] {
        let actual = decode(&path, false, Some(seek));
        assert_eq!(actual.len(), expected.len());
        let mean = |v: &[u8]| v.iter().map(|x| f64::from(*x)).sum::<f64>() / v.len() as f64;
        let (a, b) = (mean(&expected), mean(&actual));
        let n = (expected.len() - 1) as f64;
        let va = expected
            .iter()
            .map(|x| (f64::from(*x) - a).powi(2))
            .sum::<f64>()
            / n;
        let vb = actual
            .iter()
            .map(|x| (f64::from(*x) - b).powi(2))
            .sum::<f64>()
            / n;
        let cov = expected
            .iter()
            .zip(&actual)
            .map(|(x, y)| (f64::from(*x) - a) * (f64::from(*y) - b))
            .sum::<f64>()
            / n;
        let ssim = ((2.0 * a * b + 6.5025) * (2.0 * cov + 58.5225))
            / ((a * a + b * b + 6.5025) * (va + vb + 58.5225));
        assert!(ssim >= 0.99, "intent SSIM{ssim}");
        let pcm = decode(&path, true, None);
        assert!(
            pcm.as_chunks::<4>()
                .0
                .iter()
                .any(|b| f32::from_le_bytes(*b).abs() > 0.01)
        );
        let baseline_pcm = decode(&baseline_path, true, None);
        assert_eq!(pcm.len(), baseline_pcm.len());
        let squared = pcm
            .as_chunks::<4>()
            .0
            .iter()
            .zip(baseline_pcm.as_chunks::<4>().0)
            .map(|(a, b)| {
                let d = f64::from(f32::from_le_bytes(*a)) - f64::from(f32::from_le_bytes(*b));
                d * d
            })
            .sum::<f64>();
        let rms = (squared / (pcm.len() / 4) as f64).sqrt();
        assert!(rms <= 0.0001, "mask changed audio RMS{rms}");
        let metadata = probe(&path);
        assert_eq!(
            metadata,
            probe(&baseline_path),
            "mask changed timing/cadence/audio metadata"
        );
        let video = metadata["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["codec_type"] == "video")
            .unwrap();
        assert_eq!(
            video["nb_frames"]
                .as_str()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
            frames
        );
        assert_eq!(
            (video["width"].as_u64(), video["height"].as_u64()),
            (Some(240), Some(120))
        );
        assert!(
            (metadata["format"]["duration"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                - duration)
                .abs()
                <= 0.001
        );
    }
}

#[test]
fn native_inherited_mask_shutter_hold_averages_independently_across_all_intents() {
    use opencut_editor_core::{
        ExportOptions, MediaProbeFacts, MediaType, PreviewRangeOptions, Renderer,
    };
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1")
        );
        assert_ne!(std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(), Ok("1"));
        return;
    };
    let (root, core, id, item) = setup();
    let project = core.get_project(&id).unwrap();
    let track = project.tracks[1].id.clone();
    let mut leaf = serde_json::to_value(project.find_item(&item).unwrap()).unwrap();
    leaf["id"] = json!("inner");
    let mut tracks = json!([{"id":"local","name":"Local","trackType":"overlay","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":[leaf]}]);
    let component=core.edit(&id,1,op(json!({"operation":"component_create","name":"Inherited shutter","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[]}))).unwrap().changed_ids[0].clone();
    core.edit(&id, 2, op(json!({"operation":"delete_item","itemId":item})))
        .unwrap();
    let instance=core.edit(&id,3,op(json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":100,"trimStartMs":50,"durationMs":900,"timeScale":0.75,"slotValues":{}}))).unwrap().changed_ids[0].clone();
    let wav = root.path().join("media/tone.wav");
    let count = 48000_u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + count * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(96000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count * 2).to_le_bytes());
    for _ in 0..count {
        bytes.extend(8192_i16.to_le_bytes());
    }
    std::fs::write(&wav, bytes).unwrap();
    let asset = core
        .import_asset(
            &id,
            4,
            &wav,
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
    let audio_track = project.tracks[2].id.clone();
    core.edit(&id,5,op(json!({"operation":"add_media","trackId":audio_track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}))).unwrap();
    let baseline = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut m = mask();
    m["source"]["path"]["commands"][2]["to"]["y"] = json!(16);
    m["source"]["path"]["commands"][3]["to"]["y"] = json!(16);
    tracks[0]["items"][0]["masks"] = json!([m]);
    tracks[0]["items"][0]["motionBlur"] = json!({"shutterAngleDeg":180,"sampleCount":4});
    tracks[0]["items"][0]["animationChannels"] = json!([{"property":"mask.transform.opacity","target":{"kind":"mask","scope":format!("component:{component}"),"id":"reveal"},"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.25},"curve":"hold"},
        {"timeMs":275,"value":{"type":"scalar","value":0.75},"curve":"linear"},
        {"timeMs":999,"value":{"type":"scalar","value":0.75},"curve":"linear"}]}]);
    let operation = op(
        json!({"operation":"component_update","componentId":component,"name":"Inherited shutter","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[]}),
    );
    let draft = core
        .create_draft(&id, 6, vec![operation.clone()], None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit(&id, 6, operation).unwrap();
    let committed = core.get_project(&id).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    let renderer = Renderer::new(
        &ffmpeg,
        &ffprobe,
        std::env::var_os("OPENCUT_TEST_FONT_PATH").map(std::path::PathBuf::from),
    );
    let decode = |path: &std::path::Path, audio: bool, seek: Option<&str>| {
        let mut cmd = std::process::Command::new(&ffmpeg);
        cmd.args(["-v", "error", "-i"]).arg(path);
        if let Some(t) = seek {
            cmd.args(["-ss", t]);
        }
        if audio {
            cmd.args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "pipe:1",
            ]);
        } else {
            cmd.args([
                "-map",
                "0:v:0",
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ]);
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    };
    let probe = |path: &std::path::Path| {
        let result=std::process::Command::new(&ffprobe).args(["-v","error","-show_entries","stream=codec_type,width,height,r_frame_rate,nb_frames,start_time,duration,sample_rate,channels:format=duration,start_time","-of","json"]).arg(path).output().unwrap();
        assert!(result.status.success());
        serde_json::from_slice::<Value>(&result.stdout).unwrap()
    };
    let render_frame = |p: &opencut_editor_core::Project| {
        let frame = renderer.render_preview(p, &dir, 400).unwrap();
        decode(&dir.join(frame.relative_path), false, None)
    };
    let base = render_frame(&baseline);
    let offset = (28 * 64 + 32) * 3;
    assert!(base[offset] > 100 && base[offset + 1] < 30 && base[offset + 2] < 30);
    // Independent shutter roots381,393,406,418 map through50+.75*(t-100)
    // to260.75,269.75,279.5,288.5ms. Hold opacity: .25,.25,.75,.75;
    // average LINEAR source alpha is exactly .5, then final sRGB is188.
    let expected = 188_u8;
    let mut reference = None;
    for p in [&candidate, &committed, &reopened] {
        let frame = render_frame(p);
        assert!(
            frame[offset].abs_diff(expected) <= 1,
            "independent inherited shutter {frame:?}"
        );
        assert!(frame[offset + 1] <= 1 && frame[offset + 2] <= 1);
        if let Some(old) = &reference {
            assert_eq!(&frame, old);
        } else {
            reference = Some(frame);
        }
    }
    let reference = reference.unwrap();
    let mut no_blur = candidate.clone();
    no_blur.components[0].tracks[0].items[0]
        .visual_properties_mut()
        .motion_blur = None;
    let control = render_frame(&no_blur);
    assert!(
        control[offset].abs_diff(225) <= 1,
        "without shutter root400 samples .75"
    );
    assert_ne!(control, reference);
    let mut wrong_clock = candidate.clone();
    if let opencut_editor_core::TimelineItem::ComponentInstance(i) = wrong_clock
        .tracks
        .iter_mut()
        .flat_map(|t| &mut t.items)
        .find(|i| i.id() == instance)
        .unwrap()
    {
        i.time_scale = 1.0;
        i.trim_start_ms = 0;
    } else {
        panic!("instance");
    }
    let control = render_frame(&wrong_clock);
    assert!(
        control[offset].abs_diff(225) <= 1,
        "without inherited fraction all shutter samples lie after275"
    );
    assert_ne!(control, reference);
    let base_range = renderer
        .render_preview_range(
            &baseline,
            &dir,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let base_export = root.path().join("exports/base.mp4");
    renderer
        .export_video(
            &baseline,
            &dir,
            ExportOptions {
                output: &base_export,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    for (index, p) in [&candidate, &committed, &reopened].into_iter().enumerate() {
        let range = renderer
            .render_preview_range(
                p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 100,
                    end_ms: 900,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let export = root.path().join(format!("exports/masked{index}.mp4"));
        renderer
            .export_video(
                p,
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
        for (path, base_path, seek, frames, duration) in [
            (
                dir.join(range.relative_path),
                dir.join(&base_range.relative_path),
                "0.3",
                8,
                0.8,
            ),
            (export, base_export.clone(), "0.4", 10, 1.0),
        ] {
            let rgb = decode(&path, false, Some(seek));
            assert_eq!(rgb.len(), reference.len());
            assert!(
                rgb.as_chunks::<3>()
                    .0
                    .iter()
                    .any(|p| p[0] > 100 && p[1] < 30 && p[2] < 30)
            );
            let n = (rgb.len() - 1) as f64;
            let mean = |v: &[u8]| v.iter().map(|x| f64::from(*x)).sum::<f64>() / v.len() as f64;
            let (a, b) = (mean(&reference), mean(&rgb));
            let va = reference
                .iter()
                .map(|x| (f64::from(*x) - a).powi(2))
                .sum::<f64>()
                / n;
            let vb = rgb.iter().map(|x| (f64::from(*x) - b).powi(2)).sum::<f64>() / n;
            let cov = reference
                .iter()
                .zip(&rgb)
                .map(|(x, y)| (f64::from(*x) - a) * (f64::from(*y) - b))
                .sum::<f64>()
                / n;
            let ssim = ((2. * a * b + 6.5025) * (2. * cov + 58.5225))
                / ((a * a + b * b + 6.5025) * (va + vb + 58.5225));
            assert!(ssim >= 0.99, "intent alignment SSIM{ssim}");
            let audio = decode(&path, true, None);
            let base_audio = decode(&base_path, true, None);
            assert_eq!(audio.len(), base_audio.len());
            assert!(
                audio
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|b| f32::from_le_bytes(*b).abs() > 0.01)
            );
            let rms = (audio
                .as_chunks::<4>()
                .0
                .iter()
                .zip(base_audio.as_chunks::<4>().0)
                .map(|(a, b)| {
                    (f64::from(f32::from_le_bytes(*a)) - f64::from(f32::from_le_bytes(*b))).powi(2)
                })
                .sum::<f64>()
                / (audio.len() / 4) as f64)
                .sqrt();
            assert!(rms <= 0.0001);
            let metadata = probe(&path);
            assert_eq!(metadata, probe(&base_path));
            let video = metadata["streams"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["codec_type"] == "video")
                .unwrap();
            assert_eq!(
                video["nb_frames"]
                    .as_str()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
                frames
            );
            assert!(
                (metadata["format"]["duration"]
                    .as_str()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    - duration)
                    .abs()
                    <= 0.001
            );
        }
    }
}
