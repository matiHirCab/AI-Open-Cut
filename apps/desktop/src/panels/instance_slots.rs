use gpui::{Context, Div, div, prelude::*};
use opencut_editor_core::{Project, SlotKind};

use crate::{
    hierarchy::Selection,
    instance_slots,
    shell::{Shell, button},
};

pub(crate) fn render(
    shell: &Shell,
    project: &Project,
    selection: &Selection,
    cx: &mut Context<Shell>,
) -> Div {
    let mut panel = div()
        .flex()
        .flex_col()
        .gap_1()
        .py_2()
        .child("Instance slots · authoritative definition");
    let Some(instance) = instance_slots::instance(project, selection) else {
        return panel;
    };
    if let Some(draft) = shell
        .slot_draft
        .as_ref()
        .filter(|draft| draft.matches(project, Some(selection)))
    {
        panel = panel
            .child(
                div()
                    .text_sm()
                    .child(format!("Editing slot {}", draft.slot_id)),
            )
            .child(
                div()
                    .id("slot-input")
                    .track_focus(&shell.slot_focus)
                    .border_1()
                    .p_2()
                    .child(format!("{} ▏", draft.text))
                    .on_click(cx.listener(|this, _, window, _| this.slot_focus.focus(window)))
                    .on_key_down(cx.listener(Shell::slot_key)),
            );
        if draft.exhausted {
            panel = panel.child("Input capacity exceeded. Correct or reset before Apply.");
        }
        if draft.kind == SlotKind::Boolean {
            for value in ["true", "false"] {
                panel = panel.child(
                    button(
                        ("slot-hidden", usize::from(value == "true")),
                        format!("Hidden = {value}"),
                    )
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.set_slot_text(value.into(), cx)),
                    ),
                );
            }
        }
        if draft.kind == SlotKind::Asset {
            panel = panel.child("Managed project assets (literal ID; no import)");
            for (index, asset) in project
                .assets
                .iter()
                .take(instance_slots::ASSET_LIMIT)
                .enumerate()
            {
                let id = asset.id.clone();
                panel = panel.child(
                    button(
                        ("slot-asset", index),
                        format!("{} · {}", asset.id, asset.file_name),
                    )
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.set_slot_text(id.clone(), cx)),
                    ),
                );
            }
            if project.assets.len() > instance_slots::ASSET_LIMIT {
                panel = panel.child(
                    "Asset list limited to 128 entries; enter an existing managed ID above.",
                );
            }
        }
        panel = panel
            .child(
                button("slot-apply", "Apply override")
                    .on_click(cx.listener(|this, _, _, cx| this.apply_slot(false, cx))),
            )
            .child(
                button("slot-clear", "Clear override · use shared default/source")
                    .on_click(cx.listener(|this, _, _, cx| this.apply_slot(true, cx))),
            )
            .child(
                button("slot-reset", "Draft Reset · discard input").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.reset_inspector();
                        cx.notify();
                    },
                )),
            );
    }
    for (index, slot) in instance_slots::slots(project, selection).iter().enumerate() {
        panel = panel
            .child(div().text_xs().child(format!(
                "{} · {} · {:?} · required={} · {} → {}",
                slot.id,
                slot.name,
                slot.kind,
                slot.required,
                slot.binding.target_layer_id,
                serde_json::to_string(&slot.binding.property).expect("binding")
            )))
            .child(div().text_xs().child(format!(
                "{}; default={}; override={}",
                instance_slots::source(instance, slot),
                serde_json::to_string(&slot.default_value).expect("default"),
                serde_json::to_string(&instance.slot_values.get(&slot.id)).expect("override")
            )));
        if instance_slots::supported(selection, slot) {
            panel = panel.child(
                button(("instance-slot", index), format!("Edit {}", slot.name)).on_click(
                    cx.listener(move |this, _, window, cx| {
                        this.choose_slot(index, cx);
                        this.slot_focus.focus(window);
                    }),
                ),
            );
        } else {
            panel = panel.child(
                div()
                    .text_xs()
                    .child("Read-only · component-local occurrence or unsupported kind/binding"),
            );
        }
    }
    panel
}
