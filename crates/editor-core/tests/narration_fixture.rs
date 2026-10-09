#[path = "support/narration_fixture.rs"]
mod fixture;
use fixture::{inventory, op, recipe, seed};
use opencut_editor_core::{BatchEditOperation, EditorCore, ErrorCode, TimelineItem};
use serde_json::json;

fn assert_authored(f: &fixture::Fixture) {
    let project = f.project();
    let c = recipe();
    assert_eq!(project.duration_ms(), 6000);
    assert_eq!(project.schema_version, 44);
    assert_eq!(project.name, "Narration-driven fixture v1");
    assert_eq!(project.markers.len(), 6);
    for (index, cue) in c["cues"].as_array().unwrap().iter().enumerate() {
        assert_eq!(project.markers[index].name, cue["name"]);
        assert_eq!(project.markers[index].time_ms, cue["timeMs"]);
        for name in [format!("visual{index}"), format!("event{index}")] {
            let item = project
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .find(|item| item.id() == f.aliases[&name])
                .unwrap();
            assert_eq!(item.start_ms(), cue["timeMs"].as_u64().unwrap());
            assert_eq!(
                serde_json::to_value(&item.visual_properties().start_time).unwrap(),
                json!({"type":"marker","markerName":cue["name"],"offsetMs":0})
            );
            if let TimelineItem::Media(media) = item {
                let event = media.audio_event.as_ref().unwrap();
                assert_eq!((event.variant_index, event.variant_seed), (1, 1));
                assert_eq!((event.default_gain_db, event.gain_db), (-6.0, -3.0));
                assert_eq!(event.event, "narration_accent");
                assert_eq!(event.bus_id, "sfx");
                assert_eq!(media.duration_ms, 300);
                let asset = project
                    .assets
                    .iter()
                    .find(|asset| asset.id == media.asset_id)
                    .unwrap();
                assert_eq!(asset.content_hash.as_ref(), Some(&event.content_hash));
                assert_eq!(asset.file_name, "variant1.wav");
            } else {
                assert_eq!(item.duration_ms(), 400);
                assert_eq!(
                    item.visual_properties().animation_preset_provenance.len(),
                    1
                );
                assert_eq!(item.visual_properties().animation_channels.len(), 1);
            }
        }
    }
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.id == f.asset)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&asset.origin).unwrap()["generation"]["alignment"],
        c["alignment"]
    );
    assert_eq!(
        serde_json::to_value(&project.master_normalization).unwrap(),
        c["normalization"]
    );
    assert_eq!(
        serde_json::to_value(&project.audio_buses[1].ducking).unwrap(),
        c["ducking"]
    );
}

#[test]
fn narration_standalone_batch_recipe_and_exact_history() {
    for batch in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let f = seed(&root.path().join("fixture"), batch);
        assert_authored(&f);
        let saved = serde_json::to_value(f.project()).unwrap();
        let revision = f.project().revision;
        assert_eq!(revision, if batch { 6 } else { 38 });
        f.core.undo(&f.id, revision).unwrap();
        if batch {
            assert!(f.project().markers.is_empty());
        }
        f.core.redo(&f.id, revision + 1).unwrap();
        assert_authored(&f);
        let restored = serde_json::to_value(f.project()).unwrap();
        for key in [
            "markers",
            "tracks",
            "assets",
            "audioBuses",
            "masterNormalization",
        ] {
            assert_eq!(restored[key], saved[key]);
        }
        let before = inventory(&f.dir());
        let reopened = EditorCore::new(f.core.paths().clone());
        assert_eq!(
            serde_json::to_value(reopened.get_project(&f.id).unwrap()).unwrap(),
            restored
        );
        assert!(
            inventory(&f.dir()) == before,
            "reopen rewrote the owned generation"
        );
    }
}

#[test]
fn saved_and_explicit_alignment_generate_exact_equivalent_scoped_cues() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(&root.path().join("fixture"), true);
    let c = recipe();
    let mut edits = vec![];
    for (index, asset) in [&f.asset, &f.plain_asset].into_iter().enumerate() {
        let alias = format!("scope{index}");
        edits.push(json!({"operation":"component_create","name":alias,"width":64,"height":64,"durationMs":6000,"tracks":[],"resultAlias":alias}));
        let mut input = json!({"operation":"speech_markers_generate","scope":format!("component:@{alias}"),"assetId":asset,"startMs":0,"markerPolicy":{"type":"sentence"}});
        if index == 1 {
            input["alignment"] = c["alignment"].clone();
        }
        edits.push(input);
    }
    let revision = f.project().revision;
    f.core
        .edit_batch(
            &f.id,
            revision,
            serde_json::from_value::<Vec<BatchEditOperation>>(json!(edits)).unwrap(),
        )
        .unwrap();
    let p = f.project();
    for component in &p.components {
        let cues: Vec<_> = component
            .markers
            .iter()
            .map(|marker| json!({"name":marker.name,"timeMs":marker.time_ms}))
            .collect();
        let expected: Vec<_> = c["cues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|cue| json!({"name":cue["name"],"timeMs":cue["timeMs"]}))
            .collect();
        assert_eq!(cues, expected);
    }
    assert_eq!(p.components.len(), 2);
    assert!(
        p.assets
            .iter()
            .find(|asset| asset.id == f.plain_asset)
            .unwrap()
            .origin
            .is_none()
    );
    let scoped = p.components[1].id.clone();
    let result=f.core.edit_batch(&f.id,revision+1,serde_json::from_value::<Vec<BatchEditOperation>>(json!([
        {"operation":"speech_markers_generate","scope":format!("component:{scoped}"),"assetId":f.plain_asset,"startMs":0,"alignment":c["alignment"],"markerPolicy":{"type":"selected_word","indices":[5]},"resultAlias":"with"},
        {"operation":"marker_update","scope":format!("component:{scoped}"),"markerId":"@with","name":"with_alias","timeMs":3600,"kind":"cue"}
    ])).unwrap()).unwrap();
    assert!(
        f.project().components[1]
            .markers
            .iter()
            .any(|marker| marker.name == "with_alias" && marker.id == result.aliases["with"])
    );
}

