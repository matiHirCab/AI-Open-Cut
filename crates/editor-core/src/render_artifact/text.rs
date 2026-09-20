//! Rasterization of already-positioned glyph outlines; no text shaping here.
use super::MeasuredText;
use crate::{
    CoreError, ErrorCode,
    evaluated_scene::{EvaluatedText, EvaluatedTextAlignment, EvaluatedTextStyle},
    fonts::shaping::ShapedText,
    render_plan::PreparedText,
};
use std::collections::BTreeMap;
mod layers;

pub(super) fn measure(
    mut shaped: ShapedText,
    text: &EvaluatedText,
    faces: &BTreeMap<String, Vec<u8>>,
) -> Result<MeasuredText, CoreError> {
    let style = &text.style;
    let (mut min_x, mut min_y, mut max_x, mut max_y) =
        (0.0f64, 0.0f64, shaped.width, shaped.height);
    for (glyph, line) in shaped.glyphs.iter_mut().zip(&shaped.glyph_lines) {
        let slack = shaped.width - shaped.line_widths[*line];
        glyph.x += if shaped.layout.is_some() {
            0.0
        } else {
            match style.alignment {
                EvaluatedTextAlignment::Left => 0.0,
                EvaluatedTextAlignment::Center => slack / 2.0,
                EvaluatedTextAlignment::Right => slack,
            }
        };
        let bytes = faces.get(&glyph.face).ok_or_else(|| {
            CoreError::new(ErrorCode::AssetIntegrityFailed, "glyph face is missing")
        })?;
        let face = crate::fonts::validate_face(bytes)?;
        if let Some(bounds) = face.glyph_bounding_box(ttf_parser::GlyphId(glyph.id)) {
            let scale = f64::from(shaped.font_size) / f64::from(face.units_per_em());
            min_x = min_x.min(glyph.x + f64::from(bounds.x_min) * scale);
            max_x = max_x.max(glyph.x + f64::from(bounds.x_max) * scale);
            min_y = min_y.min(glyph.y - f64::from(bounds.y_max) * scale);
            max_y = max_y.max(glyph.y - f64::from(bounds.y_min) * scale);
        }
    }
    shaped.width = max_x - min_x;
    shaped.height = max_y - min_y;
    let (extra_left, extra_top, extra_right, extra_bottom) = layers::margins(&shaped, style);
    let left = if shaped.layout.is_some() {
        extra_left
    } else {
        style.padding.left.saturating_add(extra_left)
    };
    let top = if shaped.layout.is_some() {
        extra_top
    } else {
        style.padding.top.saturating_add(extra_top)
    };
    let width = (shaped.width.ceil() as u32)
        .saturating_add(left)
        .saturating_add(if shaped.layout.is_some() {
            0
        } else {
            style.padding.right
        })
        .saturating_add(extra_right)
        .saturating_add(2)
        .max(1);
    let height = (shaped.height.ceil() as u32)
        .saturating_add(top)
        .saturating_add(if shaped.layout.is_some() {
            0
        } else {
            style.padding.bottom
        })
        .saturating_add(extra_bottom)
        .saturating_add(2)
        .max(1);
    if width > 16384 || height > 16384 || u64::from(width) * u64::from(height) > 16777216 {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "shaped text raster exceeds affine resource bounds",
        ));
    }
    layers::check_work(&shaped, style, width, height)?;
    for glyph in &mut shaped.glyphs {
        glyph.x += f64::from(left) - min_x;
        glyph.y += f64::from(top) - min_y;
    }
    if let Some(layout) = &mut shaped.layout {
        layout.background[0] += f64::from(left) - min_x;
        layout.background[1] += f64::from(top) - min_y;
    }
    Ok(MeasuredText {
        shaped: Some((shaped, style.clone())),
        prepared: PreparedText {
            rich_runs: None,
            file_path: Default::default(),
            font_path: None,
            layer_width: width,
            layer_height: height,
            canvas_width: width,
            canvas_height: height,
            text_x: left,
            text_y: top,
        },
        content: String::new(),
    })
}

