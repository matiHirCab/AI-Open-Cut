//! Bounded original-PCM streaming and discarded-output input loudness measurement.
use super::*;
use crate::render_plan::audio_analysis::{
    AudioAnalysisDocument, AudioAnalysisPlan, LoudnessMetrics, PcmAccumulator, invalid_output,
};
use std::{fs::File, io::BufWriter, process::Child};

const FILTERS: &[&str] = &[
    "aformat",
    "aresample",
    "atrim",
    "asetpts",
    "amix",
    "loudnorm",
    "volume",
    "afade",
    "atempo",
    "adelay",
    "anullsrc",
    "color",
    "nullsink",
];
pub(crate) fn readiness(ffmpeg: &Path) -> Result<(), CoreError> {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|_| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                "audio analysis backend unavailable",
            )
        })?;
    let filters = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || FILTERS
            .iter()
            .any(|filter| !filters.contains(&format!(" {filter} ")))
    {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "audio analysis filters unavailable",
        ));
    }
    Ok(())
}

// Children inherit the headless request's process group so the existing bridge
// descendant/group cancellation still owns both passes. Always reap on local failure.
pub(super) struct OwnedChild(pub(super) Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub(super) fn spawn(command: &mut Command) -> Result<OwnedChild, CoreError> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
        .spawn()
        .map(OwnedChild)
        .map_err(|_| CoreError::render_failure(SPAWN_STAGE, None, None))
}

pub(super) fn stream_pcm(
    reader: impl Read,
    writer: &mut impl Write,
    accumulator: &mut PcmAccumulator,
    expected_frames: u64,
    on_progress: &mut dyn FnMut(RenderProgress),
) -> Result<(), CoreError> {
    let mut reader = BufReader::with_capacity(64 * 1024, reader);
    let mut frame = [0_u8; 8];
    let mut next_progress = 0;
    loop {
        let count = reader.read(&mut frame[..1]).map_err(|_| invalid_output())?;
        if count == 0 {
            break;
        }
        reader
            .read_exact(&mut frame[1..])
            .map_err(|_| invalid_output())?;
        accumulator.push([
            f32::from_le_bytes(frame[..4].try_into().map_err(|_| invalid_output())?),
            f32::from_le_bytes(frame[4..].try_into().map_err(|_| invalid_output())?),
        ])?;
        writer.write_all(&frame).map_err(|_| invalid_output())?;
        let frames = accumulator.frames();
        if frames >= next_progress {
            on_progress(RenderProgress {
                progress: 0.8 * frames as f64 / expected_frames as f64,
            });
            next_progress = frames.saturating_add(4800);
        }
    }
    writer.flush().map_err(|_| invalid_output())?;
    if accumulator.frames() != expected_frames {
        return Err(invalid_output());
    }
    Ok(())
}

pub(crate) fn parse_metrics(stderr: &[u8]) -> Result<LoudnessMetrics, CoreError> {
    let text = std::str::from_utf8(stderr).map_err(|_| invalid_output())?;
    let start = text.rfind('{').ok_or_else(invalid_output)?;
    // FFmpeg may append muxing/progress diagnostics after the measurement JSON.
    // Deserialize exactly one complete object from the already bounded stderr
    // tail, retaining rejection of incomplete or poisoned metric values.
    let report = serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<serde_json::Value>()
        .next()
        .ok_or_else(invalid_output)?
        .map_err(|_| invalid_output())?;
    let metric = |name: &str, nullable: bool| -> Result<Option<f64>, CoreError> {
        let raw = report
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(invalid_output)?;
        if nullable && raw == "-inf" {
            return Ok(None);
        }
        raw.parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(Some)
            .ok_or_else(invalid_output)
    };
    let loudness_range_lu = metric("input_lra", false)?.ok_or_else(invalid_output)?;
    if loudness_range_lu < 0.0 {
        return Err(invalid_output());
    }
    Ok(LoudnessMetrics {
        integrated_lufs: metric("input_i", true)?,
        true_peak_dbtp: metric("input_tp", true)?,
        loudness_range_lu,
        threshold_lufs: metric("input_thresh", false)?.ok_or_else(invalid_output)?,
    })
}

