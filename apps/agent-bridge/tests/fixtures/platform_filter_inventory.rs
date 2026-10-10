//! Owned native filter inventory fixture. Only -filters is simulated; all other
//! invocations use the explicitly configured real FFmpeg, without shell parsing.
use std::{
    env,
    process::{Command, exit},
};
fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.iter().any(|arg| arg == "-filters") {
        let mode = env::var("OPENCUT_PLATFORM_FILTER_MODE").expect("owned fixture mode");
        let filters = if mode == "base-missing" {
            "overlay amix adelay geq remap blend nullsrc split pad crop format"
        } else if mode == "optional-missing" {
            "overlay drawtext amix adelay geq remap blend nullsrc split pad crop format"
        } else if mode == "delay-missing" {
            "overlay drawtext amix geq remap blend nullsrc split pad crop format"
        } else {
            panic!("unexpected owned fixture mode");
        };
        for filter in filters.split_whitespace() {
            println!(" ... {filter} fixture");
        }
        return;
    }
    let real = env::var_os("OPENCUT_PLATFORM_REAL_FFMPEG").expect("real FFmpeg");
    let status = Command::new(real)
        .args(args)
        .status()
        .expect("execute real FFmpeg");
    exit(status.code().unwrap_or(1));
}
