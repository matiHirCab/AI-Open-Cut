//! Canonical authored DTO comparison only; no scene evaluator or raster/output oracle.
use opencut_editor_core::{Project, TimelineItem};
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub fn assert_recipe(p: &Project, ids: &BTreeMap<String, String>, c: &Value, reversed: bool) {
    assert_eq!(p.duration_ms(), 800);
    assert_eq!(
        p.schema_version,
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(p.name, "Masked hero reveal v1");
    assert_eq!(serde_json::to_value(&p.settings).unwrap(), c["settings"]);
    assert_eq!(p.tracks.len(), 4);
    assert_eq!(p.tracks[0].items.len(), 0);
    assert_eq!(p.tracks[1].items.len(), 4);
    assert_eq!(p.tracks[2].items.len(), 1);
    assert_eq!(p.tracks[3].items.len(), 0);
    for (index, role) in ["owner", "provider", "probe", "hero"]
        .into_iter()
        .enumerate()
    {
        let recipe = &c["roles"][role];
        let mut expected = recipe.as_object().unwrap().clone();
        for key in ["kind", "parentRole", "parent"] {
            expected.remove(key);
        }
        expected.insert("type".into(), recipe["kind"].clone());
        expected.insert("id".into(), json!(ids[role]));
        expected.insert("startMs".into(), json!(0));
        expected.insert("durationMs".into(), json!(800));
        expected.insert("stackOrder".into(), json!(index));
        expected.insert(
            "transform".into(),
            json!({"positionX":0.0,"positionY":0.0,"scale":1.0,"opacity":1.0}),
        );
        expected.insert("hidden".into(), json!(false));
        if let Some(parent) = recipe["parentRole"].as_str() {
            expected.insert("parent".into(), json!({"scope":"root","id":ids[parent]}));
        }
        if role == "hero" {
            expected.insert(
                "matte".into(),
                json!({"sourceId":ids["provider"],"channel":"alpha"}),
            );
            if reversed {
                expected
                    .get_mut("effects")
                    .unwrap()
                    .as_array_mut()
                    .unwrap()
                    .swap(1, 2);
            }
        }
        if recipe["kind"] == "shape" {
            expected.insert("keyframes".into(), json!([]));
        }
        let expected: TimelineItem = serde_json::from_value(Value::Object(expected)).unwrap();
        assert_eq!(p.tracks[1].items[index].id(), ids[role]);
        assert_eq!(
            serde_json::to_value(&p.tracks[1].items[index]).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "complete canonical {role}"
        );
    }
    assert_eq!(p.assets.len(), 1);
    let asset = serde_json::to_value(&p.assets[0]).unwrap();
    assert_eq!(asset["fileName"], "source.wav");
    assert_eq!(asset["mediaType"], "audio");
    assert_eq!(asset["durationMs"], 800);
    assert_eq!(asset["sizeBytes"], 153644);
    assert_eq!(
        asset["contentHash"],
        json!({"algorithm":"sha256","digest":c["audio"]["sourceSha256"]})
    );
    assert_eq!(
        asset["projectRelativePath"],
        format!(
            "assets/sha256/b6/{}",
            c["audio"]["sourceSha256"].as_str().unwrap()
        )
    );
    assert_eq!(
        asset["probe"],
        json!({"durationMs":800,"hasAudio":true,"hasVideo":false,"formatName":"wav","videoCodec":null,"videoWidth":null,"videoHeight":null,"audioCodec":"pcm_s16le","audioChannels":2,"audioSampleRateHz":48000})
    );
    assert_eq!(asset["hasAudio"], true);
    assert!(asset["origin"].is_null());
    let expected:TimelineItem=serde_json::from_value(json!({"type":"media","id":ids["audio"],"assetId":asset["id"],"startMs":0,"durationMs":800,"sourceInMs":0,"zIndex":0,"stackOrder":0,"transform":{"positionX":0.0,"positionY":0.0,"scale":1.0,"opacity":1.0},"hidden":false,"audio":{"volume":1.0,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]})).unwrap();
    assert_eq!(
        serde_json::to_value(&p.tracks[2].items[0]).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
    let tracks = serde_json::to_value(&p.tracks).unwrap();
    assert_eq!(tracks[1]["name"], "Overlay");
    assert_eq!(tracks[1]["trackType"], "overlay");
    assert_eq!(tracks[2]["name"], "Audio");
    assert_eq!(tracks[2]["trackType"], "audio");
    assert_eq!(tracks[2]["audioRole"], "unassigned");
    assert!(p.tracks[2].ducking.is_none());
    for t in tracks.as_array().unwrap() {
        assert_eq!(t["locked"], false);
        assert_eq!(t["hidden"], false);
        assert_eq!(t["muted"], false);
    }
}
