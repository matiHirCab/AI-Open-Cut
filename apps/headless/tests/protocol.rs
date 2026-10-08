use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

fn headless_contract() -> Value {
    serde_json::from_str(include_str!("../../../contracts/headless-protocol-v1.json")).unwrap()
}

#[test]
fn audio_bus_typed_routes_aliases_history_and_fresh_process_reopen_match_catalog() {
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/audio-buses-v1.json")).unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Bus protocol"})))["projectId"].clone();
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(read()["project"]["audioBuses"], catalog["defaultBuses"]);
    assert_eq!(
        read()["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(catalog["projectSchemaVersion"], 39);
    let created = result(&h.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
            {"operation":"create_track","name":"Routed","trackType":"audio","resultAlias":"audio"},
            {"operation":"audio_track_route","scope":"root","trackId":"@audio","busId":"music"},
            {"operation":"audio_bus_set_route","busId":"music","outputBusId":"sfx"}
        ]}),
    ));
    let track = &created["aliases"]["audio"];
    let before = read();
    assert_eq!(before["project"]["audioBuses"][1]["outputBusId"], "sfx");
    assert_eq!(
        before["project"]["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|value| &value["id"] == track)
            .unwrap()["audioBusId"],
        "music"
    );
    for (revision, edit, code, retryable) in [
        (
            0,
            json!({"operation":"audio_track_route","scope":"root","trackId":track,"busId":null}),
            "REVISION_CONFLICT",
            true,
        ),
        (
            1,
            json!({"operation":"audio_bus_set_route","busId":"sfx","outputBusId":"music"}),
            "INVALID_ARGUMENT",
            false,
        ),
        (
            1,
            json!({"operation":"audio_track_route","scope":"root","trackId":"absent","busId":"music"}),
            "TRACK_NOT_FOUND",
            false,
        ),
    ] {
        let error = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(error["error"]["code"], code);
        assert_eq!(error["error"]["retryable"], retryable);
        assert_eq!(read(), before);
    }
    let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":1,"operations":[{"operation":"audio_track_route","scope":"root","trackId":track,"busId":null}]})));
    assert_eq!(read(), before);
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"expectedRevision":1,"draftId":draft["id"]})));
    assert!(
        read()["project"]["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|value| &value["id"] == track)
            .unwrap()
            .get("audioBusId")
            .is_none()
    );
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    assert_eq!(read()["project"]["tracks"], before["project"]["tracks"]);
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})));
    let reopened = read();
    assert_eq!(reopened["project"]["revision"], 4);
    assert_eq!(read(), reopened);
}

#[test]
fn ordered_effect_aliases_arrays_drafts_failures_and_fresh_process_reopen_preserve_contract() {
    let h = Harness::new();
    let f: Value = serde_json::from_str::<Value>(include_str!(
        "../../../contracts/extended-visual-animation-v1.json"
    ))
    .unwrap()["orderedEffectCases"]
        .clone();
    let id=result(&h.request(json!({"operation":"create_project","name":"Ordered effects"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let a = &f["orders"]["shadeThenWash"];
    let b = &f["orders"]["washThenShade"];
    let created=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_shape","resultAlias":"leaf","trackId":track,"startMs":0,"durationMs":800,"geometry":f["source"]["geometry"],"fill":f["source"]["fill"],"stroke":null,"transform2d":f["source"]["transform2d"]},
        {"operation":"update_item","itemId":"@leaf","effects":a},{"operation":"update_item","itemId":"@leaf","effects":b}
    ]})));
    let item = &created["aliases"]["leaf"];
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    let stack = |state: Value| {
        state["project"]["tracks"][1]["items"][0]
            .get("effects")
            .cloned()
            .unwrap_or(json!([]))
    };
    let expected = |raw: &Value| {
        serde_json::to_value(
            serde_json::from_value::<Vec<opencut_editor_core::VisualEffect>>(raw.clone()).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"effects":a}})));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{"operation":"update_item","itemId":item,"zIndex":2}})));
    assert_eq!(stack(read()), expected(a));
    let before = read();
    let revision = before["project"]["revision"].as_u64().unwrap();
    for case in f["invalidStacks"].as_array().unwrap() {
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":item,"effects":case["value"]}}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
            json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
        ] {
            let error = event(&h.request(request));
            assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
            assert_eq!(error["error"]["retryable"], false);
            assert_eq!(read()["project"], before["project"]);
        }
    }
    let draft=result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b}]})));
    assert_eq!(read()["project"], before["project"]);
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"draftId":draft["id"],"expectedRevision":revision})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision+1})));
    assert_eq!(stack(read()), expected(a));
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":revision+2})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision+3,"edit":{"operation":"update_item","itemId":item,"effects":[]}})));
    assert_eq!(stack(read()), json!([]));
}

#[test]
fn parameterized_effect_aliases_arrays_drafts_failures_and_fresh_process_reopen_preserve_contract()
{
    let h = Harness::new();
    let native: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    let f = json!({"source":native["nativeWitness"]["source"],"orders":{"shadeThenWash":native["nativeWitness"]["orders"]["gradeThenTint"],"washThenShade":native["nativeWitness"]["orders"]["tintThenGrade"]},"invalidStacks":native["invalidStacks"]});
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Parameterized effects"})),
    )["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let a = &f["orders"]["shadeThenWash"];
    let b = &f["orders"]["washThenShade"];
    let created=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_shape","resultAlias":"leaf","trackId":track,"startMs":0,"durationMs":800,"geometry":f["source"]["geometry"],"fill":f["source"]["fill"],"stroke":null,"transform2d":f["source"]["transform2d"]},
        {"operation":"update_item","itemId":"@leaf","effects":a},{"operation":"update_item","itemId":"@leaf","effects":b}
    ]})));
    let item = &created["aliases"]["leaf"];
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    let stack = |state: Value| {
        state["project"]["tracks"][1]["items"][0]
            .get("effects")
            .cloned()
            .unwrap_or(json!([]))
    };
    let expected = |raw: &Value| {
        serde_json::to_value(
            serde_json::from_value::<Vec<opencut_editor_core::VisualEffect>>(raw.clone()).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"effects":a}})));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{"operation":"update_item","itemId":item,"zIndex":2}})));
    assert_eq!(stack(read()), expected(a));
    let before = read();
    let revision = before["project"]["revision"].as_u64().unwrap();
    for case in f["invalidStacks"].as_array().unwrap() {
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":item,"effects":case["value"]}}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
            json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
        ] {
            let error = event(&h.request(request));
            assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
            assert_eq!(error["error"]["retryable"], false);
            assert_eq!(read()["project"], before["project"]);
        }
    }
    let draft=result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b}]})));
    assert_eq!(read()["project"], before["project"]);
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"draftId":draft["id"],"expectedRevision":revision})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision+1})));
    assert_eq!(stack(read()), expected(a));
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":revision+2})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision+3,"edit":{"operation":"update_item","itemId":item,"effects":[]}})));
    assert_eq!(stack(read()), json!([]));
}

#[test]
fn group_compositing_aliases_arrays_drafts_failures_and_fresh_process_reopen_preserve_contract() {
    fn inventory(
        root: &std::path::Path,
    ) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        let mut files = std::collections::BTreeMap::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.file_name().unwrap() != ".lock" {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        files
    }
    let h = Harness::new();
    let native: Value =
        serde_json::from_str(include_str!("../../../contracts/group-compositing-v1.json")).unwrap();
    let invalid = native["effectCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["accepted"] == false)
        .map(|case| json!({"value":[case["value"]]}))
        .collect::<Vec<_>>();
    let f = json!({"orders":{"shadeThenWash":[native["nativeWitness"]["flash"],native["nativeWitness"]["particles"]],"washThenShade":[native["nativeWitness"]["particles"],native["nativeWitness"]["flash"]]},"invalidStacks":invalid});
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Parameterized effects"})),
    )["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let a = &f["orders"]["shadeThenWash"];
    let b = &f["orders"]["washThenShade"];
    let created=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_group","resultAlias":"leaf","trackId":track,"startMs":0,"durationMs":800},
        {"operation":"update_item","itemId":"@leaf","clip":{"type":"composition_bounds"},"effects":a},{"operation":"update_item","itemId":"@leaf","effects":b}
    ]})));
    let item = &created["aliases"]["leaf"];
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    let stack = |state: Value| {
        state["project"]["tracks"][1]["items"][0]
            .get("effects")
            .cloned()
            .unwrap_or(json!([]))
    };
    let expected = |raw: &Value| {
        serde_json::to_value(
            serde_json::from_value::<Vec<opencut_editor_core::VisualEffect>>(raw.clone()).unwrap(),
        )
        .unwrap()
    };
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"effects":a}})));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{"operation":"update_item","itemId":item,"zIndex":2}})));
    assert_eq!(stack(read()), expected(a));
    let before = read();
    let revision = before["project"]["revision"].as_u64().unwrap();
    for case in f["invalidStacks"].as_array().unwrap() {
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":item,"effects":case["value"]}}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
            json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"effects":case["value"]}]}),
        ] {
            let error = event(&h.request(request));
            assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
            assert_eq!(error["error"]["retryable"], false);
            assert_eq!(read()["project"], before["project"]);
        }
    }
    let unchanged = inventory(h.root.path());
    for case in native["clipCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["accepted"] == false && !case["value"].is_null())
    {
        let clip = &case["value"];
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":item,"clip":clip}}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"clip":clip}]}),
            json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b},{"operation":"update_item","itemId":item,"clip":clip}]}),
        ] {
            let error = event(&h.request(request));
            assert_eq!(
                error["error"]["code"], "INVALID_ARGUMENT",
                "{clip}: {error}"
            );
            assert_eq!(error["error"]["retryable"], false);
            assert_eq!(read()["project"], before["project"]);
            assert_eq!(inventory(h.root.path()), unchanged, "{clip}");
        }
    }
    let draft=result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"effects":b}]})));
    assert_eq!(read()["project"], before["project"]);
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"draftId":draft["id"],"expectedRevision":revision})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision+1})));
    assert_eq!(stack(read()), expected(a));
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":revision+2})));
    assert_eq!(stack(read()), expected(b));
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision+3,"edit":{"operation":"update_item","itemId":item,"effects":[]}})));
    assert_eq!(stack(read()), json!([]));
}

#[test]
fn review_range_transport_rejects_before_io_and_preserves_revision_reopen() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Review transport"})))["projectId"].clone();
    let initial = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = initial["project"]["tracks"][1]["id"].clone();
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#112233","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})));
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    for (extra, expected) in [
        (json!({"expectedRevision":0}), "REVISION_CONFLICT"),
        (
            json!({"resolution":{"width":7681,"height":720}}),
            "VALIDATION_FAILED",
        ),
        (json!({"resolution":{"width":320}}), "INVALID_ARGUMENT"),
        (
            json!({"resolution":{"width":320,"height":180,"preset":"540p"}}),
            "INVALID_ARGUMENT",
        ),
        (json!({"resolution":"1080p"}), "INVALID_ARGUMENT"),
        (json!({"fps":0}), "VALIDATION_FAILED"),
    ] {
        let mut request = json!({"operation":"render_review_range","projectId":id,"expectedRevision":1,"startMs":0,"endMs":1000});
        for (key, value) in extra.as_object().unwrap() {
            request[key] = value.clone();
        }
        let response = event(&h.request(request));
        assert_eq!(response["error"]["code"], expected, "{response}");
        assert_eq!(
            result(&h.request(json!({"operation":"open_project","projectId":id})))["project"],
            before["project"]
        );
    }
    let previews = h
        .root
        .path()
        .join("projects")
        .join(id.as_str().unwrap())
        .join("previews");
    assert_eq!(std::fs::read_dir(previews).unwrap().count(), 0);
    let status = result(&h.request(json!({"operation":"status"})));
    let ready = status["subsystems"]["rendering"]["ready"]
        .as_bool()
        .unwrap();
    for capabilities in [
        &status["capabilities"],
        &status["subsystems"]["rendering"]["capabilities"],
    ] {
        assert_eq!(
            capabilities
                .as_array()
                .unwrap()
                .contains(&json!("preview_review_presets_v1")),
            ready
        );
    }
}

fn error_catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/error-codes-v1.json")).unwrap()
}

#[test]
fn scoped_markers_roundtrip_batch_alias_and_conflict() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Marker protocol"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let written = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"marker_create","scope":"root","name":"impact","timeMs":300,"kind":"cue","resultAlias":"cue"},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":100,"width":20,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_item_start_time","scope":"root","itemId":"@box","time":{"type":"marker","markerName":"impact","offsetMs":-50}}
    ]})));
    assert_eq!(written["revision"], 1);
    let item_id = written["aliases"]["box"].clone();
    let marker_id = written["aliases"]["cue"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(state["project"]["markers"][0]["id"], marker_id);
    assert_eq!(state["project"]["tracks"][1]["items"][0]["startMs"], 250);
    let conflict = event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"marker_delete","scope":"root","markerId":marker_id}})));
    assert_eq!(conflict["error"]["code"], "REVISION_CONFLICT");
    let changed = result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"marker_update","scope":"root","markerId":marker_id,"name":"impact","timeMs":500,"kind":"cue"}})));
    assert_eq!(changed["revision"], 2);
    let reopened = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(reopened["project"]["tracks"][1]["items"][0]["startMs"], 450);
    assert_eq!(reopened["project"]["tracks"][1]["items"][0]["id"], item_id);
}

#[test]
fn typed_animation_channels_roundtrip_alias_and_failures() {
    let h = Harness::new();
    let id =
        result(&h.request(json!({"operation":"create_project","name":"Animated"})))["projectId"]
            .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let channels = json!([{"property":"transform.position_x","keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1.0}},
        {"timeMs":500,"value":{"type":"scalar","value":40.0},"curve":"hold"}
    ]}]);
    let written = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_animation_channels","itemId":"@box","animationChannels":channels}
    ]})));
    assert_eq!(written["revision"], 1);
    let item = written["aliases"]["box"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["animationChannels"],
        channels
    );
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    for (revision, edit, code) in [
        (
            0,
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[]}),
            "REVISION_CONFLICT",
        ),
        (
            1,
            json!({"operation":"set_animation_channels","itemId":"missing","animationChannels":[]}),
            "ITEM_NOT_FOUND",
        ),
        (
            1,
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.skew_x_deg","keyframes":[]}]}),
            "INVALID_ARGUMENT",
        ),
        (
            1,
            json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[{"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":{"type":"spring","mass":0,"stiffness":100,"damping":20,"initialVelocity":0}},{"timeMs":500,"value":{"type":"scalar","value":40},"curve":"hold"}]}]}),
            "INVALID_ARGUMENT",
        ),
    ] {
        let response = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(response["error"]["code"], code);
    }
    let spring = json!([{"property":"transform.position_x","keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":{"type":"spring","mass":1.0,"stiffness":100.0,"damping":20.0,"initialVelocity":0.0}},
        {"timeMs":500,"value":{"type":"scalar","value":40.0},"curve":"hold"}
    ]}]);
    let replaced = result(&h.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{
            "operation":"set_animation_channels","itemId":item,"animationChannels":spring
        }}),
    ));
    assert_eq!(replaced["revision"], 2);
    let reopened = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(reopened["project"]["revision"], 2);
    assert_eq!(
        reopened["project"]["tracks"][1]["items"][0]["animationChannels"],
        spring
    );
}

