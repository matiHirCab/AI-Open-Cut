use crate::{
    hierarchy::Hierarchy,
    shell::{Shell, button},
    theme::ActiveTheme,
};
use gpui::{Context, Window, div, prelude::*, px};

pub(crate) fn render(shell: &Shell, window: &Window, cx: &mut Context<Shell>) -> impl IntoElement {
    let colors = window.theme().colors;
    let mut panel = div().id("hierarchy").w_1_3().h_full().overflow_y_scroll().p_2().bg(colors.card)
        .child(div().text_lg().child("Hierarchy"))
        .child(div().text_xs().child("Tree = parentage. Paint order: track, z-index, item order. Each instance is one stacking block."));
    let Some(project) = &shell.session.project else {
        return panel.child("Open an existing project with --project-store and --project-id.");
    };
    let tree = Hierarchy::build(project, &shell.session.expanded);
    for (index, row) in tree.rows.into_iter().enumerate() {
        let selection = row.selection;
        let selected = shell.session.selected.as_ref() == Some(&selection);
        let mut line = div().flex().gap_1().pl(px((row.depth.min(20) * 12) as f32));
        if row.expandable {
            let target = selection.clone();
            line = line.child(
                button(
                    ("expand", index),
                    if shell.session.expanded.contains(&selection) {
                        "−"
                    } else {
                        "+"
                    },
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if !this.session.expanded.remove(&target) {
                        this.session.expanded.insert(target.clone());
                    }
                    cx.notify();
                })),
            );
        }
        line = line.child(
            div()
                .id(("select", index))
                .flex_1()
                .min_w_0()
                .py_1()
                .text_xs()
                .cursor_pointer()
                .bg(if selected { colors.accent } else { colors.card })
                .child(row.label)
                .on_click(cx.listener(move |this, _, _, cx| this.select(selection.clone(), cx))),
        );
        panel = panel.child(line);
    }
    if tree.truncated {
        panel =
            panel.child("Display limit: 4096 rows. Collapse branches to inspect other content.");
    }
    panel
}
