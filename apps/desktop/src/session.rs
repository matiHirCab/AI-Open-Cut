use std::{collections::HashSet, ffi::OsString, path::PathBuf};

use opencut_editor_core::{CoreError, EditOperation, EditorCore, PathPolicy, Project};

use crate::hierarchy::Selection;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Startup {
    pub store: PathBuf,
    pub project_id: String,
}

impl Startup {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<Self>, String> {
        let mut store = None;
        let mut project_id = None;
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            match flag.to_str() {
                Some("--project-store") if store.is_none() => {
                    store = Some(PathBuf::from(
                        args.next().ok_or("--project-store requires a directory")?,
                    ))
                }
                Some("--project-id") if project_id.is_none() => {
                    project_id = Some(
                        args.next()
                            .ok_or("--project-id requires an ID")?
                            .into_string()
                            .map_err(|_| "project ID must be Unicode")?,
                    )
                }
                _ => {
                    return Err(
                        "Usage: opencut-desktop [--project-store <directory> --project-id <id>]"
                            .into(),
                    );
                }
            }
        }
        match (store, project_id) {
            (None, None) => Ok(None),
            (Some(store), Some(project_id)) => Ok(Some(Self { store, project_id })),
            _ => Err("--project-store and --project-id must be supplied together".into()),
        }
    }

    pub fn execute(&self, command: Command) -> Result<Project, CoreError> {
        // Inspection requires no import/export access outside the explicitly selected store.
        let core = EditorCore::new(PathPolicy::new(&self.store, [&self.store], &self.store)?);
        match command {
            Command::Refresh => {}
            Command::Edit(revision, edit) => {
                core.edit(&self.project_id, revision, *edit)?;
            }
            Command::Undo(revision) => {
                core.undo(&self.project_id, revision)?;
            }
            Command::Redo(revision) => {
                core.redo(&self.project_id, revision)?;
            }
        }
        core.get_project(&self.project_id)
    }
}

pub(crate) enum Command {
    Refresh,
    Edit(u64, Box<EditOperation>),
    Undo(u64),
    Redo(u64),
}

#[derive(Default)]
pub(crate) struct Session {
    pub project: Option<Project>,
    pub selected: Option<Selection>,
    pub expanded: HashSet<Selection>,
    pub error: Option<String>,
    pub busy: bool,
    generation: u64,
}

impl Session {
    pub fn begin(&mut self) -> Option<u64> {
        if self.busy {
            return None;
        }
        self.busy = true;
        self.generation += 1;
        self.error = None;
        Some(self.generation)
    }

    pub fn finish(&mut self, generation: u64, result: Result<Project, CoreError>) {
        if generation != self.generation {
            return;
        }
        self.busy = false;
        match result {
            Ok(project) => {
                if self
                    .selected
                    .as_ref()
                    .is_some_and(|s| s.resolve(&project).is_none())
                {
                    self.selected = None;
                }
                self.expanded.retain(|s| s.resolve(&project).is_some());
                self.project = Some(project);
                self.error = None;
            }
            Err(error) => {
                let code = serde_json::to_value(error.code).expect("serialize error code");
                self.error = Some(format!(
                    "{}: {} (retryable: {}). Refresh to read current state.",
                    code.as_str().unwrap_or("UNKNOWN"),
                    error.message,
                    error.retryable
                ));
            }
        }
    }
}

pub(crate) fn parse_z_index(text: &str) -> Result<i32, String> {
    text.parse()
        .map_err(|_| "Z-index must be an integer from -2147483648 to 2147483647.".into())
}
