use serde_json::{Value, json};
#[path = "support/render_worker_startup.rs"]
mod render_worker_startup;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

struct Worker {
    child: Child,
    events: mpsc::Receiver<Value>,
    diagnostics: Arc<Mutex<String>>,
    #[cfg(feature = "raster-cache-test-hooks")]
    stats: mpsc::Receiver<Value>,
    #[cfg(feature = "raster-cache-test-hooks")]
    preview_stats: mpsc::Receiver<Value>,
}
impl Worker {
    fn start(root: &std::path::Path) -> Self {
        Self::start_with_ffmpeg(root, None)
    }
    fn start_with_ffmpeg(root: &std::path::Path, ffmpeg: Option<&std::path::Path>) -> Self {
        for name in ["projects", "exports", "media"] {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_opencut-headless"));
        command
            .arg("--render-worker")
            .env("OPENCUT_PROJECTS_DIR", root.join("projects"))
            .env("OPENCUT_EXPORTS_DIR", root.join("exports"))
            .env("OPENCUT_ALLOWED_MEDIA_DIRS", root.join("media"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(ffmpeg) = ffmpeg {
            if let Some(real) = std::env::var_os("OPENCUT_FFMPEG_PATH") {
                command.env("OPENCUT_TEST_REAL_FFMPEG_PATH", real);
            }
            command
                .env("OPENCUT_FFMPEG_PATH", ffmpeg)
                .env("OPENCUT_FFPROBE_PATH", ffmpeg);
        }
        let mut child = command.spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let (send, events) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Ok(event) = serde_json::from_str(&line) else {
                    break;
                };
                if send.send(event).is_err() {
                    break;
                }
            }
        });
        let diagnostics = Arc::new(Mutex::new(String::new()));
        let captured = diagnostics.clone();
        #[cfg(feature = "raster-cache-test-hooks")]
        let (stats_send, stats) = mpsc::channel();
        #[cfg(feature = "raster-cache-test-hooks")]
        let (preview_send, preview_stats) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                #[cfg(feature = "raster-cache-test-hooks")]
                if let Ok(value) = serde_json::from_str::<Value>(&line)
                    && let Some(stat) = value.get("rasterCacheTest")
                {
                    let _ = stats_send.send(stat.clone());
                }
                #[cfg(feature = "raster-cache-test-hooks")]
                if let Ok(value) = serde_json::from_str::<Value>(&line)
                    && let Some(stat) = value.get("previewCacheTest")
                {
                    let _ = preview_send.send(stat.clone());
                }
                let mut capture = captured.lock().unwrap();
                capture.push_str(&line);
                capture.push('\n');
            }
        });
        let result = Self {
            child,
            events,
            diagnostics,
            #[cfg(feature = "raster-cache-test-hooks")]
            stats,
            #[cfg(feature = "raster-cache-test-hooks")]
            preview_stats,
        };
        assert_eq!(result.receive(), fixture()["ready"]);
        result
    }
    fn receive(&self) -> Value {
        self.events
            .recv_timeout(Duration::from_secs(30))
            .unwrap_or_else(|error| panic!("worker: {error}; {}", self.diagnostics.lock().unwrap()))
    }
    fn request(&mut self, id: &str, request: Value) -> Value {
        writeln!(
            self.child.stdin.as_mut().unwrap(),
            "{}",
            json!({"requestId":id,"request":request})
        )
        .unwrap();
        loop {
            let event = self.receive();
            assert_eq!(event["requestId"], id);
            if event["event"]["type"] != "progress" {
                return event["event"].clone();
            }
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../contracts/render-worker-v1.json")).unwrap()
}

#[test]
fn additive_review_fixture_preserves_legacy_request_positions() {
    let contract = fixture();
    let operations: Vec<_> = contract["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["request"]["operation"].as_str().unwrap())
        .collect();
    assert_eq!(
        operations,
        [
            "render_preview",
            "render_preview_range",
            "render_draft_preview",
            "export_video",
            "render_review_range",
            "analyze_audio",
        ]
    );
}

#[test]
fn audio_analysis_worker_retains_correlated_typed_errors_and_legacy_rendering() {
    let root = tempfile::tempdir().unwrap();
    let mut worker = Worker::start(root.path());
    let contract = fixture();
    let request = contract["requests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["request"]["operation"] == "analyze_audio")
        .unwrap()["request"]
        .clone();
    for index in 0..2 {
        let error = worker.request(&format!("analysis-{index}"), request.clone());
        assert_eq!(error["type"], "error");
        assert_eq!(error["error"]["code"], "PROJECT_NOT_FOUND");
    }
    let error = worker.request("legacy-render", contract["requests"][0]["request"].clone());
    assert_eq!(error["error"]["code"], "PROJECT_NOT_FOUND");
}

#[test]
fn native_sampled_encoder_failure_is_safe_on_the_headless_wire() {
    if std::env::var_os("OPENCUT_FFMPEG_PATH").is_none() {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1"),
            "native encoder test requires FFmpeg"
        );
        return;
    }
    use opencut_editor_core::{EditorCore, PathPolicy, ProjectSettings};
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Encoder diagnostics",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let add=serde_json::from_value(json!({"operation":"add_rectangle","trackId":track,"width":8,"height":8,"color":"#ff0000","startMs":0,"durationMs":100,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap();
    let item = core.edit(&id, 0, add).unwrap().changed_ids[0].clone();
    core.edit(&id,1,serde_json::from_value(json!({"operation":"update_item","itemId":item,"effects":[{"type":"vignette","id":"identity","amount":0}],"motionBlur":{"shutterAngleDeg":180,"sampleCount":4}})).unwrap()).unwrap();
    let helper = root.path().join(if cfg!(windows) {
        "encoder.exe"
    } else {
        "encoder"
    });
    let compile = Command::new("rustc")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../crates/editor-core/tests/fixtures/failing_visual_encoder.rs"),
        )
        .arg("-o")
        .arg(&helper)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let dir = core.project_directory(&id).unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let mut worker = Worker::start_with_ffmpeg(root.path(), Some(&helper));
    let mut request = fixture()["requests"][1]["request"].clone();
    request["projectId"] = json!(id);
    request["expectedRevision"] = json!(2);
    request["startMs"] = json!(0);
    request["endMs"] = json!(100);
    request["width"] = json!(64);
    request["height"] = json!(64);
    request["fps"] = json!(10);
    request["includeAudio"] = json!(false);
    let event = worker.request("encoder-fault", request);
    assert_eq!(event["type"], "error", "{event}");
    assert_eq!(event["error"]["code"], "FFMPEG_FAILED");
    assert_eq!(event["error"]["retryable"], false);
    assert_eq!(event["error"]["failedStage"], "visual_prepare");
    assert_eq!(event["error"]["ffmpegExitCode"], 7);
    let excerpt = event["error"]["ffmpegStderrExcerpt"].as_str().unwrap();
    assert!(excerpt.len() <= 4096 && !excerpt.contains("private-review"));
    assert!(excerpt.contains("[path]"));
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    assert!(!std::fs::read_dir(&dir).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-work-")
    }));
    drop(worker);
    std::fs::remove_file(helper).unwrap();
}

