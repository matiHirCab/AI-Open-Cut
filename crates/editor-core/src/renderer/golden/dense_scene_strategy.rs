//! Issue187 finite evaluation. All authoring and rendering use existing owners.
use super::*;
use crate::{BatchEditOperation, EditorCore, MediaProbeFacts, PathPolicy};
use serde_json::{Value, json};

const HERO: &[u8] =
    include_bytes!("../../../../../apps/agent-bridge/src/recipes/native-hero-card-v1.json");
const MOTIFS: &[u8] = include_bytes!("../../../../../contracts/native-motif-pack-v1.json");
const TEAM: &[u8] = include_bytes!("../../../../../contracts/native-role-team-v1.json");
const BASELINE: &[u8] = include_bytes!("../../../tests/fixtures/illustrated-scene/recipe.json");
const WORKLOADS: [&str; 7] = [
    "baseline",
    "hero",
    "motif_gallery",
    "row1",
    "row4",
    "grid1",
    "grid4",
];
const PIXELS: usize = 1920 * 1080 * 3;

fn replace(value: &mut Value, ids: &BTreeMap<String, String>) {
    match value {
        Value::String(s) => {
            if let Some(id) = ids.get(s) {
                *s = id.clone();
            }
        }
        Value::Array(values) => {
            for v in values {
                replace(v, ids);
            }
        }
        Value::Object(values) => {
            for v in values.values_mut() {
                replace(v, ids);
            }
        }
        _ => {}
    }
}
struct Fixture {
    root: tempfile::TempDir,
    core: EditorCore,
    id: String,
    name: &'static str,
}
impl Fixture {
    fn new(name: &'static str) -> Self {
        assert!(WORKLOADS.contains(&name));
        if name == "baseline" {
            let (root, core, id) = illustrated_scene::strategy_fixture();
            return Self {
                root,
                core,
                id,
                name,
            };
        }
        let root = tempdir().unwrap();
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
                "Original finite strategy fixture",
                ProjectSettings {
                    width: 1920,
                    height: 1080,
                    fps: 30,
                },
            )
            .unwrap()
            .project_id;
        let track = core
            .get_project(&id)
            .unwrap()
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Overlay)
            .unwrap()
            .id
            .clone();
        let fixture = Self {
            root,
            core,
            id,
            name,
        };
        let mut ids = BTreeMap::from([("$overlay".into(), track)]);
        if name == "motif_gallery" {
            let c: Value = serde_json::from_slice(MOTIFS).unwrap();
            let seeds = c["motifs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["seed"].clone())
                .collect::<Vec<_>>();
            let aliases = fixture.batch(json!(seeds)).aliases;
            for m in c["motifs"].as_array().unwrap() {
                ids.insert(
                    m["componentIdPlaceholder"].as_str().unwrap().into(),
                    aliases[m["phaseAAlias"].as_str().unwrap()].clone(),
                );
            }
            let mut edits = c["motifs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["populate"].clone())
                .collect::<Vec<_>>();
            edits.extend(c["galleryInstances"].as_array().unwrap().iter().cloned());
            let mut edits = json!(edits);
            replace(&mut edits, &ids);
            fixture.batch(edits);
        } else {
            let image = fixture.root.path().join("original.ppm");
            let mut bytes = b"P6\n64 64\n255\n".to_vec();
            for _ in 0..64 {
                for x in 0..64 {
                    bytes.extend(if x < 32 { [255, 255, 255] } else { [255, 0, 0] });
                }
            }
            fs::write(&image, bytes).unwrap();
            let asset = fixture
                .core
                .import_asset(
                    &fixture.id,
                    0,
                    &image,
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
            ids.insert("$artwork".into(), asset);
            let mut edits = if name == "hero" {
                serde_json::from_slice::<Value>(HERO).unwrap()["operations"].clone()
            } else {
                let c: Value = serde_json::from_slice(TEAM).unwrap();
                let layout = if name.starts_with("row") {
                    "row"
                } else {
                    "grid"
                };
                let count = if name.ends_with('1') { "1" } else { "4" };
                let mut edits = vec![c["badgeCreate"].clone(), c["badgeDefineSlots"].clone()];
                edits.extend(
                    c["layouts"][layout][count]
                        .as_array()
                        .unwrap()
                        .iter()
                        .cloned(),
                );
                json!(edits)
            };
            replace(&mut edits, &ids);
            fixture.batch(edits);
        }
        fixture
    }
    fn batch(&self, operations: Value) -> crate::WriteResult {
        self.core
            .edit_batch::<BatchEditOperation>(
                &self.id,
                self.project().revision,
                serde_json::from_value(operations).unwrap(),
            )
            .unwrap()
    }
    fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    fn dir(&self) -> PathBuf {
        self.core.project_directory(&self.id).unwrap()
    }
    fn authoritative(&self) -> BTreeMap<String, String> {
        fn walk(dir: &Path, relative: &Path, out: &mut BTreeMap<String, String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let child = relative.join(entry.file_name());
                if path.is_dir() {
                    walk(&path, &child, out);
                } else {
                    out.insert(
                        child.to_string_lossy().replace("\\", "/"),
                        hash_bytes(&fs::read(path).unwrap()),
                    );
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.dir(), Path::new(""), &mut out);
        out
    }
}

// Independent authored paint stacks. Inherited opacity multiplies each leaf;
// it does not turn an ordinary component into an isolated flattened plate.
fn layered_over_black(layers: &[[u8; 3]], alpha: f64) -> [u8; 3] {
    let alpha = (alpha * 255.).round() / 255.;
    std::array::from_fn(|channel| {
        let mut lit = 0.;
        for rgb in layers {
            let s = f64::from(rgb[channel]) / 255.;
            let linear = if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            };
            lit = linear * alpha + lit * (1. - alpha);
        }
        (255.
            * if lit <= 0.0031308 {
                12.92 * lit
            } else {
                1.055 * lit.powf(1. / 2.4) - 0.055
            })
        .round() as u8
    })
}
fn glyph_count(
    bytes: &[u8],
    left: usize,
    right: usize,
    top: usize,
    bottom: usize,
    threshold: u8,
) -> usize {
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| (y * 1920 + x) * 3))
        .filter(|p| bytes[*p..*p + 3].iter().all(|c| *c > threshold))
        .count()
}
fn label_omission_controls(bytes: &[u8], name: &str) {
    if name == "motif_gallery" {
        return;
    }
    let boxes = if name == "baseline" {
        vec![(1230, 1670, 330, 430)]
    } else if name == "hero" {
        vec![(184, 854, 717, 785), (1064, 1734, 717, 785)]
    } else {
        (0..if name.ends_with('1') { 1 } else { 4 })
            .map(|i| {
                let (x, y) = if name.starts_with("row") {
                    (100 + 450 * i, 390)
                } else {
                    (480 + 560 * (i % 2), 160 + 400 * (i / 2))
                };
                (x + 32, x + 367, y + 248, y + 282)
            })
            .collect::<Vec<_>>()
    };
    for (left, right, top, bottom) in boxes {
        let mut omitted = bytes.to_vec();
        for y in top..bottom {
            for x in left..right {
                omitted[(y * 1920 + x) * 3..(y * 1920 + x) * 3 + 3].copy_from_slice(&[48, 73, 104]);
            }
        }
        assert!(
            std::panic::catch_unwind(|| verify(&omitted, name, false)).is_err(),
            "accepted omitted label for {name}"
        );
    }
}
fn assert_publication_inventory(
    before: &BTreeMap<String, String>,
    mut after: BTreeMap<String, String>,
    returned: &str,
) {
    let returned = returned.replace("\\", "/");
    assert!(
        !before.contains_key(&returned),
        "returned artifact replaced a preexisting file"
    );
    assert!(
        after.remove(&returned).is_some(),
        "returned artifact absent"
    );
    assert_eq!(
        before, &after,
        "render changed unrelated or preexisting files"
    );
}

