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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapedText {
    pub(crate) glyphs: Vec<ShapedGlyph>,
    pub(crate) line_widths: Vec<f64>,
    pub(crate) glyph_lines: Vec<usize>,
    pub(crate) line_height: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) font_size: u32,
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
    if binding.profile != TEXT_LAYOUT_PROFILE
        || font_size == 0
        || font_size > 1000
        || wrap_width == Some(0)
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
    let mut ascent = 0.0f64;
    let mut descent = 0.0f64;
    let mut line_height = 0.0f64;
    for hash in binding.hashes() {
        let face = validate_face(
            faces
                .get(hash)
                .ok_or_else(|| invalid("prepared font face is missing"))?,
        )?;
        let scale = f64::from(font_size) / f64::from(face.units_per_em());
        ascent = ascent.max(f64::from(face.ascender()) * scale);
        descent = descent.max(-f64::from(face.descender()) * scale);
        line_height = line_height.max(
            f64::from(
                i32::from(face.ascender()) - i32::from(face.descender())
                    + i32::from(face.line_gap()),
            ) * scale,
        );
    }
    let line_step = (line_height + f64::from(line_spacing)).max(1.0);
    let mut result = ShapedText {
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
            let mut last_break = None;
            while end < clusters.len() {
                if end > start
                    && wrap_width.is_some_and(|w| width + clusters[end].width > f64::from(w))
                {
                    break;
                }
                width += clusters[end].width;
                end += 1;
                if opportunities.contains(&clusters[end - 1].end) {
                    last_break = Some(end);
                }
            }
            if end < clusters.len()
                && let Some(last) = last_break
            {
                end = last;
            }
            let line = result.line_widths.len();
            check_work(result.glyphs.len(), line + 1)?;
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
            for index in BidiInfo::reorder_visual(&levels) {
                let cluster = &clusters[start + index];
                for glyph in &cluster.glyphs {
                    let mut glyph = glyph.clone();
                    glyph.x += x;
                    glyph.y += ascent + line as f64 * line_step;
                    if !glyph.x.is_finite() || !glyph.y.is_finite() || !glyph.advance.is_finite() {
                        return Err(invalid("non-finite shaped glyph position"));
                    }
                    result.glyphs.push(glyph);
                    result.glyph_lines.push(line);
                    check_work(result.glyphs.len(), line + 1)?;
                }
                x += cluster.width;
            }
            result.line_widths.push(x);
            result.width = result.width.max(x);
            if end == clusters.len() {
                break;
            }
            start = end;
        }
    }
    result.height =
        ascent + descent + result.line_widths.len().saturating_sub(1) as f64 * line_step;
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
