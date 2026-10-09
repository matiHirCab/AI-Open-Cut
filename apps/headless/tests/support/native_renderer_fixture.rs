//! Standalone test-owned descendant; compiled by the integration fixture, never shipped.
use std::{fs, path::PathBuf, time::Duration};

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let destination = PathBuf::from(arguments.next().expect("owned PID destination required"));
    assert!(arguments.next().is_none(), "unexpected fixture argument");
    let pending = destination.with_extension("pending");
    fs::write(&pending, format!("{}\n", std::process::id())).expect("write owned PID");
    fs::rename(pending, destination).expect("publish complete owned PID");
    std::thread::sleep(Duration::from_secs(120));
}
