//! Scoped cue markers and canonical item start resolution.

use std::collections::{BTreeMap, BTreeSet};

use crate::{CoreError, ErrorCode, Marker, Project, TimeExpression, TimelineItem};

mod speech;
pub(crate) use speech::generate_speech_markers;

pub(crate) const MAX_SAFE: u64 = 9_007_199_254_740_991;

fn invalid(message: &'static str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && value.len() <= 128
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

pub(crate) fn markers_for_scope<'a>(
    project: &'a Project,
    scope: &str,
) -> Result<&'a [Marker], CoreError> {
    if scope == "root" {
        return Ok(&project.markers);
    }
    let id = scope
        .strip_prefix("component:")
        .ok_or_else(|| invalid("invalid marker scope"))?;
    if id.is_empty() {
        return Err(invalid("invalid marker scope"));
    }
    project
        .components
        .iter()
        .find(|component| component.id == id)
        .map(|component| component.markers.as_slice())
        .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "component not found"))
}

pub(crate) fn markers_for_scope_mut<'a>(
    project: &'a mut Project,
    scope: &str,
) -> Result<&'a mut Vec<Marker>, CoreError> {
    if scope == "root" {
        return Ok(&mut project.markers);
    }
    let id = scope
        .strip_prefix("component:")
        .ok_or_else(|| invalid("invalid marker scope"))?;
    if id.is_empty() {
        return Err(invalid("invalid marker scope"));
    }
    project
        .components
        .iter_mut()
        .find(|component| component.id == id)
        .map(|component| &mut component.markers)
        .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "component not found"))
}

pub(crate) fn validate_marker(marker: &Marker, scope: &str) -> Result<(), CoreError> {
    if marker.scope != scope
        || !valid_identifier(&marker.id)
        || !valid_identifier(&marker.name)
        || marker.time_ms > MAX_SAFE
    {
        return Err(invalid("invalid scoped marker"));
    }
    Ok(())
}

pub(crate) fn shift_expression(item: &mut TimelineItem, offset_ms: u64) -> Result<(), CoreError> {
    if let Some(TimeExpression::Marker {
        offset_ms: current, ..
    }) = item.visual_properties_mut().start_time.as_mut()
    {
        let next = i128::from(*current) + i128::from(offset_ms);
        if !(-i128::from(MAX_SAFE)..=i128::from(MAX_SAFE)).contains(&next) {
            return Err(invalid(
                "duplicate marker offset exceeds safe integer bounds",
            ));
        }
        *current = next as i64;
    }
    Ok(())
}

fn validate_collection(markers: &[Marker], scope: &str) -> Result<(), CoreError> {
    if markers.len() > crate::MAX_MARKERS_PER_COMPOSITION {
        return Err(invalid("too many markers in composition"));
    }
    let mut ids = BTreeSet::new();
    for marker in markers {
        validate_marker(marker, scope)?;
        if !ids.insert(&marker.id) {
            return Err(invalid("duplicate marker ID"));
        }
    }
    Ok(())
}

fn resolve(
    markers_by_name: &BTreeMap<&str, Option<u64>>,
    expression: &TimeExpression,
) -> Result<u64, CoreError> {
    match expression {
        TimeExpression::Milliseconds { value_ms } if *value_ms <= MAX_SAFE => Ok(*value_ms),
        TimeExpression::Milliseconds { .. } => {
            Err(invalid("absolute time exceeds safe integer limit"))
        }
        TimeExpression::Marker {
            marker_name,
            offset_ms,
        } => {
            if !valid_identifier(marker_name) || offset_ms.unsigned_abs() > MAX_SAFE {
                return Err(invalid("invalid marker time expression"));
            }
            let marker_time = markers_by_name
                .get(marker_name.as_str())
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "marker name not found"))?
                .ok_or_else(|| invalid("ambiguous marker name"))?;
            let value = i128::from(marker_time) + i128::from(*offset_ms);
            if !(0..=i128::from(MAX_SAFE)).contains(&value) {
                return Err(invalid("marker-relative time outside safe range"));
            }
            Ok(value as u64)
        }
    }
}

