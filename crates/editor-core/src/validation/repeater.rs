use crate::{CoreError, ErrorCode, MAX_REPEATER_COPIES, RepeaterDescriptor};
use crate::{Project, SlotValue, TimelineItem, Track};
use std::collections::{BTreeMap, HashMap, HashSet};

struct EffectiveAudio<'a> {
    project: &'a Project,
    definitions: HashMap<&'a str, usize>,
    // Default-resolved values distinguish effective uses of a shared definition.
    cache: Vec<(usize, BTreeMap<String, SlotValue>, bool)>,
    active: HashSet<usize>,
}

pub(super) fn validate_effective_audio(project: &Project) -> Result<(), CoreError> {
    if !std::iter::once(&project.tracks)
        .chain(project.components.iter().map(|c| &c.tracks))
        .flat_map(|tracks| tracks.iter())
        .flat_map(|track| &track.items)
        .any(|item| matches!(item, TimelineItem::Repeater(_)))
    {
        return Ok(());
    }
    let mut context = EffectiveAudio {
        project,
        definitions: project
            .components
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.as_str(), i))
            .collect(),
        cache: Vec::new(),
        active: HashSet::new(),
    };
    for index in 0..project.components.len() {
        context.component(index, None)?;
    }
    context.scope(&project.tracks)?;
    Ok(())
}

impl EffectiveAudio<'_> {
    fn component(
        &mut self,
        index: usize,
        overrides: Option<&BTreeMap<String, SlotValue>>,
    ) -> Result<bool, CoreError> {
        if self.active.contains(&index) {
            return Err(super::slot_invalid("component dependency cycle"));
        }
        let component = &self.project.components[index];
        let values = component
            .slots
            .iter()
            .filter_map(|slot| {
                overrides
                    .and_then(|v| v.get(&slot.id))
                    .or(slot.default_value.as_ref())
                    .map(|value| (slot.id.clone(), value.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        // Validate even cache hits, preserving required and unknown-slot errors.
        let effective = super::apply_component_slots(self.project, component, overrides)?;
        if let Some((_, _, audio)) = self
            .cache
            .iter()
            .find(|(id, v, _)| *id == index && *v == values)
        {
            return Ok(*audio);
        }
        self.active.insert(index);
        let audio = self.scope(&effective.tracks)?;
        self.active.remove(&index);
        self.cache.push((index, values, audio));
        Ok(audio)
    }

    fn scope(&mut self, tracks: &[Track]) -> Result<bool, CoreError> {
        let mut audio = false;
        for item in tracks.iter().flat_map(|track| &track.items) {
            // Do not short-circuit: every nested local repeater must be checked.
            if matches!(
                item,
                TimelineItem::Media(_) | TimelineItem::ComponentInstance(_)
            ) {
                audio |= self.source(tracks, item, &mut HashSet::new())?;
            }
        }
        for item in tracks.iter().flat_map(|track| &track.items) {
            if let TimelineItem::Repeater(repeater) = item {
                let source = tracks
                    .iter()
                    .flat_map(|track| &track.items)
                    .find(|item| item.id() == repeater.repeater.source.id)
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "repeater source was not found")
                    })?;
                if self.source(tracks, source, &mut HashSet::new())? {
                    return Err(super::slot_invalid(
                        "repeater source closure must be visual-only",
                    ));
                }
            }
        }
        Ok(audio)
    }

    fn source(
        &mut self,
        tracks: &[Track],
        item: &TimelineItem,
        groups: &mut HashSet<String>,
    ) -> Result<bool, CoreError> {
        match item {
            TimelineItem::Media(media) => Ok(self
                .project
                .assets
                .iter()
                .any(|asset| asset.id == media.asset_id && asset.has_audio)),
            TimelineItem::ComponentInstance(instance) => {
                let index = *self
                    .definitions
                    .get(instance.component_id.as_str())
                    .ok_or_else(|| {
                        CoreError::new(ErrorCode::ItemNotFound, "component definition not found")
                    })?;
                self.component(index, Some(&instance.slot_values))
            }
            TimelineItem::Group(group) => {
                if !groups.insert(group.id.clone()) {
                    return Err(super::slot_invalid("parent cycle"));
                }
                let mut audio = false;
                for child in tracks.iter().flat_map(|track| &track.items).filter(|item| {
                    item.visual_properties()
                        .parent
                        .as_ref()
                        .is_some_and(|p| p.id == group.id)
                }) {
                    audio |= self.source(tracks, child, groups)?;
                }
                groups.remove(&group.id);
                Ok(audio)
            }
            _ => Ok(false),
        }
    }
}

pub(crate) fn validate_descriptor(value: &RepeaterDescriptor) -> Result<(), CoreError> {
    if value.copies == 0
        || value.copies > MAX_REPEATER_COPIES
        || !value.opacity_offset.is_finite()
        || !(-1.0..=1.0).contains(&value.opacity_offset)
        || value.source.scope.is_empty()
        || value.source.scope.len() > 256
        || value.source.id.is_empty()
        || value.source.id.len() > 128
        || !value
            .source
            .id
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || matches!(v, b'_' | b'-'))
    {
        return Err(CoreError::new(
            ErrorCode::InvalidArgument,
            "repeater descriptor is invalid",
        ));
    }
    value.transform_offset.validate()
}
