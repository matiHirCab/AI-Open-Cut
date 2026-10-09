use crate::{
    narration_inspector,
    shell::{Shell, button},
};
use gpui::{Context, div, prelude::*};

pub(crate) fn render(shell: &Shell, cx: &mut Context<Shell>) -> impl IntoElement {
    let mut panel = div().child(div().text_sm().py_2().child("Narration cues · read-only"));
    let Some(project) = &shell.session.project else {
        return panel;
    };
    let cursor = &shell.narration_cursor;
    let (scope, markers) = cursor.scope(project);
    let label = if cursor.scope == 0 {
        "root".to_owned()
    } else {
        format!("component:{scope}")
    };
    panel = panel.child(div().w_full().whitespace_normal().text_xs().child(format!(
            "Scope {label} · {} cues · page {} / {}",
            markers.len(),
            cursor
                .marker_page
                .min(markers.len().saturating_sub(1) / narration_inspector::MARKER_PAGE)
                + 1,
            markers
                .len()
                .div_ceil(narration_inspector::MARKER_PAGE)
                .max(1)
        )));
    for (axis, label) in [(0, "scope"), (1, "cue page")] {
        for (next, prefix) in [(false, "Previous"), (true, "Next")] {
            panel =
                panel.child(
                    button(
                        ("narration-nav", axis * 2 + usize::from(next)),
                        format!("{prefix} {label}"),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.navigate_narration(axis as u8, next, cx)
                    })),
                );
        }
    }
    for marker in cursor.rows(project) {
        let id = marker.id.clone();
        panel = panel.child(
            button(
                gpui::SharedString::from(format!("narration-marker-{}", marker.id)),
                format!("{} · {} ms · cue", marker.name, marker.time_ms),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.narration_cursor.marker_id = Some(id.clone());
                this.narration_cursor.revision = this
                    .session
                    .project
                    .as_ref()
                    .map(|project| project.revision);
                cx.notify();
            })),
        );
    }
    if let Some(marker) = cursor.selected(project) {
        panel = panel.child(div().w_full().whitespace_normal().text_xs().child(format!(
            "Selected cue: {} · ID {} · scope {} · local {} ms",
            marker.name, marker.id, marker.scope, marker.time_ms
        )));
    }
    if markers.is_empty() {
        panel = panel.child("No cues in this scope.");
    }
    for text in narration_inspector::bus_descriptions(project) {
        panel = panel.child(
            div()
                .w_full()
                .whitespace_normal()
                .text_xs()
                .py_1()
                .child(text),
        );
    }
    panel
}
