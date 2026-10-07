//! Independent group/overlay plates and actual public render lifecycle.
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
                    .expect("native group-compositing oracle requires bundled font")
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
                    "required native group-compositing tools missing"
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

// Independently frozen unsigned hash words for seed1, indices0..7, lanes0..2.
// The oracle does not call or duplicate the runtime wrapping hash generator.
const PARTICLE_WORDS: [[u32; 3]; 8] = [
    [2336985851, 938826240, 1456866664],
    [84929298, 829401672, 2771797279],
    [2877292033, 4199349311, 3397877525],
    [4067787588, 1783773585, 2445685191],
    [3863548237, 661299992, 4101546638],
    [2606412158, 1268270209, 2067976603],
    [2944314353, 2443517269, 3235942571],
    [2958880360, 2302107493, 891655201],
];
fn linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}
fn oracle(stack: &Value, time: u64) -> Vec<[u8; 4]> {
    let side = 74usize;
    let origin = -5f64;
    let mut plane = vec![[0f64; 4]; side * side];
    for y in 0..24 {
        for x in 0..32 {
            plane[(y + 8 + 5) * side + x + 4 + 5] = [0., 0.4, 0., 0.4];
        }
    }
    for effect in stack.as_array().unwrap() {
        match effect["type"].as_str().unwrap() {
            "screen_flash" => {
                let start = effect["startMs"].as_u64().unwrap();
                let duration = effect["durationMs"].as_u64().unwrap();
                let strength = if time >= start && time < start + duration {
                    effect["intensity"].as_f64().unwrap()
                        * effect["color"]["a"].as_f64().unwrap()
                        * (1. - (time - start) as f64 / duration as f64)
                } else {
                    0.
                };
                let color = ["r", "g", "b"].map(|c| linear(effect["color"][c].as_f64().unwrap()));
                for pixel in &mut plane {
                    for c in 0..3 {
                        pixel[c] += (pixel[3] - pixel[c]) * color[c] * strength;
                    }
                }
            }
            "particle_overlay" => {
                assert_eq!(effect["count"], 8);
                assert_eq!(effect["seed"], 1);
                let radius = effect["radiusPx"].as_f64().unwrap();
                let speed = effect["speedPxPerSecond"].as_f64().unwrap();
                let life = effect["lifetimeMs"].as_u64().unwrap();
                let color = ["r", "g", "b"].map(|c| linear(effect["color"][c].as_f64().unwrap()));
                for words in PARTICLE_WORDS {
                    let [u, v, phase] = words.map(|word| f64::from(word) / 4294967296.);
                    let center = [
                        64. * u,
                        (64. * v
                            + speed
                                * ((time % life) as f64 + phase * life as f64)
                                    .rem_euclid(life as f64)
                                / 1000.)
                            .rem_euclid(64.),
                    ];
                    for y in 0..side {
                        for x in 0..side {
                            let mut covered = 0u32;
                            for row in 0..4 {
                                for column in 0..4 {
                                    let dx =
                                        x as f64 + origin + (column as f64 + 0.5) / 4. - center[0];
                                    let dy =
                                        y as f64 + origin + (row as f64 + 0.5) / 4. - center[1];
                                    covered += u32::from(dx * dx + dy * dy <= radius * radius);
                                }
                            }
                            let alpha =
                                effect["color"]["a"].as_f64().unwrap() * f64::from(covered) / 16.;
                            let pixel = &mut plane[y * side + x];
                            for c in 0..3 {
                                pixel[c] = color[c] * alpha + pixel[c] * (1. - alpha);
                            }
                            pixel[3] = alpha + pixel[3] * (1. - alpha);
                        }
                    }
                }
            }
            "gaussian_blur" => {
                assert_eq!(effect["radiusPx"], 1);
                let mut kernel = (-3i32..=3)
                    .map(|x| (-0.5 * f64::from(x * x)).exp())
                    .collect::<Vec<_>>();
                let sum = kernel.iter().sum::<f64>();
                for weight in &mut kernel {
                    *weight /= sum;
                }
                let mut horizontal = vec![[0f64; 4]; side * side];
                let mut output = horizontal.clone();
                for y in 0..side {
                    for x in 0..side {
                        for (offset, weight) in kernel.iter().enumerate() {
                            let sx = x as i32 + offset as i32 - 3;
                            if (0..side as i32).contains(&sx) {
                                for c in 0..4 {
                                    horizontal[y * side + x][c] +=
                                        plane[y * side + sx as usize][c] * weight;
                                }
                            }
                        }
                    }
                }
                for y in 0..side {
                    for x in 0..side {
                        for (offset, weight) in kernel.iter().enumerate() {
                            let sy = y as i32 + offset as i32 - 3;
                            if (0..side as i32).contains(&sy) {
                                for c in 0..4 {
                                    output[y * side + x][c] +=
                                        horizontal[sy as usize * side + x][c] * weight;
                                }
                            }
                        }
                    }
                }
                plane = output;
            }
            other => panic!("unexpected independent oracle effect {other}"),
        }
    }
    (0..64)
        .flat_map(|y| (0..64).map(move |x| (x, y)))
        .map(|(x, y)| {
            let p = plane[(y + 5) * side + x + 5];
            [
                encoded(p[0] * 0.65),
                encoded(p[1] * 0.65),
                encoded(p[2] * 0.65),
                255,
            ]
        })
        .collect()
}
fn stacks() -> [Value; 3] {
    let f: Value =
        serde_json::from_str(include_str!("../../../contracts/group-compositing-v1.json")).unwrap();
    let witness = &f["nativeWitness"];
    let flash = witness["flash"].clone();
    let particles = witness["particles"].clone();
    let blur = json!({"id":"positive","type":"gaussian_blur","radiusPx":1});
    [
        json!([flash, particles, blur]),
        json!([particles, flash, blur]),
        json!([flash, blur, particles]),
    ]
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
            "Native aggregate",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let result=core.edit_batch(&id,0,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":800,"resultAlias":"owner"},
        {"operation":"update_item","itemId":"@owner","clip":{"type":"composition_bounds"},"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.65}},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":32,"height":24,"color":"#00ff00","transform":{"positionX":4,"positionY":8,"scale":1,"opacity":0.4},"resultAlias":"child"},
        {"operation":"item_set_parent","itemId":"@child","parent":{"scope":"root","id":"@owner"}}
    ])).unwrap()).unwrap();
    let owner = result.aliases["owner"].clone();
    (root, core, id, owner)
}
fn frame(
    native: &Native,
    renderer: &Renderer,
    project: &Project,
    paths: (&Path, &Path, &Path),
    time: u64,
    stack: &Value,
) -> Vec<u8> {
    let (dir, captured, root) = paths;
    let expected = oracle(stack, time);
    let converted = native.authored_plate(root, &expected);
    let result = renderer.render_preview(project, dir, time).unwrap();
    close_bytes(
        &raw_pam(captured),
        expected.as_flattened(),
        "complete independent aggregate PAM",
    );
    let pixels = native.decode(&dir.join(result.relative_path), false, None);
    close_bytes(&pixels, &converted, "complete converted aggregate plate");
    pixels
}
#[test]
fn independent_complete_flash_particle_positive_gaussian_and_noncommuting_order_plates() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, owner) = setup();
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let mut witnessed = Vec::new();
    for stack in stacks() {
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":owner,"effects":stack}),
        );
        witnessed.push(frame(
            &native,
            &renderer,
            &core.get_project(&id).unwrap(),
            (&dir, &captured, root.path()),
            200,
            &stack,
        ));
    }
    assert_ne!(
        witnessed[0], witnessed[1],
        "flash/particles must expose ordered color"
    );
    assert_ne!(
        witnessed[0], witnessed[2],
        "particles/positive Gaussian must expose noncommuting coverage"
    );
}
#[test]
fn group_compositing_public_intents_drafts_history_original_clock_grid_duration_and_positive_pcm() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, owner) = setup();
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let [a, b, _] = stacks();
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":owner,"effects":a}),
    );
    let before = core.get_project(&id).unwrap();
    let draft = core
        .create_draft(
            &id,
            before.revision,
            vec![op(
                json!({"operation":"update_item","itemId":owner,"effects":b}),
            )],
            None,
        )
        .unwrap();
    let candidate = core.get_draft_state(&id, &draft.id).unwrap().project;
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let saved = ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap());
    let draft_pixels = frame(
        &native,
        &renderer,
        &candidate,
        (&dir, &captured, root.path()),
        200,
        &b,
    );
    assert_eq!(
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap()),
        saved
    );
    core.commit_draft(&id, &draft.id, before.revision).unwrap();
    let committed = core.get_project(&id).unwrap();
    let mut ranges: Option<Vec<u8>> = None;
    let mut exports: Option<Vec<u8>> = None;
    let mut references = Vec::new();
    for (index, (project, stack)) in [(&before, &a), (&candidate, &b), (&committed, &b)]
        .into_iter()
        .enumerate()
    {
        let reference = frame(
            &native,
            &renderer,
            project,
            (&dir, &captured, root.path()),
            200,
            stack,
        );
        references.push(reference.clone());
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
        let path = dir.join(range.relative_path);
        assert!(ssim(&reference, &native.decode(&path, false, Some("0"))) >= 0.99);
        let expected600 = native.authored_plate(root.path(), &oracle(stack, 600));
        assert!(
            ssim(&expected600, &native.decode(&path, false, Some("0.4"))) >= 0.99,
            "nonzero range keeps original600ms owner clock"
        );
        let pcm = native.decode(&path, true, None);
        if let Some(old) = &ranges {
            pcm_equal(&pcm, old);
        } else {
            assert!(
                pcm.as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| f32::from_le_bytes(*p).abs() > 0.01)
            );
            ranges = Some(pcm);
        }
        let probe = native.probe(&path);
        let video = probe["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|stream| stream["codec_type"] == "video")
            .unwrap();
        assert_eq!(video["nb_frames"], "6");
        assert_eq!(video["width"], 64);
        assert_eq!(video["height"], 64);
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
        let output = root.path().join(format!("exports/group-{index}.mp4"));
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
        assert!(ssim(&expected600, &native.decode(&output, false, Some("0.6"))) >= 0.99);
        let pcm = native.decode(&output, true, None);
        if let Some(old) = &exports {
            pcm_equal(&pcm, old);
        } else {
            assert!(
                pcm.as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| f32::from_le_bytes(*p).abs() > 0.01)
            );
            exports = Some(pcm);
        }
        let probe = native.probe(&output);
        let video = probe["streams"]
            .as_array()
            .unwrap()
            .iter()
            .find(|stream| stream["codec_type"] == "video")
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
    assert_eq!(draft_pixels, references[1]);
    assert_ne!(references[0], references[1]);
    assert_eq!(references[1], references[2]);
    core.undo(&id, committed.revision).unwrap();
    let undone = core.get_project(&id).unwrap();
    assert_eq!(
        frame(
            &native,
            &renderer,
            &undone,
            (&dir, &captured, root.path()),
            200,
            &a
        ),
        references[0]
    );
    core.redo(&id, undone.revision).unwrap();
    let redone = core.get_project(&id).unwrap();
    let reopened = EditorCore::new(core.paths().clone())
        .get_project(&id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&redone).unwrap(),
        serde_json::to_value(&reopened).unwrap()
    );
    assert_eq!(
        frame(
            &native,
            &renderer,
            &reopened,
            (&dir, &captured, root.path()),
            200,
            &b
        ),
        references[1]
    );
}

