//! Streaming, offline SVG normalization. Never resolves resources or emits source excerpts.
use crate::{
    CoreError, CornerRadii, ErrorCode, FillRule, LineCap, LineJoin, MAX_SVG_ATTRIBUTES,
    MAX_SVG_BYTES, MAX_SVG_COMMANDS, MAX_SVG_DEPTH, MAX_SVG_ELEMENTS, MAX_VECTOR_COORDINATE,
    MAX_VECTOR_DIMENSION, MAX_VECTOR_PATH_COMMANDS, Paint, PathCommand, ShapeGeometry, Stroke,
    SvgDocument, SvgShape, VectorColor, VectorPath, VectorPoint, validate_shape,
};
use quick_xml::{Reader, events::Event};
use std::collections::BTreeMap;

fn invalid(category: &str) -> CoreError {
    CoreError::new(
        ErrorCode::InvalidArgument,
        format!("invalid SVG: {category}"),
    )
}
fn number(s: &str) -> Result<f64, CoreError> {
    let v: f64 = s.trim().parse().map_err(|_| invalid("number"))?;
    if !v.is_finite() || v.abs() > MAX_VECTOR_COORDINATE {
        return Err(invalid("numeric bounds"));
    }
    Ok(v)
}
fn length(s: &str) -> Result<f64, CoreError> {
    number(s.trim().strip_suffix("px").unwrap_or(s))
}
fn dimension(v: f64) -> Result<(), CoreError> {
    if !v.is_finite() || v <= 0.0 || v > MAX_VECTOR_DIMENSION {
        return Err(invalid("dimension"));
    }
    Ok(())
}
fn paint(s: &str) -> Result<Option<Paint>, CoreError> {
    if s == "none" {
        return Ok(None);
    }
    let hex = s
        .strip_prefix('#')
        .ok_or_else(|| invalid("paint or resource"))?;
    if !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("paint or resource"));
    }
    let mut c = [255_u8; 4];
    let short = hex.len() <= 4;
    for (i, value) in c
        .iter_mut()
        .enumerate()
        .take(if short { hex.len() } else { hex.len() / 2 })
    {
        *value = if short {
            u8::from_str_radix(&hex[i..i + 1], 16).unwrap() * 17
        } else {
            u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()
        };
    }
    Ok(Some(Paint::Solid {
        color: VectorColor {
            r: f64::from(c[0]) / 255.,
            g: f64::from(c[1]) / 255.,
            b: f64::from(c[2]) / 255.,
            a: f64::from(c[3]) / 255.,
        },
    }))
}

