//! Separate milestone-one evidence. The flat-scene reference/report stays unchanged.
use super::*;
use crate::evaluated_scene::{EvaluatedVisualSource, evaluate_layer_affine};

mod fixture {
    use crate as opencut_editor_core;
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/rule_card.rs"
    ));
}

const FONT_HASH: &str = "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280";
const REFERENCE_NAMES: [&str; 5] = [
    "original.rgb",
    "moved.rgb",
    "audio.f32le",
    "original.plan",
    "moved.plan",
];

#[derive(Debug, Deserialize, Serialize)]
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
    // Embedded content makes the complete reference set one atomic file. Keys are
    // fixed portable fixture-relative names, never resource input to a renderer.
    files: BTreeMap<String, Reference>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    sha256: String,
    bytes: Vec<u8>,
}

fn reference_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rule-card/references.json")
}

fn recipe_hash() -> String {
    hash_bytes(include_bytes!(
        "../../../tests/fixtures/rule-card/recipe.json"
    ))
}

fn validate(references: &References) -> Result<(), String> {
    if references.schema_version != 1
        || references.fixture_id != "rule-card-av-v1"
        || references.recipe_sha256 != recipe_hash()
        || references.font_sha256 != FONT_HASH
        || references.canvas != [WIDTH, HEIGHT, FPS]
        || references.timestamps_ms != SAMPLE_TIMESTAMPS_MS
        || references.duration_ms != DURATION_MS
        || references.audio_sample_rate_hz != AUDIO_SAMPLE_RATE_HZ
        || references.files.len() != REFERENCE_NAMES.len()
    {
        return Err("invalid rule-card identity or metadata".into());
    }
    for name in REFERENCE_NAMES {
        safe_reference_path(reference_path().parent().unwrap(), name)?;
        let reference = references
            .files
            .get(name)
            .ok_or("missing rule-card reference")?;
        if reference.bytes.is_empty()
            || reference.bytes.len() > 1_048_576
            || hash_bytes(&reference.bytes) != reference.sha256
        {
            return Err("invalid rule-card reference hash or size".into());
        }
        if name.ends_with(".rgb") && reference.bytes.len() != (WIDTH * HEIGHT * 3) as usize {
            return Err("invalid rule-card frame length".into());
        }
        if name.ends_with(".f32le") {
            let pcm = bytes_to_f32(&reference.bytes)?;
            if !(48000..=52800).contains(&pcm.len()) || pcm.iter().all(|s| s.abs() < 0.001) {
                return Err("invalid or silent rule-card audio".into());
            }
        }
        if name.ends_with(".plan") && std::str::from_utf8(&reference.bytes).is_err() {
            return Err("invalid semantic text".into());
        }
    }
    Ok(())
}

fn load() -> References {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(reference_path())
        .expect("open reviewed rule-card references")
        .take(4_194_305)
        .read_to_end(&mut bytes)
        .expect("read bounded rule-card reference envelope");
    assert!(
        bytes.len() <= 4_194_304,
        "rule-card reference envelope is bounded"
    );
    let references: References =
        serde_json::from_slice(&bytes).expect("strict rule-card reference envelope");
    validate(&references).expect("validate rule-card references before rendering");
    references
}

fn assert_oracle(project: &Project, moved: bool) {
    let evaluated = evaluate_project(project, WIDTH, HEIGHT, FPS).unwrap();
    let scene = &evaluated.scene;
    assert_eq!(scene.visual_layers.len(), 18);
    assert_eq!(scene.audio_layers.len(), 1);
    assert_eq!(
        evaluate_project(project, WIDTH, HEIGHT, FPS).unwrap(),
        evaluated
    );
    for (index, layer) in scene.visual_layers.iter().enumerate() {
        let card = index / 6;
        let child = index % 6;
        let (x, y) = [
            (0., 0.),
            (0., 0.),
            (8., 5.),
            (8., 18.),
            (8., 32.),
            (40., 5.),
        ][child];
        let affine = evaluate_layer_affine(layer, (8, 8), (WIDTH, HEIGHT)).unwrap();
        assert_eq!(
            affine.matrix,
            [
                1.,
                0.,
                0.,
                1.,
                4. + 44. * card as f64 + x + if moved { 6. } else { 0. },
                8. + 4. * card as f64 + y + if moved { 4. } else { 0. }
            ],
            "independent child translation/order oracle at layer {index}"
        );
        if child == 0 {
            assert_eq!(affine.opacity, [1., 0.8, 0.6][card]);
        }
        if let EvaluatedVisualSource::Text(text) = &layer.source {
            let expected = match child {
                2 => ["1", "2", "3"][card],
                3 => ["Plan", "Build", "Check"][card],
                4 => ["Think", "Make", "Test"][card],
                _ => panic!("unexpected text layer"),
            };
            assert_eq!(text.text, expected);
        }
        if let EvaluatedVisualSource::Media { asset_id, .. } = &layer.source {
            assert_eq!(child, 5);
            assert_eq!(asset_id, &project.assets[card].id);
        }
    }
}

