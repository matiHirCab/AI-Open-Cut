// Frozen presentation predecessor at abc06b75; separate formatter and whole-source reference.
//! Presentation-only field descriptions. EditorCore remains the validator and mutator.
use crate::inspector_edit::{Field, FieldKind};
use opencut_editor_core::{TextLayout, TimelineItem, Transform2D};
use serde_json::Value;

fn hex_color(value: &Value) -> Option<String> {
    let color = value.as_object()?;
    let channel = |name| {
        let number = color.get(name)?.as_f64()?;
        (number.is_finite() && (0.0..=1.0).contains(&number))
            .then_some((number * 255.0).round() as u8)
    };
    Some(format!(
        "#{:02X}{:02X}{:02X}",
        channel("r")?,
        channel("g")?,
        channel("b")?
    ))
}

pub(crate) fn add(
    result: &mut Vec<Field>,
    source: &Value,
    label: impl Into<String>,
    path: impl Into<String>,
    update_key: &'static str,
    kind: FieldKind,
) {
    let path = path.into();
    let value = source.pointer(&path).unwrap_or(&Value::Null);
    let value = match kind {
        FieldKind::HexColor => hex_color(value).unwrap_or_default(),
        FieldKind::Boolean => value.as_bool().unwrap_or(false).to_string(),
        FieldKind::Milliseconds | FieldKind::SignedMilliseconds if value.is_null() => "0".into(),
        _ if value.is_null() => String::new(),
        _ => value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string()),
    };
    result.push(Field {
        label: label.into(),
        path,
        update_key,
        kind,
        value,
    });
}

fn effective_transform(item: &TimelineItem) -> Transform2D {
    if let Some(transform) = item.visual_properties().transform2d {
        return transform;
    }
    let legacy = &item.visual_properties().transform;
    let mut transform = Transform2D::default();
    transform.position.x = legacy.position_x;
    transform.position.y = legacy.position_y;
    transform.scale_x = legacy.scale;
    transform.scale_y = legacy.scale;
    transform.opacity = legacy.opacity;
    transform
}

