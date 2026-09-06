#[path = "../../../crates/editor-core/tests/support/rule_card.rs"]
mod fixture;

use crate::{
    hierarchy::{Hierarchy, ROW_LIMIT, Selection, editable},
    session::{Command, Session, Startup, parse_z_index},
};
use opencut_editor_core::{EditOperation, ErrorCode, ParentReference, TimelineItem};
use std::collections::HashSet;

fn startup(f: &fixture::Fixture) -> Startup {
    Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    }
}
fn run(session: &mut Session, startup: &Startup, command: Command) {
    let ticket = session.begin().unwrap();
    session.finish(ticket, startup.execute(command));
}

#[test]
fn startup_and_load() {
    assert_eq!(Startup::parse([]).unwrap(), None);
    for args in [
        vec!["--project-id"],
        vec!["--project-store", "x"],
        vec!["--unknown"],
        vec!["--project-store", "x", "--project-store", "y"],
    ] {
        assert!(Startup::parse(args.into_iter().map(Into::into)).is_err());
    }
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let config = Startup::parse([
        "--project-store".into(),
        f.core.paths().projects_root().as_os_str().to_owned(),
        "--project-id".into(),
        f.id.clone().into(),
    ])
    .unwrap()
    .unwrap();
    let p = config.execute(Command::Refresh).unwrap();
    assert_eq!(p.revision, f.project().revision);
    assert_eq!(p.id, f.id);
}

#[test]
fn invalid_load() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let mut config = startup(&f);
    config.project_id = "missing".into();
    let mut session = Session::default();
    run(&mut session, &config, Command::Refresh);
    assert!(session.project.is_none());
    assert!(session.error.unwrap().contains("PROJECT_NOT_FOUND"));
    config.project_id = "../escape".into();
    assert!(config.execute(Command::Refresh).is_err());
    let file = f
        .core
        .paths()
        .project_dir(&f.id)
        .unwrap()
        .join("project.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    value["schemaVersion"] = serde_json::json!(999);
    let invalid = serde_json::to_vec(&value).unwrap();
    std::fs::write(&file, &invalid).unwrap();
    assert!(startup(&f).execute(Command::Refresh).is_err());
    assert_eq!(std::fs::read(file).unwrap(), invalid);
}

#[test]
fn history_and_conflict() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let config = startup(&f);
    let mut session = Session::default();
    run(&mut session, &config, Command::Refresh);
    let selection = Selection::root(&f.aliases["instance0"]);
    session.selected = Some(selection.clone());
    let revision = session.project.as_ref().unwrap().revision;
    run(
        &mut session,
        &config,
        Command::Edit(
            revision,
            Box::new(EditOperation::ItemSetZIndex {
                item_id: selection.item_id.clone(),
                z_index: i32::MIN,
            }),
        ),
    );
    assert_eq!(
        selection
            .resolve(session.project.as_ref().unwrap())
            .unwrap()
            .1
            .visual_properties()
            .z_index,
        i32::MIN
    );
    run(&mut session, &config, Command::Undo(revision + 1));
    assert_eq!(
        selection
            .resolve(session.project.as_ref().unwrap())
            .unwrap()
            .1
            .visual_properties()
            .z_index,
        0
    );
    run(&mut session, &config, Command::Redo(revision + 2));
    assert_eq!(
        selection
            .resolve(&config.execute(Command::Refresh).unwrap())
            .unwrap()
            .1
            .visual_properties()
            .z_index,
        i32::MIN
    );
    for parent in [
        None,
        Some(ParentReference {
            scope: "root".into(),
            id: f.aliases["parent"].clone(),
        }),
    ] {
        let revision = session.project.as_ref().unwrap().revision;
        run(
            &mut session,
            &config,
            Command::Edit(
                revision,
                Box::new(EditOperation::ItemSetParent {
                    item_id: selection.item_id.clone(),
                    parent: parent.clone(),
                }),
            ),
        );
        assert_eq!(
            selection
                .resolve(session.project.as_ref().unwrap())
                .unwrap()
                .1
                .visual_properties()
                .parent,
            parent
        );
    }
    let displayed = session.project.as_ref().unwrap().revision;
    f.move_parent();
    run(&mut session, &config, Command::Undo(displayed));
    assert_eq!(session.project.as_ref().unwrap().revision, displayed);
    assert!(
        session
            .error
            .as_ref()
            .unwrap()
            .contains("REVISION_CONFLICT")
    );
    assert!(session.error.as_ref().unwrap().contains("retryable: true"));
    run(&mut session, &config, Command::Refresh);
    assert_eq!(session.project.as_ref().unwrap().revision, displayed + 1);
    assert_eq!(session.selected, Some(selection));
}

