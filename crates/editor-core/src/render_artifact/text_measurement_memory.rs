//! Active-matte measurement admission; default text measurement allocates no ledger.
use crate::{CoreError, ErrorCode, evaluated_scene::EvaluatedText, fonts::shaping::ShapedGlyph};
use std::{collections::BTreeMap, mem::size_of};
fn invalid() -> CoreError {
    CoreError::new(
        ErrorCode::InvalidArgument,
        "matte text measurement exceeds shared memory bounds",
    )
}
fn add(a: u64, b: u64) -> Result<u64, CoreError> {
    a.checked_add(b).ok_or_else(invalid)
}
fn bounded_add(a: u64, b: u64, limit: u64) -> Result<u64, CoreError> {
    {
        let sum = add(a, b)?;
        if sum <= limit {
            Ok(sum)
        } else {
            Err(invalid())
        }
    }
}
fn mul(a: u64, b: u64) -> Result<u64, CoreError> {
    a.checked_mul(b).ok_or_else(invalid)
}
fn vec_bound(n: u64, element: u64) -> Result<u64, CoreError> {
    if n == 0 {
        Ok(0)
    } else {
        mul(mul(3, n.max(4))?, element)
    }
}
#[cfg(test)]
thread_local! { static CENSUS_LOOKUPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
/// Borrowed census precedes Rustybuzz's eager Face/lookup allocations. Source
/// ceilings are pinned in glyph-memory-certification.md; no font-format limit.
pub(super) fn opaque_face_bytes(bytes: &[u8], remaining: u64) -> Result<u64, CoreError> {
    let face = crate::fonts::validate_face(bytes)?;
    let mut result = 0;
    for (table, enum_size) in [
        (
            face.tables().gsub,
            size_of::<ttf_parser::gsub::SubstitutionSubtable>(),
        ),
        (
            face.tables().gpos,
            size_of::<ttf_parser::gpos::PositioningSubtable>(),
        ),
    ] {
        let Some(table) = table else {
            continue;
        };
        result = bounded_add(
            result,
            vec_bound(u64::from(table.lookups.len()), 128)?,
            remaining,
        )?;
        for lookup in table.lookups {
            #[cfg(test)]
            CENSUS_LOOKUPS.with(|n| n.set(n.get() + 1));
            result = bounded_add(
                result,
                vec_bound(u64::from(lookup.subtables.len()), enum_size as u64)?,
                remaining,
            )?;
        }
        let variation = table
            .variations
            .and_then(|v| v.find_index(face.variation_coordinates()));
        let mut indices = 0;
        for index in 0..table.features.len() {
            let feature = variation
                .and_then(|i| table.variations?.find_substitute(index, i))
                .or_else(|| table.features.get(index));
            if let Some(feature) = feature {
                indices = add(indices, u64::from(feature.lookup_indices.len()))?;
                bounded_add(result, vec_bound(mul(2, indices)?, 16)?, remaining)?;
            }
        }
        result = bounded_add(result, vec_bound(mul(2, indices)?, 16)?, remaining)?;
    }
    for term in [
        vec_bound(48, 48)?,
        vec_bound(48, 64)?,
        mul(2, add(vec_bound(17, 16)?, vec_bound(1, 16)?)?)?,
        256,
        32,
    ] {
        result = bounded_add(result, term, remaining)?;
    }
    let mut chains = 0u64;
    if let Some(table) = &face.tables().morx {
        for _ in table.chains {
            chains = add(chains, 1)?;
            bounded_add(
                result,
                add(
                    vec_bound(chains, 3 * size_of::<usize>() as u64)?,
                    mul(chains, vec_bound(1, 12)?)?,
                )?,
                remaining,
            )?;
        }
    }
    result = bounded_add(
        result,
        add(
            vec_bound(chains, 3 * size_of::<usize>() as u64)?,
            mul(chains, vec_bound(1, 12)?)?,
        )?,
        remaining,
    )?;
    result = bounded_add(result, vec_bound(1, 16)?, remaining)?;
    result = bounded_add(
        result,
        mul(65 * 3 * 64, size_of::<usize>() as u64)?,
        remaining,
    )?;
    for term in [
        vec_bound(130, 24)?,
        vec_bound(129, 24)?,
        vec_bound(193, size_of::<ttf_parser::Transform>() as u64)?,
        vec_bound(129, 4)?,
    ] {
        result = bounded_add(result, term, remaining)?;
    }
    Ok(result)
}
pub(super) fn shaping_scratch(
    text: &EvaluatedText,
    faces: &BTreeMap<String, Vec<u8>>,
    remaining: u64,
) -> Result<u64, CoreError> {
    let binding = text.font_binding.as_ref().ok_or_else(invalid)?;
    let n = text
        .rich_runs
        .as_ref()
        .ok_or_else(invalid)?
        .iter()
        .try_fold(0u64, |n, r| add(n, r.text.len() as u64))?;
    let d = add(
        crate::evaluated_scene::composition_resources::text_heap(text)?,
        size_of::<EvaluatedText>() as u64,
    )?;
    let mut face_bytes = 0;
    for hash in binding.hashes() {
        face_bytes = face_bytes.max(hash.len() as u64);
    }
    let mut color = text.color.capacity() as u64;
    if let Some(runs) = &text.rich_runs {
        for run in runs {
            if let Some(c) = &run.color {
                color = color.max(c.capacity() as u64);
            }
        }
    }
    let mut paints = 0;
    if let Some(spans) = &text.spans {
        for span in spans {
            if let Some(c) = &span.style.color {
                color = color.max(c.capacity() as u64);
            }
            if let Some(p) = &span.style.paint_layers {
                paints =
                    paints.max(crate::evaluated_scene::composition_resources::text_paints_heap(p)?);
            }
        }
    }
    let glyph = add(
        size_of::<ShapedGlyph>() as u64,
        add(face_bytes, add(color, paints)?)?,
    )?;
    let r = mul(64, n)?.max(16384);
    let mut result = 0;
    for term in [
        mul(120, r)?,
        mul(8 * 16384, glyph)?,
        mul(2048, add(n, 1)?)?,
        24 * (16384 + 4096),
        mul(16, d)?,
    ] {
        result = add(result, term)?;
    }
    let available = remaining.checked_sub(result).ok_or_else(invalid)?;
    let mut opaque = 0;
    for hash in binding.hashes() {
        opaque = opaque.max(opaque_face_bytes(
            faces.get(hash).ok_or_else(invalid)?,
            available,
        )?);
    }
    result = bounded_add(result, opaque, remaining)?;
    Ok(result)
}
/// Active cache hits admit their complete transient before cloning payload.
pub(super) fn clone_cached_shape(
    memory: &MeasurementMemory,
    shaped: &crate::fonts::shaping::ShapedText,
    transient: u64,
) -> Result<crate::fonts::shaping::ShapedText, CoreError> {
    let required = crate::evaluated_scene::composition_resources::shaped_heap_bytes(shaped)?
        .checked_mul(3)
        .and_then(|n| n.checked_add(transient))
        .ok_or_else(|| {
            CoreError::new(
                ErrorCode::InvalidArgument,
                "matte text clone memory overflow",
            )
        })?;
    memory.admit(required)?;
    Ok(shaped.clone())
}
/// Includes spare HashMap buckets and growth/control/alignment slack. Call
/// before insertion, with next_len; old retained buckets are counted separately.
pub(crate) fn map_bytes<K, V>(entries: usize) -> Result<u64, CoreError> {
    if entries == 0 {
        return Ok(0);
    }
    let buckets = entries
        .checked_mul(2)
        .and_then(|n| n.max(4).checked_next_power_of_two())
        .ok_or_else(invalid)? as u64;
    add(mul(buckets, add(size_of::<(K, V)>() as u64, 1)?)?, 64)
}
pub(crate) struct MeasurementMemory {
    limit: u64,
    pub(crate) retained: u64,
}
impl MeasurementMemory {
    pub(crate) fn new(limit: u64) -> Self {
        Self { limit, retained: 0 }
    }
    pub(crate) fn admit(&self, transient: u64) -> Result<(), CoreError> {
        if add(self.retained, transient)? > self.limit {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    pub(crate) fn remaining(&self) -> Result<u64, CoreError> {
        self.limit.checked_sub(self.retained).ok_or_else(invalid)
    }
    pub(crate) fn adopt(&mut self, retained: u64) -> Result<(), CoreError> {
        if retained > self.limit {
            return Err(invalid());
        }
        self.retained = retained;
        Ok(())
    }
}

pub(super) fn shaped_cache_bytes(
    cache: &std::collections::HashMap<String, crate::fonts::shaping::ShapedText>,
) -> Result<u64, CoreError> {
    let mut bytes = map_bytes::<String, crate::fonts::shaping::ShapedText>(cache.capacity())?;
    for (key, value) in cache {
        bytes = add(
            bytes,
            add(
                key.capacity() as u64,
                crate::evaluated_scene::composition_resources::shaped_heap_bytes(value)?,
            )?,
        )?;
    }
    Ok(bytes)
}
pub(super) fn measured_bytes(
    map: &std::collections::HashMap<String, super::MeasuredText>,
) -> Result<u64, CoreError> {
    let mut bytes = map_bytes::<String, super::MeasuredText>(map.capacity())?;
    for (key, value) in map {
        bytes = add(
            bytes,
            add(key.capacity() as u64, value.content.capacity() as u64)?,
        )?;
        if let Some((shaped, style)) = &value.shaped {
            bytes = add(
                bytes,
                add(
                    crate::evaluated_scene::composition_resources::shaped_heap_bytes(shaped)?,
                    crate::evaluated_scene::composition_resources::text_style_heap_bytes(style)?,
                )?,
            )?;
        }
        bytes = add(bytes, value.prepared.file_path.capacity() as u64)?;
        if let Some(path) = &value.prepared.font_path {
            bytes = add(bytes, path.capacity() as u64)?;
        }
        if let Some(runs) = &value.prepared.rich_runs {
            bytes = add(
                bytes,
                mul(
                    runs.capacity() as u64,
                    size_of::<crate::render_plan::PreparedTextRun>() as u64,
                )?,
            )?;
            for run in runs {
                for part in [
                    run.file_path.capacity() as u64,
                    run.font_path.as_ref().map_or(0, |p| p.capacity() as u64),
                    run.content.capacity() as u64,
                    run.color.capacity() as u64,
                ] {
                    bytes = add(bytes, part)?;
                }
            }
        }
    }
    Ok(bytes)
}
pub(super) fn warnings_bytes(warnings: &Vec<String>) -> Result<u64, CoreError> {
    warnings.iter().try_fold(
        mul(warnings.capacity() as u64, size_of::<String>() as u64)?,
        |n, s| add(n, s.capacity() as u64),
    )
}
/// Escape expansion6x, geometric buffer old/new3x, plus structural field
/// punctuation and input clone payload. This starts before document/key clone.
pub(crate) fn key_scratch(
    layer: &crate::evaluated_scene::EvaluatedVisualLayer,
) -> Result<u64, CoreError> {
    let heap = crate::evaluated_scene::composition_resources::layer_heap_bytes(layer)?;
    add(mul(24, heap)?, 65536)
}

// Cursor allowances follow Rust1.97 and the platform implementations cited in
// the approved lookup appendix. glibc caps its DIR buffer at1MiB; Darwin's
// nonunion cursor overlaps2048+8192 bytes when growing; Windows has fixed
// WIN32_FIND_DATAW records. Headers/control blocks fit the additional4096.
fn cursor_bytes() -> std::io::Result<u64> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    return Ok(1_048_576 + 4096);
    #[cfg(target_os = "macos")]
    return Ok(2048 + 8192 + 4096);
    #[cfg(windows)]
    return Ok(4096);
    #[cfg(not(any(
        all(target_os = "linux", target_env = "gnu"),
        target_os = "macos",
        windows
    )))]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unqualified directory cursor",
    ))
}
fn lookup_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::OutOfMemory,
        "font lookup exceeds admitted memory",
    )
}
fn lookup_add(a: u64, b: u64) -> std::io::Result<u64> {
    a.checked_add(b).ok_or_else(lookup_error)
}
fn lookup_mul(a: u64, b: u64) -> std::io::Result<u64> {
    a.checked_mul(b).ok_or_else(lookup_error)
}
fn lookup_admit(live: u64, extra: u64, limit: u64) -> std::io::Result<()> {
    if lookup_add(live, extra)? > limit {
        Err(lookup_error())
    } else {
        Ok(())
    }
}
#[cfg(not(target_os = "macos"))]
type FontCursor = std::fs::ReadDir;
#[cfg(target_os = "macos")]
type FontCursor = nix::dir::OwningIter;
#[cfg(not(target_os = "macos"))]
fn open_font_cursor(path: &std::path::Path) -> std::io::Result<FontCursor> {
    cursor_bytes()?;
    std::fs::read_dir(path)
}
#[cfg(target_os = "macos")]
fn checked_font_dir_with(
    file: std::fs::File,
    union: impl FnOnce(&std::fs::File) -> std::io::Result<bool>,
    convert: impl FnOnce(std::os::fd::OwnedFd) -> nix::Result<nix::dir::Dir>,
    close: impl FnOnce(std::os::fd::RawFd) -> nix::Result<()>,
) -> std::io::Result<nix::dir::Dir> {
    use std::os::fd::AsRawFd;
    if union(&file)? {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "union directory has unbounded eager cursor",
        ));
    }
    let raw = file.as_raw_fd();
    // Pinned nix0.31.3 consumes OwnedFd before fdopendir and leaks on Err.
    // Darwin leaves the FD open on failure. Close ONLY that failure branch.
    match convert(file.into()) {
        Ok(dir) => Ok(dir),
        Err(error) => {
            close(raw)?;
            Err(error.into())
        }
    }
}
#[cfg(target_os = "macos")]
fn checked_font_dir(file: std::fs::File) -> std::io::Result<nix::dir::Dir> {
    checked_font_dir_with(
        file,
        |file| {
            Ok(nix::sys::statfs::fstatfs(file)?
                .flags()
                .contains(nix::mount::MntFlags::MNT_UNION))
        },
        nix::dir::Dir::from_fd,
        nix::unistd::close,
    )
}
#[cfg(target_os = "macos")]
fn open_font_cursor(path: &std::path::Path) -> std::io::Result<FontCursor> {
    checked_font_dir(std::fs::File::open(path)?).map(IntoIterator::into_iter)
}
struct FontFrame {
    root: std::path::PathBuf,
    cursor: FontCursor,
}
fn frames_bytes(frames: &Vec<FontFrame>) -> std::io::Result<u64> {
    let mut n = lookup_mul(frames.capacity() as u64, size_of::<FontFrame>() as u64)?;
    for frame in frames {
        n = lookup_add(
            n,
            lookup_add(
                cursor_bytes()?,
                lookup_mul(3, frame.root.capacity() as u64)?,
            )?,
        )?;
    }
    Ok(n)
}
fn entry_scratch(root: &std::path::Path) -> std::io::Result<u64> {
    // Linux dirent.d_reclen is u16. macOS MAXNAMLEN1023 and Windows260
    // UTF16 units are bounded platform records, not authored path limits.
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    let name = 65_536;
    #[cfg(target_os = "macos")]
    let name = 1024;
    #[cfg(windows)]
    let name = 1040;
    #[cfg(not(any(
        all(target_os = "linux", target_env = "gnu"),
        target_os = "macos",
        windows
    )))]
    let name = 0;
    // Includes entry-name copy, joined path old/new growth, lossy UTF8,
    // lowercase/replace and canonicalization's platform path buffers. Windows
    // extended paths are at most32767UTF16 units;256KiB covers both encodings.
    lookup_add(
        262_144,
        lookup_add(
            lookup_mul(16, name)?,
            lookup_mul(16, root.as_os_str().len() as u64)?,
        )?,
    )
}
fn push_font_frame(
    frames: &mut Vec<FontFrame>,
    root: &std::path::Path,
    limit: u64,
) -> std::io::Result<()> {
    let live = frames_bytes(frames)?;
    let next = frames.len().checked_add(1).ok_or_else(lookup_error)?;
    let growth = lookup_mul(next as u64, size_of::<FontFrame>() as u64)?;
    let payload = lookup_add(
        cursor_bytes()?,
        lookup_mul(3, root.as_os_str().len() as u64)?,
    )?;
    lookup_admit(
        live,
        lookup_add(growth, lookup_add(payload, entry_scratch(root)?)?)?,
        limit,
    )?;
    if frames.capacity() < next {
        frames
            .try_reserve_exact(next - frames.len())
            .map_err(|_| lookup_error())?;
    }
    let cursor = open_font_cursor(root)?;
    frames.push(FontFrame {
        root: root.to_path_buf(),
        cursor,
    });
    lookup_admit(frames_bytes(frames)?, 0, limit)
}
fn propagate_resource_failure(error: std::io::Error) -> std::io::Result<()> {
    if matches!(
        error.kind(),
        std::io::ErrorKind::OutOfMemory | std::io::ErrorKind::Unsupported
    ) {
        Err(error)
    } else {
        Ok(())
    }
}
fn next_font_path(frame: &mut FontFrame) -> Option<std::io::Result<std::path::PathBuf>> {
    #[cfg(not(target_os = "macos"))]
    return frame
        .cursor
        .next()
        .map(|result| result.map(|entry| entry.path()));
    #[cfg(target_os = "macos")]
    loop {
        use std::os::unix::ffi::OsStrExt;
        let entry = match frame.cursor.next()? {
            Ok(entry) => entry,
            Err(error) => return Some(Err(error.into())),
        };
        let name = entry.file_name().to_bytes();
        if name == b"." || name == b".." {
            continue;
        }
        return Some(Ok(frame.root.join(std::ffi::OsStr::from_bytes(name))));
    }
}
pub(super) fn admitted_font_lookup(
    root: &std::path::Path,
    family: &str,
    limit: u64,
) -> std::io::Result<Option<std::path::PathBuf>> {
    admitted_font_lookup_with(root, family, limit, next_font_path)
}
fn admitted_font_lookup_with(
    root: &std::path::Path,
    family: &str,
    limit: u64,
    mut next: impl FnMut(&mut FontFrame) -> Option<std::io::Result<std::path::PathBuf>>,
) -> std::io::Result<Option<std::path::PathBuf>> {
    let mut frames = Vec::new();
    match push_font_frame(&mut frames, root, limit) {
        Ok(()) => (),
        Err(error) => {
            propagate_resource_failure(error)?;
            return Ok(None);
        }
    }
    while !frames.is_empty() {
        let live = frames_bytes(&frames)?;
        let frame = frames.last_mut().unwrap();
        lookup_admit(live, entry_scratch(&frame.root)?, limit)?;
        let path = match next(frame) {
            None => {
                frames.pop();
                continue;
            }
            Some(Err(error)) => {
                propagate_resource_failure(error)?;
                continue;
            }
            Some(Ok(path)) => path,
        };
        // The old resolver follows symlinks through metadata. Retain that
        // behavior; a directory cycle is bounded by the shared heap admission.
        let metadata = match std::fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                propagate_resource_failure(error)?;
                continue;
            }
        };
        if metadata.is_dir() {
            if let Err(error) = push_font_frame(&mut frames, &path, limit) {
                propagate_resource_failure(error)?;
            }
        } else if metadata.is_file()
            && matches!(
                path.extension()
                    .and_then(|s| s.to_str())
                    .map(str::to_ascii_lowercase)
                    .as_deref(),
                Some("ttf" | "otf" | "ttc")
            )
        {
            let Some(stem) = path.file_stem() else {
                continue;
            };
            let stem = stem
                .to_string_lossy()
                .to_lowercase()
                .replace([' ', '-', '_'], "");
            if stem.contains(family) {
                match std::fs::canonicalize(&path) {
                    Ok(found) => {
                        lookup_admit(frames_bytes(&frames)?, found.capacity() as u64, limit)?;
                        return Ok(Some(found));
                    }
                    Err(error) => {
                        propagate_resource_failure(error)?;
                        continue;
                    }
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::super::FileSystemArtifactIo;
    use super::*;
    use crate::fonts::shaping::{observed_shape_calls, shape_admitted};

    #[test]
    fn admitted_font_lookup_preserves_nested_first_match_and_missing_fallback() {
        let root = tempfile::tempdir().unwrap();
        let child = root.path().join("nested");
        std::fs::create_dir(&child).unwrap();
        std::fs::write(child.join("Example-Family.ttf"), b"font").unwrap();
        let expected =
            super::super::find_font_file(&FileSystemArtifactIo, root.path(), "examplefamily")
                .unwrap();
        let found = admitted_font_lookup(root.path(), "examplefamily", 16 * 1024 * 1024)
            .unwrap()
            .unwrap();
        assert_eq!(found, expected);
        assert_eq!(
            admitted_font_lookup(
                &root.path().join("missing"),
                "examplefamily",
                16 * 1024 * 1024
            )
            .unwrap(),
            None
        );
        assert_eq!(
            admitted_font_lookup(root.path(), "absent", 16 * 1024 * 1024).unwrap(),
            None
        );
        let error = admitted_font_lookup(root.path(), "examplefamily", 1).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::OutOfMemory);
    }

    #[cfg(unix)]
    #[test]
    fn admitted_font_lookup_follows_symlink_files_and_directories_and_bounds_cycles() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::write(target.path().join("Linked-Font.ttf"), b"font").unwrap();
        symlink(target.path(), root.path().join("linked-dir")).unwrap();
        let expected =
            super::super::find_font_file(&FileSystemArtifactIo, root.path(), "linkedfont").unwrap();
        assert_eq!(
            admitted_font_lookup(root.path(), "linkedfont", 16 * 1024 * 1024).unwrap(),
            Some(expected)
        );
        symlink(
            target.path().join("Linked-Font.ttf"),
            root.path().join("Alias-Family.otf"),
        )
        .unwrap();
        assert_eq!(
            admitted_font_lookup(root.path(), "aliasfamily", 16 * 1024 * 1024).unwrap(),
            Some(std::fs::canonicalize(target.path().join("Linked-Font.ttf")).unwrap())
        );
        let cycle = tempfile::tempdir().unwrap();
        symlink(cycle.path(), cycle.path().join("cycle")).unwrap();
        // A fixed Linux-sized budget lets Darwin hit its symlink/path limit
        // before admission fails. Admit two actual cursors plus entry and
        // descriptor scratch, so repeated traversal fails at the memory ledger.
        let limit = entry_scratch(cycle.path()).unwrap() + 2 * cursor_bytes().unwrap() + 8192;
        let mut entries = 0;
        let error = admitted_font_lookup_with(cycle.path(), "doesnotexist", limit, |frame| {
            let next = next_font_path(frame);
            entries += usize::from(next.is_some());
            next
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::OutOfMemory);
        assert!(entries >= 2, "must actually follow the directory cycle");
    }

    #[test]
    fn admitted_font_lookup_rejects_before_cursor_and_qualifies_actual_owned_peak() {
        let root = tempfile::tempdir().unwrap();
        for index in 0..300 {
            std::fs::write(root.path().join(format!("Font-{index}.ttf")), b"font").unwrap();
        }
        let limit = 16 * 1024 * 1024;
        let (peak, allocations, _) = super::super::shapes::fill_allocation_tests::observe(|| {
            let found = admitted_font_lookup(root.path(), "font299", limit)
                .unwrap()
                .unwrap();
            assert!(found.ends_with("Font-299.ttf"));
            drop(found);
        });
        assert!(allocations > 0);
        assert!(peak as u64 <= limit);
        let missing = root.path().join("not-opened");
        // Admission precedes opening even a nonexistent directory: this proves
        // the failure is the ledger, rather than a filesystem fallback.
        assert_eq!(
            admitted_font_lookup(&missing, "font", 0)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::OutOfMemory
        );
    }

    #[derive(Debug)]
    struct UnsupportedLookup;
    impl super::super::ArtifactIo for UnsupportedLookup {
        fn request_id(&self) -> String {
            "test".into()
        }
        fn create_dir(&self, p: &std::path::Path) -> std::io::Result<()> {
            std::fs::create_dir(p)
        }
        fn remove_dir_all(&self, p: &std::path::Path) -> std::io::Result<()> {
            std::fs::remove_dir_all(p)
        }
        fn read(&self, p: &std::path::Path) -> std::io::Result<Vec<u8>> {
            std::fs::read(p)
        }
        fn write(&self, p: &std::path::Path, b: &[u8]) -> std::io::Result<()> {
            std::fs::write(p, b)
        }
        fn list(&self, _: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
            panic!("active lookup cannot fall back to unbounded list")
        }
        fn entry_kind(
            &self,
            p: &std::path::Path,
        ) -> std::io::Result<super::super::ArtifactEntryKind> {
            super::super::ArtifactIo::entry_kind(&FileSystemArtifactIo, p)
        }
        fn canonicalize_artifact_path(
            &self,
            p: &std::path::Path,
        ) -> std::io::Result<std::path::PathBuf> {
            std::fs::canonicalize(p)
        }
        fn artifact_path_exists(&self, p: &std::path::Path) -> bool {
            p.exists()
        }
        fn remove(&self, p: &std::path::Path) -> std::io::Result<()> {
            std::fs::remove_file(p)
        }
        fn rename(&self, p: &std::path::Path, q: &std::path::Path) -> std::io::Result<()> {
            std::fs::rename(p, q)
        }
        fn size(&self, p: &std::path::Path) -> std::io::Result<u64> {
            std::fs::metadata(p).map(|m| m.len())
        }
    }
    #[test]
    fn admitted_resolver_keeps_absent_fallback_and_fails_closed_for_unsupported_port() {
        let root = tempfile::tempdir().unwrap();
        let fallback = root.path().join("fallback.ttf");
        std::fs::write(&fallback, b"font").unwrap();
        let binding = crate::evaluated_scene::FontResourceBinding {
            pinned_faces: Default::default(),
            font_resource_id: "font".into(),
            requested_path: None,
            requested_family: Some("absent".into()),
        };
        let roots = [root.path().to_path_buf()];
        let mut warnings = Vec::new();
        let mut memory = MeasurementMemory::new(16 * 1024 * 1024);
        let found = super::super::resolve_evaluated_font_admitted(
            &FileSystemArtifactIo,
            "text",
            Some(&binding),
            Some(&fallback),
            &roots,
            &mut warnings,
            (&mut memory, 0),
        )
        .unwrap();
        assert_eq!(found, Some(fallback.clone()));
        assert_eq!(warnings.len(), 1);
        warnings.clear();
        let error = super::super::resolve_evaluated_font_admitted(
            &UnsupportedLookup,
            "text",
            Some(&binding),
            Some(&fallback),
            &roots,
            &mut warnings,
            (&mut memory, 0),
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::DependencyUnavailable);
        assert!(!error.retryable);
        assert!(
            warnings.is_empty(),
            "unsupported admission must not silently fall back"
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn admitted_cursor_advancement_propagates_resource_failures_before_fallback() {
        let root = tempfile::tempdir().unwrap();
        for kind in [
            std::io::ErrorKind::OutOfMemory,
            std::io::ErrorKind::Unsupported,
        ] {
            let mut calls = 0;
            let error = admitted_font_lookup_with(root.path(), "absent", 16 * 1024 * 1024, |_| {
                calls += 1;
                Some(Err(std::io::Error::from(kind)))
            })
            .unwrap_err();
            assert_eq!(error.kind(), kind);
            assert_eq!(calls, 1);
            assert_eq!(
                propagate_resource_failure(std::io::Error::from(kind))
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        for kind in [
            std::io::ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied,
        ] {
            assert!(propagate_resource_failure(std::io::Error::from(kind)).is_ok());
            let mut first = true;
            assert_eq!(
                admitted_font_lookup_with(root.path(), "absent", 16 * 1024 * 1024, |_| {
                    if std::mem::take(&mut first) {
                        Some(Err(std::io::Error::from(kind)))
                    } else {
                        None
                    }
                })
                .unwrap(),
                None
            );
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }

    fn document(text: &EvaluatedText) -> crate::RichTextDocument {
        crate::RichTextDocument {
            runs: text.rich_runs.clone().unwrap(),
            spans: text.spans.clone(),
        }
    }
    #[test]
    fn active_shaping_peak_and_layouts_fit_source_derived_reservation() {
        let (base, faces) =
            crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
        //128 applies to the private eager lookup HEADER, not its public
        //subtable enum. Those allocations use their actual ABI size.
        let gsub = size_of::<ttf_parser::gsub::SubstitutionSubtable>() as u64;
        let gpos = size_of::<ttf_parser::gpos::PositioningSubtable>() as u64;
        assert_eq!(vec_bound(4, gsub).unwrap(), 12 * gsub);
        assert_eq!(vec_bound(4, gpos).unwrap(), 12 * gpos);
        eprintln!("public subtable ABI GSUB={gsub} GPOS={gpos}");
        assert_eq!(size_of::<ttf_parser::Transform>(), 24);
        for content in [
            "fi ffi Arabic مرحبا Hebrew שלום\nsecond",
            "\u{1}".repeat(4096).as_str(),
            "M".repeat(4096).as_str(),
        ] {
            let mut text = base.clone();
            text.text = content.into();
            text.rich_runs = Some(crate::RichTextDocument::plain(content.into()).runs);
            let doc = document(&text);
            let bound = shaping_scratch(&text, &faces, 1 << 30).unwrap();
            let (peak, allocations, reallocations) =
                super::super::shapes::fill_allocation_tests::observe(|| {
                    let shaped = shape_admitted(
                        &doc,
                        text.font_binding.as_ref().unwrap(),
                        &faces,
                        text.font_size,
                        &text.color,
                        Some(500),
                        0,
                    )
                    .unwrap();
                    assert!(!shaped.glyphs.is_empty());
                    drop(shaped);
                });
            eprintln!(
                "shape bytes={} peak={peak} bound={bound} alloc={allocations} realloc={reallocations}",
                content.len()
            );
            assert!(peak as u64 <= bound);
            assert!(allocations > 0);
        }
    }

    #[test]
    fn escaped_control_character_cache_key_peak_fits_growth_and_document_clone_bound() {
        let (mut text, _) =
            crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
        text.text = "\u{1}".repeat(4096);
        text.rich_runs = Some(crate::RichTextDocument::plain(text.text.clone()).runs);
        let owned = crate::evaluated_scene::composition_resources::text_heap(&text).unwrap()
            + size_of::<EvaluatedText>() as u64;
        let bound = 24 * owned + 65536;
        let (peak, _, _) = super::super::shapes::fill_allocation_tests::observe(|| {
            let doc = document(&text);
            let key = serde_json::to_string(&(
                &doc,
                &text.font_binding,
                text.font_size,
                &text.color,
                text.style.wrap_width_px,
                text.style.line_spacing_px,
            ))
            .unwrap();
            assert!(
                key.len() >= 6 * 4096,
                "JSON must expand every control character independently"
            );
            drop((key, doc));
        });
        assert!(peak as u64 <= bound);
    }

    #[test]
    fn active_painted_glyph_payload_clones_and_spare_capacity_are_charged() {
        let (mut text, faces) =
            crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
        let paints = (0..16)
            .map(|_| crate::TextPaintLayer::Fill {
                color: "#ff0011".into(),
                opacity: 0.5,
            })
            .collect();
        text.spans = Some(
            vec![crate::TextSpan {
                start: 0,
                end: 2,
                style: crate::TextSpanStyle {
                    paint_layers: Some(paints),
                    ..Default::default()
                },
            }]
            .into_boxed_slice(),
        );
        let doc = document(&text);
        let shaped = shape_admitted(
            &doc,
            text.font_binding.as_ref().unwrap(),
            &faces,
            text.font_size,
            &text.color,
            None,
            0,
        )
        .unwrap();
        assert!(
            shaped
                .glyphs
                .iter()
                .all(|g| g.paint_layers.as_ref().unwrap().len() == 16)
        );
        let heap =
            crate::evaluated_scene::composition_resources::shaped_heap_bytes(&shaped).unwrap();
        let (peak, _, _) = super::super::shapes::fill_allocation_tests::observe(|| {
            let first = shaped.clone();
            let second = first.clone();
            drop((first, second));
        });
        assert!(peak as u64 <= heap * 3);
        let mut spare = shaped.clone();
        spare.glyphs.reserve(100);
        spare.glyphs[0].face.reserve(100);
        spare.glyphs[0].color.reserve(100);
        spare.glyphs[0].paint_layers.as_mut().unwrap().reserve(100);
        assert!(
            crate::evaluated_scene::composition_resources::shaped_heap_bytes(&spare).unwrap()
                > heap
        );
        let mut cache = std::collections::HashMap::with_capacity(100);
        cache.insert("painted".into(), spare);
        let retained = shaped_cache_bytes(&cache).unwrap();
        let mut memory = MeasurementMemory::new(retained);
        memory.adopt(retained).unwrap();
        assert!(memory.admit(1).is_err());
    }

    #[test]
    fn cache_hit_clone_refuses_before_painted_payload_allocation_and_accepts_exact_boundary() {
        let (mut text, faces) =
            crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
        text.spans = Some(
            vec![crate::TextSpan {
                start: 0,
                end: 2,
                style: crate::TextSpanStyle {
                    paint_layers: Some(
                        (0..16)
                            .map(|_| crate::TextPaintLayer::Fill {
                                color: "#ff0011".into(),
                                opacity: 0.5,
                            })
                            .collect(),
                    ),
                    ..Default::default()
                },
            }]
            .into_boxed_slice(),
        );
        let doc = document(&text);
        let shaped = shape_admitted(
            &doc,
            text.font_binding.as_ref().unwrap(),
            &faces,
            text.font_size,
            &text.color,
            None,
            0,
        )
        .unwrap();
        assert!(
            shaped
                .glyphs
                .iter()
                .all(|g| g.paint_layers.as_ref().unwrap().len() == 16)
        );
        let mut cache = std::collections::HashMap::with_capacity(32);
        cache.insert("painted".to_owned(), shaped);
        let mut results = std::collections::HashMap::with_capacity(32);
        results.insert(
            "existing".to_owned(),
            super::super::text::measure(cache["painted"].clone(), &text, &faces).unwrap(),
        );
        let retained = shaped_cache_bytes(&cache)
            .unwrap()
            .checked_add(measured_bytes(&results).unwrap())
            .unwrap();
        let heap =
            crate::evaluated_scene::composition_resources::shaped_heap_bytes(&cache["painted"])
                .unwrap();
        let scratch = 1024;
        let required = 3 * heap + scratch;
        let before = cache["painted"].clone();
        let capacities = (
            cache.capacity(),
            results.capacity(),
            cache["painted"].glyphs.capacity(),
        );
        let mut insufficient = MeasurementMemory::new(retained + required - 1);
        insufficient.adopt(retained).unwrap();
        let (peak, _, _) = super::super::shapes::fill_allocation_tests::observe(|| {
            let error = clone_cached_shape(&insufficient, cache.get("painted").unwrap(), scratch)
                .unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidArgument);
            assert!(!error.retryable);
            drop(error);
        });
        assert!(
            peak < 512 && (peak as u64) < heap,
            "cache hit must refuse before glyph/face/color/paint cloning: peak={peak}, H={heap}"
        );
        assert_eq!(cache["painted"], before);
        assert_eq!(results["existing"].shaped.as_ref().unwrap().0, before);
        assert_eq!(
            (
                cache.capacity(),
                results.capacity(),
                cache["painted"].glyphs.capacity()
            ),
            capacities
        );
        assert_eq!(insufficient.retained, retained);
        let mut fitting = MeasurementMemory::new(retained + required);
        fitting.adopt(retained).unwrap();
        let (peak, allocations, _) = super::super::shapes::fill_allocation_tests::observe(|| {
            let clone =
                clone_cached_shape(&fitting, cache.get("painted").unwrap(), scratch).unwrap();
            assert_eq!(clone, before);
            drop(clone);
        });
        assert!(allocations > 0 && peak > 0);
        assert!((peak as u64) <= 3 * heap);
        assert_eq!(fitting.retained, retained);
        assert_eq!(cache["painted"], before);
    }

    #[test]
    fn opaque_font_census_rejects_before_any_shaping_at_remaining_bound() {
        let (_, faces) =
            crate::evaluated_scene::text_layout::tests::sample(crate::TextLayout::default());
        observed_shape_calls(true);
        for bytes in faces.values() {
            let bound = opaque_face_bytes(bytes, 1 << 30).unwrap();
            assert!(bound > 0);
            assert_eq!(
                opaque_face_bytes(bytes, bound - 1).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                opaque_face_bytes(bytes, 0).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(observed_shape_calls(false), 0);
    }

    #[test]
    fn aliased_logical_font_tables_stop_census_before_finishing_or_shaping() {
        let mut font = crate::fonts::DEFAULT_FACES[0].to_vec();
        let records = u16::from_be_bytes(font[4..6].try_into().unwrap()) as usize;
        let record = (0..records)
            .map(|n| 12 + 16 * n)
            .find(|&n| &font[n..n + 4] == b"GSUB")
            .unwrap();
        let mut table = vec![0, 1, 0, 0, 0, 10, 0, 12, 0, 14, 0, 0, 0, 0];
        table.extend_from_slice(&4096u16.to_be_bytes());
        for _ in 0..4096 {
            table.extend_from_slice(&8194u16.to_be_bytes());
        }
        table.extend_from_slice(&[0, 1, 0, 0]);
        table.extend_from_slice(&8192u16.to_be_bytes());
        for _ in 0..8192 {
            table.extend_from_slice(&16390u16.to_be_bytes());
        }
        table.extend_from_slice(&[0, 1, 0, 6, 0, 0, 0, 1, 0, 1, 0, 0]);
        let offset = font.len() as u32;
        font[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
        font[record + 12..record + 16].copy_from_slice(&(table.len() as u32).to_be_bytes());
        font.extend(table);
        let face = crate::fonts::validate_face(&font).unwrap();
        let gsub = face.tables().gsub.unwrap();
        assert_eq!(gsub.lookups.len(), 4096);
        assert_eq!(gsub.lookups.get(0).unwrap().subtables.len(), 8192);
        CENSUS_LOOKUPS.with(|n| n.set(0));
        observed_shape_calls(true);
        assert_eq!(
            opaque_face_bytes(&font, 2 * 1024 * 1024).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        let visited = CENSUS_LOOKUPS.with(|n| n.get());
        assert!(
            visited > 0 && visited < 10,
            "census must stop on first excess, visited={visited}"
        );
        assert_eq!(observed_shape_calls(false), 0);
    }

    #[test]
    fn admitted_cumulative_glyph_guard_precedes_face_color_and_paint_clones() {
        fn expanded_font(source: &[u8]) -> Vec<u8> {
            let mut font = source.to_vec();
            let glyph = crate::fonts::validate_face(&font)
                .unwrap()
                .glyph_index('A')
                .unwrap()
                .0;
            let records = u16::from_be_bytes(font[4..6].try_into().unwrap()) as usize;
            let record = (0..records)
                .map(|n| 12 + 16 * n)
                .find(|&n| &font[n..n + 4] == b"GSUB")
                .unwrap();
            let mut table = vec![
                0, 1, 0, 0, 0, 10, 0, 30, 0, 44, 0, 1, b'D', b'F', b'L', b'T', 0, 8, 0, 4, 0, 0, 0,
                0, 255, 255, 0, 1, 0, 0, 0, 1, b'c', b'c', b'm', b'p', 0, 8, 0, 0, 0, 1, 0, 0, 0,
                1, 0, 4, 0, 2, 0, 0, 0, 1, 0, 8, 0, 1, 0, 8, 0, 1, 0, 14, 0, 1, 0, 1,
            ];
            table.extend_from_slice(&glyph.to_be_bytes());
            table.extend_from_slice(&32u16.to_be_bytes());
            for _ in 0..32 {
                table.extend_from_slice(&glyph.to_be_bytes());
            }
            let offset = font.len() as u32;
            font[record + 8..record + 12].copy_from_slice(&offset.to_be_bytes());
            font[record + 12..record + 16].copy_from_slice(&(table.len() as u32).to_be_bytes());
            font.extend(table);
            font
        }
        let regular = expanded_font(crate::fonts::DEFAULT_FACES[0]);
        let bold = expanded_font(crate::fonts::DEFAULT_FACES[1]);
        let regular_hash = crate::fonts::record(&regular).unwrap().sha256;
        let bold_hash = crate::fonts::record(&bold).unwrap().sha256;
        let faces = BTreeMap::from([(regular_hash.clone(), regular), (bold_hash.clone(), bold)]);
        let binding = crate::FontBinding {
            regular: regular_hash.clone(),
            bold: bold_hash.clone(),
            italic: regular_hash,
            bold_italic: bold_hash,
            profile: crate::TEXT_LAYOUT_PROFILE.into(),
            warnings: vec![],
        };
        let doc = crate::RichTextDocument {
            spans: None,
            runs: vec![
                crate::RichTextRun {
                    text: "A".repeat(300),
                    bold: None,
                    italic: None,
                    color: None,
                },
                crate::RichTextRun {
                    text: "A".repeat(300),
                    bold: Some(true),
                    italic: None,
                    color: None,
                },
            ],
        };
        crate::fonts::shaping::observed_glyph_clones(true);
        let ordinary = crate::fonts::shaping::shape(&doc, &binding, &faces, 20, "#ffffff", None, 0)
            .unwrap_err();
        assert_eq!(ordinary.code, ErrorCode::InvalidArgument);
        let ordinary_clones = crate::fonts::shaping::observed_glyph_clones(true);
        assert_eq!(
            ordinary_clones, 19200,
            "two independently safe9600-glyph segments expose cumulative clone admission"
        );
        let admitted = shape_admitted(&doc, &binding, &faces, 20, "#ffffff", None, 0).unwrap_err();
        assert_eq!(admitted.code, ordinary.code);
        assert_eq!(
            crate::fonts::shaping::observed_glyph_clones(true),
            crate::MAX_SHAPED_GLYPHS
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn admitted_mac_union_and_failed_conversion_close_once_without_opening_cursor() {
        use std::cell::Cell;
        use std::os::fd::{AsRawFd, IntoRawFd};
        let root = tempfile::tempdir().unwrap();
        let converted = Cell::new(0);
        let closed = Cell::new(0);
        let file = std::fs::File::open(root.path()).unwrap();
        let error = checked_font_dir_with(
            file,
            |_| Ok(true),
            |fd| {
                converted.set(converted.get() + 1);
                nix::dir::Dir::from_fd(fd)
            },
            |fd| {
                closed.set(closed.get() + 1);
                nix::unistd::close(fd)
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(converted.get(), 0);
        assert_eq!(closed.get(), 0); // File still owns the pre-conversion FD.
        let file = std::fs::File::open(root.path()).unwrap();
        let raw = file.as_raw_fd();
        assert!(
            checked_font_dir_with(
                file,
                |_| Ok(false),
                |fd| {
                    assert_eq!(fd.into_raw_fd(), raw);
                    Err(nix::errno::Errno::ENOMEM)
                },
                |fd| {
                    assert_eq!(fd, raw);
                    closed.set(closed.get() + 1);
                    let result = nix::unistd::close(fd);
                    assert_eq!(result, Ok(()));
                    result
                }
            )
            .is_err()
        );
        assert_eq!(closed.get(), 1);
        let dir = checked_font_dir_with(
            std::fs::File::open(root.path()).unwrap(),
            |_| Ok(false),
            nix::dir::Dir::from_fd,
            |_| panic!("successful conversion must not close saved FD"),
        )
        .unwrap();
        drop(dir);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn admitted_mac_non_directory_conversion_closes_saved_descriptor() {
        use std::cell::Cell;
        use std::os::fd::AsRawFd;
        let file = tempfile::tempfile().unwrap();
        let raw = file.as_raw_fd();
        let closed = Cell::new(0);
        let error = checked_font_dir_with(
            file,
            |file| {
                Ok(nix::sys::statfs::fstatfs(file)?
                    .flags()
                    .contains(nix::mount::MntFlags::MNT_UNION))
            },
            nix::dir::Dir::from_fd,
            |fd| {
                assert_eq!(fd, raw);
                closed.set(closed.get() + 1);
                let result = nix::unistd::close(fd);
                assert_eq!(result, Ok(()));
                result
            },
        )
        .unwrap_err();
        assert_eq!(
            error.raw_os_error(),
            Some(nix::errno::Errno::ENOTDIR as i32)
        );
        assert_eq!(closed.get(), 1);
        // Do not inspect/re-close a now-free FD number: another parallel test
        // may reuse it. The actual successful close above certifies cleanup.
    }
}
