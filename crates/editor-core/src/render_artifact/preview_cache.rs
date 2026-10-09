//! Disposable encoded previews. Lookup is permitted only after ordinary preflight.
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    io::Write,
    sync::{Arc, Mutex},
};

pub(crate) type Key = [u8; 32];
const VERSION: &str = "opencut-preview-artifact-v1";
const MAX_ENTRIES: usize = 32;
const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
struct Entry {
    key: Key,
    bytes: Arc<[u8]>,
}
#[derive(Debug, Default)]
struct State {
    entries: VecDeque<Entry>,
    bytes: usize,
}

pub(crate) struct PreviewCache {
    state: Mutex<State>,
    max_entries: usize,
    max_bytes: usize,
    #[cfg(any(test, feature = "raster-cache-test-hooks"))]
    hits: std::sync::atomic::AtomicUsize,
    #[cfg(any(test, feature = "raster-cache-test-hooks"))]
    misses: std::sync::atomic::AtomicUsize,
}
impl std::fmt::Debug for PreviewCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewCache")
            .field("max_entries", &self.max_entries)
            .field("max_bytes", &self.max_bytes)
            .finish_non_exhaustive()
    }
}
impl Default for PreviewCache {
    fn default() -> Self {
        Self::new(MAX_ENTRIES, MAX_BYTES)
    }
}
impl PreviewCache {
    #[cfg(test)]
    pub(crate) fn test_with_limits(entries: usize, bytes: usize) -> Self {
        Self::new(entries, bytes)
    }

    fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            state: Mutex::default(),
            max_entries,
            max_bytes,
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            hits: Default::default(),
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            misses: Default::default(),
        }
    }
    pub(crate) fn payload_capacity(&self) -> usize {
        if self.max_entries == 0 {
            0
        } else {
            self.max_bytes.saturating_sub(size_of::<Key>())
        }
    }
    pub(crate) fn get(&self, key: Key) -> Option<Arc<[u8]>> {
        let mut state = self.state.lock().ok()?;
        if let Some(index) = state.entries.iter().position(|entry| entry.key == key) {
            let entry = state.entries.remove(index)?;
            let bytes = entry.bytes.clone();
            state.entries.push_back(entry);
            #[cfg(any(test, feature = "raster-cache-test-hooks"))]
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Some(bytes);
        }
        #[cfg(any(test, feature = "raster-cache-test-hooks"))]
        self.misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        None
    }
    pub(crate) fn insert(&self, key: Key, bytes: Vec<u8>) {
        let Some(cost) = bytes.len().checked_add(size_of::<Key>()) else {
            return;
        };
        if bytes.is_empty() || cost > self.max_bytes || self.max_entries == 0 {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if let Some(index) = state.entries.iter().position(|entry| entry.key == key) {
            let previous = state.entries.remove(index).unwrap();
            state.bytes -= previous.bytes.len() + size_of::<Key>();
        }
        while state.entries.len() >= self.max_entries || state.bytes > self.max_bytes - cost {
            let Some(previous) = state.entries.pop_front() else {
                return;
            };
            state.bytes -= previous.bytes.len() + size_of::<Key>();
        }
        state.bytes += cost;
        state.entries.push_back(Entry {
            key,
            bytes: bytes.into(),
        });
    }
    #[cfg(any(test, feature = "raster-cache-test-hooks"))]
    pub(crate) fn counts(&self) -> (usize, usize) {
        (
            self.hits.load(std::sync::atomic::Ordering::Relaxed),
            self.misses.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}

/// Stream validated identity: no materialized project JSON or approximate floats.
pub(crate) fn key(value: &impl Serialize) -> Option<Key> {
    struct HashWriter(Sha256);
    impl Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(Sha256::new());
    serde_json::to_writer(&mut writer, &(VERSION, value)).ok()?;
    Some(writer.0.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn k(index: u8) -> Key {
        [index; 32]
    }
    #[test]
    fn filesystem_payload_reader_admits_before_allocation_and_rejects_nonfiles() {
        use crate::render_artifact::{ArtifactIo, FileSystemArtifactIo};
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("encoded.png");
        std::fs::write(&path, [1u8; 9]).unwrap();
        let io = FileSystemArtifactIo;
        assert!(io.read_preview_payload(&path, 8).is_err());
        assert_eq!(io.read_preview_payload(&path, 9).unwrap(), vec![1; 9]);
        assert!(io.read_preview_payload(root.path(), 9).is_err());
        std::fs::write(&path, []).unwrap();
        assert!(io.read_preview_payload(&path, 9).is_err());
        #[cfg(unix)]
        {
            let link = root.path().join("link");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(io.read_preview_payload(&link, 9).is_err());
        }
    }

    #[test]
    fn inclusive_limits_lru_accounting_oversized_and_replacement() {
        let cache = PreviewCache::new(2, 80);
        assert_eq!(cache.payload_capacity(), 48);
        cache.insert(k(1), vec![1; 8]);
        cache.insert(k(2), vec![2; 8]);
        assert_eq!(cache.state.lock().unwrap().bytes, 80);
        assert_eq!(&*cache.get(k(1)).unwrap(), &[1; 8]);
        cache.insert(k(3), vec![3; 8]);
        assert!(cache.get(k(2)).is_none());
        assert!(cache.get(k(1)).is_some());
        cache.insert(k(4), vec![4; 49]);
        assert_eq!(cache.state.lock().unwrap().bytes, 80);
        assert!(cache.get(k(4)).is_none());
        cache.insert(k(1), vec![9; 1]);
        assert_eq!(cache.state.lock().unwrap().bytes, 73);
        assert_eq!(&*cache.get(k(1)).unwrap(), &[9]);
        cache.insert(k(5), vec![]);
        assert!(cache.get(k(5)).is_none());
        assert_eq!(cache.counts().0, 3);
        assert_eq!(PreviewCache::new(0, 80).payload_capacity(), 0);
    }
    #[test]
    fn clones_concurrent_inserts_and_poison_keep_bounded_immutable_payloads() {
        let cache = Arc::new(PreviewCache::new(4, 4 * 40));
        let threads: Vec<_> = (0..16)
            .map(|index| {
                let cache = cache.clone();
                std::thread::spawn(move || {
                    cache.insert(k(index), vec![index; 8]);
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        {
            let state = cache.state.lock().unwrap();
            assert!(state.entries.len() <= 4 && state.bytes <= 160);
            assert_eq!(
                state.bytes,
                state
                    .entries
                    .iter()
                    .map(|entry| 32 + entry.bytes.len())
                    .sum::<usize>()
            );
        }
        cache.insert(k(99), vec![99; 8]);
        let retained = cache.get(k(99)).unwrap();
        for index in 20..30 {
            cache.insert(k(index), vec![index; 8]);
        }
        assert_eq!(&*retained, &[99; 8]);
        let other = cache.clone();
        assert!(
            std::thread::spawn(move || {
                let _guard = other.state.lock().unwrap();
                panic!("poison optional cache");
            })
            .join()
            .is_err()
        );
        assert!(cache.get(k(29)).is_none());
        cache.insert(k(30), vec![30; 8]);
    }
    #[test]
    fn exact_stream_identity_preserves_nearby_floats_and_request_dimensions() {
        let baseline = (
            "project", 1_u64, "range", 0_u64, 1000_u64, 64, 64, 10, true, 0.25_f64,
        );
        let expected = key(&baseline).unwrap();
        assert_eq!(key(&baseline), Some(expected));
        assert_ne!(key(&("other", baseline)), Some(expected));
        assert_ne!(
            key(&(
                "project", 2_u64, "range", 0_u64, 1000_u64, 64, 64, 10, true, 0.25_f64
            )),
            Some(expected)
        );
        assert_ne!(
            key(&(
                "project",
                1_u64,
                "range",
                0_u64,
                1000_u64,
                64,
                64,
                10,
                true,
                f64::from_bits(0.25_f64.to_bits() + 1)
            )),
            Some(expected)
        );
    }
}
