//! Canonical timeline mutation, alias, revision, and history rules.

use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;
mod animation_presets;

use crate::{
    AudioSettings, CoreError, EditOperation, ErrorCode, History, KeyframeProperty, Marker,
    MediaItem, MediaType, Project, RectangleItem, SolidColorItem, TextItem, TimeExpression,
    TimelineItem, Track, TrackType, TransitionItem,
    markers::set_item_start,
    validation::{
        validate_audio, validate_color, validate_dimensions, validate_duration,
        validate_item_track, validate_keyframes, validate_text, validate_text_style,
        validate_track_audio_settings, validate_track_media, validate_transform,
        validate_visual_track,
    },
};

pub(crate) const HISTORY_LIMIT: usize = 100;
pub(crate) fn validate_alias(alias: &str) -> Result<(), CoreError> {
    if alias.is_empty()
        || alias.len() > 64
        || !alias
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
        || !alias
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "resultAlias has an invalid format",
        ));
    }
    Ok(())
}

pub(crate) fn is_single_id_creator(edit: &EditOperation) -> bool {
    matches!(
        edit,
        EditOperation::SpeechMarkersGenerate { .. }
            | EditOperation::MarkerCreate { .. }
            | EditOperation::ComponentCreate { .. }
            | EditOperation::AddComponentInstance { .. }
            | EditOperation::ComponentInstanceDuplicate { .. }
            | EditOperation::AddGroup { .. }
            | EditOperation::AddMedia { .. }
            | EditOperation::AddText { .. }
            | EditOperation::AddSolidColor { .. }
            | EditOperation::AddRectangle { .. }
            | EditOperation::AddShape { .. }
            | EditOperation::AddSvg { .. }
            | EditOperation::AddGrid { .. }
            | EditOperation::AddRepeater { .. }
            | EditOperation::AddTransition { .. }
            | EditOperation::CreateTrack { .. }
    )
}

pub(crate) fn resolve_alias(
    value: &mut String,
    aliases: &BTreeMap<String, String>,
) -> Result<(), CoreError> {
    let Some(alias) = value.strip_prefix('@') else {
        return Ok(());
    };
    *value = aliases.get(alias).cloned().ok_or_else(|| {
        CoreError::new(
            ErrorCode::ValidationFailed,
            format!("batch alias @{alias} is missing or referenced before creation"),
        )
    })?;
    Ok(())
}

pub(crate) fn resolve_operation_aliases(
    edit: &mut EditOperation,
    aliases: &BTreeMap<String, String>,
) -> Result<(), CoreError> {
    match edit {
        EditOperation::SpeechMarkersGenerate {
            scope, asset_id, ..
        } => {
            resolve_scope_alias(scope, aliases)?;
            resolve_alias(asset_id, aliases)?;
        }
        EditOperation::MarkerCreate { scope, .. } => resolve_scope_alias(scope, aliases)?,
        EditOperation::MarkerUpdate {
            scope, marker_id, ..
        }
        | EditOperation::MarkerDelete { scope, marker_id } => {
            resolve_scope_alias(scope, aliases)?;
            resolve_alias(marker_id, aliases)?;
        }
        EditOperation::SetItemStartTime { scope, item_id, .. } => {
            resolve_scope_alias(scope, aliases)?;
            resolve_alias(item_id, aliases)?;
        }
        EditOperation::AddComponentInstance {
            track_id,
            component_id,
            parent,
            ..
        } => {
            resolve_alias(track_id, aliases)?;
            resolve_alias(component_id, aliases)?;
            if let Some(parent) = parent {
                resolve_alias(&mut parent.id, aliases)?;
            }
        }
        EditOperation::ComponentInstanceUpdate {
            item_id,
            component_id,
            ..
        } => {
            resolve_alias(item_id, aliases)?;
            resolve_alias(component_id, aliases)?;
        }
        EditOperation::ComponentCreate { tracks, .. } => {
            resolve_component_aliases(tracks, aliases)?
        }
        EditOperation::ComponentUpdate {
            component_id,
            tracks,
            ..
        } => {
            resolve_alias(component_id, aliases)?;
            resolve_component_aliases(tracks, aliases)?;
        }
        EditOperation::ComponentDelete { component_id }
        | EditOperation::ComponentDefineSlots { component_id, .. } => {
            resolve_alias(component_id, aliases)?
        }
        EditOperation::GroupUngroup { group_id } => resolve_alias(group_id, aliases)?,
        EditOperation::AddGroup {
            track_id, parent, ..
        }
        | EditOperation::AddGrid {
            track_id, parent, ..
        }
        | EditOperation::AddSvg {
            track_id, parent, ..
        }
        | EditOperation::AddShape {
            track_id, parent, ..
        } => {
            resolve_alias(track_id, aliases)?;
            if let Some(parent) = parent {
                resolve_alias(&mut parent.id, aliases)?;
            }
        }
        EditOperation::AddRepeater {
            track_id, repeater, ..
        } => {
            resolve_alias(track_id, aliases)?;
            resolve_alias(&mut repeater.source.id, aliases)?;
        }
        EditOperation::UpdateItem {
            item_id,
            repeater,
            matte,
            ..
        } => {
            resolve_alias(item_id, aliases)?;
            if let Some(Some(matte)) = matte {
                resolve_alias(&mut matte.source_id, aliases)?;
            }
            if let Some(repeater) = repeater {
                resolve_alias(&mut repeater.source.id, aliases)?;
            }
        }
        EditOperation::ItemSetParent { item_id, parent } => {
            resolve_alias(item_id, aliases)?;
            if let Some(parent) = parent {
                resolve_alias(&mut parent.id, aliases)?;
            }
        }
        EditOperation::SetAnimationChannels {
            item_id,
            animation_channels,
        } => {
            resolve_alias(item_id, aliases)?;
            for channel in animation_channels {
                if let Some(target) = &mut channel.target {
                    resolve_scope_alias(&mut target.scope, aliases)?;
                    if target.scope == "root"
                        && !channel.property.mask()
                        && !matches!(
                            target.kind,
                            crate::AnimationTargetKind::Effect | crate::AnimationTargetKind::Mask
                        )
                    {
                        resolve_alias(&mut target.id, aliases)?;
                    }
                }
            }
        }
        EditOperation::AddMedia { track_id, .. }
        | EditOperation::AddText { track_id, .. }
        | EditOperation::AddSolidColor { track_id, .. }
        | EditOperation::AddRectangle { track_id, .. } => resolve_alias(track_id, aliases)?,
        EditOperation::ComponentInstanceDuplicate { item_id, .. }
        | EditOperation::ItemSetZIndex { item_id, .. }
        | EditOperation::ItemReorder { item_id, .. }
        | EditOperation::TrimItem { item_id, .. }
        | EditOperation::DeleteItem { item_id }
        | EditOperation::ApplyAnimationPreset { item_id, .. }
        | EditOperation::SetKeyframes { item_id, .. }
        | EditOperation::SetAudio { item_id, .. }
        | EditOperation::SplitItem { item_id, .. }
        | EditOperation::SetItemVisibility { item_id, .. } => resolve_alias(item_id, aliases)?,
        EditOperation::MoveItem {
            item_id, track_id, ..
        } => {
            resolve_alias(item_id, aliases)?;
            resolve_alias(track_id, aliases)?;
        }
        EditOperation::AddTransition {
            track_id,
            from_item_id,
            to_item_id,
            ..
        } => {
            resolve_alias(track_id, aliases)?;
            resolve_alias(from_item_id, aliases)?;
            if let Some(value) = to_item_id {
                resolve_alias(value, aliases)?;
            }
        }
        EditOperation::DuplicateItems { item_ids, .. } => {
            for value in item_ids {
                resolve_alias(value, aliases)?;
            }
        }
        EditOperation::TrackReorder { track_id, .. }
        | EditOperation::UpdateTrack { track_id, .. }
        | EditOperation::DeleteTrack { track_id } => resolve_alias(track_id, aliases)?,
        EditOperation::CreateTrack { .. } => {}
    }
    Ok(())
}

