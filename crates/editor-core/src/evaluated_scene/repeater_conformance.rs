use super::*;
use serde_json::{Value, json};

fn track(items: Value) -> Value {
    json!({"id":"track","name":"Track","trackType":"overlay","items":items})
}
fn instance(id: &str, definition: &str, order: usize) -> Value {
    json!({"type":"component_instance","id":id,"componentId":definition,"startMs":0,
        "durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{},"zIndex":0,"stackOrder":order})
}
fn repeat(source: &str, scope: &str, copies: u16, order: usize) -> Value {
    json!({"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":order,
        "repeater":{"source":{"scope":scope,"id":source},"copies":copies,
        "transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
        "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}})
}
fn rectangle(id: &str, order: usize) -> Value {
    json!({"type":"rectangle","id":id,"startMs":0,"durationMs":1000,"width":10,"height":10,
        "color":"#ff0000","keyframes":[],"zIndex":0,"stackOrder":order})
}
fn transition(id: &str, target: &str, order: usize) -> Value {
    json!({"type":"transition","id":id,"transitionType":"fade","fromItemId":target,
        "startMs":0,"durationMs":10,"zIndex":0,"stackOrder":order})
}
fn definition(id: &str, items: Value) -> Value {
    json!({"id":id,"name":id,"width":100,"height":100,"durationMs":1000,"slots":[],"tracks":[track(items)]})
}
fn project_value(items: Value, definitions: Value) -> Value {
    json!({"schemaVersion":17,"id":"audit","revision":0,"name":"Audit","createdAtMs":1,"updatedAtMs":1,
        "settings":{"width":100,"height":100,"fps":30},"assets":[],"tracks":[track(items)],"components":definitions})
}
pub(crate) fn transition_fixture(facts: usize, copies: u16, extra: bool) -> Project {
    let mut items = vec![rectangle("rect", 0)];
    for n in 0..facts {
        items.push(transition(&format!("fade{n}"), "rect", n + 1));
    }
    let mut root = vec![
        instance("instance", "leaf", 0),
        repeat("instance", "root", copies, 1),
    ];
    if extra {
        root.extend([rectangle("extra", 2), transition("extra-fade", "extra", 3)]);
    }
    serde_json::from_value(project_value(
        json!(root),
        json!([definition("leaf", json!(items))]),
    ))
    .unwrap()
}
fn reject(p: &Project, message: &str) {
    GENERATED_MATERIALIZATIONS.with(|n| n.set(0));
    let error = match evaluate_project(p, 100, 100, 30) {
        Err(error) => error,
        Ok(_) => panic!("invalid project was accepted"),
    };
    assert_eq!(error.code, ErrorCode::InvalidArgument, "{error:?}");
    assert!(error.message.contains(message), "{error:?}");
    GENERATED_MATERIALIZATIONS.with(|n| assert_eq!(n.get(), 0));
}

#[test]
fn signed_copy_offsets_shift_complete_component_source_clock() {
    for (offset, expected) in [
        (100, [(200.0, 300.0), (300.0, 400.0), (400.0, 500.0)]),
        (-100, [(200.0, 300.0), (100.0, 200.0), (0.0, 100.0)]),
    ] {
        let mut child = rectangle("rectangle", 0);
        child["startMs"] = json!(200);
        child["durationMs"] = json!(100);
        let mut copies = repeat("instance", "root", 2, 1);
        copies["repeater"]["timeOffsetMs"] = json!(offset);
        let mut value = project_value(
            json!([instance("instance", "leaf", 0), copies]),
            json!([definition("leaf", json!([child]))]),
        );
        value["schemaVersion"] = json!(26);
        value.as_object_mut().unwrap().remove("audioBuses");
        value.as_object_mut().unwrap().remove("soundDefinitions");
        value["components"][0]["markers"] = json!([]);
        value["fonts"] = json!({});
        value["markers"] = json!([]);
        let project: Project = serde_json::from_value(value).unwrap();
        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        assert_eq!(scene.visual_layers.len(), 3);
        let ordinary = scene
            .visual_layers
            .iter()
            .find(|l| !l.item_id.starts_with("repeater:"))
            .unwrap();
        let mut generated = scene
            .visual_layers
            .iter()
            .filter(|l| l.item_id.starts_with("repeater:"))
            .collect::<Vec<_>>();
        generated.sort_by(|a, b| a.item_id.cmp(&b.item_id));
        for (layer, (start, end)) in [ordinary, generated[0], generated[1]]
            .into_iter()
            .zip(expected)
        {
            let clock = layer.instance.unwrap();
            assert_eq!((clock.start_ms, clock.end_ms), (start, end));
        }
        assert_eq!(ordinary.instance.unwrap().offset, 0.0);
        assert_eq!(generated[0].instance.unwrap().offset, -(offset as f64));
    }
}

#[test]
fn shifted_sources_can_enter_ancestor_clips_but_cannot_escape_them() {
    for (child_start, offset, expected) in [
        (300, -100, vec![(true, 200.0, 250.0)]),
        (200, 100, vec![(false, 200.0, 250.0)]),
    ] {
        let parent = json!({"type":"group","id":"parent","startMs":0,"durationMs":250,"zIndex":0,"stackOrder":0});
        let mut source = instance("instance", "leaf", 1);
        source["parent"] = json!({"scope":"root","id":"parent"});
        let mut child = rectangle("rectangle", 0);
        child["startMs"] = json!(child_start);
        child["durationMs"] = json!(100);
        let mut copies = repeat("instance", "root", 1, 2);
        copies["repeater"]["timeOffsetMs"] = json!(offset);
        let mut value = project_value(
            json!([parent, source, copies]),
            json!([definition("leaf", json!([child]))]),
        );
        value["schemaVersion"] = json!(26);
        value.as_object_mut().unwrap().remove("audioBuses");
        value.as_object_mut().unwrap().remove("soundDefinitions");
        value["components"][0]["markers"] = json!([]);
        value["fonts"] = json!({});
        value["markers"] = json!([]);
        let project: Project = serde_json::from_value(value).unwrap();
        let scene = evaluate_project(&project, 100, 100, 30).unwrap().scene;
        let actual = scene
            .visual_layers
            .iter()
            .map(|layer| {
                let clock = layer.instance.unwrap();
                (
                    layer.item_id.starts_with("repeater:"),
                    clock.start_ms,
                    clock.end_ms,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn projected_transition_budgets_include_copies_and_retained_roles() {
    reject(&transition_fixture(17, 256, false), "transition");
    for mode in [
        "root",
        "track",
        "repeater",
        "instance",
        "clipped",
        "transition",
        "transition-track",
        "self",
        "local",
        "unused",
        "nested",
    ] {
        for extra in [false, true] {
            let mut p = transition_fixture(16, 255, extra);
            match mode {
                "track" => p.tracks[0].hidden = true,
                "repeater" => p.tracks[0].items[1].visual_properties_mut().hidden = true,
                "instance" => p.tracks[0].items[0].visual_properties_mut().hidden = true,
                "clipped" => {
                    p.components[0].duration_ms = 2000;
                    if let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] {
                        i.trim_start_ms = 1000;
                    }
                }
                "transition" => {
                    for item in &mut p.components[0].tracks[0].items[1..] {
                        item.visual_properties_mut().hidden = true;
                    }
                }
                "transition-track" => {
                    let transitions = p.components[0].tracks[0].items.split_off(1);
                    let mut hidden = p.components[0].tracks[0].clone();
                    hidden.id = "hidden".into();
                    hidden.hidden = true;
                    hidden.items = transitions;
                    for (n, item) in hidden.items.iter_mut().enumerate() {
                        item.visual_properties_mut().stack_order = n as u32;
                    }
                    p.components[0].tracks.push(hidden);
                }
                "self" => {
                    p.components[0].tracks[0].items.truncate(9);
                    for item in &mut p.components[0].tracks[0].items[1..] {
                        if let TimelineItem::Transition(t) = item {
                            t.to_item_id = Some("rect".into());
                        }
                    }
                }
                "local" | "unused" => {
                    if let TimelineItem::Repeater(r) = &mut p.tracks[0].items[1] {
                        r.repeater.source.scope = "component:wrapper".into();
                    }
                    p.components.push(
                        serde_json::from_value(definition(
                            "wrapper",
                            serde_json::to_value(&p.tracks[0].items).unwrap(),
                        ))
                        .unwrap(),
                    );
                    p.tracks[0].items = if mode == "unused" {
                        vec![]
                    } else {
                        vec![serde_json::from_value(instance("outer", "wrapper", 0)).unwrap()]
                    };
                }
                "nested" => {
                    p.components[0].tracks[0].items.truncate(9);
                    let wrapper = definition(
                        "wrapper",
                        json!([
                            instance("inner", "leaf", 0),
                            repeat("inner", "component:wrapper", 1, 1)
                        ]),
                    );
                    p.components.push(serde_json::from_value(wrapper).unwrap());
                    if let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] {
                        i.component_id = "wrapper".into();
                    }
                }
                _ => {}
            }
            if extra {
                reject(&p, "transition");
            } else {
                let result = evaluate_project(&p, 100, 100, 30).unwrap();
                let facts = result
                    .scene
                    .visual_layers
                    .iter()
                    .map(|l| l.transitions.len())
                    .sum::<usize>();
                if matches!(
                    mode,
                    "track" | "instance" | "clipped" | "transition" | "transition-track" | "unused"
                ) {
                    assert_eq!(facts, 0, "{mode}");
                } else if mode != "repeater" {
                    assert_eq!(facts, 4096, "{mode}");
                }
            }
        }
    }
}

pub(crate) fn audio_fixture() -> Project {
    let media = json!({"type":"media","id":"media","assetId":"silent","startMs":0,"durationMs":1000,
        "sourceInMs":0,"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0},"keyframes":[],"zIndex":0,"stackOrder":0});
    let mut leaf = definition("leaf", json!([media]));
    leaf["slots"] = json!([{"id":"asset","name":"Asset","kind":"asset","required":false,
        "binding":{"targetLayerId":"media","property":"media.asset"},"constraints":{}}]);
    let mut root = instance("instance", "leaf", 0);
    root["slotValues"] =
        json!({"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":"audible"}}});
    let mut p = project_value(
        json!([root, repeat("instance", "root", 1, 1)]),
        json!([leaf]),
    );
    p["assets"] = json!([
        {"id":"silent","mediaType":"video","fileName":"silent.mp4","projectRelativePath":"assets/silent.mp4","durationMs":2000,"hasAudio":false},
        {"id":"audible","mediaType":"video","fileName":"audible.mp4","projectRelativePath":"assets/audible.mp4","durationMs":2000,"hasAudio":true}
    ]);
    serde_json::from_value(p).unwrap()
}

#[test]
fn transition_domains_are_independent_and_late_failure_never_clones() {
    let mut p = transition_fixture(16, 255, false);
    if let TimelineItem::Repeater(r) = &mut p.tracks[0].items[1] {
        r.repeater.source.scope = "component:first".into();
    }
    let first = definition("first", serde_json::to_value(&p.tracks[0].items).unwrap());
    let mut second = first.clone();
    second["id"] = json!("second");
    second["tracks"][0]["items"][1]["repeater"]["source"]["scope"] = json!("component:second");
    p.components.extend(
        serde_json::from_value::<Vec<crate::ComponentDefinition>>(json!([first, second])).unwrap(),
    );
    p.tracks[0].items.clear();
    for _ in 0..2 {
        assert!(
            evaluate_project(&p, 100, 100, 30)
                .unwrap()
                .scene
                .visual_layers
                .is_empty()
        );
        p.components.reverse();
    }
    let last = p.components.last_mut().unwrap();
    last.tracks[0].items.extend(
        serde_json::from_value::<Vec<TimelineItem>>(json!([
            rectangle("extra", 2),
            transition("extra-fade", "extra", 3)
        ]))
        .unwrap(),
    );
    reject(&p, "transition");
}

#[test]
fn effective_audio_defaults_overrides_and_cache_are_isolated() {
    let mut value = serde_json::to_value(audio_fixture()).unwrap();
    let audible = value["tracks"][0]["items"][0]["slotValues"]["asset"].clone();
    let mut silent = audible.clone();
    silent["value"]["id"] = json!("silent");
    // Authored audio is replaced by a silent default: raw classification is wrong.
    value["components"][0]["tracks"][0]["items"][0]["assetId"] = json!("audible");
    value["components"][0]["slots"][0]["defaultValue"] = silent.clone();
    value["tracks"][0]["items"][0]["slotValues"] = json!({});
    let mut other = instance("other", "leaf", 2);
    other["slotValues"] = json!({"asset":audible});
    value["tracks"][0]["items"]
        .as_array_mut()
        .unwrap()
        .push(other);
    for reverse in [false, true] {
        let mut v = value.clone();
        if reverse {
            v["tracks"][0]["items"].as_array_mut().unwrap().reverse();
            for (i, item) in v["tracks"][0]["items"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                item["stackOrder"] = json!(i);
            }
        }
        let p: Project = serde_json::from_value(v.clone()).unwrap();
        crate::validation::validate_project_visual_properties(&p).unwrap();
        let result = evaluate_project(&p, 100, 100, 30).unwrap();
        assert_eq!(result.scene.visual_layers.len(), 3);
        assert_eq!(result.scene.audio_layers.len(), 1);
        let r = v["tracks"][0]["items"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|v| v["type"] == "repeater")
            .unwrap();
        r["repeater"]["source"]["id"] = json!("other");
        reject(&serde_json::from_value(v).unwrap(), "visual-only");
    }
    let mut missing = value.clone();
    missing["tracks"][0]["items"][0]["slotValues"] =
        json!({"asset":{"type":"asset","value":{"kind":"asset","scope":"project","id":"absent"}}});
    let p: Project = serde_json::from_value(missing).unwrap();
    assert_eq!(
        crate::validation::validate_project_visual_properties(&p)
            .unwrap_err()
            .code,
        ErrorCode::AssetNotFound
    );
    let mut wrong = value;
    wrong["tracks"][0]["items"][0]["slotValues"] = json!({"asset":{"type":"text","value":"wrong"}});
    let p: Project = serde_json::from_value(wrong).unwrap();
    assert_eq!(
        crate::validation::validate_project_visual_properties(&p)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn effective_repeater_audio_is_rejected_before_publication() {
    for mode in [
        "root", "hidden", "muted", "clipped", "default", "group", "nested", "local", "unused",
    ] {
        let mut value = serde_json::to_value(audio_fixture()).unwrap();
        match mode {
            "hidden" => value["tracks"][0]["hidden"] = json!(true),
            "muted" => {
                value["components"][0]["tracks"][0]["items"][0]["audio"]["muted"] = json!(true)
            }
            "clipped" => {
                value["components"][0]["durationMs"] = json!(2000);
                value["tracks"][0]["items"][0]["trimStartMs"] = json!(1000);
            }
            "default" => {
                value["components"][0]["slots"][0]["defaultValue"] =
                    value["tracks"][0]["items"][0]["slotValues"]["asset"].clone();
                value["tracks"][0]["items"][0]["slotValues"] = json!({});
            }
            "group" => {
                value["tracks"][0]["items"][0]["parent"] = json!({"scope":"root","id":"group"});
                value["tracks"][0]["items"][1]["repeater"]["source"]["id"] = json!("group");
                value["tracks"][0]["items"].as_array_mut().unwrap().push(json!({"type":"group","id":"group","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":2}));
            }
            "nested" => {
                let inner = value["tracks"][0]["items"][0].clone();
                value["components"]
                    .as_array_mut()
                    .unwrap()
                    .push(definition("wrapper", json!([inner])));
                value["tracks"][0]["items"][0] = instance("instance", "wrapper", 0);
            }
            "local" | "unused" => {
                value["tracks"][0]["items"][1]["repeater"]["source"]["scope"] =
                    json!("component:wrapper");
                let items = value["tracks"][0]["items"].clone();
                value["components"]
                    .as_array_mut()
                    .unwrap()
                    .push(definition("wrapper", items));
                value["tracks"][0]["items"] = if mode == "unused" {
                    json!([])
                } else {
                    json!([instance("outer", "wrapper", 0)])
                };
            }
            _ => {}
        }
        let p: Project = serde_json::from_value(value).unwrap();
        let error = crate::validation::validate_project_visual_properties(&p).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument, "{mode}: {error:?}");
        reject(&p, "visual-only");
    }
}

#[test]
fn shifted_transition_budget_boundaries_preflight_before_copies() {
    for offset in [-60000, 0, 60000] {
        let mut exact = transition_fixture(16, 255, false);
        exact.schema_version = crate::PROJECT_SCHEMA_VERSION;
        exact.audio_buses = crate::default_audio_buses();
        if let TimelineItem::Repeater(item) = &mut exact.tracks[0].items[1] {
            item.repeater.time_offset_ms = offset;
        }
        assert!(evaluate_project(&exact, 100, 100, 30).is_ok());
        let mut over = transition_fixture(16, 255, true);
        over.schema_version = crate::PROJECT_SCHEMA_VERSION;
        over.audio_buses = crate::default_audio_buses();
        if let TimelineItem::Repeater(item) = &mut over.tracks[0].items[1] {
            item.repeater.time_offset_ms = offset;
        }
        reject(&over, "transition");
    }
}

#[test]
fn hidden_stagger_overflow_and_unused_animated_extent_fail_before_copies() {
    let mut last = rectangle("last", 2);
    last["startMs"] = json!(u64::MAX - 1);
    last["durationMs"] = json!(1);
    last["parent"] = json!({"scope":"root","id":"parent"});
    let mut first = rectangle("first", 1);
    first["parent"] = json!({"scope":"root","id":"parent"});
    let group = json!({"type":"group","id":"parent","startMs":0,"durationMs":u64::MAX,"hidden":true,"staggerMs":1,"stackOrder":0,"zIndex":0});
    let mut value = project_value(json!([group, first, last]), json!([]));
    value["schemaVersion"] = json!(crate::PROJECT_SCHEMA_VERSION);
    value["audioBuses"] = json!(crate::default_audio_buses());
    value["soundDefinitions"] = json!([]);
    value["markers"] = json!([]);
    value["fonts"] = json!({});
    reject_inherited(
        &serde_json::from_value(value).unwrap(),
        "stagger clock overflow",
    );
    let group = json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0,"animationChannels":[{"property":"transform.scale_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":900,"value":{"type":"scalar","value":100},"curve":"hold"}]}]});
    let mut child = rectangle("child", 1);
    child["width"] = json!(500);
    child["parent"] = json!({"scope":"component:unused","id":"parent"});
    let mut value = project_value(
        json!([]),
        json!([definition("unused", json!([group, child]))]),
    );
    value["schemaVersion"] = json!(crate::PROJECT_SCHEMA_VERSION);
    value["audioBuses"] = json!(crate::default_audio_buses());
    value["soundDefinitions"] = json!([]);
    value["markers"] = json!([]);
    value["fonts"] = json!({});
    value["components"][0]["markers"] = json!([]);
    reject_inherited(&serde_json::from_value(value).unwrap(), "animated");
}

#[test]
fn staggered_controller_restores_short_source_with_fractional_signed_clocks() {
    for (rate, offset, hidden_rank) in [(1.0, 0, false), (1.5, 30, false), (0.5, -30, true)] {
        let source = json!({"type":"shape","id":"source","startMs":0,"durationMs":100,
            "stackOrder":0,"geometry":{"type":"rectangle","width":10,"height":10},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"keyframes":[]});
        let mut controller = repeat(
            "source",
            "component:leaf",
            1,
            if hidden_rank { 2 } else { 1 },
        );
        controller["repeater"]["timeOffsetMs"] = json!(offset);
        let mut children = vec![source];
        if hidden_rank {
            let mut hidden = rectangle("hidden", 1);
            hidden["hidden"] = json!(true);
            children.push(hidden);
        }
        children.push(controller);
        let mut root = instance("instance", "leaf", 0);
        root["timeScale"] = json!(rate);
        root["staggerMs"] = json!(200);
        let mut value = project_value(json!([root]), json!([definition("leaf", json!(children))]));
        value["schemaVersion"] = json!(26);
        value.as_object_mut().unwrap().remove("audioBuses");
        value.as_object_mut().unwrap().remove("soundDefinitions");
        value["markers"] = json!([]);
        value["fonts"] = json!({});
        value["components"][0]["markers"] = json!([]);
        value["components"][0]["durationMs"] = json!(2000);
        let p: Project = serde_json::from_value(value).unwrap();
        GENERATED_MATERIALIZATIONS.with(|n| n.set(0));
        preflight_inherited_project(&p).unwrap();
        GENERATED_MATERIALIZATIONS.with(|n| assert_eq!(n.get(), 0));
        let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
        let ordinary = scene
            .visual_layers
            .iter()
            .find(|l| {
                matches!(l.source, EvaluatedVisualSource::Shape(_))
                    && !l.item_id.starts_with("repeater:")
            })
            .unwrap();
        let copy = scene
            .visual_layers
            .iter()
            .find(|l| l.item_id.starts_with("repeater:"))
            .expect("delayed source copy must survive");
        let delay: f64 = if hidden_rank { 400.0 } else { 200.0 };
        // Controller clipping is independent of the signed source delay.
        let expected_start = delay.max(delay + offset as f64) / rate;
        let expected_end = (delay + offset as f64 + 100.0) / rate;
        let c = copy.instance.unwrap();
        assert_eq!((c.start_ms, c.end_ms), (expected_start, expected_end));
        assert_eq!(c.offset, -(delay + offset as f64));
        let c = ordinary.instance.unwrap();
        assert_eq!((c.start_ms, c.end_ms, c.offset), (0.0, 100.0 / rate, 0.0));
    }
}

#[test]
fn nested_repeater_controller_shifts_source_stages_once_and_preserves_outside_clock() {
    let shape = json!({"type":"shape","id":"source","startMs":0,"durationMs":100,"stackOrder":0,"zIndex":0,"keyframes":[],"geometry":{"type":"rectangle","width":10,"height":10},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null});
    let mut inner_copy = repeat("source", "component:leaf", 1, 1);
    inner_copy["repeater"]["timeOffsetMs"] = json!(10);
    let mut inner = instance("inner", "leaf", 0);
    inner["timeScale"] = json!(0.5);
    inner["staggerMs"] = json!(30);
    let channel = json!({"property":"transform.position_x","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},{"timeMs":400,"value":{"type":"scalar","value":20},"curve":"hold"}]});
    inner["animationChannels"] = json!([channel.clone()]);
    let outer_copy = repeat("inner", "component:outer", 1, 1);
    let mut root = instance("root", "outer", 0);
    root["startMs"] = json!(100);
    root["durationMs"] = json!(500);
    root["trimStartMs"] = json!(50);
    root["timeScale"] = json!(1.5);
    root["staggerMs"] = json!(200);
    root["animationChannels"] = json!([channel]);
    let mut value = project_value(
        json!([root]),
        json!([
            definition("outer", json!([inner, outer_copy])),
            definition("leaf", json!([shape, inner_copy]))
        ]),
    );
    value["schemaVersion"] = json!(26);
    value.as_object_mut().unwrap().remove("audioBuses");
    value.as_object_mut().unwrap().remove("soundDefinitions");
    value["markers"] = json!([]);
    value["fonts"] = json!({});
    for c in value["components"].as_array_mut().unwrap() {
        c["markers"] = json!([]);
        c["durationMs"] = json!(2000);
    }
    let p: Project = serde_json::from_value(value).unwrap();
    GENERATED_MATERIALIZATIONS.with(|n| n.set(0));
    preflight_inherited_project(&p).unwrap();
    GENERATED_MATERIALIZATIONS.with(|n| assert_eq!(n.get(), 0));
    let scene = evaluate_project(&p, 100, 100, 30).unwrap().scene;
    assert_eq!(scene.visual_layers.len(), 4);
    let mut offsets = scene
        .visual_layers
        .iter()
        .map(|l| l.instance.unwrap().offset)
        .collect::<Vec<_>>();
    offsets.sort_by(f64::total_cmp);
    assert_eq!(offsets, vec![-190.0, -150.0, -90.0, -50.0]);
    for l in &scene.visual_layers {
        let root_stage = l
            .ancestor_stages
            .iter()
            .find(|s| s.item_id == "root")
            .unwrap()
            .animation
            .as_ref()
            .unwrap();
        assert_eq!((root_stage.clock.rate, root_stage.clock.offset), (1.0, 0.0));
        let inner_stage = l
            .ancestor_stages
            .iter()
            .find(|s| s.item_id == "inner")
            .unwrap()
            .animation
            .as_ref()
            .unwrap();
        let copied_outer = l.instance.unwrap().offset <= -150.0;
        assert_eq!(
            inner_stage.clock.offset,
            if copied_outer { -300.0 } else { -100.0 }
        );
        let c = l.instance.unwrap();
        assert!((c.end_ms - (100.0 - c.offset) / c.rate).abs() < 1e-9);
    }
}

#[test]
fn publication_projection_bounds_reject_without_generated_materialization() {
    let mut p = transition_fixture(17, 255, false);
    if let TimelineItem::Repeater(r) = &mut p.tracks[0].items[1] {
        r.repeater.time_offset_ms = 1;
    }
    p.schema_version = 26;
    GENERATED_MATERIALIZATIONS.with(|n| n.set(0));
    assert!(
        preflight_inherited_project(&p)
            .unwrap_err()
            .message
            .contains("transition")
    );
    GENERATED_MATERIALIZATIONS.with(|n| assert_eq!(n.get(), 0));
}

fn reject_inherited(project: &Project, message: &str) {
    GENERATED_MATERIALIZATIONS.with(|n| n.set(0));
    let error = preflight_inherited_project(project).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    assert!(error.message.contains(message), "{error:?}");
    GENERATED_MATERIALIZATIONS.with(|n| assert_eq!(n.get(), 0));
    reject(project, message);
}
