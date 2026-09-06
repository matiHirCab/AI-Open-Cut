use gpui::{Context, Entity, FocusHandle, KeyDownEvent, Window, div, prelude::*};
use opencut_editor_core::{EditOperation, ParentReference};

use crate::{
    hierarchy::{Selection, editable},
    panels::{self, Preview},
    session::{Command, Session, Startup, parse_z_index},
    theme::ActiveTheme,
};

pub(crate) struct Shell {
    startup: Option<Startup>,
    pub session: Session,
    pub z_text: String,
    pub z_focus: FocusHandle,
    preview: Entity<Preview>,
}

impl Shell {
    pub fn new(startup: Option<Startup>, cx: &mut Context<Self>) -> Self {
        let mut shell = Self {
            startup,
            session: Session::default(),
            z_text: String::new(),
            z_focus: cx.focus_handle(),
            preview: cx.new(|_| Preview),
        };
        shell.dispatch(Command::Refresh, cx);
        shell
    }

    pub fn dispatch(&mut self, command: Command, cx: &mut Context<Self>) {
        let Some(startup) = self.startup.clone() else {
            return;
        };
        let Some(generation) = self.session.begin() else {
            return;
        };
        let work = cx
            .background_executor()
            .spawn(async move { startup.execute(command) });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                this.session.finish(generation, result);
                this.reset_z_text();
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn select(&mut self, selection: Selection, cx: &mut Context<Self>) {
        self.session.selected = Some(selection);
        self.reset_z_text();
        cx.notify();
    }

    fn reset_z_text(&mut self) {
        self.z_text = self
            .session
            .project
            .as_ref()
            .and_then(|p| self.session.selected.as_ref()?.resolve(p))
            .map(|(_, i)| i.visual_properties().z_index.to_string())
            .unwrap_or_default();
    }

    pub fn set_parent(&mut self, parent: Option<String>, cx: &mut Context<Self>) {
        self.edit_selected(
            |item_id| EditOperation::ItemSetParent {
                item_id,
                parent: parent.map(|id| ParentReference {
                    scope: "root".into(),
                    id,
                }),
            },
            cx,
        );
    }

    fn edit_selected(
        &mut self,
        edit: impl FnOnce(String) -> EditOperation,
        cx: &mut Context<Self>,
    ) {
        if let (Some(project), Some(selected)) = (&self.session.project, &self.session.selected)
            && editable(project, selected)
        {
            self.dispatch(
                Command::Edit(project.revision, Box::new(edit(selected.item_id.clone()))),
                cx,
            );
        }
    }

    pub fn apply_z(&mut self, cx: &mut Context<Self>) {
        match parse_z_index(&self.z_text) {
            Ok(z_index) => self.edit_selected(
                |item_id| EditOperation::ItemSetZIndex { item_id, z_index },
                cx,
            ),
            Err(error) => {
                self.session.error = Some(error);
                cx.notify();
            }
        }
    }

    pub fn z_key(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.session.busy {
            return;
        }
        match event.keystroke.key.as_str() {
            "enter" => self.apply_z(cx),
            "escape" => self.reset_z_text(),
            "backspace" => {
                self.z_text.pop();
            }
            "a" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                self.z_text.clear()
            }
            _ => {
                if let Some(text) = &event.keystroke.key_char
                    && text.chars().all(|c| c.is_ascii_digit() || c == '-')
                    && self.z_text.len() < 12
                {
                    self.z_text.push_str(text);
                }
            }
        }
        cx.notify();
    }
}

pub(crate) fn button(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .border_1()
        .rounded_md()
        .cursor_pointer()
        .child(label.into())
}

impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = window.theme().colors;
        let revision = self.session.project.as_ref().map(|p| p.revision);
        let title = self
            .session
            .project
            .as_ref()
            .map(|p| format!("{} · revision {}", p.name, p.revision))
            .unwrap_or_else(|| "No project loaded".into());
        let mut toolbar = div().flex().gap_2().p_2().child(title);
        if self.startup.is_some() {
            toolbar = toolbar.child(
                button("refresh", "Refresh")
                    .on_click(cx.listener(|this, _, _, cx| this.dispatch(Command::Refresh, cx))),
            );
        }
        if let Some(revision) = revision {
            toolbar = toolbar
                .child(button("undo", "Undo").on_click(
                    cx.listener(move |this, _, _, cx| this.dispatch(Command::Undo(revision), cx)),
                ))
                .child(button("redo", "Redo").on_click(
                    cx.listener(move |this, _, _, cx| this.dispatch(Command::Redo(revision), cx)),
                ));
        }
        if self.session.busy {
            toolbar = toolbar.child("Working…");
        }
        let mut view = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(toolbar);
        if let Some(error) = &self.session.error {
            view = view.child(
                div()
                    .p_2()
                    .text_color(colors.destructive)
                    .child(error.clone()),
            );
        }
        view.child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .border_b_1()
                .border_color(colors.border)
                .child(panels::browser::render(self, window, cx))
                .child(self.preview.clone())
                .child(panels::inspector::render(self, window, cx)),
        )
        .child(panels::timeline::render(self, window, cx))
    }
}
