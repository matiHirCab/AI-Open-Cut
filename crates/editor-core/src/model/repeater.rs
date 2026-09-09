//! Strict, resource-free repeater records.
use crate::{CoreError, ErrorCode, TransformPosition};
use serde::{Deserialize, Serialize};

pub const MAX_REPEATER_COPIES: u16 = 256;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeaterSourceReference {
    pub scope: String,
    pub id: String,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeaterTransformOffset {
    pub position: TransformPosition,
    pub scale_x: f64,
    pub scale_y: f64,
    pub rotation_deg: f64,
    pub skew_x_deg: f64,
    pub skew_y_deg: f64,
}

impl RepeaterTransformOffset {
    pub fn validate(&self) -> Result<(), CoreError> {
        let position_limit = match self.position.unit {
            crate::PositionUnit::Pixels => 1_000_000.0,
            crate::PositionUnit::Normalized => 100.0,
        };
        let bounded = |v: f64, lo: f64, hi: f64| v.is_finite() && (lo..=hi).contains(&v);
        if !bounded(self.position.x, -position_limit, position_limit)
            || !bounded(self.position.y, -position_limit, position_limit)
            || !bounded(self.scale_x, 0.0, 100.0)
            || self.scale_x == 0.0
            || !bounded(self.scale_y, 0.0, 100.0)
            || self.scale_y == 0.0
            || !bounded(self.rotation_deg, -36_000.0, 36_000.0)
            || !bounded(self.skew_x_deg, -80.0, 80.0)
            || !bounded(self.skew_y_deg, -80.0, 80.0)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "repeater transform offset exceeds finite numeric bounds",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepeaterDescriptor {
    pub source: RepeaterSourceReference,
    pub copies: u16,
    pub transform_offset: RepeaterTransformOffset,
    pub opacity_offset: f64,
}