pub(crate) fn execute(
    ffmpeg: &Path,
    plan: &AudioAnalysisPlan,
    filter_path: &Path,
    pcm_path: &Path,
    on_progress: &mut dyn FnMut(RenderProgress),
) -> Result<AudioAnalysisDocument, CoreError> {
    let mut accumulator = PcmAccumulator::new(plan.expected_frames, plan.options.waveform_bins)?;
    let mut pcm = BufWriter::with_capacity(
        64 * 1024,
        File::create(pcm_path).map_err(|_| invalid_output())?,
    );
    let mut command = Command::new(ffmpeg);
    append_render_inputs_mode(&mut command, &plan.render, false);
    command.arg("-nostdin");
    if plan.render.serial_bezier_filters {
        command.args(["-filter_complex_threads", "1"]);
    }
    command
        .arg("-filter_complex_script")
        .arg(filter_path)
        .args([
            "-map",
            "[analysis]",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-c:a",
            "pcm_f32le",
            "-f",
            "f32le",
            "pipe:1",
        ]);
    let mut child = spawn(&mut command)?;
    let stdout = child.0.stdout.take().ok_or_else(invalid_output)?;
    let stderr = child.0.stderr.take().ok_or_else(invalid_output)?;
    let stderr_reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
    let streamed = stream_pcm(
        stdout,
        &mut pcm,
        &mut accumulator,
        plan.expected_frames,
        on_progress,
    );
    if streamed.is_err() {
        let _ = child.0.kill();
    }
    let status = child.0.wait();
    let stderr = stderr_reader
        .join()
        .map_err(|_| invalid_output())?
        .map_err(|_| invalid_output())?;
    let status = status
        .map_err(|_| CoreError::render_failure(RENDER_STAGE, None, stderr_excerpt(&stderr)))?;
    if streamed.is_err() || !status.success() {
        return Err(CoreError::render_failure(
            RENDER_STAGE,
            status.code(),
            stderr_excerpt(&stderr),
        ));
    }
    drop(pcm);
    drop(child);
    on_progress(RenderProgress { progress: 0.85 });

    let mut measurement = Command::new(ffmpeg);
    measurement
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "info",
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-i",
        ])
        .arg(pcm_path)
        .args([
            "-af",
            "loudnorm=I=-24:TP=-2:LRA=7:print_format=json",
            "-f",
            "null",
            "-",
        ]);
    let mut child = spawn(&mut measurement)?;
    // No measurement stdout bytes are retained; a bounded reader also drains a
    // misbehaving configured executable without a pipe deadlock or heap growth.
    let stdout = child.0.stdout.take().ok_or_else(invalid_output)?;
    let stderr = child.0.stderr.take().ok_or_else(invalid_output)?;
    let stdout_reader = thread::spawn(move || read_bounded_tail(stdout, 1));
    let stderr_reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
    let status = child.0.wait();
    stdout_reader
        .join()
        .map_err(|_| invalid_output())?
        .map_err(|_| invalid_output())?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| invalid_output())?
        .map_err(|_| invalid_output())?;
    let status = status
        .map_err(|_| CoreError::render_failure(RENDER_STAGE, None, stderr_excerpt(&stderr)))?;
    if !status.success() {
        return Err(CoreError::render_failure(
            RENDER_STAGE,
            status.code(),
            stderr_excerpt(&stderr),
        ));
    }
    let metrics = parse_metrics(&stderr).map_err(|mut error| {
        error.ffmpeg_stderr_excerpt = stderr_excerpt(&stderr);
        error
    })?;
    let document = accumulator.finish(plan.options, metrics)?;
    document.validate(plan.options).map_err(|mut error| {
        error.ffmpeg_stderr_excerpt = stderr_excerpt(&stderr);
        error
    })?;
    on_progress(RenderProgress { progress: 0.95 });
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn metric_nulls_preserve_short_audible_peak_and_ignore_normalized_output() {
        let input=br#"{"input_i":"-inf","input_tp":"-21.07","input_lra":"0.00","input_thresh":"-70.00","output_i":"-inf","target_offset":"inf"}"#;
        let metrics = parse_metrics(input).unwrap();
        assert!(metrics.integrated_lufs.is_none());
        assert_eq!(metrics.true_peak_dbtp, Some(-21.07));
        let mut with_logs = input.to_vec();
        with_logs.extend_from_slice(b"\n[out#0/null] audio:750KiB muxing overhead: unknown\nsize=N/A time=00:00:01.00 speed=43.5x\n");
        assert_eq!(
            parse_metrics(&with_logs).unwrap().true_peak_dbtp,
            Some(-21.07)
        );
        for input in [
            br#"{}"#.as_slice(),
            br#"{"input_i":"-20","input_tp":"0","input_lra":"0","input_thresh":"-70""#,
            br#"{"input_i":"nan","input_tp":"0","input_lra":"0","input_thresh":"-70"}"#,
            br#"{"input_i":"-20","input_tp":"inf","input_lra":"0","input_thresh":"-70"}"#,
            br#"{"input_i":"-20","input_tp":"0","input_lra":"-1","input_thresh":"-70"}"#,
        ] {
            assert_eq!(
                parse_metrics(input).unwrap_err().code,
                ErrorCode::FfmpegFailed
            );
        }
    }
    #[test]
    fn stream_checks_exact_stereo_frames_finite_values_and_overflow_before_write() {
        for samples in [
            vec![],
            vec![0; 7],
            vec![0; 16],
            vec![0, 0, 192, 127, 0, 0, 0, 0],
        ] {
            let mut accumulator = PcmAccumulator::new(1, 1).unwrap();
            let mut output = Vec::new();
            assert!(
                stream_pcm(
                    Cursor::new(samples),
                    &mut output,
                    &mut accumulator,
                    1,
                    &mut |_| {}
                )
                .is_err()
            );
            assert!(output.len() <= 8);
        }
        let mut accumulator = PcmAccumulator::new(1, 1).unwrap();
        let mut output = Vec::new();
        let mut progress = Vec::new();
        stream_pcm(
            Cursor::new([0_u8; 8]),
            &mut output,
            &mut accumulator,
            1,
            &mut |p| progress.push(p.progress),
        )
        .unwrap();
        assert_eq!(output, [0_u8; 8]);
        assert!(
            progress
                .iter()
                .all(|p| p.is_finite() && (0.0..=1.0).contains(p))
        );
    }

    #[cfg(unix)]
    #[test]
    fn owned_process_passes_reap_failed_exits_partial_overflow_poison_and_bad_measurements() {
        use crate::render_plan::RenderIntent;
        use std::os::unix::fs::PermissionsExt;
        for mode in [
            "short",
            "partial",
            "extra",
            "nonfinite",
            "pcm-exit",
            "measure-exit",
            "measure-malformed",
            "success",
        ] {
            let root = tempfile::tempdir().unwrap();
            let tool = root.path().join("analysis-fixture.sh");
            let first = match mode {
                "short" => "head -c 376 /dev/zero",
                "partial" => "head -c 383 /dev/zero",
                "extra" => "head -c 392 /dev/zero",
                "nonfinite" => "printf '\\000\\000\\300\\177\\000\\000\\000\\000'",
                "pcm-exit" => {
                    "head -c 384 /dev/zero; printf '%s\\n' '/private/audio.pcm failed' >&2; exit 7"
                }
                _ => "head -c 384 /dev/zero",
            };
            let measure = match mode {
                "measure-exit" => "printf '%s\\n' '/private/measurement.pcm failed' >&2; exit 7",
                "measure-malformed" => "printf '%s\\n' '{\"input_i\":' >&2",
                _ => {
                    "printf '%s\\n' '{\"input_i\":\"-inf\",\"input_tp\":\"-inf\",\"input_lra\":\"0\",\"input_thresh\":\"-70\",\"target_offset\":\"inf\"}' '[out#0/null] discarded' >&2"
                }
            };
            let script = format!(
                "#!/bin/sh\ncase \"$*\" in\n*loudnorm*) echo $$ > '{}'; {measure};;\n*) echo $$ > '{}'; {first};;\nesac\n",
                root.path().join("measurement.pid").display(),
                root.path().join("pcm.pid").display()
            );
            std::fs::write(&tool, script).unwrap();
            std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o700)).unwrap();
            let options = crate::AudioAnalysisOptions {
                start_ms: 0,
                end_ms: 1,
                waveform_bins: 2,
            };
            let plan = AudioAnalysisPlan {
                options,
                expected_frames: 48,
                render: RenderPlan {
                    text_layout_fidelity: false,
                    serial_bezier_filters: false,
                    detail_fidelity: false,
                    filter_graph: String::new(),
                    width: 2,
                    height: 2,
                    fps: 30,
                    duration_ms: 1,
                    intent: RenderIntent::Export,
                    media_inputs: vec![],
                    media_paths: vec![],
                },
            };
            let result = execute(
                &tool,
                &plan,
                &root.path().join("filter.txt"),
                &root.path().join("private-original.pcm"),
                &mut |_| {},
            );
            if mode == "success" {
                assert_eq!(result.unwrap().summary.frame_count, 48);
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, ErrorCode::FfmpegFailed, "{mode}");
                if mode.ends_with("exit") {
                    assert_eq!(error.ffmpeg_exit_code, Some(7));
                    assert!(!error.ffmpeg_stderr_excerpt.unwrap().contains("/private/"));
                }
            }
            for phase in ["pcm", "measurement"] {
                if let Ok(pid) = std::fs::read_to_string(root.path().join(format!("{phase}.pid"))) {
                    assert!(
                        !Command::new("kill")
                            .args(["-0", pid.trim()])
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .unwrap()
                            .success(),
                        "{mode} must reap {phase}"
                    );
                }
            }
            let measured = root.path().join("measurement.pid").exists();
            assert_eq!(
                measured,
                matches!(mode, "measure-exit" | "measure-malformed" | "success"),
                "{mode} must not measure failed original PCM"
            );
        }
    }
}
