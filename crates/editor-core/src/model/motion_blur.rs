use crate::{CoreError, ErrorCode};
use serde::{Deserialize, Serialize};

/// Per-leaf centered exposure; omission and a single sample retain legacy output.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MotionBlur {
    pub shutter_angle_deg: f64,
    pub sample_count: u32,
}

impl MotionBlur {
    pub const MAX_SAMPLES: u32 = 16;
    pub const MAX_PIXEL_WORK: u64 = 268_435_456;

    pub fn validate(self) -> Result<(), CoreError> {
        if !self.shutter_angle_deg.is_finite()
            || !(0.0..=360.0).contains(&self.shutter_angle_deg)
            || !(1..=Self::MAX_SAMPLES).contains(&self.sample_count)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "motion blur exceeds shutter or sample bounds",
            ));
        }
        Ok(())
    }

    pub fn enabled(self) -> bool {
        self.shutter_angle_deg > 0.0 && self.sample_count > 1
    }

    /// Root-grid integer resolution is applied once, before inherited mappings.
    /// Midpoints use the exact stored binary64 angle; no rounded subtraction or
    /// epsilon can move a sample across an integer boundary.
    /// Offset arithmetic preserves adjacent timestamps throughout the u64 range.
    pub fn sample_times(
        self,
        at_ms: u64,
        fps: u32,
        duration_ms: u64,
    ) -> Result<Vec<u64>, CoreError> {
        self.validate()?;
        if fps == 0 || duration_ms == 0 {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "invalid shutter timeline",
            ));
        }
        if !self.enabled() {
            return Ok(vec![at_ms]);
        }
        let arithmetic_error = || {
            CoreError::new(
                ErrorCode::InvalidArgument,
                "motion blur midpoint arithmetic exceeds bounds",
            )
        };
        // Every enabled, validated angle is positive and <= 360. Its exact
        // value is significand / 2^shift, with shift in 44..=1074. The largest
        // numerator below is < 2^68; an overflowing denominator is therefore
        // strictly larger, including for subnormal angles.
        let bits = self.shutter_angle_deg.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as u32;
        let significand =
            (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
        let shift = 1075 - exponent.max(1);
        let denominator = 720_u128
            .checked_mul(u128::from(fps))
            .and_then(|value| value.checked_mul(u128::from(self.sample_count)))
            .ok_or_else(arithmetic_error)?;
        let scaled_denominator = 1_u128
            .checked_shl(shift)
            .and_then(|scale| denominator.checked_mul(scale));
        (0..self.sample_count)
            .map(|i| {
                // angle * 1000 * (2*i + 1 - N) / (720 * FPS * N).
                let coefficient = i64::from(2 * i + 1) - i64::from(self.sample_count);
                let numerator = u128::from(significand)
                    .checked_mul(1000)
                    .and_then(|value| value.checked_mul(u128::from(coefficient.unsigned_abs())))
                    .ok_or_else(arithmetic_error)?;
                let magnitude = match scaled_denominator {
                    Some(divisor) => {
                        numerator / divisor
                            + u128::from(coefficient < 0 && numerator % divisor != 0)
                    }
                    None => u128::from(coefficient < 0),
                };
                let magnitude = i64::try_from(magnitude).map_err(|_| arithmetic_error())?;
                let delta = if coefficient < 0 {
                    -magnitude
                } else {
                    magnitude
                };
                Ok(at_ms.saturating_add_signed(delta).min(duration_ms - 1))
            })
            .collect()
    }
}