#[test]
fn typed_animation_loops_work_standalone_and_in_aliased_batch() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Loops"})))["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let looped = json!([{"property":"transform.position_x","keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},
        {"timeMs":250,"value":{"type":"scalar","value":20.0},"curve":"linear"},
        {"timeMs":500,"value":{"type":"scalar","value":0.0},"curve":"hold"}],
        "loop":{"mode":"repeat","iterations":3}}]);
    let written = result(&h.request(json!({"operation":"edit_batch","projectId":id,
        "expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1200,
            "width":10,"height":10,"color":"#ff0000",
            "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"set_animation_channels","itemId":"@box","animationChannels":looped}
    ]})));
    assert_eq!(written["revision"], 1);
    let item = written["aliases"]["box"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["animationChannels"],
        looped
    );
    let bad = json!([{"property":"transform.position_x","keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
        {"timeMs":500,"value":{"type":"scalar","value":20},"curve":"hold"}],
        "loop":{"mode":"repeat","iterations":2}}]);
    let error = event(&h.request(json!({"operation":"edit","projectId":id,
        "expectedRevision":1,"edit":{"operation":"set_animation_channels","itemId":item,
            "animationChannels":bad}})));
    assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
    let replaced = result(&h.request(json!({"operation":"edit","projectId":id,
        "expectedRevision":1,"edit":{"operation":"set_animation_channels","itemId":item,
            "animationChannels":[{"property":"transform.position_x","keyframes":[
                {"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},
                {"timeMs":500,"value":{"type":"scalar","value":20},"curve":"hold"}],
                "loop":{"mode":"ping_pong","iterations":"infinite"}}]}})));
    assert_eq!(replaced["revision"], 2);
    let reopened = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        reopened["project"]["tracks"][1]["items"][0]["animationChannels"][0]["loop"],
        json!({"mode":"ping_pong","iterations":"infinite"})
    );
}

#[test]
fn persisted_layout_errors_keep_exact_headless_codes_and_files() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Persisted layouts"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let added = result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_text","trackId":track,"text":"M","startMs":0,"durationMs":1000,"fontSize":30,"color":"#ffffff","style":{"layout":{}},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})));
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let path = dir.join("project.json");
    let original = std::fs::read(&path).unwrap();
    for layout in [
        json!({"trackingPx":-1}),
        json!(null),
        json!({"bounds":{"heightPx":null}}),
    ] {
        let mut project: Value = serde_json::from_slice(&original).unwrap();
        project["tracks"][1]["items"][0]["style"]["layout"] = layout;
        let bytes = serde_json::to_vec(&project).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let failure = event(&h.request(json!({"operation":"open_project","projectId":id})));
        assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(failure["error"]["retryable"], false);
        assert!(!failure.to_string().contains("OPENCUT_LAYOUT_DECODE"));
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    std::fs::write(&path, &original).unwrap();
    let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":1,"operations":[{"operation":"update_item","itemId":added["changedIds"][0],"style":{"layout":{}}}]})));
    let draft_path = dir
        .join("drafts")
        .join(format!("{}.json", draft["id"].as_str().unwrap()));
    let mut draft: Value = serde_json::from_slice(&std::fs::read(&draft_path).unwrap()).unwrap();
    draft["operations"][0]["style"]["layout"] = Value::Null;
    let bytes = serde_json::to_vec(&draft).unwrap();
    std::fs::write(&draft_path, &bytes).unwrap();
    let failure = event(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(failure["error"]["retryable"], false);
    assert!(!failure.to_string().contains("OPENCUT_LAYOUT_DECODE"));
    assert_eq!(std::fs::read(&draft_path).unwrap(), bytes);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let mut unrelated: Value = serde_json::from_slice(&original).unwrap();
    unrelated["name"] = json!(42);
    std::fs::write(&path, serde_json::to_vec(&unrelated).unwrap()).unwrap();
    assert_eq!(
        event(&h.request(json!({"operation":"open_project","projectId":id})))["error"]["code"],
        "INTERNAL_ERROR"
    );
}

#[test]
fn rich_text_documents_roundtrip_batches_drafts_and_failures() {
    let h = Harness::new();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/rich-text-documents-v1.json"
    ))
    .unwrap();
    let id =
        result(&h.request(json!({"operation":"create_project","name":"Rich text"})))["projectId"]
            .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let document = &catalog["valid"][1]["document"];
    let added = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_text","trackId":track,"resultAlias":"title","document":document,"startMs":0,"durationMs":1000,"fontSize":48,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}},
        {"operation":"update_item","itemId":"@title","color":"#00ff00"}]})));
    let item = &added["changedIds"][0];
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["document"],
        *document
    );
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["text"],
        catalog["valid"][1]["text"]
    );
    let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":1,"operations":[{"operation":"update_item","itemId":item,"text":"plain"}]})));
    let draft_state = result(
        &h.request(json!({"operation":"get_draft_state","projectId":id,"draftId":draft["id"]})),
    );
    assert_eq!(
        draft_state["project"]["tracks"][1]["items"][0]["document"],
        json!({"runs":[{"text":"plain"}]})
    );
    for fixture in catalog["invalid"].as_array().unwrap() {
        let failure = event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"document":fixture["document"]}})));
        assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT", "{failure}");
    }
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        state
    );
}

#[test]
fn effective_repeater_audio_rejection_is_atomic_over_headless() {
    use opencut_editor_core::{
        BatchEditOperation, EditorCore, MediaProbeFacts, MediaType, PathPolicy, ProjectSettings,
    };
    let harness = Harness::new();
    let media = harness.root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            harness.root.path().join("projects"),
            [&media],
            harness.root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Audio closure", ProjectSettings::default())
        .unwrap()
        .project_id;
    let mut assets = Vec::new();
    for (n, audio) in [false, true].into_iter().enumerate() {
        let path = media.join(format!("source{n}.mp4"));
        std::fs::write(&path, format!("source{n}")).unwrap();
        assets.push(
            core.import_asset(
                &id,
                n as u64,
                &path,
                MediaType::Video,
                MediaProbeFacts {
                    duration_ms: Some(1000),
                    has_audio: audio,
                    has_video: true,
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
                .clone(),
        );
    }
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/repeaters-v1.json")).unwrap();
    let mut descriptor = catalog["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!("@instance");
    let added = core.edit_batch::<BatchEditOperation>(&id,2,serde_json::from_value(json!([
        {"operation":"component_create","resultAlias":"leaf","name":"Leaf","width":100,"height":100,"durationMs":1000,
        "slots":[{"id":"asset","name":"Asset","kind":"asset","required":false,"binding":{"targetLayerId":"media","property":"media.asset"},"constraints":{}}],
        "tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"media","id":"media","assetId":assets[0],"startMs":0,"durationMs":1000,
        "sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}]}]},
        {"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"resultAlias":"instance"},
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor}
    ])).unwrap()).unwrap();
    let bad = json!({"operation":"component_instance_update","itemId":added.aliases["instance"],"componentId":added.aliases["leaf"],
        "startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":assets[1]}}}});
    let dir = core.paths().project_dir(&id).unwrap();
    let files = || {
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        )
    };
    let before = files();
    for operation in ["edit_batch", "create_draft"] {
        let response = event(&harness.request(
            json!({"operation":operation,"projectId":id,"expectedRevision":3,"operations":[bad]}),
        ));
        assert_eq!(response["error"]["code"], "INVALID_ARGUMENT", "{response}");
        assert_eq!(files(), before);
    }
    let response = event(&harness.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[bad]}),
    ));
    assert_eq!(response["error"]["code"], "REVISION_CONFLICT");
    let state = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(state["project"]["revision"], 3);
    assert_eq!(files(), before);
}

struct Harness {
    root: TempDir,
}

impl Harness {
    fn project_files(&self, id: &str) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        fn visit(
            root: &std::path::Path,
            path: &std::path::Path,
            out: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
        ) {
            for entry in std::fs::read_dir(path).unwrap() {
                let e = entry.unwrap();
                if e.file_type().unwrap().is_dir() {
                    visit(root, &e.path(), out);
                } else {
                    out.insert(
                        e.path().strip_prefix(root).unwrap().to_owned(),
                        std::fs::read(e.path()).unwrap(),
                    );
                }
            }
        }
        let root = self.root.path().join("projects").join(id);
        let mut files = std::collections::BTreeMap::new();
        visit(&root, &root, &mut files);
        files
    }
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
        }
    }

    fn request(&self, request: Value) -> Output {
        self.request_raw(&serde_json::to_string(&request).unwrap())
    }

    fn request_raw(&self, request: &str) -> Output {
        let projects = self.root.path().join("projects");
        let media = self.root.path().join("media");
        let exports = self.root.path().join("exports");
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&exports).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_opencut-headless"));
        command
            .env("OPENCUT_PROJECTS_DIR", projects)
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
            .env("OPENCUT_EXPORTS_DIR", exports)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in ["OPENCUT_FFMPEG_PATH", "OPENCUT_FFPROBE_PATH"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        // Text defaults are embedded; ambient font configuration must not be
        // required by headless or packaged workflows.
        command.env_remove("OPENCUT_DEFAULT_FONT_PATH");
        let mut child = command.spawn().unwrap();
        std::io::Write::write_all(&mut child.stdin.take().unwrap(), request.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    }

    fn health_with_missing_rendering(&self) -> Output {
        let projects = self.root.path().join("health-projects");
        let media = self.root.path().join("health-media");
        let exports = self.root.path().join("health-exports");
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&exports).unwrap();
        Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
            .arg("--health")
            .env("OPENCUT_PROJECTS_DIR", projects)
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
            .env("OPENCUT_EXPORTS_DIR", exports)
            .env(
                "OPENCUT_FFMPEG_PATH",
                self.root.path().join("missing-ffmpeg"),
            )
            .env(
                "OPENCUT_FFPROBE_PATH",
                self.root.path().join("missing-ffprobe"),
            )
            .output()
            .unwrap()
    }
}

fn native_parity_is_configured() -> bool {
    let configured = [
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
        std::env::var_os("OPENCUT_TEST_FONT_PATH"),
    ];
    if configured.iter().all(Option::is_none) {
        return false;
    }
    assert!(
        configured.iter().all(Option::is_some),
        "native lifecycle parity requires FFmpeg, FFprobe, and font paths together"
    );
    true
}

#[test]
fn native_svg_numeric_rejection_preserves_state() {
    for source in [
        "<svg width=\"100\" height=\"100\" viewBox=\"0 0 0.0001 0.0001\"><rect x=\"-5000\" y=\"-5000\" width=\"10000\" height=\"10000\"/></svg>",
        "<svg width=\"100\" height=\"100\" viewBox=\"0 0 .001 .001\"><polygon points=\"-5000,-5000 5000,5000 5000,5000.0001 -5000,-4999.9999\" fill=\"#f00\"/></svg>",
    ] {
        assert_svg_numeric_rejection(source);
    }
}

