use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
    TrackType,
};
use serde_json::{Value, json};

fn edit(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

#[test]
fn font_size_update_preserves_text_and_history() {
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("out"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Font size update",
            ProjectSettings {
                width: 320,
                height: 180,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core
        .get_project(&id)
        .unwrap()
        .tracks
        .into_iter()
        .find(|track| track.track_type == TrackType::Overlay)
        .unwrap()
        .id;
    let created = core
        .edit(
            &id,
            0,
            edit(json!({"operation":"add_text","trackId":track,"text":"Layered","startMs":0,"durationMs":1000,"fontSize":24,"color":"#ffffff","fontFamily":null,"fontPath":null,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})),
        )
        .unwrap();
    let text_id = created.changed_ids[0].clone();
    let before =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&text_id).unwrap()).unwrap();
    let changed = core
        .edit(
            &id,
            created.revision,
            edit(json!({"operation":"update_item","itemId":text_id,"fontSize":48})),
        )
        .unwrap();
    let after =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&text_id).unwrap()).unwrap();
    assert_eq!(after["fontSize"], 48);
    let mut expected = before.clone();
    expected["fontSize"] = json!(48);
    assert_eq!(after, expected);
    core.undo(&id, changed.revision).unwrap();
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&text_id).unwrap()).unwrap(),
        before
    );
    let revision = core.get_project(&id).unwrap().revision;
    core.redo(&id, revision).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(reopened.find_item(&text_id).unwrap()).unwrap(),
        after
    );

    let files = || {
        let dir = core.paths().project_dir(&id).unwrap();
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap())
    };
    let stable = files();
    let current = reopened.revision;
    for (value, code) in [
        (
            json!({"operation":"update_item","itemId":text_id,"fontSize":0}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":text_id,"fontSize":1001}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":"missing","fontSize":48}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(core.edit(&id, current, edit(value)).unwrap_err().code, code);
        assert_eq!(files(), stable);
    }
    for invalid in [Value::Null, json!(1.5), json!(-1)] {
        assert!(
            serde_json::from_value::<EditOperation>(
                json!({"operation":"update_item","itemId":text_id,"fontSize":invalid})
            )
            .is_err()
        );
    }
    assert_eq!(
        core.edit(
            &id,
            current - 1,
            edit(json!({"operation":"update_item","itemId":text_id,"fontSize":30}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    let batch: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"update_item","itemId":text_id,"fontSize":30},
        {"operation":"delete_item","itemId":"missing"}
    ]))
    .unwrap();
    assert_eq!(
        core.edit_batch(&id, current, batch).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(), stable);

    let draft = core
        .create_draft(
            &id,
            current,
            vec![edit(
                json!({"operation":"update_item","itemId":text_id,"fontSize":64}),
            )],
            None,
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(
            core.get_draft_state(&id, &draft.id)
                .unwrap()
                .project
                .find_item(&text_id)
                .unwrap()
        )
        .unwrap()["fontSize"],
        64
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&text_id).unwrap()).unwrap()
            ["fontSize"],
        48
    );
    core.commit_draft(&id, &draft.id, current).unwrap();
    assert_eq!(
        serde_json::to_value(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap()
                .find_item(&text_id)
                .unwrap()
        )
        .unwrap()["fontSize"],
        64
    );

    let revision = core.get_project(&id).unwrap().revision;
    let solid = core.edit(&id, revision, edit(json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap();
    assert_eq!(
        core.edit(
            &id,
            solid.revision,
            edit(json!({"operation":"update_item","itemId":solid.changed_ids[0],"fontSize":72}))
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
    let locked = core
        .edit(
            &id,
            solid.revision,
            edit(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    assert_eq!(
        core.edit(
            &id,
            locked.revision,
            edit(json!({"operation":"update_item","itemId":text_id,"fontSize":72}))
        )
        .unwrap_err()
        .code,
        ErrorCode::TrackLocked
    );
}
