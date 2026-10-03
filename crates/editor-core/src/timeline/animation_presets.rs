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
    let compiled = compile(preset_id, preset_version, parameters)?;
    let item = &project.tracks[track_index].items[item_index];
    let mut candidate = item.clone();
    let visual = candidate.visual_properties_mut();
    if compiled.motion_blur.is_some()
        && visual.motion_blur.is_some()
        && collision_policy == AnimationPresetCollisionPolicy::Reject
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "animation preset motionBlur collision",
        ));
    }
    for channel in compiled.channels {
        let property = channel.property;
        if let Some(index) = visual
            .animation_channels
            .iter()
            .position(|c| c.property == property && c.target.is_none())
        {
            if collision_policy == AnimationPresetCollisionPolicy::Reject {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "animation preset channel collision",
                ));
            }
            visual.animation_channels[index] = channel;
        } else {
            visual.animation_channels.push(channel);
        }
        visual
            .animation_preset_provenance
            .insert(property, compiled.source.clone());
    }
    if let Some(blur) = compiled.motion_blur {
        visual.motion_blur = Some(blur);
    }
    crate::validation::animation_channels::validate_channels(
        &candidate.visual_properties().animation_channels,
        item,
        project,
    )?;
    crate::validation::extended_visual::validate_static(&candidate, project)?;
    project.tracks[track_index].items[item_index] = candidate;
    Ok(())
}

// Compilation is pure; candidate validation and publication stay with their existing owners.
fn compile_scalar(
    preset_id: String,
    preset_version: u32,
    parameters: crate::ScalarTweenParameters,
) -> Result<(AnimationChannel, AnimationPresetProvenance), CoreError> {
    if preset_id != "scalar_tween" || preset_version != 1 {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "unsupported animation preset or version",
        ));
    }
    let end =
        crate::validation::animation_presets::validate_parameters(&parameters.clone().into())?;
    let channel = AnimationChannel {
        clock: None,
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
            parameters: parameters.into(),
        },
    ))
}

struct CompiledPreset {
    channels: Vec<AnimationChannel>,
    source: AnimationPresetProvenance,
    motion_blur: Option<crate::MotionBlur>,
}