// Small SVG number lexer: no expression evaluator, implicit units, or coercion.
struct Numbers<'a> {
    s: &'a str,
    pos: usize,
}
impl<'a> Numbers<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, pos: 0 }
    }
    fn spaces(&mut self) {
        while self
            .s
            .as_bytes()
            .get(self.pos)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }
    fn done(&mut self) -> bool {
        self.spaces();
        self.pos == self.s.len()
    }
    fn next(&mut self) -> Result<f64, CoreError> {
        self.spaces();
        let b = self.s.as_bytes();
        if b.get(self.pos) == Some(&b',') {
            self.pos += 1;
            self.spaces();
        }
        let start = self.pos;
        if matches!(b.get(self.pos), Some(b'+' | b'-')) {
            self.pos += 1;
        }
        let mut digits = 0;
        while b.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
            digits += 1;
        }
        if b.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            while b.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(invalid("number grammar"));
        }
        if matches!(b.get(self.pos), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(b.get(self.pos), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            let exp = self.pos;
            while b.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
            }
            if exp == self.pos {
                return Err(invalid("exponent"));
            }
        }
        number(&self.s[start..self.pos])
    }
    fn point(&mut self) -> Result<VectorPoint, CoreError> {
        Ok(VectorPoint {
            x: self.next()?,
            y: self.next()?,
        })
    }
}
fn numbers(s: &str, max: usize) -> Result<Vec<f64>, CoreError> {
    let mut n = Numbers::new(s);
    let mut result = vec![];
    if s.trim_start().starts_with(',') {
        return Err(invalid("number list"));
    }
    while !n.done() {
        if result.len() == max {
            return Err(invalid("number list budget"));
        }
        result.push(n.next()?);
    }
    Ok(result)
}
#[cfg(test)]
fn path(s: &str, fill_rule: FillRule) -> Result<VectorPath, CoreError> {
    path_with_budget(s, fill_rule, MAX_VECTOR_PATH_COMMANDS)
}
fn path_with_budget(s: &str, fill_rule: FillRule, budget: usize) -> Result<VectorPath, CoreError> {
    let mut n = Numbers::new(s);
    let mut commands = vec![];
    let mut command = b' ';
    let mut current = VectorPoint { x: 0., y: 0. };
    let mut initial = current;
    let mut closed = false;
    let budget = budget.min(MAX_VECTOR_PATH_COMMANDS);
    while !n.done() {
        if commands.len() == budget {
            return Err(invalid("path command budget"));
        }
        if n.s.as_bytes()[n.pos].is_ascii_alphabetic() {
            command = n.s.as_bytes()[n.pos];
            n.pos += 1;
            n.spaces();
            if n.s.as_bytes().get(n.pos) == Some(&b',') {
                return Err(invalid("path separator"));
            }
        }
        if closed && matches!(command, b'L' | b'H' | b'V' | b'Q' | b'C') {
            if budget.saturating_sub(commands.len()) < 2 {
                return Err(invalid("path command budget"));
            }
            commands.push(PathCommand::MoveTo { to: initial });
            closed = false;
        }
        let value = match command {
            b'M' => {
                current = n.point()?;
                initial = current;
                closed = false;
                command = b'L';
                PathCommand::MoveTo { to: current }
            }
            b'L' => {
                current = n.point()?;
                PathCommand::LineTo { to: current }
            }
            b'H' => {
                current.x = n.next()?;
                PathCommand::LineTo { to: current }
            }
            b'V' => {
                current.y = n.next()?;
                PathCommand::LineTo { to: current }
            }
            b'Q' => {
                let control = n.point()?;
                current = n.point()?;
                PathCommand::QuadraticTo {
                    control,
                    to: current,
                }
            }
            b'C' => {
                let control1 = n.point()?;
                let control2 = n.point()?;
                current = n.point()?;
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to: current,
                }
            }
            b'Z' => {
                current = initial;
                closed = true;
                command = b' ';
                PathCommand::Close {}
            }
            _ => return Err(invalid("unsupported path command")),
        };
        commands.push(value);
    }
    let path = VectorPath {
        fill_rule,
        commands,
    };
    path.validate()?;
    Ok(path)
}

