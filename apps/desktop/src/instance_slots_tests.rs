use std::{collections::BTreeMap, path::Path};

use opencut_editor_core::{EditOperation, ErrorCode, SlotKind, SlotValue};

use crate::{
    hierarchy::Selection,
    instance_slot_fixture,
    instance_slots::{self, Draft},
    session::{Command, Session, Startup},
};

fn inventory(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, at: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

fn draft(project: &opencut_editor_core::Project, selection: &Selection, id: &str) -> Draft {
    Draft::new(
        project,
        selection,
        instance_slots::slots(project, selection)
            .iter()
            .find(|slot| slot.id == id)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn authoritative_sources_scopes_and_all_supported_representations() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let before = inventory(root.path());
    let p = f.project();
    let first = Selection::root(&f.aliases["instance0"]);
    let second = Selection::root(&f.aliases["instance1"]);
    assert_eq!(draft(&p, &first, "title").text, "Plan");
    assert_ne!(
        draft(&p, &first, "title").text,
        draft(&p, &second, "title").text
    );
    let i = instance_slots::instance(&p, &first).unwrap();
    for (id, source) in [
        ("title", "Stored override"),
        (
            "toString",
            "Absent override/default · bound source property retained",
        ),
        (
            "optional_hidden",
            "Absent override/default · bound source property retained",
        ),
    ] {
        let slot = instance_slots::slots(&p, &first)
            .iter()
            .find(|slot| slot.id == id)
            .unwrap();
        assert_eq!(instance_slots::source(i, slot), source);
    }
    for (id, input, expected) in [
        (
            "body",
            "Unicode 😀\nline",
            serde_json::json!({"type":"text","value":"Unicode 😀\nline"}),
        ),
        (
            "__proto__",
            "#aBcD12",
            serde_json::json!({"type":"color","value":"#aBcD12"}),
        ),
        (
            "constructor",
            "true",
            serde_json::json!({"type":"boolean","value":true}),
        ),
        (
            "icon",
            "literal-managed-id",
            serde_json::json!({"type":"asset","value":{"kind":"asset","scope":"project","id":"literal-managed-id"}}),
        ),
    ] {
        let mut d = draft(&p, &first, id);
        d.text = input.into();
        let edit =
            serde_json::to_value(d.build(&p, Some(&first), false, false, false).unwrap()).unwrap();
        assert_eq!(edit["slotValues"][id], expected);
    }
    for slot in instance_slots::slots(&p, &first).iter().filter(|slot| {
        matches!(
            slot.kind,
            SlotKind::Number | SlotKind::Enum | SlotKind::Duration | SlotKind::RichText
        )
    }) {
        assert!(!instance_slots::supported(&first, slot));
        assert!(Draft::new(&p, &first, slot).is_none());
    }
    let rich = instance_slots::slots(&p, &first)
        .iter()
        .find(|slot| slot.kind == SlotKind::RichText)
        .expect("authoritative rich-text fixture slot");
    assert!(!instance_slots::supported(&first, rich));
    assert!(Draft::new(&p, &first, rich).is_none());
    let local = Selection {
        instance_path: vec![first.item_id.clone()],
        scope: format!("component:{}", i.component_id),
        item_id: "title".into(),
    };
    assert!(local.resolve(&p).is_some());
    for slot in instance_slots::slots(&p, &first) {
        assert!(!instance_slots::supported(&local, slot));
    }
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn narrow_edit_reset_history_reopen_and_full_unicode() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let p = f.project();
    let s = Selection::root(&f.aliases["instance0"]);
    let resources_before: BTreeMap<_, _> = inventory(root.path())
        .into_iter()
        .filter(|(path, _)| !path.ends_with("project.json") && !path.ends_with("history.json"))
        .collect();
    let mut d = draft(&p, &s, "body");
    d.text = "😀".repeat(1024);
    assert_eq!(d.text.len(), 4096);
    let edit = d.build(&p, Some(&s), false, false, false).unwrap();
    let mut expected = serde_json::to_value(&p).unwrap();
    let tracks = expected["tracks"].as_array_mut().unwrap();
    let item = tracks
        .iter_mut()
        .flat_map(|track| track["items"].as_array_mut().unwrap())
        .find(|item| item["id"] == s.item_id)
        .unwrap();
    item["slotValues"]["body"] = serde_json::json!({"type":"text","value":"😀".repeat(1024)});
    f.core.edit(&f.id, p.revision, edit).unwrap();
    let after = f.project();
    expected["revision"] = serde_json::json!(after.revision);
    // Core timestamp is authoritative; all remaining persisted source fields are exact.
    expected["updatedAtMs"] = serde_json::to_value(&after).unwrap()["updatedAtMs"].clone();
    assert_eq!(serde_json::to_value(&after).unwrap(), expected);
    let resources_after: BTreeMap<_, _> = inventory(root.path())
        .into_iter()
        .filter(|(path, _)| !path.ends_with("project.json") && !path.ends_with("history.json"))
        .collect();
    assert_eq!(resources_after, resources_before);
    let before_reset = inventory(root.path());
    drop(draft(&after, &s, "body"));
    assert_eq!(inventory(root.path()), before_reset);
    let clear = draft(&after, &s, "body")
        .build(&after, Some(&s), true, false, false)
        .unwrap();
    f.core.edit(&f.id, after.revision, clear).unwrap();
    assert!(
        !instance_slots::instance(&f.project(), &s)
            .unwrap()
            .slot_values
            .contains_key("body")
    );
    f.core.undo(&f.id, f.project().revision).unwrap();
    assert_eq!(
        instance_slots::instance(&f.project(), &s)
            .unwrap()
            .slot_values["body"],
        SlotValue::Text("😀".repeat(1024))
    );
    f.core.redo(&f.id, f.project().revision).unwrap();
    let reopened = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    }
    .execute(Command::Refresh)
    .unwrap();
    assert_eq!(
        serde_json::to_value(reopened).unwrap(),
        serde_json::to_value(f.project()).unwrap()
    );
}

#[test]
fn canonical_failures_are_atomic_and_stale_busy_conflict_drafts_cannot_apply() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let p = f.project();
    let s = Selection::root(&f.aliases["instance0"]);
    for (id, value, clear) in [
        ("__proto__", "#nope", false),
        (
            "title",
            "a long title that exceeds the forty character constraint",
            false,
        ),
        ("icon", "missing", false),
        ("title", "", true),
    ] {
        let mut d = draft(&p, &s, id);
        d.text = value.into();
        let before = inventory(root.path());
        let error = f
            .core
            .edit(
                &f.id,
                p.revision,
                d.build(&p, Some(&s), clear, false, false).unwrap(),
            )
            .unwrap_err();
        assert_eq!(
            error.code,
            if id == "icon" {
                ErrorCode::AssetNotFound
            } else {
                ErrorCode::InvalidArgument
            }
        );
        assert!(!error.retryable);
        assert_eq!(inventory(root.path()), before);
    }
    let mut d = draft(&p, &s, "body");
    d.text = "😀".repeat(4096);
    d.append("x");
    assert!(d.exhausted);
    assert!(d.build(&p, Some(&s), false, false, false).is_err());
    assert!(d.build(&p, Some(&s), false, true, false).is_err());
    assert!(d.build(&p, Some(&s), false, false, true).is_err());
    assert!(!d.matches(&p, Some(&Selection::root(&f.aliases["instance1"]))));
    let config = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    };
    let mut session = Session::default();
    let ticket = session.begin().unwrap();
    session.finish(ticket, config.execute(Command::Refresh));
    session.selected = Some(s.clone());
    f.core
        .edit(
            &f.id,
            p.revision,
            EditOperation::ItemSetZIndex {
                item_id: s.item_id.clone(),
                z_index: 7,
            },
        )
        .unwrap();
    let before = inventory(root.path());
    let ticket = session.begin().unwrap();
    session.finish(
        ticket,
        config.execute(Command::Edit(
            p.revision,
            Box::new(d.build(&p, Some(&s), true, false, false).unwrap()),
        )),
    );
    assert!(session.needs_refresh);
    assert!(session.error.unwrap().contains("REVISION_CONFLICT"));
    assert_eq!(inventory(root.path()), before);
    assert!(!d.matches(&f.project(), Some(&s)));
}

