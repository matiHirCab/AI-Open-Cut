//! Coverage rasterization of evaluated geometry; paint semantics stay explicit.
use crate::evaluated_scene::shapes::EvaluatedShape;
use crate::{CoreError, ErrorCode, FillRule, LineCap, LineJoin, Paint, VectorColor, VectorPoint};

fn linear(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(v: f64) -> f64 {
    if v <= 0.0031308 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
fn premul(c: VectorColor) -> [f64; 4] {
    [linear(c.r) * c.a, linear(c.g) * c.a, linear(c.b) * c.a, c.a]
}
pub(super) fn paint_at(paint: &Paint, p: VectorPoint) -> [f64; 4] {
    let (t, stops) = match paint {
        Paint::Solid { color } => return premul(*color),
        Paint::LinearGradient { start, end, stops } => {
            let x = end.x - start.x;
            let y = end.y - start.y;
            (
                ((p.x - start.x) * x + (p.y - start.y) * y) / (x * x + y * y),
                stops,
            )
        }
        Paint::RadialGradient {
            center,
            radius,
            stops,
        } => ((p.x - center.x).hypot(p.y - center.y) / radius, stops),
    };
    let t = t.clamp(0.0, 1.0);
    let end = stops
        .partition_point(|s| s.offset < t)
        .clamp(1, stops.len() - 1);
    let a = &stops[end - 1];
    let b = &stops[end];
    let u = (t - a.offset) / (b.offset - a.offset);
    let a = premul(a.color);
    let b = premul(b.color);
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * u)
}
pub(super) fn rasterize(shape: &EvaluatedShape) -> Result<Vec<u8>, CoreError> {
    let fail = || CoreError::new(ErrorCode::InvalidArgument, "shape raster allocation failed");
    let (w, h) = shape.size;
    let mut fill = tiny_skia::Pixmap::new(w, h).ok_or_else(fail)?;
    let mut stroke = tiny_skia::Pixmap::new(w, h).ok_or_else(fail)?;
    let mut builder = tiny_skia::PathBuilder::new();
    for c in &shape.contours {
        if c.points.len() < 2 {
            continue;
        }
        let p = c.points[0];
        builder.move_to(
            ((p.x - shape.origin.0) * shape.density) as f32,
            ((p.y - shape.origin.1) * shape.density) as f32,
        );
        for p in &c.points[1..] {
            builder.line_to(
                ((p.x - shape.origin.0) * shape.density) as f32,
                ((p.y - shape.origin.1) * shape.density) as f32,
            );
        }
        if c.closed {
            builder.close();
        }
    }
    if let Some(path) = builder.finish() {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        if shape.fill.is_some() {
            fill.fill_path(
                &path,
                &paint,
                match shape.fill_rule {
                    FillRule::Nonzero => tiny_skia::FillRule::Winding,
                    FillRule::Evenodd => tiny_skia::FillRule::EvenOdd,
                },
                tiny_skia::Transform::identity(),
                None,
            );
        }
        if let Some(s) = &shape.stroke {
            let dash = if s.dash.is_empty() {
                None
            } else {
                tiny_skia::StrokeDash::new(
                    s.dash.iter().map(|v| (*v * shape.density) as f32).collect(),
                    (s.dash_offset.rem_euclid(s.dash.iter().sum()) * shape.density) as f32,
                )
            };
            let style = tiny_skia::Stroke {
                width: (s.width * shape.density) as f32,
                miter_limit: s.miter_limit as f32,
                line_cap: match s.line_cap {
                    LineCap::Butt => tiny_skia::LineCap::Butt,
                    LineCap::Round => tiny_skia::LineCap::Round,
                    LineCap::Square => tiny_skia::LineCap::Square,
                },
                line_join: match s.line_join {
                    LineJoin::Miter => tiny_skia::LineJoin::Miter,
                    LineJoin::Round => tiny_skia::LineJoin::Round,
                    LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
                },
                dash,
            };
            stroke.stroke_path(
                &path,
                &paint,
                &style,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
    let mut bytes =
        format!("P7\nWIDTH {w}\nHEIGHT {h}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n")
            .into_bytes();
    bytes.reserve(w as usize * h as usize * 4);
    for y in 0..h {
        for x in 0..w {
            let p = VectorPoint {
                x: shape.origin.0 + (f64::from(x) + 0.5) / shape.density,
                y: shape.origin.1 + (f64::from(y) + 0.5) / shape.density,
            };
            let i = (y * w + x) as usize;
            let f = f64::from(fill.pixels()[i].alpha()) / 255.0;
            let s = f64::from(stroke.pixels()[i].alpha()) / 255.0;
            let a = shape
                .fill
                .as_ref()
                .map_or([0.0; 4], |paint| paint_at(paint, p).map(|v| v * f));
            let b = shape
                .stroke
                .as_ref()
                .map_or([0.0; 4], |style| paint_at(&style.paint, p).map(|v| v * s));
            let color: [f64; 4] = std::array::from_fn(|j| b[j] + a[j] * (1.0 - b[3]));
            for v in &color[..3] {
                bytes.push(if color[3] == 0.0 {
                    0
                } else {
                    (srgb(v / color[3]).clamp(0.0, 1.0) * 255.0).round() as u8
                });
            }
            bytes.push((color[3].clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn shape(
        geometry: serde_json::Value,
        fill: serde_json::Value,
        stroke: serde_json::Value,
    ) -> EvaluatedShape {
        EvaluatedShape::new(
            serde_json::from_value(geometry).unwrap(),
            serde_json::from_value(fill).unwrap(),
            serde_json::from_value(stroke).unwrap(),
            1.0,
        )
        .unwrap()
    }
    fn pixel(s: &EvaluatedShape, x: f64, y: f64) -> [u8; 4] {
        let bytes = rasterize(s).unwrap();
        let offset = bytes.len() - s.size.0 as usize * s.size.1 as usize * 4;
        let x = (x - s.origin.0) as usize;
        let y = (y - s.origin.1) as usize;
        bytes
            [offset + (y * s.size.0 as usize + x) * 4..offset + (y * s.size.0 as usize + x) * 4 + 4]
            .try_into()
            .unwrap()
    }
    fn red() -> serde_json::Value {
        json!({"type":"solid","color":{"r":1,"g":0,"b":0,"a":1}})
    }
    #[test]
    fn density_preserves_local_gradient_and_dashed_stroke_metrics() {
        let make = |factor: f64, density| {
            let fill=serde_json::from_value(json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":20.0*factor,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0.5}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":1}}]})).unwrap();
            let stroke=serde_json::from_value(json!({"paint":red(),"width":factor,"dash":[2.0*factor,2.0*factor],"dashOffset":factor,"lineCap":"round","lineJoin":"miter","miterLimit":4})).unwrap();
            EvaluatedShape::new(
                crate::ShapeGeometry::Rectangle {
                    width: 20.0 * factor,
                    height: 10.0 * factor,
                },
                Some(fill),
                Some(stroke),
                density,
            )
            .unwrap()
        };
        let a = make(1.0, 4.0);
        let b = make(4.0, 1.0);
        assert_eq!(a.size, b.size);
        let a = rasterize(&a).unwrap();
        let b = rasterize(&b).unwrap();
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 1));
    }

    #[test]
    fn magnified_ellipse_preserves_output_coverage() {
        let make = |width, density| {
            EvaluatedShape::new(
                crate::ShapeGeometry::Ellipse {
                    width,
                    height: width,
                },
                Some(serde_json::from_value(red()).unwrap()),
                None,
                density,
            )
            .unwrap()
        };
        let small = make(2.0, 50.0);
        let large = make(100.0, 1.0);
        assert_eq!(small.size, large.size);
        let a = rasterize(&small).unwrap();
        let b = rasterize(&large).unwrap();
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(&b).all(|(a, b)| a.abs_diff(*b) <= 1));
    }

    #[test]
    fn independent_gradient_color_and_alpha_oracle() {
        let paint:Paint=serde_json::from_value(json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":10,"y":0},"stops":[{"offset":0,"color":{"r":0,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":1,"g":1,"b":1,"a":1}}]})).unwrap();
        assert_eq!(
            paint_at(&paint, VectorPoint { x: 5.0, y: 0.0 }),
            [0.5, 0.5, 0.5, 1.0]
        );
        assert!((srgb(0.5) - 0.735356983).abs() < 1e-9);
        assert_eq!(
            paint_at(&paint, VectorPoint { x: -20.0, y: 0.0 }),
            [0.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(paint_at(&paint, VectorPoint { x: 20.0, y: 0.0 }), [1.0; 4]);
        let transparent:Paint=serde_json::from_value(json!({"type":"radialGradient","center":{"x":0,"y":0},"radius":10,"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":1}}]})).unwrap();
        assert_eq!(
            paint_at(&transparent, VectorPoint { x: 5.0, y: 0.0 }),
            [0.0, 0.0, 0.5, 0.5]
        );
    }
    #[test]
    fn independent_fill_rules_and_move_only_coverage() {
        let commands = json!([{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":20,"y":0}},{"type":"lineTo","to":{"x":20,"y":20}},{"type":"lineTo","to":{"x":0,"y":20}},{"type":"close"},{"type":"moveTo","to":{"x":5,"y":5}},{"type":"lineTo","to":{"x":15,"y":5}},{"type":"lineTo","to":{"x":15,"y":15}},{"type":"lineTo","to":{"x":5,"y":15}}]);
        for rule in ["nonzero", "evenodd"] {
            let s = shape(
                json!({"type":"path","path":{"fillRule":rule,"commands":commands}}),
                red(),
                json!(null),
            );
            assert_eq!(
                pixel(&s, 10.0, 10.0)[3],
                if rule == "nonzero" { 255 } else { 0 }
            );
            assert_eq!(pixel(&s, 2.0, 2.0), [255, 0, 0, 255]);
        }
        let s = shape(
            json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}}]}}),
            red(),
            json!(null),
        );
        assert_eq!(pixel(&s, 0.0, 0.0), [0; 4]);
    }
    #[test]
    fn centered_stroke_caps_dash_phase_and_fill_order() {
        let line = json!({"type":"line","start":{"x":10,"y":10},"end":{"x":30,"y":10}});
        for cap in ["butt", "round", "square"] {
            let s = shape(
                line.clone(),
                json!(null),
                json!({"paint":red(),"width":6,"dash":[],"dashOffset":0,"lineCap":cap,"lineJoin":"miter","miterLimit":4}),
            );
            assert_eq!(pixel(&s, 20.0, 8.0)[3], 255);
            assert_eq!(pixel(&s, 8.0, 10.0)[3], if cap == "butt" { 0 } else { 255 });
        }
        for offset in [0, 4, -4] {
            let s = shape(
                line.clone(),
                json!(null),
                json!({"paint":red(),"width":2,"dash":[4,4],"dashOffset":offset,"lineCap":"butt","lineJoin":"bevel","miterLimit":1}),
            );
            assert_eq!(pixel(&s, 11.0, 10.0)[3], if offset == 0 { 255 } else { 0 });
        }
        let s = shape(
            json!({"type":"rectangle","width":20,"height":20}),
            red(),
            json!({"paint":{"type":"solid","color":{"r":0,"g":0,"b":1,"a":1}},"width":4,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"round","miterLimit":4}),
        );
        assert_eq!(pixel(&s, 1.0, 10.0), [0, 0, 255, 255]);
        assert_eq!(pixel(&s, 10.0, 10.0), [255, 0, 0, 255]);
    }
    #[test]
    fn independent_join_miter_fallback_and_dash_reset_oracles() {
        let geometry = json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":20}},{"type":"lineTo","to":{"x":10,"y":0}},{"type":"lineTo","to":{"x":20,"y":20}}]}});
        for (join, limit, tip) in [
            ("miter", 4, 255),
            ("miter", 1, 0),
            ("bevel", 4, 0),
            ("round", 4, 0),
        ] {
            let s = shape(
                geometry.clone(),
                json!(null),
                json!({"paint":red(),"width":6,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":join,"miterLimit":limit}),
            );
            assert_eq!(pixel(&s, 9.0, -5.0)[3], tip, "{join} {limit}");
        }
        let geometry = json!({"type":"path","path":{"fillRule":"nonzero","commands":[{"type":"moveTo","to":{"x":0,"y":0}},{"type":"lineTo","to":{"x":13,"y":0}},{"type":"moveTo","to":{"x":0,"y":10}},{"type":"lineTo","to":{"x":13,"y":10}}]}});
        let s = shape(
            geometry,
            json!(null),
            json!({"paint":red(),"width":2,"dash":[4,4],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4}),
        );
        assert_eq!(pixel(&s, 1.0, 0.0)[3], 255);
        assert_eq!(pixel(&s, 1.0, 10.0)[3], 255);
        assert_eq!(pixel(&s, 5.0, 10.0)[3], 0);
    }
}
