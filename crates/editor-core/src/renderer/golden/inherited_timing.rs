//! Native temporal and inherited-transform evidence shared by every render intent.
use super::*;
use serde_json::json;
// Independent output transfer oracle; never calls compositor production helpers.
fn encoded_linear_coverage(alpha: f64) -> f64 {
    255.0
        * if alpha <= 0.0031308 {
            12.92 * alpha
        } else {
            1.055 * alpha.powf(1.0 / 2.4) - 0.055
        }
}

fn fixture() -> Project {
    let mut project = fixture_project();
    project.schema_version = PROJECT_SCHEMA_VERSION;
    project.components.clear();
    let channel = |property: &str, first: f64, last: f64| {
        json!({
            "property":property,"keyframes":[
                {"timeMs":0,"value":{"type":"scalar","value":first},"curve":"linear"},
                {"timeMs":800,"value":{"type":"scalar","value":last},"curve":"hold"}
            ]
        })
    };
    project.tracks[0].items = serde_json::from_value(json!([
        {"type":"group","id":"parent","startMs":0,"durationMs":1000,"staggerMs":100,
         "zIndex":0,"stackOrder":0,"animationChannels":[
            channel("transform.position_x",0.0,40.0),
            channel("transform.scale_x",1.0,2.0),
            channel("transform.scale_y",1.0,1.5),
            channel("transform.opacity",1.0,0.5)
         ]},
        {"type":"shape","id":"red","geometry":{"type":"rectangle","width":10,"height":10},
         "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
         "startMs":0,"durationMs":1000,"keyframes":[],"zIndex":0,"stackOrder":1,
         "parent":{"scope":"root","id":"parent"},
         "transform":{"positionX":10,"positionY":20,"scale":1,"opacity":1}},
        {"type":"shape","id":"blue","geometry":{"type":"rectangle","width":10,"height":10},
         "fill":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"stroke":null,
         "startMs":0,"durationMs":400,"keyframes":[],"zIndex":0,"stackOrder":2,
         "parent":{"scope":"root","id":"parent"},
         "transform":{"positionX":10,"positionY":50,"scale":1,"opacity":1}},
        {"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"zIndex":0,"stackOrder":3,
         "repeater":{"source":{"scope":"root","id":"parent"},"copies":1,"timeOffsetMs":200,
         "transformOffset":{"position":{"x":60,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,
         "rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}}
    ]))
    .unwrap();
    project
}

#[test]
fn native_inherited_timing_render_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut project = fixture();
    project.id = core
        .create_project("Inherited timing", project.settings.clone())
        .unwrap()
        .project_id;
    let directory = core.paths().project_dir(&project.id).unwrap();
    write_tone_wav(&directory.join("assets/tone.wav"));
    fs::write(
        directory.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let draft = core
        .create_draft(
            &project.id,
            project.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"set_item_visibility","itemId":"copies","hidden":false}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let materialized = core
        .get_draft_state(&project.id, &draft.id)
        .unwrap()
        .project;
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let frame = renderer
        .render_preview(&materialized, &directory, 500)
        .unwrap();
    if let Some(path) = env::var_os("OPENCUT_TIMING_CAPTURE") {
        fs::copy(directory.join(&frame.relative_path), path).unwrap();
    }
    let frame_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &directory.join(frame.relative_path), 0);
    let range = renderer
        .render_preview_range(
            &project,
            &directory,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: WIDTH,
                height: HEIGHT,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("inherited-timing.mp4");
    renderer
        .export_video(
            &project,
            &directory,
            ExportOptions {
                output: &output,
                width: WIDTH,
                height: HEIGHT,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let range_rgb =
        grids::decode_rgb_frame(&tools.ffmpeg, &directory.join(&range.relative_path), 500);
    let export_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &output, 500);
    let range_audio = decode_mono_f32(&tools.ffmpeg, &directory.join(&range.relative_path));
    let export_audio = decode_mono_f32(&tools.ffmpeg, &output);
    assert!(aligned_rms_error(&range_audio, &export_audio, 1024).unwrap() <= PCM_RMS_MAXIMUM);
    assert!(range_audio.iter().any(|sample| sample.abs() > 0.01));
    assert!(structural_similarity(&frame_rgb, &range_rgb).unwrap() >= SSIM_MINIMUM);
    assert!(structural_similarity(&range_rgb, &export_rgb).unwrap() >= SSIM_MINIMUM);
    let pixel = |x: usize, y: usize| {
        &frame_rgb[(y * WIDTH as usize + x) * 3..(y * WIDTH as usize + x) * 3 + 3]
    };
    // Independent matrix/clock oracle at t=500:
    // ordinary parent samples 500ms; copy samples 300ms; blue's branch samples
    // 400ms and 200ms respectively, so its ordinary half-open span has ended.
    for (x, y, channel, expected) in [(47, 30, 0, 216), (94, 28, 0, 233), (94, 64, 2, 233)] {
        let rgb = pixel(x, y);
        assert!(
            (i32::from(rgb[channel]) - expected).abs() <= 12,
            "pixel ({x},{y}): {rgb:?}"
        );
        for (index, value) in rgb.iter().enumerate() {
            if index != channel {
                assert!(*value < 12, "{rgb:?}");
            }
        }
    }
    assert!(
        pixel(47, 70).iter().all(|value| *value < 12),
        "ordinary blue must end at 500ms"
    );
}

const NESTED_WIDTH: u32 = 96;
const NESTED_HEIGHT: u32 = 56;

fn nested_fixture() -> Project {
    let mut p = fixture_project();
    p.schema_version = PROJECT_SCHEMA_VERSION;
    p.settings.width = NESTED_WIDTH;
    p.settings.height = NESTED_HEIGHT;
    let keys = |property: &str, a: f64, b: f64, curve: serde_json::Value| json!({"property":property,"loop":{"mode":"ping_pong","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":a},"curve":curve},{"timeMs":200,"value":{"type":"scalar","value":b},"curve":"hold"}]});
    let bezier = json!({"type":"cubic_bezier","x1":0.3333333333333333,"y1":0.0,"x2":0.6666666666666666,"y2":1.0});
    let spring = json!({"type":"spring","mass":1,"stiffness":16,"damping":8,"initialVelocity":0});
    let hidden = |id: &str, order: usize, parent: serde_json::Value| json!({"type":"rectangle","id":id,"startMs":0,"durationMs":1500,"width":1,"height":1,"color":"#ffffff","keyframes":[],"hidden":true,"stackOrder":order,"zIndex":0,"parent":parent});
    let source = json!({"type":"shape","id":"source","startMs":0,"durationMs":300,"stackOrder":1,"zIndex":0,"keyframes":[],"geometry":{"type":"rectangle","width":20,"height":20},"fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,"transform":{"positionX":4,"positionY":8,"scale":1,"opacity":1},"animationChannels":[{"property":"transform.position_x","loop":{"mode":"repeat","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":4},"curve":"linear"},{"timeMs":75,"value":{"type":"scalar","value":10},"curve":"linear"},{"timeMs":150,"value":{"type":"scalar","value":4},"curve":"hold"}]},{"property":"transform.opacity","loop":{"mode":"repeat","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.8},"curve":bezier},{"timeMs":75,"value":{"type":"scalar","value":1.0},"curve":"linear"},{"timeMs":150,"value":{"type":"scalar","value":0.8},"curve":"hold"}]}]});
    let copies = json!({"type":"repeater","id":"copies","startMs":0,"durationMs":800,"stackOrder":2,"zIndex":0,"repeater":{"source":{"scope":"component:leaf","id":"source"},"copies":1,"timeOffsetMs":-25,"opacityOffset":0,"transformOffset":{"position":{"x":50,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}});
    let marker = json!({"type":"media","id":"marker","assetId":"markers","sourceInMs":0,"startMs":0,"durationMs":800,"stackOrder":3,"zIndex":0,"keyframes":[],"transform":{"positionX":4,"positionY":36,"scale":1,"opacity":1},"audio":{"volume":1,"muted":false,"fadeInMs":0,"fadeOutMs":0}});
    let audio = json!({"type":"media","id":"nested-audio","assetId":"tone","sourceInMs":100,"startMs":0,"durationMs":600,"stackOrder":0,"zIndex":0,"keyframes":[],"audio":{"volume":0.5,"muted":false,"fadeInMs":0,"fadeOutMs":0},"animationChannels":[{"property":"audio.gain_db","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":-6},"curve":"hold"}]}]});
    p.components = serde_json::from_value(json!([
        {"id":"outer","name":"Outer","width":NESTED_WIDTH,"height":NESTED_HEIGHT,"durationMs":2000,"slots":[],"markers":[],"tracks":[{"id":"outer-track","name":"Outer","trackType":"overlay","items":[
            {"type":"group","id":"group","startMs":0,"durationMs":2000,"staggerMs":100,"stackOrder":0,"zIndex":0,"animationChannels":[keys("transform.position_x",0.0,4.0,json!("linear")), keys("transform.opacity",0.2,1.0,json!("linear"))]},
            hidden("outer-hidden",1,json!({"scope":"component:outer","id":"group"})),
            {"type":"component_instance","id":"inner","componentId":"leaf","startMs":20,"durationMs":1800,"trimStartMs":10,"timeScale":0.5,"staggerMs":50,"slotValues":{},"stackOrder":2,"zIndex":0,"parent":{"scope":"component:outer","id":"group"},"animationChannels":[keys("transform.position_x",0.0,3.2,spring)]}
        ]}]},
        {"id":"leaf","name":"Leaf","width":NESTED_WIDTH,"height":NESTED_HEIGHT,"durationMs":1500,"slots":[],"markers":[],"tracks":[
            {"id":"leaf-track","name":"Leaf","trackType":"overlay","items":[hidden("leaf-hidden",0,serde_json::Value::Null),source,copies,marker]},
            {"id":"leaf-audio","name":"Audio","trackType":"audio","items":[audio]}
        ]}
    ])).unwrap();
    p.tracks[0].items = serde_json::from_value(json!([
        {"type":"group","id":"outside","startMs":0,"durationMs":1000,"stackOrder":0,"zIndex":0,"animationChannels":[keys("transform.position_x",0.0,4.0,json!("linear"))]},
        {"type":"component_instance","id":"root-instance","componentId":"outer","startMs":100,"durationMs":700,"trimStartMs":50,"timeScale":1.5,"slotValues":{},"stackOrder":1,"zIndex":0,"parent":{"scope":"root","id":"outside"}}
    ])).unwrap();
    p.tracks[1].items.clear();
    p.assets.push(serde_json::from_value(json!({"id":"markers","mediaType":"video","fileName":"markers.mkv","projectRelativePath":"assets/markers.mkv","durationMs":1000,"hasAudio":false})).unwrap());
    p
}

