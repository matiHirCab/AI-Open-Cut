use super::*;
use crate::render_artifact::{self, FileSystemArtifactIo};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn inclusive_budgets_lru_and_oversize_bypass() {
    let cache = RasterCache {
        max_entries: 2,
        max_bytes: 68,
        ..Default::default()
    };
    let calls = AtomicUsize::new(0);
    let render = || {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(vec![1, 2])
    };
    cache.raster([1; 32], render).unwrap();
    cache.raster([2; 32], render).unwrap();
    assert_eq!(cache.state.lock().unwrap().bytes, 68);
    cache.raster([1; 32], render).unwrap(); // refresh 1, evict 2
    cache.raster([3; 32], render).unwrap();
    cache.raster([1; 32], render).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 3);
    cache.raster([2; 32], render).unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), 4);
    for _ in 0..2 {
        assert_eq!(cache.raster([9; 32], || Ok(vec![9; 37])).unwrap().len(), 37);
    }
    assert_eq!(cache.state.lock().unwrap().bytes, 68);
    let byte_limited = RasterCache {
        max_entries: 10,
        max_bytes: 67,
        ..Default::default()
    };
    for i in 0..10 {
        byte_limited.raster([i; 32], render).unwrap();
    }
    assert_eq!(byte_limited.state.lock().unwrap().entries.len(), 1);
    let entry_limited = RasterCache {
        max_entries: 2,
        ..Default::default()
    };
    for i in 0..10 {
        entry_limited.raster([i; 32], render).unwrap();
    }
    assert_eq!(entry_limited.state.lock().unwrap().entries.len(), 2);
}

#[test]
fn failed_rasters_retry_and_poison_bypasses() {
    let cache = RasterCache::default();
    assert!(
        cache
            .raster([1; 32], || Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "injected"
            )))
            .is_err()
    );
    assert!(cache.state.lock().unwrap().entries.is_empty());
    assert_eq!(&*cache.raster([1; 32], || Ok(vec![7])).unwrap(), &[7]);
    assert!(
        std::panic::catch_unwind(|| {
            let _guard = cache.state.lock().unwrap();
            panic!("poison");
        })
        .is_err()
    );
    assert_eq!(&*cache.raster([1; 32], || Ok(vec![8])).unwrap(), &[8]);
}

#[test]
fn concurrent_misses_are_immutable_and_accounted_once() {
    let cache = RasterCache::default();
    let barrier = std::sync::Barrier::new(8);
    std::thread::scope(|threads| {
        for _ in 0..8 {
            threads.spawn(|| {
                let bytes = cache
                    .raster([1; 32], || {
                        barrier.wait();
                        Ok(vec![4; 64])
                    })
                    .unwrap();
                assert_eq!(&*bytes, &[4; 64]);
                for n in 2..150 {
                    cache.raster([n; 32], || Ok(vec![n; 64])).unwrap();
                }
                assert_eq!(&*bytes, &[4; 64]); // remains valid after eviction
            });
        }
    });
    let state = cache.state.lock().unwrap();
    assert_eq!(state.entries.len(), 128);
    assert_eq!(state.bytes, 128 * 96);
}