#[derive(Clone)]
struct Style {
    fill: Option<Paint>,
    stroke: Option<Paint>,
    metrics: Stroke,
    fill_rule: FillRule,
}
impl Default for Style {
    fn default() -> Self {
        let black = Paint::Solid {
            color: VectorColor {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
        };
        Self {
            fill: Some(black.clone()),
            stroke: None,
            fill_rule: FillRule::Nonzero,
            metrics: Stroke {
                paint: black,
                width: 1.,
                dash: vec![],
                dash_offset: 0.,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Miter,
                miter_limit: 4.,
            },
        }
    }
}
impl Style {
    fn set(&mut self, key: &str, value: &str) -> Result<bool, CoreError> {
        match key {
            "fill" => self.fill = paint(value)?,
            "stroke" => self.stroke = paint(value)?,
            "fill-rule" => {
                self.fill_rule = match value {
                    "nonzero" => FillRule::Nonzero,
                    "evenodd" => FillRule::Evenodd,
                    _ => return Err(invalid("fill rule")),
                }
            }
            "stroke-width" => self.metrics.width = length(value)?,
            "stroke-miterlimit" => self.metrics.miter_limit = number(value)?,
            "stroke-dashoffset" => self.metrics.dash_offset = length(value)?,
            "stroke-dasharray" => {
                self.metrics.dash = if value == "none" {
                    vec![]
                } else {
                    numbers(value, 64)?
                }
            }
            "stroke-linecap" => {
                self.metrics.line_cap = match value {
                    "butt" => LineCap::Butt,
                    "round" => LineCap::Round,
                    "square" => LineCap::Square,
                    _ => return Err(invalid("line cap")),
                }
            }
            "stroke-linejoin" => {
                self.metrics.line_join = match value {
                    "miter" => LineJoin::Miter,
                    "round" => LineJoin::Round,
                    "bevel" => LineJoin::Bevel,
                    _ => return Err(invalid("line join")),
                }
            }
            _ => return Ok(false),
        }
        self.metrics.validate()?;
        Ok(true)
    }
}
type Attributes = BTreeMap<String, String>;
fn get(a: &Attributes, key: &str, default: Option<f64>) -> Result<f64, CoreError> {
    a.get(key)
        .map(|s| length(s))
        .unwrap_or_else(|| default.ok_or_else(|| invalid("missing geometry")))
}
fn geometry(
    tag: &str,
    a: &Attributes,
    style: &Style,
    budget: usize,
) -> Result<SvgShape, CoreError> {
    let primitive_count = match tag {
        "rect" => {
            if get(a, "rx", Some(get(a, "ry", Some(0.))?))? != 0. {
                10
            } else {
                5
            }
        }
        "circle" | "ellipse" => 6,
        "line" => 2,
        _ => 0,
    };
    if primitive_count > budget {
        return Err(invalid("document command budget"));
    }
    let mut offset = VectorPoint { x: 0., y: 0. };
    let geometry = match tag {
        "rect" => {
            offset = VectorPoint {
                x: get(a, "x", Some(0.))?,
                y: get(a, "y", Some(0.))?,
            };
            let width = get(a, "width", None)?;
            let height = get(a, "height", None)?;
            let rx = get(a, "rx", Some(get(a, "ry", Some(0.))?))?;
            let ry = get(a, "ry", Some(rx))?;
            if rx < 0. || rx != ry {
                return Err(invalid("rectangle radii"));
            }
            if rx == 0. {
                ShapeGeometry::Rectangle { width, height }
            } else {
                let r = rx.min(width.min(height) * 0.5);
                ShapeGeometry::RoundedRectangle {
                    width,
                    height,
                    radii: CornerRadii {
                        top_left: r,
                        top_right: r,
                        bottom_left: r,
                        bottom_right: r,
                    },
                }
            }
        }
        "circle" | "ellipse" => {
            let rx = get(a, if tag == "circle" { "r" } else { "rx" }, None)?;
            let ry = if tag == "circle" {
                rx
            } else {
                get(a, "ry", None)?
            };
            offset = VectorPoint {
                x: get(a, "cx", Some(0.))? - rx,
                y: get(a, "cy", Some(0.))? - ry,
            };
            ShapeGeometry::Ellipse {
                width: rx * 2.,
                height: ry * 2.,
            }
        }
        "line" => ShapeGeometry::Line {
            start: VectorPoint {
                x: get(a, "x1", Some(0.))?,
                y: get(a, "y1", Some(0.))?,
            },
            end: VectorPoint {
                x: get(a, "x2", Some(0.))?,
                y: get(a, "y2", Some(0.))?,
            },
        },
        "polygon" | "polyline" => {
            let values = numbers(
                a.get("points").ok_or_else(|| invalid("missing points"))?,
                8192.min(budget.saturating_sub(usize::from(tag == "polygon")) * 2),
            )?;
            if values.len() % 2 != 0 || values.len() < if tag == "polygon" { 6 } else { 4 } {
                return Err(invalid("points"));
            }
            let mut commands = Vec::new();
            for p in values.as_chunks::<2>().0 {
                if commands.len() == MAX_VECTOR_PATH_COMMANDS {
                    return Err(invalid("path command budget"));
                }
                let to = VectorPoint { x: p[0], y: p[1] };
                commands.push(if commands.is_empty() {
                    PathCommand::MoveTo { to }
                } else {
                    PathCommand::LineTo { to }
                });
            }
            // Polygon uses its native implicit closure to retain the 4096-point bound.
            if tag == "polygon" && style.fill_rule == FillRule::Nonzero {
                ShapeGeometry::Polygon {
                    points: values
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|p| VectorPoint { x: p[0], y: p[1] })
                        .collect(),
                }
            } else {
                if tag == "polygon" {
                    if commands.len() == MAX_VECTOR_PATH_COMMANDS {
                        return Err(invalid("path command budget"));
                    }
                    commands.push(PathCommand::Close {});
                }
                ShapeGeometry::Path {
                    path: VectorPath {
                        commands,
                        fill_rule: style.fill_rule,
                    },
                }
            }
        }
        "path" => ShapeGeometry::Path {
            path: path_with_budget(
                a.get("d").ok_or_else(|| invalid("missing path"))?,
                style.fill_rule,
                budget,
            )?,
        },
        _ => return Err(invalid("unsupported element")),
    };
    let fill = if tag == "line" {
        None
    } else {
        style.fill.clone()
    };
    let stroke = style.stroke.clone().map(|paint| Stroke {
        paint,
        ..style.metrics.clone()
    });
    let result = SvgShape {
        geometry,
        offset,
        fill,
        stroke,
    };
    validate_shape(&result.geometry, &result.fill, &result.stroke)?;
    result.offset.validate()?;
    Ok(result)
}
pub(crate) fn command_count(g: &ShapeGeometry) -> usize {
    match g {
        ShapeGeometry::Path { path } => path.commands.len(),
        ShapeGeometry::Polygon { points } => points.len() + 1,
        ShapeGeometry::Rectangle { .. } => 5,
        ShapeGeometry::RoundedRectangle { .. } => 10,
        ShapeGeometry::Ellipse { .. } => 6,
        ShapeGeometry::Line { .. } => 2,
        ShapeGeometry::Star { .. } => usize::MAX,
    }
}
pub(crate) fn validate_document(doc: &SvgDocument) -> Result<(), CoreError> {
    if doc.version != 1 || doc.shapes.len() >= MAX_SVG_ELEMENTS {
        return Err(invalid("document version or element budget"));
    }
    dimension(doc.width)?;
    dimension(doc.height)?;
    for v in doc.view_box {
        if !v.is_finite() || v.abs() > MAX_VECTOR_COORDINATE {
            return Err(invalid("viewBox"));
        }
    }
    if doc.view_box[2] <= 0. || doc.view_box[3] <= 0. {
        return Err(invalid("viewBox extents"));
    }
    let scale = (doc.width / doc.view_box[2]).min(doc.height / doc.view_box[3]);
    if !scale.is_finite() || scale <= 0. {
        return Err(invalid("viewBox mapping"));
    }
    let mut count = 0_usize;
    for s in &doc.shapes {
        s.offset.validate()?;
        validate_shape(&s.geometry, &s.fill, &s.stroke)?;
        if s.fill
            .iter()
            .chain(s.stroke.iter().map(|s| &s.paint))
            .any(|p| !matches!(p, Paint::Solid { .. }))
        {
            return Err(invalid("normalized paint"));
        }
        count = count
            .checked_add(command_count(&s.geometry))
            .ok_or_else(|| invalid("document command budget"))?;
        if count > MAX_SVG_COMMANDS {
            return Err(invalid("document command budget"));
        }
    }
    Ok(())
}

