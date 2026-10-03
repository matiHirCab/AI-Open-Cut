use opencut_editor_core::{
    EditOperation, EditorCore, ExportOptions, PathPolicy, PreviewRangeOptions, Project,
    ProjectSettings, Renderer, TimelineItem,
};
use serde_json::{Value, json};
use std::path::Path;

fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
fn scalar(property: &str, last_time: u64, first: f64, last: f64, curve: Value) -> Value {
    json!({"property":property,"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":first},"curve":curve},{"timeMs":last_time,"value":{"type":"scalar","value":last},"curve":"hold"}]})
}
fn decode(ffmpeg: &Path, path: &Path, at: u64) -> Vec<u8> {
    let output = std::process::Command::new(ffmpeg)
        .args(["-v", "error", "-i"])
        .arg(path)
        .args([
            "-ss",
            &format!("{:.6}", at as f64 / 1000.0),
            "-frames:v",
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
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout.len(),
        64 * 64 * 3,
        "{} at {at}",
        path.display()
    );
    output.stdout
}
fn expected(mut project: Project, source: f64, tint: bool) -> Project {
    // Independent equations: time-linear x Bézier has y=t^3; legacy ease-in
    // is t^2; finite ping-pong 0..400..800 returns to its first value forever.
    let progress = (source / 800.0).clamp(0.0, 1.0);
    let phase = if source >= 800.0 {
        0.0
    } else if source <= 400.0 {
        source / 400.0
    } else {
        (800.0 - source) / 400.0
    };
    let srgb = |v: f64| {
        if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    for item in project.tracks.iter_mut().flat_map(|track| &mut track.items) {
        if let TimelineItem::Rectangle(rect) = item {
            rect.transform.position_x = 20.0 + 20.0 * progress.powi(3);
            rect.transform.position_y = 20.0;
            rect.transform.scale = 0.8 + 0.4 * progress.powi(2);
            rect.transform.opacity = 1.0 - 0.5 * phase;
            rect.keyframes.clear();
            rect.visual_properties.animation_channels = serde_json::from_value(json!([scalar(
                "transform.position_x",
                800,
                rect.transform.position_x,
                rect.transform.position_x,
                json!("hold")
            )]))
            .unwrap();
            rect.visual_properties.legacy_animation_clock = None;
            if tint {
                rect.visual_properties.effects=serde_json::from_value(json!([{"id":"tint","type":"color_tint","color":{"r":srgb(1.0-progress),"g":srgb(progress),"b":0,"a":1}}])).unwrap();
            }
        }
    }
    project
}
fn compare(reference: &[u8], actual: &[u8], label: &str) {
    let mse = reference
        .iter()
        .zip(actual)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
        .sum::<f64>()
        / reference.len() as f64;
    assert!(mse < 20.0, "{label}: decoded frame MSE={mse}");
    // Assert location separately so sparse backgrounds cannot hide retiming drift.
    let centroid = |pixels: &[u8]| {
        let mut weight = 0.0;
        let mut x = 0.0;
        let mut y = 0.0;
        for (index, pixel) in pixels.chunks_exact(3).enumerate() {
            let w = f64::from(*pixel.iter().max().unwrap());
            weight += w;
            x += (index % 64) as f64 * w;
            y += (index / 64) as f64 * w;
        }
        assert!(weight > 1000.0);
        (x / weight, y / weight)
    };
    let (x, y) = centroid(reference);
    let (a, b) = centroid(actual);
    assert!(
        (x - a).abs() < 0.4 && (y - b).abs() < 0.4,
        "{label}: centroid expected({x},{y}) actual({a},{b})"
    );
}

#[test]
fn native_edited_bezier_legacy_finite_loop_and_compound_match_all_intents() {
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1"),
            "animation edit render parity requires native tools"
        );
        return;
    };
    for tint in [false, true] {
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
                "Retained source parity",
                ProjectSettings {
                    width: 64,
                    height: 64,
                    fps: 10,
                },
            )
            .unwrap()
            .project_id;
        let track = core.get_project(&id).unwrap().tracks[1].id.clone();
        let item=core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":12,"height":8,"color":"#ffffff","transform":{"positionX":20,"positionY":20,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
        let mut channels = vec![
            scalar(
                "transform.position_x",
                800,
                20.0,
                40.0,
                json!({"type":"cubic_bezier","x1":1.0/3.0,"y1":0,"x2":2.0/3.0,"y2":0}),
            ),
            scalar("transform.opacity", 400, 1.0, 0.5, json!("linear")),
        ];
        channels[1]["loop"] = json!({"mode":"ping_pong","iterations":1});
        if tint {
            core.edit(&id,1,op(json!({"operation":"update_item","itemId":item,"effects":[{"id":"tint","type":"color_tint","color":{"r":1,"g":0,"b":0,"a":1}}]}))).unwrap();
            channels.push(json!({"property":"effect.tint_color","target":{"scope":"root","kind":"effect","id":"tint"},"keyframes":[{"timeMs":0,"value":{"type":"rgba","r":1,"g":0,"b":0,"a":1},"curve":"linear"},{"timeMs":800,"value":{"type":"rgba","r":0,"g":1,"b":0,"a":1},"curve":"hold"}]}));
        }
        let rev = if tint { 2 } else { 1 };
        core.edit(&id,rev,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":channels}))).unwrap();
        core.edit(&id,rev+1,op(json!({"operation":"set_keyframes","itemId":item,"keyframes":[{"property":"scale","timeMs":0,"value":{"type":"scalar","value":0.8},"easing":"ease_in"},{"property":"scale","timeMs":800,"value":{"type":"scalar","value":1.2},"easing":"linear"}]}))).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
        let original = core.get_project(&id).unwrap();
        let split = core
            .edit(
                &id,
                rev + 2,
                op(json!({"operation":"split_item","itemId":item,"splitMs":400})),
            )
            .unwrap();
        let right = split
            .changed_ids
            .iter()
            .find(|id| **id != item)
            .unwrap()
            .clone();
        core.edit(
            &id,
            rev + 3,
            op(json!({"operation":"trim_item","itemId":right,"startMs":500,"durationMs":500})),
        )
        .unwrap();
        let copy = core
            .edit(
                &id,
                rev + 4,
                op(json!({"operation":"duplicate_items","itemIds":[right],"offsetMs":1000})),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let edited = core.get_project(&id).unwrap();
        let draft = core
            .create_draft(
                &id,
                rev + 5,
                vec![op(
                    json!({"operation":"trim_item","itemId":copy,"startMs":1600,"durationMs":400}),
                )],
                None,
            )
            .unwrap();
        let draft = core.get_draft_state(&id, &draft.id).unwrap().project;
        let range = renderer
            .render_preview_range(
                &edited,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 2000,
                    width: 64,
                    height: 64,
                    fps: 10,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let export = root.path().join("exports/edited.mp4");
        std::fs::create_dir_all(export.parent().unwrap()).unwrap();
        renderer
            .export_video(
                &edited,
                &dir,
                ExportOptions {
                    output: &export,
                    width: 64,
                    height: 64,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for at in [200, 500, 700, 1600, 1800, 1900] {
            let source = if at >= 1500 { at - 1000 } else { at };
            let reference = renderer
                .render_preview(
                    &expected(original.clone(), source as f64, tint),
                    &dir,
                    source,
                )
                .unwrap();
            let reference = decode(Path::new(&ffmpeg), &dir.join(reference.relative_path), 0);
            let frame = renderer.render_preview(&edited, &dir, at).unwrap();
            compare(
                &reference,
                &decode(Path::new(&ffmpeg), &dir.join(frame.relative_path), 0),
                &format!("frame at={at} tint={tint}"),
            );
            let frame = renderer.render_preview(&draft, &dir, at).unwrap();
            compare(
                &reference,
                &decode(Path::new(&ffmpeg), &dir.join(frame.relative_path), 0),
                &format!("draft at={at} tint={tint}"),
            );
            for (name, path) in [
                ("range", dir.join(&range.relative_path)),
                ("export", export.clone()),
            ] {
                compare(&reference, &decode(Path::new(&ffmpeg), &path, at), name);
            }
        }
        if !tint {
            // Root frame timestamps are integer, but the inherited 0.751 scale
            // yields source samples such as 600.8ms; no millisecond rounding.
            let mut nested = edited.clone();
            let local = nested.tracks[1].clone();
            nested.components.push(serde_json::from_value(json!({"id":"source","name":"Retained source","width":64,"height":64,"durationMs":2000,"slots":[],"markers":[],"tracks":[local]})).unwrap());
            nested.tracks[1].items=vec![serde_json::from_value(json!({"type":"component_instance","id":"instance","componentId":"source","startMs":0,"durationMs":2600,"trimStartMs":0,"timeScale":0.751,"zIndex":0,"stackOrder":0})).unwrap()];
            let range = renderer
                .render_preview_range(
                    &nested,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 2600,
                        width: 64,
                        height: 64,
                        fps: 10,
                        include_audio: false,
                    },
                    |_| {},
                )
                .unwrap();
            let export = root.path().join("exports/fractional.mp4");
            renderer
                .export_video(
                    &nested,
                    &dir,
                    ExportOptions {
                        output: &export,
                        width: 64,
                        height: 64,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            for at in [800, 1200, 2200, 2400] {
                let source = at as f64 * 0.751;
                let source = if source >= 1500.0 {
                    source - 1000.0
                } else {
                    source
                };
                let reference = renderer
                    .render_preview(&expected(original.clone(), source, false), &dir, 0)
                    .unwrap();
                let reference = decode(Path::new(&ffmpeg), &dir.join(reference.relative_path), 0);
                let frame = renderer.render_preview(&nested, &dir, at).unwrap();
                compare(
                    &reference,
                    &decode(Path::new(&ffmpeg), &dir.join(frame.relative_path), 0),
                    "fractional frame",
                );
                for (name, path) in [
                    ("fractional range", dir.join(&range.relative_path)),
                    ("fractional export", export.clone()),
                ] {
                    compare(&reference, &decode(Path::new(&ffmpeg), &path, at), name);
                }
            }
        }
    }
}

#[test]
fn native_edited_audio_keeps_independent_gain_and_legacy_volume_envelopes() {
    use opencut_editor_core::{MediaProbeFacts, MediaType};
    let (Some(ffmpeg), Some(ffprobe)) = (
        std::env::var_os("OPENCUT_FFMPEG_PATH"),
        std::env::var_os("OPENCUT_FFPROBE_PATH"),
    ) else {
        assert_ne!(
            std::env::var("OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED").as_deref(),
            Ok("1"),
            "audio edit parity requires native tools"
        );
        return;
    };
    for mode in ["volume", "gain", "visual"] {
        let gain = mode == "gain";
        let visual = mode == "visual";
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("media");
        std::fs::create_dir(&media).unwrap();
        let source = media.join("tone.wav");
        let output = std::process::Command::new(&ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=1",
                "-c:a",
                "pcm_f32le",
            ])
            .arg(&source)
            .output()
            .unwrap();
        assert!(output.status.success());
        let probe = std::process::Command::new(&ffprobe)
            .args([
                "-v",
                "error",
                "-select_streams",
                "a:0",
                "-show_packets",
                "-show_entries",
                "packet=size",
                "-of",
                "json",
            ])
            .arg(&source)
            .output()
            .unwrap();
        assert!(probe.status.success());
        let packets: Value = serde_json::from_slice(&probe.stdout).unwrap();
        let packet_samples = packets["packets"][0]["size"]
            .as_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            / 4;
        assert!(packet_samples > 0);
        let source = if visual {
            let video = media.join("visual-tone.mp4");
            let generated = std::process::Command::new(&ffmpeg)
                .args([
                    "-v",
                    "error",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=black:s=64x64:r=10:d=1",
                    "-i",
                ])
                .arg(&source)
                .args(["-c:v", "libx264", "-c:a", "aac", "-shortest"])
                .arg(&video)
                .output()
                .unwrap();
            assert!(
                generated.status.success(),
                "{}",
                String::from_utf8_lossy(&generated.stderr)
            );
            video
        } else {
            source
        };
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
                "Audio source parity",
                ProjectSettings {
                    width: 64,
                    height: 64,
                    fps: 10,
                },
            )
            .unwrap()
            .project_id;
        let asset = core
            .import_asset(
                &id,
                0,
                &source,
                if visual {
                    MediaType::Video
                } else {
                    MediaType::Audio
                },
                MediaProbeFacts {
                    duration_ms: Some(1000),
                    has_video: visual,
                    video_width: if visual { Some(64) } else { None },
                    video_height: if visual { Some(64) } else { None },
                    has_audio: true,
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let track = core.get_project(&id).unwrap().tracks[if visual { 0 } else { 2 }]
            .id
            .clone();
        let item=core.edit(&id,1,op(json!({"operation":"add_media","trackId":track,"assetId":asset,"startMs":0,"sourceInMs":0,"durationMs":1000}))).unwrap().changed_ids[0].clone();
        core.edit(&id,2,op(json!({"operation":"set_animation_channels","itemId":item,"animationChannels":if gain {vec![scalar("audio.gain_db",800,-12.0,0.0,json!("linear"))]} else if visual {vec![scalar("transform.position_x",800,0.0,20.0,json!("linear"))]} else {vec![]}}))).unwrap();
        core.edit(&id,3,op(json!({"operation":"set_keyframes","itemId":item,"keyframes":if gain || visual {json!([])} else {json!([{ "property":"volume","timeMs":0,"value":{"type":"scalar","value":0.4},"easing":"ease_in"},{"property":"volume","timeMs":800,"value":{"type":"scalar","value":1},"easing":"linear"}])}}))).unwrap();
        let raw_duplicate = core
            .create_draft(
                &id,
                4,
                vec![op(
                    json!({"operation":"duplicate_items","itemIds":[item],"offsetMs":1500}),
                )],
                None,
            )
            .unwrap();
        let raw_duplicate = core
            .get_draft_state(&id, &raw_duplicate.id)
            .unwrap()
            .project;
        let split = core
            .edit(
                &id,
                4,
                op(json!({"operation":"split_item","itemId":item,"splitMs":400})),
            )
            .unwrap();
        let right = split
            .changed_ids
            .iter()
            .find(|id| **id != item)
            .unwrap()
            .clone();
        core.edit(
            &id,
            5,
            op(json!({"operation":"trim_item","itemId":right,"startMs":500,"durationMs":500})),
        )
        .unwrap();
        let copy = core
            .edit(
                &id,
                6,
                op(json!({"operation":"duplicate_items","itemIds":[right],"offsetMs":1000})),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let draft = core
            .create_draft(
                &id,
                7,
                vec![op(
                    json!({"operation":"trim_item","itemId":copy,"startMs":1600,"durationMs":400}),
                )],
                None,
            )
            .unwrap();
        let projects = [
            ("raw_duplicate", raw_duplicate),
            ("range", core.get_project(&id).unwrap()),
            (
                "draft",
                core.get_draft_state(&id, &draft.id).unwrap().project,
            ),
        ];
        let dir = core.paths().project_dir(&id).unwrap();
        let renderer = Renderer::new(&ffmpeg, &ffprobe, None);
        for (name, mut project) in projects {
            if gain {
                use opencut_editor_core::{AudioTrackRole, DuckingSettings};
                project.tracks[2].audio_role = AudioTrackRole::Music;
                project.tracks[2].ducking = Some(DuckingSettings {
                    enabled: true,
                    gain: 0.25,
                    attack_ms: 0,
                    release_ms: 0,
                });
                let mut voice = project.tracks[2].clone();
                voice.id = "voice".into();
                voice.audio_role = AudioTrackRole::Voiceover;
                voice.ducking = None;
                let mut item = voice.items[0].clone();
                if let TimelineItem::Media(media) = &mut item {
                    media.id = "quiet-voice".into();
                    media.start_ms = 1400;
                    media.duration_ms = 600;
                    media.source_in_ms = 0;
                    media.audio.volume = 0.000001;
                    media.keyframes.clear();
                    media.visual_properties.animation_channels.clear();
                    media.visual_properties.legacy_animation_clock = None;
                }
                voice.items = vec![item];
                project.tracks.push(voice);
            }
            let range = renderer
                .render_preview_range(
                    &project,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 2000,
                        width: 64,
                        height: 64,
                        fps: 10,
                        include_audio: true,
                    },
                    |_| {},
                )
                .unwrap();
            let export = root.path().join(format!("exports/{name}.mp4"));
            std::fs::create_dir_all(export.parent().unwrap()).unwrap();
            renderer
                .export_video(
                    &project,
                    &dir,
                    ExportOptions {
                        output: &export,
                        width: 64,
                        height: 64,
                        overwrite: false,
                    },
                    |_| {},
                )
                .unwrap();
            let mut outputs = vec![(dir.join(range.relative_path), 0usize), (export, 0usize)];
            if gain {
                let later = renderer
                    .render_preview_range(
                        &project,
                        &dir,
                        PreviewRangeOptions {
                            start_ms: 1500,
                            end_ms: 2000,
                            width: 64,
                            height: 64,
                            fps: 10,
                            include_audio: true,
                        },
                        |_| {},
                    )
                    .unwrap();
                outputs.push((dir.join(later.relative_path), 1500usize));
            }
            for (path, window_start) in outputs {
                let output = std::process::Command::new(&ffmpeg)
                    .args(["-v", "error", "-i"])
                    .arg(&path)
                    .args([
                        "-map", "0:a:0", "-ac", "1", "-ar", "48000", "-f", "f32le", "pipe:1",
                    ])
                    .output()
                    .unwrap();
                assert!(output.status.success());
                let pcm: Vec<f64> = output
                    .stdout
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()) as f64)
                    .collect();
                let silence = if name == "raw_duplicate" {
                    vec![1200usize]
                } else if name == "draft" {
                    vec![450usize, 1200, 1550]
                } else {
                    vec![450usize, 1200]
                };
                for at in silence.into_iter().filter(|at| *at >= window_start) {
                    let start = (at - window_start) * 48 - 240;
                    let end = (at - window_start) * 48 + 240;
                    let rms = (pcm[start..end].iter().map(|v| v * v).sum::<f64>()
                        / (end - start) as f64)
                        .sqrt();
                    assert!(
                        rms < 0.001,
                        "{name} {} gap at{at}: RMS={rms}",
                        path.display()
                    );
                }
                for at in [200usize, 600, 900, 1700, 1900]
                    .into_iter()
                    .filter(|at| *at >= window_start)
                {
                    let (clip_start, clock_offset, source_in) = if name == "raw_duplicate" {
                        (if at >= 1500 { 1500 } else { 0 }, 0usize, 0usize)
                    } else if at >= 1500 {
                        (
                            if name == "draft" { 1600 } else { 1500 },
                            if name == "draft" { 600 } else { 500 },
                            400,
                        )
                    } else if at >= 500 {
                        (500, 500, 400)
                    } else {
                        (0, 0, 0)
                    };
                    let start = (at - window_start) * 48 - 480;
                    let end = (at - window_start) * 48 + 480;
                    // Independently probed mono-float WAV packet sizes define
                    // the existing volume eval=frame discretization. Each packet
                    // holds its analytic source envelope at its start time.
                    let expected = ((start..end)
                        .map(|sample| {
                            let global = sample + window_start * 48;
                            let local = global - clip_start * 48;
                            let source = clock_offset as f64
                                + (local / packet_samples * packet_samples) as f64 / 48.0;
                            let t = (source / 800.0).min(1.0);
                            let envelope = if gain {
                                10.0_f64.powf((-12.0 + 12.0 * t) / 20.0)
                            } else if visual {
                                1.0
                            } else {
                                0.4 + 0.6 * t * t
                            };
                            let duck = if gain && at >= 1400 { 0.25 } else { 1.0 };
                            let amplitude = 0.125
                                * (std::f64::consts::TAU * 440.0 * (local + source_in * 48) as f64
                                    / 48000.0)
                                    .sin()
                                * envelope
                                * duck;
                            amplitude * amplitude
                        })
                        .sum::<f64>()
                        / (end - start) as f64)
                        .sqrt();
                    let rms = (pcm[start..end].iter().map(|v| v * v).sum::<f64>()
                        / (end - start) as f64)
                        .sqrt();
                    assert!(
                        (rms - expected).abs() < 0.002,
                        "{name} {} at{at}: RMS={rms} expected={expected}",
                        path.display()
                    );
                }
            }
        }
    }
}
