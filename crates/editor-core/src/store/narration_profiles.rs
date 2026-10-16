//! Shared local narration library. Domain validation and persistence stay in core.
use super::*;
use crate::{
    MAX_NARRATION_PROFILES, NarrationProfile, NarrationProfileApplication,
    NarrationProfileAttribution, NarrationProfileSettings, SpeechSynthesisRequest,
};
use serde::{Deserialize, Serialize};

const LIBRARY_FILE: &str = "narration-profiles.json";
const LIBRARY_VERSION: u32 = 1;
const MAX_LIBRARY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Library {
    version: u32,
    profiles: Vec<NarrationProfile>,
}

impl EditorCore {
    fn read_narration_library(&self) -> Result<Library, CoreError> {
        let path = self.paths.projects_root().join(LIBRARY_FILE);
        let library: Library = crate::persistence::read_optional_bounded_json(
            self.storage.as_ref(),
            &path,
            MAX_LIBRARY_BYTES,
        )?
        .unwrap_or_else(|| Library {
            version: LIBRARY_VERSION,
            profiles: vec![],
        });
        if library.version != LIBRARY_VERSION || library.profiles.len() > MAX_NARRATION_PROFILES {
            return Err(profile_invalid(
                "unsupported narration library version or profile count",
            ));
        }
        let mut ids = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for profile in &library.profiles {
            profile.validate()?;
            if !ids.insert(&profile.id) || !names.insert(&profile.settings.name) {
                return Err(profile_invalid("duplicate narration profile identity/name"));
            }
        }
        Ok(library)
    }

    pub fn list_narration_profiles(&self) -> Result<Vec<NarrationProfile>, CoreError> {
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        let mut profiles = self.read_narration_library()?.profiles;
        profiles.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(profiles)
    }

    pub fn get_narration_profile(&self, id: &str) -> Result<NarrationProfile, CoreError> {
        self.list_narration_profiles()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| profile_invalid("narration profile not found; list available profiles"))
    }

    pub fn create_narration_profile(
        &self,
        mut settings: NarrationProfileSettings,
    ) -> Result<NarrationProfile, CoreError> {
        settings.name = settings.name.trim().into();
        canonicalize_profile_options(&mut settings.text_options);
        settings.validate()?;
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        let mut library = self.read_narration_library()?;
        if library.profiles.len() == MAX_NARRATION_PROFILES {
            return Err(profile_invalid("narration profile limit reached"));
        }
        if library
            .profiles
            .iter()
            .any(|p| p.settings.name == settings.name)
        {
            return Err(profile_invalid("narration profile name already exists"));
        }
        let profile = NarrationProfile {
            id: Uuid::new_v4().to_string(),
            version: 1,
            settings,
        };
        library.profiles.push(profile.clone());
        write_json_atomic(
            self.storage.as_ref(),
            &self.paths.projects_root().join(LIBRARY_FILE),
            &library,
        )?;
        Ok(profile)
    }

    pub fn update_narration_profile(
        &self,
        id: &str,
        expected_version: u64,
        mut settings: NarrationProfileSettings,
    ) -> Result<NarrationProfile, CoreError> {
        settings.name = settings.name.trim().into();
        canonicalize_profile_options(&mut settings.text_options);
        settings.validate()?;
        let _lock = self.storage.lock_exclusive(self.paths.projects_root())?;
        let mut library = self.read_narration_library()?;
        let index = library
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| {
                profile_invalid("narration profile not found; list available profiles")
            })?;
        let previous = &library.profiles[index];
        profile_version(previous, expected_version)?;
        if library
            .profiles
            .iter()
            .any(|p| p.id != id && p.settings.name == settings.name)
        {
            return Err(profile_invalid("narration profile name already exists"));
        }
        if expected_version >= 9_007_199_254_740_991 {
            return Err(profile_invalid("narration profile version exhausted"));
        }
        let profile = NarrationProfile {
            id: id.into(),
            version: expected_version + 1,
            settings,
        };
        library.profiles[index] = profile.clone();
        write_json_atomic(
            self.storage.as_ref(),
            &self.paths.projects_root().join(LIBRARY_FILE),
            &library,
        )?;
        Ok(profile)
    }

    pub fn resolve_narration_profile(
        &self,
        application: NarrationProfileApplication,
    ) -> Result<SpeechSynthesisRequest, CoreError> {
        let profile = self.get_narration_profile(&application.profile_id)?;
        profile_version(&profile, application.expected_version)?;
        let profile_name = profile.settings.name.clone();
        let mut request = SpeechSynthesisRequest {
            text: application.text.trim().into(),
            language: application.language.unwrap_or(profile.settings.language),
            voice_id: application.voice_id.unwrap_or(profile.settings.voice_id),
            speed: application.speed.unwrap_or(profile.settings.speed),
            text_options: application
                .text_options
                .unwrap_or(profile.settings.text_options),
            narration_profile: Some(NarrationProfileAttribution {
                id: profile.id,
                version: profile.version,
                name: profile.settings.name,
            }),
        };
        canonicalize_profile_options(&mut request.text_options);
        NarrationProfileSettings {
            name: profile_name,
            language: request.language.clone(),
            voice_id: request.voice_id.clone(),
            speed: request.speed,
            text_options: request.text_options.clone(),
        }
        .validate()?;
        request.validate()?;
        Ok(request)
    }
}

fn profile_invalid(message: &str) -> CoreError {
    CoreError::new(ErrorCode::ValidationFailed, message)
}
fn profile_version(profile: &NarrationProfile, expected: u64) -> Result<(), CoreError> {
    if profile.version != expected {
        return Err(CoreError::new(
            ErrorCode::RevisionConflict,
            format!(
                "narration profile version is {}; read it and explicitly reapply",
                profile.version
            ),
        ));
    }
    Ok(())
}

fn canonicalize_profile_options(options: &mut crate::SpeechTextOptions) {
    for entry in &mut options.pronunciations {
        entry.term = entry.term.trim().into();
        entry.spoken = entry.spoken.trim().into();
    }
}
