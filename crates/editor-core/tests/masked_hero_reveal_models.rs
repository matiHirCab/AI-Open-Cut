#[path = "support/masked_hero_reveal.rs"]
mod hero;
use hero::{Fixture, catalog, inventory, op, reference};
use opencut_editor_core::{EditorCore, ErrorCode, PathPolicy};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[test]
fn frozen_complete_plates_audio_and_exact_standalone_alias_recipe() {
    let c = catalog();
    assert_eq!(c["projectSchemaVersion"], 38);
    assert_eq!(c["headlessProtocolVersion"], 1);
    assert_eq!(c["registeredToolCount"], 78);
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(
                include_str!("../../../contracts/masked-hero-reveal-v1.json")
                    .replacen(
                        "\"projectSchemaVersion\": 38",
                        "\"projectSchemaVersion\": 37",
                        1
                    )
                    .as_bytes()
            )
        ),
        "a5756dd3ebf95255c2fe72e7711c7e713f0b5683f4f8c2eb2cfe41fb60079541"
    );
    for (name, hash) in [
        (
            "reference-audio-provenance.json",
            "4c820c658261701b24bab9c6b68bcbd3143fe69e9428eb04b5d0c5be5d88d4de",
        ),
        (
            "reference-conversion-provenance.json",
            "75084162b91f2ae1538ad3c62bef5f28ece0412414fb456912ef85cf4f054dff",
        ),
        (
            "reference-film-provenance.json",
            "d1bc6781b6dbcdc84324f28a34f827b88d3987ef1c2d660189fdeed83de7daa2",
        ),
    ] {
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(std::fs::read(reference(name)).unwrap())
            ),
            hash
        );
    }
    for record in c["plates"]["records"].as_array().unwrap() {
        let bytes = std::fs::read(reference(record["path"].as_str().unwrap())).unwrap();
        assert_eq!(bytes.len(), 64 * 64 * 4);
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), record["sha256"]);
    }
    let wav = std::fs::read(reference("source.wav")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&wav)),
        c["audio"]["sourceSha256"]
    );
    assert_eq!(wav.len(), 44 + 38400 * 4);
    for batch in [false, true] {
        let f = Fixture::new(batch);
        assert_eq!(f.ids.len(), 5);
        f.assert_recipe(false);
        assert_eq!(f.rev(), if batch { 2 } else { 15 });
    }
}
#[test]
fn reordered_draft_commit_undo_redo_and_fresh_reopen_preserve_complete_generations() {
    for batch in [false, true] {
        let f = Fixture::new(batch);
        let before = f.core.get_project(&f.id).unwrap();
        let files = inventory(&f.dir());
        let draft = f
            .core
            .create_draft(
                &f.id,
                f.rev(),
                vec![f.reorder()],
                Some("Reverse hero effects".into()),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(f.core.get_project(&f.id).unwrap()).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        for (path, bytes) in files {
            if path.is_dir() {
                assert_eq!(bytes, b"directory");
            } else {
                assert_eq!(std::fs::read(path).unwrap(), bytes);
            }
        }
        let candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
        let mut expected = serde_json::to_value(&before).unwrap();
        let actual = serde_json::to_value(&candidate).unwrap();
        let index = before.tracks[1]
            .items
            .iter()
            .position(|i| i.id() == f.ids["hero"])
            .unwrap();
        expected["tracks"][1]["items"][index]["effects"] =
            actual["tracks"][1]["items"][index]["effects"].clone();
        assert_eq!(actual, expected);
        f.core.commit_draft(&f.id, &draft.id, f.rev()).unwrap();
        f.assert_recipe(true);
        f.core.undo(&f.id, f.rev()).unwrap();
        f.assert_recipe(false);
        f.core.redo(&f.id, f.rev()).unwrap();
        f.assert_recipe(true);
        let media = f.root.path().join("media");
        let fresh = EditorCore::new(
            PathPolicy::new(
                f.root.path().join("projects"),
                [&media],
                f.root.path().join("exports"),
            )
            .unwrap(),
        );
        let stable = inventory(&f.dir());
        assert_eq!(
            serde_json::to_value(fresh.get_project(&f.id).unwrap()).unwrap(),
            serde_json::to_value(f.core.get_project(&f.id).unwrap()).unwrap()
        );
        assert_eq!(inventory(&f.dir()), stable);
    }
}
fn rejected(f: &Fixture, value: Value, code: ErrorCode) {
    let before = inventory(f.root.path());
    let error = f.core.edit(&f.id, f.rev(), op(value)).unwrap_err();
    assert_eq!(error.code, code, "{error}");
    assert!(!error.retryable);
    assert_eq!(inventory(f.root.path()), before);
}
#[test]
fn canonical_hero_invalid_edits_batches_drafts_conflicts_and_locks_preserve_every_byte() {
    let f = Fixture::new(true);
    let h = &f.ids["hero"];
    let g = &f.ids["owner"];
    f.core
        .edit(
            &f.id,
            f.rev(),
            op(json!({"operation":"item_set_z_index","itemId":g,"zIndex":2})),
        )
        .unwrap();
    f.core.undo(&f.id, f.rev()).unwrap();
    f.assert_recipe(false);
    f.core
        .create_draft(&f.id, f.rev(), vec![f.reorder()], None)
        .unwrap();
    let stable = inventory(f.root.path());
    assert!(
        serde_json::from_value::<opencut_editor_core::EditOperation>(
            json!({"operation":"update_item","itemId":g,"clip":{"type":"unknown"}})
        )
        .is_err()
    );
    assert_eq!(inventory(f.root.path()), stable);
    let mut nonfinite = op(
        json!({"operation":"update_item","itemId":h,"transform2d":catalog()["roles"]["hero"]["transform2d"]}),
    );
    if let opencut_editor_core::EditOperation::UpdateItem {
        transform2d: Some(Some(transform)),
        ..
    } = &mut nonfinite
    {
        transform.position.x = f64::NAN;
    } else {
        panic!("expected complete transform");
    }
    let error = f.core.edit(&f.id, f.rev(), nonfinite).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(!error.retryable);
    assert_eq!(inventory(f.root.path()), stable);
    for (value, code) in [
        (
            json!({"operation":"update_item","itemId":h,"geometry":{"type":"rectangle","width":0,"height":40}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"item_set_parent","itemId":g,"parent":{"scope":"root","id":h}}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":h,"matte":{"sourceId":"missing-provider","channel":"alpha"}}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"update_item","itemId":h,"effects":[catalog()["roles"]["hero"]["effects"][0].clone(),catalog()["roles"]["hero"]["effects"][0].clone()]}),
            ErrorCode::InvalidArgument,
        ),
        (
            json!({"operation":"update_item","itemId":h,"masks":[]}),
            ErrorCode::ItemNotFound,
        ),
    ] {
        rejected(&f, value, code);
    }
    rejected(
        &f,
        json!({"operation":"update_item","itemId":f.ids["provider"],"matte":{"sourceId":h,"channel":"alpha"}}),
        ErrorCode::InvalidArgument,
    );
    let before = inventory(f.root.path());
    let bad = op(
        json!({"operation":"update_item","itemId":h,"geometry":{"type":"rectangle","width":0,"height":40}}),
    );
    let e=f.core.edit_batch::<opencut_editor_core::BatchEditOperation>(&f.id,f.rev(),serde_json::from_value(json!([{"operation":"item_set_z_index","itemId":g,"zIndex":9},{"operation":"update_item","itemId":h,"geometry":{"type":"rectangle","width":0,"height":40}}])).unwrap()).unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArgument);
    assert!(!e.retryable);
    assert_eq!(inventory(f.root.path()), before);
    let e = f
        .core
        .create_draft(&f.id, f.rev(), vec![f.reorder(), bad.clone()], None)
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArgument);
    assert!(!e.retryable);
    assert_eq!(inventory(f.root.path()), before);
    let draft = f
        .core
        .create_draft(&f.id, f.rev(), vec![f.reorder()], None)
        .unwrap();
    let before = inventory(f.root.path());
    let e = f
        .core
        .update_draft(&f.id, &draft.id, f.rev(), vec![bad], None)
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidArgument);
    assert!(!e.retryable);
    assert_eq!(inventory(f.root.path()), before);
    let e = f.core.edit(&f.id, f.rev() - 1, f.reorder()).unwrap_err();
    assert_eq!(e.code, ErrorCode::RevisionConflict);
    assert!(e.retryable);
    assert_eq!(inventory(f.root.path()), before);
    let track = f.core.get_project(&f.id).unwrap().tracks[1].id.clone();
    f.core
        .edit(
            &f.id,
            f.rev(),
            op(json!({"operation":"update_track","trackId":track,"locked":true})),
        )
        .unwrap();
    rejected(
        &f,
        json!({"operation":"item_set_z_index","itemId":h,"zIndex":2}),
        ErrorCode::TrackLocked,
    );
}
#[test]
fn independently_derived_collective_work_bound_rejects_before_standalone_batch_or_draft_persistence()
 {
    let c = catalog();
    let budget = &c["negativeBudget"];
    let source_width = 48 * 32 + 2;
    let source_height = 40 * 32 + 2;
    let expanded_width = source_width + 2 * 16 * 32;
    let expanded_height = source_height + 2 * 16 * 32;
    let scan = 2 * 16 * 32 + 3;
    assert_eq!(budget["sourceSize"], json!([source_width, source_height]));
    assert_eq!(
        budget["expandedSize"],
        json!([expanded_width, expanded_height])
    );
    assert_eq!(expanded_width * expanded_height, 5907972);
    assert_eq!(scan, 1027);
    let work = 256_u64 * 20 * scan * scan;
    assert_eq!(work, 5400212480);
    assert!(work > 268435456);
    for mode in ["standalone", "batch", "draft"] {
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
                "Collective work admission",
                serde_json::from_value(c["settings"].clone()).unwrap(),
            )
            .unwrap()
            .project_id;
        let track = core.get_project(&id).unwrap().tracks[1].id.clone();
        let mut transform = budget["transform2d"].clone();
        transform["scaleX"] = json!(1);
        transform["scaleY"] = json!(1);
        let add = json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":800,"geometry":budget["geometry"],"fill":c["roles"]["hero"]["fill"],"stroke":null,"transform2d":transform});
        let update = |item: &str| {
            op(
                json!({"operation":"update_item","itemId":item,"transform2d":budget["transform2d"],"effects":budget["effects"],"masks":[],"matte":null}),
            )
        };
        let (before, error) = if mode == "batch" {
            let before = inventory(root.path());
            let mut add = add;
            add["resultAlias"] = json!("budget");
            let operations =
                serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
                    add,
                    serde_json::to_value(update("@budget")).unwrap()
                ]))
                .unwrap();
            (before, core.edit_batch(&id, 0, operations).unwrap_err())
        } else {
            let item = core.edit(&id, 0, op(add)).unwrap().changed_ids[0].clone();
            if mode == "draft" {
                let draft = core
                    .create_draft(
                        &id,
                        1,
                        vec![op(
                            json!({"operation":"item_set_z_index","itemId":item,"zIndex":0}),
                        )],
                        None,
                    )
                    .unwrap();
                let before = inventory(root.path());
                (
                    before,
                    core.update_draft(&id, &draft.id, 1, vec![update(&item)], None)
                        .unwrap_err(),
                )
            } else {
                let before = inventory(root.path());
                (before, core.edit(&id, 1, update(&item)).unwrap_err())
            }
        };
        assert_eq!(error.code, ErrorCode::InvalidArgument, "{mode}: {error}");
        assert!(!error.retryable);
        assert!(
            error.message.contains("particle work exceeds limits"),
            "{mode}: {error}"
        );
        assert_eq!(
            inventory(root.path()),
            before,
            "{mode} complete current/history/redo/drafts/resources"
        );
    }
}
