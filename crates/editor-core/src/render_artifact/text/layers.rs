//! Ordered glyph paints over the canonical positioned outlines.
use super::*;
use crate::TextPaintLayer;

pub(super) fn enabled(shaped: &ShapedText, style: &EvaluatedTextStyle) -> bool {
    style.paint_layers.is_some() || shaped.glyphs.iter().any(|g| g.paint_layers.is_some())
}

fn paints<'a>(
    glyph: &'a crate::fonts::shaping::ShapedGlyph,
    style: &'a EvaluatedTextStyle,
) -> Option<&'a [TextPaintLayer]> {
    glyph
        .paint_layers
        .as_deref()
        .or(style.paint_layers.as_deref())
}

fn groups(shaped: &ShapedText, style: &EvaluatedTextStyle) -> Vec<Vec<usize>> {
    let mut indices: Vec<_> = (0..shaped.glyphs.len()).collect();
    indices.sort_by_key(|i| shaped.glyphs[*i].cluster);
    let mut result: Vec<Vec<usize>> = vec![];
    for index in indices {
        let glyph = &shaped.glyphs[index];
        if let Some(group) = result.last_mut() {
            let previous = &shaped.glyphs[*group.last().unwrap()];
            let same_group = match (paints(previous, style), paints(glyph, style)) {
                (None, None) => true,
                (Some(a), Some(b)) => a == b && previous.face == glyph.face,
                _ => false,
            };
            if same_group {
                group.push(index);
                continue;
            }
        }
        result.push(vec![index]);
    }
    result
}

pub(super) fn margins(shaped: &ShapedText, style: &EvaluatedTextStyle) -> (u32, u32, u32, u32) {
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
    if !enabled(shaped, style) {
        return legacy();
    }
    let mut result = (0, 0, 0, 0);
    for glyph in &shaped.glyphs {
        let Some(layers) = paints(glyph, style) else {
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

pub(super) fn check_work(
    shaped: &ShapedText,
    style: &EvaluatedTextStyle,
    width: u32,
    height: u32,
) -> Result<(), CoreError> {
    if !enabled(shaped, style) {
        return Ok(());
    }
    let mut passes = 0u64;
    for group in groups(shaped, style) {
        passes += paints(&shaped.glyphs[group[0]], style).map_or(3, |p| {
            p.iter()
                .map(|layer| match layer {
                    TextPaintLayer::Shadow { blur_sigma_px, .. } if *blur_sigma_px > 0.0 => 3,
                    _ => 1,
                })
                .sum()
        });
    }
    if u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|p| p.checked_mul(passes))
        .is_none_or(|p| p > 268_435_456)
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "text paint pixel work exceeds limit",
        ));
    }
    Ok(())
}

fn blur(mask: &mut tiny_skia::Pixmap, sigma: f64) {
    if sigma == 0.0 {
        return;
    }
    let (w, h) = (mask.width() as usize, mask.height() as usize);
    let radius = (3.0 * sigma).ceil() as isize;
    let mut weights: Vec<f64> = (-radius..=radius)
        .map(|d| (-0.5 * (d as f64 / sigma).powi(2)).exp())
        .collect();
    let sum: f64 = weights.iter().sum();
    for value in &mut weights {
        *value /= sum;
    }
    let mut horizontal = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            horizontal[y * w + x] = weights
                .iter()
                .enumerate()
                .filter_map(|(i, weight)| {
                    let source = x as isize + i as isize - radius;
                    (source >= 0 && source < w as isize)
                        .then(|| f64::from(mask.pixels()[y * w + source as usize].alpha()) * weight)
                })
                .sum();
        }
    }
    for y in 0..h {
        for x in 0..w {
            let alpha: f64 = weights
                .iter()
                .enumerate()
                .filter_map(|(i, weight)| {
                    let source = y as isize + i as isize - radius;
                    (source >= 0 && source < h as isize)
                        .then(|| horizontal[source as usize * w + x] * weight)
                })
                .sum();
            let a = alpha.round().clamp(0.0, 255.0) as u8;
            mask.pixels_mut()[y * w + x] =
                tiny_skia::PremultipliedColorU8::from_rgba(a, a, a, a).unwrap();
        }
    }
}

