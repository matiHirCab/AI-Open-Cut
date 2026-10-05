use crate::{
    AnimationChannel, AnimationChannelProperty as P, AnimationChannelValue as V,
    AnimationTargetKind as K, CoreError, ErrorCode, MediaCrop, MediaType, Paint, PathCommand,
    Project, ShapeGeometry, TimelineItem, VisualEffect,
};
use std::collections::BTreeSet;

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}
fn bounded(v: f64, min: f64, max: f64) -> bool {
    v.is_finite() && (min..=max).contains(&v)
}

pub(crate) fn validate_crop(crop: MediaCrop) -> Result<(), CoreError> {
    if !bounded(crop.x, 0.0, 1.0)
        || !bounded(crop.y, 0.0, 1.0)
        || !bounded(crop.width, 0.0, 1.0)
        || crop.width == 0.0
        || !bounded(crop.height, 0.0, 1.0)
        || crop.height == 0.0
        || crop.x + crop.width > 1.0
        || crop.y + crop.height > 1.0
    {
        return Err(invalid(
            "crop must be a positive normalized source rectangle",
        ));
    }
    Ok(())
}

pub(crate) fn validate_effect(effect: &VisualEffect) -> Result<(), CoreError> {
    if effect.id().is_empty() || effect.id().len() > 128 {
        return Err(invalid("effect ID exceeds bounds"));
    }
    match effect {
        VisualEffect::GaussianBlur { radius_px, .. } => {
            if !bounded(*radius_px, 0.0, 128.0) {
                return Err(invalid("blur radius exceeds bounds"));
            }
        }
        VisualEffect::Glow {
            radius_px,
            intensity,
            color,
            ..
        } => {
            if !bounded(*radius_px, 0.0, 128.0) || !bounded(*intensity, 0.0, 1.0) {
                return Err(invalid("glow exceeds bounds"));
            }
            color.validate()?;
        }
        VisualEffect::ColorTint { color, .. } => color.validate()?,
        VisualEffect::Vignette { amount, .. } => {
            if !bounded(*amount, 0.0, 1.0) {
                return Err(invalid("vignette exceeds bounds"));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_static(item: &TimelineItem, project: &Project) -> Result<(), CoreError> {
    let visual = item.visual_properties();
    if let Some(settings) = visual.motion_blur {
        settings.validate()?;
        let eligible = matches!(
            item,
            TimelineItem::Text(_)
                | TimelineItem::SolidColor(_)
                | TimelineItem::Rectangle(_)
                | TimelineItem::Shape(_)
                | TimelineItem::Svg(_)
                | TimelineItem::Grid(_)
        ) || matches!(item, TimelineItem::Media(media) if project.assets.iter().any(|a| a.id == media.asset_id && a.media_type != MediaType::Audio));
        if !eligible {
            return Err(invalid("motion blur requires a supported visual leaf"));
        }
    }
    if let Some(crop) = visual.crop {
        let TimelineItem::Media(media) = item else {
            return Err(invalid("crop requires visual media"));
        };
        if project
            .assets
            .iter()
            .find(|a| a.id == media.asset_id)
            .is_none_or(|a| a.media_type == MediaType::Audio)
        {
            return Err(invalid("crop requires visual media"));
        }
        validate_crop(crop)?;
    }
    if !visual.effects.is_empty()
        && !matches!(
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
        return Err(invalid("effect stack requires a leaf visual item"));
    }
    if !visual.effects.is_empty()
        && let TimelineItem::Media(media) = item
        && project
            .assets
            .iter()
            .find(|a| a.id == media.asset_id)
            .is_none_or(|a| a.media_type == MediaType::Audio)
    {
        return Err(invalid("effects require visual media"));
    }
    if visual.effects.len() > 16 {
        return Err(invalid("maxEffectsPerItem exceeded"));
    }
    let mut ids = BTreeSet::new();
    for effect in &visual.effects {
        validate_effect(effect)?;
        if !ids.insert(effect.id()) {
            return Err(invalid("duplicate effect ID"));
        }
    }
    Ok(())
}

pub(crate) fn point_count(path: &crate::VectorPath) -> usize {
    path.commands
        .iter()
        .map(|c| match c {
            PathCommand::MoveTo { .. } | PathCommand::LineTo { .. } => 1,
            PathCommand::QuadraticTo { .. } => 2,
            PathCommand::CubicTo { .. } => 3,
            PathCommand::Close {} => 0,
        })
        .sum()
}

fn item_scope<'a>(item: &TimelineItem, project: &'a Project) -> Option<&'a str> {
    if project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .any(|i| std::ptr::eq(i, item))
    {
        return Some("root");
    }
    project
        .components
        .iter()
        .find(|c| {
            c.tracks
                .iter()
                .flat_map(|t| &t.items)
                .any(|i| std::ptr::eq(i, item))
        })
        .map(|c| c.id.as_str())
}

pub(crate) fn validate_target(
    channel: &AnimationChannel,
    item: &TimelineItem,
    project: &Project,
) -> Result<(), CoreError> {
    use P::*;
    if matches!(
        channel.property,
        RotationDeg | CropX | CropY | CropWidth | CropHeight
    ) {
        if channel.target.is_some() {
            return Err(invalid("transform and crop channels are targetless"));
        }
        if channel.property != RotationDeg && !matches!(item, TimelineItem::Media(_)) {
            return Err(invalid("crop channel requires visual media"));
        }
        return Ok(());
    }
    let target = channel
        .target
        .as_ref()
        .ok_or_else(|| invalid("extended channel requires scoped target"))?;
    let scope = item_scope(item, project).ok_or_else(|| invalid("animation item scope missing"))?;
    let scoped = if scope == "root" {
        "root".to_owned()
    } else {
        format!("component:{scope}")
    };
    if target.scope != scoped {
        return Err(invalid("animation target scope mismatch"));
    }
    if channel.property.mask() {
        let mask = mask_target(channel, item)?;
        let crate::MaskSource::Path { paint, .. } = &mask.source;
        if channel.property == P::MaskPaintColor && !matches!(paint, Paint::Solid { .. }) {
            return Err(invalid("mask color target requires solid paint"));
        }
        if channel.property == P::MaskGradientStops
            && !matches!(
                paint,
                Paint::LinearGradient { .. } | Paint::RadialGradient { .. }
            )
        {
            return Err(invalid("mask gradient target requires gradient paint"));
        }
        return Ok(());
    }
    match channel.property {
        PathPoints | PathTrim | GradientStops => {
            let TimelineItem::Shape(shape) = item else {
                return Err(invalid("graphic channel requires a shape"));
            };
            if target.id != shape.id {
                return Err(CoreError::new(
                    ErrorCode::ItemNotFound,
                    "graphic target missing",
                ));
            }
            if matches!(channel.property, PathPoints | PathTrim) {
                if target.kind != K::GraphicGeometry
                    || !matches!(shape.geometry, ShapeGeometry::Path { .. })
                {
                    return Err(invalid("path channel requires structured path geometry"));
                }
            } else {
                gradient(channel, item)?;
            }
        }
        BlurRadius | GlowRadius | TintColor | VignetteAmount => {
            if target.kind != K::Effect {
                return Err(invalid("effect channel requires effect target"));
            }
            let effect = item
                .visual_properties()
                .effects
                .iter()
                .find(|e| e.id() == target.id)
                .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "effect target missing"))?;
            if !matches!(
                (channel.property, effect),
                (BlurRadius, VisualEffect::GaussianBlur { .. })
                    | (GlowRadius, VisualEffect::Glow { .. })
                    | (TintColor, VisualEffect::ColorTint { .. })
                    | (VignetteAmount, VisualEffect::Vignette { .. })
            ) {
                return Err(invalid("effect channel type mismatch"));
            }
        }
        _ => return Err(invalid("unsupported extended property")),
    }
    Ok(())
}

