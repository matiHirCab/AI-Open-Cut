//! Canonical editor-domain validation.
//!
//! Transports, persistence, rendering infrastructure, and presentation code call
//! these rules rather than maintaining parallel validation implementations.

use std::collections::BTreeMap;

use crate::{
    AudioSettings, AudioTrackRole, CoreError, DuckingSettings, ErrorCode, Keyframe,
    KeyframeProperty, KeyframeValue, MediaType, Project, ProjectSettings, TextStyle, TimelineItem,
    TrackType, Transform,
};
pub(crate) fn validate_draft_label(label: Option<&str>) -> Result<(), CoreError> {
    if label.is_some_and(|value| value.trim().is_empty() || value.chars().count() > 200) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "draft label must be non-empty and at most 200 characters",
        ));
    }
    Ok(())
}

pub(crate) fn validate_project_settings(settings: &ProjectSettings) -> Result<(), CoreError> {
    if settings.width == 0
        || settings.height == 0
        || settings.width > 7_680
        || settings.height > 4_320
        || !(1..=120).contains(&settings.fps)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "resolution or frame rate is outside supported bounds",
        ));
    }
    Ok(())
}

pub(crate) fn validate_duration(duration_ms: u64) -> Result<(), CoreError> {
    if duration_ms == 0 {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "duration must be greater than zero",
        ));
    }
    Ok(())
}

pub(crate) fn validate_transform(transform: &Transform) -> Result<(), CoreError> {
    if !transform.position_x.is_finite()
        || !transform.position_y.is_finite()
        || !transform.scale.is_finite()
        || !transform.opacity.is_finite()
        || transform.scale <= 0.0
        || transform.scale > 100.0
        || !(0.0..=1.0).contains(&transform.opacity)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "transform contains an invalid position, scale, or opacity",
        ));
    }
    Ok(())
}

pub(crate) fn validate_project_stacking(project: &Project) -> Result<(), CoreError> {
    for track in &project.tracks {
        for (index, item) in track.items.iter().enumerate() {
            if item.visual_properties().stack_order as usize != index {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "stackOrder must match item array position",
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_project_visual_properties(project: &Project) -> Result<(), CoreError> {
    validate_project_stacking(project)?;
    validate_parent_graph(project)?;
    for item in project.tracks.iter().flat_map(|track| &track.items) {
        validate_transform(&item.visual_properties().transform)?;
        if let Some(value) = &item.visual_properties().transform2d {
            value.validate()?;
            if matches!(item, TimelineItem::Transition(_))
                || matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|asset| asset.id == media.asset_id && asset.media_type == MediaType::Audio))
                || item
                    .keyframes()
                    .iter()
                    .any(|key| key.property != KeyframeProperty::Volume)
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "Transform2D requires a visual source without legacy transform keyframes",
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_parent_graph(project: &Project) -> Result<(), CoreError> {
    if project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .any(|i| matches!(i, TimelineItem::ComponentInstance(_)))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "root component instances are not supported",
        ));
    }
    validate_scope(project, &project.tracks, "root")?;
    validate_components(project)
}

fn validate_scope(
    project: &Project,
    tracks: &[crate::Track],
    scope: &str,
) -> Result<(), CoreError> {
    let invalid = |message: &str| CoreError::new(ErrorCode::InvalidArgument, message);
    let is_visual = |item: &TimelineItem| {
        !matches!(item, TimelineItem::Transition(_))
            && !matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|asset| asset.id == media.asset_id && asset.media_type == MediaType::Audio))
    };
    if tracks
        .iter()
        .flat_map(|track| &track.items)
        .filter(|item| is_visual(item))
        .count()
        > 4096
    {
        return Err(invalid("maxLayersPerComposition exceeded"));
    }
    let mut index = BTreeMap::new();
    for track in tracks {
        for item in &track.items {
            if index.insert(item.id(), item).is_some() {
                return Err(invalid("duplicate timeline item ID"));
            }
            if let TimelineItem::Group(group) = item {
                if track.track_type != TrackType::Overlay {
                    return Err(invalid("groups require overlay tracks"));
                }
                validate_duration(group.duration_ms)?;
                if group.start_ms.checked_add(group.duration_ms).is_none() {
                    return Err(invalid("group interval overflows"));
                }
                group
                    .visual_properties
                    .transform2d
                    .unwrap_or_default()
                    .validate()?;
                if group.visual_properties.transform != Transform::default() {
                    return Err(invalid("groups do not accept legacy transforms"));
                }
            }
            if item.visual_properties().parent.is_some() && !is_visual(item) {
                return Err(invalid("parenting requires a visual item"));
            }
        }
    }
    for item in tracks.iter().flat_map(|track| &track.items) {
        if let TimelineItem::Transition(transition) = item
            && std::iter::once(&transition.from_item_id)
                .chain(transition.to_item_id.iter())
                .any(|id| {
                    matches!(
                        index.get(id.as_str()),
                        Some(TimelineItem::Group(_) | TimelineItem::ComponentInstance(_))
                    )
                })
        {
            return Err(invalid("groups cannot be transition endpoints"));
        }
        let mut current = item;
        let mut visited = vec![item.id()];
        while let Some(parent) = &current.visual_properties().parent {
            if parent.scope != scope
                || parent.id.is_empty()
                || parent.id.len() > 128
                || !parent
                    .id
                    .bytes()
                    .all(|v| v.is_ascii_alphanumeric() || matches!(v, b'_' | b'-'))
            {
                return Err(invalid("parent reference must name a root group"));
            }
            let target = index
                .get(parent.id.as_str())
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "parent group not found"))?;
            if !matches!(target, TimelineItem::Group(_)) {
                return Err(invalid("parent must be a group"));
            }
            if visited.contains(&parent.id.as_str()) {
                return Err(invalid("parent cycle"));
            }
            if visited.len() > 32 {
                return Err(invalid("maxParentDepth exceeded"));
            }
            visited.push(parent.id.as_str());
            current = target;
        }
    }
    Ok(())
}

