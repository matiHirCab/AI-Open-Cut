//! One checked master stage shared by every ordinary render and audio analysis.
use crate::evaluated_scene::EvaluatedScene;
pub(crate) use crate::evaluated_scene::master_normalization::EvaluatedMasterNormalization;
pub(crate) use crate::evaluated_scene::master_normalization::PreparedMasterNormalization;
use crate::{CoreError, ErrorCode};

fn invalid() -> CoreError {
    CoreError::new(
        ErrorCode::FfmpegFailed,
        "master normalization preparation is invalid or unavailable",
    )
}

pub(crate) fn processing_filter(
    normalization: &EvaluatedMasterNormalization,
) -> Result<String, CoreError> {
    let prepared = normalization.prepared.as_ref().ok_or_else(invalid)?;
    normalization.settings.validate()?;
    let mut filters = vec![
        "aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo".to_owned(),
        "aresample=48000".to_owned(),
    ];
    match prepared {
        PreparedMasterNormalization::Identity => {}
        PreparedMasterNormalization::LimitingOnly { gain } => {
            if !gain.is_finite() || !(0.0..=1.0).contains(gain) {
                return Err(invalid());
            }
            if *gain != 1.0 {
                filters.push(format!("volume={gain}:precision=double"));
            }
        }
        PreparedMasterNormalization::Measured {
            pregain_db,
            integrated_lufs,
            true_peak_dbtp,
            loudness_range_lu,
            threshold_lufs,
            offset_lu,
            postgain_db,
        } => {
            for (value, min, max) in [
                (*pregain_db, -800.0, 0.0),
                (*integrated_lufs, -99.0, 0.0),
                (*true_peak_dbtp, -99.0, 99.0),
                (*loudness_range_lu, 0.0, 99.0),
                (*threshold_lufs, -99.0, 0.0),
                (*offset_lu, -99.0, 99.0),
                (*postgain_db, -99.0, 99.0),
            ] {
                if !value.is_finite() || !(min..=max).contains(&value) {
                    return Err(invalid());
                }
            }
            if *pregain_db != 0.0 {
                filters.push(format!("volume={pregain_db}dB:precision=double"));
            }
            let settings = &normalization.settings;
            filters.push(format!("loudnorm=I={}:TP={}:LRA={}:measured_I={integrated_lufs}:measured_TP={true_peak_dbtp}:measured_LRA={loudness_range_lu}:measured_thresh={threshold_lufs}:offset={offset_lu}:linear=true",settings.target_integrated_lufs,settings.target_true_peak_dbtp,settings.target_loudness_range_lu));
            filters
                .push("aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo".into());
            filters.push("aresample=48000".into());
            if *postgain_db != 0.0 {
                filters.push(format!("volume={postgain_db}dB:precision=double"));
            }
        }
    }
    filters.push("aformat=sample_fmts=flt:sample_rates=48000:channel_layouts=stereo".into());
    filters.push("aresample=48000".into());
    Ok(filters.join(","))
}

pub(crate) fn finish_master(
    filters: &mut [String],
    scene: &EvaluatedScene,
) -> Result<Option<String>, CoreError> {
    let Some(normalization) = &scene.master_normalization else {
        return Ok(None);
    };
    crate::evaluated_scene::master_normalization::admit_duration(scene.duration_ms)?;
    let processing = processing_filter(normalization)?;
    let final_filter = filters.last_mut().ok_or_else(invalid)?;
    let prefix = final_filter.strip_suffix("[audio]").ok_or_else(invalid)?;
    *final_filter = format!("{prefix}[unnormalized_master]");
    Ok(Some(format!("[unnormalized_master]{processing}[audio]")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_plan::audio_analysis::{AudioAnalysisOptions, build_audio_analysis_plan};
    use serde_json::json;

    #[test]
    fn omitted_disabled_scene_and_plans_stay_exact_and_active_unprepared_facts_fail() {
        let root = tempfile::tempdir().unwrap();
        let core = crate::EditorCore::new(
            crate::PathPolicy::new(
                root.path().join("projects"),
                [root.path()],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let id = core
            .create_project("Root controls", crate::ProjectSettings::default())
            .unwrap()
            .project_id;
        let mut project = core.get_project(&id).unwrap();
        core.edit(&id,0,serde_json::from_value(json!({"operation":"add_solid_color","trackId":project.tracks[1].id,"startMs":0,"durationMs":1000,"color":"#112233","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap()).unwrap();
        project = core.get_project(&id).unwrap();
        let old = crate::evaluated_scene::evaluate_project(&project, 32, 32, 24)
            .unwrap()
            .scene;
        let options = AudioAnalysisOptions {
            start_ms: 0,
            end_ms: 1000,
            waveform_bins: 1,
        };
        let old_plan = build_audio_analysis_plan(&old, vec![], vec![], options).unwrap();
        let mut settings: crate::MasterNormalization = serde_json::from_value(
            serde_json::from_str::<serde_json::Value>(include_str!(
                "../../../../contracts/master-normalization-v1.json"
            ))
            .unwrap()["disabledExample"]
                .clone(),
        )
        .unwrap();
        project.master_normalization = Some(settings.clone());
        let disabled = crate::evaluated_scene::evaluate_project(&project, 32, 32, 24)
            .unwrap()
            .scene;
        assert_eq!(old, disabled);
        assert_eq!(format!("{old:?}"), format!("{disabled:?}"));
        assert_eq!(
            old_plan.render,
            build_audio_analysis_plan(&disabled, vec![], vec![], options)
                .unwrap()
                .render
        );
        settings.enabled = true;
        project.master_normalization = Some(settings);
        let active = crate::evaluated_scene::evaluate_project(&project, 32, 32, 24)
            .unwrap()
            .scene;
        assert!(
            active
                .master_normalization
                .as_ref()
                .unwrap()
                .prepared
                .is_none()
        );
        assert_eq!(
            build_audio_analysis_plan(&active, vec![], vec![], options)
                .unwrap_err()
                .code,
            ErrorCode::FfmpegFailed
        );
    }

    #[test]
    fn prepared_coefficients_reject_poison_without_raw_expressions_or_silent_clamping() {
        let settings: crate::MasterNormalization = serde_json::from_value(
            serde_json::from_str::<serde_json::Value>(include_str!(
                "../../../../contracts/master-normalization-v1.json"
            ))
            .unwrap()["settingsExample"]
                .clone(),
        )
        .unwrap();
        for gain in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
            let facts = EvaluatedMasterNormalization {
                settings: settings.clone(),
                prepared: Some(PreparedMasterNormalization::LimitingOnly { gain }),
            };
            assert_eq!(
                processing_filter(&facts).unwrap_err().code,
                ErrorCode::FfmpegFailed
            );
        }
        for field in 0..7 {
            let mut values = [0.0, -20.0, -18.0, 2.0, -30.0, 0.0, 0.0];
            values[field] = f64::NAN;
            let facts = EvaluatedMasterNormalization {
                settings: settings.clone(),
                prepared: Some(PreparedMasterNormalization::Measured {
                    pregain_db: values[0],
                    integrated_lufs: values[1],
                    true_peak_dbtp: values[2],
                    loudness_range_lu: values[3],
                    threshold_lufs: values[4],
                    offset_lu: values[5],
                    postgain_db: values[6],
                }),
            };
            assert_eq!(
                processing_filter(&facts).unwrap_err().code,
                ErrorCode::FfmpegFailed
            );
        }
    }
}