fn near(bytes: &[u8], x: usize, y: usize, rgb: [u8; 3], tolerance: u8) {
    for c in 0..3 {
        assert!(
            bytes[(y * 1920 + x) * 3 + c].abs_diff(rgb[c]) <= tolerance,
            "independent {x},{y}: {:?} != {rgb:?}",
            &bytes[(y * 1920 + x) * 3..(y * 1920 + x) * 3 + 3]
        );
    }
}
fn verify(bytes: &[u8], name: &str, movie: bool) {
    assert_eq!(bytes.len(), PIXELS * if movie { 3 } else { 1 });
    if name == "baseline" {
        illustrated_scene::strategy_verify_pixels(bytes, if movie { 8 } else { 2 });
        return;
    }
    for (index, frame) in bytes.chunks_exact(PIXELS).enumerate() {
        let t = if movie {
            index as f64 * 1000. / 30.
        } else {
            500.
        };
        let tolerance = if movie { 8 } else { 3 };
        near(frame, 1910, 1070, [0, 0, 0], tolerance);
        if name == "hero" {
            let alpha = (t / 100.).clamp(0., 1.);
            for (x, y, rgb) in [
                (175, 275, [238, 238, 238]),
                (230, 300, [22, 32, 51]),
                (200, 775, [48, 73, 104]),
                (250, 662, [255, 204, 51]),
                (440, 400, [255, 255, 255]),
                (615, 400, [255, 0, 0]),
            ] {
                let bg = [24, 38, 61];
                let device = [238, 238, 238];
                let screen = [22, 32, 51];
                let layers = match x {
                    175 => vec![bg, device],
                    230 => vec![bg, device, screen],
                    200 => vec![bg, rgb],
                    250 => vec![bg, device, rgb],
                    _ => vec![bg, device, screen, rgb],
                };
                near(frame, x, y, layered_over_black(&layers, alpha), tolerance);
            }
            near(frame, 1055, 275, [238, 238, 238], tolerance);
            near(frame, 1495, 400, [255, 0, 0], tolerance);
            if !movie {
                assert!(
                    glyph_count(frame, 184, 854, 717, 785, 240) >= 100,
                    "native hero left glyphs absent"
                );
            }
            assert!(
                glyph_count(frame, 1064, 1734, 717, 785, 240) >= 100,
                "native hero right glyphs absent"
            );
        } else if name == "motif_gallery" {
            for (x, y, rgb, roi) in [
                (100, 100, [0, 156, 188], Some((99, 99, 2, 2))),
                (120, 120, [0, 0, 0], None),
                (450, 130, [248, 249, 250], None),
                (420, 160, [34, 51, 68], None),
                (560, 160, [255, 204, 0], None),
                (920, 80, [255, 204, 0], None),
                (920, 160, [255, 107, 0], None),
                (1180, 190, [255, 107, 0], None),
                (1290, 180, [255, 204, 0], None),
                (1580, 93, [0, 186, 224], Some((1578, 91, 4, 4))),
                (1580, 105, [0, 0, 0], None),
                (100, 420, [188, 188, 188], Some((99, 419, 2, 2))),
                (560, 480, [0, 255, 255], None),
                (450, 450, [255, 255, 255], None),
                (420, 480, [0, 0, 0], None),
                (920, 400, [255, 255, 255], None),
                (920, 480, [0, 0, 0], None),
                (1180, 510, [255, 0, 0], None),
                (1290, 500, [255, 255, 255], None),
                (1580, 413, [224, 224, 224], Some((1578, 411, 4, 4))),
            ] {
                if movie && let Some((left, top, width, height)) = roi {
                    for c in 0..3 {
                        let mean = (top..top + height)
                            .flat_map(|y| {
                                (left..left + width)
                                    .map(move |x| f64::from(frame[(y * 1920 + x) * 3 + c]))
                            })
                            .sum::<f64>()
                            / (width * height) as f64;
                        assert!(
                            (mean - f64::from(rgb[c])).abs() <= 8.,
                            "independent motif ROI {x},{y}"
                        );
                    }
                } else {
                    near(frame, x, y, rgb, tolerance);
                }
            }
        } else {
            let count = if name.ends_with('1') { 1 } else { 4 };
            for i in 0..count {
                let (x, y) = if name.starts_with("row") {
                    (100 + 450 * i, 390)
                } else {
                    (480 + 560 * (i % 2), 160 + 400 * (i / 2))
                };
                let alpha = ((t - 80. * i as f64) / 100.).clamp(0., 1.);
                let bg = [[24, 38, 61], [32, 64, 64], [64, 32, 64], [64, 48, 32]][i];
                for (dx, dy, rgb) in [
                    (10, 10, bg),
                    (if movie { 27 } else { 25 }, 25, [238, 238, 238]),
                    (35, 35, [22, 32, 51]),
                    (25, 250, [48, 73, 104]),
                    (180, 130, [255, 255, 255]),
                    (230, 130, [255, 0, 0]),
                    (320, 60, [255, 107, 0]),
                ] {
                    near(
                        frame,
                        x + dx,
                        y + dy,
                        layered_over_black(
                            &match dx {
                                10 => vec![bg],
                                25 | 27 => vec![bg, rgb],
                                35 => vec![bg, [238, 238, 238], rgb],
                                _ => vec![bg, [238, 238, 238], [22, 32, 51], rgb],
                            },
                            alpha,
                        ),
                        tolerance,
                    );
                }
                if !movie {
                    near(frame, x + 40, y + 220, [255, 204, 51], 3);
                    near(frame, x + 320, y + 25, [255, 204, 0], 3);
                    let white = glyph_count(frame, x + 32, x + 367, y + 248, y + 282, 240);
                    assert!(white > 30, "independent member glyph presence");
                }
            }
        }
    }
}

