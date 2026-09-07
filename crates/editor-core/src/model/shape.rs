//! Persisted shape vocabulary. Geometry validation is pure and resource-free.
use crate::{CoreError, CornerRadii, ErrorCode, Paint, Stroke, VectorPath, VectorPoint};
use serde::{Deserialize, Serialize};

pub const MAX_SHAPE_POINTS: usize = 4096;
pub const MAX_SHAPE_STAR_POINTS: u32 = 2048;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ShapeGeometry {
    Rectangle {
        width: f64,
        height: f64,
    },
    RoundedRectangle {
        width: f64,
        height: f64,
        radii: CornerRadii,
    },
    Ellipse {
        width: f64,
        height: f64,
    },
    Line {
        start: VectorPoint,
        end: VectorPoint,
    },
    Polygon {
        points: Vec<VectorPoint>,
    },
    Star {
        center: VectorPoint,
        outer_radius: f64,
        inner_radius: f64,
        point_count: u32,
        rotation_deg: f64,
    },
    Path {
        path: VectorPath,
    },
}

impl ShapeGeometry {
    pub fn validate(&self) -> Result<(), CoreError> {
        let dimension = |v: f64| {
            if v.is_finite() && v > 0.0 && v <= 16384.0 {
                Ok(())
            } else {
                Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "shape dimension must be in (0,16384]",
                ))
            }
        };
        match self {
            Self::Rectangle { width, height } | Self::Ellipse { width, height } => {
                dimension(*width)?;
                dimension(*height)
            }
            Self::RoundedRectangle {
                width,
                height,
                radii,
            } => {
                dimension(*width)?;
                dimension(*height)?;
                radii.validate()
            }
            Self::Line { start, end } => {
                start.validate()?;
                end.validate()?;
                if start == end {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "line endpoints must differ",
                    ));
                }
                Ok(())
            }
            Self::Polygon { points } => {
                if !(3..=MAX_SHAPE_POINTS).contains(&points.len()) {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "polygon requires 3 through 4096 points",
                    ));
                }
                points.iter().try_for_each(VectorPoint::validate)
            }
            Self::Star {
                center,
                outer_radius,
                inner_radius,
                point_count,
                rotation_deg,
            } => {
                center.validate()?;
                dimension(*outer_radius)?;
                dimension(*inner_radius)?;
                if inner_radius >= outer_radius
                    || !(3..=MAX_SHAPE_STAR_POINTS).contains(point_count)
                    || !rotation_deg.is_finite()
                    || rotation_deg.abs() > 36000.0
                {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "invalid star radii, point count or rotation",
                    ));
                }
                Ok(())
            }
            Self::Path { path } => path.validate(),
        }
    }
}

pub fn validate_shape(
    geometry: &ShapeGeometry,
    fill: &Option<Paint>,
    stroke: &Option<Stroke>,
) -> Result<(), CoreError> {
    geometry.validate()?;
    if fill.is_none() && stroke.is_none()
        || matches!(geometry, ShapeGeometry::Line { .. }) && (fill.is_some() || stroke.is_none())
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "shape requires paint; lines require only stroke",
        ));
    }
    if let Some(fill) = fill {
        fill.validate()?;
    }
    if let Some(stroke) = stroke {
        stroke.validate()?;
    }
    Ok(())
}

/// A present nullable field: missing values are malformed, unlike optional edit patches.
pub(super) fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}