fn resolve_scope_alias(
    scope: &mut String,
    aliases: &BTreeMap<String, String>,
) -> Result<(), CoreError> {
    if let Some(component) = scope.strip_prefix("component:") {
        let mut component = component.to_owned();
        resolve_alias(&mut component, aliases)?;
        *scope = format!("component:{component}");
    }
    Ok(())
}

pub(crate) fn apply_operation(
    project: &mut Project,
    operation: EditOperation,
) -> Result<(Vec<String>, &'static str), CoreError> {
    apply_operation_mode(project, operation, false)
}

pub(crate) fn apply_operation_in_batch(
    project: &mut Project,
    operation: EditOperation,
) -> Result<(Vec<String>, &'static str), CoreError> {
    apply_operation_mode(project, operation, true)
}

fn apply_operation_mode(
    project: &mut Project,
    operation: EditOperation,
    defer_mattes: bool,
) -> Result<(Vec<String>, &'static str), CoreError> {
    let (mut ids, summary) = apply_operation_inner(project, operation)?;
    crate::markers::reconcile_project(project)?;
    for id in normalize_stack_order(project)? {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    if defer_mattes {
        crate::validation::validate_project_visual_properties_without_mattes(project)?;
    } else {
        crate::validation::validate_project_visual_properties(project)?;
    }
    Ok((ids, summary))
}

pub(crate) fn normalize_stack_order(project: &mut Project) -> Result<Vec<String>, CoreError> {
    let mut changed = Vec::new();
    for track in &mut project.tracks {
        for (index, item) in track.items.iter_mut().enumerate() {
            let order = u32::try_from(index).map_err(|_| {
                CoreError::new(ErrorCode::InvalidArgument, "too many items for stack order")
            })?;
            if item.visual_properties().stack_order != order {
                item.visual_properties_mut().stack_order = order;
                changed.push(item.id().to_owned());
            }
        }
    }
    Ok(changed)
}

fn apply_operation_inner(
    project: &mut Project,
    operation: EditOperation,
) -> Result<(Vec<String>, &'static str), CoreError> {
    match operation {
        EditOperation::SpeechMarkersGenerate {
            scope,
            asset_id,
            start_ms,
            marker_policy,
            alignment,
        } => {
            let ids = crate::markers::generate_speech_markers(
                project,
                &scope,
                &asset_id,
                start_ms,
                &marker_policy,
                alignment.as_ref(),
            )?;
            Ok((ids, "Generated speech markers"))
        }
        EditOperation::MarkerCreate {
            scope,
            name,
            time_ms,
            kind,
        } => {
            let id = format!("m_{}", Uuid::new_v4());
            let marker = Marker {
                id: id.clone(),
                name,
                scope: scope.clone(),
                time_ms,
                kind,
            };
            crate::markers::validate_marker(&marker, &scope)?;
            let markers = crate::markers::markers_for_scope_mut(project, &scope)?;
            markers.push(marker);
            Ok((vec![id], "Created marker"))
        }
        EditOperation::MarkerUpdate {
            scope,
            marker_id,
            name,
            time_ms,
            kind,
        } => {
            let marker = crate::markers::markers_for_scope_mut(project, &scope)?
                .iter_mut()
                .find(|marker| marker.id == marker_id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "marker not found"))?;
            marker.name = name;
            marker.time_ms = time_ms;
            marker.kind = kind;
            crate::markers::validate_marker(marker, &scope)?;
            Ok((vec![marker_id], "Updated marker"))
        }
        EditOperation::MarkerDelete { scope, marker_id } => {
            let markers = crate::markers::markers_for_scope_mut(project, &scope)?;
            let index = markers
                .iter()
                .position(|marker| marker.id == marker_id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "marker not found"))?;
            markers.remove(index);
            Ok((vec![marker_id], "Deleted marker"))
        }
        EditOperation::SetItemStartTime {
            scope,
            item_id,
            time,
        } => {
            crate::markers::markers_for_scope(project, &scope)?;
            let item = if scope == "root" {
                find_editable_item_mut(project, &item_id)?
            } else {
                let component_id = scope.strip_prefix("component:").expect("validated scope");
                let component = project
                    .components
                    .iter_mut()
                    .find(|component| component.id == component_id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "component not found")
                    })?;
                let track = component
                    .tracks
                    .iter_mut()
                    .find(|track| track.items.iter().any(|item| item.id() == item_id))
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "timeline item not found")
                    })?;
                if track.locked {
                    return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
                }
                track
                    .items
                    .iter_mut()
                    .find(|item| item.id() == item_id)
                    .expect("found item")
            };
            match time {
                TimeExpression::Milliseconds { value_ms } => {
                    if value_ms > crate::markers::MAX_SAFE
                        || value_ms
                            .checked_add(item.duration_ms())
                            .is_none_or(|end| end > crate::markers::MAX_SAFE)
                    {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "absolute item start exceeds safe integer bounds",
                        ));
                    }
                    set_item_start(item, value_ms);
                    item.visual_properties_mut().start_time = None;
                }
                expression @ TimeExpression::Marker { .. } => {
                    item.visual_properties_mut().start_time = Some(expression);
                }
            }
            Ok((vec![item_id], "Updated item start time"))
        }
        EditOperation::AddComponentInstance {
            track_id,
            component_id,
            start_ms,
            trim_start_ms,
            duration_ms,
            time_scale,
            stagger_ms,
            slot_values,
            transform,
            transform2d,
            hidden,
            z_index,
            parent,
        } => {
            if stagger_ms > 60_000 {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "staggerMs exceeds bounds",
                ));
            }
            if transform2d.is_some() && transform != crate::Transform::default() {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "instance transform forms cannot be combined",
                ));
            }
            let id = Uuid::new_v4().to_string();
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "instances require overlay tracks",
                ));
            }
            let visual_properties = crate::VisualProperties {
                blend_mode: crate::BlendMode::Normal,
                legacy_animation_clock: None,
                crop: None,
                clip: None,
                motion_blur: None,
                effects: Vec::new(),
                masks: Vec::new(),
                matte: None,
                matte_only: false,
                start_time: None,
                animation_channels: Vec::new(),
                animation_preset_provenance: Default::default(),
                transform,
                transform2d,
                hidden,
                z_index,
                parent,
                stack_order: track.items.len() as u32,
            };
            track.items.push(TimelineItem::ComponentInstance(
                crate::ComponentInstanceItem {
                    id: id.clone(),
                    stagger_ms,
                    component_id,
                    start_ms,
                    trim_start_ms,
                    duration_ms,
                    time_scale,
                    slot_values,
                    visual_properties,
                },
            ));
            Ok((vec![id], "created component instance"))
        }
        EditOperation::ComponentInstanceDuplicate {
            item_id,
            offset_ms,
            slot_values,
        } => {
            let (track_index, item_index) = find_item_location(project, &item_id)?;
            let track = &mut project.tracks[track_index];
            if track.locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            let TimelineItem::ComponentInstance(source) = &track.items[item_index] else {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "duplication requires a root component instance",
                ));
            };
            let mut copy = source.clone();
            if let Some(TimeExpression::Marker {
                offset_ms: current, ..
            }) = copy.visual_properties.start_time.as_mut()
            {
                let next = i128::from(*current) + i128::from(offset_ms);
                if !(-9_007_199_254_740_991_i128..=9_007_199_254_740_991_i128).contains(&next) {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "duplicate marker offset exceeds safe integer bounds",
                    ));
                }
                *current = next as i64;
            }
            copy.start_ms = copy
                .start_ms
                .checked_add(offset_ms)
                .filter(|time| *time <= 9_007_199_254_740_991)
                .ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::InvalidArgument,
                        "duplicate time exceeds safe integer bounds",
                    )
                })?;
            copy.id = Uuid::new_v4().to_string();
            copy.visual_properties.stack_order = track.items.len() as u32;
            if let Some(values) = slot_values {
                copy.slot_values = values;
            }
            let id = copy.id.clone();
            track.items.push(TimelineItem::ComponentInstance(copy));
            Ok((vec![id], "duplicated component instance"))
        }
        EditOperation::ComponentInstanceUpdate {
            item_id,
            component_id,
            start_ms,
            trim_start_ms,
            duration_ms,
            time_scale,
            stagger_ms,
            slot_values,
        } => {
            if stagger_ms.is_some_and(|value| value > 60_000) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "staggerMs exceeds bounds",
                ));
            }
            let TimelineItem::ComponentInstance(instance) =
                find_editable_item_mut(project, &item_id)?
            else {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "update requires a component instance",
                ));
            };
            instance.component_id = component_id;
            if instance.start_ms != start_ms {
                instance.visual_properties.start_time = None;
            }
            instance.start_ms = start_ms;
            instance.trim_start_ms = trim_start_ms;
            instance.duration_ms = duration_ms;
            instance.time_scale = time_scale;
            if let Some(value) = stagger_ms {
                instance.stagger_ms = value;
            }
            if let Some(values) = slot_values {
                instance.slot_values = values;
            }
            Ok((vec![item_id], "updated component instance"))
        }
        EditOperation::ComponentCreate {
            name,
            width,
            height,
            duration_ms,
            mut tracks,
            slots,
        } => {
            animation_presets::reconcile_raw_tracks(&mut tracks, &[]);
            let id = Uuid::new_v4().to_string();
            project.components.push(crate::ComponentDefinition {
                id: id.clone(),
                name,
                width,
                height,
                duration_ms,
                tracks,
                slots: slots.unwrap_or_default(),
                markers: Vec::new(),
            });
            Ok((vec![id], "created component"))
        }
        EditOperation::ComponentUpdate {
            component_id,
            name,
            width,
            height,
            duration_ms,
            mut tracks,
            slots,
        } => {
            let current = project
                .components
                .iter_mut()
                .find(|c| c.id == component_id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "component not found"))?;
            animation_presets::reconcile_raw_tracks(&mut tracks, &current.tracks);
            // Full component replacements retain trusted bindings by local
            // identity unless the font selector actually changes. Clients need
            // not echo persisted resource metadata in editing requests.
            for item in tracks.iter_mut().flat_map(|track| &mut track.items) {
                if let TimelineItem::Text(text) = item {
                    let old = current
                        .tracks
                        .iter()
                        .flat_map(|track| &track.items)
                        .find_map(|old| {
                            if let TimelineItem::Text(old) = old
                                && old.id == text.id
                            {
                                Some(old)
                            } else {
                                None
                            }
                        });
                    if text.font_binding.is_some()
                        && text.font_binding.as_ref()
                            != old.and_then(|old| old.font_binding.as_ref())
                    {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "change font selectors rather than retained binding metadata",
                        ));
                    }
                    text.font_binding = old
                        .filter(|old| {
                            old.font_family == text.font_family && old.font_path == text.font_path
                        })
                        .and_then(|old| old.font_binding.clone());
                }
            }
            let locked: Vec<_> = current.tracks.iter().filter(|t| t.locked).collect();
            let retained: Vec<_> = tracks
                .iter()
                .filter(|t| locked.iter().any(|v| v.id == t.id))
                .collect();
            if serde_json::to_value(&locked)? != serde_json::to_value(&retained)? {
                return Err(CoreError::new(
                    ErrorCode::TrackLocked,
                    "component update alters locked tracks",
                ));
            }
            let slots = slots.unwrap_or_else(|| current.slots.clone());
            validate_slot_locks(current, &slots)?;
            *current = crate::ComponentDefinition {
                id: component_id.clone(),
                name,
                width,
                height,
                duration_ms,
                tracks,
                slots,
                markers: current.markers.clone(),
            };
            Ok((vec![component_id], "updated component"))
        }
        EditOperation::ComponentDefineSlots {
            component_id,
            slots,
        } => {
            let current = project
                .components
                .iter_mut()
                .find(|c| c.id == component_id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "component not found"))?;
            validate_slot_locks(current, &slots)?;
            current.slots = slots;
            Ok((vec![component_id], "defined component slots"))
        }
        EditOperation::ComponentDelete { component_id } => {
            let index = project
                .components
                .iter()
                .position(|c| c.id == component_id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "component not found"))?;
            if project.components[index].tracks.iter().any(|t| t.locked) {
                return Err(CoreError::new(
                    ErrorCode::TrackLocked,
                    "component has locked tracks",
                ));
            }
            if project.tracks.iter().chain(project.components.iter().flat_map(|c| &c.tracks)).flat_map(|t| &t.items)
                .any(|i| matches!(i, TimelineItem::ComponentInstance(v) if v.component_id == component_id)) {
                return Err(CoreError::new(ErrorCode::InvalidArgument, "component is referenced"));
            }
            project.components.remove(index);
            Ok((vec![component_id], "deleted component"))
        }
        EditOperation::GroupUngroup { group_id } => {
            let (group_track, group_index) = find_item_location(project, &group_id)?;
            let TimelineItem::Group(group) = &project.tracks[group_track].items[group_index] else {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "ungroup requires a group",
                ));
            };
            let parent = group.visual_properties.parent.clone();
            if project
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .any(|item| matches!(item, TimelineItem::Repeater(r) if r.repeater.source.id == group_id))
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "group is referenced by a repeater",
                ));
            }
            // Check every affected track before changing even the candidate graph.
            for (index, track) in project.tracks.iter().enumerate() {
                let affected = index == group_track
                    || track.items.iter().any(|item| {
                        item.visual_properties()
                            .parent
                            .as_ref()
                            .is_some_and(|p| p.id == group_id)
                    });
                if affected && track.locked {
                    return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
                }
            }
            let mut changed = vec![group_id.clone()];
            for track in &mut project.tracks {
                for item in &mut track.items {
                    if item
                        .visual_properties()
                        .parent
                        .as_ref()
                        .is_some_and(|p| p.id == group_id)
                    {
                        item.visual_properties_mut().parent = parent.clone();
                        changed.push(item.id().to_owned());
                    }
                }
            }
            project.tracks[group_track].items.remove(group_index);
            Ok((changed, "Ungrouped items"))
        }
        EditOperation::AddGroup {
            track_id,
            start_ms,
            duration_ms,
            stagger_ms,
            transform2d,
            parent,
        } => {
            validate_duration(duration_ms)?;
            if stagger_ms > 60_000 {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "staggerMs exceeds bounds",
                ));
            }
            let transform2d = transform2d.unwrap_or_default();
            transform2d.validate()?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "groups require overlay tracks",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Group(crate::GroupItem {
                id: id.clone(),
                stagger_ms,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties {
                    transform2d: Some(transform2d),
                    parent,
                    ..Default::default()
                },
            }));
            Ok((vec![id], "Added group"))
        }
        EditOperation::ItemSetParent { item_id, parent } => {
            let item = find_editable_item_mut(project, &item_id)?;
            if matches!(item, TimelineItem::Repeater(_)) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "repeaters cannot be parented",
                ));
            }
            item.visual_properties_mut().parent = parent;
            Ok((vec![item_id], "Updated item parent"))
        }
        EditOperation::ItemSetZIndex { item_id, z_index } => {
            let (track, index) = find_item_location(project, &item_id)?;
            if project.tracks[track].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            let item = &project.tracks[track].items[index];
            if matches!(item, TimelineItem::Transition(_))
                || matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|asset| asset.id == media.asset_id && asset.media_type == MediaType::Audio))
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "z-index requires a visual source",
                ));
            }
            project.tracks[track].items[index]
                .visual_properties_mut()
                .z_index = z_index;
            Ok((vec![item_id], "Updated item z-index"))
        }
        EditOperation::ItemReorder { item_id, index } => {
            let (track, current) = find_item_location(project, &item_id)?;
            let track = &mut project.tracks[track];
            if track.locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            if index >= track.items.len() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "item index is outside the track",
                ));
            }
            let item = track.items.remove(current);
            track.items.insert(index, item);
            Ok((vec![item_id], "Reordered item"))
        }
        EditOperation::TrackReorder { track_id, index } => apply_operation_inner(
            project,
            EditOperation::UpdateTrack {
                track_id,
                index: Some(index),
                name: None,
                locked: None,
                hidden: None,
                muted: None,
                audio_role: None,
                ducking: None,
            },
        ),
        EditOperation::AddMedia {
            track_id,
            asset_id,
            start_ms,
            duration_ms,
            source_in_ms,
        } => {
            validate_duration(duration_ms)?;
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == asset_id)
                .ok_or_else(|| CoreError::new(ErrorCode::AssetNotFound, "asset was not found"))?;
            let asset_media_type = asset.media_type;
            if let Some(asset_duration) = asset.duration_ms
                && asset_media_type != MediaType::Image
                && source_in_ms.saturating_add(duration_ms) > asset_duration
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "source range exceeds the asset duration",
                ));
            }
            let track = editable_track_mut(project, &track_id)?;
            validate_track_media(track.track_type, asset_media_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Media(MediaItem {
                id: id.clone(),
                asset_id,
                start_ms,
                duration_ms,
                source_in_ms,
                visual_properties: crate::VisualProperties::default(),
                audio: AudioSettings::default(),
                keyframes: vec![],
            }));
            Ok((vec![id], "Added media item"))
        }
        EditOperation::AddText {
            track_id,
            text,
            document,
            start_ms,
            duration_ms,
            font_size,
            color,
            font_family,
            font_path,
            style,
            transform,
        } => {
            let document = match (text, document) {
                (Some(text), None) => {
                    validate_text(&text, font_size, &color)?;
                    crate::RichTextDocument::plain(text)
                }
                (None, Some(document)) => {
                    crate::validation::validate_rich_text(&document)?;
                    document
                }
                _ => {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "provide exactly one of text or document",
                    ));
                }
            };
            let text = document.text();
            crate::validation::validate_text_document(&document, &text)?;
            validate_duration(duration_ms)?;
            validate_transform(&transform)?;
            validate_text(&text, font_size, &color)?;
            validate_text_style(&style)?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "text items require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Text(TextItem {
                font_binding: None,
                id: id.clone(),
                text,
                document,
                start_ms,
                duration_ms,
                font_size,
                color,
                font_family,
                font_path,
                style,
                visual_properties: crate::VisualProperties::new(transform, false),
                keyframes: vec![],
            }));
            Ok((vec![id], "Added text item"))
        }
        EditOperation::AddSolidColor {
            track_id,
            color,
            start_ms,
            duration_ms,
            transform,
        } => {
            validate_duration(duration_ms)?;
            validate_color(&color)?;
            validate_transform(&transform)?;
            let track = editable_track_mut(project, &track_id)?;
            validate_visual_track(track.track_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::SolidColor(SolidColorItem {
                id: id.clone(),
                color,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties::new(transform, false),
                keyframes: vec![],
            }));
            Ok((vec![id], "Added solid color item"))
        }
        EditOperation::AddRepeater {
            track_id,
            start_ms,
            duration_ms,
            repeater,
        } => {
            validate_duration(duration_ms)?;
            crate::validation::repeater::validate_descriptor(&repeater)?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "repeaters require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track
                .items
                .push(TimelineItem::Repeater(crate::RepeaterItem {
                    id: id.clone(),
                    repeater,
                    start_ms,
                    duration_ms,
                    visual_properties: crate::VisualProperties::default(),
                }));
            Ok((vec![id], "Added repeater item"))
        }
        EditOperation::AddGrid {
            track_id,
            start_ms,
            duration_ms,
            grid,
            transform2d,
            parent,
        } => {
            crate::validation::grid::validate_grid(&grid)?;
            let transform2d = transform2d.unwrap_or_default();
            transform2d.validate()?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "shapes require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Grid(crate::GridItem {
                id: id.clone(),
                grid,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties {
                    transform2d: Some(transform2d),
                    parent,
                    ..Default::default()
                },
                keyframes: vec![],
            }));
            Ok((vec![id], "Added grid item"))
        }
        EditOperation::AddSvg {
            track_id,
            start_ms,
            duration_ms,
            svg,
            transform2d,
            parent,
        } => {
            let document = crate::validation::svg::parse(&svg)?;
            let transform2d = transform2d.unwrap_or_default();
            transform2d.validate()?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "shapes require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Svg(crate::SvgItem {
                id: id.clone(),
                document,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties {
                    transform2d: Some(transform2d),
                    parent,
                    ..Default::default()
                },
                keyframes: vec![],
            }));
            Ok((vec![id], "Added SVG item"))
        }
        EditOperation::AddShape {
            track_id,
            start_ms,
            duration_ms,
            geometry,
            fill,
            stroke,
            transform2d,
            parent,
        } => {
            crate::validate_shape(&geometry, &fill, &stroke)?;
            let transform2d = transform2d.unwrap_or_default();
            transform2d.validate()?;
            let track = editable_track_mut(project, &track_id)?;
            if track.track_type != TrackType::Overlay {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "shapes require an overlay track",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Shape(crate::ShapeItem {
                id: id.clone(),
                geometry,
                fill,
                stroke,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties {
                    transform2d: Some(transform2d),
                    parent,
                    ..Default::default()
                },
                keyframes: vec![],
            }));
            Ok((vec![id], "Added shape item"))
        }
        EditOperation::AddRectangle {
            track_id,
            color,
            width,
            height,
            start_ms,
            duration_ms,
            transform,
        } => {
            validate_duration(duration_ms)?;
            validate_color(&color)?;
            validate_dimensions(width, height)?;
            validate_transform(&transform)?;
            let track = editable_track_mut(project, &track_id)?;
            validate_visual_track(track.track_type)?;
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Rectangle(RectangleItem {
                id: id.clone(),
                color,
                width,
                height,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties::new(transform, false),
                keyframes: vec![],
            }));
            Ok((vec![id], "Added rectangle item"))
        }
        EditOperation::UpdateItem {
            item_id,
            crop,
            clip,
            effects,
            masks,
            matte,
            matte_only,
            blend_mode,
            motion_blur,
            grid,
            repeater,
            stagger_ms,
            geometry,
            fill,
            stroke,
            transform,
            transform2d,
            text,
            document,
            color,
            font_size,
            width,
            height,
            font_family,
            font_path,
            style,
        } => {
            if text.is_some() && document.is_some() {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "text and document cannot be updated together",
                ));
            }
            if transform.is_some() && transform2d.is_some() {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "transform and transform2d cannot be updated together",
                ));
            }
            let is_audio = project.find_item(&item_id).is_some_and(|item| {
                matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|asset| asset.id == media.asset_id && asset.media_type == MediaType::Audio))
            });
            let item = find_editable_item_mut(project, &item_id)?;
            if let Some(value) = clip {
                item.visual_properties_mut().clip = value;
            }
            if let Some(value) = crop {
                item.visual_properties_mut().crop = Some(*value);
            }
            if let Some(value) = motion_blur {
                value.validate()?;
                animation_presets::blur_changed(item.visual_properties_mut(), value);
                item.visual_properties_mut().motion_blur = Some(value);
            }
            if let Some(value) = matte {
                if let Some(reference) = &value {
                    reference.validate()?;
                }
                item.visual_properties_mut().matte = value;
            }
            if let Some(value) = matte_only {
                item.visual_properties_mut().matte_only = value;
            }
            if let Some(value) = blend_mode {
                item.visual_properties_mut().blend_mode = value;
            }
            if let Some(value) = masks {
                item.visual_properties_mut().masks = value;
            }
            if let Some(value) = effects {
                item.visual_properties_mut().effects = value;
            }
            if let Some(value) = stagger_ms {
                if value > 60_000 {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "staggerMs exceeds bounds",
                    ));
                }
                match item {
                    TimelineItem::Group(group) => group.stagger_ms = value,
                    TimelineItem::ComponentInstance(instance) => instance.stagger_ms = value,
                    _ => {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "staggerMs requires a group or component instance",
                        ));
                    }
                }
            }
            if let Some(document) = document {
                let TimelineItem::Text(text_item) = item else {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "document requires a text item",
                    ));
                };
                crate::validation::validate_rich_text(&document)?;
                let text = document.text();
                crate::validation::validate_text_document(&document, &text)?;
                text_item.text = text;
                text_item.document = document;
            }
            if let Some(repeater) = repeater {
                let TimelineItem::Repeater(item) = item else {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "repeater requires a repeater item",
                    ));
                };
                crate::validation::repeater::validate_descriptor(&repeater)?;
                item.repeater = *repeater;
            }
            if let Some(grid) = grid {
                let TimelineItem::Grid(item) = item else {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "grid requires a grid item",
                    ));
                };
                crate::validation::grid::validate_grid(&grid)?;
                item.grid = *grid;
            }
            if geometry.is_some() || fill.is_some() || stroke.is_some() {
                let TimelineItem::Shape(shape) = item else {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "geometry, fill and stroke require a shape item",
                    ));
                };
                if let Some(v) = geometry {
                    shape.geometry = *v;
                }
                if let Some(v) = fill {
                    shape.fill = v.map(|paint| *paint);
                }
                if let Some(v) = stroke {
                    shape.stroke = v.map(|stroke| *stroke);
                }
                crate::validate_shape(&shape.geometry, &shape.fill, &shape.stroke)?;
            }
            if let Some(value) = transform2d {
                if is_audio
                    || matches!(
                        item,
                        TimelineItem::Transition(_) | TimelineItem::Repeater(_)
                    )
                {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "Transform2D requires a visual source",
                    ));
                }
                if let Some(value) = &value {
                    value.validate()?;
                    if item
                        .keyframes()
                        .iter()
                        .any(|key| key.property != KeyframeProperty::Volume)
                    {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "Transform2D cannot use legacy transform keyframes",
                        ));
                    }
                }
                item.visual_properties_mut().transform2d = value;
            }
            if let Some(transform) = transform {
                item.visual_properties_mut().transform2d = None;
                validate_transform(&transform)?;
                match item {
                    TimelineItem::Media(media) => media.transform = transform,
                    TimelineItem::ComponentInstance(instance) => {
                        instance.visual_properties.transform = transform
                    }
                    TimelineItem::Group(_) => {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "groups do not accept legacy transforms",
                        ));
                    }
                    TimelineItem::Text(text_item) => text_item.transform = transform,
                    TimelineItem::SolidColor(item) => item.transform = transform,
                    TimelineItem::Rectangle(item) => item.transform = transform,
                    TimelineItem::Shape(item) => item.transform = transform,
                    TimelineItem::Svg(item) => item.transform = transform,
                    TimelineItem::Grid(item) => item.transform = transform,
                    TimelineItem::Repeater(_) => {
                        return Err(CoreError::new(
                            ErrorCode::InvalidArgument,
                            "repeaters do not accept common transforms",
                        ));
                    }
                    TimelineItem::Caption(_) => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "captions do not have transforms",
                        ));
                    }
                    TimelineItem::Transition(_) => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "transitions do not have transforms",
                        ));
                    }
                }
            }
            if let Some(text) = text {
                match item {
                    TimelineItem::Text(text_item) => {
                        validate_text(&text, text_item.font_size, &text_item.color)?;
                        text_item.document = crate::RichTextDocument::plain(text.clone());
                        text_item.text = text;
                    }
                    TimelineItem::Caption(caption) => {
                        validate_text(&text, caption.style.font_size, &caption.style.color)?;
                        caption.text = text;
                    }
                    _ => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "only text items accept text updates",
                        ));
                    }
                }
            }
            if let Some(color) = color {
                validate_color(&color)?;
                match item {
                    TimelineItem::Text(text) => text.color = color,
                    TimelineItem::SolidColor(shape) => shape.color = color,
                    TimelineItem::Rectangle(shape) => shape.color = color,
                    _ => {
                        return Err(CoreError::new(
                            ErrorCode::ValidationFailed,
                            "item does not accept color updates",
                        ));
                    }
                }
            }
            if width.is_some() || height.is_some() {
                let TimelineItem::Rectangle(rectangle) = item else {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "dimensions require a rectangle item",
                    ));
                };
                let width = width.unwrap_or(rectangle.width);
                let height = height.unwrap_or(rectangle.height);
                validate_dimensions(width, height)?;
                rectangle.width = width;
                rectangle.height = height;
            }
            if let Some(value) = font_size {
                let TimelineItem::Text(text) = item else {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "font size updates require a text item",
                    ));
                };
                if !(1..=1_000).contains(&value) {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "font size must be in [1,1000]",
                    ));
                }
                text.font_size = value;
            }
            if font_family.is_some() || font_path.is_some() || style.is_some() {
                let TimelineItem::Text(text) = item else {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "font and style updates require a text item",
                    ));
                };
                if let Some(value) = font_family {
                    text.font_binding = None;
                    text.font_family = value;
                }
                if let Some(value) = font_path {
                    text.font_binding = None;
                    text.font_path = value;
                }
                if let Some(value) = style {
                    validate_text_style(&value)?;
                    text.style = *value;
                }
            }
            Ok((vec![item_id], "Updated timeline item"))
        }
        EditOperation::MoveItem {
            item_id,
            track_id,
            start_ms,
        } => {
            ensure_item_track_unlocked(project, &item_id)?;
            let mut item = remove_item(project, &item_id)?;
            set_item_start(&mut item, start_ms);
            item.visual_properties_mut().start_time = None;
            let track = editable_track_mut(project, &track_id)?;
            validate_item_track(&item, track.track_type)?;
            track.items.push(item);
            Ok((vec![item_id], "Moved timeline item"))
        }
        EditOperation::TrimItem {
            item_id,
            start_ms,
            duration_ms,
            source_in_ms,
        } => {
            validate_duration(duration_ms)?;
            let item = find_editable_item_mut(project, &item_id)?;
            let previous_start = item.start_ms();
            let delta = i128::from(start_ms) - i128::from(previous_start);
            retain_animation_clock(item, delta, duration_ms)?;
            if previous_start != start_ms {
                item.visual_properties_mut().start_time = None;
            }
            match item {
                TimelineItem::Media(media) => {
                    media.start_ms = start_ms;
                    media.duration_ms = duration_ms;
                    if let Some(source_in_ms) = source_in_ms {
                        media.source_in_ms = source_in_ms;
                    }
                }
                TimelineItem::Text(text) => {
                    text.start_ms = start_ms;
                    text.duration_ms = duration_ms;
                }
                TimelineItem::ComponentInstance(_) => {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "use component_instance_update to change instance timing",
                    ));
                }
                TimelineItem::Group(group) => {
                    group.start_ms = start_ms;
                    group.duration_ms = duration_ms;
                }
                TimelineItem::SolidColor(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Rectangle(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Shape(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Svg(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Grid(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Repeater(item) => {
                    item.start_ms = start_ms;
                    item.duration_ms = duration_ms;
                }
                TimelineItem::Caption(caption) => {
                    caption.start_ms = start_ms;
                    caption.duration_ms = duration_ms;
                }
                TimelineItem::Transition(transition) => {
                    transition.start_ms = start_ms;
                    transition.duration_ms = duration_ms;
                }
            }
            Ok((vec![item_id], "Trimmed timeline item"))
        }
        EditOperation::DeleteItem { item_id } => {
            ensure_item_track_unlocked(project, &item_id)?;
            if project.tracks.iter().flat_map(|track| &track.items).any(
                |item| matches!(item, TimelineItem::Repeater(r) if r.repeater.source.id == item_id),
            ) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "item is referenced by a repeater",
                ));
            }
            if project
                .tracks
                .iter()
                .flat_map(|track| &track.items)
                .any(|item| {
                    item.visual_properties()
                        .parent
                        .as_ref()
                        .is_some_and(|parent| parent.id == item_id)
                })
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "group has surviving children",
                ));
            }
            remove_item(project, &item_id)?;
            for track in &mut project.tracks {
                track.items.retain(|item| match item {
                    TimelineItem::Transition(transition) => {
                        transition.from_item_id != item_id
                            && transition.to_item_id.as_deref() != Some(&item_id)
                    }
                    _ => true,
                });
            }
            Ok((vec![item_id], "Deleted timeline item"))
        }
        EditOperation::SetKeyframes { item_id, keyframes } => {
            validate_keyframes(&keyframes)?;
            let item = find_editable_item_mut(project, &item_id)?;
            crate::validation::animation_channels::validate_legacy_collision(
                &item.visual_properties().animation_channels,
                &keyframes,
            )?;
            if matches!(
                item,
                TimelineItem::Group(_)
                    | TimelineItem::ComponentInstance(_)
                    | TimelineItem::Repeater(_)
            ) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "groups and component instances do not accept keyframes",
                ));
            }
            if keyframes
                .iter()
                .any(|keyframe| keyframe.property == KeyframeProperty::Volume)
                && !matches!(item, TimelineItem::Media(_))
            {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "volume keyframes require a media item",
                ));
            }

            if item.visual_properties().transform2d.is_some()
                && keyframes
                    .iter()
                    .any(|key| key.property != KeyframeProperty::Volume)
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "Transform2D cannot use legacy transform keyframes",
                ));
            }
            let destination = item.keyframes_mut().ok_or_else(|| {
                CoreError::new(
                    ErrorCode::ValidationFailed,
                    "transitions do not accept transform keyframes",
                )
            })?;
            *destination = keyframes;
            item.visual_properties_mut().legacy_animation_clock = None;
            Ok((vec![item_id], "Set item keyframes"))
        }
        EditOperation::ApplyAnimationPreset {
            item_id,
            preset_id,
            preset_version,
            parameters,
            collision_policy,
        } => {
            animation_presets::apply(
                project,
                &item_id,
                preset_id,
                preset_version,
                parameters,
                collision_policy,
            )?;
            Ok((vec![item_id], "Applied animation preset"))
        }
        EditOperation::SetAnimationChannels {
            item_id,
            animation_channels,
        } => {
            let (track_index, item_index) = find_item_location(project, &item_id)?;
            if project.tracks[track_index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            crate::validation::animation_channels::validate_channels(
                &animation_channels,
                &project.tracks[track_index].items[item_index],
                project,
            )?;
            let visual = project.tracks[track_index].items[item_index].visual_properties_mut();
            visual.animation_channels = animation_channels;
            visual.animation_preset_provenance.clear();
            Ok((vec![item_id], "Set item animation channels"))
        }
        EditOperation::AddTransition {
            track_id,
            transition_type,
            from_item_id,
            to_item_id,
            start_ms,
            duration_ms,
        } => {
            validate_duration(duration_ms)?;
            if project.find_item(&from_item_id).is_none()
                || to_item_id
                    .as_ref()
                    .is_some_and(|id| project.find_item(id).is_none())
            {
                return Err(CoreError::new(
                    ErrorCode::ItemNotFound,
                    "transition endpoint was not found",
                ));
            }
            if std::iter::once(&from_item_id)
                .chain(to_item_id.iter())
                .any(|id| {
                    matches!(
                        project.find_item(id),
                        Some(
                            TimelineItem::Group(_)
                                | TimelineItem::ComponentInstance(_)
                                | TimelineItem::Repeater(_)
                        )
                    )
                })
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "groups and component instances cannot be transition endpoints",
                ));
            }
            let track = editable_track_mut(project, &track_id)?;
            if matches!(track.track_type, TrackType::Audio | TrackType::Caption) {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "visual transitions cannot be added to audio tracks",
                ));
            }
            let id = Uuid::new_v4().to_string();
            track.items.push(TimelineItem::Transition(TransitionItem {
                id: id.clone(),
                transition_type,
                from_item_id,
                to_item_id,
                start_ms,
                duration_ms,
                visual_properties: crate::VisualProperties::default(),
            }));
            Ok((vec![id], "Added transition"))
        }
        EditOperation::SetAudio { item_id, audio } => {
            validate_audio(&audio)?;
            let item = find_editable_item_mut(project, &item_id)?;
            match item {
                TimelineItem::Group(_)
                | TimelineItem::ComponentInstance(_)
                | TimelineItem::Shape(_)
                | TimelineItem::Svg(_)
                | TimelineItem::Grid(_) => {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "this item does not accept audio",
                    ));
                }
                TimelineItem::Repeater(_) => {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "this item does not accept audio",
                    ));
                }
                TimelineItem::Media(media) => media.audio = audio,
                _ => {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "audio settings require a media item",
                    ));
                }
            }
            Ok((vec![item_id], "Updated item audio"))
        }
        EditOperation::SplitItem { item_id, split_ms } => {
            let (track_index, item_index) = find_item_location(project, &item_id)?;
            if project.tracks[track_index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            let item = &mut project.tracks[track_index].items[item_index];
            if matches!(
                item,
                TimelineItem::Group(_) | TimelineItem::ComponentInstance(_)
            ) {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "groups and component instances cannot be split",
                ));
            }
            if split_ms <= item.start_ms() || split_ms >= item.end_ms() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "split time must be strictly inside the item",
                ));
            }
            let right_id = Uuid::new_v4().to_string();
            let right_duration = item.end_ms() - split_ms;
            let left_duration = split_ms - item.start_ms();
            retain_animation_clock(item, 0, left_duration)?;
            let mut right = match item {
                TimelineItem::Group(_) | TimelineItem::ComponentInstance(_) => {
                    return Err(CoreError::new(
                        ErrorCode::InvalidArgument,
                        "groups and component instances cannot be split",
                    ));
                }
                TimelineItem::Media(media) => {
                    let mut right = media.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    right.source_in_ms = right.source_in_ms.saturating_add(left_duration);
                    media.duration_ms = left_duration;
                    TimelineItem::Media(right)
                }
                TimelineItem::Text(text) => {
                    let mut right = text.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    text.duration_ms = left_duration;
                    TimelineItem::Text(right)
                }
                TimelineItem::SolidColor(shape) => {
                    let mut right = shape.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    shape.duration_ms = left_duration;
                    TimelineItem::SolidColor(right)
                }
                TimelineItem::Rectangle(shape) => {
                    let mut right = shape.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    shape.duration_ms = left_duration;
                    TimelineItem::Rectangle(right)
                }
                TimelineItem::Shape(shape) => {
                    let mut right = shape.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    shape.duration_ms = left_duration;
                    TimelineItem::Shape(right)
                }
                TimelineItem::Svg(shape) => {
                    let mut right = shape.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    shape.duration_ms = left_duration;
                    TimelineItem::Svg(right)
                }
                TimelineItem::Grid(shape) => {
                    let mut right = shape.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    shape.duration_ms = left_duration;
                    TimelineItem::Grid(right)
                }
                TimelineItem::Repeater(repeater) => {
                    let mut right = repeater.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    repeater.duration_ms = left_duration;
                    TimelineItem::Repeater(right)
                }
                TimelineItem::Caption(caption) => {
                    let mut right = caption.clone();
                    right.id = right_id.clone();
                    right.start_ms = split_ms;
                    right.duration_ms = right_duration;
                    caption.duration_ms = left_duration;
                    TimelineItem::Caption(right)
                }
                TimelineItem::Transition(_) => {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "transitions cannot be split",
                    ));
                }
            };
            remap_animation_self_targets(&mut right, &item_id, &right_id);
            retain_animation_clock(&mut right, i128::from(left_duration), right_duration)?;
            item.visual_properties_mut().start_time = None;
            right.visual_properties_mut().start_time = None;
            project.tracks[track_index]
                .items
                .insert(item_index + 1, right);
            Ok((vec![item_id, right_id], "Split timeline item"))
        }
        EditOperation::DuplicateItems {
            item_ids,
            offset_ms,
        } => {
            if item_ids.is_empty() || item_ids.len() > 100 {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "duplicate requires between 1 and 100 items",
                ));
            }
            let mut copies = Vec::with_capacity(item_ids.len());
            for item_id in item_ids {
                let (track_index, item_index) = find_item_location(project, &item_id)?;
                if project.tracks[track_index].locked {
                    return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
                }
                let mut copy = project.tracks[track_index].items[item_index].clone();
                if matches!(copy, TimelineItem::Transition(_)) {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "transitions cannot be duplicated",
                    ));
                }
                let new_start = copy.start_ms().checked_add(offset_ms).ok_or_else(|| {
                    CoreError::new(ErrorCode::ValidationFailed, "duplicate time overflow")
                })?;
                let new_id = Uuid::new_v4().to_string();
                let copy_duration = copy.duration_ms();
                retain_animation_clock(&mut copy, 0, copy_duration)?;
                remap_animation_self_targets(&mut copy, &item_id, &new_id);
                set_item_id(&mut copy, new_id.clone());
                set_item_start(&mut copy, new_start);
                crate::markers::shift_expression(&mut copy, offset_ms)?;
                copies.push((track_index, copy, new_id));
            }
            let mut changed_ids = Vec::with_capacity(copies.len());
            for (track_index, copy, id) in copies {
                project.tracks[track_index].items.push(copy);
                changed_ids.push(id);
            }
            Ok((changed_ids, "Duplicated timeline items"))
        }
        EditOperation::CreateTrack {
            name,
            track_type,
            index,
            audio_role,
            ducking,
        } => {
            let name = name.trim();
            if name.is_empty() || name.chars().count() > 128 {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "track name must be non-empty and at most 128 characters",
                ));
            }
            let id = Uuid::new_v4().to_string();
            validate_track_audio_settings(track_type, audio_role, ducking.as_ref())?;
            let track = Track {
                id: id.clone(),
                name: name.into(),
                track_type,
                locked: false,
                hidden: false,
                muted: false,
                audio_role,
                ducking,
                items: vec![],
            };
            let index = index.unwrap_or(project.tracks.len());
            if index > project.tracks.len() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "track index is outside the timeline",
                ));
            }
            project.tracks.insert(index, track);
            Ok((vec![id], "Created track"))
        }
        EditOperation::UpdateTrack {
            track_id,
            name,
            index,
            locked,
            hidden,
            muted,
            audio_role,
            ducking,
        } => {
            let current_index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))?;
            if project.tracks[current_index].locked
                && !(locked == Some(false)
                    && name.is_none()
                    && index.is_none()
                    && hidden.is_none()
                    && muted.is_none()
                    && audio_role.is_none()
                    && ducking.is_none())
            {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            if let Some(name) = name {
                let name = name.trim();
                if name.is_empty() || name.chars().count() > 128 {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "track name must be non-empty and at most 128 characters",
                    ));
                }
                project.tracks[current_index].name = name.into();
            }
            if let Some(locked) = locked {
                project.tracks[current_index].locked = locked;
            }
            if let Some(hidden) = hidden {
                project.tracks[current_index].hidden = hidden;
            }
            if let Some(muted) = muted {
                project.tracks[current_index].muted = muted;
            }
            if audio_role.is_some() || ducking.is_some() {
                let role = audio_role.unwrap_or(project.tracks[current_index].audio_role);
                let settings =
                    ducking.unwrap_or_else(|| project.tracks[current_index].ducking.clone());
                validate_track_audio_settings(
                    project.tracks[current_index].track_type,
                    role,
                    settings.as_ref(),
                )?;
                project.tracks[current_index].audio_role = role;
                project.tracks[current_index].ducking = settings;
            }
            if let Some(index) = index {
                if index >= project.tracks.len() {
                    return Err(CoreError::new(
                        ErrorCode::ValidationFailed,
                        "track index is outside the timeline",
                    ));
                }
                let track = project.tracks.remove(current_index);
                project.tracks.insert(index, track);
            }
            Ok((vec![track_id], "Updated track"))
        }
        EditOperation::DeleteTrack { track_id } => {
            let index = project
                .tracks
                .iter()
                .position(|track| track.id == track_id)
                .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))?;
            if project.tracks[index].locked {
                return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
            }
            if !project.tracks[index].items.is_empty() {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "only empty tracks can be deleted",
                ));
            }
            project.tracks.remove(index);
            Ok((vec![track_id], "Deleted track"))
        }
        EditOperation::SetItemVisibility { item_id, hidden } => {
            let item = find_editable_item_mut(project, &item_id)?;
            item.set_hidden(hidden);
            Ok((vec![item_id], "Updated item visibility"))
        }
    }
}