fn paint_background(
    pixmap: &mut tiny_skia::Pixmap,
    shaped: &ShapedText,
    style: &EvaluatedTextStyle,
) -> Result<(), CoreError> {
    let background = paint(&style.background_color, style.background_opacity)?;
    let Some(layout) = &shaped.layout else {
        if let tiny_skia::Shader::SolidColor(color) = background.shader {
            pixmap.fill(color);
        }
        return Ok(());
    };
    let [x, y, w, h] = layout.background.map(|v| v as f32);
    let r = style
        .layout
        .as_ref()
        .map_or(0.0, |l| l.background_corner_radius_px as f32)
        .min(w / 2.0)
        .min(h / 2.0);
    let mut path = tiny_skia::PathBuilder::new();
    // Quarter-circle cubic approximation, shared by both glyph paint paths.
    let k = 0.552_284_8 * r;
    path.move_to(x + r, y);
    path.line_to(x + w - r, y);
    path.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    path.line_to(x + w, y + h - r);
    path.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    path.line_to(x + r, y + h);
    path.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    path.line_to(x, y + r);
    path.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    path.close();
    if let Some(path) = path.finish() {
        pixmap.fill_path(
            &path,
            &background,
            tiny_skia::FillRule::Winding,
            tiny_skia::Transform::identity(),
            None,
        );
    }
    Ok(())
}

struct Outline(tiny_skia::PathBuilder);
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.0.close();
    }
}

fn paint(color: &str, opacity: f64) -> Result<tiny_skia::Paint<'static>, CoreError> {
    let color = color.trim_start_matches('#');
    if color.len() != 6 {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "invalid glyph color",
        ));
    }
    let rgb = u32::from_str_radix(color, 16)
        .map_err(|_| CoreError::new(ErrorCode::InvalidArgument, "invalid glyph color"))?;
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(
        (rgb >> 16) as u8,
        (rgb >> 8) as u8,
        rgb as u8,
        (opacity * 255.0).round() as u8,
    );
    Ok(paint)
}

pub(super) fn rasterize(
    shaped: &ShapedText,
    faces: &BTreeMap<String, Vec<u8>>,
    prepared: &PreparedText,
    style: &EvaluatedTextStyle,
) -> Result<Vec<u8>, CoreError> {
    if layers::enabled(shaped, style) {
        return layers::rasterize(shaped, faces, prepared, style);
    }
    let (w, h) = (prepared.layer_width, prepared.layer_height);
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| {
        CoreError::new(ErrorCode::InvalidArgument, "cannot allocate glyph raster")
    })?;
    paint_background(&mut pixmap, shaped, style)?;
    paint_legacy(
        &mut pixmap,
        shaped.glyphs.iter(),
        shaped.font_size,
        faces,
        style,
    )?;
    Ok(encode(&pixmap))
}

