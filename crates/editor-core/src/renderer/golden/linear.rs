//! Independent transfer/color and held-media clocks through production rendering.
use super::*;
use serde_json::json;

fn vfr_source(tools: &NativeTools, root: &Path, offset: bool, precise: bool) -> PathBuf {
    let colors = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]];
    let count = if offset {
        24
    } else if precise {
        3
    } else {
        4
    };
    let raw = root.join(if offset { "offset.rgb" } else { "vfr.rgb" });
    let bytes = (0..count)
        .flat_map(|i| colors[i % 4].repeat(4))
        .collect::<Vec<_>>();
    fs::write(&raw, bytes).unwrap();
    let output = root.join(if offset { "offset.mp4" } else { "vfr.mp4" });
    let filter = if precise {
        "settb=1/100000000,setpts='if(eq(N,0),0,if(eq(N,1),10000040,10000060))'"
    } else if offset {
        "settb=1/1000000,setpts=N*100000+mod(N\\,3)*250+5000000"
    } else {
        "settb=1/1000000,setpts='if(eq(N,0),0,if(eq(N,1),100250,if(eq(N,2),300500,1000750)))'"
    };
    let status = Command::new(&tools.ffmpeg)
        .args([
            "-v",
            "error",
            "-nostdin",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            "2x2",
            "-framerate",
            "10",
            "-i",
        ])
        .arg(&raw)
        .args([
            "-vf",
            filter,
            "-fps_mode",
            "passthrough",
            "-c:v",
            "libx264",
            "-qp",
            "0",
            "-g",
            if offset { "4" } else { "10" },
            "-keyint_min",
            if offset { "4" } else { "10" },
            "-sc_threshold",
            "0",
            "-pix_fmt",
            "yuv444p",
            "-enc_time_base",
            if precise { "1/100000000" } else { "1/1000000" },
            "-video_track_timescale",
            if precise { "100000000" } else { "1000000" },
            "-y",
        ])
        .arg(&output)
        .status()
        .unwrap();
    assert!(status.success());
    let probe = Command::new(&tools.ffprobe)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "frame=best_effort_timestamp,best_effort_timestamp_time,key_frame",
            "-of",
            "json",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(probe.status.success());
    let frames: serde_json::Value = serde_json::from_slice(&probe.stdout).unwrap();
    let frames = frames["frames"].as_array().unwrap();
    assert_eq!(frames.len(), count);
    for (i, frame) in frames.iter().enumerate() {
        if precise {
            assert_eq!(
                frame["best_effort_timestamp"].as_i64().unwrap(),
                [0, 10000040, 10000060][i]
            );
            continue;
        }
        let expected = if offset {
            5.0 + i as f64 * 0.1 + (i % 3) as f64 * 0.00025
        } else {
            [0.0, 0.10025, 0.3005, 1.00075][i]
        };
        let actual = frame["best_effort_timestamp_time"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(
            (actual - expected).abs() < 1e-6,
            "independent source PTS{i}: {actual} vs{expected}"
        );
        if offset {
            assert_eq!(frame["key_frame"].as_u64().unwrap(), u64::from(i % 4 == 0));
        }
    }
    output
}

