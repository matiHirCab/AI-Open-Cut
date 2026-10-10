use std::{
    collections::BTreeSet,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// An isolated test-process tree, excluding Cargo and unrelated sibling tests.
pub struct Sampler {
    stop: Arc<AtomicBool>,
    peak: Arc<AtomicU64>,
    samples: Arc<AtomicU64>,
    native: Arc<AtomicU64>,
    finals: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
}
impl Sampler {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let peak = Arc::new(AtomicU64::new(0));
        let samples = Arc::new(AtomicU64::new(0));
        let native = Arc::new(AtomicU64::new(0));
        let finals = Arc::new(AtomicU64::new(0));
        let final_counter = finals.clone();
        let worker = (stop.clone(), peak.clone(), samples.clone(), native.clone());
        let root = Pid::from_u32(std::process::id());
        let handle = thread::spawn(move || {
            let mut system = System::new();
            let mut final_processes = BTreeSet::new();
            loop {
                system.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing()
                        .with_memory()
                        .with_cmd(UpdateKind::OnlyIfNotSet),
                );
                let mut members = BTreeSet::from([root]);
                loop {
                    let before = members.len();
                    for (pid, process) in system.processes() {
                        if process.thread_kind().is_none()
                            && process
                                .parent()
                                .is_some_and(|parent| members.contains(&parent))
                        {
                            members.insert(*pid);
                        }
                    }
                    if members.len() == before {
                        break;
                    }
                }
                let mut bytes = 0_u64;
                let mut native_members = 0_u64;
                for pid in &members {
                    if let Some(process) = system.process(*pid) {
                        if process.thread_kind().is_some() {
                            continue;
                        }
                        bytes = bytes
                            .checked_add(process.memory())
                            .expect("resident byte sum overflow");
                        let name = process.name().to_string_lossy();
                        if *pid != root
                            && (name.starts_with("ffmpeg") || name.starts_with("ffprobe"))
                        {
                            native_members += 1;
                            if process.cmd().windows(3).any(|arguments| {
                                arguments[0] == "-map"
                                    && arguments[1] == "[video]"
                                    && std::path::Path::new(&arguments[2])
                                        .extension()
                                        .is_some_and(|extension| extension == "png")
                            }) && final_processes.insert(*pid)
                            {
                                final_counter.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
                worker.1.fetch_max(bytes, Ordering::Relaxed);
                // Publish completed final-process observations to the request barrier.
                worker.2.fetch_add(1, Ordering::Release);
                worker.3.fetch_max(native_members, Ordering::Relaxed);
                if worker.0.load(Ordering::Acquire) {
                    break;
                }
                thread::sleep(Duration::from_millis(5));
            }
        });
        Self {
            stop,
            peak,
            samples,
            native,
            finals,
            handle: Some(handle),
        }
    }
    /// Drain in-flight refreshes so one request's observations cannot count as the next.
    pub fn final_calls(&self) -> u64 {
        let minimum = self.samples.load(Ordering::Acquire) + 2;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while self.samples.load(Ordering::Acquire) < minimum {
            assert!(
                std::time::Instant::now() < deadline,
                "native sampler refresh stalled"
            );
            thread::sleep(Duration::from_millis(5));
        }
        self.finals.load(Ordering::Acquire)
    }
    fn join(&mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Release);
            handle.join()
        } else {
            Ok(())
        }
    }
    pub fn finish(mut self) -> (u64, u64, u64) {
        self.join().expect("join release process sampler");
        (
            self.peak.load(Ordering::Relaxed),
            self.samples.load(Ordering::Relaxed),
            self.native.load(Ordering::Relaxed),
        )
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        let result = self.join();
        if !thread::panicking() {
            result.expect("join release sampler on failure");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        process::{Child, Command, Stdio},
        time::Instant,
    };
    static CONTROL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct OwnedChild(Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            self.0.wait().expect("settle owned sampler child");
        }
    }
    #[test]
    #[ignore = "spawned only by exact owned collector controls"]
    fn allocation_child() {
        let root = env::var_os("OPENCUT_RELEASE_SAMPLER_CONTROL").expect("owned control directory");
        let root = std::path::PathBuf::from(root);
        let mode = env::var("OPENCUT_RELEASE_SAMPLER_MODE").unwrap();
        if mode == "exit" {
            std::process::exit(7);
        }
        if mode == "no_ready" {
            thread::sleep(Duration::from_secs(120));
            return;
        }
        let mut bytes = vec![0_u8; 32 * 1024 * 1024];
        bytes.fill(73);
        std::hint::black_box(&bytes);
        fs::write(root.join("ready"), "32MiB touched").unwrap();
        while !root.join("release").exists() {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(std::hint::black_box(bytes)[0], 73);
    }
    fn observe(
        mode: &str,
        required: u64,
        readiness: Duration,
        observation: Duration,
    ) -> Result<(u64, u64, u64), String> {
        let sampler = Sampler::start();
        sampler.final_calls();
        let baseline = sampler.peak.load(Ordering::Relaxed);
        let root = tempfile::tempdir().unwrap();
        let mut child = OwnedChild(
            Command::new(env::current_exe().unwrap())
                .args([
                    "--exact",
                    "--ignored",
                    "measurement::tests::allocation_child",
                ])
                .env("OPENCUT_RELEASE_SAMPLER_CONTROL", root.path())
                .env("OPENCUT_RELEASE_SAMPLER_MODE", mode)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + readiness;
        while !root.path().join("ready").exists() {
            if child.0.try_wait().unwrap().is_some() {
                return Err("child exited before readiness".into());
            }
            if Instant::now() >= deadline {
                return Err("readiness timeout".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        let deadline = Instant::now() + observation;
        while sampler
            .peak
            .load(Ordering::Relaxed)
            .saturating_sub(baseline)
            < required
        {
            if Instant::now() >= deadline {
                return Err("observation timeout".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
        fs::write(root.path().join("release"), []).unwrap();
        let values = sampler.finish();
        assert!(values.0 >= baseline + required && values.1 > 0);
        Ok(values)
    }
    #[test]
    fn actual_owned_child_allocation_and_stop_join() {
        let _guard = CONTROL_LOCK.lock().unwrap();
        observe(
            "hold",
            24 * 1024 * 1024,
            Duration::from_secs(5),
            Duration::from_secs(5),
        )
        .unwrap();
    }
    #[test]
    fn collector_readiness_observation_and_exit_failures_settle() {
        let _guard = CONTROL_LOCK.lock().unwrap();
        assert!(
            observe(
                "no_ready",
                1,
                Duration::from_millis(40),
                Duration::from_secs(5)
            )
            .unwrap_err()
            .contains("readiness")
        );
        assert!(
            observe("hold", u64::MAX, Duration::from_secs(5), Duration::ZERO)
                .unwrap_err()
                .contains("observation")
        );
        assert!(
            observe("exit", 1, Duration::from_secs(5), Duration::from_secs(5))
                .unwrap_err()
                .contains("exited")
        );
    }
}