// These expected functions deliberately do not use the production curve sampler.
fn ping_phase(t: f64) -> f64 {
    let t = t.max(0.0) % 400.0;
    if t <= 200.0 {
        t / 200.0
    } else {
        (400.0 - t) / 200.0
    }
}
fn expected_parent_x(t: f64) -> f64 {
    let outer = (t - 100.0) * 1.5 + 50.0;
    let inner = outer - 100.0 - 20.0;
    let u = ping_phase(inner);
    // Critical damped spring with omega=4, exact endpoints use stored values.
    let spring = if u == 1.0 {
        1.0
    } else {
        1.0 - (1.0 + 4.0 * u) * (-4.0 * u).exp()
    };
    4.0 * ping_phase(t) + 4.0 * ping_phase(outer) + 3.2 * spring
}
fn expected_source_x(local: f64) -> f64 {
    let phase = local.max(0.0) % 150.0;
    4.0 + 6.0
        * if phase <= 75.0 {
            phase / 75.0
        } else {
            (150.0 - phase) / 75.0
        }
}
fn assert_nested_pixels(rgb: &[u8], t: u64) {
    let t = t as f64;
    let raw_leaf = ((t - 100.0) * 1.5 + 50.0 - 100.0 - 20.0) * 0.5 + 10.0;
    for (delay, translation) in [(50.0, 0.0), (125.0, 50.0)] {
        let local = raw_leaf - delay;
        let active = (100.0..800.0).contains(&t) && (0.0..300.0).contains(&local);
        let expected = expected_parent_x(t) + expected_source_x(local) + translation;
        let lit: Vec<usize> = (translation as usize..(translation as usize + 46))
            .filter(|x| rgb[(16 * NESTED_WIDTH as usize + x) * 3] > 20)
            .collect();
        if active {
            assert!(!lit.is_empty(), "missing shape t={t} delay={delay}");
            assert!(
                (lit[0] as f64 - expected).abs() <= 2.0,
                "left t={t} delay={delay}: {:?} expected={expected}",
                lit
            );
            let center = (expected + 10.0) as usize;
            let pixel = &rgb[(16 * NESTED_WIDTH as usize + center) * 3
                ..(16 * NESTED_WIDTH as usize + center) * 3 + 3];
            let phase = local % 150.0;
            let triangle = if phase <= 75.0 {
                phase / 75.0
            } else {
                (150.0 - phase) / 75.0
            };
            let outer = (t - 100.0) * 1.5 + 50.0;
            let phase = ping_phase(outer);
            let parent_alpha = 0.2 + 0.8 * phase;
            let source_alpha = if (local % 150.0) <= 75.0 {
                0.8 + 0.2 * (3.0 * triangle * triangle - 2.0 * triangle * triangle * triangle)
            } else {
                0.8 + 0.2 * triangle
            };
            let expected_alpha = parent_alpha * source_alpha;
            assert!(
                pixel.iter().all(|value| (i32::from(*value)
                    - (encoded_linear_coverage(expected_alpha)).round() as i32)
                    .abs()
                    <= 12),
                "opacity t={t}: {pixel:?}"
            );
        } else {
            assert!(lit.is_empty(), "half-open clipping t={t}: {lit:?}");
        }
    }
    // Rank-three media delay is 150 local ms; source samples are 10 fps markers.
    if (100.0..800.0).contains(&t) && raw_leaf >= 150.0 {
        // Hold the last source presentation timestamp at or before the exact
        // mapped media clock, independently subtracting the rank-three delay.
        // At root 700 ms, source 275 ms holds frame 2 rather than future frame 3.
        let frame = ((raw_leaf - 150.0) / 100.0).floor() as usize;
        let outer = (t - 100.0) * 1.5 + 50.0;
        let phase = ping_phase(outer);
        let alpha = 0.2 + 0.8 * phase;
        let source = if frame.is_multiple_of(2) { 240.0 } else { 80.0 };
        let s: f64 = source / 255.0;
        let linear = if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        };
        let value = encoded_linear_coverage(linear * alpha).round() as i32;
        let expected = [value, value, value];
        let x = (expected_parent_x(t) + 14.0) as usize;
        let pixel =
            &rgb[(44 * NESTED_WIDTH as usize + x) * 3..(44 * NESTED_WIDTH as usize + x) * 3 + 3];
        for i in 0..3 {
            assert!(
                (i32::from(pixel[i]) - expected[i]).abs() <= 12,
                "source marker t={t} frame={frame}: {pixel:?}"
            );
        }
    }
}