pub(crate) fn validate_text(text: &str, font_size: u32, color: &str) -> Result<(), CoreError> {
    if text.is_empty() || text.len() > 4_096 || !(1..=1_000).contains(&font_size) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text, font size, or color is invalid",
        ));
    }
    validate_color(color)?;
    Ok(())
}

pub(crate) fn validate_text_style(style: &TextStyle) -> Result<(), CoreError> {
    validate_color(&style.outline_color)?;
    validate_color(&style.shadow.color)?;
    validate_color(&style.background_color)?;
    if style
        .wrap_width_px
        .is_some_and(|value| value == 0 || value > 7_680)
        || style.outline_width_px > 100
        || !style.shadow.opacity.is_finite()
        || !(0.0..=1.0).contains(&style.shadow.opacity)
        || !style.background_opacity.is_finite()
        || !(0.0..=1.0).contains(&style.background_opacity)
        || style.line_spacing_px.unsigned_abs() > 4_320
        || [
            style.padding.top,
            style.padding.right,
            style.padding.bottom,
            style.padding.left,
        ]
        .into_iter()
        .any(|value| value > 4_320)
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text style is outside supported bounds",
        ));
    }
    Ok(())
}

pub(crate) fn validate_dimensions(width: u32, height: u32) -> Result<(), CoreError> {
    if width == 0 || height == 0 || width > 7_680 || height > 4_320 {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "shape dimensions are outside supported bounds",
        ));
    }
    Ok(())
}

pub(crate) fn validate_color(color: &str) -> Result<(), CoreError> {
    if color.len() != 7
        || !color.starts_with('#')
        || !color.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "color must use #RRGGBB format",
        ));
    }
    Ok(())
}

pub(crate) fn validate_audio(audio: &AudioSettings) -> Result<(), CoreError> {
    if !audio.volume.is_finite() || !(0.0..=4.0).contains(&audio.volume) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "audio volume must be between 0 and 4",
        ));
    }
    Ok(())
}

