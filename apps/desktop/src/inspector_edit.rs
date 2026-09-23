//! Presentation-only field descriptions. EditorCore remains the validator and mutator.
use opencut_editor_core::{EditOperation, TextLayout, TimelineItem, Transform2D};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
pub(crate) enum FieldKind {
    Text,
    Number,
    Integer,
    SignedInteger,
    OptionalNumber,
    HexColor,
    Boolean,
    Choice(&'static [&'static str]),
}

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub label: String,
    pub path: String,
    pub update_key: &'static str,
    pub kind: FieldKind,
    pub value: String,
}

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

fn add(
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
    result
}

fn parsed_value(field: &Field, input: &str, original: &Value) -> Result<Value, String> {
    let input = if matches!(field.kind, FieldKind::Text) {
        input
    } else {
        input.trim()
    };
    match field.kind {
        FieldKind::Text => Ok(json!(input)),
        FieldKind::Integer => input
            .parse::<u32>()
            .map(|value| json!(value))
            .map_err(|_| format!("{} needs a nonnegative integer.", field.label)),
        FieldKind::SignedInteger => input
            .parse::<i32>()
            .map(|value| json!(value))
            .map_err(|_| format!("{} needs a signed integer.", field.label)),
        FieldKind::Number | FieldKind::OptionalNumber => {
            if input.is_empty() && matches!(field.kind, FieldKind::OptionalNumber) {
                return Ok(Value::Null);
            }
            let number = input
                .parse::<f64>()
                .map_err(|_| format!("{} needs a number.", field.label))?;
            serde_json::Number::from_f64(number)
                .map(Value::Number)
                .ok_or_else(|| format!("{} needs a finite number.", field.label))
        }
        FieldKind::Boolean => input
            .parse::<bool>()
            .map(|value| json!(value))
            .map_err(|_| "Use true or false.".into()),
        FieldKind::Choice(choices) => choices
            .contains(&input)
            .then(|| json!(input))
            .ok_or_else(|| format!("Use one of: {}.", choices.join(", "))),
        FieldKind::HexColor => {
            let bytes = input
                .strip_prefix('#')
                .filter(|s| s.len() == 6)
                .ok_or("Use #RRGGBB.")?;
            let rgb: Vec<_> = (0..3)
                .map(|i| u8::from_str_radix(&bytes[i * 2..i * 2 + 2], 16))
                .collect::<Result<_, _>>()
                .map_err(|_| "Use #RRGGBB.")?;
            Ok(
                json!({"r":f64::from(rgb[0])/255.0,"g":f64::from(rgb[1])/255.0,"b":f64::from(rgb[2])/255.0,"a":original["a"]}),
            )
        }
    }
}

