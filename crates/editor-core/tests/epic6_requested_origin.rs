//! Independent controls use static authored sources and the unchanged aligned renderer.
use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, ExportOptions, MediaProbeFacts, MediaType, PathPolicy,
    PreviewRangeOptions, Project, ProjectSettings, Renderer,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn op(v: Value) -> EditOperation {
    serde_json::from_value(v).unwrap()
}
fn check_tools(ffmpeg: &Path, ffprobe: &Path, font: &Path) -> Result<(), String> {
    if !font.is_file() {
        return Err("cannot read native audit font: configured font must be a file".into());
    }
    std::fs::read(font).map_err(|error| format!("cannot read native audit font: {error}"))?;
    Renderer::new(ffmpeg, ffprobe, Some(font.to_path_buf()))
        .readiness()
        .map_err(|error| error.to_string())
}
fn tools() -> Option<(PathBuf, PathBuf, PathBuf)> {
    let paths = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
        std::env::var_os("OPENCUT_TEST_FONT_PATH"),
    );
    let (Some(f), Some(p), Some(t)) = paths else {
        for flag in [
            "OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED",
            "OPENCUT_GOLDEN_REQUIRED",
        ] {
            assert_ne!(
                std::env::var(flag).as_deref(),
                Ok("1"),
                "requested-origin native proof requires tools/font"
            );
        }
        return None;
    };
    let result = (PathBuf::from(f), PathBuf::from(p), PathBuf::from(t));
    check_tools(&result.0, &result.1, &result.2)
        .expect("configured requested-origin native tools and font must be usable");
    Some(result)
}
#[test]
fn native_requested_origin_dependency_gate_rejects_unusable_tools_and_font() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing-native-dependency");
    assert!(
        check_tools(&missing, &ffprobe, &font)
            .unwrap_err()
            .contains("cannot start FFmpeg")
    );
    assert!(
        check_tools(&ffmpeg, &missing, &font)
            .unwrap_err()
            .contains("cannot start FFprobe")
    );
    assert!(
        check_tools(&ffmpeg, &ffprobe, &missing)
            .unwrap_err()
            .contains("cannot read native audit font")
    );
    assert!(
        check_tools(&ffmpeg, &ffprobe, root.path())
            .unwrap_err()
            .contains("configured font must be a file")
    );
}
fn rgb(ffmpeg: &Path, path: &Path, index: u64) -> Vec<u8> {
    let out = std::process::Command::new(ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-vf",
            &format!("select=eq(n\\,{index})"),
            "-frames:v",
            "1",
            "-threads",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout.len(),
        96 * 96 * 3,
        "{} frame{index}",
        path.display()
    );
    out.stdout
}
fn mass(p: &[u8]) -> (f64, f64, f64) {
    let (mut m, mut x, mut y) = (0., 0., 0.);
    for (i, p) in p.as_chunks::<3>().0.iter().enumerate() {
        let w = f64::from(p[0]);
        m += w;
        x += w * (i % 96) as f64;
        y += w * (i / 96) as f64;
    }
    (m, x / m, y / m)
}
fn compare(actual: &[u8], reference: &[u8], active: bool, label: &str) {
    assert_eq!(actual.len(), reference.len());
    let mse = actual
        .iter()
        .zip(reference)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / actual.len() as f64;
    assert!(mse < 20., "{label}: MSE{mse}");
    let (a, x, y) = mass(actual);
    let (r, rx, ry) = mass(reference);
    if active {
        assert!(
            a > 1000. && r > 1000.,
            "{label}: missing actual/reference paint {a}/{r}"
        );
        assert!(
            (x - rx).hypot(y - ry) < 0.4,
            "{label}: centroid {x},{y} vs{rx},{ry}"
        );
    } else {
        assert!(
            actual.iter().map(|b| f64::from(*b).powi(2)).sum::<f64>() / (actual.len() as f64) < 20.,
            "{label}: inactive paint"
        );
    }
}
struct Fixture {
    _root: tempfile::TempDir,
    core: EditorCore,
    id: String,
    item: String,
    dir: PathBuf,
}
impl Fixture {
    fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    fn edit(&self, v: Value) -> Vec<String> {
        self.core
            .edit(&self.id, self.project().revision, op(v))
            .unwrap()
            .changed_ids
    }
    fn bytes(&self) -> (Vec<u8>, Vec<u8>) {
        (
            std::fs::read(self.dir.join("project.json")).unwrap(),
            std::fs::read(self.dir.join("history.json")).unwrap(),
        )
    }
}
fn seed(kind: &str, ffmpeg: &Path, text: &str) -> Fixture {
    seed_with_caption_style(kind, ffmpeg, text, "#000000")
}
fn seed_with_caption_style(kind: &str, ffmpeg: &Path, text: &str, background: &str) -> Fixture {
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
            "Epic6 exact requested source",
            ProjectSettings {
                width: 96,
                height: 96,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let dir = core.paths().project_dir(&id).unwrap();
    let mut f = Fixture {
        _root: root,
        core,
        id,
        item: String::new(),
        dir,
    };
    let p = f.project();
    let overlay = p.tracks[1].id.clone();
    let backing=f.edit(json!({"operation":"add_rectangle","trackId":overlay,"color":"#000000","width":96,"height":96,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))[0].clone();
    f.edit(json!({"operation":"set_item_visibility","itemId":backing,"hidden":true}));
    if kind == "caption" {
        let file = media.join("silence.wav");
        let out = std::process::Command::new(ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=mono",
                "-t",
                "1",
                "-c:a",
                "pcm_s16le",
            ])
            .arg(&file)
            .output()
            .unwrap();
        assert!(out.status.success());
        let asset = f
            .core
            .import_asset(
                &f.id,
                f.project().revision,
                &file,
                MediaType::Audio,
                MediaProbeFacts {
                    duration_ms: Some(1000),
                    has_audio: true,
                    audio_sample_rate_hz: Some(48000),
                    audio_channels: Some(1),
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let req=serde_json::from_value(json!({"projectId":f.id,"expectedRevision":f.project().revision,"assetId":asset,"captionTrackId":p.tracks[3].id,"providerId":"native-independent","modelId":"literal","modelVersion":null,"language":"en","generatedAtMs":1,"segments":[{"text":text,"startMs":713,"endMs":799,"confidence":null,"words":[]}],"style":{"fontSize":16,"color":"#ff0000","backgroundColor":background,"bottomMarginPx":12}})).unwrap();
        f.core.commit_transcription(req).unwrap();
        f.item = f.project().tracks[3].items[0].id().into();
        return f;
    }
    let mut edit =
        json!({"operation":format!("add_{kind}"),"trackId":overlay,"startMs":713,"durationMs":86});
    if matches!(kind, "rectangle" | "solid_color") {
        edit["transform"] = json!({"positionX":0,"positionY":0,"scale":1,"opacity":1});
    }
    match kind {
        "rectangle" => {
            edit["width"] = json!(20);
            edit["height"] = json!(20);
            edit["color"] = json!("#ff0000");
        }
        "solid_color" => edit["color"] = json!("#ff0000"),
        "shape" => {
            edit["geometry"] = json!({"type":"rectangle","width":20,"height":20});
            edit["fill"] = json!({"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}});
            edit["stroke"] = Value::Null;
        }
        "svg" => {
            edit["svg"] = json!(
                "<svg width=\"20\" height=\"20\"><rect width=\"20\" height=\"20\" fill=\"#f00\"/></svg>"
            )
        }
        "grid" => {
            let contracts: Value =
                serde_json::from_str(include_str!("../../../contracts/procedural-grids-v1.json"))
                    .unwrap();
            edit["grid"] = contracts["valid"][0]["grid"].clone();
            edit["grid"]["pattern"]["stroke"]["paint"]["color"]["a"] = json!(1);
        }
        "text" => {
            edit["text"] = json!("HH");
            edit["fontSize"] = json!(24);
            edit["color"] = json!("#ff0000");
            edit["transform"] = json!({"positionX":12,"positionY":12,"scale":1,"opacity":1});
        }
        "media" => {
            let file = media.join("red.png");
            let out = std::process::Command::new(ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=red:s=20x20",
                    "-frames:v",
                    "1",
                    "-threads",
                    "1",
                ])
                .arg(&file)
                .output()
                .unwrap();
            assert!(out.status.success());
            let asset = f
                .core
                .import_asset(
                    &f.id,
                    f.project().revision,
                    &file,
                    MediaType::Image,
                    MediaProbeFacts {
                        has_video: true,
                        video_width: Some(20),
                        video_height: Some(20),
                        ..Default::default()
                    },
                )
                .unwrap()
                .changed_ids[0]
                .clone();
            edit["trackId"] = json!(p.tracks[0].id);
            edit["assetId"] = json!(asset);
            edit["sourceInMs"] = json!(0);
        }
        _ => panic!("unknown source"),
    }
    f.item = f.edit(edit)[0].clone();
    f
}
fn frame(r: &Renderer, f: &Fixture, p: &Project, at: u64) -> PathBuf {
    f.dir
        .join(r.render_preview(p, &f.dir, at).unwrap().relative_path)
}
fn range(r: &Renderer, f: &Fixture, p: &Project, start: u64, end: u64) -> PathBuf {
    f.dir.join(
        r.render_preview_range(
            p,
            &f.dir,
            PreviewRangeOptions {
                start_ms: start,
                end_ms: end,
                width: 96,
                height: 96,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap()
        .relative_path,
    )
}
#[test]
fn native_all_eight_static_sources_preserve_exact_half_open_requested_activity() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    for kind in [
        "rectangle",
        "solid_color",
        "shape",
        "svg",
        "grid",
        "text",
        "media",
        "caption",
    ] {
        let f = seed(kind, &ffmpeg, "HH");
        let original = f.project();
        let before = f.bytes();
        let frames: [Vec<u8>; 3] =
            [713, 726, 798].map(|at| rgb(&ffmpeg, &frame(&renderer, &f, &original, at), 0));
        let active_range = range(&renderer, &f, &original, 713, 913);
        let first = rgb(&ffmpeg, &active_range, 0);
        let second = rgb(&ffmpeg, &active_range, 1);
        assert_eq!(f.bytes(), before, "{kind} rendering mutated durable state");
        f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
        let control = f.project();
        let control_before = f.bytes();
        let aligned = rgb(&ffmpeg, &frame(&renderer, &f, &control, 700), 0);
        for (at, actual) in [713, 726, 798].into_iter().zip(frames) {
            compare(&actual, &aligned, true, &format!("{kind} frame{at}"));
        }
        // Same range codec/context, independently authored activity for its two samples.
        f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":800}));
        let control = f.project();
        let matched = range(&renderer, &f, &control, 713, 913);
        compare(
            &first,
            &rgb(&ffmpeg, &matched, 0),
            true,
            &format!("{kind} range713"),
        );
        compare(
            &second,
            &rgb(&ffmpeg, &matched, 1),
            false,
            &format!("{kind} range813"),
        );
        let black = vec![0; 96 * 96 * 3];
        for at in [700, 799, 800] {
            compare(
                &rgb(&ffmpeg, &frame(&renderer, &f, &original, at), 0),
                &black,
                false,
                &format!("{kind} inactive{at}"),
            );
        }
        // Actual aligned legacy source and render reads leave current persistent records alone.
        assert_ne!(f.bytes(), control_before);
        let current = f.bytes();
        let _ = frame(&renderer, &f, &control, 700);
        assert_eq!(f.bytes(), current);
        let current_tracks = serde_json::to_value(f.project()).unwrap()["tracks"].clone();
        f.core.undo(&f.id, f.project().revision).unwrap();
        f.core.undo(&f.id, f.project().revision).unwrap();
        let restored = EditorCore::new(f.core.paths().clone())
            .get_project(&f.id)
            .unwrap();
        assert_eq!(
            (
                restored.find_item(&f.item).unwrap().start_ms(),
                restored.find_item(&f.item).unwrap().duration_ms()
            ),
            (713, 86)
        );
        assert_eq!(
            serde_json::to_value(restored).unwrap()["tracks"],
            serde_json::to_value(&original).unwrap()["tracks"],
            "{kind} Undo/reopen changed original interval or source"
        );
        f.core.redo(&f.id, f.project().revision).unwrap();
        f.core.redo(&f.id, f.project().revision).unwrap();
        let reopened = EditorCore::new(f.core.paths().clone())
            .get_project(&f.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(reopened).unwrap()["tracks"],
            current_tracks,
            "{kind} Redo/reopen changed control state"
        );
    }
}
fn impulse() -> Value {
    json!({"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0},"curve":"linear"},{"timeMs":700,"value":{"type":"scalar","value":0},"curve":"linear"},{"timeMs":726,"value":{"type":"scalar","value":1},"curve":"linear"},{"timeMs":800,"value":{"type":"scalar","value":0},"curve":"linear"},{"timeMs":826,"value":{"type":"scalar","value":1},"curve":"hold"}]})
}
#[test]
fn native_solid_and_inherited_caption_match_independent_half_opacity_controls() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    for kind in ["solid_color", "caption"] {
        let f = seed(kind, &ffmpeg, "HH : % \\ sample");
        f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
        let target = if kind == "caption" {
            let before = f.bytes();
            let err=f.core.edit(&f.id,f.project().revision,op(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[impulse()]}))).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgument);
            assert_eq!(f.bytes(), before);
            let group=f.edit(json!({"operation":"add_group","trackId":f.project().tracks[1].id,"startMs":0,"durationMs":1000}))[0].clone();
            f.edit(json!({"operation":"update_item","itemId":group,"transform2d":null}));
            f.edit(json!({"operation":"item_set_parent","itemId":f.item,"parent":{"scope":"root","id":group}}));
            group
        } else {
            f.item.clone()
        };
        f.edit(json!({"operation":"set_animation_channels","itemId":target,"animationChannels":[impulse()]}));
        let source = f.project();
        let before = f.bytes();
        let actual = [713, 813].map(|at| rgb(&ffmpeg, &frame(&renderer, &f, &source, at), 0));
        let actual_range = range(&renderer, &f, &source, 713, 913);
        assert_eq!(f.bytes(), before);
        f.edit(json!({"operation":"set_animation_channels","itemId":target,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.5},"curve":"hold"}]}]}));
        let expected = f.project();
        let reference = rgb(&ffmpeg, &frame(&renderer, &f, &expected, 700), 0);
        for (at, a) in [713, 813].into_iter().zip(actual) {
            compare(
                &a,
                &reference,
                true,
                &format!("{kind} inherited/own at{at}"),
            );
        }
        let reference_range = range(&renderer, &f, &expected, 713, 913);
        for n in 0..2 {
            compare(
                &rgb(&ffmpeg, &actual_range, n),
                &rgb(&ffmpeg, &reference_range, n),
                true,
                &format!("{kind} range sample{n}"),
            );
        }
        let export = f._root.path().join("exports/control.mp4");
        std::fs::create_dir_all(export.parent().unwrap()).unwrap();
        renderer
            .export_video(
                &expected,
                &f.dir,
                ExportOptions {
                    output: &export,
                    width: 96,
                    height: 96,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        assert!(mass(&rgb(&ffmpeg, &export, 7)).0 > 1000.);
    }
}

