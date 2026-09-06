//! Reusable, reference-free vector data. Timeline/render activation belongs to ShapeItem.
//! See docs/vector-primitives.md for coordinate, color, stroke, and fill semantics.
use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Serialize};

// Force the map visitor even when a format also supports positional struct sequences.
// Streaming MapAccess preserves duplicate fields for the derived Fields decoder.
fn deserialize_object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for ObjectVisitor<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an object with named fields")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(ObjectVisitor(std::marker::PhantomData))
}

pub const MAX_VECTOR_COORDINATE: f64 = 1_000_000.;
pub const MAX_VECTOR_GRADIENT_STOPS: usize = 64;
pub const MAX_VECTOR_DASH_ENTRIES: usize = 64;
pub const MAX_VECTOR_PATH_COMMANDS: usize = 4096;
pub const MAX_VECTOR_DIMENSION: f64 = 16384.;
pub const MAX_VECTOR_MITER_LIMIT: f64 = 1000.;

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}
fn bounded(value: f64, min: f64, max: f64, name: &str) -> Result<(), CoreError> {
    if !value.is_finite() || value < min || value > max {
        return Err(invalid(name));
    }
    Ok(())
}
fn positive(value: f64, max: f64, name: &str) -> Result<(), CoreError> {
    bounded(value, 0., max, name)?;
    if value == 0. {
        return Err(invalid(name));
    }
    Ok(())
}

/// Unassociated sRGB channels and linear alpha, all in [0, 1].
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct VectorColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl<'de> Deserialize<'de> for VectorColor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            r: f64,
            g: f64,
            b: f64,
            a: f64,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            r: fields.r,
            g: fields.g,
            b: fields.b,
            a: fields.a,
        })
    }
}
impl VectorColor {
    pub fn validate(&self) -> Result<(), CoreError> {
        for v in [self.r, self.g, self.b, self.a] {
            bounded(v, 0., 1., "color channel must be finite and in [0,1]")?;
        }
        Ok(())
    }
}

/// Local pixels, top-left origin, positive X right and Y down.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct VectorPoint {
    pub x: f64,
    pub y: f64,
}

impl<'de> Deserialize<'de> for VectorPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            x: f64,
            y: f64,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            x: fields.x,
            y: fields.y,
        })
    }
}
impl VectorPoint {
    pub fn validate(&self) -> Result<(), CoreError> {
        for v in [self.x, self.y] {
            bounded(
                v,
                -MAX_VECTOR_COORDINATE,
                MAX_VECTOR_COORDINATE,
                "point coordinate out of range",
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GradientStop {
    pub offset: f64,
    pub color: VectorColor,
}

impl<'de> Deserialize<'de> for GradientStop {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            offset: f64,
            color: VectorColor,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            offset: fields.offset,
            color: fields.color,
        })
    }
}
impl GradientStop {
    pub fn validate(&self) -> Result<(), CoreError> {
        bounded(self.offset, 0., 1., "gradient offset out of range")?;
        self.color.validate()
    }
}

/// Gradients use pad extension and premultiplied linear-light interpolation.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Paint {
    Solid {
        color: VectorColor,
    },
    LinearGradient {
        start: VectorPoint,
        end: VectorPoint,
        stops: Vec<GradientStop>,
    },
    RadialGradient {
        center: VectorPoint,
        radius: f64,
        stops: Vec<GradientStop>,
    },
}

