use crate::{
    animation_inspector,
    compositing_inspector::{self, Action, EFFECT_TYPES, Eligibility},
    hierarchy::{editable, kind},
    shell::{Shell, button},
    theme::ActiveTheme,
};
use gpui::{Context, Window, div, prelude::*};
use opencut_editor_core::TimelineItem;

pub(crate) fn render(shell: &Shell, window: &Window, cx: &mut Context<Shell>) -> impl IntoElement {
    let colors = window.theme().colors;
    let mut panel = div()
        .id("inspector")
        .w_1_3()
        .h_full()
        .overflow_y_scroll()
        .p_2()
        .border_l_1()
        .border_color(colors.border)
        .bg(colors.card)
        .child(div().text_lg().child("Inspector"));
    let Some((project, selection, track, item)) = shell.session.project.as_ref().and_then(|p| {
        let s = shell.session.selected.as_ref()?;
        let (t, i) = s.resolve(p)?;
        Some((p, s, t, i))
    }) else {
        return panel.child("Select a layer.");
    };
    let parent = item
        .visual_properties()
        .parent
        .as_ref()
        .map(|p| format!("{} / {}", p.scope, p.id))
        .unwrap_or_else(|| "None".into());
    for text in [
        format!("{} · {}", kind(item), item.id()),
        format!("Scope: {}", selection.scope),
        format!("Instance path: {}", selection.instance_path.join(" / ")),
        format!(
            "Track: {} ({}){}",
            track.name,
            track.id,
            if track.locked { " · locked" } else { "" }
        ),
        format!(
            "Local time: {} ms · duration {} ms",
            item.start_ms(),
            item.duration_ms()
        ),
        format!("Parent: {parent}"),
        format!(
            "Z-index: {} · stack order: {}",
            item.visual_properties().z_index,
            item.visual_properties().stack_order
        ),
        format!("Hidden: {}", item.hidden()),
    ] {
        panel = panel.child(div().py_1().text_xs().child(text));
    }
    if let TimelineItem::ComponentInstance(instance) = item {
        panel = panel.child(
            div()
                .text_sm()
                .child("Stored slot overrides (defaults live in definition)"),
        );
        for (id, value) in &instance.slot_values {
            panel = panel.child(div().text_xs().child(format!(
                "{id}: {}",
                serde_json::to_string(value).expect("serialize slot")
            )));
        }
    }
    match item {
        TimelineItem::Shape(shape) => {
            panel = panel.child(div().text_xs().child(format!(
                "Geometry: {}",
                serde_json::to_string(&shape.geometry).unwrap()
            )));
            if shape.fill.is_none() {
                panel = panel.child("Fill: none");
            }
            if shape.stroke.is_none() {
                panel = panel.child("Stroke: none");
            }
        }
        TimelineItem::Grid(grid) => {
            panel = panel.child(div().text_xs().child(format!(
                "Grid: {}",
                serde_json::to_string(&grid.grid).unwrap()
            )));
        }
        TimelineItem::Text(text) => {
            panel = panel.child(div().text_xs().child(format!(
                "Font: {}",
                text.font_family.as_deref().unwrap_or("resolved default")
            )));
            panel = panel.child(div().text_xs().child(format!(
                    "Font binding: {}",
                    text.font_binding
                        .as_ref()
                        .map(|binding| serde_json::to_string(binding).unwrap())
                        .unwrap_or_else(|| "none".into())
                )));
            panel = panel.child(div().text_xs().child(format!(
                "Document: {}",
                serde_json::to_string(&text.document).unwrap()
            )));
        }
        _ => {}
    }
    panel = panel.child(
        div()
            .py_2()
            .text_sm()
            .child("Authored animation · source keys"),
    );
    for description in animation_inspector::descriptions(item, shell.animation_cursor) {
        panel = panel.child(div().text_xs().py_1().child(description));
    }
    let channels = &item.visual_properties().animation_channels;
    for (axis, len, previous_id, next_id, previous_label, next_label) in [
        (
            0,
            channels.len(),
            "animation-channel-prev",
            "animation-channel-next",
            "Previous channel",
            "Next channel",
        ),
        (
            1,
            channels
                .get(shell.animation_cursor.channel_index(item))
                .map_or(0, |c| c.keyframes.len()),
            "animation-key-prev",
            "animation-key-next",
            "Previous source key",
            "Next source key",
        ),
        (
            2,
            item.keyframes().len(),
            "animation-legacy-prev",
            "animation-legacy-next",
            "Previous legacy key",
            "Next legacy key",
        ),
    ] {
        if len > 1 {
            panel = panel.child(
                div()
                    .flex()
                    .gap_1()
                    .child(button(previous_id, previous_label).on_click(cx.listener(
                        move |this, _, _, cx| this.change_animation_cursor(axis, false, cx),
                    )))
                    .child(button(next_id, next_label).on_click(cx.listener(
                        move |this, _, _, cx| this.change_animation_cursor(axis, true, cx),
                    ))),
            );
        }
    }
    panel = panel.child(
        div()
            .py_2()
            .text_sm()
            .child("Compositing · authored values"),
    );
    for text in compositing_inspector::descriptions(item, &shell.compositing_cursor) {
        panel = panel.child(div().text_xs().py_1().child(text));
    }
    if !animation_inspector::editable(selection) {
        return panel.child("Component-local content · read-only");
    }
    let eligibility = compositing_inspector::eligibility(project, selection, item);
    let mut actions = vec![];
    if eligibility == Eligibility::Leaf {
        actions.extend([
            ("Add mask · fixed 64 local px".to_owned(), Action::AddMask),
            ("Delete selected mask".into(), Action::DeleteMask),
            ("Mask earlier".into(), Action::MoveMask(false)),
            ("Mask later".into(), Action::MoveMask(true)),
            ("Clear matte".into(), Action::ClearMatte),
        ]);
    }
    if matches!(eligibility, Eligibility::Leaf | Eligibility::Aggregate) {
        for kind in EFFECT_TYPES {
            actions.push((format!("Add effect · {kind}"), Action::AddEffect(kind)));
        }
        actions.extend([
            ("Delete selected effect".into(), Action::DeleteEffect),
            ("Effect earlier".into(), Action::MoveEffect(false)),
            ("Effect later".into(), Action::MoveEffect(true)),
        ]);
    }
    if eligibility == Eligibility::Aggregate {
        actions.extend([
            ("Set composition bounds clip".into(), Action::SetClip(true)),
            ("Clear clip".into(), Action::SetClip(false)),
        ]);
    }
    for (index, (label, action)) in actions.into_iter().enumerate() {
        panel = panel.child(button(("compositing-action", index), label).on_click(
            cx.listener(move |this, _, _, cx| this.compositing_action(action.clone(), cx)),
        ));
    }
    for (axis, label) in [
        (0usize, "mask"),
        (1, "effect"),
        (2, "path command"),
        (3, "gradient stop"),
    ] {
        if eligibility == Eligibility::Leaf || (axis == 1 && eligibility == Eligibility::Aggregate)
        {
            panel = panel.child(
                div()
                    .flex()
                    .gap_1()
                    .child(
                        button(("compositing-prev", axis), format!("Previous {label}")).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.change_compositing_cursor(axis as u8, false, cx)
                            }),
                        ),
                    )
                    .child(
                        button(("compositing-next", axis), format!("Next {label}")).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.change_compositing_cursor(axis as u8, true, cx)
                            }),
                        ),
                    ),
            );
        }
    }
    let fields = shell.inspector_fields(item);

    if !fields.is_empty() {
        panel = panel.child(
            div()
                .py_2()
                .text_sm()
                .child("Supported fields · select one to edit"),
        );
        if shell.inspector_field.is_some() {
            panel = panel
                .child(
                    div()
                        .py_2()
                        .child("Edit selected value · Enter to apply, Esc to reset"),
                )
                .child(
                    div()
                        .id("inspector-input")
                        .track_focus(&shell.inspector_focus)
                        .border_1()
                        .p_2()
                        .child(format!("{} ▏", shell.inspector_text))
                        .on_click(
                            cx.listener(|this, _, window, _| this.inspector_focus.focus(window)),
                        )
                        .on_key_down(cx.listener(Shell::inspector_key)),
                )
                .child(
                    button("inspector-apply", "Apply field")
                        .on_click(cx.listener(|this, _, _, cx| this.apply_inspector(cx))),
                )
                .child(
                    button("inspector-reset", "Reset field").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.reset_inspector();
                            cx.notify();
                        },
                    )),
                );
        }
        for (index, field) in fields.iter().enumerate() {
            let label = format!("{}: {}", field.label, field.value);
            panel = panel.child(
                button(("inspector-field", index), label).on_click(cx.listener(
                    move |this, _, window, cx| {
                        this.choose_inspector_field(index, cx);
                        this.inspector_focus.focus(window);
                    },
                )),
            );
        }
        if shell
            .inspector_field
            .and_then(|i| fields.get(i))
            .is_some_and(|f| f.update_key == "animationChannels")
        {
            panel = panel.child(div().py_2().child("Applying any animation field replaces the authored channel collection and clears all preset attribution on this item, even if the value is unchanged. Source clocks and untouched keys/channels are preserved."));
        }
    }
    if !editable(project, selection) {
        return panel.child(
            "Audio-only item · gain animation controls only; visual fields are unavailable.",
        );
    }
    panel = panel
        .child(
            div()
                .py_2()
                .child("Z-index · click, Ctrl+A to clear, type, Enter to apply"),
        )
        .child(
            div()
                .id("z-input")
                .track_focus(&shell.z_focus)
                .border_1()
                .p_2()
                .child(format!("{} ▏", shell.z_text))
                .on_click(cx.listener(|this, _, window, _| this.z_focus.focus(window)))
                .on_key_down(cx.listener(Shell::z_key)),
        )
        .child(
            button("z-apply", "Apply z-index")
                .on_click(cx.listener(|this, _, _, cx| this.apply_z(cx))),
        )
        .child(div().py_2().child("Parent · preserves local coordinates"))
        .child(
            button("detach", "Detach (no parent)")
                .on_click(cx.listener(|this, _, _, cx| this.set_parent(None, cx))),
        );
    for (index, group) in project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|i| matches!(i, TimelineItem::Group(_)))
        .enumerate()
    {
        let id = group.id().to_owned();
        panel = panel
            .child(button(("parent", index), format!("Group {id}")).on_click(
                cx.listener(move |this, _, _, cx| this.set_parent(Some(id.clone()), cx)),
            ));
    }
    panel
}