fn assert_svg_numeric_rejection(source: &str) {
    if !native_parity_is_configured() {
        return;
    }
    let harness = Harness::new();
    let created = result(&harness.request(json!({"operation":"create_project","name":"SVG numeric rejection","settings":{"width":100,"height":100,"fps":10}})));
    let id = created["projectId"].as_str().unwrap();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["trackType"] == "overlay")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":source}})));
    let before = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let failed = event(&harness.request(
        json!({"operation":"render_preview","projectId":id,"expectedRevision":1,"timeMs":0}),
    ));
    assert_eq!(failed["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(
        result(&harness.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

fn event(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn result(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let envelope = event(output);
    assert_eq!(envelope["type"], "result");
    envelope["result"].clone()
}

#[test]
fn svg_contract_reaches_core_and_preserves_batch_atomicity() {
    let h = Harness::new();
    let id =
        result(&h.request(json!({"operation":"create_project","name":"SVG"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/svg-ingestion-v1.json")).unwrap();
    let mut revision = 0;
    for f in catalog["valid"].as_array().unwrap() {
        let r=result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":f["svg"]}})));
        revision = r["revision"].as_u64().unwrap();
    }
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    for f in catalog["invalid"].as_array().unwrap() {
        let e=event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":f["svg"]}})));
        assert_eq!(e["error"]["code"], "INVALID_ARGUMENT", "{e}");
    }
    let batch = json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":catalog["valid"][0]["svg"],"resultAlias":"icon"},{"operation":"item_set_z_index","itemId":"@icon","zIndex":3},{"operation":"delete_item","itemId":"missing"}]});
    assert_eq!(event(&h.request(batch))["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn component_lifecycle_native_contract_and_atomic_history() {
    let harness = Harness::new();
    let id = result(&harness.request(json!({"operation":"create_project","name":"Lifecycle"})))["projectId"].clone();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["trackType"] == "overlay")
        .unwrap()["id"]
        .clone();
    let created = result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"component_create","name":"Leaf","width":64,"height":64,"durationMs":1000,"tracks":[],"resultAlias":"leaf"},
        {"operation":"component_define_slots","componentId":"@leaf","slots":[]},
        {"operation":"add_component_instance","trackId":track,"componentId":"@leaf","startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"resultAlias":"instance"},
        {"operation":"component_instance_duplicate","itemId":"@instance","offsetMs":100,"resultAlias":"copy"},
        {"operation":"item_set_z_index","itemId":"@copy","zIndex":2}
    ]})));
    let source = created["aliases"]["copy"].clone();
    let catalog: Value = serde_json::from_slice(include_bytes!(
        "../../../contracts/component-lifecycle-v1.json"
    ))
    .unwrap();
    let directory = harness
        .root
        .path()
        .join("projects")
        .join(id.as_str().unwrap());
    let files = || {
        ["project.json", "history.json"].map(|name| std::fs::read(directory.join(name)).unwrap())
    };
    let before = files();
    for edit in catalog["invalidOperations"].as_array().unwrap() {
        let failed =
            event(&harness.request(
                json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":edit}),
            ));
        assert_eq!(
            failed["error"]["code"], "INVALID_ARGUMENT",
            "{edit}: {failed}"
        );
        assert_eq!(files(), before);
    }
    for (revision, edit, code) in [
        (
            0,
            json!({"operation":"component_instance_duplicate","itemId":source,"offsetMs":0}),
            "REVISION_CONFLICT",
        ),
        (
            1,
            json!({"operation":"component_instance_duplicate","itemId":"missing","offsetMs":0}),
            "ITEM_NOT_FOUND",
        ),
    ] {
        let failed = event(&harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(files(), before);
    }
    let failed = event(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"component_instance_duplicate","itemId":source,"offsetMs":0},{"operation":"delete_item","itemId":"missing"}]})));
    assert_eq!(failed["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(files(), before);
    let expected = result(&harness.request(json!({"operation":"get_state","projectId":id})))["project"]["tracks"].clone();
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":1})));
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":2})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        expected
    );
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":3,"edit":{"operation":"component_instance_duplicate","itemId":source,"offsetMs":0,"slotValues":{}}})));
}

#[test]
fn closed_slot_records_fail_native_decoding_without_mutating_history() {
    let harness = Harness::new();
    let project =
        result(&harness.request(json!({"operation":"create_project","name":"Closed slots"})));
    let id = project["projectId"].as_str().unwrap();
    let directory = harness.root.path().join("projects").join(id);
    let files = || {
        ["project.json", "history.json"].map(|name| std::fs::read(directory.join(name)).unwrap())
    };
    let before = files();
    let catalog: Value =
        serde_json::from_slice(include_bytes!("../../../contracts/template-slots-v1.json"))
            .unwrap();
    for fixture in catalog["regressions"]["closedRecords"].as_array().unwrap() {
        let mut edits = vec![
            json!({"operation":"component_define_slots","componentId":"card","slots":[fixture["slot"]]}),
        ];
        if let Some(value) = fixture.get("override") {
            edits.push(json!({"operation":"component_create","name":"Outer","width":320,"height":240,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"component_instance","id":"nested","componentId":"leaf","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{"__proto__":value}}]}]}));
        }
        for edit in edits {
            for batch in [false, true] {
                let request = if batch {
                    json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[{"operation":"component_create","name":"First","width":320,"height":240,"durationMs":1000,"tracks":[],"resultAlias":"card"},edit]})
                } else {
                    json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":edit})
                };
                let failed = event(&harness.request(request));
                assert_eq!(
                    failed["error"]["code"], "INVALID_ARGUMENT",
                    "{}: {failed}",
                    fixture["id"]
                );
                assert!(
                    failed["error"]["message"]
                        .as_str()
                        .unwrap()
                        .contains("invalid headless request"),
                    "{failed}"
                );
                assert_eq!(files(), before, "{}", fixture["id"]);
            }
        }
    }
}

#[test]
fn template_slots_standalone_and_alias_batches_have_typed_atomic_results() {
    let harness = Harness::new();
    let project = result(&harness.request(json!({"operation":"create_project","name":"Slots"})));
    let id = project["projectId"].as_str().unwrap();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/template-slots-v1.json")).unwrap();
    let slot = catalog["valid"][0]["slot"].clone();
    let create = json!({"operation":"component_create","resultAlias":"card","name":"Card","width":320,"height":240,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[{"type":"text","id":"title","text":"Base","document":{"runs":[{"text":"Base"}]},"fontSize":24,"color":"#ffffff","startMs":0,"durationMs":1000,"keyframes":[]}]}]});
    result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[create,{"operation":"component_define_slots","componentId":"@card","slots":[slot.clone()]}]})));
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let component = state["project"]["components"][0]["id"].as_str().unwrap();
    assert_eq!(
        state["project"]["components"][0]["slots"],
        json!([slot.clone()])
    );
    let failure=event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"component_define_slots","componentId":component,"slots":[]}})));
    assert_eq!(failure["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(
        result(&harness.request(json!({"operation":"get_state","projectId":id}))),
        state
    );
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"component_define_slots","componentId":component,"slots":[]}})));
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["components"],
        state["project"]["components"]
    );
}

#[test]
fn component_protocol_aliases_failures_and_exact_history() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Components"})));
    let id = created["projectId"].as_str().unwrap();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/component-definitions-v1.json"
    ))
    .unwrap();
    let mut create = catalog["validOperations"][0].clone();
    create["resultAlias"] = json!("leaf");
    let batch = result(&harness.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[create]}),
    ));
    let component = batch["aliases"]["leaf"].clone();
    let state = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(state["project"]["components"][0]["id"], component);
    let dir = harness.root.path().join("projects").join(id);
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    for (revision, edit, code) in [
        (
            0,
            json!({"operation":"component_delete","componentId":component}),
            "REVISION_CONFLICT",
        ),
        (
            1,
            json!({"operation":"component_delete","componentId":"missing"}),
            "ITEM_NOT_FOUND",
        ),
        (
            1,
            json!({"operation":"component_delete","componentId":component,"resultAlias":null}),
            "INVALID_ARGUMENT",
        ),
    ] {
        let output = harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[catalog["validOperations"][0],edit]}));
        let failed = event(&output);
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(failed["error"]["retryable"], code == "REVISION_CONFLICT");
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before.0);
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), before.1);
    }
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"component_delete","componentId":component}})));
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    let restored = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        restored["project"]["components"],
        state["project"]["components"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})));
    let removed = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(removed["project"]["components"], json!([]));
}

#[test]
fn create_read_and_edit_use_result_envelopes_and_typed_ids() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({
        "operation": "create_project",
        "name": "Protocol project"
    })));
    let project_id = created["projectId"].as_str().unwrap();
    assert_eq!(created["revision"], 0);

    let state = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let overlay_id = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|track| track["trackType"] == "overlay")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let edited = result(&harness.request(json!({
        "operation": "edit",
        "projectId": project_id,
        "expectedRevision": 0,
        "edit": {
            "operation": "add_text",
            "trackId": overlay_id,
            "text": "contract",
            "startMs": 0,
            "durationMs": 1000,
            "fontSize": 48,
            "color": "#ffffff",
            "fontFamily": null,
            "transform": { "positionX": 0.0, "positionY": 0.0, "scale": 1.0, "opacity": 1.0 }
        }
    })));
    assert_eq!(edited["revision"], 1);
    assert_eq!(edited["changedIds"].as_array().unwrap().len(), 1);
}

#[test]
fn group_protocol_aliases_detachment_history_and_atomic_errors() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({"operation":"create_project","name":"Groups"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let created=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"parent":{"scope":"root","id":"@parent"},"resultAlias":"child"}
    ]})));
    let child = &created["aliases"]["child"];
    let parent = &created["aliases"]["parent"];
    let before = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        before["project"]["tracks"][1]["items"][1]["parent"]["id"],
        *parent
    );
    let bad = harness.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
            {"operation":"item_set_parent","itemId":child,"parent":null},
            {"operation":"item_set_parent","itemId":parent,"parent":{"scope":"root","id":"missing"}}
        ]}),
    );
    assert_eq!(event(&bad)["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id}))),
        before
    );
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"item_set_parent","itemId":child,"parent":null}})));
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    let restored = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        restored["project"]["tracks"][1]["items"],
        before["project"]["tracks"][1]["items"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})));
    let detached = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert!(
        detached["project"]["tracks"][1]["items"][1]
            .get("parent")
            .is_none()
    );
}

#[test]
fn revision_conflicts_are_typed_error_envelopes_with_nonzero_exit() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({ "operation": "create_project", "name": "Conflict" })));
    let project_id = created["projectId"].as_str().unwrap();
    let output = harness.request(json!({
        "operation": "undo",
        "projectId": project_id,
        "expectedRevision": 99
    }));
    assert!(!output.status.success());
    assert_eq!(
        event(&output),
        json!({
            "type": "error",
            "error": {
                "code": "REVISION_CONFLICT",
                "failedStage": null,
                "ffmpegExitCode": null,
                "ffmpegStderrExcerpt": null,
                "message": "expected revision 99, current revision is 0",
                "retryable": true
            }
        })
    );

    let render = harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 99,
        "timeMs": 0
    }));
    assert!(!render.status.success());
    assert_eq!(event(&render)["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(
        event(&render)["error"]["message"],
        "expected revision 99, current revision is 0"
    );
}

#[test]
fn native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts() {
    if !native_parity_is_configured() {
        return;
    }
    let harness = Harness::new();
    let created = result(&harness.request(json!({
        "operation": "create_project",
        "name": "Native lifecycle",
        "settings": { "width": 160, "height": 90, "fps": 10 }
    })));
    let project_id = created["projectId"].as_str().unwrap();
    let state = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let overlay_id = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|track| track["trackType"] == "overlay")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let transform = json!({
        "positionX": 0.0,
        "positionY": 0.0,
        "scale": 1.0,
        "opacity": 1.0
    });

    let edited = result(&harness.request(json!({
        "operation": "edit",
        "projectId": project_id,
        "expectedRevision": 0,
        "edit": {
            "operation": "add_solid_color",
            "trackId": overlay_id,
            "color": "#224466",
            "startMs": 0,
            "durationMs": 1000,
            "transform": transform
        }
    })));
    assert_eq!(edited["revision"], 1);
    let solid_id = edited["changedIds"][0].as_str().unwrap();
    let stale_project_dir = harness.root.path().join("projects").join(project_id);
    let stale_project_before = std::fs::read(stale_project_dir.join("project.json")).unwrap();
    let stale_history_before = std::fs::read(stale_project_dir.join("history.json")).unwrap();
    let stale_previews_before = std::fs::read_dir(stale_project_dir.join("previews"))
        .unwrap()
        .count();
    let stale = harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 0,
        "timeMs": 500
    }));
    assert!(!stale.status.success());
    assert_eq!(event(&stale)["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(
        std::fs::read(stale_project_dir.join("project.json")).unwrap(),
        stale_project_before
    );
    assert_eq!(
        std::fs::read(stale_project_dir.join("history.json")).unwrap(),
        stale_history_before
    );
    assert_eq!(
        std::fs::read_dir(stale_project_dir.join("previews"))
            .unwrap()
            .count(),
        stale_previews_before
    );
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 1,
        "timeMs": 500
    })));

    let undone = result(&harness.request(json!({
        "operation": "undo",
        "projectId": project_id,
        "expectedRevision": 1
    })));
    assert_eq!(undone["revision"], 2);
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 2,
        "timeMs": 0
    })));

    let redone = result(&harness.request(json!({
        "operation": "redo",
        "projectId": project_id,
        "expectedRevision": 2
    })));
    assert_eq!(redone["revision"], 3);
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 3,
        "timeMs": 500
    })));

    let reopened = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let reopened_again = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    assert_eq!(reopened, reopened_again);
    assert_eq!(reopened["project"]["revision"], 3);

    let draft = result(&harness.request(json!({
        "operation": "create_draft",
        "projectId": project_id,
        "expectedRevision": 3,
        "label": "isolated render",
        "operations": [{
            "operation": "update_item",
            "itemId": solid_id,
            "color": "#ee8844",
            "transform2d": {
                "position": {"x": 0.5, "y": 0.5, "unit": "normalized"},
                "anchor": {"x": 0.5, "y": 0.5}, "scaleX": 0.8, "scaleY": 0.7,
                "rotationDeg": 15, "skewXDeg": 4, "skewYDeg": -2, "opacity": 0.8
            }
        }]
    })));
    let draft_id = draft["id"].as_str().unwrap();
    let project_dir = harness.root.path().join("projects").join(project_id);
    let project_before = std::fs::read(project_dir.join("project.json")).unwrap();
    let history_before = std::fs::read(project_dir.join("history.json")).unwrap();
    let draft_path = project_dir.join("drafts").join(format!("{draft_id}.json"));
    let draft_before = std::fs::read(&draft_path).unwrap();
    let committed_before = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let draft_record_before = result(&harness.request(json!({
        "operation": "get_draft",
        "projectId": project_id,
        "draftId": draft_id
    })));
    let materialized_before = result(&harness.request(json!({
        "operation": "get_draft_state",
        "projectId": project_id,
        "draftId": draft_id
    })));

    result(&harness.request(json!({
        "operation": "render_draft_preview",
        "projectId": project_id,
        "draftId": draft_id,
        "timeMs": 500
    })));

    let committed_after = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let draft_record_after = result(&harness.request(json!({
        "operation": "get_draft",
        "projectId": project_id,
        "draftId": draft_id
    })));
    let materialized_after = result(&harness.request(json!({
        "operation": "get_draft_state",
        "projectId": project_id,
        "draftId": draft_id
    })));
    assert_eq!(committed_after, committed_before);
    assert_eq!(committed_after["project"]["revision"], 3);
    assert_eq!(draft_record_after, draft_record_before);
    assert_eq!(materialized_after, materialized_before);
    assert_eq!(
        std::fs::read(project_dir.join("project.json")).unwrap(),
        project_before
    );
    assert_eq!(
        std::fs::read(project_dir.join("history.json")).unwrap(),
        history_before
    );
    assert_eq!(std::fs::read(draft_path).unwrap(), draft_before);
}