#[test]
fn signed_private_provider_excludes_owner_halo_and_uses_full_world_unequal_shutters_and_opacity() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, _) = setup();
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    // Replace the setup's ordinary subtree with the independent signed witness.
    let old = core.get_project(&id).unwrap().tracks[1]
        .items
        .iter()
        .map(|item| item.id().to_owned())
        .collect::<Vec<_>>();
    for item in old.into_iter().rev() {
        edit(&core, &id, json!({"operation":"delete_item","itemId":item}));
    }
    core.edit_batch(&id,core.get_project(&id).unwrap().revision,serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_group","trackId":track,"startMs":0,"durationMs":800,"resultAlias":"g"},
        {"operation":"update_item","itemId":"@g","transform2d":null,"effects":[{"id":"owner-halo","type":"gaussian_blur","radiusPx":1}]},
        {"operation":"set_animation_channels","itemId":"@g","animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"hold"},{"timeMs":200,"value":{"type":"scalar","value":0.8},"curve":"hold"}]}]},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":4,"height":4,"color":"#ffffff","transform":{"positionX":-2,"positionY":4,"scale":1,"opacity":1},"resultAlias":"provider"},
        {"operation":"item_set_parent","itemId":"@provider","parent":{"scope":"root","id":"@g"}},
        {"operation":"update_item","itemId":"@provider","matteOnly":true,"motionBlur":{"shutterAngleDeg":180,"sampleCount":2},"effects":[{"id":"private-halo","type":"gaussian_blur","radiusPx":1}]},
        {"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":4,"height":4,"color":"#00ff00","transform":{"positionX":-2,"positionY":4,"scale":1,"opacity":1},"resultAlias":"recipient"},
        {"operation":"item_set_parent","itemId":"@recipient","parent":{"scope":"root","id":"@g"}},
        {"operation":"update_item","itemId":"@recipient","matte":{"sourceId":"@provider","channel":"alpha"},"motionBlur":{"shutterAngleDeg":180,"sampleCount":4}}
    ])).unwrap()).unwrap();
    let original_owner = core.get_project(&id).unwrap().tracks[1]
        .items
        .iter()
        .find(|item| matches!(item, opencut_editor_core::TimelineItem::Group(_)))
        .unwrap()
        .id()
        .to_owned();
    let outer = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":800}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":outer,"transform2d":null,"effects":[{"id":"nested-identity","type":"screen_flash","startMs":0,"durationMs":1000,"intensity":0,"color":{"r":1,"g":1,"b":1,"a":1}}]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":original_owner,"parent":{"scope":"root","id":outer}}),
    );
    // Provider shutter opacity averages .2,.5,.5,.8 over recipient times
    // 181,193,206,218ms, giving .5. Owner output at200ms contributes .8 once.
    // Private provider includes its own Gaussian, excludes the owner Gaussian;
    // direct matteOnly output never contributes white or its halo.
    let mut kernel = (-3i32..=3)
        .map(|i| (-0.5 * f64::from(i * i)).exp())
        .collect::<Vec<_>>();
    let sum = kernel.iter().sum::<f64>();
    for v in &mut kernel {
        *v /= sum;
    }
    let inside = |x: i32, y: i32| (-2..2).contains(&x) && (4..8).contains(&y);
    let provider = |x: i32, y: i32| {
        let mut value = 0.;
        for (a, ka) in kernel.iter().enumerate() {
            for (b, kb) in kernel.iter().enumerate() {
                if inside(x + a as i32 - 3, y + b as i32 - 3) {
                    value += ka * kb;
                }
            }
        }
        value
    };
    let expected_for = |factor: f64| {
        let mut expected = vec![[0u8, 0, 0, 255]; 64 * 64];
        for y in 0..64 {
            for x in 0..64 {
                let mut value = 0.;
                for (a, ka) in kernel.iter().enumerate() {
                    for (b, kb) in kernel.iter().enumerate() {
                        let (sx, sy) = (x as i32 + a as i32 - 3, y as i32 + b as i32 - 3);
                        if inside(sx, sy) {
                            value += ka * kb * provider(sx, sy);
                        }
                    }
                }
                expected[y * 64 + x][1] = encoded(value * factor);
            }
        }
        expected
    };
    let expected = expected_for(0.4);
    assert!(
        expected[4 * 64][1] > 0,
        "offcanvas coverage must reach visible pixels"
    );
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let result = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 200)
        .unwrap();
    close_bytes(
        &raw_pam(&captured),
        expected.as_flattened(),
        "complete signed private-provider/temporal aggregate plate",
    );
    close_bytes(
        &native.decode(&dir.join(result.relative_path), false, None),
        &native.authored_plate(root.path(), &expected),
        "converted signed provider plate",
    );
    let project = core.get_project(&id).unwrap();
    let items = &project.tracks[1].items;
    let provider_id = items
        .iter()
        .find(|item| item.visual_properties().matte_only)
        .unwrap()
        .id()
        .to_owned();
    let recipient_id = items
        .iter()
        .find(|item| item.visual_properties().matte.is_some())
        .unwrap()
        .id()
        .to_owned();
    let owner_id = items
        .iter()
        .find(|item| matches!(item, opencut_editor_core::TimelineItem::Group(_)))
        .unwrap()
        .id()
        .to_owned();
    // All public intents retain unequal private/recipient shutter clocks, with
    // a frozen owner200ms gain. At600ms every provider shutter has gain.8.
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    controlled_intents(
        &native,
        &renderer,
        (&core, &id, &owner_id),
        &dir,
        root.path(),
        &expected,
        &expected_for(0.64),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider_id,"color":"#ff0000"}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":recipient_id,"matte":{"sourceId":provider_id,"channel":"luma"}}),
    );
    let luma = expected_for(0.4 * 0.2126);
    assert_ne!(luma, expected);
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (&luma, "red linear-luma controlled matte"),
    );
    let other = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":800}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":other,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8},"effects":[{"id":"recipient-halo","type":"gaussian_blur","radiusPx":1}]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":recipient_id,"parent":{"scope":"root","id":other}}),
    );
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (&luma, "cross-isolation luma provider excludes owner halos"),
    );
    let black = vec![[0, 0, 0, 255]; 64 * 64];
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":owner_id,"animationChannels":[]}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":owner_id,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0}}),
    );
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (&black, "zero provider ancestry gain without division"),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":owner_id,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":other,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0}}),
    );
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (&black, "zero controlled recipient gain"),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":other,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1e-30,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}}),
    );
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (
            &black,
            "near-collapsed controlled query retains transparent pixel-center output",
        ),
    );
    // Each positive-scale stage has finite analytic inverse. Composition loses
    // its tiny orthogonal component in f64; no inverse-owner reconstruction is
    // possible. The independent forward line falls outside provider support.
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":provider_id,"width":64,"height":64,"color":"#ffffff","effects":[],"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":recipient_id,"width":64,"height":64,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}),
    );
    let (sin, cos) = 45f64.to_radians().sin_cos();
    let [a, b, c, d] = [cos * cos, sin * cos, -cos * sin, -sin * sin];
    assert_eq!(a * d - b * c, 0.);
    let parent = edit(
        &core,
        &id,
        json!({"operation":"add_group","trackId":track,"startMs":0,"durationMs":800}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":parent,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1e-30,"rotationDeg":45,"skewXDeg":0,"skewYDeg":0,"opacity":1}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":other,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":45,"skewXDeg":0,"skewYDeg":0,"opacity":0.8}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":other,"parent":{"scope":"root","id":parent}}),
    );
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (
            &black,
            "floating-singular controlled outward query without owner inversion",
        ),
    );
}

