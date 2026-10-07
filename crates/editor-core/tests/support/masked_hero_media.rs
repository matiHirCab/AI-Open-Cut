//! Actual configured codecs, lossless boundaries and independently sourced stereo comparisons.
use opencut_editor_core::Renderer;
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
pub fn media_reference(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/masked-hero-reveal-v1")
        .join(name)
}
pub struct Native {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
    pub font: PathBuf,
}
impl Native {
    pub fn configured() -> Option<Self> {
        let configured = match (
            std::env::var_os("OPENCUT_FFMPEG_PATH"),
            std::env::var_os("OPENCUT_FFPROBE_PATH"),
        ) {
            (Some(ffmpeg), Some(ffprobe)) => Some(Self {
                ffmpeg: ffmpeg.into(),
                ffprobe: ffprobe.into(),
                font: std::env::var_os("OPENCUT_TEST_FONT_PATH")
                    .expect("native masked-hero-reveal oracle requires bundled font")
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
                    "required native masked-hero-reveal tools missing"
                );
            }
        }
        configured
    }
    pub fn capturing_renderer(&self, root: &Path) -> (Renderer, PathBuf) {
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
    pub fn authored_plate(&self, root: &Path, pixels: &[[u8; 4]]) -> Vec<u8> {
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
    pub fn decode(&self, path: &Path, audio: bool, seek: Option<&str>) -> Vec<u8> {
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
    pub fn probe(&self, path: &Path) -> Value {
        let result = Command::new(&self.ffprobe).args(["-v", "error", "-show_entries", "format=duration:stream=codec_type,codec_name,time_base,duration,start_time,nb_frames,width,height,r_frame_rate,sample_rate,channels", "-of", "json"]).arg(path).output().unwrap();
        assert!(result.status.success());
        serde_json::from_slice(&result.stdout).unwrap()
    }
}
pub fn ssim(a: &[u8], b: &[u8]) -> f64 {
    assert_eq!(a.len(), b.len());
    assert!(!a.is_empty(), "SSIM requires nonempty pixels");
    let n = a.len() as f64;
    let ma = a.iter().map(|x| f64::from(*x)).sum::<f64>() / n;
    let mb = b.iter().map(|x| f64::from(*x)).sum::<f64>() / n;
    let va = a.iter().map(|x| (f64::from(*x) - ma).powi(2)).sum::<f64>() / n;
    let vb = b.iter().map(|x| (f64::from(*x) - mb).powi(2)).sum::<f64>() / n;
    let cov = a
        .iter()
        .zip(b)
        .map(|(x, y)| (f64::from(*x) - ma) * (f64::from(*y) - mb))
        .sum::<f64>()
        / n;
    ((2.0 * ma * mb + 6.5025) * (2.0 * cov + 58.5225))
        / ((ma * ma + mb * mb + 6.5025) * (va + vb + 58.5225))
}
pub fn raw_pam(path: &Path) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap();
    let end = bytes.windows(7).position(|b| b == b"ENDHDR\n").unwrap() + 7;
    let header = std::str::from_utf8(&bytes[..end]).unwrap();
    assert!(header.contains("WIDTH 64\nHEIGHT 64\nDEPTH 4\n"));
    bytes[end..].to_vec()
}
pub fn close_bytes(actual: &[u8], expected: &[u8], label: &str) {
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
pub fn audio_reference(native: &Native, root: &Path, start: u64, restart: bool) -> PathBuf {
    let output = root.join(format!("reference-audio-{start}-{restart}.m4a"));
    let mut cmd = Command::new(&native.ffmpeg);
    cmd.args(["-v", "error", "-nostdin", "-i"])
        .arg(media_reference("source.wav"));
    cmd.args([
        "-ss",
        &format!("{}", if restart { 0.0 } else { start as f64 / 1000.0 }),
        "-t",
        &format!("{}", (800 - start) as f64 / 1000.0),
        "-c:a",
        "aac",
        "-y",
    ])
    .arg(&output);
    let result = cmd.output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
pub fn stereo(pcm: &[u8]) -> Vec<[f64; 2]> {
    let (chunks, tail) = pcm.as_chunks::<8>();
    assert!(tail.is_empty());
    chunks
        .iter()
        .map(|p| {
            [
                f64::from(f32::from_le_bytes(p[..4].try_into().unwrap())),
                f64::from(f32::from_le_bytes(p[4..].try_into().unwrap())),
            ]
        })
        .collect()
}
pub fn aligned_stereo(actual: &[u8], reference: &[u8]) -> [f64; 3] {
    let a = stereo(actual);
    let b = stereo(reference);
    assert!(
        a.iter().chain(&b).flatten().all(|v| v.is_finite()),
        "decoded stereo PCM must be finite"
    );
    assert!(
        a.len().abs_diff(b.len()) <= 4800,
        "PCM lengths must stay within one video frame"
    );
    assert!(
        a.len().min(b.len()) > 4800,
        "positive overlap must cover more than the alignment budget"
    );
    let shorter = a.len().min(b.len());
    for c in 0..2 {
        assert!((a.iter().map(|p| p[c] * p[c]).sum::<f64>() / a.len() as f64).sqrt() > 0.01);
    }
    let score = |offset: isize| {
        let ai = offset.max(0) as usize;
        let bi = (-offset).max(0) as usize;
        let len = (a.len() - ai).min(b.len() - bi);
        assert!(len >= shorter - 4800);
        let mut sums = [0.0; 2];
        for (x, y) in a[ai..ai + len].iter().zip(&b[bi..bi + len]) {
            for c in 0..2 {
                sums[c] += (x[c] - y[c]).powi(2);
            }
        }
        [
            (sums[0] / len as f64).sqrt(),
            (sums[1] / len as f64).sqrt(),
            ((sums[0] + sums[1]) / (len * 2) as f64).sqrt(),
        ]
    };
    let zero = score(0);
    if zero.iter().all(|v| *v <= 0.0001) {
        return zero;
    }
    let mut best = zero;
    for offset in -4800..=4800 {
        let current = score(offset);
        if current[2] < best[2] {
            best = current;
        }
        if current.iter().all(|v| *v <= 0.0001) {
            return current;
        }
    }
    best
}
#[test]
fn stereo_comparator_rejects_nonfinite_appended_tail_and_insufficient_overlap() {
    let mut valid = Vec::new();
    for n in 0..9600 {
        for hz in [437.0, 659.0] {
            let v =
                (0.1 * (2.0 * std::f32::consts::PI * hz * n as f32 / 48000.0).sin()).to_le_bytes();
            valid.extend(v);
        }
    }
    assert_eq!(aligned_stereo(&valid, &valid), [0.0; 3]);
    let mut tail = valid.clone();
    tail.extend(vec![0; 4801 * 8]);
    assert!(std::panic::catch_unwind(|| aligned_stereo(&tail, &valid)).is_err());
    let mut nonfinite = valid.clone();
    nonfinite[..4].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(std::panic::catch_unwind(|| aligned_stereo(&nonfinite, &valid)).is_err());
    assert!(
        std::panic::catch_unwind(|| aligned_stereo(&valid[..1000 * 8], &valid[..1000 * 8]))
            .is_err()
    );
}
pub fn independent_film(native: &Native, root: &Path, generation: &str, start: u64) -> PathBuf {
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(media_reference("reference-film-provenance.json")).unwrap(),
    )
    .unwrap();
    let intent = if start == 0 { "full" } else { "partial" };
    let record = manifest["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["generation"] == generation && r["intent"] == intent)
        .unwrap();
    let raw = root.join(format!("independent-{generation}-full.rgba"));
    let mut pixels = Vec::new();
    for time in (0..800).step_by(100) {
        pixels.extend(
            std::fs::read(media_reference(&format!("{generation}-{time:04}.rgba"))).unwrap(),
        );
    }
    std::fs::write(&raw, pixels).unwrap();
    let output = root.join(format!("independent-{generation}-{intent}.mp4"));
    let mut args: Vec<String> = serde_json::from_value(record["command"].clone()).unwrap();
    args.remove(0);
    for arg in &mut args {
        if arg.ends_with("/source.wav") {
            *arg = media_reference("source.wav").to_str().unwrap().into();
        } else if arg.ends_with(&format!("/{generation}-full.rgba")) {
            *arg = raw.to_str().unwrap().into();
        }
    }
    *args.last_mut().unwrap() = output.to_str().unwrap().into();
    let result = Command::new(&native.ffmpeg).args(args).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    output
}
pub fn decoded_movie(native: &Native, path: &Path) -> (Vec<u8>, Vec<u8>) {
    let result = Command::new(&native.ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-map", "0:v:0", "-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    (result.stdout, native.decode(path, true, None))
}
