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
fn component_protocol_aliases_failures_and_exact_history() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Components"})));
    let id = created["projectId"].as_str().unwrap();
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/component-definitions-v1.json"
    ))
    .unwrap();
    let mut create = catalog["validOperations"][0].clone();
    create["resultAlias"] = json!("leaf");
    let batch = result(&harness.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[create]}),
    ));
    let component = batch["aliases"]["leaf"].clone();
    let state = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(state["project"]["schemaVersion"], 11);
    assert_eq!(state["project"]["components"][0]["id"], component);
    let dir = harness.root.path().join("projects").join(id);
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    for (revision, edit, code) in [
        (
            0,
            json!({"operation":"component_delete","componentId":component}),
            "REVISION_CONFLICT",
        ),
        (
            1,
            json!({"operation":"component_delete","componentId":"missing"}),
            "ITEM_NOT_FOUND",
        ),
        (
            1,
            json!({"operation":"component_delete","componentId":component,"resultAlias":null}),
            "INVALID_ARGUMENT",
        ),
    ] {
        let output = harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":revision,"operations":[catalog["validOperations"][0],edit]}));
        let failed = event(&output);
        assert_eq!(failed["error"]["code"], code);
        assert_eq!(failed["error"]["retryable"], code == "REVISION_CONFLICT");
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before.0);
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), before.1);
    }
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"component_delete","componentId":component}})));
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    let restored = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        restored["project"]["components"],
        state["project"]["components"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})));
    let removed = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(removed["project"]["components"], json!([]));
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
fn group_protocol_aliases_detachment_history_and_atomic_errors() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({"operation":"create_project","name":"Groups"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let created=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"parent":{"scope":"root","id":"@parent"},"resultAlias":"child"}
    ]})));
    let child = &created["aliases"]["child"];
    let parent = &created["aliases"]["parent"];
    let before = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        before["project"]["tracks"][1]["items"][1]["parent"]["id"],
        *parent
    );
    let bad = harness.request(
        json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[
            {"operation":"item_set_parent","itemId":child,"parent":null},
            {"operation":"item_set_parent","itemId":parent,"parent":{"scope":"root","id":"missing"}}
        ]}),
    );
    assert_eq!(event(&bad)["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id}))),
        before
    );
    result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{"operation":"item_set_parent","itemId":child,"parent":null}})));
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":2})));
    let restored = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        restored["project"]["tracks"][1]["items"],
        before["project"]["tracks"][1]["items"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":3})));
    let detached = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert!(
        detached["project"]["tracks"][1]["items"][1]
            .get("parent")
            .is_none()
    );
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
    let stale_project_dir = harness.root.path().join("projects").join(project_id);
    let stale_project_before = std::fs::read(stale_project_dir.join("project.json")).unwrap();
    let stale_history_before = std::fs::read(stale_project_dir.join("history.json")).unwrap();
    let stale_previews_before = std::fs::read_dir(stale_project_dir.join("previews"))
        .unwrap()
        .count();
    let stale = harness.request(json!({
        "operation": "render_preview",
        "projectId": project_id,
        "expectedRevision": 0,
        "timeMs": 500
    }));
    assert!(!stale.status.success());
    assert_eq!(event(&stale)["error"]["code"], "REVISION_CONFLICT");
    assert_eq!(
        std::fs::read(stale_project_dir.join("project.json")).unwrap(),
        stale_project_before
    );
    assert_eq!(
        std::fs::read(stale_project_dir.join("history.json")).unwrap(),
        stale_history_before
    );
    assert_eq!(
        std::fs::read_dir(stale_project_dir.join("previews"))
            .unwrap()
            .count(),
        stale_previews_before
    );
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
            "color": "#ee8844",
            "transform2d": {
                "position": {"x": 0.5, "y": 0.5, "unit": "normalized"},
                "anchor": {"x": 0.5, "y": 0.5}, "scaleX": 0.8, "scaleY": 0.7,
                "rotationDeg": 15, "skewXDeg": 4, "skewYDeg": -2, "opacity": 0.8
            }
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

#[test]
fn transform2d_round_trips_and_resets_through_public_protocol() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Transform2D"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let add=result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{
        "operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000",
        "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}
    }})));
    let item = &add["changedIds"][0];
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/transform2d-v1.json")).unwrap();
    let transform = &catalog["valid"][1]["value"];
    let update = result(&harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":{
            "operation":"update_item","itemId":item,"transform2d":transform
        }}),
    ));
    assert_eq!(update["revision"], 2);
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    let actual: opencut_editor_core::Transform2D =
        serde_json::from_value(state["project"]["tracks"][1]["items"][0]["transform2d"].clone())
            .unwrap();
    assert_eq!(actual, serde_json::from_value(transform.clone()).unwrap());
    let malformed = harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{
            "operation":"update_item","itemId":item,"transform2d":{"scaleX":1}
        }}),
    );
    assert_eq!(event(&malformed)["error"]["code"], "INVALID_ARGUMENT");
    result(&harness.request(
        json!({"operation":"edit","projectId":id,"expectedRevision":2,"edit":{
            "operation":"update_item","itemId":item,"transform2d":null
        }}),
    ));
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    assert!(
        state["project"]["tracks"][1]["items"][0]
            .get("transform2d")
            .is_none()
    );
}

