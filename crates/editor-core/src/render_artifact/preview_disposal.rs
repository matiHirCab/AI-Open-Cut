//! Disposal of process-owned renderer previews, confined by native handles.
use crate::{CoreError, ErrorCode};
use std::{fs::File, io, path::Path};

fn unsafe_output() -> CoreError {
    CoreError::new(
        ErrorCode::ValidationFailed,
        "Preview artifact disposal is unavailable or unsafe",
    )
}

/// Removes only a renderer-generated preview name under the configured root.
/// Callers must hold process-local ownership; this is not project/media deletion.
pub fn dispose_owned_preview(root: &Path, project: &str, relative: &str) -> Result<(), CoreError> {
    dispose_with(root, project, relative, || {})
}

fn dispose_with(
    root: &Path,
    project: &str,
    relative: &str,
    boundary: impl FnOnce(),
) -> Result<(), CoreError> {
    let canonical_uuid =
        |value: &str| uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value);
    if !canonical_uuid(project) {
        return Err(unsafe_output());
    }
    let name = relative
        .strip_prefix("previews/")
        .ok_or_else(unsafe_output)?;
    let identifier = name
        .strip_prefix("preview-range-")
        .and_then(|v| v.strip_suffix(".mp4"))
        .or_else(|| {
            name.strip_prefix("preview-")
                .and_then(|v| v.strip_suffix(".png"))
        })
        .ok_or_else(unsafe_output)?;
    if !canonical_uuid(identifier) {
        return Err(unsafe_output());
    }
    match native::dispose(root, project, name, boundary) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(unsafe_output()),
    }
}

#[cfg(unix)]
mod native {
    use super::*;
    use std::{
        ffi::CString,
        fs::OpenOptions,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::OpenOptionsExt,
        },
    };
    fn directory(parent: &File, name: &str) -> io::Result<File> {
        child(
            parent,
            name,
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    }
    fn child(parent: &File, name: &str, flags: i32) -> io::Result<File> {
        let name = CString::new(name).map_err(|_| io::Error::other("invalid name"))?;
        // SAFETY: parent remains held; name is NUL terminated; flags require no mode argument.
        let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openat returned a new descriptor whose sole owner is this File.
        Ok(unsafe { File::from_raw_fd(fd) })
    }
    pub(super) fn dispose(
        root: &Path,
        project: &str,
        name: &str,
        boundary: impl FnOnce(),
    ) -> io::Result<()> {
        let root = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(root)?;
        let project = directory(&root, project)?;
        let previews = directory(&project, "previews")?;
        let file = child(
            &previews,
            name,
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::other("unsafe target"));
        }
        boundary();
        let name = CString::new(name).map_err(|_| io::Error::other("invalid name"))?;
        // SAFETY: held previews descriptor anchors the operation even after an ancestor rename.
        if unsafe { libc::unlinkat(previews.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        fs::OpenOptions,
        os::windows::{
            fs::{MetadataExt, OpenOptionsExt},
            io::AsRawHandle,
        },
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FileDispositionInfo, SetFileInformationByHandle,
    };
    fn open(path: &Path, directory: bool) -> io::Result<File> {
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | if directory { 0 } else { 0x0001_0000 })
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?;
        let metadata = file.metadata()?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || metadata.is_dir() != directory
            || (!directory && !metadata.is_file())
        {
            return Err(io::Error::other("unsafe target"));
        }
        Ok(file)
    }
    pub(super) fn dispose(
        root: &Path,
        project: &str,
        name: &str,
        boundary: impl FnOnce(),
    ) -> io::Result<()> {
        let root_handle = open(root, true)?;
        let project_path = root.join(project);
        let project_handle = open(&project_path, true)?;
        let previews_path = project_path.join("previews");
        let previews_handle = open(&previews_path, true)?;
        let file = open(&previews_path.join(name), false)?;
        boundary();
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // SAFETY: valid owned file handle with DELETE access; correctly sized disposition structure.
        let result = unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileDispositionInfo,
                (&info as *const FILE_DISPOSITION_INFO).cast(),
                std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        };
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        drop(file);
        drop((previews_handle, project_handle, root_handle));
        Ok(())
    }
}