#[test]
fn rejected_shutter_records_preserve_standalone_and_batch_bytes() {
    let h = Harness::new();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/motion-blur-sampling-v1.json"
    ))
    .unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Shutter failures"})))["projectId"].clone();
    let initial = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = initial["project"]["tracks"][1]["id"].clone();
    let added = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":4,"height":4,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","motionBlur":{"shutterAngleDeg":180,"sampleCount":4}}
    ]})));
    let item = added["aliases"]["leaf"].clone();
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        before["project"]["tracks"][1]["items"][0]["motionBlur"],
        json!({"shutterAngleDeg":180.0,"sampleCount":4})
    );
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let project_bytes = std::fs::read(dir.join("project.json")).unwrap();
    let history_bytes = std::fs::read(dir.join("history.json")).unwrap();
    for case in catalog["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["accepted"] == false)
    {
        let edit = json!({"operation":"update_item","itemId":item,"motionBlur":case["value"]});
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":edit}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
                {"operation":"update_item","itemId":item,"motionBlur":{"shutterAngleDeg":0,"sampleCount":1}},edit
            ]}),
        ] {
            let output = h.request(request);
            assert!(!output.status.success(), "{}", case["name"]);
            let failure = event(&output);
            assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT", "{failure}");
            assert_eq!(failure["error"]["retryable"], false);
            assert_eq!(
                std::fs::read(dir.join("project.json")).unwrap(),
                project_bytes
            );
            assert_eq!(
                std::fs::read(dir.join("history.json")).unwrap(),
                history_bytes
            );
        }
    }
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn malformed_and_unknown_fields_are_invalid_argument_errors() {
    let harness = Harness::new();
    for request in [
        json!({ "operation": "not_a_command" }),
        json!({ "operation": "status", "unexpected": true }),
        json!({ "operation": "get_state", "projectId": "missing", "startMs": 0 }),
    ] {
        let output = harness.request(request);
        assert!(!output.status.success());
        let envelope = event(&output);
        assert_eq!(envelope["type"], "error");
        assert_eq!(envelope["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(envelope["error"]["retryable"], false);
    }
}

#[test]
fn canonical_status_requests_negotiate_protocol_version_and_capabilities() {
    let harness = Harness::new();
    let contract = headless_contract();

    for request_name in [
        "statusDefault",
        "statusCurrent",
        "statusLayoutCurrent",
        "statusStyledCurrent",
    ] {
        let status = result(&harness.request(contract["requests"][request_name].clone()));
        assert_eq!(status["protocolVersion"], contract["version"]);
        assert_eq!(
            status["projectSchemaVersion"],
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        assert_eq!(status["textLayoutVersion"], 2);
        assert_eq!(
            status["subsystems"]["editor"]["capabilities"],
            contract["status"]["editorCapabilities"]
        );
        for field in contract["status"]["requiredFields"].as_array().unwrap() {
            assert!(
                status.get(field.as_str().unwrap()).is_some(),
                "missing canonical status field {field}"
            );
        }
    }
}

#[test]
fn ready_renderer_advertises_canonical_linear_composition_in_protocol_v1() {
    let harness = Harness::new();
    let contract = headless_contract();
    for request_name in ["statusDefault", "statusCurrent"] {
        let status = result(&harness.request(contract["requests"][request_name].clone()));
        assert_eq!(status["protocolVersion"], 1);
        let ready = status["subsystems"]["rendering"]["ready"]
            .as_bool()
            .expect("rendering readiness must be a boolean");
        if std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref() == Ok("1") {
            assert!(ready, "required native renderer is unavailable: {status}");
        }
        let rendering_capabilities = if ready {
            assert_eq!(status["subsystems"]["rendering"]["error"], Value::Null);
            contract["status"]["renderingCapabilities"].clone()
        } else {
            assert_eq!(
                status["subsystems"]["rendering"]["error"]["code"],
                "DEPENDENCY_UNAVAILABLE"
            );
            assert_eq!(
                status["subsystems"]["rendering"]["error"]["retryable"],
                false
            );
            json!([])
        };
        assert_eq!(
            status["subsystems"]["rendering"]["capabilities"],
            rendering_capabilities
        );
        assert_eq!(
            status["subsystems"]["editor"]["capabilities"],
            contract["status"]["editorCapabilities"]
        );
        let mut expected = contract["status"]["editorCapabilities"]
            .as_array()
            .unwrap()
            .clone();
        expected.extend(rendering_capabilities.as_array().unwrap().iter().cloned());
        assert_eq!(status["capabilities"], json!(expected));
        for capabilities in [
            &status["capabilities"],
            &status["subsystems"]["rendering"]["capabilities"],
        ] {
            assert_eq!(
                capabilities
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|capability| **capability == json!("linear_light_compositing_v1"))
                    .count(),
                usize::from(ready)
            );
        }
        assert!(
            !status["subsystems"]["editor"]["capabilities"]
                .as_array()
                .unwrap()
                .contains(&json!("linear_light_compositing_v1"))
        );
    }
}

#[test]
fn canonical_unsupported_version_and_unknown_field_are_stable_errors() {
    let harness = Harness::new();
    let contract = headless_contract();
    let expected_error = &contract["negotiation"]["unsupportedError"];
    let catalog = error_catalog();

    for request_name in [
        "statusUnsupported",
        "statusUnknownField",
        "statusLayoutUnsupported",
        "statusStyledUnsupported",
    ] {
        let output = harness.request(contract["requests"][request_name].clone());
        assert!(!output.status.success());
        let error = event(&output)["error"].clone();
        assert_eq!(error["code"], expected_error["code"]);
        assert_eq!(error["retryable"], expected_error["retryable"]);
        let code = error["code"].as_str().unwrap();
        assert_eq!(error["retryable"], catalog["codes"][code]["retryable"]);
    }
}

#[test]
fn every_stdout_line_is_a_json_event_envelope() {
    let harness = Harness::new();
    let output = harness.request(json!({ "operation": "status" }));
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let parsed: Value = serde_json::from_str(lines[0]).unwrap();
    assert!(matches!(
        parsed["type"].as_str(),
        Some("result" | "progress" | "error")
    ));
    assert!(
        headless_contract()["events"]
            .as_array()
            .unwrap()
            .contains(&parsed["type"])
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn health_succeeds_when_editor_is_ready_and_rendering_is_degraded() {
    let harness = Harness::new();
    let output = harness.health_with_missing_rendering();
    let status = result(&output);
    assert_eq!(status["ready"], true);
    assert_eq!(status["subsystems"]["editor"]["ready"], true);
    assert_eq!(status["subsystems"]["rendering"]["ready"], false);
    let capabilities = status["capabilities"].as_array().unwrap();
    assert!(capabilities.contains(&json!("projects")));
    assert!(capabilities.contains(&json!("timeline")));
    assert!(capabilities.contains(&json!("mask_models_v1")));
    assert!(capabilities.contains(&json!("mask_animation_v1")));
    assert!(capabilities.contains(&json!("matte_models_v1")));
    assert!(!capabilities.contains(&json!("track_mattes_v1")));
    assert!(!capabilities.contains(&json!("mask_rendering_v1")));
    assert_eq!(
        status["subsystems"]["editor"]["capabilities"],
        headless_contract()["status"]["editorCapabilities"]
    );
    assert_eq!(
        status["projectSchemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert!(!capabilities.contains(&json!("preview")));
    assert!(!capabilities.contains(&json!("export")));
    assert!(!capabilities.contains(&json!("evaluated_scene_rendering")));
    assert!(!capabilities.contains(&json!("linear_light_compositing_v1")));
    assert_eq!(
        status["subsystems"]["rendering"]["error"]["code"],
        "DEPENDENCY_UNAVAILABLE"
    );
    assert!(!capabilities.contains(&json!("preview_review_presets_v1")));
    assert_eq!(status["subsystems"]["rendering"]["capabilities"], json!([]));
    assert!(capabilities.contains(&json!("shape_items")));
    assert!(!capabilities.contains(&json!("shape_rendering")));
}

#[test]
fn transform2d_round_trips_and_resets_through_public_protocol() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Transform2D"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let add=result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{
        "operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000",
        "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}
    }})));
    let item = &add["changedIds"][0];
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/transform2d-v1.json")).unwrap();
    let transform = &catalog["valid"][1]["value"];
    let update = result(&harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{
            "operation":"update_item","itemId":item,"transform2d":transform
        }}),
    ));
    assert_eq!(update["revision"], 2);
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    let actual: opencut_editor_core::Transform2D =
        serde_json::from_value(state["project"]["tracks"][1]["items"][0]["transform2d"].clone())
            .unwrap();
    assert_eq!(actual, serde_json::from_value(transform.clone()).unwrap());
    let malformed = harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{
            "operation":"update_item","itemId":item,"transform2d":{"scaleX":1}
        }}),
    );
    assert_eq!(event(&malformed)["error"]["code"], "INVALID_ARGUMENT");
    result(&harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{
            "operation":"update_item","itemId":item,"transform2d":null
        }}),
    ));
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert!(
        state["project"]["tracks"][1]["items"][0]
            .get("transform2d")
            .is_none()
    );
}

#[test]
fn stacking_public_protocol_and_batch_aliases() {
    let harness = Harness::new();
    let status = result(&harness.request(json!({"operation":"status"})));
    assert!(
        status["capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("stacking"))
    );
    let created = result(&harness.request(json!({"operation":"create_project","name":"Stacking"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let added=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"item_set_z_index","itemId":"@box","zIndex":-5},
        {"operation":"item_reorder","itemId":"@box","index":0},
        {"operation":"track_reorder","trackId":track,"index":0}
    ]})));
    assert_eq!(added["revision"], 1);
    let item = &added["aliases"]["box"];
    for (revision, edit) in [
        (
            1,
            json!({"operation":"item_set_z_index","itemId":item,"zIndex":2147483647}),
        ),
        (
            2,
            json!({"operation":"item_reorder","itemId":item,"index":0}),
        ),
        (
            3,
            json!({"operation":"track_reorder","trackId":track,"index":1}),
        ),
    ] {
        assert_eq!(
            result(&harness.request(
                json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit})
            ))["revision"],
            revision + 1
        );
    }
    let state = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["zIndex"],
        2147483647
    );
    assert_eq!(state["project"]["tracks"][1]["items"][0]["stackOrder"], 0);
    let malformed=harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":4,"edit":{"operation":"item_set_z_index","itemId":item,"zIndex":0,"url":"https://example.com"}}));
    assert_eq!(event(&malformed)["error"]["code"], "INVALID_ARGUMENT");
}

#[test]
fn ungroup_protocol_workflow_aliases_rollback_and_history() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({"operation":"create_project","name":"Ungroup"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let edit = |revision, value| {
        result(&harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":value}),
        ))
    };
    let group = edit(
        0,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
    );
    let group_id = &group["changedIds"][0];
    let child = edit(
        1,
        json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":7,"positionY":9,"scale":1,"opacity":1}}),
    );
    let child_id = &child["changedIds"][0];
    edit(
        2,
        json!({"operation":"item_set_parent","itemId":child_id,"parent":{"scope":"root","id":group_id}}),
    );
    edit(
        3,
        json!({"operation":"item_set_z_index","itemId":child_id,"zIndex":-7}),
    );
    let before = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    for (revision, target, code) in [
        (0, group_id.clone(), "REVISION_CONFLICT"),
        (4, json!("absent"), "ITEM_NOT_FOUND"),
        (4, child_id.clone(), "INVALID_ARGUMENT"),
    ] {
        let response = event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"group_ungroup","groupId":target}})));
        assert_eq!(response["error"]["code"], code);
        assert_eq!(response["error"]["retryable"], code == "REVISION_CONFLICT");
    }
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-parent-v1.json")).unwrap();
    for fixture in catalog["invalid"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["value"]["operation"] == "group_ungroup")
    {
        let output = harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":4,"edit":fixture["value"]}),
        );
        assert!(!output.status.success());
        assert_eq!(event(&output)["type"], "error");
    }
    let failed = event(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":4,"operations":[{"operation":"group_ungroup","groupId":group_id},{"operation":"group_ungroup","groupId":group_id}]})));
    assert_eq!(failed["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id}))),
        before
    );
    edit(
        4,
        json!({"operation":"update_track","trackId":track,"locked":true}),
    );
    let locked=event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":5,"edit":{"operation":"group_ungroup","groupId":group_id}})));
    assert_eq!(locked["error"]["code"], "TRACK_LOCKED");
    edit(
        5,
        json!({"operation":"update_track","trackId":track,"locked":false}),
    );
    let removed = edit(6, json!({"operation":"group_ungroup","groupId":group_id}));
    assert_eq!(removed["changedIds"], json!([group_id, child_id]));
    let after = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert!(
        after["project"]["tracks"][1]["items"][0]
            .get("parent")
            .is_none()
    );
    assert_eq!(after["project"]["tracks"][1]["items"][0]["zIndex"], -7);
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":7})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        before["project"]["tracks"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":8})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        after["project"]["tracks"]
    );
    let batch=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":9,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"g"},
        {"operation":"item_set_parent","itemId":child_id,"parent":{"scope":"root","id":"@g"}},
        {"operation":"item_set_z_index","itemId":"@g","zIndex":5},
        {"operation":"group_ungroup","groupId":"@g"}
    ]})));
    assert_eq!(batch["revision"], 10);
    assert!(batch["aliases"]["g"].is_string());
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        after["project"]["tracks"]
    );
}

#[test]
fn ungroup_null_alias_batch_is_rejected_before_any_publication() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Null alias"})));
    let id = created["projectId"].as_str().unwrap();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let created=result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}})));
    let group = &created["changedIds"][0];
    let directory = harness.root.path().join("projects").join(id);
    let snapshot = || {
        (
            std::fs::read(directory.join("project.json")).unwrap(),
            std::fs::read(directory.join("history.json")).unwrap(),
        )
    };
    let before = snapshot();
    for alias in [Value::Null, json!("removed"), json!(42)] {
        let malformed = json!({"operation":"group_ungroup","groupId":group,"resultAlias":alias});
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":malformed}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"item_set_z_index","itemId":group,"zIndex":7},malformed]}),
        ] {
            let output = harness.request(request);
            assert!(!output.status.success());
            assert_eq!(event(&output)["error"]["code"], "INVALID_ARGUMENT");
            assert_eq!(event(&output)["error"]["retryable"], false);
            assert_eq!(snapshot(), before);
            assert_eq!(
                result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]
                    ["revision"],
                1
            );
        }
    }
}

#[test]
fn component_nested_input_failures_preserve_headless_generation() {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/component-definitions-v1.json"
    ))
    .unwrap();
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Nested validation"})));
    let id = created["projectId"].as_str().unwrap();
    let dir = harness.root.path().join("projects").join(id);
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    for fixture in catalog["itemValidationFixtures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            f["valid"] == false && f["operation"]["tracks"][0]["items"][0]["type"] == "text"
        })
    {
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":fixture["operation"]}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[catalog["validOperations"][0],fixture["operation"]]}),
        ] {
            let failed = event(&harness.request(request));
            assert_eq!(
                failed["error"]["code"], "INVALID_ARGUMENT",
                "{}",
                fixture["id"]
            );
            assert_eq!(failed["error"]["retryable"], false);
            assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before.0);
            assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), before.1);
        }
    }
}

