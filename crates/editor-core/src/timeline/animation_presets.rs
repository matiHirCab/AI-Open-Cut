use crate::{
    ANIMATION_PRESET_COMPILER_VERSION, AnimationChannel, AnimationChannelKeyframe,
    AnimationChannelValue, AnimationCurve, AnimationPresetCollisionPolicy,
    AnimationPresetParameters, AnimationPresetProvenance, CoreError, ErrorCode, Project,
    SimpleAnimationCurve, Track,
};

pub(super) fn apply(
    project: &mut Project,
    item_id: &str,
    preset_id: String,
    preset_version: u32,
    parameters: AnimationPresetParameters,
    collision_policy: AnimationPresetCollisionPolicy,
) -> Result<(), CoreError> {
    let (track_index, item_index) = super::find_item_location(project, item_id)?;
    if project.tracks[track_index].locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    let (channel, source) = compile(preset_id, preset_version, parameters)?;
    let property = channel.property;
    let item = &project.tracks[track_index].items[item_index];
    let mut channels = item.visual_properties().animation_channels.clone();
    if let Some(index) = channels
        .iter()
        .position(|c| c.property == property && c.target.is_none())
    {
        if collision_policy == AnimationPresetCollisionPolicy::Reject {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "animation preset channel collision",
            ));
        }
        channels[index] = channel;
    } else {
        channels.push(channel);
    }
    crate::validation::animation_channels::validate_channels(&channels, item, project)?;
    let visual = project.tracks[track_index].items[item_index].visual_properties_mut();
    visual.animation_channels = channels;
    visual.animation_preset_provenance.insert(property, source);
    Ok(())
}

// Compilation is pure; candidate validation and publication stay with their existing owners.
fn compile(
    preset_id: String,
    preset_version: u32,
    parameters: AnimationPresetParameters,
) -> Result<(AnimationChannel, AnimationPresetProvenance), CoreError> {
    if preset_id != "scalar_tween" || preset_version != 1 {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "unsupported animation preset or version",
        ));
    }
    let end = crate::validation::animation_presets::validate_parameters(&parameters)?;
    let channel = AnimationChannel {
        property: parameters.property,
        target: None,
        keyframes: vec![
            AnimationChannelKeyframe {
                time_ms: parameters.start_ms,
                value: AnimationChannelValue::Scalar {
                    value: parameters.from,
                },
                curve: parameters.curve,
            },
            AnimationChannelKeyframe {
                time_ms: end,
                value: AnimationChannelValue::Scalar {
                    value: parameters.to,
                },
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Hold),
            },
        ],
        r#loop: None,
    };
    Ok((
        channel,
        AnimationPresetProvenance {
            preset_id,
            preset_version,
            compiler_version: ANIMATION_PRESET_COMPILER_VERSION,
            parameters,
        },
    ))
}

pub(super) fn reconcile_raw_tracks(tracks: &mut [Track], prior: &[Track]) {
    for item in tracks.iter_mut().flat_map(|t| &mut t.items) {
        let old = prior
            .iter()
            .flat_map(|t| &t.items)
            .find(|old| old.id() == item.id());
        let visual = item.visual_properties_mut();
        visual.animation_preset_provenance.clear();
        if let Some(old) = old {
            for (property, source) in &old.visual_properties().animation_preset_provenance {
                let before = old
                    .visual_properties()
                    .animation_channels
                    .iter()
                    .find(|c| c.property == *property && c.target.is_none());
                let after = visual
                    .animation_channels
                    .iter()
                    .find(|c| c.property == *property && c.target.is_none());
                if before.is_some() && before == after {
                    visual
                        .animation_preset_provenance
                        .insert(*property, source.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnimationChannelProperty;

    #[test]
    fn compiler_samples_use_independent_fixed_curve_oracles_and_fractional_times() {
        let cases = [
            (serde_json::json!("hold"), 0.0),
            (serde_json::json!("linear"), 0.5),
            (
                serde_json::json!({"type":"cubic_bezier","x1":0.0,"y1":0.0,"x2":1.0,"y2":1.0}),
                0.5,
            ),
            (
                serde_json::json!({"type":"spring","mass":1.0,"stiffness":100.0,"damping":20.0,"initialVelocity":0.0}),
                1.0 - 6.0 * (-5.0_f64).exp(),
            ),
        ];
        for property in [
            AnimationChannelProperty::PositionX,
            AnimationChannelProperty::PositionY,
            AnimationChannelProperty::ScaleX,
            AnimationChannelProperty::ScaleY,
            AnimationChannelProperty::Opacity,
            AnimationChannelProperty::GainDb,
        ] {
            let (from, to) = if property == AnimationChannelProperty::Opacity {
                (0.0, 1.0)
            } else {
                (0.5, 1.0)
            };
            for (curve, midpoint) in &cases {
                let (channel, _) = compile(
                    "scalar_tween".into(),
                    1,
                    AnimationPresetParameters {
                        property,
                        start_ms: 10,
                        duration_ms: 500,
                        from,
                        to,
                        curve: serde_json::from_value(curve.clone()).unwrap(),
                    },
                )
                .unwrap();
                for (time, expected) in [
                    (0.0, from),
                    (10.0, from),
                    (260.0, from + (to - from) * midpoint),
                    (510.0, to),
                    (999.5, to),
                ] {
                    assert!(
                        (crate::animation::sample_scalar_channel_at(&channel, time).unwrap()
                            - expected)
                            .abs()
                            < 1e-10,
                        "{property:?} {curve} {time}"
                    );
                }
            }
        }
        let (channel, _) = compile(
            "scalar_tween".into(),
            1,
            AnimationPresetParameters {
                property: AnimationChannelProperty::PositionX,
                start_ms: 10,
                duration_ms: 500,
                from: 0.0,
                to: 100.0,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            },
        )
        .unwrap();
        assert!(
            (crate::animation::sample_scalar_channel_at(&channel, 10.5).unwrap() - 0.1).abs()
                < 1e-12
        );
    }

    #[test]
    fn compiler_rejects_native_nonfinite_values_without_serialization() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for from_endpoint in [true, false] {
                let parameters = AnimationPresetParameters {
                    property: AnimationChannelProperty::Opacity,
                    start_ms: 0,
                    duration_ms: 500,
                    from: if from_endpoint { value } else { 0.0 },
                    to: if from_endpoint { 1.0 } else { value },
                    curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
                };
                assert_eq!(
                    compile("scalar_tween".into(), 1, parameters)
                        .unwrap_err()
                        .code,
                    ErrorCode::InvalidArgument
                );
            }
        }
    }
}