pub(crate) fn validate_operations_against(
    project: &Project,
    operations: &[EditOperation],
) -> Result<(), CoreError> {
    let mut candidate = project.clone();
    for operation in operations.iter().cloned() {
        apply_operation_in_batch(&mut candidate, operation)?;
    }
    crate::validation::matte::validate_project(&candidate)?;
    Ok(())
}

pub(crate) fn find_track_mut<'a>(
    project: &'a mut Project,
    track_id: &str,
) -> Result<&'a mut Track, CoreError> {
    project
        .tracks
        .iter_mut()
        .find(|track| track.id == track_id)
        .ok_or_else(|| CoreError::new(ErrorCode::TrackNotFound, "track was not found"))
}

pub(crate) fn editable_track_mut<'a>(
    project: &'a mut Project,
    track_id: &str,
) -> Result<&'a mut Track, CoreError> {
    let track = find_track_mut(project, track_id)?;
    if track.locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(track)
}

pub(crate) fn find_editable_item_mut<'a>(
    project: &'a mut Project,
    item_id: &str,
) -> Result<&'a mut TimelineItem, CoreError> {
    let (track_index, item_index) = find_item_location(project, item_id)?;
    if project.tracks[track_index].locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(&mut project.tracks[track_index].items[item_index])
}

