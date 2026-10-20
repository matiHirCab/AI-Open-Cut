use super::*;
use crate::{
    ChannelApplyInput, ChannelApplyMode, ChannelNarrationChoice, ComponentDefinition,
    NarrationProfile,
};
use std::collections::{BTreeMap, BTreeSet};

mod remap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelAppliedComponent {
    pub recipe_key: String,
    pub component_id: String,
    pub before_digest: Option<String>,
    pub after_digest: String,
    pub changed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelAppliedCaptionRole {
    pub component_id: String,
    pub text_slot_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppliedPackIdentity {
    pub id: String,
    pub version: u64,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPackApplyResult {
    pub write: WriteResult,
    pub pack: AppliedPackIdentity,
    pub components: Vec<ChannelAppliedComponent>,
    pub narration_profile: Option<NarrationProfile>,
    pub caption_role: Option<ChannelAppliedCaptionRole>,
}

impl EditorCore {
    pub fn apply_channel_pack(
        &self,
        input: ChannelApplyInput,
    ) -> Result<ChannelPackApplyResult, CoreError> {
        // Both root-locked lookups finish before entering the project transaction.
        let pack = self.get_channel_pack(&input.pack_id)?;
        check_version(&pack, input.expected_version)?;
        let reference = match &input.narration {
            ChannelNarrationChoice::Saved {} => pack.package.narration_profile.as_ref(),
            ChannelNarrationChoice::Disabled {} => None,
            ChannelNarrationChoice::Replacement { reference } => Some(reference),
        };
        let narration_profile = reference
            .map(|reference| {
                valid_identity(&reference.profile_id, reference.expected_version)?;
                let profile = self.get_narration_profile(&reference.profile_id)?;
                if profile.version != reference.expected_version {
                    return Err(CoreError::new(
                        ErrorCode::RevisionConflict,
                        "narration profile changed; choose an explicit current reference",
                    ));
                }
                Ok(profile)
            })
            .transpose()?;
        let (catalog, faces) = portable::verified_fonts(&pack.package)?;
        let (write, components) = self.edit_with_staged_change(
            &input.project_id,
            input.expected_revision,
            true,
            |prepared, _dir, _rollback| {
                let original = prepared.project.clone();
                let images =
                    self.resolve_channel_images(&input, &pack.package, &prepared.project)?;
                let (mut batch, component_ids, used) =
                    seed_or_map(&mut prepared.project, &pack.package, &input.application)?;
                let mut definitions = Vec::new();
                for recipe in &pack.package.recipes {
                    definitions.push(remap::definition(
                        &recipe.definition,
                        &component_ids,
                        &images,
                    )?);
                }
                let definitions = dependency_first(definitions)?;
                let mut operations = Vec::new();
                for definition in &definitions {
                    if matches!(input.application, ChannelApplyMode::Reapply { .. }) {
                        let current = original
                            .components
                            .iter()
                            .find(|old| old.id == definition.id)
                            .ok_or_else(|| {
                                CoreError::new(
                                    ErrorCode::ItemNotFound,
                                    "mapped component is missing",
                                )
                            })?;
                        for marker in &current.markers {
                            operations.push(
                                EditOperation::MarkerDelete {
                                    scope: format!("component:{}", definition.id),
                                    marker_id: marker.id.clone(),
                                }
                                .into(),
                            );
                        }
                    }
                    let mut tracks = definition.tracks.clone();
                    for text in tracks
                        .iter_mut()
                        .flat_map(|track| &mut track.items)
                        .filter_map(|item| match item {
                            TimelineItem::Text(text) => Some(text),
                            _ => None,
                        })
                    {
                        text.font_binding = None;
                    }
                    operations.push(
                        EditOperation::ComponentUpdate {
                            component_id: definition.id.clone(),
                            name: definition.name.clone(),
                            width: definition.width,
                            height: definition.height,
                            duration_ms: definition.duration_ms,
                            tracks,
                            slots: Some(definition.slots.clone()),
                        }
                        .into(),
                    );
                    for marker in &definition.markers {
                        operations.push(
                            EditOperation::MarkerCreate {
                                scope: format!("component:{}", definition.id),
                                name: marker.name.clone(),
                                time_ms: marker.time_ms,
                                kind: marker.kind.clone(),
                            }
                            .into(),
                        );
                    }
                }
                if let ChannelApplyMode::Instantiate { placements } = &input.application {
                    if placements.is_empty() || placements.len() > crate::MAX_CHANNEL_PLACEMENTS {
                        return Err(invalid("channel placements require 1..32 records"));
                    }
                    for placement in placements {
                        let component_id = component_ids
                            .get(&placement.recipe_key)
                            .ok_or_else(|| invalid("placement recipe is missing"))?
                            .clone();
                        operations.push(
                            EditOperation::AddComponentInstance {
                                track_id: placement.track_id.clone(),
                                component_id,
                                start_ms: placement.start_ms,
                                trim_start_ms: placement.trim_start_ms,
                                duration_ms: placement.duration_ms,
                                time_scale: placement.time_scale,
                                stagger_ms: placement.stagger_ms,
                                slot_values: placement.slot_values.clone(),
                                transform: placement.transform.clone(),
                                transform2d: placement.transform2d,
                                hidden: placement.hidden,
                                z_index: placement.z_index,
                                parent: placement.parent.clone(),
                            }
                            .into(),
                        );
                    }
                }
                if used + operations.len() > EDIT_LIMIT {
                    return Err(invalid(
                        "channel application exceeds the existing native 100-operation batch limit",
                    ));
                }
                let populated = apply_edit_batch(&mut prepared.project, operations)?;
                batch.changed_ids.extend(populated.changed_ids);
                batch.aliases.extend(populated.aliases);
                // Public ComponentUpdate's binding guard is preserved. Only this
                // verified core package path can attach its exact managed faces.
                for definition in &definitions {
                    let current = prepared
                        .project
                        .components
                        .iter_mut()
                        .find(|component| component.id == definition.id)
                        .ok_or_else(|| invalid("materialized component is missing"))?;
                    if current.markers.len() != definition.markers.len() {
                        return Err(invalid(
                            "materialized marker count disagrees with its validated recipe",
                        ));
                    }
                    // MarkerCreate appends in operation order. Native permits equal
                    // name/time/kind tuples, so associate by that exact operation
                    // position rather than treating the visible tuple as identity.
                    for (marker, target) in current.markers.iter_mut().zip(&definition.markers) {
                        if marker.name != target.name
                            || marker.time_ms != target.time_ms
                            || marker.kind != target.kind
                        {
                            return Err(invalid(
                                "materialized marker disagrees with its validated operation",
                            ));
                        }
                        for id in &mut batch.changed_ids {
                            if id == &marker.id {
                                id.clone_from(&target.id);
                            }
                        }
                        marker.id.clone_from(&target.id);
                    }
                    for item in current.tracks.iter_mut().flat_map(|track| &mut track.items) {
                        if let TimelineItem::Text(text) = item {
                            let binding = definition
                                .tracks
                                .iter()
                                .flat_map(|track| &track.items)
                                .find_map(|item| match item {
                                    TimelineItem::Text(source) if source.id == text.id => {
                                        source.font_binding.clone()
                                    }
                                    _ => None,
                                })
                                .ok_or_else(|| {
                                    invalid("materialized text lacks its verified package binding")
                                })?;
                            text.font_binding = Some(binding);
                        }
                    }
                    if let Some(old) = original.components.iter().find(|old| old.id == current.id) {
                        crate::timeline::validate_component_track_locks(
                            &old.tracks,
                            &current.tracks,
                        )?;
                    }
                }
                for (hash, record) in &catalog {
                    if let Some(existing) = prepared.project.fonts.get(hash)
                        && existing != record
                    {
                        return Err(invalid(
                            "target font catalog disagrees with package identity",
                        ));
                    }
                    prepared.changed |= !prepared.project.fonts.contains_key(hash);
                    prepared.project.fonts.insert(hash.clone(), record.clone());
                }
                for (hash, bytes) in &faces {
                    prepared.fonts.insert(hash.clone(), bytes.clone());
                }
                validate_project_visual_properties(&prepared.project)?;
                crate::validation::validate_recursive_graphs(&prepared.project)?;
                crate::assets::validate_project_asset_references(
                    &prepared.project,
                    "channel application",
                )?;
                let mut reports = Vec::new();
                for recipe in &pack.package.recipes {
                    let id = &component_ids[&recipe.key];
                    let before_digest = original
                        .components
                        .iter()
                        .find(|component| &component.id == id)
                        .map(component_digest)
                        .transpose()?;
                    let current = prepared
                        .project
                        .components
                        .iter()
                        .find(|component| &component.id == id)
                        .ok_or_else(|| invalid("materialized component is missing"))?;
                    let after_digest = component_digest(current)?;
                    reports.push(ChannelAppliedComponent {
                        recipe_key: recipe.key.clone(),
                        component_id: id.clone(),
                        changed: before_digest.as_ref() != Some(&after_digest),
                        before_digest,
                        after_digest,
                    });
                }
                Ok((batch, reports))
            },
        )?;
        let caption_role =
            pack.package
                .caption_role
                .as_ref()
                .map(|role| ChannelAppliedCaptionRole {
                    component_id: components
                        .iter()
                        .find(|component| component.recipe_key == role.recipe_key)
                        .expect("validated caption recipe was materialized")
                        .component_id
                        .clone(),
                    text_slot_id: role.text_slot_id.clone(),
                });
        Ok(ChannelPackApplyResult {
            write,
            pack: AppliedPackIdentity {
                id: pack.id,
                version: pack.version,
                name: pack.package.name,
            },
            components,
            narration_profile,
            caption_role,
        })
    }

    fn resolve_channel_images(
        &self,
        input: &ChannelApplyInput,
        package: &ChannelPackPackage,
        project: &Project,
    ) -> Result<BTreeMap<String, String>, CoreError> {
        let mut bindings = BTreeMap::new();
        for binding in &input.images {
            if bindings
                .insert(binding.dependency_key.clone(), binding.asset_id.clone())
                .is_some()
                || !package
                    .images
                    .iter()
                    .any(|image| image.key == binding.dependency_key)
            {
                return Err(CoreError::new(
                    ErrorCode::InvalidArgument,
                    "artwork bindings must be unique and match declared dependency keys",
                ));
            }
        }
        for image in &package.images {
            let id = bindings.get(&image.key).ok_or_else(|| {
                CoreError::new(
                    ErrorCode::DependencyUnavailable,
                    "bind every saved artwork dependency to an explicit target managed asset",
                )
            })?;
            let asset = project
                .assets
                .iter()
                .find(|asset| &asset.id == id)
                .ok_or_else(|| {
                    CoreError::new(
                        ErrorCode::AssetNotFound,
                        "explicit target artwork asset is missing",
                    )
                })?;
            self.verify_channel_image(&input.project_id, asset, image)?;
        }
        Ok(bindings)
    }
}

pub(super) fn component_digest(component: &ComponentDefinition) -> Result<String, CoreError> {
    // Workspace feature unification can enable insertion-preserving maps.
    // Explicitly canonicalize objects while preserving arrays and numeric values.
    fn canonicalize(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Array(values) => {
                for value in values {
                    canonicalize(value);
                }
            }
            serde_json::Value::Object(map) => {
                let mut entries: Vec<_> = std::mem::take(map).into_iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (key, mut value) in entries {
                    canonicalize(&mut value);
                    map.insert(key, value);
                }
            }
            _ => {}
        }
    }
    let mut value =
        serde_json::to_value(component).map_err(|_| invalid("cannot encode component digest"))?;
    canonicalize(&mut value);
    let bytes =
        serde_json::to_vec(&value).map_err(|_| invalid("cannot encode component digest"))?;
    Ok(crate::fonts::content_hash(&bytes))
}

// These dependencies have already passed the native graph validator. Ordering
// ensures each existing per-operation slot check sees populated child schemas.
fn dependency_first(
    mut pending: Vec<ComponentDefinition>,
) -> Result<Vec<ComponentDefinition>, CoreError> {
    let mut ordered = Vec::with_capacity(pending.len());
    let mut populated = BTreeSet::new();
    while !pending.is_empty() {
        let index = pending
            .iter()
            .position(|definition| {
                definition
                    .tracks
                    .iter()
                    .flat_map(|track| &track.items)
                    .all(|item| {
                        if let TimelineItem::ComponentInstance(instance) = item {
                            populated.contains(&instance.component_id)
                        } else {
                            true
                        }
                    })
            })
            .ok_or_else(|| invalid("validated channel component dependencies cannot be ordered"))?;
        let definition = pending.remove(index);
        populated.insert(definition.id.clone());
        ordered.push(definition);
    }
    Ok(ordered)
}

fn seed_or_map(
    project: &mut Project,
    package: &ChannelPackPackage,
    mode: &ChannelApplyMode,
) -> Result<(AppliedBatch, BTreeMap<String, String>, usize), CoreError> {
    match mode {
        ChannelApplyMode::Instantiate { .. } => {
            let operations = package
                .recipes
                .iter()
                .enumerate()
                .map(|(index, recipe)| BatchEditOperation {
                    edit: EditOperation::ComponentCreate {
                        name: recipe.definition.name.clone(),
                        width: recipe.definition.width,
                        height: recipe.definition.height,
                        duration_ms: recipe.definition.duration_ms,
                        tracks: vec![],
                        slots: Some(vec![]),
                    },
                    result_alias: Some(format!("cp{index}")),
                })
                .collect();
            let applied = apply_edit_batch(project, operations)?;
            let ids = package
                .recipes
                .iter()
                .enumerate()
                .map(|(index, recipe)| {
                    (
                        recipe.key.clone(),
                        applied.aliases[&format!("cp{index}")].clone(),
                    )
                })
                .collect();
            Ok((applied, ids, package.recipes.len()))
        }
        ChannelApplyMode::Reapply { components } => {
            let mut ids = BTreeMap::new();
            let mut targets = BTreeSet::new();
            for mapping in components {
                if !package
                    .recipes
                    .iter()
                    .any(|recipe| recipe.key == mapping.recipe_key)
                    || ids
                        .insert(mapping.recipe_key.clone(), mapping.component_id.clone())
                        .is_some()
                    || !targets.insert(mapping.component_id.clone())
                {
                    return Err(invalid(
                        "reapply requires a complete distinct recipe-to-component mapping",
                    ));
                }
                if !project
                    .components
                    .iter()
                    .any(|component| component.id == mapping.component_id)
                {
                    return Err(CoreError::new(
                        ErrorCode::ItemNotFound,
                        "mapped reapply component is missing",
                    ));
                }
            }
            if ids.len() != package.recipes.len() {
                return Err(invalid("reapply requires every recipe mapping"));
            }
            Ok((
                AppliedBatch {
                    changed_ids: vec![],
                    aliases: BTreeMap::new(),
                    summary: "Reapplied channel pack",
                },
                ids,
                0,
            ))
        }
    }
}