#[test]
fn shape_contract_standalone_batch_and_atomic_failures() {
    let harness = Harness::new();
    let id=result(&harness.request(json!({"operation":"create_project","name":"Shapes"})))["projectId"].clone();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/shape-items-v1.json")).unwrap();
    let mut revision = 0;
    for f in catalog["valid"].as_array().unwrap() {
        let mut edit = f["value"].clone();
        edit["trackId"] = track.clone();
        let r = result(&harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        revision = r["revision"].as_u64().unwrap();
    }
    let dir = harness
        .root
        .path()
        .join("projects")
        .join(id.as_str().unwrap());
    let files = || ["project.json", "history.json"].map(|n| std::fs::read(dir.join(n)).unwrap());
    let before = files();
    for f in catalog["invalid"].as_array().unwrap() {
        let mut edit = f["value"].clone();
        edit["trackId"] = track.clone();
        let failed = event(&harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(failed["error"]["code"], "INVALID_ARGUMENT", "{f}: {failed}");
        assert_eq!(files(), before);
    }
    let mut edit = catalog["valid"][0]["value"].clone();
    edit["trackId"] = track.clone();
    edit["resultAlias"] = json!("shape");
    let r=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[edit,{"operation":"update_item","itemId":"@shape","stroke":null}]})));
    revision = r["revision"].as_u64().unwrap();
    assert!(r["aliases"]["shape"].is_string());
    let before = files();
    let mut valid = catalog["valid"][0]["value"].clone();
    valid["trackId"] = track.clone();
    let failed=event(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[valid,{"operation":"delete_item","itemId":"missing"}]})));
    assert_eq!(failed["type"], "error");
    assert_eq!(files(), before);
    let failed=event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"delete_item","itemId":r["aliases"]["shape"]}})));
    assert_eq!(failed["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(files(), before);
    let undone = result(
        &harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision})),
    );
    result(
        &harness.request(
            json!({"operation":"redo","projectId":id,"expectedRevision":undone["revision"]}),
        ),
    );
    let opened = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        opened["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        opened["project"]["tracks"][1]["items"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
}

#[test]
fn raw_shape_duplicates_fail_before_single_batch_or_draft_mutation() {
    let h = Harness::new();
    let output=h.request(json!({"operation":"create_project","name":"Raw shapes","settings":{"width":160,"height":120,"fps":24}}));
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = response["result"]["projectId"].as_str().unwrap();
    let dir = h.root.path().join("projects").join(id);
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    let project: Value = serde_json::from_slice(&before).unwrap();
    let track = project["tracks"][1]["id"].as_str().unwrap();
    let edit = format!(
        r#"{{"operation":"add_shape","trackId":"{track}","startMs":0,"durationMs":1000,"geometry":{{"type":"ellipse","width":2,"height":2}},"fill":{{"type":"solid","color":{{"r":2,"r":1,"g":0,"b":0,"a":1}}}},"stroke":null}}"#
    );
    let good = edit.replace("\"r\":2,", "");
    for (operation, fields) in [
        ("edit", format!(r#""edit":{edit}"#)),
        ("edit_batch", format!(r#""operations":[{good},{edit}]"#)),
        ("create_draft", format!(r#""operations":[{good},{edit}]"#)),
    ] {
        let raw = format!(
            r#"{{"operation":"{operation}","projectId":"{id}","expectedRevision":0,{fields}}}"#
        );
        let output = h.request_raw(&raw);
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["type"], "error", "{response}");
        assert!(
            response.to_string().contains("duplicate field"),
            "{response}"
        );
        assert_eq!(before, std::fs::read(dir.join("project.json")).unwrap());
        assert_eq!(history, std::fs::read(dir.join("history.json")).unwrap());
    }
}

#[test]
fn grid_contract_reaches_core_and_preserves_batch_atomicity() {
    let h = Harness::new();
    let id =
        result(&h.request(json!({"operation":"create_project","name":"SVG"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/procedural-grids-v1.json")).unwrap();
    let mut revision = 0;
    for f in catalog["valid"].as_array().unwrap() {
        let r=result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"add_grid","trackId":track,"startMs":0,"durationMs":1000,"grid":f["grid"]}})));
        revision = r["revision"].as_u64().unwrap();
    }
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    for f in catalog["invalid"].as_array().unwrap() {
        let e=event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"add_grid","trackId":track,"startMs":0,"durationMs":1000,"grid":f["grid"]}})));
        assert_eq!(e["error"]["code"], "INVALID_ARGUMENT", "{e}");
    }
    let batch = json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"add_grid","trackId":track,"startMs":0,"durationMs":1000,"grid":catalog["valid"][0]["grid"],"resultAlias":"icon"},{"operation":"item_set_z_index","itemId":"@icon","zIndex":3},{"operation":"delete_item","itemId":"missing"}]});
    assert_eq!(event(&h.request(batch))["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn repeater_contract_reaches_core_drafts_and_atomic_batches() {
    let harness = Harness::new();
    let id = result(&harness.request(json!({"operation":"create_project","name":"Repeaters"})))
        ["projectId"]
        .clone();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/repeaters-v1.json")).unwrap();
    let shape = result(&harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":0,
        "edit":{"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
        "geometry":{"type":"rectangle","width":20,"height":20},
        "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}}),
    ));
    let source = shape["changedIds"][0].clone();
    let mut descriptor = catalog["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = source;
    let added = result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,
        "edit":{"operation":"add_repeater","trackId":track,"startMs":100,"durationMs":800,"repeater":descriptor}})));
    let repeater = added["changedIds"][0].clone();
    let draft = result(&harness.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":2,
        "operations":[{"operation":"update_item","itemId":repeater,"repeater":descriptor}],"label":"repeater"})));
    let draft_state = result(
        &harness
            .request(json!({"operation":"get_draft_state","projectId":id,"draftId":draft["id"]})),
    );
    assert_eq!(
        draft_state["project"]["tracks"][1]["items"][1]["type"],
        "repeater"
    );
    let before = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let failed = event(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":2,"operations":[
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor,"resultAlias":"copies"},
        {"operation":"delete_item","itemId":"missing"}
    ]})));
    assert_eq!(failed["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&harness.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn repeater_replacement_aliases_reach_core_and_roll_back_across_reopen() {
    let harness = Harness::new();
    let id = result(&harness.request(json!({"operation":"create_project","name":"Aliases"})))["projectId"].clone();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/repeaters-v1.json")).unwrap();
    let mut descriptor = catalog["valid"][0]["repeater"].clone();
    descriptor["source"]["id"] = json!("@source");
    let shape = json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
        "geometry":{"type":"rectangle","width":20,"height":20},
        "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"resultAlias":"source"});
    let mut replacement = shape.clone();
    replacement["resultAlias"] = json!("replacement");
    let mut updated = descriptor.clone();
    updated["source"]["id"] = json!("@replacement");
    let operations = json!([shape,
        {"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":descriptor,"resultAlias":"copies"},
        replacement, {"operation":"update_item","itemId":"@copies","repeater":updated}]);
    let success = result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":operations})));
    assert_eq!(success["revision"], 1);
    let after = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        after["project"]["tracks"][1]["items"][1]["repeater"]["source"]["id"],
        success["aliases"]["replacement"]
    );
    let directory = harness
        .root
        .path()
        .join("projects")
        .join(id.as_str().unwrap());
    let bytes = || {
        (
            std::fs::read(directory.join("project.json")).unwrap(),
            std::fs::read(directory.join("history.json")).unwrap(),
        )
    };
    let before = bytes();
    for case in ["missing", "forward", "trailing"] {
        let mut bad = operations.clone();
        let code = match case {
            "missing" => {
                bad[3]["repeater"]["source"]["id"] = json!("@missing");
                "VALIDATION_FAILED"
            }
            "forward" => {
                bad.as_array_mut().unwrap().swap(2, 3);
                "VALIDATION_FAILED"
            }
            _ => {
                bad.as_array_mut()
                    .unwrap()
                    .push(json!({"operation":"delete_item","itemId":"missing"}));
                "ITEM_NOT_FOUND"
            }
        };
        let failed = event(&harness.request(
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":bad}),
        ));
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(bytes(), before);
        assert_eq!(
            result(&harness.request(json!({"operation":"get_state","projectId":id}))),
            after
        );
    }
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":1})));
    let undone = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert!(
        undone["project"]["tracks"][1]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":2})));
    assert_eq!(
        result(&harness.request(json!({"operation":"get_state","projectId":id})))["project"]["tracks"],
        after["project"]["tracks"]
    );
}

#[test]
fn inherited_timing_batch_round_trip_and_rollback() {
    let h = Harness::new();
    let id=result(&h.request(json!({"operation":"create_project","name":"Inherited timing"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let channels = json!([{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]);
    let written=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"staggerMs":125,"resultAlias":"parent"},
        {"operation":"update_item","itemId":"@parent","transform2d":null},
        {"operation":"set_animation_channels","itemId":"@parent","animationChannels":channels}
    ]})));
    assert_eq!(written["revision"], 1);
    let parent = written["aliases"]["parent"].clone();
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(before["project"]["tracks"][1]["items"][0]["staggerMs"], 125);
    assert_eq!(
        before["project"]["tracks"][1]["items"][0]["animationChannels"],
        channels
    );
    let failed = event(&h.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
            {"operation":"update_item","itemId":parent,"staggerMs":250},
            {"operation":"update_item","itemId":"missing","staggerMs":5}
        ]}),
    ));
    assert_eq!(failed["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn inherited_bounds_batch_fails_before_transport_reports_publication() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Bounds"})))["projectId"]
        .clone();
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = before["project"]["tracks"][1]["id"].clone();
    let failed=event(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"update_item","itemId":"@parent","transform2d":null},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":500,"height":10,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@parent"}},
        {"operation":"set_animation_channels","itemId":"@parent","animationChannels":[{"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":100},"curve":"hold"}]}]}
    ]})));
    assert_eq!(failed["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(failed["error"]["retryable"], false);
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id}))),
        before
    );
}

#[test]
fn versioned_presets_match_fixed_primitives_and_preserve_wire_failures_history_and_reopen() {
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/animation-presets-v1.json")).unwrap();
    let id =
        result(&h.request(json!({"operation":"create_project","name":"Presets"})))["projectId"]
            .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let mut request = catalog["examples"]["apply"].clone();
    request["itemId"] = json!("@seed");
    let written=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"seed"},request]})));
    assert_eq!(written["revision"], 1);
    request["itemId"] = written["aliases"]["seed"].clone();
    let saved = result(&h.request(json!({"operation":"open_project","projectId":id})));
    let item = &saved["project"]["tracks"][1]["items"][0];
    assert_eq!(
        item["animationChannels"],
        json!([catalog["examples"]["resolvedChannel"]])
    );
    assert_eq!(
        item["animationPresetProvenance"]["transform.opacity"],
        catalog["examples"]["provenance"]
    );
    for (revision, edit, code) in [
        (0, request.clone(), "REVISION_CONFLICT"),
        (1, request.clone(), "INVALID_ARGUMENT"),
        (
            1,
            {
                let mut e = request.clone();
                e["presetVersion"] = json!(2);
                e
            },
            "INVALID_ARGUMENT",
        ),
        (
            1,
            {
                let mut e = request.clone();
                e["itemId"] = json!("missing");
                e
            },
            "ITEM_NOT_FOUND",
        ),
        (
            1,
            {
                let mut e = request.clone();
                e.as_object_mut().unwrap().remove("presetVersion");
                e
            },
            "INVALID_ARGUMENT",
        ),
        (
            1,
            {
                let mut e = request.clone();
                e["presetVersion"] = json!("latest");
                e
            },
            "INVALID_ARGUMENT",
        ),
    ] {
        let failed = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(failed["error"]["retryable"], code == "REVISION_CONFLICT");
        assert_eq!(
            result(&h.request(json!({"operation":"get_state","projectId":id}))),
            saved
        );
    }
    for parameters in [
        r#"["transform.opacity",0,500,0.0,1.0]"#,
        r#"["transform.opacity",0,500,0.0,1.0,"linear"]"#,
        r#"{"property":"transform.opacity","startMs":0,"durationMs":500,"from":99.0,"from":0.0,"to":1.0,"curve":"linear"}"#,
        r#"{"property":"transform.opacity","startMs":0,"durationMs":500,"from":0.0,"to":1.0,"curve":{"type":"cubic_bezier","x1":99.0,"x1":0.0,"y1":0.0,"x2":1.0,"y2":1.0}}"#,
    ] {
        let mut raw_edit = request.clone();
        raw_edit["parameters"] = json!("PARAMETERS_TOKEN");
        let raw = serde_json::to_string(&json!({
            "operation":"edit","projectId":id,"expectedRevision":1,"edit":raw_edit
        }))
        .unwrap()
        .replace("\"PARAMETERS_TOKEN\"", parameters);
        let failed = event(&h.request_raw(&raw));
        assert_eq!(failed["error"]["code"], "INVALID_ARGUMENT", "{raw}");
        assert_eq!(failed["error"]["retryable"], false);
        assert_eq!(
            result(&h.request(json!({"operation":"get_state","projectId":id}))),
            saved
        );
    }
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_track","trackId":track,"locked":true}})));
    let locked = event(
        &h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":request})),
    );
    assert_eq!(locked["error"]["code"], "TRACK_LOCKED");
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{"operation":"update_track","trackId":track,"locked":false}})));
    request["collisionPolicy"] = json!("replace");
    request["parameters"]["from"] = json!(1.0);
    request["parameters"]["to"] = json!(0.0);
    result(
        &h.request(json!({"operation":"edit","projectId":id,"expectedRevision":3,"edit":request})),
    );
    let replaced = result(&h.request(json!({"operation":"open_project","projectId":id})));
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":4})));
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id})))["project"]["tracks"],
        saved["project"]["tracks"]
    );
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":5})));
    assert_eq!(
        result(&h.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        replaced["project"]["tracks"]
    );
}

