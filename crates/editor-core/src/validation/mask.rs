//! Canonical stored-mask ownership and metadata validation; no raster activation.
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
        TimelineItem::Media(media) => project.is_none_or(|project| {
            // Missing resources retain the existing ASSET_NOT_FOUND owner/error.
            // Only a known nonvisual asset establishes an ineligible target.
            project
                .assets
                .iter()
                .find(|asset| asset.id == media.asset_id)
                .is_none_or(|asset| matches!(asset.media_type, MediaType::Image | MediaType::Video))
        }),
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
            EditOperation::ComponentCreate { tracks, .. }
            | EditOperation::ComponentUpdate { tracks, .. } => {
                composition(tracks, None)?;
            }
            _ => {}
        }
    }
    Ok(())
}
