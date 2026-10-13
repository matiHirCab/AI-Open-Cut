//! Disposable original-artwork fixture shared by source tests and the GUI example.
use crate::rule_card_fixture_support as rule_card;

use opencut_editor_core::{EditOperation, SlotValue};
use serde_json::json;
use std::path::Path;

pub(crate) fn seed(root: &Path) -> rule_card::Fixture {
    let mut f = rule_card::seed(root);
    f.move_parent();
    f.core.undo(&f.id, f.project().revision).unwrap();
    f.core
        .edit(
            &f.id,
            f.project().revision,
            EditOperation::DeleteItem {
                item_id: f.aliases["instance2"].clone(),
            },
        )
        .unwrap();
    let project = f.project();
    let component = &project.components[0];
    let mut tracks = component.tracks.clone();
    let mut rich = tracks[0]
        .items
        .iter()
        .find_map(|item| match item {
            opencut_editor_core::TimelineItem::Text(text) if text.id == "title" => {
                Some(text.clone())
            }
            _ => None,
        })
        .unwrap();
    rich.id = "rich_source".into();
    rich.font_binding = None;
    rich.visual_properties.stack_order = tracks[0].items.len() as u32;
    tracks[0]
        .items
        .push(opencut_editor_core::TimelineItem::Text(rich));
    let mut slots = component.slots.clone();
    let title = slots.iter_mut().find(|slot| slot.id == "title").unwrap();
    title.required = true;
    title.default_value = None;
    title.constraints.max_length = Some(40);
    for value in [
        json!({"id":"__proto__","name":"Title color","kind":"color","required":false,"defaultValue":{"type":"color","value":"#AbCdEf"},"binding":{"targetLayerId":"title","property":"text.color"},"constraints":{}}),
        json!({"id":"constructor","name":"Hidden background","kind":"boolean","required":false,"defaultValue":{"type":"boolean","value":false},"binding":{"targetLayerId":"background","property":"visual.hidden"},"constraints":{}}),
        json!({"id":"toString","name":"Body color","kind":"color","required":false,"binding":{"targetLayerId":"body","property":"text.color"},"constraints":{}}),
        json!({"id":"optional_hidden","name":"Optional hidden accent","kind":"boolean","required":false,"binding":{"targetLayerId":"accent","property":"visual.hidden"},"constraints":{}}),
        json!({"id":"alignment","name":"Read-only alignment","kind":"enum","required":false,"defaultValue":{"type":"enum","value":"left"},"binding":{"targetLayerId":"title","property":"text.alignment"},"constraints":{"choices":["left","right"]}}),
        json!({"id":"duration","name":"Read-only duration (ms)","kind":"duration","required":false,"defaultValue":{"type":"duration","value":1000},"binding":{"targetLayerId":"body","property":"item.durationMs"},"constraints":{}}),
        json!({"id":"rich","name":"Read-only rich text","kind":"rich_text","required":false,"defaultValue":{"type":"rich_text","value":{"runs":[{"text":"Styled","bold":true}]}},"binding":{"targetLayerId":"rich_source","property":"text.document"},"constraints":{}}),
    ] {
        slots.push(serde_json::from_value(value).unwrap());
    }
    f.core
        .edit(
            &f.id,
            project.revision,
            EditOperation::ComponentUpdate {
                component_id: component.id.clone(),
                name: component.name.clone(),
                width: component.width,
                height: component.height,
                duration_ms: component.duration_ms,
                tracks,
                slots: Some(slots),
            },
        )
        .unwrap();
    // Give the two occurrences distinct color and visibility overrides through core.
    for (index, color) in ["#aBcD12", "#123aBc"].into_iter().enumerate() {
        let project = f.project();
        let instance = project
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .find_map(|item| match item {
                opencut_editor_core::TimelineItem::ComponentInstance(instance)
                    if instance.id == f.aliases[&format!("instance{index}")] =>
                {
                    Some(instance)
                }
                _ => None,
            })
            .unwrap();
        let mut values = instance.slot_values.clone();
        values.insert("__proto__".into(), SlotValue::Color(color.into()));
        values.insert("constructor".into(), SlotValue::Boolean(index == 1));
        f.core
            .edit(
                &f.id,
                project.revision,
                EditOperation::ComponentInstanceUpdate {
                    item_id: instance.id.clone(),
                    component_id: instance.component_id.clone(),
                    start_ms: instance.start_ms,
                    trim_start_ms: instance.trim_start_ms,
                    duration_ms: instance.duration_ms,
                    time_scale: instance.time_scale,
                    stagger_ms: Some(instance.stagger_ms),
                    slot_values: Some(values),
                },
            )
            .unwrap();
    }
    let project = f.project();
    let nested = project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .find_map(|item| match item {
            opencut_editor_core::TimelineItem::ComponentInstance(instance)
                if instance.id == f.aliases["instance0"] =>
            {
                Some(instance)
            }
            _ => None,
        })
        .unwrap();
    let mut nested = serde_json::to_value(nested).unwrap();
    nested["type"] = json!("component_instance");
    nested["id"] = json!("nested");
    nested["stackOrder"] = json!(0);
    nested.as_object_mut().unwrap().remove("parent");
    let result = f.core.edit(&f.id, project.revision, rule_card::operation(json!({
        "operation":"component_create","name":"Read-only nested source","width":160,"height":90,"durationMs":1000,
        "tracks":[{"id":"wrapper_track","name":"Nested layer","trackType":"overlay","items":[nested]}],"slots":[]
    }))).unwrap();
    let wrapper = result.changed_ids[0].clone();
    let project = f.project();
    let track = project
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Overlay)
        .unwrap()
        .id
        .clone();
    for index in 0..2 {
        let result = f
            .core
            .edit(
                &f.id,
                f.project().revision,
                rule_card::operation(json!({
                    "operation":"add_component_instance","trackId":track,"componentId":wrapper,
                    "startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1
                })),
            )
            .unwrap();
        f.aliases
            .insert(format!("wrapper{index}"), result.changed_ids[0].clone());
    }
    f
}