fn shape() -> EvaluatedShape {
    EvaluatedShape::new(
        crate::ShapeGeometry::Rectangle {
            width: 10.,
            height: 10.,
        },
        Some(
            serde_json::from_value(json!({"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}}))
                .unwrap(),
        ),
        None,
        1.,
    )
    .unwrap()
}

#[test]
fn vector_dependencies_invalidate_and_preserve_pixels() {
    let original = shape();
    let key = shape_key([0; 32], &original).unwrap();
    let cache = RasterCache::default();
    let bytes = cache
        .raster(key, || render_artifact::shapes::rasterize(&original))
        .unwrap();
    let start = bytes.len() - (original.size.0 * original.size.1 * 4) as usize;
    let center =
        start + ((original.size.1 / 2 * original.size.0 + original.size.0 / 2) * 4) as usize;
    assert_eq!(&bytes[center..center + 4], &[255, 0, 0, 255]);
    for field in 0..10 {
        let mut changed = original.clone();
        match field {
            0 => changed.geometry = crate::ShapeGeometry::Ellipse { width: 10., height: 10. },
            1 => changed.fill = None,
            2 => changed.stroke = Some(serde_json::from_value(json!({"paint":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4})).unwrap()),
            3 => changed.fill_rule = crate::FillRule::Evenodd,
            4 => changed.contours[0].points[0].x += 0.0000001,
            5 => changed.contours[0].closed = !changed.contours[0].closed,
            6 => changed.bounds[0] += 1.,
            7 => changed.origin.0 += 1.,
            8 => changed.size.0 += 1,
            _ => changed.density = 2.,
        }
        let next = shape_key([0; 32], &changed).unwrap();
        assert_ne!(key, next, "field {field}");
        let expected = render_artifact::shapes::rasterize(&changed).unwrap();
        assert_eq!(
            &*cache.raster(next, || Ok(expected.clone())).unwrap(),
            expected
        );
        assert_eq!(
            &*cache
                .raster(next, || panic!("warm rasterizer called"))
                .unwrap(),
            expected
        );
    }
    assert_eq!(
        &*cache
            .raster(key, || panic!("unrelated key evicted"))
            .unwrap(),
        &*bytes
    );
    let source = "<svg width=\"10\" height=\"10\"><rect width=\"10\" height=\"10\" fill=\"#f00\"/><rect width=\"5\" height=\"5\" fill=\"#00f\"/></svg>";
    let svg = EvaluatedShape::new_svg(crate::validation::svg::parse(source).unwrap(), 1.).unwrap();
    let key = shape_key([0; 32], &svg).unwrap();
    let cold = render_artifact::shapes::rasterize(&svg).unwrap();
    assert_eq!(&*cache.raster(key, || Ok(cold.clone())).unwrap(), cold);
    assert_eq!(&*cache.raster(key, || panic!()).unwrap(), cold);
    let mut reversed = svg.clone();
    reversed.svg_children.as_mut().unwrap().reverse();
    assert_ne!(key, shape_key([0; 32], &reversed).unwrap());
    assert_ne!(cold, render_artifact::shapes::rasterize(&reversed).unwrap());
    let mut viewport = svg;
    viewport.svg_document.as_mut().unwrap().width = 11.;
    assert_ne!(key, shape_key([0; 32], &viewport).unwrap());
}

pub(crate) fn fixture() -> (tempfile::TempDir, crate::EditorCore, crate::Project) {
    let root = tempfile::tempdir().unwrap();
    let fonts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/fonts");
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            root.path().join("projects"),
            [root.path(), fonts.as_path()],
            root.path().join("exports"),
        )
        .unwrap(),
    )
    .with_font_config(crate::FontConfig {
        roots: vec![fonts],
        default_path: None,
    });
    let id = core
        .create_project(
            "Raster cache",
            crate::ProjectSettings {
                width: 160,
                height: 90,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let p = core.get_project(&id).unwrap();
    core.edit(&id, 0, serde_json::from_value(json!({"operation":"add_text","trackId":p.tracks[1].id,"text":"Cache AV","fontFamily":"DejaVu Sans","fontSize":18,"color":"#ffffff","startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap()).unwrap();
    let project = core.get_project(&id).unwrap();
    (root, core, project)
}

#[test]
fn text_dependencies_and_scope_are_complete() {
    let (_root, core, project) = fixture();
    let dir = core.project_directory(&project.id).unwrap();
    let mut evaluated = crate::evaluated_scene::evaluate_project(&project, 160, 90, 10).unwrap();
    let media =
        render_artifact::prepare_media_resources(&FileSystemArtifactIo, &evaluated, &dir).unwrap();
    let measured = render_artifact::measure_evaluated_text_layers_with_budget(
        &FileSystemArtifactIo,
        &evaluated,
        None,
        &[],
        &mut vec![],
        &media.font_faces,
        &mut Default::default(),
    )
    .unwrap();
    let layer = &evaluated.scene.visual_layers[0];
    let crate::evaluated_scene::EvaluatedVisualSource::Text(text) = &layer.source else {
        panic!()
    };
    let m = &measured[&layer.item_id];
    let (shaped, style) = m.shaped.as_ref().unwrap();
    let original = text_key([0; 32], text, shaped, &m.prepared).unwrap();
    let cache = RasterCache::default();
    let cold =
        render_artifact::text::rasterize(shaped, &media.font_faces, &m.prepared, style).unwrap();
    assert!(cold[cold.len() / 2..].iter().any(|b| *b != 0));
    assert_eq!(&*cache.raster(original, || Ok(cold.clone())).unwrap(), cold);
    assert_eq!(&*cache.raster(original, || panic!()).unwrap(), cold);
    for field in 0..19 {
        let mut t = text.clone();
        let mut s = shaped.clone();
        match field {
            0 => t.text.push('!'),
            1 => t.rich_runs.as_mut().unwrap()[0].text.push('!'),
            2 => t.font_binding.as_mut().unwrap().regular.push('0'),
            3 => t.font_binding.as_mut().unwrap().profile.push('0'),
            4 => t.style.background_color = "#aabbcc".into(),
            5 => t.style.background_opacity = 0.5,
            6 => t.style.outline_width_px = 2,
            7 => t.style.outline_color = "#ff0000".into(),
            8 => t.style.shadow.offset_x = 3,
            9 => t.style.shadow.opacity = 0.7,
            10 => t.style.shadow.color = "#123456".into(),
            11 => t.style.padding.top += 1,
            12 => {
                t.style.layout = Some(Box::new(
                    serde_json::from_value(json!({"trackingPx":0.1})).unwrap(),
                ))
            }
            13 => s.glyphs[0].x += 0.0000001,
            14 => s.glyphs[0].id += 1,
            15 => s.glyphs[0].face.push('0'),
            16 => s.font_size += 1,
            17 => s.glyphs[0].color = "#123456".into(),
            _ => {
                s.layout = Some(crate::fonts::shaping::ShapedLayout {
                    background: [1., 2., 3., 4.],
                    content_width: 3.,
                    content_height: 4.,
                    overflow_x: false,
                    overflow_y: false,
                })
            }
        }
        assert_ne!(
            original,
            text_key([0; 32], &t, &s, &m.prepared).unwrap(),
            "field {field}"
        );
    }
    let base = scope(&evaluated).unwrap();
    evaluated.revision += 1;
    assert_ne!(base, scope(&evaluated).unwrap());
    evaluated.revision -= 1;
    evaluated.project_id.push('x');
    assert_ne!(base, scope(&evaluated).unwrap());
    evaluated.project_id.pop();
    evaluated.scene.canvas.width += 1;
    assert_ne!(base, scope(&evaluated).unwrap());
    let mut a = KeyWriter::new([0; 32], "text").unwrap();
    a.field(&"a").unwrap();
    a.field(&"bc").unwrap();
    let mut b = KeyWriter::new([0; 32], "text").unwrap();
    b.field(&"ab").unwrap();
    b.field(&"c").unwrap();
    assert_ne!(a.finish(), b.finish());
}
