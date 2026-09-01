//! Deterministic persisted-schema migration owner.

use crate::{CoreError, ErrorCode, History, PROJECT_SCHEMA_VERSION, Project};

pub(crate) fn migrate_project_documents(
    project: &mut Project,
    history: &mut History,
) -> Result<bool, CoreError> {
    let mut changed = migrate_project(project)?;
    for snapshot in history.undo.iter_mut().chain(&mut history.redo) {
        changed |= migrate_project(snapshot)?;
    }
    Ok(changed)
}

fn migrate_project(project: &mut Project) -> Result<bool, CoreError> {
    match project.schema_version {
        1..=5 => {
            project.schema_version = PROJECT_SCHEMA_VERSION;
            Ok(true)
        }
        PROJECT_SCHEMA_VERSION => Ok(false),
        version => Err(CoreError::new(
            ErrorCode::InternalError,
            format!(
                "unsupported project schema version {version}; this build supports up to {PROJECT_SCHEMA_VERSION}"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProjectSettings;

    fn project(schema_version: u32) -> Project {
        Project {
            schema_version,
            id: "project".into(),
            revision: 0,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![],
        }
    }

    #[test]
    fn migrates_current_and_retained_history_together() {
        let mut current = project(1);
        let mut history = History {
            undo: vec![project(2)],
            redo: vec![project(5)],
        };
        assert!(migrate_project_documents(&mut current, &mut history).unwrap());
        assert_eq!(current.schema_version, PROJECT_SCHEMA_VERSION);
        assert!(
            history
                .undo
                .iter()
                .chain(&history.redo)
                .all(|value| value.schema_version == PROJECT_SCHEMA_VERSION)
        );
        assert!(!migrate_project_documents(&mut current, &mut history).unwrap());
    }

    #[test]
    fn rejects_unknown_future_schema_without_rewrite() {
        let future = PROJECT_SCHEMA_VERSION + 1;
        let mut current = project(future);
        let mut history = History::default();
        assert_eq!(
            migrate_project_documents(&mut current, &mut history)
                .unwrap_err()
                .code,
            ErrorCode::InternalError
        );
        assert_eq!(current.schema_version, future);
    }
}