fn read_output(tools: &NativeTools, path: &Path, movie: bool) -> Vec<u8> {
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
    assert!(probe.status.success());
    let facts: Value = serde_json::from_slice(&probe.stdout).unwrap();
    assert_eq!(facts["streams"][0]["width"], 1920);
    assert_eq!(facts["streams"][0]["height"], 1080);
    assert_eq!(
        facts["streams"][0]["nb_read_frames"],
        if movie { "3" } else { "1" }
    );
    let result = Command::new(&tools.ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"])
        .output()
        .unwrap();
    assert!(result.status.success());
    result.stdout
}
fn counts(renderer: &Renderer) -> [usize; 4] {
    let (ph, pm) = renderer.preview_cache.counts();
    [
        renderer.raster_cache.hits.load(Ordering::Relaxed),
        renderer.raster_cache.misses.load(Ordering::Relaxed),
        ph,
        pm,
    ]
}
fn capture(
    tools: &NativeTools,
    fixture: &Fixture,
    renderer: &Renderer,
    movie: bool,
    reference: &mut Option<Vec<u8>>,
) -> Value {
    let project = fixture.project();
    let before = fixture.authoritative();
    let prior = counts(renderer);
    let session = crate::error::dense_scene_observations::Session::start();
    let sampler = ProcessTreeSampler::start();
    let started = Instant::now();
    let result = if movie {
        renderer.render_preview_range(
            &project,
            &fixture.dir(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 100,
                width: 1920,
                height: 1080,
                fps: 30,
                include_audio: false,
            },
            |_| {},
        )
    } else {
        renderer.render_preview(&project, &fixture.dir(), 500)
    };
    let ms = started.elapsed().as_secs_f64() * 1000.;
    let peak = sampler.finish();
    let owner = session.finish();
    let after = counts(renderer);
    let delta = std::array::from_fn::<_, 4, _>(|i| after[i] - prior[i]);
    let artifact = result.unwrap_or_else(|e| {
        assert_eq!(before,fixture.authoritative(),"rejected request changed state/preexisting outputs");
        eprintln!("dense_scene_rejected_cell_evidence={}",json!({"workload":fixture.name,"intent":if movie {"range"}else{"frame"},"admissionStatus":"typed_rejected","code":e.code,"retryable":e.retryable,"renderBudget":e.render_budget,"facadeMilliseconds":ms,"sampledProcessTreeResidentBytes":peak,"ownerObservations":owner,"authoritativeAndPreexistingOutputsUnchanged":true}));
        panic!("{} {} rejected: {e:?}; incomplete evaluation blocks acceptance",fixture.name,if movie {"range"}else{"frame"});
    });
    let path = fixture.dir().join(&artifact.relative_path);
    if let Some(directory) = env::var_os("OPENCUT_DENSE_ARTIFACT_DIR") {
        let directory =
            Path::new(&directory)
                .join(fixture.name)
                .join(if movie { "range" } else { "frame" });
        fs::create_dir_all(&directory).unwrap();
        fs::copy(
            &path,
            directory.join(format!(
                "{}.{}",
                Uuid::new_v4(),
                if movie { "mp4" } else { "png" }
            )),
        )
        .unwrap();
    }
    let bytes = read_output(tools, &path, movie);
    verify(&bytes, fixture.name, movie);
    if reference.is_none() && !movie {
        label_omission_controls(&bytes, fixture.name);
    }
    if let Some(expected) = reference {
        assert_eq!(&bytes, expected, "decoded fresh/warm parity");
    } else {
        *reference = Some(bytes.clone());
    }
    assert_publication_inventory(&before, fixture.authoritative(), &artifact.relative_path);
    let observation = json!({"facadeMilliseconds":ms,"sampledProcessTreeResidentBytes":peak,"cacheWork":{"rasterHits":delta[0],"rasterMisses":delta[1],"encodedPreviewHits":delta[2],"encodedPreviewMisses":delta[3]},"ownerObservations":owner,"decodedSha256":hash_bytes(&bytes),"artifactSha256":hash_bytes(&fs::read(&path).unwrap()),"frameCount":if movie {3}else{1},"authoritativeUnchanged":true,"preexistingOutputsUnchanged":true,"admissionStatus":"admitted"});
    eprintln!(
        "dense_scene_sample {} {}: {ms:.3}ms",
        fixture.name,
        if movie { "range" } else { "frame" }
    );
    observation
}

fn validate_report(report: &Value) -> Result<(), &'static str> {
    if report["version"] != 1
        || report["canvas"] != json!({"width":1920,"height":1080,"fps":30})
        || report["frameMs"] != 500
        || report["rangeMs"] != json!([0, 100])
        || report["durationMs"] != 1000
        || report["decisionRule"]
            != "maximum of three fresh samples; all fourteen cells; 5000ms frame/8000ms range"
    {
        return Err("matrix settings/rule");
    }
    for key in [
        "gitRevision",
        "trackedDiffSha256",
        "driverSha256",
        "testBinarySha256",
    ] {
        let Some(s) = report["source"][key].as_str() else {
            return Err("source identity");
        };
        if s.len() != if key == "gitRevision" { 40 } else { 64 }
            || !s.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err("source hash");
        }
    }
    for (name, bytes) in [
        ("baseline", BASELINE),
        ("hero", HERO),
        ("motifs", MOTIFS),
        ("team", TEAM),
    ] {
        if report["source"]["fixtureSha256"][name] != hash_bytes(bytes) {
            return Err("immutable fixture identity");
        }
    }
    for key in [
        "os",
        "architecture",
        "osVersion",
        "cpu",
        "ffmpeg",
        "ffprobe",
        "fontSha256",
    ] {
        if report["environment"][key]
            .as_str()
            .is_none_or(str::is_empty)
        {
            return Err("environment identity");
        }
    }
    if report["environment"]["debugAssertions"].as_bool().is_none()
        || report["source"]["runtimeCheckout"]
            .as_str()
            .is_none_or(str::is_empty)
    {
        return Err("runtime/build identity");
    }
    if report["source"]["sourceDirty"].as_bool().is_none()
        || report["source"]["sourceDirty"]
            != (report["source"]["trackedDiffSha256"] != hash_bytes(b""))
    {
        return Err("source dirtiness");
    }
    for key in ["logicalCpus", "totalMemoryBytes"] {
        if report["environment"][key].as_u64().is_none_or(|n| n == 0) {
            return Err("host capacity");
        }
    }
    for key in ["cpuMax", "memoryMax"] {
        if report["environment"].get(key).is_none() {
            return Err("quota metadata absent");
        }
    }
    let cpu = &report["environment"]["cpuMax"];
    let memory = &report["environment"]["memoryMax"];
    if (cpu.is_null() || memory.is_null())
        && report["semantics"]["quotaMetadata"]
            != "null quota means unavailable, not unbounded or reference-equivalent"
    {
        return Err("misleading unavailable quotas");
    }
    if !cpu.is_null() {
        let Some(cpu) = cpu.as_str() else {
            return Err("CPU quota type");
        };
        let fields = cpu.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 2
            || (fields[0] != "max" && fields[0].parse::<u64>().ok().is_none_or(|n| n == 0))
            || fields[1].parse::<u64>().ok().is_none_or(|n| n == 0)
        {
            return Err("CPU quota");
        };
    }
    if !memory.is_null() {
        let Some(memory) = memory.as_str() else {
            return Err("memory quota type");
        };
        if memory != "max" && memory.parse::<u64>().ok().is_none_or(|n| n == 0) {
            return Err("memory quota");
        };
    }

    if report["semantics"]["roadmap"]
        != "unimplemented96-leaf/six-instance/3000ms targets not measured"
        || report["semantics"]["portablePerformance"] != "report-only; fixed reference host only"
        || report["semantics"]["timings"]
            != "direct inclusive non-additive; no sum/subtraction; absent stage unobserved"
    {
        return Err("misleading measurement meaning");
    }
    for key in [
        "fresh",
        "repeated",
        "rasterWarmEncodedCold",
        "timings",
        "memory",
        "charges",
        "portablePerformance",
        "roadmap",
    ] {
        if report["semantics"][key].as_str().is_none_or(str::is_empty) {
            return Err("measurement semantics");
        }
    }
    let cells = report["cells"].as_array().ok_or("cells")?;
    if cells.len() != 14 {
        return Err("cell count");
    }
    for (index, cell) in cells.iter().enumerate() {
        if cell["workload"] != WORKLOADS[index / 2]
            || cell["intent"] != if index % 2 == 0 { "frame" } else { "range" }
        {
            return Err("cell identity/order");
        }
        if cell["expandedVisualLeaves"] != [7, 14, 16, 9, 36, 9, 36][index / 2]
            || cell["componentDefinitions"] != [0, 1, 5, 1, 1, 1, 1][index / 2]
        {
            return Err("bounded fixture inventory");
        }
        for category in ["fresh", "warmup", "repeated", "rasterWarmEncodedCold"] {
            let samples: &[Value] = if category == "warmup" {
                std::slice::from_ref(&cell[category])
            } else {
                cell[category].as_array().ok_or("samples")?
            };
            if samples.len() != if category == "warmup" { 1 } else { 3 } {
                return Err("sample count");
            }
            for sample in samples {
                if sample["facadeMilliseconds"]
                    .as_f64()
                    .is_none_or(|n| !n.is_finite() || n < 0.)
                    || sample["sampledProcessTreeResidentBytes"]
                        .as_u64()
                        .is_none_or(|n| n == 0)
                    || sample["authoritativeUnchanged"] != true
                    || sample["preexistingOutputsUnchanged"] != true
                    || sample["admissionStatus"] != "admitted"
                    || sample["frameCount"] != if index % 2 == 0 { 1 } else { 3 }
                {
                    return Err("measurement/state");
                }
                for key in ["decodedSha256", "artifactSha256"] {
                    if sample[key]
                        .as_str()
                        .is_none_or(|s| s.len() != 64 || !s.bytes().all(|c| c.is_ascii_hexdigit()))
                    {
                        return Err("output hash");
                    }
                }
                let w = &sample["cacheWork"];
                for key in [
                    "rasterHits",
                    "rasterMisses",
                    "encodedPreviewHits",
                    "encodedPreviewMisses",
                ] {
                    if w[key].as_u64().is_none() {
                        return Err("cache count");
                    }
                }
                if (category == "fresh" || category == "warmup")
                    && (w["rasterMisses"].as_u64().unwrap() == 0
                        || w["encodedPreviewHits"] != 0
                        || w["encodedPreviewMisses"] != 1)
                {
                    return Err("false fresh work");
                }
                if category == "repeated"
                    && (w["rasterHits"].as_u64().unwrap() == 0
                        || w["rasterMisses"] != 0
                        || w["encodedPreviewHits"] != 1
                        || w["encodedPreviewMisses"] != 0)
                {
                    return Err("false encoded reuse");
                }
                if category == "rasterWarmEncodedCold"
                    && (w["rasterHits"].as_u64().unwrap() == 0
                        || w["rasterMisses"] != 0
                        || w["encodedPreviewHits"] != 0
                        || w["encodedPreviewMisses"] != 1)
                {
                    return Err("false raster reuse");
                }
                let times = sample["ownerObservations"]["directInclusiveTimings"]
                    .as_array()
                    .ok_or("direct timings")?;
                if times.len() > 3
                    || times
                        .iter()
                        .map(|t| t["name"].as_str().unwrap_or(""))
                        .collect::<BTreeSet<_>>()
                        .len()
                        != times.len()
                {
                    return Err("duplicate/unbounded timing");
                }
                let charges = sample["ownerObservations"]["ownerChargeMaxima"]
                    .as_array()
                    .ok_or("owner charges")?;
                if charges.is_empty()
                    || !charges.iter().any(|c| c["budget"] == "composition_bytes")
                    || charges
                        .iter()
                        .map(|c| c["budget"].as_str().unwrap_or(""))
                        .collect::<BTreeSet<_>>()
                        .len()
                        != charges.len()
                    || charges.len() > 8
                {
                    return Err("unbounded owner charges");
                }
                for charge in charges {
                    if !crate::error::dense_scene_observations::valid_charge(charge) {
                        return Err("owner charge identity/meaning/bounds");
                    }
                }
                for t in times {
                    if t["milliseconds"]
                        .as_f64()
                        .is_none_or(|n| !n.is_finite() || n < 0.)
                        || t["calls"].as_u64().is_none_or(|n| n == 0)
                        || ![
                            "source_rasterization",
                            "native_visual_preparation",
                            "final_backend_execution",
                        ]
                        .contains(&t["name"].as_str().unwrap_or(""))
                    {
                        return Err("direct timing");
                    }
                }
                let has = |name| times.iter().any(|t| t["name"] == name);
                if let Some(raster) = times.iter().find(|t| t["name"] == "source_rasterization")
                    && raster["calls"] != w["rasterMisses"]
                {
                    return Err("raster call/miss mismatch");
                }
                if has("source_rasterization") != (w["rasterMisses"].as_u64().unwrap() > 0)
                    || !has("native_visual_preparation")
                    || (has("final_backend_execution") != (category != "repeated"))
                {
                    return Err("stage/work disagreement");
                }
                if sample["decodedSha256"] != cell["fresh"][0]["decodedSha256"] {
                    return Err("parity hash");
                }
            }
        }
        if cell["warmup"].as_object().is_none() {
            return Err("missing excluded warmup");
        }
    }
    Ok(())
}

