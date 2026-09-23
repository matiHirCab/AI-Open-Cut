//! Separately reviewed static rules-screen references; existing golden baseline is unchanged.
use super::*;
use crate::evaluated_scene::EvaluatedVisualSource;

mod fixture {
    use crate as opencut_editor_core;
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/rules_screen.rs"
    ));
}

const FONT_HASH: &str = "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280";
const SIZES: [(u32, u32); 3] = [(960, 540), (1280, 720), (1920, 1080)];

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct References {
    schema_version: u32,
    fixture_id: String,
    recipe_sha256: String,
    font_sha256: String,
    canvas: [u32; 3],
    timestamps_ms: [u64; 3],
    duration_ms: u64,
    audio_sample_rate_hz: u32,
    files: BTreeMap<String, Reference>,
    plans: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    sha256: String,
    bytes: Vec<u8>,
}

fn reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules-screen/references.json")
}

fn recipe_hash() -> String {
    hash_bytes(include_bytes!(
        "../../../tests/fixtures/rules-screen/recipe.json"
    ))
}

fn key(edited: bool, width: u32, height: u32) -> String {
    format!(
        "{}-{width}x{height}",
        if edited { "edited" } else { "original" }
    )
}

fn expected_names() -> BTreeSet<String> {
    std::iter::once("audio.f32le".to_owned())
        .chain(SIZES.into_iter().flat_map(|(width, height)| {
            [false, true]
                .into_iter()
                .map(move |edited| format!("{}.png", key(edited, width, height)))
        }))
        .collect()
}

fn validate(references: &References) -> Result<(), String> {
    if references.schema_version != 1
        || references.fixture_id != "rules-screen-av-v1"
        || references.recipe_sha256 != recipe_hash()
        || references.font_sha256 != FONT_HASH
        || references.canvas != [1920, 1080, FPS]
        || references.timestamps_ms != SAMPLE_TIMESTAMPS_MS
        || references.duration_ms != DURATION_MS
        || references.audio_sample_rate_hz != AUDIO_SAMPLE_RATE_HZ
        || references.files.keys().cloned().collect::<BTreeSet<_>>() != expected_names()
        || references.plans.len() != 6
    {
        return Err("invalid rules-screen reference identity".into());
    }
    for (name, file) in &references.files {
        if file.bytes.is_empty()
            || file.bytes.len() > 4_194_304
            || file.sha256 != hash_bytes(&file.bytes)
        {
            return Err(format!("invalid rules-screen reference {name}"));
        }
        if name.ends_with(".png") && !file.bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(format!("invalid rules-screen PNG {name}"));
        }
        if name.ends_with(".f32le") {
            let samples = bytes_to_f32(&file.bytes)?;
            if !(48000..=52800).contains(&samples.len())
                || samples.iter().all(|sample| sample.abs() < 0.001)
            {
                return Err("invalid rules-screen audio".into());
            }
        }
    }
    for (width, height) in SIZES {
        for edited in [false, true] {
            let plan = references
                .plans
                .get(&key(edited, width, height))
                .ok_or("missing rules-screen semantic plan")?;
            if plan.len() != 64 || !plan.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("invalid rules-screen semantic hash".into());
            }
        }
    }
    Ok(())
}

fn load() -> References {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(reference_path())
        .expect("open reviewed rules-screen references")
        .take(16_777_217)
        .read_to_end(&mut bytes)
        .expect("read bounded rules-screen references");
    assert!(
        bytes.len() <= 16_777_216,
        "rules-screen references are bounded"
    );
    let references: References =
        serde_json::from_slice(&bytes).expect("strict rules-screen references");
    validate(&references).expect("validate rules-screen references before rendering");
    references
}

