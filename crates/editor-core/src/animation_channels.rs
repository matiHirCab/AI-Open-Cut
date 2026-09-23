use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    CoreError, ErrorCode, Keyframe, KeyframeProperty, MediaType, ParentReference, Project,
    TimelineItem,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AnimationChannelProperty {
    #[serde(rename = "transform.position_x")]
    PositionX,
    #[serde(rename = "transform.position_y")]
    PositionY,
    #[serde(rename = "transform.scale_x")]
    ScaleX,
    #[serde(rename = "transform.scale_y")]
    ScaleY,
    #[serde(rename = "transform.rotation_deg")]
    RotationDeg,
    #[serde(rename = "transform.skew_x_deg")]
    SkewXDeg,
    #[serde(rename = "transform.skew_y_deg")]
    SkewYDeg,
    #[serde(rename = "transform.anchor_x")]
    AnchorX,
    #[serde(rename = "transform.anchor_y")]
    AnchorY,
    #[serde(rename = "transform.opacity")]
    Opacity,
    #[serde(rename = "media.crop_x")]
    CropX,
    #[serde(rename = "media.crop_y")]
    CropY,
    #[serde(rename = "media.crop_width")]
    CropWidth,
    #[serde(rename = "media.crop_height")]
    CropHeight,
    #[serde(rename = "media.source_position_ms")]
    SourcePositionMs,
    #[serde(rename = "media.playback_rate")]
    PlaybackRate,
    #[serde(rename = "graphic.path_points")]
    PathPoints,
    #[serde(rename = "graphic.path_trim")]
    PathTrim,
    #[serde(rename = "graphic.fill_color")]
    FillColor,
    #[serde(rename = "graphic.stroke_color")]
    StrokeColor,
    #[serde(rename = "graphic.stroke_width")]
    StrokeWidth,
    #[serde(rename = "graphic.gradient_stops")]
    GradientStops,
    #[serde(rename = "effect.blur_radius")]
    BlurRadius,
    #[serde(rename = "effect.glow_radius")]
    GlowRadius,
    #[serde(rename = "effect.tint_color")]
    TintColor,
    #[serde(rename = "effect.vignette_amount")]
    VignetteAmount,
    #[serde(rename = "effect.particle_amount")]
    ParticleAmount,
    #[serde(rename = "audio.gain_db")]
    GainDb,
    #[serde(rename = "audio.pan")]
    Pan,
}

impl AnimationChannelProperty {
    pub(crate) fn active(self) -> bool {
        matches!(
            self,
            Self::PositionX
                | Self::PositionY
                | Self::ScaleX
                | Self::ScaleY
                | Self::Opacity
                | Self::GainDb
        )
    }

    pub(crate) fn visual(self) -> bool {
        self.active() && self != Self::GainDb
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationCurve {
    Hold,
    Linear,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnimationChannelValue {
    Scalar { value: f64 },
    Point { x: f64, y: f64 },
    Rgba { r: f64, g: f64, b: f64, a: f64 },
    PathPoints { points: Vec<AnimationPoint> },
    GradientStops { stops: Vec<AnimationGradientStop> },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationGradientStop {
    pub offset: f64,
    pub color: [f64; 4],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannelKeyframe {
    pub time_ms: u64,
    pub value: AnimationChannelValue,
    pub curve: AnimationCurve,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationChannel {
    pub property: AnimationChannelProperty,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<ParentReference>,
    pub keyframes: Vec<AnimationChannelKeyframe>,
}

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
