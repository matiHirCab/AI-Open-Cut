//! Core-only blend applicability, reused inside the existing authored walk.
use crate::{CoreError, ErrorCode, MediaType, Project, TimelineItem};

pub(crate) fn validate_item(item: &TimelineItem, project: &Project) -> Result<(), CoreError> {
    if item.visual_properties().blend_mode.is_normal() {
        return Ok(());
    }
    let eligible = match item {
        TimelineItem::Media(media) => project
            .assets
            .iter()
            .find(|asset| asset.id == media.asset_id)
            .is_none_or(|asset| matches!(asset.media_type, MediaType::Image | MediaType::Video)),
        TimelineItem::Text(_)
        | TimelineItem::SolidColor(_)
        | TimelineItem::Rectangle(_)
        | TimelineItem::Shape(_)
        | TimelineItem::Svg(_)
        | TimelineItem::Grid(_)
        | TimelineItem::Caption(_) => true,
        _ => false,
    };
    if !eligible {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "non-normal blend requires a visual source leaf or Caption",
        ));
    }
    Ok(())
}
