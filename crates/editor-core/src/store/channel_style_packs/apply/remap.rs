use super::*;

fn local_identity(component_id: &str, kind: &str, source_id: &str) -> Result<String, CoreError> {
    let input = serde_json::to_vec(&(component_id, kind, source_id))
        .map_err(|_| invalid("cannot encode local identity"))?;
    Ok(format!("cp1{kind}_{}", crate::fonts::content_hash(&input)))
}

fn mapped(map: &BTreeMap<String, String>, value: &str, label: &str) -> Result<String, CoreError> {
    map.get(value).cloned().ok_or_else(|| invalid(label))
}

fn slot(value: &mut crate::SlotValue, images: &BTreeMap<String, String>) -> Result<(), CoreError> {
    if let crate::SlotValue::Asset(asset) = value {
        asset.id = mapped(images, &asset.id, "slot artwork dependency is missing")?;
    }
    Ok(())
}

fn id_mut(item: &mut TimelineItem) -> &mut String {
    match item {
        TimelineItem::ComponentInstance(value) => &mut value.id,
        TimelineItem::Group(value) => &mut value.id,
        TimelineItem::Media(value) => &mut value.id,
        TimelineItem::Text(value) => &mut value.id,
        TimelineItem::SolidColor(value) => &mut value.id,
        TimelineItem::Rectangle(value) => &mut value.id,
        TimelineItem::Shape(value) => &mut value.id,
        TimelineItem::Svg(value) => &mut value.id,
        TimelineItem::Grid(value) => &mut value.id,
        TimelineItem::Repeater(value) => &mut value.id,
        TimelineItem::Caption(value) => &mut value.id,
        TimelineItem::Transition(value) => &mut value.id,
    }
}

pub(super) fn definition(
    source: &ComponentDefinition,
    components: &BTreeMap<String, String>,
    images: &BTreeMap<String, String>,
) -> Result<ComponentDefinition, CoreError> {
    let mut definition = source.clone();
    definition.id = mapped(components, &source.id, "component dependency is missing")?;
    let old_scope = format!("component:{}", source.id);
    let scope = format!("component:{}", definition.id);
    let mut local = BTreeMap::new();
    for item in source.tracks.iter().flat_map(|track| &track.items) {
        if local
            .insert(
                item.id().to_owned(),
                local_identity(&definition.id, "i", item.id())?,
            )
            .is_some()
        {
            return Err(invalid("duplicate local recipe item identity"));
        }
    }
    for track in &mut definition.tracks {
        track.id = local_identity(&definition.id, "t", &track.id)?;
        for item in &mut track.items {
            *id_mut(item) = mapped(&local, item.id(), "local item mapping is missing")?;
            let properties = item.visual_properties_mut();
            if let Some(parent) = &mut properties.parent {
                if parent.scope != old_scope {
                    return Err(invalid("recipe parent reference escapes its component"));
                }
                parent.scope.clone_from(&scope);
                parent.id = mapped(&local, &parent.id, "recipe parent is missing")?;
            }
            if let Some(matte) = &mut properties.matte {
                matte.source_id =
                    mapped(&local, &matte.source_id, "recipe matte source is missing")?;
            }
            for channel in &mut properties.animation_channels {
                if let Some(target) = &mut channel.target {
                    if target.scope != old_scope {
                        return Err(invalid("recipe animation target escapes its component"));
                    }
                    target.scope.clone_from(&scope);
                    // Graphic/effect/mask targets are local resource ids, not
                    // timeline ids. Their identity remains within this new scope.
                }
            }
            match item {
                TimelineItem::ComponentInstance(instance) => {
                    instance.component_id = mapped(
                        components,
                        &instance.component_id,
                        "nested component dependency is missing",
                    )?;
                    for value in instance.slot_values.values_mut() {
                        slot(value, images)?;
                    }
                }
                TimelineItem::Media(media) => {
                    media.asset_id = mapped(
                        images,
                        &media.asset_id,
                        "recipe artwork dependency is missing",
                    )?
                }
                TimelineItem::Repeater(repeater) => {
                    if repeater.repeater.source.scope != old_scope {
                        return Err(invalid("recipe repeater reference escapes its component"));
                    }
                    repeater.repeater.source.scope.clone_from(&scope);
                    repeater.repeater.source.id = mapped(
                        &local,
                        &repeater.repeater.source.id,
                        "recipe repeater source is missing",
                    )?;
                }
                TimelineItem::Transition(transition) => {
                    transition.from_item_id = mapped(
                        &local,
                        &transition.from_item_id,
                        "recipe transition source is missing",
                    )?;
                    transition.to_item_id = transition
                        .to_item_id
                        .as_deref()
                        .map(|id| mapped(&local, id, "recipe transition destination is missing"))
                        .transpose()?;
                }
                TimelineItem::Text(_)
                | TimelineItem::SolidColor(_)
                | TimelineItem::Rectangle(_)
                | TimelineItem::Shape(_)
                | TimelineItem::Svg(_)
                | TimelineItem::Grid(_)
                | TimelineItem::Group(_)
                | TimelineItem::Caption(_) => {}
            }
        }
    }
    for template_slot in &mut definition.slots {
        template_slot.binding.target_layer_id = mapped(
            &local,
            &template_slot.binding.target_layer_id,
            "recipe slot target is missing",
        )?;
        if let Some(value) = &mut template_slot.default_value {
            slot(value, images)?;
        }
    }
    for marker in &mut definition.markers {
        if marker.scope != old_scope {
            return Err(invalid("recipe marker escapes its component"));
        }
        marker.id = local_identity(&definition.id, "m", &marker.id)?;
        marker.scope.clone_from(&scope);
    }
    Ok(definition)
}