impl<'de> Deserialize<'de> for Paint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
        enum Fields {
            Solid {
                color: VectorColor,
            },
            LinearGradient {
                start: VectorPoint,
                end: VectorPoint,
                stops: Vec<GradientStop>,
            },
            RadialGradient {
                center: VectorPoint,
                radius: f64,
                stops: Vec<GradientStop>,
            },
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(match fields {
            Fields::Solid { color } => Self::Solid { color },
            Fields::LinearGradient { start, end, stops } => {
                Self::LinearGradient { start, end, stops }
            }
            Fields::RadialGradient {
                center,
                radius,
                stops,
            } => Self::RadialGradient {
                center,
                radius,
                stops,
            },
        })
    }
}
impl Paint {
    pub fn validate(&self) -> Result<(), CoreError> {
        let stops = match self {
            Self::Solid { color } => return color.validate(),
            Self::LinearGradient { stops, .. } | Self::RadialGradient { stops, .. } => stops,
        };
        if !(2..=MAX_VECTOR_GRADIENT_STOPS).contains(&stops.len()) {
            return Err(invalid("gradient stops must contain 2 through 64 entries"));
        }
        match self {
            Self::LinearGradient { start, end, .. } => {
                start.validate()?;
                end.validate()?;
                if start == end {
                    return Err(invalid("linear gradient endpoints must differ"));
                }
            }
            Self::RadialGradient { center, radius, .. } => {
                center.validate()?;
                positive(*radius, MAX_VECTOR_COORDINATE, "radial radius out of range")?;
            }
            Self::Solid { .. } => unreachable!(),
        }
        for stop in stops {
            stop.validate()?;
        }
        if stops[0].offset != 0.
            || stops[stops.len() - 1].offset != 1.
            || stops
                .windows(2)
                .any(|pair| pair[0].offset >= pair[1].offset)
        {
            return Err(invalid("gradient stops must increase strictly from 0 to 1"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

impl<'de> Deserialize<'de> for LineCap {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "butt" => Ok(Self::Butt),
            "round" => Ok(Self::Round),
            "square" => Ok(Self::Square),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["butt", "round", "square"],
            )),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl<'de> Deserialize<'de> for LineJoin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "miter" => Ok(Self::Miter),
            "round" => Ok(Self::Round),
            "bevel" => Ok(Self::Bevel),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["miter", "round", "bevel"],
            )),
        }
    }
}

/// Centered stroke. Dash phase restarts per subpath; positive offset advances it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub paint: Paint,
    pub width: f64,
    pub dash: Vec<f64>,
    pub dash_offset: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
}

