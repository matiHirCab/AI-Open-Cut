//! Literal cue tables and independent sample summation; existing native oracles remain intact.
use super::*;
use serde_json::json;
#[path = "../../../tests/support/narration_fixture.rs"]
mod fixture;
use super::master_normalization::{RecordingProcess, independent_metrics};
use crate::render_artifact::media_input_requests;
use crate::render_plan::{RenderIntent, build_render_plan};

fn reference_mix() -> Vec<f32> {
    let voice = fixture::samples(437.0, 0.1, 6000, true);
    let music = fixture::samples(211.0, 0.08, 6000, false);
    let accent = fixture::samples(877.0, 0.1, 300, false);
    let mut mix: Vec<_> = voice
        .iter()
        .zip(&music)
        .map(|(&v, &m)| v as f32 / 32768.0 + m as f32 / 32768.0 * 0.25)
        .collect();
    let event_gain = 10_f64.powf(-9.0 / 20.0) as f32;
    for start in [500, 1000, 1500, 2400, 3200, 4300] {
        for (index, &sample) in accent.iter().enumerate() {
            mix[start * 48 * 2 + index] += sample as f32 / 32768.0 * event_gain;
        }
    }
    mix
}
fn rms(actual: &[f32], expected: &[f32]) -> f64 {
    assert_eq!(actual.len(), expected.len());
    assert!(!actual.is_empty());
    assert!(actual.iter().chain(expected).all(|v| v.is_finite()));
    (actual
        .iter()
        .zip(expected)
        .map(|(&a, &b)| (a as f64 - b as f64).powi(2))
        .sum::<f64>()
        / actual.len() as f64)
        .sqrt()
}
fn rgb(tools: &NativeTools, path: &Path) -> Vec<u8> {
    let output = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), 64 * 64 * 3);
    output.stdout
}
fn constant_plate(tools: &NativeTools, color: &str, alpha: f64) -> Vec<u8> {
    // Constant codec reference: no production timeline, preset sampler or render graph.
    // Canonical opacity composites in linear light. Apply the IEC sRGB transfer
    // independently to the literal saturated cue colors over an opaque black plate.
    let mut encoded = String::from("#");
    for index in [1, 3, 5] {
        let source = u8::from_str_radix(&color[index..index + 2], 16).unwrap() as f64 / 255.0;
        let linear = if source <= 0.04045 {
            source / 12.92
        } else {
            ((source + 0.055) / 1.055).powf(2.4)
        } * alpha;
        let srgb = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        encoded.push_str(&format!("{:02x}", (srgb * 255.0).round() as u8));
    }
    let filter = format!("color=c={encoded}:s=64x64:r=10:d=0.1,format=rgb24[out]");
    let output = Command::new(&tools.ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-filter_complex_threads",
            "1",
            "-filter_complex",
            &filter,
            "-map",
            "[out]",
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), 64 * 64 * 3);
    output.stdout
}
fn decoded_pcm(tools: &NativeTools, path: &Path) -> Vec<f32> {
    let output = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-vn", "-ar", "48000", "-ac", "2", "-f", "f32le", "pipe:1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len() % 4, 0);
    output
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| f32::from_le_bytes(*v))
        .collect()
}
fn encoded_reference(tools: &NativeTools, root: &Path) -> Vec<f32> {
    // Literal source files and fixed bus equations, independent of scene evaluation,
    // production plan builders and the project's authored operation results.
    for (name, hz, amplitude, duration, gated) in [
        ("reference-voice.wav", 437.0, 0.1, 6000, true),
        ("reference-music.wav", 211.0, 0.08, 6000, false),
        ("reference-accent.wav", 877.0, 0.1, 300, false),
    ] {
        fixture::wav(
            &root.join(name),
            &fixture::samples(hz, amplitude, duration, gated),
        );
    }
    let mut filters = vec![
        "anullsrc=r=48000:cl=stereo:d=6[clock]".to_owned(),
        "[0:a]volume=1,adelay=0:all=1,volume=1,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,amix=inputs=1:normalize=0,volume=1,pan=stereo|c0=c0|c1=c1[voice]".to_owned(),
        "[1:a]volume=1,adelay=0:all=1,volume=1,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,amix=inputs=1:normalize=0,volume=1,volume=0.25,pan=stereo|c0=c0|c1=c1[music]".to_owned(),
        "[2:a]asplit=6[e0][e1][e2][e3][e4][e5]".to_owned(),
    ];
    for (index, start) in [500, 1000, 1500, 2400, 3200, 4300].into_iter().enumerate() {
        filters.push(format!("[e{index}]volume={:.6},adelay={start}:all=1,volume=1,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo[s{index}]", 10_f64.powf(-9.0 / 20.0)));
    }
    filters.push("[s0][s1][s2][s3][s4][s5]amix=inputs=6:duration=longest:normalize=0,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,volume=1,pan=stereo|c0=c0|c1=c1[sfx]".to_owned());
    filters.push("[clock][voice][music][sfx]amix=inputs=4:duration=longest:normalize=0,aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo,volume=1,pan=stereo|c0=c0|c1=c1[out]".to_owned());
    let output = root.join("independent-mix.m4a");
    let mut command = Command::new(&tools.ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin"]);
    for name in [
        "reference-voice.wav",
        "reference-music.wav",
        "reference-accent.wav",
    ] {
        command.arg("-i").arg(root.join(name));
    }
    let result = command
        .args([
            "-filter_complex",
            &filters.join(";"),
            "-map",
            "[out]",
            "-c:a",
            "aac",
            "-t",
            "6",
            "-y",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    decoded_pcm(tools, &output)
}

fn movie_evidence(tools: &NativeTools, path: &Path, start_ms: u64, end_ms: u64) {
    let output = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-i"])
        .arg(path)
        .args(["-an", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frame_bytes = 64 * 64 * 3;
    assert_eq!(output.stdout.len() % frame_bytes, 0);
    let frame_count = output.stdout.len() / frame_bytes;
    assert!((frame_count as i64 * 100 - (end_ms - start_ms) as i64).abs() <= 100);
    let cues = [
        (500, "#ff0000"),
        (1000, "#00ff00"),
        (1500, "#0000ff"),
        (2400, "#ffff00"),
        (3200, "#00ffff"),
        (4300, "#ff00ff"),
    ];
    for (cue, color) in cues {
        for (offset, alpha) in [(0, 0.0), (100, 0.5), (200, 1.0), (400, 0.0)] {
            let at = cue + offset;
            if at < start_ms || at >= end_ms {
                continue;
            }
            let index = ((at - start_ms) / 100) as usize;
            let actual = &output.stdout[index * frame_bytes..(index + 1) * frame_bytes];
            let expected = constant_plate(tools, color, alpha);
            assert!(
                ssim(actual, &expected) >= 0.99,
                "movie {} at {at}: SSIM{}",
                path.display(),
                ssim(actual, &expected)
            );
        }
    }
    let probe = Command::new(&tools.ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(probe.status.success());
    let duration: f64 = String::from_utf8(probe.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!((duration * 1000.0 - (end_ms - start_ms) as f64).abs() <= 100.0);
}

fn ssim(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len() as f64;
    let ma = a.iter().map(|&v| v as f64).sum::<f64>() / n;
    let mb = b.iter().map(|&v| v as f64).sum::<f64>() / n;
    let va = a.iter().map(|&v| (v as f64 - ma).powi(2)).sum::<f64>() / n;
    let vb = b.iter().map(|&v| (v as f64 - mb).powi(2)).sum::<f64>() / n;
    let covariance = a
        .iter()
        .zip(b)
        .map(|(&a, &b)| (a as f64 - ma) * (b as f64 - mb))
        .sum::<f64>()
        / n;
    ((2.0 * ma * mb + 6.5025) * (2.0 * covariance + 58.5225))
        / ((ma * ma + mb * mb + 6.5025) * (va + vb + 58.5225))
}

#[test]
fn narration_fixture_exact_shared_semantic_plans_and_negative_timing_control() {
    let root = tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let mut p = f.project();
    // Pure plans reject unprepared active normalization. The exact semantic proof
    // uses its explicit inactive mode; ordinary active native paths are proved below.
    p.master_normalization.as_mut().unwrap().enabled = false;
    let evaluated = evaluate_project(&p, 64, 64, 10).unwrap();
    assert_eq!(evaluated.scene.visual_layers.len(), 6);
    assert_eq!(evaluated.scene.audio_layers.len(), 8);
    let inputs = media_input_requests(&evaluated).unwrap();
    let paths = inputs
        .iter()
        .map(|input| f.dir().join(&input.project_relative_path))
        .collect::<Vec<_>>();
    let mut plans = vec![];
    for intent in [
        RenderIntent::Frame { at_ms: 1500 },
        RenderIntent::Range {
            start_ms: 0,
            end_ms: 6000,
            include_audio: true,
        },
        RenderIntent::Range {
            start_ms: 1000,
            end_ms: 5000,
            include_audio: true,
        },
        RenderIntent::Export,
    ] {
        let plan = build_render_plan(
            &evaluated.scene,
            &Default::default(),
            inputs.clone(),
            paths.clone(),
            None,
            intent,
            &mut vec![],
        )
        .unwrap();
        plans.push(plan.filter_graph);
    }
    assert!(
        plans.windows(2).all(|plans| plans[0] == plans[1]),
        "shared nonempty filters drifted across intents"
    );
    let draft=f.core.create_draft(&f.id,f.project().revision,vec![fixture::op(json!({"operation":"marker_update","scope":"root","markerId":p.markers[0].id,"name":"EVERY","timeMs":600,"kind":"cue"}))],None).unwrap();
    let candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
    assert_eq!(
        candidate
            .tracks
            .iter()
            .flat_map(|t| &t.items)
            .find(|i| i.id() == f.aliases["visual0"])
            .unwrap()
            .start_ms(),
        600
    );
    assert_eq!(
        candidate
            .tracks
            .iter()
            .flat_map(|t| &t.items)
            .find(|i| i.id() == f.aliases["event0"])
            .unwrap()
            .start_ms(),
        600
    );
    assert_eq!(f.project().markers[0].time_ms, 500);
    assert_eq!(
        candidate
            .assets
            .iter()
            .find(|a| a.id == f.plain_asset)
            .unwrap()
            .origin
            .as_ref()
            .map(|_| ()),
        None
    );
    assert_eq!(
        serde_json::to_value(
            &candidate
                .assets
                .iter()
                .find(|a| a.id == f.asset)
                .unwrap()
                .origin
        )
        .unwrap()["generation"]["alignment"],
        fixture::recipe()["alignment"]
    );
    assert!(!fixture::inventory(&f.dir()).is_empty());
}

#[test]
fn native_narration_cues_presets_events_ducking_and_normalized_preview_export() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let p = f.project();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut inactive = p.clone();
    inactive.master_normalization.as_mut().unwrap().enabled = false;
    for (index, (start, color)) in [
        (500, "#ff0000"),
        (1000, "#00ff00"),
        (1500, "#0000ff"),
        (2400, "#ffff00"),
        (3200, "#00ffff"),
        (4300, "#ff00ff"),
    ]
    .into_iter()
    .enumerate()
    {
        for (offset, alpha) in [(0, 0.0), (100, 0.5), (200, 1.0), (400, 0.0)] {
            let output = renderer
                .render_preview(&inactive, &f.dir(), start + offset)
                .unwrap();
            let actual = rgb(&tools, &f.dir().join(output.relative_path));
            let reference = constant_plate(&tools, color, alpha);
            assert!(
                ssim(&actual, &reference) >= 0.99,
                "cue{index} offset{offset} SSIM{}",
                ssim(&actual, &reference)
            );
            if alpha == 1.0 {
                assert!(
                    ssim(&actual, &vec![0; actual.len()]) < 0.99,
                    "blank negative control did not fail"
                );
            }
        }
    }
    let process = Arc::new(RecordingProcess::default());
    let observed = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()))
        .with_adapters(process.clone(), Arc::new(FileSystemArtifactIo));
    let reference = reference_mix();
    let unnormalized = observed
        .render_preview_range(
            &inactive,
            &f.dir(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 6000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let actual = process.rendered_pcm.lock().unwrap().clone();
    assert!(
        rms(&actual, &reference) <= 0.0001,
        "independent event/routing/duck RMS{}",
        rms(&actual, &reference)
    );
    let mut shifted = reference.clone();
    shifted.rotate_right(4800 * 2);
    assert!(
        rms(&actual, &shifted) > 0.0001,
        "shifted timing negative control"
    );
    let mut wrong_gain = reference.clone();
    for start in [500, 1000, 1500, 2400, 3200, 4300] {
        for (index, sample) in fixture::samples(877.0, 0.1, 300, false).iter().enumerate() {
            wrong_gain[start * 96 + index] -=
                *sample as f32 / 32768.0 * 10_f64.powf(-9.0 / 20.0) as f32;
        }
    }
    assert!(
        rms(&actual, &wrong_gain) > 0.0001,
        "missing captured event/gain negative control"
    );
    let unnormalized_path = f.dir().join(unnormalized.relative_path);
    movie_evidence(&tools, &unnormalized_path, 0, 6000);
    let decoded = decoded_pcm(&tools, &unnormalized_path);
    let expected_decoded = encoded_reference(&tools, root.path());
    let decoded_error = rms(&decoded, &expected_decoded);
    if decoded_error > 0.0001 {
        fs::write(
            root.path().join("actual-precodec.f32le"),
            actual
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let retained = root.keep();
        panic!(
            "aligned decoded independent PCM RMS{decoded_error}; retained {}",
            retained.display()
        );
    }
    // Draft selection with an equivalent cue edit must retain native media.
    let draft = f
        .core
        .create_draft(
            &f.id,
            p.revision,
            vec![fixture::op(json!({
                "operation":"marker_update", "scope":"root", "markerId":p.markers[0].id,
                "name":"EVERY", "timeMs":500, "kind":"cue"
            }))],
            None,
        )
        .unwrap();
    let mut candidate = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
    candidate.master_normalization.as_mut().unwrap().enabled = false;
    let draft_output = observed
        .render_preview_range(
            &candidate,
            &f.dir(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 6000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let draft_path = f.dir().join(draft_output.relative_path);
    movie_evidence(&tools, &draft_path, 0, 6000);
    assert!(rms(&decoded_pcm(&tools, &draft_path), &expected_decoded) <= 0.0001);
    let mut delivered: Option<Vec<f32>> = None;
    for start in [0, 1000] {
        let range_output = observed
            .render_preview_range(
                &p,
                &f.dir(),
                PreviewRangeOptions {
                    start_ms: start,
                    end_ms: 6000,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        movie_evidence(
            &tools,
            &f.dir().join(range_output.relative_path),
            start,
            6000,
        );
        assert!(rms(&process.original.lock().unwrap(), &reference) <= 0.0001);
        let pcm = process.rendered_pcm.lock().unwrap().clone();
        let (integrated, peak) =
            independent_metrics(&tools, &pcm, &root.path().join("delivered.pcm"));
        assert!(
            (integrated + 24.0).abs() <= 0.1,
            "delivered {integrated} LUFS"
        );
        assert!(peak <= -1.99, "delivered peak {peak}");
        if start == 0 {
            let bad_level: Vec<_> = pcm.iter().map(|v| v * 0.5).collect();
            let (bad_integrated, _) =
                independent_metrics(&tools, &bad_level, &root.path().join("wrong-level.pcm"));
            assert!(
                (bad_integrated + 24.0).abs() > 0.1,
                "wrong normalization negative control"
            );
        }

        if let Some(ref previous) = delivered {
            assert!(
                rms(&pcm, previous) <= 0.0001,
                "crop changed full-root normalization"
            );
        }
        delivered = Some(pcm);
    }
    let output = root.path().join("exports/narration.mp4");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    observed
        .export_video(
            &p,
            &f.dir(),
            ExportOptions {
                output: &output,
                width: 64,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    assert!(
        rms(
            &process.rendered_pcm.lock().unwrap(),
            delivered.as_ref().unwrap()
        ) <= 0.0001,
        "export PCM differs"
    );
    movie_evidence(&tools, &output, 0, 6000);
}