#[test]
fn failed_edits() {
    for value in ["", "1.5", "NaN", "2147483648", "-2147483649", "--1"] {
        assert!(parse_z_index(value).is_err());
    }
    assert_eq!(parse_z_index("-2147483648").unwrap(), i32::MIN);
    assert_eq!(parse_z_index("2147483647").unwrap(), i32::MAX);
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let config = startup(&f);
    let original = f.project();
    for (id, parent, expected) in [
        ("missing".to_owned(), None, ErrorCode::ItemNotFound),
        (
            f.aliases["parent"].clone(),
            Some(ParentReference {
                scope: "root".into(),
                id: f.aliases["parent"].clone(),
            }),
            ErrorCode::InvalidArgument,
        ),
    ] {
        let error = config
            .execute(Command::Edit(
                original.revision,
                Box::new(EditOperation::ItemSetParent {
                    item_id: id,
                    parent,
                }),
            ))
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(f.project().revision, original.revision);
    }
    let track = original
        .tracks
        .iter()
        .find(|t| t.items.iter().any(|i| i.id() == f.aliases["instance0"]))
        .unwrap();
    f.core
        .edit(
            &f.id,
            original.revision,
            fixture::operation(
                serde_json::json!({"operation":"update_track","trackId":track.id,"locked":true}),
            ),
        )
        .unwrap();
    let mut session = Session::default();
    run(&mut session, &config, Command::Refresh);
    let before = serde_json::to_value(session.project.as_ref().unwrap()).unwrap();
    run(
        &mut session,
        &config,
        Command::Edit(
            original.revision + 1,
            Box::new(EditOperation::ItemSetZIndex {
                item_id: f.aliases["instance0"].clone(),
                z_index: 5,
            }),
        ),
    );
    assert!(session.error.as_ref().unwrap().contains("TRACK_LOCKED"));
    assert_eq!(
        serde_json::to_value(session.project.as_ref().unwrap()).unwrap(),
        before
    );
}

#[test]
fn selection_reconciliation() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let config = startup(&f);
    let mut session = Session::default();
    run(&mut session, &config, Command::Refresh);
    let selection = Selection::root(&f.aliases["instance0"]);
    session.selected = Some(selection.clone());
    session.expanded.insert(selection.clone());
    f.core
        .edit(
            &f.id,
            f.project().revision,
            fixture::operation(
                serde_json::json!({"operation":"delete_item","itemId":selection.item_id}),
            ),
        )
        .unwrap();
    run(&mut session, &config, Command::Refresh);
    assert!(session.selected.is_none());
    assert!(session.expanded.is_empty());
    let ticket = session.begin().unwrap();
    assert!(session.begin().is_none(), "writes must serialize");
    session.finish(ticket - 1, config.execute(Command::Refresh));
    assert!(session.busy, "stale results must be ignored");
    session.finish(ticket, config.execute(Command::Refresh));
    assert!(!session.busy);
}

#[test]
fn repeated_instances() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let p = f.project();
    let mut expanded = HashSet::from([Selection::root(&f.aliases["parent"])]);
    for i in 0..3 {
        expanded.insert(Selection::root(&f.aliases[&format!("instance{i}")]));
    }
    let tree = Hierarchy::build(&p, &expanded);
    let titles: Vec<_> = tree
        .rows
        .iter()
        .filter(|r| r.selection.item_id == "title")
        .collect();
    assert_eq!(titles.len(), 3);
    assert_ne!(titles[0].selection, titles[1].selection);
    for row in titles {
        assert_eq!(row.selection.resolve(&p).unwrap().1.id(), "title");
        assert!(!editable(&p, &row.selection));
        assert_eq!(row.selection.instance_path.len(), 1);
    }
    assert!(editable(&p, &Selection::root(&f.aliases["instance0"])));
    let audio = p
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| matches!(i, TimelineItem::Media(_)))
        .unwrap();
    assert!(!editable(&p, &Selection::root(audio.id())));
}

#[test]
fn cross_track_groups() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let mut p = f.project();
    let overlay = p
        .tracks
        .iter_mut()
        .find(|t| t.items.iter().any(|i| i.id() == f.aliases["instance0"]))
        .unwrap();
    let index = overlay
        .items
        .iter()
        .position(|i| i.id() == f.aliases["instance0"])
        .unwrap();
    let mut other = overlay.clone();
    other.id = "other".into();
    other.name = "Other track".into();
    other.items = vec![overlay.items.remove(index)];
    other.items[0].set_hidden(true);
    p.tracks.push(other);
    let tree = Hierarchy::build(&p, &HashSet::from([Selection::root(&f.aliases["parent"])]));
    let row = tree
        .rows
        .iter()
        .find(|r| r.selection.item_id == f.aliases["instance0"])
        .unwrap();
    assert_eq!(row.depth, 1);
    assert!(row.label.contains("Other track · hidden"));
}

#[test]
fn bounded_rows() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(root.path());
    let mut p = f.project();
    let track = p
        .tracks
        .iter_mut()
        .find(|t| t.items.iter().any(|i| i.id() == f.aliases["instance0"]))
        .unwrap();
    let source = track
        .items
        .iter()
        .find(|i| i.id() == f.aliases["instance0"])
        .unwrap()
        .clone();
    track.items.clear();
    let mut expanded = HashSet::new();
    for i in 0..700 {
        let TimelineItem::ComponentInstance(mut item) = source.clone() else {
            panic!()
        };
        item.id = format!("instance-{i}");
        item.visual_properties.parent = None;
        expanded.insert(Selection::root(&item.id));
        track.items.push(TimelineItem::ComponentInstance(item));
    }
    let tree = Hierarchy::build(&p, &expanded);
    assert_eq!(tree.rows.len(), ROW_LIMIT);
    assert!(tree.truncated);
    assert!(!Hierarchy::build(&p, &HashSet::new()).truncated);
}
