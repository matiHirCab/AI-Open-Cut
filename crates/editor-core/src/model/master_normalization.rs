use serde::{Deserialize, Serialize};

use crate::error::{CoreError, ErrorCode};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MasterNormalization {
    pub enabled: bool,
    pub target_integrated_lufs: f64,
    pub target_loudness_range_lu: f64,
    pub target_true_peak_dbtp: f64,
}

impl MasterNormalization {
    pub(crate) fn validate(&self) -> Result<(), CoreError> {
        for (value, minimum, maximum) in [
            (self.target_integrated_lufs, -70.0, -5.0),
            (self.target_loudness_range_lu, 1.0, 50.0),
            (self.target_true_peak_dbtp, -9.0, 0.0),
        ] {
            if !value.is_finite() || !(minimum..=maximum).contains(&value) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "master normalization targets must be finite and within their bounds",
                ));
            }
        }
        Ok(())
    }
}

impl super::Project {
    pub(crate) fn validate_master_normalization(&self) -> Result<(), CoreError> {
        if let Some(normalization) = &self.master_normalization {
            if self.schema_version < 44 {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "master normalization requires schema44",
                ));
            }
            normalization.validate()?;
        }
        Ok(())
    }
}
