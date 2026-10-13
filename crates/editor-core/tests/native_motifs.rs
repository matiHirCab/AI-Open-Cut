use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    TimelineItem, TrackType,
};
use serde_json::{Value, json};

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/native-motif-pack-v1.json")).unwrap()
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
fn resolve(value: &mut Value, replacements: &std::collections::BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(id) = replacements.get(text) {
                *text = id.clone();
            }
        }
        Value::Array(values) => {
            for value in values {
                resolve(value, replacements);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                resolve(value, replacements);
            }
        }
        _ => {}
    }
}
fn phase_a(core: &EditorCore, id: &str, track: &str) -> std::collections::BTreeMap<String, String> {
    let c = catalog();
    let seeds: Vec<Value> = c["motifs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["seed"].clone())
        .collect();
    let aliases = core
        .edit_batch::<BatchEditOperation>(id, 0, serde_json::from_value(json!(seeds)).unwrap())
        .unwrap()
        .aliases;
    let mut values = std::collections::BTreeMap::from([("$overlay".into(), track.into())]);
    for motif in c["motifs"].as_array().unwrap() {
        values.insert(
            motif["componentIdPlaceholder"].as_str().unwrap().into(),
            aliases[motif["phaseAAlias"].as_str().unwrap()].clone(),
        );
    }
    values
}
fn phase_b(values: &std::collections::BTreeMap<String, String>) -> Value {
    let c = catalog();
    let mut operations: Vec<Value> = c["motifs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["populate"].clone())
        .collect();
    operations.extend(c["galleryInstances"].as_array().unwrap().iter().cloned());
    let mut result = json!(operations);
    resolve(&mut result, values);
    result
}
#[test]
fn real_native_two_phase_construction_history_and_reopen() {
    let (_root, core, id, track) = setup();
    let values = phase_a(&core, &id, &track);
    let empty = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(empty["components"].as_array().unwrap().len(), 5);
    let result = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(phase_b(&values)).unwrap())
        .unwrap();
    assert_eq!(result.aliases.len(), 10);
    let state = core.get_project(&id).unwrap();
    let expected = serde_json::to_value(&state).unwrap();
    assert_eq!(expected["assets"].as_array().unwrap().len(), 0);
    let components = expected["components"].as_array().unwrap();
    assert_eq!(
        components
            .iter()
            .map(|c| c["slots"].as_array().unwrap().len())
            .sum::<usize>(),
        9
    );
    for (component, motif) in components
        .iter()
        .zip(catalog()["motifs"].as_array().unwrap())
    {
        let mut items = motif["populate"]["tracks"][0]["items"].clone();
        resolve(&mut items, &values);
        let typed: Vec<TimelineItem> = serde_json::from_value(items).unwrap();
        assert_eq!(
            component["tracks"][0]["items"],
            serde_json::to_value(typed).unwrap()
        );
    }
    core.undo(&id, 2).unwrap();
    let undone = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(undone["components"], empty["components"]);
    core.redo(&id, 3).unwrap();
    let restored = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(restored["components"], expected["components"]);
    assert_eq!(restored["tracks"], expected["tracks"]);
    core.undo(&id, 4).unwrap();
    core.undo(&id, 5).unwrap();
    assert!(core.get_project(&id).unwrap().components.is_empty());
    core.redo(&id, 6).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().components).unwrap(),
        empty["components"]
    );
    core.redo(&id, 7).unwrap();
    let restored = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let reopened = serde_json::to_value(
        EditorCore::new(core.paths().clone())
            .get_project(&id)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(reopened, restored);
}
#[test]
fn failed_populate_preserves_committed_empty_phase_and_exact_history() {
    let (_root, core, id, track) = setup();
    let values = phase_a(&core, &id, &track);
    let before = files(&core, &id);
    let mut invalid = phase_b(&values);
    invalid
        .as_array_mut()
        .unwrap()
        .push(json!({"operation":"component_delete","componentId":"missing"}));
    let error = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(invalid).unwrap())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::ItemNotFound);
    assert!(!error.retryable);
    assert_eq!(files(&core, &id), before);
    let mut invalid = phase_b(&values);
    invalid[4]["tracks"][0]["items"][0]["parent"] =
        json!({"scope":"component:wrong","id":"missing"});
    let error = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(invalid).unwrap())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(files(&core, &id), before);
    let mut invalid = phase_b(&values);
    invalid[14]["slotValues"] = json!({"line_color":{"type":"color","value":"#fff"}});
    let error = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(invalid).unwrap())
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(files(&core, &id), before);
    let error = core
        .edit(
            &id,
            0,
            op(json!({"operation":"component_delete","componentId":values["$dotComponentId"]})),
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(error.retryable);
    assert_eq!(files(&core, &id), before);
}

