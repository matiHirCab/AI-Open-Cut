//! Fixed root facts; measured coefficients are request-local and never persisted.
use crate::{CoreError, ErrorCode, MasterNormalization};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMasterNormalization {
    pub(crate) settings: MasterNormalization,
    pub(crate) prepared: Option<PreparedMasterNormalization>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PreparedMasterNormalization {
    Identity,
    LimitingOnly {
        gain: f64,
    },
    Measured {
        pregain_db: f64,
        integrated_lufs: f64,
        true_peak_dbtp: f64,
        loudness_range_lu: f64,
        threshold_lufs: f64,
        offset_lu: f64,
        postgain_db: f64,
    },
}

pub(crate) fn admit_duration(duration_ms: u64) -> Result<(), CoreError> {
    if !(1..=600_000).contains(&duration_ms) {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "active master normalization requires a root duration of 1..600000ms",
        ));
    }
    Ok(())
}

pub(crate) fn finalize(project: &crate::Project, scene: &mut super::EvaluatedScene) {
    scene.master_normalization = project
        .master_normalization
        .as_ref()
        .filter(|settings| settings.enabled)
        .map(|settings| EvaluatedMasterNormalization {
            settings: settings.clone(),
            prepared: None,
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_duration_boundaries_admit_only_bounded_complete_root_work() {
        for duration in [0, 600_001, u64::MAX] {
            assert_eq!(
                admit_duration(duration).unwrap_err().code,
                ErrorCode::InvalidArgument
            );
        }
        for duration in [1, 600_000] {
            admit_duration(duration).unwrap();
            let frames = duration.checked_mul(48).unwrap();
            assert!(frames <= 28_800_000);
            assert!(frames.checked_mul(8).unwrap() <= 230_400_000);
        }
    }
}