pub(crate) fn find_item_location(
    project: &Project,
    item_id: &str,
) -> Result<(usize, usize), CoreError> {
    project
        .tracks
        .iter()
        .enumerate()
        .find_map(|(track_index, track)| {
            track
                .items
                .iter()
                .position(|item| item.id() == item_id)
                .map(|item_index| (track_index, item_index))
        })
        .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "timeline item was not found"))
}

pub(crate) fn ensure_item_track_unlocked(
    project: &Project,
    item_id: &str,
) -> Result<(), CoreError> {
    let (track_index, _) = find_item_location(project, item_id)?;
    if project.tracks[track_index].locked {
        return Err(CoreError::new(ErrorCode::TrackLocked, "track is locked"));
    }
    Ok(())
}

pub(crate) fn remove_item(project: &mut Project, item_id: &str) -> Result<TimelineItem, CoreError> {
    for track in &mut project.tracks {
        if let Some(index) = track.items.iter().position(|item| item.id() == item_id) {
            return Ok(track.items.remove(index));
        }
    }
    Err(CoreError::new(
        ErrorCode::ItemNotFound,
        "timeline item was not found",
    ))
}

pub(crate) fn set_item_id(item: &mut TimelineItem, id: String) {
    match item {
        TimelineItem::Group(group) => group.id = id,
        TimelineItem::ComponentInstance(item) => item.id = id,
        TimelineItem::Media(media) => media.id = id,
        TimelineItem::Text(text) => text.id = id,
        TimelineItem::SolidColor(shape) => shape.id = id,
        TimelineItem::Rectangle(shape) => shape.id = id,
        TimelineItem::Shape(shape) => shape.id = id,
        TimelineItem::Svg(shape) => shape.id = id,
        TimelineItem::Grid(shape) => shape.id = id,
        TimelineItem::Repeater(item) => item.id = id,
        TimelineItem::Caption(caption) => caption.id = id,
        TimelineItem::Transition(transition) => transition.id = id,
    }
}