#[test]
fn independent_palettes_reset_and_malformed_colors_preserve_shared_sources() {
    let (_root, core, id, track) = setup();
    let values = phase_a(&core, &id, &track);
    let aliases = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(phase_b(&values)).unwrap())
        .unwrap()
        .aliases;
    let before = core.get_project(&id).unwrap();
    let definition_bytes = serde_json::to_value(&before.components).unwrap();
    let untouched =
        serde_json::to_value(before.find_item(&aliases["speed_lines_alternate"])).unwrap();
    let update = |palette: Value| json!({"operation":"component_instance_update","itemId":aliases["speed_lines_default"],"componentId":values["$speedComponentId"],"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":palette});
    for color in ["#00fFaa", "#000000", "#FFFFFF"] {
        let revision = core.get_project(&id).unwrap().revision;
        core.edit(
            &id,
            revision,
            op(update(json!({"line_color":{"type":"color","value":color}}))),
        )
        .unwrap();
        let p = core.get_project(&id).unwrap();
        assert_eq!(
            serde_json::to_value(&p.components).unwrap(),
            definition_bytes
        );
        assert_eq!(
            serde_json::to_value(p.find_item(&aliases["speed_lines_alternate"])).unwrap(),
            untouched
        );
        let item = serde_json::to_value(p.find_item(&aliases["speed_lines_default"])).unwrap();
        assert_eq!(item["slotValues"]["line_color"]["value"], color);
    }
    for color in ["#fff", "red", "#FFFFFF00", "#GG0000"] {
        let before = files(&core, &id);
        let revision = core.get_project(&id).unwrap().revision;
        assert_eq!(
            core.edit(
                &id,
                revision,
                op(update(json!({"line_color":{"type":"color","value":color}})))
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    let revision = core.get_project(&id).unwrap().revision;
    core.edit(&id, revision, op(update(json!({})))).unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(p.find_item(&aliases["speed_lines_default"])).unwrap()["slotValues"],
        json!({})
    );
    assert_eq!(
        serde_json::to_value(p.components).unwrap(),
        definition_bytes
    );
    let reset = serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap();
    core.undo(&id, revision + 1).unwrap();
    let prior = serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap();
    assert_ne!(prior, reset);
    let p = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(p.find_item(&aliases["speed_lines_default"])).unwrap()["slotValues"]["line_color"]
            ["value"],
        "#FFFFFF"
    );
    assert_eq!(
        serde_json::to_value(&p.components).unwrap(),
        definition_bytes
    );
    assert_eq!(
        serde_json::to_value(p.find_item(&aliases["speed_lines_alternate"])).unwrap(),
        untouched
    );
    core.undo(&id, revision + 2).unwrap();
    assert_eq!(
        serde_json::to_value(
            core.get_project(&id)
                .unwrap()
                .find_item(&aliases["speed_lines_default"])
        )
        .unwrap()["slotValues"]["line_color"]["value"],
        "#000000"
    );
    core.redo(&id, revision + 3).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().tracks).unwrap(),
        prior
    );
    core.redo(&id, revision + 4).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(serde_json::to_value(reopened.tracks).unwrap(), reset);
    assert_eq!(
        serde_json::to_value(reopened.components).unwrap(),
        definition_bytes
    );
}

