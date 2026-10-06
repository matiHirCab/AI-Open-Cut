use opencut_editor_core::{EditOperation, EditorCore, PathPolicy, ProjectSettings};
use serde_json::{Value, json};
fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn setup() -> (tempfile::TempDir, EditorCore, String, String, String) {
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
            "Track mattes",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let add = |revision, color| {
        core.edit(&id,revision,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":24,"height":16,"color":color,"transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone()
    };
    let provider = add(0, "#ffffff");
    let recipient = add(1, "#ff0000");
    (root, core, id, provider, recipient)
}
#[test]
fn public_matte_fields_are_observable_and_omission_null_and_visibility_are_distinct() {
    let (_root, core, id, provider, recipient) = setup();
    let matte = json!({"sourceId":provider,"channel":"alpha"});
    core.edit(
        &id,
        2,
        op(json!({"operation":"update_item","itemId":recipient,"matte":matte,"matteOnly":true})),
    )
    .unwrap();
    let read = || {
        serde_json::to_value(
            core.get_project(&id)
                .unwrap()
                .find_item(&recipient)
                .unwrap(),
        )
        .unwrap()
    };
    let item = read();
    assert_eq!(
        item["matte"], matte,
        "approved matte reference must persist"
    );
    assert_eq!(item["matteOnly"], true);
    core.edit(
        &id,
        3,
        op(json!({"operation":"update_item","itemId":recipient,"color":"#00ff00"})),
    )
    .unwrap();
    assert_eq!(read()["matte"], matte);
    assert_eq!(read()["matteOnly"], true);
    core.edit(
        &id,
        4,
        op(json!({"operation":"update_item","itemId":recipient,"matte":null,"matteOnly":false})),
    )
    .unwrap();
    let item = read();
    assert!(item.get("matte").is_none());
    assert!(item.get("matteOnly").is_none());
    assert!(
        serde_json::from_value::<EditOperation>(
            json!({"operation":"update_item","itemId":recipient,"matteOnly":null})
        )
        .is_err()
    );
}

#[test]
fn matte_graph_failures_are_atomic_and_delete_clear_batch_validates_final_candidate() {
    use opencut_editor_core::ErrorCode;
    let (_root, core, id, provider, recipient) = setup();
    let edit = |item: &str, source: &str| {
        op(
            json!({"operation":"update_item","itemId":item,"matte":{"sourceId":source,"channel":"alpha"}}),
        )
    };
    let before = serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap();
    for (source, code) in [
        (recipient.as_str(), ErrorCode::InvalidArgument),
        ("absent", ErrorCode::ItemNotFound),
    ] {
        assert_eq!(
            core.edit(&id, 2, edit(&recipient, source))
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(
            serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap(),
            before
        );
    }
    core.edit(&id, 2, edit(&recipient, &provider)).unwrap();
    let valid = serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap();
    assert_eq!(
        core.edit(&id, 3, edit(&provider, &recipient))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap(),
        valid
    );
    assert_eq!(
        core.edit(
            &id,
            3,
            op(json!({"operation":"delete_item","itemId":provider}))
        )
        .unwrap_err()
        .code,
        ErrorCode::ItemNotFound
    );
    core.edit_batch(
        &id,
        3,
        vec![
            op(json!({"operation":"delete_item","itemId":provider})),
            op(json!({"operation":"update_item","itemId":recipient,"matte":null})),
        ],
    )
    .unwrap();
    assert!(
        core.get_project(&id)
            .unwrap()
            .find_item(&provider)
            .is_none()
    );
    assert!(
        core.get_project(&id)
            .unwrap()
            .find_item(&recipient)
            .unwrap()
            .visual_properties()
            .matte
            .is_none()
    );
}

#[test]
fn strict_matte_wire_and_source_schema_guards_preserve_old_defaults() {
    use opencut_editor_core::{MatteReference, Project};
    for raw in [
        r#"{"sourceId":"x","sourceId":"y","channel":"alpha"}"#,
        r#"{"sourceId":"x","channel":"alpha","extra":1}"#,
        r#"["x","alpha"]"#,
        r#"null"#,
    ] {
        assert!(
            serde_json::from_str::<MatteReference>(raw).is_err(),
            "{raw}"
        );
    }
    let (_root, core, id, provider, _recipient) = setup();
    let mut value = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    value["schemaVersion"] = json!(33);
    assert!(serde_json::from_value::<Project>(value.clone()).is_ok());
    for (name, metadata) in [
        ("matte", Value::Null),
        ("matteOnly", json!(false)),
        ("matte", json!({"sourceId":provider,"channel":"alpha"})),
    ] {
        let mut injected = value.clone();
        injected["tracks"][1]["items"][0][name] = metadata;
        assert!(
            serde_json::from_value::<Project>(injected).is_err(),
            "old schema presence {name}"
        );
    }
}

#[test]
fn component_transient_matte_dag_is_deferred_only_until_final_batch_or_draft_candidate() {
    use opencut_editor_core::{BatchEditOperation, ErrorCode};
    let (_root, core, id, provider, recipient) = setup();
    let project = core.get_project(&id).unwrap();
    let mut tracks = serde_json::to_value(vec![project.tracks[1].clone()]).unwrap();
    tracks[0]["items"][0]["matte"] = json!({"sourceId":recipient,"channel":"alpha"});
    tracks[0]["items"][1]["matte"] = json!({"sourceId":provider,"channel":"alpha"});
    let create = json!({"operation":"component_create","name":"Local","width":64,"height":64,"durationMs":1000,"tracks":tracks});
    assert_eq!(
        core.edit(&id, 2, op(create.clone())).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut fixed = tracks.clone();
    fixed[0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("matte");
    let mut aliased = create;
    aliased["resultAlias"] = json!("local");
    let update = json!({"operation":"component_update","componentId":"@local","name":"Local","width":64,"height":64,"durationMs":1000,"tracks":fixed});
    let operations: Vec<BatchEditOperation> =
        serde_json::from_value(json!([aliased, update])).unwrap();
    core.edit_batch(&id, 2, operations).unwrap();
    let component = core.get_project(&id).unwrap().components[0].id.clone();
    // Drafts carry canonical component IDs, while their local sourceId remains literal.
    let bad = op(
        json!({"operation":"component_update","componentId":component,"name":"Local","width":64,"height":64,"durationMs":1000,"tracks":tracks}),
    );
    let good = op(
        json!({"operation":"component_update","componentId":component,"name":"Local","width":64,"height":64,"durationMs":1000,"tracks":fixed}),
    );
    let draft = core.create_draft(&id, 3, vec![bad, good], None).unwrap();
    core.commit_draft(&id, &draft.id, 3).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().components[0].tracks[0].items[1]
            .visual_properties()
            .matte
            .as_ref()
            .unwrap()
            .source_id,
        provider
    );
}

fn inventory(
    core: &EditorCore,
    id: &str,
) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn walk(
        dir: &std::path::Path,
        out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = std::collections::BTreeMap::new();
    walk(&core.paths().project_dir(id).unwrap(), &mut out);
    out
}
fn predecessor33(value: &mut Value) {
    value["schemaVersion"] = json!(33);
    for track in value["tracks"].as_array_mut().unwrap() {
        for item in track["items"].as_array_mut().unwrap() {
            item.as_object_mut().unwrap().remove("matte");
            item.as_object_mut().unwrap().remove("matteOnly");
        }
    }
    for component in value["components"].as_array_mut().unwrap() {
        for track in component["tracks"].as_array_mut().unwrap() {
            for item in track["items"].as_array_mut().unwrap() {
                item.as_object_mut().unwrap().remove("matte");
                item.as_object_mut().unwrap().remove("matteOnly");
            }
        }
    }
}
#[test]
fn genuine33_current_undo_redo_and_own_base_draft_adopt34_without_semantic_changes() {
    use opencut_editor_core::PROJECT_SCHEMA_VERSION;
    let (_root, core, id, _provider, recipient) = setup();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    let mask = catalog["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["accepted"] == true)
        .unwrap()["value"]
        .clone();
    core.edit(
        &id,
        2,
        op(json!({"operation":"update_item","itemId":recipient,"masks":[mask]})),
    )
    .unwrap();
    let draft = core
        .create_draft(
            &id,
            3,
            vec![op(
                json!({"operation":"update_item","itemId":recipient,"color":"#0000ff"}),
            )],
            None,
        )
        .unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let project_path = dir.join("project.json");
    let history_path = dir.join("history.json");
    let mut source: Value = serde_json::from_slice(&std::fs::read(&project_path).unwrap()).unwrap();
    predecessor33(&mut source);
    let mut old = source.clone();
    old["revision"] = json!(2);
    let mut redo = source.clone();
    redo["revision"] = json!(77);
    let history = json!({"undo":[old],"redo":[redo]});
    std::fs::write(&project_path, serde_json::to_vec(&source).unwrap()).unwrap();
    std::fs::write(&history_path, serde_json::to_vec(&history).unwrap()).unwrap();
    let before = inventory(&core, &id);
    let migrated = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    let mut expected = source.clone();
    expected["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION);
    assert_eq!(migrated, expected);
    let mut expected_history = history;
    for lane in ["undo", "redo"] {
        for snapshot in expected_history[lane].as_array_mut().unwrap() {
            snapshot["schemaVersion"] = json!(PROJECT_SCHEMA_VERSION);
        }
    }
    let actual_history: Value =
        serde_json::from_slice(&std::fs::read(&history_path).unwrap()).unwrap();
    assert_eq!(actual_history, expected_history);
    assert_eq!(core.get_draft(&id, &draft.id).unwrap().base_revision, 3);
    let after = inventory(&core, &id);
    for (path, bytes) in before {
        if path != project_path && path != history_path {
            assert_eq!(after[&path], bytes, "migration changed {}", path.display());
        }
    }
    let stable = inventory(&core, &id);
    EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(inventory(&core, &id), stable);
}

#[test]
fn premature33_stored_and_draft_field_presence_rejects_before_any_inventory_change() {
    use opencut_editor_core::ErrorCode;
    for draft_field in [false, true] {
        for (field, value) in [
            ("matte", Value::Null),
            ("matteOnly", json!(false)),
            ("matteOnly", json!(true)),
            ("matteOnly", Value::Null),
            ("matte", json!({})),
            ("matte", json!({"sourceId":"provider","channel":"unknown"})),
        ] {
            let (_root, core, id, provider, recipient) = setup();
            let draft = core
                .create_draft(
                    &id,
                    2,
                    vec![op(
                        json!({"operation":"update_item","itemId":recipient,"color":"#00ff00"}),
                    )],
                    None,
                )
                .unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let project_path = dir.join("project.json");
            let mut source: Value =
                serde_json::from_slice(&std::fs::read(&project_path).unwrap()).unwrap();
            predecessor33(&mut source);
            if draft_field {
                let path = dir.join("drafts").join(format!("{}.json", draft.id));
                let mut raw: Value =
                    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                raw["operations"][0][field] = value;
                std::fs::write(path, serde_json::to_vec(&raw).unwrap()).unwrap();
            } else {
                source["tracks"][1]["items"][0][field] = value;
            }
            std::fs::write(project_path, serde_json::to_vec(&source).unwrap()).unwrap();
            let before = inventory(&core, &id);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "{draft_field}/{field}"
            );
            assert!(!error.retryable);
            assert_eq!(inventory(&core, &id), before);
            let _ = provider;
        }
    }
}

#[test]
fn canonical_references_utf8_bound_and_raw_duplicates_match_public_domain() {
    use opencut_editor_core::MatteReference;
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/track-mattes-v1.json")).unwrap();
    for (field, accepted) in [("validReferences", true), ("invalidReferences", false)] {
        for value in catalog[field].as_array().unwrap() {
            let result = serde_json::from_value::<MatteReference>(value.clone())
                .is_ok_and(|m| m.validate().is_ok());
            assert_eq!(result, accepted, "canonical reference {value}");
        }
    }
    for json in catalog["rawDuplicateReferences"].as_array().unwrap() {
        assert!(serde_json::from_str::<MatteReference>(json.as_str().unwrap()).is_err());
    }
    for (id, accepted) in [("é".repeat(64), true), ("é".repeat(65), false)] {
        let reference =
            serde_json::from_value::<MatteReference>(json!({"sourceId":id,"channel":"alpha"}))
                .unwrap();
        assert_eq!(reference.validate().is_ok(), accepted);
    }
}

#[test]
fn matte_frame_excess_rejects_before_missing_backend_workspace_and_output_changes() {
    use opencut_editor_core::{ErrorCode, Renderer};
    let (_root, core, id, provider, recipient) = setup();
    let mut project = core.get_project(&id).unwrap();
    project.settings.width = 256;
    project.settings.height = 128;
    let mut source = project.find_item(&provider).unwrap().clone();
    source.visual_properties_mut().matte_only = true;
    let recipient_source = project.find_item(&recipient).unwrap().clone();
    let track = project
        .tracks
        .iter_mut()
        .find(|t| t.items.iter().any(|i| i.id() == provider))
        .unwrap();
    track.items = vec![source];
    for index in 0..2048 {
        let mut item = recipient_source.clone();
        let opencut_editor_core::TimelineItem::Rectangle(rectangle) = &mut item else {
            panic!("rectangle fixture changed")
        };
        rectangle.id = format!("recipient-{index}");
        item.visual_properties_mut().stack_order = (index + 1) as u32;
        item.visual_properties_mut().matte = Some(opencut_editor_core::MatteReference {
            source_id: provider.clone(),
            channel: opencut_editor_core::MatteChannel::Alpha,
        });
        track.items.push(item);
    }
    let dir = core.paths().project_dir(&id).unwrap();
    let before = inventory(&core, &id);
    // The intentionally absent backend would return DEPENDENCY_UNAVAILABLE if
    // any process stage were reached before the complete frame work guard.
    let renderer = Renderer::new(
        "/nonexistent/opencut-matte-ffmpeg",
        "/nonexistent/opencut-matte-ffprobe",
        None,
    );
    let error = renderer.render_preview(&project, &dir, 400).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(
        error.message.contains("matte work"),
        "unexpected guard {error:?}"
    );
    assert!(!error.retryable);
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            opencut_editor_core::PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 256,
                height: 128,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(range.code, ErrorCode::InvalidArgument);
    assert!(range.message.contains("matte work"));
    let export_dir = core.paths().exports_root().to_owned();
    std::fs::create_dir_all(&export_dir).unwrap();
    let output = export_dir.join("unchanged.mp4");
    std::fs::write(&output, b"preexisting output must survive").unwrap();
    let export = renderer
        .export_video(
            &project,
            &dir,
            opencut_editor_core::ExportOptions {
                output: &output,
                width: 256,
                height: 128,
                overwrite: true,
            },
            |_| {},
        )
        .unwrap_err();
    assert_eq!(export.code, ErrorCode::InvalidArgument);
    assert!(export.message.contains("matte work"));
    assert_eq!(
        std::fs::read(&output).unwrap(),
        b"preexisting output must survive"
    );
    assert_eq!(inventory(&core, &id), before);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let revision = project.revision;
    let draft = core
        .create_draft(
            &id,
            revision,
            vec![op(
                json!({"operation":"update_item","itemId":"recipient-0","color":"#00ff00"}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    let before = inventory(&core, &id);
    let draft_error = renderer.render_preview(&candidate, &dir, 400).unwrap_err();
    assert_eq!(draft_error.code, ErrorCode::InvalidArgument);
    assert!(draft_error.message.contains("matte work"));
    assert_eq!(
        inventory(&core, &id),
        before,
        "resource/work rejection created workspace/output bytes"
    );
}

#[test]
fn existing_matte_graph_failures_do_not_publish_unrelated_retained33_adoption() {
    use opencut_editor_core::ErrorCode;
    for batch in [false, true] {
        let (_root, core, id, provider, recipient) = setup();
        core.edit(&id,2,op(json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":provider,"channel":"alpha"}}))).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let history_path = dir.join("history.json");
        let mut history: Value =
            serde_json::from_slice(&std::fs::read(&history_path).unwrap()).unwrap();
        predecessor33(&mut history["undo"][0]);
        std::fs::write(&history_path, serde_json::to_vec(&history).unwrap()).unwrap();
        let before = inventory(&core, &id);
        let deletion = op(json!({"operation":"delete_item","itemId":provider}));
        let error = if batch {
            core.edit_batch(
                &id,
                3,
                vec![
                    op(json!({"operation":"update_item","itemId":recipient,"color":"#00ff00"})),
                    deletion,
                ],
            )
            .unwrap_err()
        } else {
            core.edit(&id, 3, deletion).unwrap_err()
        };
        assert_eq!(error.code, ErrorCode::ItemNotFound);
        assert!(!error.retryable);
        assert_eq!(
            inventory(&core, &id),
            before,
            "failed existing graph mutation adopted unrelated history"
        );
    }
}

#[test]
fn hidden_and_unused_scoped_dags_reach_each_inclusive_edge_and_depth_bound() {
    use opencut_editor_core::{ErrorCode, MatteChannel, MatteReference, TimelineItem};
    fn leaves(seed: &TimelineItem, prefix: &str, edges: usize, chain: bool) -> Vec<TimelineItem> {
        (0..=edges)
            .map(|index| {
                let mut item = seed.clone();
                let TimelineItem::Rectangle(rect) = &mut item else {
                    panic!("rectangle fixture")
                };
                rect.id = format!("{prefix}-{index}");
                rect.visual_properties.stack_order = index as u32;
                rect.visual_properties.hidden = true;
                rect.visual_properties.matte_only = true;
                if index > 0 {
                    rect.visual_properties.matte = Some(MatteReference {
                        source_id: format!("{prefix}-{}", if chain { index - 1 } else { 0 }),
                        channel: MatteChannel::Alpha,
                    });
                }
                item
            })
            .collect()
    }
    for (root_edges, local_edges, extra_edges, chain, accepted) in [
        (2048, 0, 0, false, true),
        (2049, 0, 0, false, false),
        (2048, 2048, 0, false, true),
        (2048, 2048, 1, false, false),
        (32, 0, 0, true, true),
        (33, 0, 0, true, false),
    ] {
        let (_root, core, id, provider, _) = setup();
        let mut project = core.get_project(&id).unwrap();
        let seed = project.find_item(&provider).unwrap().clone();
        let root_track = project
            .tracks
            .iter_mut()
            .find(|t| t.items.iter().any(|i| i.id() == provider))
            .unwrap();
        root_track.items = leaves(&seed, "root", root_edges, chain);
        if local_edges > 0 {
            let mut tracks = vec![root_track.clone()];
            tracks[0].id = "local".into();
            tracks[0].items = leaves(&seed, "local", local_edges, false);
            project.components.push(serde_json::from_value(json!({"id":"unused","name":"Unused","width":64,"height":64,"durationMs":1000,"tracks":tracks,"slots":[]})).unwrap());
        }
        if extra_edges > 0 {
            let mut track = project.tracks[1].clone();
            track.id = "extra".into();
            track.items = leaves(&seed, "extra", extra_edges, false);
            project.components.push(serde_json::from_value(json!({"id":"extra-definition","name":"Extra","width":64,"height":64,"durationMs":1000,"tracks":[track],"slots":[]})).unwrap());
        }
        let path = core.paths().project_dir(&id).unwrap().join("project.json");
        std::fs::write(path, serde_json::to_vec(&project).unwrap()).unwrap();
        let before = inventory(&core, &id);
        let result = core.get_project(&id);
        if accepted {
            assert!(
                result.is_ok(),
                "inclusive scoped/depth bound {root_edges}/{local_edges}/{chain}: {result:?}"
            );
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(!error.retryable);
        }
        assert_eq!(
            inventory(&core, &id),
            before,
            "DAG admission changed authoritative inventory"
        );
    }
}

#[test]
fn hidden_referenced_provider_safe_endpoints_do_not_hide_unsafe_continuous_mask_interior() {
    let authored = |sy: f64, skew: f64| {
        json!({"id":"reveal","source":{"type":"path","path":{"fillRule":"nonzero","commands":[
        {"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":600,"y":0}},
        {"type":"lineTo","to":{"x":600,"y":1}},{"type":"lineTo","to":{"x":0,"y":1}},{"type":"close"}]},
        "paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}},"channel":"alpha","operation":"intersect","inverted":false,"featherPx":0,"expansionPx":0,
        "transform":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":0.2,"scaleY":sy,"rotationDeg":0,"skewXDeg":skew,"skewYDeg":-72,"opacity":1}})
    };
    // Independent hand geometry: both endpoints604x5 /5855x14 are safe,
    // but the coupled interior needs21259x40, beyond the unchanged16384axis.
    for (sy, skew) in [(0.01, -75.), (24., 12.)] {
        let (_root, core, id, provider, recipient) = setup();
        core.edit_batch(&id,2,vec![
            op(json!({"operation":"update_item","itemId":provider,"masks":[authored(sy,skew)],"matteOnly":true})),
            op(json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":provider,"channel":"alpha"}})),
            op(json!({"operation":"set_item_visibility","itemId":provider,"hidden":true}))
        ]).unwrap();
    }
    let (_root, core, id, provider, recipient) = setup();
    core.edit_batch(&id,2,vec![
        op(json!({"operation":"update_item","itemId":provider,"masks":[authored(0.01,-75.)],"matteOnly":true})),
        op(json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":provider,"channel":"alpha"}})),
        op(json!({"operation":"set_item_visibility","itemId":provider,"hidden":true}))
    ]).unwrap();
    let scalar = |property: &str, a: f64, b: f64| {
        json!({"property":property,"target":{"kind":"mask","scope":"root","id":"reveal"},"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":a},"curve":"linear"},
        {"timeMs":999,"value":{"type":"scalar","value":b},"curve":"linear"}]})
    };
    let before = inventory(&core, &id);
    let error = core
        .edit(
            &id,
            3,
            op(
                json!({"operation":"set_animation_channels","itemId":provider,"animationChannels":[
        scalar("mask.transform.scale_y",0.01,24.),scalar("mask.transform.skew_x_deg",-75.,12.)]}),
            ),
        )
        .unwrap_err();
    assert_eq!(error.code, opencut_editor_core::ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn retained_component33_defaults_adopt_and_undo_redo_without_read_rewrites() {
    let (_root, core, id, _, _) = setup();
    let tracks =
        serde_json::to_value(vec![core.get_project(&id).unwrap().tracks[1].clone()]).unwrap();
    let component = core.edit(&id, 2, op(json!({"operation":"component_create","name":"Retained local","width":64,"height":64,"durationMs":1000,"tracks":tracks}))).unwrap().changed_ids[0].clone();
    let dir = core.paths().project_dir(&id).unwrap();
    let project_path = dir.join("project.json");
    let history_path = dir.join("history.json");
    let mut raw: Value = serde_json::from_slice(&std::fs::read(&project_path).unwrap()).unwrap();
    predecessor33(&mut raw);
    let mut history: Value =
        serde_json::from_slice(&std::fs::read(&history_path).unwrap()).unwrap();
    for lane in ["undo", "redo"] {
        for snapshot in history[lane].as_array_mut().unwrap() {
            predecessor33(snapshot);
        }
    }
    std::fs::write(&project_path, serde_json::to_vec(&raw).unwrap()).unwrap();
    std::fs::write(&history_path, serde_json::to_vec(&history).unwrap()).unwrap();
    let before = inventory(&core, &id);
    let mut expected = raw;
    expected["schemaVersion"] = json!(opencut_editor_core::PROJECT_SCHEMA_VERSION);
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        expected
    );
    let adopted = inventory(&core, &id);
    for (path, bytes) in before {
        if path != project_path && path != history_path {
            assert_eq!(adopted[&path], bytes);
        }
    }
    let revision = core.get_project(&id).unwrap().revision;
    let undo = core.undo(&id, revision).unwrap();
    assert!(core.get_project(&id).unwrap().components.is_empty());
    core.redo(&id, undo.revision).unwrap();
    let restored = core.get_project(&id).unwrap();
    assert_eq!(restored.components[0].id, component);
    assert_eq!(
        serde_json::to_value(&restored.components).unwrap(),
        expected["components"]
    );
    let stable = inventory(&core, &id);
    EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(inventory(&core, &id), stable);
}

#[test]
fn premature_component_fields_and_late_future_history_reject_without_adoption() {
    use opencut_editor_core::ErrorCode;
    for lane in ["current", "undo", "redo"] {
        for (field, value) in [
            ("matte", Value::Null),
            ("matteOnly", json!(false)),
            ("matte", json!({"sourceId":"x","channel":"unknown"})),
        ] {
            let (_root, core, id, _, _) = setup();
            let tracks =
                serde_json::to_value(vec![core.get_project(&id).unwrap().tracks[1].clone()])
                    .unwrap();
            core.edit(&id, 2, op(json!({"operation":"component_create","name":"Local","width":64,"height":64,"durationMs":1000,"tracks":tracks}))).unwrap();
            let dir = core.paths().project_dir(&id).unwrap();
            let mut raw: Value =
                serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
            predecessor33(&mut raw);
            let mut old = raw.clone();
            old["revision"] = json!(2);
            let mut future = raw.clone();
            future["revision"] = json!(77);
            let mut history = json!({"undo":[old],"redo":[future]});
            let source = match lane {
                "current" => &mut raw,
                "undo" => &mut history["undo"][0],
                _ => &mut history["redo"][0],
            };
            source["components"][0]["tracks"][0]["items"][0][field] = value;
            std::fs::write(dir.join("project.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
            std::fs::write(
                dir.join("history.json"),
                serde_json::to_vec(&history).unwrap(),
            )
            .unwrap();
            let before = inventory(&core, &id);
            let error = core.get_project(&id).unwrap_err();
            assert_eq!(
                error.code,
                ErrorCode::InvalidArgument,
                "{lane}/{field}: {error:?}"
            );
            assert!(!error.retryable);
            assert_eq!(inventory(&core, &id), before);
        }
    }
    let (_root, core, id, _, _) = setup();
    let dir = core.paths().project_dir(&id).unwrap();
    let mut raw: Value =
        serde_json::from_slice(&std::fs::read(dir.join("project.json")).unwrap()).unwrap();
    predecessor33(&mut raw);
    let mut future = raw.clone();
    future["schemaVersion"] = json!(opencut_editor_core::PROJECT_SCHEMA_VERSION + 1);
    future["revision"] = json!(77);
    std::fs::write(dir.join("project.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&json!({"undo":[raw],"redo":[future]})).unwrap(),
    )
    .unwrap();
    let before = inventory(&core, &id);
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::InternalError
    );
    assert_eq!(inventory(&core, &id), before);
}

#[test]
fn stale_matte_drafts_validate_own_available_base_and_never_replay_unavailable_base() {
    use opencut_editor_core::ErrorCode;
    for available in [true, false] {
        let (_root, core, id, provider, recipient) = setup();
        let draft = core.create_draft(&id, 2, vec![op(json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":provider,"channel":"alpha"}}))], None).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let draft_path = dir.join("drafts").join(format!("{}.json", draft.id));
        let revision = if available {
            core.edit(
                &id,
                2,
                op(json!({"operation":"delete_item","itemId":recipient})),
            )
            .unwrap()
            .revision
        } else {
            let mut raw: Value =
                serde_json::from_slice(&std::fs::read(&draft_path).unwrap()).unwrap();
            raw["baseRevision"] = json!(999);
            std::fs::write(&draft_path, serde_json::to_vec(&raw).unwrap()).unwrap();
            2
        };
        let before = inventory(&core, &id);
        let draft_bytes = std::fs::read(&draft_path).unwrap();
        core.get_project(&id).unwrap();
        core.get_draft(&id, &draft.id).unwrap();
        assert_eq!(inventory(&core, &id), before);
        for error in [
            core.get_draft_state(&id, &draft.id).unwrap_err(),
            core.commit_draft(&id, &draft.id, revision).unwrap_err(),
        ] {
            assert_eq!(
                error.code,
                ErrorCode::RevisionConflict,
                "available={available}: {error:?}"
            );
        }
        assert_eq!(std::fs::read(&draft_path).unwrap(), draft_bytes);
        assert_eq!(inventory(&core, &id), before);
        // Current editing remains available without applying the retained draft.
        core.edit(
            &id,
            revision,
            op(json!({"operation":"update_item","itemId":provider,"color":"#00ff00"})),
        )
        .unwrap();
        let current = core.get_project(&id).unwrap();
        assert!(
            current
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .all(|item| item.visual_properties().matte.is_none())
        );
        assert_eq!(std::fs::read(&draft_path).unwrap(), draft_bytes);
        core.discard_draft(&id, &draft.id).unwrap();
        assert!(!draft_path.exists());
    }
}