fn semantic(f: &fixture::Fixture) -> String {
    let project = f.project();
    let evaluated = evaluate_project(&project, f.width, f.height, FPS).unwrap();
    assert_eq!(evaluated.scene.visual_layers.len(), 18);
    assert_eq!(evaluated.scene.audio_layers.len(), 1);
    assert_eq!(
        evaluate_project(&project, f.width, f.height, FPS).unwrap(),
        evaluated
    );
    let words: Vec<_> = evaluated
        .scene
        .visual_layers
        .iter()
        .filter_map(|layer| {
            if let EvaluatedVisualSource::Text(text) = &layer.source {
                Some(text.text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        words,
        ["EVERY.", "EVERY.", "SINGLE.", "SINGLE.", "ONE.", "ONE."]
    );
    let mut plan = format!("{:#?}", evaluated.scene);
    for (index, asset) in project.assets.iter().enumerate() {
        plan = plan.replace(&asset.id, &format!("asset-{index}"));
    }
    for (index, track) in project.tracks.iter().enumerate() {
        plan = plan.replace(&track.id, &format!("track-{index}"));
    }
    for (index, item) in project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .enumerate()
    {
        plan = plan.replace(item.id(), &format!("item-{index}"));
    }
    hash_bytes(plan.as_bytes())
}

#[test]
fn rules_screen_semantic_reference_ignores_generated_ids() {
    let left_root = tempdir().unwrap();
    let right_root = tempdir().unwrap();
    let left = fixture::seed_at(left_root.path(), 960, 540);
    let right = fixture::seed_at(right_root.path(), 960, 540);
    assert_eq!(semantic(&left), semantic(&right));
    left.resize_impact_word();
    right.resize_impact_word();
    assert_eq!(semantic(&left), semantic(&right));
}

fn decode_sized_rgb_frame(
    ffmpeg: &Path,
    path: &Path,
    time_ms: u64,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let output = Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{:.3}", time_ms as f64 / 1_000.0))
        .arg("-i")
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
        .expect("decode rules-screen RGB frame");
    assert!(output.status.success(), "rules-screen RGB decode failed");
    assert_eq!(output.stdout.len(), (width * height * 3) as usize);
    output.stdout
}

struct Outputs {
    frames: Vec<Vec<u8>>,
    range_frames: Vec<Vec<u8>>,
    export_frames: Vec<Vec<u8>>,
    audio: Vec<f32>,
    export_audio: Vec<f32>,
}

fn render(tools: &NativeTools, f: &fixture::Fixture, state: usize) -> Outputs {
    let project = f.project();
    let project_dir = f.core.paths().project_dir(&f.id).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    renderer.readiness().unwrap();
    let mut frames = Vec::new();
    for time in SAMPLE_TIMESTAMPS_MS {
        let preview = renderer
            .render_preview(&project, &project_dir, time)
            .unwrap();
        let path = project_dir.join(preview.relative_path);
        frames.push(decode_sized_rgb_frame(
            &tools.ffmpeg,
            &path,
            0,
            f.width,
            f.height,
        ));
    }
    let range = renderer
        .render_preview_range(
            &project,
            &project_dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: f.width,
                height: f.height,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let range_path = project_dir.join(range.relative_path);
    let export_path = f
        .core
        .paths()
        .exports_root()
        .join(format!("rules-{state}-{}x{}.mp4", f.width, f.height));
    renderer
        .export_video(
            &project,
            &project_dir,
            ExportOptions {
                output: &export_path,
                width: f.width,
                height: f.height,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    for path in [&range_path, &export_path] {
        assert!(
            renderer
                .probe(path)
                .unwrap()
                .duration_ms
                .unwrap()
                .abs_diff(DURATION_MS)
                <= DURATION_MS / u64::from(FPS)
        );
    }
    Outputs {
        frames,
        range_frames: SAMPLE_TIMESTAMPS_MS
            .into_iter()
            .map(|time| decode_sized_rgb_frame(&tools.ffmpeg, &range_path, time, f.width, f.height))
            .collect(),
        export_frames: SAMPLE_TIMESTAMPS_MS
            .into_iter()
            .map(|time| {
                decode_sized_rgb_frame(&tools.ffmpeg, &export_path, time, f.width, f.height)
            })
            .collect(),
        audio: decode_mono_f32(&tools.ffmpeg, &range_path),
        export_audio: decode_mono_f32(&tools.ffmpeg, &export_path),
    }
}

fn compare(
    tools: &NativeTools,
    outputs: &Outputs,
    references: &References,
    edited: bool,
    width: u32,
    height: u32,
) {
    let name = format!("{}.png", key(edited, width, height));
    let root = tempdir().unwrap();
    let path = root.path().join(&name);
    fs::write(&path, &references.files[&name].bytes).unwrap();
    let expected = decode_sized_rgb_frame(&tools.ffmpeg, &path, 0, width, height);
    assert_eq!(expected.len(), (width * height * 3) as usize);
    for frame in outputs
        .frames
        .iter()
        .chain(&outputs.range_frames)
        .chain(&outputs.export_frames)
    {
        let ssim = structural_similarity(&expected, frame).unwrap();
        assert!(
            ssim >= SSIM_MINIMUM,
            "rules-screen {name} visual drift: SSIM {ssim}"
        );
    }
    let audio = bytes_to_f32(&references.files["audio.f32le"].bytes).unwrap();
    for actual in [&outputs.audio, &outputs.export_audio] {
        assert!(
            aligned_rms_error(&audio, actual, (AUDIO_SAMPLE_RATE_HZ / FPS) as usize).unwrap()
                <= PCM_RMS_MAXIMUM,
            "rules-screen audio drift"
        );
    }
}

pub(super) fn conformance(tools: &NativeTools) {
    let references = load();
    assert_eq!(
        tools.font_sha256, FONT_HASH,
        "rules-screen requires the reviewed font"
    );
    for (width, height) in SIZES {
        let root = tempdir().unwrap();
        let mut f = if width == 1920 {
            fixture::seed(root.path())
        } else {
            fixture::seed_at(root.path(), width, height)
        };
        for state in 0..5 {
            match state {
                1 => f.resize_impact_word(),
                2 => {
                    f.core.undo(&f.id, f.project().revision).unwrap();
                }
                3 => {
                    f.core.redo(&f.id, f.project().revision).unwrap();
                }
                4 => {
                    f.core = crate::EditorCore::new(f.core.paths().clone());
                }
                _ => {}
            }
            let edited = matches!(state, 1 | 3 | 4);
            assert_eq!(semantic(&f), references.plans[&key(edited, width, height)]);
            let before = serde_json::to_vec(&f.project()).unwrap();
            let outputs = render(tools, &f, state);
            compare(tools, &outputs, &references, edited, width, height);
            assert_eq!(serde_json::to_vec(&f.project()).unwrap(), before);
        }
    }
}

#[test]
#[ignore = "Explicit review-only rules-screen reference capture"]
fn capture_rules_screen_references() {
    let tools = configured_native_tools().expect("configure native tools for explicit capture");
    assert_eq!(tools.font_sha256, FONT_HASH);
    let mut files = BTreeMap::new();
    let mut plans = BTreeMap::new();
    for (width, height) in SIZES {
        let root = tempdir().unwrap();
        let f = if width == 1920 {
            fixture::seed(root.path())
        } else {
            fixture::seed_at(root.path(), width, height)
        };
        for edited in [false, true] {
            if edited {
                f.resize_impact_word();
            }
            eprintln!("capturing rules-screen {}", key(edited, width, height));
            plans.insert(key(edited, width, height), semantic(&f));
            let project = f.project();
            let project_dir = f.core.paths().project_dir(&f.id).unwrap();
            let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
            let preview = renderer.render_preview(&project, &project_dir, 0).unwrap();
            let png = fs::read(project_dir.join(preview.relative_path)).unwrap();
            let name = format!("{}.png", key(edited, width, height));
            files.insert(
                name,
                Reference {
                    sha256: hash_bytes(&png),
                    bytes: png,
                },
            );
            if !edited && width == 960 {
                let range = renderer
                    .render_preview_range(
                        &project,
                        &project_dir,
                        PreviewRangeOptions {
                            start_ms: 0,
                            end_ms: DURATION_MS,
                            width,
                            height,
                            fps: FPS,
                            include_audio: true,
                        },
                        |_| {},
                    )
                    .unwrap();
                let samples =
                    decode_mono_f32(&tools.ffmpeg, &project_dir.join(range.relative_path));
                let bytes: Vec<u8> = samples.iter().flat_map(|v| v.to_le_bytes()).collect();
                files.insert(
                    "audio.f32le".into(),
                    Reference {
                        sha256: hash_bytes(&bytes),
                        bytes,
                    },
                );
            }
        }
    }
    let references = References {
        schema_version: 1,
        fixture_id: "rules-screen-av-v1".into(),
        recipe_sha256: recipe_hash(),
        font_sha256: FONT_HASH.into(),
        canvas: [1920, 1080, FPS],
        timestamps_ms: SAMPLE_TIMESTAMPS_MS,
        duration_ms: DURATION_MS,
        audio_sample_rate_hz: AUDIO_SAMPLE_RATE_HZ,
        files,
        plans,
    };
    validate(&references).unwrap();
    let mut file = tempfile::NamedTempFile::new_in(reference_path().parent().unwrap()).unwrap();
    file.write_all(&serde_json::to_vec(&references).unwrap())
        .unwrap();
    file.as_file().sync_all().unwrap();
    file.persist(reference_path()).unwrap();
}
