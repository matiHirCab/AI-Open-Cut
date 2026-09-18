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
        glyph.x += match style.alignment {
            EvaluatedTextAlignment::Left => 0.0,
            EvaluatedTextAlignment::Center => slack / 2.0,
            EvaluatedTextAlignment::Right => slack,
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
    let left = style.padding.left.saturating_add(extra_left);
    let top = style.padding.top.saturating_add(extra_top);
    let width = (shaped.width.ceil() as u32)
        .saturating_add(left)
        .saturating_add(style.padding.right)
        .saturating_add(extra_right)
        .saturating_add(2)
        .max(1);
    let height = (shaped.height.ceil() as u32)
        .saturating_add(top)
        .saturating_add(style.padding.bottom)
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
    let background = paint(&style.background_color, style.background_opacity)?;
    if let tiny_skia::Shader::SolidColor(color) = background.shader {
        pixmap.fill(color);
    }
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
