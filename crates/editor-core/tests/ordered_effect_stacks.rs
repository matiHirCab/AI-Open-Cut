use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

fn fixture() -> Value {
    let mut f = serde_json::from_str::<Value>(include_str!(
        "../../../contracts/extended-visual-animation-v1.json"
    ))
    .unwrap()["orderedEffectCases"]
        .clone();
    for stack in f["orders"].as_object_mut().unwrap().values_mut() {
        let typed: Vec<opencut_editor_core::VisualEffect> =
            serde_json::from_value(stack.clone()).unwrap();
        *stack = serde_json::to_value(typed).unwrap();
    }
    f
}
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn inventory(root: &std::path::Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &std::path::Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.file_name().unwrap() != ".lock" {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out);
    out
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
            "Ordered effects",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let source = fixture()["source"].clone();
    let item = core.edit(&id,0,op(json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":800,"geometry":source["geometry"],"fill":source["fill"],"stroke":source["stroke"]}))).unwrap().changed_ids[0].clone();
    (root, core, id, item)
}
fn effects(core: &EditorCore, id: &str, item: &str) -> Value {
    serde_json::to_value(core.get_project(id).unwrap().find_item(item).unwrap())
        .unwrap()
        .get("effects")
        .cloned()
        .unwrap_or(json!([]))
}
fn update(core: &EditorCore, id: &str, item: &str, stack: &Value) {
    core.edit(
        id,
        core.get_project(id).unwrap().revision,
        op(json!({"operation":"update_item","itemId":item,"effects":stack})),
    )
    .unwrap();
}

#[test]
fn ordered_effect_public_lifecycle_preserves_exact_arrays_aliases_drafts_history_and_reopen() {
    let (root, core, id, item) = setup();
    let f = fixture();
    let a = &f["orders"]["shadeThenWash"];
    let b = &f["orders"]["washThenShade"];
    update(&core, &id, &item, a);
    assert_eq!(effects(&core, &id, &item), *a);
    let revision = core.get_project(&id).unwrap().revision;
    core.edit(
        &id,
        revision,
        op(json!({"operation":"update_item","itemId":item,"zIndex":7})),
    )
    .unwrap();
    assert_eq!(effects(&core, &id, &item), *a);
    let dir = core.paths().project_dir(&id).unwrap();
    let before = inventory(&dir);
    let revision = core.get_project(&id).unwrap().revision;
    let draft = core
        .create_draft(
            &id,
            revision,
            vec![op(
                json!({"operation":"update_item","itemId":item,"effects":b}),
            )],
            None,
        )
        .unwrap();
    let materialized = core.get_draft_state(&id, &draft.id).unwrap();
    assert_eq!(
        serde_json::to_value(materialized.project.find_item(&item).unwrap()).unwrap()["effects"],
        *b
    );
    assert_eq!(effects(&core, &id, &item), *a);
    let after = inventory(&dir);
    for (path, bytes) in before {
        assert_eq!(after.get(&path), Some(&bytes), "{}", path.display());
    }
    core.commit_draft(&id, &draft.id, revision).unwrap();
    assert_eq!(effects(&core, &id, &item), *b);
    core.undo(&id, core.get_project(&id).unwrap().revision)
        .unwrap();
    assert_eq!(effects(&core, &id, &item), *a);
    core.redo(&id, core.get_project(&id).unwrap().revision)
        .unwrap();
    assert_eq!(effects(&core, &id, &item), *b);
    update(&core, &id, &item, &json!([]));
    assert_eq!(effects(&core, &id, &item), json!([]));
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let revision = core.get_project(&id).unwrap().revision;
    let batch:Vec<BatchEditOperation>=serde_json::from_value(json!([
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":8,"height":8,"color":"#ff0000","resultAlias":"leaf","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}},
        {"operation":"update_item","itemId":"@leaf","effects":a},
        {"operation":"update_item","itemId":"@leaf","effects":b}
    ])).unwrap();
    let result = core.edit_batch(&id, revision, batch).unwrap();
    let leaf = &result.aliases["leaf"];
    assert_eq!(effects(&core, &id, leaf), *b);
    assert_eq!(core.get_project(&id).unwrap().revision, revision + 1);
    let stable = inventory(&dir);
    let reopened = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    assert_eq!(effects(&reopened, &id, leaf), *b);
    assert_eq!(inventory(&dir), stable);
}

#[test]
fn ordered_effect_failures_preserve_complete_current_history_draft_and_resources() {
    let (_root, core, id, item) = setup();
    let f = fixture();
    update(&core, &id, &item, &f["orders"]["shadeThenWash"]);
    let dir = core.paths().project_dir(&id).unwrap();
    let revision = core.get_project(&id).unwrap().revision;
    let draft=core.create_draft(&id,revision,vec![op(json!({"operation":"update_item","itemId":item,"effects":f["orders"]["shadeThenWash"]}))],None).unwrap();
    let before = inventory(&dir);
    for case in f["invalidStacks"].as_array().unwrap() {
        let raw = json!({"operation":"update_item","itemId":item,"effects":case["value"]});
        let parsed = serde_json::from_value::<EditOperation>(raw.clone());
        if let Ok(operation) = parsed {
            assert_eq!(
                core.edit(&id, revision, operation).unwrap_err().code,
                ErrorCode::InvalidArgument,
                "{}",
                case["id"]
            );
        } else {
            assert!(serde_json::from_value::<EditOperation>(raw).is_err());
        }
        assert_eq!(inventory(&dir), before);
        let raw = json!([{"operation":"update_item","itemId":item,"effects":f["orders"]["washThenShade"]},{"operation":"update_item","itemId":item,"effects":case["value"]}]);
        if let Ok(batch) = serde_json::from_value::<Vec<BatchEditOperation>>(raw) {
            assert_eq!(
                core.edit_batch(&id, revision, batch).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(inventory(&dir), before);
        let ops = serde_json::from_value::<Vec<EditOperation>>(
            json!([{"operation":"update_item","itemId":item,"effects":f["orders"]["washThenShade"]},{"operation":"update_item","itemId":item,"effects":case["value"]}]),
        );
        if let Ok(ops) = ops {
            assert_eq!(
                core.update_draft(&id, &draft.id, revision, ops, None)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(inventory(&dir), before);
    }
    assert_eq!(
        core.edit(
            &id,
            revision,
            op(json!({"operation":"update_item","itemId":"missing","effects":[]}))
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        core.edit(
            &id,
            revision - 1,
            op(json!({"operation":"update_item","itemId":item,"effects":[]}))
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(inventory(&dir), before);
}

#[test]
fn sixteen_effects_and_utf8_id_boundary_accept_but_target_removal_or_retyping_is_atomic() {
    let (_root, core, id, item) = setup();
    let mut stack = (0..16)
        .map(|i| json!({"type":"vignette","id":format!("effect-{i}"),"amount":0}))
        .collect::<Vec<_>>();
    stack[0]["id"] = json!("é".repeat(64));
    update(&core, &id, &item, &json!(stack));
    assert_eq!(effects(&core, &id, &item).as_array().unwrap().len(), 16);
    assert_eq!(
        effects(&core, &id, &item)[0]["id"].as_str().unwrap().len(),
        128
    );
    update(&core, &id, &item, &fixture()["orders"]["shadeThenWash"]);
    let revision = core.get_project(&id).unwrap().revision;
    core.edit(&id, revision, op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"shade"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}))).unwrap();
    let revision = core.get_project(&id).unwrap().revision;
    let draft = core.create_draft(&id, revision, vec![op(json!({"operation":"update_item","itemId":item,"effects":fixture()["orders"]["washThenShade"]}))], None).unwrap();
    let before = inventory(&core.paths().project_dir(&id).unwrap());
    for (stack, code) in [
        (json!([]), ErrorCode::ItemNotFound),
        (
            json!([{"type":"gaussian_blur","id":"shade","radiusPx":1}]),
            ErrorCode::InvalidArgument,
        ),
    ] {
        let edit = op(json!({"operation":"update_item","itemId":item,"effects":stack}));
        assert_eq!(
            core.edit(&id, revision, edit.clone()).unwrap_err().code,
            code
        );
        assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
        let first = op(
            json!({"operation":"update_item","itemId":item,"effects":fixture()["orders"]["washThenShade"]}),
        );
        assert_eq!(
            core.edit_batch(&id, revision, vec![first.clone(), edit.clone()])
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
        assert_eq!(
            core.update_draft(&id, &draft.id, revision, vec![first, edit], None)
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
    }
}
