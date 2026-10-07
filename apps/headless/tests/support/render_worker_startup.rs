//! Test-only startup evidence; never changes renderer/process semantics.
use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

const SOURCE_LIMIT: usize = 16 * 1024;
pub const EVENT_LIMIT: usize = 32;

fn bounded(bytes: &[u8]) -> String {
    let end = bytes.len().min(SOURCE_LIMIT);
    let mut result = String::from_utf8_lossy(&bytes[..end]).into_owned();
    if bytes.len() > SOURCE_LIMIT {
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
}
