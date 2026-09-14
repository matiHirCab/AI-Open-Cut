use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, FontConfig, PathPolicy, Project, ProjectSettings,
};
use serde_json::{Value, json};

fn edit(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn add(track: &str, text: &str) -> EditOperation {
    edit(
        json!({"operation":"add_text","trackId":track,"text":text,"fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}),
    )
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let fonts = root.path().join("sources");
    std::fs::create_dir(&fonts).unwrap();
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/fonts")
                .join(name),
            fonts.join(name),
        )
        .unwrap();
    }
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    )
    .with_font_config(FontConfig {
        roots: vec![fonts.clone()],
        default_path: Some(fonts.join("DejaVuSans.ttf")),
    });
    let id = core
        .create_project("Draft fonts", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn change_source(root: &std::path::Path) {
    use std::io::Write;
    std::fs::OpenOptions::new()
        .append(true)
        .open(root.join("sources/DejaVuSans.ttf"))
        .unwrap()
        .write_all(&[0])
        .unwrap();
}

// Replacing a draft compares selectors with the previous draft action, while
// component application inherits from the base project. Exercise both views.
fn selector_reversion_case(
    selector_key: &str,
    explicit_base: bool,
    outcome: &str,
    reintroduce: bool,
) {
    use std::io::Write;
    let (root, core, id, track) = setup();
    let regular = root.path().join("sources/DejaVuSans.ttf");
    let original = std::fs::read(&regular).unwrap();
    let mut create = component_action(&core, &id, &track);
    if explicit_base {
        create["tracks"][0]["items"][0][selector_key] = if selector_key == "fontPath" {
            json!(regular)
        } else {
            json!("base family")
        };
    }
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let base = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut replacement = create;
    replacement["operation"] = json!("component_update");
    replacement["componentId"] = base["components"][0]["id"].clone();
    let mut previous = replacement.clone();
    if reintroduce {
        previous["tracks"][0]["items"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        previous["tracks"][0]["items"][0]["stackOrder"] = json!(0);
    } else {
        previous["tracks"][0]["items"][0][selector_key] = json!("draft selector");
    }
    change_source(root.path());
    let draft = core
        .create_draft(&id, 1, vec![edit(previous)], None)
        .unwrap();
    let old = local_bindings(&core, &id, &draft.id);
    if !reintroduce {
        assert_ne!(
            old["first"],
            base["components"][0]["tracks"][0]["items"][0]["fontBinding"]
        );
    }
    match outcome {
        "different" => change_source(root.path()),
        "styled" => {
            std::fs::write(&regular, &original).unwrap();
            std::fs::OpenOptions::new()
                .append(true)
                .open(root.path().join("sources/DejaVuSans-Bold.ttf"))
                .unwrap()
                .write_all(&[0])
                .unwrap();
        }
        "equal" => std::fs::write(&regular, &original).unwrap(),
        _ => unreachable!(),
    }
    let dir = core.project_directory(&id).unwrap();
    for attempt in 0..2 {
        replacement["name"] = json!(format!("Replacement {attempt}"));
        let before = snapshot(&dir, &draft.id);
        let fonts = owned_fonts(&dir);
        let result = core.update_draft(&id, &draft.id, 1, vec![edit(replacement.clone())], None);
        if outcome != "equal" {
            let error = result.unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(error.message.contains("inherited"), "{}", error.message);
            assert!(error.message.contains("version 2"), "{}", error.message);
            assert_eq!(before, snapshot(&dir, &draft.id));
            assert_eq!(fonts, owned_fonts(&dir));
            assert_eq!(old, local_bindings(&core, &id, &draft.id));
        } else {
            // Artificial clearing must never add a step that normal replay
            // would ignore because it already inherited the base binding.
            assert!(result.unwrap().font_steps.unwrap()[0].is_empty());
            let preview = local_bindings(&core, &id, &draft.id);
            assert_eq!(
                preview["first"],
                base["components"][0]["tracks"][0]["items"][0]["fontBinding"]
            );
            assert_eq!(preview["second"], old["second"]);
            let reopened = EditorCore::new(core.paths().clone());
            assert_eq!(preview, local_bindings(&reopened, &id, &draft.id));
        }
    }
    if outcome == "equal" {
        let preview = core.get_draft_state(&id, &draft.id).unwrap().project;
        let reopened = EditorCore::new(core.paths().clone());
        reopened.commit_draft(&id, &draft.id, 1).unwrap();
        assert_eq!(
            serde_json::to_value(preview).unwrap()["components"],
            serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap()["components"]
        );
    }
}

#[test]
fn component_selector_reversion_rejects_different_current_bindings_atomically() {
    for key in ["fontFamily", "fontPath"] {
        for explicit in [false, true] {
            for outcome in ["different", "styled"] {
                selector_reversion_case(key, explicit, outcome, false);
            }
        }
    }
}

#[test]
fn component_selector_reversion_accepts_equal_complete_bindings_through_commit() {
    for key in ["fontFamily", "fontPath"] {
        for explicit in [false, true] {
            selector_reversion_case(key, explicit, "equal", false);
        }
    }
}

#[test]
fn component_reintroduced_local_ids_require_current_resolution() {
    for outcome in ["different", "styled", "equal"] {
        selector_reversion_case("fontFamily", false, outcome, true);
    }
}

#[test]
fn component_matching_distinguishes_resolution_from_inheritance() {
    let (_root, core, id, track) = setup();
    let create = component_action(&core, &id, &track);
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut inherited = create;
    inherited["operation"] = json!("component_update");
    inherited["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    let mut changed = inherited.clone();
    changed["tracks"][0]["items"][0]["fontFamily"] = json!("draft selector");
    for reverse in [false, true] {
        let mut operations = vec![edit(inherited.clone()), edit(changed.clone())];
        if reverse {
            operations.reverse();
        }
        let draft = core.create_draft(&id, 1, operations, None).unwrap();
        let mut replacement = inherited.clone();
        replacement["name"] = json!("Edited replacement");
        let dir = core.project_directory(&id).unwrap();
        let before = snapshot(&dir, &draft.id);
        let fonts = owned_fonts(&dir);
        let error = core
            .update_draft(&id, &draft.id, 1, vec![edit(replacement)], None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(before, snapshot(&dir, &draft.id));
        assert_eq!(fonts, owned_fonts(&dir));
    }
}

fn prepared_prefix_case(changed: bool, intent_prefix: bool, reverse: bool, remove_source: bool) {
    let (root, core, id, track) = setup();
    let create = component_action(&core, &id, &track);
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut prefix = create;
    prefix["operation"] = json!("component_update");
    prefix["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    prefix["tracks"][0]["items"][0]["fontFamily"] = json!("inherited family");
    let mut previous = prefix.clone();
    previous["name"] = json!("old later action");
    if changed {
        previous["tracks"][0]["items"][0]["fontFamily"] = json!("previous family");
    }
    let mut previous_operations = vec![edit(prefix.clone()), edit(previous)];
    if intent_prefix {
        let mut third = prefix.clone();
        third["name"] = json!("old third action");
        previous_operations.push(edit(third));
    }
    let draft = core
        .create_draft(&id, 1, previous_operations, None)
        .unwrap();
    let expected = local_bindings(&core, &id, &draft.id);
    let mut first = prefix.clone();
    first["name"] = json!("replacement one");
    let mut second = prefix.clone();
    second["name"] = json!("replacement two");
    if intent_prefix {
        prefix["name"] = json!("edited prefix");
    }
    if reverse {
        std::mem::swap(&mut first, &mut second);
    }
    if remove_source {
        std::fs::remove_file(root.path().join("sources/DejaVuSans.ttf")).unwrap();
    } else if !changed {
        change_source(root.path());
    }
    let dir = core.project_directory(&id).unwrap();
    let before = snapshot(&dir, &draft.id);
    let fonts = owned_fonts(&dir);
    let mut operations = vec![edit(prefix), edit(first), edit(second.clone())];
    if !intent_prefix && !changed {
        for index in 0..4 {
            let mut next = second.clone();
            next["name"] = json!(format!("later chain {index}"));
            operations.push(edit(next));
        }
    }
    let result = core.update_draft(&id, &draft.id, 1, operations.clone(), None);
    if changed {
        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(before, snapshot(&dir, &draft.id));
        assert_eq!(fonts, owned_fonts(&dir));
    } else {
        result.unwrap();
        assert_eq!(expected, local_bindings(&core, &id, &draft.id));
        if !intent_prefix {
            operations.remove(1);
            operations[1..].reverse();
        }
        core.update_draft(&id, &draft.id, 1, operations, None)
            .unwrap();
        let reopened = EditorCore::new(core.paths().clone());
        assert_eq!(expected, local_bindings(&reopened, &id, &draft.id));
        let preview = reopened.get_draft_state(&id, &draft.id).unwrap().project;
        reopened.commit_draft(&id, &draft.id, 1).unwrap();
        assert_eq!(
            serde_json::to_value(preview).unwrap()["components"],
            serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap()["components"]
        );
    }
}

#[test]
fn prepared_prefix_allows_equivalent_inherited_replacements() {
    for reverse in [false, true] {
        for remove in [false, true] {
            prepared_prefix_case(false, false, reverse, remove);
            prepared_prefix_case(false, true, reverse, remove);
        }
    }
}

#[test]
fn prepared_prefix_rejects_resolution_versus_inheritance() {
    for reverse in [false, true] {
        prepared_prefix_case(true, false, reverse, false);
    }
}

#[test]
fn freshly_resolved_prefix_is_not_published_when_later_matching_is_ambiguous() {
    use std::io::Write;
    for reverse in [false, true] {
        let (root, core, id, track) = setup();
        let create = component_action(&core, &id, &track);
        core.edit(&id, 0, edit(create.clone())).unwrap();
        let mut action = create;
        action["operation"] = json!("component_update");
        action["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
        action["tracks"][0]["items"][0]["fontFamily"] = json!("old one");
        let mut other = action.clone();
        other["tracks"][0]["items"][0]["fontFamily"] = json!("old two");
        let draft = core
            .create_draft(&id, 1, vec![edit(action.clone()), edit(other)], None)
            .unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(root.path().join("sources/DejaVuSans-Bold.ttf"))
            .unwrap()
            .write_all(&[0])
            .unwrap();
        action["tracks"][0]["items"][0]["fontFamily"] = json!("new family");
        let operations = (0..3)
            .map(|index| {
                let mut next = action.clone();
                next["name"] = json!(format!(
                    "replacement {}",
                    if reverse { 2 - index } else { index }
                ));
                edit(next)
            })
            .collect();
        let dir = core.project_directory(&id).unwrap();
        let before = snapshot(&dir, &draft.id);
        let fonts = owned_fonts(&dir);
        let error = core
            .update_draft(&id, &draft.id, 1, operations, None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(before, snapshot(&dir, &draft.id));
        assert_eq!(fonts, owned_fonts(&dir));
    }
}
fn bindings(project: &Project) -> Vec<String> {
    project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter_map(|item| match item {
            opencut_editor_core::TimelineItem::Text(t) => {
                Some(t.font_binding.as_ref().unwrap().regular.clone())
            }
            _ => None,
        })
        .collect()
}
fn snapshot(dir: &std::path::Path, draft: &str) -> Vec<Vec<u8>> {
    [
        "project.json".to_owned(),
        "history.json".to_owned(),
        format!("drafts/{draft}.json"),
    ]
    .iter()
    .map(|p| std::fs::read(dir.join(p)).unwrap_or_default())
    .collect()
}

fn owned_fonts(dir: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    std::fs::read_dir(dir.join("fonts"))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                std::fs::read(path).unwrap(),
            )
        })
        .collect()
}

fn component_action(core: &EditorCore, id: &str, track: &str) -> Value {
    let seed = core
        .create_draft(id, 0, vec![add(track, "seed")], None)
        .unwrap();
    let state = core.get_draft_state(id, &seed.id).unwrap().project;
    let mut first = serde_json::to_value(&state.tracks[1].items[0]).unwrap();
    first.as_object_mut().unwrap().remove("fontBinding");
    first["id"] = json!("first");
    let mut second = first.clone();
    second["id"] = json!("second");
    second["stackOrder"] = json!(1);
    json!({"operation":"component_create","name":"Component","width":320,"height":180,"durationMs":1000,"tracks":[{"id":"local","name":"Text","trackType":"overlay","items":[first,second]}],"slots":[]})
}

fn local_bindings(
    core: &EditorCore,
    id: &str,
    draft: &str,
) -> std::collections::BTreeMap<String, Value> {
    let state = serde_json::to_value(core.get_draft_state(id, draft).unwrap().project).unwrap();
    state["components"][0]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["items"].as_array().unwrap())
        .map(|t| {
            (
                t["id"].as_str().unwrap().to_owned(),
                t["fontBinding"].clone(),
            )
        })
        .collect()
}

#[test]
fn one_old_font_action_cannot_choose_between_two_replacements() {
    for reverse in [false, true] {
        let (root, core, id, track) = setup();
        change_source(root.path());
        let draft = core
            .create_draft(&id, 0, vec![add(&track, "old")], None)
            .unwrap();
        let reopened = EditorCore::new(core.paths().clone());
        let dir = core.project_directory(&id).unwrap();
        let before = snapshot(&dir, &draft.id);
        let fonts = owned_fonts(&dir);
        let mut actions = vec![add(&track, "new"), add(&track, "old edited")];
        if reverse {
            actions.reverse();
        }
        let error = reopened
            .update_draft(&id, &draft.id, 0, actions, None)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("ambiguous"));
        assert_eq!(before, snapshot(&dir, &draft.id));
        assert_eq!(fonts, owned_fonts(&dir));
    }
}

#[test]
fn component_selector_edit_preserves_untouched_local_fonts() {
    for update in [false, true] {
        let (root, core, id, track) = setup();
        let mut action = component_action(&core, &id, &track);
        let revision = if update {
            core.edit(&id, 0, edit(action.clone())).unwrap();
            action["operation"] = json!("component_update");
            action["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
            action["tracks"][0]["items"][0]["fontFamily"] = json!("old family");
            action["tracks"][0]["items"][1]["fontFamily"] = json!("old family");
            1
        } else {
            0
        };
        change_source(root.path());
        let draft = core
            .create_draft(&id, revision, vec![edit(action.clone())], None)
            .unwrap();
        let before = local_bindings(&core, &id, &draft.id);
        std::fs::remove_file(root.path().join("sources/DejaVuSans.ttf")).unwrap();
        let reopened = EditorCore::new(core.paths().clone());
        action["tracks"][0]["items"][1]["fontFamily"] = json!("new family");
        reopened
            .update_draft(&id, &draft.id, revision, vec![edit(action)], None)
            .unwrap();
        let after = local_bindings(&reopened, &id, &draft.id);
        assert_eq!(before["first"], after["first"]);
        assert_ne!(before["second"], after["second"]);
        let reopened = EditorCore::new(core.paths().clone());
        assert_eq!(after, local_bindings(&reopened, &id, &draft.id));
        let preview = serde_json::to_value(
            reopened
                .get_draft_state(&id, &draft.id)
                .unwrap()
                .project
                .components[0]
                .tracks
                .clone(),
        )
        .unwrap();
        reopened.commit_draft(&id, &draft.id, revision).unwrap();
        assert_eq!(
            preview,
            serde_json::to_value(
                reopened.get_project(&id).unwrap().components[0]
                    .tracks
                    .clone()
            )
            .unwrap()
        );
    }
}

#[test]
fn component_shared_selector_must_be_representable_in_draft_v2() {
    for changed_default in [false, true] {
        let (root, core, id, track) = setup();
        let mut action = component_action(&core, &id, &track);
        if changed_default {
            change_source(root.path());
        }
        let draft = core
            .create_draft(&id, 0, vec![edit(action.clone())], None)
            .unwrap();
        let before = local_bindings(&core, &id, &draft.id);
        let reopened = EditorCore::new(core.paths().clone());
        action["tracks"][0]["items"][1]["id"] = json!("new-child");
        let dir = core.project_directory(&id).unwrap();
        let bytes = snapshot(&dir, &draft.id);
        let fonts = owned_fonts(&dir);
        let result = reopened.update_draft(&id, &draft.id, 0, vec![edit(action)], None);
        if changed_default {
            let error = result.unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(error.message.contains("selector"));
            assert_eq!(bytes, snapshot(&dir, &draft.id));
            assert_eq!(fonts, owned_fonts(&dir));
        } else {
            assert_eq!(result.unwrap().version, 2);
            let after = local_bindings(&reopened, &id, &draft.id);
            assert_eq!(before["first"], after["first"]);
            assert_eq!(after["first"], after["new-child"]);
            reopened.commit_draft(&id, &draft.id, 0).unwrap();
        }
    }
}

#[test]
fn component_local_retention_survives_structure_and_non_font_edits() {
    let (root, core, id, track) = setup();
    let mut action = component_action(&core, &id, &track);
    change_source(root.path());
    let draft = core
        .create_draft(&id, 0, vec![edit(action.clone())], None)
        .unwrap();
    let before = local_bindings(&core, &id, &draft.id);
    let reopened = EditorCore::new(core.paths().clone());
    let mut moved = action["tracks"][0]["items"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    action["tracks"][0]["items"][0]["stackOrder"] = json!(0);
    moved["text"] = json!("edited");
    moved["document"] =
        json!({"runs":[{"text":"edited","bold":true,"italic":true,"color":"#ff0000"}]});
    moved["color"] = json!("#00ff00");
    moved["fontSize"] = json!(32);
    moved["startMs"] = json!(10);
    moved["durationMs"] = json!(900);
    moved["transform"]["positionX"] = json!(12);
    action["tracks"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"moved","name":"Moved","trackType":"overlay","items":[moved]}));
    action["tracks"].as_array_mut().unwrap().reverse();
    reopened
        .update_draft(&id, &draft.id, 0, vec![edit(action.clone())], None)
        .unwrap();
    assert_eq!(before, local_bindings(&reopened, &id, &draft.id));
    // Remove one local identity, then introduce an unrelated selector.
    action["tracks"].as_array_mut().unwrap().pop();
    reopened
        .update_draft(&id, &draft.id, 0, vec![edit(action.clone())], None)
        .unwrap();
    assert_eq!(
        before["first"],
        local_bindings(&reopened, &id, &draft.id)["first"]
    );
    let mut new_child = action["tracks"][0]["items"][0].clone();
    new_child["id"] = json!("new-child");
    new_child["fontFamily"] = json!("new family");
    new_child["stackOrder"] = json!(1);
    action["tracks"][0]["items"]
        .as_array_mut()
        .unwrap()
        .push(new_child);
    reopened
        .update_draft(&id, &draft.id, 0, vec![edit(action)], None)
        .unwrap();
    let after = local_bindings(&reopened, &id, &draft.id);
    assert_eq!(before["first"], after["first"]);
    assert_ne!(before["first"]["regular"], after["new-child"]["regular"]);
}

#[test]
fn component_update_retention_is_scoped_across_identical_local_ids() {
    let (root, core, id, track) = setup();
    let create = component_action(&core, &id, &track);
    core.edit_batch(&id, 0, vec![edit(create.clone()), edit(create.clone())])
        .unwrap();
    let project = core.get_project(&id).unwrap();
    let actions: Vec<_> = project
        .components
        .iter()
        .map(|component| {
            let mut update = create.clone();
            update["operation"] = json!("component_update");
            update["componentId"] = json!(component.id);
            for item in update["tracks"][0]["items"].as_array_mut().unwrap() {
                item["fontFamily"] = json!("draft family");
            }
            update
        })
        .collect();
    let draft = core
        .create_draft(&id, 1, vec![edit(actions[0].clone())], None)
        .unwrap();
    change_source(root.path());
    core.update_draft(
        &id,
        &draft.id,
        1,
        actions.iter().cloned().map(edit).collect(),
        None,
    )
    .unwrap();
    let before =
        serde_json::to_value(core.get_draft_state(&id, &draft.id).unwrap().project).unwrap();
    let mut actions = actions;
    for action in &mut actions {
        action["tracks"][0]["items"][1]["fontFamily"] = json!("changed family");
    }
    actions.reverse();
    let reopened = EditorCore::new(core.paths().clone());
    reopened
        .update_draft(
            &id,
            &draft.id,
            1,
            actions.into_iter().map(edit).collect(),
            None,
        )
        .unwrap();
    let after =
        serde_json::to_value(reopened.get_draft_state(&id, &draft.id).unwrap().project).unwrap();
    for index in 0..2 {
        assert_eq!(
            before["components"][index]["tracks"][0]["items"][0]["fontBinding"],
            after["components"][index]["tracks"][0]["items"][0]["fontBinding"]
        );
    }
    assert_ne!(
        after["components"][0]["tracks"][0]["items"][0]["fontBinding"]["regular"],
        after["components"][1]["tracks"][0]["items"][0]["fontBinding"]["regular"]
    );
}

#[test]
fn component_matching_compares_actual_local_bindings_not_shared_selector_steps() {
    let (root, core, id, track) = setup();
    let create = component_action(&core, &id, &track);
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut update = create;
    update["operation"] = json!("component_update");
    update["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    // The first child stays bound to the base project. Only the second child
    // contributes a newly resolved selector step in each replacement.
    update["tracks"][0]["items"][1]["fontFamily"] = json!("family one");
    let draft = core
        .create_draft(&id, 1, vec![edit(update.clone())], None)
        .unwrap();
    change_source(root.path());
    let mut later = update.clone();
    later["tracks"][0]["items"][1]["fontFamily"] = Value::Null;
    core.update_draft(
        &id,
        &draft.id,
        1,
        vec![edit(update.clone()), edit(later)],
        None,
    )
    .unwrap();
    // Both old operations yield the same retained first child, while the
    // second child's selector changes. Either match is therefore equivalent.
    update["tracks"][0]["items"][1]["fontFamily"] = json!("new family");
    core.update_draft(&id, &draft.id, 1, vec![edit(update)], None)
        .unwrap();
    let bindings = local_bindings(&core, &id, &draft.id);
    assert_ne!(bindings["first"]["regular"], bindings["second"]["regular"]);
}

#[test]
fn inherited_component_binding_conflicts_fail_before_font_publication() {
    let (root, core, id, track) = setup();
    let mut create = component_action(&core, &id, &track);
    create["tracks"][0]["items"][0]["fontFamily"] = json!("base family");
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut original = create;
    original["operation"] = json!("component_update");
    original["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    original["name"] = json!("Original draft");
    original["tracks"][0]["items"][0]["fontFamily"] = Value::Null;
    change_source(root.path());
    let draft = core
        .create_draft(&id, 1, vec![edit(original.clone())], None)
        .unwrap();
    change_source(root.path());
    let mut first = original.clone();
    first["name"] = json!("First inserted action");
    first["tracks"][0]["items"][0]["fontFamily"] = json!("temporary selector");
    let mut second = original.clone();
    second["name"] = json!("Second inserted action");
    let dir = core.project_directory(&id).unwrap();
    let before = snapshot(&dir, &draft.id);
    let fonts = owned_fonts(&dir);
    let error = core
        .update_draft(
            &id,
            &draft.id,
            1,
            vec![edit(first), edit(second), edit(original)],
            None,
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("selector"));
    assert_eq!(before, snapshot(&dir, &draft.id));
    assert_eq!(fonts, owned_fonts(&dir));
}

#[test]
fn component_actions_without_new_bindings_have_equivalent_empty_retention() {
    let (_root, core, id, track) = setup();
    let create = component_action(&core, &id, &track);
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut update = create;
    update["operation"] = json!("component_update");
    update["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    let draft = core
        .create_draft(&id, 1, vec![edit(update.clone())], None)
        .unwrap();
    let before = local_bindings(&core, &id, &draft.id);
    update["name"] = json!("Renamed");
    core.update_draft(
        &id,
        &draft.id,
        1,
        vec![edit(update.clone()), edit(update)],
        None,
    )
    .unwrap();
    assert_eq!(before, local_bindings(&core, &id, &draft.id));
}

#[test]
fn component_replacement_retains_fonts_inherited_from_removed_draft_actions() {
    let (root, core, id, track) = setup();
    let mut create = component_action(&core, &id, &track);
    create["tracks"][0]["items"][0]["fontFamily"] = json!("base family");
    core.edit(&id, 0, edit(create.clone())).unwrap();
    let mut first = create;
    first["operation"] = json!("component_update");
    first["componentId"] = json!(core.get_project(&id).unwrap().components[0].id);
    first["tracks"][0]["items"][0]["fontFamily"] = Value::Null;
    let mut later = first.clone();
    later["name"] = json!("Later action");
    change_source(root.path());
    let draft = core
        .create_draft(&id, 1, vec![edit(first), edit(later.clone())], None)
        .unwrap();
    let before = local_bindings(&core, &id, &draft.id);
    let reopened = EditorCore::new(core.paths().clone());
    reopened
        .update_draft(&id, &draft.id, 1, vec![edit(later)], None)
        .unwrap();
    assert_eq!(before, local_bindings(&reopened, &id, &draft.id));
}

#[test]
fn component_draft_bindings_use_scoped_local_identity() {
    let (root, core, id, track) = setup();
    let mut root_add = serde_json::to_value(add(&track, "root")).unwrap();
    root_add["fontFamily"] = json!("missing family");
    core.edit(&id, 0, edit(root_add)).unwrap();
    let mut text =
        serde_json::to_value(&core.get_project(&id).unwrap().tracks[1].items[0]).unwrap();
    text.as_object_mut().unwrap().remove("fontBinding");
    let mut second = text.clone();
    second["id"] = json!("second");
    second["stackOrder"] = json!(1);
    text["fontFamily"] = Value::Null;
    let create = json!({"operation":"component_create","name":"Component","width":320,"height":180,"durationMs":1000,"tracks":[{"id":"local","name":"Text","trackType":"overlay","items":[text,second]}],"slots":[]});
    core.edit(&id, 1, edit(create.clone())).unwrap();
    core.edit(&id, 2, edit(create)).unwrap();
    let before = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    change_source(root.path());
    let mut component = before["components"][0].clone();
    component["tracks"][0]["items"][0]["fontFamily"] = json!("missing family");
    for item in component["tracks"][0]["items"].as_array_mut().unwrap() {
        item.as_object_mut().unwrap().remove("fontBinding");
    }
    let operation = edit(
        json!({"operation":"component_update","componentId":component["id"],"name":component["name"],"width":320,"height":180,"durationMs":1000,"tracks":component["tracks"],"slots":[]}),
    );
    let draft = core
        .create_draft(&id, 3, vec![operation.clone()], None)
        .unwrap();
    let state =
        serde_json::to_value(core.get_draft_state(&id, &draft.id).unwrap().project).unwrap();
    assert_ne!(
        state["components"][0]["tracks"][0]["items"][0]["fontBinding"],
        before["components"][0]["tracks"][0]["items"][0]["fontBinding"]
    );
    assert_eq!(
        state["components"][0]["tracks"][0]["items"][1]["fontBinding"],
        before["components"][0]["tracks"][0]["items"][1]["fontBinding"]
    );
    assert_eq!(state["components"][1], before["components"][1]);
    assert_eq!(state["tracks"], before["tracks"]);
    let reopened = EditorCore::new(core.paths().clone());
    let mut changed = serde_json::to_value(operation).unwrap();
    changed["tracks"][0]["items"][0]["text"] = json!("edited");
    changed["tracks"][0]["items"][0]["document"] =
        json!({"runs":[{"text":"edited","italic":true}]});
    reopened
        .update_draft(
            &id,
            &draft.id,
            3,
            vec![
                edit(json!({"operation":"update_track","trackId":track,"name":"Changed"})),
                edit(changed),
            ],
            None,
        )
        .unwrap();
    let preview =
        serde_json::to_value(reopened.get_draft_state(&id, &draft.id).unwrap().project).unwrap();
    assert_eq!(
        preview["components"][0]["tracks"][0]["items"][0]["fontBinding"],
        state["components"][0]["tracks"][0]["items"][0]["fontBinding"]
    );
    reopened.commit_draft(&id, &draft.id, 3).unwrap();
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap()["components"],
        preview["components"]
    );
}

#[test]
fn draft_reset_does_not_capture_another_items_old_binding() {
    let (root, core, id, track) = setup();
    core.edit_batch(&id, 0, vec![add(&track, "first"), add(&track, "second")])
        .unwrap();
    let before = core.get_project(&id).unwrap();
    change_source(root.path());
    let reset = edit(
        json!({"operation":"update_item","itemId":before.tracks[1].items[0].id(),"fontFamily":null}),
    );
    let draft = core.create_draft(&id, 1, vec![reset], None).unwrap();
    let state = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_ne!(bindings(&state)[0], bindings(&before)[0]);
    assert_eq!(bindings(&state)[1], bindings(&before)[1]);
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        bindings(&state),
        bindings(&reopened.get_draft_state(&id, &draft.id).unwrap().project)
    );
    reopened.commit_draft(&id, &draft.id, 1).unwrap();
    assert_eq!(
        bindings(&state),
        bindings(&reopened.get_project(&id).unwrap())
    );
}

#[test]
fn draft_reordering_and_non_font_edits_retain_bindings() {
    let (root, core, id, track) = setup();
    change_source(root.path());
    let initial = add(&track, "first");
    let draft = core
        .create_draft(&id, 0, vec![initial.clone()], None)
        .unwrap();
    let expected = bindings(&core.get_draft_state(&id, &draft.id).unwrap().project);
    std::fs::remove_file(root.path().join("sources/DejaVuSans.ttf")).unwrap();
    let reopened = EditorCore::new(core.paths().clone());
    let unrelated = edit(json!({"operation":"update_track","trackId":track,"name":"Renamed"}));
    for operations in [
        vec![unrelated.clone(), initial.clone()],
        vec![initial.clone(), unrelated],
        vec![add(&track, "changed")],
    ] {
        reopened
            .update_draft(&id, &draft.id, 0, operations, None)
            .unwrap();
        assert_eq!(
            expected,
            bindings(&reopened.get_draft_state(&id, &draft.id).unwrap().project)
        );
    }
    let mut styled = serde_json::to_value(add(&track, "unused")).unwrap();
    styled.as_object_mut().unwrap().remove("text");
    styled["document"] = json!({"runs":[{"text":"styled","bold":true,"color":"#ff0000"}]});
    styled["fontSize"] = json!(30);
    reopened
        .update_draft(&id, &draft.id, 0, vec![edit(styled)], None)
        .unwrap();
    assert_eq!(
        expected,
        bindings(&reopened.get_draft_state(&id, &draft.id).unwrap().project)
    );
    let mut changed_selector = serde_json::to_value(add(&track, "changed")).unwrap();
    changed_selector["fontFamily"] = json!("missing family");
    reopened
        .update_draft(&id, &draft.id, 0, vec![edit(changed_selector)], None)
        .unwrap();
    assert_ne!(
        expected,
        bindings(&reopened.get_draft_state(&id, &draft.id).unwrap().project)
    );
}

#[test]
fn ambiguous_draft_matching_is_atomic_and_equivalent_matches_succeed() {
    let (root, core, id, track) = setup();
    let first = add(&track, "first");
    let second = add(&track, "second");
    let draft = core
        .create_draft(&id, 0, vec![first.clone()], None)
        .unwrap();
    change_source(root.path());
    core.update_draft(&id, &draft.id, 0, vec![first.clone(), second.clone()], None)
        .unwrap();
    let state = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_ne!(bindings(&state)[0], bindings(&state)[1]);
    // Exact matches must win before broad intent matching, even after reorder.
    core.update_draft(&id, &draft.id, 0, vec![second, first], None)
        .unwrap();
    assert_eq!(
        bindings(&core.get_draft_state(&id, &draft.id).unwrap().project),
        bindings(&state).into_iter().rev().collect::<Vec<_>>()
    );
    let dir = core.project_directory(&id).unwrap();
    let before = snapshot(&dir, &draft.id);
    let files = std::fs::read_dir(dir.join("fonts")).unwrap().count();
    let error = core
        .update_draft(
            &id,
            &draft.id,
            0,
            vec![add(&track, "edited1"), add(&track, "edited2")],
            None,
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("ambiguous"));
    assert_eq!(before, snapshot(&dir, &draft.id));
    assert_eq!(files, std::fs::read_dir(dir.join("fonts")).unwrap().count());
    let equivalent = core
        .create_draft(&id, 0, vec![add(&track, "same"), add(&track, "same")], None)
        .unwrap();
    core.update_draft(
        &id,
        &equivalent.id,
        0,
        vec![add(&track, "changed1"), add(&track, "changed2")],
        None,
    )
    .unwrap();
    let hashes = bindings(&core.get_draft_state(&id, &equivalent.id).unwrap().project);
    assert_eq!(hashes[0], hashes[1]);
}