#[test]
fn native_inherited_timing_nested_independent_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut project = nested_fixture();
    project.id = core
        .create_project("Nested oracle", project.settings.clone())
        .unwrap()
        .project_id;
    let dir = core.paths().project_dir(&project.id).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    // Lossless 10 fps 20-pixel source markers, authored independently of scene evaluation.
    let mut raw = Vec::new();
    for frame in 0usize..10 {
        for _ in 0..400 {
            raw.extend_from_slice(if frame.is_multiple_of(2) {
                &[240, 240, 240]
            } else {
                &[80, 80, 80]
            });
        }
    }
    let input = dir.join("markers.rgb");
    fs::write(&input, raw).unwrap();
    assert!(
        Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-video_size",
                "20x20",
                "-framerate",
                "10",
                "-i"
            ])
            .arg(&input)
            .args(["-c:v", "ffv1"])
            .arg(dir.join("assets/markers.mkv"))
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let draft=core.create_draft(&project.id,project.revision,vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":"root-instance","hidden":false})).unwrap()],None).unwrap();
    let draft_project = core
        .get_draft_state(&project.id, &draft.id)
        .unwrap()
        .project;
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: NESTED_WIDTH,
                height: NESTED_HEIGHT,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("nested.mp4");
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: NESTED_WIDTH,
                height: NESTED_HEIGHT,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    // Independent audio graph: unstaggered leaf clock = .75*t-50 ms,
    // source trim=100 ms, active root interval [100,800), gain=.5*10^(-6/20).
    // Build an AAC reference from those constants without any scene/plan helper.
    // Keep source trim before atempo, then clip its root interval: cropping before
    // tempo instead changes the DSP window. Mixing with independently authored
    // one-second silence preserves AAC packet timestamps and output duration.
    let reference = root.path().join("audio-reference.m4a");
    assert!(Command::new(&tools.ffmpeg)
        .args(["-v","error","-ss","0.1","-t","0.6","-i"]).arg(dir.join("assets/tone.wav"))
        .args(["-f","lavfi","-i","anullsrc=r=48000:cl=stereo:d=1","-filter_complex",
            "[0:a]atrim=duration=0.6,asetpts=PTS-STARTPTS,volume=0.2505936168136361,atempo=0.75,asetpts=PTS+0.06666666666666667/TB,atrim=start=0.1:end=0.8,asetpts=PTS-STARTPTS,adelay=100:all=1[a];[1:a][a]amix=inputs=2:duration=longest:normalize=0[out]",
            "-map","[out]","-c:a","aac","-t","1"]).arg(&reference).status().unwrap().success());
    let expected = decode_mono_f32(&tools.ffmpeg, &reference);
    let actual = decode_mono_f32(&tools.ffmpeg, &dir.join(&range.relative_path));
    let exported = decode_mono_f32(&tools.ffmpeg, &output);
    eprintln!(
        "independent audio lengths: range={} export={} reference={}",
        actual.len(),
        exported.len(),
        expected.len()
    );
    fs::copy(
        dir.join(&range.relative_path),
        env::temp_dir().join("issue42-nested-range.mp4"),
    )
    .unwrap();
    fs::copy(&output, env::temp_dir().join("issue42-nested-export.mp4")).unwrap();
    fs::copy(
        &reference,
        env::temp_dir().join("issue42-nested-audio-reference.m4a"),
    )
    .unwrap();
    assert!(aligned_rms_error(&actual, &exported, 1024).unwrap() <= PCM_RMS_MAXIMUM);
    assert!(
        aligned_rms_error(&actual, &expected, 1024).unwrap() <= PCM_RMS_MAXIMUM,
        "independent audio RMS {}",
        aligned_rms_error(&actual, &expected, 1024).unwrap()
    );
    let first = actual.iter().position(|s| s.abs() > 0.01).unwrap();
    let last = actual.iter().rposition(|s| s.abs() > 0.01).unwrap();
    assert!(first.abs_diff(4800) <= 1024, "audio onset: {first}");
    assert!(last.abs_diff(38400) <= 1024, "audio exclusive end: {last}");
    assert!(actual[..4000].iter().all(|s| s.abs() < 0.001));
    assert!(actual[6000..30000].iter().any(|s| s.abs() > 0.01));
    for t in [0, 100, 200, 300, 400, 500, 600, 700, 800, 900] {
        eprintln!("independent nested oracle t={t}");
        let frame = renderer.render_preview(&project, &dir, t).unwrap();
        let rgb = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0);
        let d = renderer.render_preview(&draft_project, &dir, t).unwrap();
        let d_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(d.relative_path), 0);
        let r_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(&range.relative_path), t);
        let e_rgb = grids::decode_rgb_frame(&tools.ffmpeg, &output, t);
        for observation in [&rgb, &d_rgb, &r_rgb, &e_rgb] {
            assert_nested_pixels(observation, t);
        }
        assert!(structural_similarity(&rgb, &d_rgb).unwrap() >= SSIM_MINIMUM);
        let ssim = structural_similarity(&rgb, &r_rgb).unwrap();
        if ssim < SSIM_MINIMUM {
            fs::write(env::temp_dir().join(format!("issue42-frame-{t}.rgb")), &rgb).unwrap();
            fs::write(
                env::temp_dir().join(format!("issue42-range-{t}.rgb")),
                &r_rgb,
            )
            .unwrap();
        }
        assert!(ssim >= SSIM_MINIMUM, "frame/range SSIM t={t}: {ssim}");
        assert!(structural_similarity(&r_rgb, &e_rgb).unwrap() >= SSIM_MINIMUM);
    }
}

