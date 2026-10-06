//! Independent native track-matte oracles, exercised through public projects.
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
                    .expect("native matte oracle requires bundled font")
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
                    "required native matte tools missing"
                );
            }
        }
        configured
    }
    fn renderer(&self) -> Renderer {
        Renderer::new(&self.ffmpeg, &self.ffprobe, Some(self.font.clone()))
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
    for &(x, y, _) in points {
        for (a, b) in actual[y * 96 + x].iter().zip(expected[y * 96 + x]) {
            assert!(
                (i32::from(*a) - i32::from(b)).abs() <= 1,
                "converted pixel({x},{y}) expected{b}, got{a}"
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
    assert!(rms <= 0.0001, "matte changed audio RMS{rms}");
}
fn inventory(root: &Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(
        root: &Path,
        path: &Path,
        result: &mut std::collections::BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            assert!(
                !kind.is_symlink(),
                "fixture managed inventory must not follow symlinks"
            );
            if kind.is_dir() {
                result.insert(path.strip_prefix(root).unwrap().to_owned(), None);
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    Some(std::fs::read(path).unwrap()),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(root, root, &mut result);
    result
}
fn zero_provider_integrity(
    native: &Native,
    root: &Path,
    dir: &Path,
    project: &Project,
    warm: &Renderer,
    captured: &Path,
    payload: (&Path, &[u8], &[u8], bool),
) {
    let (managed, original, corrupt, prepared_scene_required) = payload;
    assert_eq!(
        std::fs::read(managed).unwrap(),
        original,
        "valid zero-provider control must restore original pinned bytes"
    );
    let (fresh, fresh_capture) = native.capturing_renderer(root);
    assert_eq!(fresh_capture, captured);
    for (index, renderer) in [warm, warm, &fresh].into_iter().enumerate() {
        if captured.exists() {
            std::fs::remove_file(captured).unwrap();
        }
        let frame = renderer.render_preview(project, dir, 400).unwrap();
        let pixels = native.decode(&dir.join(&frame.relative_path), false, None);
        assert_eq!(pixels.len(), 96 * 64 * 3);
        assert!(
            pixels.iter().all(|byte| *byte == 0),
            "valid contextual-zero public frame must remain exactly black"
        );
        if captured.exists() {
            let raw = raw_pam(captured);
            assert!(
                raw.iter().all(|byte| *byte == 0),
                "hidden/inactive provider must give actual transparent-zero coverage on black canvas"
            );
            assert_eq!(pixels, raw);
        } else {
            // The unused-only control has no evaluated matte occurrence and may
            // retain the ordinary compositor. Every original active/hidden/
            // inactive case still REQUIRES its actual prepared-PAM oracle.
            assert!(
                !prepared_scene_required,
                "evaluated matte occurrence must produce the actual prepared scene"
            );
        }
        let diagnostic_dir = std::env::var_os("OPENCUT_NATIVE_DIAGNOSTIC_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("diagnostics"));
        std::fs::create_dir_all(&diagnostic_dir).unwrap();
        let stem = format!("{}-zero-revision{}-{index}", project.id, project.revision);
        if captured.exists() {
            std::fs::copy(
                captured,
                diagnostic_dir.join(format!("{stem}-linear-scene.pam")),
            )
            .unwrap();
        }
        std::fs::write(
            diagnostic_dir.join(format!("{stem}-project.json")),
            serde_json::to_vec_pretty(project).unwrap(),
        )
        .unwrap();
        std::fs::copy(
            dir.join(frame.relative_path),
            diagnostic_dir.join(format!("{stem}-preview.png")),
        )
        .unwrap();
    }
    std::fs::write(managed, corrupt).unwrap();
    let after_deliberate_corruption = inventory(dir);
    for renderer in [warm, &fresh] {
        let error = renderer.render_preview(project, dir, 400).unwrap_err();
        assert_eq!(
            inventory(dir),
            after_deliberate_corruption,
            "hidden/inactive pinned-media failure published bytes or directories"
        );
        assert_eq!(
            error.code,
            opencut_editor_core::ErrorCode::AssetIntegrityFailed
        );
        assert!(!error.retryable);
    }
    std::fs::write(managed, original).unwrap();
    assert_eq!(std::fs::read(managed).unwrap(), original);
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
        "redo did not restore native matte generation"
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
    let output = root.join("exports/mattes.mp4");
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
            "native matte intent SSIM below existing gate"
        );
        pcm_equal(&native.decode(&path, true, None), &base_pcm);
        let probe = native.probe(&path);
        assert_eq!(probe, base_probe, "matte changed timing/audio metadata");
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
fn native_alpha_luma_chain_hidden_offscreen_mask_effect_affine_and_audio_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Independent matte board");
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let provider = rectangle(&core, &id, &track, "#ff0000", (0., 0., 96, 64), 0.5);
    let chain = rectangle(&core, &id, &track, "#ffffff", (0., 0., 96, 64), 0.5);
    let alpha = rectangle(&core, &id, &track, "#ff0000", (4., 4., 16, 16), 1.);
    let luma = rectangle(&core, &id, &track, "#ffffff", (24., 4., 16, 16), 1.);
    let chained = rectangle(&core, &id, &track, "#0000ff", (4., 28., 16, 16), 1.);
    let hidden = rectangle(&core, &id, &track, "#ffffff", (64., 4., 16, 16), 1.);
    let hidden_recipient = rectangle(&core, &id, &track, "#ffffff", (64., 4., 16, 16), 1.);
    let group = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"transform2d":{"position":{"x":64,"y":84,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0.25,"y":0.75},"opacity":0.5}}),
    );
    let transformed = rectangle(&core, &id, &track, "#ff0000", (0., 0., 12, 8), 1.);
    let mut operations = vec![
        op(json!({"operation":"update_item","itemId":provider,"matteOnly":true})),
        op(
            json!({"operation":"update_item","itemId":chain,"matteOnly":true,"matte":{"sourceId":provider,"channel":"alpha"}}),
        ),
        op(
            json!({"operation":"update_item","itemId":alpha,"matte":{"sourceId":provider,"channel":"alpha"}}),
        ),
        op(
            json!({"operation":"update_item","itemId":luma,"matte":{"sourceId":provider,"channel":"luma"}}),
        ),
        op(
            json!({"operation":"update_item","itemId":chained,"matte":{"sourceId":chain,"channel":"alpha"}}),
        ),
        op(json!({"operation":"set_item_visibility","itemId":hidden,"hidden":true})),
        op(json!({"operation":"update_item","itemId":hidden,"matteOnly":true})),
        op(
            json!({"operation":"update_item","itemId":hidden_recipient,"matte":{"sourceId":hidden,"channel":"alpha"}}),
        ),
    ];
    let mut mask: Value =
        serde_json::from_str::<Value>(include_str!("../../../contracts/mask-models-v1.json"))
            .unwrap()["cases"][0]["value"]
            .clone();
    mask["source"]["paint"] = json!({"type":"solid","color":{"r":1,"g":1,"b":1,"a":0.5}});
    mask["transform"] = json!({"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1});
    mask["featherPx"] = json!(0);
    mask["expansionPx"] = json!(0);
    mask["source"]["path"]["commands"] = json!([{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":12,"y":0}},{"type":"lineTo","to":{"x":12,"y":8}},{"type":"lineTo","to":{"x":0,"y":8}},{"type":"close"}]);
    operations.push(op(json!({"operation":"item_set_parent","itemId":transformed,"parent":{"scope":"root","id":group}})));
    operations.push(op(json!({"operation":"update_item","itemId":transformed,"transform2d":{"position":{"x":4,"y":4,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":90,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0.25,"y":0.75},"opacity":1},"masks":[mask],"effects":[{"type":"color_tint","id":"green","color":{"r":0,"g":1,"b":0,"a":1}}],"matte":{"sourceId":provider,"channel":"luma"}})));
    for (x, y, rx, ry) in [(110., 4., 64., 28.), (4., 90., 64., 48.)] {
        let off = rectangle(&core, &id, &track, "#ffffff", (x, y, 16, 12), 1.);
        let recipient = rectangle(&core, &id, &track, "#ffffff", (rx, ry, 16, 12), 1.);
        operations.push(op(
            json!({"operation":"update_item","itemId":off,"matteOnly":true}),
        ));
        operations.push(op(json!({"operation":"update_item","itemId":recipient,"matte":{"sourceId":off,"channel":"alpha"}})));
        let off_recipient = rectangle(&core, &id, &track, "#ffffff", (x, y, 16, 12), 1.);
        operations.push(op(json!({"operation":"update_item","itemId":off_recipient,"matte":{"sourceId":provider,"channel":"alpha"}})));
    }
    let baseline = core.get_project(&id).unwrap();
    let before = serde_json::to_vec(&baseline).unwrap();
    let draft = core
        .create_draft(&id, baseline.revision, operations.clone(), None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert!(
        candidate.find_item(&hidden).unwrap().hidden(),
        "zero provider fixture must actually be hidden"
    );
    assert_eq!(
        candidate
            .find_item(&transformed)
            .unwrap()
            .visual_properties()
            .parent
            .as_ref()
            .unwrap()
            .id,
        group,
        "noncentral ancestor fixture must actually attach its parent"
    );
    assert_eq!(
        serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap(),
        before,
        "draft changed authoritative state"
    );
    core.edit_batch(&id, baseline.revision, operations).unwrap();
    let current = core.get_project(&id).unwrap();
    // Premul red provider=.5; luma=.2126*.5=.1063; chained alpha=.5*.5=.25.
    // Local half-painted mask→green tint→ancestor gain.5→provider luma yields .026575.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[
            (12, 12, [0.5, 0., 0.]),
            (32, 12, [0.1063; 3]),
            (12, 36, [0., 0., 0.25]),
            (46, 43, [0., 0.026575, 0.]),
            (72, 12, [0.; 3]),
            (72, 32, [0.; 3]),
            (72, 52, [0.; 3]),
        ],
        &[
            (4, 4, 20, 20, [0.5, 0., 0.]),
            (24, 4, 40, 20, [0.1063; 3]),
            (4, 28, 20, 44, [0., 0., 0.25]),
            (42, 37, 50, 49, [0., 0.026575, 0.]),
        ],
    );
}

fn held_opacity(values: &[(u64, f64)]) -> Value {
    json!([{"property":"transform.opacity","keyframes":values.iter().map(|(time,value)|json!({"timeMs":time,"value":{"type":"scalar","value":value},"curve":"hold"})).collect::<Vec<_>>()}])
}

#[test]
fn native_pinned_text_provider_real_fitting_and_glyph_interior_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Pinned text matte fitting");
    let media = root.path().join("media");
    let font_bytes = std::fs::read(&native.font).unwrap();
    assert_eq!(
        font_bytes.as_slice(),
        include_bytes!("fixtures/fonts/DejaVuSans.ttf"),
        "the glyph oracle requires the actual bundled pinned regular face"
    );

    // This independent outline witness does not shape or rasterize the subject,
    // search its pixels, or reuse any production measurement result.
    #[derive(Default)]
    struct BlockOutline(Vec<(f32, f32)>);
    impl ttf_parser::OutlineBuilder for BlockOutline {
        fn move_to(&mut self, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn line_to(&mut self, x: f32, y: f32) {
            self.0.push((x, y));
        }
        fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
            panic!("pinned block glyph must have only straight edges");
        }
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
            panic!("pinned block glyph must have only straight edges");
        }
        fn close(&mut self) {}
    }
    let face = ttf_parser::Face::parse(&font_bytes, 0).unwrap();
    assert_eq!(face.units_per_em(), 2048);
    assert_eq!(
        (face.ascender(), face.descender(), face.line_gap()),
        (1901, -483, 0)
    );
    let glyph = face.glyph_index('\u{2588}').unwrap();
    assert_eq!(face.glyph_hor_advance(glyph), Some(1575));
    let mut outline = BlockOutline::default();
    face.outline_glyph(glyph, &mut outline).unwrap();
    assert_eq!(
        outline.0,
        [
            (-20., -512.),
            (-20., 1921.),
            (1595., 1921.),
            (1595., -512.),
            (-20., -512.),
        ]
    );
    let fits = |size: f64| 1575. * size / 2048. <= 30. && (1901. + 483.) * size / 2048. <= 44.;
    assert!(fits(37.));
    assert!(!fits(38.));
    // At selected size37, the fixed straight-edged glyph becomes a rectangle
    // [0,1615*37/2048] x [0,2433*37/2048] after min-origin correction.
    // Global pixel center(20.5,28.5), with authored translation(8,8), is
    // more than eight pixels from every boundary: coverage is independently1.
    let interior = (20.5 - 8., 28.5 - 8.);
    let glyph_extent = (1615. * 37. / 2048., 2433. * 37. / 2048.);
    assert!(interior.0 > 8. && glyph_extent.0 - interior.0 > 8.);
    assert!(interior.1 > 8. && glyph_extent.1 - interior.1 > 8.);

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
    add_audio(&core, &id, &media.join("tone.wav"));
    let provider = edit(
        &core,
        &id,
        json!({"operation":"add_text","trackId":track,"text":"\u{2588}",
            "fontFamily":"DejaVu Sans","fontSize":40,"color":"#ffffff",
            "startMs":0,"durationMs":1000,
            "transform":{"positionX":8,"positionY":8,"scale":1,"opacity":0.5}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider,"style":{
            "backgroundOpacity":0,"alignment":"left",
            "layout":{"bounds":{"widthPx":30,"heightPx":44},
                "fit":"shrink","wrap":"none","verticalAlignment":"top"}}}),
    );
    let recipient = rectangle(&core, &id, &track, "#00ff00", (0., 0., 96, 64), 1.);
    let baseline = core.get_project(&id).unwrap();
    let stored = serde_json::to_value(baseline.find_item(&provider).unwrap()).unwrap();
    assert_eq!(stored["fontSize"], 40);
    assert_eq!(stored["style"]["backgroundOpacity"], 0.0);
    assert_eq!(stored["style"]["layout"]["fit"], "shrink");
    assert_eq!(
        stored["style"]["layout"]["bounds"],
        json!({"widthPx":30.0,"heightPx":44.0})
    );
    use sha2::{Digest, Sha256};
    let regular_hash = format!("{:x}", Sha256::digest(&font_bytes));
    assert_eq!(stored["fontBinding"]["regular"], regular_hash);
    assert_eq!(baseline.fonts.len(), 4);
    let pinned = baseline.fonts.get(&regular_hash).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    assert_eq!(
        std::fs::read(dir.join(&pinned.relative_path)).unwrap(),
        font_bytes
    );
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::remove_file(media.join(name)).unwrap();
    }

    let operations = vec![
        op(json!({"operation":"update_item","itemId":provider,"matteOnly":true})),
        op(json!({"operation":"update_item","itemId":recipient,
            "matte":{"sourceId":provider,"channel":"alpha"}})),
    ];
    let before = serde_json::to_vec(&baseline).unwrap();
    let draft = core
        .create_draft(&id, baseline.revision, operations.clone(), None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(
        serde_json::to_vec(&core.get_project(&id).unwrap()).unwrap(),
        before
    );
    core.edit_batch(&id, baseline.revision, operations).unwrap();
    let current = core.get_project(&id).unwrap();
    // Transparent provider glyph coverage1 * inherited alpha.5; green opaque
    // recipient therefore has independent linear premul RGB(0,.5,0).
    // Existing repeated same-renderer frame/candidate/reopen/range/export paths
    // exercise real measured-cache reuse and final shaped-layout cloning.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[(20, 28, [0., 0.5, 0.]), (60, 32, [0.; 3])],
        &[(8, 8, 30, 44, [0., 0.5, 0.])],
    );
}

#[test]
fn native_repeater_copy_shutters_before_aggregate_and_nested_recipient_shutter_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Overlapping copy shutter witness");
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let provider = edit(
        &core,
        &id,
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
        "geometry":{"type":"rectangle","width":80,"height":32},"fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider,"transform":{"positionX":4,"positionY":4,"scale":1,"opacity":1}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":provider,"animationChannels":held_opacity(&[(0,0.),(300,0.5),(400,0.)])}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider,"motionBlur":{"shutterAngleDeg":180,"sampleCount":2}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"add_repeater","trackId":track,"startMs":0,"durationMs":1000,"repeater":{"source":{"scope":"root","id":provider},"copies":1,"timeOffsetMs":100,"opacityOffset":0,"transformOffset":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}),
    );
    let direct = rectangle(&core, &id, &track, "#ffffff", (4., 4., 32, 32), 1.);
    let nested = rectangle(&core, &id, &track, "#ffffff", (44., 4., 32, 32), 1.);
    let baseline = core.get_project(&id).unwrap();
    let operations = vec![
        op(json!({"operation":"update_item","itemId":provider,"matteOnly":true})),
        op(
            json!({"operation":"update_item","itemId":direct,"matte":{"sourceId":provider,"channel":"alpha"}}),
        ),
        op(
            json!({"operation":"update_item","itemId":nested,"matte":{"sourceId":provider,"channel":"alpha"},"motionBlur":{"shutterAngleDeg":180,"sampleCount":2}}),
        ),
    ];
    let draft = core
        .create_draft(&id, baseline.revision, operations.clone(), None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit_batch(&id, baseline.revision, operations).unwrap();
    let current = core.get_project(&id).unwrap();
    // At400: canonical midpoint root times387,412. Base alpha [.5,0],
    // The positive100ms offset delays the copy, sampling root−100; its alpha
    // is[0,.5], so isolated copy means .25,.25 and aggregate
    // .25+.25*(1-.25)=.4375 (aggregate-before-average would be .5).
    // Recipient's own387 sample asks provider midpoints374,399: aggregate .5.
    // Its412 sample asks399,424: aggregate .4375. Own average=.46875.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[
            (20, 20, [0.4375; 3]),
            (60, 20, [0.46875; 3]),
            (90, 50, [0.; 3]),
        ],
        &[(4, 4, 36, 36, [0.4375; 3]), (44, 4, 76, 36, [0.46875; 3])],
    );
}

#[test]
fn native_component_local_mattes_bind_two_occurrences_and_original_clocks_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Independent component matte occurrences");
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let seed = rectangle(&core, &id, &track, "#ffffff", (4., 4., 24, 24), 1.);
    let mut provider =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&seed).unwrap()).unwrap();
    provider["id"] = json!("provider");
    provider["matteOnly"] = json!(true);
    provider["animationChannels"] = json!([{"property":"transform.opacity","keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.25},"curve":"hold"},
        {"timeMs":124,"value":{"type":"scalar","value":0.25},"curve":"linear"},
        {"timeMs":126,"value":{"type":"scalar","value":0.75},"curve":"hold"},
        {"timeMs":250,"value":{"type":"scalar","value":0.25},"curve":"hold"}],
        "loop":{"mode":"repeat","iterations":"infinite"}}]);
    let mut recipient = provider.clone();
    recipient["id"] = json!("recipient");
    recipient.as_object_mut().unwrap().remove("matteOnly");
    recipient
        .as_object_mut()
        .unwrap()
        .remove("animationChannels");
    recipient["matte"] = json!({"sourceId":"provider","channel":"alpha"});
    recipient["stackOrder"] = json!(1);
    let tracks = json!([{"id":"local","name":"Local","trackType":"overlay","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":[provider,recipient]}]);
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Local matte clocks","width":32,"height":32,"durationMs":1000,"tracks":tracks,"slots":[]}),
    );
    edit(&core, &id, json!({"operation":"delete_item","itemId":seed}));
    let parent = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":1000,"staggerMs":100,
        "transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"anchor":{"x":0,"y":0},"opacity":1}}),
    );
    let a = edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"trimStartMs":0,"durationMs":1000,"timeScale":1,"slotValues":{}}),
    );
    let b = edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":100,"trimStartMs":0,"durationMs":900,"timeScale":0.5,"slotValues":{}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":a,"transform":{"positionX":4,"positionY":4,"scale":1,"opacity":1}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":b,"transform":{"positionX":44,"positionY":4,"scale":1,"opacity":1}}),
    );
    for item in [&a, &b] {
        edit(
            &core,
            &id,
            json!({"operation":"item_set_parent","itemId":item,"parent":{"scope":"root","id":parent}}),
        );
        assert_eq!(
            core.get_project(&id)
                .unwrap()
                .find_item(item)
                .unwrap()
                .visual_properties()
                .parent
                .as_ref()
                .unwrap()
                .id,
            parent,
            "staggered component fixture must actually attach its parent"
        );
    }
    let baseline = core.get_project(&id).unwrap();
    let mut new_tracks = tracks.clone();
    new_tracks[0]["items"][1]["color"] = json!("#00ff00");
    let operation = op(
        json!({"operation":"component_update","componentId":component,"name":"Local matte clocks","width":32,"height":32,"durationMs":1000,"tracks":new_tracks,"slots":[]}),
    );
    let draft = core
        .create_draft(&id, baseline.revision, vec![operation.clone()], None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit(&id, baseline.revision, operation).unwrap();
    let current = core.get_project(&id).unwrap();
    // Same local IDs, different instance paths: root400 maps to local400
    // then repeat-loop150 versus
    // (400-100 instance start-100 sibling stagger)*.5=100. Provider alpha is
    // .75 versus .25, not pooled .8125. Omitting stagger maps the second to150
    // and wrongly crosses its124..126 transition. Omitting repeat makes the
    // first sample stay at the final250ms value.25 instead of.75.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&baseline, &candidate, &current),
        &[
            (20, 20, [0., 0.75, 0.]),
            (60, 20, [0., 0.25, 0.]),
            (90, 50, [0.; 3]),
        ],
        &[
            (8, 8, 32, 32, [0., 0.75, 0.]),
            (48, 8, 72, 32, [0., 0.25, 0.]),
        ],
    );
    // A separate actual native frame probes a genuinely fractional inherited
    // clock: (451-100-100)*.5=125.5, halfway between125 and126. The124..126
    // linear segment yields .25+.5*(1.5/2)=.625; truncating to125 gives.5.
    let (renderer, captured) = native.capturing_renderer(root.path());
    let frame = renderer
        .render_preview(&current, &core.paths().project_dir(&id).unwrap(), 451)
        .unwrap();
    let points = [(20, 20, [0., 0.75, 0.]), (60, 20, [0., 0.625, 0.])];
    assert_pixels(&raw_pam(&captured), &points);
    let converted = native.authored_plate(
        root.path(),
        &[
            (8, 8, 32, 32, [0., 0.75, 0.]),
            (48, 8, 72, 32, [0., 0.625, 0.]),
        ],
    );
    assert_converted(
        &native.decode(
            &core
                .paths()
                .project_dir(&id)
                .unwrap()
                .join(frame.relative_path),
            false,
            None,
        ),
        &converted,
        &points,
    );
}

