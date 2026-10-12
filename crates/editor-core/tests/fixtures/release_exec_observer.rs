// Owned collector control only; this never supplies native rendering evidence.
#[cfg(unix)]
fn main() {
    use std::{fs, os::unix::process::CommandExt, path::PathBuf, process::Command};
    let args = std::env::args_os().collect::<Vec<_>>();
    let wait = |path: PathBuf| {
        while !path.exists() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    };
    if args[1] == "launch" {
        let root = PathBuf::from(&args[3]);
        fs::write(root.join("before"), std::process::id().to_string()).unwrap();
        wait(root.join("exec"));
        let error = Command::new(&args[2])
            .arg("-map")
            .arg("[video]")
            .arg(root.join("frame.png"))
            .arg(&root)
            .exec();
        panic!("owned collector exec failed: {error}");
    }
    assert_eq!(args[1], "-map");
    assert_eq!(args[2], "[video]");
    let root = PathBuf::from(&args[4]);
    fs::write(root.join("after"), std::process::id().to_string()).unwrap();
    wait(root.join("release"));
}

#[cfg(not(unix))]
fn main() {
    panic!("same-PID exec collector control requires Unix");
}
