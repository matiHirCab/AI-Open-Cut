//! Canonical positioned-glyph raster bounds shared by edit and render preflight.
use crate::{
    CoreError, ErrorCode, TextPaintLayer,
    evaluated_scene::{EvaluatedText, EvaluatedTextAlignment, EvaluatedTextStyle},
    fonts::shaping::ShapedText,
};
use std::collections::BTreeMap;
pub(crate) struct MeasuredTextBounds {
    pub shaped: ShapedText,
    pub width: u32,
    pub height: u32,
    pub left: u32,
    pub top: u32,
}
pub(crate) fn margins(shaped: &ShapedText, style: &EvaluatedTextStyle) -> (u32, u32, u32, u32) {
    let legacy = || {
        (
            style
                .outline_width_px
                .saturating_add(style.shadow.offset_x.min(0).unsigned_abs()),
            style
                .outline_width_px
                .saturating_add(style.shadow.offset_y.min(0).unsigned_abs()),
            style
                .outline_width_px
                .saturating_add(style.shadow.offset_x.max(0) as u32),
            style
                .outline_width_px
                .saturating_add(style.shadow.offset_y.max(0) as u32),
        )
    };
    if !(style.paint_layers.is_some() || shaped.glyphs.iter().any(|g| g.paint_layers.is_some())) {
        return legacy();
    }
    let mut result = (0, 0, 0, 0);
    for glyph in &shaped.glyphs {
        let Some(layers) = glyph
            .paint_layers
            .as_deref()
            .or(style.paint_layers.as_deref())
        else {
            let old = legacy();
            result = (
                result.0.max(old.0),
                result.1.max(old.1),
                result.2.max(old.2),
                result.3.max(old.3),
            );
            continue;
        };
        for layer in layers {
            let (x, y, radius) = match layer {
                TextPaintLayer::Fill { .. } => (0.0, 0.0, 0.0),
                TextPaintLayer::Stroke { width_px, .. } => (0.0, 0.0, width_px / 2.0),
                TextPaintLayer::Shadow {
                    offset_x_px,
                    offset_y_px,
                    blur_sigma_px,
                    ..
                } => (*offset_x_px, *offset_y_px, (3.0 * blur_sigma_px).ceil()),
            };
            result.0 = result.0.max((radius - x).max(0.0).ceil() as u32);
            result.1 = result.1.max((radius - y).max(0.0).ceil() as u32);
            result.2 = result.2.max((radius + x).max(0.0).ceil() as u32);
            result.3 = result.3.max((radius + y).max(0.0).ceil() as u32);
        }
    }
    result
}

pub(crate) fn measure(
    mut shaped: ShapedText,
    text: &EvaluatedText,
    faces: &BTreeMap<String, Vec<u8>>,
) -> Result<MeasuredTextBounds, CoreError> {
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
    let (extra_left, extra_top, extra_right, extra_bottom) = margins(&shaped, style);
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
    for glyph in &mut shaped.glyphs {
        glyph.x += f64::from(left) - min_x;
        glyph.y += f64::from(top) - min_y;
    }
    if let Some(layout) = &mut shaped.layout {
        layout.background[0] += f64::from(left) - min_x;
        layout.background[1] += f64::from(top) - min_y;
    }
    Ok(MeasuredTextBounds {
        shaped,
        width,
        height,
        left,
        top,
    })
}