pub(crate) fn parse(source: &str) -> Result<SvgDocument, CoreError> {
    if source.len() > MAX_SVG_BYTES {
        return Err(invalid("source byte budget"));
    }
    let mut reader = Reader::from_str(source);
    reader.config_mut().check_comments = true;
    let mut stack: Vec<(String, Style)> = vec![];
    let mut doc = None;
    let mut elements = 0;
    let mut commands = 0_usize;
    let mut declaration = false;
    loop {
        let event = reader.read_event().map_err(|_| invalid("XML structure"))?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                if stack.len() == MAX_SVG_DEPTH || elements == MAX_SVG_ELEMENTS {
                    return Err(invalid("XML complexity"));
                }
                elements += 1;
                let name = e.name();
                let tag =
                    std::str::from_utf8(name.as_ref()).map_err(|_| invalid("element encoding"))?;
                if stack.is_empty() && (doc.is_some() || tag != "svg")
                    || !stack.is_empty() && tag == "svg"
                {
                    return Err(invalid("root element"));
                }
                if !matches!(
                    tag,
                    "svg"
                        | "g"
                        | "rect"
                        | "circle"
                        | "ellipse"
                        | "line"
                        | "polygon"
                        | "polyline"
                        | "path"
                ) {
                    return Err(invalid("unsupported element or namespace"));
                }
                if stack
                    .last()
                    .is_some_and(|(tag, _)| !matches!(tag.as_str(), "svg" | "g"))
                {
                    return Err(invalid("element nesting"));
                }
                let mut style = stack.last().map_or_else(Style::default, |(_, s)| s.clone());
                let mut attrs = Attributes::new();
                for (i, a) in e.attributes().enumerate() {
                    if i == MAX_SVG_ATTRIBUTES {
                        return Err(invalid("attribute budget"));
                    }
                    let a = a.map_err(|_| invalid("XML attribute"))?;
                    let key = std::str::from_utf8(a.key.as_ref())
                        .map_err(|_| invalid("attribute encoding"))?;
                    let value = a
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|_| invalid("attribute entity"))?;
                    if key == "xmlns" && tag == "svg" && value == "http://www.w3.org/2000/svg" {
                        continue;
                    }
                    if style.set(key, &value)? {
                        continue;
                    }
                    let allowed = match tag {
                        "svg" => {
                            matches!(key, "width" | "height" | "viewBox" | "preserveAspectRatio")
                        }
                        "rect" => matches!(key, "x" | "y" | "width" | "height" | "rx" | "ry"),
                        "circle" => matches!(key, "cx" | "cy" | "r"),
                        "ellipse" => matches!(key, "cx" | "cy" | "rx" | "ry"),
                        "line" => matches!(key, "x1" | "y1" | "x2" | "y2"),
                        "polygon" | "polyline" => key == "points",
                        "path" => key == "d",
                        _ => false,
                    };
                    if !allowed {
                        return Err(invalid("unsupported attribute or resource"));
                    }
                    attrs.insert(key.to_owned(), value.into_owned());
                }
                if tag == "svg" {
                    let width = get(&attrs, "width", None)?;
                    let height = get(&attrs, "height", None)?;
                    dimension(width)?;
                    dimension(height)?;
                    let view_box = if let Some(v) = attrs.get("viewBox") {
                        numbers(v, 4)?.try_into().map_err(|_| invalid("viewBox"))?
                    } else {
                        [0., 0., width, height]
                    };
                    if attrs
                        .get("preserveAspectRatio")
                        .is_some_and(|v| v != "xMidYMid meet")
                    {
                        return Err(invalid("aspect ratio"));
                    }
                    doc = Some(SvgDocument {
                        version: 1,
                        width,
                        height,
                        view_box,
                        shapes: vec![],
                    });
                    validate_document(doc.as_ref().unwrap())?;
                } else if tag != "g" {
                    let shape = geometry(tag, &attrs, &style, MAX_SVG_COMMANDS - commands)?;
                    commands = commands
                        .checked_add(command_count(&shape.geometry))
                        .ok_or_else(|| invalid("document command budget"))?;
                    if commands > MAX_SVG_COMMANDS {
                        return Err(invalid("document command budget"));
                    }
                    doc.as_mut().unwrap().shapes.push(shape);
                }
                if !empty {
                    stack.push((tag.to_owned(), style));
                }
            }
            Event::End(_) => {
                stack.pop().ok_or_else(|| invalid("closing element"))?;
            }
            Event::Decl(e) => {
                if declaration
                    || elements != 0
                    || e.version()
                        .map_err(|_| invalid("XML declaration"))?
                        .as_ref()
                        != b"1.0"
                {
                    return Err(invalid("XML declaration"));
                }
                if let Some(encoding) = e.encoding()
                    && !encoding
                        .map_err(|_| invalid("encoding"))?
                        .eq_ignore_ascii_case(b"UTF-8")
                {
                    return Err(invalid("encoding"));
                }
                declaration = true;
            }
            Event::Comment(_) => {}
            Event::Text(e) => {
                if !e
                    .xml_content()
                    .map_err(|_| invalid("text encoding"))?
                    .chars()
                    .all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
                {
                    return Err(invalid("text or font content"));
                }
            }
            Event::GeneralRef(e) => {
                if !e
                    .resolve_char_ref()
                    .map_err(|_| invalid("text entity"))?
                    .is_some_and(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
                {
                    return Err(invalid("text entity"));
                }
            }
            Event::Eof => break,
            _ => {
                return Err(invalid(
                    "DTD, processing instruction or unsupported content",
                ));
            }
        }
    }
    if !stack.is_empty() {
        return Err(invalid("unclosed element"));
    }
    let doc = doc.ok_or_else(|| invalid("missing root"))?;
    validate_document(&doc)?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wrap(body: &str) -> String {
        format!("<svg width=\"40\" height=\"20\">{body}</svg>")
    }
    #[test]
    fn svg_close_continuations_and_normalized_budgets() {
        for suffix in ["L40 50", "H40", "V50", "Q30 40 40 50", "C20 30 30 40 40 50"] {
            let p = path(&format!("M10 11 L20 21 Z {suffix}"), FillRule::Nonzero).unwrap();
            assert_eq!(
                p.commands[3],
                PathCommand::MoveTo {
                    to: VectorPoint { x: 10., y: 11. }
                }
            );
            if suffix == "H40" {
                assert_eq!(
                    p.commands[4],
                    PathCommand::LineTo {
                        to: VectorPoint { x: 40., y: 11. }
                    }
                );
            }
            if suffix == "V50" {
                assert_eq!(
                    p.commands[4],
                    PathCommand::LineTo {
                        to: VectorPoint { x: 10., y: 50. }
                    }
                );
            }
        }
        assert_eq!(
            path("M10 11 L20 21 Z M30 31 L40 41", FillRule::Nonzero)
                .unwrap()
                .commands
                .len(),
            5
        );
        let d = format!("M10 11 L20 21 Z H40 {}", "L40 41 ".repeat(4091));
        assert_eq!(path(&d, FillRule::Nonzero).unwrap().commands.len(), 4096);
        assert!(path(&format!("{d} L1 1"), FillRule::Nonzero).is_err());
        assert!(path_with_budget("M0 0 L1 1 Z H2", FillRule::Nonzero, 4).is_err());
        let child = format!("<path d=\"{d}\"/>");
        assert!(parse(&wrap(&child.repeat(16))).is_ok());
        assert!(parse(&wrap(&(child.repeat(16) + "<path d=\"M0 0\"/>"))).is_err());
    }
    #[test]
    fn hostile_xml_and_unsupported_features_never_normalize() {
        for body in [
            "<script/>",
            "<foreignObject/>",
            "<image href=\"file:///private\"/>",
            "<text>secret</text>",
            "<style/>",
            "<use/>",
            "<filter/>",
            "<mask/>",
            "<clipPath/>",
            "<linearGradient/>",
            "<animate/>",
            "<svg/>",
            "<?secret data?>",
            "<!DOCTYPE svg>",
            "<![CDATA[secret]]>",
            "&secret;",
            "<rect onload=\"secret\"/>",
            "<rect xmlns=\"other\"/>",
            "<rect width=\"1\" width=\"2\"/>",
            "<g></rect>",
            "<g>",
            "<rect style=\"fill:#fff\"/>",
            "<path d=\"m0 0\"/>",
            "<path d=\"M0 0 A1 1 0 0 0 1 1\"/>",
            "<rect width=\"1\" height=\"1\" fill=\"&#117;rl(file:///private)\"/>",
        ] {
            let err = parse(&wrap(body)).unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidArgument, "{body}");
            assert!(!err.message.contains("private"));
            assert!(!err.message.contains("secret"));
        }
        for source in [
            "",
            "<g/>",
            "<svg width=\"1\" height=\"1\"/><svg width=\"1\" height=\"1\"/>",
            "<!DOCTYPE svg [<!ENTITY x SYSTEM 'https://example.org'>]><svg width=\"1\" height=\"1\"/>",
        ] {
            assert!(parse(source).is_err());
        }
    }
    #[test]
    fn geometry_styles_and_viewbox_have_independent_oracles() {
        let doc=parse("<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg xmlns=\"http://www.w3.org/2000/svg\" width=\"80px\" height=\"20\" viewBox=\"10 20 20 20\"><!-- inert --><g fill=\"#f008\" stroke=\"#12345678\" stroke-width=\"2\"><rect x=\"3\" y=\"4\" width=\"12\" height=\"8\" rx=\"10\"/><ellipse cx=\"5\" cy=\"6\" rx=\"2\" ry=\"3\"/><line x2=\"8\"/><polygon points=\"0,0 4,0 4,4\"/><polyline points=\"0,0 4,4\"/><path d=\"M0 0 2 2 H3 V4 Q5 6 7 8 C1 2 3 4 5 6 Z\"/></g></svg>").unwrap();
        assert_eq!(doc.view_box, [10., 20., 20., 20.]);
        assert_eq!(doc.shapes.len(), 6);
        assert_eq!(doc.shapes[0].offset, VectorPoint { x: 3., y: 4. });
        let ShapeGeometry::RoundedRectangle { radii, .. } = &doc.shapes[0].geometry else {
            panic!()
        };
        assert_eq!(radii.top_left, 4.);
        assert_eq!(doc.shapes[1].offset, VectorPoint { x: 3., y: 3. });
        assert!(doc.shapes[2].fill.is_none());
        assert_eq!(doc.shapes[0].stroke.as_ref().unwrap().width, 2.);
        assert_eq!(doc.shapes[0].fill, paint("#ff000088").unwrap());
        assert_eq!(
            parse(&wrap("<rect width=\"1\" height=\"1\" fill=\"&#35;f00\"/>"))
                .unwrap()
                .shapes[0]
                .fill,
            paint("#f00").unwrap()
        );
    }
    #[test]
    fn every_numeric_and_work_budget_is_bounded() {
        let minimal = "<svg width=\"1\" height=\"1\"/>";
        assert!(
            parse(&format!(
                "{}{}",
                minimal,
                " ".repeat(MAX_SVG_BYTES - minimal.len())
            ))
            .is_ok()
        );
        assert!(
            parse(&format!(
                "{}{}",
                minimal,
                " ".repeat(MAX_SVG_BYTES - minimal.len() + 1)
            ))
            .is_err()
        );
        for (depth, valid) in [(31, true), (32, false)] {
            assert_eq!(
                parse(&wrap(&format!(
                    "{}{}",
                    "<g>".repeat(depth),
                    "</g>".repeat(depth)
                )))
                .is_ok(),
                valid
            );
        }
        for (count, valid) in [(4095, true), (4096, false)] {
            assert_eq!(parse(&wrap(&"<g/>".repeat(count))).is_ok(), valid);
        }
        for (count, valid) in [(4096, true), (4097, false)] {
            let d = format!("M0 0 {}", "L1 1 ".repeat(count - 1));
            assert_eq!(path(&d, FillRule::Nonzero).is_ok(), valid);
        }
        let d = format!("M0 0 {}", "L1 1 ".repeat(4095));
        let child = format!("<path d=\"{d}\"/>");
        assert!(parse(&wrap(&child.repeat(16))).is_ok());
        assert!(parse(&wrap(&child.repeat(17))).is_err());
        let attrs = (0..33).map(|i| format!(" x{i}=\"1\"")).collect::<String>();
        assert!(parse(&wrap(&format!("<g{attrs}/>"))).is_err());
        for value in ["NaN", "inf", "1e309", "0", "-1", "16385", "10%", "1em"] {
            assert!(parse(&format!("<svg width=\"{value}\" height=\"1\"/>")).is_err());
        }
        assert!(parse("<svg width=\"16384\" height=\"1\"/>").is_ok());
        for d in [
            "L0 0",
            "M,0 0",
            "M0 0,",
            "M0 0 Z Z",
            "M0 0 Q1 1",
            "M0 0 LNaN 1",
        ] {
            assert!(path(d, FillRule::Nonzero).is_err(), "{d}");
        }
    }
}
