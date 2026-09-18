#[test]
fn styled_text_canonical_mutations_and_atomic_failures() {
    let (_root, core, id, track) = setup();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../../contracts/styled-text-layers-v1.json"
    ))
    .unwrap();
    for fixture in catalog["valid"].as_array().unwrap() {
        let mut request = add(&track, fixture["document"].clone());
        if let Some(paints) = fixture.get("paintLayers") {
            request["style"] = json!({"paintLayers":paints});
        }
        let result = edit(&core, &id, request).unwrap();
        assert_eq!(
            item(&core, &id, &result.changed_ids[0])["document"],
            fixture["document"]
        );
    }
    for fixture in catalog["invalid"].as_array().unwrap() {
        let mut request = add(
            &track,
            fixture
                .get("document")
                .cloned()
                .unwrap_or(json!({"runs":[{"text":"hello"}]})),
        );
        if let Some(paints) = fixture.get("paintLayers") {
            request["style"] = json!({"paintLayers":paints});
        }
        let before = bytes(&core, &id);
        if let Ok(operation) = serde_json::from_value::<EditOperation>(request) {
            assert_eq!(
                core.edit(&id, core.get_project(&id).unwrap().revision, operation)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(bytes(&core, &id), before);
    }
}

#[test]
fn schema_19_to_20_preserves_history_drafts_and_reopen() {
    let (_root, core, id, track) = setup();
    let result = edit(&core, &id, add(&track, json!({"runs":[{"text":"old"}]}))).unwrap();
    let item_id = &result.changed_ids[0];
    let draft = core
        .create_draft(
            &id,
            result.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"update_item","itemId":item_id,"text":"draft"}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut old = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    old["schemaVersion"] = json!(19);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&old).unwrap()).unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[old],"redo":[old]})).unwrap(),
    )
    .unwrap();
    let draft_before =
        std::fs::read(dir.join("drafts").join(format!("{}.json", draft.id))).unwrap();
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    old["schemaVersion"] = json!(20);
    assert_eq!(migrated, old);
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(history["undo"][0], old);
    assert_eq!(history["redo"][0], old);
    assert_eq!(
        core.get_draft_state(&id, &draft.id)
            .unwrap()
            .project
            .find_item(item_id)
            .unwrap()
            .id(),
        item_id
    );
    assert_eq!(
        std::fs::read(dir.join("drafts").join(format!("{}.json", draft.id))).unwrap(),
        draft_before
    );
    let before = bytes(&core, &id);
    core.get_project(&id).unwrap();
    assert_eq!(before, bytes(&core, &id));
    for location in ["current", "undo", "redo"] {
        let mut invalid = old.clone();
        invalid["schemaVersion"] = json!(19);
        invalid["tracks"][1]["items"][0]["document"]["spans"] = json!([]);
        let current = if location == "current" {
            &invalid
        } else {
            &old
        };
        let history = json!({"undo":[if location == "undo" {&invalid} else {&old}],"redo":[if location == "redo" {&invalid} else {&old}]});
        std::fs::write(
            dir.join("project.json"),
            serde_json::to_vec(current).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("history.json"),
            serde_json::to_vec(&history).unwrap(),
        )
        .unwrap();
        let before = bytes(&core, &id);
        assert!(core.get_project(&id).is_err());
        assert_eq!(before, bytes(&core, &id));
    }
}

#[test]
fn malformed_retained_styled_draft_blocks_migration_without_publication() {
    let (_root, core, id, track) = setup();
    let result = edit(&core, &id, add(&track, json!({"runs":[{"text":"old"}]}))).unwrap();
    let draft = core.create_draft(&id, result.revision, vec![serde_json::from_value(json!({"operation":"update_item","itemId":result.changed_ids[0],"text":"draft"})).unwrap()], None).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    project["schemaVersion"] = json!(19);
    let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
    let mut malformed = serde_json::to_value(&draft).unwrap();
    malformed["operations"][0]
        .as_object_mut()
        .unwrap()
        .remove("text");
    malformed["operations"][0]["document"] =
        json!({"runs":[{"text":"a"}],"spans":[{"start":0,"end":2,"style":{"bold":true}}]});
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    std::fs::write(&draft_path, serde_json::to_vec(&malformed).unwrap()).unwrap();
    let before = bytes(&core, &id);
    let draft_before = std::fs::read(&draft_path).unwrap();
    assert!(core.get_project(&id).is_err());
    assert_eq!(bytes(&core, &id), before);
    assert_eq!(std::fs::read(draft_path).unwrap(), draft_before);
}
