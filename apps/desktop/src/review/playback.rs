//! Presentation-only playback of a completed, core-owned review artifact.

use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::Duration,
};

use super::{Artifact, Request};

#[cfg(target_os = "linux")]
mod native;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fault {
    Unsupported,
    Backend,
    Missing,
    Invalid,
    Decode,
    Busy,
    Stopped,
}

impl Fault {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => "Embedded playback is supported on Linux with libmpv 2 only.",
            Self::Backend => "Embedded playback needs an installed compatible libmpv 2 backend.",
            Self::Missing => {
                "The captured review file is missing or unavailable. Render a new review."
            }
            Self::Invalid => "This artifact cannot be played as a completed local range review.",
            Self::Decode => {
                "The playback backend could not decode or output this review's video/audio."
            }
            Self::Busy => "Playback controls are busy. Try the control again.",
            Self::Stopped => "Playback has stopped. Open the captured review again.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Loading,
    Paused,
    Playing,
    Ended,
    Error(Fault),
    Closed,
}

impl Phase {
    pub fn controls_available(self) -> bool {
        [Self::Paused, Self::Playing, Self::Ended].contains(&self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Command {
    Pause(bool),
    Seek(u64),
    Loop(bool),
}

#[derive(Clone)]
pub(crate) struct Snapshot {
    pub phase: Phase,
    pub position_ms: u64,
    pub looping: bool,
    pub frame: Option<Arc<Vec<u8>>>,
    pub frame_id: u64,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            phase: Phase::Loading,
            position_ms: 0,
            looping: false,
            frame: None,
            frame_id: 0,
        }
    }
}

pub(super) struct Sample {
    phase: Phase,
    position_ms: u64,
    frame: Option<Vec<u8>>,
}

trait Backend {
    fn command(&mut self, command: Command) -> Result<(), Fault>;
    fn sample(&mut self) -> Result<Sample, Fault>;
}

pub(crate) struct Player {
    pub project_id: String,
    pub revision: u64,
    pub path: std::path::PathBuf,
    pub start_ms: u64,
    pub duration_ms: u64,
    sender: Option<mpsc::SyncSender<Command>>,
    snapshot: Arc<Mutex<Snapshot>>,
    worker: Option<JoinHandle<()>>,
}

impl Player {
    pub fn open(artifact: &Artifact) -> Result<Self, Fault> {
        let Request::Range(options) = artifact.request else {
            return Err(Fault::Invalid);
        };
        let duration_ms = options
            .end_ms
            .checked_sub(options.start_ms)
            .filter(|n| *n > 0)
            .ok_or(Fault::Invalid)?;
        if artifact.rendered.mime_type != "video/mp4"
            || artifact.dimensions.0 == 0
            || artifact.dimensions.1 == 0
        {
            return Err(Fault::Invalid);
        }
        if !std::fs::symlink_metadata(&artifact.path)
            .is_ok_and(|m| m.file_type().is_file() && m.len() == artifact.rendered.size_bytes)
        {
            return Err(Fault::Missing);
        }
        if !cfg!(target_os = "linux") {
            return Err(Fault::Unsupported);
        }
        let source = artifact.clone();
        Self::start(artifact, duration_ms, move || {
            #[cfg(target_os = "linux")]
            {
                native::Native::open(&source).map(|backend| Box::new(backend) as Box<dyn Backend>)
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = source;
                Err(Fault::Unsupported)
            }
        })
    }

    fn start(
        artifact: &Artifact,
        duration_ms: u64,
        factory: impl FnOnce() -> Result<Box<dyn Backend>, Fault> + Send + 'static,
    ) -> Result<Self, Fault> {
        let Request::Range(options) = artifact.request else {
            return Err(Fault::Invalid);
        };
        let (sender, receiver) = mpsc::sync_channel(8);
        let snapshot = Arc::new(Mutex::new(Snapshot::default()));
        let output = snapshot.clone();
        let worker = thread::Builder::new()
            .name("review-playback".into())
            .spawn(move || {
                let result = run(factory, receiver, &output, duration_ms);
                let mut state = output.lock().unwrap_or_else(|p| p.into_inner());
                state.phase = result.err().map_or(Phase::Closed, Phase::Error);
                state.frame = None;
            })
            .map_err(|_| Fault::Backend)?;
        Ok(Self {
            project_id: artifact.project_id.clone(),
            revision: artifact.revision,
            path: artifact.path.clone(),
            start_ms: options.start_ms,
            duration_ms,
            sender: Some(sender),
            snapshot,
            worker: Some(worker),
        })
    }

    pub fn matches(&self, artifact: &Artifact) -> bool {
        self.project_id == artifact.project_id
            && self.revision == artifact.revision
            && self.path == artifact.path
    }

    pub fn snapshot(&self) -> Snapshot {
        let finished = self
            .worker
            .as_ref()
            .is_some_and(|worker| worker.is_finished());
        let mut state = self
            .snapshot
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone();
        if finished && !matches!(state.phase, Phase::Closed | Phase::Error(_)) {
            state.phase = Phase::Error(Fault::Decode);
            state.frame = None;
        }
        state
    }

