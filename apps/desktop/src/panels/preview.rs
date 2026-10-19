use gpui::{Context, Window, div, img, prelude::*};
use opencut_editor_core::PreviewPreset;
use std::time::Duration;

use crate::{
    review::{
        Request,
        playback::{Command, Phase, Player, Presentation},
    },
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
            if let Some(playback) = &shell.review.playback
                && playback.player.matches(artifact)
            {
                let state = playback.player.snapshot();
                let status = match state.phase {
                    Phase::Loading => "Loading",
                    Phase::Paused => "Paused",
                    Phase::Playing => "Playing",
                    Phase::Ended => "Ended",
                    Phase::Error(fault) => fault.message(),
                    Phase::Closed => "Stopped",
                };
                panel = panel.child(div().text_sm().child(format!(
                    "{status} · {} / {} ms · captured project {} ms",
                    state.position_ms,
                    playback.player.duration_ms,
                    playback.player.start_ms.saturating_add(state.position_ms)
                )));
                if let Some(image) = &playback.rendered {
                    panel = panel.child(img(image.clone()).w_full().h_64().flex_shrink_0());
                }
                if let Some(message) = playback.feedback {
                    panel = panel.child(div().text_color(colors.destructive).child(message));
                }
                if state.phase.controls_available() {
                    let paused = state.phase == Phase::Playing;
                    let rewind = state.position_ms.saturating_sub(1000);
                    let forward = state.position_ms.saturating_add(1000);
                    let looping = !state.looping;
                    panel = panel.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_1()
                            .child(
                                button("review-play-pause", if paused { "Pause" } else { "Play" })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if let Some(p) = &mut this.review.playback {
                                            let result = if p.player.snapshot().phase
                                                == Phase::Ended
                                                && !paused
                                            {
                                                p.player.send(Command::Seek(0)).and_then(|()| {
                                                    p.player.send(Command::Pause(false))
                                                })
                                            } else {
                                                p.player.send(Command::Pause(paused))
                                            };
                                            p.feedback = result.err().map(|e| e.message());
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                button("review-seek-back", "Seek −1 s").on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.playback_command(Command::Seek(rewind), cx)
                                    },
                                )),
                            )
                            .child(
                                button("review-seek-start", "Seek start").on_click(cx.listener(
                                    |this, _, _, cx| this.playback_command(Command::Seek(0), cx),
                                )),
                            )
                            .child(button("review-seek-forward", "Seek +1 s").on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.playback_command(Command::Seek(forward), cx)
                                }),
                            ))
                            .child(
                                button(
                                    "review-loop",
                                    if state.looping {
                                        "Loop: on"
                                    } else {
                                        "Loop: off"
                                    },
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.playback_command(Command::Loop(looping), cx)
                                    },
                                )),
                            ),
                    );
                }
                panel = panel.child(button("review-playback-stop", "Close player").on_click(
                    cx.listener(|this, _, window, cx| {
                        this.stop_playback(Some(window), cx);
                        cx.notify();
                    }),
                ));
            } else {
                panel = panel.child(
                    button("review-playback-open", "Open embedded player").on_click(cx.listener(
                        |this, _, window, cx| {
                            this.open_playback(window, cx);
                        },
                    )),
                );
            }
            panel = panel.child(
                button("review-reveal", "Reveal MP4 for external playback")
                    .on_click(cx.listener(move |_, _, _, cx| cx.reveal_path(&path))),
            );
        }
    }
    panel
}

impl Shell {
    fn playback_command(&mut self, command: Command, cx: &mut Context<Self>) {
        if let Some(player) = &mut self.review.playback {
            player.feedback = player.player.send(command).err().map(|e| e.message());
        }
        cx.notify();
    }

    fn stop_playback(&mut self, window: Option<&mut Window>, cx: &mut gpui::App) {
        if let Some(mut playback) = self.review.playback.take() {
            playback.player.stop();
            playback.evict(window, cx);
        }
    }

    fn open_playback(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_playback(Some(&mut *window), cx);
        let Some(artifact) = &self.review.range else {
            return;
        };
        let player = match Player::open(artifact) {
            Ok(player) => player,
            Err(fault) => {
                self.review.feedback = Some(fault.message().into());
                cx.notify();
                return;
            }
        };
        self.review.feedback = None;
        let playback = Presentation::new(player);
        let generation = playback.id;
        self.review.playback = Some(playback);
        if self.review.playback_release.is_none() {
            self.review.playback_release =
                Some(cx.on_release(|this, cx| this.stop_playback(None, cx)));
        }
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                let keep = this.update_in(cx, |this, window, cx| {
                    let Some(playback) = &mut this.review.playback else {
                        return false;
                    };
                    if playback.id != generation {
                        return false;
                    }
                    let retained = this
                        .review
                        .range
                        .as_ref()
                        .is_some_and(|a| playback.player.matches(a));
                    let same_project = this
                        .session
                        .project
                        .as_ref()
                        .is_some_and(|p| p.id == playback.player.project_id);
                    if !retained || !same_project {
                        this.stop_playback(Some(window), cx);
                        cx.notify();
                        return false;
                    }
                    if playback.update_image(window, cx) {
                        cx.notify();
                    }
                    true
                });
                if !matches!(keep, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }
}