#[test]
fn reference_geometry_is_literal_bounded_and_has_independent_contour_counts() {
    let (_root, core, id, track) = setup();
    let values = phase_a(&core, &id, &track);
    core.edit_batch::<BatchEditOperation>(
        &id,
        1,
        serde_json::from_value(phase_b(&values)).unwrap(),
    )
    .unwrap();
    let p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let components = p["components"].as_array().unwrap();
    let dots = &components[0]["tracks"][0]["items"][0]["grid"];
    assert_eq!(dots["width"].as_f64(), Some(280.));
    assert_eq!(dots["height"].as_f64(), Some(200.));
    assert_eq!(dots["pattern"]["spacingX"].as_f64(), Some(40.));
    assert_eq!(dots["pattern"]["spacingY"].as_f64(), Some(40.));
    let marks = (dots["width"].as_f64().unwrap() / 40.).floor() as usize + 1;
    let rows = (dots["height"].as_f64().unwrap() / 40.).floor() as usize + 1;
    assert_eq!(marks * rows, 48);
    assert_eq!(
        components[1]["tracks"][0]["items"][0]["stroke"]["width"].as_f64(),
        Some(12.0)
    );
    let rays = components[2]["tracks"][0]["items"][0]["geometry"]["path"]["commands"]
        .as_array()
        .unwrap();
    assert_eq!(rays.iter().filter(|c| c["type"] == "moveTo").count(), 8);
    assert_eq!(rays.iter().filter(|c| c["type"] == "close").count(), 8);
    let speed = &components[4]["tracks"][0]["items"][0];
    let contours = speed["geometry"]["path"]["commands"].as_array().unwrap();
    assert_eq!(contours.len(), 30);
    for (i, contour) in contours.chunks_exact(5).enumerate() {
        let x = 20. + 12. * i as f64;
        let y = 50. + 24. * i as f64;
        for (command, (cx, cy)) in
            contour[..4]
                .iter()
                .zip([(x, y), (x + 180., y), (x + 180., y + 6.), (x, y + 6.)])
        {
            assert_eq!(command["to"]["x"].as_f64(), Some(cx));
            assert_eq!(command["to"]["y"].as_f64(), Some(cy));
        }
        assert_eq!(contour[4]["type"], "close");
    }
    assert_eq!(speed["fill"]["color"]["a"].as_f64(), Some(0.75));
    assert_eq!(
        components
            .iter()
            .map(|c| c["tracks"][0]["items"].as_array().unwrap().len())
            .sum::<usize>(),
        8
    );
    for component in components {
        assert_eq!(component["width"].as_f64(), Some(320.));
        assert_eq!(component["height"].as_f64(), Some(240.));
        let roles = component["tracks"][0]["items"].as_array().unwrap();
        for (i, role) in roles.iter().enumerate() {
            assert_eq!(role["stackOrder"].as_u64(), Some(i as u64));
        }
    }
}
#[test]
fn native_parameter_limits_and_locked_tracks_reject_without_partial_population() {
    let (_root, core, id, track) = setup();
    let values = phase_a(&core, &id, &track);
    let before = files(&core, &id);
    for invalid in [
        {
            let mut b = phase_b(&values);
            b[0]["tracks"][0]["items"][0]["grid"]["pattern"]["spacingX"] = json!(0);
            b
        },
        {
            let mut b = phase_b(&values);
            b[4]["tracks"][0]["items"][0]["transform2d"]["scaleX"] = json!(0);
            b
        },
        {
            let mut b = phase_b(&values);
            b[4]["tracks"][0]["items"][0]["geometry"]["path"]["commands"][0]["to"]["x"] =
                json!(1_000_001);
            b
        },
    ] {
        assert_eq!(
            core.edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(invalid).unwrap())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(files(&core, &id), before);
    }
    let path = core.paths().project_dir(&id).unwrap().join("project.json");
    let mut p = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for t in p["tracks"].as_array_mut().unwrap() {
        if t["id"] == track {
            t["locked"] = json!(true);
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&p).unwrap()).unwrap();
    let before = files(&core, &id);
    assert_eq!(
        core.edit_batch::<BatchEditOperation>(
            &id,
            1,
            serde_json::from_value(phase_b(&values)).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(&core, &id), before);
}

#[test]
fn setup_failure_is_atomic_and_existing_scalar_boundaries_are_retained() {
    let (_root, core, id, track) = setup();
    let initial = files(&core, &id);
    let mut seeds: Vec<Value> = catalog()["motifs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["seed"].clone())
        .collect();
    seeds.push(json!({"operation":"component_delete","componentId":"missing"}));
    assert!(
        core.edit_batch::<BatchEditOperation>(
            &id,
            0,
            serde_json::from_value(json!(seeds)).unwrap()
        )
        .is_err()
    );
    assert_eq!(files(&core, &id), initial);
    let values = phase_a(&core, &id, &track);
    let aliases = core
        .edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(phase_b(&values)).unwrap())
        .unwrap()
        .aliases;
    let sources = serde_json::to_value(core.get_project(&id).unwrap().components).unwrap();
    let original = serde_json::to_value(
        core.get_project(&id)
            .unwrap()
            .find_item(&aliases["speed_lines_alternate"]),
    )
    .unwrap();
    let transform = json!({"position":{"x":-1_000_000,"y":1_000_000,"unit":"pixels"},"anchor":{"x":1,"y":0},"scaleX":100,"scaleY":0.0001,"rotationDeg":36000,"skewXDeg":80,"skewYDeg":-80,"opacity":0});
    core.edit(&id,2,op(json!({"operation":"update_item","itemId":aliases["speed_lines_default"],"transform2d":transform}))).unwrap();
    let p = core.get_project(&id).unwrap();
    assert_eq!(serde_json::to_value(&p.components).unwrap(), sources);
    assert_eq!(
        serde_json::to_value(p.find_item(&aliases["speed_lines_alternate"])).unwrap(),
        original
    );
    let before = files(&core, &id);
    let mut invalid = transform;
    invalid["position"]["x"] = json!(-1_000_001);
    assert_eq!(core.edit(&id,3,op(json!({"operation":"update_item","itemId":aliases["speed_lines_default"],"transform2d":invalid}))).unwrap_err().code,ErrorCode::InvalidArgument);
    assert_eq!(files(&core, &id), before);
}