pub(crate) fn mask_target<'a>(
    channel: &AnimationChannel,
    item: &'a TimelineItem,
) -> Result<&'a crate::Mask, CoreError> {
    if !matches!(
        item,
        TimelineItem::Media(_)
            | TimelineItem::Text(_)
            | TimelineItem::SolidColor(_)
            | TimelineItem::Rectangle(_)
            | TimelineItem::Shape(_)
            | TimelineItem::Svg(_)
            | TimelineItem::Grid(_)
    ) {
        return Err(invalid("mask animation requires an eligible visual leaf"));
    }
    let target = channel
        .target
        .as_ref()
        .filter(|t| t.kind == K::Mask)
        .ok_or_else(|| invalid("mask property requires mask target"))?;
    if target.id.is_empty() || target.id.len() > 128 {
        return Err(invalid("mask target ID exceeds UTF8 byte bounds"));
    }
    item.visual_properties()
        .masks
        .iter()
        .find(|m| m.id == target.id)
        .ok_or_else(|| CoreError::new(ErrorCode::ItemNotFound, "mask target missing"))
}

fn validate_mask_value(
    channel: &AnimationChannel,
    item: &TimelineItem,
    value: &V,
) -> Result<(), CoreError> {
    let mask = mask_target(channel, item)?;
    let crate::MaskSource::Path { path, paint } = &mask.source;
    let valid = match (channel.property, value) {
        (P::MaskPathPoints, V::PathPoints { points }) => {
            points.len() == point_count(path)
                && points.len() <= 4096
                && points.iter().all(|p| {
                    bounded(p.x, -1000000.0, 1000000.0) && bounded(p.y, -1000000.0, 1000000.0)
                })
        }
        (P::MaskPaintColor, V::Rgba { r, g, b, a }) => {
            matches!(paint, Paint::Solid { .. })
                && [r, g, b, a].iter().all(|v| bounded(**v, 0.0, 1.0))
        }
        (P::MaskGradientStops, V::GradientStops { stops }) => {
            let count = match paint {
                Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. } => {
                    stops.len()
                }
                _ => 0,
            };
            stops.len() == count
                && (2..=32).contains(&count)
                && stops.first().is_some_and(|s| s.offset == 0.0)
                && stops.last().is_some_and(|s| s.offset == 1.0)
                && stops.windows(2).all(|s| s[0].offset < s[1].offset)
                && stops.iter().all(|s| {
                    bounded(s.offset, 0.0, 1.0) && s.color.iter().all(|v| bounded(*v, 0.0, 1.0))
                })
        }
        (p, V::Scalar { value }) => mask_scalar_bounds(p, mask.transform.position.unit)
            .is_some_and(|(low, high)| {
                if matches!(p, P::MaskTransformScaleX | P::MaskTransformScaleY) {
                    value.is_finite() && *value > 0.0 && *value <= high
                } else {
                    bounded(*value, low, high)
                }
            }),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("mask channel value or topology exceeds bounds"))
    }
}

