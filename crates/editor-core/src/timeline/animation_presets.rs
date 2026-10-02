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

    #[test]
    fn saved_primitives_and_retired_provenance_share_the_exact_evaluated_scene() {
        use serde_json::json;
        let mut project:Project=serde_json::from_value(json!({"schemaVersion":29,"id":"project","revision":0,"name":"Scene","createdAtMs":1,"updatedAtMs":1,"settings":{"width":64,"height":64,"fps":20},"assets":[],"fonts":{},"markers":[],"components":[],"tracks":[{"id":"track","name":"Visual","trackType":"overlay","items":[{"type":"rectangle","id":"item","zIndex":0,"stackOrder":0,"keyframes":[],"hidden":false,"color":"#ff0000","width":32,"height":32,"startMs":0,"durationMs":1000,"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}]}]})).unwrap();
        apply(
            &mut project,
            "item",
            "scalar_tween".into(),
            1,
            AnimationPresetParameters {
                property: AnimationChannelProperty::Opacity,
                start_ms: 0,
                duration_ms: 500,
                from: 0.0,
                to: 1.0,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            },
            AnimationPresetCollisionPolicy::Reject,
        )
        .unwrap();
        let mut manual = project.clone();
        let visual = manual.tracks[0].items[0].visual_properties_mut();
        visual.animation_preset_provenance.clear();
        visual.animation_channels=serde_json::from_value(json!([{"property":"transform.opacity","keyframes":[{"timeMs":0,"value":{"type":"scalar","value":0.0},"curve":"linear"},{"timeMs":500,"value":{"type":"scalar","value":1.0},"curve":"hold"}]}])).unwrap();
        let evaluate = |p: &Project| {
            crate::evaluated_scene::evaluate_project(p, 64, 64, 20)
                .unwrap()
                .scene
        };
        assert_eq!(evaluate(&project), evaluate(&manual));
        let source = project.tracks[0].items[0]
            .visual_properties_mut()
            .animation_preset_provenance
            .values_mut()
            .next()
            .unwrap();
        source.preset_id = "retired_seed".into();
        source.preset_version = 99;
        source.compiler_version = 88;
        source.parameters.from = 0.25;
        assert_eq!(evaluate(&project), evaluate(&manual));
        let nested = |p: &Project| {
            let mut value = serde_json::to_value(p).unwrap();
            let child = value["tracks"][0].clone();
            value["components"] = json!([{"id":"child","name":"Child","width":64,"height":64,"durationMs":1000,"tracks":[child],"slots":[],"markers":[]}]);
            value["tracks"][0]["items"] = json!([{"type":"component_instance","id":"instance","componentId":"child","startMs":100,"durationMs":500,"trimStartMs":25,"timeScale":0.75,"slotValues":{},"stackOrder":0,"zIndex":0}]);
            serde_json::from_value::<Project>(value).unwrap()
        };
        let scene = evaluate(&nested(&project));
        assert_eq!(scene, evaluate(&nested(&manual)));
        let clock = scene.visual_layers[0].instance.unwrap();
        assert_eq!(clock.rate, 0.75);
        assert_eq!(clock.offset, -50.0);
        assert_eq!(clock.rate * 101.0 + clock.offset, 25.75);
        // A synthetic candidate exercises the retained inherited channel ceiling,
        // which cannot be reached with six distinct targetless seed properties.
        let visual = manual.tracks[0].items[0].visual_properties_mut();
        visual.animation_channels = vec![visual.animation_channels[0].clone(); 64];
        let before = serde_json::to_value(&manual).unwrap();
        let error = apply(
            &mut manual,
            "item",
            "scalar_tween".into(),
            1,
            AnimationPresetParameters {
                property: AnimationChannelProperty::PositionX,
                start_ms: 0,
                duration_ms: 500,
                from: 0.0,
                to: 20.0,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            },
            AnimationPresetCollisionPolicy::Reject,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("maxChannelsPerItem"));
        assert_eq!(serde_json::to_value(&manual).unwrap(), before);
    }
}