fn paint_legacy<'a>(
    pixmap: &mut tiny_skia::Pixmap,
    glyphs: impl Iterator<Item = &'a crate::fonts::shaping::ShapedGlyph>,
    font_size: u32,
    faces: &BTreeMap<String, Vec<u8>>,
    style: &EvaluatedTextStyle,
) -> Result<(), CoreError> {
    let mut outlines = vec![];
    let mut outline_cache = BTreeMap::new();
    for glyph in glyphs {
        let bytes = faces.get(&glyph.face).ok_or_else(|| {
            CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "shaped glyph face is missing",
            )
        })?;
        let face = crate::fonts::validate_face(bytes)?;
        let outline = outline_cache
            .entry((&glyph.face, glyph.id))
            .or_insert_with(|| {
                let mut outline = Outline(tiny_skia::PathBuilder::new());
                face.outline_glyph(ttf_parser::GlyphId(glyph.id), &mut outline);
                outline.0.finish().map(std::sync::Arc::new)
            });
        if let Some(path) = outline {
            let scale = font_size as f32 / f32::from(face.units_per_em());
            outlines.push((
                std::sync::Arc::clone(path),
                tiny_skia::Transform::from_row(
                    scale,
                    0.0,
                    0.0,
                    -scale,
                    glyph.x as f32,
                    glyph.y as f32,
                ),
                glyph.color.as_str(),
            ));
        }
    }
    for (path, transform, _) in &outlines {
        if style.shadow.opacity > 0.0 {
            let transform = transform
                .post_translate(style.shadow.offset_x as f32, style.shadow.offset_y as f32);
            pixmap.fill_path(
                path,
                &paint(&style.shadow.color, style.shadow.opacity)?,
                tiny_skia::FillRule::Winding,
                transform,
                None,
            );
        }
    }
    for (path, transform, color) in outlines {
        if style.outline_width_px > 0 {
            let stroke = tiny_skia::Stroke {
                width: 2.0 * style.outline_width_px as f32 / transform.sx,
                ..Default::default()
            };
            pixmap.stroke_path(
                &path,
                &paint(&style.outline_color, 1.0)?,
                &stroke,
                transform,
                None,
            );
        }
        pixmap.fill_path(
            &path,
            &paint(color, 1.0)?,
            tiny_skia::FillRule::Winding,
            transform,
            None,
        );
    }
    Ok(())
}

