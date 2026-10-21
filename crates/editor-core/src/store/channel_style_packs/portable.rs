use super::*;
use crate::{MAX_CHANNEL_JSON_DEPTH, MAX_CHANNEL_PACK_BYTES};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Decode once without silently overwriting repeated native map keys.
pub(super) struct UniqueValue(pub(super) usize, pub(super) usize);

impl<'de> DeserializeSeed<'de> for UniqueValue {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for UniqueValue {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("bounded JSON without repeated object keys")
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite JSON number"))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
        Ok(v.into())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        if self.0 >= self.1 {
            return Err(serde::de::Error::custom("channel JSON nesting limit"));
        }
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(UniqueValue(self.0 + 1, self.1))? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        if self.0 >= self.1 {
            return Err(serde::de::Error::custom("channel JSON nesting limit"));
        }
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom("repeated channel JSON object key"));
            }
            values.insert(key, map.next_value_seed(UniqueValue(self.0 + 1, self.1))?);
        }
        Ok(Value::Object(values))
    }
}

pub(super) fn decode_unique_json(json: &str) -> Result<Value, CoreError> {
    if json.len() > MAX_CHANNEL_PACK_BYTES {
        return Err(invalid("channel package byte limit exceeded"));
    }
    let mut decoder = serde_json::Deserializer::from_str(json);
    let value = UniqueValue(0, MAX_CHANNEL_JSON_DEPTH)
        .deserialize(&mut decoder)
        .map_err(|_| invalid("invalid channel JSON, repeated key or nesting limit"))?;
    decoder
        .end()
        .map_err(|_| invalid("trailing channel JSON data"))?;
    Ok(value)
}

pub(super) fn decode_package(json: &str) -> Result<ChannelPackPackage, CoreError> {
    let package = serde_json::from_value(decode_unique_json(json)?)
        .map_err(|_| invalid("invalid closed channel package fields"))?;
    validate_package(&package)?;
    Ok(package)
}

pub(super) fn package_bytes(package: &ChannelPackPackage) -> Result<Vec<u8>, CoreError> {
    let bytes =
        serde_json::to_vec(package).map_err(|_| invalid("cannot encode channel package"))?;
    if bytes.len() > MAX_CHANNEL_PACK_BYTES {
        return Err(invalid("channel package byte limit exceeded"));
    }
    let encoded =
        std::str::from_utf8(&bytes).map_err(|_| invalid("cannot encode channel UTF-8"))?;
    decode_unique_json(encoded)?;
    Ok(bytes)
}

pub(super) fn validate_package(package: &ChannelPackPackage) -> Result<(), CoreError> {
    valid_name(&package.name)?;
    package_bytes(package)?;
    if package.format_version != CHANNEL_PACK_FORMAT_VERSION
        || package.component_schema_version != crate::CHANNEL_PACK_COMPONENT_SCHEMA_VERSION
    {
        return Err(invalid("unsupported channel format or component schema"));
    }
    validate_native_package(package)
}

