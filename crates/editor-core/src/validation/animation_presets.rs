use crate::{AnimationPresetParameters, CoreError, ErrorCode, MAX_PRESET_TIME_MS, TimelineItem};

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::InvalidArgument, message)
}

pub(crate) fn validate_parameters(
    parameters: &AnimationPresetParameters,
) -> Result<u64, CoreError> {
    if !(parameters.property.legacy_visual()
        || parameters.property == crate::AnimationChannelProperty::GainDb)
    {
        return Err(invalid("property is not supported by scalar_tween"));
    }
    let end = parameters
        .start_ms
        .checked_add(parameters.duration_ms)
        .filter(|end| *end <= MAX_PRESET_TIME_MS)
        .ok_or_else(|| invalid("preset timing exceeds safe integer bounds"))?;
    if parameters.duration_ms == 0 {
        return Err(invalid("preset duration must be positive"));
    }
    super::animation_channels::validate_scalar_value(parameters.property, parameters.from)?;
    super::animation_channels::validate_scalar_value(parameters.property, parameters.to)?;
    super::animation_channels::validate_curve(parameters.curve, false)?;
    Ok(end)
}

// Validate descriptive data only: there is deliberately no catalog dispatch or expansion here.
pub(crate) fn validate_provenance(item: &TimelineItem) -> Result<(), CoreError> {
    let visual = item.visual_properties();
    if visual.animation_preset_provenance.len() > 64 {
        return Err(invalid("maxPresetProvenancePerItem exceeded"));
    }
    for (property, source) in &visual.animation_preset_provenance {
        let id = source.preset_id.as_bytes();
        if id.is_empty()
            || id.len() > 64
            || !id[0].is_ascii_lowercase()
            || !id
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
            || source.preset_version == 0
            || source.compiler_version == 0
            || *property != source.parameters.property
            || !visual
                .animation_channels
                .iter()
                .any(|c| c.property == *property && c.target.is_none())
        {
            return Err(invalid("invalid or orphaned animation preset provenance"));
        }
        validate_parameters(&source.parameters)?;
    }
    Ok(())
}