fn semantic(project: &Project) -> Vec<u8> {
    let mut text = format!(
        "{:#?}\n",
        evaluate_project(project, WIDTH, HEIGHT, FPS).unwrap().scene
    );
    for (index, asset) in project.assets.iter().enumerate() {
        text = text.replace(&asset.id, &format!("asset-{index}"));
    }
    for (index, track) in project.tracks.iter().enumerate() {
        text = text.replace(&track.id, &format!("track-{index}"));
    }
    text.into_bytes()
}

struct Outputs {
    frames: Vec<Vec<u8>>,
    range_frames: Vec<Vec<u8>>,
    export_frames: Vec<Vec<u8>>,
    audio: Vec<f32>,
    export_audio: Vec<f32>,
}

fn render(tools: &NativeTools, f: &fixture::Fixture) -> Outputs {
    let project = f.project();
    let root = f.core.paths().project_dir(&f.id).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    renderer.readiness().unwrap();
    let frames = SAMPLE_TIMESTAMPS_MS
        .into_iter()
        .map(|at| {
            let artifact = renderer.render_preview(&project, &root, at).unwrap();
            decode_rgb_frame(&tools.ffmpeg, &root.join(artifact.relative_path), 0)
        })
        .collect();
    let range = renderer
        .render_preview_range(
            &project,
            &root,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: WIDTH,
                height: HEIGHT,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let range_path = root.join(range.relative_path);
    let export_path = f
        .core
        .paths()
        .exports_root()
        .join(format!("rule-card-{}.mp4", project.revision));
    renderer
        .export_video(
            &project,
            &root,
            ExportOptions {
                output: &export_path,
                width: WIDTH,
                height: HEIGHT,
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
            .map(|at| decode_rgb_frame(&tools.ffmpeg, &range_path, at))
            .collect(),
        export_frames: SAMPLE_TIMESTAMPS_MS
            .into_iter()
            .map(|at| decode_rgb_frame(&tools.ffmpeg, &export_path, at))
            .collect(),
        audio: decode_mono_f32(&tools.ffmpeg, &range_path),
        export_audio: decode_mono_f32(&tools.ffmpeg, &export_path),
    }
}

fn compare(outputs: &Outputs, references: &References, moved: bool) {
    let frame = &references.files[if moved { "moved.rgb" } else { "original.rgb" }].bytes;
    for actual in outputs
        .frames
        .iter()
        .chain(&outputs.range_frames)
        .chain(&outputs.export_frames)
    {
        let ssim = structural_similarity(frame, actual).unwrap();
        assert!(ssim >= SSIM_MINIMUM, "rule-card visual drift: SSIM {ssim}");
    }
    let audio = bytes_to_f32(&references.files["audio.f32le"].bytes).unwrap();
    for actual in [&outputs.audio, &outputs.export_audio] {
        assert!(
            aligned_rms_error(&audio, actual, (AUDIO_SAMPLE_RATE_HZ / FPS) as usize).unwrap()
                <= PCM_RMS_MAXIMUM,
            "rule-card audio drift"
        );
    }
}

pub(super) fn conformance(tools: &NativeTools) {
    let references = load();
    assert_eq!(
        tools.font_sha256, FONT_HASH,
        "rule-card requires the reviewed font"
    );
    let root = tempdir().unwrap();
    let mut f = fixture::seed(root.path());
    let definition = serde_json::to_value(f.project().components).unwrap();
    for state in 0..5 {
        match state {
            1 => f.move_parent(),
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
        let moved = matches!(state, 1 | 3 | 4);
        let project = f.project();
        assert_oracle(&project, moved);
        assert_eq!(
            serde_json::to_value(&project.components).unwrap(),
            definition
        );
        assert_eq!(
            semantic(&project),
            references.files[if moved { "moved.plan" } else { "original.plan" }].bytes
        );
        let before = serde_json::to_vec(&project).unwrap();
        // Reopening has the same revision as redo; exports are deliberately unique
        // artifacts per invocation while retaining exactly the same project snapshot.
        let old_export = f
            .core
            .paths()
            .exports_root()
            .join(format!("rule-card-{}.mp4", project.revision));
        if old_export.exists() {
            fs::remove_file(old_export).unwrap();
        }
        compare(&render(tools, &f), &references, moved);
        assert_eq!(serde_json::to_vec(&f.project()).unwrap(), before);
    }
}

#[test]
#[ignore = "Explicit review-only recapture: atomically replaces the separate rule-card reference set"]
fn capture_rule_card_references() {
    let tools = configured_native_tools().expect("configure native tools for explicit recapture");
    assert_eq!(tools.font_sha256, FONT_HASH);
    let root = tempdir().unwrap();
    let f = fixture::seed(root.path());
    let mut files = BTreeMap::new();
    let mut captures = Vec::new();
    for moved in [false, true] {
        if moved {
            f.move_parent();
        }
        assert_oracle(&f.project(), moved);
        let outputs = render(&tools, &f);
        let mut add = |name: &str, bytes: Vec<u8>| {
            files.insert(
                name.into(),
                Reference {
                    sha256: hash_bytes(&bytes),
                    bytes,
                },
            );
        };
        add(
            if moved { "moved.rgb" } else { "original.rgb" },
            outputs.frames[0].clone(),
        );
        add(
            if moved { "moved.plan" } else { "original.plan" },
            semantic(&f.project()),
        );
        if !moved {
            add(
                "audio.f32le",
                outputs.audio.iter().flat_map(|v| v.to_le_bytes()).collect(),
            );
        }
        captures.push((moved, outputs));
    }
    let references = References {
        schema_version: 1,
        fixture_id: "rule-card-av-v1".into(),
        recipe_sha256: recipe_hash(),
        font_sha256: FONT_HASH.into(),
        canvas: [WIDTH, HEIGHT, FPS],
        timestamps_ms: SAMPLE_TIMESTAMPS_MS,
        duration_ms: DURATION_MS,
        audio_sample_rate_hz: AUDIO_SAMPLE_RATE_HZ,
        files,
    };
    validate(&references).unwrap();
    for (moved, outputs) in &captures {
        compare(outputs, &references, *moved);
    }
    let mut file = tempfile::NamedTempFile::new_in(reference_path().parent().unwrap()).unwrap();
    file.write_all(&serde_json::to_vec(&references).unwrap())
        .unwrap();
    file.as_file().sync_all().unwrap();
    file.persist(reference_path()).unwrap();
    conformance(&tools);
}

#[test]
fn rule_card_oracle_detects_slot_transform_and_order_drift() {
    let root = tempdir().unwrap();
    let f = fixture::seed(root.path());
    assert_oracle(&f.project(), false);
    for property in ["text", "transform", "order"] {
        let mut project = f.project();
        let local = &mut project.components[0].tracks[0].items;
        match property {
            "text" => {
                if let crate::TimelineItem::ComponentInstance(i) = project
                    .tracks
                    .iter_mut()
                    .flat_map(|t| &mut t.items)
                    .find(|i| matches!(i, crate::TimelineItem::ComponentInstance(_)))
                    .unwrap()
                {
                    i.slot_values
                        .insert("title".into(), crate::SlotValue::Text("Wrong".into()));
                }
            }
            "transform" => {
                local[2]
                    .visual_properties_mut()
                    .transform2d
                    .as_mut()
                    .unwrap()
                    .position
                    .x += 8.;
            }
            _ => {
                local.swap(0, 1);
                for (i, item) in local.iter_mut().enumerate() {
                    item.visual_properties_mut().stack_order = i as u32;
                }
            }
        }
        assert!(
            std::panic::catch_unwind(|| assert_oracle(&project, false)).is_err(),
            "oracle accepted {property} drift"
        );
    }
}

#[test]
fn rule_card_metadata_and_coordinated_drift_fail_closed() {
    let mut wrong_font = load();
    wrong_font.font_sha256 = "0".repeat(64);
    assert!(validate(&wrong_font).is_err());
    let mut future = load();
    future.schema_version += 1;
    assert!(validate(&future).is_err());
    let mut references = load();
    references.files.get_mut("original.rgb").unwrap().bytes[0] ^= 255;
    assert!(validate(&references).is_err());
    let mut references = load();
    references.timestamps_ms[1] = 1001;
    assert!(validate(&references).is_err());
    let mut references = load();
    let entry = references.files.remove("original.rgb").unwrap();
    references.files.insert("../outside.rgb".into(), entry);
    assert!(validate(&references).is_err());
    let references = load();
    let outputs = Outputs {
        frames: vec![vec![0; (WIDTH * HEIGHT * 3) as usize]],
        range_frames: vec![],
        export_frames: vec![],
        audio: vec![],
        export_audio: vec![],
    };
    assert!(std::panic::catch_unwind(|| compare(&outputs, &references, false)).is_err());
}

#[test]
fn rule_card_missing_dependencies_fail_required_gate() {
    let root = tempdir().unwrap();
    let font = root.path().join("readable-font.ttf");
    fs::write(&font, b"tool startup must fail before font parsing").unwrap();
    let output = Command::new(env::current_exe().unwrap())
        .args([
            "--exact",
            "renderer::golden::native_golden_render_conformance",
        ])
        .env("OPENCUT_GOLDEN_REQUIRED", "1")
        .env("OPENCUT_TEST_FONT_PATH", font)
        .env("OPENCUT_FFMPEG_PATH", root.path().join("missing-ffmpeg"))
        .env("OPENCUT_FFPROBE_PATH", root.path().join("missing-ffprobe"))
        .env_remove("OPENCUT_UPDATE_GOLDENS")
        .env_remove("OPENCUT_CAPTURE_GOLDENS_TO")
        .env_remove("OPENCUT_GOLDEN_REPORT_PATH")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("cannot start configured tool"));
}