pub(crate) fn check_revision(project: &Project, expected_revision: u64) -> Result<(), CoreError> {
    if project.revision != expected_revision {
        return Err(CoreError::new(
            ErrorCode::RevisionConflict,
            format!(
                "expected revision {expected_revision}, current revision is {}",
                project.revision
            ),
        ));
    }
    Ok(())
}

pub(crate) fn bump_revision(project: &mut Project) -> Result<(), CoreError> {
    project.revision = project
        .revision
        .checked_add(1)
        .ok_or_else(|| CoreError::new(ErrorCode::InternalError, "project revision overflow"))?;
    project.updated_at_ms = now_ms()?;
    Ok(())
}

pub(crate) fn push_undo(history: &mut History, project: &Project) {
    history.undo.push(project.clone());
    if history.undo.len() > HISTORY_LIMIT {
        history.undo.remove(0);
    }
    history.redo.clear();
}

pub(crate) fn now_ms() -> Result<u64, CoreError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CoreError::new(ErrorCode::InternalError, error.to_string()))?
        .as_millis();
    u64::try_from(millis)
        .map_err(|_| CoreError::new(ErrorCode::InternalError, "system time overflow"))
}

fn remap_animation_self_targets(item: &mut TimelineItem, source_id: &str, new_id: &str) {
    for channel in &mut item.visual_properties_mut().animation_channels {
        if let Some(target) = &mut channel.target
            && matches!(
                target.kind,
                crate::AnimationTargetKind::GraphicGeometry
                    | crate::AnimationTargetKind::GraphicFill
                    | crate::AnimationTargetKind::GraphicStroke
            )
            && target.id == source_id
        {
            target.id = new_id.to_owned();
        }
    }
}