#[test]
fn native_inherited_timing_media_half_open_conformance() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let mut project = nested_fixture();
    // Isolate activity from curve raster work. Fractional clocks, hidden ranks,
    // signed copies and the root [100,800) boundary remain unchanged.
    for track in project
        .tracks
        .iter_mut()
        .chain(project.components.iter_mut().flat_map(|c| &mut c.tracks))
    {
        for item in &mut track.items {
            item.visual_properties_mut().animation_channels.clear();
        }
    }
    project.id = core
        .create_project("Media exclusive ends", project.settings.clone())
        .unwrap()
        .project_id;
    let dir = core.paths().project_dir(&project.id).unwrap();
    write_tone_wav(&dir.join("assets/tone.wav"));
    assert!(
        Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=white:s=20x20:r=10:d=1",
                "-c:v",
                "ffv1",
            ])
            .arg(dir.join("assets/markers.mkv"))
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let draft = core
        .create_draft(
            &project.id,
            project.revision,
            vec![
                serde_json::from_value(json!({
                    "operation":"set_item_visibility", "itemId":"root-instance", "hidden":false,
                }))
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let draft_project = core
        .get_draft_state(&project.id, &draft.id)
        .unwrap()
        .project;
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: NESTED_WIDTH,
                height: NESTED_HEIGHT,
                fps: FPS,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("media-ends.mp4");
    renderer
        .export_video(
            &project,
            &dir,
            ExportOptions {
                output: &output,
                width: NESTED_WIDTH,
                height: NESTED_HEIGHT,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    for t in [0, 700, 800, 900] {
        let frame = renderer.render_preview(&project, &dir, t).unwrap();
        let draft_frame = renderer.render_preview(&draft_project, &dir, t).unwrap();
        for rgb in [
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0),
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(draft_frame.relative_path), 0),
            grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(&range.relative_path), t),
            grids::decode_rgb_frame(&tools.ffmpeg, &output, t),
        ] {
            if t == 700 {
                // Independent static position (4,36), 20px marker; no ancestor
                // channels. Source and copied shapes have both already ended.
                let center = (44 * NESTED_WIDTH as usize + 14) * 3;
                assert!(
                    rgb[center..center + 3]
                        .iter()
                        .all(|v| v.abs_diff(255) <= 12),
                    "media interior t={t}"
                );
            } else {
                // Root start is 100, exclusive end is 800: every pixel must be
                // absent, not merely equal to another output of the evaluator.
                assert!(rgb.iter().all(|v| *v <= 2), "media clipping t={t}");
            }
        }
    }
}

