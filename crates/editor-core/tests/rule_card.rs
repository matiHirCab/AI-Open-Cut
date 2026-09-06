#[path = "support/rule_card.rs"]
mod fixture;

use fixture::{operation, seed};
use opencut_editor_core::{BatchEditOperation, EditorCore, ErrorCode, TimelineItem};
use serde_json::json;

#[test]
fn rule_card_lifecycle() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(root.path());
    let original = f.project();
    assert_eq!(original.components.len(), 1);
    assert_eq!(original.components[0].tracks[0].items.len(), 6);
    for i in 0..3 {
        let TimelineItem::ComponentInstance(instance) = original
            .find_item(&f.aliases[&format!("instance{i}")])
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(instance.component_id, f.aliases["card"]);
        assert_eq!(
            serde_json::to_value(&instance.slot_values["number"]).unwrap(),
            json!({"type":"text","value":(i+1).to_string()})
        );
    }
    f.move_parent();
    let moved = f.project();
    assert_eq!(
        serde_json::to_value(&moved.components).unwrap(),
        serde_json::to_value(&original.components).unwrap()
    );
    f.core.undo(&f.id, moved.revision).unwrap();
    assert_eq!(
        serde_json::to_value(f.project().tracks).unwrap(),
        serde_json::to_value(&original.tracks).unwrap()
    );
    f.core.redo(&f.id, f.project().revision).unwrap();
    let reopened = EditorCore::new(f.core.paths().clone())
        .get_project(&f.id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(reopened.tracks).unwrap(),
        serde_json::to_value(moved.tracks).unwrap()
    );
}

#[test]
fn rule_card_failures_preserve_files() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(root.path());
    let dir = f.core.paths().project_dir(&f.id).unwrap();
    let files =
        || ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap());
    let before = files();
    let revision = f.project().revision;
    let instance = &f.aliases["instance0"];
    for (edit, expected) in [
        (
            json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":0,"slotValues":{"opacity":{"type":"number","value":2}}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"component_instance_duplicate","itemId":instance,"offsetMs":0,"slotValues":{"icon":{"type":"asset","value":{"kind":"asset","scope":"project","id":"missing"}}}}),
            ErrorCode::AssetNotFound,
        ),
        (
            json!({"operation":"component_instance_update","itemId":instance,"componentId":"missing","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"item_set_parent","itemId":f.aliases["parent"],"parent":{"scope":"root","id":f.aliases["parent"]}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"item_set_parent","itemId":instance,"parent":{"scope":"root","id":"missing"}}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        assert_eq!(
            f.core
                .edit(&f.id, revision, operation(edit))
                .unwrap_err()
                .code,
            expected
        );
        assert_eq!(files(), before);
    }
    let ordering = json!({"operation":"item_set_z_index","itemId":instance,"zIndex":-4});
    let error = f
        .core
        .edit(&f.id, revision - 1, operation(ordering.clone()))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::RevisionConflict);
    assert!(error.retryable);
    assert_eq!(files(), before);
    let edits: Vec<BatchEditOperation> = serde_json::from_value(
        json!([ordering.clone(), {"operation":"delete_item","itemId":"missing"}]),
    )
    .unwrap();
    assert_eq!(
        f.core.edit_batch(&f.id, revision, edits).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(files(), before);
    f.core
        .edit(&f.id, revision, operation(ordering.clone()))
        .unwrap();
    assert_eq!(
        f.project()
            .find_item(instance)
            .unwrap()
            .visual_properties()
            .z_index,
        -4
    );
    let track = f
        .project()
        .tracks
        .into_iter()
        .find(|t| t.items.iter().any(|i| i.id() == instance))
        .unwrap()
        .id;
    f.core
        .edit(
            &f.id,
            revision + 1,
            operation(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    let locked = files();
    assert_eq!(
        f.core
            .edit(&f.id, revision + 2, operation(ordering))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(files(), locked);
}
