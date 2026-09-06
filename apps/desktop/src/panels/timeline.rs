use crate::{
    hierarchy::{Selection, kind},
    shell::Shell,
    theme::ActiveTheme,
};
use gpui::{Context, Window, div, prelude::*};

pub(crate) fn render(shell: &Shell, window: &Window, cx: &mut Context<Shell>) -> impl IntoElement {
    let colors = window.theme().colors;
    let mut panel = div()
        .id("timeline")
        .h_1_3()
        .overflow_y_scroll()
        .p_2()
        .bg(colors.card)
        .child(div().text_lg().child("Timeline · root composition"));
    if let Some(project) = &shell.session.project {
        for (track_index, track) in project.tracks.iter().enumerate() {
            panel = panel.child(div().text_sm().child(format!(
                "Track {track_index}: {}{}",
                track.name,
                if track.locked { " · locked" } else { "" }
            )));
            for (index, item) in track.items.iter().enumerate() {
                let selection = Selection::root(item.id());
                let selected = shell.session.selected.as_ref() == Some(&selection);
                panel = panel.child(
                    div()
                        .id(gpui::SharedString::from(format!(
                            "track-{track_index}-{index}"
                        )))
                        .text_xs()
                        .py_1()
                        .cursor_pointer()
                        .bg(if selected { colors.accent } else { colors.card })
                        .child(format!(
                            "{} {} · {}–{} ms · z {} · order {}{}",
                            kind(item),
                            item.id(),
                            item.start_ms(),
                            item.end_ms(),
                            item.visual_properties().z_index,
                            item.visual_properties().stack_order,
                            if item.hidden() { " · hidden" } else { "" }
                        ))
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.select(selection.clone(), cx)),
                        ),
                );
            }
        }
    }
    panel
}