#[test]
fn worker_rejects_mutations_and_keeps_typed_errors_correlated() {
    let root = tempfile::tempdir().unwrap();
    let mut worker = Worker::start(root.path());
    let contract = fixture();
    let error = worker.request(
        "mutation-1",
        contract["rejectedOperation"]["request"].clone(),
    );
    assert_eq!(
        error["error"]["code"],
        contract["rejectedOperationError"]["code"]
    );
    assert_eq!(error["error"]["retryable"], false);
    assert_eq!(
        std::fs::read_dir(root.path().join("projects"))
            .unwrap()
            .count(),
        0
    );
    for index in 0..2 {
        let error = worker.request(
            &format!("missing-{index}"),
            contract["requests"][0]["request"].clone(),
        );
        assert_eq!(error["error"]["code"], "PROJECT_NOT_FOUND");
    }
    let error = worker.request("../invalid", contract["requests"][0]["request"].clone());
    assert_eq!(error["error"]["code"], "INVALID_ARGUMENT");
}

#[cfg(feature = "raster-cache-test-hooks")]
#[test]
fn native_worker_reuses_rasters_across_requests_and_restarts_cold() {
    use opencut_editor_core::{EditorCore, FontConfig, PathPolicy, ProjectSettings};
    if std::env::var_os("OPENCUT_FFMPEG_PATH").is_none() {
        assert_ne!(
            std::env::var("OPENCUT_GOLDEN_REQUIRED").as_deref(),
            Ok("1"),
            "native worker tools required"
        );
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let fonts = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/editor-core/resources/fonts")
        .canonicalize()
        .unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path(), fonts.as_path()],
            root.path().join("exports"),
        )
        .unwrap(),
    )
    .with_font_config(FontConfig {
        roots: vec![fonts],
        default_path: None,
    });
    let id = core
        .create_project(
            "Worker",
            ProjectSettings {
                width: 160,
                height: 90,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let project = core.get_project(&id).unwrap();
    let edits: Vec<opencut_editor_core::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_text","trackId":project.tracks[1].id,"text":"Cache AV","fontFamily":"DejaVu Sans","fontSize":18,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}},
        {"operation":"add_shape","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":20,"height":20},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null},
        {"operation":"add_svg","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"svg":"<svg width=\"10\" height=\"10\"><rect width=\"10\" height=\"10\" fill=\"#00f\"/></svg>"}
    ])).unwrap();
    core.edit_batch(&id, 0, edits).unwrap();
    let project = core.get_project(&id).unwrap();
    let revision = project.revision;
    let frame =
        json!({"operation":"render_preview","projectId":id,"expectedRevision":revision,"timeMs":0});
    let mut worker = Worker::start(root.path());
    let first = worker.request("cold", frame.clone());
    assert_eq!(first["type"], "result", "{first}");
    let warm = worker.request("warm", frame.clone());
    assert_eq!(warm["type"], "result", "{warm}");
    let dir = core.project_directory(&id).unwrap();
    assert_eq!(
        std::fs::read(dir.join(first["result"]["relativePath"].as_str().unwrap())).unwrap(),
        std::fs::read(dir.join(warm["result"]["relativePath"].as_str().unwrap())).unwrap()
    );
    let mut range = fixture()["requests"][1]["request"].clone();
    range["projectId"] = json!(id);
    range["expectedRevision"] = json!(revision);
    assert_eq!(worker.request("range", range)["type"], "result");
    let mut export = fixture()["requests"][3]["request"].clone();
    export["projectId"] = json!(id);
    export["expectedRevision"] = json!(revision);
    assert_eq!(worker.request("export", export)["type"], "result");
    let draft = core.create_draft(&id,revision,vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":project.tracks[1].items[0].id(),"hidden":false})).unwrap()],None).unwrap();
    assert_eq!(
        worker.request(
            "draft",
            json!({"operation":"render_draft_preview","projectId":id,"draftId":draft.id,"timeMs":0})
        )["type"],
        "result"
    );
    let mut stale = frame.clone();
    stale["expectedRevision"] = json!(0);
    assert_eq!(
        worker.request("stale", stale)["error"]["code"],
        "REVISION_CONFLICT"
    );
    // Consume each diagnostic explicitly; stdout and stderr are separate streams.
    let stats: Vec<Value> = (0..6)
        .map(|_| worker.stats.recv_timeout(Duration::from_secs(5)).unwrap())
        .collect();
    let cold = stats.iter().find(|s| s["requestId"] == "cold").unwrap();
    let warm = stats.iter().find(|s| s["requestId"] == "warm").unwrap();
    assert_eq!(cold["misses"], 3);
    assert_eq!(warm["misses"], 3);
    assert_eq!(warm["hits"], 3);
    for stat in &stats {
        assert_eq!(stat["misses"], 3);
    }
    let mut fresh = Worker::start(root.path());
    let reopened = fresh.request("fresh", frame);
    assert_eq!(reopened["type"], "result");
    assert_eq!(
        fresh.stats.recv_timeout(Duration::from_secs(5)).unwrap()["misses"],
        3
    );
    assert_eq!(
        std::fs::read(dir.join(first["result"]["relativePath"].as_str().unwrap())).unwrap(),
        std::fs::read(dir.join(reopened["result"]["relativePath"].as_str().unwrap())).unwrap()
    );
    let authoritative = std::fs::read(dir.join("project.json")).unwrap();
    core.update_draft(&id,&draft.id,revision,vec![serde_json::from_value(json!({"operation":"update_item","itemId":project.tracks[1].items[0].id(),"text":"Updated draft"})).unwrap()],None).unwrap();
    let draft_request =
        json!({"operation":"render_draft_preview","projectId":id,"draftId":draft.id,"timeMs":0});
    let changed_draft = worker.request("changed-draft", draft_request.clone());
    assert_eq!(changed_draft["type"], "result");
    let fresh_draft = fresh.request("fresh-draft", draft_request.clone());
    assert_eq!(fresh_draft["type"], "result");
    let changed_pixels =
        std::fs::read(dir.join(changed_draft["result"]["relativePath"].as_str().unwrap())).unwrap();
    assert_ne!(
        changed_pixels,
        std::fs::read(dir.join(first["result"]["relativePath"].as_str().unwrap())).unwrap()
    );
    assert_eq!(
        changed_pixels,
        std::fs::read(dir.join(fresh_draft["result"]["relativePath"].as_str().unwrap())).unwrap()
    );
    assert_eq!(
        worker.stats.recv_timeout(Duration::from_secs(5)).unwrap()["misses"],
        4
    );
    assert_eq!(
        worker.request("repeat-draft", draft_request)["type"],
        "result"
    );
    assert_eq!(
        worker.stats.recv_timeout(Duration::from_secs(5)).unwrap()["misses"],
        4
    );
    assert_eq!(
        authoritative,
        std::fs::read(dir.join("project.json")).unwrap()
    );
    core.edit(&id, revision, serde_json::from_value(json!({"operation":"update_item","itemId":project.tracks[1].items[0].id(),"text":"Fresh snapshot"})).unwrap()).unwrap();
    let current = core.get_project(&id).unwrap();
    let updated = worker.request("updated",json!({"operation":"render_preview","projectId":id,"expectedRevision":current.revision,"timeMs":0}));
    assert_eq!(updated["type"], "result");
    assert_ne!(
        std::fs::read(dir.join(first["result"]["relativePath"].as_str().unwrap())).unwrap(),
        std::fs::read(dir.join(updated["result"]["relativePath"].as_str().unwrap())).unwrap()
    );
    assert_eq!(
        worker.stats.recv_timeout(Duration::from_secs(5)).unwrap()["misses"],
        7
    );
}