#[test]
fn nested_repeated_instances_use_definition_basis_noncentral_pivots_scoped_ids_and_one_outer_gain()
{
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, owner) = setup();
    let project = core.get_project(&id).unwrap();
    let track = project.tracks[1].id.clone();
    let child = project.tracks[1]
        .items
        .iter()
        .find(|item| item.id() != owner)
        .unwrap()
        .id()
        .to_owned();
    edit(
        &core,
        &id,
        json!({"operation":"delete_item","itemId":child}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":owner,"transform2d":{"position":{"x":0,"y":0,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.5}}),
    );
    let component = edit(
        &core,
        &id,
        json!({"operation":"component_create","name":"Fixed32x24basis","width":32,"height":24,"durationMs":1000,"tracks":[{"id":"local-track","name":"Local","trackType":"overlay","items":[{"id":"green","type":"rectangle","startMs":0,"durationMs":1000,"width":32,"height":24,"color":"#00ff00","zIndex":99,"stackOrder":0,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.4},"keyframes":[]}]}]}),
    );
    let particles = stacks()[0][1].clone();
    let mut instances = Vec::new();
    for (index, position) in [[16, 24], [48, 40]].into_iter().enumerate() {
        let instance = edit(
            &core,
            &id,
            json!({"operation":"add_component_instance","trackId":track,"componentId":component,"startMs":0,"durationMs":800,"trimStartMs":0,"timeScale":1,
            "transform2d":{"position":{"x":position[0],"y":position[1],"unit":"pixels"},"anchor":{"x":0.25,"y":0.75},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":0.6}}),
        );
        instances.push(instance.clone());
        edit(
            &core,
            &id,
            json!({"operation":"item_set_parent","itemId":instance,"parent":{"scope":"root","id":owner}}),
        );
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":instance,"clip":{"type":"composition_bounds"},"effects":[particles],"stackOrder":index+1}),
        );
    }
    let expected = independent_nested_plate(None);
    assert!(expected.iter().any(|p| p[2] > 20));
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    let result = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 200)
        .unwrap();
    close_bytes(
        &raw_pam(&captured),
        expected.as_flattened(),
        "complete nested fixed-basis/noncentral-pivot/scoped-ID plate",
    );
    close_bytes(
        &native.decode(&dir.join(result.relative_path), false, None),
        &native.authored_plate(root.path(), &expected),
        "converted nested instance plate",
    );
    let mut tracks =
        serde_json::to_value(&core.get_project(&id).unwrap().components[0].tracks).unwrap();
    tracks[0]["items"][0]["effects"] = json!([stacks()[0][0]]);
    edit(
        &core,
        &id,
        json!({"operation":"component_update","componentId":component,"name":"Fixed32x24basis","width":32,"height":24,"durationMs":1000,"tracks":tracks}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"component_instance_update","itemId":instances[0],"componentId":component,"startMs":0,"trimStartMs":100,"timeScale":2,"durationMs":400}),
    );
    let project = core.get_project(&id).unwrap();
    let instance = project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .find(|item| item.id() == instances[0])
        .unwrap();
    let opencut_editor_core::TimelineItem::ComponentInstance(instance) = instance else {
        panic!("retimed item must remain a component instance");
    };
    assert_eq!(
        (
            instance.trim_start_ms,
            instance.time_scale,
            instance.duration_ms
        ),
        (100, 2., 400)
    );
    let retimed = independent_nested_plate(Some([500., 200.]));
    assert_ne!(retimed, expected);
    let result = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 200)
        .unwrap();
    close_bytes(
        &raw_pam(&captured),
        retimed.as_flattened(),
        "complete retimed definition leaf flash plus root-clock owner particles",
    );
    close_bytes(
        &native.decode(&dir.join(result.relative_path), false, None),
        &native.authored_plate(root.path(), &retimed),
        "converted retimed repeated instance plate",
    );
}

