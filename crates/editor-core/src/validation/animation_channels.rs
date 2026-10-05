use std::collections::BTreeSet;

use crate::{
    AnimationChannel, AnimationChannelProperty, AnimationChannelValue, AnimationCurve,
    AnimationLoopIterations, AnimationLoopMode, CoreError, ErrorCode, Keyframe, KeyframeProperty,
    MediaType, ParameterizedAnimationCurve, Project, TimelineItem,
};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) const MAX_SAFE_CLOCK: i64 = 9_007_199_254_740_991;

pub(crate) fn validate_clock(
    clock: crate::AnimationClock,
    duration_ms: u64,
) -> Result<(), CoreError> {
    if clock.offset_ms.unsigned_abs() > MAX_SAFE_CLOCK as u64
        || clock.source_duration_ms == 0
        || clock.source_duration_ms > MAX_SAFE_CLOCK as u64
        || duration_ms > MAX_SAFE_CLOCK as u64
        || clock
            .offset_ms
            .checked_add(duration_ms as i64)
            .is_none_or(|end| end.unsigned_abs() > MAX_SAFE_CLOCK as u64)
    {
        return Err(invalid("retained animation clock exceeds safe bounds"));
    }
    Ok(())
}

pub(crate) fn validate_legacy_clock(item: &TimelineItem) -> Result<(), CoreError> {
    if let Some(clock) = item.visual_properties().legacy_animation_clock {
        validate_clock(clock, item.duration_ms())?;
        super::validate_legacy_keyframe_limit(item.keyframes())?;
        super::validate_keyframes(item.keyframes()).map_err(|error| invalid(&error.message))?;
        if item.keyframes().is_empty()
            || item
                .keyframes()
                .iter()
                .any(|key| key.time_ms > clock.source_duration_ms)
        {
            return Err(invalid("retained legacy animation source is invalid"));
        }
    }
    Ok(())
}

