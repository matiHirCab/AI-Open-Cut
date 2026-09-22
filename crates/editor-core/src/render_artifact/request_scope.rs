//! Immutable per-request artifact identity over the existing I/O adapter.
use super::*;

#[derive(Debug)]
struct RequestScope {
    inner: Arc<dyn ArtifactIo>,
    id: String,
}

pub(crate) fn with_request_id(
    inner: Arc<dyn ArtifactIo>,
    id: &str,
) -> Result<Arc<dyn ArtifactIo>, CoreError> {
    if !valid_request_id(id) {
        return Err(CoreError::new(
            crate::ErrorCode::InvalidArgument,
            "invalid artifact request identity",
        ));
    }
    Ok(Arc::new(RequestScope {
        inner,
        id: id.to_owned(),
    }))
}

impl ArtifactIo for RequestScope {
    fn request_id(&self) -> String {
        self.id.clone()
    }
    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        self.inner.create_dir(path)
    }
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        self.inner.remove_dir_all(path)
    }
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.inner.read(path)
    }
    fn read_font(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.inner.read_font(path)
    }
    fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
        self.inner.write(path, contents)
    }
    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.inner.list(path)
    }
    fn entry_kind(&self, path: &Path) -> std::io::Result<ArtifactEntryKind> {
        self.inner.entry_kind(path)
    }
    fn canonicalize_artifact_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        self.inner.canonicalize_artifact_path(path)
    }
    fn artifact_path_exists(&self, path: &Path) -> bool {
        self.inner.artifact_path_exists(path)
    }
    fn remove(&self, path: &Path) -> std::io::Result<()> {
        self.inner.remove(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        self.inner.rename(from, to)
    }
    fn size(&self, path: &Path) -> std::io::Result<u64> {
        self.inner.size(path)
    }
}