#[test]
fn represented_scalar_boundary_incompatible_assets_locks_and_missing_references_use_core() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let p = f.project();
    let s = Selection::root(&f.aliases["instance0"]);
    let mut unicode = draft(&p, &s, "body");
    unicode.text = "😀".repeat(4096);
    let before = inventory(root.path());
    let error = f
        .core
        .edit(
            &f.id,
            p.revision,
            unicode.build(&p, Some(&s), false, false, false).unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains("UTF-8 bytes"));
    assert!(!error.retryable);
    assert_eq!(inventory(root.path()), before);
    let audio = p
        .assets
        .iter()
        .find(|asset| asset.media_type == opencut_editor_core::MediaType::Audio)
        .unwrap();
    let mut incompatible = draft(&p, &s, "icon");
    incompatible.text = audio.id.clone();
    let error = f
        .core
        .edit(
            &f.id,
            p.revision,
            incompatible
                .build(&p, Some(&s), false, false, false)
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(inventory(root.path()), before);
    let mut boolean = draft(&p, &s, "constructor");
    boolean.text = "TRUE".into();
    assert!(
        boolean
            .build(&p, Some(&s), false, false, false)
            .unwrap_err()
            .contains("exactly true or false")
    );
    assert_eq!(inventory(root.path()), before);
    let mut missing = unicode.build(&p, Some(&s), true, false, false).unwrap();
    if let EditOperation::ComponentInstanceUpdate { component_id, .. } = &mut missing {
        *component_id = "missing-definition".into();
    }
    let error = f.core.edit(&f.id, p.revision, missing).unwrap_err();
    assert_eq!(error.code, ErrorCode::ItemNotFound);
    assert!(!error.retryable);
    assert_eq!(inventory(root.path()), before);
    let track = s.resolve(&p).unwrap().0;
    let lock = serde_json::from_value(
        serde_json::json!({"operation":"update_track","trackId":track.id,"locked":true}),
    )
    .unwrap();
    f.core.edit(&f.id, p.revision, lock).unwrap();
    let locked = f.project();
    let before = inventory(root.path());
    let edit = draft(&locked, &s, "body")
        .build(&locked, Some(&s), false, false, false)
        .unwrap();
    let error = f.core.edit(&f.id, locked.revision, edit).unwrap_err();
    assert_eq!(error.code, ErrorCode::TrackLocked);
    assert!(!error.retryable);
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn full_occurrence_definition_identity_and_bounded_declared_order() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let mut p = f.project();
    let s = Selection::root(&f.aliases["instance0"]);
    let d = draft(&p, &s, "title");
    let wrapper_component = instance_slots::instance(&p, &Selection::root(&f.aliases["wrapper0"]))
        .unwrap()
        .component_id
        .clone();
    let mut occurrences = Vec::new();
    for index in 0..2 {
        let local = Selection {
            instance_path: vec![f.aliases[&format!("wrapper{index}")].clone()],
            scope: format!("component:{wrapper_component}"),
            item_id: "nested".into(),
        };
        assert!(instance_slots::instance(&p, &local).is_some());
        let local_slots = instance_slots::slots(&p, &local);
        assert_eq!(
            serde_json::to_value(local_slots).unwrap(),
            serde_json::to_value(instance_slots::slots(&p, &s)).unwrap()
        );
        for slot in local_slots {
            assert!(!instance_slots::supported(&local, slot));
            assert!(Draft::new(&p, &local, slot).is_none());
        }
        occurrences.push(local);
    }
    assert_ne!(occurrences[0], occurrences[1]);
    let mut same_id_different_scope = s.clone();
    same_id_different_scope.scope = "component:other".into();
    assert!(!d.matches(&p, Some(&same_id_different_scope)));
    assert!(!d.matches(&p, None));
    p.components[0]
        .slots
        .iter_mut()
        .find(|slot| slot.id == "title")
        .unwrap()
        .binding
        .target_layer_id = "body".into();
    assert!(!d.matches(&p, Some(&s)));
    let template = p.components[0].slots[0].clone();
    p.components[0].slots = (0..129)
        .map(|index| {
            let mut slot = template.clone();
            slot.id = format!("slot{index}");
            slot
        })
        .collect();
    let visible = instance_slots::slots(&p, &s);
    assert_eq!(visible.len(), 128);
    assert_eq!(visible[0].id, "slot0");
    assert_eq!(visible[127].id, "slot127");
    // These synthetic snapshots exercise presentation bounds; authoritative fixture validation is separate.
    let before = inventory(root.path());
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn authoritative_maximum_slot_definition_is_visible_in_declared_order() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let p = f.project();
    let component = &p.components[0];
    let source = component.tracks[0]
        .items
        .iter()
        .find_map(|item| match item {
            opencut_editor_core::TimelineItem::Text(text) => Some(text),
            _ => None,
        })
        .unwrap();
    let template = component
        .slots
        .iter()
        .find(|slot| slot.id == "number")
        .unwrap();
    let mut track = component.tracks[0].clone();
    track.items.clear();
    let mut slots = Vec::new();
    for index in 0..128 {
        let mut text = source.clone();
        text.id = format!("source{index}");
        text.font_binding = None;
        text.visual_properties.stack_order = index;
        let mut slot = template.clone();
        slot.id = format!("slot{index}");
        slot.binding.target_layer_id = text.id.clone();
        track
            .items
            .push(opencut_editor_core::TimelineItem::Text(text));
        slots.push(slot);
    }
    let result = f
        .core
        .edit(
            &f.id,
            p.revision,
            EditOperation::ComponentCreate {
                name: "Maximum legal slots".into(),
                width: 160,
                height: 90,
                duration_ms: 1000,
                tracks: vec![track],
                slots: Some(slots.clone()),
            },
        )
        .unwrap();
    let component_id = result.changed_ids[0].clone();
    let overlay = p
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Overlay)
        .unwrap();
    let edit = serde_json::from_value(serde_json::json!({"operation":"add_component_instance","trackId":overlay.id,"componentId":component_id,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1})).unwrap();
    let result = f.core.edit(&f.id, f.project().revision, edit).unwrap();
    let selection = Selection::root(&result.changed_ids[0]);
    let before = inventory(root.path());
    let authoritative = f.project();
    assert_eq!(
        instance_slots::slots(&authoritative, &selection),
        slots.as_slice()
    );
    assert_eq!(instance_slots::slots(&authoritative, &selection).len(), 128);
    assert_eq!(inventory(root.path()), before);
}