fn independent_nested_plate(times: Option<[f64; 2]>) -> Vec<[u8; 4]> {
    let mut plane = vec![[0f64; 4]; 64 * 64];
    // Each instance composites onto the transparent outer aggregate at .6;
    // its shared .5 ancestor is applied only after both have blended.
    for (index, [tx, ty]) in [[8f64, 6f64], [40., 22.]].into_iter().enumerate() {
        let mut source = vec![[0f64; 4]; 64 * 64];
        for y in 0..64 {
            for x in 0..64 {
                if (tx..tx + 32.).contains(&(x as f64 + 0.5))
                    && (ty..ty + 24.).contains(&(y as f64 + 0.5))
                {
                    let red =
                        times.map_or(0., |times| 0.4 * 0.8 * 0.6 * (1. - times[index] / 1000.));
                    source[y * 64 + x] = [red, 0.4, 0., 0.4];
                }
            }
        }
        for words in PARTICLE_WORDS {
            let [u, v, phase] = words.map(|w| f64::from(w) / 4294967296.);
            let center = [
                tx + 32. * u,
                ty + (24. * v + 12. * (200. + phase * 1000.).rem_euclid(1000.) / 1000.)
                    .rem_euclid(24.),
            ];
            for y in 0..64 {
                for x in 0..64 {
                    let mut hits = 0u32;
                    for row in 0..4 {
                        for column in 0..4 {
                            let dx = x as f64 + (column as f64 + 0.5) / 4. - center[0];
                            let dy = y as f64 + (row as f64 + 0.5) / 4. - center[1];
                            hits += u32::from(dx * dx + dy * dy <= 4.);
                        }
                    }
                    let alpha = 0.7 * f64::from(hits) / 16.;
                    let pixel = &mut source[y * 64 + x];
                    pixel[0] *= 1. - alpha;
                    pixel[1] = linear(0.5) * alpha + pixel[1] * (1. - alpha);
                    pixel[2] = alpha + pixel[2] * (1. - alpha);
                    pixel[3] = alpha + pixel[3] * (1. - alpha);
                }
            }
        }
        for (destination, source) in plane.iter_mut().zip(source) {
            let alpha = source[3] * 0.6;
            for c in 0..3 {
                destination[c] = source[c] * 0.6 + destination[c] * (1. - alpha);
            }
            destination[3] = alpha + destination[3] * (1. - alpha);
        }
    }
    plane
        .iter()
        .map(|p| {
            [
                encoded(p[0] * 0.5),
                encoded(p[1] * 0.5),
                encoded(p[2] * 0.5),
                255,
            ]
        })
        .collect::<Vec<_>>()
}