#[cfg(not(any(unix, windows)))]
mod native {
    use super::*;
    pub(super) fn dispose(_: &Path, _: &str, _: &str, _: impl FnOnce()) -> io::Result<()> {
        Err(io::Error::other("unsupported platform"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn fixture() -> (tempfile::TempDir, String, String) {
        let root = tempfile::tempdir().unwrap();
        let project = uuid::Uuid::new_v4().to_string();
        let relative = format!("previews/preview-{}.png", uuid::Uuid::new_v4());
        fs::create_dir_all(root.path().join(&project).join("previews")).unwrap();
        fs::write(root.path().join(&project).join(&relative), b"owned").unwrap();
        (root, project, relative)
    }
    #[test]
    fn owned_missing_and_foreign_names() {
        let (root, project, relative) = fixture();
        fs::write(
            root.path().join(&project).join("previews/foreign.png"),
            b"foreign",
        )
        .unwrap();
        assert!(dispose_owned_preview(root.path(), &project, "previews/foreign.png").is_err());
        assert!(dispose_owned_preview(root.path(), &project, "../../outside.png").is_err());
        dispose_owned_preview(root.path(), &project, &relative).unwrap();
        dispose_owned_preview(root.path(), &project, &relative).unwrap();
        assert_eq!(
            fs::read(root.path().join(&project).join("previews/foreign.png")).unwrap(),
            b"foreign"
        );
    }
    #[test]
    fn ancestor_replacement_cannot_redirect_deletion() {
        for level in ["root", "project", "previews"] {
            let (root, project, relative) = fixture();
            let outside = tempfile::tempdir().unwrap();
            let name = Path::new(&relative).file_name().unwrap();
            fs::write(outside.path().join(name), b"preserve").unwrap();
            let ancestor = match level {
                "root" => root.path().to_path_buf(),
                "project" => root.path().join(&project),
                _ => root.path().join(&project).join("previews"),
            };
            let moved = ancestor.with_extension("moved");
            dispose_with(root.path(), &project, &relative, || {
                #[cfg(unix)]
                {
                    fs::rename(&ancestor, &moved).unwrap();
                    std::os::unix::fs::symlink(outside.path(), &ancestor).unwrap();
                }
                #[cfg(windows)]
                {
                    assert!(fs::rename(&ancestor, &moved).is_err());
                }
            })
            .unwrap();
            assert_eq!(fs::read(outside.path().join(name)).unwrap(), b"preserve");
            #[cfg(unix)]
            {
                fs::remove_file(&ancestor).unwrap();
                fs::rename(&moved, &ancestor).unwrap();
            }
            assert!(!root.path().join(&project).join(&relative).exists());
        }
    }
    #[test]
    fn unicode_parent_and_handle_release() {
        let (root, project, relative) = fixture();
        let unicode = root.path().join("媒体 café");
        fs::create_dir(&unicode).unwrap();
        fs::rename(root.path().join(&project), unicode.join(&project)).unwrap();
        dispose_owned_preview(&unicode, &project, &relative).unwrap();
        let moved = root.path().join("moved");
        fs::rename(&unicode, &moved).unwrap();
        fs::remove_dir_all(&moved).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn trusted_parent_alias_allowed_but_root_and_child_links_refused() {
        let (root, project, relative) = fixture();
        let parent = tempfile::tempdir().unwrap();
        let alias = parent.path().join("alias");
        std::os::unix::fs::symlink(root.path(), &alias).unwrap();
        let actual = root.path().join("projects");
        fs::create_dir(&actual).unwrap();
        fs::rename(root.path().join(&project), actual.join(&project)).unwrap();
        assert!(dispose_owned_preview(&alias, &project, &relative).is_err());
        dispose_owned_preview(&alias.join("projects"), &project, &relative).unwrap();
        for level in ["project", "previews"] {
            let (root, project, relative) = fixture();
            let target = if level == "previews" {
                root.path().join(&project).join("previews")
            } else {
                root.path().join(&project)
            };
            let moved = target.with_extension("saved");
            fs::rename(&target, &moved).unwrap();
            std::os::unix::fs::symlink(&moved, &target).unwrap();
            assert!(dispose_owned_preview(root.path(), &project, &relative).is_err());
            let original = if level == "previews" {
                moved.join(Path::new(&relative).file_name().unwrap())
            } else {
                moved.join(&relative)
            };
            assert_eq!(fs::read(original).unwrap(), b"owned");
        }
    }
    #[test]
    fn links_are_rejected() {
        let (root, project, relative) = fixture();
        let outside = tempfile::tempdir().unwrap();
        let foreign = outside.path().join("foreign");
        fs::write(&foreign, b"preserve").unwrap();
        let target = root.path().join(&project).join(&relative);
        fs::remove_file(&target).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&foreign, &target).unwrap();
        #[cfg(windows)]
        {
            // Junctions need no developer-mode privileges and exercise reparse refusal.
            fs::remove_dir(root.path().join(&project).join("previews")).unwrap();
            let status = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(root.path().join(&project).join("previews"))
                .arg(outside.path())
                .status()
                .unwrap();
            assert!(status.success());
        }
        assert!(dispose_owned_preview(root.path(), &project, &relative).is_err());
        assert_eq!(fs::read(&foreign).unwrap(), b"preserve");
    }
}
