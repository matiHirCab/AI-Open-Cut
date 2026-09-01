//! Durable persistence boundary and filesystem adapter.

use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{CoreError, ErrorCode, History, PROJECT_SCHEMA_VERSION, Project};

pub(crate) const TRANSACTION_VERSION: u32 = 1;
pub(crate) const TRANSACTION_FILE: &str = ".project-transaction.json";
pub(crate) const PERSISTENCE_RECOVERY_PENDING: &str = "PERSISTENCE_RECOVERY_PENDING";
pub(crate) const DRAFT_CLEANUP_FAILED: &str = "DRAFT_CLEANUP_FAILED";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ProjectTransaction {
    pub(crate) version: u32,
    pub(crate) project: Project,
    pub(crate) history: History,
    pub(crate) committed_draft_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PersistencePhase {
    BeforeJournal,
    AfterJournal,
    AfterProject,
    AfterHistory,
    AfterDraftCleanup,
    AfterJournalCleanup,
    GarbageCollection,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PersistenceFaults {
    #[cfg(test)]
    next: std::sync::Arc<std::sync::Mutex<Option<PersistencePhase>>>,
}

impl PersistenceFaults {
    pub(crate) fn checkpoint(&self, phase: PersistencePhase) -> Result<(), CoreError> {
        #[cfg(test)]
        {
            let mut next = self.next.lock().expect("persistence fault plan poisoned");
            if *next == Some(phase) {
                *next = None;
                return Err(CoreError::new(
                    crate::ErrorCode::InternalError,
                    format!("injected persistence fault after {phase:?}"),
                ));
            }
        }
        #[cfg(not(test))]
        let _ = phase;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn inject(&self, phase: PersistencePhase) {
        *self.next.lock().expect("persistence fault plan poisoned") = Some(phase);
    }
}

pub(crate) trait Storage: Debug + Send + Sync {
    fn lock_exclusive(&self, dir: &Path) -> Result<Box<dyn StorageLock>, CoreError>;
    fn open_read(&self, path: &Path) -> std::io::Result<Box<dyn Read>>;
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>>;
    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn copy(&self, from: &Path, to: &Path) -> std::io::Result<u64>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn storage_path_is_file(&self, path: &Path) -> bool;
    fn storage_path_exists(&self, path: &Path) -> bool;
    fn entry_kind(&self, path: &Path) -> std::io::Result<StorageEntryKind>;
    fn canonicalize_storage_path(&self, path: &Path) -> std::io::Result<PathBuf>;
    fn atomic_replace(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
    fn remove_durable(&self, path: &Path) -> std::io::Result<()>;
}

pub(crate) trait StorageLock: Debug + Send {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageEntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FileSystemStorage;

impl Storage for FileSystemStorage {
    fn lock_exclusive(&self, dir: &Path) -> Result<Box<dyn StorageLock>, CoreError> {
        Ok(Box::new(ProjectLock::exclusive(dir)?))
    }

    fn open_read(&self, path: &Path) -> std::io::Result<Box<dyn Read>> {
        File::open(path).map(|file| Box::new(file) as Box<dyn Read>)
    }

    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn list(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        std::fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect()
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> std::io::Result<u64> {
        std::fs::copy(from, to)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }

    fn storage_path_is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn storage_path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn entry_kind(&self, path: &Path) -> std::io::Result<StorageEntryKind> {
        let metadata = std::fs::symlink_metadata(path)?;
        let file_type = metadata.file_type();
        #[cfg(windows)]
        let is_link = {
            use std::os::windows::fs::MetadataExt;

            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            file_type.is_symlink() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        };
        #[cfg(not(windows))]
        let is_link = file_type.is_symlink();
        Ok(if is_link {
            StorageEntryKind::Symlink
        } else if file_type.is_file() {
            StorageEntryKind::File
        } else if file_type.is_dir() {
            StorageEntryKind::Directory
        } else {
            StorageEntryKind::Other
        })
    }

    fn canonicalize_storage_path(&self, path: &Path) -> std::io::Result<PathBuf> {
        path.canonicalize()
    }

    fn atomic_replace(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
        let result = (|| {
            let mut file = File::create(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            std::fs::rename(&temporary, path)?;
            sync_parent(path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }

    fn remove_durable(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_file(path)?;
        sync_parent(path)
    }
}

pub(crate) fn read_json<T: DeserializeOwned>(
    storage: &dyn Storage,
    path: &Path,
) -> Result<T, CoreError> {
    let bytes = storage
        .read(path)
        .map_err(|error| CoreError::io("cannot read persisted JSON", error))?;
    serde_json::from_slice(&bytes).map_err(CoreError::from)
}

pub(crate) fn list_paths(storage: &dyn Storage, path: &Path) -> Result<Vec<PathBuf>, CoreError> {
    storage
        .list(path)
        .map_err(|error| CoreError::io("cannot list persisted directory", error))
}

pub(crate) fn list_project_directories(
    storage: &dyn Storage,
    path: &Path,
) -> Result<Vec<PathBuf>, CoreError> {
    let entries = storage
        .list(path)
        .map_err(|error| CoreError::io("cannot list projects", error))?;
    entries
        .into_iter()
        .filter_map(|entry| match storage.entry_kind(&entry) {
            Ok(StorageEntryKind::Directory) => Some(Ok(entry)),
            Ok(_) => None,
            Err(error) => Some(Err(CoreError::io("cannot inspect project entry", error))),
        })
        .collect()
}

pub(crate) fn write_json_atomic<T: Serialize>(
    storage: &dyn Storage,
    path: &Path,
    value: &T,
) -> Result<(), CoreError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    storage
        .atomic_replace(path, &bytes)
        .map_err(|error| CoreError::io("cannot publish data", error))
}

pub(crate) fn remove_file_if_exists(storage: &dyn Storage, path: &Path) -> Result<(), CoreError> {
    if !storage.storage_path_exists(path) {
        return Ok(());
    }
    storage
        .remove_durable(path)
        .map_err(|error| CoreError::io("cannot remove file", error))
}

pub(crate) fn remove_file_durable(storage: &dyn Storage, path: &Path) -> Result<(), CoreError> {
    storage
        .remove_durable(path)
        .map_err(|error| CoreError::io("cannot remove file", error))
}

pub(crate) fn persist_transaction(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    project: &Project,
    history: &History,
    committed_draft_id: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    faults.checkpoint(PersistencePhase::BeforeJournal)?;
    let transaction = ProjectTransaction {
        version: TRANSACTION_VERSION,
        project: project.clone(),
        history: history.clone(),
        committed_draft_id: committed_draft_id.map(str::to_owned),
    };
    write_json_atomic(storage, &transaction_path(dir), &transaction)?;

    let mut warnings = Vec::new();
    if faults.checkpoint(PersistencePhase::AfterJournal).is_err()
        || write_json_atomic(storage, &project_path(dir), &transaction.project).is_err()
        || faults.checkpoint(PersistencePhase::AfterProject).is_err()
        || write_json_atomic(storage, &history_path(dir), &transaction.history).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        return Ok(warnings);
    }

    if faults.checkpoint(PersistencePhase::AfterHistory).is_err() {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        if transaction.committed_draft_id.is_some() {
            warnings.push(DRAFT_CLEANUP_FAILED.into());
        }
        return Ok(warnings);
    }

    if let Some(draft_id) = transaction.committed_draft_id.as_deref()
        && remove_file_if_exists(storage, &draft_path(dir, draft_id)?).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        warnings.push(DRAFT_CLEANUP_FAILED.into());
        return Ok(warnings);
    }

    if faults
        .checkpoint(PersistencePhase::AfterDraftCleanup)
        .is_err()
        || remove_file_durable(storage, &transaction_path(dir)).is_err()
    {
        warnings.push(PERSISTENCE_RECOVERY_PENDING.into());
        return Ok(warnings);
    }

    let _ = faults.checkpoint(PersistencePhase::AfterJournalCleanup);
    Ok(warnings)
}

pub(crate) fn recover_transaction(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
) -> Result<(), CoreError> {
    cleanup_orphaned_transaction_temps(storage, dir)?;
    let path = transaction_path(dir);
    if !storage.storage_path_exists(&path) {
        return Ok(());
    }
    let transaction = read_transaction(storage, &path)?;
    validate_transaction(dir, &transaction)?;
    replay_transaction(storage, faults, dir, &transaction)
}

fn cleanup_orphaned_transaction_temps(storage: &dyn Storage, dir: &Path) -> Result<(), CoreError> {
    let entries = storage
        .list(dir)
        .map_err(|error| recovery_error(format!("cannot inspect transaction files: {error}")))?;
    for path in entries {
        let file_type = storage.entry_kind(&path).map_err(|error| {
            recovery_error(format!("cannot inspect transaction file type: {error}"))
        })?;
        if file_type == StorageEntryKind::File
            && path.file_name().is_some_and(is_transaction_temp_name)
        {
            remove_file_durable(storage, &path).map_err(as_recovery_error)?;
        }
    }
    Ok(())
}

fn is_transaction_temp_name(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    [".project-transaction.tmp-", "project.tmp-", "history.tmp-"]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|suffix| Uuid::parse_str(suffix).is_ok())
        })
}

fn read_transaction(storage: &dyn Storage, path: &Path) -> Result<ProjectTransaction, CoreError> {
    let data = storage
        .read(path)
        .map_err(|error| recovery_error(format!("cannot read transaction journal: {error}")))?;
    serde_json::from_slice(&data)
        .map_err(|error| recovery_error(format!("invalid transaction journal: {error}")))
}

fn validate_transaction(dir: &Path, transaction: &ProjectTransaction) -> Result<(), CoreError> {
    if transaction.version != TRANSACTION_VERSION {
        return Err(recovery_error(format!(
            "unsupported transaction journal version {}",
            transaction.version
        )));
    }
    let directory_id = dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| recovery_error("project directory has no valid identifier"))?;
    if transaction.project.id != directory_id {
        return Err(recovery_error(
            "transaction project identity is inconsistent",
        ));
    }
    for snapshot in std::iter::once(&transaction.project)
        .chain(transaction.history.undo.iter())
        .chain(transaction.history.redo.iter())
    {
        if snapshot.id != transaction.project.id {
            return Err(recovery_error(
                "transaction history contains a different project identity",
            ));
        }
        if snapshot.schema_version != PROJECT_SCHEMA_VERSION {
            return Err(recovery_error(format!(
                "transaction contains unsupported project schema version {}",
                snapshot.schema_version
            )));
        }
    }
    if let Some(draft_id) = transaction.committed_draft_id.as_deref() {
        draft_path(dir, draft_id)
            .map_err(|_| recovery_error("transaction contains an invalid draft identifier"))?;
    }
    Ok(())
}

fn replay_transaction(
    storage: &dyn Storage,
    faults: &PersistenceFaults,
    dir: &Path,
    transaction: &ProjectTransaction,
) -> Result<(), CoreError> {
    faults
        .checkpoint(PersistencePhase::AfterJournal)
        .map_err(as_recovery_error)?;
    write_json_atomic(storage, &project_path(dir), &transaction.project)
        .map_err(as_recovery_error)?;
    faults
        .checkpoint(PersistencePhase::AfterProject)
        .map_err(as_recovery_error)?;
    write_json_atomic(storage, &history_path(dir), &transaction.history)
        .map_err(as_recovery_error)?;
    faults
        .checkpoint(PersistencePhase::AfterHistory)
        .map_err(as_recovery_error)?;
    if let Some(draft_id) = transaction.committed_draft_id.as_deref() {
        remove_file_if_exists(storage, &draft_path(dir, draft_id)?).map_err(as_recovery_error)?;
    }
    faults
        .checkpoint(PersistencePhase::AfterDraftCleanup)
        .map_err(as_recovery_error)?;
    remove_file_durable(storage, &transaction_path(dir)).map_err(as_recovery_error)?;
    faults
        .checkpoint(PersistencePhase::AfterJournalCleanup)
        .map_err(as_recovery_error)
}

fn draft_path(dir: &Path, id: &str) -> Result<PathBuf, CoreError> {
    let id = Uuid::parse_str(id)
        .map_err(|_| CoreError::new(ErrorCode::InvalidArgument, "draft id is invalid"))?;
    Ok(dir.join("drafts").join(format!("{id}.json")))
}

pub(crate) fn project_path(dir: &Path) -> PathBuf {
    dir.join("project.json")
}

pub(crate) fn history_path(dir: &Path) -> PathBuf {
    dir.join("history.json")
}

pub(crate) fn transaction_path(dir: &Path) -> PathBuf {
    dir.join(TRANSACTION_FILE)
}

fn recovery_error(message: impl Into<String>) -> CoreError {
    CoreError::new(ErrorCode::ProjectRecoveryFailed, message)
}

fn as_recovery_error(error: CoreError) -> CoreError {
    recovery_error(error.message)
}

#[cfg(unix)]
fn sync_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

pub(crate) struct ProjectLock(File);

impl Debug for ProjectLock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProjectLock")
            .finish_non_exhaustive()
    }
}

impl StorageLock for ProjectLock {}

impl ProjectLock {
    pub(crate) fn exclusive(dir: &Path) -> Result<Self, CoreError> {
        let file = open_lock(dir)?;
        file.lock_exclusive()
            .map_err(|error| CoreError::io("cannot lock project", error))?;
        Ok(Self(file))
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

fn open_lock(dir: &Path) -> Result<File, CoreError> {
    std::fs::create_dir_all(dir)
        .map_err(|error| CoreError::io("cannot create project directory", error))?;
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(dir.join("project.lock"))
        .map_err(|error| CoreError::io("cannot open project lock", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorCode;

    #[derive(Debug)]
    struct FailingStorage;

    impl Storage for FailingStorage {
        fn lock_exclusive(&self, _dir: &Path) -> Result<Box<dyn StorageLock>, CoreError> {
            Err(CoreError::new(
                ErrorCode::InternalError,
                "injected lock failure",
            ))
        }

        fn open_read(&self, _path: &Path) -> std::io::Result<Box<dyn Read>> {
            Err(std::io::Error::other("injected open failure"))
        }

        fn read(&self, _path: &Path) -> std::io::Result<Vec<u8>> {
            Err(std::io::Error::other("injected read failure"))
        }

        fn list(&self, _path: &Path) -> std::io::Result<Vec<PathBuf>> {
            Err(std::io::Error::other("injected list failure"))
        }

        fn create_dir_all(&self, _path: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("injected create failure"))
        }

        fn copy(&self, _from: &Path, _to: &Path) -> std::io::Result<u64> {
            Err(std::io::Error::other("injected copy failure"))
        }

        fn rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("injected rename failure"))
        }

        fn storage_path_is_file(&self, _path: &Path) -> bool {
            false
        }

        fn storage_path_exists(&self, _path: &Path) -> bool {
            false
        }

        fn entry_kind(&self, _path: &Path) -> std::io::Result<StorageEntryKind> {
            Err(std::io::Error::other("injected classification failure"))
        }

        fn canonicalize_storage_path(&self, _path: &Path) -> std::io::Result<PathBuf> {
            Err(std::io::Error::other("injected canonicalize failure"))
        }

        fn atomic_replace(&self, _path: &Path, _bytes: &[u8]) -> std::io::Result<()> {
            Err(std::io::Error::other("injected write failure"))
        }

        fn remove_durable(&self, _path: &Path) -> std::io::Result<()> {
            Err(std::io::Error::other("injected remove failure"))
        }
    }

    #[test]
    fn storage_failures_are_injectable_without_domain_changes() {
        let error =
            read_json::<serde_json::Value>(&FailingStorage, Path::new("project.json")).unwrap_err();
        assert_eq!(error.code, ErrorCode::InternalError);
        assert!(error.message.contains("cannot read persisted JSON"));

        let error = write_json_atomic(&FailingStorage, Path::new("project.json"), &42).unwrap_err();
        assert!(error.message.contains("cannot publish data"));
    }
}