    pub fn send(&self, command: Command) -> Result<(), Fault> {
        let command = match command {
            Command::Seek(time) => Command::Seek(time.min(self.duration_ms.saturating_sub(1))),
            other => other,
        };
        self.sender
            .as_ref()
            .ok_or(Fault::Stopped)?
            .try_send(command)
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => Fault::Busy,
                mpsc::TrySendError::Disconnected(_) => Fault::Stopped,
            })
    }

    pub fn stop(&mut self) {
        // Disconnect rather than enqueue shutdown behind a full control queue.
        self.sender.take();
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            let mut state = self.snapshot.lock().unwrap_or_else(|p| p.into_inner());
            state.phase = Phase::Error(Fault::Decode);
            state.frame = None;
        }
    }
}

/// Window-owned decoded-image state; at most one displayed and one pending image.
pub(crate) struct Presentation {
    pub id: u64,
    pub player: Player,
    pub feedback: Option<&'static str>,
    displayed: Option<std::sync::Arc<gpui::Image>>,
    pub rendered: Option<std::sync::Arc<gpui::RenderImage>>,
    pending: Option<(u64, std::sync::Arc<gpui::Image>)>,
    frame_id: u64,
    visible: Option<(Phase, u64, bool, u64)>,
}

impl Presentation {
    pub fn new(player: Player) -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            player,
            feedback: None,
            displayed: None,
            rendered: None,
            pending: None,
            frame_id: 0,
            visible: None,
        }
    }

    pub fn evict(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::App) {
        if let Some(image) = self.displayed.take() {
            gpui::ImageSource::Image(image).remove_asset(cx);
        }
        if let Some((_, image)) = self.pending.take() {
            gpui::ImageSource::Image(image).remove_asset(cx);
        }
        if let Some(image) = self.rendered.take() {
            cx.drop_image(image, window);
        }
    }

    pub fn update_image(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> bool {
        let snapshot = self.player.snapshot();
        let visible = (
            snapshot.phase,
            snapshot.position_ms,
            snapshot.looping,
            snapshot.frame_id,
        );
        let mut changed = self.visible != Some(visible);
        self.visible = Some(visible);
        if let Some((id, image)) = &self.pending
            && let Some(rendered) = image.clone().get_render_image(window, cx)
        {
            let id = *id;
            let image = image.clone();
            // Keep the pending cache entry; evict only the previous display.
            self.pending.take();
            self.evict(Some(&mut *window), cx);
            self.displayed = Some(image);
            self.rendered = Some(rendered);
            self.frame_id = id;
            changed = true;
        }
        if self.pending.is_none()
            && snapshot.frame_id != self.frame_id
            && let Some(bytes) = snapshot.frame
        {
            self.pending = Some((
                snapshot.frame_id,
                std::sync::Arc::new(gpui::Image::from_bytes(
                    gpui::ImageFormat::Bmp,
                    (*bytes).clone(),
                )),
            ));
            changed = true;
        }
        changed
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(
    factory: impl FnOnce() -> Result<Box<dyn Backend>, Fault>,
    receiver: mpsc::Receiver<Command>,
    output: &Mutex<Snapshot>,
    duration_ms: u64,
) -> Result<(), Fault> {
    let mut backend = factory()?;
    loop {
        match receiver.recv_timeout(Duration::from_millis(33)) {
            Ok(command) => {
                backend.command(command)?;
                if let Command::Loop(looping) = command {
                    output.lock().unwrap_or_else(|p| p.into_inner()).looping = looping;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let sample = backend.sample()?;
        let mut state = output.lock().unwrap_or_else(|p| p.into_inner());
        state.phase = sample.phase;
        state.position_ms = sample.position_ms.min(duration_ms);
        if let Some(frame) = sample.frame {
            state.frame_id = state.frame_id.wrapping_add(1);
            state.frame = Some(Arc::new(frame));
        }
    }
}

/// Presentation size is bounded independently of the captured source metadata.
#[cfg(any(target_os = "linux", test))]
fn display_size((width, height): (u32, u32)) -> (u32, u32) {
    let scale = (1280.0 / f64::from(width))
        .min(720.0 / f64::from(height))
        .min(1.0);
    (
        (f64::from(width) * scale).round().max(1.0) as u32,
        (f64::from(height) * scale).round().max(1.0) as u32,
    )
}

/// A bounded top-down, opaque BMP avoids another presentation dependency.
#[cfg(any(target_os = "linux", test))]
fn bitmap(width: u32, height: u32, bgr0: &[u32]) -> Vec<u8> {
    let stride = (width as usize * 3).next_multiple_of(4);
    let size = 54 + stride * height as usize;
    let mut bytes = vec![0; size];
    bytes[..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&(size as u32).to_le_bytes());
    bytes[10..14].copy_from_slice(&54u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&(width as i32).to_le_bytes());
    bytes[22..26].copy_from_slice(&(-(height as i32)).to_le_bytes());
    bytes[26..28].copy_from_slice(&1u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24u16.to_le_bytes());
    for y in 0..height as usize {
        for x in 0..width as usize {
            let pixel = bgr0[y * width as usize + x].to_ne_bytes();
            bytes[54 + y * stride + x * 3..54 + y * stride + x * 3 + 3]
                .copy_from_slice(&pixel[..3]);
        }
    }
    bytes
}