// Independent native reproduction and nested hidden-rank/signed-copy extension.
#[test]
fn native_inherited_timing_fractional_loop_seams() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    for signed in [None, Some(1), Some(-1)] {
        let root = tempdir().unwrap();
        let core = crate::EditorCore::new(
            crate::PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let mut p = fixture_project();
        p.schema_version = PROJECT_SCHEMA_VERSION;
        p.settings.width = 200;
        p.settings.height = 40;
        p.assets.clear();
        let channel = |property: &str, a: f64, b: f64| json!({"property":property,"loop":{"mode":"repeat","iterations":"infinite"},"keyframes":[{"timeMs":0,"value":{"type":"scalar","value":a},"curve":"linear"},{"timeMs":99,"value":{"type":"scalar","value":b},"curve":"linear"},{"timeMs":100,"value":{"type":"scalar","value":a},"curve":"hold"}]});
        let group = json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":0,"staggerMs":if signed.is_some(){1}else{0},"animationChannels":[channel("transform.position_x",0.0,100.0)]});
        let mut child = json!({"type":"shape","id":"child","startMs":0,"durationMs":1000,"stackOrder":if signed.is_some(){2}else{1},"parent":{"scope":"component:leaf","id":"parent"},"geometry":{"type":"rectangle","width":10,"height":10},"fill":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},"stroke":null,"keyframes":[],"transform":{"positionX":10,"positionY":20,"scale":1,"opacity":1}});
        let mut items = vec![group];
        if signed.is_some() {
            child["animationChannels"] = json!([channel("transform.opacity", 0.2, 0.8)]);
            items.push(json!({"type":"rectangle","id":"hidden","hidden":true,"startMs":0,"durationMs":1000,"stackOrder":1,"width":1,"height":1,"color":"#ffffff","keyframes":[],"parent":{"scope":"component:leaf","id":"parent"}}));
        }
        items.push(child);
        if let Some(offset) = signed {
            items.push(json!({"type":"repeater","id":"copy","startMs":0,"durationMs":1000,"stackOrder":3,"repeater":{"source":{"scope":"component:leaf","id":"parent"},"copies":1,"timeOffsetMs":offset,"opacityOffset":0,"transformOffset":{"position":{"x":50,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0}}}));
        }
        p.components=serde_json::from_value(json!([{"id":"leaf","name":"Leaf","width":200,"height":40,"durationMs":1200,"slots":[],"markers":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":items}]}])).unwrap();
        let component = if signed.is_some() { "outer" } else { "leaf" };
        if signed.is_some() {
            p.components.push(serde_json::from_value(json!({"id":"outer","name":"Outer","width":200,"height":40,"durationMs":2200,"slots":[],"markers":[],"tracks":[{"id":"outer-track","name":"Outer","trackType":"overlay","items":[{"type":"rectangle","id":"rank-zero","hidden":true,"stackOrder":0,"startMs":0,"durationMs":2200,"width":1,"height":1,"color":"#ffffff","keyframes":[]},{"type":"component_instance","id":"inner","componentId":"leaf","stackOrder":1,"startMs":0,"durationMs":2200,"trimStartMs":0,"timeScale":0.5,"staggerMs":1,"slotValues":{}}]}]})).unwrap());
        }
        p.tracks[0].items=serde_json::from_value(json!([{"type":"component_instance","id":"root","componentId":component,"startMs":0,"durationMs":1000,"trimStartMs":if signed.is_some(){1}else{0},"timeScale":if signed.is_some(){1.99}else{0.995},"staggerMs":if signed.is_some(){1}else{0},"slotValues":{}}])).unwrap();
        for track in p.tracks.iter_mut().skip(1) {
            track.items.clear();
        }
        p.id = core
            .create_project("Fractional phase", p.settings.clone())
            .unwrap()
            .project_id;
        let dir = core.paths().project_dir(&p.id).unwrap();
        fs::write(dir.join("project.json"), serde_json::to_vec(&p).unwrap()).unwrap();
        let draft = core
            .create_draft(
                &p.id,
                p.revision,
                vec![
                    serde_json::from_value(
                        json!({"operation":"set_item_visibility","itemId":"root","hidden":false}),
                    )
                    .unwrap(),
                ],
                None,
            )
            .unwrap();
        let draft_project = core.get_draft_state(&p.id, &draft.id).unwrap().project;
        let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
        let range = renderer
            .render_preview_range(
                &p,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 200,
                    height: 40,
                    fps: FPS,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let export = root.path().join("fractional.mp4");
        renderer
            .export_video(
                &p,
                &dir,
                ExportOptions {
                    output: &export,
                    width: 200,
                    height: 40,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        for t in [100, 200] {
            let frame = renderer.render_preview(&p, &dir, t).unwrap();
            let draft_frame = renderer.render_preview(&draft_project, &dir, t).unwrap();
            let observations = [
                grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(frame.relative_path), 0),
                grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(draft_frame.relative_path), 0),
                grids::decode_rgb_frame(&tools.ffmpeg, &dir.join(&range.relative_path), t),
                grids::decode_rgb_frame(&tools.ffmpeg, &export, t),
            ];
            let triangle = |local: f64| {
                let phase = local.rem_euclid(100.0);
                if phase <= 99.0 {
                    phase / 99.0
                } else {
                    100.0 - phase
                }
            };
            // Nested map: ((1.99*t + trim 1 - hidden-rank delay 1)*.5) = .995*t.
            // The rank-one controller then shifts the group by 1 + signed offset.
            for (copy, delay) in [
                (false, 0.0),
                (true, signed.map_or(0.0, |v| 1.0 + f64::from(v))),
            ] {
                if copy && signed.is_none() {
                    continue;
                }
                let parent_time = 0.995 * t as f64 - delay;
                let x = 10.0 + 100.0 * triangle(parent_time) + if copy { 50.0 } else { 0.0 };
                let alpha = if signed.is_some() {
                    0.2 + 0.6 * triangle(parent_time - 1.0)
                } else {
                    1.0
                };
                for rgb in &observations {
                    let lane = ((x - 4.0).max(0.0) as usize)..((x + 15.0).min(200.0) as usize);
                    let lit = lane
                        .filter(|x| rgb[(24 * 200 + x) * 3] > 20)
                        .collect::<Vec<_>>();
                    assert!(
                        !lit.is_empty(),
                        "missing t={t} signed={signed:?} copy={copy}"
                    );
                    assert!(
                        (lit[0] as f64 - x).abs() <= 1.0,
                        "t={t} signed={signed:?} copy={copy}: left={} expected={x}",
                        lit[0]
                    );
                    let center = x.floor() as usize + 5;
                    assert!(
                        (f64::from(rgb[(24 * 200 + center) * 3]) - encoded_linear_coverage(alpha))
                            .abs()
                            <= 12.0,
                        "independent opacity t={t} signed={signed:?} copy={copy}"
                    );
                }
            }
            assert!(
                structural_similarity(&observations[0], &observations[1]).unwrap() >= SSIM_MINIMUM
            );
            assert!(
                structural_similarity(&observations[0], &observations[2]).unwrap() >= SSIM_MINIMUM
            );
            assert!(
                structural_similarity(&observations[2], &observations[3]).unwrap() >= SSIM_MINIMUM
            );
        }
    }
}