pub(crate) fn validate_keyframes(keyframes: &[Keyframe]) -> Result<(), CoreError> {
    let mut previous_by_property = BTreeMap::new();
    for keyframe in keyframes {
        match (keyframe.property, &keyframe.value) {
            (KeyframeProperty::Position, KeyframeValue::Position { x, y })
                if x.is_finite() && y.is_finite() => {}
            (KeyframeProperty::Scale, KeyframeValue::Scalar { value })
                if value.is_finite() && *value > 0.0 && *value <= 100.0 => {}
            (KeyframeProperty::Opacity, KeyframeValue::Scalar { value })
                if value.is_finite() && (0.0..=1.0).contains(value) => {}
            (KeyframeProperty::Volume, KeyframeValue::Scalar { value })
                if value.is_finite() && (0.0..=4.0).contains(value) => {}
            _ => {
                return Err(CoreError::new(
                    ErrorCode::ValidationFailed,
                    "keyframe value does not match its property",
                ));
            }
        }
        if let Some(time_ms) = previous_by_property.get(&keyframe.property)
            && keyframe.time_ms <= *time_ms
        {
            return Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "keyframes for a property must be strictly increasing",
            ));
        }
        previous_by_property.insert(keyframe.property, keyframe.time_ms);
    }
    Ok(())
}

pub(crate) fn validate_ducking(settings: &DuckingSettings) -> Result<(), CoreError> {
    if !settings.gain.is_finite()
        || !(0.0..=1.0).contains(&settings.gain)
        || settings.attack_ms > 60_000
        || settings.release_ms > 60_000
    {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "ducking settings are invalid",
        ));
    }
    Ok(())
}

pub(crate) fn validate_track_audio_settings(
    track_type: TrackType,
    role: AudioTrackRole,
    ducking: Option<&DuckingSettings>,
) -> Result<(), CoreError> {
    if track_type != TrackType::Audio && (role != AudioTrackRole::Unassigned || ducking.is_some()) {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "audio roles require an audio track",
        ));
    }
    if ducking.is_some() && role != AudioTrackRole::Music {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "ducking settings require a music track",
        ));
    }
    if let Some(settings) = ducking {
        validate_ducking(settings)?;
    }
    Ok(())
}

pub(crate) fn validate_visual_track(track: TrackType) -> Result<(), CoreError> {
    if matches!(track, TrackType::Video | TrackType::Overlay) {
        Ok(())
    } else {
        Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "visual items require a video or overlay track",
        ))
    }
}

pub(crate) fn validate_track_media(track: TrackType, media: MediaType) -> Result<(), CoreError> {
    let allowed = matches!(
        (track, media),
        (TrackType::Video, MediaType::Image | MediaType::Video)
            | (TrackType::Overlay, MediaType::Image | MediaType::Video)
            | (TrackType::Audio, MediaType::Audio)
    );
    if !allowed {
        return Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "media type is incompatible with the destination track",
        ));
    }
    Ok(())
}

pub(crate) fn validate_item_track(item: &TimelineItem, track: TrackType) -> Result<(), CoreError> {
    match item {
        TimelineItem::Text(_) if track != TrackType::Overlay => Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "text items require an overlay track",
        )),
        TimelineItem::SolidColor(_) | TimelineItem::Rectangle(_)
            if !matches!(track, TrackType::Video | TrackType::Overlay) =>
        {
            Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "shape items require a video or overlay track",
            ))
        }
        TimelineItem::Caption(_) if track != TrackType::Caption => Err(CoreError::new(
            ErrorCode::ValidationFailed,
            "caption items require a caption track",
        )),
        TimelineItem::Transition(_) if matches!(track, TrackType::Audio | TrackType::Caption) => {
            Err(CoreError::new(
                ErrorCode::ValidationFailed,
                "visual transitions require a video or overlay track",
            ))
        }
        _ => Ok(()),
    }
}