pub(crate) fn resolve_time(
    project: &Project,
    scope: &str,
    expression: &TimeExpression,
) -> Result<u64, CoreError> {
    let mut by_name = BTreeMap::new();
    for marker in markers_for_scope(project, scope)? {
        if let Some(value) = by_name.get_mut(marker.name.as_str()) {
            *value = None;
        } else {
            by_name.insert(marker.name.as_str(), Some(marker.time_ms));
        }
    }
    resolve(&by_name, expression)
}

fn check_interval(item: &TimelineItem, start: u64, duration: Option<u64>) -> Result<(), CoreError> {
    let end = start
        .checked_add(item.duration_ms())
        .ok_or_else(|| invalid("marker-relative interval overflow"))?;
    if end > MAX_SAFE || duration.is_some_and(|limit| end > limit) {
        return Err(invalid("marker-relative item exceeds composition bounds"));
    }
    Ok(())
}

fn reconcile_scope(
    markers: &[Marker],
    tracks: &mut [crate::Track],
    duration: Option<u64>,
) -> Result<(), CoreError> {
    let mut markers_by_name = BTreeMap::new();
    for marker in markers {
        if let Some(value) = markers_by_name.get_mut(marker.name.as_str()) {
            *value = None;
        } else {
            markers_by_name.insert(marker.name.as_str(), Some(marker.time_ms));
        }
    }
    for item in tracks.iter_mut().flat_map(|track| &mut track.items) {
        if let Some(expression) = item.visual_properties().start_time.clone() {
            if !matches!(expression, TimeExpression::Marker { .. }) {
                return Err(invalid("stored startTime must be marker-relative"));
            }
            let start = resolve(&markers_by_name, &expression)?;
            check_interval(item, start, duration)?;
            set_item_start(item, start);
        }
    }
    Ok(())
}

pub(crate) fn set_item_start(item: &mut TimelineItem, start_ms: u64) {
    match item {
        TimelineItem::Group(group) => group.start_ms = start_ms,
        TimelineItem::ComponentInstance(item) => item.start_ms = start_ms,
        TimelineItem::Media(media) => media.start_ms = start_ms,
        TimelineItem::Text(text) => text.start_ms = start_ms,
        TimelineItem::SolidColor(shape) => shape.start_ms = start_ms,
        TimelineItem::Rectangle(shape) => shape.start_ms = start_ms,
        TimelineItem::Shape(shape) => shape.start_ms = start_ms,
        TimelineItem::Svg(shape) => shape.start_ms = start_ms,
        TimelineItem::Grid(shape) => shape.start_ms = start_ms,
        TimelineItem::Repeater(item) => item.start_ms = start_ms,
        TimelineItem::Caption(caption) => caption.start_ms = start_ms,
        TimelineItem::Transition(transition) => transition.start_ms = start_ms,
    }
}

pub(crate) fn reconcile_project(project: &mut Project) -> Result<(), CoreError> {
    validate_collection(&project.markers, "root")?;
    reconcile_scope(&project.markers, &mut project.tracks, None)?;
    for component in &mut project.components {
        let scope = format!("component:{}", component.id);
        validate_collection(&component.markers, &scope)?;
        reconcile_scope(
            &component.markers,
            &mut component.tracks,
            Some(component.duration_ms),
        )?;
    }
    Ok(())
}

pub(crate) fn validate_project(project: &Project) -> Result<(), CoreError> {
    if project.markers.is_empty()
        && project
            .components
            .iter()
            .all(|component| component.markers.is_empty())
        && project
            .tracks
            .iter()
            .chain(
                project
                    .components
                    .iter()
                    .flat_map(|component| component.tracks.iter()),
            )
            .flat_map(|track| &track.items)
            .all(|item| item.visual_properties().start_time.is_none())
    {
        return Ok(());
    }
    let mut candidate = project.clone();
    reconcile_project(&mut candidate)?;
    for (actual, expected) in project
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .zip(candidate.tracks.iter().flat_map(|track| &track.items))
    {
        if actual.start_ms() != expected.start_ms() {
            return Err(invalid("stored marker-relative startMs is inconsistent"));
        }
    }
    for (actual, expected) in project.components.iter().zip(&candidate.components) {
        for (item, resolved) in actual
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .zip(expected.tracks.iter().flat_map(|track| &track.items))
        {
            if item.start_ms() != resolved.start_ms() {
                return Err(invalid(
                    "stored component marker-relative startMs is inconsistent",
                ));
            }
        }
    }
    Ok(())
}
