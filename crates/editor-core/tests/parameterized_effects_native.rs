//! Independent parameterized-effect plates and actual public render lifecycle.
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
                    .expect("native parameterized-effect oracle requires bundled font")
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
                    "required native parameterized-effect tools missing"
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
    fn authored_plate(&self, root: &Path, pixels: &[[u8; 4]]) -> Vec<u8> {
        assert_eq!(pixels.len(), 64 * 64);
        let path = root.join("independently-authored.pam");
        let mut bytes =
            b"P7\nWIDTH 64\nHEIGHT 64\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
        for pixel in pixels {
            bytes.extend(pixel);
        }
        std::fs::write(&path, bytes).unwrap();
        let output = root.join("independently-converted.png");
        let result=Command::new(&self.ffmpeg).args(["-v","error","-nostdin","-f","lavfi","-i","color=c=black:s=64x64:r=10:d=1"])
            .args(["-loop","1","-i"]).arg(&path).args(["-filter_complex_threads","1","-filter_complex",
            "[0:v]format=yuv420p[base0];[1:v]fps=10,settb=AVTB,setpts=PTS-STARTPTS+0/TB,format=rgba[plate];[base0][plate]overlay=format=auto:x=0:y=0:eof_action=pass[composed];[composed]scale=64:64:force_original_aspect_ratio=decrease,pad=64:64:(ow-iw)/2:(oh-ih)/2,format=yuv420p[video]","-map","[video]","-frames:v","1","-y"]).arg(&output).output().unwrap();
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
fn encoded(linear: f64) -> u8 {
    let value = if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (255.0 * value).round() as u8
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
    assert_eq!(a, b, "effect order changes must preserve exact decoded PCM");
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
    assert!(rms <= 0.0001, "effect order changed audio RMS{rms}");
}
fn write_tone(path: &Path) {
    let count = 38400_u32;
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
                duration_ms: Some(800),
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
        json!({"operation":"add_media","trackId":track,"assetId":asset,"sourceInMs":0,"startMs":0,"durationMs":800}),
    );
}

fn fixture() -> Value {
    let native: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    let w = &native["nativeWitness"];
    json!({"project":w["project"],"source":w["source"],"orders":{"shadeThenWash":w["clockOrders"]["gradeThenTint"],"washThenShade":w["clockOrders"]["tintThenGrade"]},"heldSamples":w["heldSamples"]})
}

fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
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
            "Independent ordered effects",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let source = fixture()["source"].clone();
    let item = edit(
        &core,
        &id,
        json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":800,"geometry":source["geometry"],"fill":source["fill"],"stroke":source["stroke"]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":item,"transform2d":source["transform2d"]}),
    );
    (root, core, id, item)
}

