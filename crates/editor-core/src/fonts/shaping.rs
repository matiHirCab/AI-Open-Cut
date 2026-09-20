//! The opencut-text-v2 shaping profile. No platform font or locale access.
use super::{invalid, validate_face};
use crate::{
    CoreError, FontBinding, MAX_SHAPED_GLYPHS, MAX_TEXT_LINES, RichTextDocument,
    TEXT_LAYOUT_PROFILE,
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_bidi::{BidiInfo, Level};
use unicode_script::{Script, UnicodeScript};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedGlyph {
    pub(crate) paint_layers: Option<Vec<crate::TextPaintLayer>>,
    pub(crate) face: String,
    pub(crate) id: u16,
    pub(crate) cluster: u32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) advance: f64,
    pub(crate) color: String,
}

#[derive(Clone, PartialEq)]
pub(crate) struct ShapedText {
    pub(crate) layout: Option<ShapedLayout>,
    pub(crate) glyphs: Vec<ShapedGlyph>,
    pub(crate) line_widths: Vec<f64>,
    pub(crate) glyph_lines: Vec<usize>,
    pub(crate) line_height: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) font_size: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedLayout {
    pub(crate) background: [f64; 4],
    pub(crate) content_width: f64,
    pub(crate) content_height: f64,
    pub(crate) overflow_x: bool,
    pub(crate) overflow_y: bool,
}
impl std::fmt::Debug for ShapedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_struct("ShapedText");
        value
            .field("glyphs", &self.glyphs)
            .field("line_widths", &self.line_widths)
            .field("glyph_lines", &self.glyph_lines)
            .field("line_height", &self.line_height)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("font_size", &self.font_size);
        if let Some(layout) = &self.layout {
            value.field("layout", layout);
        }
        value.finish()
    }
}
/// Logical-order width arithmetic for the advanced profile only.
#[derive(Clone, Copy, Default)]
struct LayoutWidth {
    width: f64,
    clusters: usize,
}
impl LayoutWidth {
    fn appended(self, advance: f64, tracking: f64) -> Self {
        let mut width = self.width;
        if self.clusters > 0 {
            width += tracking;
        }
        width += advance;
        Self {
            width,
            clusters: self.clusters + 1,
        }
    }
}
#[derive(Clone, Copy, Default)]
pub(crate) struct LayoutOptions {
    pub(crate) width: Option<f64>,
    pub(crate) wrap: crate::TextWrap,
    pub(crate) tracking: f64,
    pub(crate) line_height: Option<f64>,
    pub(crate) advanced: bool,
}

struct Cluster {
    start: usize,
    end: usize,
    glyphs: Vec<ShapedGlyph>,
    width: f64,
    level: Level,
}

struct Segment {
    start: usize,
    end: usize,
    face: String,
    script: Script,
    level: Level,
}

fn check_work(glyphs: usize, lines: usize) -> Result<(), CoreError> {
    if glyphs > MAX_SHAPED_GLYPHS || lines > MAX_TEXT_LINES {
        return Err(invalid("text shaping exceeds glyph or line limit"));
    }
    Ok(())
}

