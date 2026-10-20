use super::*;
use crate::{ChannelCaptureInput, ChannelFontFace, ChannelImageDependency, ChannelRecipe};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::collections::{BTreeMap, BTreeSet};

impl EditorCore {
    pub fn capture_channel_pack(
        &self,
        capture: ChannelCaptureInput,
    ) -> Result<ChannelCaptureResult, CoreError> {
        let (package, warnings) = self.capture_channel_package(capture)?;
        let pack = self.publish_channel_pack(None, package)?;
        Ok(ChannelCaptureResult { pack, warnings })
    }

    pub fn update_channel_pack(
        &self,
        id: &str,
        expected_version: u64,
        capture: ChannelCaptureInput,
    ) -> Result<ChannelCaptureResult, CoreError> {
        let (package, warnings) = self.capture_channel_package(capture)?;
        let pack = self.publish_channel_pack(Some((id, expected_version)), package)?;
        Ok(ChannelCaptureResult { pack, warnings })
    }

    /// Deliberately never enters the ordinary loader: that loader may recover or
    /// migrate authoritative state, even when a caller only wants a snapshot.
    fn capture_channel_package(
        &self,
        capture: ChannelCaptureInput,
    ) -> Result<(ChannelPackPackage, Vec<String>), CoreError> {
        let dir = self.existing_project_dir(&capture.project_id)?;
        let _lock = self.storage.lock_exclusive(&dir)?;
        crate::persistence::require_no_pending_transaction(self.storage.as_ref(), &dir)?;
        let source: Project = read_json(self.storage.as_ref(), &project_path(&dir))?;
        check_revision(&source, capture.expected_revision)?;
        if source.schema_version != PROJECT_SCHEMA_VERSION {
            return Err(invalid(
                "capture requires the current project schema; explicitly open/migrate the source first",
            ));
        }
        validate_project_visual_properties(&source)?;
        crate::validation::validate_recursive_graphs(&source)?;
        crate::assets::fonts::verify_project_fonts(self.storage.as_ref(), &dir, &source)?;
        let definitions: BTreeMap<_, _> = source
            .components
            .iter()
            .map(|definition| (definition.id.as_str(), definition))
            .collect();
        let mut selected = BTreeSet::new();
        if capture.component_ids.is_empty()
            || capture.component_ids.len() > crate::MAX_CHANNEL_RECIPES
        {
            return Err(invalid(
                "capture requires a bounded nonempty component selection",
            ));
        }
        for id in &capture.component_ids {
            if !selected.insert(id.clone()) {
                return Err(invalid("capture component selection must be unique"));
            }
        }
        let mut pending: Vec<_> = selected.iter().cloned().collect();
        while let Some(id) = pending.pop() {
            let definition = definitions.get(id.as_str()).ok_or_else(|| {
                CoreError::new(ErrorCode::ItemNotFound, "capture component is missing")
            })?;
            for item in definition.tracks.iter().flat_map(|track| &track.items) {
                if let TimelineItem::ComponentInstance(instance) = item
                    && selected.insert(instance.component_id.clone())
                {
                    if selected.len() > crate::MAX_CHANNEL_RECIPES {
                        return Err(invalid("captured component closure exceeds its bound"));
                    }
                    pending.push(instance.component_id.clone());
                }
            }
        }
        let mut package = ChannelPackPackage {
            format_version: CHANNEL_PACK_FORMAT_VERSION,
            component_schema_version: PROJECT_SCHEMA_VERSION,
            name: capture.name.trim().into(),
            recipes: selected
                .iter()
                .map(|key| ChannelRecipe {
                    key: key.clone(),
                    definition: (*definitions[key.as_str()]).clone(),
                })
                .collect(),
            fonts: vec![],
            images: vec![],
            caption_role: capture.caption_role,
            narration_profile: capture.narration_profile,
        };
        let mut hashes = BTreeSet::new();
        let mut warnings = Vec::new();
        for text in package
            .recipes
            .iter_mut()
            .flat_map(|recipe| &mut recipe.definition.tracks)
            .flat_map(|track| &mut track.items)
            .filter_map(|item| match item {
                TimelineItem::Text(text) => Some(text),
                _ => None,
            })
        {
            let binding = text
                .font_binding
                .as_mut()
                .ok_or_else(|| invalid("source text lacks an exact managed font binding"))?;
            if !binding.warnings.is_empty() {
                binding.warnings = vec![portable::CAPTURE_FONT_WARNING.into()];
                if warnings.is_empty() {
                    warnings.push(portable::CAPTURE_FONT_WARNING.into());
                }
            }
            text.font_path = None;
            text.font_family = None;
            hashes.extend(binding.hashes().into_iter().map(str::to_owned));
        }
        if hashes.len() > crate::MAX_CHANNEL_FONTS {
            return Err(invalid("captured font count exceeds the portable bound"));
        }
        let mut package_size = portable::package_bytes(&package)?.len();
        for hash in hashes {
            let face = source
                .fonts
                .get(&hash)
                .ok_or_else(|| invalid("captured text references a missing managed face"))?;
            let mut font = ChannelFontFace {
                sha256: hash,
                size_bytes: face.size_bytes,
                face_index: face.face_index,
                data_base64: String::new(),
            };
            let encoded_size = usize::try_from(face.size_bytes)
                .ok()
                .and_then(|size| size.checked_add(2))
                .and_then(|size| (size / 3).checked_mul(4))
                .ok_or_else(|| invalid("portable font byte count overflow"))?;
            let metadata_size = serde_json::to_vec(&font)?.len();
            package_size = package_size
                .checked_add(encoded_size)
                .and_then(|size| size.checked_add(metadata_size))
                .and_then(|size| size.checked_add(usize::from(!package.fonts.is_empty())))
                .filter(|size| *size <= crate::MAX_CHANNEL_PACK_BYTES)
                .ok_or_else(|| {
                    invalid("captured exact fonts exceed the portable package byte limit")
                })?;
            let bytes = crate::assets::fonts::managed_bytes(self.storage.as_ref(), &dir, face)?;
            font.data_base64 = STANDARD.encode(bytes);
            package.fonts.push(font);
        }
        let mut subset = source.clone();
        subset.tracks.clear();
        subset.sound_definitions.clear();
        subset.components = package
            .recipes
            .iter()
            .map(|recipe| recipe.definition.clone())
            .collect();
        let images: BTreeSet<_> = crate::assets::project_asset_references(&subset)
            .into_iter()
            .map(|reference| reference.asset_id)
            .collect();
        for id in images {
            let asset = source
                .assets
                .iter()
                .find(|asset| asset.id == id)
                .ok_or_else(|| {
                    CoreError::new(ErrorCode::AssetNotFound, "captured artwork is missing")
                })?;
            let image = image_declaration(asset)?;
            self.verify_channel_image(&capture.project_id, asset, &image)?;
            package.images.push(image);
        }
        portable::validate_package(&package)?;
        Ok((package, warnings))
    }

