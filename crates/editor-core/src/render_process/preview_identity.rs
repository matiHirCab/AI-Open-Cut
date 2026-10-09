//! Exact executable identity; inability to establish it disables optional reuse.
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

fn resolve(path: &Path) -> Option<PathBuf> {
    if path.components().count() > 1 || path.is_absolute() {
        return std::fs::canonicalize(path).ok();
    }
    // Ambiguous PATH lookup is optional-cache bypass. Canonical aliases of the
    // same binary are safe; distinct candidates can vary with access policy.
    #[cfg(unix)]
    {
        let mut unique = None;
        for root in std::env::split_paths(&std::env::var_os("PATH")?) {
            let candidate = root.join(path);
            let metadata = match std::fs::metadata(&candidate) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return None,
            };
            if !metadata.is_file() {
                continue;
            }
            let resolved = std::fs::canonicalize(candidate).ok()?;
            if unique
                .as_ref()
                .is_some_and(|previous| previous != &resolved)
            {
                return None;
            }
            unique = Some(resolved);
        }
        unique
    }
    #[cfg(not(unix))]
    None
}

pub(super) fn identity(ffmpeg: &Path, ffprobe: &Path) -> Option<[u8; 32]> {
    let mut hash = Sha256::new();
    hash.update(b"opencut-preview-process-v1");
    for executable in [ffmpeg, ffprobe] {
        let path = resolve(executable)?;
        let mut file = std::fs::File::open(&path).ok()?;
        let metadata = file.metadata().ok()?;
        if !metadata.is_file() {
            return None;
        }
        // Scripts can delegate to mutable interpreters/backends that this owner
        // cannot fingerprint. They retain ordinary uncached rendering behavior.
        let mut signature = [0u8; 4];
        file.read_exact(&mut signature).ok()?;
        if signature != *b"\x7fELF" && signature[..2] != *b"MZ" {
            return None;
        }
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::Start(0)).ok()?;
        let path = path.to_str()?;
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path.as_bytes());
        hash.update(metadata.len().to_le_bytes());
        let mut buffer = [0; 65536];
        let mut read = 0u64;
        // Identify a direct FFmpeg-family CLI, not an arbitrary native forwarding
        // wrapper whose delegated implementation is outside this fingerprint.
        let marker = b"%s version ";
        let mut previous = Vec::new();
        let mut direct_cli = false;
        loop {
            let count = file.read(&mut buffer).ok()?;
            if count == 0 {
                break;
            }
            read = read.checked_add(count as u64)?;
            hash.update(&buffer[..count]);
            if !direct_cli {
                previous.extend_from_slice(&buffer[..count]);
                direct_cli = previous
                    .windows(marker.len())
                    .any(|window| window == marker);
                if previous.len() >= marker.len() {
                    previous.drain(..previous.len() - (marker.len() - 1));
                }
            }
        }
        if read != metadata.len() || !direct_cli {
            return None;
        }
    }
    Some(hash.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_native_binary_content_and_backend_paths_are_exact_while_scripts_bypass() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        std::fs::write(&first, b"\x7fELFfirst native bytes %s version ").unwrap();
        std::fs::write(&second, b"\x7fELFsecond native bytes %s version ").unwrap();
        let key = identity(&first, &second).unwrap();
        assert_eq!(identity(&first, &second).unwrap(), key);
        std::fs::write(&second, b"\x7fELFchanged native bytes %s version ").unwrap();
        assert_ne!(identity(&first, &second).unwrap(), key);
        assert_ne!(identity(&second, &first).unwrap(), key);
        std::fs::write(&first, b"\x7fELFnative forwarding wrapper").unwrap();
        assert!(identity(&first, &second).is_none());
        std::fs::write(&first, b"#!/bin/sh\nexec elsewhere\n").unwrap();
        assert!(identity(&first, &second).is_none());
        assert!(identity(root.path(), &second).is_none());
        assert!(identity(&root.path().join("missing"), &second).is_none());
    }
}
