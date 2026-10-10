// Private process-lifetime fixture. Stub metrics/PCM are never native media proof.
use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let (_, tail) = json.split_once(&format!("\"{key}\""))?;
    let tail = tail
        .trim_start()
        .strip_prefix(':')?
        .trim_start()
        .strip_prefix('"')?;
    tail.split_once('"').map(|(value, _)| value)
}

fn phase(input: &str, filter: &str, scripted: bool, piped: bool) -> Option<&'static str> {
    let name = Path::new(input).file_name()?.to_str()?;
    if scripted && name == "master-normalization-capture-filter.txt" {
        Some("capture")
    } else if name == "master-normalization-original.pcm" && piped {
        Some(if filter.contains("volume=0.3") {
            "correction"
        } else {
            "processing"
        })
    } else if name == "master-normalization-original.pcm"
        && filter == "loudnorm=I=-24:TP=-2:LRA=7:print_format=json"
    {
        Some("original_measurement")
    } else if name == "master-normalization-original.pcm" && filter.contains("loudnorm=") {
        Some("target_measurement")
    } else if name.starts_with("master-normalization-") && filter.contains("ebur128=") {
        Some(if name.contains("corrected") {
            "final_ebu_verification"
        } else {
            "ebu_verification"
        })
    } else if name.starts_with("master-normalization-") && filter.contains("loudnorm=") {
        Some(if name.contains("corrected") {
            "final_truepeak_verification"
        } else {
            "truepeak_verification"
        })
    } else {
        None
    }
}

fn number(text: &str, key: &str) -> io::Result<u64> {
    let tail = text.split_once(key).ok_or(io::ErrorKind::InvalidInput)?.1;
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    digits
        .parse()
        .map_err(|_| io::ErrorKind::InvalidInput.into())
}

fn hang() -> ! {
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args == ["--fixture-descendant"] {
        hang();
    }
    let executable = env::current_exe()?;
    let probe = executable
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "ffprobe");
    if args.iter().any(|arg| arg == "-version") {
        // Deliberately not the production "%s version " identity marker.
        println!("{} fake 1.0", if probe { "ffprobe" } else { "ffmpeg" });
        return Ok(());
    }
    if args.iter().any(|arg| arg == "-filters") {
        println!(" ... overlay ... drawtext ... amix ... remap ... blend ... nullsrc ... split ... geq ... pad ... crop ... format ... aformat ... aresample ... atrim ... asetpts ... loudnorm ... volume ... afade ... atempo ... adelay ... anullsrc ... color ... nullsink ... ebur128 ... ametadata ... ");
        return Ok(());
    }
    if probe {
        if value(&args, "-i").is_none() && args.is_empty() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        println!("{{\"format\":{{\"duration\":\"0.100\",\"format_name\":\"fixture\"}},\"streams\":[{{\"codec_name\":\"rawvideo\",\"codec_type\":\"video\",\"height\":1,\"width\":1}},{{\"channels\":1,\"codec_name\":\"pcm_s16le\",\"codec_type\":\"audio\",\"sample_rate\":\"24000\"}}]}}");
        return Ok(());
    }
    let scripted = value(&args, "-filter_complex_script");
    let input = scripted
        .or_else(|| value(&args, "-i"))
        .ok_or(io::ErrorKind::InvalidInput)?;
    let root = Path::new(input)
        .parent()
        .and_then(Path::parent)
        .ok_or(io::ErrorKind::InvalidInput)?;
    let filter = value(&args, "-af").unwrap_or("");
    let piped = args.last().is_some_and(|arg| arg == "pipe:1");
    let selected = phase(input, filter, scripted.is_some(), piped).unwrap_or("");
    let fault = fs::read_to_string(root.join(".normalization-test-fault")).unwrap_or_default();
    let fault_kind = if field(&fault, "phase") == Some(selected) {
        field(&fault, "kind")
    } else {
        None
    };
    if !selected.is_empty() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis();
        writeln!(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(root.join(".normalization-phase-events.jsonl"))?,
            "{{\"phase\":\"{selected}\",\"pid\":{},\"enteredAtMs\":{now}}}",
            std::process::id()
        )?;
        if fs::read_to_string(root.join(".normalization-test-phase"))
            .ok()
            .as_deref()
            == Some(selected)
        {
            let child = Command::new(&executable)
                .arg("--fixture-descendant")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            fs::write(
                root.join(format!(".normalization-test-{selected}.pid")),
                format!(
                    "{{\"backend\":{},\"descendant\":{}}}",
                    std::process::id(),
                    child.id()
                ),
            )?;
            hang();
        }
    }
    if fault_kind == Some("exit") {
        std::process::exit(23);
    }
    if piped && args.iter().any(|arg| arg == "f32le") {
        let frames = if let Some(script) = scripted {
            let script = fs::read_to_string(script)?;
            number(&script, "end_sample=")?
                .checked_sub(number(&script, "atrim=start_sample=")?)
                .ok_or(io::ErrorKind::InvalidInput)?
        } else {
            fs::metadata(input)?.len() / 8
        };
        if frames == 0 || frames > 28_800_000 {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let sample: f32 = if fault_kind == Some("nonfinite") {
            f32::NAN
        } else if selected == "capture" {
            0.125
        } else {
            0.1
        };
        let buffer = sample.to_le_bytes().repeat(2048);
        let mut remaining = frames * 8;
        remaining = match fault_kind {
            Some("partial") => remaining - 1,
            Some("short") => remaining - 8,
            Some("extra") => remaining + 8,
            _ => remaining,
        };
        let mut stdout = io::stdout().lock();
        while remaining > 0 {
            let count = remaining.min(buffer.len() as u64) as usize;
            stdout.write_all(&buffer[..count])?;
            remaining -= count as u64;
        }
        stdout.flush()?;
    } else if filter.contains("ebur128=") {
        let integrated = if fault_kind == Some("poison") {
            "nan"
        } else if Path::new(input)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains("corrected"))
        {
            "-16.000"
        } else {
            "-16.300"
        };
        println!("frame:1\nlavfi.r128.I={integrated}");
    } else if filter.contains("loudnorm=") {
        let peak = if fault_kind == Some("infeasible") {
            "0.00"
        } else {
            "-20.00"
        };
        let offset = if fault_kind == Some("poison") {
            "nan"
        } else {
            "0.00"
        };
        eprintln!("{{\"input_i\":\"-18.00\",\"input_lra\":\"0.00\",\"input_thresh\":\"-28.00\",\"input_tp\":\"{peak}\",\"target_offset\":\"{offset}\"}}");
    } else {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    Ok(())
}

fn main() {
    if run().is_err() {
        eprintln!("private normalization fixture refused request");
        std::process::exit(2);
    }
}