#[test]
fn rejected_legacy_preset_requests_preserve_wire_errors_and_persisted_generation() {
    let h = Harness::new();
    let id = result(&h.request(json!({"operation":"create_project","name":"Legacy presets"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let added = result(&h.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":0,
        "edit":{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,
            "width":32,"height":32,"color":"#ff0000",
            "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}}),
    ));
    let item = added["changedIds"][0].clone();
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    for name in ["project.json", "history.json"] {
        let path = dir.join(name);
        let mut document: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if name == "project.json" {
            document["schemaVersion"] = json!(28);
            document.as_object_mut().unwrap().remove("audioBuses");
            document.as_object_mut().unwrap().remove("soundDefinitions");
        } else {
            for kind in ["undo", "redo"] {
                for snapshot in document[kind].as_array_mut().unwrap() {
                    snapshot["schemaVersion"] = json!(28);
                    snapshot.as_object_mut().unwrap().remove("audioBuses");
                    snapshot.as_object_mut().unwrap().remove("soundDefinitions");
                }
            }
        }
        std::fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    }
    let bytes = || {
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        )
    };
    let before = bytes();
    let mut preset = json!({"operation":"apply_animation_preset","itemId":item,
        "presetId":"scalar_tween","presetVersion":2,
        "parameters":{"property":"transform.opacity","startMs":0,"durationMs":400,
            "from":0,"to":1,"curve":"linear"}});
    for request in [
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":preset}),
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
            {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":8,"height":8,
                "color":"#00ff00","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"seed"},
            {"operation":"apply_animation_preset","itemId":"@seed","presetId":"scalar_tween","presetVersion":2,
                "parameters":preset["parameters"]}]}),
    ] {
        let response = h.request(request);
        assert!(!response.status.success());
        let failure = event(&response);
        assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(failure["error"]["retryable"], false);
        assert_eq!(bytes(), before);
    }
    preset["presetVersion"] = json!(1);
    let edited = result(
        &h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":preset})),
    );
    assert_eq!(edited["revision"], 2);
    let saved = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        saved["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        saved["project"]["tracks"][1]["items"][0]["animationPresetProvenance"]["transform.opacity"]
            ["presetVersion"],
        1
    );
    let history: Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    assert_eq!(history["undo"].as_array().unwrap().len(), 2);
    assert!(
        history["undo"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["schemaVersion"] == opencut_editor_core::PROJECT_SCHEMA_VERSION)
    );
    assert_eq!(
        result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})))["revision"],
        3
    );
    assert_eq!(
        result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})))["revision"],
        4
    );
}

#[test]
fn motion_pack_raw_wire_errors_preserve_schema29_documents() {
    let h = Harness::new();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../contracts/initial-motion-preset-pack-v1.json"
    ))
    .unwrap();
    let id=result(&h.request(json!({"operation":"create_project","name":"Pack raw decoding"})))["projectId"].clone();
    let track =
        result(&h.request(json!({"operation":"get_state","projectId":id})))["project"]["tracks"][1]
            ["id"]
            .clone();
    let item=result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})))["changedIds"][0].clone();
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    for name in ["project.json", "history.json"] {
        let path = dir.join(name);
        let mut document: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if name == "project.json" {
            document["schemaVersion"] = json!(29);
            document.as_object_mut().unwrap().remove("audioBuses");
            document.as_object_mut().unwrap().remove("soundDefinitions");
        } else {
            for kind in ["undo", "redo"] {
                for snapshot in document[kind].as_array_mut().unwrap() {
                    snapshot["schemaVersion"] = json!(29);
                    snapshot.as_object_mut().unwrap().remove("audioBuses");
                    snapshot.as_object_mut().unwrap().remove("soundDefinitions");
                }
            }
        }
        std::fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    }
    let bytes = || {
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
        )
    };
    let before = bytes();
    for parameters in fixture["invalidRawParameters"].as_array().unwrap() {
        let request = format!(
            "{{\"operation\":\"edit\",\"projectId\":{id},\"expectedRevision\":1,\"edit\":{{\"operation\":\"apply_animation_preset\",\"itemId\":{item},\"presetId\":\"impact_slam\",\"presetVersion\":1,\"parameters\":{}}}}}",
            parameters.as_str().unwrap()
        );
        let response = h.request_raw(&request);
        assert!(!response.status.success());
        let failure = event(&response);
        assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(failure["error"]["retryable"], false);
        assert_eq!(bytes(), before);
    }
    for entry in fixture["presets"].as_array().unwrap() {
        let response=h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
            {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":8,"height":8,"color":"#00ff00","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"motion"},
            {"operation":"apply_animation_preset","itemId":"@motion","presetId":entry["id"],"presetVersion":2,"parameters":entry["parameters"]}]}));
        assert_eq!(
            event(&response)["error"]["code"],
            "INVALID_ARGUMENT",
            "{}",
            event(&response)
        );
        assert_eq!(event(&response)["error"]["retryable"], false);
        assert_eq!(bytes(), before);
    }
}

#[test]
fn motion_pack_headless_canonical_sources_aliases_and_history_survive_reopen() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../contracts/initial-motion-preset-pack-v1.json"
    ))
    .unwrap();
    for entry in fixture["presets"].as_array().unwrap() {
        let h = Harness::new();
        let id = result(&h.request(json!({"operation":"create_project","name":"Pack lifecycle"})))
            ["projectId"]
            .clone();
        let initial = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let track = initial["project"]["tracks"][1]["id"].clone();
        let applied=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
            {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"motion"},
            {"operation":"apply_animation_preset","itemId":"@motion","presetId":entry["id"],"presetVersion":1,"parameters":entry["parameters"]}]})));
        assert_eq!(applied["revision"], 1);
        assert!(applied["aliases"]["motion"].is_string());
        let saved = result(&h.request(json!({"operation":"open_project","projectId":id})));
        assert_eq!(
            saved["project"]["schemaVersion"],
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        assert_eq!(
            saved["project"]["tracks"][1]["items"][0]["animationPresetProvenance"],
            entry["expectedProvenance"]
        );
        result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":1})));
        assert_eq!(
            result(&h.request(json!({"operation":"get_state","projectId":id})))["project"]["tracks"],
            initial["project"]["tracks"]
        );
        result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":2})));
        assert_eq!(
            result(&h.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
            saved["project"]["tracks"]
        );
    }
}

#[test]
fn canonical_unknown_time_members_reject_standalone_and_aliased_batch_without_mutation() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/motion-graphics-v1.json")).unwrap();
    // The governed public gate consumes every case through the owning native
    // decoder, including supported variants, before exercising wire failures.
    for fixture in catalog["timeExpressionCases"].as_array().unwrap() {
        assert_eq!(
            serde_json::from_value::<opencut_editor_core::TimeExpression>(fixture["value"].clone())
                .is_ok(),
            fixture["accept"].as_bool().unwrap(),
            "{}",
            fixture["id"]
        );
    }
    let h = Harness::new();
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Closed time variants"})),
    )["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let added=result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":5,"height":5,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})));
    let item = added["changedIds"][0].clone();
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let files = || {
        (
            std::fs::read(dir.join("project.json")).unwrap(),
            std::fs::read(dir.join("history.json")).unwrap(),
            std::fs::read_dir(dir.join("assets")).map_or(0, |files| files.count()),
        )
    };
    let before = files();
    for fixture in catalog["timeExpressionCases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["accept"] == false)
    {
        for batch in [false, true] {
            let edit = json!({"operation":"set_item_start_time","scope":"root","itemId":if batch{json!("@box")}else{item.clone()},"time":fixture["value"]});
            let request = if batch {
                json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":5,"height":5,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},edit]})
            } else {
                json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":edit})
            };
            let error = event(&h.request(request));
            assert_eq!(
                error["error"]["code"], "INVALID_ARGUMENT",
                "{} batch={batch}: {error}",
                fixture["id"]
            );
            assert_eq!(error["error"]["retryable"], false);
            assert_eq!(files(), before);
        }
    }
    assert_eq!(
        result(&h.request(json!({"operation":"get_state","projectId":id})))["project"]["revision"],
        1
    );
}

#[test]
fn mask_metadata_roundtrips_aliases_order_clear_and_atomic_failure() {
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Mask metadata"})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let typed_masks: Vec<opencut_editor_core::Mask> =
        serde_json::from_value(catalog["stackCases"][1]["value"].clone()).unwrap();
    let masks = serde_json::to_value(typed_masks).unwrap();
    let added = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","masks":masks}
    ]})));
    let item = added["aliases"]["leaf"].clone();
    let before = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        before["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(before["project"]["tracks"][1]["items"][0]["masks"], masks);
    for request in [
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"masks":null}}),
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"update_item","itemId":item,"masks":[]},{"operation":"delete_item","itemId":"missing"}]}),
    ] {
        assert!(!h.request(request).status.success());
        assert_eq!(
            result(&h.request(json!({"operation":"open_project","projectId":id}))),
            before
        );
    }
    let reversed: Vec<_> = masks.as_array().unwrap().iter().rev().cloned().collect();
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"update_item","itemId":item,"masks":reversed}})));
    let ordered = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        ordered["project"]["tracks"][1]["items"][0]["masks"],
        json!(reversed)
    );
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{"operation":"update_item","itemId":item,"masks":[]}})));
    let cleared = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert!(
        cleared["project"]["tracks"][1]["items"][0]
            .get("masks")
            .is_none()
    );
}

#[test]
fn canonical_raw_mask_duplicates_reject_single_batch_and_draft_before_publication() {
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Raw mask rejection"})))
        ["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let added = result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})));
    let item = added["changedIds"][0].clone();
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    for case in catalog["rawRejectedCases"].as_array().unwrap() {
        let mask = case["json"].as_str().unwrap();
        let edit = format!(r#"{{"operation":"update_item","itemId":{item},"masks":[{mask}]}}"#);
        let good = serde_json::to_string(
            &json!({"operation":"update_item","itemId":item,"color":"#123456"}),
        )
        .unwrap();
        for (operation, fields) in [
            ("edit", format!(r#""edit":{edit}"#)),
            ("edit_batch", format!(r#""operations":[{good},{edit}]"#)),
            ("create_draft", format!(r#""operations":[{good},{edit}]"#)),
        ] {
            let request = format!(
                r#"{{"operation":"{operation}","projectId":{id},"expectedRevision":1,{fields}}}"#
            );
            let output = h.request_raw(&request);
            assert!(!output.status.success(), "{}", case["name"]);
            let error = event(&output);
            assert_eq!(error["error"]["code"], "INVALID_ARGUMENT", "{error}");
            assert_eq!(error["error"]["retryable"], false);
            assert!(error.to_string().contains("duplicate field"), "{error}");
            assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
            assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
        }
    }
}

#[test]
fn mask_animation_canonical_channels_roundtrip_and_raw_targets_reject_atomically() {
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-rendering-v1.json")).unwrap();
    let models: Value =
        serde_json::from_str(include_str!("../../../contracts/mask-models-v1.json")).unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Typed mask channels","settings":{"width":64,"height":64,"fps":10}})))["projectId"].clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let mut mask = models["cases"][0]["value"].clone();
    mask["id"] = json!("@paint");
    mask["source"]["path"] = json!({"fillRule":"evenodd","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":4,"y":0}},{"type":"lineTo","to":{"x":4,"y":4}},{"type":"lineTo","to":{"x":0,"y":4}},{"type":"close"}]});
    mask["transform"] = json!({"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1});
    mask["featherPx"] = json!(0);
    mask["expansionPx"] = json!(0);
    let mut gradient = mask.clone();
    gradient["id"] = json!("gradient");
    gradient["source"]["paint"] = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":4,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":1}}]});
    let channels: Vec<Value> = catalog["channelCases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fixture| {
            let mut channel = fixture["channel"].clone();
            channel["target"]["id"] = if channel["property"] == "mask.gradient_stops" {
                json!("gradient")
            } else {
                json!("@paint")
            };
            channel
        })
        .collect();
    assert_eq!(channels.len(), 15);
    let typed_channels: Vec<opencut_editor_core::AnimationChannel> =
        serde_json::from_value(json!(channels)).unwrap();
    let expected_channels = serde_json::to_value(typed_channels).unwrap();
    let added = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","masks":[mask,gradient]},
        {"operation":"set_animation_channels","itemId":"@leaf","animationChannels":channels}
    ]})));
    let item = added["aliases"]["leaf"].clone();
    let before = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        before["project"]["tracks"][1]["items"][0]["animationChannels"],
        expected_channels
    );
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let project_bytes = std::fs::read(dir.join("project.json")).unwrap();
    let history_bytes = std::fs::read(dir.join("history.json")).unwrap();
    let raw_channel = r#"{"property":"mask.transform.opacity","target":{"kind":"mask","kind":"mask","scope":"root","id":"@paint"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"hold"}]}"#;
    let edit = format!(
        r#"{{"operation":"set_animation_channels","itemId":{item},"animationChannels":[{raw_channel}]}}"#
    );
    let good =
        serde_json::to_string(&json!({"operation":"update_item","itemId":item,"color":"#123456"}))
            .unwrap();
    for (operation, fields) in [
        ("edit", format!(r#""edit":{edit}"#)),
        ("edit_batch", format!(r#""operations":[{good},{edit}]"#)),
        ("create_draft", format!(r#""operations":[{good},{edit}]"#)),
    ] {
        let raw = format!(
            r#"{{"operation":"{operation}","projectId":{id},"expectedRevision":1,{fields}}}"#
        );
        let error = event(&h.request_raw(&raw));
        assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(error["error"]["retryable"], false);
        assert!(error.to_string().contains("duplicate field"), "{error}");
        assert_eq!(
            std::fs::read(dir.join("project.json")).unwrap(),
            project_bytes
        );
        assert_eq!(
            std::fs::read(dir.join("history.json")).unwrap(),
            history_bytes
        );
    }
}

#[test]
fn canonical_raw_matte_duplicates_reject_single_batch_and_draft_atomically() {
    fn inventory(
        root: &std::path::Path,
    ) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
        let mut files = std::collections::BTreeMap::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        files
    }
    let h = Harness::new();
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/track-mattes-v1.json")).unwrap();
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Raw matte duplicates"})),
    )["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let added = result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}})));
    let item = added["changedIds"][0].clone();
    let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":1,"operations":[{"operation":"update_item","itemId":item,"color":"#123456"}]})));
    let draft_id = draft["id"].clone();
    let dir = h.root.path().join("projects").join(id.as_str().unwrap());
    let before = inventory(&dir);
    let references = catalog["rawDuplicateReferences"].as_array().unwrap();
    let mut edits: Vec<String> = references
        .iter()
        .map(|value| {
            let reference = value.as_str().unwrap();
            format!(r#"{{"operation":"update_item","itemId":{item},"matte":{reference}}}"#)
        })
        .collect();
    edits.push(format!(
        r#"{{"operation":"update_item","itemId":{item},"matte":null,"matte":null}}"#
    ));
    edits.push(format!(
        r#"{{"operation":"update_item","itemId":{item},"matteOnly":false,"matteOnly":true}}"#
    ));
    for edit in edits {
        let prefix = serde_json::to_string(
            &json!({"operation":"update_item","itemId":item,"color":"#123456"}),
        )
        .unwrap();
        for (operation, fields) in [
            ("edit", format!(r#""edit":{edit}"#)),
            ("edit_batch", format!(r#""operations":[{prefix},{edit}]"#)),
            ("create_draft", format!(r#""operations":[{prefix},{edit}]"#)),
            (
                "update_draft",
                format!(r#""draftId":{draft_id},"operations":[{prefix},{edit}]"#),
            ),
        ] {
            let output = h.request_raw(&format!(
                r#"{{"operation":"{operation}","projectId":{id},"expectedRevision":1,{fields}}}"#
            ));
            assert!(!output.status.success());
            let error = event(&output);
            assert_eq!(error["error"]["code"], "INVALID_ARGUMENT", "{error}");
            assert_eq!(error["error"]["retryable"], false);
            assert!(error.to_string().contains("duplicate field"), "{error}");
            assert_eq!(inventory(&dir), before, "{operation} must publish no bytes");
        }
    }
}

#[test]
fn matte_aliases_scoped_graph_and_final_atomic_provider_deletion_roundtrip() {
    let h = Harness::new();
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Matte graph protocol"})),
    )["projectId"]
        .clone();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"][1]["id"].clone();
    let add = |alias: &str| json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":alias});
    let created = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[add("provider"),add("recipient"),{"operation":"update_item","itemId":"@provider","matteOnly":true},{"operation":"update_item","itemId":"@recipient","matte":{"sourceId":"@provider","channel":"alpha"}}]})));
    let provider = created["aliases"]["provider"].clone();
    let recipient = created["aliases"]["recipient"].clone();
    let before = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        before["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        before["project"]["tracks"][1]["items"][0]["matteOnly"],
        true
    );
    assert_eq!(
        before["project"]["tracks"][1]["items"][1]["matte"],
        json!({"sourceId":provider,"channel":"alpha"})
    );
    for (edit, code) in [
        (
            json!({"operation":"update_item","itemId":provider,"matte":{"sourceId":recipient,"channel":"alpha"}}),
            "INVALID_ARGUMENT",
        ),
        (
            json!({"operation":"delete_item","itemId":provider}),
            "ITEM_NOT_FOUND",
        ),
        (
            json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":"missing","channel":"alpha"}}),
            "ITEM_NOT_FOUND",
        ),
    ] {
        let output =
            h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":edit}));
        assert!(!output.status.success());
        let error = event(&output);
        assert_eq!(error["error"]["code"], code, "{error}");
        assert_eq!(error["error"]["retryable"], false);
        assert_eq!(
            result(&h.request(json!({"operation":"open_project","projectId":id}))),
            before
        );
    }
    result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"delete_item","itemId":provider},{"operation":"update_item","itemId":recipient,"matte":null}]})));
    let cleared = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        cleared["project"]["tracks"][1]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        cleared["project"]["tracks"][1]["items"][0]
            .get("matte")
            .is_none()
    );
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    let restored = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(restored["project"]["tracks"], before["project"]["tracks"]);
}