#[test]
fn native_retained_solid_finite_phase_and_draft_candidates_have_independent_controls() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    for mode in ["repeat", "ping_pong"] {
        let f = seed("solid_color", &ffmpeg, "");
        f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
        let channel = if mode == "repeat" {
            json!({"property":"transform.opacity","loop":{"mode":mode,"iterations":4},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"linear"},{"timeMs":100,"value":{"type":"scalar","value":0.8},"curve":"linear"},{"timeMs":200,"value":{"type":"scalar","value":0.2},"curve":"hold"}]})
        } else {
            json!({"property":"transform.opacity","loop":{"mode":mode,"iterations":2},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"linear"},{"timeMs":200,"value":{"type":"scalar","value":0.8},"curve":"hold"}]})
        };
        f.edit(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[channel]}));
        let split = f.edit(json!({"operation":"split_item","itemId":f.item,"splitMs":700}));
        let right = split.iter().find(|id| **id != f.item).unwrap().clone();
        let p = f.project();
        let retained = &p
            .find_item(&right)
            .unwrap()
            .visual_properties()
            .animation_channels[0];
        assert_eq!(retained.clock.unwrap().offset_ms, 700);
        assert_eq!(retained.clock.unwrap().source_duration_ms, 1000);
        let mut candidate = serde_json::to_value(retained).unwrap();
        for key in candidate["keyframes"].as_array_mut().unwrap() {
            key["value"]["value"] = json!(key["value"]["value"].as_f64().unwrap() + 0.1);
        }
        let draft=f.core.create_draft(&f.id,p.revision,vec![op(json!({"operation":"set_animation_channels","itemId":right,"animationChannels":[candidate]}))],None).unwrap();
        let draft_project = f.core.get_draft_state(&f.id, &draft.id).unwrap().project;
        let draft_path = f.dir.join("drafts").join(format!("{}.json", draft.id));
        let draft_bytes = std::fs::read(&draft_path).unwrap();
        assert!(!draft_bytes.is_empty());
        let before = f.bytes();
        let control = seed("solid_color", &ffmpeg, "");
        control.edit(
            json!({"operation":"trim_item","itemId":control.item,"startMs":0,"durationMs":1000}),
        );
        // Exact integer phase: repeat713→113/200 descending; pingpong713→87/200.
        // Both budgets exhaust at800 and hold the first endpoint thereafter.
        let first = if mode == "repeat" {
            0.2 + 0.6 * 87.0 / 100.0
        } else {
            0.2 + 0.6 * 87.0 / 200.0
        };
        for (is_draft, project) in [(false, &p), (true, &draft_project)] {
            let offset = if is_draft { 0.1 } else { 0.0 };
            let actual = range(&renderer, &f, project, 713, 913);
            control.edit(json!({"operation":"set_animation_channels","itemId":control.item,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first+offset},"curve":"hold"},{"timeMs":813,"value":{"type":"scalar","value":0.2+offset},"curve":"hold"}]}]}));
            let matched = range(&renderer, &control, &control.project(), 713, 913);
            for (n, at, value) in [(0, 713, first + offset), (1, 813, 0.2 + offset)] {
                control.edit(json!({"operation":"set_animation_channels","itemId":control.item,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":value},"curve":"hold"}]}]}));
                let expected_project = control.project();
                let reference = rgb(
                    &ffmpeg,
                    &frame(&renderer, &control, &expected_project, 700),
                    0,
                );
                compare(
                    &rgb(&ffmpeg, &frame(&renderer, &f, project, at), 0),
                    &reference,
                    true,
                    &format!("{mode} draft{is_draft} frame{at}"),
                );
                compare(
                    &rgb(&ffmpeg, &actual, n),
                    &rgb(&ffmpeg, &matched, n),
                    true,
                    &format!("{mode} draft{is_draft} range{at}"),
                );
            }
            for (at, value) in [
                (
                    799,
                    if mode == "repeat" {
                        0.2 + 0.6 / 100.0
                    } else {
                        0.2 + 0.6 / 200.0
                    },
                ),
                (800, 0.2),
                (801, 0.2),
            ] {
                constant_opacity(&control, value + offset);
                compare(
                    &rgb(&ffmpeg, &frame(&renderer, &f, project, at), 0),
                    &rgb(
                        &ffmpeg,
                        &frame(&renderer, &control, &control.project(), 700),
                        0,
                    ),
                    true,
                    &format!("{mode} draft{is_draft} finite boundary{at}"),
                );
            }
            let mut grid = independent_grid_channel(mode, 0, 1000);
            for key in grid["keyframes"].as_array_mut().unwrap() {
                if key["timeMs"].as_u64().unwrap() >= 700 {
                    key["value"]["value"] = json!(key["value"]["value"].as_f64().unwrap() + offset);
                }
            }
            control.edit(json!({"operation":"set_animation_channels","itemId":control.item,"animationChannels":[grid]}));
            let expected_project = control.project();
            let export = f
                ._root
                .path()
                .join(format!("exports/{mode}-{is_draft}.mp4"));
            std::fs::create_dir_all(export.parent().unwrap()).unwrap();
            renderer
                .export_video(
                    project,
                    &f.dir,
                    ExportOptions {
                        output: &export,
                        width: 96,
                        height: 96,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            let expected_export = control
                ._root
                .path()
                .join(format!("exports/{mode}-{is_draft}.mp4"));
            std::fs::create_dir_all(expected_export.parent().unwrap()).unwrap();
            renderer
                .export_video(
                    &expected_project,
                    &control.dir,
                    ExportOptions {
                        output: &expected_export,
                        width: 96,
                        height: 96,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            if let Some(destination) = std::env::var_os("OPENCUT_EPIC6_RENDER_EVIDENCE_DIR") {
                let destination =
                    PathBuf::from(destination).join(format!("finite-{mode}-draft-{is_draft}"));
                std::fs::create_dir_all(&destination).unwrap();
                for (name, bytes) in [
                    (
                        "actual-project.json",
                        serde_json::to_vec_pretty(project).unwrap(),
                    ),
                    (
                        "reference-project.json",
                        serde_json::to_vec_pretty(&expected_project).unwrap(),
                    ),
                    ("draft.json", draft_bytes.clone()),
                    ("actual-export.mp4", std::fs::read(&export).unwrap()),
                    (
                        "reference-export.mp4",
                        std::fs::read(&expected_export).unwrap(),
                    ),
                ] {
                    use std::io::Write;
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(destination.join(name))
                        .unwrap();
                    file.write_all(&bytes).unwrap();
                }
            }
            compare(
                &rgb(&ffmpeg, &export, 8),
                &rgb(&ffmpeg, &expected_export, 8),
                true,
                &format!("{mode} draft{is_draft} global800"),
            );
        }
        assert_eq!(f.bytes(), before);
        assert_eq!(std::fs::read(draft_path).unwrap(), draft_bytes);
        f.core.undo(&f.id, f.project().revision).unwrap();
        f.core.redo(&f.id, f.project().revision).unwrap();
        let reopened = EditorCore::new(f.core.paths().clone())
            .get_project(&f.id)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&reopened).unwrap()["tracks"],
            serde_json::to_value(&p).unwrap()["tracks"]
        );
    }
}

#[test]
fn native_caption_fractional_inherited_transform_and_colored_literal_source_preserve_edges() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    let f = seed_with_caption_style("caption", &ffmpeg, r"HH % : \ sample", "#203060");
    f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
    assert_eq!(
        serde_json::to_value(f.project().find_item(&f.item).unwrap()).unwrap()["style"]["backgroundColor"],
        "#203060"
    );
    let group=f.edit(json!({"operation":"add_group","trackId":f.project().tracks[1].id,"startMs":0,"durationMs":1000}))[0].clone();
    f.edit(json!({"operation":"update_item","itemId":group,"transform2d":null}));
    f.edit(
        json!({"operation":"item_set_parent","itemId":f.item,"parent":{"scope":"root","id":group}}),
    );
    let curve = |property: &str, lo: f64, hi: f64| {
        let mut c = impulse();
        c["property"] = json!(property);
        for key in c["keyframes"].as_array_mut().unwrap() {
            key["value"]["value"] = json!(lo + (hi - lo) * key["value"]["value"].as_f64().unwrap());
        }
        c
    };
    f.edit(json!({"operation":"set_animation_channels","itemId":group,"animationChannels":[curve("transform.position_x",0.,13.),curve("transform.scale_x",1.,1.5)]}));
    let source = f.project();
    let before = f.bytes();
    let actual = [713, 813].map(|at| rgb(&ffmpeg, &frame(&renderer, &f, &source, at), 0));
    let range_actual = range(&renderer, &f, &source, 713, 913);
    let constant = |property: &str, value: f64| json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":value},"curve":"hold"}]});
    f.edit(json!({"operation":"set_animation_channels","itemId":group,"animationChannels":[constant("transform.position_x",6.5),constant("transform.scale_x",1.25)]}));
    let expected = f.project();
    let reference = rgb(&ffmpeg, &frame(&renderer, &f, &expected, 700), 0);
    for (at, image) in [713, 813].into_iter().zip(actual) {
        compare(
            &image,
            &reference,
            true,
            &format!("caption fractional geometry{at}"),
        );
    }
    let range_reference = range(&renderer, &f, &expected, 713, 913);
    for n in 0..2 {
        compare(
            &rgb(&ffmpeg, &range_actual, n),
            &rgb(&ffmpeg, &range_reference, n),
            true,
            &format!("caption fractional encoded{n}"),
        );
    }
    assert_ne!(f.bytes(), before);
    let current = f.bytes();
    let _ = frame(&renderer, &f, &expected, 713);
    assert_eq!(f.bytes(), current);
}

fn finite_opacity_channel(mode: &str) -> Value {
    let keys = if mode == "repeat" {
        json!([{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"linear"},{"timeMs":100,"value":{"type":"scalar","value":0.8},"curve":"linear"},{"timeMs":200,"value":{"type":"scalar","value":0.2},"curve":"hold"}])
    } else {
        json!([{"timeMs":0,"value":{"type":"scalar","value":0.2},"curve":"linear"},{"timeMs":200,"value":{"type":"scalar","value":0.8},"curve":"hold"}])
    };
    json!({"property":"transform.opacity","loop":{"mode":mode,"iterations":if mode=="repeat" {4}else{2}},"keyframes":keys})
}
fn constant_opacity(f: &Fixture, value: f64) {
    f.edit(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":value},"curve":"hold"}]}]}));
}
fn exported(renderer: &Renderer, f: &Fixture, p: &Project, name: &str) -> PathBuf {
    let output = f._root.path().join("exports").join(format!("{name}.mp4"));
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    renderer
        .export_video(
            p,
            &f.dir,
            ExportOptions {
                output: &output,
                width: 96,
                height: 96,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    output
}

// Independent complete10fps sequences contain only authored hold constants;
// they have no loops, retained clocks or production sampler-derived values.
fn independent_grid_channel(mode: &str, start: u64, end: u64) -> Value {
    let values = if mode == "repeat" {
        [0.2, 0.8, 0.2, 0.8, 0.2, 0.8, 0.2, 0.8, 0.2, 0.2]
    } else {
        [0.2, 0.5, 0.8, 0.5, 0.2, 0.5, 0.8, 0.5, 0.2, 0.2]
    };
    let key =
        |time, value| json!({"timeMs":time,"value":{"type":"scalar","value":value},"curve":"hold"});
    // A713ms left trim has no visible tick until800, already exhausted.
    let mut keys = vec![key(0, 0.2)];
    for (index, value) in values.into_iter().enumerate() {
        let global = index as u64 * 100;
        if global >= start && global < end {
            if global == start {
                keys.clear();
            }
            keys.push(key(global - start, value));
        }
    }
    json!({"property":"transform.opacity","keyframes":keys})
}

#[test]
fn native_synthetic_global_grid_split_trim_and_fractional_phase_have_independent_controls() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    for kind in ["solid_color", "rectangle"] {
        let control = seed(kind, &ffmpeg, "");
        control.edit(
            json!({"operation":"trim_item","itemId":control.item,"startMs":0,"durationMs":1000}),
        );
        for mode in ["repeat", "ping_pong"] {
            for (split, trim, end) in [
                (700, false, 1000),
                (713, false, 1000),
                (800, false, 1000),
                (700, true, 1000),
                (713, true, 1000),
                (713, true, 813),
                (713, true, 799),
            ] {
                let f = seed(kind, &ffmpeg, "");
                f.edit(
                    json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}),
                );
                f.edit(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[finite_opacity_channel(mode)]}));
                if trim {
                    f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":split,"durationMs":end-split}));
                } else {
                    f.edit(json!({"operation":"split_item","itemId":f.item,"splitMs":split}));
                }
                let p = f.project();
                let right = p
                    .tracks
                    .iter()
                    .flat_map(|t| &t.items)
                    .find(|i| !i.visual_properties().hidden && i.start_ms() == split)
                    .unwrap();
                assert_eq!(
                    right.visual_properties().animation_channels[0]
                        .clock
                        .unwrap()
                        .offset_ms,
                    split as i64
                );
                assert_eq!(
                    right.visual_properties().animation_channels[0]
                        .clock
                        .unwrap()
                        .source_duration_ms,
                    1000
                );
                let reference = seed(kind, &ffmpeg, "");
                let reference_start = if trim { split } else { 0 };
                let reference_end = if trim { end } else { 1000 };
                reference.edit(json!({"operation":"trim_item","itemId":reference.item,"startMs":reference_start,"durationMs":reference_end-reference_start}));
                reference.edit(json!({"operation":"set_animation_channels","itemId":reference.item,"animationChannels":[independent_grid_channel(mode,reference_start,reference_end)]}));
                let expected = reference.project();
                let authored = &expected
                    .find_item(&reference.item)
                    .unwrap()
                    .visual_properties()
                    .animation_channels[0];
                assert!(authored.clock.is_none() && authored.r#loop.is_none());
                let reference_export =
                    exported(&renderer, &reference, &expected, "independent-grid");
                let reference_range = range(&renderer, &reference, &expected, 800, 1000);
                let before = f.bytes();
                let export = exported(&renderer, &f, &p, "actual");
                let actual_range = range(&renderer, &f, &p, 800, 1000);
                if let Some(destination) = std::env::var_os("OPENCUT_EPIC6_RENDER_EVIDENCE_DIR") {
                    let destination = PathBuf::from(destination).join(format!(
                        "grid-{kind}-{mode}-split{split}-trim{trim}-end{end}"
                    ));
                    std::fs::create_dir_all(&destination).unwrap();
                    for (name, bytes) in [
                        (
                            "actual-project.json",
                            serde_json::to_vec_pretty(&p).unwrap(),
                        ),
                        (
                            "reference-project.json",
                            serde_json::to_vec_pretty(&expected).unwrap(),
                        ),
                        ("actual-export.mp4", std::fs::read(&export).unwrap()),
                        (
                            "reference-export.mp4",
                            std::fs::read(&reference_export).unwrap(),
                        ),
                        ("actual-range.mp4", std::fs::read(&actual_range).unwrap()),
                        (
                            "reference-range.mp4",
                            std::fs::read(&reference_range).unwrap(),
                        ),
                    ] {
                        use std::io::Write;
                        let mut file = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(destination.join(name))
                            .unwrap();
                        file.write_all(&bytes).unwrap();
                    }
                }
                for n in 0..2 {
                    let active = 800 + n * 100 < end;
                    let expected_export = rgb(&ffmpeg, &reference_export, 8 + n);
                    let expected_range = rgb(&ffmpeg, &reference_range, n);
                    compare(
                        &rgb(&ffmpeg, &export, 8 + n),
                        &expected_export,
                        active,
                        &format!(
                            "{kind} {mode} split{split} trim{trim} end{end} export{}",
                            800 + n * 100
                        ),
                    );
                    compare(
                        &rgb(&ffmpeg, &actual_range, n),
                        &expected_range,
                        active,
                        &format!(
                            "{kind} {mode} split{split} trim{trim} end{end} aligned{}",
                            800 + n * 100
                        ),
                    );
                }
                assert_eq!(f.bytes(), before);
            }
        }
        // An authored 0.751 component rate yields source600.8 at root800;
        // the enclosing grid fix must not quantize the inherited loop phase.
        for (mode, value) in [
            ("repeat", 0.2 + 0.6 * 0.8 / 100.0),
            ("ping_pong", 0.2 + 0.6 * 199.2 / 200.0),
        ] {
            let f = seed(kind, &ffmpeg, "");
            f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
            f.edit(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[finite_opacity_channel(mode)]}));
            let mut inherited = f.project();
            let local = inherited.tracks[1].clone();
            inherited.components.push(serde_json::from_value(json!({"id":"fractional-source","name":"Independent fractional source","width":96,"height":96,"durationMs":1000,"slots":[],"markers":[],"tracks":[local]})).unwrap());
            inherited.tracks[1].items=vec![serde_json::from_value(json!({"type":"component_instance","id":"fractional-instance","componentId":"fractional-source","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":0.751,"zIndex":0,"stackOrder":0})).unwrap()];
            let keys: Vec<_>=(0..10).map(|n| {
                let source=n as f64*751.0/10.0;
                let phase=source % if mode=="repeat" {200.0}else{400.0};
                let progress=if mode=="repeat" {
                    if phase<=100.0 {phase/100.0}else{(200.0-phase)/100.0}
                } else if phase<=200.0 {phase/200.0}else{(400.0-phase)/200.0};
                json!({"timeMs":n*100,"value":{"type":"scalar","value":0.2+0.6*progress},"curve":"hold"})
            }).collect();
            assert!((keys[8]["value"]["value"].as_f64().unwrap() - value).abs() < 1e-12);
            control.edit(json!({"operation":"set_animation_channels","itemId":control.item,"animationChannels":[{"property":"transform.opacity","keyframes":keys}]}));
            let reference = exported(
                &renderer,
                &control,
                &control.project(),
                &format!("fractional-{mode}"),
            );
            let actual = exported(&renderer, &f, &inherited, &format!("fractional-{mode}"));
            compare(
                &rgb(&ffmpeg, &actual, 8),
                &rgb(&ffmpeg, &reference, 8),
                true,
                &format!("{kind} {mode} inherited source600.8"),
            );
        }
    }
}