pub(crate) fn mask_scalar_bounds(property: P, unit: crate::PositionUnit) -> Option<(f64, f64)> {
    Some(match property {
        P::MaskFeatherPx => (0.0, 128.0),
        P::MaskExpansionPx => (-128.0, 128.0),
        P::MaskTransformPositionX | P::MaskTransformPositionY => {
            if unit == crate::PositionUnit::Normalized {
                (-100.0, 100.0)
            } else {
                (-1000000.0, 1000000.0)
            }
        }
        P::MaskTransformScaleX | P::MaskTransformScaleY => (0.000001, 100.0),
        P::MaskTransformAnchorX | P::MaskTransformAnchorY | P::MaskTransformOpacity => (0.0, 1.0),
        P::MaskTransformRotationDeg => (-36000.0, 36000.0),
        P::MaskTransformSkewXDeg | P::MaskTransformSkewYDeg => (-80.0, 80.0),
        _ => return None,
    })
}

fn gradient<'a>(
    channel: &AnimationChannel,
    item: &'a TimelineItem,
) -> Result<&'a [crate::GradientStop], CoreError> {
    let TimelineItem::Shape(shape) = item else {
        return Err(invalid("gradient requires shape"));
    };
    let paint = match channel.target.as_ref().map(|t| t.kind) {
        Some(K::GraphicFill) => shape.fill.as_ref(),
        Some(K::GraphicStroke) => shape.stroke.as_ref().map(|s| &s.paint),
        _ => None,
    };
    match paint {
        Some(Paint::LinearGradient { stops, .. } | Paint::RadialGradient { stops, .. }) => {
            Ok(stops)
        }
        _ => Err(invalid("gradient target requires gradient paint")),
    }
}

