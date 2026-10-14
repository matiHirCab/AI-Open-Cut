//! Issue 181's small original benchmark; no production renderer or contract changes.
use super::*;
use crate::{EditOperation, EditorCore, MediaProbeFacts, PathPolicy};
use serde_json::{Value, json};

const RECIPE: &[u8] = include_bytes!("../../../tests/fixtures/illustrated-scene/recipe.json");
const FONT_HASH: &str = "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280";
const CANVAS: [u32; 3] = [1920, 1080, 30];
const ROLES: [&str; 7] = [
    "background",
    "device_frame",
    "screen_panel",
    "label_panel",
    "accent",
    "artwork",
    "label",
];

fn recipe() -> Value {
    serde_json::from_slice(RECIPE).unwrap()
}

pub(super) fn strategy_fixture() -> (tempfile::TempDir, EditorCore, String) {
    let fixture = Fixture::new(&recipe());
    (fixture.root, fixture.core, fixture.id)
}

pub(super) fn strategy_verify_pixels(bytes: &[u8], tolerance: i16) {
    assert_pixels(&recipe(), bytes, tolerance);
}

fn validate(value: &Value) -> Result<(), &'static str> {
    if value["version"] != 1
        || value["fixtureId"] != "illustrated-hero-card-v1"
        || value["canvas"] != json!({"width":1920,"height":1080,"fps":30})
        || value["durationMs"] != 1000
        || value["sampleTimesMs"] != json!([0, 500, 966])
        || value["rangeMs"] != json!([0, 100])
        || value["caps"]
            != json!({"visualLeaves":16,"instances":1,"textScalars":64,"artworkCount":1,"artworkDimension":64,"pathCommands":128,"animatedChannels":0})
    {
        return Err("illustrated fixture identity/workload/caps differ");
    }
    let roles = value["roles"].as_array().ok_or("missing roles")?;
    if roles.len() != ROLES.len()
        || value["inputs"]["text"]
            .as_str()
            .is_none_or(|s| s.is_empty() || s.chars().count() > 64)
        || value["inputs"]["artwork"] != "original-64x64-left-white-right-red-ppm"
    {
        return Err("invalid illustrated inputs/count");
    }
    for (role, expected) in roles.iter().zip(ROLES) {
        if role["id"] != expected
            || !matches!(role["kind"].as_str(), Some("shape" | "text" | "image"))
        {
            return Err("invalid/duplicate native role");
        }
        let bounds = role["box"].as_array().ok_or("missing role box")?;
        if bounds.len() != 4 || bounds.iter().any(|n| n.as_u64().is_none()) {
            return Err("invalid role box");
        }
        let b: Vec<u64> = bounds.iter().map(|n| n.as_u64().unwrap()).collect();
        if b[2] == 0
            || b[3] == 0
            || b[0].checked_add(b[2]).is_none_or(|n| n > 1920)
            || b[1].checked_add(b[3]).is_none_or(|n| n > 1080)
        {
            return Err("role box exceeds canvas");
        }
        let rgb = role["rgb"].as_array().ok_or("missing paint")?;
        if rgb.len() != 3 || rgb.iter().any(|n| n.as_u64().is_none_or(|n| n > 255)) {
            return Err("invalid paint");
        }
    }
    // Authored analytic samples are independent of production evaluation/rasterization.
    let expected = [
        ([100, 100], [24, 38, 61]),
        ([280, 200], [238, 238, 238]),
        ([350, 250], [22, 32, 51]),
        ([1250, 570], [48, 73, 104]),
        ([400, 785], [255, 204, 51]),
        ([650, 400], [255, 255, 255]),
        ([840, 400], [255, 0, 0]),
        ([1145, 500], [24, 38, 61]),
    ];
    let samples = value["independentSamples"]
        .as_array()
        .ok_or("missing samples")?;
    if samples.len() != expected.len()
        || samples.iter().zip(expected).any(|(sample, (point, rgb))| {
            sample["point"] != json!(point) || sample["rgb"] != json!(rgb)
        })
    {
        return Err("independent samples differ");
    }
    let targets = value["targets"].as_array().ok_or("missing targets")?;
    if targets.len() != 3 {
        return Err("expected three later targets");
    }
    for (target, (id, leaves, instances, text, artwork, commands, channels)) in
        targets.iter().zip([
            ("illustrated_device_intro", 24, 1, 128, 1, 256, 8),
            ("sticker_motif_intro", 48, 2, 128, 1, 1024, 16),
            ("role_badge_team_intro", 96, 6, 512, 6, 2048, 32),
        ])
    {
        if target["id"] != id
            || target["implemented"] != false
            || target["durationMs"] != 3000
            || target["roles"].as_array().is_none_or(|r| r.len() != 6)
            || target["roles"].as_array().is_some_and(|roles| {
                roles
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .collect::<BTreeSet<_>>()
                    .len()
                    != 6
            })
            || target["layout"].as_str().is_none_or(str::is_empty)
            || target["caps"]
                != json!({"visualLeaves":leaves,"instances":instances,"textScalars":text,"artworkCount":artwork,"artworkDimension":1024,"pathCommands":commands,"animatedChannels":channels})
        {
            return Err("later target inventory/caps differ");
        }
    }
    Ok(())
}