#[test]
fn native_linear_held_vfr_source_clocks() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    let source = vfr_source(&tools, root.path(), false, false);
    let executor = crate::render_process::SystemProcessExecutor;
    for (time, color) in [
        (0.0, [255, 0, 0]),
        (50.0, [255, 0, 0]),
        (100.0, [255, 0, 0]),
        (100.5, [0, 255, 0]),
        (300.4, [0, 255, 0]),
        (300.6, [0, 0, 255]),
        (1100.0, [255, 255, 255]),
    ] {
        let bytes = executor
            .decode_visual_frame_at(&tools.ffmpeg, &source, time, (2, 2))
            .unwrap();
        for pixel in bytes.as_chunks::<4>().0 {
            assert_eq!(pixel[3], 255);
            for c in 0..3 {
                assert!(
                    pixel[c].abs_diff(color[c]) <= 1,
                    "source t{time}: {pixel:?}"
                );
            }
        }
    }
    let precise = vfr_source(&tools, root.path(), false, true);
    for (time, color) in [
        (100.0002, [255, 0, 0]),
        (100.0005, [0, 255, 0]),
        (100.0007, [0, 0, 255]),
    ] {
        let bytes = executor
            .decode_visual_frame_at(&tools.ffmpeg, &precise, time, (2, 2))
            .unwrap();
        for p in bytes.as_chunks::<4>().0 {
            for c in 0..3 {
                assert!(p[c].abs_diff(color[c]) <= 1, "submicro t{time}: {p:?}");
            }
        }
    }

    let cfr = root.path().join("coarse.mkv");
    let status = Command::new(&tools.ffmpeg)
        .args([
            "-v",
            "error",
            "-nostdin",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgb24",
            "-video_size",
            "2x2",
            "-framerate",
            "10",
            "-i",
        ])
        .arg(root.path().join("vfr.rgb"))
        .args(["-c:v", "ffv1", "-pix_fmt", "bgra", "-y"])
        .arg(&cfr)
        .status()
        .unwrap();
    assert!(status.success());
    for (time, color) in [
        (0.0, [255, 0, 0]),
        (99.99, [255, 0, 0]),
        (100.0, [0, 255, 0]),
        (199.99, [0, 255, 0]),
        (200.0, [0, 0, 255]),
    ] {
        let bytes = executor
            .decode_visual_frame_at(&tools.ffmpeg, &cfr, time, (2, 2))
            .unwrap();
        assert_eq!(&bytes[..3], &color, "coarse source boundary t{time}");
    }
    let source = vfr_source(&tools, root.path(), true, false);
    for (time, color) in [
        (0.0, [255, 0, 0]),
        (100.0, [255, 0, 0]),
        (100.5, [0, 255, 0]),
        (1234.5, [255, 0, 0]),
        (1999.9, [255, 255, 255]),
        (2399.9, [255, 255, 255]),
    ] {
        let bytes = executor
            .decode_visual_frame_at(&tools.ffmpeg, &source, time, (2, 2))
            .unwrap();
        for pixel in bytes.as_chunks::<4>().0 {
            for c in 0..3 {
                assert!(
                    pixel[c].abs_diff(color[c]) <= 1,
                    "offset t{time}: {pixel:?}"
                );
            }
        }
    }
}

