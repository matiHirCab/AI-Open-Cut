use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};

fn fixture() -> Value {
    let mut f: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    for stack in f["nativeWitness"]["orders"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        let typed: Vec<opencut_editor_core::VisualEffect> =
            serde_json::from_value(stack.clone()).unwrap();
        *stack = serde_json::to_value(typed).unwrap();
    }
    json!({"source":f["nativeWitness"]["source"],"orders":{"shadeThenWash":f["nativeWitness"]["orders"]["gradeThenTint"],"washThenShade":f["nativeWitness"]["orders"]["tintThenGrade"]},"invalidStacks":f["invalidStacks"]})
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
            "Parameterized effects",
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
fn parameterized_effect_public_lifecycle_preserves_exact_arrays_aliases_drafts_history_and_reopen()
{
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
fn parameterized_effect_failures_preserve_complete_current_history_draft_and_resources() {
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
fn parameterized_catalog_endpoints_and_nonfinite_controls_are_validated_by_core() {
    let f: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    for case in f["effectCases"].as_array().unwrap() {
        let (_root, core, id, item) = setup();
        let operation = serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":item,"effects":[case["value"]]}),
        );
        let accepted = operation.is_ok_and(|op| core.edit(&id, 1, op).is_ok());
        assert_eq!(
            accepted,
            case["accepted"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
    }
    for field in 0..3 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let (_root, core, id, item) = setup();
            let mut operation = op(
                json!({"operation":"update_item","itemId":item,"effects":[f["nativeWitness"]["identity"]]}),
            );
            if let EditOperation::UpdateItem {
                effects: Some(effects),
                ..
            } = &mut operation
            {
                if let opencut_editor_core::VisualEffect::ColorAdjustment {
                    exposure_stops,
                    contrast,
                    saturation,
                    ..
                } = &mut effects[0]
                {
                    *[exposure_stops, contrast, saturation][field] = value;
                } else {
                    panic!("canonical color variant missing");
                }
            } else {
                panic!("canonical edit missing effects");
            }
            let before = inventory(&core.paths().project_dir(&id).unwrap());
            assert_eq!(
                core.edit(&id, 1, operation).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
        }
    }
}

#[test]
fn color_stack_limits_and_incompatible_animation_targets_are_atomic() {
    let (_root, core, id, item) = setup();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    let mut stack: Vec<Value> = (0..16)
        .map(|i| {
            let mut effect = catalog["nativeWitness"]["identity"].clone();
            effect["id"] = json!(format!("grade-{i}"));
            effect
        })
        .collect();
    stack[0]["id"] = json!("é".repeat(64));
    update(&core, &id, &item, &json!(stack));
    let revision = core.get_project(&id).unwrap().revision;
    let before = inventory(&core.paths().project_dir(&id).unwrap());
    let channel = json!({"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"grade-1"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]});
    assert_eq!(core.edit(&id, revision, op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel]}))).unwrap_err().code, ErrorCode::InvalidArgument);
    assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
    update(
        &core,
        &id,
        &item,
        &json!([{"type":"vignette","id":"shade","amount":0.5}]),
    );
    let revision = core.get_project(&id).unwrap().revision;
    let channel = json!({"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"shade"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]});
    core.edit(&id, revision, op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel]}))).unwrap();
    let revision = core.get_project(&id).unwrap().revision;
    let mut grade = catalog["nativeWitness"]["identity"].clone();
    grade["id"] = json!("shade");
    let before = inventory(&core.paths().project_dir(&id).unwrap());
    let replacement = op(json!({"operation":"update_item","itemId":item,"effects":[grade]}));
    assert_eq!(
        core.edit(&id, revision, replacement.clone())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        core.create_draft(&id, revision, vec![replacement], None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let group = core
        .edit(
            &id,
            revision,
            op(json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":800})),
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let revision = core.get_project(&id).unwrap().revision;
    // Schema37 deliberately enables these existing effects on controlled groups.
    let typed: Vec<opencut_editor_core::VisualEffect> = serde_json::from_value(json!([
        catalog["nativeWitness"]["identity"],
        {"type":"vignette","id":"group-shade","amount":0.5}
    ]))
    .unwrap();
    let stack = serde_json::to_value(typed).unwrap();
    core.edit(
        &id,
        revision,
        op(json!({"operation":"update_item","itemId":group,"effects":stack})),
    )
    .unwrap();
    assert_eq!(effects(&core, &id, &group), stack);
    let revision = core.get_project(&id).unwrap().revision;
    let before = inventory(&core.paths().project_dir(&id).unwrap());
    for unsupported in [
        json!({"operation":"update_item","itemId":group,"matteOnly":true}),
        json!({"operation":"update_item","itemId":group,"blendMode":"multiply"}),
        json!({"operation":"set_animation_channels","itemId":group,"animationChannels":[{"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"group-shade"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}),
    ] {
        assert_eq!(
            core.edit(&id, revision, op(unsupported)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
        assert_eq!(effects(&core, &id, &group), stack);
    }
    assert_eq!(inventory(&core.paths().project_dir(&id).unwrap()), before);
}

#[test]
fn color_overflow_rejects_hidden_and_unused_components_before_publication() {
    use opencut_editor_core::{TimelineItem, VisualEffect};
    for hidden in [false, true] {
        let (_root, core, id, _) = setup();
        let track = core.get_project(&id).unwrap().tracks[1].id.clone();
        let item = core.edit(&id, 1, op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":3000,"height":3000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
        let stack: Vec<_> = (0..16)
            .map(|i| VisualEffect::ColorAdjustment {
                id: format!("grade-{i}"),
                exposure_stops: 0.0,
                contrast: 1.0,
                saturation: 1.0,
            })
            .collect();
        let revision = core.get_project(&id).unwrap().revision;
        if hidden {
            core.edit(
                &id,
                revision,
                op(json!({"operation":"update_item","itemId":item,"hidden":true})),
            )
            .unwrap();
        }
        let revision = core.get_project(&id).unwrap().revision;
        let dir = core.paths().project_dir(&id).unwrap();
        let before = inventory(&dir);
        assert_eq!(
            core.edit(
                &id,
                revision,
                op(json!({"operation":"update_item","itemId":item,"effects":stack}))
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(inventory(&dir), before);
        let mut project = core.get_project(&id).unwrap();
        let rectangle = project.tracks[1]
            .items
            .iter_mut()
            .find(|leaf| leaf.id() == item)
            .unwrap();
        assert!(matches!(rectangle, TimelineItem::Rectangle(_)));
        rectangle.visual_properties_mut().effects = stack;
        let retained_leaf = serde_json::to_value(rectangle).unwrap();
        let component = op(
            json!({"operation":"component_create","name":"Unused overbudget color","width":64,"height":64,"durationMs":800,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[retained_leaf]}]}),
        );
        assert_eq!(
            core.edit(&id, revision, component).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(inventory(&dir), before);
    }
}
