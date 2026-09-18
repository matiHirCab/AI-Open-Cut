use super::{RichTextDocument, RichTextRun, deserialize_present};
use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub const MAX_TEXT_SPANS: usize = 256;
pub const MAX_TEXT_PAINT_LAYERS: usize = 16;
type TextPaintRange = (std::ops::Range<usize>, Vec<TextPaintLayer>);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TextPaintLayer {
    Fill {
        color: String,
        opacity: f64,
    },
    Stroke {
        color: String,
        opacity: f64,
        width_px: f64,
    },
    Shadow {
        color: String,
        opacity: f64,
        offset_x_px: f64,
        offset_y_px: f64,
        blur_sigma_px: f64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextSpan {
    pub start: u32,
    pub end: u32,
    pub style: TextSpanStyle,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextSpanStyle {
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub bold: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub italic: Option<bool>,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_present",
        skip_serializing_if = "Option::is_none"
    )]
    pub paint_layers: Option<Vec<TextPaintLayer>>,
}

fn invalid() -> CoreError {
    CoreError::new(
        ErrorCode::InvalidArgument,
        "invalid styled text span or paint layer",
    )
}

impl RichTextDocument {
    fn checked_span_boundaries(&self) -> Result<Vec<usize>, CoreError> {
        let boundaries = self.grapheme_boundaries();
        if self.spans.iter().flatten().any(|span| {
            boundaries.get(span.start as usize).is_none()
                || boundaries.get(span.end as usize).is_none()
        }) {
            return Err(invalid());
        }
        Ok(boundaries)
    }
    pub(crate) fn grapheme_boundaries(&self) -> Vec<usize> {
        let text = self.text();
        text.grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()))
            .collect()
    }

    /// Resolve only typography/color here. Paint-only boundaries never split the shaper's face segments.
    pub(crate) fn effective_runs(&self) -> Result<Vec<RichTextRun>, CoreError> {
        let Some(spans) = self.spans.as_ref().filter(|s| !s.is_empty()) else {
            return Ok(self.runs.clone());
        };
        let bounds = self.checked_span_boundaries()?;
        let mut result = Vec::new();
        let mut offset = 0;
        for run in &self.runs {
            let end = offset + run.text.len();
            let mut cuts = vec![offset, end];
            for span in spans {
                cuts.extend(
                    [bounds[span.start as usize], bounds[span.end as usize]]
                        .into_iter()
                        .filter(|v| *v > offset && *v < end),
                );
            }
            cuts.sort_unstable();
            cuts.dedup();
            for pair in cuts.windows(2) {
                let mut part = run.clone();
                part.text = run.text[pair[0] - offset..pair[1] - offset].to_owned();
                if let Some(span) = spans.iter().find(|s| {
                    bounds[s.start as usize] <= pair[0] && pair[0] < bounds[s.end as usize]
                }) {
                    part.bold = span.style.bold.or(part.bold);
                    part.italic = span.style.italic.or(part.italic);
                    part.color = span.style.color.clone().or(part.color);
                }
                result.push(part);
            }
            offset = end;
        }
        Ok(result)
    }

    pub(crate) fn paint_ranges(&self) -> Result<Vec<TextPaintRange>, CoreError> {
        let Some(spans) = self.spans.as_ref().filter(|spans| !spans.is_empty()) else {
            return Ok(vec![]);
        };
        let bounds = self.checked_span_boundaries()?;
        Ok(spans
            .iter()
            .filter_map(|span| {
                span.style.paint_layers.as_ref().map(|layers| {
                    (
                        bounds[span.start as usize]..bounds[span.end as usize],
                        layers.clone(),
                    )
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::styled_text::{validate_text_paints, validate_text_spans};
    use serde_json::{Value, json};

    #[test]
    fn unicode_16_official_grapheme_break_conformance() {
        let data = include_str!("../../tests/fixtures/GraphemeBreakTest-16.0.0.txt");
        let mut cases = 0;
        for (line_number, line) in data.lines().enumerate() {
            let body = line.split('#').next().unwrap().trim();
            if body.is_empty() {
                continue;
            }
            let mut text = String::new();
            let mut expected = vec![];
            for token in body.split_whitespace() {
                match token {
                    "÷" => expected.push(text.len()),
                    "×" => {}
                    hex => {
                        text.push(char::from_u32(u32::from_str_radix(hex, 16).unwrap()).unwrap())
                    }
                }
            }
            assert_eq!(
                RichTextDocument::plain(text).grapheme_boundaries(),
                expected,
                "Unicode fixture line {}",
                line_number + 1
            );
            cases += 1;
        }
        assert!(cases > 1000);
    }

    #[test]
    fn canonical_unicode_and_paint_fixtures() {
        assert_eq!(unicode_segmentation::UNICODE_VERSION, (16, 0, 0));
        let catalog: Value = serde_json::from_str(include_str!(
            "../../../../contracts/styled-text-layers-v1.json"
        ))
        .unwrap();
        for fixture in catalog["graphemes"].as_array().unwrap() {
            let document = RichTextDocument::plain(fixture["text"].as_str().unwrap().into());
            assert_eq!(
                serde_json::to_value(document.grapheme_boundaries()).unwrap(),
                fixture["boundaries"]
            );
        }
        for valid in [true, false] {
            for fixture in catalog[if valid { "valid" } else { "invalid" }]
                .as_array()
                .unwrap()
            {
                let result = if let Some(document) = fixture.get("document") {
                    serde_json::from_value::<RichTextDocument>(document.clone())
                        .map_err(|_| invalid())
                        .and_then(|document| validate_text_spans(&document))
                } else {
                    validate_text_paints(
                        &serde_json::from_value::<Vec<TextPaintLayer>>(
                            fixture["paintLayers"].clone(),
                        )
                        .unwrap(),
                    )
                };
                assert_eq!(result.is_ok(), valid, "{}", fixture["id"]);
            }
        }
    }

    #[test]
    fn bounded_spans_reject_null_and_preserve_cross_run_graphemes() {
        let mut document: RichTextDocument = serde_json::from_value(json!({"runs":[{"text":"a","bold":true},{"text":"́b"}],"spans":[{"start":0,"end":1,"style":{"bold":false,"color":"#ff0000"}}]})).unwrap();
        let runs = document.effective_runs().unwrap();
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            "áb"
        );
        assert_eq!(runs[0].bold, Some(false));
        assert_eq!(runs[1].bold, Some(false));
        assert_eq!(runs[1].color.as_deref(), Some("#ff0000"));
        document.runs = RichTextDocument::plain("x".repeat(256)).runs;
        document.spans = Some(
            (0..256)
                .map(|start| TextSpan {
                    start,
                    end: start + 1,
                    style: TextSpanStyle {
                        bold: Some(false),
                        ..Default::default()
                    },
                })
                .collect(),
        );
        validate_text_spans(&document).unwrap();
        let extra = document.spans.as_ref().unwrap()[0].clone();
        let mut spans = document.spans.take().unwrap().into_vec();
        spans.push(extra);
        document.spans = Some(spans.into_boxed_slice());
        assert!(validate_text_spans(&document).is_err());
        for value in [
            json!({"spans":null}),
            json!({"spans":[{"start":0.5,"end":1,"style":{"bold":true}}]}),
            json!({"spans":[{"start":0,"end":1,"style":{"bold":null}}]}),
        ] {
            let mut input = json!({"runs":[{"text":"a"}]});
            input["spans"] = value["spans"].clone();
            assert!(serde_json::from_value::<RichTextDocument>(input).is_err());
        }
    }

    #[test]
    fn paint_limits_are_finite_and_inclusive() {
        let stroke = TextPaintLayer::Stroke {
            color: "#ffffff".into(),
            opacity: 1.0,
            width_px: 200.0,
        };
        validate_text_paints(&vec![stroke.clone(); 16]).unwrap();
        assert!(validate_text_paints(&vec![stroke; 17]).is_err());
        for sigma in [0.0, 64.0, 64.01, f64::NAN, f64::INFINITY] {
            let shadow = TextPaintLayer::Shadow {
                color: "#ffffff".into(),
                opacity: 0.5,
                offset_x_px: -4096.0,
                offset_y_px: 4096.0,
                blur_sigma_px: sigma,
            };
            assert_eq!(
                validate_text_paints(&[shadow]).is_ok(),
                sigma.is_finite() && sigma <= 64.0
            );
        }
        for width in [0.0, -1.0, 0.25, 200.0, 200.01, f64::NAN, f64::INFINITY] {
            assert_eq!(
                validate_text_paints(&[TextPaintLayer::Stroke {
                    color: "#123456".into(),
                    opacity: 0.0,
                    width_px: width
                }])
                .is_ok(),
                width.is_finite() && width > 0.0 && width <= 200.0
            );
        }
        for offset in [-4096.01, -4096.0, 4096.0, 4096.01, f64::NAN] {
            assert_eq!(
                validate_text_paints(&[TextPaintLayer::Shadow {
                    color: "#123456".into(),
                    opacity: 1.0,
                    offset_x_px: offset,
                    offset_y_px: 0.0,
                    blur_sigma_px: 0.0
                }])
                .is_ok(),
                offset.is_finite() && offset.abs() <= 4096.0
            );
        }
    }
}