#[test]
fn native_linear_overlap_matches_frame_range_draft_and_export() {
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
    let id = core
        .create_project(
            "Linear overlap",
            ProjectSettings {
                width: 64,
                height: 64,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let mut project = core.get_project(&id).unwrap();
    let track = project.tracks[1].id.clone();
    core.edit_batch(&id,0,vec![
        serde_json::from_value::<crate::EditOperation>(json!({"operation":"add_rectangle","trackId":track,"width":64,"height":64,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.5}})).unwrap(),
        serde_json::from_value::<crate::EditOperation>(json!({"operation":"add_shape","trackId":track,"geometry":{"type":"rectangle","width":32,"height":32},"fill":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":0.5}},"stroke":null,"startMs":0,"durationMs":1000,"transform2d":{"position":{"x":16,"y":16,"unit":"pixels"},"anchor":{"x":0,"y":0},"scaleX":1,"scaleY":1,"skewXDeg":0,"skewYDeg":0,"rotationDeg":0,"opacity":1}})).unwrap()
    ]).unwrap();
    project = core.get_project(&id).unwrap();
    let dir = core.project_directory(&id).unwrap();
    let draft = core
        .create_draft(&id, project.revision, vec![serde_json::from_value(json!({"operation":"set_item_visibility","itemId":project.tracks[1].items[0].id(),"hidden":false})).unwrap()], None)
        .unwrap();
    let drafted = core.get_draft_state(&id, &draft.id).unwrap().project;
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let preview = renderer.render_preview(&project, &dir, 500).unwrap();
    let draft_frame = renderer.render_preview(&drafted, &dir, 500).unwrap();
    let range = renderer
        .render_preview_range(
            &project,
            &dir,
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 64,
                height: 64,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let output = root.path().join("linear.mp4");
    renderer
        .export_video(
            &project,
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
    let decode = |path: &Path, time: u64| {
        let result = Command::new(&tools.ffmpeg)
            .args([
                "-v",
                "error",
                "-ss",
                &format!("{}", time as f64 / 1000.0),
                "-i",
            ])
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
            .unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout.len(), 64 * 64 * 3);
        result.stdout
    };
    let frames = [
        decode(&dir.join(preview.relative_path), 0),
        decode(&dir.join(draft_frame.relative_path), 0),
        decode(&dir.join(range.relative_path), 500),
        decode(&output, 500),
    ];
    for (intent, frame) in frames.iter().enumerate() {
        // Independent source-over: white(.5) over black, then blue(.5): [.25,.25,.75].
        for (x, y, expected) in [(8, 8, [188, 188, 188]), (24, 24, [137, 137, 225])] {
            let pixel = &frame[(y * 64 + x) * 3..(y * 64 + x) * 3 + 3];
            for c in 0..3 {
                assert!(
                    (i32::from(pixel[c]) - expected[c]).abs() <= 3,
                    "intent{intent} ({x},{y}):{pixel:?}"
                );
            }
        }
        assert!(structural_similarity(&frames[0], frame).unwrap() >= SSIM_MINIMUM);
    }
}

#[test]
fn native_linear_every_current_source_family_uses_normal_scene_blend() {
    let Some(tools) = configured_native_tools() else {
        return;
    };
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let image = root.path().join("assets/red.pam");
    let mut pam =
        b"P7\nWIDTH 24\nHEIGHT 20\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
    pam.extend([255, 0, 0, 255].repeat(24 * 20));
    fs::write(&image, pam).unwrap();
    let mut project = fixture_project();
    project.settings = ProjectSettings {
        width: 240,
        height: 120,
        fps: 10,
    };
    for track in &mut project.tracks {
        track.items.clear();
    }
    let mut asset = project.assets[0].clone();
    asset.id = "image".into();
    asset.media_type = MediaType::Image;
    asset.project_relative_path = "assets/red.pam".into();
    asset.file_name = "red.pam".into();
    asset.has_audio = false;
    project.assets.push(asset);
    let transform =
        |x: f64, y: f64, scale: f64| json!({"positionX":x,"positionY":y,"scale":scale,"opacity":1});
    let doc = crate::validation::svg::parse(
        "<svg width=\"24\" height=\"20\"><rect width=\"24\" height=\"20\" fill=\"#ff0000\"/></svg>",
    )
    .unwrap();
    let values = vec![
        json!({"type":"solid_color","id":"solid","color":"#ff0000","startMs":0,"durationMs":1000,"transform":transform(5.0,5.0,0.1),"keyframes":[]}),
        json!({"type":"rectangle","id":"rectangle","color":"#ff0000","width":24,"height":20,"startMs":0,"durationMs":1000,"transform":transform(40.0,5.0,1.0),"keyframes":[]}),
        json!({"type":"shape","id":"shape","geometry":{"type":"rectangle","width":24,"height":20},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,"startMs":0,"durationMs":1000,"transform":transform(75.0,5.0,1.0),"keyframes":[]}),
        json!({"type":"svg","id":"svg","document":doc,"startMs":0,"durationMs":1000,"transform":transform(110.0,5.0,1.0),"keyframes":[]}),
        json!({"type":"grid","id":"grid","grid":{"width":24,"height":20,"pattern":{"type":"dot","spacingX":8,"spacingY":8,"radius":3,"paint":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}}}},"startMs":0,"durationMs":1000,"transform":transform(145.0,5.0,1.0),"keyframes":[]}),
        json!({"type":"media","id":"image-item","assetId":"image","sourceInMs":0,"startMs":0,"durationMs":1000,"transform":transform(180.0,5.0,1.0),"audio":{"volume":1,"muted":true,"fadeInMs":0,"fadeOutMs":0},"keyframes":[]}),
        json!({"type":"text","id":"text","text":"HH","document":{"runs":[{"text":"HH"}]},"fontSize":20,"color":"#ff0000","startMs":0,"durationMs":1000,"transform":transform(10.0,60.0,1.0),"keyframes":[]}),
        json!({"type":"caption","id":"caption","text":"HH","startMs":0,"durationMs":1000,"style":{"fontSize":20,"color":"#ff0000","backgroundColor":"#000000","bottomMarginPx":15},"source":{"assetId":"tone","providerId":"fixture","modelId":"fixture","language":"en","generatedAtMs":0,"originalText":"HH","words":[]}}),
    ];
    project.tracks[0].items = values
        .into_iter()
        .enumerate()
        .map(|(i, mut v)| {
            v["stackOrder"] = json!(i);
            serde_json::from_value(v).unwrap()
        })
        .collect();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let raw_scene = |project: &Project| {
        let evaluated = evaluate_project(project, 240, 120, 10).unwrap();
        let media = prepare_media_resources(renderer.artifact_io.as_ref(), &evaluated, root.path())
            .unwrap();
        let built = renderer
            .prepare_render(
                &evaluated,
                media,
                root.path(),
                RenderIntent::Frame { at_ms: 500 },
            )
            .unwrap();
        let input = built
            .plan
            .media_inputs
            .iter()
            .find(|i| i.item_id == "linear-scene")
            .unwrap();
        let bytes = fs::read(&built.plan.media_paths[input.input_index - 2]).unwrap();
        let marker = b"ENDHDR\n";
        let offset = bytes
            .windows(marker.len())
            .position(|v| v == marker)
            .unwrap()
            + marker.len();
        bytes[offset..]
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| p[..3].iter().copied())
            .collect::<Vec<_>>()
    };
    let base = raw_scene(&project);
    let regions = [
        (5, 5, 29, 17),
        (40, 5, 64, 25),
        (75, 5, 99, 25),
        (110, 5, 134, 25),
        (145, 5, 169, 25),
        (180, 5, 204, 25),
        (5, 55, 60, 90),
        (95, 80, 145, 120),
    ];
    let points = regions.map(|(l, t, r, b)| {
        (t..b)
            .flat_map(|y| (l..r).map(move |x| (x, y)))
            .max_by_key(|(x, y)| {
                let dx = *x as i32 - (l + r) as i32 / 2;
                let dy = *y as i32 - (t + b) as i32 / 2;
                (base[(y * 240 + x) * 3], -(dx * dx + dy * dy))
            })
            .unwrap()
    });
    for (family, (x, y)) in points.iter().enumerate() {
        let p = &base[(y * 240 + x) * 3..(y * 240 + x) * 3 + 3];
        assert!(
            p[0] >= 240 && p[1] < 5 && p[2] < 5,
            "source family{family} at{x},{y}: {p:?}"
        );
    }
    project.tracks[0].items.push(serde_json::from_value(json!({"type":"rectangle","id":"top-blue","stackOrder":8,"color":"#0000ff","width":240,"height":120,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":0.5},"keyframes":[]})).unwrap());
    let frame = renderer.render_preview(&project, root.path(), 500).unwrap();
    let range = renderer
        .render_preview_range(
            &project,
            root.path(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 240,
                height: 120,
                fps: 10,
                include_audio: false,
            },
            |_| {},
        )
        .unwrap();
    let export = root.path().join("mixed.mp4");
    renderer
        .export_video(
            &project,
            root.path(),
            ExportOptions {
                output: &export,
                width: 240,
                height: 120,
                overwrite: false,
            },
            |_| {},
        )
        .unwrap();
    let observations = [
        grids::decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0),
        grids::decode_rgb_frame(&tools.ffmpeg, &root.path().join(range.relative_path), 500),
        grids::decode_rgb_frame(&tools.ffmpeg, &export, 500),
    ];
    let raw = raw_scene(&project);
    for (family, (x, y)) in points.iter().enumerate() {
        let p = &raw[(y * 240 + x) * 3..(y * 240 + x) * 3 + 3];
        // Glyph coverage can be below one; independently linearize the
        // observed base source, apply blue normal source-over, then encode.
        let start = (y * 240 + x) * 3;
        let expected = std::array::from_fn::<_, 3, _>(|c| {
            let encoded = f64::from(base[start + c]) / 255.0;
            let linear = if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            };
            let out = linear * 0.5 + if c == 2 { 0.5 } else { 0.0 };
            let encoded = if out <= 0.0031308 {
                12.92 * out
            } else {
                1.055 * out.powf(1.0 / 2.4) - 0.055
            };
            (encoded * 255.0).round() as i32
        });
        for (c, expected) in expected.into_iter().enumerate() {
            assert!(
                (i32::from(p[c]) - expected).abs() <= 1,
                "raw family{family} ({x},{y}):{p:?}"
            );
        }
    }
    for rgb in &observations {
        assert!(structural_similarity(&observations[0], rgb).unwrap() >= SSIM_MINIMUM);
    }
}