fn compile(
    preset_id: String,
    preset_version: u32,
    parameters: AnimationPresetParameters,
) -> Result<CompiledPreset, CoreError> {
    use crate::{
        AnimationChannelProperty as P, AnimationLoop, AnimationLoopMode,
        MotionPresetParameters as M,
    };
    if let AnimationPresetParameters::Scalar(p) = parameters {
        let (channel, source) = compile_scalar(preset_id, preset_version, p)?;
        return Ok(CompiledPreset {
            channels: vec![channel],
            source,
            motion_blur: None,
        });
    }
    let AnimationPresetParameters::Pack(mut p) = parameters else {
        unreachable!()
    };
    if preset_version != 1 || preset_id != p.id() {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "unsupported animation preset or version/parameter kind",
        ));
    }
    p.materialize_iterations();
    crate::validation::animation_presets::validate_parameters(&AnimationPresetParameters::Pack(
        p.clone(),
    ))?;
    let (start, duration) = p.timing();
    let phase = |q: u64| -> Result<u64, CoreError> {
        let offset = u128::from(q) * u128::from(duration) / 8;
        u64::try_from(offset)
            .ok()
            .and_then(|offset| start.checked_add(offset))
            .filter(|time| *time <= crate::MAX_PRESET_TIME_MS)
            .ok_or_else(|| {
                CoreError::new(
                    ErrorCode::InvalidArgument,
                    "preset phase time exceeds bounds",
                )
            })
    };
    let r#loop = p.iterations().map(|iterations| AnimationLoop {
        mode: AnimationLoopMode::Repeat,
        iterations,
    });
    let make = |property, phases: &[u64], values: &[f64]| -> Result<AnimationChannel, CoreError> {
        let mut keyframes = Vec::with_capacity(phases.len());
        for (index, (&q, &value)) in phases.iter().zip(values).enumerate() {
            let time_ms = phase(q)?;
            if keyframes
                .last()
                .is_some_and(|prior: &AnimationChannelKeyframe| prior.time_ms >= time_ms)
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "preset phases collapse",
                ));
            }
            keyframes.push(AnimationChannelKeyframe {
                time_ms,
                value: AnimationChannelValue::Scalar { value },
                curve: AnimationCurve::Simple(if index + 1 == phases.len() {
                    SimpleAnimationCurve::Hold
                } else {
                    SimpleAnimationCurve::Linear
                }),
            });
        }
        Ok(AnimationChannel {
            clock: None,
            property,
            target: None,
            keyframes,
            r#loop,
        })
    };
    let (channels, motion_blur) = match &p {
        M::ImpactSlam {
            center_x: x,
            center_y: y,
            shake_amplitude_px: a,
            scale_from,
            scale_overshoot,
            scale_to,
            opacity_from,
            opacity_to,
            flash_opacity,
            motion_blur,
            ..
        } => (
            vec![
                make(
                    P::PositionX,
                    &[0, 4, 5, 6, 7, 8],
                    &[*x, *x, *x + *a, *x - *a / 2.0, *x + *a / 4.0, *x],
                )?,
                make(
                    P::PositionY,
                    &[0, 4, 5, 6, 7, 8],
                    &[*y, *y, *y - *a, *y + *a / 2.0, *y - *a / 4.0, *y],
                )?,
                make(
                    P::ScaleX,
                    &[0, 4, 6, 8],
                    &[*scale_from, *scale_overshoot, *scale_to, *scale_to],
                )?,
                make(
                    P::ScaleY,
                    &[0, 4, 6, 8],
                    &[*scale_from, *scale_overshoot, *scale_to, *scale_to],
                )?,
                make(
                    P::Opacity,
                    &[0, 4, 5, 6, 8],
                    &[
                        *opacity_from,
                        *opacity_to,
                        *flash_opacity,
                        *opacity_to,
                        *opacity_to,
                    ],
                )?,
            ],
            Some(*motion_blur),
        ),
        M::SlideLeft {
            position_from_x,
            position_to_x,
            ..
        } => (
            vec![make(
                P::PositionX,
                &[0, 8],
                &[*position_from_x, *position_to_x],
            )?],
            None,
        ),
        M::Scan {
            position_from_x,
            position_to_x,
            ..
        } => (
            vec![make(
                P::PositionX,
                &[0, 4, 8],
                &[*position_from_x, *position_to_x, *position_from_x],
            )?],
            None,
        ),
        M::Pulse {
            scale_from,
            scale_peak,
            ..
        } => (
            vec![
                make(
                    P::ScaleX,
                    &[0, 4, 8],
                    &[*scale_from, *scale_peak, *scale_from],
                )?,
                make(
                    P::ScaleY,
                    &[0, 4, 8],
                    &[*scale_from, *scale_peak, *scale_from],
                )?,
            ],
            None,
        ),
        M::RadarExpand {
            scale_from,
            scale_to,
            opacity_peak,
            ..
        } => (
            vec![
                make(
                    P::ScaleX,
                    &[0, 6, 8],
                    &[*scale_from, *scale_to, *scale_from],
                )?,
                make(
                    P::ScaleY,
                    &[0, 6, 8],
                    &[*scale_from, *scale_to, *scale_from],
                )?,
                make(P::Opacity, &[0, 2, 6, 8], &[0.0, *opacity_peak, 0.0, 0.0])?,
            ],
            None,
        ),
    };
    Ok(CompiledPreset {
        channels,
        motion_blur,
        source: AnimationPresetProvenance {
            preset_id,
            preset_version,
            compiler_version: crate::MOTION_PRESET_COMPILER_VERSION,
            parameters: AnimationPresetParameters::Pack(p),
        },
    })
}

pub(super) fn blur_changed(visual: &mut crate::VisualProperties, blur: crate::MotionBlur) {
    if !same_bytes(&visual.motion_blur, &Some(blur)) {
        visual
            .animation_preset_provenance
            .retain(|_, source| !source.parameters.is_impact());
    }
}