#[test]
fn stacking_public_protocol_and_batch_aliases() {
    let harness = Harness::new();
    let status = result(&harness.request(json!({"operation":"status"})));
    assert!(
        status["capabilities"]
            .as_array()
            .unwrap()
            .contains(&json!("stacking"))
    );
    let created = result(&harness.request(json!({"operation":"create_project","name":"Stacking"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let added=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":30,"height":20,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":"box"},
        {"operation":"item_set_z_index","itemId":"@box","zIndex":-5},
        {"operation":"item_reorder","itemId":"@box","index":0},
        {"operation":"track_reorder","trackId":track,"index":0}
    ]})));
    assert_eq!(added["revision"], 1);
    let item = &added["aliases"]["box"];
    for (revision, edit) in [
        (
            1,
            json!({"operation":"item_set_z_index","itemId":item,"zIndex":2147483647}),
        ),
        (
            2,
            json!({"operation":"item_reorder","itemId":item,"index":0}),
        ),
        (
            3,
            json!({"operation":"track_reorder","trackId":track,"index":1}),
        ),
    ] {
        assert_eq!(
            result(&harness.request(
                json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":edit})
            ))["revision"],
            revision + 1
        );
    }
    let state = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert_eq!(
        state["project"]["schemaVersion"],
        opencut_editor_core::PROJECT_SCHEMA_VERSION
    );
    assert_eq!(
        state["project"]["tracks"][1]["items"][0]["zIndex"],
        2147483647
    );
    assert_eq!(state["project"]["tracks"][1]["items"][0]["stackOrder"], 0);
    let malformed=harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":4,"edit":{"operation":"item_set_z_index","itemId":item,"zIndex":0,"url":"https://example.com"}}));
    assert_eq!(event(&malformed)["error"]["code"], "INVALID_ARGUMENT");
}

#[test]
fn ungroup_protocol_workflow_aliases_rollback_and_history() {
    let harness = Harness::new();
    let created = result(&harness.request(json!({"operation":"create_project","name":"Ungroup"})));
    let id = &created["projectId"];
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let edit = |revision, value| {
        result(&harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":value}),
        ))
    };
    let group = edit(
        0,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
    );
    let group_id = &group["changedIds"][0];
    let child = edit(
        1,
        json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":20,"height":10,"color":"#ff0000","transform":{"positionX":7,"positionY":9,"scale":1,"opacity":1}}),
    );
    let child_id = &child["changedIds"][0];
    edit(
        2,
        json!({"operation":"item_set_parent","itemId":child_id,"parent":{"scope":"root","id":group_id}}),
    );
    edit(
        3,
        json!({"operation":"item_set_z_index","itemId":child_id,"zIndex":-7}),
    );
    let before = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    for (revision, target, code) in [
        (0, group_id.clone(), "REVISION_CONFLICT"),
        (4, json!("absent"), "ITEM_NOT_FOUND"),
        (4, child_id.clone(), "INVALID_ARGUMENT"),
    ] {
        let response = event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":revision,"edit":{"operation":"group_ungroup","groupId":target}})));
        assert_eq!(response["error"]["code"], code);
        assert_eq!(response["error"]["retryable"], code == "REVISION_CONFLICT");
    }
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/group-parent-v1.json")).unwrap();
    for fixture in catalog["invalid"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["value"]["operation"] == "group_ungroup")
    {
        let output = harness.request(
            json!({"operation":"edit","projectId":id,"expectedRevision":4,"edit":fixture["value"]}),
        );
        assert!(!output.status.success());
        assert_eq!(event(&output)["type"], "error");
    }
    let failed = event(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":4,"operations":[{"operation":"group_ungroup","groupId":group_id},{"operation":"group_ungroup","groupId":group_id}]})));
    assert_eq!(failed["error"]["code"], "ITEM_NOT_FOUND");
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id}))),
        before
    );
    edit(
        4,
        json!({"operation":"update_track","trackId":track,"locked":true}),
    );
    let locked=event(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":5,"edit":{"operation":"group_ungroup","groupId":group_id}})));
    assert_eq!(locked["error"]["code"], "TRACK_LOCKED");
    edit(
        5,
        json!({"operation":"update_track","trackId":track,"locked":false}),
    );
    let removed = edit(6, json!({"operation":"group_ungroup","groupId":group_id}));
    assert_eq!(removed["changedIds"], json!([group_id, child_id]));
    let after = result(&harness.request(json!({"operation":"open_project","projectId":id})));
    assert!(
        after["project"]["tracks"][1]["items"][0]
            .get("parent")
            .is_none()
    );
    assert_eq!(after["project"]["tracks"][1]["items"][0]["zIndex"], -7);
    result(&harness.request(json!({"operation":"undo","projectId":id,"expectedRevision":7})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        before["project"]["tracks"]
    );
    result(&harness.request(json!({"operation":"redo","projectId":id,"expectedRevision":8})));
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        after["project"]["tracks"]
    );
    let batch=result(&harness.request(json!({"operation":"edit_batch","projectId":id,"expectedRevision":9,"operations":[
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"resultAlias":"g"},
        {"operation":"item_set_parent","itemId":child_id,"parent":{"scope":"root","id":"@g"}},
        {"operation":"item_set_z_index","itemId":"@g","zIndex":5},
        {"operation":"group_ungroup","groupId":"@g"}
    ]})));
    assert_eq!(batch["revision"], 10);
    assert!(batch["aliases"]["g"].is_string());
    assert_eq!(
        result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]["tracks"],
        after["project"]["tracks"]
    );
}

