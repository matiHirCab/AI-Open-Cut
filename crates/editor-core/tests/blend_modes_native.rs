//! Independent native blend oracles, exercised through public projects.
use opencut_editor_core::{
    EditOperation, EditorCore, ExportOptions, MediaProbeFacts, MediaType, PathPolicy,
    PreviewRangeOptions, Project, ProjectSettings, Renderer,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn edit(core: &EditorCore, id: &str, value: Value) -> String {
    core.edit(id, core.get_project(id).unwrap().revision, op(value))
        .unwrap()
        .changed_ids
        .first()
        .cloned()
        .unwrap_or_default()
}
struct Native {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    font: PathBuf,
}
impl Native {
    fn configured() -> Option<Self> {
        let configured = match (
            std::env::var_os("OPENCUT_FFMPEG_PATH"),
            std::env::var_os("OPENCUT_FFPROBE_PATH"),
        ) {
            (Some(ffmpeg), Some(ffprobe)) => Some(Self {
                ffmpeg: ffmpeg.into(),
                ffprobe: ffprobe.into(),
                font: std::env::var_os("OPENCUT_TEST_FONT_PATH")
                    .expect("native blend oracle requires bundled font")
                    .into(),
            }),
            _ => None,
        };
        if configured.is_none() {
            for flag in [
                "OPENCUT_GOLDEN_REQUIRED",
                "OPENCUT_TRACK_MATTE_RENDER_REQUIRED",
                "OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED",
            ] {
                assert_ne!(
                    std::env::var(flag).as_deref(),
                    Ok("1"),
                    "required native blend tools missing"
                );
            }
        }
        configured
    }
    fn capturing_renderer(&self, root: &Path) -> (Renderer, PathBuf) {
        let source = root.join("ffmpeg_capture.rs");
        let executable = root.join(format!("ffmpeg-capture{}", std::env::consts::EXE_SUFFIX));
        let captured = root.join("actual-linear-scene.pam");
        let program = format!(
            r#"
use std::{{path::Path, process::{{Command, Stdio}}}};
fn main() {{
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    for argument in &args {{
        let path = Path::new(argument);
        if path.file_name() == Some(std::ffi::OsStr::new("linear-scene.pam")) {{
            std::fs::copy(path, {:?}).expect("copy actual prepared scene");
        }}
    }}
    let status = Command::new({:?}).args(args).stdin(Stdio::inherit())
        .stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().expect("forward real FFmpeg");
    if let Some(code) = status.code() {{ std::process::exit(code); }}
    #[cfg(unix)] {{ use std::os::unix::process::ExitStatusExt; std::process::exit(128 + status.signal().unwrap_or(1)); }}
    #[cfg(not(unix))] std::process::exit(1);
}}
"#,
            captured.to_str().unwrap(),
            self.ffmpeg.to_str().unwrap()
        );
        std::fs::write(&source, program).unwrap();
        let result = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition", "2024", "--crate-name", "ffmpeg_capture"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        (
            Renderer::new(executable, &self.ffprobe, Some(self.font.clone())),
            captured,
        )
    }
    fn authored_plate(
        &self,
        root: &Path,
        regions: &[(usize, usize, usize, usize, [f64; 3])],
    ) -> Vec<u8> {
        let mut pixels = vec![[0, 0, 0, 255]; 96 * 64];
        for &(left, top, right, bottom, color) in regions {
            for y in top..bottom {
                for x in left..right {
                    pixels[y * 96 + x] =
                        [encoded(color[0]), encoded(color[1]), encoded(color[2]), 255];
                }
            }
        }
        let path = root.join("independently-authored.pam");
        let mut bytes =
            b"P7\nWIDTH 96\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
        for pixel in pixels {
            bytes.extend(pixel);
        }
        std::fs::write(&path, bytes).unwrap();
        let output = root.join("independently-converted.png");
        let result = Command::new(&self.ffmpeg).args(["-v", "error", "-nostdin", "-f", "lavfi", "-i", "color=c=black:s=96x64:r=10:d=1"])
            .args(["-loop", "1", "-i"]).arg(&path).args(["-filter_complex_threads", "1", "-filter_complex",
                "[0:v]format=yuv420p[base0];[1:v]fps=10,settb=AVTB,setpts=PTS-STARTPTS+0/TB,format=rgba[plate];[base0][plate]overlay=format=auto:x=0:y=0:eof_action=pass[composed];[composed]scale=96:64:force_original_aspect_ratio=decrease,pad=96:64:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]",
                "-map", "[video]", "-frames:v", "1", "-y"]).arg(&output).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        self.decode(&output, false, None)
    }
    fn decode(&self, path: &Path, audio: bool, seek: Option<&str>) -> Vec<u8> {
        let mut cmd = Command::new(&self.ffmpeg);
        cmd.args(["-v", "error", "-i"]).arg(path);
        if let Some(seek) = seek {
            cmd.args(["-ss", seek]);
        }
        if audio {
            cmd.args([
                "-map",
                "0:a:0",
                "-f",
                "f32le",
                "-acodec",
                "pcm_f32le",
                "pipe:1",
            ]);
        } else {
            cmd.args([
                "-map",
                "0:v:0",
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ]);
        }
        let result = cmd.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        result.stdout
    }
    fn probe(&self, path: &Path) -> Value {
        let result = Command::new(&self.ffprobe).args(["-v", "error", "-show_entries", "format=duration:stream=codec_type,time_base,duration,nb_frames,width,height,sample_rate,channels", "-of", "json"]).arg(path).output().unwrap();
        assert!(result.status.success());
        serde_json::from_slice(&result.stdout).unwrap()
    }
}
fn setup(name: &str) -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [&media],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project(
            name,
            ProjectSettings {
                width: 96,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}
fn rectangle(
    core: &EditorCore,
    id: &str,
    track: &str,
    color: &str,
    bounds: (f64, f64, u32, u32),
    alpha: f64,
) -> String {
    let (x, y, w, h) = bounds;
    edit(
        core,
        id,
        json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":w,"height":h,"color":color,
        "transform":{"positionX":x,"positionY":y,"scale":1,"opacity":alpha}}),
    )
}
fn encoded(linear: f64) -> u8 {
    let value = if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (255.0 * value).round() as u8
}
fn assert_pixels(bytes: &[u8], points: &[(usize, usize, [f64; 3])]) {
    let (pixels, remainder) = bytes.as_chunks::<3>();
    assert!(remainder.is_empty());
    assert_eq!(pixels.len(), 96 * 64);
    for &(x, y, expected) in points {
        for (actual, expected) in pixels[y * 96 + x].iter().zip(expected) {
            assert!(
                (i32::from(*actual) - i32::from(encoded(expected))).abs() <= 1,
                "pixel({x},{y}) expected linear{expected}, got{actual}"
            );
        }
    }
}
fn raw_pam(path: &Path) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("actual production linear-scene.pam must be captured");
    let end = bytes.windows(7).position(|b| b == b"ENDHDR\n").unwrap() + 7;
    let header = std::str::from_utf8(&bytes[..end]).unwrap();
    for field in [
        "P7",
        "WIDTH 96",
        "HEIGHT 64",
        "DEPTH 4",
        "MAXVAL 255",
        "TUPLTYPE RGB_ALPHA",
    ] {
        assert!(
            header.lines().any(|line| line == field),
            "missing actual PAM field {field}"
        );
    }
    let (pixels, rest) = bytes[end..].as_chunks::<4>();
    assert!(rest.is_empty());
    assert_eq!(pixels.len(), 96 * 64);
    pixels
        .iter()
        .flat_map(|p| {
            assert_eq!(p[3], 255, "actual composed scene alpha must be opaque");
            p[..3].iter().copied()
        })
        .collect()
}
fn assert_converted(actual: &[u8], expected: &[u8], points: &[(usize, usize, [f64; 3])]) {
    assert_eq!(actual.len(), expected.len());
    let (actual, rest) = actual.as_chunks::<3>();
    assert!(rest.is_empty());
    let (expected, rest) = expected.as_chunks::<3>();
    assert!(rest.is_empty());
    assert!(!points.is_empty());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        for (a, b) in actual.iter().zip(expected) {
            assert!(
                (i32::from(*a) - i32::from(*b)).abs() <= 1,
                "independent full plate pixel{index}: {a} != {b}"
            );
        }
    }
}
fn ssim(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len() as f64;
    let ma = a.iter().map(|x| f64::from(*x)).sum::<f64>() / n;
    let mb = b.iter().map(|x| f64::from(*x)).sum::<f64>() / n;
    let va = a.iter().map(|x| (f64::from(*x) - ma).powi(2)).sum::<f64>() / (n - 1.0);
    let vb = b.iter().map(|x| (f64::from(*x) - mb).powi(2)).sum::<f64>() / (n - 1.0);
    let cov = a
        .iter()
        .zip(b)
        .map(|(x, y)| (f64::from(*x) - ma) * (f64::from(*y) - mb))
        .sum::<f64>()
        / (n - 1.0);
    ((2.0 * ma * mb + 6.5025) * (2.0 * cov + 58.5225))
        / ((ma * ma + mb * mb + 6.5025) * (va + vb + 58.5225))
}
fn pcm_equal(a: &[u8], b: &[u8]) {
    assert_eq!(a, b, "blend changes must preserve exact decoded PCM");
    assert_eq!(a.len(), b.len());
    let (a, ra) = a.as_chunks::<4>();
    let (b, rb) = b.as_chunks::<4>();
    assert!(ra.is_empty() && rb.is_empty() && !a.is_empty());
    assert!(a.iter().any(|p| f32::from_le_bytes(*p).abs() > 0.01));
    let rms = (a
        .iter()
        .zip(b)
        .map(|(a, b)| {
            (f64::from(f32::from_le_bytes(*a)) - f64::from(f32::from_le_bytes(*b))).powi(2)
        })
        .sum::<f64>()
        / a.len() as f64)
        .sqrt();
    assert!(rms <= 0.0001, "blend changed audio RMS{rms}");
}
fn write_tone(path: &Path) {
    let count = 48000_u32;
    let mut bytes = Vec::new();
    bytes.extend(b"RIFF");
    bytes.extend((36 + count * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(96000_u32.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((count * 2).to_le_bytes());
    for _ in 0..count {
        bytes.extend(8192_i16.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}
fn add_audio(core: &EditorCore, id: &str, path: &Path) {
    write_tone(path);
    let asset = core
        .import_asset(
            id,
            core.get_project(id).unwrap().revision,
            path,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let track = core.get_project(id).unwrap().tracks[2].id.clone();
    edit(
        core,
        id,
        json!({"operation":"add_media","trackId":track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}),
    );
}
// The frame oracle is computed from authored linear colors, never captured from
// an effect/matte helper. Encoded all-intent comparisons retain the existing SSIM gate.
fn all_intents(
    native: &Native,
    root: &Path,
    core: &EditorCore,
    id: &str,
    versions: (&Project, &Project, &Project),
    points: &[(usize, usize, [f64; 3])],
    regions: &[(usize, usize, usize, usize, [f64; 3])],
) {
    let (baseline, candidate, current) = versions;
    let (renderer, captured) = native.capturing_renderer(root);
    let converted = native.authored_plate(root, regions);
    let dir = core.paths().project_dir(id).unwrap();
    let before_frame = renderer.render_preview(baseline, &dir, 400).unwrap();
    let before_pixels = native.decode(&dir.join(before_frame.relative_path), false, None);
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(id)
        .unwrap();
    let mut reference = None;
    for (index, project) in [candidate, current, &reopened].into_iter().enumerate() {
        if captured.exists() {
            std::fs::remove_file(&captured).unwrap();
        }
        let frame = renderer.render_preview(project, &dir, 400).unwrap();
        let pixels = native.decode(&dir.join(&frame.relative_path), false, None);
        // Save ACTUAL inputs/results before any assertion for independent diagnostics.
        let diagnostics = std::env::var_os("OPENCUT_NATIVE_DIAGNOSTIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("diagnostics"));
        std::fs::create_dir_all(&diagnostics).unwrap();
        std::fs::write(
            diagnostics.join(format!("{id}-{index}-project.json")),
            serde_json::to_vec_pretty(project).unwrap(),
        )
        .unwrap();
        std::fs::copy(
            &captured,
            diagnostics.join(format!("{id}-{index}-linear-scene.pam")),
        )
        .unwrap();
        std::fs::copy(
            dir.join(frame.relative_path),
            diagnostics.join(format!("{id}-{index}-preview.png")),
        )
        .unwrap();
        assert_pixels(&raw_pam(&captured), points);
        assert_converted(&pixels, &converted, points);
        if let Some(reference) = &reference {
            assert_eq!(&pixels, reference, "draft/current/reopen frame differs");
        } else {
            reference = Some(pixels);
        }
    }
    core.undo(id, core.get_project(id).unwrap().revision)
        .unwrap();
    let undone = core.get_project(id).unwrap();
    let undone_frame = renderer.render_preview(&undone, &dir, 400).unwrap();
    assert_eq!(
        native.decode(&dir.join(undone_frame.relative_path), false, None),
        before_pixels,
        "undo did not restore native prior generation"
    );
    core.redo(id, undone.revision).unwrap();
    let redone = core.get_project(id).unwrap();
    let redone_frame = renderer.render_preview(&redone, &dir, 400).unwrap();
    assert_eq!(
        &native.decode(&dir.join(redone_frame.relative_path), false, None),
        reference.as_ref().unwrap(),
        "redo did not restore native blend generation"
    );
    let render_range = |p: &Project| {
        renderer
            .render_preview_range(
                p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 100,
                    end_ms: 900,
                    width: 96,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap()
    };
    let base_range = render_range(baseline);
    let base_range_path = dir.join(base_range.relative_path);
    // Decode before a later preview request can replace an existing output path.
    let base_range_pcm = native.decode(&base_range_path, true, None);
    let base_range_probe = native.probe(&base_range_path);
    let range = render_range(current);
    let range_path = dir.join(range.relative_path);
    let output = root.join("exports/blends.mp4");
    let base_output = root.join("exports/baseline.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    for (project, path) in [(baseline, &base_output), (current, &output)] {
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: path,
                    width: 96,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
    }
    let reference = reference.unwrap();
    for (path, seek, frames, duration, base_pcm, base_probe) in [
        (range_path, "0.3", 8, 0.8, base_range_pcm, base_range_probe),
        (
            output,
            "0.4",
            10,
            1.0,
            native.decode(&base_output, true, None),
            native.probe(&base_output),
        ),
    ] {
        let actual = native.decode(&path, false, Some(seek));
        assert!(
            ssim(&reference, &actual) >= 0.99,
            "native blend intent SSIM below existing gate"
        );
        pcm_equal(&native.decode(&path, true, None), &base_pcm);
        let probe = native.probe(&path);
        assert_eq!(probe, base_probe, "blend changed timing/audio metadata");
        let video = probe["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["codec_type"] == "video")
            .unwrap();
        assert_eq!(
            video["nb_frames"]
                .as_str()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
            frames
        );
        assert!(
            (probe["format"]["duration"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                - duration)
                .abs()
                <= 0.001
        );
    }
}

#[test]
fn native_seven_modes_full_independent_plate_draft_history_reopen_range_export_audio() {
    let Some(native) = Native::configured() else {
        return;
    };
    let contract: Value =
        serde_json::from_str(include_str!("../../../contracts/blend-modes-v1.json")).unwrap();
    for mode in contract["fields"]["blendMode"]["values"]
        .as_array()
        .unwrap()
    {
        let mode = mode.as_str().unwrap();
        let (root, core, id, track) = setup(&format!("Native {mode}"));
        let tone = root.path().join("media/tone.wav");
        write_tone(&tone);
        add_audio(&core, &id, &tone);
        rectangle(&core, &id, &track, "#80c040", (0.0, 0.0, 96, 64), 1.0);
        let leaf = rectangle(&core, &id, &track, "#e06020", (0.0, 0.0, 96, 64), 0.5);
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":leaf,"effects":[{"type":"color_tint","id":"identity-tint","color":{"r":0,"g":0,"b":0,"a":0}}]}),
        );
        let baseline = core.get_project(&id).unwrap();
        let operation = op(json!({"operation":"update_item","itemId":leaf,"blendMode":mode}));
        let draft = core
            .create_draft(
                &id,
                baseline.revision,
                vec![operation],
                Some(format!("{mode} blend")),
            )
            .unwrap();
        let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
        core.commit_draft(&id, &draft.id, baseline.revision)
            .unwrap();
        let current = core.get_project(&id).unwrap();
        assert_eq!(
            serde_json::to_value(
                current
                    .find_item(&leaf)
                    .unwrap()
                    .visual_properties()
                    .blend_mode
            )
            .unwrap(),
            mode
        );
        let oracle =
            &contract["numericOracles"]["nativeAuthoredColored"]["expected"][mode]["unroundedF64"];
        let color = std::array::from_fn(|i| oracle[i].as_f64().unwrap());
        all_intents(
            &native,
            root.path(),
            &core,
            &id,
            (&baseline, &candidate, &current),
            &[(48, 32, color)],
            &[(0, 0, 96, 64, color)],
        );
    }
}

#[test]
fn native_public_first_multiply_is_black_and_default_normal_remains_red() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("First multiply");
    let leaf = rectangle(&core, &id, &track, "#ff0000", (0.0, 0.0, 96, 64), 1.0);
    let renderer = Renderer::new(&native.ffmpeg, &native.ffprobe, Some(native.font.clone()));
    let dir = core.paths().project_dir(&id).unwrap();
    let normal = core.get_project(&id).unwrap();
    let frame = renderer.render_preview(&normal, &dir, 400).unwrap();
    let pixels = native.decode(&dir.join(frame.relative_path), false, None);
    assert_eq!(pixels.len(), 96 * 64 * 3);
    let center = (32 * 96 + 48) * 3;
    assert!(pixels[center] > 240 && pixels[center + 1] < 5 && pixels[center + 2] < 5);
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":leaf,"blendMode":"multiply"}),
    );
    let multiply = core.get_project(&id).unwrap();
    let frame = renderer.render_preview(&multiply, &dir, 400).unwrap();
    let pixels = native.decode(&dir.join(frame.relative_path), false, None);
    assert!(pixels[center..center + 3].iter().all(|v| *v <= 1));
    let persisted = serde_json::to_value(&multiply).unwrap();
    assert_eq!(persisted["tracks"][1]["items"][0]["blendMode"], "multiply");
    assert_eq!(
        serde_json::to_value(
            EditorCore::new(core.paths().clone())
                .get_project(&id)
                .unwrap()
        )
        .unwrap(),
        persisted
    );
    assert!(root.path().exists());
}

#[test]
fn native_public_noncommuting_reorder_draft_commit_undo_redo_reopen_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let contract: Value =
        serde_json::from_str(include_str!("../../../contracts/blend-modes-v1.json")).unwrap();
    let oracle = &contract["numericOracles"]["nativePublicOrder"];
    let (root, core, id, track) = setup("Native noncommuting order");
    let tone = root.path().join("media/tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    rectangle(&core, &id, &track, "#80c040", (0.0, 0.0, 96, 64), 1.0);
    let multiply = rectangle(&core, &id, &track, "#e06020", (0.0, 0.0, 96, 64), 0.5);
    let screen = rectangle(&core, &id, &track, "#2080e0", (0.0, 0.0, 96, 64), 0.375);
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":multiply,"blendMode":"multiply"}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":screen,"blendMode":"screen"}),
    );
    let baseline = core.get_project(&id).unwrap();
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let frame = renderer.render_preview(&baseline, &dir, 400).unwrap();
    let before_color = std::array::from_fn(|i| oracle["multiplyThenScreen"][i].as_f64().unwrap());
    let expected = native.authored_plate(root.path(), &[(0, 0, 96, 64, before_color)]);
    assert_pixels(&raw_pam(&captured), &[(48, 32, before_color)]);
    assert_converted(
        &native.decode(&dir.join(frame.relative_path), false, None),
        &expected,
        &[(48, 32, before_color)],
    );
    let draft = core
        .create_draft(
            &id,
            baseline.revision,
            vec![op(
                json!({"operation":"item_reorder","itemId":screen,"index":1}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.commit_draft(&id, &draft.id, baseline.revision)
        .unwrap();
    let current = core.get_project(&id).unwrap();
    assert_eq!(current.tracks[1].items[1].id(), screen);
    let color = std::array::from_fn(|i| oracle["screenThenMultiply"][i].as_f64().unwrap());
    assert!((color[2] - before_color[2]).abs() > 0.1);
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[(48, 32, color)],
        &[(0, 0, 96, 64, color)],
    );
}

#[test]
fn native_multiline_caption_assembles_white_glyph_and_translucent_box_before_multiply() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Whole multiline Caption");
    let tone = root.path().join("media/tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    let asset = core.get_project(&id).unwrap().assets[0].id.clone();
    rectangle(&core, &id, &track, "#aaaaaa", (0., 0., 96, 64), 1.);
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Multiline caption","width":96,"height":64,"durationMs":1000,"slots":[],"tracks":[{"id":"captions","name":"Captions","trackType":"caption","items":[{"type":"caption","id":"whole-caption","text":"\u{2588}\n\u{2588}","startMs":0,"durationMs":1000,"style":{"fontSize":16,"color":"#ffffff","backgroundColor":"#000000","bottomMarginPx":0},"source":{"assetId":asset,"providerId":"native-oracle","modelId":"manual","modelVersion":null,"language":"en","generatedAtMs":1,"originalText":"Blocks","confidence":null,"words":[]},"transform2d":opencut_editor_core::Transform2D::default(),"blendMode":"multiply","stackOrder":0,"zIndex":0}]}]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{}}),
    );
    let project = core.get_project(&id).unwrap();
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let frame = renderer.render_preview(&project, &dir, 400).unwrap();
    let background = ((170.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4);
    // Independently selected interior points of the bundled DejaVu block glyph:
    // two white source fragments must each retain the backdrop, whereas a
    // forbidden box-first multiply then glyph-multiply leaves only25 percent.
    assert_pixels(
        &raw_pam(&captured),
        &[
            (18, 20, [background; 3]),
            (18, 40, [background; 3]),
            (4, 4, [background * 0.25; 3]),
            (80, 32, [background; 3]),
        ],
    );
    let reference = native.decode(&dir.join(frame.relative_path), false, None);
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    let again = renderer.render_preview(&reopened, &dir, 400).unwrap();
    assert_eq!(
        native.decode(&dir.join(again.relative_path), false, None),
        reference
    );
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 96,
                height: 64,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let range_path = dir.join(range.relative_path);
    assert!(ssim(&reference, &native.decode(&range_path, false, Some("0.3"))) >= 0.99);
    let output = root.path().join("exports/caption.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: 96,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    assert!(ssim(&reference, &native.decode(&output, false, Some("0.4"))) >= 0.99);
    assert!(!native.decode(&range_path, true, None).is_empty());
    assert!(!native.decode(&output, true, None).is_empty());
}

#[test]
fn native_real_shutter_source_average_precedes_nonlinear_add_once() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Real source-average add");
    let tone = root.path().join("media/tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    rectangle(&core, &id, &track, "#e1e1e1", (0., 0., 96, 64), 1.);
    let leaf = rectangle(&core, &id, &track, "#ffffff", (0., 0., 96, 64), 1.);
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":leaf,"effects":[{"type":"color_tint","id":"shutter-color","color":{"r":0,"g":0,"b":0,"a":1}}]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":leaf,"animationChannels":[{"property":"effect.tint_color","target":{"kind":"effect","scope":"root","id":"shutter-color"},"keyframes":[{"timeMs":0,"value":{"type":"rgba","r":0,"g":0,"b":0,"a":1},"curve":"hold"},{"timeMs":400,"value":{"type":"rgba","r":1,"g":1,"b":1,"a":1},"curve":"hold"}]}]}),
    );
    let baseline = core.get_project(&id).unwrap();
    let draft=core.create_draft(&id,baseline.revision,vec![op(json!({"operation":"update_item","itemId":leaf,"blendMode":"add","motionBlur":{"shutterAngleDeg":180,"sampleCount":2}}))],None).unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.commit_draft(&id, &draft.id, baseline.revision)
        .unwrap();
    let current = core.get_project(&id).unwrap();
    // At10fps and180-degree shutter the two actual leaf sample times straddle
    // the authored400ms held color change. Black/white source average=.5;
    // add against the authored .7529 backdrop saturates1, unlike blend first.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[(48, 32, [1.; 3])],
        &[(0, 0, 96, 64, [1.; 3])],
    );
}

#[test]
fn native_provider_blend_is_coverage_independent_and_only_visible_direct_draw_changes() {
    let Some(native) = Native::configured() else {
        return;
    };
    for matte_only in [true, false] {
        let (root, core, id, track) = setup("Provider blend independence");
        let tone = root.path().join("media/tone.wav");
        write_tone(&tone);
        add_audio(&core, &id, &tone);
        rectangle(&core, &id, &track, "#aaaaaa", (0., 0., 96, 64), 1.);
        let provider = rectangle(&core, &id, &track, "#ffffff", (16., 16., 32, 32), 0.5);
        let receiver = rectangle(&core, &id, &track, "#0000ff", (16., 16., 32, 32), 1.);
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":provider,"matteOnly":matte_only}),
        );
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":receiver,"blendMode":"multiply","matte":{"sourceId":provider,"channel":"alpha"}}),
        );
        let baseline = core.get_project(&id).unwrap();
        let draft = core
            .create_draft(
                &id,
                baseline.revision,
                vec![op(
                    json!({"operation":"update_item","itemId":provider,"blendMode":"multiply"}),
                )],
                None,
            )
            .unwrap();
        let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
        core.commit_draft(&id, &draft.id, baseline.revision)
            .unwrap();
        let current = core.get_project(&id).unwrap();
        let gray = ((170.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4);
        let inside = [gray * 0.5, gray * 0.5, gray];
        all_intents(
            &native,
            root.path(),
            &core,
            &id,
            (&baseline, &candidate, &current),
            &[(32, 32, inside), (80, 32, [gray; 3])],
            &[(0, 0, 96, 64, [gray; 3]), (16, 16, 48, 48, inside)],
        );
        let dir = core.paths().project_dir(&id).unwrap();
        let renderer = Renderer::new(&native.ffmpeg, &native.ffprobe, Some(native.font.clone()));
        let before = renderer.render_preview(&baseline, &dir, 400).unwrap();
        let before = native.decode(&dir.join(before.relative_path), false, None);
        let after = renderer.render_preview(&current, &dir, 400).unwrap();
        let after = native.decode(&dir.join(after.relative_path), false, None);
        if matte_only {
            assert_eq!(
                before, after,
                "matteOnly provider mode must not change coverage or backdrop"
            );
        } else {
            assert_ne!(
                before, after,
                "visible provider mode must change its own direct contribution"
            );
        }
    }
}

#[test]
fn native_blend_follows_mask_effect_affine_matte_opacity_and_animated_ancestor_stages() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Complete owning blend stages");
    let tone = root.path().join("media/tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    rectangle(&core, &id, &track, "#808080", (0., 0., 96, 64), 1.);
    let provider = rectangle(&core, &id, &track, "#ffffff", (0., 0., 96, 64), 0.5);
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider,"matteOnly":true,"blendMode":"multiply"}),
    );
    let group = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":group,"transform2d":null}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":group,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"},{"timeMs":999,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}),
    );
    let leaf = rectangle(&core, &id, &track, "#ff0000", (0., 0., 16, 12), 0.5);
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":leaf,"parent":{"scope":"root","id":group}}),
    );
    let mut mask: Value =
        serde_json::from_str::<Value>(include_str!("../../../contracts/mask-models-v1.json"))
            .unwrap()["cases"][0]["value"]
            .clone();
    mask["source"]["paint"] = json!({"type":"solid","color":{"r":1,"g":1,"b":1,"a":0.5}});
    mask["transform"] = serde_json::to_value(opencut_editor_core::Transform2D::default()).unwrap();
    mask["featherPx"] = json!(0);
    mask["expansionPx"] = json!(0);
    mask["source"]["path"]["commands"] = json!([{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":16,"y":0}},{"type":"lineTo","to":{"x":16,"y":12}},{"type":"lineTo","to":{"x":0,"y":12}},{"type":"close"}]);
    let mut transform = opencut_editor_core::Transform2D::default();
    transform.position.x = 32.;
    transform.position.y = 32.;
    transform.anchor.x = 0.25;
    transform.anchor.y = 0.75;
    transform.rotation_deg = 90.;
    transform.opacity = 0.5;
    let baseline = core.get_project(&id).unwrap();
    let draft=core.create_draft(&id,baseline.revision,vec![op(json!({"operation":"update_item","itemId":leaf,"blendMode":"screen","transform2d":transform,"masks":[mask],"effects":[{"type":"color_tint","id":"green","color":{"r":0,"g":1,"b":0,"a":1}}],"matte":{"sourceId":provider,"channel":"alpha"},"motionBlur":{"shutterAngleDeg":180,"sampleCount":2}}))],None).unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.commit_draft(&id, &draft.id, baseline.revision)
        .unwrap();
    let current = core.get_project(&id).unwrap();
    let gray = ((128.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4);
    let color = [gray, gray + (1. - gray) * 0.0625, gray];
    // R90 about authored(.25,.75) maps the16x12 source to[29,41]x[28,44].
    // Own opacity .5, ancestor .5, white mask .5 and provider alpha .5 give
    // complete green-source alpha1/16 before the sole screen operation.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[(35, 36, color), (80, 32, [gray; 3])],
        &[(0, 0, 96, 64, [gray; 3]), (29, 28, 41, 44, color)],
    );
}

#[test]
fn native_repeater_and_component_leaves_retain_individual_modes_without_container_isolation() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Expanded leaf blend modes");
    let tone = root.path().join("media/tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    rectangle(&core, &id, &track, "#808080", (0., 0., 96, 64), 1.);
    let leaf = edit(
        &core,
        &id,
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":16,"height":16},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":leaf,"transform":{"positionX":16,"positionY":16,"scale":1,"opacity":0.5}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":leaf},"copies":2,"timeOffsetMs":0,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}),
    );
    let mut local =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&leaf).unwrap()).unwrap();
    local["id"] = json!("local-shape");
    local["stackOrder"] = json!(0);
    local["transform"]["positionX"] = json!(0);
    local["transform"]["positionY"] = json!(0);
    let tracks = json!([{"id":"local","name":"Local","trackType":"overlay","items":[local]}]);
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Local eligible leaf","width":24,"height":24,"durationMs":1000,"slots":[],"tracks":tracks}),
    );
    let instance = edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":instance,"transform":{"positionX":48,"positionY":16,"scale":1,"opacity":1}}),
    );
    let baseline = core.get_project(&id).unwrap();
    let mut tracks = tracks;
    tracks[0]["items"][0]["blendMode"] = json!("screen");
    let draft=core.create_draft(&id,baseline.revision,vec![op(json!({"operation":"update_item","itemId":leaf,"blendMode":"screen"})),op(json!({"operation":"component_update","componentId":component,"name":"Local eligible leaf","width":24,"height":24,"durationMs":1000,"slots":[],"tracks":tracks}))],None).unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.commit_draft(&id, &draft.id, baseline.revision)
        .unwrap();
    let current = core.get_project(&id).unwrap();
    let gray = ((128.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4);
    let repeated = [gray + (1. - gray) * 0.875, gray, gray];
    let local = [gray + (1. - gray) * 0.5, gray, gray];
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[(24, 24, repeated), (56, 24, local), (80, 32, [gray; 3])],
        &[
            (0, 0, 96, 64, [gray; 3]),
            (16, 16, 32, 32, repeated),
            (48, 16, 64, 32, local),
        ],
    );
}

