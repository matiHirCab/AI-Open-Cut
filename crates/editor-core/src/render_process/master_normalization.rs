//! Bounded request-owned PCM passes and independent delivered-output verification.
use super::*;
use crate::MasterNormalization;
use crate::render_plan::{
    audio_analysis::{AudioAnalysisPlan, PcmAccumulator, invalid_output},
    master_normalization::{
        EvaluatedMasterNormalization, PreparedMasterNormalization, processing_filter,
    },
};
use std::{fs::File, io::BufWriter, path::PathBuf};

pub(crate) fn readiness(ffmpeg: &Path) -> Result<(), CoreError> {
    super::audio_analysis::readiness(ffmpeg)?;
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-filters"])
        .output()
        .map_err(|_| {
            CoreError::new(
                ErrorCode::DependencyUnavailable,
                "master normalization backend unavailable",
            )
        })?;
    let filters = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || ["ebur128", "ametadata"]
            .iter()
            .any(|filter| !filters.contains(&format!(" {filter} ")))
    {
        return Err(CoreError::new(
            ErrorCode::DependencyUnavailable,
            "master normalization filters unavailable",
        ));
    }
    Ok(())
}

fn pcm_command(ffmpeg: &Path, path: &Path, filter: &str) -> Command {
    let mut command = Command::new(ffmpeg);
    command
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
        .arg(path)
        .args(["-af", filter]);
    command
}

fn measure(ffmpeg: &Path, pcm: &Path, filter: &str) -> Result<(Vec<u8>, Vec<u8>), CoreError> {
    let mut command = pcm_command(ffmpeg, pcm, filter);
    command.args(["-f", "null", "-"]);
    let mut child = super::audio_analysis::spawn(&mut command)?;
    let stdout = child.0.stdout.take().ok_or_else(invalid_output)?;
    let stderr = child.0.stderr.take().ok_or_else(invalid_output)?;
    let stdout_reader = thread::spawn(move || read_bounded_tail(stdout, STDERR_TAIL_BYTES));
    let stderr_reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
    let status = child.0.wait();
    let stdout = stdout_reader
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
    Ok((stdout, stderr))
}