#[cfg(windows)]
#[test]
fn crashed_worker_terminates_renderer_descendants() {
    exercise_windows_crash_fixture(false);
    if std::env::var("OPENCUT_WINDOWS_STARTUP_COMPARISON").as_deref() == Ok("1") {
        exercise_windows_crash_fixture(true);
    }
}

#[cfg(windows)]
fn exercise_windows_crash_fixture(instrumented: bool) {
    use opencut_editor_core::{EditorCore, PathPolicy, ProjectSettings};
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::Threading::{
            OpenProcess, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, TerminateProcess,
            WaitForSingleObject,
        },
    };
    let root = tempfile::tempdir().unwrap();
    let pid_file = root.path().join("renderer.pid");
    let tool = root.path().join("fake-ffmpeg.cmd");
    let shell_entry = root.path().join("fixture-shell-entry.log");
    let powershell_stderr = root.path().join("fixture-powershell.stderr");
    let script = if instrumented {
        format!(
            "@echo off\r\necho shell-entered>> \"{}\"\r\nif \"%1\"==\"-version\" (echo ffmpeg version 7.1.1 & exit /b 0)\r\npowershell.exe -NoProfile -NonInteractive -Command \"$PID | Set-Content -LiteralPath '{}'; Start-Sleep -Seconds 120\" 2>\"{}\"\r\n",
            shell_entry.display(),
            pid_file.display(),
            powershell_stderr.display()
        )
    } else {
        // Keep the exact original fixture body for a controlled native comparison.
        format!(
            "@echo off\r\nif \"%1\"==\"-version\" (echo ffmpeg version 7.1.1 & exit /b 0)\r\npowershell.exe -NoProfile -NonInteractive -Command \"$PID | Set-Content -LiteralPath '{}'; Start-Sleep -Seconds 120\"\r\n",
            pid_file.display()
        )
    };
    std::fs::write(&tool, script).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Crash",
            ProjectSettings {
                width: 160,
                height: 90,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let mut worker = Worker::start_with_ffmpeg(root.path(), Some(&tool));
    writeln!(worker.child.stdin.as_mut().unwrap(),"{}",json!({"requestId":"crash","request":{"operation":"render_preview","projectId":id,"expectedRevision":0,"timeMs":0}})).unwrap();
    let started = std::time::Instant::now();
    let deadline = started + Duration::from_secs(10);
    let mut events = Vec::new();
    let mut events_truncated = false;
    let pid_text = loop {
        for _ in 0..=render_worker_startup::EVENT_LIMIT {
            let Ok(event) = worker.events.try_recv() else {
                break;
            };
            let failed = event["requestId"] != "crash"
                || matches!(event["event"]["type"].as_str(), Some("error" | "result"));
            if events.len() < render_worker_startup::EVENT_LIMIT {
                events.push(event);
            } else {
                events_truncated = true;
                *events.last_mut().unwrap() = event;
            }
            assert!(
                !failed,
                "worker terminated or mismatched the renderer request before PID observation; instrumented={instrumented}; elapsed={:?}; {}; {}; PID record: {}",
                started.elapsed(),
                render_worker_startup::evidence(
                    &shell_entry,
                    &powershell_stderr,
                    &events,
                    events_truncated,
                    &worker.diagnostics.lock().unwrap()
                ),
                render_worker_startup::process_evidence(worker.child.id()),
                render_worker_startup::pid_evidence(&pid_file)
            );
        }
        match std::fs::read_to_string(&pid_file) {
            Ok(contents) if contents.ends_with('\n') => break contents,
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied
                ) || error.raw_os_error() == Some(32) => {}
            Err(error) => panic!(
                "{error}; instrumented={instrumented}; elapsed={:?}; {}; {}; PID record: {}",
                started.elapsed(),
                render_worker_startup::evidence(
                    &shell_entry,
                    &powershell_stderr,
                    &events,
                    events_truncated,
                    &worker.diagnostics.lock().unwrap()
                ),
                render_worker_startup::process_evidence(worker.child.id()),
                render_worker_startup::pid_evidence(&pid_file)
            ),
        }
        assert!(
            std::time::Instant::now() < deadline,
            "renderer PID record never became readable; instrumented={instrumented}; elapsed={:?}; {}; {}; PID record: {}",
            started.elapsed(),
            render_worker_startup::evidence(
                &shell_entry,
                &powershell_stderr,
                &events,
                events_truncated,
                &worker.diagnostics.lock().unwrap()
            ),
            render_worker_startup::process_evidence(worker.child.id()),
            render_worker_startup::pid_evidence(&pid_file)
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let pid: u32 = pid_text
        .trim()
        .trim_start_matches('\u{feff}')
        .parse()
        .unwrap();
    eprintln!(
        "native startup observation: instrumented={instrumented}; elapsed={:?}; renderer PID={pid}; {}",
        started.elapsed(),
        render_worker_startup::process_evidence(worker.child.id())
    );
    // SAFETY: access is restricted to waiting/termination of the owned fixture PID.
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
    assert!(!handle.is_null());
    // SAFETY: OpenProcess returned a newly owned handle.
    let process = unsafe { OwnedHandle::from_raw_handle(handle) };
    // SAFETY: the handle remains live for each bounded wait.
    assert_eq!(
        unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
        WAIT_TIMEOUT
    );
    worker.child.kill().unwrap();
    worker.child.wait().unwrap();
    // SAFETY: the handle pins this exact child even if its PID is later recycled.
    let result = unsafe { WaitForSingleObject(process.as_raw_handle(), 5000) };
    if result != WAIT_OBJECT_0 {
        // Always clean up a failed regression's fixture, without touching other processes.
        unsafe {
            TerminateProcess(process.as_raw_handle(), 1);
        }
    }
    assert_eq!(
        result, WAIT_OBJECT_0,
        "renderer child survived an abrupt worker crash"
    );
}

#[cfg(feature = "raster-cache-test-hooks")]
#[test]
fn native_worker_reuses_encoded_previews_and_preserves_drafts_errors_restart() {
    use opencut_editor_core::{EditorCore, PathPolicy, ProjectSettings};
    if std::env::var_os("OPENCUT_FFMPEG_PATH").is_none() {
        assert_ne!(
            std::env::var("OPENCUT_RASTER_CACHE_TESTS_REQUIRED").as_deref(),
            Ok("1"),
            "required native backend missing"
        );
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            "Encoded preview",
            ProjectSettings {
                width: 32,
                height: 32,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let state = core.get_project(&id).unwrap();
    let write = core.edit(&id, 0, serde_json::from_value(json!({"operation":"add_solid_color","trackId":state.tracks[1].id,"startMs":0,"durationMs":1000,"color":"#112233","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap()).unwrap();
    let item = write.changed_ids[0].clone();
    let revision = write.revision;
    let dir = core.project_directory(&id).unwrap();
    let request =
        json!({"operation":"render_preview","projectId":id,"expectedRevision":revision,"timeMs":0});
    let mut worker = Worker::start(root.path());
    let cold = worker.request("encoded-cold", request.clone());
    let warm = worker.request("encoded-warm", request.clone());
    assert_eq!(cold["type"], "result", "{cold}");
    assert_eq!(warm["type"], "result", "{warm}");
    let read = |result: &Value| {
        std::fs::read(dir.join(result["result"]["relativePath"].as_str().unwrap())).unwrap()
    };
    assert_ne!(
        cold["result"]["relativePath"],
        warm["result"]["relativePath"]
    );
    assert_eq!(read(&cold), read(&warm));
    let cold_stat = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let warm_stat = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        (
            cold_stat["hits"].as_u64(),
            cold_stat["misses"].as_u64(),
            cold_stat["finalExecutions"].as_u64()
        ),
        (Some(0), Some(1), Some(1))
    );
    assert_eq!(
        (
            warm_stat["hits"].as_u64(),
            warm_stat["misses"].as_u64(),
            warm_stat["finalExecutions"].as_u64()
        ),
        (Some(1), Some(1), Some(1))
    );
    let range = json!({"operation":"render_preview_range","projectId":id,"expectedRevision":revision,"startMs":0,"endMs":1000,"width":32,"height":32,"fps":10,"includeAudio":true});
    let first = worker.request("encoded-range", range.clone());
    let repeat = worker.request("encoded-range-warm", range);
    assert_eq!(first["type"], "result", "{first}");
    assert_eq!(repeat["type"], "result", "{repeat}");
    assert_eq!(read(&first), read(&repeat));
    let _ = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let stat = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        (
            stat["hits"].as_u64(),
            stat["misses"].as_u64(),
            stat["finalExecutions"].as_u64()
        ),
        (Some(2), Some(2), Some(2))
    );
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let draft = core
        .create_draft(
            &id,
            revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"update_item","itemId":item,"color":"#abcdef"}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let draft_request =
        json!({"operation":"render_draft_preview","projectId":id,"draftId":draft.id,"timeMs":0});
    let changed = worker.request("encoded-draft", draft_request.clone());
    let changed_warm = worker.request("encoded-draft-warm", draft_request);
    assert_eq!(changed["type"], "result", "{changed}");
    assert_eq!(changed_warm["type"], "result", "{changed_warm}");
    assert_ne!(read(&changed), read(&cold));
    assert_eq!(read(&changed), read(&changed_warm));
    let _ = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    let stat = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        (
            stat["hits"].as_u64(),
            stat["misses"].as_u64(),
            stat["finalExecutions"].as_u64()
        ),
        (Some(3), Some(3), Some(3))
    );
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    let mut stale = request.clone();
    stale["expectedRevision"] = json!(0);
    assert_eq!(
        worker.request("encoded-stale", stale)["error"]["code"],
        "REVISION_CONFLICT"
    );
    let stat = worker
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(stat["finalExecutions"], 3);
    let mut fresh = Worker::start(root.path());
    let reopened = fresh.request("encoded-fresh", request);
    assert_eq!(read(&reopened), read(&cold));
    let stat = fresh
        .preview_stats
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert_eq!(
        (
            stat["hits"].as_u64(),
            stat["misses"].as_u64(),
            stat["finalExecutions"].as_u64()
        ),
        (Some(0), Some(1), Some(1))
    );
}