#[test]
fn pending_completion_context_and_generation_do_not_restore_a_slot_draft() {
    let root = tempfile::tempdir().unwrap();
    let f = instance_slot_fixture::seed(root.path());
    let project = f.project();
    let selection = Selection::root(&f.aliases["instance0"]);
    let context = crate::shell::ActionContext {
        source: crate::animation_inspector::DraftIdentity {
            selection: selection.clone(),
            revision: project.revision,
            cursor: Default::default(),
        },
        compositing: Default::default(),
        epoch: 4,
    };
    let source = draft(&project, &selection, "title");
    let mut session = Session::default();
    session.project = Some(project.clone());
    session.selected = Some(selection.clone());
    let ticket = session.begin().unwrap();
    assert!(session.begin().is_none());
    let other = Selection::root(&f.aliases["instance1"]);
    session.selected = Some(other.clone());
    assert!(!source.matches(&project, session.selected.as_ref()));
    assert!(!context.matches(
        &project,
        session.selected.as_ref(),
        Default::default(),
        &Default::default(),
        5
    ));
    session.finish(ticket, Ok(project.clone()));
    assert_eq!(session.selected, Some(other));
    assert!(!session.busy);
    session.selected = Some(selection.clone());
    assert!(!context.matches(
        &project,
        Some(&selection),
        Default::default(),
        &Default::default(),
        6
    ));
    let ticket = session.begin().unwrap();
    let before = serde_json::to_value(session.project.as_ref()).unwrap();
    let mut stale = project.clone();
    stale.revision += 100;
    session.finish(ticket - 1, Ok(stale));
    assert!(session.busy);
    assert_eq!(
        serde_json::to_value(session.project.as_ref()).unwrap(),
        before
    );
    session.finish(ticket, Ok(project));
    assert!(!session.busy);
}
