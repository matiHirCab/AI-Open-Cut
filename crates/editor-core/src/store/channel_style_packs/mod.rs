//! Shared channel data. No template execution, provider orchestration or propagation.
use super::*;
use crate::{
    CHANNEL_PACK_FORMAT_VERSION, ChannelPack, ChannelPackMetadata, ChannelPackPackage,
    MAX_CHANNEL_LIBRARY_BYTES, MAX_CHANNEL_PACKS, MAX_CHANNEL_VERSION,
};

mod apply;
mod capture;
mod portable;
pub use apply::{ChannelAppliedCaptionRole, ChannelAppliedComponent, ChannelPackApplyResult};
#[cfg(test)]
mod tests;

const LIBRARY_FILE: &str = "channel-style-packs.json";

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Library {
    version: u32,
    packs: Vec<ChannelPack>,
}

impl<'de> Deserialize<'de> for Library {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::DeserializeSeed;
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Fields {
            version: u32,
            packs: Vec<ChannelPack>,
        }
        let value = portable::UniqueValue(0, crate::MAX_CHANNEL_JSON_DEPTH + 3)
            .deserialize(deserializer)?;
        let fields: Fields = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
        Ok(Self {
            version: fields.version,
            packs: fields.packs,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelPackList {
    pub packs: Vec<ChannelPackMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChannelCaptureResult {
    pub pack: ChannelPack,
    pub warnings: Vec<String>,
}

fn invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}

fn valid_name(name: &str) -> Result<(), CoreError> {
    if name != name.trim()
        || name.is_empty()
        || name.chars().count() > 80
        || name.chars().any(char::is_control)
    {
        return Err(invalid(
            "pack name must be trimmed, nonempty and at most 80 non-control scalars",
        ));
    }
    Ok(())
}

fn valid_identity(id: &str, version: u64) -> Result<(), CoreError> {
    let id = Uuid::parse_str(id).map_err(|_| invalid("invalid channel pack identity"))?;
    if id.get_version_num() != 4 || version == 0 || version > MAX_CHANNEL_VERSION {
        return Err(invalid("invalid channel pack identity or version"));
    }
    Ok(())
}

fn check_version(pack: &ChannelPack, expected: u64) -> Result<(), CoreError> {
    if pack.version != expected {
        return Err(CoreError::new(
            ErrorCode::RevisionConflict,
            "channel pack changed; read its version and explicitly reapply",
        ));
    }
    Ok(())
}

impl EditorCore {
    fn read_channel_library(&self) -> Result<Library, CoreError> {
        let library: Library = crate::persistence::read_optional_bounded_json(
            self.storage.as_ref(),
            &self.paths.projects_root().join(LIBRARY_FILE),
            MAX_CHANNEL_LIBRARY_BYTES as u64,
        )?
        .unwrap_or(Library {
            version: CHANNEL_PACK_FORMAT_VERSION,
            packs: vec![],
        });
        if library.version != CHANNEL_PACK_FORMAT_VERSION || library.packs.len() > MAX_CHANNEL_PACKS
        {
            return Err(invalid("unsupported or oversized channel library"));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for pack in &library.packs {
            valid_identity(&pack.id, pack.version)?;
            portable::validate_package(&pack.package)?;
            if !ids.insert(&pack.id) || !names.insert(&pack.package.name) {
                return Err(invalid("duplicate channel pack identity or name"));
            }
        }
        Ok(library)
    }

    pub fn list_channel_packs(&self) -> Result<ChannelPackList, CoreError> {
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        let mut packs = self
            .read_channel_library()?
            .packs
            .into_iter()
            .map(|pack| {
                Ok(ChannelPackMetadata {
                    id: pack.id,
                    version: pack.version,
                    name: pack.package.name.clone(),
                    recipe_count: pack.package.recipes.len(),
                    font_count: pack.package.fonts.len(),
                    image_count: pack.package.images.len(),
                    has_caption_role: pack.package.caption_role.is_some(),
                    has_narration_profile: pack.package.narration_profile.is_some(),
                    package_bytes: portable::package_bytes(&pack.package)?.len(),
                })
            })
            .collect::<Result<Vec<_>, CoreError>>()?;
        packs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(ChannelPackList { packs })
    }

    pub fn get_channel_pack(&self, id: &str) -> Result<ChannelPack, CoreError> {
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        self.read_channel_library()?
            .packs
            .into_iter()
            .find(|pack| pack.id == id)
            .ok_or_else(|| invalid("channel pack not found; list available packs"))
    }

    pub fn import_channel_pack(
        &self,
        name: &str,
        package_json: &str,
    ) -> Result<ChannelPack, CoreError> {
        let mut package = portable::decode_package(package_json)?;
        package.name = name.trim().into();
        portable::validate_package(&package)?;
        self.publish_channel_pack(None, package)
    }

    fn publish_channel_pack(
        &self,
        update: Option<(&str, u64)>,
        package: ChannelPackPackage,
    ) -> Result<ChannelPack, CoreError> {
        portable::validate_package(&package)?;
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        let mut library = self.read_channel_library()?;
        let (id, version, index) = match update {
            Some((id, expected)) => {
                let index = library
                    .packs
                    .iter()
                    .position(|pack| pack.id == id)
                    .ok_or_else(|| invalid("channel pack not found; list available packs"))?;
                check_version(&library.packs[index], expected)?;
                if expected >= MAX_CHANNEL_VERSION {
                    return Err(invalid("channel pack version exhausted"));
                }
                (id.to_owned(), expected + 1, Some(index))
            }
            None => {
                if library.packs.len() >= MAX_CHANNEL_PACKS {
                    return Err(invalid("channel pack count limit reached"));
                }
                (Uuid::new_v4().to_string(), 1, None)
            }
        };
        if library
            .packs
            .iter()
            .any(|pack| pack.id != id && pack.package.name == package.name)
        {
            return Err(invalid(
                "channel pack name already exists; choose an explicit unique name",
            ));
        }
        let pack = ChannelPack {
            id,
            version,
            package,
        };
        if let Some(index) = index {
            library.packs[index] = pack.clone();
        } else {
            library.packs.push(pack.clone());
        }
        let bytes = serde_json::to_vec_pretty(&library)
            .map_err(|_| invalid("cannot encode channel library"))?;
        if bytes.len() > MAX_CHANNEL_LIBRARY_BYTES {
            return Err(invalid("channel library byte limit reached"));
        }
        write_json_atomic(
            self.storage.as_ref(),
            &self.paths.projects_root().join(LIBRARY_FILE),
            &library,
        )?;
        Ok(pack)
    }
}
