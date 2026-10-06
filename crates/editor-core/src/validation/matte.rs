//! Canonical authored track-matte eligibility and same-composition DAG rules.
use crate::{CoreError, EditOperation, ErrorCode, MediaType, Project, TimelineItem, Track};
use std::collections::HashMap;

pub(crate) const MAX_MATTE_EDGES_PER_COMPOSITION: usize = 2_048;
pub(crate) const MAX_MATTE_EDGES_PER_PROJECT: usize = 4_096;
pub(crate) const MAX_MATTE_DEPTH: usize = 32;

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}
pub(crate) fn eligible(item: &TimelineItem, assets: Option<&HashMap<&str, MediaType>>) -> bool {
    match item {
        TimelineItem::Media(media) => assets.is_none_or(|a| {
            a.get(media.asset_id.as_str())
                .is_none_or(|kind| matches!(kind, MediaType::Image | MediaType::Video))
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
pub(crate) fn composition(
    tracks: &[Track],
    assets: Option<&HashMap<&str, MediaType>>,
) -> Result<usize, CoreError> {
    // The exact default path adds no DAG tables, even for hidden/unused records.
    if !tracks
        .iter()
        .flat_map(|t| &t.items)
        .any(|i| i.visual_properties().matte.is_some() || i.visual_properties().matte_only)
    {
        return Ok(0);
    }
    let items: Vec<_> = tracks.iter().flat_map(|t| &t.items).collect();
    let by_id: HashMap<_, _> = items.iter().enumerate().map(|(n, i)| (i.id(), n)).collect();
    let mut providers = vec![None; items.len()];
    let mut edges = 0usize;
    for (index, item) in items.iter().enumerate() {
        let visual = item.visual_properties();
        if (visual.matte.is_some() || visual.matte_only) && !eligible(item, assets) {
            return Err(invalid("track mattes require an eligible visual leaf"));
        }
        if let Some(reference) = &visual.matte {
            reference.validate()?;
            let provider = *by_id.get(reference.source_id.as_str()).ok_or_else(|| {
                CoreError::new(
                    ErrorCode::ItemNotFound,
                    "matte provider does not exist in the owning composition",
                )
            })?;
            if index == provider {
                return Err(invalid("a matte cannot reference its own item"));
            }
            if !eligible(items[provider], assets) {
                return Err(invalid("matte provider requires an eligible visual leaf"));
            }
            providers[index] = Some(provider);
            edges = edges
                .checked_add(1)
                .ok_or_else(|| invalid("matte edge count overflow"))?;
            if edges > MAX_MATTE_EDGES_PER_COMPOSITION {
                return Err(invalid("matte edges exceed composition bounds"));
            }
        }
    }
    // A functional graph needs one iterative traversal per unvisited node.
    // Done-node depths allow shared suffixes without repeated traversals.
    let mut state = vec![0u8; items.len()];
    let mut depths = vec![0usize; items.len()];
    let mut path = Vec::new();
    for start in 0..items.len() {
        if state[start] != 0 {
            continue;
        }
        path.clear();
        let mut next = Some(start);
        while let Some(index) = next {
            if state[index] == 1 {
                return Err(invalid("matte provider graph contains a cycle"));
            }
            if state[index] == 2 {
                break;
            }
            state[index] = 1;
            path.push(index);
            next = providers[index];
        }
        let mut depth = next.map_or(0, |index| depths[index]);
        while let Some(index) = path.pop() {
            if providers[index].is_some() {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| invalid("matte depth overflow"))?;
            }
            if depth > MAX_MATTE_DEPTH {
                return Err(invalid("matte provider path exceeds depth bounds"));
            }
            depths[index] = depth;
            state[index] = 2;
        }
    }
    Ok(edges)
}
pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
    if !project
        .tracks
        .iter()
        .chain(project.components.iter().flat_map(|c| &c.tracks))
        .flat_map(|t| &t.items)
        .any(|i| i.visual_properties().matte.is_some() || i.visual_properties().matte_only)
    {
        return Ok(());
    }
    let assets = project
        .assets
        .iter()
        .map(|a| (a.id.as_str(), a.media_type))
        .collect::<HashMap<_, _>>();
    let mut edges = 0usize;
    for tracks in
        std::iter::once(&project.tracks).chain(project.components.iter().map(|c| &c.tracks))
    {
        edges = edges
            .checked_add(composition(tracks, Some(&assets))?)
            .ok_or_else(|| invalid("matte edge count overflow"))?;
        if edges > MAX_MATTE_EDGES_PER_PROJECT {
            return Err(invalid("matte edges exceed project bounds"));
        }
    }
    Ok(())
}
pub(crate) fn validate_operations(operations: &[EditOperation]) -> Result<(), CoreError> {
    for operation in operations {
        match operation {
            EditOperation::UpdateItem {
                matte: Some(Some(reference)),
                ..
            } => reference.validate()?,
            EditOperation::ComponentCreate { tracks, .. }
            | EditOperation::ComponentUpdate { tracks, .. } => {
                for item in tracks.iter().flat_map(|t| &t.items) {
                    let visual = item.visual_properties();
                    if let Some(reference) = &visual.matte {
                        reference.validate()?;
                    }
                    if (visual.matte.is_some() || visual.matte_only) && !eligible(item, None) {
                        return Err(invalid("track mattes require an eligible visual leaf"));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
