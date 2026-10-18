// A deterministic decoder stand-in. Mode arrives through the ordinary -i
// argument so tests never mutate process-global environment variables.
use std::io::Write;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let input = args.windows(2).find(|pair| pair[0] == "-i").unwrap();
    let source = std::path::Path::new(&input[1]);
    let mode = source.file_name().unwrap().to_str().unwrap();
    let (bytes, status): (&[u8], i32) = match mode {
        "empty-ok" | "linger-empty" => (&[], 0),
        "empty-failed" => (&[], 7),
        "partial-ok" | "linger-partial" => (&[1, 2, 3], 0),
        "partial-failed" => (&[1, 2, 3], 7),
        "trailing-partial" => (&[1, 2, 3, 4, 5], 7),
        "complete-failed" => (&[1, 2, 3, 4], 7),
        "complete-ok" => (&[1, 2, 3, 4, 5, 6, 7, 8], 0),
        _ => panic!("unknown test mode"),
    };
    let mut stderr = std::io::stderr().lock();
    stderr.write_all("é".repeat(20_000).as_bytes()).unwrap();
    writeln!(stderr, "\nCannot open 'C:\\private-decode\\secret.mp4' or '/private-decode/secret.mp4': injected decode diagnostic").unwrap();
    std::io::stdout().write_all(bytes).unwrap();
    std::io::stdout().flush().unwrap();
    if mode.starts_with("linger-") {
        close_stdout();
        std::thread::sleep(std::time::Duration::from_secs(2));
        std::fs::write(
            source.with_extension("survived"),
            b"producer was not terminated",
        )
        .unwrap();
    }
    std::process::exit(status);
}

#[cfg(unix)]
fn close_stdout() {
    use std::os::fd::FromRawFd;
    // This fixture owns its stdout pipe and makes no further stdout writes.
    unsafe { drop(std::fs::File::from_raw_fd(1)) };
}

#[cfg(windows)]
fn close_stdout() {
    use std::os::windows::io::FromRawHandle;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(id: u32) -> *mut std::ffi::c_void;
    }
    // STD_OUTPUT_HANDLE is the owned pipe inherited by this fixture process.
    let handle = unsafe { GetStdHandle(-11_i32 as u32) };
    assert!(!handle.is_null() && handle as isize != -1);
    unsafe { drop(std::fs::File::from_raw_handle(handle)) };
}
