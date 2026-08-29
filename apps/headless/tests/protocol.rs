use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

struct Harness {
    root: TempDir,
}

impl Harness {
    fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
        }
    }

    fn request(&self, request: Value) -> Output {
        let projects = self.root.path().join("projects");
        let media = self.root.path().join("media");
        let exports = self.root.path().join("exports");
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&exports).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
            .env("OPENCUT_PROJECTS_DIR", projects)
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
            .env("OPENCUT_EXPORTS_DIR", exports)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        serde_json::to_writer(child.stdin.take().unwrap(), &request).unwrap();
        child.wait_with_output().unwrap()
    }

    fn health_with_missing_rendering(&self) -> Output {
        let projects = self.root.path().join("health-projects");
        let media = self.root.path().join("health-media");
        let exports = self.root.path().join("health-exports");
        std::fs::create_dir_all(&projects).unwrap();
        std::fs::create_dir_all(&media).unwrap();
        std::fs::create_dir_all(&exports).unwrap();
        Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
            .arg("--health")
            .env("OPENCUT_PROJECTS_DIR", projects)
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
            .env("OPENCUT_EXPORTS_DIR", exports)
            .env(
                "OPENCUT_FFMPEG_PATH",
                self.root.path().join("missing-ffmpeg"),
            )
            .env(
                "OPENCUT_FFPROBE_PATH",
                self.root.path().join("missing-ffprobe"),
            )
            .output()
            .unwrap()
    }
}

fn event(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn result(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let envelope = event(output);
    assert_eq!(envelope["type"], "result");
    envelope["result"].clone()
}

#[test]
fn create_read_and_edit_use_result_envelopes_and_typed_ids() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({
        "operation": "create_project",
        "name": "Protocol project"
    })));
    let project_id = created["projectId"].as_str().unwrap();
    assert_eq!(created["revision"], 0);

    let state = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let overlay_id = state["project"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|track| track["trackType"] == "overlay")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let edited = result(&harness.request(json!({
        "operation": "edit",
        "projectId": project_id,
        "expectedRevision": 0,
        "edit": {
            "operation": "add_text",
            "trackId": overlay_id,
            "text": "contract",
            "startMs": 0,
            "durationMs": 1000,
            "fontSize": 48,
            "color": "#ffffff",
            "fontFamily": null,
            "transform": { "positionX": 0.0, "positionY": 0.0, "scale": 1.0, "opacity": 1.0 }
        }
    })));
    assert_eq!(edited["revision"], 1);
    assert_eq!(edited["changedIds"].as_array().unwrap().len(), 1);
}

#[test]
fn revision_conflicts_are_typed_error_envelopes_with_nonzero_exit() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({ "operation": "create_project", "name": "Conflict" })));
    let project_id = created["projectId"].as_str().unwrap();
    let output = harness.request(json!({
        "operation": "undo",
        "projectId": project_id,
        "expectedRevision": 99
    }));
    assert!(!output.status.success());
    assert_eq!(
        event(&output),
        json!({
            "type": "error",
            "error": {
                "code": "REVISION_CONFLICT",
                "message": "expected revision 99, current revision is 0",
                "retryable": true
            }
        })
    );
}

#[test]
fn malformed_and_unknown_fields_are_invalid_argument_errors() {
    let harness = Harness::new();
    for request in [
        json!({ "operation": "not_a_command" }),
        json!({ "operation": "status", "unexpected": true }),
        json!({ "operation": "get_state", "projectId": "missing", "startMs": 0 }),
    ] {
        let output = harness.request(request);
        assert!(!output.status.success());
        let envelope = event(&output);
        assert_eq!(envelope["type"], "error");
        assert_eq!(envelope["error"]["code"], "INVALID_ARGUMENT");
        assert_eq!(envelope["error"]["retryable"], false);
    }
}

#[test]
fn every_stdout_line_is_a_json_event_envelope() {
    let harness = Harness::new();
    let output = harness.request(json!({ "operation": "status" }));
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    let parsed: Value = serde_json::from_str(lines[0]).unwrap();
    assert!(matches!(
        parsed["type"].as_str(),
        Some("result" | "progress" | "error")
    ));
    assert!(output.stderr.is_empty());
}

#[test]
fn health_succeeds_when_editor_is_ready_and_rendering_is_degraded() {
    let harness = Harness::new();
    let output = harness.health_with_missing_rendering();
    let status = result(&output);
    assert_eq!(status["ready"], true);
    assert_eq!(status["subsystems"]["editor"]["ready"], true);
    assert_eq!(status["subsystems"]["rendering"]["ready"], false);
    let capabilities = status["capabilities"].as_array().unwrap();
    assert!(capabilities.contains(&json!("projects")));
    assert!(capabilities.contains(&json!("timeline")));
    assert!(!capabilities.contains(&json!("preview")));
    assert!(!capabilities.contains(&json!("export")));
}