#[test]
fn stale_matte_draft_preview_conflicts_without_artifacts_or_replaying_current() {
    for available in [true, false] {
        let h = Harness::new();
        let id =
            result(&h.request(json!({"operation":"create_project","name":"Stale matte preview"})))
                ["projectId"]
                .clone();
        let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let track = state["project"]["tracks"][1]["id"].clone();
        let add = |alias: &str| json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#0000ff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":alias});
        let created = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[add("provider"),add("recipient")]})));
        let recipient = created["aliases"]["recipient"].clone();
        let provider = created["aliases"]["provider"].clone();
        let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":1,"operations":[{"operation":"update_item","itemId":recipient,"matte":{"sourceId":provider,"channel":"alpha"}}]})));
        let dir = h.root.path().join("projects").join(id.as_str().unwrap());
        let draft_path = dir
            .join("drafts")
            .join(format!("{}.json", draft["id"].as_str().unwrap()));
        if available {
            result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"delete_item","itemId":recipient}})));
        } else {
            let mut raw: Value =
                serde_json::from_slice(&std::fs::read(&draft_path).unwrap()).unwrap();
            raw["baseRevision"] = json!(999);
            std::fs::write(&draft_path, serde_json::to_vec(&raw).unwrap()).unwrap();
        }
        let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let draft_before = std::fs::read(&draft_path).unwrap();
        let history_before = std::fs::read(dir.join("history.json")).unwrap();
        let project_before = std::fs::read(dir.join("project.json")).unwrap();
        let previews = dir.join("previews");
        assert_eq!(std::fs::read_dir(&previews).unwrap().count(), 0);
        result(&h.request(json!({"operation":"get_draft","projectId":id,"draftId":draft["id"]})));
        for operation in ["get_draft_state", "render_draft_preview"] {
            let mut request = json!({"operation":operation,"projectId":id,"draftId":draft["id"]});
            if operation == "render_draft_preview" {
                request["timeMs"] = json!(0);
            }
            let output = h.request(request);
            assert!(!output.status.success());
            let error = event(&output);
            assert_eq!(
                error["error"]["code"], "REVISION_CONFLICT",
                "available={available}: {error}"
            );
            assert_eq!(error["error"]["retryable"], true);
            assert!(
                error.get("result").is_none(),
                "no artifact response: {error}"
            );
            assert_eq!(std::fs::read_dir(&previews).unwrap().count(), 0);
            assert_eq!(std::fs::read(&draft_path).unwrap(), draft_before);
            assert_eq!(
                std::fs::read(dir.join("history.json")).unwrap(),
                history_before
            );
            assert_eq!(
                std::fs::read(dir.join("project.json")).unwrap(),
                project_before
            );
            assert_eq!(
                result(&h.request(json!({"operation":"get_state","projectId":id}))),
                before
            );
        }
    }
}

#[test]
fn canonical_blend_selections_aliases_history_drafts_and_raw_duplicates_roundtrip() {
    let h = Harness::new();
    let contract: Value =
        serde_json::from_str(include_str!("../../../contracts/blend-modes-v1.json")).unwrap();
    let id=result(&h.request(json!({"operation":"create_project","name":"Blend protocol"})))["projectId"].clone();
    let initial = result(&h.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        initial["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    let track = initial["project"]["tracks"][1]["id"].clone();
    let created=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[{"operation":"add_rectangle","resultAlias":"leaf","trackId":track,"startMs":0,"durationMs":1000,"width":16,"height":16,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}},{"operation":"update_item","itemId":"@leaf","blendMode":"multiply"}]})));
    let item = created["aliases"]["leaf"].clone();
    let mut revision = 1;
    for case in contract["acceptedSelections"].as_array().unwrap() {
        if let Some(mode) = case.get("existingMode") {
            result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":item,"blendMode":mode}})));
            revision += 1;
        }
        let mut operation = case["operation"].clone();
        operation["itemId"] = item.clone();
        result(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":operation}),
        ));
        revision += 1;
        let current = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let expected = case.get("expectedMode").unwrap_or(&case["name"]);
        if expected == "normal" {
            assert!(
                current["project"]["tracks"][1]["items"][0]
                    .get("blendMode")
                    .is_none()
            );
        } else {
            assert_eq!(
                current["project"]["tracks"][1]["items"][0]["blendMode"],
                *expected
            );
        }
    }
    let before = result(&h.request(json!({"operation":"get_state","projectId":id})));
    for case in contract["rejectedSelections"].as_array().unwrap() {
        let mut operation = case["operation"].clone();
        operation["itemId"] = item.clone();
        let response = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":operation}),
        ));
        assert_eq!(response["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(response["error"]["retryable"], false);
        assert_eq!(
            result(&h.request(json!({"operation":"get_state","projectId":id})))["project"],
            before["project"]
        );
    }
    let raw = format!(
        r#"{{"operation":"edit","projectId":{id},"expectedRevision":{revision},"edit":{{"operation":"update_item","itemId":{item},"blendMode":"normal","blendMode":"multiply"}}}}"#
    );
    let response = event(&h.request_raw(&raw));
    assert_eq!(response["error"]["code"], "INVALID_ARGUMENT");
    let draft=result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":item,"blendMode":"multiply"}]})));
    let state = result(
        &h.request(json!({"operation":"get_draft_state","projectId":id,"draftId":draft["id"]})),
    );
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["blendMode"],
        "multiply"
    );
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"expectedRevision":revision,"draftId":draft["id"]})));
    revision += 1;
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision})));
    revision += 1;
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":revision})));
    assert_eq!(
        result(&h.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"]
            [1]["items"][0]["blendMode"],
        "multiply"
    );
}

#[test]
fn desktop_compositing_catalog_fresh_alias_standalone_clear_failure_history_reopen_parity() {
    let h = Harness::new();
    let c: Value = serde_json::from_str(include_str!(
        "../../../contracts/desktop-compositing-controls-v1.json"
    ))
    .unwrap();
    let id = result(
        &h.request(json!({"operation":"create_project","name":"Desktop controls API","settings":{"width":64,"height":64,"fps":10}})),
    )["projectId"]
        .clone();
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    let state = read();
    let track = state["project"]["tracks"][1]["id"].clone();
    let created=result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"provider"},
        {"operation":"update_item","itemId":"@provider","matteOnly":true},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":64,"height":64,"color":"#ffffff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"leaf"},
        {"operation":"update_item","itemId":"@leaf","masks":[c["defaultMask"]],"effects":[c["effectDefaults"]["glow"],{"id":"tint","type":"color_tint","color":{"r":0.12345678901234566,"g":0.3333333333333333,"b":0.9876543210987654,"a":0.8765432109876543}}],"matte":{"sourceId":"@provider","channel":"luma"},"blendMode":"screen"},
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"owner"},
        {"operation":"update_item","itemId":"@owner","effects":[c["effectDefaults"]["screen_flash"]],"clip":c["clipValue"]}
    ]})));
    let leaf = created["aliases"]["leaf"].clone();
    let owner = created["aliases"]["owner"].clone();
    let baseline = read();
    let project: opencut_editor_core::Project =
        serde_json::from_value(baseline["project"].clone()).unwrap();
    let value = serde_json::to_value(project.find_item(leaf.as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(value["matte"]["sourceId"], created["aliases"]["provider"]);
    assert_eq!(value["effects"][1]["color"]["r"], 0.12345678901234566);
    let revision = baseline["project"]["revision"].as_u64().unwrap();
    for edit in [
        json!({"operation":"update_item","itemId":leaf,"masks":[c["defaultMask"],c["defaultMask"]]}),
        json!({"operation":"update_item","itemId":leaf,"matte":{"sourceId":"missing","channel":"alpha"}}),
        json!({"operation":"update_item","itemId":"missing","effects":[]}),
    ] {
        let before = h.project_files(id.as_str().unwrap());
        let standalone = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        assert_eq!(standalone["type"], "error");
        assert_eq!(read(), baseline);
        assert_eq!(h.project_files(id.as_str().unwrap()), before);
        let failed=event(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[{"operation":"update_item","itemId":owner,"clip":null},edit]})));
        assert_eq!(failed["type"], "error");
        assert_eq!(failed["error"]["code"], standalone["error"]["code"]);
        assert_eq!(read(), baseline);
        assert_eq!(h.project_files(id.as_str().unwrap()), before);
    }
    let before = h.project_files(id.as_str().unwrap());
    let stale=event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision-1,"edit":{"operation":"update_item","itemId":leaf,"blendMode":"multiply"}})));
    assert_eq!(stale["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(h.project_files(id.as_str().unwrap()), before);
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_item","itemId":leaf,"blendMode":"overlay"}})));
    let updated = read();
    let project: opencut_editor_core::Project =
        serde_json::from_value(updated["project"].clone()).unwrap();
    let mut expected = value;
    expected["blendMode"] = json!("overlay");
    assert_eq!(
        serde_json::to_value(project.find_item(leaf.as_str().unwrap()).unwrap()).unwrap(),
        expected
    );
    result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision+1,"operations":[{"operation":"update_item","itemId":leaf,"matte":null,"masks":[],"effects":[]},{"operation":"update_item","itemId":owner,"clip":null}]})));
    let cleared = read();
    let project: opencut_editor_core::Project =
        serde_json::from_value(cleared["project"].clone()).unwrap();
    let v = project
        .find_item(leaf.as_str().unwrap())
        .unwrap()
        .visual_properties();
    assert!(v.matte.is_none() && v.masks.is_empty() && v.effects.is_empty());
    assert!(
        project
            .find_item(owner.as_str().unwrap())
            .unwrap()
            .visual_properties()
            .clip
            .is_none()
    );
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision+2})));
    let undo = read();
    let mut expected = updated["project"].clone();
    expected["revision"] = undo["project"]["revision"].clone();
    assert!(
        undo["project"]["updatedAtMs"].as_u64().unwrap()
            >= updated["project"]["updatedAtMs"].as_u64().unwrap()
    );
    expected["updatedAtMs"] = undo["project"]["updatedAtMs"].clone();
    assert_eq!(undo["project"], expected);
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":revision+3})));
    let reopened = read();
    let mut expected = cleared["project"].clone();
    expected["revision"] = reopened["project"]["revision"].clone();
    assert!(
        reopened["project"]["updatedAtMs"].as_u64().unwrap()
            >= cleared["project"]["updatedAtMs"].as_u64().unwrap()
    );
    expected["updatedAtMs"] = reopened["project"]["updatedAtMs"].clone();
    assert_eq!(reopened["project"], expected);
    let revision = reopened["project"]["revision"].as_u64().unwrap();
    result(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"update_track","trackId":track,"locked":true}})));
    let locked = read();
    let before = h.project_files(id.as_str().unwrap());
    let rejected=event(&h.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision+1,"edit":{"operation":"update_item","itemId":leaf,"effects":[]}})));
    assert_eq!(rejected["error"]["code"], "TRACK_LOCKED");
    assert_eq!(read(), locked);
    assert_eq!(h.project_files(id.as_str().unwrap()), before);
}

#[test]
fn canonical_speech_alignment_round_trips_and_rejects_before_project_publication() {
    let h = Harness::new();
    let source = h.root.path().join("alignment_probe.rs");
    let probe = h
        .root
        .path()
        .join(format!("alignment_probe{}", std::env::consts::EXE_SUFFIX));
    // A test-only process boundary, not a production media parser. Derive the
    // descriptor independently from this suite's fixed PCM WAV header instead
    // of relying on ambient FFprobe in hermetic correctness/contract jobs.
    std::fs::write(
        &source,
        r#"
fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments.get(1).is_some_and(|argument| argument == "-version") {
        println!("test-only PCM probe");
        return;
    }
    let bytes = std::fs::read(arguments.last().unwrap()).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..16], b"WAVEfmt ");
    assert_eq!(&bytes[36..40], b"data");
    let channels = u16::from_le_bytes(bytes[22..24].try_into().unwrap());
    let sample_rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let bits = u16::from_le_bytes(bytes[34..36].try_into().unwrap());
    let data_bytes = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
    assert_eq!(u16::from_le_bytes(bytes[20..22].try_into().unwrap()), 1);
    assert_eq!(bits, 16);
    assert_eq!(bytes.len(), 44 + data_bytes as usize);
    let duration = f64::from(data_bytes)
        / (f64::from(sample_rate) * f64::from(channels) * f64::from(bits / 8));
    println!("{{\"streams\":[{{\"codec_type\":\"audio\",\"codec_name\":\"pcm_s16le\",\"sample_rate\":\"{sample_rate}\",\"channels\":{channels}}}],\"format\":{{\"duration\":\"{duration:.6}\",\"format_name\":\"wav\"}}}}");
}
"#,
    )
    .unwrap();
    let compiled = Command::new("rustc")
        .arg("--edition=2024")
        .arg(&source)
        .arg("-o")
        .arg(&probe)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "cannot compile hermetic PCM probe: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    run_speech_alignment_protocol_cases(&h, &probe);
}

