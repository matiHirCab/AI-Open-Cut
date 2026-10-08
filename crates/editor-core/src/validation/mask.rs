//! Canonical stored-mask ownership and structural animation validation.
use crate::{
    CoreError, EditOperation, ErrorCode, MAX_MASK_COMMANDS_PER_COMPOSITION,
    MAX_MASK_COMMANDS_PER_PROJECT, MAX_MASKS_PER_COMPOSITION, MAX_MASKS_PER_ITEM,
    MAX_MASKS_PER_PROJECT, Mask, MediaType, Project, TimelineItem, Track,
};
use std::collections::BTreeSet;

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) fn validate_stack(masks: &[Mask]) -> Result<usize, CoreError> {
    if masks.len() > MAX_MASKS_PER_ITEM {
        return Err(invalid("mask stack exceeds the item limit"));
    }
    let mut ids = BTreeSet::new();
    let mut commands = 0usize;
    for mask in masks {
        mask.validate()?;
        if !ids.insert(&mask.id) {
            return Err(invalid("mask IDs must be unique within an item"));
        }
        commands = commands
            .checked_add(mask.command_count())
            .ok_or_else(|| invalid("mask command count overflow"))?;
    }
    Ok(commands)
}

fn eligible(item: &TimelineItem, project: Option<&Project>) -> bool {
    match item {
        TimelineItem::Media(media) => {
            media.audio_event.is_none()
                && project.is_none_or(|project| {
                    // Missing resources retain the existing ASSET_NOT_FOUND owner/error.
                    // Only a known nonvisual asset establishes an ineligible target.
                    project
                        .assets
                        .iter()
                        .find(|asset| asset.id == media.asset_id)
                        .is_none_or(|asset| {
                            matches!(asset.media_type, MediaType::Image | MediaType::Video)
                        })
                })
        }
        TimelineItem::Text(_)
        | TimelineItem::SolidColor(_)
        | TimelineItem::Rectangle(_)
        | TimelineItem::Shape(_)
        | TimelineItem::Svg(_)
        | TimelineItem::Grid(_) => true,
        _ => false,
    }
}
fn composition(tracks: &[Track], project: Option<&Project>) -> Result<(usize, usize), CoreError> {
    let mut count = 0usize;
    let mut commands = 0usize;
    for item in tracks.iter().flat_map(|track| &track.items) {
        let masks = &item.visual_properties().masks;
        let local = validate_stack(masks)?;
        if !masks.is_empty() && !eligible(item, project) {
            return Err(invalid("nonempty masks require an eligible visual leaf"));
        }
        count = count
            .checked_add(masks.len())
            .ok_or_else(|| invalid("mask count overflow"))?;
        commands = commands
            .checked_add(local)
            .ok_or_else(|| invalid("mask command count overflow"))?;
        if count > MAX_MASKS_PER_COMPOSITION || commands > MAX_MASK_COMMANDS_PER_COMPOSITION {
            return Err(invalid("stored masks exceed composition bounds"));
        }
    }
    Ok((count, commands))
}
pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
    let mut count = 0usize;
    let mut commands = 0usize;
    for tracks in
        std::iter::once(&project.tracks).chain(project.components.iter().map(|c| &c.tracks))
    {
        let (local_count, local_commands) = composition(tracks, Some(project))?;
        count = count
            .checked_add(local_count)
            .ok_or_else(|| invalid("mask count overflow"))?;
        commands = commands
            .checked_add(local_commands)
            .ok_or_else(|| invalid("mask command count overflow"))?;
        if count > MAX_MASKS_PER_PROJECT || commands > MAX_MASK_COMMANDS_PER_PROJECT {
            return Err(invalid("stored masks exceed project bounds"));
        }
    }
    Ok(())
}
// An evicted draft base must not be replaced with current state. Validate its
// known records and explicit component payloads without resolving absent targets.
pub(crate) fn validate_operations(operations: &[EditOperation]) -> Result<(), CoreError> {
    for operation in operations {
        match operation {
            EditOperation::UpdateItem {
                masks: Some(masks), ..
            } => {
                validate_stack(masks)?;
            }
            EditOperation::SetAnimationChannels {
                animation_channels, ..
            } => {
                structural_channels(animation_channels)?;
            }
            EditOperation::ComponentCreate { tracks, .. }
            | EditOperation::ComponentUpdate { tracks, .. } => {
                composition(tracks, None)?;
                for item in tracks.iter().flat_map(|t| &t.items) {
                    structural_channels(&item.visual_properties().animation_channels)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn structural_channels(channels: &[crate::AnimationChannel]) -> Result<(), CoreError> {
    use crate::{AnimationChannelProperty as P, AnimationChannelValue as V, AnimationTargetKind};
    if channels.len() > 64 {
        return Err(invalid("animation channel limit exceeded"));
    }
    let mut identities = BTreeSet::new();
    for channel in channels {
        if channel
            .target
            .as_ref()
            .is_some_and(|t| t.kind == AnimationTargetKind::Mask)
            && !channel.property.mask()
        {
            return Err(invalid("mask target requires a mask property"));
        }
        if !channel.property.mask() {
            continue;
        }
        super::animation_channels::validate_loop_structure(channel)?;
        let target = channel
            .target
            .as_ref()
            .ok_or_else(|| invalid("mask channel requires target"))?;
        if target.kind != AnimationTargetKind::Mask
            || target.id.is_empty()
            || target.id.len() > 128
            || !(target.scope == "root"
                || target
                    .scope
                    .strip_prefix("component:")
                    .is_some_and(|id| !id.is_empty()))
        {
            return Err(invalid("mask animation target exceeds structural bounds"));
        }
        if !identities.insert((channel.property, target.scope.as_str(), target.id.as_str())) {
            return Err(invalid("duplicate mask animation channel"));
        }
        if channel.keyframes.len() > 1000
            || channel
                .keyframes
                .windows(2)
                .any(|p| p[0].time_ms >= p[1].time_ms)
        {
            return Err(invalid("mask animation keyframes exceed bounds"));
        }
        if let Some(clock) = channel.clock {
            super::animation_channels::validate_clock(clock, 1)?;
            if channel.keyframes.is_empty()
                || channel
                    .keyframes
                    .iter()
                    .any(|k| k.time_ms >= clock.source_duration_ms)
            {
                return Err(invalid("mask retained source exceeds bounds"));
            }
        }
        for (index, key) in channel.keyframes.iter().enumerate() {
            super::animation_channels::validate_curve(
                key.curve,
                index + 1 == channel.keyframes.len(),
            )?;
            let okay = match (&key.value, channel.property) {
                (V::PathPoints { points }, P::MaskPathPoints) => {
                    points.len() <= 4096
                        && points.iter().all(|p| {
                            p.x.is_finite()
                                && p.y.is_finite()
                                && p.x.abs() <= 1_000_000.0
                                && p.y.abs() <= 1_000_000.0
                        })
                }
                (V::Rgba { r, g, b, a }, P::MaskPaintColor) => [r, g, b, a]
                    .into_iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
                (V::GradientStops { stops }, P::MaskGradientStops) => {
                    (2..=32).contains(&stops.len())
                        && stops.first().is_some_and(|s| s.offset == 0.0)
                        && stops.last().is_some_and(|s| s.offset == 1.0)
                        && stops.windows(2).all(|p| p[0].offset < p[1].offset)
                        && stops.iter().all(|s| {
                            s.offset.is_finite()
                                && (0.0..=1.0).contains(&s.offset)
                                && s.color
                                    .iter()
                                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                        })
                }
                (V::Scalar { value }, p) => {
                    super::extended_visual::mask_scalar_bounds(p, crate::PositionUnit::Pixels)
                        .is_some_and(|(lo, hi)| {
                            value.is_finite()
                                && if matches!(p, P::MaskTransformScaleX | P::MaskTransformScaleY) {
                                    *value > 0.0 && *value <= hi
                                } else {
                                    (lo..=hi).contains(value)
                                }
                        })
                }
                _ => false,
            };
            if !okay {
                return Err(invalid("mask animation value exceeds structural bounds"));
            }
        }
    }
    Ok(())
}
