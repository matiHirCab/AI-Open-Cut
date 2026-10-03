use crate::{AnimationPresetParameters, CoreError, ErrorCode, MAX_PRESET_TIME_MS, TimelineItem};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) fn validate_parameters(
    parameters: &AnimationPresetParameters,
) -> Result<u64, CoreError> {
    use crate::{
        AnimationChannelProperty as P, AnimationLoopIterations, MotionPresetParameters as M,
    };
    let (start, duration, minimum) = match parameters {
        AnimationPresetParameters::Scalar(p) => {
            if !(p.property.legacy_visual() || p.property == P::GainDb) {
                return Err(invalid("property is not supported by scalar_tween"));
            }
            scalar(p.property, p.from)?;
            scalar(p.property, p.to)?;
            super::animation_channels::validate_curve(p.curve, false)?;
            (p.start_ms, p.duration_ms, 1)
        }
        AnimationPresetParameters::Pack(p) => {
            if let Some(AnimationLoopIterations::Finite(count)) = p.iterations()
                && !(1..=10_000).contains(&count)
            {
                return Err(invalid("preset iterations exceed bounds"));
            }
            let minimum = match p {
                M::ImpactSlam {
                    center_x,
                    center_y,
                    shake_amplitude_px: a,
                    scale_from,
                    scale_overshoot,
                    scale_to,
                    opacity_from,
                    opacity_to,
                    flash_opacity,
                    motion_blur,
                    ..
                } => {
                    if !a.is_finite()
                        || *a <= 0.0
                        || scale_overshoot <= scale_to
                        || flash_opacity >= opacity_to
                    {
                        return Err(invalid("invalid impact_slam endpoint inequalities"));
                    }
                    for x in [
                        *center_x,
                        *center_x + *a,
                        *center_x - *a / 2.0,
                        *center_x + *a / 4.0,
                    ] {
                        scalar(P::PositionX, x)?;
                    }
                    for y in [
                        *center_y,
                        *center_y - *a,
                        *center_y + *a / 2.0,
                        *center_y - *a / 4.0,
                    ] {
                        scalar(P::PositionY, y)?;
                    }
                    for value in [*scale_from, *scale_overshoot, *scale_to] {
                        scalar(P::ScaleX, value)?;
                    }
                    for value in [*opacity_from, *opacity_to, *flash_opacity] {
                        scalar(P::Opacity, value)?;
                    }
                    motion_blur.validate()?;
                    if !motion_blur.enabled() {
                        return Err(invalid("impact_slam requires enabled motionBlur"));
                    }
                    8
                }
                M::SlideLeft {
                    position_from_x,
                    position_to_x,
                    ..
                } => {
                    scalar(P::PositionX, *position_from_x)?;
                    scalar(P::PositionX, *position_to_x)?;
                    if position_from_x <= position_to_x {
                        return Err(invalid("slide_left requires decreasing position"));
                    }
                    1
                }
                M::Scan {
                    position_from_x,
                    position_to_x,
                    ..
                } => {
                    scalar(P::PositionX, *position_from_x)?;
                    scalar(P::PositionX, *position_to_x)?;
                    if position_from_x == position_to_x {
                        return Err(invalid("scan requires distinct endpoints"));
                    }
                    2
                }
                M::Pulse {
                    scale_from,
                    scale_peak,
                    ..
                } => {
                    scalar(P::ScaleX, *scale_from)?;
                    scalar(P::ScaleX, *scale_peak)?;
                    if scale_peak <= scale_from {
                        return Err(invalid("pulse requires increasing scale"));
                    }
                    2
                }
                M::RadarExpand {
                    scale_from,
                    scale_to,
                    opacity_peak,
                    ..
                } => {
                    scalar(P::ScaleX, *scale_from)?;
                    scalar(P::ScaleX, *scale_to)?;
                    scalar(P::Opacity, *opacity_peak)?;
                    if scale_to <= scale_from || *opacity_peak <= 0.0 {
                        return Err(invalid(
                            "radar_expand requires increasing scale and positive opacity",
                        ));
                    }
                    4
                }
            };
            let (start, duration) = p.timing();
            (start, duration, minimum)
        }
    };
    if duration < minimum {
        return Err(invalid("preset duration is below its phase minimum"));
    }
    start
        .checked_add(duration)
        .filter(|end| *end <= MAX_PRESET_TIME_MS)
        .ok_or_else(|| invalid("preset timing exceeds safe integer bounds"))
}

fn scalar(property: crate::AnimationChannelProperty, value: f64) -> Result<(), CoreError> {
    super::animation_channels::validate_scalar_value(property, value)
}

// Validate descriptive data only: there is deliberately no catalog dispatch or expansion here.
pub(crate) fn validate_provenance(item: &TimelineItem) -> Result<(), CoreError> {
    let visual = item.visual_properties();
    if visual.animation_preset_provenance.len() > 64 {
        return Err(invalid("maxPresetProvenancePerItem exceeded"));
    }
    for (property, source) in &visual.animation_preset_provenance {
        let id = source.preset_id.as_bytes();
        if id.is_empty()
            || id.len() > 64
            || !id[0].is_ascii_lowercase()
            || !id
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
            || source.preset_version == 0
            || source.compiler_version == 0
            || !source.parameters.properties().contains(property)
            || !visual
                .animation_channels
                .iter()
                .any(|c| c.property == *property && c.target.is_none())
        {
            return Err(invalid("invalid or orphaned animation preset provenance"));
        }
        if let AnimationPresetParameters::Pack(p) = &source.parameters
            && matches!(
                p,
                crate::MotionPresetParameters::Scan { .. }
                    | crate::MotionPresetParameters::Pulse { .. }
                    | crate::MotionPresetParameters::RadarExpand { .. }
            )
            && p.iterations().is_none()
        {
            return Err(invalid(
                "persisted looped preset requires effective iterations",
            ));
        }
        validate_parameters(&source.parameters)?;
    }
    Ok(())
}
