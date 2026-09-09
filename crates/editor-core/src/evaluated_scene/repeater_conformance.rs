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