#[test]
fn ungroup_null_alias_batch_is_rejected_before_any_publication() {
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Null alias"})));
    let id = created["projectId"].as_str().unwrap();
    let state = result(&harness.request(json!({"operation":"get_state","projectId":id})));
    let track = &state["project"]["tracks"][1]["id"];
    let created=result(&harness.request(json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":{"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}})));
    let group = &created["changedIds"][0];
    let directory = harness.root.path().join("projects").join(id);
    let snapshot = || {
        (
            std::fs::read(directory.join("project.json")).unwrap(),
            std::fs::read(directory.join("history.json")).unwrap(),
        )
    };
    let before = snapshot();
    for alias in [Value::Null, json!("removed"), json!(42)] {
        let malformed = json!({"operation":"group_ungroup","groupId":group,"resultAlias":alias});
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":1,"edit":malformed}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":1,"operations":[{"operation":"item_set_z_index","itemId":group,"zIndex":7},malformed]}),
        ] {
            let output = harness.request(request);
            assert!(!output.status.success());
            assert_eq!(event(&output)["error"]["code"], "INVALID_ARGUMENT");
            assert_eq!(event(&output)["error"]["retryable"], false);
            assert_eq!(snapshot(), before);
            assert_eq!(
                result(&harness.request(json!({"operation":"open_project","projectId":id})))["project"]
                    ["revision"],
                1
            );
        }
    }
}

#[test]
fn component_nested_input_failures_preserve_headless_generation() {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../contracts/component-definitions-v1.json"
    ))
    .unwrap();
    let harness = Harness::new();
    let created =
        result(&harness.request(json!({"operation":"create_project","name":"Nested validation"})));
    let id = created["projectId"].as_str().unwrap();
    let dir = harness.root.path().join("projects").join(id);
    let before = (
        std::fs::read(dir.join("project.json")).unwrap(),
        std::fs::read(dir.join("history.json")).unwrap(),
    );
    for fixture in catalog["itemValidationFixtures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            f["valid"] == false && f["operation"]["tracks"][0]["items"][0]["type"] == "text"
        })
    {
        for request in [
            json!({"operation":"edit","projectId":id,"expectedRevision":0,"edit":fixture["operation"]}),
            json!({"operation":"edit_batch","projectId":id,"expectedRevision":0,"operations":[catalog["validOperations"][0],fixture["operation"]]}),
        ] {
            let failed = event(&harness.request(request));
            assert_eq!(
                failed["error"]["code"], "INVALID_ARGUMENT",
                "{}",
                fixture["id"]
            );
            assert_eq!(failed["error"]["retryable"], false);
            assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before.0);
            assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), before.1);
        }
    }
}