fn union_glyph(
    mask: &mut tiny_skia::Pixmap,
    path: &tiny_skia::Path,
    transform: tiny_skia::Transform,
    white: &tiny_skia::Paint<'_>,
) -> Result<(), CoreError> {
    let path = path.clone().transform(transform).ok_or_else(|| {
        CoreError::new(
            ErrorCode::InvalidArgument,
            "invalid glyph coverage transform",
        )
    })?;
    let bounds = path.bounds();
    let left = bounds.left().floor().max(0.0) as u32;
    let top = bounds.top().floor().max(0.0) as u32;
    let right = bounds.right().ceil().min(mask.width() as f32).max(0.0) as u32;
    let bottom = bounds.bottom().ceil().min(mask.height() as f32).max(0.0) as u32;
    if right <= left || bottom <= top {
        return Ok(());
    }
    let width = right - left;
    let mut glyph = tiny_skia::Pixmap::new(width, bottom - top).ok_or_else(|| {
        CoreError::new(ErrorCode::InvalidArgument, "cannot allocate glyph coverage")
    })?;
    glyph.fill_path(
        &path,
        white,
        tiny_skia::FillRule::Winding,
        tiny_skia::Transform::from_translate(-(left as f32), -(top as f32)),
        None,
    );
    let stride = mask.width() as usize;
    for y in top..bottom {
        for x in left..right {
            let source = glyph.pixels()[((y - top) * width + x - left) as usize];
            let destination = &mut mask.pixels_mut()[y as usize * stride + x as usize];
            if source.alpha() > destination.alpha() {
                *destination = source;
            }
        }
    }
    Ok(())
}

