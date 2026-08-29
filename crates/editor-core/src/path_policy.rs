use std::path::{Component, Path, PathBuf};

use crate::{CoreError, ErrorCode};

#[derive(Clone, Debug)]
pub struct PathPolicy {
    projects_root: PathBuf,
    media_roots: Vec<PathBuf>,
    exports_root: PathBuf,
    generated_media_roots: Vec<PathBuf>,
}

impl PathPolicy {
    pub fn new(
        projects_root: impl AsRef<Path>,
        media_roots: impl IntoIterator<Item = impl AsRef<Path>>,
        exports_root: impl AsRef<Path>,
    ) -> Result<Self, CoreError> {
        let projects_root = canonical_directory(projects_root.as_ref(), "projects root")?;
        let exports_root = canonical_directory(exports_root.as_ref(), "exports root")?;
        let media_roots = media_roots
            .into_iter()
            .map(|root| canonical_directory(root.as_ref(), "media root"))
            .collect::<Result<Vec<_>, _>>()?;
        if media_roots.is_empty() {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "at least one allowed media root is required",
            ));
        }
        Ok(Self {
            projects_root,
            media_roots,
            exports_root,
            generated_media_roots: vec![],
        })
    }

    pub fn with_generated_media_root(
        mut self,
        generated_media_root: impl AsRef<Path>,
    ) -> Result<Self, CoreError> {
        self.generated_media_roots.push(canonical_directory(
            generated_media_root.as_ref(),
            "generated media root",
        )?);
        Ok(self)
    }

    pub fn with_generated_media_roots(
        mut self,
        roots: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> Result<Self, CoreError> {
        for root in roots {
            let root = canonical_directory(root.as_ref(), "generated media root")?;
            if !self.generated_media_roots.contains(&root) {
                self.generated_media_roots.push(root);
            }
        }
        Ok(self)
    }

    pub fn projects_root(&self) -> &Path {
        &self.projects_root
    }

    pub fn exports_root(&self) -> &Path {
        &self.exports_root
    }

    pub fn import_path(&self, requested: impl AsRef<Path>) -> Result<PathBuf, CoreError> {
        reject_parent_components(requested.as_ref())?;
        let resolved = requested
            .as_ref()
            .canonicalize()
            .map_err(|error| CoreError::io("cannot resolve media path", error))?;
        if !resolved.is_file() {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "media path must identify a file",
            ));
        }
        if !self
            .media_roots
            .iter()
            .any(|root| resolved.starts_with(root))
        {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "media path is outside the configured roots",
            ));
        }
        Ok(resolved)
    }

    pub fn generated_media_path(&self, requested: impl AsRef<Path>) -> Result<PathBuf, CoreError> {
        if self.generated_media_roots.is_empty() {
            return Err(CoreError::new(
                ErrorCode::DependencyUnavailable,
                "generated media roots are not configured",
            ));
        }
        reject_parent_components(requested.as_ref())?;
        let resolved = requested
            .as_ref()
            .canonicalize()
            .map_err(|error| CoreError::io("cannot resolve generated media", error))?;
        if !resolved.is_file()
            || !self
                .generated_media_roots
                .iter()
                .any(|root| resolved.parent() == Some(root.as_path()))
        {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "generated media must be a direct child of a configured root",
            ));
        }
        Ok(resolved)
    }

    pub fn project_dir(&self, project_id: &str) -> Result<PathBuf, CoreError> {
        validate_identifier(project_id)?;
        Ok(self.projects_root.join(project_id))
    }

    pub fn export_path(&self, relative: impl AsRef<Path>) -> Result<PathBuf, CoreError> {
        let relative = relative.as_ref();
        if relative.is_absolute() {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "export path must be relative to the configured export root",
            ));
        }
        reject_parent_components(relative)?;
        if relative.as_os_str().is_empty() {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "export path cannot be empty",
            ));
        }
        let candidate = self.exports_root.join(relative);
        let parent = candidate.parent().ok_or_else(|| {
            CoreError::new(ErrorCode::PathNotAllowed, "export path has no parent")
        })?;
        std::fs::create_dir_all(parent)
            .map_err(|error| CoreError::io("cannot create export directory", error))?;
        let parent = parent
            .canonicalize()
            .map_err(|error| CoreError::io("cannot resolve export directory", error))?;
        if !parent.starts_with(&self.exports_root) {
            return Err(CoreError::new(
                ErrorCode::PathNotAllowed,
                "export path escapes the configured export root",
            ));
        }
        let file_name = candidate.file_name().ok_or_else(|| {
            CoreError::new(ErrorCode::InvalidArgument, "export file name is missing")
        })?;
        let resolved = parent.join(file_name);
        if resolved.exists() {
            let canonical = resolved
                .canonicalize()
                .map_err(|error| CoreError::io("cannot resolve export target", error))?;
            if !canonical.starts_with(&self.exports_root) {
                return Err(CoreError::new(
                    ErrorCode::PathNotAllowed,
                    "export target escapes the configured export root",
                ));
            }
        }
        Ok(resolved)
    }
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf, CoreError> {
    std::fs::create_dir_all(path)
        .map_err(|error| CoreError::io(&format!("cannot create {label}"), error))?;
    path.canonicalize()
        .map_err(|error| CoreError::io(&format!("cannot resolve {label}"), error))
}

fn reject_parent_components(path: &Path) -> Result<(), CoreError> {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(CoreError::new(
            ErrorCode::PathTraversal,
            "parent-directory traversal is not allowed",
        ));
    }
    Ok(())
}

fn validate_identifier(value: &str) -> Result<(), CoreError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "project ID contains unsupported characters",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn enforces_import_and_export_roots() {
        let root = tempdir().unwrap();
        let projects = root.path().join("projects");
        let media = root.path().join("media");
        let exports = root.path().join("exports");
        std::fs::create_dir_all(&media).unwrap();
        std::fs::write(media.join("ok.png"), b"png").unwrap();
        let policy = PathPolicy::new(&projects, [&media], &exports).unwrap();

        assert!(policy.import_path(media.join("ok.png")).is_ok());
        assert_eq!(
            policy.export_path("../escape.mp4").unwrap_err().code,
            ErrorCode::PathTraversal
        );
        assert_eq!(
            policy
                .export_path(root.path().join("absolute.mp4"))
                .unwrap_err()
                .code,
            ErrorCode::PathNotAllowed
        );
    }

    #[test]
    fn confines_generated_media_to_its_direct_root() {
        let root = tempdir().unwrap();
        let media = root.path().join("media");
        let generated = root.path().join("generated");
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(generated.join("nested")).unwrap();
        std::fs::write(generated.join("speech.wav"), b"wav").unwrap();
        std::fs::write(generated.join("nested").join("speech.wav"), b"wav").unwrap();
        let policy = PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap()
        .with_generated_media_root(&generated)
        .unwrap();

        assert!(
            policy
                .generated_media_path(generated.join("speech.wav"))
                .is_ok()
        );
        assert_eq!(
            policy
                .generated_media_path(generated.join("nested").join("speech.wav"))
                .unwrap_err()
                .code,
            ErrorCode::PathNotAllowed
        );
    }
}
