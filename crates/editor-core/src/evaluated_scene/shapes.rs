//! Canonical path compilation, analytic bounds and bounded curve subdivision.
use super::invalid;
use super::{
    EvaluatedAffine, EvaluatedKeyframeValue, EvaluatedProperty, EvaluatedScene,
    EvaluatedVisualLayer, EvaluatedVisualSource, IDENTITY_MATRIX, affine_from_matrices,
    multiply_matrix, transform_matrices,
};
use crate::{CoreError, FillRule, Paint, PathCommand, ShapeGeometry, Stroke, VectorPoint};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

pub(crate) const MAX_SEGMENTS: usize = 65536;
pub(crate) const MAX_SCENE_SEGMENTS: usize = 1048576;
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Contour {
    pub points: Vec<VectorPoint>,
    pub closed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedShape {
    pub geometry: ShapeGeometry,
    pub fill: Option<Paint>,
    pub stroke: Option<Stroke>,
    pub fill_rule: FillRule,
    pub contours: Vec<Contour>,
    /// Analytic unstroked [left,top,right,bottom], independent of flattening.
    pub bounds: [f64; 4],
    pub origin: (f64, f64),
    pub size: (u32, u32),
    pub density: f64,
    work_segments: usize,
}
fn point(x: f64, y: f64) -> VectorPoint {
    VectorPoint { x, y }
}
fn mix(a: VectorPoint, b: VectorPoint, t: f64) -> VectorPoint {
    point(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}
fn distance(a: VectorPoint, b: VectorPoint) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}
struct Compiler {
    contours: Vec<Contour>,
    bounds: [f64; 4],
    count: usize,
    tolerance: f64,
    limit: usize,
}
impl Compiler {
    fn bound(&mut self, p: VectorPoint) {
        self.bounds[0] = self.bounds[0].min(p.x);
        self.bounds[1] = self.bounds[1].min(p.y);
        self.bounds[2] = self.bounds[2].max(p.x);
        self.bounds[3] = self.bounds[3].max(p.y);
    }
    fn start(&mut self, p: VectorPoint) {
        self.contours.push(Contour {
            points: vec![p],
            closed: false,
        });
    }
    fn end(&self) -> VectorPoint {
        *self.contours.last().unwrap().points.last().unwrap()
    }
    fn push(&mut self, p: VectorPoint) -> Result<(), CoreError> {
        if self.count == self.limit {
            return Err(invalid("shape segment limit exceeded"));
        }
        self.count += 1;
        self.contours.last_mut().unwrap().points.push(p);
        Ok(())
    }
    fn line(&mut self, p: VectorPoint) -> Result<(), CoreError> {
        self.bound(self.end());
        self.bound(p);
        self.push(p)
    }
    fn curve(
        &mut self,
        a: VectorPoint,
        b: VectorPoint,
        c: VectorPoint,
        d: VectorPoint,
    ) -> Result<(), CoreError> {
        self.bound(a);
        self.bound(d);
        // Interior extrema of each cubic coordinate solve the quadratic derivative.
        for (a0, b0, c0, d0) in [(a.x, b.x, c.x, d.x), (a.y, b.y, c.y, d.y)] {
            let aa = -a0 + 3.0 * b0 - 3.0 * c0 + d0;
            let bb = 2.0 * (a0 - 2.0 * b0 + c0);
            let cc = b0 - a0;
            let roots = if aa.abs() < 1e-14 {
                vec![-cc / bb]
            } else {
                let disc = bb * bb - 4.0 * aa * cc;
                if disc >= 0.0 {
                    vec![
                        (-bb - disc.sqrt()) / (2.0 * aa),
                        (-bb + disc.sqrt()) / (2.0 * aa),
                    ]
                } else {
                    vec![]
                }
            };
            for t in roots {
                if t > 0.0 && t < 1.0 {
                    self.bound(mix(
                        mix(mix(a, b, t), mix(b, c, t), t),
                        mix(mix(b, c, t), mix(c, d, t), t),
                        t,
                    ));
                }
            }
        }
        self.subdivide([a, b, c, d], 0)
    }
    fn subdivide(&mut self, p: [VectorPoint; 4], depth: u32) -> Result<(), CoreError> {
        let [a, b, c, d] = p;
        // Distance to the chord segment (not its infinite line) also bounds loops/cusps.
        let chord = |p: VectorPoint| {
            let dx = d.x - a.x;
            let dy = d.y - a.y;
            let len = dx * dx + dy * dy;
            let t = if len == 0.0 {
                0.0
            } else {
                ((p.x - a.x) * dx + (p.y - a.y) * dy) / len
            }
            .clamp(0.0, 1.0);
            distance(p, mix(a, d, t))
        };
        if chord(b).max(chord(c)) <= self.tolerance {
            return self.push(d);
        }
        if depth == 16 {
            return Err(invalid("shape curve subdivision limit exceeded"));
        }
        let ab = mix(a, b, 0.5);
        let bc = mix(b, c, 0.5);
        let cd = mix(c, d, 0.5);
        let abc = mix(ab, bc, 0.5);
        let bcd = mix(bc, cd, 0.5);
        let middle = mix(abc, bcd, 0.5);
        self.subdivide([a, ab, abc, middle], depth + 1)?;
        self.subdivide([middle, bcd, cd, d], depth + 1)
    }
    fn arc(
        &mut self,
        center: VectorPoint,
        rx: f64,
        ry: f64,
        start: f64,
        sweep: f64,
    ) -> Result<(), CoreError> {
        let radius = rx.max(ry);
        let step = if radius == 0.0 {
            sweep.abs()
        } else {
            (2.0 * (1.0 - (self.tolerance / radius).min(1.0)).acos()).max(1e-12)
        };
        let n = (sweep.abs() / step).ceil().max(1.0) as usize;
        if n > MAX_SEGMENTS - self.count {
            return Err(invalid("shape arc segment limit exceeded"));
        }
        for i in 1..=n {
            let t = start + sweep * i as f64 / n as f64;
            self.push(point(center.x + rx * t.cos(), center.y + ry * t.sin()))?;
        }
        Ok(())
    }
}
impl EvaluatedShape {
    pub fn new(
        geometry: ShapeGeometry,
        fill: Option<Paint>,
        stroke: Option<Stroke>,
        scale: f64,
    ) -> Result<Self, CoreError> {
        Self::with_budget(geometry, fill, stroke, scale, MAX_SEGMENTS)
    }
    fn with_budget(
        geometry: ShapeGeometry,
        fill: Option<Paint>,
        stroke: Option<Stroke>,
        scale: f64,
        budget: usize,
    ) -> Result<Self, CoreError> {
        crate::validate_shape(&geometry, &fill, &stroke)?;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(invalid("invalid shape sampling scale"));
        }
        let density = scale.max(1.0);
        let mut c = Compiler {
            contours: vec![],
            bounds: [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            count: 0,
            limit: budget.min(MAX_SEGMENTS),
            tolerance: 0.25 / scale.max(1.0),
        };
        let mut fill_rule = FillRule::Nonzero;
        match &geometry {
            ShapeGeometry::Rectangle {
                width: w,
                height: h,
            } => {
                c.start(point(0.0, 0.0));
                for p in [
                    point(*w, 0.0),
                    point(*w, *h),
                    point(0.0, *h),
                    point(0.0, 0.0),
                ] {
                    c.line(p)?;
                }
                c.contours.last_mut().unwrap().closed = true;
            }
            ShapeGeometry::RoundedRectangle {
                width: w,
                height: h,
                radii,
            } => {
                let r = radii.resolve(*w, *h)?;
                c.bound(point(0.0, 0.0));
                c.bound(point(*w, *h));
                c.start(point(r.top_left, 0.0));
                for (p, center, radius, start) in [
                    (
                        point(w - r.top_right, 0.0),
                        point(w - r.top_right, r.top_right),
                        r.top_right,
                        -FRAC_PI_2,
                    ),
                    (
                        point(*w, h - r.bottom_right),
                        point(w - r.bottom_right, h - r.bottom_right),
                        r.bottom_right,
                        0.0,
                    ),
                    (
                        point(r.bottom_left, *h),
                        point(r.bottom_left, h - r.bottom_left),
                        r.bottom_left,
                        FRAC_PI_2,
                    ),
                    (
                        point(0.0, r.top_left),
                        point(r.top_left, r.top_left),
                        r.top_left,
                        PI,
                    ),
                ] {
                    c.line(p)?;
                    c.arc(center, radius, radius, start, FRAC_PI_2)?;
                }
                c.contours.last_mut().unwrap().closed = true;
            }
            ShapeGeometry::Ellipse {
                width: w,
                height: h,
            } => {
                c.bound(point(0.0, 0.0));
                c.bound(point(*w, *h));
                c.start(point(*w, h / 2.0));
                c.arc(point(w / 2.0, h / 2.0), w / 2.0, h / 2.0, 0.0, TAU)?;
                c.contours.last_mut().unwrap().closed = true;
            }
            ShapeGeometry::Line { start, end } => {
                c.start(*start);
                c.line(*end)?;
            }
            ShapeGeometry::Polygon { points } => {
                c.start(points[0]);
                for p in &points[1..] {
                    c.line(*p)?;
                }
                c.line(points[0])?;
                c.contours.last_mut().unwrap().closed = true;
            }
            ShapeGeometry::Star {
                center,
                outer_radius,
                inner_radius,
                point_count,
                rotation_deg,
            } => {
                let n = 2 * point_count;
                for i in 0..n {
                    let angle =
                        rotation_deg.to_radians() - FRAC_PI_2 + TAU * f64::from(i) / f64::from(n);
                    let r = if i % 2 == 0 {
                        *outer_radius
                    } else {
                        *inner_radius
                    };
                    let p = point(center.x + r * angle.cos(), center.y + r * angle.sin());
                    if i == 0 {
                        c.start(p);
                    } else {
                        c.line(p)?;
                    }
                }
                c.line(c.contours[0].points[0])?;
                c.contours[0].closed = true;
            }
            ShapeGeometry::Path { path } => {
                fill_rule = path.fill_rule;
                for cmd in &path.commands {
                    match cmd {
                        PathCommand::MoveTo { to } => c.start(*to),
                        PathCommand::LineTo { to } => c.line(*to)?,
                        PathCommand::QuadraticTo { control, to } => {
                            let a = c.end();
                            c.curve(
                                a,
                                mix(a, *control, 2.0 / 3.0),
                                mix(*to, *control, 2.0 / 3.0),
                                *to,
                            )?;
                        }
                        PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => c.curve(c.end(), *control1, *control2, *to)?,
                        PathCommand::Close {} => {
                            let p = c.contours.last().unwrap().points[0];
                            c.line(p)?;
                            c.contours.last_mut().unwrap().closed = true;
                        }
                    }
                }
            }
        }
        if !c.bounds[0].is_finite() {
            c.bounds = [0.0; 4];
        }
        if stroke
            .as_ref()
            .is_some_and(|s| ((s.width * density) as f32) <= 0.0)
        {
            return Err(invalid("shape stroke precision exceeds raster limits"));
        }
        let pad = stroke.as_ref().map_or(1.0 / density, |s| {
            s.width
                * 0.5
                * if s.line_join == crate::LineJoin::Miter {
                    s.miter_limit
                } else {
                    2.0_f64.sqrt()
                }
                + 1.0 / density
        });
        let origin = (
            ((c.bounds[0] - pad) * density).floor() / density,
            ((c.bounds[1] - pad) * density).floor() / density,
        );
        let w = ((c.bounds[2] + pad) * density).ceil() - (origin.0 * density).round();
        let h = ((c.bounds[3] + pad) * density).ceil() - (origin.1 * density).round();
        if !w.is_finite() || !h.is_finite() || w > 16384.0 || h > 16384.0 || w * h > 16777216.0 {
            return Err(invalid("shape raster bounds exceed complexity limits"));
        }
        // Include dash splitting in both per-shape and aggregate allocation budgets.
        let mut work_segments = c.count;
        if let Some(s) = &stroke
            && !s.dash.is_empty()
        {
            let period: f64 = s.dash.iter().sum();
            if s.dash.iter().any(|v| ((*v * density) as f32) <= 0.0) {
                return Err(invalid("shape dash precision exceeds raster limits"));
            }
            for contour in &c.contours {
                let length: f64 = contour
                    .points
                    .windows(2)
                    .map(|p| distance(p[0], p[1]))
                    .sum();
                let splits = (length / period * s.dash.len() as f64).ceil();
                if splits > c.limit.saturating_sub(work_segments) as f64 {
                    return Err(invalid("shape dash segment limit exceeded"));
                }
                work_segments += splits as usize;
            }
        }
        Ok(Self {
            geometry,
            fill,
            stroke,
            fill_rule,
            contours: c.contours,
            work_segments,
            density,
            bounds: c.bounds,
            origin,
            size: (w.max(1.0) as u32, h.max(1.0) as u32),
        })
    }
    pub fn segments(&self) -> usize {
        self.work_segments
    }
}

pub(super) fn affine(
    layer: &EvaluatedVisualLayer,
    shape: &EvaluatedShape,
    output_canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    // Local units belong to the component; sampling bounds belong to the output.
    let local_canvas = layer.instance.map_or(output_canvas, |i| i.canvas);
    let (mut m, opacity) = if let Some(mut t) = layer.transform2d {
        let extent = |axis| {
            let extent = shape.bounds[axis + 2] - shape.bounds[axis];
            if extent == 0.0 && matches!(shape.geometry, ShapeGeometry::Path { .. }) {
                1.0
            } else {
                extent
            }
        };
        let ax = shape.bounds[0] + t.anchor.x * extent(0);
        let ay = shape.bounds[1] + t.anchor.y * extent(1);
        t.anchor = crate::TransformAnchor { x: 0.0, y: 0.0 };
        let (m, _) = transform_matrices(t, (1, 1), local_canvas)?;
        (
            multiply_matrix(
                m,
                [1.0, 0.0, 0.0, 1.0, shape.origin.0 - ax, shape.origin.1 - ay],
            ),
            t.opacity,
        )
    } else {
        let s = layer.transform.scale;
        (
            [
                s,
                0.0,
                0.0,
                s,
                layer.transform.position_x + s * shape.origin.0,
                layer.transform.position_y + s * shape.origin.1,
            ],
            layer.transform.opacity,
        )
    };
    let parent = layer.ancestors.map_or(IDENTITY_MATRIX, |p| p.matrix);
    m = multiply_matrix(parent, m);
    m = multiply_matrix(
        m,
        [1.0 / shape.density, 0.0, 0.0, 1.0 / shape.density, 0.0, 0.0],
    );
    let [a, b, c, d, x, y] = m;
    let det = a * d - b * c;
    let inv = [
        d / det,
        -b / det,
        -c / det,
        a / det,
        (c * y - d * x) / det,
        (b * x - a * y) / det,
    ];
    if m.iter().chain(&inv).any(|v| !v.is_finite()) {
        return Err(invalid("non-finite shape matrix"));
    }
    let mut result = affine_from_matrices(
        m,
        inv,
        shape.size,
        opacity * layer.ancestors.map_or(1.0, |p| p.opacity),
    )?;
    if layer.has_animated_geometry() {
        let mut xs = vec![layer.transform.position_x];
        let mut ys = vec![layer.transform.position_y];
        let mut scales = vec![layer.transform.scale];
        for k in &layer.keyframes {
            match (k.property, k.value) {
                (EvaluatedProperty::Position, EvaluatedKeyframeValue::Position { x, y }) => {
                    xs.push(x);
                    ys.push(y);
                }
                (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) => {
                    scales.push(value)
                }
                _ => {}
            }
        }
        let extremes = |v: Vec<f64>| {
            [
                v.iter().copied().fold(f64::INFINITY, f64::min),
                v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ]
        };
        let mut bounds = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        let mut sample = layer.clone();
        sample.keyframes.clear();
        for x in extremes(xs) {
            for y in extremes(ys.clone()) {
                for scale in extremes(scales.clone()) {
                    sample.transform.position_x = x;
                    sample.transform.position_y = y;
                    sample.transform.scale = scale;
                    let a = affine(&sample, shape, output_canvas)?;
                    bounds[0] = bounds[0].min(a.left);
                    bounds[1] = bounds[1].min(a.top);
                    bounds[2] = bounds[2].max(a.left + f64::from(a.width));
                    bounds[3] = bounds[3].max(a.top + f64::from(a.height));
                }
            }
        }
        result.left = bounds[0].clamp(0.0, f64::from(output_canvas.0));
        result.top = bounds[1].clamp(0.0, f64::from(output_canvas.1));
        result.width =
            (bounds[2].clamp(0.0, f64::from(output_canvas.0)) - result.left).max(0.0) as u32;
        result.height =
            (bounds[3].clamp(0.0, f64::from(output_canvas.1)) - result.top).max(0.0) as u32;
    }
    Ok(result)
}

/// Largest singular value of the linear transform, normalized to avoid overflow.
fn magnification(m: [f64; 6]) -> f64 {
    let n = m[..4].iter().fold(0.0_f64, |n, v| n.max(v.abs()));
    if n == 0.0 {
        return 0.0;
    }
    let [a, b, c, d] = std::array::from_fn(|i| m[i] / n);
    n * ((a + d).hypot(b - c) + (a - d).hypot(b + c)) * 0.5
}

pub(super) fn refine_scene(scene: &mut EvaluatedScene) -> Result<(), CoreError> {
    let mut replacements = vec![];
    let mut count = 0;
    let mut raster_bytes = 0_u64;
    for (i, layer) in scene.visual_layers.iter().enumerate() {
        if let EvaluatedVisualSource::Shape(shape) = &layer.source {
            // Density comes from authored transforms, before raster padding or
            // source-space compensation can affect geometry validation.
            let local = if let Some(mut t) = layer.transform2d {
                t.anchor = crate::TransformAnchor { x: 0.0, y: 0.0 };
                transform_matrices(
                    t,
                    (1, 1),
                    layer
                        .instance
                        .map_or((scene.canvas.width, scene.canvas.height), |i| i.canvas),
                )?
                .0
            } else {
                [
                    layer.transform.scale,
                    0.0,
                    0.0,
                    layer.transform.scale,
                    0.0,
                    0.0,
                ]
            };
            let base_scale = magnification(multiply_matrix(
                layer.ancestors.map_or(IDENTITY_MATRIX, |p| p.matrix),
                local,
            ));
            let mut scale = base_scale;
            if layer.transform2d.is_none() {
                for k in &layer.keyframes {
                    if let (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) =
                        (k.property, k.value)
                    {
                        scale = scale.max(base_scale / layer.transform.scale * value);
                    }
                }
            }
            let value = EvaluatedShape::with_budget(
                shape.geometry.clone(),
                shape.fill.clone(),
                shape.stroke.clone(),
                scale,
                MAX_SCENE_SEGMENTS - count,
            )?;
            // Three RGBA buffers (fill coverage, stroke coverage, output). The
            // aggregate bound derives from the existing surface and layer limits.
            raster_bytes = raster_bytes
                .checked_add(u64::from(value.size.0) * u64::from(value.size.1) * 12)
                .ok_or_else(|| invalid("scene shape raster allocation overflow"))?;
            if raster_bytes > 16_777_216 * 12 * super::MAX_EVALUATED_VISUAL_LAYERS as u64 {
                return Err(invalid("scene shape raster allocation limit exceeded"));
            }
            count += value.segments();
            if count > MAX_SCENE_SEGMENTS {
                return Err(invalid("scene shape segment limit exceeded"));
            }
            let resolved = affine(layer, &value, (scene.canvas.width, scene.canvas.height))?;
            replacements.push((i, value, resolved));
        }
    }
    for (i, value, resolved) in replacements {
        scene.visual_layers[i].affine = Some(resolved);
        scene.visual_layers[i].source_size = Some(value.size);
        scene.visual_layers[i].source = EvaluatedVisualSource::Shape(Box::new(value));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn compile(geometry: serde_json::Value) -> EvaluatedShape {
        EvaluatedShape::new(
            serde_json::from_value(geometry).unwrap(),
            Some(Paint::Solid {
                color: crate::VectorColor {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            }),
            None,
            1.0,
        )
        .unwrap()
    }
    #[test]
    fn density_surface_limits_and_singular_value_oracles() {
        assert_eq!(magnification([3.0, 0.0, 0.0, 2.0, 9.0, 8.0]), 3.0);
        assert!(
            (magnification([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]) - (1.0 + 5_f64.sqrt()) / 2.0).abs()
                < 1e-12
        );
        let make = |width, density| {
            EvaluatedShape::new(
                ShapeGeometry::Rectangle {
                    width,
                    height: width,
                },
                Some(Paint::Solid {
                    color: crate::VectorColor {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                }),
                None,
                density,
            )
        };
        assert_eq!(make(2047.0, 2.0).unwrap().size, (4096, 4096));
        assert!(make(2047.1, 2.0).is_err());
        assert!(make(100.0, 100.0).is_err());
    }

    #[test]
    fn independent_analytic_curve_bounds_and_subdivision() {
        let shape = compile(
            json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"quadraticTo","control":{"x":10,"y":20},"to":{"x":20,"y":0}}]}}),
        );
        assert_eq!(shape.bounds, [0.0, 0.0, 20.0, 10.0]);
        for p in &shape.contours[0].points {
            let t = p.x / 20.0;
            assert!((p.y - 40.0 * t * (1.0 - t)).abs() < 1e-9);
        }
        let shape = compile(
            json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"cubicTo","control1":{"x":0,"y":40},"control2":{"x":40,"y":40},"to":{"x":40,"y":0}}]}}),
        );
        assert_eq!(shape.bounds, [0.0, 0.0, 40.0, 30.0]);
        let high =
            EvaluatedShape::new(shape.geometry.clone(), shape.fill.clone(), None, 100.0).unwrap();
        assert!(high.segments() > shape.segments());
        assert!(EvaluatedShape::new(shape.geometry, shape.fill, None, 1e30).is_err());
    }
    #[test]
    fn star_orientation_radius_resolution_and_empty_paths() {
        let star = compile(
            json!({"type":"star","center":{"x":20,"y":20},"outerRadius":10,"innerRadius":5,"pointCount":4,"rotationDeg":0}),
        );
        assert_eq!(star.contours[0].points[0], point(20.0, 10.0));
        assert!(star.contours[0].points[1].x > 20.0);
        assert_eq!(star.bounds, [10.0, 10.0, 30.0, 30.0]);
        let empty = compile(
            json!({"type":"path","path":{"fillRule":"evenodd","commands":[{"type":"moveTo","to":{"x":500,"y":600}}]}}),
        );
        assert_eq!(empty.segments(), 0);
        let rounded = compile(
            json!({"type":"roundedRectangle","width":20,"height":10,"radii":{"topLeft":20,"topRight":20,"bottomLeft":20,"bottomRight":20}}),
        );
        assert_eq!(rounded.contours[0].points[0], point(5.0, 0.0));
        assert_eq!(rounded.bounds, [0.0, 0.0, 20.0, 10.0]);
    }
    #[test]
    fn segment_and_surface_bounds_fail_before_raster_work() {
        let geometry = ShapeGeometry::Rectangle {
            width: 16384.0,
            height: 16384.0,
        };
        let fill = Some(Paint::Solid {
            color: crate::VectorColor {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        });
        assert!(EvaluatedShape::new(geometry, fill.clone(), None, 1.0).is_err());
        let mut c = Compiler {
            contours: vec![],
            bounds: [0.0; 4],
            count: MAX_SEGMENTS - 1,
            tolerance: 0.25,
            limit: MAX_SEGMENTS,
        };
        c.start(point(0.0, 0.0));
        c.push(point(1.0, 1.0)).unwrap();
        assert!(c.push(point(2.0, 2.0)).is_err());
        assert_eq!(c.count, MAX_SEGMENTS);
        let geometry = ShapeGeometry::Polygon {
            points: vec![point(0.0, 0.0); 4096],
        };
        assert!(EvaluatedShape::new(geometry, fill, None, 1.0).is_ok());
    }
}