fn encode(pixmap: &tiny_skia::Pixmap) -> Vec<u8> {
    let (w, h) = (pixmap.width(), pixmap.height());
    let mut output =
        format!("P7\nWIDTH {w}\nHEIGHT {h}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n")
            .into_bytes();
    for pixel in pixmap.pixels() {
        let c = pixel.demultiply();
        output.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    output
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn logical_box_mapping_ignores_paint_margins_and_fractional_raster_rounding() {
        use crate::evaluated_scene::text_layout::{resolve, tests::sample};
        use crate::evaluated_scene::{
            EvaluatedVisualSource, evaluate_layer_affine, evaluate_project,
        };
        for anchor in [
            "top_left",
            "top_center",
            "top_right",
            "center_left",
            "center",
            "center_right",
            "bottom_left",
            "bottom_center",
            "bottom_right",
        ] {
            for parent in [false, true] {
                for ink in ["MM", "j", "\n"] {
                    for effect in [false, true] {
                        for affine_transform in [false, true] {
                            let project: crate::Project = serde_json::from_value(serde_json::json!({
                        "schemaVersion":18,"id":"p","revision":0,"name":"P","createdAtMs":0,"updatedAtMs":0,
                        "settings":{"width":320,"height":240,"fps":30},"assets":[],"components":[],
                        "tracks":[{"id":"t","name":"T","trackType":"overlay","items":[{
                            "type":"text","id":"text","zIndex":0,"stackOrder":0,"keyframes":[],"text":"M","document":{"runs":[{"text":"M","color":"#ff0000"}]},
                            "fontSize":30,"color":"#ffffff","startMs":0,"durationMs":1000,
                            "style":{"anchor":anchor},"transform":{"positionX":50,"positionY":40,"scale":1,"opacity":1}
                        }]}]})).unwrap();
                            let mut layer = evaluate_project(&project, 320, 240, 30)
                                .unwrap()
                                .scene
                                .visual_layers
                                .remove(0);
                            let (mut text, faces) = sample(crate::TextLayout {
                                bounds: Some(crate::TextBounds {
                                    width_px: if ink == "\n" { None } else { Some(100.5) },
                                    height_px: Some(70.25),
                                }),
                                ..Default::default()
                            });
                            text.style.anchor = match &layer.source {
                                EvaluatedVisualSource::Text(t) => t.style.anchor,
                                _ => unreachable!(),
                            };
                            text.font_size = 30;
                            text.text = ink.into();
                            text.rich_runs = Some(vec![crate::RichTextRun {
                                text: ink.into(),
                                italic: Some(true),
                                bold: None,
                                color: None,
                            }]);
                            if effect {
                                text.style.shadow.opacity = 1.0;
                                text.style.shadow.offset_x = -20;
                                text.style.outline_width_px = 3;
                            }
                            let shaped = resolve(
                                &text,
                                &faces,
                                &mut crate::evaluated_scene::text_layout::GlyphBudget::default(),
                            )
                            .unwrap();
                            let measured = measure(shaped, &text, &faces).unwrap();
                            let shaped = measured.shaped.unwrap().0;
                            let [bx, by, w, h] = shaped.layout.as_ref().unwrap().background;
                            let ax = if anchor.ends_with("right") {
                                1.0
                            } else if anchor.ends_with("center") || anchor == "center" {
                                0.5
                            } else {
                                0.0
                            };
                            let ay = if anchor.starts_with("bottom") {
                                1.0
                            } else if anchor.starts_with("center") {
                                0.5
                            } else {
                                0.0
                            };
                            text.shaped = Some(shaped);
                            layer.source = EvaluatedVisualSource::Text(Box::new(text));
                            if affine_transform {
                                layer.transform2d = Some(serde_json::from_value(serde_json::json!({"position":{"x":50,"y":40,"unit":"pixels"},"anchor":{"x":ax,"y":ay},"scaleX":1.5,"scaleY":0.75,"rotationDeg":90,"skewXDeg":0,"skewYDeg":0,"opacity":1})).unwrap());
                            }
                            if parent {
                                layer.ancestors =
                                    Some(crate::evaluated_scene::EvaluatedAncestors {
                                        matrix: [0.0, 2.0, -1.0, 0.0, 220.0, 0.0],
                                        inverse: [0.0, -1.0, 0.5, 0.0, 0.0, 220.0],
                                        opacity: 1.0,
                                        clip: layer.span,
                                    });
                            }
                            let (expected_x, expected_y) =
                                if parent { (180.0, 100.0) } else { (50.0, 40.0) };
                            let affine = evaluate_layer_affine(
                                &layer,
                                (
                                    measured.prepared.layer_width,
                                    measured.prepared.layer_height,
                                ),
                                (320, 240),
                            )
                            .unwrap();
                            let [a, b, c, d, x, y] = affine.matrix;
                            // The authored anchor of the logical box maps to the authored position.
                            assert!(
                                (a * (bx + ax * w) + c * (by + ay * h) + x - expected_x).abs()
                                    < 1e-9,
                                "{anchor}, effect={effect}, affine={affine_transform}"
                            );
                            assert!(
                                (b * (bx + ax * w) + d * (by + ay * h) + y - expected_y).abs()
                                    < 1e-9
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn rounded_background_is_confined_to_padding_box_without_filling_effect_margins() {
        let mut pixmap = tiny_skia::Pixmap::new(30, 20).unwrap();
        let shaped = ShapedText {
            glyphs: vec![],
            glyph_lines: vec![],
            line_widths: vec![0.0],
            line_height: 10.0,
            width: 20.0,
            height: 10.0,
            font_size: 10,
            layout: Some(crate::fonts::shaping::ShapedLayout {
                background: [4.0, 4.0, 20.0, 10.0],
                content_width: 0.0,
                content_height: 10.0,
                overflow_x: false,
                overflow_y: false,
            }),
        };
        let style = crate::evaluated_scene::evaluate_text_style(&crate::TextStyle {
            background_color: "#ff0000".into(),
            background_opacity: 1.0,
            layout: Some(Box::new(crate::TextLayout {
                background_corner_radius_px: 100.0,
                ..Default::default()
            })),
            ..Default::default()
        })
        .unwrap();
        paint_background(&mut pixmap, &shaped, &style).unwrap();
        assert_eq!(pixmap.pixel(0, 0).unwrap().alpha(), 0);
        assert_eq!(pixmap.pixel(4, 4).unwrap().alpha(), 0);
        assert_eq!(pixmap.pixel(14, 8).unwrap().red(), 255);
        assert_eq!(pixmap.pixel(25, 8).unwrap().alpha(), 0);
    }
}
