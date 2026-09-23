#[test]
fn advanced_layout_aliases_history_drafts_replacements_and_rollback() {
    let (_root, core, id, track) = setup();
    let layout = json!({"trackingPx":0.5,"lineHeightPx":36.0,"bounds":{"widthPx":240.0,"heightPx":100.0},"wrap":"word","fit":"shrink","verticalAlignment":"center","backgroundCornerRadiusPx":8.0});
    let mut request = add(&track, json!({"runs":[{"text":"A title"}]}));
    request["resultAlias"] = json!("title");
    let created = core.edit_batch::<BatchEditOperation>(&id, 0, serde_json::from_value(json!([request, {"operation":"update_item","itemId":"@title","style":{"layout":layout}}])).unwrap()).unwrap();
    let item_id = &created.changed_ids[0];
    assert_eq!(created.revision, 1);
    assert_eq!(item(&core, &id, item_id)["style"]["layout"], layout);
    let before = bytes(&core, &id);
    let failed = core.edit_batch::<BatchEditOperation>(&id, 1, serde_json::from_value(json!([
        {"operation":"update_item","itemId":item_id,"text":"changed"},
        {"operation":"update_item","itemId":item_id,"style":{"layout":{"fit":"fit_box"}}}
    ])).unwrap()).unwrap_err();
    assert_eq!(failed.code, ErrorCode::InvalidArgument);
    assert_eq!(bytes(&core, &id), before);
    let update: EditOperation = serde_json::from_value(json!({"operation":"update_item","itemId":item_id,"text":"new"})).unwrap();
    assert_eq!(core.edit(&id, 0, update.clone()).unwrap_err().code, ErrorCode::RevisionConflict);
    assert_eq!(edit(&core, &id, json!({"operation":"update_item","itemId":"missing","style":{"layout":layout}})).unwrap_err().code, ErrorCode::ItemNotFound);
    assert_eq!(bytes(&core, &id), before);
    let draft = core.create_draft(&id, 1, vec![update], None).unwrap();
    let draft_item = serde_json::to_value(core.get_draft_state(&id, &draft.id).unwrap().project.find_item(item_id).unwrap()).unwrap();
    assert_eq!(draft_item["style"]["layout"], layout);
    assert_eq!(bytes(&core, &id), before);
    edit(&core, &id, json!({"operation":"update_item","itemId":item_id,"style":{}})).unwrap();
    assert!(item(&core, &id, item_id)["style"].get("layout").is_none());
    core.undo(&id, 2).unwrap();
    assert_eq!(item(&core, &id, item_id)["style"]["layout"], layout);
    core.redo(&id, 3).unwrap();
    assert!(item(&core, &id, item_id)["style"].get("layout").is_none());
    let committed = bytes(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(bytes(&core, &id), committed);
}

#[test]
fn schema_20_layout_migration_preserves_complete_current_and_history() {
    let (_root, core, id, track) = setup();
    let created = edit(&core, &id, add(&track, json!({"runs":[{"text":"legacy"}]}))).unwrap();
    let item_id = &created.changed_ids[0];
    let dir = core.paths().project_dir(&id).unwrap();
    let mut old = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    old["schemaVersion"] = json!(20);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(dir.join("history.json"), serde_json::to_vec(&json!({"undo":[old],"redo":[old]})).unwrap()).unwrap();
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    old["schemaVersion"] = json!(22);
    assert_eq!(migrated, old);
    let history: Value = serde_json::from_slice(&bytes(&core, &id).1).unwrap();
    assert_eq!(history, json!({"undo":[old],"redo":[old]}));
    assert!(item(&core, &id, item_id)["style"].get("layout").is_none());
    let first = bytes(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(bytes(&core, &id), first);
    old["schemaVersion"] = json!(20);
    old["tracks"][1]["items"][0]["style"]["layout"] = json!({});
    assert!(serde_json::from_value::<Project>(old).is_err());
}

#[test]
fn retained_invalid_layout_rejects_reopen_and_migration_without_publication() {
    fn inventory(dir: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn walk(base: &std::path::Path, dir: &std::path::Path, out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() { walk(base, &path, out); }
                else { out.insert(path.strip_prefix(base).unwrap().to_owned(), std::fs::read(path).unwrap()); }
            }
        }
        let mut out = std::collections::BTreeMap::new();
        walk(dir, dir, &mut out);
        out
    }
    for version in [20, 21] {
        for variant in ["update_item", "add_text", "component_create", "component_update"] {
        for invalid_style in invalid_persisted_layout_styles() {
            let (_root, core, id, track) = setup();
            let created = edit(&core, &id, add(&track, json!({"runs":[{"text":"legacy"}]}))).unwrap();
            let op = serde_json::from_value(json!({"operation":"update_item","itemId":created.changed_ids[0],"style":{"layout":{}}})).unwrap();
            let draft = core.create_draft(&id, 1, vec![op], None).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let path = dir.join("drafts").join(format!("{}.json", draft.id));
            let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let mut payload = match variant {
                "update_item" => json!({"operation":variant,"itemId":created.changed_ids[0],"style":invalid_style}),
                "add_text" => { let mut op = add(&track, json!({"runs":[{"text":"draft"}]})); op["style"] = invalid_style.clone(); op },
                _ => {
                    let mut nested = item(&core, &id, &created.changed_ids[0]);
                    nested["style"] = invalid_style.clone();
                    json!({"operation":variant,"componentId":"retained-component","name":"C","width":320,"height":240,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[nested]}]})
                }
            };
            if variant == "component_create" { payload.as_object_mut().unwrap().remove("componentId"); }
            value["operations"][0] = payload;
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            if version == 20 {
                for name in ["project.json", "history.json"] {
                    let path = dir.join(name);
                    let value = String::from_utf8(std::fs::read(&path).unwrap()).unwrap().replace("\"schemaVersion\":21", "\"schemaVersion\":20").replace("\"schemaVersion\": 21", "\"schemaVersion\": 20");
                    std::fs::write(path, value).unwrap();
                }
            }
            let before = inventory(&dir);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument, "{variant}: {invalid_style}");
            assert!(!error.retryable);
            assert!(!error.message.contains("OPENCUT_LAYOUT_DECODE"));
            assert_eq!(inventory(&dir), before);
        }
    }
    }
}