fn process_pcm(
    ffmpeg: &Path,
    original: &Path,
    output: &Path,
    filter: &str,
    plan: &AudioAnalysisPlan,
    on_progress: &mut dyn FnMut(RenderProgress),
) -> Result<(), CoreError> {
    let mut command = pcm_command(ffmpeg, original, filter);
    command.args([
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
    let mut child = super::audio_analysis::spawn(&mut command)?;
    let stdout = child.0.stdout.take().ok_or_else(invalid_output)?;
    let stderr = child.0.stderr.take().ok_or_else(invalid_output)?;
    let stderr_reader = thread::spawn(move || read_bounded_tail(stderr, STDERR_TAIL_BYTES));
    let streamed = (|| {
        let mut output = BufWriter::with_capacity(
            64 * 1024,
            File::create(output).map_err(|_| invalid_output())?,
        );
        let mut accumulator = PcmAccumulator::new(plan.expected_frames, 1)?;
        super::audio_analysis::stream_pcm(
            stdout,
            &mut output,
            &mut accumulator,
            plan.expected_frames,
            on_progress,
        )
    })();
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
    Ok(())
}

fn integrated_metadata(stdout: &[u8]) -> Result<f64, CoreError> {
    let text = std::str::from_utf8(stdout).map_err(|_| invalid_output())?;
    let value = text
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("lavfi.r128.I="))
        .ok_or_else(invalid_output)?
        .parse::<f64>()
        .map_err(|_| invalid_output())?;
    if !value.is_finite() || !(-1000.0..=1000.0).contains(&value) {
        return Err(invalid_output());
    }
    Ok(value)
}

fn verify(ffmpeg: &Path, pcm: &Path) -> Result<(f64, f64), CoreError> {
    let (stdout, _) = measure(
        ffmpeg,
        pcm,
        "ebur128=peak=true:metadata=1,ametadata=mode=print:key=lavfi.r128.I:file=-",
    )?;
    let integrated = integrated_metadata(&stdout)?;
    let (_, stderr) = measure(ffmpeg, pcm, "loudnorm=I=-24:TP=-2:LRA=7:print_format=json")?;
    let peak = super::audio_analysis::parse_metrics(&stderr)?
        .true_peak_dbtp
        .ok_or_else(invalid_output)?;
    Ok((integrated, peak))
}

fn target_report(stderr: &[u8], pregain_db: f64) -> Result<PreparedMasterNormalization, CoreError> {
    let metrics = super::audio_analysis::parse_metrics(stderr)?;
    let text = std::str::from_utf8(stderr).map_err(|_| invalid_output())?;
    let start = text.rfind('{').ok_or_else(invalid_output)?;
    let report = serde_json::Deserializer::from_str(&text[start..])
        .into_iter::<serde_json::Value>()
        .next()
        .ok_or_else(invalid_output)?
        .map_err(|_| invalid_output())?;
    let offset_lu = report
        .get("target_offset")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(invalid_output)?
        .parse()
        .map_err(|_| invalid_output())?;
    Ok(PreparedMasterNormalization::Measured {
        pregain_db,
        integrated_lufs: metrics.integrated_lufs.ok_or_else(invalid_output)?,
        true_peak_dbtp: metrics.true_peak_dbtp.ok_or_else(invalid_output)?,
        loudness_range_lu: metrics.loudness_range_lu,
        threshold_lufs: metrics.threshold_lufs,
        offset_lu,
        postgain_db: 0.0,
    })
}

pub(crate) fn prepare(
    ffmpeg: &Path,
    plan: &AudioAnalysisPlan,
    settings: &MasterNormalization,
    filter_path: &Path,
    workspace: &Path,
    on_progress: &mut dyn FnMut(RenderProgress),
) -> Result<PreparedMasterNormalization, CoreError> {
    settings.validate()?;
    let original = workspace.join("master-normalization-original.pcm");
    let processed = workspace.join("master-normalization-processed.pcm");
    let corrected = workspace.join("master-normalization-corrected.pcm");
    let document =
        super::audio_analysis::execute(ffmpeg, plan, filter_path, &original, on_progress)?;
    if document.summary.linear_sample_peak == 0.0 {
        return Ok(PreparedMasterNormalization::Identity);
    }
    let mut normalization = EvaluatedMasterNormalization {
        settings: settings.clone(),
        prepared: None,
    };
    if document.summary.integrated_lufs.is_none() {
        let peak = document.summary.true_peak_dbtp.ok_or_else(invalid_output)?;
        let attenuation = (settings.target_true_peak_dbtp - 0.02 - peak).min(0.0);
        normalization.prepared = Some(PreparedMasterNormalization::LimitingOnly {
            gain: 10_f64.powf(attenuation / 20.0),
        });
        let filter = processing_filter(&normalization)?;
        process_pcm(ffmpeg, &original, &processed, &filter, plan, on_progress)?;
        let (_, stderr) = measure(
            ffmpeg,
            &processed,
            "loudnorm=I=-24:TP=-2:LRA=7:print_format=json",
        )?;
        if super::audio_analysis::parse_metrics(&stderr)?
            .true_peak_dbtp
            .ok_or_else(invalid_output)?
            > settings.target_true_peak_dbtp + 0.01
        {
            return Err(invalid_output());
        }
        return normalization.prepared.ok_or_else(invalid_output);
    }
    let pregain_db = (-12.0
        - document
            .summary
            .sample_peak_dbfs
            .ok_or_else(invalid_output)?)
    .min(0.0);
    let target = format!(
        "volume={pregain_db}dB:precision=double,loudnorm=I={}:TP={}:LRA={}:print_format=json",
        settings.target_integrated_lufs,
        settings.target_true_peak_dbtp,
        settings.target_loudness_range_lu
    );
    let (_, stderr) = measure(ffmpeg, &original, &target)?;
    normalization.prepared = Some(target_report(&stderr, pregain_db)?);
    process_pcm(
        ffmpeg,
        &original,
        &processed,
        &processing_filter(&normalization)?,
        plan,
        on_progress,
    )?;
    let (integrated, peak) = verify(ffmpeg, &processed)?;
    let correction = settings.target_integrated_lufs - integrated;
    let mut final_path: PathBuf = processed;
    if correction.abs() > 0.1 {
        if !correction.is_finite()
            || correction.abs() > 99.0
            || peak + correction > settings.target_true_peak_dbtp - 0.02
        {
            return Err(invalid_output());
        }
        let Some(PreparedMasterNormalization::Measured { postgain_db, .. }) =
            &mut normalization.prepared
        else {
            return Err(invalid_output());
        };
        *postgain_db = correction;
        // Re-run from the original with precisely the final shared lowering;
        // no extra f32 quantization boundary can diverge from final rendering.
        process_pcm(
            ffmpeg,
            &original,
            &corrected,
            &processing_filter(&normalization)?,
            plan,
            on_progress,
        )?;
        final_path = corrected;
    }
    let (integrated, peak) = if final_path.ends_with("master-normalization-corrected.pcm") {
        verify(ffmpeg, &final_path)?
    } else {
        (integrated, peak)
    };
    if (integrated - settings.target_integrated_lufs).abs() > 0.1
        || peak > settings.target_true_peak_dbtp + 0.01
    {
        return Err(invalid_output());
    }
    normalization.prepared.ok_or_else(invalid_output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integrated_metadata_uses_last_precise_finite_value_and_rejects_poison() {
        assert_eq!(
            integrated_metadata(b"lavfi.r128.I=-70.000\nframe:1\nlavfi.r128.I=-15.973\n").unwrap(),
            -15.973
        );
        for bytes in [
            b"".as_slice(),
            b"I: -16.0",
            b"lavfi.r128.I=nan",
            b"lavfi.r128.I=-16\nlavfi.r128.I=inf",
        ] {
            assert!(integrated_metadata(bytes).is_err());
        }
    }
}