pub(super) fn rasterize(
    shaped: &ShapedText,
    faces: &BTreeMap<String, Vec<u8>>,
    prepared: &PreparedText,
    style: &EvaluatedTextStyle,
) -> Result<Vec<u8>, CoreError> {
    let (w, h) = (prepared.layer_width, prepared.layer_height);
    check_work(shaped, style, w, h)?;
    let allocation_error = || {
        CoreError::new(
            ErrorCode::InvalidArgument,
            "cannot allocate text paint raster",
        )
    };
    let mut output = tiny_skia::Pixmap::new(w, h).ok_or_else(allocation_error)?;
    super::paint_background(&mut output, shaped, style)?;
    let mut outlines = Vec::with_capacity(shaped.glyphs.len());
    for glyph in &shaped.glyphs {
        let bytes = faces.get(&glyph.face).ok_or_else(|| {
            CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "shaped glyph face is missing",
            )
        })?;
        let face = crate::fonts::validate_face(bytes)?;
        let mut outline = Outline(tiny_skia::PathBuilder::new());
        face.outline_glyph(ttf_parser::GlyphId(glyph.id), &mut outline);
        let scale = shaped.font_size as f32 / f32::from(face.units_per_em());
        outlines.push((
            outline.0.finish(),
            tiny_skia::Transform::from_row(scale, 0.0, 0.0, -scale, glyph.x as f32, glyph.y as f32),
        ));
    }
    for mut group in groups(shaped, style) {
        let glyph = &shaped.glyphs[group[0]];
        let Some(layers) = paints(glyph, style) else {
            // Groups follow logical segment order; legacy glyphs retain visual draw order.
            group.sort_unstable();
            paint_legacy(
                &mut output,
                group.iter().map(|index| &shaped.glyphs[*index]),
                shaped.font_size,
                faces,
                style,
            )?;
            continue;
        };
        for layer in layers {
            let (color, opacity, dx, dy, sigma, width) = match layer {
                TextPaintLayer::Fill { color, opacity } => (color, *opacity, 0.0, 0.0, 0.0, None),
                TextPaintLayer::Stroke {
                    color,
                    opacity,
                    width_px,
                } => (color, *opacity, 0.0, 0.0, 0.0, Some(*width_px)),
                TextPaintLayer::Shadow {
                    color,
                    opacity,
                    offset_x_px,
                    offset_y_px,
                    blur_sigma_px,
                } => (
                    color,
                    *opacity,
                    *offset_x_px,
                    *offset_y_px,
                    *blur_sigma_px,
                    None,
                ),
            };
            if opacity == 0.0 {
                continue;
            }
            let mut mask = tiny_skia::Pixmap::new(w, h).ok_or_else(allocation_error)?;
            let white = paint("#ffffff", 1.0)?;
            for index in &group {
                let (Some(path), transform) = &outlines[*index] else {
                    continue;
                };
                let transform = transform.post_translate(dx as f32, dy as f32);
                if let Some(width) = width {
                    mask.stroke_path(
                        path,
                        &white,
                        &tiny_skia::Stroke {
                            width: width as f32 / transform.sx,
                            line_join: tiny_skia::LineJoin::Round,
                            ..Default::default()
                        },
                        transform,
                        None,
                    );
                } else if matches!(layer, TextPaintLayer::Shadow { .. }) {
                    union_glyph(&mut mask, path, transform, &white)?;
                } else {
                    mask.fill_path(path, &white, tiny_skia::FillRule::Winding, transform, None);
                }
            }
            blur(&mut mask, sigma);
            let tiny_skia::Shader::SolidColor(tint) = paint(color, opacity)?.shader else {
                unreachable!()
            };
            let rgb = tint.to_color_u8();
            for pixel in mask.pixels_mut() {
                let alpha = (f64::from(pixel.alpha()) * opacity).round() as u8;
                *pixel = tiny_skia::ColorU8::from_rgba(rgb.red(), rgb.green(), rgb.blue(), alpha)
                    .premultiply();
            }
            output.draw_pixmap(
                0,
                0,
                mask.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
    Ok(encode(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (
        ShapedText,
        BTreeMap<String, Vec<u8>>,
        crate::evaluated_scene::EvaluatedText,
    ) {
        let faces: BTreeMap<_, _> = crate::fonts::DEFAULT_FACES
            .iter()
            .map(|b| (crate::fonts::record(b).unwrap().sha256, b.to_vec()))
            .collect();
        let hashes = crate::fonts::DEFAULT_FACES.map(|b| crate::fonts::record(b).unwrap().sha256);
        let binding = crate::FontBinding {
            profile: crate::TEXT_LAYOUT_PROFILE.into(),
            regular: hashes[0].clone(),
            bold: hashes[1].clone(),
            italic: hashes[2].clone(),
            bold_italic: hashes[3].clone(),
            warnings: vec![],
        };
        let document = crate::RichTextDocument::plain("OO".into());
        let shaped =
            crate::fonts::shaping::shape(&document, &binding, &faces, 48, "#ffffff", None, 0)
                .unwrap();
        let text = crate::evaluated_scene::EvaluatedText {
            spans: None,
            font_binding: Some(binding),
            shaped: None,
            rich_runs: Some(document.runs),
            text: "OO".into(),
            font_size: 48,
            color: "#ffffff".into(),
            font_resource_id: None,
            style: crate::evaluated_scene::evaluate_text_style(&crate::TextStyle::default())
                .unwrap(),
        };
        (shaped, faces, text)
    }

    #[test]
    fn invisible_span_preserves_legacy_pixels_across_colors_faces_and_rtl() {
        for runs in [
            serde_json::json!([{"text":"AV "}]),
            serde_json::json!([{"text":"A","color":"#ff8800"},{"text":"V","bold":true},{"text":" "}]),
            serde_json::json!([{"text":"אב","color":"#ff8800"},{"text":"ג","italic":true},{"text":" "}]),
        ] {
            let (_, faces, mut text) = sample();
            text.font_size = 110;
            text.style.outline_width_px = 8;
            text.style.outline_color = "#ff0000".into();
            text.style.shadow.opacity = 0.5;
            text.style.shadow.offset_x = 3;
            text.style.shadow.offset_y = 3;
            let base: crate::RichTextDocument =
                serde_json::from_value(serde_json::json!({"runs":runs})).unwrap();
            let mut changed = base.clone();
            let end = base.grapheme_boundaries().len() - 1;
            changed.spans = Some(
                serde_json::from_value(
                    serde_json::json!([{"start":end-1,"end":end,"style":{"paintLayers":[]}}]),
                )
                .unwrap(),
            );
            assert_eq!(
                render_document(&base, &text, &faces),
                render_document(&changed, &text, &faces),
                "{}",
                base.text()
            );
        }
    }

    fn shape_document(
        document: &crate::RichTextDocument,
        text: &crate::evaluated_scene::EvaluatedText,
        faces: &BTreeMap<String, Vec<u8>>,
    ) -> ShapedText {
        crate::fonts::shaping::shape(
            document,
            text.font_binding.as_ref().unwrap(),
            faces,
            text.font_size,
            &text.color,
            None,
            0,
        )
        .unwrap()
    }

    fn render_document(
        document: &crate::RichTextDocument,
        text: &crate::evaluated_scene::EvaluatedText,
        faces: &BTreeMap<String, Vec<u8>>,
    ) -> (u32, u32, Vec<u8>) {
        let measured =
            super::super::measure(shape_document(document, text, faces), text, faces).unwrap();
        let (shaped, style) = measured.shaped.as_ref().unwrap();
        (
            measured.prepared.layer_width,
            measured.prepared.layer_height,
            super::super::rasterize(shaped, faces, &measured.prepared, style).unwrap(),
        )
    }

    #[test]
    fn explicit_stack_ignores_color_but_respects_stack_replacement() {
        let (_, faces, mut text) = sample();
        text.font_size = 110;
        text.style.paint_layers = Some(vec![
            TextPaintLayer::Stroke {
                color: "#ff0000".into(),
                opacity: 1.0,
                width_px: 60.0,
            },
            TextPaintLayer::Fill {
                color: "#ffffff".into(),
                opacity: 1.0,
            },
        ]);
        let base = crate::RichTextDocument::plain("AV".into());
        let mut changed = base.clone();
        changed.spans = Some(
            serde_json::from_value(
                serde_json::json!([{"start":1,"end":2,"style":{"color":"#00ff00"}}]),
            )
            .unwrap(),
        );
        assert_eq!(
            render_document(&base, &text, &faces),
            render_document(&changed, &text, &faces)
        );
        changed.spans.as_mut().unwrap()[0].style.paint_layers = Some(vec![TextPaintLayer::Fill {
            color: "#00ff00".into(),
            opacity: 1.0,
        }]);
        assert_ne!(
            render_document(&base, &text, &faces),
            render_document(&changed, &text, &faces)
        );
    }

    #[test]
    fn ignored_color_keeps_inclusive_paint_work_acceptance() {
        let (_, faces, mut text) = sample();
        text.style.paint_layers = Some(vec![
            TextPaintLayer::Fill {
                color: "#ffffff".into(),
                opacity: 1.0
            };
            16
        ]);
        let base = crate::RichTextDocument::plain("AV".into());
        let mut changed = base.clone();
        changed.spans = Some(
            serde_json::from_value(
                serde_json::json!([{"start":1,"end":2,"style":{"color":"#00ff00"}}]),
            )
            .unwrap(),
        );
        for document in [&base, &changed] {
            let shaped = shape_document(document, &text, &faces);
            check_work(&shaped, &text.style, 4096, 4096).unwrap();
            assert!(check_work(&shaped, &text.style, 4097, 4096).is_err());
        }
    }

    #[test]
    fn ordered_paints_empty_ink_and_legacy_fallback_are_distinct() {
        let (shaped, faces, mut text) = sample();
        let base = super::super::measure(shaped.clone(), &text, &faces).unwrap();
        let (positioned, style) = base.shaped.as_ref().unwrap();
        let legacy = super::super::rasterize(positioned, &faces, &base.prepared, style).unwrap();
        let fill = |color: &str| TextPaintLayer::Fill {
            color: color.into(),
            opacity: 1.0,
        };
        text.style.paint_layers = Some(vec![fill("#ff0000"), fill("#00ff00")]);
        let measured = super::super::measure(shaped.clone(), &text, &faces).unwrap();
        let (positioned, style) = measured.shaped.as_ref().unwrap();
        let green = rasterize(positioned, &faces, &measured.prepared, style).unwrap();
        assert_ne!(green, legacy);
        text.style.paint_layers.as_mut().unwrap().reverse();
        let red = rasterize(positioned, &faces, &measured.prepared, &text.style).unwrap();
        assert_ne!(red, green);
        text.style.paint_layers = Some(vec![]);
        let empty = rasterize(positioned, &faces, &measured.prepared, &text.style).unwrap();
        let header_end = empty.windows(7).position(|v| v == b"ENDHDR\n").unwrap() + 7;
        assert!(empty[header_end..].iter().all(|v| *v == 0));
        text.style.paint_layers = None;
        assert_eq!(
            super::super::rasterize(
                &base.shaped.as_ref().unwrap().0,
                &faces,
                &base.prepared,
                &text.style
            )
            .unwrap(),
            legacy
        );
    }

    #[test]
    fn fractional_effect_bounds_and_work_limits_are_conservative() {
        let (shaped, _, mut text) = sample();
        text.style.paint_layers = Some(vec![
            TextPaintLayer::Shadow {
                color: "#000000".into(),
                opacity: 0.5,
                offset_x_px: -2.5,
                offset_y_px: 4.25,
                blur_sigma_px: 2.0,
            },
            TextPaintLayer::Stroke {
                color: "#ffffff".into(),
                opacity: 1.0,
                width_px: 3.5,
            },
        ]);
        assert_eq!(margins(&shaped, &text.style), (9, 2, 4, 11));
        text.style.paint_layers = Some(vec![
            TextPaintLayer::Fill {
                color: "#ffffff".into(),
                opacity: 1.0
            };
            16
        ]);
        check_work(&shaped, &text.style, 4096, 4096).unwrap();
        assert!(check_work(&shaped, &text.style, 4097, 4096).is_err());
    }

    #[test]
    fn gaussian_impulse_has_normalized_symmetric_finite_support() {
        let mut mask = tiny_skia::Pixmap::new(9, 9).unwrap();
        mask.pixels_mut()[40] =
            tiny_skia::PremultipliedColorU8::from_rgba(255, 255, 255, 255).unwrap();
        let untouched = mask.clone();
        blur(&mut mask, 0.0);
        assert_eq!(mask, untouched);
        blur(&mut mask, f64::from_bits(1));
        assert_eq!(mask, untouched, "subnormal sigma must remain finite");
        blur(&mut mask, 1.0);
        // Independent Gaussian impulse values (normalized taps, radius 3).
        assert_eq!(mask.pixels()[40].alpha(), 41);
        assert_eq!(mask.pixels()[39].alpha(), 25);
        assert_eq!(mask.pixels()[31].alpha(), 25);
        assert_eq!(mask.pixels()[30].alpha(), 15);
        assert_eq!(mask.pixels()[36].alpha(), 0);
        assert_eq!(mask.pixels()[4].alpha(), 0);
        for y in 0..9 {
            for x in 0..9 {
                assert_eq!(
                    mask.pixels()[y * 9 + x],
                    mask.pixels()[(8 - y) * 9 + (8 - x)]
                );
            }
        }
    }

    #[test]
    fn overlapping_glyph_shadows_use_union_coverage() {
        let (shaped, faces, mut text) = sample();
        text.style.paint_layers = Some(vec![TextPaintLayer::Shadow {
            color: "#ff0000".into(),
            opacity: 0.5,
            offset_x_px: 0.5,
            offset_y_px: -0.5,
            blur_sigma_px: 1.0,
        }]);
        let measured = super::super::measure(shaped, &text, &faces).unwrap();
        let (mut single, style) = measured.shaped.clone().unwrap();
        single.glyphs.truncate(1);
        let mut duplicate = single.clone();
        duplicate.glyphs.push(single.glyphs[0].clone());
        assert_eq!(
            rasterize(&single, &faces, &measured.prepared, &style).unwrap(),
            rasterize(&duplicate, &faces, &measured.prepared, &style).unwrap()
        );
    }
}