fn checkout_fence(runtime: &Path, executing: &Path) {
    assert_eq!(
        runtime.canonicalize().unwrap(),
        executing.canonicalize().unwrap(),
        "compiled runtime came from another checkout"
    );
}
pub(super) fn conformance(tools: &NativeTools) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executing = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();
    assert!(executing.status.success());
    checkout_fence(
        &root,
        Path::new(String::from_utf8(executing.stdout).unwrap().trim()),
    );
    assert_eq!(
        tools.font_sha256,
        "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280"
    );
    let new_renderer = || Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut cells = Vec::new();
    for name in WORKLOADS {
        let fixture = Fixture::new(name);
        let project = fixture.project();
        let evaluated = evaluate_project(&project, 1920, 1080, 30).unwrap();
        let original = fixture.authoritative();
        for movie in [false, true] {
            let mut reference = None;
            let fresh = (0..3)
                .map(|_| capture(tools, &fixture, &new_renderer(), movie, &mut reference))
                .collect::<Vec<_>>();
            let renderer = new_renderer();
            let warmup = capture(tools, &fixture, &renderer, movie, &mut reference);
            let repeated = (0..3)
                .map(|_| capture(tools, &fixture, &renderer, movie, &mut reference))
                .collect::<Vec<_>>();
            let controls = (0..3)
                .map(|_| {
                    let mut control = renderer.clone();
                    control.preview_cache = Arc::default();
                    capture(tools, &fixture, &control, movie, &mut reference)
                })
                .collect::<Vec<_>>();
            cells.push(json!({"workload":name,"intent":if movie {"range"}else{"frame"},"expandedVisualLeaves":evaluated.scene.visual_layers.len(),"componentDefinitions":project.components.len(),"authoritativeFixtureSha256":hash_bytes(&serde_json::to_vec(&original).unwrap()),"fresh":fresh,"warmup":warmup,"repeated":repeated,"rasterWarmEncodedCold":controls}));
        }
        let reopened = EditorCore::new(fixture.core.paths().clone())
            .get_project(&fixture.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap(),
            serde_json::to_value(project).unwrap()
        );
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git = |args: &[&str]| {
        let r = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(r.status.success());
        r.stdout
    };
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    let report = json!({"version":1,"canvas":{"width":1920,"height":1080,"fps":30},"durationMs":1000,"frameMs":500,"rangeMs":[0,100],"decisionRule":"maximum of three fresh samples; all fourteen cells; 5000ms frame/8000ms range",
        "source":{"gitRevision":String::from_utf8(git(&["rev-parse","HEAD"])).unwrap().trim(),"trackedDiffSha256":hash_bytes(&git(&["diff","HEAD","--"])),"sourceDirty":!git(&["diff","HEAD","--"]).is_empty(),"driverSha256":hash_bytes(include_bytes!("dense_scene_strategy.rs")),"testBinarySha256":hash_bytes(&fs::read(env::current_exe().unwrap()).unwrap()),"runtimeCheckout":root.canonicalize().unwrap(),"fixtureSha256":{"baseline":hash_bytes(BASELINE),"hero":hash_bytes(HERO),"motifs":hash_bytes(MOTIFS),"team":hash_bytes(TEAM)}},
        "environment":{"os":env::consts::OS,"architecture":env::consts::ARCH,"osVersion":System::long_os_version(),"cpu":system.cpus().first().map(|c|c.brand()),"logicalCpus":system.cpus().len(),"totalMemoryBytes":system.total_memory(),"cpuMax":fs::read_to_string("/sys/fs/cgroup/cpu.max").ok().map(|s|s.trim().to_owned()),"memoryMax":fs::read_to_string("/sys/fs/cgroup/memory.max").ok().map(|s|s.trim().to_owned()),"debugAssertions":cfg!(debug_assertions),"ffmpeg":tools.ffmpeg_version,"ffprobe":tools.ffprobe_version,"fontSha256":tools.font_sha256},
        "semantics":{"fresh":"new Renderer; OS caches not flushed","repeated":"same Renderer/request; encoded preview cache eligible","rasterWarmEncodedCold":"same immutable project/request and shared raster cache; independently empty encoded cache each control","timings":"direct inclusive non-additive; no sum/subtraction; absent stage unobserved","memory":"maximum sampled resident bytes; parent plus recursive process descendants;5ms; includes allocator/cache retention, excludes independent output decode","charges":"maxima observed at original owner admission calls; not a full scene estimator or RSS; conservative memory certificates separate","portablePerformance":"report-only; fixed reference host only","quotaMetadata":"null quota means unavailable, not unbounded or reference-equivalent","roadmap":"unimplemented96-leaf/six-instance/3000ms targets not measured"},"cells":cells});
    eprintln!("dense_scene_candidate_observation={report}");
    validate_report(&report).unwrap();
    eprintln!("dense_scene_strategy_observation={report}");
    if let Some(path) = env::var_os("OPENCUT_DENSE_REPORT_PATH") {
        fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}

#[test]
fn native_bounded_dense_scene_strategy() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}