fn transform(x: u64, y: u64, scale: f64) -> Value {
    json!({"position":{"x":x,"y":y,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":scale,"scaleY":scale,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0,"opacity":1})
}

struct Fixture {
    root: tempfile::TempDir,
    core: EditorCore,
    id: String,
    roles: BTreeMap<String, String>,
}

impl Fixture {
    fn new(value: &Value) -> Self {
        validate(value).expect("validate illustrated inventory before authoring");
        let root = tempdir().unwrap();
        let input = root.path().join("original.ppm");
        let mut image = b"P6\n64 64\n255\n".to_vec();
        for _y in 0..64 {
            for x in 0..64 {
                image.extend(if x < 32 { [255, 255, 255] } else { [255, 0, 0] });
            }
        }
        fs::write(&input, image).unwrap();
        let core = EditorCore::new(
            PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let id = core
            .create_project(
                "Original illustrated hero card v1",
                ProjectSettings {
                    width: CANVAS[0],
                    height: CANVAS[1],
                    fps: CANVAS[2],
                },
            )
            .unwrap()
            .project_id;
        let asset = core
            .import_asset(
                &id,
                0,
                &input,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    format_name: Some("ppm_pipe".into()),
                    video_codec: Some("ppm".into()),
                    video_width: Some(64),
                    video_height: Some(64),
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let track = core
            .get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let mut fixture = Self {
            root,
            core,
            id,
            roles: BTreeMap::new(),
        };
        for role in value["roles"].as_array().unwrap() {
            let b: Vec<u64> = role["box"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap())
                .collect();
            let rgb: Vec<u64> = role["rgb"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap())
                .collect();
            let operation = match role["kind"].as_str().unwrap() {
                "shape" => {
                    json!({"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,
                    "geometry":{"type":"rectangle","width":b[2],"height":b[3]},
                    "fill":{"type":"solid","color":{"r":rgb[0] as f64/255.,"g":rgb[1] as f64/255.,"b":rgb[2] as f64/255.,"a":1}},"stroke":null,"transform2d":transform(b[0],b[1],1.)})
                }
                "text" => {
                    json!({"operation":"add_text","trackId":track,"text":value["inputs"]["text"],"startMs":0,"durationMs":1000,"fontSize":48,"color":"#ffffff","style":{"layout":{"bounds":{"widthPx":440,"heightPx":100},"wrap":"none"}},"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})
                }
                "image" => {
                    json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"durationMs":1000,"sourceInMs":0})
                }
                _ => unreachable!(),
            };
            let added = fixture.edit(operation).changed_ids[0].clone();
            if role["kind"] != "shape" {
                fixture.edit(json!({"operation":"update_item","itemId":added,
                    "transform2d":transform(b[0],b[1],if role["kind"]=="image" {4.} else {1.})}));
            }
            fixture
                .roles
                .insert(role["id"].as_str().unwrap().into(), added);
        }
        fixture
    }
    fn edit(&self, value: Value) -> crate::WriteResult {
        self.core
            .edit(
                &self.id,
                self.project().revision,
                serde_json::from_value::<EditOperation>(value).unwrap(),
            )
            .unwrap()
    }
    fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    fn dir(&self) -> PathBuf {
        self.core.project_directory(&self.id).unwrap()
    }
}

fn decode(tools: &NativeTools, path: &Path, movie: bool) -> Vec<u8> {
    let probe = Command::new(&tools.ffprobe)
        .args([
            "-v",
            "error",
            "-count_frames",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,nb_read_frames",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(probe.status.success(), "illustrated output probe failed");
    let facts: Value = serde_json::from_slice(&probe.stdout).unwrap();
    assert_eq!(facts["streams"][0]["width"], CANVAS[0]);
    assert_eq!(facts["streams"][0]["height"], CANVAS[1]);
    assert_eq!(
        facts["streams"][0]["nb_read_frames"],
        if movie { "3" } else { "1" }
    );
    let mut command = Command::new(&tools.ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path);
    if !movie {
        command.args(["-frames:v", "1"]);
    }
    let output = command
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "illustrated native decode: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout.len(),
        (CANVAS[0] * CANVAS[1] * 3) as usize * if movie { 3 } else { 1 }
    );
    output.stdout
}

fn assert_pixels(value: &Value, bytes: &[u8], tolerance: i16) {
    let size = (CANVAS[0] * CANVAS[1] * 3) as usize;
    assert_eq!(bytes.len() % size, 0);
    for frame in bytes.chunks_exact(size) {
        let label_pixels = (330..430)
            .flat_map(|y| (1230..1670).map(move |x| (y * CANVAS[0] as usize + x) * 3))
            .filter(|offset| frame[*offset..*offset + 3].iter().all(|v| *v >= 240))
            .count();
        assert!(
            label_pixels >= 100,
            "independent label box has no visible white glyphs"
        );
        for sample in value["independentSamples"].as_array().unwrap() {
            let x = sample["point"][0].as_u64().unwrap() as usize;
            let y = sample["point"][1].as_u64().unwrap() as usize;
            for channel in 0..3 {
                let actual = i16::from(frame[(y * CANVAS[0] as usize + x) * 3 + channel]);
                let expected = sample["rgb"][channel].as_i64().unwrap() as i16;
                assert!(
                    (actual - expected).abs() <= tolerance,
                    "independent illustrated sample {}, channel {channel}: {actual} != {expected}",
                    sample["reason"]
                );
            }
        }
    }
}

fn capture(
    tools: &NativeTools,
    renderer: &Renderer,
    fixture: &Fixture,
    value: &Value,
    movie: bool,
    time: u64,
) -> f64 {
    let before = serde_json::to_vec(&fixture.project()).unwrap();
    let project = fixture.project();
    let started = Instant::now();
    let artifact = if movie {
        renderer
            .render_preview_range(
                &project,
                &fixture.dir(),
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 100,
                    width: CANVAS[0],
                    height: CANVAS[1],
                    fps: CANVAS[2],
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap()
    } else {
        renderer
            .render_preview(&project, &fixture.dir(), time)
            .unwrap()
    };
    let elapsed = started.elapsed().as_secs_f64() * 1000.;
    assert!(elapsed.is_finite() && elapsed >= 0.);
    assert_eq!(artifact.text_layouts.len(), 1);
    let text = &artifact.text_layouts[0];
    assert_eq!(text.item_id, fixture.roles["label"]);
    assert_eq!(text.resolved_font_size, 48);
    assert_eq!(text.line_count, 1);
    assert!(
        text.content_width_px > 0.
            && text.content_width_px <= 440.
            && text.content_height_px <= 100.
    );
    let path = fixture.dir().join(&artifact.relative_path);
    assert_pixels(
        value,
        &decode(tools, &path, movie),
        if movie { 8 } else { 2 },
    );
    if movie || time == 500 {
        fs::copy(
            &path,
            fixture.root.path().join(if movie {
                "range0-100.mp4"
            } else {
                "frame500.png"
            }),
        )
        .unwrap();
    }
    assert_eq!(serde_json::to_vec(&fixture.project()).unwrap(), before);
    eprintln!(
        "illustrated_scene_sample intent={} sample={} elapsed_ms={elapsed:.3}",
        if movie { "range" } else { "frame" },
        if movie {
            "[0,100)ms".to_owned()
        } else {
            format!("{time}ms")
        }
    );
    elapsed
}

pub(super) fn conformance(tools: &NativeTools) {
    assert_eq!(tools.font_sha256, FONT_HASH);
    let value = recipe();
    let fixture = Fixture::new(&value);
    let project = fixture.project();
    assert_eq!(
        project.tracks.iter().map(|t| t.items.len()).sum::<usize>(),
        7
    );
    assert_eq!(fixture.roles.len(), 7);
    let new_renderer = || Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let warm = new_renderer();
    for time in [0, 966] {
        capture(tools, &warm, &fixture, &value, false, time);
    }
    let mut observations = Vec::new();
    for movie in [false, true] {
        let fresh: Vec<_> = (0..3)
            .map(|_| capture(tools, &new_renderer(), &fixture, &value, movie, 500))
            .collect();
        let warmup = capture(tools, &warm, &fixture, &value, movie, 500);
        let repeated: Vec<_> = (0..3)
            .map(|_| capture(tools, &warm, &fixture, &value, movie, 500))
            .collect();
        observations.push(json!({"intent":if movie {"range"}else{"frame"},"freshRendererMs":fresh,"warmupMs":warmup,"repeatedRequestMs":repeated}));
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git = |args: &[&str]| {
        let result = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(result.status.success());
        result.stdout
    };
    let binary = env::current_exe().unwrap();
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    let report = json!({"schemaVersion":1,"fixtureId":value["fixtureId"],"canvas":value["canvas"],"durationMs":1000,"frameMs":500,"rangeMs":[0,100],"freshSamples":3,"warmupSamples":1,"repeatedSamples":3,
        "source":{"gitRevision":String::from_utf8(git(&["rev-parse","HEAD"])).unwrap().trim(),"trackedDiffSha256":hash_bytes(&git(&["diff","HEAD","--"])),"recipeSha256":hash_bytes(RECIPE),"driverSha256":hash_bytes(include_bytes!("illustrated_scene.rs")),"testBinarySha256":hash_bytes(&fs::read(binary).unwrap())},
        "environment":{"os":env::consts::OS,"architecture":env::consts::ARCH,"osVersion":System::long_os_version(),"cpu":system.cpus().first().map(|c|c.brand()),"logicalCpus":system.cpus().len(),"totalMemoryBytes":system.total_memory(),"cpuMax":fs::read_to_string("/sys/fs/cgroup/cpu.max").ok().map(|s|s.trim().to_owned()),"memoryMax":fs::read_to_string("/sys/fs/cgroup/memory.max").ok().map(|s|s.trim().to_owned()),"debugAssertions":cfg!(debug_assertions),"ffmpeg":tools.ffmpeg_version,"ffprobe":tools.ffprobe_version,"fontSha256":tools.font_sha256},
        "semantics":{"fresh":"new Renderer; OS filesystem caches not flushed","repeated":"same Renderer, same immutable request; raster and encoded-preview caches eligible","timer":"facade request through returned artifact; independent decode excluded","portableTiming":"report-only; no universal CI threshold"},
        "guards":{"sampledOutputFramePixelWork":268435456,"surfacePixels":16777216,"compositionBytes":1073741824,"outcome":"all fixed workloads admitted; existing guards unchanged"},"outputEvidence":{"frameSha256":hash_bytes(&fs::read(fixture.root.path().join("frame500.png")).unwrap()),"rangeSha256":hash_bytes(&fs::read(fixture.root.path().join("range0-100.mp4")).unwrap()),"independentColorSamplesPerFrame":8,"visibleLabelMinimumWhitePixels":100,"rangeFrameCount":3},"observations":observations});
    eprintln!("illustrated_scene_observation={report}");
    if let Some(destination) = env::var_os("OPENCUT_ILLUSTRATED_REPORT_PATH") {
        fs::write(destination, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    if let Some(destination) = env::var_os("OPENCUT_ILLUSTRATED_ARTIFACT_DIR") {
        fs::create_dir_all(&destination).unwrap();
        for name in ["frame500.png", "range0-100.mp4"] {
            fs::copy(
                fixture.root.path().join(name),
                Path::new(&destination).join(name),
            )
            .unwrap();
        }
    }
    let reopened = EditorCore::new(
        PathPolicy::new(
            fixture.root.path().join("projects"),
            [fixture.root.path()],
            fixture.root.path().join("exports"),
        )
        .unwrap(),
    );
    assert_eq!(
        serde_json::to_vec(&reopened.get_project(&fixture.id).unwrap()).unwrap(),
        serde_json::to_vec(&project).unwrap()
    );
}

#[test]
fn illustrated_inventory_is_bounded_and_original() {
    validate(&recipe()).unwrap();
    let fixture = Fixture::new(&recipe());
    assert_eq!(fixture.roles.len(), 7);
}

#[test]
fn illustrated_inventory_rejects_workload_and_oracle_drift() {
    for (pointer, replacement) in [
        ("/canvas/width", json!(1921)),
        ("/caps/visualLeaves", json!(17)),
        ("/roles/0/box/2", json!(1921)),
        ("/roles/1/id", json!("background")),
        ("/inputs/text", json!("x".repeat(65))),
        ("/independentSamples/0/rgb/0", json!(25)),
        ("/targets/0/implemented", json!(true)),
        ("/targets/2/caps/instances", json!(7)),
        ("/targets/0/roles/1", json!("device_frame")),
        ("/roles/0/box/0", json!(u64::MAX)),
    ] {
        let mut bad = recipe();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(validate(&bad).is_err(), "accepted {pointer}");
    }
    let size = (CANVAS[0] * CANVAS[1] * 3) as usize;
    assert!(std::panic::catch_unwind(|| assert_pixels(&recipe(), &vec![0; size], 2)).is_err());
}

#[test]
fn native_illustrated_scene_benchmark() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}
