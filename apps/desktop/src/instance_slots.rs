//! Authoritative slot presentation and typed input, without domain validation.
use opencut_editor_core::{
    ComponentInstanceItem, EditOperation, Project, SlotAssetKind, SlotAssetReference,
    SlotAssetScope, SlotBinding, SlotKind, SlotProperty, SlotValue, TemplateSlot, TimelineItem,
};

use crate::hierarchy::Selection;

pub(crate) const INPUT_BYTES: usize = 16_384;
pub(crate) const SLOT_LIMIT: usize = 128;
pub(crate) const ASSET_LIMIT: usize = 128;

pub(crate) fn instance<'a>(
    project: &'a Project,
    selection: &Selection,
) -> Option<&'a ComponentInstanceItem> {
    match selection.resolve(project)?.1 {
        TimelineItem::ComponentInstance(instance) => Some(instance),
        _ => None,
    }
}

pub(crate) fn slots<'a>(project: &'a Project, selection: &Selection) -> &'a [TemplateSlot] {
    instance(project, selection)
        .and_then(|instance| {
            project
                .components
                .iter()
                .find(|c| c.id == instance.component_id)
        })
        .map_or(&[], |component| {
            &component.slots[..component.slots.len().min(SLOT_LIMIT)]
        })
}

pub(crate) fn supported(selection: &Selection, slot: &TemplateSlot) -> bool {
    selection.instance_path.is_empty()
        && selection.scope == "root"
        && matches!(
            (slot.kind, slot.binding.property),
            (SlotKind::Text, SlotProperty::TextDocument)
                | (SlotKind::Color, SlotProperty::TextColor)
                | (SlotKind::Color, SlotProperty::ShapeFillColor)
                | (SlotKind::Color, SlotProperty::ShapeStrokeColor)
                | (SlotKind::Color, SlotProperty::GridDotPaintColor)
                | (SlotKind::Boolean, SlotProperty::VisualHidden)
                | (SlotKind::Asset, SlotProperty::MediaAsset)
        )
}

pub(crate) fn representation(value: Option<&SlotValue>) -> String {
    match value {
        Some(SlotValue::Text(value) | SlotValue::Color(value)) => value.clone(),
        Some(SlotValue::Boolean(value)) => value.to_string(),
        Some(SlotValue::Asset(value)) => value.id.clone(),
        Some(value) => serde_json::to_string(value).expect("serialize slot value"),
        None => String::new(),
    }
}

pub(crate) fn source(instance: &ComponentInstanceItem, slot: &TemplateSlot) -> &'static str {
    if instance.slot_values.contains_key(&slot.id) {
        "Stored override"
    } else if slot.default_value.is_some() {
        "Shared definition default"
    } else {
        "Absent override/default · bound source property retained"
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Draft {
    pub selection: Selection,
    pub revision: u64,
    pub component_id: String,
    pub slot_id: String,
    pub kind: SlotKind,
    pub binding: SlotBinding,
    pub text: String,
    pub exhausted: bool,
}

impl Draft {
    pub(crate) fn new(
        project: &Project,
        selection: &Selection,
        slot: &TemplateSlot,
    ) -> Option<Self> {
        let instance = instance(project, selection)?;
        if !supported(selection, slot) {
            return None;
        }
        let text = representation(
            instance
                .slot_values
                .get(&slot.id)
                .or(slot.default_value.as_ref()),
        );
        Some(Self {
            selection: selection.clone(),
            revision: project.revision,
            component_id: instance.component_id.clone(),
            slot_id: slot.id.clone(),
            kind: slot.kind,
            binding: slot.binding.clone(),
            exhausted: text.len() > INPUT_BYTES,
            text,
        })
    }

    pub(crate) fn matches(&self, project: &Project, selected: Option<&Selection>) -> bool {
        selected == Some(&self.selection)
            && self.revision == project.revision
            && instance(project, &self.selection)
                .is_some_and(|instance| instance.component_id == self.component_id)
            && slots(project, &self.selection).iter().any(|slot| {
                slot.id == self.slot_id
                    && slot.kind == self.kind
                    && slot.binding == self.binding
                    && supported(&self.selection, slot)
            })
    }

    pub(crate) fn append(&mut self, text: &str) {
        if self.text.len().saturating_add(text.len()) <= INPUT_BYTES {
            self.text.push_str(text);
        } else {
            self.exhausted = true;
        }
    }

    pub(crate) fn build(
        &self,
        project: &Project,
        selected: Option<&Selection>,
        clear: bool,
        busy: bool,
        needs_refresh: bool,
    ) -> Result<EditOperation, String> {
        if busy || needs_refresh || !self.matches(project, selected) {
            return Err(
                "Slot draft is stale or editing is unavailable. Refresh and select a slot again."
                    .into(),
            );
        }
        let instance = instance(project, &self.selection).expect("matched instance");
        let mut values = instance.slot_values.clone();
        if clear {
            values.remove(&self.slot_id);
        } else {
            if self.exhausted {
                return Err(
                    "Input capacity exceeded; correct or reset the draft before applying.".into(),
                );
            }
            let value = match self.kind {
                SlotKind::Text => SlotValue::Text(self.text.clone()),
                SlotKind::Color => SlotValue::Color(self.text.clone()),
                SlotKind::Boolean => SlotValue::Boolean(match self.text.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err("Hidden value must be exactly true or false.".into()),
                }),
                SlotKind::Asset => SlotValue::Asset(SlotAssetReference {
                    kind: SlotAssetKind::Asset,
                    scope: SlotAssetScope::Project,
                    id: self.text.clone(),
                }),
                _ => return Err("This slot is read-only.".into()),
            };
            values.insert(self.slot_id.clone(), value);
        }
        Ok(EditOperation::ComponentInstanceUpdate {
            item_id: instance.id.clone(),
            component_id: instance.component_id.clone(),
            start_ms: instance.start_ms,
            trim_start_ms: instance.trim_start_ms,
            duration_ms: instance.duration_ms,
            time_scale: instance.time_scale,
            stagger_ms: Some(instance.stagger_ms),
            slot_values: Some(values),
        })
    }
}