#[test]
fn report_rejects_matrix_identity_state_cost_and_false_cache_evidence() {
    let hash = "a".repeat(64);
    let sample = |kind: &str, frame_count: i32| {
        let (rh, rm, eh, em) = match kind {
            "repeated" => (1, 0, 1, 0),
            "rasterWarmEncodedCold" => (1, 0, 0, 1),
            _ => (0, 1, 0, 1),
        };
        let mut timings = vec![];
        if rm > 0 {
            timings.push(json!({"name":"source_rasterization","calls":1,"milliseconds":1.}));
        }
        timings.push(json!({"name":"native_visual_preparation","calls":1,"milliseconds":2.}));
        if em > 0 {
            timings.push(json!({"name":"final_backend_execution","calls":1,"milliseconds":2.}));
        }
        json!({"facadeMilliseconds":10.,"sampledProcessTreeResidentBytes":100000,"cacheWork":{"rasterHits":rh,"rasterMisses":rm,"encodedPreviewHits":eh,"encodedPreviewMisses":em},"ownerObservations":{"ownerChargeMaxima":[{"budget":"composition_bytes","limit":"1073741824","work":"1000","units":"bytes","measurement":"conservative","overflow":false,"contributors":[]}],"directInclusiveTimings":timings},"decodedSha256":hash,"artifactSha256":hash,"frameCount":frame_count,"authoritativeUnchanged":true,"preexistingOutputsUnchanged":true,"admissionStatus":"admitted"})
    };
    let mut cells = vec![];
    for (i, name) in WORKLOADS.into_iter().enumerate() {
        for movie in [false, true] {
            let count = if movie { 3 } else { 1 };
            let fresh = sample("fresh", count);
            cells.push(json!({"workload":name,"intent":if movie {"range"}else{"frame"},"expandedVisualLeaves":([7,14,16,9,36,9,36][i]),"componentDefinitions":([0,1,5,1,1,1,1][i]),"fresh":[fresh.clone(),fresh.clone(),fresh.clone()],"warmup":fresh,"repeated":[sample("repeated",count),sample("repeated",count),sample("repeated",count)],"rasterWarmEncodedCold":[sample("rasterWarmEncodedCold",count),sample("rasterWarmEncodedCold",count),sample("rasterWarmEncodedCold",count)]}));
        }
    }
    let report = json!({"version":1,"canvas":{"width":1920,"height":1080,"fps":30},"durationMs":1000,"frameMs":500,"rangeMs":[0,100],"decisionRule":"maximum of three fresh samples; all fourteen cells; 5000ms frame/8000ms range","source":{"gitRevision":"a".repeat(40),"trackedDiffSha256":hash,"sourceDirty":true,"driverSha256":hash,"testBinarySha256":hash,"runtimeCheckout":"synthetic validation control only","fixtureSha256":{"baseline":hash_bytes(BASELINE),"hero":hash_bytes(HERO),"motifs":hash_bytes(MOTIFS),"team":hash_bytes(TEAM)}},"environment":{"os":"synthetic","architecture":"synthetic","osVersion":"synthetic","cpu":"synthetic","ffmpeg":"synthetic","ffprobe":"synthetic","fontSha256":hash,"debugAssertions":false,"logicalCpus":4,"totalMemoryBytes":1000000,"cpuMax":null,"memoryMax":null},"semantics":{"fresh":"declared","repeated":"declared","rasterWarmEncodedCold":"declared","timings":"direct inclusive non-additive; no sum/subtraction; absent stage unobserved","memory":"RSS","charges":"owner maxima","portablePerformance":"report-only; fixed reference host only","quotaMetadata":"null quota means unavailable, not unbounded or reference-equivalent","roadmap":"unimplemented96-leaf/six-instance/3000ms targets not measured"},"cells":cells});
    validate_report(&report).unwrap();
    for (pointer, value) in [
        ("/canvas/width", json!(1919)),
        ("/rangeMs/1", json!(400)),
        ("/source/gitRevision", json!("unknown")),
        ("/source/fixtureSha256/team", json!(hash)),
        ("/environment/cpu", Value::Null),
        ("/semantics/roadmap", Value::Null),
        ("/cells/0/expandedVisualLeaves", json!(96)),
        ("/cells/0/fresh/0/facadeMilliseconds", json!(-1)),
        ("/cells/0/fresh/0/sampledProcessTreeResidentBytes", json!(0)),
        ("/cells/0/fresh/0/authoritativeUnchanged", json!(false)),
        ("/cells/1/fresh/0/frameCount", json!(12)),
        ("/cells/0/repeated/0/cacheWork/rasterMisses", json!(1)),
        (
            "/cells/0/rasterWarmEncodedCold/0/cacheWork/rasterHits",
            json!(0),
        ),
        (
            "/cells/0/rasterWarmEncodedCold/0/cacheWork/encodedPreviewHits",
            json!(1),
        ),
        (
            "/cells/0/fresh/0/ownerObservations/directInclusiveTimings/0/milliseconds",
            json!(-1),
        ),
        (
            "/cells/0/fresh/0/ownerObservations/ownerChargeMaxima/0/work",
            json!("1073741825"),
        ),
        (
            "/cells/0/fresh/0/ownerObservations/ownerChargeMaxima/0/measurement",
            json!("measured"),
        ),
        (
            "/cells/0/fresh/0/ownerObservations/ownerChargeMaxima/0/limit",
            json!("2147483648"),
        ),
        ("/cells/0/repeated/0/decodedSha256", json!("b".repeat(64))),
        ("/source/sourceDirty", Value::Null),
        ("/environment/logicalCpus", json!(0)),
        ("/environment/totalMemoryBytes", Value::Null),
        ("/semantics/roadmap", json!("96 leaves ready")),
        (
            "/cells/0/fresh/0/ownerObservations/ownerChargeMaxima",
            json!([]),
        ),
        ("/cells/0/fresh/0/cacheWork/rasterMisses", json!(42)),
        ("/cells/0/fresh/0/preexistingOutputsUnchanged", json!(false)),
        ("/cells/0/fresh/0/admissionStatus", json!("ready")),
        ("/environment/cpuMax", json!(false)),
        ("/environment/cpuMax", json!("invalid")),
        ("/environment/memoryMax", json!([])),
        ("/environment/memoryMax", json!("invalid")),
        ("/semantics/quotaMetadata", json!("null means unlimited")),
        ("/cells/0/repeated/0/cacheWork/rasterHits", json!(0)),
        ("/cells/0/warmup", Value::Null),
        ("/cells/0/fresh", json!([])),
    ] {
        let mut bad = report.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            validate_report(&bad).is_err(),
            "accepted malformed/false evidence at {pointer}"
        );
    }
    for key in ["cpuMax", "memoryMax"] {
        let mut missing = report.clone();
        missing["environment"].as_object_mut().unwrap().remove(key);
        assert!(
            validate_report(&missing).is_err(),
            "accepted absent quota field"
        );
    }
    if let Some(path) = env::var_os("OPENCUT_DENSE_VALIDATE_REPORT") {
        let bytes = fs::read(path).unwrap();
        let retained: Value = serde_json::from_slice(&bytes).unwrap();
        validate_report(&retained).unwrap();
        eprintln!(
            "dense_scene_retained_validation={}",
            json!({"kind":"validator replay only; no new native renders","reportSha256":hash_bytes(&bytes),"capturedDriverSha256":retained["source"]["driverSha256"],"validatorDriverSha256":hash_bytes(include_bytes!("dense_scene_strategy.rs")),"cells":retained["cells"].as_array().unwrap().len()})
        );
    }
}

