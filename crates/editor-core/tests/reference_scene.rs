#[path = "support/reference_scene.rs"]
mod fixture;
use fixture::{recipe, seed};
use opencut_editor_core::{BatchEditOperation, EditorCore, ErrorCode, PathPolicy};
use serde_json::{Value, json};
fn content(p: &opencut_editor_core::Project) -> Value {
    let mut v = serde_json::to_value(p).unwrap();
    v.as_object_mut().unwrap().remove("revision");
    v.as_object_mut().unwrap().remove("updatedAtMs");
    v
}
fn files(f: &fixture::Fixture) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        path: &std::path::Path,
        result: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if [".lock", "previews", "analyses"]
                .iter()
                .any(|name| path.file_name().unwrap() == *name)
            {
                continue;
            }
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(&f.dir(), &f.dir(), &mut result);
    result
}
// Independent acceptance witnesses: deliberately not derived from the recipe's
// own witnessAliases, so deleting both a feature and its declaration cannot pass.
fn complete_recipe(c: &Value) -> bool {
    let Some(operations) = c["operations"].as_array() else {
        return false;
    };
    let Some(groups) = c["capabilityGroups"].as_array() else {
        return false;
    };
    if groups.iter().map(|g| g["id"].as_str()).collect::<Vec<_>>()
        != [
            "compositions",
            "vectors",
            "typography",
            "animation",
            "compositing",
            "stacking",
            "atomic",
            "narration",
            "audio",
            "review",
        ]
        .map(Some)
    {
        return false;
    }
    let aliases: [&[&str]; 10] = [
        &["card", "parent", "card0", "card1", "card2"],
        &["grid", "decoration", "decorative-copies"],
        &["word0", "word1", "word2"],
        &["grid", "card0", "radar", "word0"],
        &["hero-owner", "hero-provider", "hero-hero"],
        &["grid", "parent", "word0"],
        &["card", "parent", "card0"],
        &["word0", "word1", "word2"],
        &["music", "events", "event0", "event5"],
        &[], // Artifact/cache/export witnesses are executed by the native harness.
    ];
    if groups
        .iter()
        .zip(aliases)
        .any(|(group, expected)| group["witnessAliases"] != json!(expected))
    {
        return false;
    }
    let required = [
        "component_create",
        "component_define_slots",
        "add_group",
        "add_grid",
        "add_shape",
        "add_repeater",
        "add_text",
        "apply_animation_preset",
        "item_set_z_index",
        "speech_markers_generate",
        "sound_event_register",
        "timeline_add_audio_event",
        "audio_bus_set_ducking",
        "audio_bus_set_dsp",
        "audio_master_set_normalization",
        "set_animation_channels",
    ];
    if required
        .iter()
        .any(|name| !operations.iter().any(|o| o["operation"] == *name))
    {
        return false;
    }
    if !operations.iter().any(|o| {
        o["operation"] == "update_item"
            && o["itemId"] == "@hero-hero"
            && o["blendMode"] == "screen"
            && o["masks"].as_array().is_some_and(|m| m.len() == 1)
            && o["matte"]["sourceId"] == "@hero-provider"
            && o["effects"].as_array().is_some_and(|effects| {
                effects
                    .iter()
                    .map(|e| e["type"].as_str())
                    .collect::<Vec<_>>()
                    == [Some("gaussian_blur"), Some("glow"), Some("color_tint")]
            })
    }) {
        return false;
    }
    for (i, title) in ["Plan", "Build", "Check"].iter().enumerate() {
        let alias = format!("card{i}");
        if !operations.iter().any(|o| {
            o["resultAlias"] == alias
                && o["operation"] == "add_component_instance"
                && o["componentId"] == "@card"
                && o["slotValues"]["title"]["value"] == *title
                && o["parent"]["id"] == "@parent"
        }) {
            return false;
        }
    }
    for (i, (name, time)) in [
        ("EVERY", 500),
        ("SINGLE", 1000),
        ("ONE", 1500),
        ("rules", 2400),
        ("Starting_with", 3200),
        ("Venusaur", 4300),
    ]
    .iter()
    .enumerate()
    {
        if c["cues"][i]["name"] != *name || c["cues"][i]["timeMs"] != *time {
            return false;
        }
        let alias = format!("event{i}");
        if !operations.iter().any(|o| {
            o["resultAlias"] == alias
                && o["operation"] == "timeline_add_audio_event"
                && o["at"]["markerName"] == *name
        }) {
            return false;
        }
    }
    true
}
#[test]
fn complete_reference_coverage_rejects_removed_groups_operations_overrides_and_bindings() {
    let c = recipe();
    assert!(complete_recipe(&c));
    for i in 0..10 {
        let mut broken = c.clone();
        broken["capabilityGroups"].as_array_mut().unwrap().remove(i);
        assert!(!complete_recipe(&broken), "missing group {i}");
    }
    for i in 0..9 {
        let mut broken = c.clone();
        broken["capabilityGroups"][i]["witnessAliases"]
            .as_array_mut()
            .unwrap()
            .remove(0);
        assert!(!complete_recipe(&broken), "missing witness {i}");
    }
    for name in [
        "component_create",
        "add_grid",
        "add_repeater",
        "add_text",
        "apply_animation_preset",
        "set_animation_channels",
        "speech_markers_generate",
        "audio_bus_set_ducking",
        "audio_master_set_normalization",
    ] {
        let mut broken = c.clone();
        broken["operations"]
            .as_array_mut()
            .unwrap()
            .retain(|o| o["operation"] != name);
        assert!(!complete_recipe(&broken), "missing operation {name}");
    }
    for alias in ["card0", "card1", "card2", "event0", "event5"] {
        let mut broken = c.clone();
        let operation = broken["operations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|o| o["resultAlias"] == alias)
            .unwrap();
        if alias.starts_with("card") {
            operation["slotValues"]["title"]["value"] = json!("Incomplete");
        } else {
            operation["at"]["markerName"] = json!("wrong");
        }
        assert!(!complete_recipe(&broken), "changed {alias}");
    }
    let mut broken = c;
    broken["cues"][0]["timeMs"] = json!(600);
    assert!(!complete_recipe(&broken));
    broken = recipe();
    broken["operations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|o| o["itemId"] == "@hero-hero" && o["operation"] == "update_item")
        .unwrap()["blendMode"] = json!("normal");
    assert!(!complete_recipe(&broken));
}
fn witnesses(f: &fixture::Fixture) {
    let p = f.project();
    let c = recipe();
    assert_eq!(p.schema_version, 44);
    assert_eq!(p.duration_ms(), 6000);
    assert_eq!(p.components.len(), 1);
    assert_eq!(p.markers.len(), 6);
    assert_eq!(
        c["capabilityGroups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "compositions",
            "vectors",
            "typography",
            "animation",
            "compositing",
            "stacking",
            "atomic",
            "narration",
            "audio",
            "review"
        ]
    );
    for group in c["capabilityGroups"].as_array().unwrap() {
        for alias in group["witnessAliases"].as_array().unwrap() {
            assert!(f.aliases.contains_key(alias.as_str().unwrap()));
        }
    }
    assert_eq!(p.components[0].tracks[0].items.len(), 6);
    let items = serde_json::to_value(&p.tracks).unwrap();
    let find = |alias: &str| {
        items
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|t| t["items"].as_array().unwrap())
            .find(|x| x["id"] == f.aliases[alias])
            .unwrap()
    };
    for (i, title) in ["Plan", "Build", "Check"].iter().enumerate() {
        let item = find(&format!("card{i}"));
        assert_eq!(item["componentId"], f.aliases["card"]);
        assert_eq!(item["slotValues"]["title"]["value"], *title);
        assert_eq!(item["slotValues"]["number"]["value"], (i + 1).to_string());
        assert_eq!(item["parent"]["id"], f.aliases["parent"]);
        assert_eq!(
            item["animationPresetProvenance"]["transform.position_x"]["presetId"],
            "slide_left"
        );
    }
    assert_eq!(find("grid")["zIndex"], -10);
    assert_eq!(find("grid")["grid"]["pattern"]["type"], "diagonal");
    assert_eq!(
        find("grid")["animationChannels"][0]["loop"]["iterations"],
        "infinite"
    );
    assert_eq!(find("decorative-copies")["repeater"]["copies"], 3);
    assert_eq!(
        find("word0")["animationPresetProvenance"]["transform.scale_x"]["presetId"],
        "impact_slam"
    );
    assert_eq!(find("word0")["startMs"], 500);
    assert_eq!(find("hero-hero")["startMs"], 4300);
    assert_eq!(find("hero-hero")["blendMode"], "screen");
    assert_eq!(
        find("hero-hero")["matte"]["sourceId"],
        f.aliases["hero-provider"]
    );
    for (i, cue) in c["cues"].as_array().unwrap().iter().enumerate() {
        assert_eq!(p.markers[i].name, cue["name"].as_str().unwrap());
        assert_eq!(p.markers[i].time_ms, cue["timeMs"].as_u64().unwrap());
        assert_eq!(find(&format!("event{i}"))["startMs"], cue["timeMs"]);
        assert_eq!(find(&format!("event{i}"))["audioEvent"]["busId"], "sfx");
    }
    assert!(
        p.audio_buses
            .iter()
            .find(|b| b.id == "music")
            .unwrap()
            .ducking
            .is_some()
    );
    assert!(
        p.audio_buses
            .iter()
            .find(|b| b.id == "master")
            .unwrap()
            .dsp
            .as_ref()
            .unwrap()
            .compressor
            .is_some()
    );
    assert!(p.master_normalization.as_ref().unwrap().enabled);
}
#[test]
fn complete_reference_standalone_and_atomic_authoring_witnesses() {
    for batch in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let f = seed(root.path(), batch);
        witnesses(&f);
    }
}
#[test]
fn complete_reference_failures_history_and_reopen_preserve_all_content() {
    let root = tempfile::tempdir().unwrap();
    let f = seed(root.path(), true);
    witnesses(&f);
    let initial = f.project();
    let before = files(&f);
    for (expected, operation) in [
        (
            ErrorCode::ItemNotFound,
            json!({"operation":"item_set_z_index","itemId":"missing","zIndex":1}),
        ),
        (
            ErrorCode::InvalidArgument,
            json!({"operation":"component_instance_duplicate","itemId":f.aliases["card0"],"offsetMs":0,"slotValues":{"title":{"type":"number","value":1}}}),
        ),
    ] {
        let error = f
            .core
            .edit(
                &f.id,
                initial.revision,
                serde_json::from_value(operation).unwrap(),
            )
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!error.retryable);
        assert_eq!(files(&f), before);
    }
    assert_eq!(
        f.core
            .edit(
                &f.id,
                initial.revision - 1,
                serde_json::from_value(
                    json!({"operation":"item_set_z_index","itemId":f.aliases["grid"],"zIndex":2})
                )
                .unwrap()
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(files(&f), before);
    let bad:Vec<BatchEditOperation>=serde_json::from_value(json!([{"operation":"item_set_z_index","itemId":f.aliases["grid"],"zIndex":2},{"operation":"delete_item","itemId":"missing"}])).unwrap();
    assert!(f.core.edit_batch(&f.id, initial.revision, bad).is_err());
    assert_eq!(files(&f), before);
    let operations: Vec<BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"component_instance_duplicate","itemId":f.aliases["card0"],"offsetMs":0,"slotValues":{"title":{"type":"text","value":"Inspect"}}},
        {"operation":"update_item","itemId":f.aliases["parent"],"staggerMs":120}
    ])).unwrap();
    f.core
        .edit_batch(&f.id, initial.revision, operations)
        .unwrap();
    let edited = f.project();
    assert_eq!(edited.revision, initial.revision + 1);
    assert_eq!(
        serde_json::to_value(&edited.components[0]).unwrap(),
        serde_json::to_value(&initial.components[0]).unwrap()
    );
    f.core.undo(&f.id, edited.revision).unwrap();
    assert_eq!(f.project().revision, initial.revision + 2);
    assert!(f.project().updated_at_ms >= edited.updated_at_ms);
    assert_eq!(content(&f.project()), content(&initial));
    f.core.redo(&f.id, f.project().revision).unwrap();
    assert_eq!(f.project().revision, initial.revision + 3);
    assert!(f.project().updated_at_ms >= edited.updated_at_ms);
    assert_eq!(content(&f.project()), content(&edited));
    let fresh = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path().join("media")],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    assert_eq!(
        serde_json::to_value(fresh.get_project(&f.id).unwrap()).unwrap(),
        serde_json::to_value(f.project()).unwrap()
    );
}
