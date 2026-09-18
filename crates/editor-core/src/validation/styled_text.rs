//! Canonical styled text constraints, shared by editing, persistence and evaluation.
use crate::{
    CoreError, ErrorCode, MAX_TEXT_PAINT_LAYERS, MAX_TEXT_SPANS, RichTextDocument, TextPaintLayer,
    TextSpanStyle,
};
fn invalid() -> CoreError {
    CoreError::new(
        ErrorCode::InvalidArgument,
        "invalid styled text span or paint layer",
    )
}
/// Validate retained typed draft payloads before migration publishes any changes.
pub(crate) fn validate_draft_text(operations: &[crate::EditOperation]) -> Result<(), CoreError> {
    fn visit(value: &serde_json::Value) -> Result<(), CoreError> {
        match value {
            serde_json::Value::Object(object) => {
                if object.get("runs").is_some_and(serde_json::Value::is_array) {
                    let document = serde_json::from_value::<RichTextDocument>(value.clone())
                        .map_err(|_| invalid())?;
                    super::validate_rich_text(&document)?;
                }
                if let Some(layers) = object.get("paintLayers").filter(|v| v.is_array()) {
                    let layers = serde_json::from_value::<Vec<TextPaintLayer>>(layers.clone())
                        .map_err(|_| invalid())?;
                    validate_text_paints(&layers)?;
                }
                for child in object.values() {
                    visit(child)?;
                }
            }
            serde_json::Value::Array(values) => {
                for child in values {
                    visit(child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    visit(&serde_json::to_value(operations).map_err(|_| invalid())?)
}
pub(crate) fn validate_text_paints(layers: &[TextPaintLayer]) -> Result<(), CoreError> {
    if layers.len() > MAX_TEXT_PAINT_LAYERS {
        return Err(invalid());
    }
    for layer in layers {
        let (color, opacity) = match layer {
            TextPaintLayer::Fill { color, opacity } => (color, opacity),
            TextPaintLayer::Stroke {
                color,
                opacity,
                width_px,
            } => {
                if !width_px.is_finite() || *width_px <= 0.0 || *width_px > 200.0 {
                    return Err(invalid());
                }
                (color, opacity)
            }
            TextPaintLayer::Shadow {
                color,
                opacity,
                offset_x_px,
                offset_y_px,
                blur_sigma_px,
            } => {
                if [*offset_x_px, *offset_y_px]
                    .iter()
                    .any(|x| !x.is_finite() || x.abs() > 4096.0)
                    || !blur_sigma_px.is_finite()
                    || !(0.0..=64.0).contains(blur_sigma_px)
                {
                    return Err(invalid());
                }
                (color, opacity)
            }
        };
        if !opacity.is_finite()
            || !(0.0..=1.0).contains(opacity)
            || crate::validation::validate_color(color).is_err()
        {
            return Err(invalid());
        }
    }
    Ok(())
}

pub(crate) fn validate_text_spans(document: &RichTextDocument) -> Result<(), CoreError> {
    let Some(spans) = &document.spans else {
        return Ok(());
    };
    if spans.len() > MAX_TEXT_SPANS {
        return Err(invalid());
    }
    let count = document.grapheme_boundaries().len() - 1;
    let mut end = 0;
    for span in spans {
        if span.start < end
            || span.start >= span.end
            || span.end as usize > count
            || span.style == TextSpanStyle::default()
        {
            return Err(invalid());
        }
        if span
            .style
            .color
            .as_ref()
            .is_some_and(|c| crate::validation::validate_color(c).is_err())
        {
            return Err(invalid());
        }
        if let Some(layers) = &span.style.paint_layers {
            validate_text_paints(layers)?;
        }
        end = span.end;
    }
    Ok(())
}