fn directory_inventory(root: &Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(
        root: &Path,
        directory: &Path,
        entries: &mut std::collections::BTreeMap<PathBuf, Option<Vec<u8>>>,
    ) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                entries.insert(relative, None);
                visit(root, &path, entries);
            } else {
                entries.insert(relative, Some(std::fs::read(path).unwrap()));
            }
        }
    }
    let mut entries = std::collections::BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

#[test]
fn native_fullscene_endpoint_errors_and_retained_phase_preserve_reviewed_policy() {
    let Some((ffmpeg, ffprobe, font)) = tools() else {
        return;
    };
    let renderer = Renderer::new(&ffmpeg, &ffprobe, Some(font));
    for kind in ["solid_color", "rectangle"] {
        let f = seed(kind, &ffmpeg, "");
        f.edit(json!({"operation":"trim_item","itemId":f.item,"startMs":0,"durationMs":1000}));
        let mut channel = finite_opacity_channel("repeat");
        channel["clock"] = json!({"offsetMs":700,"sourceDurationMs":1700});
        f.edit(json!({"operation":"set_animation_channels","itemId":f.item,"animationChannels":[channel]}));
        let p = f.project();
        let authoritative = f.bytes();
        let reference = seed(kind, &ffmpeg, "");
        reference.edit(
            json!({"operation":"trim_item","itemId":reference.item,"startMs":0,"durationMs":1000}),
        );
        // Fullspan retained source700 starts at the triangle peak; source800
        // exhausts at root100, before the supported interior900 sample.
        for (at, value) in [(0, 0.8), (100, 0.2), (900, 0.2)] {
            constant_opacity(&reference, value);
            compare(
                &rgb(&ffmpeg, &frame(&renderer, &f, &p, at), 0),
                &rgb(
                    &ffmpeg,
                    &frame(&renderer, &reference, &reference.project(), at),
                    0,
                ),
                true,
                &format!("{kind} fullspan retained at{at}"),
            );
        }
        assert_eq!(f.bytes(), authoritative);
        for at in [1000, 1001] {
            let inventory = directory_inventory(&f.dir);
            let error = renderer.render_preview(&p, &f.dir, at).unwrap_err();
            assert!(!error.retryable);
            assert!(error.ffmpeg_exit_code.is_none());
            if at == 1000 {
                assert_eq!(error.code, ErrorCode::FfmpegFailed);
                assert_eq!(error.failed_stage.as_deref(), Some("publish"));
            } else {
                assert_eq!(error.code, ErrorCode::ValidationFailed);
                assert!(error.failed_stage.is_none());
            }
            assert_eq!(
                directory_inventory(&f.dir),
                inventory,
                "{kind} endpoint{at} changed project directory"
            );
            assert_eq!(f.bytes(), authoritative);
        }
    }
}
