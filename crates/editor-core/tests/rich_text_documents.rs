use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, Project, ProjectSettings,
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
        .create_project("Rich text", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

fn add(track: &str, document: Value) -> Value {
    json!({"operation":"add_text","trackId":track,"document":document,"startMs":0,"durationMs":1000,
        "fontSize":48,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})
}

fn edit(
    core: &EditorCore,
    id: &str,
    operation: Value,
) -> Result<opencut_editor_core::WriteResult, opencut_editor_core::CoreError> {
    core.edit(
        id,
        core.get_project(id).unwrap().revision,
        serde_json::from_value(operation).unwrap(),
    )
}

fn item(core: &EditorCore, id: &str, item: &str) -> Value {
    serde_json::to_value(core.get_project(id).unwrap().find_item(item).unwrap()).unwrap()
}

fn bytes(core: &EditorCore, id: &str) -> (Vec<u8>, Vec<u8>) {
    let dir = core.paths().project_dir(id).unwrap();
    (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    )
}

#[test]
fn canonical_documents_and_limits_preserve_unicode_and_reject_invalid_input() {
    let (_root, core, id, track) = setup();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/rich-text-documents-v1.json"
    ))
    .unwrap();
    assert_eq!(
        catalog["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    for fixture in catalog["valid"].as_array().unwrap() {
        let created = edit(&core, &id, add(&track, fixture["document"].clone())).unwrap();
        let saved = item(&core, &id, &created.changed_ids[0]);
        assert_eq!(saved["document"], fixture["document"]);
        assert_eq!(saved["text"], fixture["text"]);
    }
    for fixture in catalog["invalid"].as_array().unwrap() {
        let before = bytes(&core, &id);
        let request =
            serde_json::from_value::<EditOperation>(add(&track, fixture["document"].clone()));
        if fixture["stage"] == "structure" {
            assert!(request.is_err(), "{}", fixture["id"]);
        } else {
            assert_eq!(
                core.edit(
                    &id,
                    core.get_project(&id).unwrap().revision,
                    request.unwrap()
                )
                .unwrap_err()
                .code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(bytes(&core, &id), before);
    }
    let boundary = json!({"runs":vec![json!({"text":"<svg>literal</svg>"});256]});
    assert!(edit(&core, &id, add(&track, boundary)).is_err());
    let boundary = json!({"runs":vec![json!({"text":"é".repeat(8)});256]});
    edit(&core, &id, add(&track, boundary)).unwrap();
    for document in [
        json!({"runs":vec![json!({"text":"x"});257]}),
        json!({"runs":[{"text":"é".repeat(2049)}]}),
    ] {
        assert_eq!(
            edit(&core, &id, add(&track, document)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert!(
        serde_json::from_str::<opencut_editor_core::RichTextDocument>(
            r#"{"runs":[{"text":"\ud800"}]}"#
        )
        .is_err()
    );
}

#[test]
fn edits_preserve_projection_aliases_history_and_atomic_failures() {
    let (_root, core, id, track) = setup();
    let document =
        json!({"runs":[{"text":"Hello", "bold":true},{"text":" world", "color":"#ff0000"}]});
    let mut request = add(&track, document.clone());
    request["resultAlias"] = json!("title");
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([request,
        {"operation":"update_item","itemId":"@title","color":"#00ff00"}]))
    .unwrap();
    let created = core.edit_batch(&id, 0, batch).unwrap();
    let item_id = &created.changed_ids[0];
    assert_eq!(item(&core, &id, item_id)["document"], document);
    let before = bytes(&core, &id);
    let bad_batch = serde_json::from_value::<Vec<BatchEditOperation>>(json!([
        {"operation":"update_item","itemId":item_id,"text":"temporary"},
        {"operation":"update_item","itemId":"missing","document":document}]))
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, 1, bad_batch).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(bytes(&core, &id), before);
    for op in [
        json!({"operation":"update_item","itemId":item_id,"text":"x","document":document}),
        {
            let mut v = add(&track, document.clone());
            v["text"] = json!("x");
            v
        },
    ] {
        assert_eq!(
            edit(&core, &id, op).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        core.edit(
            &id,
            0,
            serde_json::from_value(add(&track, document.clone())).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(bytes(&core, &id), before);
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":item_id,"text":"plain"}),
    )
    .unwrap();
    assert_eq!(
        item(&core, &id, item_id)["document"],
        json!({"runs":[{"text":"plain"}]})
    );
    core.undo(&id, 2).unwrap();
    assert_eq!(item(&core, &id, item_id)["document"], document);
    core.redo(&id, 3).unwrap();
    assert_eq!(item(&core, &id, item_id)["text"], "plain");
    for field in ["text", "document"] {
        assert!(
            serde_json::from_value::<EditOperation>(
                json!({"operation":"update_item","itemId":item_id,field:null})
            )
            .is_err()
        );
    }
}

fn legacy(project: &mut Value, version: u32) {
    project["schemaVersion"] = json!(version);
    for track in project["tracks"].as_array_mut().unwrap() {
        for item in track["items"].as_array_mut().unwrap() {
            if item["type"] == "text" {
                item.as_object_mut().unwrap().remove("document");
            }
        }
    }
}

#[test]
fn migration_upgrades_current_and_both_history_stacks_without_rewriting_reopen() {
    let (_root, core, id, track) = setup();
    let created = edit(
        &core,
        &id,
        add(&track, json!({"runs":[{"text":"old é\n"}]})),
    )
    .unwrap();
    let item_id = &created.changed_ids[0];
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":item_id,"text":"new"}),
    )
    .unwrap();
    core.undo(&id, 2).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let (project_bytes, history_bytes) = bytes(&core, &id);
    let mut project: Value = serde_json::from_slice(&project_bytes).unwrap();
    let mut history: Value = serde_json::from_slice(&history_bytes).unwrap();
    legacy(&mut project, 17);
    for key in ["undo", "redo"] {
        for snapshot in history[key].as_array_mut().unwrap() {
            legacy(snapshot, 16);
        }
    }
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&history).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 18);
    assert_eq!(
        item(&core, &id, item_id)["document"],
        json!({"runs":[{"text":"old é\n"}]})
    );
    let first = bytes(&core, &id);
    let retained: Value = serde_json::from_slice(&first.1).unwrap();
    for key in ["undo", "redo"] {
        for snapshot in retained[key].as_array().unwrap() {
            assert_eq!(snapshot["schemaVersion"], 18);
        }
    }
    core.get_project(&id).unwrap();
    assert_eq!(bytes(&core, &id), first);
    core.redo(&id, migrated.revision).unwrap();
    assert_eq!(item(&core, &id, item_id)["text"], "new");
}

#[test]
fn persisted_current_and_retained_documents_fail_closed() {
    for location in ["current", "undo", "redo"] {
        for failure in ["old-document", "missing-document", "mismatch", "future"] {
            let (_root, core, id, track) = setup();
            edit(&core, &id, add(&track, json!({"runs":[{"text":"base"}]}))).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let mut project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            let mut invalid = project.clone();
            match failure {
                "old-document" => invalid["schemaVersion"] = json!(17),
                "missing-document" => {
                    invalid["tracks"][1]["items"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("document");
                }
                "mismatch" => invalid["tracks"][1]["items"][0]["text"] = json!("other"),
                _ => invalid["schemaVersion"] = json!(19),
            }
            if failure == "old-document" || failure == "missing-document" {
                assert!(serde_json::from_value::<Project>(invalid.clone()).is_err());
            }
            let mut history = json!({"undo":[],"redo":[]});
            if location == "current" {
                project = invalid;
            } else {
                history[location] = json!([invalid]);
            }
            std::fs::write(
                dir.join("project.json"),
                serde_json::to_vec(&project).unwrap(),
            )
            .unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&history).unwrap(),
            )
            .unwrap();
            let before = bytes(&core, &id);
            assert!(core.get_project(&id).is_err(), "{location}/{failure}");
            assert_eq!(bytes(&core, &id), before);
        }
    }
}

#[test]
fn documents_survive_copy_split_move_trim_components_and_drafts() {
    let (_root, core, id, track) = setup();
    let document =
        json!({"runs":[{"text":"left ","bold":true},{"text":"right","color":"#123456"}]});
    let created = edit(&core, &id, add(&track, document.clone())).unwrap();
    let item_id = created.changed_ids[0].clone();
    let duplicated = edit(
        &core,
        &id,
        json!({"operation":"duplicate_items","itemIds":[item_id],"offsetMs":1000}),
    )
    .unwrap();
    for copied in &duplicated.changed_ids {
        assert_eq!(item(&core, &id, copied)["document"], document);
    }
    let split = edit(
        &core,
        &id,
        json!({"operation":"split_item","itemId":item_id,"splitMs":500}),
    )
    .unwrap();
    for split_id in &split.changed_ids {
        assert_eq!(item(&core, &id, split_id)["document"], document);
    }
    edit(
        &core,
        &id,
        json!({"operation":"move_item","itemId":item_id,"trackId":track,"startMs":100}),
    )
    .unwrap();
    edit(
        &core,
        &id,
        json!({"operation":"trim_item","itemId":item_id,"startMs":100,"durationMs":200}),
    )
    .unwrap();
    assert_eq!(item(&core, &id, &item_id)["document"], document);
    let mut local = item(&core, &id, &item_id);
    local["id"] = json!("local-title");
    local["stackOrder"] = json!(0);
    let component = json!({"operation":"component_create","name":"Unused hidden text","width":100,"height":100,"durationMs":1000,
        "tracks":[{"id":"local","name":"Local","trackType":"overlay","hidden":true,"items":[local]}]});
    let mut invalid = component.clone();
    invalid["tracks"][0]["items"][0]["document"]["runs"][0]["text"] = json!("mismatch");
    let before = bytes(&core, &id);
    assert_eq!(
        edit(&core, &id, invalid).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(bytes(&core, &id), before);
    edit(&core, &id, component).unwrap();
    let state = core.get_project(&id).unwrap();
    assert_eq!(
        serde_json::to_value(&state.components[0].tracks[0].items[0]).unwrap()["document"],
        document
    );
    let draft=core.create_draft(&id,state.revision,vec![serde_json::from_value(json!({"operation":"update_item","itemId":item_id,"document":{"runs":[{"text":"draft"}]}})).unwrap()],None).unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap();
    assert_eq!(
        serde_json::to_value(materialized.project.find_item(&item_id).unwrap()).unwrap()["text"],
        "draft"
    );
    assert_eq!(item(&core, &id, &item_id)["document"], document);
    let solid=edit(&core,&id,json!({"operation":"add_solid_color","trackId":track,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap();
    assert_eq!(
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":solid.changed_ids[0],"document":document})
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_track","trackId":track,"locked":true}),
    )
    .unwrap();
    let before = bytes(&core, &id);
    assert_eq!(
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":item_id,"document":document})
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(bytes(&core, &id), before);
}

#[test]
fn every_supported_source_version_preserves_simple_text() {
    for version in 1..=17 {
        let (_root, core, id, track) = setup();
        let created = edit(
            &core,
            &id,
            add(&track, json!({"runs":[{"text":" é 👋\n"}]})),
        )
        .unwrap();
        let mut old = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        legacy(&mut old, version);
        let dir = core.paths().project_dir(&id).unwrap();
        std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&json!({"undo":[old],"redo":[]})).unwrap(),
        )
        .unwrap();
        assert_eq!(core.get_project(&id).unwrap().schema_version, 18);
        assert_eq!(
            item(&core, &id, &created.changed_ids[0])["document"],
            json!({"runs":[{"text":" é 👋\n"}]})
        );
    }
}

#[test]
fn legacy_component_text_requests_remain_compatible_with_strict_persistence() {
    let (_root, core, id, _track) = setup();
    let text = json!({"type":"text","id":"title","text":"Legacy", "startMs":0,"durationMs":1000,"fontSize":24,"color":"#ffffff","keyframes":[]});
    let tracks = json!([{"id":"local","name":"Local","trackType":"overlay","items":[text]}]);
    let created=edit(&core,&id,json!({"operation":"component_create","name":"Legacy","width":100,"height":100,"durationMs":1000,"tracks":tracks})).unwrap();
    let component_id = &created.changed_ids[0];
    let state = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    assert_eq!(
        state["components"][0]["tracks"][0]["items"][0]["document"],
        json!({"runs":[{"text":"Legacy"}]})
    );
    edit(
        &core,
        &id,
        json!({"operation":"component_update","componentId":component_id,"name":"Legacy","width":100,"height":100,"durationMs":1000,"tracks":tracks}),
    )
    .unwrap();
    let mut invalid = state;
    invalid["components"][0]["tracks"][0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("document");
    assert!(serde_json::from_value::<Project>(invalid).is_err());
}
