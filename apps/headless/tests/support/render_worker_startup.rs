//! Test-only startup evidence; never changes renderer/process semantics.
use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

const SOURCE_LIMIT: usize = 16 * 1024;
pub const EVENT_LIMIT: usize = 32;
const PROCESS_LIMIT: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessRecord {
    pub pid: u32,
    pub parent: u32,
    pub name: String,
}

fn descendants(root: u32, records: &[ProcessRecord]) -> (Vec<ProcessRecord>, bool) {
    let mut owned = std::collections::BTreeSet::from([root]);
    let mut result = Vec::new();
    loop {
        let mut added = false;
        for record in records {
            if record.pid != root && owned.contains(&record.parent) && owned.insert(record.pid) {
                if result.len() == PROCESS_LIMIT {
                    return (result, true);
                }
                result.push(record.clone());
                added = true;
            }
        }
        if !added {
            return (result, false);
        }
    }
}

#[cfg(windows)]
pub fn process_evidence(root: u32) -> String {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };
    // SAFETY: a read-only process snapshot is requested, without opening unrelated processes.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if handle == INVALID_HANDLE_VALUE {
        return format!(
            "process snapshot unavailable: {}",
            std::io::Error::last_os_error()
        );
    }
    // SAFETY: the successful call returned a new owned snapshot handle.
    let snapshot = unsafe { OwnedHandle::from_raw_handle(handle) };
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: the handle remains live and entry has the required initialized size/layout.
    if unsafe { Process32FirstW(snapshot.as_raw_handle(), &mut entry) } == 0 {
        return format!(
            "process snapshot unreadable: {}",
            std::io::Error::last_os_error()
        );
    }
    let mut records = Vec::new();
    let mut snapshot_truncated = false;
    loop {
        let length = entry
            .szExeFile
            .iter()
            .position(|character| *character == 0)
            .unwrap_or(entry.szExeFile.len());
        records.push(ProcessRecord {
            pid: entry.th32ProcessID,
            parent: entry.th32ParentProcessID,
            name: String::from_utf16_lossy(&entry.szExeFile[..length]),
        });
        if records.len() == 16_384 {
            snapshot_truncated = true;
            break;
        }
        // SAFETY: same live snapshot and initialized output structure as the first read.
        if unsafe { Process32NextW(snapshot.as_raw_handle(), &mut entry) } == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
                return format!("process snapshot enumeration failed: {error}");
            }
            break;
        }
    }
    let (owned, truncated) = descendants(root, &records);
    bounded(
        format!(
            "owned worker {root} descendants: {owned:?}; descendant limit reached: {truncated}; snapshot limit reached: {snapshot_truncated}"
        )
        .as_bytes(),
    )
}

fn bounded(bytes: &[u8]) -> String {
    let end = bytes.len().min(SOURCE_LIMIT);
    let mut result = String::from_utf8_lossy(&bytes[..end]).into_owned();
    let expanded = result.len() > SOURCE_LIMIT;
    if expanded {
        let mut boundary = SOURCE_LIMIT;
        while !result.is_char_boundary(boundary) {
            boundary -= 1;
        }
        result.truncate(boundary);
    }
    if bytes.len() > SOURCE_LIMIT || expanded {
        result.push_str(" [truncated at 16384 bytes]");
    }
    result
}

fn file_evidence(path: &Path) -> String {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return "missing".into(),
        Err(error) => return format!("unreadable ({:?})", error.kind()),
    };
    let mut bytes = Vec::new();
    match file.take((SOURCE_LIMIT + 1) as u64).read_to_end(&mut bytes) {
        Ok(_) => bounded(&bytes),
        Err(error) => format!("unreadable ({:?})", error.kind()),
    }
}