/// Shared retained-source validation for edits and committed-journal recovery.
fn validate_retained_sources(
    channels: &[AnimationChannel],
    item: &TimelineItem,
) -> Result<(), CoreError> {
    validate_legacy_clock(item)?;
    for channel in channels {
        if let Some(clock) = channel.clock {
            validate_clock(clock, item.duration_ms())?;
            if channel.keyframes.is_empty()
                || channel
                    .keyframes
                    .iter()
                    .any(|key| key.time_ms >= clock.source_duration_ms)
            {
                return Err(invalid("retained animation source is invalid"));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_project_retained_clocks(project: &Project) -> Result<(), CoreError> {
    for item in project
        .tracks
        .iter()
        .chain(
            project
                .components
                .iter()
                .flat_map(|component| &component.tracks),
        )
        .flat_map(|track| &track.items)
    {
        let visual = item.visual_properties();
        if visual.legacy_animation_clock.is_some()
            || visual
                .animation_channels
                .iter()
                .any(|channel| channel.clock.is_some())
        {
            validate_channels(&visual.animation_channels, item, project)?;
        }
    }
    for item in project
        .components
        .iter()
        .flat_map(|component| &component.tracks)
        .flat_map(|track| &track.items)
    {
        let visual = item.visual_properties();
        if visual.legacy_animation_clock.is_some()
            || visual
                .animation_channels
                .iter()
                .any(|channel| channel.clock.is_some())
        {
            super::validate_component_keyframe_limit(item.keyframes())?;
        }
    }
    Ok(())
}

pub(crate) fn validate_legacy_collision(
    channels: &[AnimationChannel],
    keyframes: &[Keyframe],
) -> Result<(), CoreError> {
    for channel in channels {
        let legacy = match channel.property {
            AnimationChannelProperty::PositionX | AnimationChannelProperty::PositionY => {
                KeyframeProperty::Position
            }
            AnimationChannelProperty::ScaleX | AnimationChannelProperty::ScaleY => {
                KeyframeProperty::Scale
            }
            AnimationChannelProperty::Opacity => KeyframeProperty::Opacity,
            AnimationChannelProperty::GainDb => KeyframeProperty::Volume,
            _ => continue,
        };
        if keyframes.iter().any(|keyframe| keyframe.property == legacy) {
            return Err(invalid(
                "typed animation channel conflicts with legacy keyframes",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_channels(
    channels: &[AnimationChannel],
    item: &TimelineItem,
    project: &Project,
) -> Result<(), CoreError> {
    validate_retained_sources(channels, item)?;
    if channels.len() > 64 {
        return Err(invalid("maxChannelsPerItem exceeded"));
    }
    let mut identities = BTreeSet::new();
    for channel in channels {
        if !channel.property.active() {
            return Err(invalid("animation channel is not active"));
        }
        if !channel.property.extended() && channel.target.is_some() {
            return Err(invalid(
                "active animation channel does not accept a target reference",
            ));
        }
        if !identities.insert((channel.property, channel.target.clone())) {
            return Err(invalid("duplicate animation channel"));
        }
        if channel.property.visual() {
            if (channel.property.legacy_visual() && item.visual_properties().transform2d.is_some())
                || !matches!(
                    item,
                    TimelineItem::Media(_)
                        | TimelineItem::Text(_)
                        | TimelineItem::SolidColor(_)
                        | TimelineItem::Rectangle(_)
                        | TimelineItem::Shape(_)
                        | TimelineItem::Svg(_)
                        | TimelineItem::Grid(_)
                        | TimelineItem::Group(_)
                        | TimelineItem::ComponentInstance(_)
                )
            {
                return Err(invalid(
                    "visual animation requires a legacy-transform visual item",
                ));
            }
            if let TimelineItem::Media(media) = item {
                let asset = project
                    .assets
                    .iter()
                    .find(|asset| asset.id == media.asset_id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::AssetNotFound, "animation media asset missing")
                    })?;
                if asset.media_type == MediaType::Audio {
                    return Err(invalid("audio-only media cannot use visual animation"));
                }
            }
        } else {
            let TimelineItem::Media(media) = item else {
                return Err(invalid("audio gain requires media"));
            };
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == media.asset_id)
                .ok_or_else(|| {
                    CoreError::new(ErrorCode::AssetNotFound, "animation media asset missing")
                })?;
            if !asset.has_audio {
                return Err(invalid("audio gain requires media with audio"));
            }
        }
        if channel.property.extended() {
            super::extended_visual::validate_target(channel, item, project)?;
        }
        if channel.keyframes.len() > 1_000 {
            return Err(invalid("maxKeyframesPerChannel exceeded"));
        }
        validate_loop_structure(channel)?;
        let source_duration = channel
            .clock
            .map_or(item.duration_ms(), |clock| clock.source_duration_ms);
        let mut previous = None;
        for (index, keyframe) in channel.keyframes.iter().enumerate() {
            if keyframe.time_ms >= source_duration
                || previous.is_some_and(|time| keyframe.time_ms <= time)
            {
                return Err(invalid(
                    "animation keyframe times must increase within item duration",
                ));
            }
            previous = Some(keyframe.time_ms);
            validate_curve(keyframe.curve, index + 1 == channel.keyframes.len())?;
            if channel.property.extended() {
                super::extended_visual::validate_value(channel, item, &keyframe.value)?;
                continue;
            }
            let AnimationChannelValue::Scalar { value } = keyframe.value else {
                return Err(invalid("active animation channel requires scalar values"));
            };
            validate_scalar_value(channel.property, value)?;
        }
    }
    validate_legacy_collision(channels, item.keyframes())
}

pub(crate) fn validate_curve(curve: AnimationCurve, terminal: bool) -> Result<(), CoreError> {
    match curve {
        AnimationCurve::Simple(_) => {}
        AnimationCurve::Parameterized(curve) => {
            if terminal {
                return Err(invalid("parameterized curve requires a following keyframe"));
            }
            let valid = match curve {
                ParameterizedAnimationCurve::CubicBezier { x1, y1, x2, y2 } => {
                    [x1, y1, x2, y2]
                        .iter()
                        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
                        && x1 <= x2
                }
                ParameterizedAnimationCurve::Spring {
                    mass,
                    stiffness,
                    damping,
                    initial_velocity,
                } => {
                    mass.is_finite()
                        && (0.01..=100.0).contains(&mass)
                        && stiffness.is_finite()
                        && (0.01..=10_000.0).contains(&stiffness)
                        && damping.is_finite()
                        && (0.01..=1_000.0).contains(&damping)
                        && initial_velocity.is_finite()
                        && (-100.0..=100.0).contains(&initial_velocity)
                }
            };
            if !valid {
                return Err(invalid("animation curve parameters exceed finite bounds"));
            }
        }
    }

    Ok(())
}

pub(crate) fn validate_scalar_value(
    property: AnimationChannelProperty,
    value: f64,
) -> Result<(), CoreError> {
    let within = match property {
        AnimationChannelProperty::PositionX | AnimationChannelProperty::PositionY => {
            (-1_000_000.0..=1_000_000.0).contains(&value)
        }
        AnimationChannelProperty::ScaleX | AnimationChannelProperty::ScaleY => {
            value > 0.0 && value <= 100.0
        }
        AnimationChannelProperty::Opacity => (0.0..=1.0).contains(&value),
        AnimationChannelProperty::GainDb => (-96.0..=12.0).contains(&value),
        _ => false,
    };
    if !value.is_finite() || !within {
        return Err(invalid("animation channel value exceeds finite bounds"));
    }
    Ok(())
}

pub(crate) fn validate_loop_structure(channel: &AnimationChannel) -> Result<(), CoreError> {
    if let Some(loop_spec) = channel.r#loop {
        if channel.keyframes.len() < 2 {
            return Err(invalid("animation loop requires two keyframes"));
        }
        if matches!(
            loop_spec.iterations,
            AnimationLoopIterations::Finite(0 | 10_001..)
        ) {
            return Err(invalid("animation loop iterations exceed bounds"));
        }
        if loop_spec.mode == AnimationLoopMode::Repeat
            && channel.keyframes.first().map(|keyframe| &keyframe.value)
                != channel.keyframes.last().map(|keyframe| &keyframe.value)
        {
            return Err(invalid("repeat loop endpoints must match"));
        }
    }
    Ok(())
}
