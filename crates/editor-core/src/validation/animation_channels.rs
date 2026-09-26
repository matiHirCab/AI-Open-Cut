use std::collections::BTreeSet;

use crate::{
    AnimationChannel, AnimationChannelProperty, AnimationChannelValue, CoreError, ErrorCode,
    Keyframe, KeyframeProperty, MediaType, Project, TimelineItem,
};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
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
    if channels.len() > 64 {
        return Err(invalid("maxChannelsPerItem exceeded"));
    }
    let mut identities = BTreeSet::new();
    for channel in channels {
        if !channel.property.active() {
            return Err(invalid("animation channel is not active"));
        }
        if channel.target.is_some() {
            return Err(invalid(
                "active animation channel does not accept a target reference",
            ));
        }
        if !identities.insert(channel.property) {
            return Err(invalid("duplicate animation channel"));
        }
        if channel.property.visual() {
            if item.visual_properties().transform2d.is_some()
                || !matches!(
                    item,
                    TimelineItem::Media(_)
                        | TimelineItem::Text(_)
                        | TimelineItem::SolidColor(_)
                        | TimelineItem::Rectangle(_)
                        | TimelineItem::Shape(_)
                        | TimelineItem::Svg(_)
                        | TimelineItem::Grid(_)
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
        if channel.keyframes.len() > 1_000 {
            return Err(invalid("maxKeyframesPerChannel exceeded"));
        }
        let mut previous = None;
        for keyframe in &channel.keyframes {
            if keyframe.time_ms >= item.duration_ms()
                || previous.is_some_and(|time| keyframe.time_ms <= time)
            {
                return Err(invalid(
                    "animation keyframe times must increase within item duration",
                ));
            }
            previous = Some(keyframe.time_ms);
            let AnimationChannelValue::Scalar { value } = keyframe.value else {
                return Err(invalid("active animation channel requires scalar values"));
            };
            let within = match channel.property {
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
        }
    }
    validate_legacy_collision(channels, item.keyframes())
}