#[test]
fn native_canonical_speech_alignment_round_trips_and_rejects_before_project_publication() {
    if !native_parity_is_configured() {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1"),
            "required native alignment parity needs configured FFprobe"
        );
        return;
    }
    let probe = std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap();
    run_speech_alignment_protocol_cases(&Harness::new(), std::path::Path::new(&probe));
}

fn run_speech_alignment_protocol_cases(h: &Harness, probe: &std::path::Path) {
    let created = result(&h.request(json!({"operation":"create_project","name":"Aligned speech"})));
    let id = created["projectId"].as_str().unwrap();
    let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let track = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|track| track["trackType"] == "audio")
        .unwrap()["id"]
        .clone();
    let generated = h.root.path().join("generated");
    std::fs::create_dir(&generated).unwrap();
    let wav = generated.join("speech.wav");
    // One second, mono 16-bit PCM at 24 kHz. This is independent test media,
    // Both probe modes derive the duration from this independent media.
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend(48_036_u32.to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(24_000_u32.to_le_bytes());
    bytes.extend(48_000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(48_000_u32.to_le_bytes());
    bytes.resize(48_044, 0);
    std::fs::write(&wav, bytes).unwrap();
    let send = |request: Value| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_opencut-headless"));
        command
            .env("OPENCUT_PROJECTS_DIR", h.root.path().join("projects"))
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", h.root.path().join("media"))
            .env("OPENCUT_EXPORTS_DIR", h.root.path().join("exports"))
            .env("OPENCUT_GENERATED_MEDIA_DIRS", &generated)
            .env_remove("OPENCUT_DEFAULT_FONT_PATH")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in ["OPENCUT_FFMPEG_PATH", "OPENCUT_FFPROBE_PATH"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command.env("OPENCUT_FFPROBE_PATH", probe);
        let mut child = command.spawn().unwrap();
        std::io::Write::write_all(
            &mut child.stdin.take().unwrap(),
            serde_json::to_string(&request).unwrap().as_bytes(),
        )
        .unwrap();
        child.wait_with_output().unwrap()
    };
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/speech-alignment-v1.json")).unwrap();
    let request = |revision: u64, alignment: &Value| {
        json!({
            "operation":"commit_generated_asset","projectId":id,"expectedRevision":revision,
            "path":wav,"trackId":track,"startMs":0,"displayName":"speech",
            "origin":{"type":"speech_synthesis","generation":{
                "request":{"text":"Hello","language":"en","voiceId":"af_heart","speed":1},
                "providerId":"synthesis-provider","modelId":"synthesis-model","modelVersion":null,
                "sampleRateHz":24000,"generatedAtMs":1,"alignment":alignment}}
        })
    };
    let mut revision = 0;
    for case in catalog["valid"].as_array().unwrap() {
        let response = result(&send(request(revision, &case["alignment"])));
        revision += 1;
        assert_eq!(response["revision"], revision, "{}", case["id"]);
        let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let asset = state["project"]["assets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|asset| asset["id"] == response["assetId"])
            .unwrap();
        assert_eq!(
            asset["origin"]["generation"]["alignment"],
            case["alignment"]
        );
        assert_eq!(asset["durationMs"], 1000);
        assert_eq!(asset["probe"]["audioSampleRateHz"], 24000);
        assert_eq!(catalog["projectSchemaVersion"], 38);
        assert_eq!(
            state["project"]["schemaVersion"],
            opencut_editor_core::PROJECT_SCHEMA_VERSION
        );
        let before = h.project_files(id);
        let mut forced = case["alignment"].clone();
        forced["quality"] = json!("forced");
        let validate = json!({"operation":"validate_speech_alignment", "projectId":id,
            "assetId":response["assetId"],"knownText":"Hello", "expectedRevision":revision,
            "alignment":forced});
        assert_eq!(result(&h.request(validate.clone()))["revision"], revision);
        assert_eq!(h.project_files(id), before);
        for (field, value, code) in [
            ("knownText", json!(" \n"), "VALIDATION_FAILED"),
            ("knownText", json!("é".repeat(2049)), "VALIDATION_FAILED"),
            ("expectedRevision", json!(revision - 1), "REVISION_CONFLICT"),
            ("assetId", json!("missing"), "ASSET_NOT_FOUND"),
            ("alignment", Value::Null, "INVALID_ARGUMENT"),
        ] {
            let mut rejected = validate.clone();
            rejected[field] = value;
            assert_eq!(event(&h.request(rejected))["error"]["code"], code);
            assert_eq!(h.project_files(id), before);
        }
    }
    let aligned_state = result(&h.request(json!({"operation":"get_state","projectId":id})));
    let aligned_asset = aligned_state["project"]["assets"][0]["id"].clone();
    for case in catalog["invalid"].as_array().unwrap() {
        let before = h.project_files(id);
        let failure = event(&send(request(revision, &case["alignment"])));
        assert_eq!(failure["type"], "error", "{}", case["id"]);
        assert_eq!(failure["error"]["retryable"], false, "{}", case["id"]);
        assert_eq!(h.project_files(id), before, "{}", case["id"]);
        let mut invalid_alignment = case["alignment"].clone();
        if matches!(
            invalid_alignment["quality"].as_str(),
            Some("native" | "estimated" | "forced")
        ) {
            invalid_alignment["quality"] = json!("forced");
        }
        let validation = event(&h.request(json!({"operation":"validate_speech_alignment",
            "projectId":id,"assetId":aligned_asset,"knownText":"Hello", "expectedRevision":revision,
            "alignment":invalid_alignment})));
        assert_eq!(validation["type"], "error", "{}", case["id"]);
        assert_eq!(validation["error"]["retryable"], false, "{}", case["id"]);
        assert_eq!(h.project_files(id), before, "{}", case["id"]);
    }
    // Issue 62 consumes the same independent PCM source in both hermetic and native probe modes.
    let markers: Value = serde_json::from_str(include_str!(
        "../../../contracts/speech-alignment-markers-v1.json"
    ))
    .unwrap();
    for policy in markers["policies"].as_array().unwrap() {
        let before = h.project_files(id);
        let edit = json!({"operation":"speech_markers_generate","scope":"root","assetId":aligned_asset,
            "startMs":1000,"markerPolicy":policy,"alignment":markers["alignment"]});
        let stale = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision-1,"edit":edit}),
        ));
        assert_eq!(stale["error"]["code"], "REVISION_CONFLICT");
        assert_eq!(h.project_files(id), before);
        let response = result(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit}),
        ));
        revision += 1;
        assert_eq!(response["revision"], revision);
        let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let actual: Vec<Value> = state["project"]["markers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|marker| json!({"name":marker["name"],"timeMs":marker["timeMs"]}))
            .collect();
        assert_eq!(
            json!(actual),
            markers["expected"][policy["type"].as_str().unwrap()]
        );
        result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":revision})));
        revision += 1;
    }
    let before = h.project_files(id);
    for policy in markers["invalidPolicies"].as_array().unwrap() {
        let failure = event(&h.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,
            "edit":{"operation":"speech_markers_generate","scope":"root","assetId":aligned_asset,
                "startMs":1000,"markerPolicy":policy,"alignment":markers["alignment"]}}),
        ));
        assert_eq!(failure["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(h.project_files(id), before);
    }

    if native_parity_is_configured() {
        let state = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let visual_track = state["project"]["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|track| track["trackType"] == "overlay")
            .unwrap()["id"]
            .clone();
        let bound = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[
            {"operation":"speech_markers_generate","scope":"root","assetId":aligned_asset,"startMs":1000,"markerPolicy":{"type":"selected_word","indices":[0]},"alignment":markers["alignment"],"resultAlias":"cue"},
            {"operation":"add_rectangle","trackId":visual_track,"startMs":0,"durationMs":200,"width":1920,"height":1080,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"visual"},
            {"operation":"set_item_start_time","scope":"root","itemId":"@visual","time":{"type":"marker","markerName":"EVERY","offsetMs":0}}
        ]})));
        revision += 1;
        let current = result(&h.request(json!({"operation":"get_state","projectId":id})));
        let item = current["project"]["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|track| track["items"].as_array().unwrap())
            .find(|item| item["id"] == bound["aliases"]["visual"])
            .unwrap();
        assert_eq!(item["startMs"], 1100);
        let render_result = |output: &Output| {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
            let events: Vec<Value> = serde_json::Deserializer::from_slice(&output.stdout)
                .into_iter::<Value>()
                .map(Result::unwrap)
                .collect();
            let final_event = events.last().expect("render result event");
            assert_eq!(final_event["type"], "result");
            assert!(
                events
                    .iter()
                    .all(|event| event["type"] == "progress" || event["type"] == "result")
            );
            final_event["result"].clone()
        };
        let range = render_result(&h.request(
            json!({"operation":"render_preview_range","projectId":id,"expectedRevision":revision,
            "startMs":900,"endMs":1300,"width":64,"height":64,"fps":10,"includeAudio":false}),
        ));
        let export = render_result(&h.request(
            json!({"operation":"export_video","projectId":id,"expectedRevision":revision,
            "relativePath":"speech-markers.mp4","width":64,"height":64,"overwrite":false}),
        ));
        let range_path = h
            .root
            .path()
            .join("projects")
            .join(id)
            .join(range["relativePath"].as_str().unwrap());
        let export_path = h
            .root
            .path()
            .join("exports")
            .join(export["relativePath"].as_str().unwrap());
        let pixel = |path: &std::path::Path, time: &str| {
            let output = Command::new(std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap())
                .args(["-v", "error", "-ss", time, "-i"])
                .arg(path)
                .args([
                    "-frames:v",
                    "1",
                    "-vf",
                    "format=rgb24,crop=1:1:32:32",
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgb24",
                    "pipe:1",
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(output.stdout.len(), 3);
            output.stdout
        };
        for (range_time, export_time, red) in [("0", "0.9", false), ("0.3", "1.2", true)] {
            for observed in [
                pixel(&range_path, range_time),
                pixel(&export_path, export_time),
            ] {
                if red {
                    assert!(
                        observed[0] > 200 && observed[1] < 30 && observed[2] < 30,
                        "{observed:?}"
                    );
                } else {
                    assert!(observed.iter().all(|channel| *channel < 30), "{observed:?}");
                }
            }
        }
    }
}

#[test]
fn timeline_audio_events_typed_marker_alias_draft_rollback_history_and_fresh_process_reopen() {
    let h = Harness::new();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/timeline-audio-events-v1.json"
    ))
    .unwrap();
    let id = result(&h.request(json!({"operation":"create_project","name":"Semantic audio"})))["projectId"].as_str().unwrap().to_owned();
    let path = h.root.path().join("media/semantic.wav");
    std::fs::write(&path, b"immutable typed transport source").unwrap();
    let core = opencut_editor_core::EditorCore::new(
        opencut_editor_core::PathPolicy::new(
            h.root.path().join("projects"),
            [h.root.path().join("media")],
            h.root.path().join("exports"),
        )
        .unwrap(),
    );
    let asset = core
        .import_asset(
            &id,
            0,
            path,
            opencut_editor_core::MediaType::Audio,
            opencut_editor_core::MediaProbeFacts {
                duration_ms: Some(500),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let batch = result(&h.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
        {"operation":"sound_event_register","event":"impact","variantAssetIds":[asset],"defaultGainDb":-6,"busId":"sfx","variantSeed":0,"resultAlias":"sound"},
        {"operation":"create_track","name":"Events","trackType":"audio","resultAlias":"events"},
        {"operation":"marker_create","scope":"root","name":"impact","timeMs":300,"kind":"cue","resultAlias":"cue"},
        {"operation":"timeline_add_audio_event","scope":"root","trackId":"@events","event":"@sound","at":{"type":"marker","markerName":"impact","offsetMs":-50},"durationMs":300,"gainDb":-3,"resultAlias":"placed"}
    ]})));
    assert_eq!(batch["revision"], 2);
    let read = || result(&h.request(json!({"operation":"get_state","projectId":id})));
    let state = read();
    assert_eq!(
        state["project"]["schemaVersion"],
        catalog["projectSchemaVersion"]
    );
    let track = batch["aliases"]["events"].clone();
    let item = |state: &Value| {
        state["project"]["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == track)
            .unwrap()["items"][0]
            .clone()
    };
    assert_eq!(item(&state)["startMs"], 250);
    assert_eq!(item(&state)["audioEvent"]["defaultGainDb"], -6.0);
    assert_eq!(item(&state)["audioEvent"]["gainDb"], -3.0);
    let before = h.project_files(&id);
    for request in [
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":catalog["input"]}),
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":2,"operations":[{"operation":"timeline_add_audio_event","scope":"root","trackId":track,"event":"missing","at":{"type":"milliseconds","valueMs":0}}]}),
    ] {
        assert_eq!(event(&h.request(request))["type"], "error");
        assert_eq!(h.project_files(&id), before);
    }
    let draft = result(&h.request(json!({"operation":"create_draft","projectId":id,"expectedRevision":2,"operations":[
        {"operation":"timeline_add_audio_event","scope":"root","trackId":track,"event":"impact","at":{"type":"milliseconds","valueMs":600}}
    ]})));
    assert_eq!(draft["audioEventAssetIds"], json!([asset]));
    assert_eq!(read()["project"], state["project"]);
    result(&h.request(json!({"operation":"commit_draft","projectId":id,"draftId":draft["id"],"expectedRevision":2})));
    result(&h.request(json!({"operation":"undo","projectId":id,"expectedRevision":3})));
    assert_eq!(item(&read()), item(&state));
    result(&h.request(json!({"operation":"redo","projectId":id,"expectedRevision":4})));
    let reopened = result(&h.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(reopened["project"], read()["project"]);
    assert_eq!(reopened["project"]["revision"], 5);
}
