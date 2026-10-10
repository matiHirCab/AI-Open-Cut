#[path = "support/reference_scene.rs"]
mod fixture;
#[path = "support/release_measurement.rs"]
mod measurement;

use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, ErrorCode, ExportOptions, PathPolicy,
    PreviewRangeOptions, Project, Renderer,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

const RECIPE_SHA: &str = "41deb91f35ccbbf379e7d3ffd0e4d402077fbaa37bee3a9253363fe8221e9cc0";
const FONTS: [(&str, &str); 4] = [
    (
        "DejaVuSans.ttf",
        "7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954",
    ),
    (
        "DejaVuSans-Bold.ttf",
        "e6476c1b80502924294eed40894c5b18e06c181444ca953e5334262df9c27724",
    ),
    (
        "DejaVuSans-Oblique.ttf",
        "4af75fa16ee6d3ad43e1ecec41862c24954af26a55c6bb1ebb27bd486a50f5f4",
    ),
    (
        "DejaVuSans-BoldOblique.ttf",
        "eb436dca0c2594b73d8b603b892e374fdfd8d885d25ffb4f18df4c4c0b49e50f",
    ),
];
fn required(name: &str) -> PathBuf {
    let value =
        env::var_os(name).unwrap_or_else(|| panic!("Required native release input: {name}"));
    let path = PathBuf::from(value);
    assert!(
        path.is_absolute(),
        "Native release paths must be absolute: {name}"
    );
    path
}
fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
fn media_command(tool: &Path, args: &[&str], paths: &[&Path]) -> std::process::Output {
    let output = Command::new(tool).args(args).args(paths).output().unwrap();
    assert!(
        output.status.success(),
        "actual media command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn decode(tool: &Path, path: &Path, audio: bool) -> Vec<u8> {
    let mut command = Command::new(tool);
    command.args(["-v", "error", "-i"]).arg(path);
    if audio {
        command.args(["-vn", "-f", "f32le", "-ac", "2", "-ar", "48000", "-"]);
    } else {
        command.args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "actual decode failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
fn gold(rgb: &[u8]) -> bool {
    let offset = (89 * 192 + 127) * 3;
    rgb.len() == 192 * 108 * 3
        && rgb[offset] >= 250
        && (i16::from(rgb[offset + 1]) - 204).abs() <= 3
        && rgb[offset + 2] <= 3
}
fn voice_windows(pcm: &[u8]) -> bool {
    if pcm.len() != 6000 * 48 * 2 * 4 {
        return false;
    }
    let energy = |ms: usize| {
        let square = (ms * 48..(ms + 10) * 48)
            .map(|frame| {
                let value = f32::from_le_bytes(pcm[frame * 8..frame * 8 + 4].try_into().unwrap());
                f64::from(value).powi(2)
            })
            .sum::<f64>();
        (square / 480.0).sqrt()
    };
    [0, 450, 950, 1450, 2000, 3000, 4000, 5100]
        .into_iter()
        .all(|ms| energy(ms) == 0.0)
        && [550, 1050, 1550, 2450, 3250, 4350]
            .into_iter()
            .all(|ms| energy(ms) > 0.06)
}
fn probe(tool: &Path, path: &Path) -> Value {
    let output = media_command(
        tool,
        &[
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ],
        &[path],
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let streams = metadata["streams"].as_array().unwrap();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .unwrap();
    let audio = streams
        .iter()
        .find(|stream| stream["codec_type"] == "audio")
        .unwrap();
    assert_eq!(video["codec_name"], "h264");
    assert_eq!(video["width"], 192);
    assert_eq!(video["height"], 108);
    assert_eq!(video["avg_frame_rate"], "10/1");
    assert_eq!(video["nb_frames"], "60");
    assert_eq!(audio["codec_name"], "aac");
    assert_eq!(audio["channels"], 2);
    assert_eq!(audio["sample_rate"], "48000");
    let audio_duration = audio["duration"].as_str().unwrap().parse::<f64>().unwrap();
    assert!(audio_duration.is_finite() && (audio_duration - 6.0).abs() <= 0.1);
    let duration = metadata["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((duration - 6.0).abs() <= 0.1);
    metadata
}
fn inventory(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(directory: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() == ".lock" {
                continue;
            }
            if path.is_dir() {
                visit(&path, result);
            } else {
                result.insert(path.clone(), fs::read(&path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(directory, &mut result);
    result
}
fn state_content(project: &Project) -> Value {
    let mut value = serde_json::to_value(project).unwrap();
    value.as_object_mut().unwrap().remove("revision");
    value.as_object_mut().unwrap().remove("updatedAtMs");
    value
}
fn renderer(ffmpeg: &Path, ffprobe: &Path, font: &Path) -> Renderer {
    Renderer::new(ffmpeg, ffprobe, Some(font.to_path_buf()))
        .with_font_roots([font.parent().unwrap().to_path_buf()])
}

#[test]
#[ignore = "mandatory release driver supplies actual tools, four fonts, fresh report identity and default release build"]
fn default_native_release_scene() {
    assert_eq!(
        env::var("OPENCUT_MOTION_RELEASE_REQUIRED").as_deref(),
        Ok("1")
    );
    if cfg!(debug_assertions) {
        panic!("release workload must be optimized");
    }
    if cfg!(feature = "raster-cache-test-hooks") {
        panic!("default native core cannot use private cache instrumentation");
    }
    let ffmpeg = required("OPENCUT_MOTION_REAL_FFMPEG");
    let ffprobe = required("OPENCUT_FFPROBE_PATH");
    let font = required("OPENCUT_TEST_FONT_PATH");
    let output = required("OPENCUT_MOTION_CORE_EVIDENCE_DIR");
    let nonce = env::var("OPENCUT_MOTION_RUN_NONCE").expect("fresh native run nonce");
    fs::create_dir_all(&output).unwrap();
    assert!(
        !output.join("core-report.json").exists(),
        "native report must be fresh"
    );
    for path in [&ffmpeg, &ffprobe, &font] {
        assert!(
            path.is_file(),
            "actual native dependency absent: {}",
            path.display()
        );
    }
    let fonts = FONTS
        .into_iter()
        .map(|(name, expected)| {
            let actual = hash(&font.parent().unwrap().join(name));
            assert_eq!(actual, expected, "fixed font changed: {name}");
            (name, actual)
        })
        .collect::<BTreeMap<_, _>>();
    let recipe_bytes = include_bytes!("../../../contracts/complete-reference-scene-v1.json");
    assert_eq!(format!("{:x}", Sha256::digest(recipe_bytes)), RECIPE_SHA);
    let recipe = fixture::recipe();
    assert_eq!(recipe["operations"].as_array().unwrap().len(), 67);
    let work = output.join("work");
    assert!(!work.exists());
    let fixture = fixture::seed(&work, true);
    let project = fixture.project();
    assert_eq!(project.schema_version, 44);
    assert_eq!(
        (
            project.settings.width,
            project.settings.height,
            project.settings.fps
        ),
        (192, 108, 10)
    );
    assert_eq!(project.components.len(), 1);
    let mut times = project
        .markers
        .iter()
        .map(|marker| marker.time_ms)
        .collect::<Vec<_>>();
    times.sort();
    assert_eq!(times, [500, 1000, 1500, 2400, 3200, 4300]);
    let directory = fixture.dir();
    fs::create_dir_all(work.join("exports")).unwrap();
    let before = inventory(&directory);
    let voice = project
        .assets
        .iter()
        .find(|asset| {
            matches!(
                asset.origin,
                Some(opencut_editor_core::GeneratedAssetOrigin::SpeechSynthesis(
                    _
                ))
            )
        })
        .unwrap();
    let voice_pcm = decode(&ffmpeg, &directory.join(&voice.project_relative_path), true);
    assert!(voice_windows(&voice_pcm));
    assert!(!voice_windows(&vec![0; voice_pcm.len()]));
    let mut captures = Vec::new();
    let mut frame_bytes = Vec::new();
    for capture in 0..4 {
        let renderer = renderer(&ffmpeg, &ffprobe, &font);
        let sampler = measurement::Sampler::start();
        let clock = Instant::now();
        let cold_before = sampler.final_calls();
        let cold = renderer.render_preview(&project, &directory, 650).unwrap();
        let cold_after = sampler.final_calls();
        assert!(
            cold_after > cold_before,
            "default cold frame must execute native final encoder"
        );
        let cold_path = directory.join(&cold.relative_path);
        let bytes = fs::read(&cold_path).unwrap();
        let warm = renderer.render_preview(&project, &directory, 650).unwrap();
        assert_ne!(cold.relative_path, warm.relative_path);
        assert_eq!(
            fs::read(directory.join(&warm.relative_path)).unwrap(),
            bytes
        );
        assert_eq!(fs::read(&cold_path).unwrap(), bytes);
        let warm_calls = sampler.final_calls() - cold_after;
        if cfg!(target_os = "macos") {
            assert!(
                warm_calls > 0,
                "existing Mach-O identity must retain uncached fallback"
            );
        } else {
            assert_eq!(
                warm_calls, 0,
                "eligible default cached frame must avoid final PNG encoding"
            );
        }
        let rgb = decode(&ffmpeg, &cold_path, false);
        assert!(gold(&rgb));
        let mut absent = rgb.clone();
        absent[(89 * 192 + 127) * 3..(89 * 192 + 127) * 3 + 3].fill(0);
        assert!(!gold(&absent));
        let range = renderer
            .render_preview_range(
                &project,
                &directory,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 6000,
                    width: 192,
                    height: 108,
                    fps: 10,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let range_path = directory.join(&range.relative_path);
        let export_path = work.join("exports").join(format!("capture-{capture}.mp4"));
        renderer
            .export_video(
                &project,
                &directory,
                ExportOptions {
                    output: &export_path,
                    width: 192,
                    height: 108,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        let range_probe = probe(&ffprobe, &range_path);
        let export_probe = probe(&ffprobe, &export_path);
        let left = decode(&ffmpeg, &range_path, true);
        let right = decode(&ffmpeg, &export_path, true);
        assert_eq!(left.len(), right.len());
        assert_eq!(left.len() % 8, 0);
        assert!(
            (283_200..=292_800).contains(&(left.len() / 8)),
            "actual six-second stereo PCM within one video frame"
        );
        let mut energy = 0.0;
        let mut squared = 0.0;
        for (left, right) in left.chunks_exact(4).zip(right.chunks_exact(4)) {
            let left = f64::from(f32::from_le_bytes(left.try_into().unwrap()));
            let right = f64::from(f32::from_le_bytes(right.try_into().unwrap()));
            assert!(left.is_finite() && right.is_finite());
            energy += left * left;
            squared += (left - right).powi(2);
        }
        let count = (left.len() / 4) as f64;
        let rms = (squared / count).sqrt();
        assert!(rms <= 0.0001);
        assert!((energy / count).sqrt() > 0.01);
        let comparison = Command::new(&ffmpeg)
            .args(["-v", "info", "-i"])
            .arg(&range_path)
            .arg("-i")
            .arg(&export_path)
            .args(["-lavfi", "[0:v][1:v]ssim", "-an", "-f", "null", "-"])
            .output()
            .unwrap();
        assert!(comparison.status.success());
        let message = String::from_utf8_lossy(&comparison.stderr);
        let ssim = message
            .rsplit_once("All:")
            .unwrap()
            .1
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(ssim.is_finite() && ssim >= 0.99);
        let elapsed = u64::try_from(clock.elapsed().as_millis()).unwrap();
        let (peak, samples, native) = sampler.finish();
        if capture > 0 {
            assert!(
                peak > 0 && samples > 0 && native > 0,
                "actual native process-tree observations required"
            );
            if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
                assert!(elapsed <= 600_000);
                assert!(peak <= 2_147_483_648);
            }
            captures.push(json!({"elapsedMs":elapsed,"peakResidentBytes":peak,"memorySamples":samples,"nativeMembersObserved":native,"coldFinalCalls":cold_after-cold_before,"warmFinalCalls":warm_calls,"ssim":ssim,"pcmRms":rms,"alignmentOffsetSamples":0,"rangeProbe":range_probe,"exportProbe":export_probe}));
        }
        assert_eq!(
            serde_json::to_value(fixture.project()).unwrap(),
            serde_json::to_value(&project).unwrap()
        );
        frame_bytes = bytes;
        if capture == 3 {
            fs::copy(&cold_path, output.join("frame650.png")).unwrap();
            fs::copy(&export_path, output.join("reference.mp4")).unwrap();
        }
    }
    // Rendering may add disposable outputs; authoritative files/resources remain exact.
    for (path, bytes) in &before {
        assert_eq!(fs::read(path).unwrap(), *bytes);
    }
    let immutable = inventory(&directory);
    for (expected, operation, revision) in [
        (
            ErrorCode::ItemNotFound,
            json!({"operation":"item_set_z_index","itemId":"missing","zIndex":2}),
            project.revision,
        ),
        (
            ErrorCode::InvalidArgument,
            json!({"operation":"component_instance_duplicate","itemId":fixture.aliases["card0"],"offsetMs":0,"slotValues":{"title":{"type":"number","value":1}}}),
            project.revision,
        ),
        (
            ErrorCode::RevisionConflict,
            json!({"operation":"item_set_z_index","itemId":fixture.aliases["grid"],"zIndex":2}),
            project.revision - 1,
        ),
    ] {
        assert_eq!(
            fixture
                .core
                .edit(
                    &fixture.id,
                    revision,
                    serde_json::from_value::<EditOperation>(operation).unwrap()
                )
                .unwrap_err()
                .code,
            expected
        );
        assert_eq!(inventory(&directory), immutable);
    }
    let bad: Vec<BatchEditOperation> = serde_json::from_value(json!([{"operation":"item_set_z_index","itemId":fixture.aliases["grid"],"zIndex":2},{"operation":"delete_item","itemId":"missing"}])).unwrap();
    assert!(
        fixture
            .core
            .edit_batch(&fixture.id, project.revision, bad)
            .is_err()
    );
    assert_eq!(inventory(&directory), immutable);
    let edit: EditOperation = serde_json::from_value(
        json!({"operation":"item_set_z_index","itemId":fixture.aliases["grid"],"zIndex":2}),
    )
    .unwrap();
    fixture
        .core
        .edit(&fixture.id, project.revision, edit)
        .unwrap();
    let edited = fixture.project();
    fixture.core.undo(&fixture.id, edited.revision).unwrap();
    assert_eq!(fixture.project().revision, project.revision + 2);
    assert_eq!(state_content(&fixture.project()), state_content(&project));
    let restored = renderer(&ffmpeg, &ffprobe, &font)
        .render_preview(&fixture.project(), &directory, 650)
        .unwrap();
    assert_eq!(
        fs::read(directory.join(restored.relative_path)).unwrap(),
        frame_bytes
    );
    fixture
        .core
        .redo(&fixture.id, fixture.project().revision)
        .unwrap();
    assert_eq!(fixture.project().revision, project.revision + 3);
    assert_eq!(state_content(&fixture.project()), state_content(&edited));
    fixture
        .core
        .undo(&fixture.id, fixture.project().revision)
        .unwrap();
    let fresh = EditorCore::new(
        PathPolicy::new(
            work.join("projects"),
            [work.join("media")],
            work.join("exports"),
        )
        .unwrap(),
    );
    assert_eq!(
        serde_json::to_value(fresh.get_project(&fixture.id).unwrap()).unwrap(),
        serde_json::to_value(fixture.project()).unwrap()
    );
    let reopened = renderer(&ffmpeg, &ffprobe, &font)
        .render_preview(&fresh.get_project(&fixture.id).unwrap(), &directory, 650)
        .unwrap();
    assert_eq!(
        fs::read(directory.join(reopened.relative_path)).unwrap(),
        frame_bytes
    );
    let invalid_before = inventory(&directory);
    assert_eq!(
        renderer(&ffmpeg, &ffprobe, &font)
            .render_preview(&fixture.project(), &directory, 6001)
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(inventory(&directory), invalid_before);
    let missing = directory.join(&voice.project_relative_path);
    let project_before_missing = fixture.project();
    let source_bytes = fs::read(&missing).unwrap();
    fs::remove_file(&missing).unwrap();
    let missing_before = inventory(&directory);
    assert_eq!(
        fixture.core.get_project(&fixture.id).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert_eq!(
        renderer(&ffmpeg, &ffprobe, &font)
            .render_preview_range(
                &project_before_missing,
                &directory,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 6000,
                    width: 192,
                    height: 108,
                    fps: 10,
                    include_audio: true
                },
                |_| {}
            )
            .unwrap_err()
            .code,
        ErrorCode::FfmpegFailed
    );
    assert_eq!(inventory(&directory), missing_before);
    fs::write(&missing, source_bytes).unwrap();
    let version = |tool: &Path| {
        String::from_utf8(media_command(tool, &["-version"], &[]).stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .to_owned()
    };
    let report = json!({"version":1,"runNonce":nonce,"fixtureSha256":RECIPE_SHA,"schemaVersion":44,"platform":env::consts::OS,"architecture":env::consts::ARCH,"buildProfile":"release","instrumented":false,"cacheMode":if cfg!(target_os="macos"){"native_identity_bypass"}else{"native_reuse"},"fonts":fonts,"tools":{"ffmpeg":version(&ffmpeg),"ffprobe":version(&ffprobe)},"warmupCaptures":1,"measuredCaptures":3,"sampleIntervalMs":5,"memoryScope":"isolated_test_process_tree","memoryAggregation":"maximum_sampled_resident","timingScope":"direct_whole_capture_including_preparation_and_oracles","captures":captures,"witnesses":{"groups":10,"operations":67,"cueTimesMs":times,"goldNegative":true,"voiceNegative":true,"stateFailures":true,"historyReopen":true}});
    fs::write(
        output.join("core-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "actual default native release report: {}",
        output.join("core-report.json").display()
    );
}
