//! Canonical path compilation, analytic bounds and bounded curve subdivision.
mod grids;
use super::invalid;
use super::{
    EvaluatedAffine, EvaluatedAncestors, EvaluatedKeyframeValue, EvaluatedProperty, EvaluatedScene,
    EvaluatedVisualLayer, EvaluatedVisualSource, IDENTITY_MATRIX, affine_from_matrices,
    multiply_matrix, transform_matrices,
};
use crate::{CoreError, FillRule, Paint, PathCommand, ShapeGeometry, Stroke, VectorPoint};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

pub(crate) const MAX_SEGMENTS: usize = 65536;
pub(crate) const MAX_SCENE_SEGMENTS: usize = 1048576;
// Conversion displacement only; independent of curve-flattening tolerance.
const SVG_COORDINATE_CONVERSION_TOLERANCE: f64 = 0.25;

fn svg_raster_point(x: f64, y: f64) -> Result<(f32, f32), CoreError> {
    let converted = (x as f32, y as f32);
    if !x.is_finite()
        || !y.is_finite()
        || !converted.0.is_finite()
        || !converted.1.is_finite()
        || (x - f64::from(converted.0)).hypot(y - f64::from(converted.1))
            > SVG_COORDINATE_CONVERSION_TOLERANCE
    {
        return Err(invalid("SVG coordinate conversion precision"));
    }
    Ok(converted)
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Contour {
    pub points: Vec<VectorPoint>,
    pub closed: bool,
}
#[derive(Clone, PartialEq)]
pub(crate) struct EvaluatedShape {
    pub geometry: ShapeGeometry,
    pub grid_descriptor: Option<crate::GridDescriptor>,
    pub svg_document: Option<crate::SvgDocument>,
    pub svg_children: Option<Vec<EvaluatedShape>>,
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
impl std::fmt::Debug for EvaluatedShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("EvaluatedShape");
        d.field("geometry", &self.geometry)
            .field("fill", &self.fill)
            .field("stroke", &self.stroke)
            .field("fill_rule", &self.fill_rule)
            .field("contours", &self.contours)
            .field("bounds", &self.bounds)
            .field("origin", &self.origin)
            .field("size", &self.size)
            .field("density", &self.density)
            .field("work_segments", &self.work_segments);
        if self.grid_descriptor.is_some() {
            d.field("grid_descriptor", &self.grid_descriptor)
                .field("grid_children", &self.svg_children);
        }
        if self.svg_document.is_some() {
            d.field("svg_document", &self.svg_document)
                .field("svg_children", &self.svg_children);
        }
        d.finish()
    }
}
fn point(x: f64, y: f64) -> VectorPoint {
    VectorPoint { x, y }
}
fn validate_svg_raster_bounds(bounds: [f64; 4]) -> Result<(), CoreError> {
    // tiny-skia's AA scan converter rounds each endpoint first, then uses
    // Rect::round_out. Its saturating conversion must not hide out-of-range
    // coordinates, widths, or x + width overflow. Keep degenerate fills valid.
    let b = bounds.map(|v| v as f32);
    let b = [b[0].floor(), b[1].floor(), b[2].ceil(), b[3].ceil()];
    if b.iter().any(|v| {
        !v.is_finite() || f64::from(*v) < f64::from(i32::MIN) || f64::from(*v) > f64::from(i32::MAX)
    }) {
        return Err(invalid("SVG backend raster bounds"));
    }
    let rect = tiny_skia::Rect::from_ltrb(b[0], b[1], b[2], b[3])
        .ok_or_else(|| invalid("SVG backend raster bounds"))?;
    if f64::from(rect.width()) > f64::from(i32::MAX)
        || f64::from(rect.height()) > f64::from(i32::MAX)
        || rect.round_out().is_none()
    {
        return Err(invalid("SVG backend raster bounds"));
    }
    Ok(())
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
fn compile_contours(
    geometry: &ShapeGeometry,
    stroke: &Option<Stroke>,
    scale: f64,
    budget: usize,
) -> Result<(Compiler, FillRule, usize), CoreError> {
    if !scale.is_finite() || scale <= 0. || !(0.25 / scale).is_finite() {
        return Err(invalid("invalid shape sampling scale"));
    }
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
        tolerance: 0.25 / scale,
    };
    let mut fill_rule = FillRule::Nonzero;
    match geometry {
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
    // Include dash splitting in both per-shape and aggregate allocation budgets.
    let mut work_segments = c.count;
    if let Some(s) = &stroke
        && !s.dash.is_empty()
    {
        let period: f64 = s.dash.iter().sum();
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
    Ok((c, fill_rule, work_segments))
}

impl EvaluatedShape {
    /// A document awaiting occurrence transforms, never a rasterization input.
    pub(super) fn pending_svg(document: crate::SvgDocument) -> Self {
        Self {
            geometry: ShapeGeometry::Rectangle {
                width: document.width,
                height: document.height,
            },
            bounds: [0., 0., document.width, document.height],
            size: (document.width.ceil() as u32, document.height.ceil() as u32),
            grid_descriptor: None,
            svg_document: Some(document),
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

    pub(crate) fn svg_raster_stroke(&self) -> Result<Option<tiny_skia::Stroke>, CoreError> {
        let Some(s) = &self.stroke else {
            return Ok(None);
        };
        let number = |v: f64| -> Result<f32, CoreError> {
            let v = v * self.density;
            if !v.is_finite() || !(v as f32).is_finite() || (v != 0. && v as f32 == 0.) {
                return Err(invalid("SVG mapped stroke precision"));
            }
            Ok(v as f32)
        };
        let dash = if s.dash.is_empty() {
            None
        } else {
            let values = s
                .dash
                .iter()
                .map(|v| number(*v))
                .collect::<Result<Vec<_>, _>>()?;
            let period: f64 = s.dash.iter().sum();
            let sum: f32 = values.iter().sum();
            if !period.is_finite() || !sum.is_finite() || sum <= 0. {
                return Err(invalid("SVG mapped dash period"));
            }
            Some(
                tiny_skia::StrokeDash::new(values, number(s.dash_offset.rem_euclid(period))?)
                    .ok_or_else(|| invalid("SVG mapped dash construction"))?,
            )
        };
        Ok(Some(tiny_skia::Stroke {
            width: number(s.width)?,
            miter_limit: s.miter_limit as f32,
            line_cap: match s.line_cap {
                crate::LineCap::Butt => tiny_skia::LineCap::Butt,
                crate::LineCap::Round => tiny_skia::LineCap::Round,
                crate::LineCap::Square => tiny_skia::LineCap::Square,
            },
            line_join: match s.line_join {
                crate::LineJoin::Miter => tiny_skia::LineJoin::Miter,
                crate::LineJoin::Round => tiny_skia::LineJoin::Round,
                crate::LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
            },
            dash,
        }))
    }

    pub(crate) fn svg_raster_path(&self) -> Result<Option<tiny_skia::Path>, CoreError> {
        let mut builder = tiny_skia::PathBuilder::new();
        let mut bounds = [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ];
        for contour in &self.contours {
            for (i, p) in contour.points.iter().enumerate() {
                let (x, y) = svg_raster_point(
                    (p.x - self.origin.0) * self.density,
                    (p.y - self.origin.1) * self.density,
                )?;
                bounds[0] = bounds[0].min(f64::from(x));
                bounds[1] = bounds[1].min(f64::from(y));
                bounds[2] = bounds[2].max(f64::from(x));
                bounds[3] = bounds[3].max(f64::from(y));
                if i == 0 {
                    builder.move_to(x, y);
                } else {
                    builder.line_to(x, y);
                }
            }
            if contour.closed {
                builder.close();
            }
        }
        // PathBuilder can discard move-only contours. Their numeric bounds
        // still have to be representable even though their coverage is empty.
        if bounds[0].is_finite() {
            validate_svg_raster_bounds(bounds)?;
        }
        let path = builder.finish();
        if let Some(path) = &path {
            let b = path.bounds();
            let stroke = self.svg_raster_stroke()?;
            let padding = stroke.as_ref().map_or(0., |s| {
                let join = if s.line_join == tiny_skia::LineJoin::Miter {
                    f64::from(s.miter_limit).max(1.)
                } else {
                    1.
                };
                // Square caps can extend sqrt(2) radii along an axis.
                let cap = if s.line_cap == tiny_skia::LineCap::Square {
                    2_f64.sqrt()
                } else {
                    1.
                };
                f64::from(s.width) * 0.5 * join.max(cap)
            });
            validate_svg_raster_bounds([
                f64::from(b.left()) - padding,
                f64::from(b.top()) - padding,
                f64::from(b.right()) + padding,
                f64::from(b.bottom()) + padding,
            ])?;
        }
        Ok(path)
    }
    #[cfg(test)]
    pub fn new_svg(document: crate::SvgDocument, scale: f64) -> Result<Self, CoreError> {
        Self::svg_with_budget(document, scale, MAX_SCENE_SEGMENTS)
    }
    pub(super) fn svg_with_budget(
        document: crate::SvgDocument,
        scale: f64,
        budget: usize,
    ) -> Result<Self, CoreError> {
        crate::validation::svg::validate_document(&document)?;
        if !scale.is_finite() || scale <= 0. {
            return Err(invalid("invalid SVG sampling scale"));
        }
        let density = scale.max(1.);
        let w = (document.width * density).ceil();
        let h = (document.height * density).ceil();
        if !w.is_finite() || !h.is_finite() || w > 16384. || h > 16384. || w * h > 16777216. {
            return Err(invalid("SVG raster bounds exceed complexity limits"));
        }
        let size = (w as u32, h as u32);
        let [vx, vy, vw, vh] = document.view_box;
        let mapping = (document.width / vw).min(document.height / vh);
        let dx = (document.width - vw * mapping) * 0.5 - vx * mapping;
        let dy = (document.height - vh * mapping) * 0.5 - vy * mapping;
        if !dx.is_finite() || !dy.is_finite() {
            return Err(invalid("SVG mapped geometry"));
        }
        let mut children = vec![];
        let mut count = 0;
        let raster_number =
            |v: f64| v.is_finite() && (v as f32).is_finite() && (v == 0. || v as f32 != 0.);
        for value in &document.shapes {
            let (mut c, fill_rule, work_segments) = compile_contours(
                &value.geometry,
                &value.stroke,
                density * mapping,
                budget - count,
            )?;
            count += work_segments;
            for contour in &mut c.contours {
                for p in &mut contour.points {
                    p.x = (p.x + value.offset.x) * mapping + dx;
                    p.y = (p.y + value.offset.y) * mapping + dy;
                    if !raster_number(p.x * density) || !raster_number(p.y * density) {
                        return Err(invalid("SVG mapped geometry"));
                    }
                }
            }
            let mut stroke = value.stroke.clone();
            if let Some(stroke) = &mut stroke {
                stroke.width *= mapping;
                stroke.dash_offset *= mapping;
                for dash in &mut stroke.dash {
                    *dash *= mapping;
                }
                if std::iter::once(&stroke.width)
                    .chain(&stroke.dash)
                    .any(|v| !raster_number(*v * density) || (*v * density) as f32 <= 0.)
                    || !raster_number(stroke.dash_offset * density)
                    || !raster_number(stroke.dash.iter().sum::<f64>() * density)
                {
                    return Err(invalid("SVG mapped stroke precision"));
                }
            }
            let child = Self {
                geometry: value.geometry.clone(),
                grid_descriptor: None,
                svg_document: None,
                svg_children: None,
                fill: value.fill.clone(),
                stroke,
                fill_rule,
                contours: c.contours,
                bounds: [0., 0., document.width, document.height],
                origin: (0., 0.),
                size,
                density,
                work_segments,
            };
            child.svg_raster_stroke()?;
            child.svg_raster_path()?;
            children.push(child);
        }
        Ok(Self {
            geometry: ShapeGeometry::Rectangle {
                width: document.width,
                height: document.height,
            },
            fill: None,
            stroke: None,
            fill_rule: FillRule::Nonzero,
            contours: vec![],
            bounds: [0., 0., document.width, document.height],
            origin: (0., 0.),
            size,
            density,
            work_segments: 0,
            grid_descriptor: None,
            svg_document: Some(document),
            svg_children: Some(children),
        })
    }
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
        let (c, fill_rule, work_segments) = compile_contours(&geometry, &stroke, density, budget)?;
        if let Some(s) = &stroke
            && s.dash.iter().any(|v| ((*v * density) as f32) <= 0.0)
        {
            return Err(invalid("shape dash precision exceeds raster limits"));
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
        Ok(Self {
            geometry,
            fill,
            stroke,
            fill_rule,
            contours: c.contours,
            work_segments,
            grid_descriptor: None,
            svg_document: None,
            svg_children: None,
            density,
            bounds: c.bounds,
            origin,
            size: (w.max(1.0) as u32, h.max(1.0) as u32),
        })
    }
    pub fn segments(&self) -> usize {
        self.svg_children
            .as_ref()
            .map_or(self.work_segments, |children| {
                children.iter().map(Self::segments).sum()
            })
    }
}

pub(super) fn affine(
    layer: &EvaluatedVisualLayer,
    shape: &EvaluatedShape,
    output_canvas: (u32, u32),
) -> Result<EvaluatedAffine, CoreError> {
    affine_with_ancestors(layer, shape, output_canvas, layer.ancestors)
}

fn affine_with_ancestors(
    layer: &EvaluatedVisualLayer,
    shape: &EvaluatedShape,
    output_canvas: (u32, u32),
    ancestors: Option<EvaluatedAncestors>,
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
    let parent = ancestors.map_or(IDENTITY_MATRIX, |p| p.matrix);
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
        opacity * ancestors.map_or(1.0, |p| p.opacity),
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
                    let a = affine_with_ancestors(&sample, shape, output_canvas, ancestors)?;
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

fn add_raster_bytes(total: u64, shape: &EvaluatedShape) -> Result<u64, CoreError> {
    let bytes = u64::from(shape.size.0)
        * u64::from(shape.size.1)
        * if shape.svg_children.is_some() { 44 } else { 12 };
    let total = total
        .checked_add(bytes)
        .ok_or_else(|| invalid("scene shape raster allocation overflow"))?;
    if total > 16_777_216 * 12 * super::MAX_EVALUATED_VISUAL_LAYERS as u64 {
        return Err(invalid("scene shape raster allocation limit exceeded"));
    }
    Ok(total)
}

#[derive(Default)]
struct SceneShapeBudget {
    segments: usize,
    raster_bytes: u64,
}

impl SceneShapeBudget {
    fn remaining_segments(&self) -> usize {
        MAX_SCENE_SEGMENTS - self.segments
    }

    fn add(&mut self, shape: &EvaluatedShape) -> Result<(), CoreError> {
        self.segments = self
            .segments
            .checked_add(shape.segments())
            .filter(|segments| *segments <= MAX_SCENE_SEGMENTS)
            .ok_or_else(|| invalid("scene shape segment limit exceeded"))?;
        self.raster_bytes = add_raster_bytes(self.raster_bytes, shape)?;
        Ok(())
    }
}

/// Validate complete occurrences, including hidden and unreachable content.
pub(super) fn preflight_svg_documents(project: &crate::Project) -> Result<(), CoreError> {
    use std::collections::{HashMap, HashSet};
    if !std::iter::once(&project.tracks)
        .chain(project.components.iter().map(|c| &c.tracks))
        .flat_map(|t| t.iter())
        .flat_map(|t| &t.items)
        .any(|i| {
            matches!(
                i,
                crate::TimelineItem::Svg(_) | crate::TimelineItem::Grid(_)
            )
        })
    {
        return Ok(());
    }
    struct Budget {
        occurrences: usize,
        segments: usize,
        bytes: u64,
    }
    fn matrix(node: &crate::TimelineItem) -> Result<[f64; 6], CoreError> {
        if let Some(mut t) = node.visual_properties().transform2d {
            t.anchor = crate::TransformAnchor { x: 0., y: 0. };
            Ok(transform_matrices(t, (1, 1), (1, 1))?.0)
        } else {
            let mut scale = node.visual_properties().transform.scale;
            for k in node.keyframes() {
                if let (crate::KeyframeProperty::Scale, crate::KeyframeValue::Scalar { value }) =
                    (k.property, &k.value)
                {
                    scale = scale.max(*value);
                }
            }
            Ok([scale, 0., 0., scale, 0., 0.])
        }
    }
    fn walk<'a>(
        tracks: &'a [crate::Track],
        outer: [f64; 6],
        definitions: &HashMap<&'a str, &'a crate::ComponentDefinition>,
        active: &mut Vec<&'a str>,
        reached: &mut HashSet<&'a str>,
        budget: &mut Budget,
    ) -> Result<(), CoreError> {
        let nodes: HashMap<_, _> = tracks
            .iter()
            .flat_map(|t| &t.items)
            .map(|i| (i.id(), i))
            .collect();
        for item in tracks.iter().flat_map(|t| &t.items) {
            if budget.occurrences == 65_536 {
                return Err(invalid("maxExpandedOccurrences exceeded"));
            }
            budget.occurrences += 1;
            if !matches!(
                item,
                crate::TimelineItem::Svg(_)
                    | crate::TimelineItem::Grid(_)
                    | crate::TimelineItem::ComponentInstance(_)
            ) {
                continue;
            }
            let mut local = matrix(item)?;
            let mut node = item;
            let mut depth = 0;
            while let Some(parent) = &node.visual_properties().parent {
                depth += 1;
                if depth > nodes.len() {
                    return Err(invalid("SVG ancestor cycle"));
                }
                node = nodes
                    .get(parent.id.as_str())
                    .copied()
                    .ok_or_else(|| invalid("missing SVG ancestor"))?;
                local = multiply_matrix(matrix(node)?, local);
            }
            let composed = multiply_matrix(outer, local);
            match item {
                crate::TimelineItem::Svg(svg) => {
                    let shape = EvaluatedShape::svg_with_budget(
                        svg.document.clone(),
                        magnification(composed),
                        MAX_SCENE_SEGMENTS - budget.segments,
                    )?;
                    budget.segments += shape.segments();
                    budget.bytes = add_raster_bytes(budget.bytes, &shape)?;
                }
                crate::TimelineItem::Grid(svg) => {
                    let shape = EvaluatedShape::grid_with_budget(
                        svg.grid.clone(),
                        magnification(composed),
                        MAX_SCENE_SEGMENTS - budget.segments,
                    )?;
                    budget.segments += shape.segments();
                    budget.bytes = add_raster_bytes(budget.bytes, &shape)?;
                }
                crate::TimelineItem::ComponentInstance(instance) => {
                    let id = instance.component_id.as_str();
                    if active.len() >= 16 || active.contains(&id) {
                        return Err(invalid("SVG component depth or cycle"));
                    }
                    let definition = definitions
                        .get(id)
                        .ok_or_else(|| invalid("missing SVG component"))?;
                    reached.insert(id);
                    active.push(id);
                    walk(
                        &definition.tracks,
                        composed,
                        definitions,
                        active,
                        reached,
                        budget,
                    )?;
                    active.pop();
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    let definitions = project
        .components
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();
    let mut reached = HashSet::new();
    let mut budget = Budget {
        occurrences: 0,
        segments: 0,
        bytes: 0,
    };
    walk(
        &project.tracks,
        IDENTITY_MATRIX,
        &definitions,
        &mut vec![],
        &mut reached,
        &mut budget,
    )?;
    // Freeze project reachability: every unreachable definition has its own
    // virtual-root contract, irrespective of declaration order.
    for c in &project.components {
        if !reached.contains(c.id.as_str()) {
            walk(
                &c.tracks,
                IDENTITY_MATRIX,
                &definitions,
                &mut vec![c.id.as_str()],
                &mut HashSet::new(),
                &mut Budget {
                    occurrences: 0,
                    segments: 0,
                    bytes: 0,
                },
            )?;
        }
    }
    Ok(())
}

fn measure_layer_shape(
    layer: &EvaluatedVisualLayer,
    ancestors: Option<EvaluatedAncestors>,
    output_canvas: (u32, u32),
    segment_budget: usize,
) -> Result<Option<EvaluatedShape>, CoreError> {
    let EvaluatedVisualSource::Shape(shape) = &layer.source else {
        return Ok(None);
    };
    // Density comes from authored transforms, before raster padding or
    // source-space compensation can affect geometry validation.
    let local = if let Some(mut t) = layer.transform2d {
        t.anchor = crate::TransformAnchor { x: 0.0, y: 0.0 };
        transform_matrices(
            t,
            (1, 1),
            layer
                .instance
                .map_or(output_canvas, |instance| instance.canvas),
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
        ancestors.map_or(IDENTITY_MATRIX, |parent| parent.matrix),
        local,
    ));
    let mut scale = base_scale;
    if layer.transform2d.is_none() {
        for keyframe in &layer.keyframes {
            if let (EvaluatedProperty::Scale, EvaluatedKeyframeValue::Scalar { value }) =
                (keyframe.property, keyframe.value)
            {
                scale = scale.max(base_scale / layer.transform.scale * value);
            }
        }
    }
    let value = if let Some(grid) = &shape.grid_descriptor {
        EvaluatedShape::grid_with_budget(grid.clone(), scale, segment_budget)?
    } else if let Some(document) = &shape.svg_document {
        EvaluatedShape::svg_with_budget(document.clone(), scale, segment_budget)?
    } else {
        EvaluatedShape::with_budget(
            shape.geometry.clone(),
            shape.fill.clone(),
            shape.stroke.clone(),
            scale,
            segment_budget,
        )?
    };
    Ok(Some(value))
}

pub(super) fn preflight_shape_layers<'a>(
    layers: impl IntoIterator<Item = (&'a EvaluatedVisualLayer, Option<EvaluatedAncestors>)>,
    output_canvas: (u32, u32),
) -> Result<(), CoreError> {
    let mut budget = SceneShapeBudget::default();
    for (layer, ancestors) in layers {
        let Some(value) =
            measure_layer_shape(layer, ancestors, output_canvas, budget.remaining_segments())?
        else {
            continue;
        };
        budget.add(&value)?;
        affine_with_ancestors(layer, &value, output_canvas, ancestors)?;
    }
    Ok(())
}

pub(super) fn refine_scene(scene: &mut EvaluatedScene) -> Result<(), CoreError> {
    let mut replacements = vec![];
    let mut budget = SceneShapeBudget::default();
    for (i, layer) in scene.visual_layers.iter().enumerate() {
        if let Some(value) = measure_layer_shape(
            layer,
            layer.ancestors,
            (scene.canvas.width, scene.canvas.height),
            budget.remaining_segments(),
        )? {
            budget.add(&value)?;
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
    fn svg_conversion_precision_boundaries_and_sampling() {
        let boundary = 8_388_608.25_f64;
        let excess = f64::from_bits(boundary.to_bits() + 1);
        for sign in [-1., 1.] {
            assert!(svg_raster_point(sign * boundary, 0.).is_ok());
            assert!(svg_raster_point(0., sign * boundary).is_ok());
            assert!(svg_raster_point(sign * excess, 0.).is_err());
            assert!(svg_raster_point(sign * boundary, boundary).is_err());
            assert!(svg_raster_point(sign * 500_000_000., 0.).is_ok());
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX] {
            assert!(svg_raster_point(value, 0.).is_err());
            assert!(svg_raster_point(0., value).is_err());
        }
        let mut move_only = compile(json!({"type":"rectangle","width":10,"height":10}));
        move_only.contours = vec![Contour {
            points: vec![point(boundary, 0.)],
            closed: false,
        }];
        assert!(move_only.svg_raster_path().is_ok());
        move_only.density = 2.;
        assert!(move_only.svg_raster_path().is_err());
        move_only.density = 1.;
        move_only.contours[0].points[0].x = excess;
        assert!(move_only.svg_raster_path().is_err());
        // Source point 8388.60825 maps to 8388608.25 at density 1,
        // but has 0.5 raster-pixel conversion error at density 2.
        let doc = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 .1 .1\"><path d=\"M8388.60825 0 L8388.73625 0 L8388.73625 .01 Z\"/></svg>").unwrap();
        assert!(EvaluatedShape::new_svg(doc.clone(), 1.).is_ok());
        assert!(EvaluatedShape::new_svg(doc, 2.).is_err());
    }
    #[test]
    fn svg_rejects_diagonal_conversion_loss() {
        let document = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 .001 .001\"><polygon points=\"-5000,-5000 5000,5000 5000,5000.0001 -5000,-4999.9999\" fill=\"#f00\"/></svg>").unwrap();
        assert!(EvaluatedShape::new_svg(document, 1.).is_err());
    }
    #[test]
    fn svg_backend_bounds_and_stroke_envelopes() {
        let mut move_only = compile(json!({"type":"rectangle","width":10,"height":10}));
        move_only.contours = vec![Contour {
            points: vec![point(5e9, 0.)],
            closed: false,
        }];
        assert!(move_only.svg_raster_path().is_err());
        move_only.contours[0].points[0].x = 10.;
        assert!(move_only.svg_raster_path().is_ok());
        for b in [
            [-100., -100., 100., 100.],
            [0.; 4],
            [0., 0., 2_147_483_520., 1.],
            [-2_147_483_648., 0., -128., 1.],
        ] {
            assert!(validate_svg_raster_bounds(b).is_ok(), "{b:?}");
        }
        for b in [
            [0., 0., 2_147_483_648., 1.],
            [-2_147_483_648., 0., 0., 1.],
            [f64::INFINITY; 4],
        ] {
            assert!(validate_svg_raster_bounds(b).is_err(), "{b:?}");
        }
        let mut s = compile(json!({"type":"rectangle","width":128,"height":128}));
        s.origin = (0., 0.);
        s.contours[0]
            .points
            .iter_mut()
            .for_each(|p| p.x += 2_147_483_008.);
        assert!(s.svg_raster_path().is_ok());
        s.stroke = Some(serde_json::from_value(serde_json::json!({"paint":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}},"width":1000,"lineCap":"square","lineJoin":"miter","miterLimit":4,"dash":[],"dashOffset":0})).unwrap());
        assert!(s.svg_raster_path().is_err());
        s.stroke.as_mut().unwrap().dash = vec![f64::MAX, f64::MAX];
        assert!(s.svg_raster_stroke().is_err());
    }
    #[test]
    fn svg_rejects_backend_integer_overflow() {
        let doc = crate::validation::svg::parse(
            "<svg width=\"100\" height=\"100\" viewBox=\"0 0 0.0001 0.0001\"><rect x=\"-5000\" y=\"-5000\" width=\"10000\" height=\"10000\" fill=\"#f00\"/></svg>",
        ).unwrap();
        assert!(EvaluatedShape::new_svg(doc, 1.).is_err());
    }
    #[test]
    fn svg_mapped_limits_precision_and_remaining_segments() {
        let parse = |s: &str| crate::validation::svg::parse(s).unwrap();
        for (width, height) in [(16384, 1), (4096, 4096)] {
            let doc = parse(&format!("<svg width=\"{width}\" height=\"{height}\"/>"));
            assert_eq!(
                EvaluatedShape::new_svg(doc.clone(), 1.).unwrap().size,
                (width, height)
            );
            assert!(EvaluatedShape::new_svg(doc, 1.0001).is_err());
        }
        let doc = parse(
            "<svg width=\"100\" height=\"100\" viewBox=\"100 200 10000 10000\"><rect x=\"100\" y=\"200\" width=\"10000\" height=\"10000\"/><path d=\"M100 200 Q5100 10200 10100 200\" stroke=\"#f00\" stroke-width=\"100\" stroke-dasharray=\"100 100\"/></svg>",
        );
        let shape = EvaluatedShape::new_svg(doc.clone(), 2.).unwrap();
        let bytes = u64::from(shape.size.0) * u64::from(shape.size.1) * 44;
        let cap = 16_777_216 * 12 * super::super::MAX_EVALUATED_VISUAL_LAYERS as u64;
        assert_eq!(add_raster_bytes(cap - bytes, &shape).unwrap(), cap);
        assert!(add_raster_bytes(cap - bytes + 1, &shape).is_err());
        let children = shape.svg_children.as_ref().unwrap();
        assert_eq!(children[0].contours[0].points[0], point(0., 0.));
        assert_eq!(children[0].contours[0].points[2], point(100., 100.));
        assert_eq!(children[1].stroke.as_ref().unwrap().width, 1.);
        assert_eq!(children[1].stroke.as_ref().unwrap().dash, vec![1., 1.]);
        assert!(EvaluatedShape::svg_with_budget(doc.clone(), 2., shape.segments()).is_ok());
        assert!(EvaluatedShape::svg_with_budget(doc, 2., shape.segments() - 1).is_err());
        for attribute in [
            "stroke-width=\"1e-44\"",
            "stroke-width=\"1\" stroke-dasharray=\"1e-44 1e-44\"",
        ] {
            let doc = parse(&format!(
                "<svg width=\"1\" height=\"1\" viewBox=\"0 0 1000 1000\"><line x2=\"1\" stroke=\"#f00\" {attribute}/></svg>"
            ));
            assert!(EvaluatedShape::new_svg(doc, 1.).is_err());
        }
    }

    #[test]
    fn scene_shape_budget_accepts_exact_segment_and_raster_limits_only() {
        let mut exact_segments = compile(json!({"type":"rectangle","width":1,"height":1}));
        exact_segments.work_segments = MAX_SCENE_SEGMENTS;
        exact_segments.size = (1, 1);
        let mut segment_budget = SceneShapeBudget::default();
        segment_budget.add(&exact_segments).unwrap();
        let mut one_segment = exact_segments.clone();
        one_segment.work_segments = 1;
        assert!(segment_budget.add(&one_segment).is_err());

        let mut full_surface = compile(json!({"type":"rectangle","width":1,"height":1}));
        full_surface.work_segments = 0;
        full_surface.size = (4096, 4096);
        let mut raster_budget = SceneShapeBudget::default();
        for _ in 0..super::super::MAX_EVALUATED_VISUAL_LAYERS {
            raster_budget.add(&full_surface).unwrap();
        }
        assert!(raster_budget.add(&full_surface).is_err());
        let cap = 16_777_216 * 12 * super::super::MAX_EVALUATED_VISUAL_LAYERS as u64;
        full_surface.size = (1, 1);
        let mut one_byte_over = SceneShapeBudget {
            segments: 0,
            raster_bytes: cap - 11,
        };
        assert!(one_byte_over.add(&full_surface).is_err());
        let mut exact_bytes = SceneShapeBudget {
            segments: 0,
            raster_bytes: cap - 12,
        };
        exact_bytes.add(&full_surface).unwrap();
        assert_eq!(exact_bytes.raster_bytes, cap);
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