fn normalized_half_mask(width: f64, height: f64) -> Value {
    json!({"id":"half","source":{"type":"path","path":{"fillRule":"nonzero","commands":[
        {"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":width/2.,"y":0}},
        {"type":"lineTo","to":{"x":width/2.,"y":height}},{"type":"lineTo","to":{"x":0,"y":height}},{"type":"close"}]},
        "paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":0.5}}},"channel":"alpha","operation":"add","inverted":false,
        "transform":{"position":{"x":0.5,"y":0,"unit":"normalized"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1},"featherPx":0,"expansionPx":0})
}

#[test]
fn native_asymmetric_crop_normalized_masks_and_matte_only_video_preserve_audio_all_intents() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, track) = setup("Cropped provider local mask basis");
    let media = root.path().join("media");
    let image = media.join("asymmetric.pam");
    let mut pam =
        b"P7\nWIDTH 96\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
    for _ in 0..64 {
        for x in 0..96 {
            pam.extend(if x < 32 {
                [255, 0, 0, 255]
            } else {
                [0, 255, 0, 255]
            });
        }
    }
    std::fs::write(&image, pam).unwrap();
    let png = media.join("asymmetric.png");
    let conversion = Command::new(&native.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(&image)
        .args(["-frames:v", "1", "-y"])
        .arg(&png)
        .output()
        .unwrap();
    assert!(
        conversion.status.success(),
        "{}",
        String::from_utf8_lossy(&conversion.stderr)
    );
    let wav = media.join("tone.wav");
    write_tone(&wav);
    let video = media.join("provider.mkv");
    let encoding = Command::new(&native.ffmpeg)
        .args(["-v", "error", "-loop", "1", "-framerate", "10", "-i"])
        .arg(&png)
        .arg("-i")
        .arg(&wav)
        .args([
            "-t",
            "1",
            "-c:v",
            "ffv1",
            "-pix_fmt",
            "bgr0",
            "-c:a",
            "pcm_s16le",
            "-y",
        ])
        .arg(&video)
        .output()
        .unwrap();
    assert!(
        encoding.status.success(),
        "{}",
        String::from_utf8_lossy(&encoding.stderr)
    );
    let asset = core
        .import_asset(
            &id,
            core.get_project(&id).unwrap().revision,
            &video,
            MediaType::Video,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                video_width: Some(96),
                video_height: Some(64),
                ..Default::default()
            },
        )
        .unwrap()
        .changed_ids[0]
        .clone();
    let video_track = core.get_project(&id).unwrap().tracks[0].id.clone();
    let provider = edit(
        &core,
        &id,
        json!({"operation":"add_media","trackId":video_track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":1000}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.5},"crop":{"x":1.0/3.0,"y":0,"width":2.0/3.0,"height":1}}),
    );
    let recipient = rectangle(&core, &id, &track, "#ffffff", (24., 24., 48, 16), 1.);
    let baseline = core.get_project(&id).unwrap();
    let dir = core.paths().project_dir(&id).unwrap();
    let initial = native
        .renderer()
        .render_preview(&baseline, &dir, 400)
        .unwrap();
    let initial_pixels = native.decode(&dir.join(initial.relative_path), false, None);
    // Explicit default fields must preserve the original no-matte route exactly.
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":recipient,"matte":null,"matteOnly":false}),
    );
    let defaults = core.get_project(&id).unwrap();
    let unchanged = native
        .renderer()
        .render_preview(&defaults, &dir, 400)
        .unwrap();
    assert_eq!(
        native.decode(&dir.join(unchanged.relative_path), false, None),
        initial_pixels,
        "explicit matte defaults changed legacy pixels"
    );
    let operations = vec![
        op(
            json!({"operation":"update_item","itemId":provider,"matteOnly":true,"masks":[normalized_half_mask(96.,64.)]}),
        ),
        op(
            json!({"operation":"update_item","itemId":recipient,"masks":[normalized_half_mask(48.,16.)],"matte":{"sourceId":provider,"channel":"luma"}}),
        ),
    ];
    let draft = core
        .create_draft(&id, defaults.revision, operations.clone(), None)
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    core.edit_batch(&id, defaults.revision, operations).unwrap();
    let current = core.get_project(&id).unwrap();
    // Crop discards the red third. Post-crop logical96x64 provider mask shifts
    // 48px and halves alpha; provider gain.5 yields premul green.25. Recipient
    // own48x16 mask shifts24px, halves white: .5*(.7152*.25)=.0894.
    // The matte-only VIDEO is the sole audio source; its waveform stays active.
    all_intents(
        &native,
        root.path(),
        &core,
        &id,
        (&defaults, &candidate, &current),
        &[(60, 32, [0.0894; 3]), (36, 32, [0.; 3]), (80, 12, [0.; 3])],
        &[(48, 24, 72, 40, [0.0894; 3])],
    );
    // Warm the SAME renderer on the SAME immutable latest project, then corrupt
    // the actual managed provider-only asset. Cache hits must still run integrity
    // checks before any render workspace/artifact publication.
    let latest = core.get_project(&id).unwrap();
    assert!(
        latest
            .find_item(&provider)
            .unwrap()
            .visual_properties()
            .matte_only
    );
    let (renderer, captured) = native.capturing_renderer(root.path());
    let mut warm_pixels = None;
    let mut warm_raw = None;
    for _ in 0..2 {
        let frame = renderer.render_preview(&latest, &dir, 400).unwrap();
        let pixels = native.decode(&dir.join(frame.relative_path), false, None);
        let raw = raw_pam(&captured);
        assert_pixels(&raw, &[(60, 32, [0.0894; 3])]);
        if let Some(expected) = &warm_pixels {
            assert_eq!(&pixels, expected, "warm provider-only cache changed pixels");
        } else {
            warm_pixels = Some(pixels);
        }
        if let Some(expected) = &warm_raw {
            assert_eq!(
                &raw, expected,
                "warm provider-only cache changed prepared scene"
            );
        } else {
            warm_raw = Some(raw);
        }
    }
    let managed_asset = latest
        .assets
        .iter()
        .find(|entry| entry.id == asset)
        .unwrap();
    let pinned_hash = managed_asset
        .content_hash
        .as_ref()
        .expect("integrity witness must use genuinely hash-pinned managed media");
    assert_eq!(pinned_hash.algorithm, "sha256");
    assert_eq!(pinned_hash.digest.len(), 64);
    let managed = dir.join(&managed_asset.project_relative_path);
    let original = std::fs::read(&managed).unwrap();
    let mut corrupt = original.clone();
    assert_eq!(managed_asset.size_bytes, Some(corrupt.len() as u64));
    assert!(
        corrupt.len() > 64,
        "managed media must have payload beyond its header prefix"
    );
    assert_eq!(
        &corrupt[..4],
        &[0x1a, 0x45, 0xdf, 0xa3],
        "fixture must retain a valid Matroska signature"
    );
    let header_prefix = corrupt[..64].to_vec();
    // Preserve the Matroska format signature so this control specifically
    // exercises managed content-integrity rather than unsupported-format sniffing.
    let payload_end = corrupt.len() - 1;
    corrupt[payload_end] ^= 0xff;
    assert_eq!(
        &corrupt[..64],
        &header_prefix,
        "integrity control must preserve the format header prefix"
    );
    std::fs::write(&managed, &corrupt).unwrap();
    let after_deliberate_corruption = inventory(&dir);
    let error = renderer.render_preview(&latest, &dir, 400).unwrap_err();
    assert_eq!(
        inventory(&dir),
        after_deliberate_corruption,
        "warm cache bypassed integrity or published failed-render bytes"
    );
    assert_eq!(
        error.code,
        opencut_editor_core::ErrorCode::AssetIntegrityFailed
    );
    assert!(!error.retryable);
    // A genuinely pinned hidden provider must still pass integrity admission,
    // even though it contributes no plane. Restore before public authored edits.
    std::fs::write(&managed, &original).unwrap();
    edit(
        &core,
        &id,
        json!({"operation":"set_item_visibility","itemId":provider,"hidden":true}),
    );
    let hidden_project = core.get_project(&id).unwrap();
    assert!(hidden_project.find_item(&provider).unwrap().hidden());
    assert_eq!(
        hidden_project
            .assets
            .iter()
            .find(|entry| entry.id == asset)
            .unwrap()
            .content_hash
            .as_ref()
            .unwrap(),
        pinned_hash
    );
    zero_provider_integrity(
        &native,
        root.path(),
        &dir,
        &hidden_project,
        &renderer,
        &captured,
        (&managed, &original, &corrupt, true),
    );

    // Component-local inactivity is evaluated in its actual retimed occurrence:
    // root400 maps to300+(400-100)*.5=450, before provider local start600.
    // The local recipient is active, references that same local provider, and
    // cannot fall back to the similarly authored root media or identity coverage.
    let mut local_provider =
        serde_json::to_value(hidden_project.find_item(&provider).unwrap()).unwrap();
    local_provider["id"] = json!("local-video");
    local_provider["hidden"] = json!(false);
    local_provider["startMs"] = json!(600);
    local_provider["durationMs"] = json!(400);
    let mut local_recipient =
        serde_json::to_value(hidden_project.find_item(&recipient).unwrap()).unwrap();
    local_recipient["id"] = json!("local-recipient");
    local_recipient["matte"] = json!({"sourceId":"local-video","channel":"luma"});
    let local_tracks = json!([
        {"id":"video","name":"Video","trackType":"video","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":[local_provider]},
        {"id":"overlay","name":"Overlay","trackType":"overlay","locked":false,"hidden":false,"muted":false,"audioRole":"unassigned","ducking":null,"items":[local_recipient]}]);
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Inactive pinned provider","width":96,"height":64,"durationMs":1000,"tracks":local_tracks,"slots":[]}),
    );
    let instance = edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":100,"trimStartMs":300,"durationMs":900,"timeScale":0.5,"slotValues":{}}),
    );
    let contextual_project = core.get_project(&id).unwrap();
    let stored = contextual_project
        .components
        .iter()
        .find(|entry| entry.id == component)
        .unwrap();
    assert_eq!(stored.tracks[0].items[0].id(), "local-video");
    assert_eq!(stored.tracks[0].items[0].start_ms(), 600);
    assert!(!stored.tracks[0].items[0].hidden());
    assert_eq!(
        serde_json::to_value(&stored.tracks[0].items[0]).unwrap()["assetId"],
        asset
    );
    assert_eq!(
        serde_json::to_value(contextual_project.find_item(&instance).unwrap()).unwrap()["trimStartMs"],
        300
    );
    assert_eq!(
        contextual_project
            .assets
            .iter()
            .find(|entry| entry.id == asset)
            .unwrap()
            .content_hash
            .as_ref()
            .unwrap(),
        pinned_hash
    );
    zero_provider_integrity(
        &native,
        root.path(),
        &dir,
        &contextual_project,
        &renderer,
        &captured,
        (&managed, &original, &corrupt, true),
    );
    // Isolate UNINSTANTIATED retained-definition admission: remove every root
    // media and instance reference while keeping the same component-local
    // provider/recipient graph and hash-pinned asset. A default black rectangle
    // keeps a real1000ms timeline without inventing an evaluated matte plane.
    core.edit_batch(&id,contextual_project.revision,vec![
        op(json!({"operation":"delete_item","itemId":instance})),
        op(json!({"operation":"delete_item","itemId":recipient})),
        op(json!({"operation":"delete_item","itemId":provider})),
        op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":96,"height":64,"color":"#000000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})),
    ]).unwrap();
    let unused_project = core.get_project(&id).unwrap();
    for item in unused_project.tracks.iter().flat_map(|track| &track.items) {
        assert!(!matches!(
            item,
            opencut_editor_core::TimelineItem::Media(_)
                | opencut_editor_core::TimelineItem::ComponentInstance(_)
        ));
        assert!(item.visual_properties().matte.is_none());
        assert!(!item.visual_properties().matte_only);
    }
    let retained = unused_project
        .components
        .iter()
        .find(|entry| entry.id == component)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&retained.tracks[0].items[0]).unwrap()["assetId"],
        asset
    );
    assert_eq!(
        retained.tracks[1].items[0]
            .visual_properties()
            .matte
            .as_ref()
            .unwrap()
            .source_id,
        "local-video"
    );
    assert_eq!(
        unused_project
            .assets
            .iter()
            .find(|entry| entry.id == asset)
            .unwrap()
            .content_hash
            .as_ref()
            .unwrap(),
        pinned_hash
    );
    zero_provider_integrity(
        &native,
        root.path(),
        &dir,
        &unused_project,
        &renderer,
        &captured,
        (&managed, &original, &corrupt, false),
    );
}