fn assert_plate(
    native: &Native,
    renderer: &Renderer,
    context: (&EditorCore, &str),
    dir: &Path,
    captured: &Path,
    root: &Path,
    reference: (&[[u8; 4]], &str),
) {
    let (core, id) = context;
    let (expected, label) = reference;
    let result = renderer
        .render_preview(&core.get_project(id).unwrap(), dir, 200)
        .unwrap();
    close_bytes(&raw_pam(captured), expected.as_flattened(), label);
    close_bytes(
        &native.decode(&dir.join(result.relative_path), false, None),
        &native.authored_plate(root, expected),
        label,
    );
}
fn controlled_intents(
    native: &Native,
    renderer: &Renderer,
    context: (&EditorCore, &str, &str),
    dir: &Path,
    root: &Path,
    expected200: &[[u8; 4]],
    expected600: &[[u8; 4]],
) {
    let (core, id, owner) = context;
    let before = core.get_project(id).unwrap();
    let saved = ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap());
    let draft = core.create_draft(id,before.revision,vec![op(json!({"operation":"update_item","itemId":owner,"effects":[{"id":"owner-halo","type":"gaussian_blur","radiusPx":1}]}))],None).unwrap();
    let candidate = core.get_draft_state(id, &draft.id).unwrap().project;
    let expected200 = native.authored_plate(root, expected200);
    let expected600 = native.authored_plate(root, expected600);
    let mut reference_pcm: Option<Vec<u8>> = None;
    for (index, project) in [&before, &candidate].into_iter().enumerate() {
        let range = renderer
            .render_preview_range(
                project,
                dir,
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
        let path = dir.join(range.relative_path);
        assert!(ssim(&expected200, &native.decode(&path, false, Some("0"))) >= 0.99);
        assert!(ssim(&expected600, &native.decode(&path, false, Some("0.4"))) >= 0.99);
        assert_eq!(
            native.probe(&path)["streams"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["codec_type"] == "video")
                .unwrap()["nb_frames"],
            "6"
        );
        let pcm = native.decode(&path, true, None);
        assert!(
            pcm.as_chunks::<4>()
                .0
                .iter()
                .any(|p| f32::from_le_bytes(*p).abs() > 0.01)
        );
        if let Some(ref old) = reference_pcm {
            pcm_equal(&pcm, old);
        } else {
            reference_pcm = Some(pcm);
        }
        let output = root.join(format!("exports/temporal-{index}.mp4"));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        renderer
            .export_video(
                project,
                dir,
                ExportOptions {
                    output: &output,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        assert!(ssim(&expected200, &native.decode(&output, false, Some("0.2"))) >= 0.99);
        assert!(ssim(&expected600, &native.decode(&output, false, Some("0.6"))) >= 0.99);
        assert_eq!(
            native.probe(&output)["streams"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["codec_type"] == "video")
                .unwrap()["nb_frames"],
            "8"
        );
        assert!(
            native
                .decode(&output, true, None)
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| f32::from_le_bytes(*p).abs() > 0.01)
        );
    }
    assert_eq!(
        ["project.json", "history.json"].map(|name| std::fs::read(dir.join(name)).unwrap()),
        saved
    );
}

#[test]
fn controlled_transparent_non_normal_children_blend_once_before_owner_gain() {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, owner) = setup();
    let project = core.get_project(&id).unwrap();
    let track = project.tracks[1].id.clone();
    let first = project.tracks[1]
        .items
        .iter()
        .find(|item| item.id() != owner)
        .unwrap()
        .id()
        .to_owned();
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":first,"color":"#ff0000","transform":{"positionX":4,"positionY":8,"scale":1,"opacity":0.5}}),
    );
    let second = edit(
        &core,
        &id,
        json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":800,"width":32,"height":24,"color":"#0000ff","transform":{"positionX":4,"positionY":8,"scale":1,"opacity":0.5}}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"item_set_parent","itemId":second,"parent":{"scope":"root","id":owner}}),
    );
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    for (mode, color) in [("multiply", 0.25), ("screen", 0.5)] {
        edit(
            &core,
            &id,
            json!({"operation":"update_item","itemId":second,"blendMode":mode}),
        );
        // Independent blend: As=Ad=.5; multiply(red,blue)=0,
        // screen(red,blue)=(1,0,1). Owner .65 is applied once.
        let mut expected = vec![[0, 0, 0, 255]; 64 * 64];
        for y in 8..32 {
            for x in 4..36 {
                expected[y * 64 + x] = [encoded(color * 0.65), 0, encoded(color * 0.65), 255];
            }
        }
        assert_plate(
            &native,
            &renderer,
            (&core, &id),
            &dir,
            &captured,
            root.path(),
            (&expected, mode),
        );
    }
}