pub(crate) fn fields(item: &TimelineItem) -> Vec<Field> {
    let mut source = serde_json::to_value(item).expect("serialize selected item");
    let mut result = Vec::new();
    match item {
        TimelineItem::Shape(shape) => {
            if matches!(
                shape.geometry,
                opencut_editor_core::ShapeGeometry::Rectangle { .. }
                    | opencut_editor_core::ShapeGeometry::RoundedRectangle { .. }
                    | opencut_editor_core::ShapeGeometry::Ellipse { .. }
            ) {
                for (name, label) in [
                    ("width", "Width · local px"),
                    ("height", "Height · local px"),
                ] {
                    add(
                        &mut result,
                        &source,
                        label,
                        format!("/geometry/{name}"),
                        "geometry",
                        FieldKind::Number,
                    );
                }
            }
            if matches!(
                shape.geometry,
                opencut_editor_core::ShapeGeometry::RoundedRectangle { .. }
            ) {
                for (name, label) in [
                    ("topLeft", "Top left radius · px"),
                    ("topRight", "Top right radius · px"),
                    ("bottomRight", "Bottom right radius · px"),
                    ("bottomLeft", "Bottom left radius · px"),
                ] {
                    add(
                        &mut result,
                        &source,
                        label,
                        format!("/geometry/radii/{name}"),
                        "geometry",
                        FieldKind::Number,
                    );
                }
            }
            if source.pointer("/fill/type").and_then(Value::as_str) == Some("solid") {
                add(
                    &mut result,
                    &source,
                    "Fill · #RRGGBB",
                    "/fill/color",
                    "fill",
                    FieldKind::HexColor,
                );
            }
            if source.pointer("/stroke/paint/type").and_then(Value::as_str) == Some("solid") {
                add(
                    &mut result,
                    &source,
                    "Stroke · #RRGGBB",
                    "/stroke/paint/color",
                    "stroke",
                    FieldKind::HexColor,
                );
            }
            if shape.stroke.is_some() {
                add(
                    &mut result,
                    &source,
                    "Stroke width · px",
                    "/stroke/width",
                    "stroke",
                    FieldKind::Number,
                );
            }
        }
        TimelineItem::Grid(grid) => {
            if source.pointer("/transform2d").is_none() {
                source["transform2d"] = serde_json::to_value(effective_transform(item)).unwrap();
            }
            add(
                &mut result,
                &source,
                "Angle · degrees",
                "/transform2d/rotationDeg",
                "transform2d",
                FieldKind::Number,
            );
            let pattern = &grid.grid.pattern;
            match pattern {
                opencut_editor_core::GridPattern::Rectangular { .. }
                | opencut_editor_core::GridPattern::Dot { .. } => {
                    for (name, label) in [
                        ("spacingX", "Horizontal spacing · px"),
                        ("spacingY", "Vertical spacing · px"),
                    ] {
                        add(
                            &mut result,
                            &source,
                            label,
                            format!("/grid/pattern/{name}"),
                            "grid",
                            FieldKind::Number,
                        );
                    }
                }
                _ => add(
                    &mut result,
                    &source,
                    "Spacing · px",
                    "/grid/pattern/spacing",
                    "grid",
                    FieldKind::Number,
                ),
            }
            if !matches!(pattern, opencut_editor_core::GridPattern::Dot { .. }) {
                if source
                    .pointer("/grid/pattern/stroke/paint/type")
                    .and_then(Value::as_str)
                    == Some("solid")
                {
                    add(
                        &mut result,
                        &source,
                        "Line · #RRGGBB",
                        "/grid/pattern/stroke/paint/color",
                        "grid",
                        FieldKind::HexColor,
                    );
                }
                add(
                    &mut result,
                    &source,
                    "Line width · px",
                    "/grid/pattern/stroke/width",
                    "grid",
                    FieldKind::Number,
                );
            }
        }
        TimelineItem::Text(text) => {
            add(
                &mut result,
                &source,
                "Content · literal text",
                "/text",
                "text",
                FieldKind::Text,
            );
            add(
                &mut result,
                &source,
                "Font size · px",
                "/fontSize",
                "fontSize",
                FieldKind::Integer,
            );
            add(
                &mut result,
                &source,
                "Base color · #RRGGBB",
                "/color",
                "color",
                FieldKind::Text,
            );
            for (index, _) in text.document.runs.iter().enumerate() {
                for (name, label, kind) in [
                    ("bold", "bold", FieldKind::Boolean),
                    ("italic", "italic", FieldKind::Boolean),
                    ("color", "color · #RRGGBB", FieldKind::Text),
                ] {
                    add(
                        &mut result,
                        &source,
                        format!("Run {} {label}", index + 1),
                        format!("/document/runs/{index}/{name}"),
                        "document",
                        kind,
                    );
                }
            }
            if source.pointer("/style/layout").is_none() {
                source["style"]["layout"] = serde_json::to_value(TextLayout::default()).unwrap();
            }
            add(
                &mut result,
                &source,
                "Tracking · px",
                "/style/layout/trackingPx",
                "style",
                FieldKind::Number,
            );
            add(
                &mut result,
                &source,
                "Line height · px (blank = auto)",
                "/style/layout/lineHeightPx",
                "style",
                FieldKind::OptionalNumber,
            );
            for (name, label) in [
                ("widthPx", "Layout width · px"),
                ("heightPx", "Layout height · px"),
            ] {
                add(
                    &mut result,
                    &source,
                    label,
                    format!("/style/layout/bounds/{name}"),
                    "style",
                    FieldKind::OptionalNumber,
                );
            }
            add(
                &mut result,
                &source,
                "Fit mode",
                "/style/layout/fit",
                "style",
                FieldKind::Choice(&["none", "shrink", "fit_width", "fit_box"]),
            );
            add(
                &mut result,
                &source,
                "Outline color · #RRGGBB",
                "/style/outlineColor",
                "style",
                FieldKind::Text,
            );
            add(
                &mut result,
                &source,
                "Outline width · px",
                "/style/outlineWidthPx",
                "style",
                FieldKind::Integer,
            );
            for (name, label, kind) in [
                ("color", "Shadow color · #RRGGBB", FieldKind::Text),
                ("opacity", "Shadow opacity", FieldKind::Number),
                ("offsetX", "Shadow X · px", FieldKind::SignedInteger),
                ("offsetY", "Shadow Y · px", FieldKind::SignedInteger),
            ] {
                add(
                    &mut result,
                    &source,
                    label,
                    format!("/style/shadow/{name}"),
                    "style",
                    kind,
                );
            }
            if let Some(layers) = source
                .pointer("/style/paintLayers")
                .and_then(Value::as_array)
            {
                for (index, layer) in layers.iter().enumerate() {
                    for (name, label, kind) in [
                        ("color", "color · #RRGGBB", FieldKind::Text),
                        ("opacity", "opacity", FieldKind::Number),
                        ("widthPx", "width · px", FieldKind::Number),
                        ("offsetXPx", "X · px", FieldKind::Number),
                        ("offsetYPx", "Y · px", FieldKind::Number),
                        ("blurSigmaPx", "blur · px", FieldKind::Number),
                    ] {
                        if layer.get(name).is_some() {
                            add(
                                &mut result,
                                &source,
                                format!("Layer {} {label}", index + 1),
                                format!("/style/paintLayers/{index}/{name}"),
                                "style",
                                kind,
                            );
                        }
                    }
                }
            }
        }
        _ => {}
    }
    if matches!(
        item,
        TimelineItem::Group(_) | TimelineItem::ComponentInstance(_)
    ) {
        add(
            &mut result,
            &source,
            "Stagger · ms",
            "/staggerMs",
            "staggerMs",
            FieldKind::Milliseconds,
        );
    }
    if matches!(item, TimelineItem::Repeater(_)) {
        add(
            &mut result,
            &source,
            "Repeater time offset · signed ms",
            "/repeater/timeOffsetMs",
            "repeater",
            FieldKind::SignedMilliseconds,
        );
    }
    result
}

