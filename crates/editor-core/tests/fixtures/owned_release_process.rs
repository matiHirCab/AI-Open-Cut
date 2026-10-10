//! Private verification runner: own descendants on success, error and timeout.
use std::{
    env,
    process::{exit, Command},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
mod platform {
    use std::{
        collections::BTreeMap,
        io,
        os::unix::process::CommandExt,
        process::{Child, Command},
        sync::atomic::{AtomicBool, Ordering},
    };
    static INTERRUPTED: AtomicBool = AtomicBool::new(false);
    unsafe extern "C" {
        fn setsid() -> i32;
        fn kill(pid: i32, signal: i32) -> i32;
        fn signal(signal: i32, handler: usize) -> usize;
    }
    extern "C" fn interrupt(_: i32) {
        INTERRUPTED.store(true, Ordering::Relaxed);
    }
    // Retain identities of observed detached descendants as well as the root group.
    // Linux subreaper ownership also catches orphans after intermediate parents exit.
    pub struct Tree {
        group: i32,
        owned: std::cell::RefCell<BTreeMap<i32, String>>,
    }
    #[cfg(target_os = "linux")]
    unsafe extern "C" {
        fn prctl(option: i32, arg2: usize, arg3: usize, arg4: usize, arg5: usize) -> i32;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }
    #[cfg(target_os = "linux")]
    fn processes() -> BTreeMap<i32, (i32, String)> {
        let mut result = BTreeMap::new();
        for entry in std::fs::read_dir("/proc")
            .expect("owned Linux process snapshot")
            .flatten()
        {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() else {
                continue;
            };
            let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
                continue;
            };
            let Some((_, fields)) = stat.rsplit_once(") ") else {
                continue;
            };
            let fields: Vec<_> = fields.split_whitespace().collect();
            if let (Some(parent), Some(start)) = (
                fields.get(1).and_then(|field| field.parse::<i32>().ok()),
                fields.get(19),
            ) {
                result.insert(pid, (parent, (*start).to_owned()));
            }
        }
        result
    }
    #[cfg(not(target_os = "linux"))]
    fn processes() -> BTreeMap<i32, (i32, String)> {
        let output = Command::new("/bin/ps")
            .args(["-axo", "pid=,ppid=,lstart="])
            .output()
            .expect("owned POSIX process snapshot");
        assert!(
            output.status.success(),
            "owned POSIX process snapshot failed"
        );
        String::from_utf8(output.stdout)
            .expect("process snapshot encoding")
            .lines()
            .filter_map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                Some((
                    fields.first()?.parse().ok()?,
                    (fields.get(1)?.parse().ok()?, fields.get(2..)?.join(" ")),
                ))
            })
            .collect()
    }
    impl Tree {
        pub fn spawn(command: &mut Command) -> io::Result<(Child, Self)> {
            #[cfg(target_os = "linux")]
            if unsafe { prctl(36, 1, 0, 0, 0) } != 0 {
                return Err(io::Error::last_os_error());
            }
            unsafe {
                signal(2, interrupt as *const () as usize);
                signal(15, interrupt as *const () as usize);
                command.pre_exec(|| {
                    if setsid() < 0 {
                        Err(io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                });
            }
            let child = command.spawn()?;
            let tree = Self {
                group: i32::try_from(child.id()).expect("owned process group"),
                owned: Default::default(),
            };
            tree.refresh();
            Ok((child, tree))
        }
        pub fn refresh(&self) {
            let snapshot = processes();
            let mut owned = self.owned.borrow_mut();
            for root in [self.group, i32::try_from(std::process::id()).unwrap()] {
                if let Some((_, identity)) = snapshot.get(&root) {
                    owned.entry(root).or_insert_with(|| identity.clone());
                }
            }
            loop {
                let before = owned.len();
                for (pid, (parent, identity)) in &snapshot {
                    if owned.get(parent).is_some_and(|owned_identity| {
                        snapshot
                            .get(parent)
                            .is_some_and(|(_, current)| current == owned_identity)
                    }) {
                        owned.entry(*pid).or_insert_with(|| identity.clone());
                    }
                }
                if owned.len() == before {
                    break;
                }
            }
        }
        pub fn interrupted(&self) -> bool {
            INTERRUPTED.load(Ordering::Relaxed)
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            self.refresh();
            let snapshot = processes();
            unsafe {
                if !snapshot.get(&self.group).is_some_and(|(_, current)| {
                    self.owned
                        .borrow()
                        .get(&self.group)
                        .is_some_and(|original| original != current)
                }) {
                    kill(-self.group, 9);
                }
                for (pid, identity) in self.owned.borrow().iter() {
                    if *pid != std::process::id() as i32
                        && snapshot
                            .get(pid)
                            .is_some_and(|(_, current)| current == identity)
                    {
                        kill(*pid, 9);
                    }
                }
            }
            #[cfg(target_os = "linux")]
            {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    let result = unsafe { waitpid(-1, std::ptr::null_mut(), 1) };
                    if result < 0 {
                        break;
                    }
                    assert!(
                        std::time::Instant::now() < deadline,
                        "owned descendants did not settle after termination"
                    );
                    if result == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                }
            }
        }
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        ffi::c_void,
        io,
        mem::{size_of, zeroed},
        os::windows::{io::AsRawHandle, process::CommandExt},
        process::{Child, Command},
        ptr::null_mut,
    };
    type Handle = *mut c_void;
    #[repr(C)]
    struct BasicLimit {
        process_time: i64,
        job_time: i64,
        flags: u32,
        minimum: usize,
        maximum: usize,
        active: u32,
        affinity: usize,
        priority: u32,
        scheduling: u32,
    }
    #[repr(C)]
    struct IoCounters {
        read_ops: u64,
        write_ops: u64,
        other_ops: u64,
        read_bytes: u64,
        write_bytes: u64,
        other_bytes: u64,
    }
    #[repr(C)]
    struct ExtendedLimit {
        basic: BasicLimit,
        io: IoCounters,
        process_memory: usize,
        job_memory: usize,
        peak_process: usize,
        peak_job: usize,
    }
    #[repr(C)]
    struct ThreadEntry {
        size: u32,
        usage: u32,
        id: u32,
        owner: u32,
        base_priority: i32,
        delta_priority: i32,
        flags: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attributes: *mut c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: i32, info: *const c_void, bytes: u32)
            -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, process: u32) -> Handle;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
        fn ResumeThread(thread: Handle) -> u32;
    }
    pub struct Tree(Handle);
    fn resume_owned(child: &Child) -> io::Result<()> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(0x4, 0) }; // TH32CS_SNAPTHREAD
        if snapshot as isize == -1 {
            return Err(io::Error::last_os_error());
        }
        let snapshot = Tree(snapshot);
        let mut entry: ThreadEntry = unsafe { zeroed() };
        entry.size = size_of::<ThreadEntry>() as u32;
        let mut available = unsafe { Thread32First(snapshot.0, &mut entry) } != 0;
        while available {
            if entry.owner == child.id() {
                let thread = unsafe { OpenThread(0x2, 0, entry.id) }; // THREAD_SUSPEND_RESUME
                if thread.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let thread = Tree(thread);
                if unsafe { ResumeThread(thread.0) } != 1 {
                    return Err(io::Error::other(
                        "owned primary thread did not resume from one suspend",
                    ));
                }
                return Ok(());
            }
            entry.size = size_of::<ThreadEntry>() as u32;
            available = unsafe { Thread32Next(snapshot.0, &mut entry) } != 0;
        }
        Err(io::Error::other("owned suspended primary thread absent"))
    }
    impl Tree {
        pub fn spawn(command: &mut Command) -> io::Result<(Child, Self)> {
            let job = unsafe { CreateJobObjectW(null_mut(), std::ptr::null()) };
            if job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let tree = Self(job);
            let mut info: ExtendedLimit = unsafe { zeroed() };
            info.basic.flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            if unsafe {
                SetInformationJobObject(
                    job,
                    9,
                    &info as *const _ as *const c_void,
                    size_of::<ExtendedLimit>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            // Assign while suspended: no descendant can escape before job ownership.
            let mut child = command.creation_flags(0x4).spawn()?;
            let process = child.as_raw_handle();
            if unsafe { AssignProcessToJobObject(job, process) } == 0 {
                let error = io::Error::last_os_error();
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
            if let Err(error) = resume_owned(&child) {
                drop(tree);
                let _ = child.wait();
                return Err(error);
            }
            Ok((child, tree))
        }
        pub fn interrupted(&self) -> bool {
            false
        }
        pub fn refresh(&self) {}
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

fn run() -> Result<i32, String> {
    let mut arguments = env::args_os().skip(1);
    let timeout = arguments
        .next()
        .ok_or("missing owned timeout")?
        .to_str()
        .ok_or("invalid owned timeout")?
        .parse::<u64>()
        .map_err(|_| "invalid owned timeout")?;
    if timeout == 0 || timeout > 7_200_000 {
        return Err("owned timeout out of range".into());
    }
    let executable = arguments.next().ok_or("missing owned executable")?;
    let mut command = Command::new(executable);
    command.args(arguments);
    let (mut child, tree) =
        platform::Tree::spawn(&mut command).map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_millis(timeout);
    loop {
        tree.refresh();
        match child.try_wait() {
            Ok(Some(status)) => {
                drop(tree);
                return Ok(status.code().unwrap_or(1));
            }
            Ok(None) if !tree.interrupted() && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10))
            }
            Ok(None) => {
                drop(tree);
                let _ = child.kill();
                let _ = child.wait();
                return Err("owned process timed out or interrupted".into());
            }
            Err(error) => {
                drop(tree);
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
}
fn main() {
    match run() {
        Ok(code) => exit(code),
        Err(error) => {
            eprintln!("release runner: {error}");
            exit(124);
        }
    }
}
