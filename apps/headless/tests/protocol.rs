use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

fn headless_contract() -> Value {
    serde_json::from_str(include_str!("../../../contracts/headless-protocol-v1.json")).unwrap()
}

fn error_catalog() -> Value {
    serde_json::from_str(include_str!("../../../contracts/error-codes-v1.json")).unwrap()
}

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
        let mut command = Command::new(env!("CARGO_BIN_EXE_opencut-headless"));
        command
            .env("OPENCUT_PROJECTS_DIR", projects)
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", media)
            .env("OPENCUT_EXPORTS_DIR", exports)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in ["OPENCUT_FFMPEG_PATH", "OPENCUT_FFPROBE_PATH"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        if let Some(value) = std::env::var_os("OPENCUT_TEST_FONT_PATH") {
            command.env("OPENCUT_DEFAULT_FONT_PATH", value);
        }
        let mut child = command.spawn().unwrap();
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

fn native_parity_is_configured() -> bool {
    let configured = [
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
        std::env::var_os("OPENCUT_TEST_FONT_PATH"),
    ];
    if configured.iter().all(Option::is_none) {
        return false;
    }
    assert!(
        configured.iter().all(Option::is_some),
        "native lifecycle parity requires FFmpeg, FFprobe, and font paths together"
    );
    true
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
                "failedStage": null,
                "ffmpegExitCode": null,
                "ffmpegStderrExcerpt": null,
                "message": "expected revision 99, current revision is 0",
                "retryable": true
            }
        })
    );

    let render = harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 99,
        "timeMs": 0
    }));
    assert!(!render.status.success());
    assert_eq!(event(&render)["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(
        event(&render)["error"]["message"],
        "expected revision 99, current revision is 0"
    );
}

#[test]
fn native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts() {
    if !native_parity_is_configured() {
        return;
    }
    let harness = Harness::new();
    let created = result(&harness.request(json!({
        "operation": "create_project",
        "name": "Native lifecycle",
        "settings": { "width": 160, "height": 90, "fps": 10 }
    })));
    let project_id = created["projectId"].as_str().unwrap();
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
    let transform = json!({
        "positionX": 0.0,
        "positionY": 0.0,
        "scale": 1.0,
        "opacity": 1.0
    });

    let edited = result(&harness.request(json!({
        "operation": "edit",
        "projectId": project_id,
        "expectedRevision": 0,
        "edit": {
            "operation": "add_solid_color",
            "trackId": overlay_id,
            "color": "#224466",
            "startMs": 0,
            "durationMs": 1000,
            "transform": transform
        }
    })));
    assert_eq!(edited["revision"], 1);
    let solid_id = edited["changedIds"][0].as_str().unwrap();
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 1,
        "timeMs": 500
    })));

    let undone = result(&harness.request(json!({
        "operation": "undo",
        "projectId": project_id,
        "expectedRevision": 1
    })));
    assert_eq!(undone["revision"], 2);
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 2,
        "timeMs": 0
    })));

    let redone = result(&harness.request(json!({
        "operation": "redo",
        "projectId": project_id,
        "expectedRevision": 2
    })));
    assert_eq!(redone["revision"], 3);
    result(&harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 3,
        "timeMs": 500
    })));

    let reopened = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let reopened_again = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    assert_eq!(reopened, reopened_again);
    assert_eq!(reopened["project"]["revision"], 3);

    let draft = result(&harness.request(json!({
        "operation": "create_draft",
        "projectId": project_id,
        "expectedRevision": 3,
        "label": "isolated render",
        "operations": [{
            "operation": "update_item",
            "itemId": solid_id,
            "color": "#ee8844"
        }]
    })));
    let draft_id = draft["id"].as_str().unwrap();
    let project_dir = harness.root.path().join("projects").join(project_id);
    let project_before = std::fs::read(project_dir.join("project.json")).unwrap();
    let history_before = std::fs::read(project_dir.join("history.json")).unwrap();
    let draft_path = project_dir.join("drafts").join(format!("{draft_id}.json"));
    let draft_before = std::fs::read(&draft_path).unwrap();
    let committed_before = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let draft_record_before = result(&harness.request(json!({
        "operation": "get_draft",
        "projectId": project_id,
        "draftId": draft_id
    })));
    let materialized_before = result(&harness.request(json!({
        "operation": "get_draft_state",
        "projectId": project_id,
        "draftId": draft_id
    })));

    result(&harness.request(json!({
        "operation": "render_draft_preview",
        "projectId": project_id,
        "draftId": draft_id,
        "timeMs": 500
    })));

    let committed_after = result(&harness.request(json!({
        "operation": "get_state",
        "projectId": project_id
    })));
    let draft_record_after = result(&harness.request(json!({
        "operation": "get_draft",
        "projectId": project_id,
        "draftId": draft_id
    })));
    let materialized_after = result(&harness.request(json!({
        "operation": "get_draft_state",
        "projectId": project_id,
        "draftId": draft_id
    })));
    assert_eq!(committed_after, committed_before);
    assert_eq!(committed_after["project"]["revision"], 3);
    assert_eq!(draft_record_after, draft_record_before);
    assert_eq!(materialized_after, materialized_before);
    assert_eq!(
        std::fs::read(project_dir.join("project.json")).unwrap(),
        project_before
    );
    assert_eq!(
        std::fs::read(project_dir.join("history.json")).unwrap(),
        history_before
    );
    assert_eq!(std::fs::read(draft_path).unwrap(), draft_before);
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
fn canonical_status_requests_negotiate_protocol_version_and_capabilities() {
    let harness = Harness::new();
    let contract = headless_contract();

    for request_name in ["statusDefault", "statusCurrent"] {
        let status = result(&harness.request(contract["requests"][request_name].clone()));
        assert_eq!(status["protocolVersion"], contract["version"]);
        assert_eq!(
            status["subsystems"]["editor"]["capabilities"],
            contract["status"]["editorCapabilities"]
        );
        for field in contract["status"]["requiredFields"].as_array().unwrap() {
            assert!(
                status.get(field.as_str().unwrap()).is_some(),
                "missing canonical status field {field}"
            );
        }
    }
}

#[test]
fn canonical_unsupported_version_and_unknown_field_are_stable_errors() {
    let harness = Harness::new();
    let contract = headless_contract();
    let expected_error = &contract["negotiation"]["unsupportedError"];
    let catalog = error_catalog();

    for request_name in ["statusUnsupported", "statusUnknownField"] {
        let output = harness.request(contract["requests"][request_name].clone());
        assert!(!output.status.success());
        let error = event(&output)["error"].clone();
        assert_eq!(error["code"], expected_error["code"]);
        assert_eq!(error["retryable"], expected_error["retryable"]);
        let code = error["code"].as_str().unwrap();
        assert_eq!(error["retryable"], catalog["codes"][code]["retryable"]);
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
    assert!(
        headless_contract()["events"]
            .as_array()
            .unwrap()
            .contains(&parsed["type"])
    );
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
    assert!(!capabilities.contains(&json!("evaluated_scene_rendering")));
}
