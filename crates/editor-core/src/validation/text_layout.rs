use crate::{CoreError, ErrorCode, TextFit, TextStyle, TextVerticalAlignment};

pub(super) fn validate(style: &TextStyle) -> Result<(), CoreError> {
    let Some(layout) = &style.layout else {
        return Ok(());
    };
    let valid = |value: f64, min, max| value.is_finite() && (min..=max).contains(&value);
    let width = layout
        .bounds
        .as_ref()
        .and_then(|b| b.width_px)
        .or(style.wrap_width_px.map(f64::from));
    let height = layout.bounds.as_ref().and_then(|b| b.height_px);
    if !valid(layout.tracking_px, 0.0, 1000.0)
        || !valid(layout.background_corner_radius_px, 0.0, 2160.0)
        || layout
            .line_height_px
            .is_some_and(|v| !valid(v, 1.0, 4320.0))
        || layout
            .bounds
            .as_ref()
            .is_some_and(|b| b.width_px.is_none() && b.height_px.is_none())
        || width.is_some_and(|v| {
            !valid(v, 1.0, 7680.0)
                || v <= f64::from(style.padding.left) + f64::from(style.padding.right)
        })
        || height.is_some_and(|v| {
            !valid(v, 1.0, 4320.0)
                || v <= f64::from(style.padding.top) + f64::from(style.padding.bottom)
        })
        || (layout.vertical_alignment != TextVerticalAlignment::Top && height.is_none())
        || match layout.fit {
            TextFit::None => false,
            TextFit::Shrink => width.is_none() && height.is_none(),
            TextFit::FitWidth => width.is_none(),
            TextFit::FitBox => width.is_none() || height.is_none(),
        }
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "text layout is outside supported bounds",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_layout_boundaries() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/advanced-text-layout-v1.json"
        ))
        .unwrap();
        for example in fixture["valid"].as_array().unwrap() {
            let style: TextStyle =
                serde_json::from_value(serde_json::json!({"layout": example["layout"]})).unwrap();
            validate(&style).unwrap();
        }
        for key in ["invalid", "invalidDomain"] {
            for example in fixture[key].as_array().unwrap() {
                let parsed = serde_json::from_value::<TextStyle>(
                    serde_json::json!({"layout": example["layout"]}),
                );
                assert!(
                    parsed.is_err() || validate(&parsed.unwrap()).is_err(),
                    "{}",
                    example["id"]
                );
            }
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut style = TextStyle {
                layout: Some(Default::default()),
                ..Default::default()
            };
            style.layout.as_mut().unwrap().tracking_px = value;
            assert!(validate(&style).is_err());
        }
        let style = TextStyle {
            layout: Some(Box::new(crate::TextLayout {
                bounds: Some(crate::TextBounds {
                    width_px: Some(10.0),
                    height_px: None,
                }),
                ..Default::default()
            })),
            padding: crate::TextPadding {
                left: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate(&style).is_err());
    }
}
