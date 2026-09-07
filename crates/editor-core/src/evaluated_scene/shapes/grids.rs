//! Procedural expansion into the common evaluated vector compositor.
use super::*;
use crate::validation::grid::{axis_count, family_range, normals, validate_grid};
use crate::{GridDescriptor, GridPattern};

/// Clip closed coverage, never the stroke centerline. Each intermediate polygon
/// is bounded before a vertex is appended; peak vertices consume scene work.
fn clip_coverage(
    contours: Vec<Contour>,
    width: f64,
    height: f64,
    budget: usize,
) -> Result<(Vec<Contour>, usize), CoreError> {
    let mut result = vec![];
    let mut count = 0;
    for contour in contours {
        let mut points = contour.points;
        if points.first() == points.last() {
            points.pop();
        }
        let mut peak = 0;
        for (axis, edge, lower) in [
            (0, 0., true),
            (0, width, false),
            (1, 0., true),
            (1, height, false),
        ] {
            let coordinate = |p: VectorPoint| if axis == 0 { p.x } else { p.y };
            let inside = |p: VectorPoint| {
                if lower {
                    coordinate(p) >= edge
                } else {
                    coordinate(p) <= edge
                }
            };
            let mut clipped = vec![];
            let mut push = |p: VectorPoint| -> Result<(), CoreError> {
                if !p.x.is_finite() || !p.y.is_finite() {
                    return Err(invalid("grid clipping precision"));
                }
                if clipped.last() != Some(&p) {
                    if clipped.len() >= budget.saturating_sub(count) {
                        return Err(invalid("grid clipping segment limit exceeded"));
                    }
                    clipped.push(p);
                }
                Ok(())
            };
            if let Some(&last) = points.last() {
                let mut a = last;
                for &b in &points {
                    if inside(a) != inside(b) {
                        let t = (edge - coordinate(a)) / (coordinate(b) - coordinate(a));
                        let mut crossing = mix(a, b, t);
                        if axis == 0 {
                            crossing.x = edge;
                        } else {
                            crossing.y = edge;
                        }
                        push(crossing)?;
                    }
                    if inside(b) {
                        push(b)?;
                    }
                    a = b;
                }
            }
            peak = peak.max(clipped.len());
            points = clipped;
        }
        if points.first() == points.last() {
            points.pop();
        }
        count += peak;
        if points.len() >= 3 {
            result.push(Contour {
                points,
                closed: true,
            });
        }
    }
    Ok((result, count))
}

/// Grids contain only straight strokes. Resolve one dash at a time so the
/// backend stroker's temporary outline stays constant-sized, then flatten with
/// the same output-space tolerance and checked compiler used by vector shapes.
fn stroke_coverage(
    child: &EvaluatedShape,
    budget: usize,
) -> Result<(Vec<Contour>, usize), CoreError> {
    let path = child
        .svg_raster_path()?
        .ok_or_else(|| invalid("missing grid line"))?;
    let mut style = child
        .svg_raster_stroke()?
        .ok_or_else(|| invalid("missing grid stroke"))?;
    let path = if let Some(dash) = style.dash.take() {
        // A short line wholly inside an off interval has no painted dashes.
        let Some(path) = path.dash(&dash, 1.) else {
            return Ok((vec![], 0));
        };
        path
    } else {
        path
    };
    let mut compiler = Compiler {
        contours: vec![],
        bounds: [0.; 4],
        count: 0,
        tolerance: 0.25 / child.density,
        limit: budget,
    };
    let mut start = tiny_skia::Point::from_xy(0., 0.);
    for segment in path.segments() {
        match segment {
            tiny_skia::PathSegment::MoveTo(p) => start = p,
            tiny_skia::PathSegment::LineTo(end) => {
                let mut builder = tiny_skia::PathBuilder::new();
                builder.move_to(start.x, start.y);
                builder.line_to(end.x, end.y);
                let outline = builder
                    .finish()
                    .and_then(|p| p.stroke(&style, 1.))
                    .ok_or_else(|| invalid("grid stroke expansion"))?;
                let local = |p: tiny_skia::Point| {
                    point(
                        f64::from(p.x) / child.density,
                        f64::from(p.y) / child.density,
                    )
                };
                for segment in outline.segments() {
                    match segment {
                        tiny_skia::PathSegment::MoveTo(p) => {
                            if compiler.count == compiler.limit {
                                return Err(invalid("grid outline segment limit exceeded"));
                            }
                            compiler.count += 1;
                            compiler.start(local(p));
                        }
                        tiny_skia::PathSegment::LineTo(p) => compiler.line(local(p))?,
                        tiny_skia::PathSegment::QuadTo(control, end) => {
                            let a = compiler.end();
                            let (b, d) = (local(control), local(end));
                            compiler.curve(a, mix(a, b, 2. / 3.), mix(d, b, 2. / 3.), d)?;
                        }
                        tiny_skia::PathSegment::CubicTo(b, c, d) => {
                            compiler.curve(compiler.end(), local(b), local(c), local(d))?
                        }
                        tiny_skia::PathSegment::Close => {
                            compiler.contours.last_mut().unwrap().closed = true
                        }
                    }
                }
                start = end;
            }
            _ => return Err(invalid("grid dash must contain straight lines")),
        }
    }
    Ok((compiler.contours, compiler.count))
}