pub(crate) fn validate_components(project: &Project) -> Result<(), CoreError> {
    let invalid = |message: &str| CoreError::new(ErrorCode::InvalidArgument, message);
    let safe_time = 9_007_199_254_740_991u64;
    if project.components.len() > 512
        || project
            .components
            .iter()
            .map(|c| c.tracks.len())
            .sum::<usize>()
            > 4096
        || project
            .components
            .iter()
            .flat_map(|c| &c.tracks)
            .map(|t| t.items.len())
            .sum::<usize>()
            > 4096
    {
        return Err(invalid("component collection limit exceeded"));
    }
    let identifier = |id: &str| {
        !id.is_empty()
            && id.len() <= 128
            && id
                .bytes()
                .all(|v| v.is_ascii_alphanumeric() || matches!(v, b'_' | b'-'))
    };
    let mut definitions = BTreeMap::new();
    for (i, component) in project.components.iter().enumerate() {
        if !identifier(&component.id) || definitions.insert(component.id.as_str(), i).is_some() {
            return Err(invalid("invalid or duplicate component ID"));
        }
    }
    let mut edges = vec![Vec::new(); project.components.len()];
    for (i, component) in project.components.iter().enumerate() {
        if component.name.trim().is_empty()
            || component.name.len() > 4096
            || component.width == 0
            || component.width > 7680
            || component.height == 0
            || component.height > 4320
            || component.duration_ms == 0
            || component.duration_ms > safe_time
        {
            return Err(invalid("invalid component metadata"));
        }
        validate_scope(
            project,
            &component.tracks,
            &format!("component:{}", component.id),
        )
        .map_err(|error| {
            if error.code == ErrorCode::ItemNotFound {
                error
            } else {
                invalid(&error.message)
            }
        })?;
        let mut track_ids = std::collections::BTreeSet::new();
        for track in &component.tracks {
            if !identifier(&track.id)
                || !track_ids.insert(&track.id)
                || track.name.trim().is_empty()
                || track.name.len() > 128
            {
                return Err(invalid("invalid or duplicate local track ID"));
            }
            validate_track_audio_settings(
                track.track_type,
                track.audio_role,
                track.ducking.as_ref(),
            )
            .map_err(|e| invalid(&e.message))?;
            for (position, item) in track.items.iter().enumerate() {
                if !identifier(item.id())
                    || item.duration_ms() == 0
                    || item.duration_ms() > safe_time
                    || item.start_ms() > safe_time
                    || item
                        .start_ms()
                        .checked_add(item.duration_ms())
                        .is_none_or(|end| end > component.duration_ms)
                    || item.visual_properties().stack_order as usize != position
                {
                    return Err(invalid(
                        "invalid component item identity, interval or ordering",
                    ));
                }
                validate_item_track(item, track.track_type).map_err(|e| invalid(&e.message))?;
                validate_transform(&item.visual_properties().transform)
                    .map_err(|e| invalid(&e.message))?;
                if let Some(transform) = item.visual_properties().transform2d {
                    transform.validate()?;
                }
                if item.keyframes().len() > 10_000 {
                    return Err(invalid("component keyframe limit exceeded"));
                }
                validate_keyframes(item.keyframes()).map_err(|e| invalid(&e.message))?;
                if item.keyframes().iter().any(|key| {
                    key.time_ms > safe_time
                        || (key.property == KeyframeProperty::Volume
                            && !matches!(item, TimelineItem::Media(_)))
                }) {
                    return Err(invalid("invalid component keyframe time or item property"));
                }
                if item.visual_properties().transform2d.is_some()
                    && item
                        .keyframes()
                        .iter()
                        .any(|k| k.property != KeyframeProperty::Volume)
                {
                    return Err(invalid("Transform2D conflicts with legacy keyframes"));
                }
                match item {
                    TimelineItem::ComponentInstance(instance) => {
                        if track.track_type != TrackType::Overlay
                            || instance.visual_properties.transform != Transform::default()
                            || !instance.time_scale.is_finite()
                            || instance.time_scale <= 0.0
                            || instance.trim_start_ms > safe_time
                        {
                            return Err(invalid("invalid nested component instance"));
                        }
                        let target =
                            *definitions
                                .get(instance.component_id.as_str())
                                .ok_or_else(|| {
                                    CoreError::new(
                                        ErrorCode::ItemNotFound,
                                        "component definition not found",
                                    )
                                })?;
                        let source_end = instance.trim_start_ms as f64
                            + instance.duration_ms as f64 * instance.time_scale;
                        if !source_end.is_finite()
                            || source_end > project.components[target].duration_ms as f64
                        {
                            return Err(invalid(
                                "component instance source interval exceeds definition duration",
                            ));
                        }
                        edges[i].push(target);
                    }
                    TimelineItem::Media(media) => {
                        let asset = project
                            .assets
                            .iter()
                            .find(|a| a.id == media.asset_id)
                            .ok_or_else(|| {
                                CoreError::new(
                                    ErrorCode::ItemNotFound,
                                    "component media asset not found",
                                )
                            })?;
                        validate_track_media(track.track_type, asset.media_type)
                            .map_err(|e| invalid(&e.message))?;
                        validate_audio(&media.audio).map_err(|e| invalid(&e.message))?;
                        if media.audio.fade_in_ms > safe_time || media.audio.fade_out_ms > safe_time
                        {
                            return Err(invalid("component media fade exceeds safe integer time"));
                        }
                        let end = media
                            .source_in_ms
                            .checked_add(media.duration_ms)
                            .ok_or_else(|| invalid("media source interval overflows"))?;
                        if media.source_in_ms > safe_time
                            || (asset.media_type != MediaType::Image
                                && asset.duration_ms.is_some_and(|d| end > d))
                        {
                            return Err(invalid("component media interval exceeds asset"));
                        }
                        if asset.media_type == MediaType::Audio
                            && media.visual_properties.transform2d.is_some()
                        {
                            return Err(invalid("audio cannot use Transform2D"));
                        }
                    }
                    TimelineItem::Text(text) => {
                        validate_text(&text.text, text.font_size, &text.color)
                            .map_err(|e| invalid(&e.message))?;
                        validate_text_style(&text.style).map_err(|e| invalid(&e.message))?;
                    }
                    TimelineItem::SolidColor(shape) => {
                        validate_color(&shape.color).map_err(|e| invalid(&e.message))?;
                    }
                    TimelineItem::Rectangle(shape) => {
                        validate_dimensions(shape.width, shape.height)
                            .map_err(|e| invalid(&e.message))?;
                        validate_color(&shape.color).map_err(|e| invalid(&e.message))?;
                    }
                    TimelineItem::Caption(caption) => {
                        if !project
                            .assets
                            .iter()
                            .any(|a| a.id == caption.source.asset_id)
                        {
                            return Err(CoreError::new(
                                ErrorCode::ItemNotFound,
                                "component caption asset not found",
                            ));
                        }
                        validate_text(&caption.text, caption.style.font_size, &caption.style.color)
                            .map_err(|e| invalid(&e.message))?;
                        validate_color(&caption.style.background_color)
                            .map_err(|e| invalid(&e.message))?;
                        let source = &caption.source;
                        let confidence_valid = |value: Option<f64>| {
                            value.is_none_or(|v| v.is_finite() && (0.0..=1.0).contains(&v))
                        };
                        if source.provider_id.trim().is_empty()
                            || source.provider_id.len() > 128
                            || source.model_id.trim().is_empty()
                            || source.model_id.len() > 128
                            || source.language.trim().is_empty()
                            || source.model_version.as_ref().is_some_and(|v| v.is_empty())
                            || source.original_text.trim().is_empty()
                            || source.original_text.len() > 4096
                            || source.generated_at_ms == 0
                            || source.generated_at_ms > safe_time
                            || !confidence_valid(source.confidence)
                            || caption.style.bottom_margin_px > 4320
                            || source.words.iter().any(|word| {
                                word.word.trim().is_empty()
                                    || word.start_ms > safe_time
                                    || word.end_ms > safe_time
                                    || word.start_ms >= word.end_ms
                                    || !confidence_valid(word.confidence)
                            })
                        {
                            return Err(invalid("invalid component caption provenance or style"));
                        }
                    }
                    TimelineItem::Transition(transition) => {
                        for id in std::iter::once(&transition.from_item_id)
                            .chain(transition.to_item_id.iter())
                        {
                            if !component
                                .tracks
                                .iter()
                                .flat_map(|t| &t.items)
                                .any(|v| v.id() == id)
                            {
                                return Err(CoreError::new(
                                    ErrorCode::ItemNotFound,
                                    "component transition endpoint not found",
                                ));
                            }
                        }
                        if transition.visual_properties.transform2d.is_some() {
                            return Err(invalid("transition cannot use Transform2D"));
                        }
                    }
                    TimelineItem::Group(_) => {}
                }
            }
        }
    }
    // A bounded iterative leaf removal computes the longest path without recursive expansion.
    let mut depths = vec![None; edges.len()];
    for _ in 0..=edges.len() {
        let mut progress = false;
        for (i, children) in edges.iter().enumerate() {
            if depths[i].is_some() {
                continue;
            }
            if children.iter().all(|child| depths[*child].is_some()) {
                let depth = children
                    .iter()
                    .map(|child| depths[*child].unwrap() + 1usize)
                    .max()
                    .unwrap_or(0);
                if depth > 16 {
                    return Err(invalid("maxComponentDepth exceeded"));
                }
                depths[i] = Some(depth);
                progress = true;
            }
        }
        if depths.iter().all(Option::is_some) {
            return Ok(());
        }
        if !progress {
            return Err(invalid("component dependency cycle"));
        }
    }
    Err(invalid("component dependency cycle"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Easing, TextItem};

    #[test]
    fn covers_scalar_style_and_shape_validation() {
        assert!(validate_project_settings(&ProjectSettings::default()).is_ok());
        assert!(
            validate_project_settings(&ProjectSettings {
                fps: 0,
                ..ProjectSettings::default()
            })
            .is_err()
        );
        assert!(validate_duration(1).is_ok());
        assert!(validate_duration(0).is_err());
        assert!(validate_transform(&Transform::default()).is_ok());
        assert!(
            validate_transform(&Transform {
                scale: f64::NAN,
                ..Transform::default()
            })
            .is_err()
        );
        assert!(validate_text("text", 48, "#ffffff").is_ok());
        assert!(validate_text("", 48, "#ffffff").is_err());
        assert!(validate_text_style(&TextStyle::default()).is_ok());
        assert!(validate_dimensions(1_920, 1_080).is_ok());
        assert!(validate_dimensions(0, 1_080).is_err());
        assert!(validate_color("#a0B1c2").is_ok());
        assert!(validate_color("white").is_err());
        assert!(validate_audio(&AudioSettings::default()).is_ok());
        assert!(
            validate_audio(&AudioSettings {
                volume: 4.1,
                ..AudioSettings::default()
            })
            .is_err()
        );
    }

    #[test]
    fn covers_keyframe_audio_and_track_validation() {
        let valid = Keyframe {
            property: KeyframeProperty::Opacity,
            time_ms: 0,
            value: KeyframeValue::Scalar { value: 0.5 },
            easing: Easing::Linear,
        };
        assert!(validate_keyframes(std::slice::from_ref(&valid)).is_ok());
        assert!(validate_keyframes(&[valid.clone(), valid]).is_err());
        assert!(validate_track_media(TrackType::Video, MediaType::Image).is_ok());
        assert!(validate_track_media(TrackType::Caption, MediaType::Video).is_err());
        assert!(validate_visual_track(TrackType::Overlay).is_ok());
        assert!(validate_visual_track(TrackType::Audio).is_err());
        assert!(
            validate_track_audio_settings(
                TrackType::Audio,
                AudioTrackRole::Music,
                Some(&DuckingSettings {
                    enabled: true,
                    gain: 0.25,
                    attack_ms: 10,
                    release_ms: 10
                }),
            )
            .is_ok()
        );
        assert!(
            validate_track_audio_settings(TrackType::Video, AudioTrackRole::Music, None).is_err()
        );

        let text = TimelineItem::Text(TextItem {
            id: "text".into(),
            text: "text".into(),
            start_ms: 0,
            duration_ms: 1,
            font_size: 48,
            color: "#ffffff".into(),
            font_family: None,
            font_path: None,
            style: TextStyle::default(),
            visual_properties: crate::VisualProperties::default(),
            keyframes: vec![],
        });
        assert!(validate_item_track(&text, TrackType::Overlay).is_ok());
        assert!(validate_item_track(&text, TrackType::Video).is_err());
    }

    #[test]
    fn covers_draft_label_bounds() {
        assert!(validate_draft_label(Some("draft")).is_ok());
        assert!(validate_draft_label(Some(" ")).is_err());
        assert!(validate_draft_label(Some(&"x".repeat(201))).is_err());
    }
}
