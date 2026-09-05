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