fn invalid_persisted_layout_styles() -> Vec<Value> {
    vec![
        json!({"layout":{"trackingPx":-1}}),
        json!({"layout":{"trackingPx":1001}}),
        json!({"layout":{"fit":"fit_box"}}),
        json!({"padding":{"left":10,"right":10,"top":0,"bottom":0},"layout":{"bounds":{"widthPx":15}}}),
        json!({"layout":null}),
        json!({"layout":{"bounds":null}}),
        json!({"layout":{"bounds":{"widthPx":null}}}),
        json!({"layout":{"lineHeightPx":null}}),
        json!({"layout":{"trackingPx":"wide"}}),
        json!({"layout":{"unknown":1}}),
        json!({"layout":{"bounds":{"unknown":1}}}),
        json!({"layout":{"fit":"auto"}}),
    ]
}

fn persisted_layout_inventory(dir: &std::path::Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(base: &std::path::Path, dir: &std::path::Path, out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() { walk(base, &path, out); }
            else { out.insert(path.strip_prefix(base).unwrap().to_owned(), std::fs::read(path).unwrap()); }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

#[test]
fn persisted_root_layouts_validate_current_hidden_undo_and_redo() {
    for location in ["current", "hidden", "undo", "redo"] {
        for style in invalid_persisted_layout_styles() {
            let (_root, core, id, track) = setup();
            edit(&core, &id, add(&track, json!({"runs":[{"text":"MMMMM"}]}))).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let mut project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
            let text = project["tracks"].as_array_mut().unwrap().iter_mut()
                .flat_map(|track| track["items"].as_array_mut().unwrap())
                .find(|item| item["type"] == "text").unwrap();
            text["style"] = style.clone();
            text["hidden"] = json!(location == "hidden");
            if matches!(location, "undo" | "redo") {
                let history = json!({"undo":if location == "undo" {vec![project.clone()]} else {vec![]},"redo":if location == "redo" {vec![project]} else {vec![]}});
                std::fs::write(dir.join("history.json"), serde_json::to_vec(&history).unwrap()).unwrap();
            } else {
                std::fs::write(dir.join("project.json"), serde_json::to_vec(&project).unwrap()).unwrap();
            }
            let before = persisted_layout_inventory(&dir);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument, "{location}: {style}");
            assert!(!error.retryable);
            assert!(!error.message.contains("OPENCUT_LAYOUT_DECODE"));
            if location == "undo" { assert_eq!(core.undo(&id, 1).unwrap_err().code, ErrorCode::InvalidArgument); }
            if location == "redo" { assert_eq!(core.redo(&id, 1).unwrap_err().code, ErrorCode::InvalidArgument); }
            assert_eq!(persisted_layout_inventory(&dir), before);
        }
    }
}

#[test]
fn persisted_layout_classification_preserves_legacy_and_unrelated_errors() {
    let (_root, core, id, track) = setup();
    edit(&core, &id, add(&track, json!({"runs":[{"text":"OPENCUT_LAYOUT_DECODE: user text"}]}))).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut legacy = original.clone();
    let text = legacy["tracks"].as_array_mut().unwrap().iter_mut()
        .flat_map(|track| track["items"].as_array_mut().unwrap()).find(|item| item["type"] == "text").unwrap();
    text["style"]["outlineWidthPx"] = json!(101);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert!(core.get_project(&id).is_ok(), "layout-absent legacy validation is unchanged");
    for (index, mut value) in [original.clone(), original.clone(), original].into_iter().enumerate() {
        match index {
            0 => value["name"] = json!(42),
            1 => value["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION + 1),
            _ => {
                value["schemaVersion"] = json!(20);
                let text = value["tracks"].as_array_mut().unwrap().iter_mut()
                    .flat_map(|track| track["items"].as_array_mut().unwrap()).find(|item| item["type"] == "text").unwrap();
                text["style"]["layout"] = json!({});
            }
        }
        std::fs::write(dir.join("project.json"), serde_json::to_vec(&value).unwrap()).unwrap();
        let before = persisted_layout_inventory(&dir);
        assert_eq!(core.get_project(&id).unwrap_err().code, ErrorCode::InternalError);
        assert_eq!(persisted_layout_inventory(&dir), before);
    }
}

#[test]
fn valid_stale_layout_draft_survives_migration_without_replay() {
    for version in [20,21] {
        let (_root, core, id, track) = setup();
        let created = edit(&core,&id,add(&track,json!({"runs":[{"text":"legacy"}]}))).unwrap();
        let draft = core.create_draft(&id,1,vec![serde_json::from_value(json!({"operation":"update_item","itemId":created.changed_ids[0],"style":{"layout":{"trackingPx":0.1}}})).unwrap()],None).unwrap();
        edit(&core,&id,json!({"operation":"update_item","itemId":created.changed_ids[0],"text":"current"})).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let draft_path = dir.join("drafts").join(format!("{}.json",draft.id));
        let before = std::fs::read(&draft_path).unwrap();
        for name in ["project.json","history.json"] {
            let path=dir.join(name);
            let raw=std::fs::read_to_string(&path).unwrap()
                .replace("\"schemaVersion\":22", &format!("\"schemaVersion\":{version}"))
                .replace("\"schemaVersion\": 22", &format!("\"schemaVersion\": {version}"));
            std::fs::write(path,raw).unwrap();
        }
        let project = core.get_project(&id).unwrap();
        assert_eq!(project.schema_version,22);
        assert_eq!(project.revision,2);
        assert_eq!(item(&core,&id,&created.changed_ids[0])["text"],"current");
        assert!(item(&core,&id,&created.changed_ids[0])["style"].get("layout").is_none());
        assert_eq!(std::fs::read(draft_path).unwrap(),before);
        assert_eq!(core.get_draft_state(&id,&draft.id).unwrap_err().code,ErrorCode::RevisionConflict);
    }
}