    pub(super) fn verify_channel_image(
        &self,
        project_id: &str,
        asset: &Asset,
        image: &ChannelImageDependency,
    ) -> Result<(), CoreError> {
        if image.content_hash.algorithm != "sha256"
            || !crate::fonts::valid_hash(&image.content_hash.digest)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "managed artwork has an invalid content identity",
            ));
        }
        if asset.media_type != MediaType::Image
            || asset.has_audio
            || asset.content_hash.as_ref() != Some(&image.content_hash)
            || asset.size_bytes != Some(image.size_bytes)
            || asset.probe.as_ref() != Some(&image.probe)
        {
            return Err(CoreError::new(
                ErrorCode::InvalidArgument,
                "explicit artwork binding does not match the saved managed image",
            ));
        }
        if asset.project_relative_path
            != crate::assets::hash_relative_path(&image.content_hash.digest)
        {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "artwork must retain its project-managed content address",
            ));
        }
        let path = self
            .project_asset_path(project_id, &asset.project_relative_path)
            .map_err(|error| {
                if error.code == ErrorCode::PathNotAllowed {
                    error
                } else {
                    CoreError::new(
                        ErrorCode::AssetIntegrityFailed,
                        "managed artwork cannot be resolved",
                    )
                }
            })?;
        let (hash, size) = crate::assets::hash_file(self.storage.as_ref(), &path)?;
        if hash != image.content_hash.digest || size != image.size_bytes {
            return Err(CoreError::new(
                ErrorCode::AssetIntegrityFailed,
                "managed artwork bytes do not match their pinned identity",
            ));
        }
        Ok(())
    }
}

fn image_declaration(asset: &Asset) -> Result<ChannelImageDependency, CoreError> {
    if asset.media_type != MediaType::Image || asset.has_audio {
        return Err(invalid("channel packs support managed image artwork only"));
    }
    Ok(ChannelImageDependency {
        key: asset.id.clone(),
        content_hash: asset
            .content_hash
            .clone()
            .ok_or_else(|| invalid("source artwork has no managed hash"))?,
        size_bytes: asset
            .size_bytes
            .ok_or_else(|| invalid("source artwork has no managed byte count"))?,
        probe: asset
            .probe
            .clone()
            .ok_or_else(|| invalid("source artwork has no managed probe facts"))?,
    })
}