pub(crate) fn shape(
    document: &RichTextDocument,
    binding: &FontBinding,
    faces: &BTreeMap<String, Vec<u8>>,
    font_size: u32,
    color: &str,
    wrap_width: Option<u32>,
    line_spacing: i32,
) -> Result<ShapedText, CoreError> {
    shape_with_layout(
        document,
        binding,
        faces,
        font_size,
        color,
        line_spacing,
        LayoutOptions {
            width: wrap_width.map(f64::from),
            ..Default::default()
        },
    )
}
pub(crate) fn shape_with_layout(
    document: &RichTextDocument,
    binding: &FontBinding,
    faces: &BTreeMap<String, Vec<u8>>,
    font_size: u32,
    color: &str,
    line_spacing: i32,
    options: LayoutOptions,
) -> Result<ShapedText, CoreError> {
    if binding.profile != TEXT_LAYOUT_PROFILE
        || font_size == 0
        || font_size > 1000
        || options.width.is_some_and(|w| !w.is_finite() || w <= 0.0)
    {
        return Err(invalid("invalid text shaping profile or dimensions"));
    }
    let text: String = document.runs.iter().map(|r| r.text.as_str()).collect();
    if text.is_empty() || text.len() > 4096 || document.runs.is_empty() || document.runs.len() > 256
    {
        return Err(invalid("text shaping exceeds document bounds"));
    }
    let mut styles = vec![];
    let mut offset = 0;
    let effective_runs = document.effective_runs()?;
    let paint_ranges = document.paint_ranges()?;
    for run in &effective_runs {
        styles.push((
            offset..offset + run.text.len(),
            binding.face_hash(run.bold.unwrap_or(false), run.italic.unwrap_or(false)),
            run.color.as_deref().unwrap_or(color),
        ));
        offset += run.text.len();
    }
    let mut metrics = BTreeMap::new();
    let mut ascent = 0.0f64;
    let mut descent = 0.0f64;
    let mut line_height = 0.0f64;
    for hash in binding.hashes() {
        if options.advanced && !styles.iter().any(|(_, selected, _)| *selected == hash) {
            continue;
        }
        let face = validate_face(
            faces
                .get(hash)
                .ok_or_else(|| invalid("prepared font face is missing"))?,
        )?;
        let scale = f64::from(font_size) / f64::from(face.units_per_em());
        let metric_ascent = f64::from(face.ascender()) * scale;
        let metric_height = f64::from(
            i32::from(face.ascender()) - i32::from(face.descender()) + i32::from(face.line_gap()),
        ) * scale;
        metrics.insert(hash, (metric_ascent, metric_height));
        ascent = ascent.max(metric_ascent);
        descent = descent.max(-f64::from(face.descender()) * scale);
        line_height = line_height.max(
            f64::from(
                i32::from(face.ascender()) - i32::from(face.descender())
                    + i32::from(face.line_gap()),
            ) * scale,
        );
    }
    let line_step = options
        .line_height
        .unwrap_or((line_height + f64::from(line_spacing)).max(1.0));
    let mut result = ShapedText {
        layout: None,
        glyphs: vec![],
        line_widths: vec![],
        glyph_lines: vec![],
        line_height: line_step,
        width: 0.0,
        height: 0.0,
        font_size,
    };
    // Resolve actual Unicode paragraphs once. A line separator forces a line
    // but retains its paragraph's base direction (unlike a paragraph separator).
    let bidi = BidiInfo::new(&text, None);
    let mut hard_lines = vec![];
    let mut line_start = 0;
    let mut trailing_separator = false;
    for (end, opportunity) in unicode_linebreak::linebreaks(&text) {
        if opportunity != unicode_linebreak::BreakOpportunity::Mandatory {
            continue;
        }
        let separator_bytes = if text[..end].ends_with("\r\n") {
            2
        } else {
            text[..end]
                .chars()
                .next_back()
                .filter(|c| {
                    matches!(
                        c,
                        '\r' | '\n' | '\u{85}' | '\u{b}' | '\u{c}' | '\u{2028}' | '\u{2029}'
                    )
                })
                .map_or(0, char::len_utf8)
        };
        hard_lines.push(line_start..end - separator_bytes);
        line_start = end;
        trailing_separator = separator_bytes > 0;
    }
    if trailing_separator {
        hard_lines.push(text.len()..text.len());
    }
    let mut line_top = 0.0;
    let mut first_ascent = None;
    for hard_line in hard_lines {
        let paragraph_offset = hard_line.start;
        let paragraph = &text[hard_line];
        let mut segments: Vec<Segment> = vec![];
        let mut preceding_script = Script::Common;
        for (local, ch) in paragraph.char_indices() {
            let global = paragraph_offset + local;
            let (_, face, _) = styles
                .iter()
                .find(|(r, _, _)| r.contains(&global))
                .ok_or_else(|| invalid("text style coverage is incomplete"))?;
            let mut script = ch.script();
            if matches!(script, Script::Common | Script::Inherited | Script::Unknown) {
                script = if !matches!(
                    preceding_script,
                    Script::Common | Script::Inherited | Script::Unknown
                ) {
                    preceding_script
                } else {
                    paragraph[local..]
                        .chars()
                        .map(|c| c.script())
                        .find(|s| {
                            !matches!(s, Script::Common | Script::Inherited | Script::Unknown)
                        })
                        .unwrap_or(Script::Latin)
                };
            }
            preceding_script = script;
            let level = bidi.levels[global];
            if let Some(segment) = segments
                .last_mut()
                .filter(|s| s.face == *face && s.script == script && s.level == level)
            {
                segment.end = local + ch.len_utf8();
            } else {
                segments.push(Segment {
                    start: local,
                    end: local + ch.len_utf8(),
                    face: (*face).into(),
                    script,
                    level,
                });
            }
        }
        let mut clusters = BTreeMap::<usize, Cluster>::new();
        for segment in segments {
            let bytes = faces
                .get(&segment.face)
                .ok_or_else(|| invalid("prepared face missing"))?;
            let face = rustybuzz::Face::from_slice(bytes, 0)
                .ok_or_else(|| invalid("invalid shaping face"))?;
            let scale = f64::from(font_size) / f64::from(face.units_per_em());
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            for (index, ch) in paragraph[segment.start..segment.end].char_indices() {
                buffer.add(ch, (paragraph_offset + segment.start + index) as u32);
            }
            buffer.set_direction(if segment.level.is_rtl() {
                rustybuzz::Direction::RightToLeft
            } else {
                rustybuzz::Direction::LeftToRight
            });
            buffer.set_script(
                rustybuzz::Script::from_iso15924_tag(ttf_parser::Tag(
                    segment.script.as_iso15924_tag(),
                ))
                .ok_or_else(|| invalid("unsupported script tag"))?,
            );
            buffer.set_language(
                "und"
                    .parse()
                    .map_err(|_| invalid("invalid profile language"))?,
            );
            buffer.set_cluster_level(rustybuzz::BufferClusterLevel::MonotoneGraphemes);
            let shaped = rustybuzz::shape(&face, &[], buffer);
            check_work(shaped.len(), 0)?;
            for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
                let start = info.cluster as usize;
                let cluster = clusters.entry(start).or_insert_with(|| Cluster {
                    start,
                    end: paragraph_offset + segment.end,
                    glyphs: vec![],
                    width: 0.0,
                    level: segment.level,
                });
                let (_, _, paint) = styles
                    .iter()
                    .find(|(r, _, _)| r.contains(&start))
                    .ok_or_else(|| invalid("glyph cluster outside document"))?;
                let advance = f64::from(position.x_advance) * scale;
                cluster.glyphs.push(ShapedGlyph {
                    paint_layers: paint_ranges
                        .iter()
                        .find(|(range, _)| range.contains(&start))
                        .map(|(_, paints)| paints.clone()),
                    face: segment.face.clone(),
                    id: u16::try_from(info.glyph_id).map_err(|_| invalid("glyph id overflow"))?,
                    cluster: info.cluster,
                    x: cluster.width + f64::from(position.x_offset) * scale,
                    y: -f64::from(position.y_offset) * scale,
                    advance,
                    color: (*paint).into(),
                });
                cluster.width += advance;
            }
        }
        let mut clusters: Vec<_> = clusters.into_values().collect();
        for index in 0..clusters.len().saturating_sub(1) {
            clusters[index].end = clusters[index].end.min(clusters[index + 1].start);
        }
        let opportunities: BTreeSet<_> = unicode_linebreak::linebreaks(paragraph)
            .map(|(i, _)| paragraph_offset + i)
            .collect();
        let mut start = 0;
        loop {
            let mut end = start;
            let mut width = 0.0;
            let mut logical = LayoutWidth::default();
            let mut last_break = None;
            while end < clusters.len() {
                let candidate = logical.appended(clusters[end].width, options.tracking);
                if end > start
                    && options.wrap != crate::TextWrap::None
                    && options.width.is_some_and(|w| {
                        if options.advanced {
                            candidate.width > w
                        } else {
                            width + options.tracking + clusters[end].width > w
                        }
                    })
                {
                    break;
                }
                width += clusters[end].width + if end > start { options.tracking } else { 0.0 };
                logical = candidate;
                end += 1;
                if opportunities.contains(&clusters[end - 1].end) {
                    last_break = Some((end, logical));
                }
            }
            if end < clusters.len()
                && options.wrap == crate::TextWrap::Word
                && let Some(last) = last_break
            {
                (end, logical) = last;
            }
            let line = result.line_widths.len();
            check_work(result.glyphs.len(), line + 1)?;
            let (baseline, advance_y) = if options.advanced {
                let mut selected: BTreeSet<&str> = clusters[start..end]
                    .iter()
                    .flat_map(|c| c.glyphs.iter().map(|g| g.face.as_str()))
                    .collect();
                if selected.is_empty() {
                    let source = paragraph_offset.min(text.len().saturating_sub(1));
                    if let Some((_, face, _)) = styles.iter().find(|(r, _, _)| r.contains(&source))
                    {
                        selected.insert(face);
                    }
                }
                let (a, h) = selected
                    .into_iter()
                    .filter_map(|face| metrics.get(face))
                    .fold((0.0f64, 0.0f64), |(a, h), (ma, mh)| {
                        (a.max(*ma), h.max(*mh))
                    });
                let baseline_ascent = *first_ascent.get_or_insert(a);
                (
                    line_top
                        + if options.line_height.is_some() {
                            baseline_ascent
                        } else {
                            a
                        },
                    options
                        .line_height
                        .unwrap_or((h + f64::from(line_spacing)).max(1.0)),
                )
            } else {
                (ascent + line as f64 * line_step, line_step)
            };
            let mut x = 0.0;
            let adjusted = bidi
                .paragraphs
                .iter()
                .find(|info| info.range.contains(&paragraph_offset))
                .filter(|_| start < end)
                .map(|paragraph_info| {
                    bidi.reordered_levels(
                        paragraph_info,
                        clusters[start].start..clusters[end - 1].end,
                    )
                });
            let levels: Vec<_> = clusters[start..end]
                .iter()
                .map(|c| adjusted.as_ref().map_or(c.level, |levels| levels[c.start]))
                .collect();
            for (visual_index, index) in BidiInfo::reorder_visual(&levels).into_iter().enumerate() {
                if visual_index > 0 {
                    x += options.tracking;
                }
                let cluster = &clusters[start + index];
                for glyph in &cluster.glyphs {
                    let mut glyph = glyph.clone();
                    glyph.x += x;
                    glyph.y += baseline;
                    if !glyph.x.is_finite() || !glyph.y.is_finite() || !glyph.advance.is_finite() {
                        return Err(invalid("non-finite shaped glyph position"));
                    }
                    result.glyphs.push(glyph);
                    result.glyph_lines.push(line);
                    check_work(result.glyphs.len(), line + 1)?;
                }
                x += cluster.width;
            }
            line_top += advance_y;
            let measured_width = if options.advanced { logical.width } else { x };
            result.line_widths.push(measured_width);
            result.width = result.width.max(measured_width);
            if end == clusters.len() {
                break;
            }
            start = end;
        }
    }
    result.height =
        ascent + descent + result.line_widths.len().saturating_sub(1) as f64 * line_step;
    if options.advanced {
        result.height = line_top;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RichTextRun,
        fonts::{DEFAULT_FACES, record},
    };

    fn inputs() -> (FontBinding, BTreeMap<String, Vec<u8>>) {
        let faces: BTreeMap<_, _> = DEFAULT_FACES
            .iter()
            .map(|b| (record(b).unwrap().sha256, b.to_vec()))
            .collect();
        let hashes = DEFAULT_FACES.map(|b| record(b).unwrap().sha256);
        (
            FontBinding {
                profile: TEXT_LAYOUT_PROFILE.into(),
                regular: hashes[0].clone(),
                bold: hashes[1].clone(),
                italic: hashes[2].clone(),
                bold_italic: hashes[3].clone(),
                warnings: vec![],
            },
            faces,
        )
    }
    #[test]
    fn explicit_line_height_keeps_baseline_step_across_face_metrics() {
        let (mut binding, mut faces) = inputs();
        // A pinned synthetic bold face with a different ascender exposes baseline drift.
        let mut bold = DEFAULT_FACES[1].to_vec();
        let offset = validate_face(&bold)
            .unwrap()
            .raw_face()
            .table_records
            .into_iter()
            .find(|record| record.tag == ttf_parser::Tag::from_bytes(b"hhea"))
            .unwrap()
            .offset as usize;
        bold[offset + 4..offset + 6].copy_from_slice(&2200i16.to_be_bytes());
        assert_eq!(validate_face(&bold).unwrap().ascender(), 2200);
        binding.bold = record(&bold).unwrap().sha256;
        faces.insert(binding.bold.clone(), bold);
        let document = serde_json::from_value(serde_json::json!({"runs":[
            {"text":"M\n"}, {"text":"M", "bold":true}
        ]}))
        .unwrap();
        let shaped = shape_with_layout(
            &document,
            &binding,
            &faces,
            48,
            "#ffffff",
            0,
            LayoutOptions {
                advanced: true,
                line_height: Some(60.5),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(shaped.glyphs[0].y, 1901.0 / 2048.0 * 48.0);
        assert_eq!(shaped.glyphs[1].y - shaped.glyphs[0].y, 60.5);
        assert_eq!(shaped.height, 121.0);
    }

    #[test]
    fn advanced_tracking_preserves_clusters_and_line_boxes() {
        let (binding, faces) = inputs();
        for text in ["office", "a\u{301}b", "אבג abc", "👩‍👩‍👧‍👦 x"] {
            let document = RichTextDocument::plain(text.into());
            let base = shape(&document, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
            let advanced = shape_with_layout(
                &document,
                &binding,
                &faces,
                48,
                "#ffffff",
                0,
                LayoutOptions {
                    tracking: 2.5,
                    advanced: true,
                    line_height: Some(60.5),
                    ..Default::default()
                },
            )
            .unwrap();
            let clusters: BTreeSet<_> = base.glyphs.iter().map(|g| g.cluster).collect();
            assert_eq!(
                base.glyphs
                    .iter()
                    .map(|g| (g.id, g.cluster))
                    .collect::<Vec<_>>(),
                advanced
                    .glyphs
                    .iter()
                    .map(|g| (g.id, g.cluster))
                    .collect::<Vec<_>>()
            );
            assert!(
                (advanced.width - base.width - clusters.len().saturating_sub(1) as f64 * 2.5).abs()
                    < 0.000001
            );
            assert_eq!(advanced.height, 60.5);
        }
    }

    #[test]
    fn advanced_wrap_modes_honor_hard_breaks_and_complete_clusters() {
        let (binding, faces) = inputs();
        for separator in [
            "\r", "\n", "\r\n", "\u{85}", "\u{b}", "\u{c}", "\u{2028}", "\u{2029}",
        ] {
            for wrap in [
                crate::TextWrap::None,
                crate::TextWrap::Word,
                crate::TextWrap::Cluster,
            ] {
                let document = RichTextDocument::plain(format!("fi{separator}"));
                let shaped = shape_with_layout(
                    &document,
                    &binding,
                    &faces,
                    48,
                    "#ffffff",
                    0,
                    LayoutOptions {
                        width: Some(1.0),
                        wrap,
                        advanced: true,
                        line_height: Some(50.0),
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(shaped.line_widths.len(), 2);
                assert_eq!(shaped.height, 100.0);
                assert_eq!(shaped.glyphs.len(), 1, "ligature remains intact");
            }
        }
        let document = RichTextDocument::plain("ab cd".into());
        let natural = shape(&document, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        let width = natural.glyphs[..4].iter().map(|g| g.advance).sum::<f64>();
        let run = |wrap| {
            shape_with_layout(
                &document,
                &binding,
                &faces,
                48,
                "#ffffff",
                0,
                LayoutOptions {
                    width: Some(width),
                    wrap,
                    advanced: true,
                    ..Default::default()
                },
            )
            .unwrap()
        };
        assert_eq!(run(crate::TextWrap::None).line_widths.len(), 1);
        assert_eq!(run(crate::TextWrap::Word).glyph_lines, vec![0, 0, 0, 1, 1]);
        assert_eq!(
            run(crate::TextWrap::Cluster).glyph_lines,
            vec![0, 0, 0, 0, 1]
        );
    }
    #[test]
    fn mandatory_separators_preserve_lines_clusters_and_paragraph_direction() {
        let (binding, faces) = inputs();
        for separator in [
            "\r", "\n", "\r\n", "\u{85}", "\u{b}", "\u{c}", "\u{2028}", "\u{2029}",
        ] {
            for wrap in [None, Some(1), Some(1000)] {
                let value = format!("A{separator}B{separator}{separator}");
                let shaped =
                    shape(&document(&value), &binding, &faces, 48, "#ffffff", wrap, 0).unwrap();
                assert_eq!(shaped.line_widths.len(), 4, "{separator:?} {wrap:?}");
                assert_eq!(shaped.glyph_lines, [0, 1]);
                assert_eq!(
                    shaped.glyphs.iter().map(|g| g.cluster).collect::<Vec<_>>(),
                    [0, 1 + separator.len() as u32]
                );
            }
        }
        let doc = serde_json::from_value(
            serde_json::json!({"runs":[{"text":"A\r"},{"text":"\nB","bold":true}]}),
        )
        .unwrap();
        let crlf = shape(&doc, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(crlf.glyph_lines, [0, 1]);
        assert_eq!(crlf.glyphs[1].cluster, 3);
        let same_paragraph = shape(
            &document("אב\u{2028}123 !"),
            &binding,
            &faces,
            48,
            "#ffffff",
            None,
            0,
        )
        .unwrap();
        let new_paragraph = shape(
            &document("אב\u{2029}123 !"),
            &binding,
            &faces,
            48,
            "#ffffff",
            None,
            0,
        )
        .unwrap();
        assert_eq!(same_paragraph.line_widths.len(), 2);
        assert_eq!(new_paragraph.line_widths.len(), 2);
        let second = |s: &ShapedText| {
            s.glyphs
                .iter()
                .zip(&s.glyph_lines)
                .filter(|(_, line)| **line == 1)
                .map(|(g, _)| g.cluster)
                .collect::<Vec<_>>()
        };
        assert_ne!(second(&same_paragraph), second(&new_paragraph));
        assert_eq!(
            shape(&document("AB"), &binding, &faces, 48, "#ffffff", None, 0)
                .unwrap()
                .line_widths
                .len(),
            1
        );
    }
    fn document(text: &str) -> RichTextDocument {
        serde_json::from_value(serde_json::json!({"runs":[{"text":text}]})).unwrap()
    }
    #[test]
    fn span_paint_keeps_ligatures_and_false_uses_pinned_regular_face() {
        let (binding, faces) = inputs();
        let base = document("ffi אבג á");
        let mut styled = base.clone();
        styled.spans = Some(serde_json::from_value(serde_json::json!([{"start":1,"end":2,"style":{"color":"#ff0000","paintLayers":[]}}])).unwrap());
        let plain = shape(&base, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        let painted = shape(&styled, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        let positions = |s: &ShapedText| {
            s.glyphs
                .iter()
                .map(|g| (g.id, g.cluster, g.x, g.y, g.advance))
                .collect::<Vec<_>>()
        };
        assert_eq!(positions(&plain), positions(&painted));
        assert_eq!(painted.glyphs[0].color, "#ffffff");
        assert_eq!(painted.glyphs[0].paint_layers, None);
        styled.runs[0].bold = Some(true);
        styled.spans = Some(
            serde_json::from_value(serde_json::json!([{"start":0,"end":3,"style":{"bold":false}}]))
                .unwrap(),
        );
        let shaped = shape(&styled, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(shaped.glyphs[0].face, binding.regular);
        assert!(shaped.glyphs.iter().any(|g| g.face == binding.bold));
    }
    #[test]
    fn work_limits_and_cluster_wrapping_are_inclusive() {
        check_work(MAX_SHAPED_GLYPHS, MAX_TEXT_LINES).unwrap();
        assert!(check_work(MAX_SHAPED_GLYPHS + 1, MAX_TEXT_LINES).is_err());
        assert!(check_work(MAX_SHAPED_GLYPHS, MAX_TEXT_LINES + 1).is_err());
        let (binding, faces) = inputs();
        let lines = shape(
            &document(&"\n".repeat(MAX_TEXT_LINES - 1)),
            &binding,
            &faces,
            12,
            "#ffffff",
            None,
            0,
        )
        .unwrap();
        assert_eq!(lines.line_widths.len(), MAX_TEXT_LINES);
        assert!(
            shape(
                &document(&"\n".repeat(MAX_TEXT_LINES)),
                &binding,
                &faces,
                12,
                "#ffffff",
                None,
                0
            )
            .is_err()
        );
        let wrapped = shape(
            &document("ffi ffi"),
            &binding,
            &faces,
            48,
            "#ffffff",
            Some(48),
            0,
        )
        .unwrap();
        let ligatures: Vec<_> = wrapped
            .glyphs
            .iter()
            .zip(&wrapped.glyph_lines)
            .filter(|(g, _)| g.id == 5044)
            .map(|(g, line)| (g.cluster, *line))
            .collect();
        assert_eq!(ligatures.len(), 2);
        assert_eq!(ligatures[0], (0, 0));
        assert_eq!(ligatures[1].0, 4);
        assert!(ligatures[1].1 > 0);
    }
    #[test]
    fn kerning_ligatures_and_combining_clusters_have_independent_expectations() {
        let (binding, faces) = inputs();
        let av = shape(&document("AV"), &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(av.glyphs.iter().map(|g| g.id).collect::<Vec<_>>(), [36, 57]);
        assert_eq!(av.glyphs[0].advance, 1270.0 * 48.0 / 2048.0);
        assert_eq!(av.glyphs[1].x, av.glyphs[0].advance);
        let ligature = shape(
            &document("ffi"),
            &binding,
            &faces,
            48,
            "#ffffff",
            Some(1),
            0,
        )
        .unwrap();
        assert_eq!(ligature.glyphs.len(), 1);
        assert_eq!(ligature.glyphs[0].id, 5044);
        assert_eq!(ligature.glyphs[0].cluster, 0);
        assert_eq!(ligature.line_widths.len(), 1);
        let combining = shape(
            &document("e\u{301}"),
            &binding,
            &faces,
            48,
            "#ffffff",
            None,
            0,
        )
        .unwrap();
        assert_eq!(combining.glyphs.len(), 1);
        assert_eq!(combining.glyphs[0].id, 171);
        assert_eq!(combining.glyphs[0].cluster, 0);
    }
    #[test]
    fn paint_boundaries_preserve_shaping_and_newlines_remain_explicit() {
        let (binding, faces) = inputs();
        let mut doc = document("A");
        let mut run: RichTextRun =
            serde_json::from_value(serde_json::json!({"text":"V", "color":"#ff0000"})).unwrap();
        doc.runs.push(run.clone());
        let colored = shape(&doc, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        let plain = shape(&document("AV"), &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(colored.width, plain.width);
        assert_eq!(colored.glyphs[1].color, "#ff0000");
        run.text = "\n\nV\n".into();
        doc.runs[1] = run;
        let lines = shape(&doc, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(lines.line_widths.len(), 4);
        assert_eq!(lines.glyph_lines, [0, 2]);
    }
    #[test]
    fn bidi_missing_glyphs_and_repeatability_are_fixed() {
        let (binding, faces) = inputs();
        let doc = document("אבג abc \u{10ffff}");
        let first = shape(&doc, &binding, &faces, 48, "#ffffff", None, 0).unwrap();
        assert_eq!(
            first,
            shape(&doc, &binding, &faces, 48, "#ffffff", None, 0).unwrap()
        );
        assert!(first.glyphs.iter().any(|g| g.id == 0));
        let clusters: Vec<_> = first
            .glyphs
            .iter()
            .filter(|g| g.cluster < 6)
            .map(|g| g.cluster)
            .collect();
        assert_eq!(clusters, [4, 2, 0]);
        let mut unknown = binding;
        unknown.profile = "future".into();
        assert!(shape(&doc, &unknown, &faces, 48, "#ffffff", None, 0).is_err());
    }
}