#[test]
fn independent_output_oracles_reject_blank_truncated_and_wrong_paint() {
    for name in WORKLOADS {
        for movie in [false, true] {
            let bytes = vec![0; PIXELS * if movie { 3 } else { 1 }];
            assert!(
                std::panic::catch_unwind(|| verify(&bytes, name, movie)).is_err(),
                "accepted blank {name}"
            );
            assert!(
                std::panic::catch_unwind(|| verify(&bytes[..PIXELS - 1], name, movie)).is_err()
            );
        }
    }
    let mut bytes = vec![0; PIXELS];
    for rgb in bytes.chunks_exact_mut(3) {
        rgb.copy_from_slice(&[255, 0, 0]);
    }
    assert!(std::panic::catch_unwind(|| verify(&bytes, "grid4", false)).is_err());
}

#[test]
fn checkout_and_preexisting_output_controls_fail_closed() {
    let one = tempdir().unwrap();
    let other = tempdir().unwrap();
    checkout_fence(one.path(), one.path());
    assert!(std::panic::catch_unwind(|| checkout_fence(one.path(), other.path())).is_err());
    let before = BTreeMap::from([
        ("project.json".into(), "source".into()),
        ("previews/old.png".into(), "old".into()),
    ]);
    let mut after = before.clone();
    after.insert("previews/new.png".into(), "new".into());
    assert_publication_inventory(&before, after.clone(), "previews/new.png");
    for changed in [None, Some("corrupted")] {
        let mut wrong = after.clone();
        if let Some(value) = changed {
            wrong.insert("previews/old.png".into(), value.into());
        } else {
            wrong.remove("previews/old.png");
        }
        assert!(
            std::panic::catch_unwind(|| assert_publication_inventory(
                &before,
                wrong,
                "previews/new.png"
            ))
            .is_err()
        );
    }
    assert!(
        std::panic::catch_unwind(|| assert_publication_inventory(
            &before,
            after,
            "previews/old.png"
        ))
        .is_err()
    );
}
