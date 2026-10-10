//! Real owned native descendant for release-runner cleanup controls.
use std::{
    env, fs,
    process::{exit, Command},
    thread,
    time::Duration,
};
fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args[0] == "hold" {
        #[cfg(unix)]
        {
            unsafe extern "C" {
                fn setsid() -> i32;
            }
            assert!(
                unsafe { setsid() } > 0,
                "actual detached descendant session"
            );
        }
        thread::sleep(Duration::from_secs(120));
        return;
    }
    let child = Command::new(env::current_exe().unwrap())
        .arg("hold")
        .spawn()
        .unwrap();
    fs::write(&args[1], child.id().to_string()).unwrap();
    thread::sleep(Duration::from_millis(250));
    if args[0] == "failure" {
        exit(7);
    }
    if args[0] == "success" {
        exit(0);
    }
    thread::sleep(Duration::from_secs(120));
}