pub fn evidence(
    shell_entry: &Path,
    powershell_stderr: &Path,
    events: &[Value],
    events_truncated: bool,
    worker_stderr: &str,
) -> String {
    let mut result = format!(
        "fixture shell entry: {}; fixture PowerShell stderr: {}; worker stderr: {}; worker events:",
        file_evidence(shell_entry),
        file_evidence(powershell_stderr),
        bounded(worker_stderr.as_bytes())
    );
    for event in events.iter().take(EVENT_LIMIT) {
        result.push('\n');
        result.push_str(&bounded(event.to_string().as_bytes()));
    }
    if events_truncated || events.len() > EVENT_LIMIT {
        result.push_str("\n[worker events truncated at 32 events]");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn typed_failure_preserves_owned_startup_and_worker_evidence() {
        let root = tempfile::tempdir().unwrap();
        let shell = root.path().join("shell");
        let stderr = root.path().join("powershell");
        std::fs::write(&shell, "shell-entered\n").unwrap();
        std::fs::write(&stderr, "fixture startup failed\n").unwrap();
        let event = json!({"requestId":"crash","event":{"type":"error","error":{"code":"RENDER_FAILED","failedStage":"spawn"}}});
        let output = evidence(
            &shell,
            &stderr,
            std::slice::from_ref(&event),
            false,
            "worker diagnostic",
        );
        assert!(output.contains("shell-entered\n"));
        assert!(output.contains("fixture startup failed\n"));
        assert!(output.contains("worker diagnostic"));
        assert!(output.contains(&event.to_string()));
        assert!(!output.contains("truncated"));
    }

    #[test]
    fn missing_and_unreadable_owned_files_are_attributable() {
        let root = tempfile::tempdir().unwrap();
        let output = evidence(&root.path().join("missing"), root.path(), &[], false, "");
        assert!(output.contains("fixture shell entry: missing"));
        assert!(output.contains("fixture PowerShell stderr: unreadable"));
    }

    #[test]
    fn multibyte_sources_and_event_count_are_bounded_and_labeled() {
        let root = tempfile::tempdir().unwrap();
        let shell = root.path().join("shell");
        let large = "é".repeat(SOURCE_LIMIT);
        std::fs::write(&shell, &large).unwrap();
        let events = vec![json!({"marker":"retained-event"}); EVENT_LIMIT + 1];
        let output = evidence(&shell, &root.path().join("missing"), &events, false, &large);
        assert_eq!(output.matches("[truncated at 16384 bytes]").count(), 2);
        assert_eq!(output.matches("retained-event").count(), EVENT_LIMIT);
        assert!(output.contains("[worker events truncated at 32 events]"));
        let output = evidence(&shell, &root.path().join("missing"), &[], true, "");
        assert!(output.contains("[worker events truncated at 32 events]"));
        let missing = root.path().join("missing");
        let output = evidence(&missing, &missing, &[json!({"payload": large})], false, "");
        assert!(output.contains("[truncated at 16384 bytes]"));
        assert!(output.matches('é').count() < SOURCE_LIMIT);
    }

    #[test]
    fn invalid_utf8_expansion_stays_within_the_rendered_source_bound() {
        let output = bounded(&vec![0xff; SOURCE_LIMIT + 1]);
        let (source, label) = output.split_once(" [truncated").unwrap();
        assert!(source.len() <= SOURCE_LIMIT);
        assert!(source.ends_with('\u{fffd}'));
        assert_eq!(label, " at 16384 bytes]");
    }

    #[test]
    fn process_evidence_filters_unrelated_trees_and_resolves_reversed_ancestry() {
        let records = vec![
            ProcessRecord {
                pid: 3,
                parent: 2,
                name: "powershell.exe".into(),
            },
            ProcessRecord {
                pid: 9,
                parent: 8,
                name: "unrelated-hidden".into(),
            },
            ProcessRecord {
                pid: 1,
                parent: 0,
                name: "worker.exe".into(),
            },
            ProcessRecord {
                pid: 2,
                parent: 1,
                name: "cmd.exe".into(),
            },
        ];
        let (owned, truncated) = descendants(1, &records);
        assert_eq!(
            owned.iter().map(|record| record.pid).collect::<Vec<_>>(),
            [2, 3]
        );
        assert!(!truncated);
        assert!(!format!("{owned:?}").contains("unrelated-hidden"));
    }

    #[test]
    fn owned_process_evidence_is_capped_without_unrelated_disclosure() {
        let records = (2..=67)
            .map(|pid| ProcessRecord {
                pid,
                parent: 1,
                name: "owned.exe".into(),
            })
            .collect::<Vec<_>>();
        let (owned, truncated) = descendants(1, &records);
        assert_eq!(owned.len(), PROCESS_LIMIT);
        assert!(truncated);
        assert!(owned.iter().all(|record| record.parent == 1));
    }
}