// Independently count the documented4x4 coverage samples. Rounded diagonal
// scanline spans cover3+2+1+0 samples,16 alpha units each:96/255.
fn coverage(x: i32, y: i32) -> f64 {
    if !(0..32).contains(&x) || !(0..24).contains(&y) {
        return 0.0;
    }
    match (y - x).cmp(&16) {
        std::cmp::Ordering::Less => 1.0,
        std::cmp::Ordering::Equal => 96.0 / 255.0,
        std::cmp::Ordering::Greater => 0.0,
    }
}
fn independent_plate(stack: &Value) -> Vec<[u8; 4]> {
    const SIDE: usize = 48;
    const ORIGIN: i32 = -8;
    let index = |x: usize, y: usize| y * SIDE + x;
    let mut source = vec![[0.0_f64; 4]; SIDE * SIDE];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let a = coverage(x as i32 + ORIGIN, y as i32 + ORIGIN);
            source[index(x, y)] = [a, 0.0, 0.0, a];
        }
    }
    for effect in stack.as_array().unwrap() {
        match effect["type"].as_str().unwrap() {
            "vignette" => {
                let amount = effect["amount"].as_f64().unwrap();
                for y in 0..SIDE {
                    for x in 0..SIDE {
                        let u = 2.0 * (x as f64 + f64::from(ORIGIN) + 0.5) / 32.0 - 1.0;
                        let v = 2.0 * (y as f64 + f64::from(ORIGIN) + 0.5) / 24.0 - 1.0;
                        let factor = 1.0 - amount * ((u * u + v * v) / 2.0).clamp(0.0, 1.0);
                        for channel in source[index(x, y)].iter_mut().take(3) {
                            *channel *= factor;
                        }
                    }
                }
            }
            "color_tint" => {
                let color = &effect["color"];
                let m = color["a"].as_f64().unwrap();
                let transfer = |v: f64| {
                    if v <= 0.04045 {
                        v / 12.92
                    } else {
                        ((v + 0.055) / 1.055).powf(2.4)
                    }
                };
                let target = ["r", "g", "b"].map(|key| transfer(color[key].as_f64().unwrap()));
                for p in &mut source {
                    for c in 0..3 {
                        p[c] = p[c] * (1.0 - m) + p[3] * m * target[c];
                    }
                }
            }
            "glow" => {
                assert_eq!(effect["radiusPx"].as_f64(), Some(1.0));
                let weights: Vec<f64> = (-3..=3).map(|x| (-0.5 * f64::from(x * x)).exp()).collect();
                let sum: f64 = weights.iter().sum();
                let weights: Vec<f64> = weights.iter().map(|w| w / sum).collect();
                let mut horizontal = vec![0.0; SIDE * SIDE];
                let mut blurred = vec![0.0; SIDE * SIDE];
                for y in 0..SIDE {
                    for x in 0..SIDE {
                        for (tap, w) in weights.iter().enumerate() {
                            let sx = x as i32 + tap as i32 - 3;
                            if (0..SIDE as i32).contains(&sx) {
                                horizontal[index(x, y)] += source[index(sx as usize, y)][3] * w;
                            }
                        }
                    }
                }
                for y in 0..SIDE {
                    for x in 0..SIDE {
                        for (tap, w) in weights.iter().enumerate() {
                            let sy = y as i32 + tap as i32 - 3;
                            if (0..SIDE as i32).contains(&sy) {
                                blurred[index(x, y)] += horizontal[index(x, sy as usize)] * w;
                            }
                        }
                    }
                }
                for (p, a) in source.iter_mut().zip(blurred) {
                    let behind = a * 0.6;
                    let uncovered = 1.0 - p[3];
                    p[2] += behind * uncovered;
                    p[3] += behind * uncovered;
                }
            }
            "color_adjustment" => {
                let gain = effect["exposureStops"].as_f64().unwrap().exp2();
                let contrast = effect["contrast"].as_f64().unwrap();
                let saturation = effect["saturation"].as_f64().unwrap();
                for p in &mut source {
                    if p[3] == 0.0 {
                        p[..3].fill(0.0);
                        continue;
                    }
                    let channels =
                        [p[0], p[1], p[2]].map(|v| (v / p[3] * gain - 0.18) * contrast + 0.18);
                    let luma = channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
                    for c in 0..3 {
                        p[c] = ((1.0 - saturation) * luma + saturation * channels[c])
                            .clamp(0.0, 1.0)
                            * p[3];
                    }
                }
            }
            "gaussian_blur" => {
                if effect["radiusPx"].as_f64() == Some(0.0) {
                    continue;
                }
                assert_eq!(effect["radiusPx"].as_f64(), Some(1.0));
                let weights: Vec<_> = (-3..=3).map(|x| (-0.5 * f64::from(x * x)).exp()).collect();
                let total: f64 = weights.iter().sum();
                let weights: Vec<_> = weights.into_iter().map(|v| v / total).collect();
                let mut horizontal = vec![[0.0; 4]; SIDE * SIDE];
                let mut vertical = vec![[0.0; 4]; SIDE * SIDE];
                for y in 0..SIDE {
                    for x in 0..SIDE {
                        for (tap, weight) in weights.iter().enumerate() {
                            let sample = x as i32 + tap as i32 - 3;
                            if (0..SIDE as i32).contains(&sample) {
                                for c in 0..4 {
                                    horizontal[index(x, y)][c] +=
                                        source[index(sample as usize, y)][c] * weight;
                                }
                            }
                        }
                    }
                }
                for y in 0..SIDE {
                    for x in 0..SIDE {
                        for (tap, weight) in weights.iter().enumerate() {
                            let sample = y as i32 + tap as i32 - 3;
                            if (0..SIDE as i32).contains(&sample) {
                                for c in 0..4 {
                                    vertical[index(x, y)][c] +=
                                        horizontal[index(x, sample as usize)][c] * weight;
                                }
                            }
                        }
                    }
                }
                source = vertical;
            }
            other => panic!("Independent fixture does not authorize {other}"),
        }
    }
    let mut plate = vec![[0, 0, 0, 255]; 64 * 64];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let local_x = x as i32 + ORIGIN;
            let local_y = y as i32 + ORIGIN;
            // Authored quarter turn: world center=(38-local_y,12+local_x).
            let wx = 37 - local_y;
            let wy = 12 + local_x;
            if (0..64).contains(&wx) && (0..64).contains(&wy) {
                let p = source[index(x, y)];
                plate[wy as usize * 64 + wx as usize] = [
                    encoded(p[0] * 0.5),
                    encoded(p[1] * 0.5),
                    encoded(p[2] * 0.5),
                    255,
                ];
            }
        }
    }
    plate
}
fn raw_pam(path: &Path) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap();
    let end = bytes.windows(7).position(|b| b == b"ENDHDR\n").unwrap() + 7;
    let header = std::str::from_utf8(&bytes[..end]).unwrap();
    assert!(header.contains("WIDTH 64\nHEIGHT 64\nDEPTH 4\n"));
    bytes[end..].to_vec()
}
fn close_bytes(actual: &[u8], expected: &[u8], label: &str) {
    assert_eq!(actual.len(), expected.len());
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        assert!(
            a.abs_diff(*b) <= 1,
            "{label} byte{i}: actual{a} expected{b}"
        );
    }
}
fn assert_frame(
    native: &Native,
    renderer: &Renderer,
    project: &Project,
    paths: (&Path, &Path, &Path),
    time: u64,
    stack: &Value,
) -> Vec<u8> {
    let (dir, captured, root) = paths;
    let plate = independent_plate(stack);
    let expected = native.authored_plate(root, &plate);
    let frame = renderer.render_preview(project, dir, time).unwrap();
    close_bytes(
        &raw_pam(captured),
        plate.as_flattened(),
        "prepared complete independent plate",
    );
    let actual = native.decode(&dir.join(frame.relative_path), false, None);
    close_bytes(&actual, &expected, "converted independent PNG");
    actual
}