pub(crate) fn build(
    item: &TimelineItem,
    field: &Field,
    input: &str,
) -> Result<EditOperation, String> {
    let mut source = serde_json::to_value(item).expect("serialize selected item");
    let original = source.pointer(&field.path).cloned().unwrap_or(Value::Null);
    let value = parsed_value(field, input, &original)?;
    if field.update_key == "transform2d" && source.pointer("/transform2d").is_none() {
        source["transform2d"] = serde_json::to_value(effective_transform(item)).unwrap();
    }
    if field.path.starts_with("/style/layout/") && source.pointer("/style/layout").is_none() {
        source["style"]["layout"] = serde_json::to_value(TextLayout::default()).unwrap();
    }
    let parts: Vec<_> = field.path.trim_start_matches('/').split('/').collect();
    let mut target = &mut source;
    for key in &parts[..parts.len() - 1] {
        if let Ok(index) = key.parse::<usize>() {
            target = &mut target[index];
        } else {
            if target.get(*key).is_none() || target[*key].is_null() {
                target[*key] = json!({});
            }
            target = &mut target[*key];
        }
    }
    let last = parts[parts.len() - 1];
    if value.is_null() && matches!(field.kind, FieldKind::OptionalNumber) {
        if let Some(map) = target.as_object_mut() {
            map.remove(last);
        }
    } else {
        target[last] = value;
    }
    // Document input recomputes the compatibility text in core; plain text input deliberately
    // uses the legacy one-run replacement rule.
    let edit = json!({"operation":"update_item","itemId":item.id(),field.update_key:source[field.update_key]});
    serde_json::from_value(edit).map_err(|error| format!("Could not prepare edit: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field<'a>(fields: &'a [Field], label: &str) -> &'a Field {
        fields.iter().find(|field| field.label == label).unwrap()
    }

    #[test]
    fn shape_and_grid_fields_patch_only_the_selected_descriptor() {
        let shape: TimelineItem = serde_json::from_value(json!({
            "type":"shape","id":"shape","startMs":0,"durationMs":1000,"keyframes":[],
            "geometry":{"type":"roundedRectangle","width":100,"height":60,"radii":{"topLeft":4,"topRight":5,"bottomRight":6,"bottomLeft":7}},
            "fill":{"type":"solid","color":{"r":1,"g":0,"b":0,"a":0.5}},"stroke":null
        })).unwrap();
        let fields = fields(&shape);
        assert!(
            fields
                .iter()
                .any(|field| field.label == "Top left radius · px")
        );
        let edit = build(&shape, field(&fields, "Width · local px"), "120").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["geometry"]["width"], 120.0);
        assert_eq!(value["geometry"]["radii"]["topLeft"], 4.0);
        assert!(value.get("fill").is_none());
        let edit = build(&shape, field(&fields, "Fill · #RRGGBB"), "#00FF00").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["fill"]["color"]["a"], 0.5);
        assert_eq!(value["fill"]["color"]["g"], 1.0);
        assert!(build(&shape, field(&fields, "Width · local px"), "bad").is_err());

        let grid: TimelineItem = serde_json::from_value(json!({
            "type":"grid","id":"grid","startMs":0,"durationMs":1000,"keyframes":[],
            "transform":{"positionX":17,"positionY":23,"scale":1.5,"opacity":0.6},
            "grid":{"width":100,"height":60,"pattern":{"type":"diagonal","spacing":12,"stroke":{
                "paint":{"type":"solid","color":{"r":1,"g":1,"b":1,"a":1}},
                "width":2,"dash":[],"dashOffset":0,"lineCap":"butt","lineJoin":"miter","miterLimit":4
            }}}
        })).unwrap();
        let fields = super::fields(&grid);
        let edit = build(&grid, field(&fields, "Angle · degrees"), "30").unwrap();
        let transformed = serde_json::to_value(edit).unwrap();
        assert_eq!(transformed["transform2d"]["rotationDeg"], 30.0);
        assert_eq!(transformed["transform2d"]["position"]["x"], 17.0);
        assert_eq!(transformed["transform2d"]["position"]["y"], 23.0);
        assert_eq!(transformed["transform2d"]["scaleX"], 1.5);
        assert_eq!(transformed["transform2d"]["opacity"], 0.6);
        let edit = build(&grid, field(&fields, "Spacing · px"), "18").unwrap();
        assert_eq!(
            serde_json::to_value(edit).unwrap()["grid"]["pattern"]["spacing"],
            18.0
        );
    }

    #[test]
    fn text_fields_preserve_documents_and_style() {
        let item: TimelineItem = serde_json::from_value(json!({
            "type":"text","id":"title","text":"Hi!","document":{"runs":[{"text":"Hi","bold":true},{"text":"!"}]},
            "startMs":0,"durationMs":1000,"fontSize":24,"color":"#ffffff","fontFamily":null,
            "style":{"outlineColor":"#000000","outlineWidthPx":2},"keyframes":[]
        })).unwrap();
        let fields = fields(&item);
        let edit = build(&item, field(&fields, "Font size · px"), "48").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["fontSize"], 48);
        assert!(value.get("document").is_none());
        let edit = build(&item, field(&fields, "Run 2 italic"), "true").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["document"]["runs"][0]["bold"], true);
        assert_eq!(value["document"]["runs"][1]["italic"], true);
        let edit = build(&item, field(&fields, "Tracking · px"), "1.5").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["style"]["layout"]["trackingPx"], 1.5);
        assert_eq!(value["style"]["outlineWidthPx"], 2);
        let edit = build(&item, field(&fields, "Shadow X · px"), "-12").unwrap();
        let value = serde_json::to_value(edit).unwrap();
        assert_eq!(value["style"]["shadow"]["offsetX"], -12);
        assert_eq!(value["style"]["outlineWidthPx"], 2);
        assert!(build(&item, field(&fields, "Font size · px"), "2.5").is_err());
    }
}