impl EvaluatedShape {
    pub(crate) fn pending_grid(grid: GridDescriptor) -> Self {
        Self {
            geometry: ShapeGeometry::Rectangle {
                width: grid.width,
                height: grid.height,
            },
            bounds: [0., 0., grid.width, grid.height],
            size: (grid.width.ceil() as u32, grid.height.ceil() as u32),
            grid_descriptor: Some(grid),
            svg_document: None,
            svg_children: None,
            fill: None,
            stroke: None,
            fill_rule: FillRule::Nonzero,
            contours: vec![],
            origin: (0., 0.),
            density: 1.,
            work_segments: 0,
        }
    }

    pub(crate) fn grid_with_budget(
        grid: GridDescriptor,
        scale: f64,
        budget: usize,
    ) -> Result<Self, CoreError> {
        validate_grid(&grid)?;
        if !scale.is_finite() || scale <= 0. {
            return Err(invalid("invalid grid sampling scale"));
        }
        let density = scale.max(1.);
        let w = (grid.width * density).ceil();
        let h = (grid.height * density).ceil();
        if !w.is_finite() || !h.is_finite() || w > 16384. || h > 16384. || w * h > 16777216. {
            return Err(invalid("grid raster bounds exceed complexity limits"));
        }
        let mut result = Self::pending_grid(grid.clone());
        result.density = density;
        result.size = (w as u32, h as u32);
        let mut children = vec![];
        let mut count = 0;
        let mut push = |geometry: ShapeGeometry,
                        offset: VectorPoint,
                        fill: Option<Paint>,
                        stroke: Option<Stroke>|
         -> Result<(), CoreError> {
            let (mut c, fill_rule, work_segments) = compile_contours(
                &geometry,
                &stroke,
                density,
                budget.min(MAX_SEGMENTS) - count,
            )?;
            count += work_segments;
            for contour in &mut c.contours {
                for p in &mut contour.points {
                    p.x += offset.x;
                    p.y += offset.y;
                }
            }
            let mut child = Self {
                geometry,
                grid_descriptor: None,
                svg_document: None,
                svg_children: None,
                fill,
                stroke,
                fill_rule,
                contours: c.contours,
                bounds: result.bounds,
                origin: (0., 0.),
                size: result.size,
                density,
                work_segments,
            };
            child.svg_raster_stroke()?;
            child.svg_raster_path()?;
            let coverage = if let Some(stroke) = &child.stroke {
                let paint = stroke.paint.clone();
                let (coverage, segments) =
                    stroke_coverage(&child, budget.min(MAX_SEGMENTS) - count)?;
                count += segments;
                child.work_segments += segments;
                child.fill = Some(paint);
                child.stroke = None;
                coverage
            } else {
                std::mem::take(&mut child.contours)
            };
            let (coverage, segments) = clip_coverage(
                coverage,
                grid.width,
                grid.height,
                budget.min(MAX_SEGMENTS) - count,
            )?;
            count += segments;
            child.work_segments += segments;
            child.contours = coverage;
            child.svg_raster_path()?;
            children.push(child);
            Ok(())
        };
        match &grid.pattern {
            GridPattern::Rectangular {
                spacing_x,
                spacing_y,
                stroke,
            } => {
                for i in 0..axis_count(grid.width, *spacing_x)? {
                    let x = i as f64 * spacing_x;
                    push(
                        ShapeGeometry::Line {
                            start: point(x, 0.),
                            end: point(x, grid.height),
                        },
                        point(0., 0.),
                        None,
                        Some(stroke.clone()),
                    )?;
                }
                for i in 0..axis_count(grid.height, *spacing_y)? {
                    let y = i as f64 * spacing_y;
                    push(
                        ShapeGeometry::Line {
                            start: point(0., y),
                            end: point(grid.width, y),
                        },
                        point(0., 0.),
                        None,
                        Some(stroke.clone()),
                    )?;
                }
            }
            GridPattern::Dot {
                spacing_x,
                spacing_y,
                radius,
                paint,
            } => {
                for j in 0..axis_count(grid.height, *spacing_y)? {
                    for i in 0..axis_count(grid.width, *spacing_x)? {
                        push(
                            ShapeGeometry::Ellipse {
                                width: 2. * radius,
                                height: 2. * radius,
                            },
                            point(i as f64 * spacing_x - radius, j as f64 * spacing_y - radius),
                            Some(paint.clone()),
                            None,
                        )?;
                    }
                }
            }
            GridPattern::Diagonal { spacing, stroke }
            | GridPattern::Isometric { spacing, stroke } => {
                for &(nx, ny) in normals(&grid.pattern) {
                    let (a, b) = family_range(grid.width, grid.height, (nx, ny), *spacing)?;
                    for k in a..=b {
                        let d = f64::from(k) * spacing;
                        let (start, end) = if ny == 0. {
                            (point(d / nx, 0.), point(d / nx, grid.height))
                        } else {
                            let x0 = d / nx;
                            let x1 = (d - ny * grid.height) / nx;
                            let left = x0.min(x1).max(0.);
                            let right = x0.max(x1).min(grid.width);
                            (
                                point(left, ((d - nx * left) / ny).clamp(0., grid.height)),
                                point(right, ((d - nx * right) / ny).clamp(0., grid.height)),
                            )
                        };
                        if start == end {
                            return Err(invalid("grid line precision"));
                        }
                        push(
                            ShapeGeometry::Line { start, end },
                            point(0., 0.),
                            None,
                            Some(stroke.clone()),
                        )?;
                    }
                }
            }
        }
        result.svg_children = Some(children);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    fn catalog() -> Value {
        serde_json::from_str(include_str!(
            "../../../../../contracts/procedural-grids-v1.json"
        ))
        .unwrap()
    }
    fn descriptor(i: usize) -> GridDescriptor {
        serde_json::from_value(catalog()["valid"][i]["grid"].clone()).unwrap()
    }

    #[test]
    fn grid_clipping_oracle_orientation_and_peak_budget() {
        // A triangle crossing two opposite edges; coordinates are independent
        // of the clipper and exercise generated intersections and orientation.
        let input = vec![point(-1., 0.), point(3., 0.), point(1., 2.)];
        let expected = vec![
            point(0., 1.),
            point(0., 0.),
            point(2., 0.),
            point(2., 1.),
            point(1., 2.),
        ];
        let contour = |points| {
            vec![Contour {
                points,
                closed: true,
            }]
        };
        let (out, cost) = clip_coverage(contour(input.clone()), 2., 2., 100).unwrap();
        assert_eq!(out[0].points, expected);
        assert_eq!(
            clip_coverage(contour(input.clone()), 2., 2., cost)
                .unwrap()
                .0,
            out
        );
        assert!(clip_coverage(contour(input.clone()), 2., 2., cost - 1).is_err());
        let area = |p: &[VectorPoint]| {
            p.iter()
                .zip(p.iter().cycle().skip(1))
                .take(p.len())
                .map(|(a, b)| a.x * b.y - b.x * a.y)
                .sum::<f64>()
        };
        let reversed = input.into_iter().rev().collect();
        let (back, _) = clip_coverage(contour(reversed), 2., 2., 100).unwrap();
        assert_eq!(area(&out[0].points), -area(&back[0].points));
        assert!(
            clip_coverage(
                contour(vec![point(f64::NAN, 0.), point(1., 1.), point(0., 1.)]),
                2.,
                2.,
                100
            )
            .is_err()
        );
    }

    #[test]
    fn grid_all_patterns_clip_coverage_before_scaled_rasterization() {
        for i in 0..4 {
            for scale in [1., 2.5] {
                for cap in ["butt", "round", "square"] {
                    let mut value = serde_json::to_value(descriptor(i)).unwrap();
                    value["width"] = json!(10.5);
                    value["height"] = json!(9.25);
                    if i != 2 {
                        value["pattern"]["stroke"]["dash"] = json!([2.5, 1.5]);
                        value["pattern"]["stroke"]["dashOffset"] = json!(1.25);
                        value["pattern"]["stroke"]["lineCap"] = json!(cap);
                    }
                    let grid: GridDescriptor = serde_json::from_value(value).unwrap();
                    let shape = EvaluatedShape::grid_with_budget(grid.clone(), scale, MAX_SEGMENTS)
                        .unwrap();
                    assert_eq!(
                        shape,
                        EvaluatedShape::grid_with_budget(grid.clone(), scale, shape.segments())
                            .unwrap()
                    );
                    assert!(
                        EvaluatedShape::grid_with_budget(grid, scale, shape.segments() - 1)
                            .is_err()
                    );
                    for child in shape.svg_children.unwrap() {
                        assert!(child.stroke.is_none());
                        for p in child.contours.iter().flat_map(|c| &c.points) {
                            assert!((0. ..=10.5).contains(&p.x) && (0. ..=9.25).contains(&p.y));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn independent_grid_geometry_oracles() {
        let s = EvaluatedShape::grid_with_budget(descriptor(0), 1., MAX_SEGMENTS).unwrap();
        let lines: Vec<_> = s
            .svg_children
            .unwrap()
            .iter()
            .map(|c| {
                let ShapeGeometry::Line { start, end } = c.geometry else {
                    panic!("expected lattice line");
                };
                vec![start.x, start.y, end.x, end.y]
            })
            .collect();
        let expected: Vec<Vec<f64>> =
            serde_json::from_value(catalog()["geometryExamples"][0]["segments"].clone()).unwrap();
        assert_eq!(lines, expected);
        for i in [1, 3] {
            let grid = descriptor(i);
            let s = EvaluatedShape::grid_with_budget(grid.clone(), 1., MAX_SEGMENTS).unwrap();
            for child in s.svg_children.unwrap() {
                let ShapeGeometry::Line { start: a, end: b } = child.geometry else {
                    panic!("expected lattice line");
                };
                assert!(a.x < b.x || (a.x == b.x && a.y < b.y));
                assert!(
                    [a, b]
                        .iter()
                        .all(|p| (0. ..=20.).contains(&p.x) && (0. ..=20.).contains(&p.y))
                );
                let dx = b.x - a.x;
                let dy = b.y - a.y;
                if i == 1 {
                    assert!((dx.abs() - dy.abs()).abs() < 1e-10);
                } else if dx != 0. {
                    assert!((dy.abs() / dx - 1. / 3_f64.sqrt()).abs() < 1e-10);
                }
                let on_family = normals(&grid.pattern).iter().any(|&(nx, ny)| {
                    let k = (nx * a.x + ny * a.y) / 10.;
                    (k - k.round()).abs() < 1e-10
                        && (nx * b.x + ny * b.y - 10. * k.round()).abs() < 1e-10
                });
                assert!(on_family);
            }
        }
        let s = EvaluatedShape::grid_with_budget(descriptor(2), 1., MAX_SEGMENTS).unwrap();
        let centers: Vec<Vec<f64>> =
            serde_json::from_value(catalog()["geometryExamples"][1]["centers"].clone()).unwrap();
        let children = s.svg_children.unwrap();
        assert_eq!(children.len(), centers.len());
        for (c, center) in children.iter().zip(centers) {
            for p in &c.contours[0].points {
                assert!((0. ..=20.).contains(&p.x) && (0. ..=20.).contains(&p.y));
                assert!(
                    p.x == 0.
                        || p.y == 0.
                        || p.x == 20.
                        || p.y == 20.
                        || ((p.x - center[0]).hypot(p.y - center[1]) - 2.).abs() < 1e-9
                );
            }
        }
    }

    #[test]
    fn grid_sampling_and_segment_budgets_fail_closed() {
        let grid = descriptor(2);
        let shape = EvaluatedShape::grid_with_budget(grid.clone(), 1., MAX_SEGMENTS).unwrap();
        let segments = shape.segments();
        assert!(EvaluatedShape::grid_with_budget(grid.clone(), 1., segments).is_ok());
        assert!(EvaluatedShape::grid_with_budget(grid.clone(), 1., segments - 1).is_err());
        for scale in [0., f64::NAN, f64::INFINITY, 1000.] {
            assert!(EvaluatedShape::grid_with_budget(grid.clone(), scale, MAX_SEGMENTS).is_err());
        }
        let mut fractional = serde_json::to_value(grid).unwrap();
        fractional["width"] = json!(0.5);
        fractional["height"] = json!(0.5);
        let shape = EvaluatedShape::grid_with_budget(
            serde_json::from_value(fractional).unwrap(),
            2.,
            MAX_SEGMENTS,
        )
        .unwrap();
        assert_eq!(shape.bounds, [0., 0., 0.5, 0.5]);
        assert_eq!(shape.size, (1, 1));
    }
}