fn validate_native_package(package: &ChannelPackPackage) -> Result<(), CoreError> {
    if package.recipes.is_empty() || package.recipes.len() > crate::MAX_CHANNEL_RECIPES {
        return Err(invalid("channel recipe count is outside its bound"));
    }
    if package.fonts.len() > crate::MAX_CHANNEL_FONTS
        || package.images.len() > crate::MAX_CHANNEL_IMAGES
    {
        return Err(invalid("channel dependency count exceeds its bound"));
    }
    if let Some(reference) = &package.narration_profile {
        valid_identity(&reference.profile_id, reference.expected_version)?;
    }
    let (catalog, faces) = verified_fonts(package)?;
    let mut keys = BTreeSet::new();
    let mut item_count = 0usize;
    let mut marker_count = 0usize;
    let mut used_fonts = BTreeSet::new();
    for recipe in &package.recipes {
        let definition = &recipe.definition;
        if recipe.key != definition.id || !keys.insert(recipe.key.as_str()) {
            return Err(invalid(
                "channel recipes require distinct matching native identities",
            ));
        }
        marker_count += definition.markers.len();
        for track in &definition.tracks {
            if track.audio_bus_id.is_some()
                || track.ducking.is_some()
                || track.track_type == TrackType::Audio
            {
                return Err(invalid(
                    "channel recipes do not support global audio resources",
                ));
            }
            item_count += track.items.len();
            for item in &track.items {
                match item {
                    TimelineItem::Text(text) => {
                        if text.font_path.is_some() || text.font_family.is_some() {
                            return Err(invalid(
                                "portable text must use pinned fonts without ambient selectors",
                            ));
                        }
                        let binding = text.font_binding.as_ref().ok_or_else(|| {
                            invalid("portable text requires exact pinned font data")
                        })?;
                        crate::fonts::validate_binding(binding, &catalog)?;
                        if binding
                            .warnings
                            .iter()
                            .any(|warning| warning != CAPTURE_FONT_WARNING)
                        {
                            return Err(invalid("portable font warnings must be sanitized"));
                        }
                        used_fonts.extend(binding.hashes().into_iter().map(str::to_owned));
                    }
                    TimelineItem::Caption(_) => {
                        return Err(invalid(
                            "capture captions as ordinary pinned text-slot components",
                        ));
                    }
                    TimelineItem::Media(media) if media.audio_event.is_some() => {
                        return Err(invalid(
                            "channel recipes do not support audio-event resources",
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
    if item_count > crate::MAX_CHANNEL_ITEMS || marker_count > crate::MAX_CHANNEL_MARKERS {
        return Err(invalid("channel item or marker count exceeds its bound"));
    }
    if used_fonts != catalog.keys().cloned().collect() {
        return Err(invalid("channel fonts must exactly cover referenced faces"));
    }
    let mut images = BTreeSet::new();
    for image in &package.images {
        if !images.insert(image.key.as_str())
            || image.content_hash.algorithm != "sha256"
            || !crate::fonts::valid_hash(&image.content_hash.digest)
            || image.size_bytes == 0
            || image.probe.has_audio
            || !image.probe.has_video
            || image.probe.video_width.is_none_or(|v| v == 0)
            || image.probe.video_height.is_none_or(|v| v == 0)
            || image.probe.audio_codec.is_some()
            || image.probe.audio_channels.is_some()
            || image.probe.audio_sample_rate_hz.is_some()
        {
            return Err(invalid(
                "channel image declaration requires unique managed image identity and probe facts",
            ));
        }
    }
    let project = validation_project(package, catalog);
    let used_images: BTreeSet<_> = crate::assets::project_asset_references(&project)
        .into_iter()
        .map(|reference| reference.asset_id)
        .collect();
    if used_images != images.into_iter().map(str::to_owned).collect() {
        return Err(invalid(
            "channel image declarations must exactly cover referenced managed assets",
        ));
    }
    crate::assets::validate_project_asset_references(&project, "channel package")?;
    validate_project_visual_properties(&project)?;
    crate::validation::validate_recursive_graphs(&project)?;
    crate::evaluated_scene::preflight_inherited_project(&project)?;
    crate::evaluated_scene::preflight_extended_fonts(&project, &faces)?;
    if let Some(role) = &package.caption_role {
        let recipe = package
            .recipes
            .iter()
            .find(|recipe| recipe.key == role.recipe_key)
            .ok_or_else(|| invalid("caption role recipe is missing"))?;
        if !recipe.definition.slots.iter().any(|slot| {
            slot.id == role.text_slot_id
                && matches!(slot.kind, crate::SlotKind::Text | crate::SlotKind::RichText)
        }) {
            return Err(invalid(
                "caption role must identify an existing text or rich-text slot",
            ));
        }
    }
    Ok(())
}

pub(super) const CAPTURE_FONT_WARNING: &str =
    "Capture retained the exact managed face; source font diagnostics were omitted";

pub(super) fn verified_fonts(
    package: &ChannelPackPackage,
) -> Result<
    (
        BTreeMap<String, crate::FontRecord>,
        crate::assets::fonts::FontBytes,
    ),
    CoreError,
> {
    let mut catalog = BTreeMap::new();
    let mut faces = BTreeMap::new();
    for font in &package.fonts {
        let bytes = STANDARD
            .decode(&font.data_base64)
            .map_err(|_| invalid("invalid portable font base64"))?;
        if STANDARD.encode(&bytes) != font.data_base64 {
            return Err(invalid("portable font base64 must be canonical"));
        }
        let record = crate::fonts::record(&bytes)?;
        if record.sha256 != font.sha256
            || record.size_bytes != font.size_bytes
            || record.face_index != font.face_index
            || catalog
                .insert(record.sha256.clone(), record.clone())
                .is_some()
        {
            return Err(invalid(
                "portable font identity, bytes or uniqueness disagree",
            ));
        }
        faces.insert(record.sha256, bytes);
    }
    crate::fonts::validate_catalog(&catalog)?;
    Ok((catalog, faces))
}

fn validation_project(
    package: &ChannelPackPackage,
    fonts: BTreeMap<String, crate::FontRecord>,
) -> Project {
    Project {
        narration_manifests: Vec::new(),
        schema_version: PROJECT_SCHEMA_VERSION,
        id: "channel_validation".into(),
        revision: 0,
        name: package.name.clone(),
        created_at_ms: 0,
        updated_at_ms: 0,
        settings: ProjectSettings::default(),
        fonts,
        tracks: vec![],
        markers: vec![],
        components: package
            .recipes
            .iter()
            .map(|recipe| recipe.definition.clone())
            .collect(),
        audio_buses: crate::default_audio_buses(),
        master_normalization: None,
        sound_definitions: vec![],
        assets: package
            .images
            .iter()
            .map(|image| Asset {
                artwork_derivation: None,
                id: image.key.clone(),
                media_type: MediaType::Image,
                file_name: "managed-image".into(),
                project_relative_path: crate::assets::hash_relative_path(
                    &image.content_hash.digest,
                ),
                duration_ms: image.probe.duration_ms,
                has_audio: false,
                origin: None,
                content_hash: Some(image.content_hash.clone()),
                size_bytes: Some(image.size_bytes),
                probe: Some(image.probe.clone()),
            })
            .collect(),
    }
}