#[test]
fn animated_controlled_outward_translation_rotation_and_zero_gain_are_frozen_across_child_shutter_intents()
 {
    let Some(native) = Native::configured() else {
        return;
    };
    let (root, core, id, owner) = setup();
    let child = core.get_project(&id).unwrap().tracks[1]
        .items
        .iter()
        .find(|item| item.id() != owner)
        .unwrap()
        .id()
        .to_owned();
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":owner,"transform2d":null,"effects":[{"id":"owner-halo","type":"gaussian_blur","radiusPx":1}]}),
    );
    let channels = [("transform.position_x",0.,40.),("transform.rotation_deg",0.,90.),("transform.opacity",0.,0.65)].map(|(property,first,last)|json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first},"curve":"hold"},{"timeMs":200,"value":{"type":"scalar","value":last},"curve":"hold"}]}));
    edit(
        &core,
        &id,
        json!({"operation":"set_animation_channels","itemId":owner,"animationChannels":channels}),
    );
    edit(
        &core,
        &id,
        json!({"operation":"update_item","itemId":child,"motionBlur":{"shutterAngleDeg":180,"sampleCount":4}}),
    );
    add_audio(&core, &id, &root.path().join("media/tone.wav"));
    let mut kernel = (-3i32..=3)
        .map(|i| (-0.5 * f64::from(i * i)).exp())
        .collect::<Vec<_>>();
    let sum = kernel.iter().sum::<f64>();
    for value in &mut kernel {
        *value /= sum;
    }
    let mut expected = vec![[0, 0, 0, 255]; 64 * 64];
    for y in 0..64 {
        for x in 0..64 {
            let (lx, ly) = (y as i32, 39 - x as i32);
            let mut coverage = 0.;
            for (a, ka) in kernel.iter().enumerate() {
                for (b, kb) in kernel.iter().enumerate() {
                    if (4..36).contains(&(lx + a as i32 - 3))
                        && (8..32).contains(&(ly + b as i32 - 3))
                    {
                        coverage += ka * kb;
                    }
                }
            }
            expected[y * 64 + x][1] = encoded(coverage * 0.4 * 0.65);
        }
    }
    assert!(expected[8 * 64 + 16][1] > 0);
    let (renderer, captured) = native.capturing_renderer(root.path());
    let dir = core.paths().project_dir(&id).unwrap();
    assert_plate(
        &native,
        &renderer,
        (&core, &id),
        &dir,
        &captured,
        root.path(),
        (
            &expected,
            "owner output200ms frozen transform/gain while child shutters cross0→200hold",
        ),
    );
    controlled_intents(
        &native,
        &renderer,
        (&core, &id, &owner),
        &dir,
        root.path(),
        &expected,
        &expected,
    );
    let zero = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 100)
        .unwrap();
    let black = vec![[0, 0, 0, 255]; 64 * 64];
    close_bytes(
        &raw_pam(&captured),
        black.as_flattened(),
        "zero output owner gain before hold transition",
    );
    close_bytes(
        &native.decode(&dir.join(zero.relative_path), false, None),
        &native.authored_plate(root.path(), &black),
        "converted zero output owner gain",
    );
}
