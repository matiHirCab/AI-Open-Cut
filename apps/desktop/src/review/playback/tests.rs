use super::*;
use opencut_editor_core::{PreviewReviewOptions, RenderArtifact};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

fn artifact(path: PathBuf) -> Artifact {
    Artifact {
        project_id: "captured".into(),
        revision: 7,
        request: Request::Range(PreviewReviewOptions {
            start_ms: 2000,
            end_ms: 4000,
            resolution: opencut_editor_core::PreviewResolution::Preset(
                opencut_editor_core::PreviewPreset::Project,
            ),
            fps: None,
            include_audio: Some(true),
        }),
        dimensions: (64, 64),
        fps: 10,
        path,
        rendered: RenderArtifact {
            text_layouts: vec![],
            relative_path: "previews/range.mp4".into(),
            mime_type: "video/mp4".into(),
            size_bytes: 0,
            warnings: vec![],
        },
    }
}

fn wait(player: &Player, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let state = player.snapshot();
        if predicate(&state) {
            return state;
        }
        assert!(
            Instant::now() < end,
            "playback wait expired: {:?}",
            state.phase
        );
        thread::sleep(Duration::from_millis(10));
    }
}

struct Fake {
    commands: Arc<Mutex<Vec<Command>>>,
    released: Arc<AtomicUsize>,
    phase: Phase,
    position: u64,
}
impl Backend for Fake {
    fn command(&mut self, command: Command) -> Result<(), Fault> {
        self.commands.lock().unwrap().push(command);
        match command {
            Command::Pause(p) => self.phase = if p { Phase::Paused } else { Phase::Playing },
            Command::Seek(p) => self.position = p,
            Command::Loop(_) => {}
        }
        Ok(())
    }
    fn sample(&mut self) -> Result<Sample, Fault> {
        Ok(Sample {
            phase: self.phase,
            position_ms: self.position,
            frame: Some(vec![1, 2, 3]),
        })
    }
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.released.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn playback_controller_clamps_seeks_retains_identity_and_releases_worker() {
    let artifact = artifact("owned.mp4".into());
    let commands = Arc::new(Mutex::new(Vec::new()));
    let released = Arc::new(AtomicUsize::new(0));
    let (recorded, drops) = (commands.clone(), released.clone());
    let mut player = Player::start(&artifact, 2000, move || {
        Ok(Box::new(Fake {
            commands: recorded,
            released: drops,
            phase: Phase::Paused,
            position: 0,
        }))
    })
    .unwrap();
    wait(&player, |s| s.phase == Phase::Paused && s.frame.is_some());
    player.send(Command::Seek(u64::MAX)).unwrap();
    assert_eq!(wait(&player, |s| s.position_ms == 1999).position_ms, 1999);
    player.send(Command::Pause(false)).unwrap();
    wait(&player, |s| s.phase == Phase::Playing);
    player.send(Command::Loop(true)).unwrap();
    wait(&player, |s| s.looping);
    player.send(Command::Pause(true)).unwrap();
    wait(&player, |s| s.phase == Phase::Paused);
    player.send(Command::Loop(false)).unwrap();
    wait(&player, |s| !s.looping);
    assert_eq!(player.start_ms, 2000);
    assert_eq!(player.revision, 7);
    assert!(player.matches(&artifact));
    let mut revised = artifact.clone();
    revised.revision += 1;
    assert!(!player.matches(&revised));
    player.stop();
    assert_eq!(released.load(Ordering::SeqCst), 1);
    assert_eq!(player.snapshot().phase, Phase::Closed);
    assert_eq!(player.send(Command::Seek(0)), Err(Fault::Stopped));
    assert!(matches!(commands.lock().unwrap()[0], Command::Seek(1999)));
}

#[test]
fn playback_admission_is_bounded_and_failed_factory_is_truthful() {
    let artifact = artifact("owned.mp4".into());
    let (release, held) = mpsc::sync_channel(1);
    let mut player = Player::start(&artifact, 2000, move || {
        held.recv_timeout(Duration::from_secs(2)).unwrap();
        Err(Fault::Backend)
    })
    .unwrap();
    for _ in 0..8 {
        player.send(Command::Pause(false)).unwrap();
    }
    assert_eq!(player.send(Command::Pause(false)), Err(Fault::Busy));
    release.send(()).unwrap();
    wait(&player, |s| s.phase == Phase::Error(Fault::Backend));
    player.stop();
    assert_eq!(player.snapshot().phase, Phase::Error(Fault::Backend));
    assert!(player.snapshot().frame.is_none());
    assert!(!Fault::Backend.message().contains("owned.mp4"));
}

#[test]
fn playback_display_bounds_and_opaque_top_down_bitmap_are_exact() {
    assert_eq!(display_size((3840, 2160)), (1280, 720));
    assert_eq!(display_size((200, 2000)), (72, 720));
    assert_eq!(display_size((16, 8)), (16, 8));
    let pixels = [
        u32::from_ne_bytes([1, 2, 3, 99]),
        u32::from_ne_bytes([4, 5, 6, 88]),
    ];
    let image = bitmap(1, 2, &pixels);
    assert_eq!(&image[..2], b"BM");
    assert_eq!(&image[22..26], &(-2i32).to_le_bytes());
    assert_eq!(&image[54..], &[1, 2, 3, 0, 4, 5, 6, 0]);
    assert_eq!(image.len(), 62);
}

#[test]
fn playback_rejects_frame_and_zero_interval_without_starting_native_work() {
    let mut a = artifact("owned.mp4".into());
    a.request = Request::Frame(0);
    assert!(matches!(Player::open(&a), Err(Fault::Invalid)));
    a.request = Request::Range(PreviewReviewOptions {
        start_ms: 4000,
        end_ms: 2000,
        resolution: opencut_editor_core::PreviewResolution::Preset(
            opencut_editor_core::PreviewPreset::Project,
        ),
        fps: None,
        include_audio: Some(true),
    });
    assert!(matches!(Player::open(&a), Err(Fault::Invalid)));
}

#[test]
fn playback_worker_panic_is_visible_and_reaped() {
    let a = artifact("owned.mp4".into());
    let mut player = Player::start(&a, 2000, || panic!("injected worker failure")).unwrap();
    wait(&player, |s| s.phase == Phase::Error(Fault::Decode));
    player.stop();
    assert!(player.worker.is_none());
    assert!(player.snapshot().frame.is_none());
    assert_eq!(player.snapshot().phase, Phase::Error(Fault::Decode));
    assert_eq!(player.send(Command::Pause(false)), Err(Fault::Stopped));
}

#[test]
#[ignore = "requires real libmpv 2, FFmpeg and configured audio output; never a readiness skip"]
fn native_playback_pixels_audio_pause_seek_loop_and_cleanup() {
    assert_eq!(std::env::var("OPENCUT_TEST_MPV").as_deref(), Ok("1"));
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("original.mp4");
    let status = std::process::Command::new(
        std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into()),
    )
    .args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        "color=red:s=64x64:r=10:d=1",
        "-f",
        "lavfi",
        "-i",
        "color=blue:s=64x64:r=10:d=1",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=880:duration=2",
        "-filter_complex",
        "[0:v][1:v]concat=n=2:v=1:a=0[v]",
        "-map",
        "[v]",
        "-map",
        "2:a",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-y",
    ])
    .arg(&path)
    .status()
    .unwrap();
    assert!(status.success());
    let mut a = artifact(path.clone());
    a.rendered.size_bytes = path.metadata().unwrap().len();
    for _ in 0..3 {
        let mut player = Player::open(&a).unwrap();
        let state = wait(&player, |s| s.phase == Phase::Paused && s.frame.is_some());
        let b = state.frame.unwrap();
        assert!(b[56] > 220 && b[54] < 30, "red frame");
        player.send(Command::Seek(1400)).unwrap();
        let state = wait(&player, |s| {
            s.position_ms.abs_diff(1400) <= 200
                && s.frame.as_ref().is_some_and(|b| b[54] > 220 && b[56] < 30)
        });
        assert_eq!(state.phase, Phase::Paused);
        let before = state.position_ms;
        thread::sleep(Duration::from_millis(250));
        assert!(
            player.snapshot().position_ms.abs_diff(before) <= 200,
            "pause drift"
        );
        player.send(Command::Loop(true)).unwrap();
        player.send(Command::Pause(false)).unwrap();
        let looped = wait(&player, |s| {
            s.phase == Phase::Playing && s.position_ms < 400 && s.looping
        });
        thread::sleep(Duration::from_millis(500));
        let progressed = player.snapshot();
        assert_eq!(progressed.phase, Phase::Playing);
        assert!(
            progressed.position_ms.abs_diff(looped.position_ms + 500) <= 200,
            "clock must continue after loop: {} -> {}",
            looped.position_ms,
            progressed.position_ms
        );
        let frame = progressed.frame.as_ref().unwrap();
        assert!(
            frame[56] > 220 && frame[54] < 30,
            "decoded red frame after loop"
        );
        player.send(Command::Pause(true)).unwrap();
        let paused = wait(&player, |s| s.phase == Phase::Paused);
        thread::sleep(Duration::from_millis(250));
        assert!(
            player.snapshot().position_ms.abs_diff(paused.position_ms) <= 200,
            "explicit pause must stop the resumed clock"
        );
        player.stop();
        assert_eq!(player.snapshot().phase, Phase::Closed);
        assert!(player.worker.is_none());
        assert!(player.snapshot().frame.is_none());
    }
    let missing = artifact(root.path().join("missing.mp4"));
    assert!(matches!(Player::open(&missing), Err(Fault::Missing)));
    // The verification harness captures a real audio monitor with a realtime clock.
    let capture =
        std::env::var_os("OPENCUT_TEST_AUDIO_CAPTURE").expect("audio capture path required");
    let bytes = std::fs::read(capture).unwrap();
    assert!(
        bytes.len() > 4096 && bytes.iter().any(|b| *b != 0),
        "actual audio samples required"
    );
    #[cfg(target_os = "linux")]
    native::assert_audio_output_failure(&a);
    let short_path = root.path().join("short.mp4");
    assert!(
        std::process::Command::new(
            std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into())
        )
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(&path)
        .args(["-t", "0.4", "-c:v", "libx264", "-c:a", "aac", "-y"])
        .arg(&short_path)
        .status()
        .unwrap()
        .success()
    );
    let mut short = a.clone();
    short.path = short_path;
    short.rendered.size_bytes = short.path.metadata().unwrap().len();
    let Request::Range(mut options) = short.request else {
        unreachable!()
    };
    options.end_ms = options.start_ms + 400;
    short.request = Request::Range(options);
    let mut short_player = Player::open(&short).unwrap();
    wait(&short_player, |s| {
        s.phase == Phase::Paused && s.frame.is_some()
    });
    short_player.send(Command::Pause(false)).unwrap();
    wait(&short_player, |s| s.phase == Phase::Ended);
    short_player.stop();
    #[cfg(target_os = "linux")]
    native::assert_audio_output_failure(&short);

    std::fs::write(&path, b"invalid local movie").unwrap();
    a.rendered.size_bytes = path.metadata().unwrap().len();
    let player = Player::open(&a).unwrap();
    wait(&player, |s| s.phase == Phase::Error(Fault::Decode));
    drop(player);
}
