use serde_json::{Value, json};
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
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                #[cfg(feature = "raster-cache-test-hooks")]
                if let Ok(value) = serde_json::from_str::<Value>(&line)
                    && let Some(stat) = value.get("rasterCacheTest")
                {
                    let _ = stats_send.send(stat.clone());
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
    std::fs::write(&tool,format!("@echo off\r\nif \"%1\"==\"-version\" (echo ffmpeg version 7.1.1 & exit /b 0)\r\npowershell.exe -NoProfile -NonInteractive -Command \"$PID | Set-Content -LiteralPath '{}'; Start-Sleep -Seconds 120\"\r\n",pid_file.display())).unwrap();
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
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !pid_file.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let pid: u32 = std::fs::read_to_string(&pid_file)
        .unwrap_or_else(|error| panic!("{error}; {}", worker.diagnostics.lock().unwrap()))
        .trim()
        .trim_start_matches('\u{feff}')
        .parse()
        .unwrap();
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