#[test]
fn native_graph_free_pinned_text_paints_assemble_once_and_hidden_pin_failure_precedes_cached_output()
 {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Whole pinned painted Text");
    let media = root.path().join("media");
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        let source = if name == "DejaVuSans.ttf" {
            native.font.clone()
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("resources/fonts")
                .join(name)
        };
        std::fs::copy(source, media.join(name)).unwrap();
    }
    let core = core.with_font_config(opencut_editor_core::FontConfig {
        roots: vec![media.clone()],
        default_path: None,
    });
    let tone = media.join("tone.wav");
    write_tone(&tone);
    add_audio(&core, &id, &tone);
    rectangle(&core, &id, &track, "#aaaaaa", (0., 0., 96, 64), 1.);
    let leaf = edit(
        &core,
        &id,
        json!({"operation":"add_text","trackId":track,"text":"\u{2588}","fontFamily":"DejaVu Sans","fontSize":40,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":16,"positionY":16,"scale":1,"opacity":1}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":leaf,"style":{"backgroundOpacity":0,"alignment":"left","layout":{"bounds":{"widthPx":30,"heightPx":44},"fit":"shrink","wrap":"none","verticalAlignment":"top"},"paintLayers":[{"kind":"fill","color":"#000000","opacity":0.75},{"kind":"fill","color":"#ffffff","opacity":1}]},"blendMode":"multiply"}),
    );
    let project = core.get_project(&id).unwrap();
    assert_eq!(project.fonts.len(), 4);
    let binding =
        serde_json::to_value(project.find_item(&leaf).unwrap()).unwrap()["fontBinding"]["regular"]
            .as_str()
            .unwrap()
            .to_owned();
    let pinned = project.fonts.get(&binding).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let pinned_path = dir.join(&pinned.relative_path);
    assert_eq!(
        std::fs::read(&pinned_path).unwrap(),
        std::fs::read(&native.font).unwrap()
    );
    // Independent bundled outline/advance units1575/2048 and extent1615x2433
    // select37px in the30x44 shrink box. Pixel center(28.5,36.5) lies well
    // inside its translated full block; the assembled white source is opaque.
    let font_bytes = std::fs::read(&native.font).unwrap();
    let face = ttf_parser::Face::parse(&font_bytes, 0).unwrap();
    assert_eq!(face.units_per_em(), 2048);
    let block = face.glyph_index('\u{2588}').unwrap();
    let advance = f64::from(face.glyph_hor_advance(block).unwrap());
    let height = f64::from(face.ascender() - face.descender());
    assert_eq!((advance, height), (1575., 2384.));
    let em = f64::from(face.units_per_em());
    assert!(advance * 37. / em <= 30. && height * 37. / em <= 44.);
    assert!(height * 38. / em > 44.);
    let gray = ((170.0_f64 / 255.0 + 0.055) / 1.055).powf(2.4);
    let (renderer, captured) = native.capturing_renderer(root.path());
    let frame = renderer.render_preview(&project, &dir, 400).unwrap();
    assert_pixels(
        &raw_pam(&captured),
        &[(28, 36, [gray; 3]), (80, 32, [gray; 3])],
    );
    let reference = native.decode(&dir.join(frame.relative_path), false, None);
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    let again = renderer.render_preview(&reopened, &dir, 400).unwrap();
    assert_eq!(
        native.decode(&dir.join(again.relative_path), false, None),
        reference
    );
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 100,
                end_ms: 900,
                width: 96,
                height: 64,
                fps: 10,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let range_path = dir.join(range.relative_path);
    assert!(ssim(&reference, &native.decode(&range_path, false, Some("0.3"))) >= 0.99);
    let output = root.path().join("exports/text.mp4");
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: 96,
                height: 64,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    assert!(ssim(&reference, &native.decode(&output, false, Some("0.4"))) >= 0.99);
    edit(
        &core,
        &id,
        json!({"operation":"set_item_visibility","itemId":leaf,"hidden":true}),
    );
    let hidden = core.get_project(&id).unwrap();
    renderer.render_preview(&hidden, &dir, 400).unwrap();
    let mut corrupt = std::fs::read(&pinned_path).unwrap();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 0xff;
    std::fs::write(&pinned_path, &corrupt).unwrap();
    fn files(dir: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        let mut result = std::collections::BTreeMap::new();
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                result.extend(files(&path));
            } else {
                result.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
        result
    }
    let before = files(&dir);
    let error = renderer.render_preview(&hidden, &dir, 400).unwrap_err();
    assert_eq!(
        error.code,
        opencut_editor_core::ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(
        files(&dir),
        before,
        "hidden pinned corruption must reject before new artifact/cache output"
    );
}
