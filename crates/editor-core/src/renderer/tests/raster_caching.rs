use super::*;
use crate::render_artifact::raster_cache::tests::fixture;
use serde_json::json;

fn renderer(io: Arc<LifecycleArtifactIo>) -> Renderer {
    Renderer::new("unused-ffmpeg", "unused-ffprobe", None).with_adapters(
        Arc::new(FakeProcess {
            readiness_error: false,
            probe_error: false,
            run_failure: None,
            executions: Mutex::new(vec![]),
        }),
        io,
    )
}

fn rasters(renderer: &Renderer, project: &Project, dir: &Path) -> Vec<Vec<u8>> {
    let evaluated = evaluate_project(
        project,
        project.settings.width,
        project.settings.height,
        project.settings.fps,
    )
    .unwrap();
    let media = prepare_media_resources(renderer.artifact_io.as_ref(), &evaluated, dir).unwrap();
    let built = renderer
        .prepare_render(&evaluated, media, dir, RenderIntent::Export)
        .unwrap();
    let mut files: Vec<_> = std::fs::read_dir(built._workspace.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pam"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| std::fs::read(p).unwrap())
        .collect()
}

fn update(item: &str, text: &str) -> crate::EditOperation {
    serde_json::from_value(json!({"operation":"update_item","itemId":item,"text":text})).unwrap()
}

#[test]
fn warm_cache_drafts_revisions_history_and_reopen() {
    let (_root, core, mut project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let renderer = renderer(Arc::default());
    let first = rasters(&renderer, &project, &dir);
    assert_eq!(first.len(), 1);
    assert_eq!(first, rasters(&renderer.clone(), &project, &dir));
    assert_eq!(renderer.raster_cache.misses.load(Ordering::Relaxed), 1);
    assert_eq!(renderer.raster_cache.hits.load(Ordering::Relaxed), 1);
    let item = project.tracks[1].items[0].id().to_owned();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    let mut drafts = vec![];
    for text in ["Draft A", "Draft B"] {
        let draft = core
            .create_draft(
                &project.id,
                project.revision,
                vec![update(&item, text)],
                None,
            )
            .unwrap();
        let snapshot = core
            .get_draft_state(&project.id, &draft.id)
            .unwrap()
            .project;
        assert_eq!(snapshot.revision, project.revision);
        drafts.push(rasters(&renderer, &snapshot, &dir));
    }
    assert_ne!(drafts[0], drafts[1]);
    assert_eq!(before, std::fs::read(dir.join("project.json")).unwrap());
    assert_eq!(
        core.edit(&project.id, 0, update(&item, "stale"))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        core.edit(&project.id, project.revision, update("missing", "x"))
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert!(
        core.edit_batch(
            &project.id,
            project.revision,
            vec![update(&item, "rollback"), update("missing", "x")]
        )
        .is_err()
    );
    assert_eq!(before, std::fs::read(dir.join("project.json")).unwrap());
    assert_eq!(first, rasters(&renderer, &project, &dir));
    core.edit(&project.id, project.revision, update(&item, "Changed"))
        .unwrap();
    project = core.get_project(&project.id).unwrap();
    let changed = rasters(&renderer, &project, &dir);
    assert_ne!(first, changed);
    core.undo(&project.id, project.revision).unwrap();
    project = core.get_project(&project.id).unwrap();
    assert_eq!(first, rasters(&renderer, &project, &dir));
    core.redo(&project.id, project.revision).unwrap();
    project = core.get_project(&project.id).unwrap();
    assert_eq!(changed, rasters(&renderer, &project, &dir));
    let fresh = super::raster_caching::renderer(Arc::default());
    assert_eq!(
        changed,
        rasters(&fresh, &core.get_project(&project.id).unwrap(), &dir)
    );
    let persisted = std::fs::read_to_string(dir.join("project.json")).unwrap();
    assert!(!persisted.contains("raster_cache"));
}

#[test]
fn warm_cache_preserves_preflight_errors_and_cleanup() {
    let (_root, core, project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let io = Arc::new(LifecycleArtifactIo::default());
    let renderer = renderer(io.clone());
    rasters(&renderer, &project, &dir);
    let hits = renderer.raster_cache.hits.load(Ordering::Relaxed);
    let misses = renderer.raster_cache.misses.load(Ordering::Relaxed);
    for corrupt in [false, true] {
        let font = dir.join(&project.fonts.values().next().unwrap().relative_path);
        let bytes = std::fs::read(&font).unwrap();
        if corrupt {
            std::fs::write(&font, b"invalid").unwrap();
        } else {
            std::fs::remove_file(&font).unwrap();
        }
        assert_eq!(
            renderer.render_preview(&project, &dir, 0).unwrap_err().code,
            ErrorCode::AssetIntegrityFailed
        );
        std::fs::write(&font, bytes).unwrap();
        assert_eq!(hits, renderer.raster_cache.hits.load(Ordering::Relaxed));
        assert_eq!(misses, renderer.raster_cache.misses.load(Ordering::Relaxed));
    }
    for invalid in [f64::NAN, f64::INFINITY, 1e100] {
        let mut bad = project.clone();
        let TimelineItem::Text(t) = &mut bad.tracks[1].items[0] else {
            panic!()
        };
        t.transform.scale = invalid;
        assert_eq!(
            renderer.render_preview(&bad, &dir, 0).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(hits, renderer.raster_cache.hits.load(Ordering::Relaxed));
        assert_eq!(misses, renderer.raster_cache.misses.load(Ordering::Relaxed));
    }
    let mut bad = project.clone();
    let TimelineItem::Text(t) = &mut bad.tracks[1].items[0] else {
        panic!()
    };
    t.font_binding.as_mut().unwrap().regular = "0".repeat(64);
    assert!(renderer.render_preview(&bad, &dir, 0).is_err());
    assert_eq!(hits, renderer.raster_cache.hits.load(Ordering::Relaxed));
    assert_eq!(misses, renderer.raster_cache.misses.load(Ordering::Relaxed));
    let mut escaped = project.clone();
    escaped.fonts.values_mut().next().unwrap().relative_path = "../escape.ttf".into();
    assert!(renderer.render_preview(&escaped, &dir, 0).is_err());
    assert_eq!(hits, renderer.raster_cache.hits.load(Ordering::Relaxed));
    assert_eq!(misses, renderer.raster_cache.misses.load(Ordering::Relaxed));
    io.fail_next(ArtifactFailure::Write);
    assert_eq!(
        renderer.render_preview(&project, &dir, 0).unwrap_err().code,
        ErrorCode::FfmpegFailed
    );
    assert!(renderer.raster_cache.hits.load(Ordering::Relaxed) > hits);
    assert!(std::fs::read_dir(&dir).unwrap().all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-work-")
    }));
}

#[test]
fn occurrence_and_composition_changes_reuse_rasters() {
    let (_root, core, mut project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let renderer = renderer(Arc::default());
    let original = rasters(&renderer, &project, &dir);
    let mut duplicate = project.tracks[1].items[0].clone();
    let TimelineItem::Text(t) = &mut duplicate else {
        panic!()
    };
    t.id = "copy".into();
    t.transform.position_x = 20.;
    t.transform.opacity = 0.5;
    t.start_ms = 100;
    t.duration_ms = 900;
    duplicate.visual_properties_mut().stack_order = 1;
    project.tracks[1].items.push(duplicate);
    let evaluated = evaluate_project(&project, 160, 90, 10).unwrap();
    let copy = evaluated
        .scene
        .visual_layers
        .iter()
        .find(|layer| layer.item_id == "copy")
        .unwrap();
    assert_eq!(copy.transform.position_x, 20.0);
    assert_eq!(copy.transform.opacity, 0.5);
    assert_eq!(copy.span.start_ms, 100);
    let copies = rasters(&renderer, &project, &dir);
    assert_eq!(copies, vec![original[0].clone(), original[0].clone()]);
    assert_eq!(renderer.raster_cache.misses.load(Ordering::Relaxed), 1);
    let local = project.tracks[1].items[0].clone();
    project.components.push(serde_json::from_value(json!({"id":"text-component","name":"Text","width":160,"height":90,"durationMs":1000,"slots":[],"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[local]}]})).unwrap());
    project.tracks[1].items.push(serde_json::from_value(json!({"type":"component_instance","id":"instance","componentId":"text-component","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"slotValues":{},"stackOrder":2})).unwrap());
    project.tracks[1].items.push(serde_json::from_value(json!({"type":"repeater","id":"copies","startMs":0,"durationMs":1000,"stackOrder":3,"repeater":{"source":{"scope":"root","id":"instance"},"copies":2,"transformOffset":{"position":{"x":10,"y":0,"unit":"pixels"},"scaleX":1,"scaleY":1,"rotationDeg":0,"skewXDeg":0,"skewYDeg":0},"opacityOffset":0}})).unwrap());
    let expanded = rasters(&renderer, &project, &dir);
    assert_eq!(expanded.len(), 5);
    assert!(expanded.iter().all(|bytes| bytes == &original[0]));
    assert_eq!(renderer.raster_cache.misses.load(Ordering::Relaxed), 1);
}

#[test]
fn valid_dependency_variants_match_fresh_and_keep_unrelated_keys() {
    let (_root, core, mut base) = fixture();
    let dir = core.project_directory(&base.id).unwrap();
    let track = &base.tracks[1].id;
    let operations: Vec<crate::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_shape","trackId":track,"startMs":0,"durationMs":1000,"geometry":{"type":"rectangle","width":20,"height":15},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null},
        {"operation":"add_svg","trackId":track,"startMs":0,"durationMs":1000,"svg":"<svg width=\"20\" height=\"20\"><rect width=\"20\" height=\"20\" fill=\"#f00\"/><rect width=\"10\" height=\"10\" fill=\"#00f\"/></svg>"}
    ])).unwrap();
    core.edit_batch(&base.id, base.revision, operations)
        .unwrap();
    base = core.get_project(&base.id).unwrap();
    let warm = renderer(Arc::default());
    let original = rasters(&warm, &base, &dir);
    for field in 0..21 {
        let mut changed = base.clone();
        let TimelineItem::Text(text) = &mut changed.tracks[1].items[0] else {
            panic!()
        };
        match field {
            0 => { text.text = "Different".into(); text.document = crate::RichTextDocument::plain(text.text.clone()); }
            1 => text.document.runs[0].bold = Some(true),
            2 => text.document.spans = Some(serde_json::from_value(json!([{"start":0,"end":2,"style":{"color":"#ff0000"}}])).unwrap()),
            3 => text.style.paint_layers = Some(serde_json::from_value(json!([{"kind":"fill","color":"#00ff00","opacity":1}])).unwrap()),
            4 => text.style.layout = Some(serde_json::from_value(json!({"trackingPx":2,"bounds":{"widthPx":110,"heightPx":60},"verticalAlignment":"center"})).unwrap()),
            5 => text.font_binding.as_mut().unwrap().regular = text.font_binding.as_ref().unwrap().bold.clone(),
            6 => text.font_size += 3,
            7 => text.style.background_opacity = 0.5,
            8 => text.style.outline_width_px = 2,
            9 => { text.style.shadow.opacity = 0.8; text.style.shadow.offset_x = -3; }
            10 => changed.settings.width += 20,
            11 => changed.revision += 1,
            12 => changed.id.push('x'),
            18 | 19 => {
                let mut layers: Vec<crate::TextPaintLayer> = serde_json::from_value(json!([
                    {"kind":"fill","color":"#ff0000","opacity":1},
                    {"kind":"fill","color":"#0000ff","opacity":1}
                ])).unwrap();
                if field == 19 { layers.reverse(); }
                text.style.paint_layers = Some(layers.into());
            }
            _ => {
                let mut value = serde_json::to_value(&changed).unwrap();
                let shape = &mut value["tracks"][1]["items"][1];
                match field {
                    13 => shape["geometry"] = json!({"type":"ellipse","width":20,"height":15}),
                    14 => shape["fill"] = json!({"type":"solid","color":{"r":0,"g":1,"b":0,"a":1}}),
                    15 => shape["stroke"] = json!({"paint":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"width":2,"dash":[],"dashOffset":0,"lineCap":"round","lineJoin":"round","miterLimit":4}),
                    16 => { shape["transform2d"]["scaleX"] = json!(2); shape["transform2d"]["scaleY"] = json!(2); }
                    _ => {
                        let document = &mut value["tracks"][1]["items"][2]["document"];
                        if field == 20 {
                            document["shapes"].as_array_mut().unwrap().reverse();
                        } else {
                            document["width"] = json!(25);
                        }
                    }
                }
                changed = serde_json::from_value(value).unwrap();
            }
        }
        let misses = warm.raster_cache.misses.load(Ordering::Relaxed);
        let actual = rasters(&warm, &changed, &dir);
        assert!(
            warm.raster_cache.misses.load(Ordering::Relaxed) > misses,
            "variant {field} must miss"
        );
        let fresh = renderer(Arc::default());
        assert_eq!(actual, rasters(&fresh, &changed, &dir), "variant {field}");
        let misses = warm.raster_cache.misses.load(Ordering::Relaxed);
        assert_eq!(actual, rasters(&warm, &changed, &dir));
        assert_eq!(misses, warm.raster_cache.misses.load(Ordering::Relaxed));
        assert_eq!(original, rasters(&warm, &base, &dir));
        assert_eq!(
            misses,
            warm.raster_cache.misses.load(Ordering::Relaxed),
            "unrelated original key must remain reusable"
        );
    }
}

#[test]
fn scoped_requests_share_cache_without_changing_adapter_or_identity() {
    let (_root, core, project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let io = Arc::new(LifecycleArtifactIo::default());
    let original = renderer(io.clone());
    let first = original.clone().with_request_id("first").unwrap();
    let second = original.clone().with_request_id("second").unwrap();
    assert_eq!(first.artifact_io.request_id(), "first");
    assert_eq!(second.artifact_io.request_id(), "second");
    assert!(original.clone().with_request_id("../escape").is_err());
    assert!(original.clone().with_request_id("").is_err());
    let bytes = rasters(&first, &project, &dir);
    assert_eq!(bytes, rasters(&second, &project, &dir));
    assert_eq!(original.raster_cache.misses.load(Ordering::Relaxed), 1);
    io.fail_next(ArtifactFailure::Write);
    assert_eq!(
        second.render_preview(&project, &dir, 0).unwrap_err().code,
        ErrorCode::FfmpegFailed
    );
    assert!(!dir.join(".opencut-work-second").exists());
}

#[test]
fn warm_preflight_matches_cold_across_routes_before_any_lookup() {
    let (_root, core, mut project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let TimelineItem::Text(t) = &mut project.tracks[1].items[0] else {
        panic!()
    };
    t.style.layout = Some(Box::default());
    let io = Arc::new(LifecycleArtifactIo::default());
    let warm = renderer(io.clone());
    rasters(&warm, &project, &dir);
    let output = dir.join("existing.mp4");
    std::fs::write(&output, b"preserved").unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    for invalid in 0..10 {
        let mut bad = project.clone();
        let mut warmed = warm.clone();
        let cold_io = Arc::new(LifecycleArtifactIo::default());
        let mut cold = renderer(cold_io.clone());
        let font_path = dir.join(&project.fonts.values().next().unwrap().relative_path);
        let font_bytes = std::fs::read(&font_path).unwrap();
        let TimelineItem::Text(t) = &mut bad.tracks[1].items[0] else {
            panic!()
        };
        match invalid {
            0 => t.transform.scale = f64::NAN,
            1 => t.font_binding.as_mut().unwrap().profile = "unsupported".into(),
            2 => t.font_binding.as_mut().unwrap().regular = "0".repeat(64),
            3 => bad.fonts.values_mut().next().unwrap().relative_path = "../escape.ttf".into(),
            4 => {
                warmed.text_glyph_limit = Some(0);
                cold.text_glyph_limit = Some(0);
            }
            5 => {
                let failed = Arc::new(FakeProcess {
                    readiness_error: true,
                    probe_error: false,
                    run_failure: None,
                    executions: Mutex::new(vec![]),
                });
                warmed.process_executor = failed.clone();
                cold.process_executor = failed;
            }
            6 => std::fs::remove_file(&font_path).unwrap(),
            7 => std::fs::write(&font_path, b"tampered").unwrap(),
            9 => {
                let escape = dir.parent().unwrap().join("outside.ttf");
                io.map_canonical_path(font_path.clone(), escape.clone());
                cold_io.map_canonical_path(font_path.clone(), escape);
            }
            _ => {
                bad.tracks[1].items[0] = TimelineItem::Media(MediaItem {
                    id: "missing".into(),
                    asset_id: "absent".into(),
                    start_ms: 0,
                    duration_ms: 1000,
                    source_in_ms: 0,
                    visual_properties: Default::default(),
                    audio: Default::default(),
                    keyframes: vec![],
                });
            }
        }
        for mode in 0..3 {
            let call = |r: &Renderer| match mode {
                0 => r.render_preview(&bad, &dir, 0),
                1 => r.render_preview_range(
                    &bad,
                    &dir,
                    PreviewRangeOptions {
                        start_ms: 0,
                        end_ms: 1000,
                        width: 160,
                        height: 90,
                        fps: 10,
                        include_audio: true,
                    },
                    |_| {},
                ),
                _ => r.export_video(
                    &bad,
                    &dir,
                    ExportOptions {
                        output: &output,
                        width: 160,
                        height: 90,
                        overwrite: false,
                    },
                    |_| {},
                ),
            };
            let counts = (
                warmed.raster_cache.hits.load(Ordering::Relaxed),
                warmed.raster_cache.misses.load(Ordering::Relaxed),
            );
            io.clear_events();
            cold_io.clear_events();
            let actual = call(&warmed).unwrap_err();
            let expected = call(&cold).unwrap_err();
            assert_eq!(
                (actual.code, actual.retryable, actual.failed_stage),
                (expected.code, expected.retryable, expected.failed_stage),
                "case {invalid}, route {mode}"
            );
            assert_eq!(
                counts,
                (
                    warmed.raster_cache.hits.load(Ordering::Relaxed),
                    warmed.raster_cache.misses.load(Ordering::Relaxed)
                )
            );
            assert_eq!(cold.raster_cache.misses.load(Ordering::Relaxed), 0);
            for events in [&io, &cold_io] {
                assert!(
                    events.events.lock().unwrap().iter().all(|event| !matches!(
                        *event,
                        "exists" | "request_id" | "create_dir" | "write" | "rename" | "remove"
                    )),
                    "case {invalid}, route {mode}"
                );
            }
            assert_eq!(std::fs::read(&output).unwrap(), b"preserved");
            assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
        }
        std::fs::write(font_path, font_bytes).unwrap();
    }
    let hits = warm.raster_cache.hits.load(Ordering::Relaxed);
    let misses = warm.raster_cache.misses.load(Ordering::Relaxed);
    let unsupported = serde_json::from_value(json!({"operation":"add_svg","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"svg":"<svg width=\"10\" height=\"10\"><script/></svg>"})).unwrap();
    assert_eq!(
        core.edit(&project.id, project.revision, unsupported)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(hits, warm.raster_cache.hits.load(Ordering::Relaxed));
    assert_eq!(misses, warm.raster_cache.misses.load(Ordering::Relaxed));
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
}

#[test]
fn composed_parent_sampling_misses_but_translation_reuses() {
    let (_root, core, project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let ops:Vec<crate::BatchEditOperation> = serde_json::from_value(json!([
        {"operation":"add_group","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"resultAlias":"parent"},
        {"operation":"add_shape","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"resultAlias":"shape","geometry":{"type":"rectangle","width":20,"height":15},"fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null},
        {"operation":"item_set_parent","itemId":"@shape","parent":{"scope":"root","id":"@parent"}}
    ])).unwrap();
    core.edit_batch(&project.id, project.revision, ops).unwrap();
    let base = core.get_project(&project.id).unwrap();
    let warm = renderer(Arc::default());
    let original = rasters(&warm, &base, &dir);
    let initial = warm.raster_cache.misses.load(Ordering::Relaxed);
    for scale in [1.0, 2.0] {
        let mut changed = serde_json::to_value(&base).unwrap();
        changed["tracks"][1]["items"][1]["transform2d"]["position"]["x"] = json!(30);
        changed["tracks"][1]["items"][1]["transform2d"]["scaleX"] = json!(scale);
        changed["tracks"][1]["items"][1]["transform2d"]["scaleY"] = json!(scale);
        let changed: Project = serde_json::from_value(changed).unwrap();
        let actual = rasters(&warm, &changed, &dir);
        assert_eq!(actual, rasters(&renderer(Arc::default()), &changed, &dir));
        if scale == 1.0 {
            assert_eq!(actual, original);
            assert_eq!(initial, warm.raster_cache.misses.load(Ordering::Relaxed));
        } else {
            assert_ne!(actual, original);
            assert_eq!(
                initial + 1,
                warm.raster_cache.misses.load(Ordering::Relaxed)
            );
        }
        let misses = warm.raster_cache.misses.load(Ordering::Relaxed);
        assert_eq!(actual, rasters(&warm, &changed, &dir));
        assert_eq!(original, rasters(&warm, &base, &dir));
        assert_eq!(misses, warm.raster_cache.misses.load(Ordering::Relaxed));
    }
}