#[test]
fn integrated_failures_and_cue_moves_preserve_atomic_ownership() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(&root.path().join("fixture"), true);
    let p = f.project();
    let rev = p.revision;
    let cue = &p.markers[0];
    let c = recipe();
    let move_cue = json!({"operation":"marker_update","scope":"root","markerId":cue.id,"name":cue.name,"timeMs":600,"kind":"cue"});
    let before = inventory(root.path());
    assert_eq!(
        f.core
            .edit(&f.id, rev - 1, op(move_cue.clone()))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    for (input, code) in [
        (
            json!({"operation":"speech_markers_generate","assetId":f.plain_asset,"scope":"root","startMs":0,"alignment":c["alignment"],"markerPolicy":{"type":"selected_word","indices":[99]}}),
            ErrorCode::ValidationFailed,
        ),
        (
            json!({"operation":"set_item_start_time","scope":"root","itemId":f.aliases["visual0"],"time":{"type":"marker","markerName":"missing","offsetMs":0}}),
            ErrorCode::ItemNotFound,
        ),
        (
            json!({"operation":"speech_markers_generate","assetId":"missing","scope":"root","startMs":0,"markerPolicy":{"type":"sentence"}}),
            ErrorCode::AssetNotFound,
        ),
    ] {
        assert_eq!(f.core.edit(&f.id, rev, op(input)).unwrap_err().code, code);
        assert!(
            inventory(root.path()) == before,
            "rejected mutation changed owned bytes"
        );
    }
    let invalid: Vec<BatchEditOperation> = serde_json::from_value(
        json!([move_cue.clone(),{"operation":"marker_delete","scope":"root","markerId":"missing"}]),
    )
    .unwrap();
    assert_eq!(
        f.core.edit_batch(&f.id, rev, invalid).unwrap_err().code,
        ErrorCode::ItemNotFound
    );
    assert!(
        inventory(root.path()) == before,
        "rejected mutation changed owned bytes"
    );
    let forward:Vec<BatchEditOperation>=serde_json::from_value(json!([{"operation":"set_item_start_time","scope":"root","itemId":"@later","time":{"type":"milliseconds","valueMs":0}}])).unwrap();
    assert_eq!(
        f.core.edit_batch(&f.id, rev, forward).unwrap_err().code,
        ErrorCode::ValidationFailed
    );
    assert!(
        inventory(root.path()) == before,
        "rejected mutation changed owned bytes"
    );
    f.core.edit(&f.id, rev, op(move_cue)).unwrap();
    for name in ["visual0", "event0"] {
        assert_eq!(
            f.project()
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .find(|item| item.id() == f.aliases[name])
                .unwrap()
                .start_ms(),
            600
        );
    }
    f.core.undo(&f.id, rev + 1).unwrap();
    assert_authored(&f);
    f.core.redo(&f.id, rev + 2).unwrap();
    let reopened = EditorCore::new(f.core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&f.id).unwrap()).unwrap(),
        serde_json::to_value(f.project()).unwrap()
    );
    // A negative retained offset is valid now, but moving its cue earlier must
    // reject the resulting negative item interval without publishing any bytes.
    f.core
        .edit(
            &f.id,
            rev + 3,
            op(json!({"operation":"set_item_start_time",
                "scope":"root", "itemId":f.aliases["visual0"],
                "time":{"type":"marker", "markerName":"EVERY", "offsetMs":-400}
            })),
        )
        .unwrap();
    let before_invalid_move = inventory(root.path());
    assert_eq!(
        f.core
            .edit(
                &f.id,
                rev + 4,
                op(json!({"operation":"marker_update",
                    "scope":"root", "markerId":cue.id, "name":"EVERY", "timeMs":100, "kind":"cue"
                }))
            )
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(inventory(root.path()), before_invalid_move);
}