/// Keep the source function intact while changing the item's active window.
fn retain_animation_clock(
    item: &mut TimelineItem,
    delta: i128,
    new_duration: u64,
) -> Result<(), CoreError> {
    let old_duration = item.duration_ms();
    let legacy_duration = item
        .keyframes()
        .iter()
        .map(|key| key.time_ms)
        .max()
        .unwrap_or(0)
        .max(old_duration);
    let has_legacy = !item.keyframes().is_empty();
    let shifted = |clock: Option<crate::AnimationClock>,
                   source_duration|
     -> Result<crate::AnimationClock, CoreError> {
        let mut clock = clock.unwrap_or(crate::AnimationClock {
            offset_ms: 0,
            source_duration_ms: source_duration,
        });
        clock.offset_ms = i64::try_from(i128::from(clock.offset_ms) + delta).map_err(|_| {
            CoreError::new(
                ErrorCode::InvalidArgument,
                "retained animation clock overflow",
            )
        })?;
        crate::validation::animation_channels::validate_clock(clock, new_duration)?;
        Ok(clock)
    };
    let visual = item.visual_properties_mut();
    if has_legacy {
        visual.legacy_animation_clock =
            Some(shifted(visual.legacy_animation_clock, legacy_duration)?);
    }
    for channel in &mut visual.animation_channels {
        if !channel.keyframes.is_empty() {
            channel.clock = Some(shifted(channel.clock, old_duration)?);
        }
    }
    Ok(())
}

