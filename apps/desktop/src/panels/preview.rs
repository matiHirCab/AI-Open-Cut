use gpui::{Context, Window, div, img, prelude::*};
use opencut_editor_core::PreviewPreset;

use crate::{
    review::Request,
    shell::{Shell, button},
    theme::ActiveTheme,
};

pub(crate) fn render(shell: &Shell, window: &Window, cx: &mut Context<Shell>) -> impl IntoElement {
    let colors = window.theme().colors;
    let mut panel = div()
        .id("review")
        .track_focus(&shell.review_focus)
        .on_key_down(cx.listener(Shell::review_key))
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .h_full()
        .overflow_y_scroll()
        .p_2()
        .gap_2()
        .bg(colors.background)
        .child(div().text_lg().child("Preview review"));
    let Some(project) = &shell.session.project else {
        return panel.child("Load a project to review its rendered scene.");
    };
    panel = panel.child(div().text_xs().child(format!(
        "Project milliseconds · {} fps · revision {} · duration {} ms",
        project.settings.fps,
        project.revision,
        project.duration_ms()
    )));
    for (index, label) in ["Frame time", "Range start", "Range end"]
        .into_iter()
        .enumerate()
    {
        panel = panel.child(
            div()
                .id(("review-time", index))
                .border_1()
                .border_color(colors.border)
                .p_1()
                .text_sm()
                .child(format!(
                    "{label}: {} ms{}",
                    shell.review.values[index],
                    if shell.review.field == Some(index) {
                        " ▏"
                    } else {
                        ""
                    }
                ))
                .on_click(cx.listener(move |this, _, window, cx| {
                    if !this.session.busy && !this.session.needs_refresh {
                        this.review.field = Some(index);
                        this.review_focus.focus(window);
                        cx.notify();
                    }
                })),
        );
    }
    let mut presets = div().flex().gap_1();
    for (index, (preset, name)) in [
        (PreviewPreset::P540, "540p"),
        (PreviewPreset::P720, "720p"),
        (PreviewPreset::Project, "Project"),
    ]
    .into_iter()
    .enumerate()
    {
        presets = presets.child(
            button(
                ("review-preset", index),
                if shell.review.preset == preset {
                    format!("[{name}]")
                } else {
                    name.into()
                },
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                if !this.session.busy && !this.session.needs_refresh {
                    this.review.preset = preset;
                    cx.notify();
                }
            })),
        );
    }
    panel = panel
        .child(presets)
        .child(
            button(
                "review-audio",
                format!(
                    "Range audio: {}",
                    if shell.review.include_audio {
                        "on"
                    } else {
                        "off"
                    }
                ),
            )
            .on_click(cx.listener(|this, _, _, cx| {
                if !this.session.busy && !this.session.needs_refresh {
                    this.review.include_audio = !this.review.include_audio;
                    cx.notify();
                }
            })),
        )
        .child(
            div()
                .flex()
                .gap_1()
                .child(
                    button("review-frame", "Render frame")
                        .on_click(cx.listener(|this, _, _, cx| this.request_review(false, cx))),
                )
                .child(
                    button("review-range", "Render range")
                        .on_click(cx.listener(|this, _, _, cx| this.request_review(true, cx))),
                ),
        );
    if shell.session.busy {
        panel = panel.child("Working… review and edits are serialized.");
    }
    if shell.session.needs_refresh {
        panel = panel.child("Refresh required before another review or edit.");
    }
    if let Some(message) = &shell.review.feedback {
        panel = panel.child(div().text_color(colors.destructive).child(message.clone()));
    }
    for artifact in [&shell.review.frame, &shell.review.range]
        .into_iter()
        .flatten()
    {
        let options = match artifact.request {
            Request::Frame(time) => format!("Frame {time} ms"),
            Request::Range(options) => format!(
                "Range {}–{} ms · {}×{} · {} fps · audio {}",
                options.start_ms,
                options.end_ms,
                artifact.dimensions.0,
                artifact.dimensions.1,
                artifact.fps,
                options.include_audio.unwrap_or(true)
            ),
        };
        panel = panel
            .child(div().text_sm().child(format!(
                "{options} · captured revision {}{}",
                artifact.revision,
                if artifact.stale(project) {
                    " · STALE — render current revision"
                } else {
                    ""
                }
            )))
            .child(div().text_xs().whitespace_normal().child(format!(
                "{} · {} · {} bytes",
                artifact.rendered.relative_path,
                artifact.rendered.mime_type,
                artifact.rendered.size_bytes
            )));
        for warning in &artifact.rendered.warnings {
            panel = panel.child(div().text_xs().child(warning.clone()));
        }
        if matches!(artifact.request, Request::Frame(_)) {
            panel = panel.child(img(artifact.path.clone()).w_full().h_64().flex_shrink_0());
        } else {
            let path = artifact.path.clone();
            panel = panel.child(
                button("review-reveal", "Reveal MP4 for external playback")
                    .on_click(cx.listener(move |_, _, _, cx| cx.reveal_path(&path))),
            );
        }
    }
    panel
}
