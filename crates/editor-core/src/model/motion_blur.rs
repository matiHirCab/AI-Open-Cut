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
        let width = self.shutter_angle_deg / 360.0 * 1000.0 / f64::from(fps);
        Ok((0..self.sample_count)
            .map(|i| {
                let delta = (width * ((f64::from(i) + 0.5) / f64::from(self.sample_count) - 0.5))
                    .floor() as i64;
                at_ms.saturating_add_signed(delta).min(duration_ms - 1)
            })
            .collect())
    }
}