fn same_bytes<T: serde::Serialize>(before: &T, after: &T) -> bool {
    matches!((serde_json::to_vec(before),serde_json::to_vec(after)),(Ok(before),Ok(after)) if before == after)
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
                // Persisted binary64 endpoints retain signed zero. Floating-point
                // PartialEq alone would label a +0 to -0 rewrite as unchanged.
                let unchanged = before.zip(after).is_some_and(|(before, after)| {
                    matches!(
                        (serde_json::to_vec(before), serde_json::to_vec(after)),
                        (Ok(before), Ok(after)) if before == after
                    )
                });
                if unchanged
                    && (!source.parameters.is_impact()
                        || same_bytes(&old.visual_properties().motion_blur, &visual.motion_blur))
                {
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
    fn pack_samples_match_fixed_canonical_seams_and_fractional_oracles() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/initial-motion-preset-pack-v1.json"
        ))
        .unwrap();
        for entry in fixture["presets"].as_array().unwrap() {
            let compiled = compile(
                entry["id"].as_str().unwrap().into(),
                1,
                serde_json::from_value(entry["parameters"].clone()).unwrap(),
            )
            .unwrap();
            for sample in entry["expected"]["samples"].as_array().unwrap() {
                let time = sample["timeMs"].as_f64().unwrap();
                for (field, property) in [
                    ("positionX", crate::AnimationChannelProperty::PositionX),
                    ("positionY", crate::AnimationChannelProperty::PositionY),
                    ("scaleX", crate::AnimationChannelProperty::ScaleX),
                    ("opacity", crate::AnimationChannelProperty::Opacity),
                ] {
                    if let Some(expected) = sample.get(field).and_then(serde_json::Value::as_f64) {
                        let channel = compiled
                            .channels
                            .iter()
                            .find(|c| c.property == property)
                            .unwrap();
                        let actual =
                            crate::animation::sample_scalar_channel_at(channel, time).unwrap();
                        assert!(
                            (actual - expected).abs() < 1e-12,
                            "{} {time} {field}: {actual} != {expected}",
                            entry["id"]
                        );
                    }
                }
            }
        }
        let fixture = &fixture["presets"][2];
        let compiled = compile(
            "scan".into(),
            1,
            serde_json::from_value(fixture["parameters"].clone()).unwrap(),
        )
        .unwrap();
        assert!(
            (crate::animation::sample_scalar_channel_at(&compiled.channels[0], 10.5).unwrap()
                - 19.5)
                .abs()
                < 1e-12
        );
        assert!(
            (crate::animation::sample_scalar_channel_at(&compiled.channels[0], 90.5).unwrap()
                - 19.5)
                .abs()
                < 1e-12
        );
    }

    #[test]
    fn pack_native_nonfinite_inputs_fail_without_json_normalization() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/initial-motion-preset-pack-v1.json"
        ))
        .unwrap();
        for entry in fixture["presets"].as_array().unwrap() {
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut parameters: AnimationPresetParameters =
                    serde_json::from_value(entry["parameters"].clone()).unwrap();
                let AnimationPresetParameters::Pack(p) = &mut parameters else {
                    panic!()
                };
                match p {
                    crate::MotionPresetParameters::ImpactSlam { center_x, .. } => *center_x = value,
                    crate::MotionPresetParameters::SlideLeft {
                        position_from_x, ..
                    }
                    | crate::MotionPresetParameters::Scan {
                        position_from_x, ..
                    } => *position_from_x = value,
                    crate::MotionPresetParameters::Pulse { scale_from, .. }
                    | crate::MotionPresetParameters::RadarExpand { scale_from, .. } => {
                        *scale_from = value
                    }
                }
                match compile(entry["id"].as_str().unwrap().into(), 1, parameters) {
                    Err(error) => assert_eq!(error.code, ErrorCode::InvalidArgument),
                    Ok(_) => panic!("nonfinite accepted"),
                }
            }
        }
    }

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
                let (channel, _) = compile_scalar(
                    "scalar_tween".into(),
                    1,
                    crate::ScalarTweenParameters {
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
        let (channel, _) = compile_scalar(
            "scalar_tween".into(),
            1,
            crate::ScalarTweenParameters {
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
                let parameters = crate::ScalarTweenParameters {
                    property: AnimationChannelProperty::Opacity,
                    start_ms: 0,
                    duration_ms: 500,
                    from: if from_endpoint { value } else { 0.0 },
                    to: if from_endpoint { 1.0 } else { value },
                    curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
                };
                assert_eq!(
                    compile_scalar("scalar_tween".into(), 1, parameters)
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
            crate::ScalarTweenParameters {
                property: AnimationChannelProperty::Opacity,
                start_ms: 0,
                duration_ms: 500,
                from: 0.0,
                to: 1.0,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            }
            .into(),
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
        if let AnimationPresetParameters::Scalar(p) = &mut source.parameters {
            p.from = 0.25;
        }
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
            crate::ScalarTweenParameters {
                property: AnimationChannelProperty::PositionX,
                start_ms: 0,
                duration_ms: 500,
                from: 0.0,
                to: 20.0,
                curve: AnimationCurve::Simple(SimpleAnimationCurve::Linear),
            }
            .into(),
            AnimationPresetCollisionPolicy::Reject,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("maxChannelsPerItem"));
        assert_eq!(serde_json::to_value(&manual).unwrap(), before);
    }

    #[test]
    fn migrated_finite_pulse_preserves_samples_through_split_left_trim_copy_and_history() {
        use crate::{AnimationClock, EditOperation, EditorCore, PathPolicy, ProjectSettings};
        use serde_json::{Value, json};
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("media");
        std::fs::create_dir(&media).unwrap();
        let core = EditorCore::new(
            PathPolicy::new(
                root.path().join("projects"),
                [&media],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let id = core
            .create_project("Migrated finite pulse", ProjectSettings::default())
            .unwrap()
            .project_id;
        let track = core.get_project(&id).unwrap().tracks[1].id.clone();
        let op = |value: Value| serde_json::from_value::<EditOperation>(value).unwrap();
        let item = core.edit(&id,0,op(json!({"operation":"add_rectangle","trackId":track,"startMs":0,"durationMs":1000,"width":32,"height":32,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}))).unwrap().changed_ids[0].clone();
        core.edit(&id,1,op(json!({"operation":"apply_animation_preset","itemId":item,"presetId":"pulse","presetVersion":1,"parameters":{"kind":"pulse","startMs":10,"durationMs":80,"scaleFrom":0.5,"scalePeak":1.5,"iterations":2}}))).unwrap();
        let dir = core.paths().project_dir(&id).unwrap();
        let path = dir.join("project.json");
        let mut source = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
        source["schemaVersion"] = json!(30);
        std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        let original = core.get_project(&id).unwrap();
        let original_item = original.find_item(&item).unwrap();
        let original_visual = original_item.visual_properties();
        assert_eq!(original.schema_version, crate::PROJECT_SCHEMA_VERSION);
        assert!(
            original_visual
                .animation_channels
                .iter()
                .all(|c| c.clock.is_none())
        );
        let provenance =
            serde_json::to_value(&original_visual.animation_preset_provenance).unwrap();
        assert_eq!(
            provenance["transform.scale_x"]["parameters"]["kind"],
            "pulse"
        );
        assert_eq!(
            provenance["transform.scale_x"]["parameters"]["iterations"],
            2
        );
        // Fixed independent pulse values include interiors, fractional samples,
        // the second turn/seam and the finite terminal held value. Production
        // sampling of the original source supplies the edit-preservation oracle.
        let samples = [
            (75.0, 0.875),
            (80.0, 0.75),
            (89.5, 0.5125),
            (90.0, 0.5),
            (100.0, 0.75),
            (130.0, 1.5),
            (130.5, 1.4875),
            (150.0, 1.0),
            (169.5, 0.5125),
            (170.0, 0.5),
            (200.5, 0.5),
        ];
        for channel in &original_visual.animation_channels {
            for (source_ms, expected) in samples {
                assert!(
                    (crate::animation::sample_scalar_channel_at(channel, source_ms).unwrap()
                        - expected)
                        .abs()
                        < 1e-12
                );
            }
        }
        let check = |project: &Project, target: &str, offset: i64, samples: &[(f64, f64)]| {
            let visual = project.find_item(target).unwrap().visual_properties();
            assert_eq!(
                serde_json::to_value(&visual.animation_preset_provenance).unwrap(),
                provenance
            );
            for (channel, source) in visual
                .animation_channels
                .iter()
                .zip(&original_visual.animation_channels)
            {
                assert!(same_bytes(&channel.keyframes, &source.keyframes));
                assert_eq!(channel.r#loop, source.r#loop);
                assert_eq!(
                    channel.clock,
                    Some(AnimationClock {
                        offset_ms: offset,
                        source_duration_ms: 1000
                    })
                );
                for &(source_ms, expected) in samples {
                    let before =
                        crate::animation::sample_scalar_channel_at(source, source_ms).unwrap();
                    let after = crate::animation::sample_scalar_channel_at(
                        channel,
                        source_ms - offset as f64,
                    )
                    .unwrap();
                    assert!(
                        (after - before).abs() < 1e-12,
                        "source {source_ms} offset {offset}"
                    );
                    assert!((after - expected).abs() < 1e-12);
                }
            }
        };
        let split = core
            .edit(
                &id,
                2,
                op(json!({"operation":"split_item","itemId":item,"splitMs":45})),
            )
            .unwrap();
        let right = split.changed_ids[1].clone();
        let split_state = core.get_project(&id).unwrap();
        check(
            &split_state,
            &item,
            0,
            &[(0.0, 0.5), (10.0, 0.5), (25.0, 0.875), (44.5, 1.3625)],
        );
        check(
            &split_state,
            &right,
            45,
            &[(45.0, 1.375), (50.0, 1.5), (50.5, 1.4875)],
        );
        check(&split_state, &right, 45, &samples);
        core.edit(
            &id,
            3,
            op(json!({"operation":"trim_item","itemId":right,"startMs":75,"durationMs":900})),
        )
        .unwrap();
        let trim_state = core.get_project(&id).unwrap();
        check(&trim_state, &right, 75, &samples);
        let copied = core
            .edit(
                &id,
                4,
                op(json!({"operation":"duplicate_items","itemIds":[right],"offsetMs":2000})),
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        let copied_state = core.get_project(&id).unwrap();
        check(&copied_state, &right, 75, &samples);
        check(&copied_state, &copied, 75, &samples);
        core.undo(&id, 5).unwrap();
        assert!(same_bytes(
            &core.get_project(&id).unwrap().tracks,
            &trim_state.tracks
        ));
        check(&core.get_project(&id).unwrap(), &right, 75, &samples);
        core.undo(&id, 6).unwrap();
        assert!(same_bytes(
            &core.get_project(&id).unwrap().tracks,
            &split_state.tracks
        ));
        check(&core.get_project(&id).unwrap(), &right, 45, &samples);
        core.undo(&id, 7).unwrap();
        assert!(same_bytes(
            &core.get_project(&id).unwrap().tracks,
            &original.tracks
        ));
        core.redo(&id, 8).unwrap();
        core.redo(&id, 9).unwrap();
        core.redo(&id, 10).unwrap();
        let restored = core.get_project(&id).unwrap();
        assert!(same_bytes(&restored.tracks, &copied_state.tracks));
        let reopened = EditorCore::new(core.paths().clone());
        let reopened_state = reopened.get_project(&id).unwrap();
        assert!(same_bytes(&reopened_state, &restored));
        check(&reopened_state, &right, 75, &samples);
        check(&reopened_state, &copied, 75, &samples);
    }
}
