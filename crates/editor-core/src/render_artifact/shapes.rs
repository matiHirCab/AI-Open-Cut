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
fn coverage(shape: &EvaluatedShape, fill: &mut tiny_skia::Pixmap, stroke: &mut tiny_skia::Pixmap) {
    fill.fill(tiny_skia::Color::TRANSPARENT);
    stroke.fill(tiny_skia::Color::TRANSPARENT);
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
}

pub(super) fn rasterize(shape: &EvaluatedShape) -> Result<Vec<u8>, CoreError> {
    if let Some(children) = &shape.svg_children {
        let fail = || CoreError::new(ErrorCode::InvalidArgument, "SVG raster allocation failed");
        let (w, h) = shape.size;
        let pixels = (w as usize).checked_mul(h as usize).ok_or_else(fail)?;
        let mut accumulated = Vec::<[f64; 4]>::new();
        accumulated.try_reserve_exact(pixels).map_err(|_| fail())?;
        accumulated.resize(pixels, [0.; 4]);
        let coverage_buffer = || -> Result<tiny_skia::Pixmap, CoreError> {
            let mut data = Vec::new();
            let bytes = pixels.checked_mul(4).ok_or_else(fail)?;
            data.try_reserve_exact(bytes).map_err(|_| fail())?;
            data.resize(bytes, 0);
            tiny_skia::Pixmap::from_vec(data, tiny_skia::IntSize::from_wh(w, h).ok_or_else(fail)?)
                .ok_or_else(fail)
        };
        let mut fill = coverage_buffer()?;
        let mut stroke = coverage_buffer()?;
        for child in children {
            fill.fill(tiny_skia::Color::TRANSPARENT);
            stroke.fill(tiny_skia::Color::TRANSPARENT);
            let style = child.svg_raster_stroke()?;
            if let Some(path) = child.svg_raster_path()? {
                let mut paint = tiny_skia::Paint::default();
                paint.set_color_rgba8(255, 255, 255, 255);
                if child.fill.is_some() {
                    fill.fill_path(
                        &path,
                        &paint,
                        match child.fill_rule {
                            FillRule::Nonzero => tiny_skia::FillRule::Winding,
                            FillRule::Evenodd => tiny_skia::FillRule::EvenOdd,
                        },
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
                if let Some(style) = style {
                    stroke.stroke_path(
                        &path,
                        &paint,
                        &style,
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
            }
            for (i, dst) in accumulated.iter_mut().enumerate() {
                let p = VectorPoint {
                    x: (i % w as usize) as f64 / shape.density + 0.5 / shape.density,
                    y: (i / w as usize) as f64 / shape.density + 0.5 / shape.density,
                };
                let f = f64::from(fill.pixels()[i].alpha()) / 255.;
                let s = f64::from(stroke.pixels()[i].alpha()) / 255.;
                let a = child
                    .fill
                    .as_ref()
                    .map_or([0.; 4], |paint| paint_at(paint, p).map(|v| v * f));
                let b = child
                    .stroke
                    .as_ref()
                    .map_or([0.; 4], |style| paint_at(&style.paint, p).map(|v| v * s));
                let src: [f64; 4] = std::array::from_fn(|j| b[j] + a[j] * (1. - b[3]));
                for j in 0..4 {
                    dst[j] = src[j] + dst[j] * (1. - src[3]);
                }
            }
        }
        let mut output =
            format!("P7\nWIDTH {w}\nHEIGHT {h}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n")
                .into_bytes();
        let bytes = pixels.checked_mul(4).ok_or_else(fail)?;
        output.len().checked_add(bytes).ok_or_else(fail)?;
        output.try_reserve_exact(bytes).map_err(|_| fail())?;
        for color in accumulated {
            for v in &color[..3] {
                output.push(if color[3] == 0. {
                    0
                } else {
                    (srgb(v / color[3]).clamp(0., 1.) * 255.).round() as u8
                });
            }
            output.push((color[3].clamp(0., 1.) * 255.).round() as u8);
        }
        return Ok(output);
    }
    let fail = || CoreError::new(ErrorCode::InvalidArgument, "shape raster allocation failed");
    let (w, h) = shape.size;
    let mut fill = tiny_skia::Pixmap::new(w, h).ok_or_else(fail)?;
    let mut stroke = tiny_skia::Pixmap::new(w, h).ok_or_else(fail)?;
    coverage(shape, &mut fill, &mut stroke);
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
    #[test]
    fn grid_clipped_dashes_caps_paints_and_empty_coverage() {
        use serde_json::{Value, json};
        let catalog: Value = serde_json::from_str(include_str!(
            "../../../../contracts/procedural-grids-v1.json"
        ))
        .unwrap();
        let mut grid = catalog["valid"][0]["grid"].clone();
        grid["width"] = json!(30.5);
        grid["height"] = json!(10.5);
        grid["pattern"]["spacingX"] = json!(20);
        grid["pattern"]["spacingY"] = json!(20);
        grid["pattern"]["stroke"]["width"] = json!(2);
        grid["pattern"]["stroke"]["dash"] = json!([4, 4]);
        grid["pattern"]["stroke"]["paint"]["color"]["a"] = json!(1);
        let alpha = |grid: &Value, x: u32, y: u32, density: f64| {
            let shape = EvaluatedShape::grid_with_budget(
                serde_json::from_value(grid.clone()).unwrap(),
                density,
                65536,
            )
            .unwrap();
            let data = rasterize(&shape).unwrap();
            let pixels = &data[data.len() - (shape.size.0 * shape.size.1 * 4) as usize..];
            pixels[((y * shape.size.0 + x) * 4 + 3) as usize]
        };
        grid["pattern"]["stroke"]["lineCap"] = json!("butt");
        assert_eq!(alpha(&grid, 20, 4, 1.), 0);
        assert!(alpha(&grid, 20, 10, 1.).abs_diff(128) <= 1);
        grid["pattern"]["stroke"]["lineCap"] = json!("square");
        assert_eq!(alpha(&grid, 20, 4, 1.), 255);
        grid["pattern"]["stroke"]["lineCap"] = json!("round");
        let rounded = alpha(&grid, 20, 4, 1.);
        assert!(rounded > 0 && rounded < 255);
        grid["pattern"]["stroke"]["dashOffset"] = json!(3);
        grid["pattern"]["stroke"]["lineCap"] = json!("butt");
        assert_eq!(alpha(&grid, 20, 3, 1.), 0);
        grid["pattern"]["stroke"]["dash"] = json!([]);
        grid["pattern"]["stroke"]["paint"]["color"]["a"] = json!(0.5);
        assert!(alpha(&grid, 20, 10, 1.).abs_diff(64) <= 1);
        assert!(alpha(&grid, 40, 20, 2.).abs_diff(128) <= 1);
        grid["pattern"]["stroke"]["paint"] = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":40,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":0}},{"offset":1,"color":{"r":1,"g":0,"b":0,"a":1}}]});
        assert!(alpha(&grid, 20, 10, 1.).abs_diff(65) <= 1);
        grid["width"] = json!(0.5);
        grid["height"] = json!(0.5);
        grid["pattern"]["stroke"]["dash"] = json!([1, 100]);
        grid["pattern"]["stroke"]["dashOffset"] = json!(2);
        assert_eq!(alpha(&grid, 0, 0, 1.), 0);
        let mut dot = catalog["valid"][2]["grid"].clone();
        dot["width"] = json!(10.5);
        dot["height"] = json!(10.5);
        dot["pattern"]["paint"]["color"]["a"] = json!(1);
        assert!(alpha(&dot, 10, 10, 1.).abs_diff(64) <= 1);
    }
    #[test]
    fn grid_fractional_edges_preserve_partial_stroke_coverage() {
        use serde_json::json;
        for (width, height, x, y, expected) in [
            (10.5, 10., 10, 5, 128),
            (11., 10., 10, 5, 128),
            (10., 10.5, 5, 10, 128),
            (10.5, 10.5, 10, 10, 112),
        ] {
            let grid = json!({"width":width,"height":height,"pattern":{"type":"rectangular","spacingX":10,"spacingY":10,"stroke":{"width":1,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4,"paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}}}}});
            let shape =
                EvaluatedShape::grid_with_budget(serde_json::from_value(grid).unwrap(), 1., 65536)
                    .unwrap();
            let data = rasterize(&shape).unwrap();
            let pixels = &data[data.len() - (shape.size.0 * shape.size.1 * 4) as usize..];
            let alpha = pixels[((y * shape.size.0 + x) * 4 + 3) as usize];
            assert!(
                alpha.abs_diff(expected) <= 1,
                "{width}x{height} ({x},{y}): {alpha} != {expected}"
            );
        }
    }
    #[test]
    fn grid_intersections_local_paints_and_fractional_viewport() {
        use serde_json::{Value, json};
        let catalog: Value = serde_json::from_str(include_str!(
            "../../../../contracts/procedural-grids-v1.json"
        ))
        .unwrap();
        let mut grid = catalog["valid"][0]["grid"].clone();
        grid["pattern"]["stroke"]["width"] = json!(2);
        let shape = EvaluatedShape::grid_with_budget(
            serde_json::from_value(grid.clone()).unwrap(),
            1.,
            65536,
        )
        .unwrap();
        let data = rasterize(&shape).unwrap();
        let pixels = &data[data.len() - 20 * 20 * 4..];
        let pixel = |x: usize, y: usize| &pixels[(y * 20 + x) * 4..(y * 20 + x + 1) * 4];
        assert_eq!(pixel(9, 9), &[255, 0, 0, 191]);
        assert_eq!(pixel(5, 9), &[255, 0, 0, 128]);
        assert_eq!(pixel(5, 5), &[0, 0, 0, 0]);
        grid["pattern"]["stroke"]["paint"] = json!({"type":"linearGradient","start":{"x":0,"y":0},"end":{"x":20,"y":0},"stops":[{"offset":0,"color":{"r":1,"g":0,"b":0,"a":1}},{"offset":1,"color":{"r":0,"g":0,"b":1,"a":1}}]});
        let shape =
            EvaluatedShape::grid_with_budget(serde_json::from_value(grid).unwrap(), 1., 65536)
                .unwrap();
        let data = rasterize(&shape).unwrap();
        let pixels = &data[data.len() - 1600..];
        let x = 5;
        let y = 9;
        let p = &pixels[(y * 20 + x) * 4..(y * 20 + x + 1) * 4];
        let expected = [
            (srgb(1. - 5.5 / 20.) * 255.).round() as u8,
            0,
            (srgb(5.5 / 20.) * 255.).round() as u8,
            255,
        ];
        assert_eq!(p, expected);
        let mut dot = catalog["valid"][2]["grid"].clone();
        dot["width"] = json!(0.5);
        dot["height"] = json!(0.5);
        dot["pattern"]["paint"]["color"]["a"] = json!(1);
        let shape =
            EvaluatedShape::grid_with_budget(serde_json::from_value(dot).unwrap(), 1., 65536)
                .unwrap();
        let data = rasterize(&shape).unwrap();
        assert_eq!(&data[data.len() - 4..], &[255, 0, 0, 64]);
    }
    use super::*;
    use serde_json::json;
    #[test]
    fn svg_repeated_alpha_and_downscaled_viewport_regressions() {
        let source = format!(
            "<svg width=\"20\" height=\"20\">{}</svg>",
            "<rect width=\"20\" height=\"20\" fill=\"#ff000001\"/>".repeat(500)
        );
        let doc = crate::validation::svg::parse(&source).unwrap();
        let bytes = rasterize(&EvaluatedShape::new_svg(doc, 1.).unwrap()).unwrap();
        let expected = ((1. - (254_f64 / 255.).powi(500)) * 255.).round() as u8;
        assert_eq!(&bytes[bytes.len() - 4..], &[255, 0, 0, expected]);
        let doc = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 10000 10000\"><rect width=\"10000\" height=\"10000\" fill=\"#f00\"/></svg>").unwrap();
        let shape = EvaluatedShape::new_svg(doc, 1.).unwrap();
        assert_eq!(shape.size, (100, 100));
        let bytes = rasterize(&shape).unwrap();
        assert!(
            bytes[bytes.len() - 40000..]
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [255, 0, 0, 255])
        );
    }
    #[test]
    fn svg_float_fill_stroke_and_sibling_order() {
        let source = "<svg width=\"10\" height=\"10\"><rect width=\"10\" height=\"10\" fill=\"#f008\" stroke=\"#00f8\" stroke-width=\"4\"/><rect width=\"10\" height=\"10\" fill=\"#0f08\"/></svg>";
        let bytes = rasterize(
            &EvaluatedShape::new_svg(crate::validation::svg::parse(source).unwrap(), 1.).unwrap(),
        )
        .unwrap();
        let alpha = 136_f64 / 255.;
        let out_alpha = 1. - (1. - alpha).powi(3);
        // At (1,5), blue stroke covers red fill; green is the next sibling.
        let expected = [alpha * (1. - alpha).powi(2), alpha, alpha * (1. - alpha)];
        let start = bytes.len() - 400;
        let pixel = &bytes[start + (5 * 10 + 1) * 4..start + (5 * 10 + 2) * 4];
        for (i, linear) in expected.into_iter().enumerate() {
            let v = linear / out_alpha;
            let encoded = if v <= 0.0031308 {
                12.92 * v
            } else {
                1.055 * v.powf(1. / 2.4) - 0.055
            };
            assert!((i32::from(pixel[i]) - (encoded * 255.).round() as i32).abs() <= 1);
        }
        assert_eq!(pixel[3], (out_alpha * 255.).round() as u8);
    }
    #[test]
    fn svg_composes_in_linear_light_and_clips_before_item_opacity() {
        let doc=crate::validation::svg::parse("<svg width=\"40\" height=\"20\" viewBox=\"10 0 20 20\"><rect x=\"-100\" width=\"200\" height=\"20\" fill=\"#f00\"/><rect x=\"20\" width=\"10\" height=\"20\" fill=\"#00f8\"/></svg>").unwrap();
        let shape = EvaluatedShape::new_svg(doc, 1.).unwrap();
        assert_eq!(shape.size, (40, 20));
        let bytes = rasterize(&shape).unwrap();
        let start = bytes.len() - 40 * 20 * 4;
        let pixel = |x: usize| &bytes[start + (10 * 40 + x) * 4..start + (10 * 40 + x) * 4 + 4];
        assert_eq!(pixel(5), [255, 0, 0, 255]);
        let p = pixel(25);
        assert!((p[0] as i32 - 182).abs() <= 1);
        assert!((p[2] as i32 - 193).abs() <= 1);
        assert_eq!(p[3], 255);
        assert!(EvaluatedShape::new_svg(shape.svg_document.clone().unwrap(), 100.).is_ok());
        assert!(EvaluatedShape::new_svg(shape.svg_document.clone().unwrap(), 200.).is_err());
    }
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
    fn svg_precision_control_has_independent_band_pixels() {
        let document = crate::validation::svg::parse("<svg width=\"100\" height=\"100\" viewBox=\"0 0 .001 .001\"><polygon points=\"-.005,-.005 .005,.005 .005,.0051 -.005,-.0049\" fill=\"#f00\"/></svg>").unwrap();
        let shape = EvaluatedShape::new_svg(document, 1.).unwrap();
        assert_eq!(pixel(&shape, 50., 55.), [255, 0, 0, 255]);
        assert_eq!(pixel(&shape, 50., 45.), [0; 4]);
        assert_eq!(pixel(&shape, 50., 65.), [0; 4]);
    }
    #[test]
    fn svg_representable_crossing_and_empty_coverage() {
        for (source, expected) in [
            (
                "<rect x=\"-100\" y=\"-100\" width=\"200\" height=\"200\" fill=\"#f00\"/>",
                [255, 0, 0, 255],
            ),
            (
                "<rect x=\"100\" y=\"100\" width=\"10\" height=\"10\"/>",
                [0; 4],
            ),
            ("<path d=\"M0 0 H20\"/>", [0; 4]),
        ] {
            let doc = crate::validation::svg::parse(&format!(
                "<svg width=\"20\" height=\"20\">{source}</svg>"
            ))
            .unwrap();
            let shape = EvaluatedShape::new_svg(doc, 1.).unwrap();
            assert_eq!(pixel(&shape, 10., 10.), expected);
        }
        for cap in ["butt", "round", "square"] {
            for join in ["miter", "round", "bevel"] {
                let doc = crate::validation::svg::parse(&format!("<svg width=\"20\" height=\"20\"><path d=\"M-10 10 H30\" fill=\"none\" stroke=\"#f00\" stroke-width=\"4\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\" stroke-dasharray=\"100 1\"/></svg>")).unwrap();
                assert_eq!(
                    pixel(&EvaluatedShape::new_svg(doc, 1.).unwrap(), 10., 10.),
                    [255, 0, 0, 255]
                );
            }
        }
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