fn resolve_component_aliases(
    tracks: &mut [Track],
    aliases: &BTreeMap<String, String>,
) -> Result<(), CoreError> {
    for item in tracks.iter_mut().flat_map(|t| &mut t.items) {
        for channel in &mut item.visual_properties_mut().animation_channels {
            if let Some(target) = &mut channel.target {
                resolve_scope_alias(&mut target.scope, aliases)?;
            }
        }
        if let TimelineItem::ComponentInstance(instance) = item {
            resolve_alias(&mut instance.component_id, aliases)?;
        }
        if let TimelineItem::Repeater(repeater) = item {
            resolve_alias(&mut repeater.repeater.source.id, aliases)?;
        }
    }
    Ok(())
}

fn validate_slot_locks(
    current: &crate::ComponentDefinition,
    slots: &[crate::TemplateSlot],
) -> Result<(), CoreError> {
    for slot in current.slots.iter().chain(slots) {
        let changed = !current.slots.contains(slot) || !slots.contains(slot);
        if changed
            && current.tracks.iter().any(|track| {
                track.locked
                    && track
                        .items
                        .iter()
                        .any(|item| item.id() == slot.binding.target_layer_id)
            })
        {
            return Err(CoreError::new(
                ErrorCode::TrackLocked,
                "slot edit alters a locked target",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PROJECT_SCHEMA_VERSION, ProjectSettings, Transform};

    fn project() -> Project {
        Project {
            markers: Vec::new(),
            fonts: Default::default(),
            components: vec![],
            schema_version: PROJECT_SCHEMA_VERSION,
            id: "project".into(),
            revision: 4,
            name: "Project".into(),
            created_at_ms: 1,
            updated_at_ms: 1,
            settings: ProjectSettings::default(),
            assets: vec![],
            tracks: vec![Track {
                id: "overlay".into(),
                name: "Overlay".into(),
                track_type: TrackType::Overlay,
                locked: false,
                hidden: false,
                muted: false,
                audio_role: crate::AudioTrackRole::Unassigned,
                ducking: None,
                items: vec![],
            }],
        }
    }

    #[test]
    fn aliases_are_validated_and_resolved_before_application() {
        assert!(validate_alias("overlay-track").is_ok());
        assert!(validate_alias("@bad").is_err());
        let mut operation = EditOperation::AddText {
            track_id: "@track".into(),
            document: None,
            text: Some("Hello".into()),
            start_ms: 0,
            duration_ms: 1_000,
            font_size: 48,
            color: "#ffffff".into(),
            font_family: None,
            font_path: None,
            style: crate::TextStyle::default(),
            transform: Transform::default(),
        };
        resolve_operation_aliases(
            &mut operation,
            &BTreeMap::from([("track".into(), "overlay".into())]),
        )
        .unwrap();
        let (changed, _) = apply_operation(&mut project(), operation).unwrap();
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn candidate_validation_rolls_back_and_history_is_revisioned() {
        let mut project = project();
        let invalid = EditOperation::AddText {
            track_id: "missing".into(),
            document: None,
            text: Some("Hello".into()),
            start_ms: 0,
            duration_ms: 1_000,
            font_size: 48,
            color: "#ffffff".into(),
            font_family: None,
            font_path: None,
            style: crate::TextStyle::default(),
            transform: Transform::default(),
        };
        assert!(validate_operations_against(&project, &[invalid]).is_err());
        assert!(project.tracks[0].items.is_empty());

        let previous = project.clone();
        let mut history = History::default();
        push_undo(&mut history, &previous);
        bump_revision(&mut project).unwrap();
        assert_eq!(project.revision, 5);
        assert_eq!(history.undo.len(), 1);
        assert!(check_revision(&project, 4).is_err());
        assert!(check_revision(&project, 5).is_ok());
    }
}
