//! Canonical descriptor acceptance and allocation-free lattice work accounting.
use crate::{CoreError, ErrorCode, GridDescriptor, GridPattern, MAX_GRID_MARKS};

fn invalid() -> CoreError {
    CoreError::new(
        ErrorCode::InvalidArgument,
        "invalid grid descriptor or complexity limit exceeded",
    )
}

fn dimension(v: f64) -> Result<(), CoreError> {
    if !v.is_finite() || v <= 0. || v > 16384. {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn normals(pattern: &GridPattern) -> &'static [(f64, f64)] {
    use std::f64::consts::FRAC_1_SQRT_2 as D;
    // The correctly rounded f64 value of sqrt(3)/2.
    const I: f64 = 0.866_025_403_784_438_6;
    match pattern {
        GridPattern::Diagonal { .. } => &[(D, D), (D, -D)],
        GridPattern::Isometric { .. } => &[(1., 0.), (0.5, I), (0.5, -I)],
        _ => &[],
    }
}

/// Non-axis families exclude extrema, which touch only one corner.
pub(crate) fn family_range(
    w: f64,
    h: f64,
    n: (f64, f64),
    spacing: f64,
) -> Result<(i32, i32), CoreError> {
    let lo = (n.0 * w).min(0.) + (n.1 * h).min(0.);
    let hi = (n.0 * w).max(0.) + (n.1 * h).max(0.);
    let (a, b) = (lo / spacing, hi / spacing);
    if !a.is_finite() || !b.is_finite() {
        return Err(invalid());
    }
    let axis = n.0 == 0. || n.1 == 0.;
    let first = if axis { a.ceil() } else { a.floor() + 1. };
    let last = if axis { b.floor() } else { b.ceil() - 1. };
    if first.abs() > MAX_GRID_MARKS as f64 || last.abs() > MAX_GRID_MARKS as f64 {
        return Err(invalid());
    }
    Ok((first as i32, last as i32))
}

pub(crate) fn axis_count(extent: f64, spacing: f64) -> Result<usize, CoreError> {
    let count = (extent / spacing).floor() + 1.;
    if !count.is_finite() || count > MAX_GRID_MARKS as f64 {
        return Err(invalid());
    }
    Ok(count as usize)
}

pub(crate) fn validate_grid(grid: &GridDescriptor) -> Result<(), CoreError> {
    dimension(grid.width)?;
    dimension(grid.height)?;
    let count = match &grid.pattern {
        GridPattern::Rectangular {
            spacing_x,
            spacing_y,
            stroke,
        } => {
            dimension(*spacing_x)?;
            dimension(*spacing_y)?;
            stroke.validate()?;
            axis_count(grid.width, *spacing_x)? + axis_count(grid.height, *spacing_y)?
        }
        GridPattern::Dot {
            spacing_x,
            spacing_y,
            radius,
            paint,
        } => {
            dimension(*spacing_x)?;
            dimension(*spacing_y)?;
            paint.validate()?;
            if !radius.is_finite() || *radius <= 0. || *radius > spacing_x.min(*spacing_y) / 2. {
                return Err(invalid());
            }
            axis_count(grid.width, *spacing_x)? * axis_count(grid.height, *spacing_y)?
        }
        GridPattern::Diagonal { spacing, stroke } | GridPattern::Isometric { spacing, stroke } => {
            dimension(*spacing)?;
            stroke.validate()?;
            let mut count = 0;
            for &n in normals(&grid.pattern) {
                let (a, b) = family_range(grid.width, grid.height, n, *spacing)?;
                count += (b - a + 1).max(0) as usize;
            }
            count
        }
    };
    if count > MAX_GRID_MARKS {
        return Err(invalid());
    }
    Ok(())
}
