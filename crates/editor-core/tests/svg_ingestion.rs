use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    TimelineItem,
};
use serde_json::{Value, json};
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
            "SVG",
            ProjectSettings {
                width: 160,
                height: 120,
                fps: 24,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn edit(track: &str, svg: &str) -> EditOperation {
    serde_json::from_value(
        json!({"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":svg}),
    )
    .unwrap()
}
fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/svg-ingestion-v1.json")).unwrap()
}

#[test]
fn normalized_svg_is_closed_and_revalidated_in_history_components_and_drafts() {
    let (_root, core, id, track) = setup();
    let source = catalog()["valid"][0]["svg"].as_str().unwrap().to_owned();
    let r = core.edit(&id, 0, edit(&track, &source)).unwrap();
    let p = core.get_project(&id).unwrap();
    let item = p.find_item(&r.changed_ids[0]).unwrap();
    let raw = serde_json::to_string(item).unwrap();
    for forged in [
        raw.replace("\"version\":1", "\"version\":1,\"version\":1"),
        raw.replace("\"width\":40.0", "\"width\":40.0,\"width\":40.0"),
        raw.replace("\"offset\":{", "\"offset\":{\"x\":0,"),
    ] {
        assert_ne!(forged, raw);
        assert!(serde_json::from_str::<TimelineItem>(&forged).is_err());
    }
    let value = serde_json::to_value(item).unwrap();
    let doc = &value["document"];
    assert!(
        serde_json::from_value::<opencut_editor_core::SvgDocument>(json!([
            1,
            40,
            20,
            [0, 0, 40, 20],
            doc["shapes"]
        ]))
        .is_err()
    );
    let before = files(&core, &id);
    let draft = core
        .create_draft(&id, r.revision, vec![edit(&track, &source)], None)
        .unwrap();
    assert_eq!(
        core.get_draft_state(&id, &draft.id).unwrap().project.tracks[1]
            .items
            .len(),
        2
    );
    assert_eq!(files(&core, &id), before);
    assert!(
        core.create_draft(&id, r.revision, vec![edit(&track, "<script/>")], None)
            .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let mut local = value.clone();
    local["id"] = json!("local");
    local["stackOrder"] = json!(0);
    let component = json!({"operation":"component_create","name":"SVG component","width":160,"height":120,"durationMs":1000,"tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[local]}]});
    let mut invalid = component.clone();
    invalid["tracks"][0]["hidden"] = json!(true);
    invalid["tracks"][0]["items"][0]["document"]["version"] = json!(2);
    assert!(
        core.edit(&id, r.revision, serde_json::from_value(invalid).unwrap())
            .is_err()
    );
    assert_eq!(files(&core, &id), before);
    let r = core
        .edit(&id, r.revision, serde_json::from_value(component).unwrap())
        .unwrap();
    assert!(
        core.edit(
            &id,
            r.revision,
            serde_json::from_value(
                json!({"operation":"update_track","trackId":track,"locked":true})
            )
            .unwrap()
        )
        .is_ok()
    );
    let p = core.get_project(&id).unwrap();
    assert_eq!(
        core.edit(&id, p.revision, edit(&track, &source))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
}
fn files(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}
#[test]
fn canonical_svg_ingestion_and_atomic_failures() {
    let (_root, core, id, track) = setup();
    let mut revision = 0;
    for fixture in catalog()["valid"].as_array().unwrap() {
        let r = core
            .edit(
                &id,
                revision,
                edit(&track, fixture["svg"].as_str().unwrap()),
            )
            .unwrap();
        revision = r.revision;
        let p = core.get_project(&id).unwrap();
        let item = p.find_item(&r.changed_ids[0]).unwrap();
        assert!(matches!(item, TimelineItem::Svg(_)));
        let raw = serde_json::to_string(item).unwrap();
        assert!(!raw.contains("<svg"));
    }
    let before = files(&core, &id);
    for fixture in catalog()["invalid"].as_array().unwrap() {
        let err = core
            .edit(
                &id,
                revision,
                edit(&track, fixture["svg"].as_str().unwrap()),
            )
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidArgument, "{fixture}");
        assert_eq!(files(&core, &id), before);
    }
    assert_eq!(
        core.edit(&id, 0, edit(&track, "hostile source"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(
            &id,
            revision,
            edit("missing", catalog()["valid"][0]["svg"].as_str().unwrap())
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackNotFound
    );
    let batch = vec![
        edit(&track, catalog()["valid"][0]["svg"].as_str().unwrap()).into(),
        serde_json::from_value::<BatchEditOperation>(
            json!({"operation":"delete_item","itemId":"missing"}),
        )
        .unwrap(),
    ];
    assert!(core.edit_batch(&id, revision, batch).is_err());
    assert_eq!(files(&core, &id), before);
}
#[test]
fn svg_alias_lifecycle_and_reopen() {
    let (_root, core, id, track) = setup();
    let source = catalog()["valid"][0]["svg"].as_str().unwrap().to_owned();
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_svg","trackId":track,"svg":source,"startMs":0,"durationMs":1000,"resultAlias":"icon"},
        {"operation":"item_set_z_index","itemId":"@icon","zIndex":2}
    ])).unwrap();
    let r = core.edit_batch(&id, 0, batch).unwrap();
    let p = core.get_project(&id).unwrap();
    let item = p.tracks[1].items[0].clone();
    let r = core.undo(&id, r.revision).unwrap();
    assert!(core.get_project(&id).unwrap().tracks[1].items.is_empty());
    let mut r = core.redo(&id, r.revision).unwrap();
    assert_eq!(
        serde_json::to_value(&core.get_project(&id).unwrap().tracks[1].items[0]).unwrap(),
        serde_json::to_value(&item).unwrap()
    );
    for value in [
        json!({"operation":"split_item","itemId":item.id(),"splitMs":500}),
        json!({"operation":"duplicate_items","itemIds":[item.id()],"offsetMs":1000}),
        json!({"operation":"update_item","itemId":item.id(),"transform":{"positionX":12,"positionY":4,"scale":1,"opacity":0.5}}),
    ] {
        r = core
            .edit(&id, r.revision, serde_json::from_value(value).unwrap())
            .unwrap();
    }
    let before = files(&core, &id);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(reopened.get_project(&id).unwrap().revision, r.revision);
    assert_eq!(files(&core, &id), before);
    let value = json!({"operation":"update_item","itemId":item.id(),"fill":null});
    assert_eq!(
        core.edit(&id, r.revision, serde_json::from_value(value).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn svg_migration_current_history_and_future_rejection() {
    for version in 1..=14 {
        let (_root, core, id, _) = setup();
        let dir = core.paths().project_dir(&id).unwrap();
        let mut state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        state["schemaVersion"] = json!(version);
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(&state).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&json!({"undo":[state.clone()],"redo":[state]})).unwrap(),
        )
        .unwrap();
        assert_eq!(core.get_project(&id).unwrap().schema_version, 15);
        let h: Value = serde_json::from_slice(&files(&core, &id).1).unwrap();
        assert_eq!(h["undo"][0]["schemaVersion"], 15);
        assert_eq!(h["redo"][0]["schemaVersion"], 15);
    }
    for location in ["current", "undo", "redo"] {
        let (_root, core, id, track) = setup();
        core.edit(
            &id,
            0,
            edit(&track, catalog()["valid"][0]["svg"].as_str().unwrap()),
        )
        .unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        for version in [14, 16] {
            let mut bad = state.clone();
            bad["schemaVersion"] = json!(version);
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(if location == "current" { &bad } else { &state }).unwrap(),
            )
            .unwrap();
            let h = if location == "undo" {
                json!({"undo":[bad],"redo":[]})
            } else if location == "redo" {
                json!({"undo":[],"redo":[bad]})
            } else {
                json!({"undo":[],"redo":[]})
            };
            std::fs::write(dir.join("history.json"), serde_json::to_vec(&h).unwrap()).unwrap();
            let before = files(&core, &id);
            assert!(core.get_project(&id).is_err());
            assert_eq!(files(&core, &id), before);
        }
    }
}
