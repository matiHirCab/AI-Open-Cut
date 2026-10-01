use std::{io::{Read,Write},process::Command};
fn main() {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if !args.iter().any(|a|a=="ffv1") {
        let real = std::env::var_os("OPENCUT_TEST_REAL_FFMPEG_PATH").or_else(||std::env::var_os("OPENCUT_FFMPEG_PATH")).expect("real FFmpeg");
        let status=Command::new(real).args(args).status().unwrap();
        std::process::exit(status.code().unwrap_or(1));
    }
    let mut stdin=std::io::stdin();let mut buffer=[0u8;4096];while stdin.read(&mut buffer).unwrap()!=0 {}
    std::fs::write(args.last().unwrap(),b"partial preparation").unwrap();
    let mut stderr=std::io::stderr();
    stderr.write_all("x".repeat(20000).as_bytes()).unwrap();
    stderr.write_all("é".repeat(10000).as_bytes()).unwrap();
    writeln!(stderr,"\nCannot open 'C:\\private-review\\secret.mkv' or '/private-review/secret.mkv': injected encoder failure").unwrap();
    std::process::exit(7);
}
