//! Shape-specific native evidence, independent of the established legacy goldens.
use super::*;
use serde_json::json;

fn decode_rgb_frame(ffmpeg: &Path, path: &Path, time_ms: u64) -> Vec<u8> {
    let output = std::process::Command::new(ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(format!("{}", time_ms as f64 / 1000.0))
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
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.len(), 240 * 120 * 3);
    output.stdout
}

fn transform(x: f64, y: f64, opacity: f64) -> crate::Transform2D {
    let mut t = crate::Transform2D::default();
    t.position.x = x;
    t.position.y = y;
    t.opacity = opacity;
    t
}

fn project() -> Project {
    let mut p = fixture_project();
    p.settings.width = 240;
    p.settings.height = 120;
    p.tracks[0].items.clear();
    let catalog: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/shape-items-v1.json"
    )))
    .unwrap();
    for (i, f) in catalog["valid"].as_array().unwrap().iter().enumerate() {
        let mut v = f["value"].clone();
        let item = v.as_object_mut().unwrap();
        item.remove("operation");
        item.remove("trackId");
        item.insert("type".into(), json!("shape"));
        item.insert("id".into(), json!(format!("shape-{i}")));
        item.insert("keyframes".into(), json!([]));
        item.insert("stackOrder".into(), json!(i));
        let x = 10.0 + (i % 4) as f64 * 55.0;
        let y = if i < 4 { 15.0 } else { 75.0 };
        let mut t = crate::Transform2D::default();
        t.position.x = x;
        t.position.y = y;
        if i == 1 {
            t.rotation_deg = 15.0;
            t.skew_x_deg = 8.0;
        }
        item.insert("transform2d".into(), serde_json::to_value(t).unwrap());
        p.tracks[0].items.push(serde_json::from_value(v).unwrap());
    }
    if let TimelineItem::Shape(ellipse) = &mut p.tracks[0].items[2] {
        ellipse.fill = Some(serde_json::from_value(json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":40,"y":0},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":1,"a":0.4}},{"offset":1,"color":{"r":0,"g":1,"b":0,"a":1}}]})).unwrap());
    }
    if let TimelineItem::Shape(polygon) = &mut p.tracks[0].items[4] {
        polygon.transform2d.as_mut().unwrap().position.x = -10.0;
    }
    if let TimelineItem::Shape(path) = &mut p.tracks[0].items[6] {
        let crate::ShapeGeometry::Path { path } = &mut path.geometry else {
            unreachable!()
        };
        path.commands.extend(serde_json::from_value::<Vec<crate::PathCommand>>(json!([
            {"type":"moveTo","to":{"x":15,"y":0}}, {"type":"lineTo","to":{"x":25,"y":0}},
            {"type":"lineTo","to":{"x":25,"y":10}}, {"type":"lineTo","to":{"x":15,"y":10}}, {"type":"close"}
        ])).unwrap());
    }
    if let TimelineItem::Shape(line) = &mut p.tracks[0].items[3] {
        let stroke = line.stroke.as_mut().unwrap();
        stroke.dash = vec![6.0, 4.0];
        stroke.dash_offset = -2.0;
        stroke.line_cap = crate::LineCap::Round;
    }
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"group","id":"parent","startMs":0,"durationMs":1000,"stackOrder":7,"transform2d":transform(0.0,0.0,0.8)})).unwrap());
    p.tracks[0].items[5].visual_properties_mut().parent = Some(crate::ParentReference {
        scope: "root".into(),
        id: "parent".into(),
    });
    let mut local = serde_json::to_value(&p.tracks[0].items[0]).unwrap();
    local["id"] = json!("local-shape");
    local["stackOrder"] = json!(0);
    local["transform2d"] = serde_json::to_value(crate::Transform2D::default()).unwrap();
    local["fill"] = json!({"type":"radialGradient","center":{"x":20,"y":10},"radius":20,"stops":[{"offset":0,"color":{"r":1,"g":1,"b":1,"a":1}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":0.5}}]});
    p.components.push(serde_json::from_value(json!({"id":"badge","name":"Badge","width":40,"height":20,"durationMs":1000,"tracks":[{"id":"local-track","name":"Shapes","trackType":"overlay","items":[local]}],"slots":[]})).unwrap());
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"component_instance","id":"badge-1","componentId":"badge","startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"stackOrder":8,"transform2d":transform(175.0,75.0,1.0)})).unwrap());
    p.tracks[0].items.push(serde_json::from_value(json!({"type":"rectangle","id":"legacy-overlap","color":"#FFFFFF","width":6,"height":6,"startMs":0,"durationMs":1000,"keyframes":[],"stackOrder":9,"transform2d":transform(180.0,80.0,1.0)})).unwrap());
    p
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    recipe_sha256: String,
    rgb: Vec<u8>,
    plan: String,
}

pub(super) fn conformance(tools: &NativeTools) {
    component_animation_conformance(tools);
    transformed_conformance(tools);
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let p = project();
    let before = serde_json::to_vec(&p).unwrap();
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let evaluated = evaluate_project(&p, 240, 120, FPS).unwrap();
    let plan = format!("{:#?}", evaluated.scene);
    assert_eq!(
        plan,
        format!("{:#?}", evaluate_project(&p, 240, 120, FPS).unwrap().scene)
    );
    let frame = renderer.render_preview(&p, root.path(), 500).unwrap();
    let frame_path = root.path().join(&frame.relative_path);
    let rgb = decode_rgb_frame(&tools.ffmpeg, &frame_path, 0);
    assert_eq!(rgb.len(), 240 * 120 * 3);
    let pixel = |x: usize, y: usize| &rgb[(y * 240 + x) * 3..(y * 240 + x) * 3 + 3];
    assert!(
        pixel(25, 25)[0] > 240 && pixel(25, 25)[1] < 10,
        "rectangle interior {:?}",
        pixel(25, 25)
    );
    assert!(pixel(2, 2).iter().all(|c| *c < 10));
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shapes/reference.json");
    let recipe_sha256 = hash_bytes(include_bytes!("../../../tests/fixtures/shapes/recipe.json"));
    if env::var("OPENCUT_UPDATE_SHAPE_GOLDENS").as_deref() == Ok("1") {
        let reference = Reference {
            recipe_sha256: recipe_sha256.clone(),
            rgb: rgb.clone(),
            plan: plan.clone(),
        };
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec(&reference).unwrap()).unwrap();
        fs::rename(temporary, &path).unwrap();
        fs::copy(
            &frame_path,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.shape-preview.png"),
        )
        .unwrap();
    }
    let reference: Reference =
        serde_json::from_slice(&fs::read(path).expect("reviewed shape reference is required"))
            .unwrap();
    assert_eq!(reference.recipe_sha256, recipe_sha256);
    compare_shape_reference_plan(&reference.plan, &plan).unwrap();
    assert!(structural_similarity(&reference.rgb, &rgb).unwrap() >= 0.99);
    let draft_root = tempdir().unwrap();
    let core = crate::EditorCore::new(
        crate::PathPolicy::new(
            draft_root.path().join("projects"),
            [draft_root.path()],
            draft_root.path().join("exports"),
        )
        .unwrap(),
    );
    let project_id = core
        .create_project("Shape draft", p.settings.clone())
        .unwrap()
        .project_id;
    let project_dir = core.paths().project_dir(&project_id).unwrap();
    let mut persisted = p.clone();
    persisted.id = project_id.clone();
    fs::write(
        project_dir.join("project.json"),
        serde_json::to_vec(&persisted).unwrap(),
    )
    .unwrap();
    fs::copy(
        root.path().join("assets/tone.wav"),
        project_dir.join("assets/tone.wav"),
    )
    .unwrap();
    let geometry = serde_json::to_value(&p.tracks[0].items[0]).unwrap()["geometry"].clone();
    let draft = core
        .create_draft(
            &project_id,
            p.revision,
            vec![
                serde_json::from_value(
                    json!({"operation":"update_item","itemId":"shape-0","geometry":geometry}),
                )
                .unwrap(),
            ],
            None,
        )
        .unwrap();
    let state = core
        .get_draft_state(&project_id, &draft.id)
        .unwrap()
        .project;
    assert_eq!(
        plan,
        format!(
            "{:#?}",
            evaluate_project(&state, 240, 120, FPS).unwrap().scene
        )
    );
    let draft = renderer.render_preview(&state, &project_dir, 500).unwrap();
    assert!(
        structural_similarity(
            &rgb,
            &decode_rgb_frame(&tools.ffmpeg, &project_dir.join(draft.relative_path), 0)
        )
        .unwrap()
            >= 0.99
    );
    let range = renderer
        .render_preview_range(
            &p,
            root.path(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: 240,
                height: 120,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let range_path = root.path().join(range.relative_path);
    let export = root.path().join("shapes.mp4");
    renderer
        .export_video(
            &p,
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
    for file in [&range_path, &export] {
        assert!(
            structural_similarity(&rgb, &decode_rgb_frame(&tools.ffmpeg, file, 500)).unwrap()
                >= 0.99
        );
    }
    let a = decode_mono_f32(&tools.ffmpeg, &range_path);
    let b = decode_mono_f32(&tools.ffmpeg, &export);
    assert!(!a.is_empty());
    assert_eq!(a.len(), b.len());
    assert!(a.iter().any(|v| v.abs() > 0.001));
    let rms = (a
        .iter()
        .zip(&b)
        .map(|(a, b)| f64::from(a - b).powi(2))
        .sum::<f64>()
        / a.len() as f64)
        .sqrt();
    assert!(rms <= 0.0001);
    assert!(
        renderer
            .probe(&range_path)
            .unwrap()
            .duration_ms
            .unwrap()
            .abs_diff(renderer.probe(&export).unwrap().duration_ms.unwrap())
            <= 1000 / u64::from(FPS)
    );
    assert_eq!(serde_json::to_vec(&p).unwrap(), before);
}

// Portability is limited to generated contour points in this stored shape oracle.
// Production evaluation and same-runtime repeated plans still compare exactly.
fn compare_shape_reference_plan(reference: &str, actual: &str) -> Result<(), String> {
    let reference_coordinates = shape_plan_coordinates(reference)?;
    let actual_coordinates = shape_plan_coordinates(actual)?;
    let reference_lines: Vec<_> = reference.split('\n').collect();
    let actual_lines: Vec<_> = actual.split('\n').collect();
    if reference_lines.len() != actual_lines.len() {
        return Err("shape semantic plan line count differs".into());
    }
    let ordered = |value: f64| {
        let bits = value.to_bits();
        if value.is_sign_negative() {
            !bits
        } else {
            bits | (1 << 63)
        }
    };
    for (index, (expected, observed)) in reference_lines.iter().zip(&actual_lines).enumerate() {
        let matches = match (reference_coordinates[index], actual_coordinates[index]) {
            (Some(a), Some(b)) => {
                let same_zero_sign = a != 0.0 || b != 0.0 || a.to_bits() == b.to_bits();
                same_zero_sign && (a - b).abs() <= 1e-12 && ordered(a).abs_diff(ordered(b)) <= 8
            }
            (None, None) => expected == observed,
            _ => false,
        };
        if !matches {
            return Err(format!(
                "shape semantic plan line {}: expected {expected:?}, observed {observed:?}",
                index + 1
            ));
        }
    }
    Ok(())
}

fn shape_plan_coordinates(plan: &str) -> Result<Vec<Option<f64>>, String> {
    const POINT_SCOPE: [&str; 9] = [
        "EvaluatedScene {",
        "visual_layers: [",
        "EvaluatedVisualLayer {",
        "source: Shape(",
        "EvaluatedShape {",
        "contours: [",
        "Contour {",
        "points: [",
        "VectorPoint {",
    ];
    struct Frame<'a> {
        header: &'a str,
        indent: usize,
        coordinates: usize,
    }
    let mut stack: Vec<Frame<'_>> = Vec::new();
    let mut result = vec![];
    let mut contours = 0;
    for (index, line) in plan.split('\n').enumerate() {
        let invalid = || {
            format!(
                "unsupported shape semantic plan structure at line {}: {line:?}",
                index + 1
            )
        };
        let text = line.trim_start_matches(' ');
        let indent = line.len() - text.len();
        let scope = |stack: &[Frame<'_>], expected: &[&str]| {
            stack
                .iter()
                .map(|frame| frame.header)
                .eq(expected.iter().copied())
        };
        let point = scope(&stack, &POINT_SCOPE);
        let closing = matches!(text, "}" | "}," | "]" | "]," | ")" | "),");
        let mut coordinate = None;
        if closing {
            let frame = stack.pop().ok_or_else(invalid)?;
            let expected = match frame.header.as_bytes().last() {
                Some(b'{') => b'}',
                Some(b'[') => b']',
                Some(b'(') => b')',
                _ => return Err(invalid()),
            };
            if indent != frame.indent
                || text.as_bytes()[0] != expected
                || (point && frame.coordinates != 2)
            {
                return Err(invalid());
            }
        } else {
            if indent != stack.last().map_or(0, |frame| frame.indent + 4) {
                return Err(invalid());
            }
            if stack.is_empty() && (index != 0 || text != POINT_SCOPE[0]) {
                return Err(invalid());
            }
            if point {
                let frame = stack.last_mut().unwrap();
                let prefix = match frame.coordinates {
                    0 => "x: ",
                    1 => "y: ",
                    _ => return Err(invalid()),
                };
                let number = text
                    .strip_prefix(prefix)
                    .and_then(|s| s.strip_suffix(','))
                    .ok_or_else(invalid)?;
                let value: f64 = number.parse().map_err(|_| invalid())?;
                if !value.is_finite() {
                    return Err(invalid());
                }
                coordinate = Some(value);
                frame.coordinates += 1;
            } else if text.ends_with(['{', '[', '(']) {
                if text == "contours: [" {
                    if !scope(&stack, &POINT_SCOPE[..5]) {
                        return Err(invalid());
                    }
                    contours += 1;
                }
                // Derived contours accept only the reviewed points/closed layout.
                if scope(&stack, &POINT_SCOPE[..6]) && text != "Contour {"
                    || scope(&stack, &POINT_SCOPE[..7]) && text != "points: ["
                    || scope(&stack, &POINT_SCOPE[..8]) && text != "VectorPoint {"
                {
                    return Err(invalid());
                }
                stack.push(Frame {
                    header: text,
                    indent,
                    coordinates: 0,
                });
            } else if scope(&stack, &POINT_SCOPE[..6])
                || scope(&stack, &POINT_SCOPE[..8])
                || scope(&stack, &POINT_SCOPE[..7])
                    && !matches!(text, "closed: true," | "closed: false,")
            {
                return Err(invalid());
            }
        }
        result.push(coordinate);
    }
    if !stack.is_empty() || contours == 0 {
        return Err("incomplete shape semantic plan".into());
    }
    Ok(result)
}

#[test]
fn shape_reference_plan_accepts_measured_platform_contours() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shapes/reference.json"
    ))
    .unwrap();
    let mut linux = reference.plan.clone();
    for (windows, replacement) in [
        ("2.3431457505076194", "2.3431457505076203"),
        ("0.5358983848622461", "0.5358983848622465"),
        ("3.819660112501051", "3.8196601125010474"),
    ] {
        assert!(linux.contains(windows));
        linux = linux.replace(windows, replacement);
    }
    assert_ne!(reference.plan, linux);
    compare_shape_reference_plan(&reference.plan, &linux).unwrap();
}

fn component_animation_project(animated: bool) -> Project {
    let mut p = project();
    p.components.clear();
    let keys = if animated {
        json!([
            {"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"},
            {"property":"scale","timeMs":1000,"value":{"type":"scalar","value":1},"easing":"linear"}
        ])
    } else {
        json!([])
    };
    let shape = json!({"type":"shape","id":"animated-shape","startMs":0,"durationMs":1000,
        "stackOrder":0,"geometry":{"type":"rectangle","width":10,"height":10},
        "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"stroke":null,
        "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"keyframes":keys});
    p.components.push(serde_json::from_value(json!({"id":"animated-component","name":"Animated","width":40,"height":40,"durationMs":1000,
        "tracks":[{"id":"shapes","name":"Shapes","trackType":"overlay","items":[shape]}],"slots":[]})).unwrap());
    p.tracks[0].items = vec![serde_json::from_value(json!({"type":"component_instance","id":"animated-instance","componentId":"animated-component",
        "startMs":0,"durationMs":1000,"trimStartMs":0,"timeScale":1,"stackOrder":0,"transform2d":transform(100.0,20.0,1.0)})).unwrap()];
    p
}

#[test]
fn component_animation_clips_in_output_space() {
    let p = component_animation_project(true);
    let mut e = evaluate_project(&p, 240, 120, FPS).unwrap();
    crate::evaluated_scene::finalize_affine_geometry(
        &mut e.scene,
        &std::collections::HashMap::new(),
    )
    .unwrap();
    assert_eq!(
        e.scene.visual_layers.len(),
        1,
        "translated shape must remain visible"
    );
    let a = e.scene.visual_layers[0].affine.as_ref().unwrap();
    assert!(a.left <= 100.0 && a.left + f64::from(a.width) >= 110.0);
    assert!(a.top <= 20.0 && a.top + f64::from(a.height) >= 30.0);
}

#[test]
fn native_component_animation_conformance() {
    if let Some(tools) = configured_native_tools() {
        component_animation_conformance(&tools);
    }
}

fn component_animation_conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut frames = vec![];
    for animated in [false, true] {
        let p = component_animation_project(animated);
        let frame = renderer.render_preview(&p, root.path(), 500).unwrap();
        let rgb = decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0);
        let center = (25 * 240 + 105) * 3;
        assert!(
            rgb[center] > 240 && rgb[center + 1] < 10,
            "translated red interior; animated={animated}"
        );
        assert!(rgb[..3].iter().all(|v| *v < 10));
        frames.push(rgb);
    }
    assert_eq!(frames[0], frames[1], "constant scale keys preserve pixels");
    for case in component_animation_cases() {
        check_component_animation_rendering(tools, &case);
    }
}

#[test]
fn native_shape_render_conformance() {
    if let Some(tools) = configured_native_tools() {
        conformance(&tools);
    }
}

#[test]
fn shape_anchor_is_independent_of_stroke_padding_and_offscreen_origin() {
    let mut p = project();
    p.components.clear();
    p.tracks[0].items.truncate(1);
    let TimelineItem::Shape(shape) = &mut p.tracks[0].items[0] else {
        unreachable!()
    };
    shape.geometry = serde_json::from_value(
        json!({"type":"polygon","points":[{"x":10,"y":20},{"x":50,"y":30},{"x":20,"y":60}]}),
    )
    .unwrap();
    let mut t = transform(-5.0, 10.0, 1.0);
    t.anchor = crate::TransformAnchor { x: 0.25, y: 0.75 };
    shape.transform2d = Some(t);
    for width in [2.0, 10.0] {
        let TimelineItem::Shape(shape) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        shape.stroke.as_mut().unwrap().width = width;
        let evaluated = evaluate_project(&p, 240, 120, FPS).unwrap();
        let layer = &evaluated.scene.visual_layers[0];
        let crate::evaluated_scene::EvaluatedVisualSource::Shape(shape) = &layer.source else {
            unreachable!()
        };
        let a =
            crate::evaluated_scene::evaluate_layer_affine(layer, shape.size, (240, 120)).unwrap();
        let x = 20.0 - shape.origin.0;
        let y = 50.0 - shape.origin.1;
        assert!((a.matrix[0] * x + a.matrix[2] * y + a.matrix[4] + 5.0).abs() < 1e-9);
        assert!((a.matrix[1] * x + a.matrix[3] * y + a.matrix[5] - 10.0).abs() < 1e-9);
        assert!(a.left < 0.0, "offscreen bounds must not relocate geometry");
    }
}

#[test]
fn shape_occurrences_preserve_clocks_opacity_identity_and_aggregate_bounds() {
    let mut p = project();
    let mut repeated = p.tracks[0].items[8].clone();
    let TimelineItem::ComponentInstance(instance) = &mut repeated else {
        unreachable!()
    };
    instance.id = "badge-2".into();
    instance.visual_properties.stack_order = 10;
    instance.start_ms = 100;
    instance.duration_ms = 400;
    instance.time_scale = 2.0;
    p.tracks[0].items.push(repeated);
    let evaluated = evaluate_project(&p, 240, 120, FPS).unwrap();
    let shapes = evaluated
        .scene
        .visual_layers
        .iter()
        .filter(|l| {
            matches!(
                l.source,
                crate::evaluated_scene::EvaluatedVisualSource::Shape(_)
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(shapes.len(), 9);
    let repeated = shapes
        .iter()
        .find(|l| l.instance.unwrap().rate == 2.0)
        .unwrap();
    let clock = repeated.instance.unwrap();
    assert_eq!((clock.start_ms, clock.end_ms), (100.0, 500.0));
    assert_eq!(clock.root_ms(400), 300.0);
    assert_eq!(
        shapes
            .iter()
            .map(|l| &l.item_id)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        9
    );
    assert!(shapes.iter().any(|l| l.ancestors.unwrap().opacity == 0.8));
    p.components.clear();
    let mut base = p.tracks[0].items[0].clone();
    let TimelineItem::Shape(shape) = &mut base else {
        unreachable!()
    };
    shape.geometry = crate::ShapeGeometry::Polygon {
        points: vec![crate::VectorPoint { x: 0.0, y: 0.0 }; 4096],
    };
    shape.stroke = None;
    p.tracks[0].items = (0..256)
        .map(|i| {
            let mut s = base.clone();
            let TimelineItem::Shape(v) = &mut s else {
                unreachable!()
            };
            v.id = format!("bounded-{i}");
            v.stack_order = i;
            s
        })
        .collect();
    assert!(evaluate_project(&p, 240, 120, FPS).is_ok());
    let TimelineItem::Shape(s) = &mut base else {
        unreachable!()
    };
    s.id = "overflow".into();
    s.stack_order = 256;
    p.tracks[0].items.push(base);
    assert_eq!(
        evaluate_project(&p, 240, 120, FPS).unwrap_err().code,
        crate::ErrorCode::InvalidArgument
    );
}

#[test]
fn fractional_shape_anchor_equivalence() {
    let mut p = project();
    p.components.clear();
    p.tracks[0].items.truncate(1);
    let mut matrices = vec![];
    for (anchor, position) in [(1.0, 100.0), (0.0, 50.0)] {
        let TimelineItem::Shape(shape) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        shape.geometry = crate::ShapeGeometry::Rectangle {
            width: 0.5,
            height: 0.5,
        };
        shape.stroke = None;
        let mut t = transform(position, position, 1.0);
        t.anchor = crate::TransformAnchor {
            x: anchor,
            y: anchor,
        };
        t.scale_x = 100.0;
        t.scale_y = 100.0;
        shape.transform2d = Some(t);
        let e = evaluate_project(&p, 240, 120, FPS).unwrap();
        let l = &e.scene.visual_layers[0];
        matrices.push(
            crate::evaluated_scene::evaluate_layer_affine(l, l.source_size.unwrap(), (240, 120))
                .unwrap()
                .matrix,
        );
    }
    assert_eq!(matrices[0], matrices[1]);
}

#[test]
fn shape_density_is_composed_and_refinement_is_idempotent() {
    let mut p = project();
    let TimelineItem::Shape(shape) = &mut p.tracks[0].items[0] else {
        unreachable!()
    };
    let t = shape.transform2d.as_mut().unwrap();
    t.scale_x = 3.0;
    t.scale_y = 2.0;
    t.skew_x_deg = 20.0;
    t.rotation_deg = 30.0;
    let mut e = evaluate_project(&p, 240, 120, FPS).unwrap();
    let before = e
        .scene
        .visual_layers
        .iter()
        .filter_map(|l| {
            if let crate::evaluated_scene::EvaluatedVisualSource::Shape(s) = &l.source {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(before[0].density > 3.0);
    crate::evaluated_scene::finalize_affine_geometry(
        &mut e.scene,
        &std::collections::HashMap::new(),
    )
    .unwrap();
    let after = e
        .scene
        .visual_layers
        .iter()
        .filter_map(|l| {
            if let crate::evaluated_scene::EvaluatedVisualSource::Shape(s) = &l.source {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for (a, b) in before.iter().zip(&after) {
        assert!((a.density - b.density).abs() < 1e-12);
        assert_eq!(a.size, b.size);
    }
    p.components.clear();
    p.tracks[0].items.truncate(1);
    let TimelineItem::Shape(shape) = &mut p.tracks[0].items[0] else {
        unreachable!()
    };
    shape.transform2d = None;
    shape.keyframes=serde_json::from_value(json!([{"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"},{"property":"scale","timeMs":1000,"value":{"type":"scalar","value":4},"easing":"linear"}])).unwrap();
    let e = evaluate_project(&p, 240, 120, FPS).unwrap();
    let crate::evaluated_scene::EvaluatedVisualSource::Shape(s) = &e.scene.visual_layers[0].source
    else {
        unreachable!()
    };
    assert_eq!(s.density, 4.0);
}

fn transformed_conformance(tools: &NativeTools) {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let mut p = project();
    p.components.clear();
    p.tracks[0].items.truncate(1);
    // Independent equivalence oracle: tiny ellipse magnified into a 100px circle.
    let mut frames = vec![];
    for (width, scale) in [(2.0, 50.0), (100.0, 1.0)] {
        let TimelineItem::Shape(s) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        s.geometry = crate::ShapeGeometry::Ellipse {
            width,
            height: width,
        };
        s.stroke = None;
        let mut t = transform(20.0, 10.0, 1.0);
        t.scale_x = scale;
        t.scale_y = scale;
        s.transform2d = Some(t);
        let result = renderer.render_preview(&p, root.path(), 0).unwrap();
        let rgb = decode_rgb_frame(&tools.ffmpeg, &root.path().join(result.relative_path), 0);
        let pixel = |x: usize, y: usize| rgb[(y * 240 + x) * 3];
        assert!(pixel(70, 60) > 240, "opaque center");
        assert!(pixel(25, 15) < 10, "exterior corner");
        frames.push(rgb);
    }
    assert_eq!(frames[0], frames[1]);
    // Parent/component transforms and animated gradients/dashes use the same
    // source raster for frame, interval and export, sampled at multiple times.
    let mut p = project();
    let TimelineItem::Shape(s) = &mut p.tracks[0].items[2] else {
        unreachable!()
    };
    s.transform2d = None;
    s.transform.position_x = 100.0;
    s.transform.position_y = 10.0;
    s.keyframes=serde_json::from_value(json!([{"property":"scale","timeMs":0,"value":{"type":"scalar","value":0.5},"easing":"linear"},{"property":"scale","timeMs":1000,"value":{"type":"scalar","value":1.5},"easing":"linear"}])).unwrap();
    let TimelineItem::Shape(s) = &mut p.tracks[0].items[3] else {
        unreachable!()
    };
    let t = s.transform2d.as_mut().unwrap();
    t.scale_x = 1.5;
    t.scale_y = 2.0;
    t.skew_x_deg = 10.0;
    t.rotation_deg = 15.0;
    let TimelineItem::Group(g) = &mut p.tracks[0].items[7] else {
        unreachable!()
    };
    g.visual_properties.transform2d.as_mut().unwrap().scale_x = 1.25;
    let TimelineItem::ComponentInstance(c) = &mut p.tracks[0].items[8] else {
        unreachable!()
    };
    c.visual_properties.transform2d.as_mut().unwrap().scale_x = 1.3;
    let range = renderer
        .render_preview_range(
            &p,
            root.path(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: DURATION_MS,
                width: 240,
                height: 120,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let export = root.path().join("transformed.mp4");
    renderer
        .export_video(
            &p,
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
    for time in [0, 500, 900] {
        let frame = renderer.render_preview(&p, root.path(), time).unwrap();
        let rgb = decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0);
        for path in [&export, &root.path().join(&range.relative_path)] {
            let actual = decode_rgb_frame(&tools.ffmpeg, path, time);
            assert!(
                structural_similarity(&rgb, &actual).unwrap() >= 0.99,
                "sample {time}"
            );
        }
    }
}

#[test]
fn zero_extent_anchor_fallback_applies_only_to_paths() {
    for (geometry, anchor_point) in [
        (
            json!({"type":"line","start":{"x":0,"y":0},"end":{"x":2,"y":0}}),
            (2.0, 0.0),
        ),
        (
            json!({"type":"line","start":{"x":0,"y":0},"end":{"x":0,"y":2}}),
            (0.0, 2.0),
        ),
        (
            json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}}]}}),
            (1.0, 1.0),
        ),
        (
            json!({"type":"ellipse","width":0.5,"height":0.25}),
            (0.5, 0.25),
        ),
    ] {
        let mut p = project();
        p.components.clear();
        p.tracks[0].items.truncate(1);
        let TimelineItem::Shape(s) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        s.geometry = serde_json::from_value(geometry).unwrap();
        if matches!(s.geometry, crate::ShapeGeometry::Line { .. }) {
            s.fill = None;
        }
        let mut t = transform(20.0, 30.0, 1.0);
        t.anchor = crate::TransformAnchor { x: 1.0, y: 1.0 };
        s.transform2d = Some(t);
        let e = evaluate_project(&p, 240, 120, FPS).unwrap();
        let l = &e.scene.visual_layers[0];
        let crate::evaluated_scene::EvaluatedVisualSource::Shape(s) = &l.source else {
            unreachable!()
        };
        let a = crate::evaluated_scene::evaluate_layer_affine(l, s.size, (240, 120)).unwrap();
        let (x, y) = (
            (anchor_point.0 - s.origin.0) * s.density,
            (anchor_point.1 - s.origin.1) * s.density,
        );
        assert!((a.matrix[0] * x + a.matrix[2] * y + a.matrix[4] - 20.0).abs() < 1e-9);
        assert!((a.matrix[1] * x + a.matrix[3] * y + a.matrix[5] - 30.0).abs() < 1e-9);
    }
}

#[test]
fn magnified_thin_shape_checks_output_padding() {
    let mut p = project();
    p.components.clear();
    let mut parent = p.tracks[0].items[7].clone();
    parent.visual_properties_mut().stack_order = 1;
    parent
        .visual_properties_mut()
        .transform2d
        .as_mut()
        .unwrap()
        .scale_x = 10.0;
    parent
        .visual_properties_mut()
        .transform2d
        .as_mut()
        .unwrap()
        .scale_y = 10.0;
    p.tracks[0].items.truncate(1);
    let TimelineItem::Shape(s) = &mut p.tracks[0].items[0] else {
        unreachable!()
    };
    s.geometry = crate::ShapeGeometry::Rectangle {
        width: 10.0,
        height: 0.001,
    };
    s.stroke = None;
    let mut t = transform(0.0, 0.0, 1.0);
    t.scale_x = 100.0;
    t.scale_y = 100.0;
    s.transform2d = Some(t);
    s.parent = Some(crate::ParentReference {
        scope: "root".into(),
        id: "parent".into(),
    });
    p.tracks[0].items.push(parent);
    let e = evaluate_project(&p, 240, 120, FPS).unwrap();
    let crate::evaluated_scene::EvaluatedVisualSource::Shape(s) = &e.scene.visual_layers[0].source
    else {
        unreachable!()
    };
    assert_eq!(s.size, (10002, 3));
}

struct ComponentAnimationCase {
    name: &'static str,
    project: Project,
    // Independent authored-to-root oracle: translation, scale, local clock.
    translation: (f64, f64),
    scale: f64,
    rate: u64,
    offset: u64,
    moving: bool,
    normalized_companion: bool,
}

impl ComponentAnimationCase {
    fn times(&self) -> [u64; 3] {
        if self.rate == 2 {
            [0, 100, 300]
        } else {
            [0, 400, 900]
        }
    }

    fn bounds(&self, time: u64) -> [f64; 4] {
        let progress = (time * self.rate + self.offset) as f64 / 1000.0;
        let (position, size) = if self.moving {
            (10.0 * progress, 10.0 * (1.0 + progress))
        } else {
            (0.0, 10.0)
        };
        let left = self.translation.0 + self.scale * position;
        let top = self.translation.1;
        [left, top, left + self.scale * size, top + self.scale * size]
    }
}

fn component_animation_cases() -> Vec<ComponentAnimationCase> {
    let mut cases = vec![];
    for (name, x, scale, nested, retimed, canvas, moving) in [
        ("constant", 100.0, 1.0, false, false, 40, false),
        ("constant-position", 100.0, 1.0, false, false, 40, false),
        ("translated", 100.0, 1.0, false, false, 40, true),
        ("scaled", 100.0, 2.0, false, false, 40, true),
        ("nested-retimed", 100.0, 2.0, true, true, 40, true),
        ("retimed", 100.0, 2.0, false, true, 40, true),
        ("left-edge", -5.0, 1.0, false, false, 400, false),
        ("right-edge", 235.0, 1.0, false, false, 40, false),
        ("offscreen-left", -30.0, 1.0, false, false, 400, true),
        ("offscreen-right", 250.0, 1.0, false, false, 40, true),
    ] {
        let mut p = component_animation_project(true);
        p.components[0].width = canvas;
        p.components[0].height = canvas;
        if name == "constant-position" {
            let TimelineItem::Shape(s) = &mut p.components[0].tracks[0].items[0] else {
                unreachable!()
            };
            s.keyframes = serde_json::from_value(json!([
                {"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},
                {"property":"position","timeMs":1000,"value":{"type":"position","x":0,"y":0},"easing":"linear"}
            ])).unwrap();
        }
        if moving {
            let TimelineItem::Shape(s) = &mut p.components[0].tracks[0].items[0] else {
                unreachable!()
            };
            s.keyframes = serde_json::from_value(json!([
                {"property":"position","timeMs":0,"value":{"type":"position","x":0,"y":0},"easing":"linear"},
                {"property":"position","timeMs":1000,"value":{"type":"position","x":10,"y":0},"easing":"linear"},
                {"property":"scale","timeMs":0,"value":{"type":"scalar","value":1},"easing":"linear"},
                {"property":"scale","timeMs":1000,"value":{"type":"scalar","value":2},"easing":"linear"}
            ])).unwrap();
        }
        let mut translation = (x, 20.0);
        let mut composed_scale = scale;
        if nested {
            let mut inner = p.tracks[0].items[0].clone();
            let TimelineItem::ComponentInstance(i) = &mut inner else {
                unreachable!()
            };
            i.id = "inner-instance".into();
            i.trim_start_ms = 50;
            i.duration_ms = 950;
            let mut t = transform(5.0, 3.0, 1.0);
            t.scale_x = 1.5;
            t.scale_y = 1.5;
            i.visual_properties.transform2d = Some(t);
            p.components.push(serde_json::from_value(json!({"id":"outer-component","name":"Outer","width":80,"height":60,"durationMs":1000,
                "tracks":[{"id":"nested-track","name":"Nested","trackType":"overlay","items":[inner]}],"slots":[]})).unwrap());
            translation = (x + scale * 5.0, 20.0 + scale * 3.0);
            composed_scale *= 1.5;
        }
        let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        if nested {
            i.component_id = "outer-component".into();
        }
        let mut t = transform(x, 20.0, 1.0);
        t.scale_x = scale;
        t.scale_y = scale;
        i.visual_properties.transform2d = Some(t);
        if retimed {
            i.trim_start_ms = 100;
            i.time_scale = 2.0;
            i.duration_ms = 400;
        }
        cases.push(ComponentAnimationCase {
            name,
            project: p,
            translation,
            scale: composed_scale,
            rate: if retimed { 2 } else { 1 },
            offset: if retimed { 100 } else { 0 } + if nested { 50 } else { 0 },
            moving,
            normalized_companion: false,
        });
    }
    let mut p = component_animation_project(true);
    // Requested 240x120 differs from both persisted settings and the 40x40 component.
    p.settings.width = 160;
    p.settings.height = 80;
    let mut companion = p.components[0].tracks[0].items[0].clone();
    let TimelineItem::Shape(s) = &mut companion else {
        unreachable!()
    };
    s.id = "normalized-companion".into();
    s.stack_order = 1;
    s.keyframes.clear();
    let mut t = transform(0.5, 0.5, 1.0);
    t.position.unit = crate::PositionUnit::Normalized;
    s.transform2d = Some(t);
    p.components[0].tracks[0].items.push(companion);
    let TimelineItem::Shape(s) = &mut p.components[0].tracks[0].items[0] else {
        unreachable!()
    };
    s.transform.position_x = 80.0;
    cases.push(ComponentAnimationCase {
        name: "normalized-output",
        project: p,
        translation: (180.0, 20.0),
        scale: 1.0,
        rate: 1,
        offset: 0,
        moving: false,
        normalized_companion: true,
    });
    cases
}

#[test]
fn component_animation_envelopes_and_local_units() {
    for case in component_animation_cases() {
        let before = serde_json::to_vec(&case.project).unwrap();
        for (width, height) in [(240, 120), (160, 80)] {
            let mut e = evaluate_project(&case.project, width, height, FPS).unwrap();
            crate::evaluated_scene::finalize_affine_geometry(
                &mut e.scene,
                &std::collections::HashMap::new(),
            )
            .unwrap();
            for time in case.times() {
                let [left, top, right, bottom] = case.bounds(time);
                if right <= 0.0 || left >= f64::from(width) {
                    continue;
                }
                let layer = &e.scene.visual_layers[0];
                let clock = layer.instance.unwrap();
                assert_eq!(
                    clock.root_ms(time * case.rate + case.offset),
                    time as f64,
                    "{} clock",
                    case.name
                );
                let a = layer.affine.unwrap();
                assert!(
                    a.left <= left.max(0.0) && a.top <= top.max(0.0),
                    "{} origin {a:?}",
                    case.name
                );
                assert!(
                    a.left + f64::from(a.width) >= right.min(f64::from(width))
                        && a.top + f64::from(a.height) >= bottom.min(f64::from(height)),
                    "{} extent {a:?}",
                    case.name
                );
                assert!(a.left + f64::from(a.width) <= f64::from(width));
            }
            if case.normalized_companion {
                let layer = e.scene.visual_layers.last().unwrap();
                let a = layer.affine.unwrap();
                // One pixel raster padding; authored normalized position is (20,20).
                assert_eq!((a.matrix[4], a.matrix[5]), (119.0, 39.0));
            }
            let first = format!("{:?}", e.scene);
            crate::evaluated_scene::finalize_affine_geometry(
                &mut e.scene,
                &std::collections::HashMap::new(),
            )
            .unwrap();
            assert_eq!(first, format!("{:?}", e.scene));
        }
        assert_eq!(before, serde_json::to_vec(&case.project).unwrap());
    }
}

#[test]
fn component_animation_offscreen_geometry_still_validates() {
    for (width, scale) in [
        (100_000.0, 1.0),
        (1000.0, 100.0),
        (f64::INFINITY, 1.0),
        (f64::NAN, 1.0),
    ] {
        let mut p = component_animation_project(true);
        let TimelineItem::ComponentInstance(i) = &mut p.tracks[0].items[0] else {
            unreachable!()
        };
        let t = i.visual_properties.transform2d.as_mut().unwrap();
        t.position.x = -1_000_000.0;
        t.scale_x = scale;
        t.scale_y = scale;
        assert!(evaluate_project(&p, 240, 120, FPS).is_ok());
        let TimelineItem::Shape(s) = &mut p.components[0].tracks[0].items[0] else {
            unreachable!()
        };
        s.geometry = crate::ShapeGeometry::Rectangle {
            width,
            height: 10.0,
        };
        assert_eq!(
            evaluate_project(&p, 240, 120, FPS).unwrap_err().code,
            crate::ErrorCode::InvalidArgument
        );
        // A missing backend would yield a different error if evaluation reached it.
        let root = tempdir().unwrap();
        let renderer = Renderer::new("missing-ffmpeg", "missing-ffprobe", None);
        assert_eq!(
            renderer
                .render_preview(&p, root.path(), 500)
                .unwrap_err()
                .code,
            crate::ErrorCode::InvalidArgument
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}

fn check_component_animation_rendering(tools: &NativeTools, case: &ComponentAnimationCase) {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("previews")).unwrap();
    fs::create_dir(root.path().join("assets")).unwrap();
    write_tone_wav(&root.path().join("assets/tone.wav"));
    let renderer = Renderer::new(&tools.ffmpeg, &tools.ffprobe, Some(tools.font.clone()));
    let before = serde_json::to_vec(&case.project).unwrap();
    fs::write(root.path().join("project.json"), &before).unwrap();
    fs::write(
        root.path().join("history.json"),
        b"retained-history-sentinel",
    )
    .unwrap();
    let range = renderer
        .render_preview_range(
            &case.project,
            root.path(),
            PreviewRangeOptions {
                start_ms: 0,
                end_ms: 1000,
                width: 240,
                height: 120,
                fps: FPS,
                include_audio: true,
            },
            |_| {},
        )
        .unwrap();
    let export = root.path().join("component-animation.mp4");
    renderer
        .export_video(
            &case.project,
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
    let mut preview_project = case.project.clone();
    preview_project.settings.width = 240;
    preview_project.settings.height = 120;
    let static_paths = if case.name.starts_with("constant") {
        let mut p = case.project.clone();
        let TimelineItem::Shape(s) = &mut p.components[0].tracks[0].items[0] else {
            unreachable!()
        };
        s.keyframes.clear();
        let range = renderer
            .render_preview_range(
                &p,
                root.path(),
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: 240,
                    height: 120,
                    fps: FPS,
                    include_audio: true,
                },
                |_| {},
            )
            .unwrap();
        let output = root.path().join("static.mp4");
        renderer
            .export_video(
                &p,
                root.path(),
                ExportOptions {
                    output: &output,
                    width: 240,
                    height: 120,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        Some((p, root.path().join(range.relative_path), output))
    } else {
        None
    };
    for time in case.times() {
        let frame = renderer
            .render_preview(&preview_project, root.path(), time)
            .unwrap();
        let expected = decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0);
        let frames = [
            expected.clone(),
            decode_rgb_frame(&tools.ffmpeg, &root.path().join(&range.relative_path), time),
            decode_rgb_frame(&tools.ffmpeg, &export, time),
        ];
        if let Some((p, range, export)) = &static_paths {
            let frame = renderer.render_preview(p, root.path(), time).unwrap();
            for (animated, static_frame) in frames.iter().zip([
                decode_rgb_frame(&tools.ffmpeg, &root.path().join(frame.relative_path), 0),
                decode_rgb_frame(&tools.ffmpeg, range, time),
                decode_rgb_frame(&tools.ffmpeg, export, time),
            ]) {
                assert_eq!(*animated, static_frame, "constant keys at {time}");
            }
        }
        for rgb in frames {
            assert!(
                structural_similarity(&expected, &rgb).unwrap() >= 0.99,
                "{} at {time}",
                case.name
            );
            let pixel = |x: usize, y: usize| &rgb[(y * 240 + x) * 3..(y * 240 + x) * 3 + 3];
            let [left, top, right, bottom] = case.bounds(time);
            let (left, right) = (left.max(0.0), right.min(240.0));
            if left < right {
                let center = pixel(
                    ((left + right) / 2.0) as usize,
                    ((top + bottom) / 2.0) as usize,
                );
                assert!(
                    // H.264 chroma quantization affects even small solid interiors.
                    center[0] > 220 && center[1] < 35 && center[2] < 35,
                    "{} at {time}: {center:?}",
                    case.name
                );
                assert!(pixel(10, 100).iter().all(|v| *v < 10));
                if right + 4.0 < 240.0 {
                    assert!(
                        pixel((right + 4.0) as usize, (top + 2.0) as usize)
                            .iter()
                            .all(|v| *v < 10),
                        "{} exterior at {time}",
                        case.name
                    );
                }
            } else {
                assert!(rgb.iter().all(|v| *v < 10), "{} fully offscreen", case.name);
            }
            if case.normalized_companion {
                assert!(pixel(125, 45)[0] > 220, "normalized component position");
            }
        }
    }
    assert_eq!(before, serde_json::to_vec(&case.project).unwrap());
    assert_eq!(before, fs::read(root.path().join("project.json")).unwrap());
    assert_eq!(
        fs::read(root.path().join("history.json")).unwrap(),
        b"retained-history-sentinel"
    );
}

fn shape_plan_with_first_contour_x(plan: &str, value: f64) -> String {
    let contour = plan.find("contours: [").unwrap();
    let start = contour + plan[contour..].find("x: ").unwrap() + 3;
    let end = start + plan[start..].find(',').unwrap();
    format!("{}{:?}{}", &plan[..start], value, &plan[end..])
}

#[test]
fn shape_reference_plan_enforces_both_numeric_bounds() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shapes/reference.json"
    ))
    .unwrap();
    for base in [2.0_f64, -2.0] {
        let expected = shape_plan_with_first_contour_x(&reference.plan, base);
        for distance in [0, 1, 8, 9] {
            let actual = shape_plan_with_first_contour_x(
                &reference.plan,
                f64::from_bits(base.to_bits() + distance),
            );
            assert_eq!(
                compare_shape_reference_plan(&expected, &actual).is_ok(),
                distance <= 8,
                "{base}, {distance} ULPs"
            );
        }
    }
    let large = 1e8_f64;
    assert!(
        compare_shape_reference_plan(
            &shape_plan_with_first_contour_x(&reference.plan, large),
            &shape_plan_with_first_contour_x(&reference.plan, f64::from_bits(large.to_bits() + 1)),
        )
        .is_err(),
        "absolute cap rejects even one ULP at large magnitudes"
    );
    let zero = shape_plan_with_first_contour_x(&reference.plan, 0.0);
    assert!(
        compare_shape_reference_plan(
            &zero,
            &shape_plan_with_first_contour_x(&reference.plan, -0.0)
        )
        .is_err()
    );
    assert!(
        compare_shape_reference_plan(
            &zero,
            &shape_plan_with_first_contour_x(&reference.plan, f64::from_bits(8))
        )
        .is_ok()
    );
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let bad = shape_plan_with_first_contour_x(&reference.plan, invalid);
        assert!(
            compare_shape_reference_plan(&bad, &bad).is_err(),
            "non-finite values never pass, including identical inputs"
        );
    }
    assert!(
        compare_shape_reference_plan(
            &reference.plan,
            &shape_plan_with_first_contour_x(&reference.plan, 0.001)
        )
        .is_err()
    );
}

#[test]
fn shape_reference_plan_rejects_unrelated_semantic_drift() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shapes/reference.json"
    ))
    .unwrap();
    for (before, after) in [
        ("width: 40.0,", "width: 40.00000000000001,"),
        ("position_x: 0.0,", "position_x: 0.000000000000001,"),
        ("density: 1.0,", "density: 1.0000000000000002,"),
        ("r: 1.0,", "r: 0.9999999999999999,"),
        ("track_index: 0,", "track_index: 1,"),
        ("end_ms: 1000,", "end_ms: 1001,"),
        ("left: 5.0,", "left: 5.000000000000001,"),
        ("closed: true,", "closed: false,"),
    ] {
        assert!(reference.plan.contains(before));
        let actual = reference.plan.replacen(before, after, 1);
        let error = compare_shape_reference_plan(&reference.plan, &actual).unwrap_err();
        assert!(error.contains("line"), "mismatch has location: {error}");
    }
    // Authored VectorPoints have identical field names but are outside contours.
    let start = reference.plan.find("geometry: Polygon {").unwrap();
    let point = start + reference.plan[start..].find("x: ").unwrap() + 3;
    let end = point + reference.plan[point..].find(',').unwrap();
    let value: f64 = reference.plan[point..end].parse().unwrap();
    let changed = format!(
        "{}{:?}{}",
        &reference.plan[..point],
        f64::from_bits(value.to_bits() + 1),
        &reference.plan[end..]
    );
    assert!(compare_shape_reference_plan(&reference.plan, &changed).is_err());
}

#[test]
fn shape_reference_plan_rejects_malformed_or_mismatched_structure() {
    let reference: Reference = serde_json::from_str(include_str!(
        "../../../tests/fixtures/shapes/reference.json"
    ))
    .unwrap();
    for actual in [
        reference.plan.replacen("VectorPoint {", "OtherPoint {", 1),
        reference.plan.replacen("contours: [", "unscoped: [", 1),
        reference.plan.replacen(
            "x: 0.0,",
            "x: 0.0,\n                                    z: 0.0,",
            1,
        ),
        reference.plan.replacen("x: 0.0,", "", 1),
        reference.plan.replacen("y: 0.0,", "x: 0.0,", 1),
        reference.plan[..reference.plan.len() - 1].to_owned(),
        format!("{}\nextra", reference.plan),
        format!("{}\n", reference.plan),
    ] {
        assert!(compare_shape_reference_plan(&reference.plan, &actual).is_err());
    }
    for malformed in ["", "EvaluatedScene {", "WrongRoot {\n}"] {
        assert!(compare_shape_reference_plan(malformed, malformed).is_err());
    }
    // Even identical malformed coordinate scopes must fail rather than bypass validation.
    let bad = reference.plan.replacen("contours: [", "contours: (", 1);
    assert!(compare_shape_reference_plan(&bad, &bad).is_err());
}