#[test]
fn independent_parameterized_color_identity_final_clamp_order_and_positive_gaussian_plates() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, item) = setup();
    let f: Value = serde_json::from_str(include_str!(
        "../../../contracts/parameterized-effects-v1.json"
    ))
    .unwrap();
    let w = &f["nativeWitness"];
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let mut green = w["orders"]["gradeThenTint"].clone();
    green[1]["color"]["g"] = json!(0.5);
    for stack in [
        json!([w["primary"]["effect"]]),
        json!([w["finalClamp"]["effect"]]),
        json!([w["identity"]]),
        json!([w["gaussianIdentity"], w["identity"]]),
        json!([w["gaussian"], w["primary"]["effect"]]),
        w["orders"]["gradeThenTint"].clone(),
        w["orders"]["tintThenGrade"].clone(),
        green,
    ] {
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":item,"effects":stack}),
        );
        assert_frame(
            &native,
            &renderer,
            &core.get_project(&id).unwrap(),
            (&dir, &captured, root.path()),
            200,
            &stack,
        );
    }
}

#[test]
fn native_parameterized_effect_intents_draft_commit_undo_redo_reopen_preserve_real_audio_and_original_clock()
 {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, item) = setup();
    let f = fixture();
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let a = &f["orders"]["shadeThenWash"];
    let b = &f["orders"]["washThenShade"];
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":item,"effects":a}),
    );
    let channel = json!({"property":"effect.vignette_amount","target":{"kind":"effect","scope":"root","id":"shade"},"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"hold"},{"timeMs":200,"value":{"type":"scalar","value":0.8},"curve":"hold"},{"timeMs":600,"value":{"type":"scalar","value":0.4},"curve":"hold"}
    ]});
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":item,"animationChannels":[channel]}),
    );
    let before = core.get_project(&id).unwrap();
    let revision = before.revision;
    let draft = core
        .create_draft(
            &id,
            revision,
            vec![op(
                json!({"operation":"update_item","itemId":item,"effects":b}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let authoritative =
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap());
    let draft_pixels = assert_frame(
        &native,
        &renderer,
        &candidate,
        (&dir, &captured, root.path()),
        200,
        b,
    );
    assert_eq!(
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap()),
        authoritative
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    core.commit_draft(&id, &draft.id, revision).unwrap();
    let committed = core.get_project(&id).unwrap();
    let mut range_pcm: Option<Vec<u8>> = None;
    let mut export_pcm: Option<Vec<u8>> = None;
    let mut range_probe = None;
    let mut export_probe = None;
    let mut reference_frames = Vec::new();
    for (index, (project, declared)) in [(&before, a), (&candidate, b), (&committed, b)]
        .into_iter()
        .enumerate()
    {
        let mut at200 = None;
        for sample in f["heldSamples"].as_array().unwrap() {
            let time = sample["timeMs"].as_u64().unwrap();
            let mut sampled = declared.clone();
            let shade = sampled
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|effect| effect["id"] == "shade")
                .unwrap();
            shade["amount"] = sample["amount"].clone();
            let actual = assert_frame(
                &native,
                &renderer,
                project,
                (&dir, &captured, root.path()),
                time,
                &sampled,
            );
            if time == 200 {
                at200 = Some(actual);
            }
        }
        let reference = at200.unwrap();
        reference_frames.push(reference.clone());
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 200,
                    end_ms: 800,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let range_path = dir.join(range.relative_path);
        let mut sampled600 = declared.clone();
        sampled600
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|effect| effect["id"] == "shade")
            .unwrap()["amount"] = json!(0.4);
        let expected600 = native.authored_plate(root.path(), &independent_plate(&sampled600));
        assert!(
            ssim(
                &expected600,
                &native.decode(&range_path, false, Some("0.4"))
            ) >= 0.99,
            "range original600ms held clock"
        );
        let range_pixels = native.decode(&range_path, false, Some("0"));
        assert!(
            ssim(&reference, &range_pixels) >= 0.99,
            "range resets original200ms clock"
        );
        let pcm = native.decode(&range_path, true, None);
        let probe = native.probe(&range_path);
        if let Some(old) = &range_pcm {
            pcm_equal(&pcm, old);
            assert_eq!(range_probe.as_ref(), Some(&probe));
        } else {
            range_pcm = Some(pcm);
            range_probe = Some(probe.clone());
        }
        let video = probe["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["codec_type"] == "video")
            .unwrap();
        assert_eq!(video["nb_frames"], "6");
        assert!(
            (probe["format"]["duration"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                - 0.6)
                .abs()
                <= 0.1
        );
        let output = root.path().join(format!("exports/order-{index}.mp4"));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        assert!(ssim(&reference, &native.decode(&output, false, Some("0.2"))) >= 0.99);
        assert!(
            ssim(&expected600, &native.decode(&output, false, Some("0.6"))) >= 0.99,
            "export original600ms held clock"
        );
        let pcm = native.decode(&output, true, None);
        let probe = native.probe(&output);
        if let Some(old) = &export_pcm {
            pcm_equal(&pcm, old);
            assert_eq!(export_probe.as_ref(), Some(&probe));
        } else {
            export_pcm = Some(pcm);
            export_probe = Some(probe.clone());
        }
        let video = probe["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["codec_type"] == "video")
            .unwrap();
        assert_eq!(video["nb_frames"], "8");
        assert!(
            (probe["format"]["duration"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap()
                - 0.8)
                .abs()
                <= 0.1
        );
    }
    assert_eq!(draft_pixels, reference_frames[1]);
    assert_ne!(reference_frames[0], reference_frames[1]);
    assert_eq!(reference_frames[1], reference_frames[2]);
    core.undo(&id, committed.revision).unwrap();
    let undone = core.get_project(&id).unwrap();
    assert_eq!(
        assert_frame(
            &native,
            &renderer,
            &undone,
            (&dir, &captured, root.path()),
            200,
            a
        ),
        reference_frames[0]
    );
    core.redo(&id, undone.revision).unwrap();
    let redone = core.get_project(&id).unwrap();
    assert_eq!(
        assert_frame(
            &native,
            &renderer,
            &redone,
            (&dir, &captured, root.path()),
            200,
            b
        ),
        reference_frames[1]
    );
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&redone).unwrap()
    );
    assert_eq!(
        assert_frame(
            &native,
            &renderer,
            &reopened,
            (&dir, &captured, root.path()),
            200,
            b
        ),
        reference_frames[1]
    );
}

#[test]
fn native_parameterized_component_scoped_effect_ids_keep_original_clock_after_reorder_and_ancestor_translation()
 {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, item) = setup();
    let f = fixture();
    let mut leaf =
        serde_json::to_value(core.get_project(&id).unwrap().find_item(&item).unwrap()).unwrap();
    edit(&core, &id, json!({"operation":"delete_item","itemId":item}));
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Scoped order","width":64,"height":64,"durationMs":2000,"tracks":[]}),
    );
    leaf["id"] = json!("leaf");
    leaf["durationMs"] = json!(2000);
    leaf["effects"] = f["orders"]["shadeThenWash"].clone();
    leaf["animationChannels"] = json!([{"property":"effect.vignette_amount","target":{"kind":"effect","scope":format!("component:{component}"),"id":"shade"},"keyframes":[
        {"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"hold"},{"timeMs":200,"value":{"type":"scalar","value":0.8},"curve":"hold"},{"timeMs":600,"value":{"type":"scalar","value":0.4},"curve":"hold"}
    ]}]);
    let definition_operation = |effects: &Value| {
        let mut local = leaf.clone();
        local["effects"] = effects.clone();
        json!({"operation":"component_update","componentId":component,"name":"Scoped order","width":64,"height":64,"durationMs":2000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[local]}]})
    };
    let update_definition = |effects: &Value| {
        edit(&core, &id, definition_operation(effects));
    };
    update_definition(&f["orders"]["shadeThenWash"]);
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let group = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":800}),
    );
    let transform = |x: f64| json!({"position":{"x":x,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1});
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":group,"transform2d":transform(4.0)}),
    );
    let instance = edit(
        &core,
        &id,
        json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":800,"trimStartMs":100,"timeScale":2,"transform2d":transform(0.0)}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":instance,"parent":{"scope":"root","id":group}}),
    );
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let before = core.get_project(&id).unwrap();
    let authoritative =
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap());
    let b = &f["orders"]["washThenShade"];
    let draft = core
        .create_draft(
            &id,
            before.revision,
            vec![op(definition_operation(b))],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    let mut sampled = b.clone();
    sampled
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|effect| effect["id"] == "shade")
        .unwrap()["amount"] = json!(0.4);
    let local = independent_plate(&sampled);
    let mut translated = vec![[0, 0, 0, 255]; 64 * 64];
    for y in 0..64 {
        for x in 0..60 {
            translated[y * 64 + x + 4] = local[y * 64 + x];
        }
    }
    let expected = native.authored_plate(root.path(), &translated);
    let frame = renderer.render_preview(&candidate, &dir, 300).unwrap();
    close_bytes(
        &raw_pam(&captured),
        translated.as_flattened(),
        "component draft original clock raw plate",
    );
    close_bytes(
        &native.decode(&dir.join(frame.relative_path), false, None),
        &expected,
        "component draft native plate",
    );
    assert_eq!(
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap()),
        authoritative
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    core.commit_draft(&id, &draft.id, before.revision).unwrap();
    let committed = core.get_project(&id).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(&committed).unwrap()
    );
    let frame = renderer.render_preview(&reopened, &dir, 300).unwrap();
    close_bytes(
        &raw_pam(&captured),
        translated.as_flattened(),
        "component reopened raw plate",
    );
    close_bytes(
        &native.decode(&dir.join(frame.relative_path), false, None),
        &expected,
        "component reopened native plate",
    );
    for declared in [&f["orders"]["shadeThenWash"], &f["orders"]["washThenShade"]] {
        update_definition(declared);
        let project = core.get_project(&id).unwrap();
        for (root_time, amount) in [(200, 0.8), (300, 0.4)] {
            let mut sampled = declared.clone();
            sampled
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|effect| effect["id"] == "shade")
                .unwrap()["amount"] = json!(amount);
            let local = independent_plate(&sampled);
            let mut translated = vec![[0, 0, 0, 255]; 64 * 64];
            for y in 0..64 {
                for x in 0..60 {
                    translated[y * 64 + x + 4] = local[y * 64 + x];
                }
            }
            let expected = native.authored_plate(root.path(), &translated);
            let frame = renderer.render_preview(&project, &dir, root_time).unwrap();
            close_bytes(
                &raw_pam(&captured),
                translated.as_flattened(),
                "scoped component clock and ancestor raw plate",
            );
            close_bytes(
                &native.decode(&dir.join(frame.relative_path), false, None),
                &expected,
                "scoped component native plate",
            );
            let range = renderer
                .render_preview_range(
                    &project,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: 200,
                        end_ms: 400,
                        width: 64,
                        height: 64,
                        fps: 10,
                        include_audio: false,
                    },
                    |_| {},
                )
                .unwrap();
            let seek = if root_time == 200 { "0" } else { "0.1" };
            assert!(
                ssim(
                    &expected,
                    &native.decode(&dir.join(range.relative_path), false, Some(seek))
                ) >= 0.99
            );
        }
        let output = root
            .path()
            .join(format!("exports/component-{}.mp4", project.revision));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        renderer
            .export_video(
                &project,
                &dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let reference = renderer.render_preview(&project, &dir, 300).unwrap();
        assert!(
            ssim(
                &native.decode(&dir.join(reference.relative_path), false, None),
                &native.decode(&output, false, Some("0.3"))
            ) >= 0.99
        );
    }
}
