//! Canonical controls, core-backed transactions and independent predecessor evidence.
use crate::{
    compositing_inspector::{self as controls, Action, Cursor, Eligibility},
    hierarchy::Selection,
    inspector_edit,
    session::{Command, Startup},
};
use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, PathPolicy, Project, ProjectSettings, TimelineItem,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn item() -> TimelineItem {
    serde_json::from_value(json!({"type":"rectangle","id":"leaf","startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default(),"keyframes":[]})).unwrap()
}
fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../contracts/desktop-compositing-controls-v1.json"
    ))
    .unwrap()
}
fn numeric(value: &Value) -> Value {
    match value {
        Value::Number(n) => json!(n.as_f64().unwrap()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), numeric(v))).collect()),
        Value::Array(a) => Value::Array(a.iter().map(numeric).collect()),
        _ => value.clone(),
    }
}
fn project() -> Project {
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("controls", ProjectSettings::default())
        .unwrap()
        .project_id;
    core.get_project(&id).unwrap()
}
fn fs(project: &Project, item: &TimelineItem, cursor: &Cursor) -> Vec<inspector_edit::Field> {
    controls::fields(project, &Selection::root(item.id()), item, cursor)
}
fn field(fields: &[inspector_edit::Field], path: &str) -> inspector_edit::Field {
    fields
        .iter()
        .find(|f| f.path == path)
        .unwrap_or_else(|| panic!("Missing {path}"))
        .clone()
}
fn inventory(path: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, p: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(p).unwrap() {
            let e = entry.unwrap();
            if e.file_type().unwrap().is_dir() {
                visit(root, &e.path(), out)
            } else {
                out.insert(
                    e.path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                    std::fs::read(e.path()).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(path, path, &mut out);
    out
}
#[test]
fn desktop_compositing_catalog_defaults_complete_fields_and_surface_identity() {
    let c = catalog();
    assert_eq!(c["projectSchemaVersion"], 37);
    assert_eq!(c["headlessProtocolVersion"], 1);
    assert_eq!(c["registeredToolCount"], 78);
    assert_eq!(c["coreMutation"], "update_item");
    assert_eq!(c["blendModes"], json!(controls::BLENDS));
    let mut selected = item();
    let p = project();
    selected
        .visual_properties_mut()
        .masks
        .push(controls::default_mask("mask-1".into()));
    assert_eq!(
        numeric(&serde_json::to_value(&selected.visual_properties().masks[0]).unwrap()),
        numeric(&c["defaultMask"])
    );
    let mask_fields = fs(&p, &selected, &Cursor::default());
    for (path, kind) in c["maskFields"].as_object().unwrap() {
        let field = field(
            &mask_fields,
            &format!("/masks/0/{}", path.replace('.', "/")),
        );
        if kind == "finite_f64" {
            assert_eq!(field.kind, inspector_edit::FieldKind::Number);
        } else if kind == "boolean" {
            assert_eq!(field.kind, inspector_edit::FieldKind::Boolean);
        } else {
            let inspector_edit::FieldKind::Choice(choices) = field.kind else {
                panic!("Expected exact mask choices");
            };
            assert_eq!(json!(choices), *kind);
        }
    }
    for kind in controls::EFFECT_TYPES {
        let effect = controls::default_effect(kind, "effect-1".into()).unwrap();
        assert_eq!(
            numeric(&serde_json::to_value(&effect).unwrap()),
            numeric(&c["effectDefaults"][kind])
        );
        selected.visual_properties_mut().effects = vec![effect];
        let fields = fs(&p, &selected, &Cursor::default());
        let actual: Vec<_> = fields
            .iter()
            .filter(|f| f.update_key == "effects")
            .collect();
        assert_eq!(
            actual.len(),
            c["effectFields"][kind].as_object().unwrap().len()
        );
        for (path, kind) in c["effectFields"][kind].as_object().unwrap() {
            let f = field(&fields, &format!("/effects/0/{}", path.replace('.', "/")));
            assert_eq!(
                f.kind,
                match kind.as_str().unwrap() {
                    "u16" => inspector_edit::FieldKind::Unsigned16,
                    "u32" => inspector_edit::FieldKind::Integer,
                    _ => inspector_edit::FieldKind::Number,
                }
            );
        }
    }
}
#[test]
fn desktop_compositing_typed_every_mask_and_effect_field_preserves_complete_records() {
    let p = project();
    let mut selected = item();
    selected.visual_properties_mut().masks = vec![
        controls::default_mask("mask-1".into()),
        controls::default_mask("mask-2".into()),
    ];
    let original = serde_json::to_value(&selected).unwrap();
    for f in fs(&p, &selected, &Cursor::default())
        .iter()
        .filter(|f| f.update_key == "masks")
    {
        let input = match f.kind {
            inspector_edit::FieldKind::Boolean => "true",
            inspector_edit::FieldKind::Choice(choices) => choices[choices.len() - 1],
            _ => "0.12345678901234566",
        };
        let value =
            serde_json::to_value(inspector_edit::build(&selected, f, input).unwrap()).unwrap();
        let mut expected = original["masks"].clone();
        *expected
            .pointer_mut(f.path.strip_prefix("/masks").unwrap())
            .unwrap() = match f.kind {
            inspector_edit::FieldKind::Boolean => json!(true),
            inspector_edit::FieldKind::Choice(_) => json!(input),
            _ => json!(input.parse::<f64>().unwrap()),
        };
        assert_eq!(value["masks"], expected, "{}", f.path);
        assert!(value.get("animationChannels").is_none());
    }
    for kind in controls::EFFECT_TYPES {
        selected.visual_properties_mut().effects = vec![
            controls::default_effect(kind, "effect-1".into()).unwrap(),
            controls::default_effect("vignette", "neighbor".into()).unwrap(),
        ];
        let original = serde_json::to_value(&selected).unwrap();
        for f in fs(&p, &selected, &Cursor::default())
            .iter()
            .filter(|f| f.update_key == "effects")
        {
            let input = match f.kind {
                inspector_edit::FieldKind::Integer => "4294967295",
                inspector_edit::FieldKind::Unsigned16 => "65535",
                _ => "0.12345678901234566",
            };
            let request =
                serde_json::to_value(inspector_edit::build(&selected, f, input).unwrap()).unwrap();
            let mut expected = original["effects"].clone();
            *expected
                .pointer_mut(f.path.strip_prefix("/effects").unwrap())
                .unwrap() = match f.kind {
                inspector_edit::FieldKind::Integer | inspector_edit::FieldKind::Unsigned16 => {
                    json!(input.parse::<u32>().unwrap())
                }
                _ => json!(input.parse::<f64>().unwrap()),
            };
            assert_eq!(request["effects"], expected);
            assert!(request.get("animationChannels").is_none());
        }
    }
}
#[test]
fn desktop_compositing_rejects_malformed_numeric_choice_and_unicode_without_panics() {
    let p = project();
    let mut selected = item();
    selected.visual_properties_mut().effects =
        vec![controls::default_effect("particle_overlay", "fx".into()).unwrap()];
    selected.visual_properties_mut().masks = vec![controls::default_mask("mask".into())];
    let fields = fs(&p, &selected, &Cursor::default());
    for path in [
        "/effects/0/count",
        "/effects/0/seed",
        "/effects/0/radiusPx",
        "/masks/0/featherPx",
        "/masks/0/channel",
    ] {
        for input in [
            "NaN", "inf", "-inf", "∞", "４", "ééé", "1e999", "1.5junk", "",
        ] {
            assert!(
                inspector_edit::build(&selected, &field(&fields, path), input).is_err(),
                "{path}:{input}"
            );
        }
    }
    for input in ["65536", "-1", "1.5"] {
        assert!(
            inspector_edit::build(&selected, &field(&fields, "/effects/0/count"), input).is_err()
        );
    }
    for input in ["4294967296", "-1", "1.5"] {
        assert!(
            inspector_edit::build(&selected, &field(&fields, "/effects/0/seed"), input).is_err()
        );
    }
}
#[test]
fn desktop_compositing_cursor_defaults_order_deletion_and_exact_first_unused_ids() {
    let p = project();
    let mut selected = item();
    selected.visual_properties_mut().masks = vec![
        controls::default_mask("mask-01".into()),
        controls::default_mask("mask-2".into()),
    ];
    let selection = Selection::root(selected.id());
    let mut cursor = Cursor::default();
    cursor.resolve(&selected);
    let (edit, next) =
        controls::action(&p, &selection, &selected, &cursor, Action::AddMask).unwrap();
    let v = serde_json::to_value(edit).unwrap();
    assert_eq!(v["masks"][2]["id"], "mask-1");
    assert_eq!(next.mask_id.as_deref(), Some("mask-1"));
    selected.visual_properties_mut().masks = serde_json::from_value(v["masks"].clone()).unwrap();
    let (edit, next) =
        controls::action(&p, &selection, &selected, &next, Action::MoveMask(false)).unwrap();
    let v = serde_json::to_value(edit).unwrap();
    assert_eq!(v["masks"][1]["id"], "mask-1");
    assert_eq!(next.mask_id.as_deref(), Some("mask-1"));
    selected.visual_properties_mut().masks = serde_json::from_value(v["masks"].clone()).unwrap();
    let (edit, next) =
        controls::action(&p, &selection, &selected, &next, Action::DeleteMask).unwrap();
    assert_eq!(next.mask_id.as_deref(), Some("mask-2"));
    assert_eq!(
        serde_json::to_value(edit).unwrap()["masks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    cursor.mask_id = Some("mask-01".into());
    cursor.navigate(&selected, 0, true);
    assert_eq!(cursor.mask_id.as_deref(), Some("mask-1"));
    cursor.navigate(&selected, 2, true);
    assert_eq!(cursor.command, 1);
}
#[test]
fn desktop_compositing_gradient_and_path_variants_remain_readonly_and_bounded() {
    let c = catalog();
    let p = project();
    let mut selected = item();
    let mut mask = c["defaultMask"].clone();
    mask["source"]["path"]["commands"] = json!([{"type":"moveTo","to":{"x":0,"y":0}},{"type":"quadraticTo","control":{"x":1,"y":2},"to":{"x":3,"y":4}},{"type":"cubicTo","control1":{"x":5,"y":6},"control2":{"x":7,"y":8},"to":{"x":9,"y":10}},{"type":"close"}]);
    mask["source"]["paint"] = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0.3}},{"offset":1,"color":{"r":0,"g":1,"b":0,"a":0.8}}]});
    selected.visual_properties_mut().masks = vec![serde_json::from_value(mask).unwrap()];
    for command in 0..4 {
        let cursor = Cursor {
            command,
            ..Cursor::default()
        };
        let fields = fs(&p, &selected, &cursor);
        let command_fields: Vec<_> = fields
            .iter()
            .filter(|f| f.path.contains("/commands/"))
            .collect();
        assert_eq!(command_fields.len(), [2, 4, 6, 0][command]);
        let variant = ["moveTo", "quadraticTo", "cubicTo", "close"][command];
        let mut actual: Vec<_> = command_fields
            .iter()
            .map(|f| {
                f.path
                    .rsplit_once(&format!("/commands/{command}/"))
                    .unwrap()
                    .1
                    .replace('/', ".")
            })
            .collect();
        actual.sort();
        let mut expected: Vec<_> = c["pathCommandFields"][variant]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        expected.sort();
        assert_eq!(actual, expected);

        assert!(!fields.iter().any(|f| f.path.contains("/paint/")));
        let descriptions = controls::descriptions(&selected, &cursor);
        assert!(
            descriptions
                .iter()
                .any(|d| d.contains("geometry read-only"))
        );
        assert!(
            descriptions
                .iter()
                .any(|d| d.contains("offset/RGBA read-only"))
        );
        let edit =
            inspector_edit::build(&selected, &field(&fields, "/masks/0/featherPx"), "2").unwrap();
        assert_eq!(
            serde_json::to_value(edit).unwrap()["masks"][0]["source"],
            serde_json::to_value(&selected).unwrap()["masks"][0]["source"]
        );
    }
}
#[test]
fn desktop_compositing_core_lifecycle_failures_full_inventory_and_history() {
    let root = tempfile::tempdir().unwrap();
    let f = crate::tests::fixture::seed(root.path());
    let config = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    };
    let p = f.project();
    let track = p
        .tracks
        .iter()
        .find(|t| t.track_type == opencut_editor_core::TrackType::Video)
        .unwrap()
        .id
        .clone();
    let created=f.core.edit(&f.id,p.revision,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap();
    let id = created.changed_ids[0].clone();
    let selection = Selection::root(&id);
    let mut cursor = Cursor::default();
    let initial = f.project();
    let initial_item = serde_json::to_value(initial.find_item(&id).unwrap()).unwrap();
    for action in [
        Action::AddMask,
        Action::AddMask,
        Action::AddEffect("glow"),
        Action::AddEffect("color_tint"),
        Action::MoveEffect(false),
    ] {
        let p = f.project();
        let (edit, next) =
            controls::action(&p, &selection, p.find_item(&id).unwrap(), &cursor, action).unwrap();
        config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap();
        cursor = next;
    }
    let p = f.project();
    let leaf = p.find_item(&id).unwrap();
    let final_item = serde_json::to_value(leaf).unwrap();
    assert_eq!(final_item["effects"][0]["type"], "color_tint");
    assert_eq!(final_item["effects"][1]["type"], "glow");
    let fs = fs(&p, leaf, &cursor);
    let before = inventory(f.core.paths().project_dir(&f.id).unwrap().as_path());
    assert!(inspector_edit::build(leaf, &field(&fs, "/masks/1/featherPx"), "NaN").is_err());
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
    let edit = inspector_edit::build(leaf, &field(&fs, "/masks/1/featherPx"), "-1").unwrap();
    let error = config
        .execute(Command::Edit(p.revision, Box::new(edit)))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
    let edit = inspector_edit::build(leaf, &field(&fs, "/blendMode"), "screen").unwrap();
    assert_eq!(
        config
            .execute(Command::Edit(p.revision - 1, Box::new(edit)))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
    let component_local_provider = p
        .components
        .iter()
        .flat_map(|c| &c.tracks)
        .flat_map(|t| &t.items)
        .next()
        .unwrap()
        .id();
    assert!(p.find_item(component_local_provider).is_none());
    for provider in ["missing", id.as_str(), component_local_provider] {
        let edit = inspector_edit::build(leaf, &field(&fs, "/matte/sourceId"), provider).unwrap();
        let error = config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap_err();
        assert_eq!(
            error.code,
            if provider == id {
                ErrorCode::InvalidArgument
            } else {
                ErrorCode::ItemNotFound
            }
        );
        assert!(!error.retryable);
        assert_eq!(
            inventory(&f.core.paths().project_dir(&f.id).unwrap()),
            before
        );
    }
    for _ in 0..5 {
        let p = f.project();
        config.execute(Command::Undo(p.revision)).unwrap();
    }
    assert_eq!(
        serde_json::to_value(f.project().find_item(&id).unwrap()).unwrap(),
        initial_item
    );
    for _ in 0..5 {
        let p = f.project();
        config.execute(Command::Redo(p.revision)).unwrap();
    }
    assert_eq!(
        serde_json::to_value(
            config
                .execute(Command::Refresh)
                .unwrap()
                .find_item(&id)
                .unwrap()
        )
        .unwrap(),
        final_item
    );
    let p = f.project();
    f.core.edit(&f.id,p.revision,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[{"property":"mask.transform.opacity","target":{"kind":"mask","scope":"root","id":"mask-2"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1.0},"curve":"linear"}]}]}))).unwrap();
    let p = f.project();
    let before = inventory(&f.core.paths().project_dir(&f.id).unwrap());
    let (edit, _) = controls::action(
        &p,
        &selection,
        p.find_item(&id).unwrap(),
        &cursor,
        Action::DeleteMask,
    )
    .unwrap();
    assert_eq!(
        config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
}
#[test]
fn desktop_compositing_eligibility_matte_clip_omission_and_exact_clear_requests() {
    let p = project();
    let mut leaf = item();
    let selection = Selection::root(leaf.id());
    assert_eq!(
        controls::eligibility(&p, &selection, &leaf),
        Eligibility::Leaf
    );
    let local = Selection {
        scope: "component:source".into(),
        instance_path: vec!["instance".into()],
        item_id: leaf.id().into(),
    };
    assert_eq!(controls::eligibility(&p, &local, &leaf), Eligibility::None);
    assert!(controls::fields(&p, &local, &leaf, &Cursor::default()).is_empty());
    assert!(controls::action(&p, &local, &leaf, &Cursor::default(), Action::AddMask).is_err());
    let field = field(&fs(&p, &leaf, &Cursor::default()), "/matte/sourceId");
    let request =
        serde_json::to_value(inspector_edit::build(&leaf, &field, "provider").unwrap()).unwrap();
    assert_eq!(
        request["matte"],
        json!({"sourceId":"provider","channel":"alpha"})
    );
    assert!(request.get("effects").is_none());
    leaf.visual_properties_mut().matte =
        serde_json::from_value(json!({"sourceId":"provider","channel":"luma"})).unwrap();
    let request =
        serde_json::to_value(inspector_edit::build(&leaf, &field, "another").unwrap()).unwrap();
    assert_eq!(request["matte"]["channel"], "luma");
    let (edit, _) = controls::action(
        &p,
        &selection,
        &leaf,
        &Cursor::default(),
        Action::ClearMatte,
    )
    .unwrap();
    assert_eq!(serde_json::to_value(edit).unwrap()["matte"], Value::Null);
    for kind in ["group", "component_instance"] {
        let value = if kind == "group" {
            json!({"type":"group","id":"owner","startMs":0,"durationMs":1000})
        } else {
            json!({"type":"component_instance","id":"owner","componentId":"definition","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1})
        };
        let owner: TimelineItem = serde_json::from_value(value).unwrap();
        let s = Selection::root("owner");
        assert_eq!(
            controls::eligibility(&p, &s, &owner),
            Eligibility::Aggregate
        );
        let fields = controls::fields(&p, &s, &owner, &Cursor::default());
        assert!(fields.is_empty());
        for set in [true, false] {
            let (edit, _) =
                controls::action(&p, &s, &owner, &Cursor::default(), Action::SetClip(set)).unwrap();
            let request = serde_json::to_value(edit).unwrap();
            assert_eq!(
                request["clip"],
                if set {
                    catalog()["clipValue"].clone()
                } else {
                    Value::Null
                }
            );
            assert!(request.get("effects").is_none());
        }
        assert!(controls::action(&p, &s, &owner, &Cursor::default(), Action::AddMask).is_err());
    }
}
#[test]
fn desktop_compositing_legacy_field_vectors_match_frozen_predecessor() {
    let root = tempfile::tempdir().unwrap();
    let f = crate::tests::fixture::seed(root.path());
    let p = f.project();
    for item in p.tracks.iter().flat_map(|t| &t.items).chain(
        p.components
            .iter()
            .flat_map(|c| &c.tracks)
            .flat_map(|t| &t.items),
    ) {
        assert_eq!(
            inspector_edit::fields(item),
            crate::compositing_predecessor::fields(item),
            "{}",
            item.id()
        );
        for cursor in [
            crate::animation_inspector::Cursor::default(),
            crate::animation_inspector::Cursor {
                channel: usize::MAX,
                key: usize::MAX,
                legacy: usize::MAX,
            },
        ] {
            for audio in [false, true] {
                assert_eq!(
                    crate::animation_inspector::fields(item, cursor, audio),
                    crate::compositing_predecessor::animation_fields(item, cursor, audio)
                );
            }
        }
    }
}
// Count actual allocations on this test thread, including transient serde materialization.
// Other parallel tests and allocator operations outside the explicit probe are unaffected.
struct AllocationProbe;
thread_local! { static ALLOCATION_BYTES: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) }; }
#[global_allocator]
static ALLOCATOR: AllocationProbe = AllocationProbe;
unsafe impl std::alloc::GlobalAlloc for AllocationProbe {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let _ = ALLOCATION_BYTES.try_with(|n| {
            if let Some(bytes) = n.get() {
                n.set(Some(bytes.saturating_add(layout.size())))
            }
        });
        // Safety: forward the caller's valid layout to the system allocator.
        unsafe { std::alloc::System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        // Safety: allocation and layout belong to the system allocator above.
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, size: usize) -> *mut u8 {
        let _ = ALLOCATION_BYTES.try_with(|n| {
            if let Some(bytes) = n.get() {
                n.set(Some(bytes.saturating_add(size)))
            }
        });
        // Safety: forward the original system allocation, valid layout and requested size.
        unsafe { std::alloc::System.realloc(ptr, layout, size) }
    }
}
fn allocations<T>(work: impl FnOnce() -> T) -> (T, usize) {
    ALLOCATION_BYTES.with(|n| n.set(Some(0)));
    let result = work();
    let bytes = ALLOCATION_BYTES.with(|n| n.replace(None).unwrap());
    (result, bytes)
}
#[test]
fn desktop_compositing_maximum_collections_do_not_materialize_unselected_payloads() {
    let p = project();
    let mut small = item();
    let mut mask = controls::default_mask("mask-1".into());
    let opencut_editor_core::MaskSource::Path { path, paint } = &mut mask.source;
    let closing_command = path.commands.pop().unwrap();
    path.commands.resize(4095, path.commands[1].clone());
    path.commands.push(closing_command);
    *paint=serde_json::from_value(json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":(0..64).map(|i|json!({"offset":f64::from(i)/63.0,"color":{"r":0.1,"g":0.2,"b":0.3,"a":0.4}})).collect::<Vec<_>>()})).unwrap();
    mask.validate().unwrap();
    small.visual_properties_mut().masks = vec![mask.clone()];
    let mut large = small.clone();
    large.visual_properties_mut().masks = (1..=16)
        .map(|i| {
            let mut m = mask.clone();
            m.id = format!("mask-{i}");
            m
        })
        .collect();
    large.visual_properties_mut().effects = (1..=16)
        .map(|i| controls::default_effect("particle_overlay", format!("effect-{i}")).unwrap())
        .collect();
    let cursor = Cursor {
        mask_id: Some("mask-16".into()),
        effect_id: Some("effect-16".into()),
        command: 4095,
        stop: 63,
    };
    let ((fields, descriptions), bytes) = allocations(|| {
        (
            fs(&p, &large, &cursor),
            controls::descriptions(&large, &cursor),
        )
    });
    assert!(fields.len() < 45);
    assert!(descriptions.len() < 10);
    assert!(descriptions.iter().any(|s| s.contains("4096 / 4096")));
    assert!(descriptions.iter().any(|s| s.contains("64 / 64")));
    assert!(bytes < 100_000, "selected-only detail allocated {bytes}");
    let (legacy, bytes) = allocations(|| inspector_edit::fields(&large));
    assert_eq!(legacy, inspector_edit::fields(&small));
    assert!(
        bytes < 10_000,
        "legacy serialized unrelated collections: {bytes}"
    );
    let (_, predecessor_bytes) = allocations(|| crate::compositing_predecessor::fields(&large));
    assert!(
        predecessor_bytes > 1_000_000,
        "probe must detect the frozen whole-item predecessor, observed {predecessor_bytes}"
    );
    let channel:opencut_editor_core::AnimationChannel=serde_json::from_value(json!({"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1.0},"curve":"linear"}]})).unwrap();
    large.visual_properties_mut().animation_channels = vec![channel.clone(); 64];
    let (_, bytes) = allocations(|| {
        crate::animation_inspector::fields(
            &large,
            crate::animation_inspector::Cursor::default(),
            false,
        )
    });
    assert!(
        bytes < 20_000,
        "animation serialized unrelated collections: {bytes}"
    );
    let (_, old) = allocations(|| {
        crate::compositing_predecessor::animation_fields(
            &large,
            crate::animation_inspector::Cursor::default(),
            false,
        )
    });
    assert!(old > 1_000_000);
}
#[test]
fn desktop_compositing_draft_identity_guards_each_scope_cursor_revision_and_field() {
    let root = tempfile::tempdir().unwrap();
    let f = crate::tests::fixture::seed(root.path());
    let p = f.project();
    let item = p
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .find(|i| !inspector_edit::fields(i).is_empty())
        .unwrap();
    let selection = Selection::root(item.id());
    let field = inspector_edit::fields(item)[0].clone();
    let anim = crate::animation_inspector::Cursor::default();
    let compositing = Cursor {
        mask_id: Some("mask-1".into()),
        effect_id: Some("effect-1".into()),
        command: 1,
        stop: 1,
    };
    let draft = crate::shell::InspectorDraft {
        source: crate::animation_inspector::DraftIdentity {
            selection: selection.clone(),
            revision: p.revision,
            cursor: anim,
        },
        compositing: compositing.clone(),
        path: field.path.clone(),
        update_key: field.update_key,
    };
    assert!(draft.matches(&p, Some(&selection), anim, &compositing, &field));
    let mut changed = p.clone();
    changed.revision += 1;
    assert!(!draft.matches(&changed, Some(&selection), anim, &compositing, &field));
    let local = Selection {
        instance_path: vec!["instance".into()],
        scope: "component:source".into(),
        item_id: item.id().into(),
    };
    assert!(!draft.matches(&p, Some(&local), anim, &compositing, &field));
    assert!(!draft.matches(&p, None, anim, &compositing, &field));
    for axis in 0..4 {
        let mut cursor = compositing.clone();
        match axis {
            0 => cursor.mask_id = Some("mask-2".into()),
            1 => cursor.effect_id = Some("effect-2".into()),
            2 => cursor.command += 1,
            _ => cursor.stop += 1,
        }
        assert!(!draft.matches(&p, Some(&selection), anim, &cursor, &field));
    }
    let mut different = field.clone();
    different.path.push_str("/x");
    assert!(!draft.matches(&p, Some(&selection), anim, &compositing, &different));
    different = field.clone();
    different.update_key = "effects";
    assert!(!draft.matches(&p, Some(&selection), anim, &compositing, &different));
    let mut other = anim;
    other.key += 1;
    assert!(!draft.matches(&p, Some(&selection), other, &compositing, &field));
    let mut session = crate::session::Session::default();
    session.project = Some(p.clone());
    session.selected = Some(selection);
    let ticket = session.begin().unwrap();
    assert!(session.busy);
    assert!(session.begin().is_none());
    let current = Selection::root(
        p.tracks
            .iter()
            .flat_map(|t| &t.items)
            .find(|i| i.id() != item.id())
            .unwrap()
            .id(),
    );
    session.selected = Some(current.clone());
    session.finish(ticket, Ok(p));
    assert_eq!(session.selected, Some(current));
    assert!(!session.busy);
    let ticket = session.begin().unwrap();
    session.finish(
        ticket,
        Err(opencut_editor_core::CoreError::new(
            ErrorCode::RevisionConflict,
            "external edit",
        )),
    );
    assert!(session.needs_refresh);
    assert!(
        session
            .error
            .as_ref()
            .unwrap()
            .contains("REVISION_CONFLICT")
    );
    let ticket = session.begin().unwrap();
    session.finish(ticket, Ok(f.project()));
    assert!(!session.needs_refresh);
}
#[test]
fn desktop_compositing_every_parameter_family_commits_through_existing_core() {
    let root = tempfile::tempdir().unwrap();
    let f = minimal_fixture(root.path());
    let config = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    };
    let p = f.project();
    let track = p.tracks[1].id.clone();
    let id=f.core.edit(&f.id,p.revision,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap().changed_ids[0].clone();
    let s = Selection::root(&id);
    let p = f.project();
    let (edit, mut cursor) = controls::action(
        &p,
        &s,
        p.find_item(&id).unwrap(),
        &Cursor::default(),
        Action::AddMask,
    )
    .unwrap();
    config
        .execute(Command::Edit(p.revision, Box::new(edit)))
        .unwrap();
    let fields = fs(&f.project(), f.project().find_item(&id).unwrap(), &cursor);
    for field in fields.iter().filter(|f| f.update_key == "masks") {
        let p = f.project();
        let item = p.find_item(&id).unwrap();
        let before = serde_json::to_value(item).unwrap();
        let input = match field.kind {
            inspector_edit::FieldKind::Boolean => "true",
            inspector_edit::FieldKind::Choice(choices) => choices[choices.len() - 1],
            _ => "0.2",
        };
        let edit = inspector_edit::build(item, field, input).unwrap();
        let request = serde_json::to_value(&edit).unwrap();
        let after = config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap();
        let mut expected = before;
        expected["masks"] = request["masks"].clone();
        assert_eq!(
            serde_json::to_value(after.find_item(&id).unwrap()).unwrap(),
            expected
        );
    }
    for kind in controls::EFFECT_TYPES {
        let p = f.project();
        let (edit, next) = controls::action(
            &p,
            &s,
            p.find_item(&id).unwrap(),
            &cursor,
            Action::AddEffect(kind),
        )
        .unwrap();
        config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap();
        cursor = next;
        let p = f.project();
        let fields = fs(&p, p.find_item(&id).unwrap(), &cursor);
        for field in fields.iter().filter(|f| f.update_key == "effects") {
            let p = f.project();
            let item = p.find_item(&id).unwrap();
            let before = serde_json::to_value(item).unwrap();
            let input = match field.kind {
                inspector_edit::FieldKind::Unsigned16 => "17",
                inspector_edit::FieldKind::Integer if field.path.ends_with("/seed") => "4294967295",
                inspector_edit::FieldKind::Integer => "200",
                _ => "0.2",
            };
            let edit = inspector_edit::build(item, field, input).unwrap();
            let request = serde_json::to_value(&edit).unwrap();
            let after = config
                .execute(Command::Edit(p.revision, Box::new(edit)))
                .unwrap();
            let mut expected = before;
            expected["effects"] = request["effects"].clone();
            assert_eq!(
                serde_json::to_value(after.find_item(&id).unwrap()).unwrap(),
                expected
            );
        }
    }
}

struct MinimalFixture {
    core: EditorCore,
    id: String,
}
impl MinimalFixture {
    fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
}
fn minimal_fixture(root: &Path) -> MinimalFixture {
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [root], root.join("exports")).unwrap(),
    );
    let id = core
        .create_project(
            "Isolated parameter conformance",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    MinimalFixture { core, id }
}
#[test]
fn desktop_compositing_completion_epoch_rejects_aba_selection_reset_and_navigation() {
    let root = tempfile::tempdir().unwrap();
    let f = minimal_fixture(root.path());
    let p = f.project();
    let id=f.core.edit(&f.id,p.revision,op(json!({"operation":"add_rectangle","trackId":p.tracks[1].id,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap().changed_ids[0].clone();
    let p = f.project();
    let selected = Selection::root(&id);
    let cursor = Cursor::default();
    let anim = crate::animation_inspector::Cursor::default();
    let context = crate::shell::ActionContext {
        source: crate::animation_inspector::DraftIdentity {
            selection: selected.clone(),
            revision: p.revision,
            cursor: anim,
        },
        compositing: cursor.clone(),
        epoch: 4,
    };
    assert!(context.matches(&p, Some(&selected), anim, &cursor, 4));
    // Returning to A after A→B→A, reset or navigation restores values but cannot
    // reuse the previous interaction's completion ticket.
    for epoch in [5, 6, 7] {
        assert!(!context.matches(&p, Some(&selected), anim, &cursor, epoch));
    }
    let mut session = crate::session::Session::default();
    session.project = Some(p.clone());
    session.selected = Some(selected.clone());
    let ticket = session.begin().unwrap();
    session.selected = Some(Selection::root("other"));
    session.selected = Some(selected);
    let same = context.matches(
        session.project.as_ref().unwrap(),
        session.selected.as_ref(),
        anim,
        &cursor,
        6,
    );
    session.finish(ticket, Ok(p));
    assert!(!same);
    assert_eq!(session.selected, Some(Selection::root(&id)));
    assert!(!session.busy);
}
#[test]
fn desktop_compositing_catalog_bounds_choices_readonly_eligibility_and_semantics_are_exact() {
    let c = catalog();
    assert_eq!(c["matteChannels"], json!(["alpha", "luma"]));
    assert_eq!(c["newMatteChannel"], "alpha");
    assert_eq!(c["clipValue"], json!({"type":"composition_bounds"}));
    assert_eq!(
        c["idPolicy"],
        json!({"maskPrefix":"mask-","effectPrefix":"effect-","firstOrdinal":1,"allocation":"first_unused_over_complete_collection","immutable":true})
    );
    assert_eq!(
        c["presentationBounds"],
        json!({"masksPerItem":opencut_editor_core::MAX_MASKS_PER_ITEM,"effectsPerItem":16,"commandsPerMask":4096,"gradientStops":64,"commandDetailsAtOnce":1,"gradientStopDetailsAtOnce":1})
    );
    assert_eq!(
        c["readOnlyMaskFields"],
        json!([
            "id",
            "source.type",
            "source.paint.type",
            "gradient geometry",
            "selected gradient stop offset",
            "selected gradient stop RGBA",
            "path command type"
        ])
    );
    assert_eq!(c["pathCoordinateType"], "finite_f64");
    assert_eq!(c["maskLabel"], "Fixed 64 local-pixel rectangle");
    assert_eq!(
        c["eligibility"],
        json!({"visual_leaves":["image_media","video_media","text","solid_color","rectangle","shape","svg","grid"],"visualLeafControls":["masks","matte","matteOnly","blendMode","effects"],"captionControls":["blendMode"],"groupControls":["effects","clip"],"componentInstanceControls":["effects","clip"],"noCompositingControls":["repeater","transition","audio_only_media"],"componentLocalMutation":false})
    );
    let root = tempfile::tempdir().unwrap();
    let f = crate::tests::fixture::seed(root.path());
    let p = f.project();
    for item in p.tracks.iter().flat_map(|t| &t.items) {
        let expected = match item {
            TimelineItem::Group(_) | TimelineItem::ComponentInstance(_) => Eligibility::Aggregate,
            TimelineItem::Caption(_) => Eligibility::Caption,
            TimelineItem::Repeater(_) | TimelineItem::Transition(_) => Eligibility::None,
            TimelineItem::Media(_) if crate::animation_inspector::audio_only(&p, item) => {
                Eligibility::None
            }
            _ => Eligibility::Leaf,
        };
        let selection = Selection::root(item.id());
        assert_eq!(controls::eligibility(&p, &selection, item), expected);
    }
    let mut leaf = item();
    leaf.visual_properties_mut().matte =
        serde_json::from_value(json!({"sourceId":"provider","channel":"alpha"})).unwrap();
    let fields = fs(&p, &leaf, &Cursor::default());
    let mut paths: Vec<_> = fields
        .iter()
        .map(|f| f.path.trim_start_matches('/').replace('/', "."))
        .collect();
    paths.sort();
    let mut expected: Vec<_> = c["leafFields"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    expected.sort();
    assert_eq!(paths, expected);
    let matte = field(&fields, "/matte/channel");
    let inspector_edit::FieldKind::Choice(choices) = matte.kind else {
        panic!("Expected matte choices")
    };
    assert_eq!(json!(choices), c["matteChannels"]);
    assert_eq!(
        c["editSemantics"],
        json!({"masks":{"omitted":"preserve","array":"ordered_replace","empty":"clear","null":"reject"},"effects":{"omitted":"preserve","array":"ordered_replace","empty":"clear","null":"reject"},"matte":{"omitted":"preserve","null":"clear"},"clip":{"omitted":"preserve","null":"clear"},"blendMode":{"omitted":"preserve","normal":"reset","null":"reject"}})
    );
}
#[test]
fn desktop_compositing_core_overflow_effect_target_and_locked_collection_failures_are_atomic() {
    let root = tempfile::tempdir().unwrap();
    let f = minimal_fixture(root.path());
    let p = f.project();
    let track = p.tracks[1].id.clone();
    let id=f.core.edit(&f.id,p.revision,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap().changed_ids[0].clone();
    let s = Selection::root(&id);
    let config = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    };
    let p = f.project();
    f.core.edit(&f.id,p.revision,op(json!({"operation":"update_item","itemId":id,"effects":[controls::default_effect("vignette","effect-1".into()).unwrap()]}))).unwrap();
    let p = f.project();
    f.core.edit(&f.id,p.revision,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[{"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"effect-1"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"linear"}]}]}))).unwrap();
    let p = f.project();
    let before = inventory(&f.core.paths().project_dir(&f.id).unwrap());
    let (edit, _) = controls::action(
        &p,
        &s,
        p.find_item(&id).unwrap(),
        &Cursor::default(),
        Action::DeleteEffect,
    )
    .unwrap();
    assert_eq!(
        config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
    let p = f.project();
    f.core.edit(&f.id,p.revision,op(json!({"operation":"update_item","itemId":id,"effects":(1..=16).map(|i|controls::default_effect("vignette",format!("effect-{i}")).unwrap()).collect::<Vec<_>>(),"masks":(1..=16).map(|i|controls::default_mask(format!("mask-{i}"))).collect::<Vec<_>>()}))).unwrap();
    let p = f.project();
    let before = inventory(&f.core.paths().project_dir(&f.id).unwrap());
    for action in [Action::AddMask, Action::AddEffect("glow")] {
        let (edit, _) = controls::action(
            &p,
            &s,
            p.find_item(&id).unwrap(),
            &Cursor::default(),
            action,
        )
        .unwrap();
        assert_eq!(
            config
                .execute(Command::Edit(p.revision, Box::new(edit)))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            inventory(&f.core.paths().project_dir(&f.id).unwrap()),
            before
        );
    }
    f.core
        .edit(
            &f.id,
            p.revision,
            op(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    let p = f.project();
    let before = inventory(&f.core.paths().project_dir(&f.id).unwrap());
    let (edit, _) = controls::action(
        &p,
        &s,
        p.find_item(&id).unwrap(),
        &Cursor::default(),
        Action::MoveMask(true),
    )
    .unwrap();
    assert_eq!(
        config
            .execute(Command::Edit(p.revision, Box::new(edit)))
            .unwrap_err()
            .code,
        ErrorCode::TrackLocked
    );
    assert_eq!(
        inventory(&f.core.paths().project_dir(&f.id).unwrap()),
        before
    );
}

#[test]
fn desktop_compositing_every_item_kind_root_local_and_media_facts() {
    let mut p = project();
    let items = all_item_kinds();
    let mut seen = std::collections::BTreeSet::new();
    for mut item in items {
        let kind = serde_json::to_value(&item).unwrap()["type"]
            .as_str()
            .unwrap()
            .to_owned();
        seen.insert(kind.clone());
        item.visual_properties_mut().masks = vec![controls::default_mask("mask-1".into())];
        item.visual_properties_mut().effects =
            vec![controls::default_effect("glow", "effect-1".into()).unwrap()];
        let media_types = if kind == "media" {
            vec![
                opencut_editor_core::MediaType::Image,
                opencut_editor_core::MediaType::Video,
                opencut_editor_core::MediaType::Audio,
            ]
        } else {
            vec![]
        };
        for media_type in media_types
            .into_iter()
            .map(Some)
            .chain((kind != "media").then_some(None))
        {
            if let (TimelineItem::Media(media), Some(media_type)) = (&item, media_type) {
                p.assets = vec![serde_json::from_value(json!({"id":media.asset_id,"mediaType":media_type,"fileName":"fixture","projectRelativePath":"media/fixture","durationMs":1000,"hasAudio":true})).unwrap()];
            }
            let expected = match kind.as_str() {
                "group" | "component_instance" => Eligibility::Aggregate,
                "caption" => Eligibility::Caption,
                "repeater" | "transition" => Eligibility::None,
                "media" if media_type == Some(opencut_editor_core::MediaType::Audio) => {
                    Eligibility::None
                }
                _ => Eligibility::Leaf,
            };
            let root = Selection::root(item.id());
            assert_eq!(
                controls::eligibility(&p, &root, &item),
                expected,
                "{kind} {media_type:?}"
            );
            let fields = controls::fields(&p, &root, &item, &Cursor::default());
            assert_eq!(
                fields.iter().any(|f| f.update_key == "masks"),
                expected == Eligibility::Leaf
            );
            assert_eq!(
                fields.iter().any(|f| f.update_key == "matte"),
                expected == Eligibility::Leaf
            );
            assert_eq!(
                fields.iter().any(|f| f.update_key == "effects"),
                matches!(expected, Eligibility::Leaf | Eligibility::Aggregate)
            );
            assert_eq!(
                fields.iter().any(|f| f.update_key == "blendMode"),
                matches!(expected, Eligibility::Leaf | Eligibility::Caption)
            );
            if expected == Eligibility::None {
                assert!(fields.is_empty());
            }
            let local = Selection {
                scope: "component:source".into(),
                instance_path: vec!["instance".into()],
                item_id: item.id().into(),
            };
            assert_eq!(controls::eligibility(&p, &local, &item), Eligibility::None);
            assert!(controls::fields(&p, &local, &item, &Cursor::default()).is_empty());
            for action in [
                Action::AddMask,
                Action::AddEffect("glow"),
                Action::SetClip(true),
                Action::ClearMatte,
            ] {
                assert!(controls::action(&p, &local, &item, &Cursor::default(), action).is_err());
            }
        }
    }
    assert_eq!(
        seen,
        [
            "component_instance",
            "group",
            "media",
            "text",
            "solid_color",
            "rectangle",
            "shape",
            "svg",
            "grid",
            "repeater",
            "caption",
            "transition"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
}

#[test]
fn desktop_compositing_nonempty_representative_predecessor_field_and_animation_equivalence() {
    let variants: Vec<TimelineItem> = serde_json::from_str(r###"[{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"rectangle","width":64,"height":32},"fill":null,"stroke":null},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"rectangle","width":64,"height":32},"fill":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"rectangle","width":64,"height":32},"fill":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"rectangle","width":64,"height":32},"fill":{"type":"radialGradient","center":{"x":32,"y":32},"radius":32,"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"roundedRectangle","width":64,"height":32,"radii":{"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}},"fill":null,"stroke":null},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"roundedRectangle","width":64,"height":32,"radii":{"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}},"fill":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"roundedRectangle","width":64,"height":32,"radii":{"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}},"fill":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"roundedRectangle","width":64,"height":32,"radii":{"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}},"fill":{"type":"radialGradient","center":{"x":32,"y":32},"radius":32,"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"ellipse","width":24,"height":16},"fill":null,"stroke":null},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"ellipse","width":24,"height":16},"fill":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"ellipse","width":24,"height":16},"fill":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"ellipse","width":24,"height":16},"fill":{"type":"radialGradient","center":{"x":32,"y":32},"radius":32,"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":64,"y":0}},{"type":"lineTo","to":{"x":64,"y":64}},{"type":"lineTo","to":{"x":0,"y":64}},{"type":"close"}]}},"fill":null,"stroke":null},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":64,"y":0}},{"type":"lineTo","to":{"x":64,"y":64}},{"type":"lineTo","to":{"x":0,"y":64}},{"type":"close"}]}},"fill":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":64,"y":0}},{"type":"lineTo","to":{"x":64,"y":64}},{"type":"lineTo","to":{"x":0,"y":64}},{"type":"close"}]}},"fill":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":64,"y":0}},{"type":"lineTo","to":{"x":64,"y":64}},{"type":"lineTo","to":{"x":0,"y":64}},{"type":"close"}]}},"fill":{"type":"radialGradient","center":{"x":32,"y":32},"radius":32,"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"rectangular","spacingX":8,"spacingY":12,"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"rectangular","spacingX":8,"spacingY":12,"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6},"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"diagonal","spacing":13,"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"diagonal","spacing":13,"stroke":{"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6},"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"isometric","spacing":14,"stroke":{"paint":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"isometric","spacing":14,"stroke":{"paint":{"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":64},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]},"width":2.5,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6},"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"dot","spacingX":8,"spacingY":12,"radius":1,"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"dot","spacingX":8,"spacingY":12,"radius":1,"paint":{"type":"solid","color":{"r":0.125,"g":0.25,"b":0.75,"a":0.625}}}},"transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6},"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1}},{"type":"text","id":"text","text":"Hi!","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[{"property":"opacity","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}],"document":{"runs":[{"text":"Hi","bold":true,"italic":false,"color":"#123456"},{"text":"!"}]},"style":{}},{"type":"text","id":"text","text":"Hi!","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[{"property":"opacity","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}],"document":{"runs":[{"text":"Hi","bold":true,"italic":false,"color":"#123456"},{"text":"!"}]},"style":{"outlineColor":"#123456","outlineWidthPx":2,"shadow":{"color":"#654321","opacity":0.625,"offsetX":-2,"offsetY":3},"paintLayers":[{"kind":"fill","color":"#123456","opacity":0.25},{"kind":"stroke","color":"#654321","opacity":0.5,"widthPx":2.5},{"kind":"shadow","color":"#abcdef","opacity":0.75,"offsetXPx":-3.25,"offsetYPx":4.5,"blurSigmaPx":2.75}],"layout":{"trackingPx":1.25,"lineHeightPx":30.5,"bounds":{"widthPx":100.5,"heightPx":80.25},"fit":"fit_box","wrap":"cluster","verticalAlignment":"center"}}},{"type":"group","id":"group","startMs":0,"durationMs":1000,"staggerMs":0},{"type":"group","id":"group","startMs":0,"durationMs":1000,"staggerMs":125},{"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"hidden":false,"zIndex":0,"stackOrder":0,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"slotValues":{},"staggerMs":0},{"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"hidden":false,"zIndex":0,"stackOrder":0,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"slotValues":{},"staggerMs":125},{"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"group"},"copies":1,"timeOffsetMs":0,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}},{"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"group"},"copies":1,"timeOffsetMs":-125,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}},{"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"group"},"copies":1,"timeOffsetMs":200,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}]"###).unwrap();
    for item in variants {
        let actual = inspector_edit::fields(&item);
        let readonly_path = matches!(&item, TimelineItem::Shape(s) if matches!(s.geometry, opencut_editor_core::ShapeGeometry::Path { .. }) && s.fill.is_none() && s.stroke.is_none());
        assert_eq!(
            actual.is_empty(),
            readonly_path,
            "legacy branch {}",
            item.id()
        );
        assert_eq!(actual, crate::compositing_predecessor::fields(&item));
    }
    let curves = [
        json!("linear"),
        json!("hold"),
        json!({"type":"cubic_bezier","x1":0.25,"y1":0.125,"x2":0.75,"y2":0.875}),
        json!({"type":"spring","mass":1,"stiffness":100,"damping":10,"initialVelocity":0.25}),
    ];
    for property in ["transform.opacity", "audio.gain_db"] {
        for curve in curves.clone() {
            for loop_value in [
                None,
                Some(json!({"mode":"repeat","iterations":3})),
                Some(json!({"mode":"ping_pong","iterations":"infinite"})),
            ] {
                let mut channel = json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.125},"curve":curve},{"timeMs":100,"value":{"type":"scalar","value":0.75},"curve":curve}]});
                if let Some(loop_value) = loop_value {
                    channel["loop"] = loop_value;
                }
                let mut item = item();
                item.visual_properties_mut().animation_channels =
                    vec![serde_json::from_value(channel).unwrap(); 2];
                for cursor in [
                    crate::animation_inspector::Cursor::default(),
                    crate::animation_inspector::Cursor {
                        channel: 1,
                        key: 1,
                        legacy: 0,
                    },
                    crate::animation_inspector::Cursor {
                        channel: usize::MAX,
                        key: usize::MAX,
                        legacy: usize::MAX,
                    },
                ] {
                    for audio in [false, true] {
                        let actual = crate::animation_inspector::fields(&item, cursor, audio);
                        assert_eq!(actual.is_empty(), audio && property != "audio.gain_db");
                        assert_eq!(
                            actual,
                            crate::compositing_predecessor::animation_fields(&item, cursor, audio)
                        );
                    }
                }
            }
        }
    }
}

fn all_item_kinds() -> Vec<TimelineItem> {
    serde_json::from_str(r###"[{"type":"group","id":"group","startMs":0,"durationMs":1000},{"type":"rectangle","id":"box","startMs":0,"durationMs":1000,"width":20,"height":20,"color":"#FF0000","transform":{"positionX":4,"positionY":4,"scale":1,"opacity":1},"keyframes":[{"property":"position","timeMs":0,"value":{"type":"position","x":4,"y":4},"easing":"linear"},{"property":"position","timeMs":1000,"value":{"type":"position","x":14,"y":4},"easing":"linear"}]},{"type":"transition","id":"fade","transitionType":"fade","fromItemId":"box","startMs":700,"durationMs":300,"stackOrder":1},{"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"hidden":false,"zIndex":0,"stackOrder":0,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"slotValues":{}},{"type":"text","id":"text","text":"Hello","fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[{"property":"opacity","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}],"document":{"runs":[{"text":"Hello"}]}},{"type":"solid_color","id":"text","color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[{"property":"volume","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}]},{"type":"media","id":"media","assetId":"component-fixture-asset","startMs":0,"durationMs":1000,"sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[{"property":"volume","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"}]},{"type":"caption","id":"caption","text":"Moved","startMs":0,"durationMs":1000,"style":{"fontSize":24,"color":"#ffffff","backgroundColor":"#000000","bottomMarginPx":4320},"source":{"assetId":"component-fixture-asset","providerId":"provider","modelId":"model","language":"en","generatedAtMs":1,"originalText":"Original","words":[{"word":"Original","startMs":2000,"endMs":3000}]}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"shape","geometry":{"type":"rectangle","width":64,"height":64},"fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"svg","document":{"version":1,"width":64,"height":64,"viewBox":[0,0,64,64],"shapes":[]}},{"id":"extra","startMs":0,"durationMs":1000,"keyframes":[],"type":"grid","grid":{"width":64,"height":64,"pattern":{"type":"dot","spacingX":8,"spacingY":8,"radius":1,"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}}}},{"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":"group"},"copies":1,"timeOffsetMs":0,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}]"###).unwrap()
}

#[test]
fn desktop_compositing_predecessor_equivalence_for_rich_optional_fields_and_nonempty_animation() {
    let base = all_item_kinds();
    let mut cases = base.clone();
    let shape = base
        .iter()
        .find(|i| matches!(i, TimelineItem::Shape(_)))
        .unwrap();
    let paints = [
        json!({"type":"solid","color":{"r":0.25,"g":0.5,"b":0.75,"a":0.625}}),
        json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":64,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":1,"b":0,"a":1}}]}),
        json!({"type":"radialGradient","center":{"x":32,"y":32},"radius":32,"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":1,"b":0,"a":1}}]}),
    ];
    for geometry in [
        json!({"type":"rectangle","width":64,"height":32}),
        json!({"type":"roundedRectangle","width":64,"height":32,"radii":{"topLeft":1,"topRight":2,"bottomRight":3,"bottomLeft":4}}),
        json!({"type":"ellipse","width":64,"height":32}),
        json!({"type":"line","start":{"x":0,"y":0},"end":{"x":64,"y":32}}),
        json!({"type":"polygon","points":[{"x":0,"y":0},{"x":32,"y":64},{"x":64,"y":0}]}),
        json!({"type":"star","center":{"x":32,"y":32},"outerRadius":30,"innerRadius":15,"pointCount":5,"rotationDeg":12}),
        json!({"type":"path","path":catalog()["defaultMask"]["source"]["path"]}),
    ] {
        for paint in &paints {
            let mut value = serde_json::to_value(shape).unwrap();
            value["geometry"] = geometry.clone();
            value["fill"] = paint.clone();
            value["stroke"] = json!({"paint":paint,"width":2.5,"dash":[1,2],"dashOffset":0.5,"lineCap":"round","lineJoin":"round","miterLimit":4});
            let item: TimelineItem = serde_json::from_value(value).unwrap();
            if let TimelineItem::Shape(s) = &item {
                s.geometry.validate().unwrap();
                s.fill.as_ref().unwrap().validate().unwrap();
                s.stroke.as_ref().unwrap().validate().unwrap();
            }
            cases.push(item);
        }
    }
    let grid = base
        .iter()
        .find(|i| matches!(i, TimelineItem::Grid(_)))
        .unwrap();
    for kind in ["rectangular", "diagonal", "isometric", "dot"] {
        for explicit in [false, true] {
            let mut value = serde_json::to_value(grid).unwrap();
            let mut pattern = json!({"type":kind});
            if matches!(kind, "rectangular" | "dot") {
                pattern["spacingX"] = json!(8.5);
                pattern["spacingY"] = json!(12.5);
            } else {
                pattern["spacing"] = json!(9.5);
            }
            if kind == "dot" {
                pattern["radius"] = json!(1.5);
                pattern["paint"] = paints[0].clone();
            } else {
                pattern["stroke"] = json!({"paint":paints[0],"width":2,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4});
            }
            value["grid"]["pattern"] = pattern;
            value["transform"] = json!({"positionX":17,"positionY":23,"scale":1.5,"opacity":0.625});
            if explicit {
                let transform = opencut_editor_core::Transform2D {
                    rotation_deg: 12.5,
                    opacity: 0.375,
                    ..Default::default()
                };
                value["transform2d"] = json!(transform);
            } else {
                value.as_object_mut().unwrap().remove("transform2d");
            }
            cases.push(serde_json::from_value(value).unwrap());
        }
    }
    let text = base
        .iter()
        .find(|i| matches!(i, TimelineItem::Text(_)))
        .unwrap();
    let layers: Value = serde_json::from_str(include_str!(
        "../../../contracts/styled-text-layers-v1.json"
    ))
    .unwrap();
    for layout in [
        None,
        Some(json!({})),
        Some(
            json!({"trackingPx":1.25,"lineHeightPx":28.5,"bounds":{"widthPx":123.5,"heightPx":64.5},"fit":"fit_box"}),
        ),
    ] {
        for paint_layers in [
            None,
            Some(json!([])),
            Some(layers["valid"][3]["paintLayers"].clone()),
        ] {
            let mut value = serde_json::to_value(text).unwrap();
            value["text"] = json!("Hello");
            value["document"] = json!({"runs":[{"text":"He","bold":true,"italic":false,"color":"#ff0088"},{"text":"llo","italic":true}]});
            if let Some(layout) = &layout {
                value["style"]["layout"] = layout.clone();
            } else {
                value["style"].as_object_mut().unwrap().remove("layout");
            }
            if let Some(layers) = paint_layers {
                value["style"]["paintLayers"] = layers;
            } else {
                value["style"]
                    .as_object_mut()
                    .unwrap()
                    .remove("paintLayers");
            }
            cases.push(serde_json::from_value(value).unwrap());
        }
    }
    for mut item in base {
        match &mut item {
            TimelineItem::Group(v) => v.stagger_ms = 125,
            TimelineItem::ComponentInstance(v) => v.stagger_ms = 250,
            TimelineItem::Repeater(v) => v.repeater.time_offset_ms = -125,
            _ => continue,
        }
        cases.push(item);
    }
    for (n, item) in cases.iter().enumerate() {
        assert_eq!(
            inspector_edit::fields(item),
            crate::compositing_predecessor::fields(item),
            "case {n} {}",
            item.id()
        );
    }
    let curves = [
        json!("hold"),
        json!("linear"),
        json!({"type":"cubic_bezier","x1":0.25,"y1":0.125,"x2":0.75,"y2":0.875}),
        json!({"type":"spring","mass":1,"stiffness":100,"damping":12,"initialVelocity":0.25}),
    ];
    for curve in curves {
        for loop_value in [
            None,
            Some(json!({"mode":"repeat","iterations":3})),
            Some(json!({"mode":"ping_pong","iterations":"infinite"})),
        ] {
            let mut animated = item();
            let mut channel = json!({"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.125},"curve":curve},{"timeMs":500,"value":{"type":"scalar","value":0.875},"curve":"hold"}]});
            if let Some(loop_value) = loop_value {
                channel["loop"] = loop_value;
            }
            animated.visual_properties_mut().animation_channels=serde_json::from_value(json!([channel,{"property":"audio.gain_db","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":-12.5},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":6.25},"curve":"hold"}]}])).unwrap();
            for channel in [0, 1, usize::MAX] {
                for key in [0, 1, usize::MAX] {
                    for audio in [false, true] {
                        let cursor = crate::animation_inspector::Cursor {
                            channel,
                            key,
                            legacy: usize::MAX,
                        };
                        let current = crate::animation_inspector::fields(&animated, cursor, audio);
                        assert_eq!(
                            current,
                            crate::compositing_predecessor::animation_fields(
                                &animated, cursor, audio
                            )
                        );
                        if !audio || channel != 0 {
                            assert!(!current.is_empty());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn desktop_compositing_all_path_coordinate_variants_commit_and_preserve_neighbors() {
    let root = tempfile::tempdir().unwrap();
    let f = minimal_fixture(root.path());
    let p = f.project();
    let id=f.core.edit(&f.id,p.revision,op(json!({"operation":"add_rectangle","trackId":p.tracks[1].id,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":opencut_editor_core::Transform::default()}))).unwrap().changed_ids[0].clone();
    let p = f.project();
    let mut mask = catalog()["defaultMask"].clone();
    mask["source"]["path"]["commands"] = json!([{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":16,"y":0}},{"type":"quadraticTo","control":{"x":32,"y":0},"to":{"x":32,"y":32}},{"type":"cubicTo","control1":{"x":32,"y":48},"control2":{"x":16,"y":64},"to":{"x":0,"y":64}},{"type":"close"}]);
    f.core.edit(&f.id,p.revision,op(json!({"operation":"update_item","itemId":id,"masks":[mask,catalog()["defaultMask"].as_object().unwrap().iter().map(|(k,v)|(k.clone(),if k=="id" {json!("neighbor")}else{v.clone()})).collect::<serde_json::Map<_,_>>()],"effects":[controls::default_effect("glow","neighbor-effect".into()).unwrap()]}))).unwrap();
    let p = f.project();
    f.core.edit(&f.id,p.revision,op(json!({"operation":"set_animation_channels","itemId":id,"animationChannels":[{"property":"mask.transform.opacity","target":{"kind":"mask","scope":"root","id":"mask-1"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.75},"curve":"linear"}]}]}))).unwrap();
    let config = Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    };
    let c = catalog();
    for (command, variant) in [
        (0, "moveTo"),
        (1, "lineTo"),
        (2, "quadraticTo"),
        (3, "cubicTo"),
    ] {
        for coordinate in c["pathCommandFields"][variant].as_array().unwrap() {
            let p = f.project();
            let item = p.find_item(&id).unwrap();
            let cursor = Cursor {
                mask_id: Some("mask-1".into()),
                command,
                ..Cursor::default()
            };
            let path = format!(
                "/masks/0/source/path/commands/{command}/{}",
                coordinate.as_str().unwrap().replace('.', "/")
            );
            let selected = field(&fs(&p, item, &cursor), &path);
            let mut expected = serde_json::to_value(item).unwrap();
            *expected.pointer_mut(&path).unwrap() = json!(0.12345678901234566);
            let edit = inspector_edit::build(item, &selected, "0.12345678901234566").unwrap();
            let result = config
                .execute(Command::Edit(p.revision, Box::new(edit)))
                .unwrap();
            assert_eq!(result.revision, p.revision + 1);
            assert_eq!(
                serde_json::to_value(result.find_item(&id).unwrap()).unwrap(),
                expected
            );
        }
    }
    let p = config.execute(Command::Refresh).unwrap();
    let item = p.find_item(&id).unwrap();
    let cursor = Cursor {
        mask_id: Some("mask-1".into()),
        command: 4,
        ..Cursor::default()
    };
    assert!(
        !fs(&p, item, &cursor)
            .iter()
            .any(|f| f.path.contains("/commands/"))
    );
}