pub(crate) fn validate_value(
    channel: &AnimationChannel,
    item: &TimelineItem,
    value: &V,
) -> Result<(), CoreError> {
    if channel.property.mask() {
        return validate_mask_value(channel, item, value);
    }
    let within = match (channel.property, value) {
        (P::RotationDeg, V::Scalar { value }) => bounded(*value, -36000.0, 36000.0),
        (P::CropX | P::CropY | P::PathTrim | P::VignetteAmount, V::Scalar { value }) => {
            bounded(*value, 0.0, 1.0)
        }
        (P::CropWidth | P::CropHeight, V::Scalar { value }) => {
            bounded(*value, 0.0, 1.0) && *value > 0.0
        }
        (P::BlurRadius | P::GlowRadius, V::Scalar { value }) => bounded(*value, 0.0, 128.0),
        (P::TintColor, V::Rgba { r, g, b, a }) => {
            [r, g, b, a].iter().all(|v| bounded(**v, 0.0, 1.0))
        }
        (P::PathPoints, V::PathPoints { points }) => {
            let TimelineItem::Shape(shape) = item else {
                return Err(invalid("path requires shape"));
            };
            let ShapeGeometry::Path { path } = &shape.geometry else {
                return Err(invalid("path requires path geometry"));
            };
            points.len() == point_count(path)
                && points.len() <= 4096
                && points.iter().all(|p| {
                    bounded(p.x, -1000000.0, 1000000.0) && bounded(p.y, -1000000.0, 1000000.0)
                })
        }
        (P::GradientStops, V::GradientStops { stops }) => {
            let authored = gradient(channel, item)?;
            stops.len() == authored.len()
                && (2..=32).contains(&stops.len())
                && stops.first().is_some_and(|s| s.offset == 0.0)
                && stops.last().is_some_and(|s| s.offset == 1.0)
                && stops.windows(2).all(|s| s[0].offset < s[1].offset)
                && stops.iter().all(|s| {
                    bounded(s.offset, 0.0, 1.0) && s.color.iter().all(|v| bounded(*v, 0.0, 1.0))
                })
        }
        _ => false,
    };
    if !within {
        return Err(invalid("extended channel value or topology exceeds bounds"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn canonical_crop_and_effect_cases_match_core_validation() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../../contracts/extended-visual-animation-v1.json"
        ))
        .unwrap();
        for case in fixture["cropCases"].as_array().unwrap() {
            let valid = serde_json::from_value::<MediaCrop>(case["value"].clone())
                .is_ok_and(|c| validate_crop(c).is_ok());
            assert_eq!(valid, case["accepted"].as_bool().unwrap(), "{case}");
        }
        for case in fixture["effectCases"].as_array().unwrap() {
            let valid = serde_json::from_value::<VisualEffect>(case["value"].clone())
                .is_ok_and(|e| validate_effect(&e).is_ok());
            assert_eq!(valid, case["accepted"].as_bool().unwrap(), "{case}");
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                validate_crop(MediaCrop {
                    x: value,
                    ..Default::default()
                })
                .is_err()
            );
            assert!(
                validate_effect(&VisualEffect::GaussianBlur {
                    id: "b".into(),
                    radius_px: value
                })
                .is_err()
            );
        }
        for id in [String::new(), "é".repeat(65)] {
            let effect = json!({"id":id,"type":"vignette","amount":0});
            assert!(validate_effect(&serde_json::from_value(effect).unwrap()).is_err());
        }
    }
}
