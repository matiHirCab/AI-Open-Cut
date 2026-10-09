use gpui::{Context, Entity, FocusHandle, KeyDownEvent, Window, div, prelude::*};
use opencut_editor_core::{EditOperation, ParentReference};

use crate::{
    animation_inspector::{self, Cursor, DraftIdentity},
    compositing_inspector::{self, Action, Cursor as CompositingCursor},
    hierarchy::{Selection, editable},
    inspector_edit::{self, Field},
    narration_inspector,
    panels::{self, Preview},
    session::{Command, Session, Startup, parse_z_index},
    theme::ActiveTheme,
};

#[derive(Clone, Debug)]
pub(crate) struct InspectorDraft {
    pub source: DraftIdentity,
    pub compositing: CompositingCursor,
    pub path: String,
    pub update_key: &'static str,
}

impl InspectorDraft {
    pub(crate) fn matches(
        &self,
        project: &opencut_editor_core::Project,
        selection: Option<&Selection>,
        animation: Cursor,
        compositing: &CompositingCursor,
        field: &Field,
    ) -> bool {
        self.source.matches(project, selection, animation)
            && &self.compositing == compositing
            && self.path == field.path
            && self.update_key == field.update_key
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ActionContext {
    pub source: DraftIdentity,
    pub compositing: CompositingCursor,
    pub epoch: u64,
}
impl ActionContext {
    pub(crate) fn matches(
        &self,
        project: &opencut_editor_core::Project,
        selection: Option<&Selection>,
        animation: Cursor,
        compositing: &CompositingCursor,
        epoch: u64,
    ) -> bool {
        self.epoch == epoch
            && &self.compositing == compositing
            && self.source.matches(project, selection, animation)
    }
}

pub(crate) struct Shell {
    startup: Option<Startup>,
    pub session: Session,
    pub z_text: String,
    pub z_focus: FocusHandle,
    pub inspector_focus: FocusHandle,
    pub inspector_field: Option<usize>,
    pub inspector_text: String,
    inspector_source: Option<InspectorDraft>,
    pub animation_cursor: Cursor,
    pub compositing_cursor: CompositingCursor,
    pub narration_cursor: narration_inspector::Cursor,
    interaction_epoch: u64,
    preview: Entity<Preview>,
}

impl Shell {
    pub fn new(startup: Option<Startup>, cx: &mut Context<Self>) -> Self {
        let mut shell = Self {
            startup,
            session: Session::default(),
            z_text: String::new(),
            z_focus: cx.focus_handle(),
            inspector_focus: cx.focus_handle(),
            inspector_field: None,
            inspector_text: String::new(),
            inspector_source: None,
            animation_cursor: Cursor::default(),
            compositing_cursor: CompositingCursor::default(),
            narration_cursor: narration_inspector::Cursor::default(),
            interaction_epoch: 0,
            preview: cx.new(|_| Preview),
        };
        shell.dispatch(Command::Refresh, cx);
        shell
    }

    pub fn dispatch(&mut self, command: Command, cx: &mut Context<Self>) {
        self.dispatch_compositing(command, None, cx);
    }

    fn dispatch_compositing(
        &mut self,
        command: Command,
        next: Option<CompositingCursor>,
        cx: &mut Context<Self>,
    ) {
        if self.session.needs_refresh && !matches!(&command, Command::Refresh) {
            cx.notify();
            return;
        }
        let Some(startup) = self.startup.clone() else {
            return;
        };
        let Some(generation) = self.session.begin() else {
            return;
        };
        self.reset_inspector();
        let context = self
            .session
            .selected
            .as_ref()
            .zip(self.session.project.as_ref())
            .map(|(selection, project)| ActionContext {
                source: DraftIdentity {
                    selection: selection.clone(),
                    revision: project.revision,
                    cursor: self.animation_cursor,
                },
                compositing: self.compositing_cursor.clone(),
                epoch: self.interaction_epoch,
            });
        let work = cx
            .background_executor()
            .spawn(async move { startup.execute(command) });
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let _ = this.update(cx, |this, cx| {
                let succeeded = result.is_ok();
                let same_context = context
                    .as_ref()
                    .zip(this.session.project.as_ref())
                    .is_some_and(|(context, project)| {
                        context.matches(
                            project,
                            this.session.selected.as_ref(),
                            this.animation_cursor,
                            &this.compositing_cursor,
                            this.interaction_epoch,
                        )
                    });
                this.session.finish(generation, result);
                if succeeded {
                    this.narration_cursor.reset();
                }
                if succeeded
                    && same_context
                    && let Some(next) = next
                {
                    this.compositing_cursor = next;
                }
                this.resolve_compositing_cursor();
                this.reset_z_text();
                if succeeded {
                    this.reset_inspector();
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub fn select(&mut self, selection: Selection, cx: &mut Context<Self>) {
        self.narration_cursor.reset();
        self.session.selected = Some(selection);
        self.animation_cursor = Cursor::default();
        self.compositing_cursor = CompositingCursor::default();
        self.resolve_compositing_cursor();
        self.reset_z_text();
        self.reset_inspector();
        cx.notify();
    }

    pub fn navigate_narration(&mut self, axis: u8, next: bool, cx: &mut Context<Self>) {
        if let Some(project) = &self.session.project {
            let item = self
                .session
                .selected
                .as_ref()
                .and_then(|selection| selection.resolve(project))
                .map(|(_, item)| item);
            self.narration_cursor.navigate(project, item, axis, next);
            cx.notify();
        }
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

    pub fn reset_inspector(&mut self) {
        self.interaction_epoch += 1;
        self.inspector_field = None;
        self.inspector_text.clear();
        self.inspector_source = None;
    }

    fn resolve_compositing_cursor(&mut self) {
        if let Some((_, item)) = self
            .session
            .project
            .as_ref()
            .and_then(|p| self.session.selected.as_ref()?.resolve(p))
        {
            self.compositing_cursor.resolve(item);
        } else {
            self.compositing_cursor = CompositingCursor::default();
        }
    }

    pub fn change_compositing_cursor(&mut self, axis: u8, next: bool, cx: &mut Context<Self>) {
        if self.session.busy || self.session.needs_refresh {
            return;
        }
        if let Some((_, item)) = self
            .session
            .project
            .as_ref()
            .and_then(|p| self.session.selected.as_ref()?.resolve(p))
        {
            self.compositing_cursor.navigate(item, axis, next);
            self.reset_inspector();
            cx.notify();
        }
    }

    pub fn compositing_action(&mut self, action: Action, cx: &mut Context<Self>) {
        if self.session.busy || self.session.needs_refresh {
            return;
        }
        let Some(project) = self.session.project.as_ref() else {
            return;
        };
        let Some(selection) = self.session.selected.as_ref() else {
            return;
        };
        let Some((_, item)) = selection.resolve(project) else {
            return;
        };
        match compositing_inspector::action(
            project,
            selection,
            item,
            &self.compositing_cursor,
            action,
        ) {
            Ok((edit, next)) => self.dispatch_compositing(
                Command::Edit(project.revision, Box::new(edit)),
                Some(next),
                cx,
            ),
            Err(error) => {
                self.reset_inspector();
                self.session.error = Some(error);
                cx.notify();
            }
        }
    }

    pub fn choose_inspector_field(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.session.busy || self.session.needs_refresh {
            return;
        }
        self.resolve_compositing_cursor();
        let Some(project) = &self.session.project else {
            return;
        };
        let Some(selection) = &self.session.selected else {
            return;
        };
        if !animation_inspector::editable(selection) {
            return;
        }
        let Some((_, item)) = selection.resolve(project) else {
            return;
        };
        let Some(field) = self.inspector_fields(item).get(index).cloned() else {
            return;
        };
        self.inspector_source = Some(InspectorDraft {
            source: DraftIdentity {
                selection: selection.clone(),
                revision: project.revision,
                cursor: self.animation_cursor,
            },
            compositing: self.compositing_cursor.clone(),
            path: field.path.clone(),
            update_key: field.update_key,
        });
        self.inspector_field = Some(index);
        self.inspector_text = field.value;
        cx.notify();
    }

    fn active_inspector_field(&self) -> Option<(&opencut_editor_core::TimelineItem, Field)> {
        let identity = self.inspector_source.as_ref()?;
        let project = self.session.project.as_ref()?;
        if !identity.source.matches(
            project,
            self.session.selected.as_ref(),
            self.animation_cursor,
        ) {
            return None;
        }
        if identity.compositing != self.compositing_cursor {
            return None;
        }
        let (_, item) = identity.source.selection.resolve(project)?;
        let field = self
            .inspector_fields(item)
            .get(self.inspector_field?)?
            .clone();
        if !identity.matches(
            project,
            self.session.selected.as_ref(),
            self.animation_cursor,
            &self.compositing_cursor,
            &field,
        ) {
            return None;
        }
        Some((item, field))
    }

    pub(crate) fn inspector_fields(&self, item: &opencut_editor_core::TimelineItem) -> Vec<Field> {
        let Some(project) = self.session.project.as_ref() else {
            return vec![];
        };
        let Some(selection) = self.session.selected.as_ref() else {
            return vec![];
        };
        if !animation_inspector::editable(selection) {
            return vec![];
        }
        let mut fields = if editable(project, selection) {
            inspector_edit::fields(item)
        } else {
            vec![]
        };
        fields.extend(animation_inspector::fields(
            item,
            self.animation_cursor,
            animation_inspector::audio_only(project, item),
        ));
        fields.extend(compositing_inspector::fields(
            project,
            selection,
            item,
            &self.compositing_cursor,
        ));
        fields
    }

    pub fn change_animation_cursor(&mut self, axis: u8, next: bool, cx: &mut Context<Self>) {
        if self.session.busy || self.session.needs_refresh {
            return;
        }
        let Some((_, item)) = self
            .session
            .project
            .as_ref()
            .and_then(|p| self.session.selected.as_ref()?.resolve(p))
        else {
            return;
        };
        let (index, len) = match axis {
            0 => (
                &mut self.animation_cursor.channel,
                item.visual_properties().animation_channels.len(),
            ),
            1 => (
                &mut self.animation_cursor.key,
                item.visual_properties()
                    .animation_channels
                    .get(
                        self.animation_cursor.channel.min(
                            item.visual_properties()
                                .animation_channels
                                .len()
                                .saturating_sub(1),
                        ),
                    )
                    .map_or(0, |c| c.keyframes.len()),
            ),
            _ => (&mut self.animation_cursor.legacy, item.keyframes().len()),
        };
        *index = (*index).min(len.saturating_sub(1));
        *index = if next {
            index.saturating_add(1).min(len.saturating_sub(1))
        } else {
            index.saturating_sub(1)
        };
        if axis == 0 {
            self.animation_cursor.key = 0;
        }
        self.reset_inspector();
        cx.notify();
    }

    pub fn apply_inspector(&mut self, cx: &mut Context<Self>) {
        if self.session.busy || self.session.needs_refresh {
            return;
        }
        let Some((item, field)) = self.active_inspector_field() else {
            self.session.error = Some("Inspector draft is stale. Select a field again.".into());
            cx.notify();
            return;
        };
        match inspector_edit::build(item, &field, &self.inspector_text) {
            Ok(edit) => {
                let revision = self.session.project.as_ref().unwrap().revision;
                self.dispatch(Command::Edit(revision, Box::new(edit)), cx);
            }
            Err(error) => {
                self.session.error = Some(error);
                cx.notify();
            }
        }
    }

    pub fn inspector_key(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.session.busy
            || self.session.needs_refresh
            || self.active_inspector_field().is_none()
        {
            return;
        }
        match event.keystroke.key.as_str() {
            "enter" if event.keystroke.modifiers.shift => self.inspector_text.push('\n'),
            "enter" => self.apply_inspector(cx),
            "escape" => self.reset_inspector(),
            "backspace" => {
                self.inspector_text.pop();
            }
            "a" if event.keystroke.modifiers.control || event.keystroke.modifiers.platform => {
                self.inspector_text.clear()
            }
            _ => {
                if let Some(text) = &event.keystroke.key_char
                    && self.inspector_text.len() + text.len() <= 4096
                {
                    self.inspector_text.push_str(text);
                }
            }
        }
        cx.notify();
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
        if self.session.busy || self.session.needs_refresh {
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
        if self.session.needs_refresh {
            toolbar = toolbar.child(div().id("refresh-required").child("Refresh required"));
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
