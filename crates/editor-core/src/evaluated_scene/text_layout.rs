//! Renderer-neutral advanced text geometry over prepared, pinned faces.
use super::{EvaluatedText, EvaluatedTextAlignment};
use crate::{
    CoreError, ErrorCode, TextFit, TextVerticalAlignment,
    fonts::shaping::{self, LayoutOptions, ShapedLayout, ShapedText},
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct GlyphBudget {
    used: usize,
    limit: usize,
}
impl Default for GlyphBudget {
    fn default() -> Self {
        Self {
            used: 0,
            limit: 16_777_216,
        }
    }
}
impl GlyphBudget {
    #[cfg(test)]
    pub(crate) fn with_limit(limit: usize) -> Self {
        Self { used: 0, limit }
    }
    fn charge(&mut self, glyphs: usize) -> Result<(), CoreError> {
        let next = self
            .used
            .checked_add(glyphs)
            .filter(|next| *next <= self.limit)
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::InvalidArgument,
                    "scene text fitting exceeds candidate glyph limit",
                )
            })?;
        self.used = next;
        Ok(())
    }
}

pub(crate) fn resolve(
    text: &EvaluatedText,
    faces: &BTreeMap<String, Vec<u8>>,
    work: &mut GlyphBudget,
) -> Result<ShapedText, CoreError> {
    let invalid = || {
        CoreError::new(
            ErrorCode::InvalidArgument,
            "advanced text requires pinned document and layout",
        )
    };
    let layout = text.style.layout.as_ref().ok_or_else(invalid)?;
    let binding = text.font_binding.as_ref().ok_or_else(invalid)?;
    let document = crate::RichTextDocument {
        runs: text.rich_runs.clone().ok_or_else(invalid)?,
        spans: text.spans.clone(),
    };
    let padding = &text.style.padding;
    let horizontal = f64::from(padding.left) + f64::from(padding.right);
    let vertical = f64::from(padding.top) + f64::from(padding.bottom);
    let width = layout
        .bounds
        .as_ref()
        .and_then(|b| b.width_px)
        .or(text.style.wrap_width_px.map(f64::from))
        .map(|w| w - horizontal);
    let height = layout
        .bounds
        .as_ref()
        .and_then(|b| b.height_px)
        .map(|h| h - vertical);
    let maximum = match layout.fit {
        TextFit::None | TextFit::Shrink => text.font_size,
        _ => 1000,
    };
    let minimum = if layout.fit == TextFit::None {
        maximum
    } else {
        1
    };
    for size in (minimum..=maximum).rev() {
        let mut shaped = shaping::shape_with_layout(
            &document,
            binding,
            faces,
            size,
            &text.color,
            text.style.line_spacing_px,
            LayoutOptions {
                width,
                wrap: layout.wrap,
                tracking: layout.tracking_px,
                line_height: layout.line_height_px,
                advanced: true,
            },
        )?;
        work.charge(shaped.glyphs.len())?;
        let overflow_x = width.is_some_and(|w| shaped.width > w);
        let overflow_y = height.is_some_and(|h| shaped.height > h);
        if size != minimum
            && layout.fit != TextFit::None
            && (overflow_x || (layout.fit != TextFit::FitWidth && overflow_y))
        {
            continue;
        }
        let content_width = shaped.width;
        let content_height = shaped.height;
        let box_width = width.unwrap_or(content_width);
        let box_height = height.unwrap_or(content_height);
        let y_slack = (box_height - content_height).max(0.0);
        let y = f64::from(padding.top)
            + match layout.vertical_alignment {
                TextVerticalAlignment::Top => 0.0,
                TextVerticalAlignment::Center => y_slack / 2.0,
                TextVerticalAlignment::Bottom => y_slack,
            };
        for (glyph, line) in shaped.glyphs.iter_mut().zip(&shaped.glyph_lines) {
            let slack = if overflow_x {
                0.0
            } else {
                (box_width - shaped.line_widths[*line]).max(0.0)
            };
            glyph.x += f64::from(padding.left)
                + match text.style.alignment {
                    EvaluatedTextAlignment::Left => 0.0,
                    EvaluatedTextAlignment::Center => slack / 2.0,
                    EvaluatedTextAlignment::Right => slack,
                };
            glyph.y += y;
        }
        shaped.width = box_width + horizontal;
        shaped.height = box_height + vertical;
        shaped.layout = Some(ShapedLayout {
            background: [0.0, 0.0, shaped.width, shaped.height],
            content_width,
            content_height,
            overflow_x,
            overflow_y,
        });
        return Ok(shaped);
    }
    Err(invalid())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn sample(layout: crate::TextLayout) -> (EvaluatedText, BTreeMap<String, Vec<u8>>) {
        let faces = crate::fonts::DEFAULT_FACES
            .iter()
            .map(|b| (crate::fonts::record(b).unwrap().sha256, b.to_vec()))
            .collect();
        let hashes = crate::fonts::DEFAULT_FACES.map(|b| crate::fonts::record(b).unwrap().sha256);
        let text = EvaluatedText {
            spans: None,
            shaped: None,
            rich_runs: Some(crate::RichTextDocument::plain("MM".into()).runs),
            text: "MM".into(),
            font_size: 48,
            color: "#ffffff".into(),
            font_resource_id: None,
            font_binding: Some(crate::FontBinding {
                profile: crate::TEXT_LAYOUT_PROFILE.into(),
                regular: hashes[0].clone(),
                bold: hashes[1].clone(),
                italic: hashes[2].clone(),
                bold_italic: hashes[3].clone(),
                warnings: vec![],
            }),
            style: super::super::evaluate_text_style(&crate::TextStyle {
                layout: Some(Box::new(layout)),
                ..Default::default()
            })
            .unwrap(),
        };
        (text, faces)
    }
    #[test]
    fn candidate_budget_is_inclusive_checked_and_atomic() {
        let mut budget = GlyphBudget::default();
        budget.charge(16_777_215).unwrap();
        budget.charge(1).unwrap();
        budget.charge(0).unwrap();
        assert_eq!(
            budget.charge(1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(budget.used, 16_777_216);
        assert!(budget.charge(usize::MAX).is_err());
        assert_eq!(budget.used, 16_777_216);
    }
    #[test]
    fn newline_only_candidates_retain_zero_glyph_accounting() {
        let (mut text, faces) = sample(crate::TextLayout {
            line_height_px: Some(1.0),
            bounds: Some(crate::TextBounds {
                width_px: Some(1.0),
                height_px: Some(1.0),
            }),
            fit: TextFit::Shrink,
            ..Default::default()
        });
        text.font_size = 1000;
        text.text = "\n\n".into();
        text.rich_runs = Some(crate::RichTextDocument::plain(text.text.clone()).runs);
        let mut budget = GlyphBudget::with_limit(0);
        let shaped = resolve(&text, &faces, &mut budget).unwrap();
        assert_eq!(shaped.font_size, 1);
        assert_eq!(shaped.line_widths.len(), 3);
        assert_eq!(budget.used, 0);
    }
    #[test]
    fn logical_widths_match_pinned_metrics_for_bidi_mixed_faces_and_word_breaks() {
        let (mut text, faces) = sample(crate::TextLayout {
            tracking_px: 0.1,
            ..Default::default()
        });
        text.font_size = 30;
        for value in ["MMMMM", "MM אב MM"] {
            text.text = value.into();
            text.rich_runs = Some(vec![crate::RichTextRun {
                text: value.into(),
                bold: Some(true),
                italic: None,
                color: None,
            }]);
            let face =
                crate::fonts::validate_face(&faces[&text.font_binding.as_ref().unwrap().bold])
                    .unwrap();
            let mut expected = 0.0;
            for (index, ch) in value.chars().enumerate() {
                if index > 0 {
                    expected += 0.1;
                }
                expected += f64::from(
                    face.glyph_hor_advance(face.glyph_index(ch).unwrap())
                        .unwrap(),
                ) * 30.0
                    / f64::from(face.units_per_em());
            }
            for wrap in [crate::TextWrap::Word, crate::TextWrap::Cluster] {
                let layout = text.style.layout.as_mut().unwrap();
                layout.wrap = wrap;
                layout.bounds = Some(crate::TextBounds {
                    width_px: Some(expected),
                    height_px: None,
                });
                let shaped = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
                assert_eq!(shaped.line_widths, vec![expected]);
                text.style
                    .layout
                    .as_mut()
                    .unwrap()
                    .bounds
                    .as_mut()
                    .unwrap()
                    .width_px = Some(expected.next_down());
                assert!(
                    resolve(&text, &faces, &mut GlyphBudget::default())
                        .unwrap()
                        .line_widths
                        .len()
                        > 1
                );
            }
        }
        text.text = "MM MM MM".into();
        text.rich_runs = Some(crate::RichTextDocument::plain(text.text.clone()).runs);
        text.style.layout.as_mut().unwrap().wrap = crate::TextWrap::Word;
        let m = 1767.0 / 2048.0 * 30.0;
        let space = 651.0 / 2048.0 * 30.0;
        let mut prefix = 0.0;
        for (i, advance) in [m, m, space, m, m, space].into_iter().enumerate() {
            if i > 0 {
                prefix += 0.1;
            }
            prefix += advance;
        }
        text.style
            .layout
            .as_mut()
            .unwrap()
            .bounds
            .as_mut()
            .unwrap()
            .width_px = Some(prefix + m + 0.2);
        let shaped = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        assert_eq!(shaped.line_widths, vec![prefix, m + 0.1 + m]);
        // A mixed-face line retains each face's advance without changing the summation order.
        text.text = "MMMM".into();
        text.rich_runs = Some(vec![
            crate::RichTextRun {
                text: "MM".into(),
                bold: None,
                italic: None,
                color: None,
            },
            crate::RichTextRun {
                text: "MM".into(),
                bold: Some(true),
                italic: None,
                color: None,
            },
        ]);
        text.style.layout.as_mut().unwrap().bounds = None;
        let measured = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        text.style.layout.as_mut().unwrap().bounds = Some(crate::TextBounds {
            width_px: Some(measured.width),
            height_px: Some(measured.height),
        });
        text.style.layout.as_mut().unwrap().fit = TextFit::Shrink;
        let bounded = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        assert_eq!(bounded.font_size, 30);
        assert_eq!(bounded.line_widths, measured.line_widths);
    }
    #[test]
    fn fractional_reported_bounds_retain_one_line_and_authored_size() {
        let (mut text, faces) = sample(crate::TextLayout {
            tracking_px: 0.1,
            ..Default::default()
        });
        text.font_size = 30;
        text.text = "MMMMM".into();
        text.rich_runs = Some(crate::RichTextDocument::plain(text.text.clone()).runs);
        let measured = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        for wrap in [crate::TextWrap::Word, crate::TextWrap::Cluster] {
            for fit in [TextFit::None, TextFit::Shrink] {
                let layout = text.style.layout.as_mut().unwrap();
                layout.bounds = Some(crate::TextBounds {
                    width_px: Some(measured.width),
                    height_px: Some(measured.height),
                });
                layout.wrap = wrap;
                layout.fit = fit;
                let bounded = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
                assert_eq!(bounded.line_widths.len(), 1);
                assert_eq!(bounded.font_size, 30);
                assert_eq!(bounded.layout.unwrap().content_width, measured.width);
            }
        }
    }
    #[test]
    fn fit_modes_choose_largest_exact_integer_and_preserve_authored_values() {
        let (mut text, faces) = sample(crate::TextLayout::default());
        // DejaVu Sans M advance is 1767 / 2048 em. This bound fits exactly size 32.
        let exact_width = 2.0 * 1767.0 / 2048.0 * 32.0;
        for fit in [
            TextFit::None,
            TextFit::Shrink,
            TextFit::FitWidth,
            TextFit::FitBox,
        ] {
            text.style.layout = Some(Box::new(crate::TextLayout {
                bounds: Some(crate::TextBounds {
                    width_px: Some(exact_width),
                    height_px: Some(100.0),
                }),
                fit,
                wrap: crate::TextWrap::None,
                ..Default::default()
            }));
            let value = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
            assert_eq!(value.font_size, if fit == TextFit::None { 48 } else { 32 });
            assert_eq!(text.font_size, 48);
            assert_eq!(value.layout.unwrap().overflow_x, fit == TextFit::None);
        }
        text.font_size = 12;
        assert_eq!(
            resolve(&text, &faces, &mut GlyphBudget::default())
                .unwrap()
                .font_size,
            32
        );
        text.style.layout.as_mut().unwrap().fit = TextFit::Shrink;
        assert_eq!(
            resolve(&text, &faces, &mut GlyphBudget::default())
                .unwrap()
                .font_size,
            12
        );
    }
    #[test]
    fn unavoidable_overflow_and_global_work_fail_before_rasterization() {
        let (text, faces) = sample(crate::TextLayout {
            tracking_px: 10.0,
            line_height_px: Some(30.0),
            bounds: Some(crate::TextBounds {
                width_px: Some(1.0),
                height_px: Some(1.0),
            }),
            wrap: crate::TextWrap::None,
            fit: TextFit::Shrink,
            ..Default::default()
        });
        let resolved = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        assert_eq!(resolved.font_size, 1);
        let layout = resolved.layout.unwrap();
        assert!(layout.overflow_x && layout.overflow_y);
        assert_eq!(resolved.glyphs.len(), 2);
        assert_eq!(
            resolve(
                &text,
                &faces,
                &mut GlyphBudget {
                    used: 16_777_216,
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidArgument
        );
    }
    #[test]
    fn fractional_box_alignment_uses_padding_and_line_boxes() {
        let (mut text, faces) = sample(crate::TextLayout {
            line_height_px: Some(60.5),
            bounds: Some(crate::TextBounds {
                width_px: Some(200.5),
                height_px: Some(100.5),
            }),
            vertical_alignment: TextVerticalAlignment::Center,
            ..Default::default()
        });
        text.style.padding.left = 10;
        text.style.padding.right = 20;
        text.style.padding.top = 3;
        text.style.padding.bottom = 7;
        text.style.alignment = EvaluatedTextAlignment::Right;
        let shaped = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
        let content = 2.0 * 1767.0 / 2048.0 * 48.0;
        assert_eq!(shaped.glyphs[0].x, 200.5 - 20.0 - content);
        // DejaVu Sans ascender = 1901 font units; center slack = 15 px.
        assert_eq!(shaped.glyphs[0].y, 1901.0 / 2048.0 * 48.0 + 3.0 + 15.0);
        assert_eq!(shaped.layout.unwrap().background, [0.0, 0.0, 200.5, 100.5]);
        for (alignment, factor) in [
            (EvaluatedTextAlignment::Left, 0.0),
            (EvaluatedTextAlignment::Center, 0.5),
            (EvaluatedTextAlignment::Right, 1.0),
        ] {
            for (vertical, y_factor) in [
                (TextVerticalAlignment::Top, 0.0),
                (TextVerticalAlignment::Center, 0.5),
                (TextVerticalAlignment::Bottom, 1.0),
            ] {
                text.style.alignment = alignment;
                text.style.layout.as_mut().unwrap().vertical_alignment = vertical;
                let shaped = resolve(&text, &faces, &mut GlyphBudget::default()).unwrap();
                assert_eq!(shaped.glyphs[0].x, 10.0 + (170.5 - content) * factor);
                assert_eq!(
                    shaped.glyphs[0].y,
                    1901.0 / 2048.0 * 48.0 + 3.0 + 30.0 * y_factor
                );
            }
        }
    }
}