impl<'de> Deserialize<'de> for Stroke {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            paint: Paint,
            width: f64,
            dash: Vec<f64>,
            dash_offset: f64,
            line_cap: LineCap,
            line_join: LineJoin,
            miter_limit: f64,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            paint: fields.paint,
            width: fields.width,
            dash: fields.dash,
            dash_offset: fields.dash_offset,
            line_cap: fields.line_cap,
            line_join: fields.line_join,
            miter_limit: fields.miter_limit,
        })
    }
}
impl Stroke {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.dash.len() > MAX_VECTOR_DASH_ENTRIES || !self.dash.len().is_multiple_of(2) {
            return Err(invalid(
                "dash must contain an even number of at most 64 entries",
            ));
        }
        self.paint.validate()?;
        positive(
            self.width,
            MAX_VECTOR_DIMENSION,
            "stroke width out of range",
        )?;
        bounded(
            self.dash_offset,
            -MAX_VECTOR_COORDINATE,
            MAX_VECTOR_COORDINATE,
            "dash offset out of range",
        )?;
        bounded(
            self.miter_limit,
            1.,
            MAX_VECTOR_MITER_LIMIT,
            "miter limit out of range",
        )?;
        for value in &self.dash {
            positive(*value, MAX_VECTOR_COORDINATE, "dash entry out of range")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CornerRadii {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl<'de> Deserialize<'de> for CornerRadii {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            top_left: f64,
            top_right: f64,
            bottom_right: f64,
            bottom_left: f64,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            top_left: fields.top_left,
            top_right: fields.top_right,
            bottom_right: fields.bottom_right,
            bottom_left: fields.bottom_left,
        })
    }
}
impl CornerRadii {
    pub fn validate(&self) -> Result<(), CoreError> {
        for value in [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ] {
            bounded(
                value,
                0.,
                MAX_VECTOR_DIMENSION,
                "corner radius out of range",
            )?;
        }
        Ok(())
    }
    /// Resolve all corners with one common factor, preserving the supplied radii.
    pub fn resolve(&self, width: f64, height: f64) -> Result<Self, CoreError> {
        self.validate()?;
        positive(width, MAX_VECTOR_DIMENSION, "rectangle width out of range")?;
        positive(
            height,
            MAX_VECTOR_DIMENSION,
            "rectangle height out of range",
        )?;
        let mut factor = 1_f64;
        for (extent, sum) in [
            (width, self.top_left + self.top_right),
            (width, self.bottom_left + self.bottom_right),
            (height, self.top_left + self.bottom_left),
            (height, self.top_right + self.bottom_right),
        ] {
            if sum > 0. {
                factor = factor.min(extent / sum);
            }
        }
        Ok(Self {
            top_left: self.top_left * factor,
            top_right: self.top_right * factor,
            bottom_right: self.bottom_right * factor,
            bottom_left: self.bottom_left * factor,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

impl<'de> Deserialize<'de> for FillRule {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "nonzero" => Ok(Self::Nonzero),
            "evenodd" => Ok(Self::Evenodd),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["nonzero", "evenodd"],
            )),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum PathCommand {
    MoveTo {
        to: VectorPoint,
    },
    LineTo {
        to: VectorPoint,
    },
    QuadraticTo {
        control: VectorPoint,
        to: VectorPoint,
    },
    CubicTo {
        control1: VectorPoint,
        control2: VectorPoint,
        to: VectorPoint,
    },
    Close {},
}

impl<'de> Deserialize<'de> for PathCommand {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
        enum Fields {
            MoveTo {
                to: VectorPoint,
            },
            LineTo {
                to: VectorPoint,
            },
            QuadraticTo {
                control: VectorPoint,
                to: VectorPoint,
            },
            CubicTo {
                control1: VectorPoint,
                control2: VectorPoint,
                to: VectorPoint,
            },
            Close {},
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(match fields {
            Fields::MoveTo { to } => Self::MoveTo { to },
            Fields::LineTo { to } => Self::LineTo { to },
            Fields::QuadraticTo { control, to } => Self::QuadraticTo { control, to },
            Fields::CubicTo {
                control1,
                control2,
                to,
            } => Self::CubicTo {
                control1,
                control2,
                to,
            },
            Fields::Close {} => Self::Close {},
        })
    }
}

/// Filling implicitly closes drawable subpaths; stroking closes only on Close.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorPath {
    pub fill_rule: FillRule,
    pub commands: Vec<PathCommand>,
}

impl<'de> Deserialize<'de> for VectorPath {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            fill_rule: FillRule,
            commands: Vec<PathCommand>,
        }
        let fields: Fields = deserialize_object(deserializer)?;
        Ok(Self {
            fill_rule: fields.fill_rule,
            commands: fields.commands,
        })
    }
}
impl VectorPath {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.commands.is_empty() || self.commands.len() > MAX_VECTOR_PATH_COMMANDS {
            return Err(invalid("path commands must contain 1 through 4096 entries"));
        }
        let mut open = false;
        let mut drawn = false;
        for command in &self.commands {
            match command {
                PathCommand::MoveTo { to } => {
                    to.validate()?;
                    open = true;
                    drawn = false;
                }
                PathCommand::Close {} => {
                    if !open || !drawn {
                        return Err(invalid("close requires a drawable open subpath"));
                    }
                    open = false;
                    drawn = false;
                }
                command => {
                    if !open {
                        return Err(invalid("drawing requires moveTo"));
                    }
                    match command {
                        PathCommand::LineTo { to } => to.validate()?,
                        PathCommand::QuadraticTo { control, to } => {
                            control.validate()?;
                            to.validate()?;
                        }
                        PathCommand::CubicTo {
                            control1,
                            control2,
                            to,
                        } => {
                            control1.validate()?;
                            control2.validate()?;
                            to.validate()?;
                        }
                        _ => unreachable!(),
                    }
                    drawn = true;
                }
            }
        }
        Ok(())
    }
}
