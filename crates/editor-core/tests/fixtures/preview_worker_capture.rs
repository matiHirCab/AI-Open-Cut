//! Disposable test transport wrapper; captures explicit test-feature stderr only.
use std::{env, fs::OpenOptions, process::{Command, Stdio}};
fn main() {
    let log = OpenOptions::new().create(true).append(true)
        .open(env::var_os("OPENCUT_PREVIEW_TEST_TRACE").expect("trace path")).expect("open trace");
    let status = Command::new(env::var_os("OPENCUT_PREVIEW_TEST_HEADLESS").expect("real headless"))
        .args(env::args_os().skip(1)).stdin(Stdio::inherit()).stdout(Stdio::inherit())
        .stderr(Stdio::from(log)).status().expect("start real headless");
    std::process::exit(status.code().unwrap_or(1));
}