pub(crate) fn animation_fields(
    item: &TimelineItem,
    cursor: crate::animation_inspector::Cursor,
    audio_only: bool,
) -> Vec<Field> {
    let channels = &item.visual_properties().animation_channels;
    let index = cursor.channel_index(item);
    let Some(channel) = channels.get(index) else {
        return vec![];
    };
    let key_index = cursor.key_index(item);
    let Some(key) = channel.keyframes.get(key_index) else {
        return vec![];
    };
    if !matches!(
        key.value,
        opencut_editor_core::AnimationChannelValue::Scalar { .. }
    ) || (audio_only
        && channel.property != opencut_editor_core::AnimationChannelProperty::GainDb)
    {
        return vec![];
    }
    let source = serde_json::to_value(item).unwrap();
    let mut fields = vec![];
    for (label, path, kind) in [
        (
            "Source key time · ms",
            format!("/animationChannels/{index}/keyframes/{key_index}/timeMs"),
            FieldKind::Milliseconds,
        ),
        (
            "Scalar key value",
            format!("/animationChannels/{index}/keyframes/{key_index}/value/value"),
            FieldKind::Number,
        ),
    ] {
        add(&mut fields, &source, label, path, "animationChannels", kind);
    }
    if let Some(curve) = source
        .pointer(&format!(
            "/animationChannels/{index}/keyframes/{key_index}/curve"
        ))
        .and_then(Value::as_object)
    {
        for name in [
            "x1",
            "y1",
            "x2",
            "y2",
            "mass",
            "stiffness",
            "damping",
            "initialVelocity",
        ] {
            if curve.contains_key(name) {
                add(
                    &mut fields,
                    &source,
                    format!("Curve {name}"),
                    format!("/animationChannels/{index}/keyframes/{key_index}/curve/{name}"),
                    "animationChannels",
                    FieldKind::Number,
                );
            }
        }
    }
    add(
        &mut fields,
        &source,
        "Loop mode · none/repeat/ping_pong",
        format!("/animationChannels/{index}/loop/mode"),
        "animationChannels",
        FieldKind::LoopMode,
    );
    let last = fields.last_mut().unwrap();
    if channel.r#loop.is_none() {
        last.value = "none".into();
    }
    if channel.r#loop.is_some() {
        add(
            &mut fields,
            &source,
            "Loop iterations · count/infinite",
            format!("/animationChannels/{index}/loop/iterations"),
            "animationChannels",
            FieldKind::LoopIterations,
        );
    }
    fields
}
