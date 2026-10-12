//! Owned readiness-only control: no media decoding, rendering or artifact output.
use std::ffi::OsStr;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == [OsStr::new("-hide_banner"), OsStr::new("-filters")] {
        println!(" overlay drawtext amix adelay geq remap blend nullsrc split pad crop format ");
        return;
    }
    if args == [OsStr::new("-version")] {
        println!("owned budget readiness control");
        return;
    }
    let marker = std::env::var_os("OPENCUT_TEST_BUDGET_EXECUTION_MARKER")
        .expect("owned execution marker must be configured");
    std::fs::write(marker, b"unexpected execution").expect("write owned marker");
    std::process::exit(91);
}
