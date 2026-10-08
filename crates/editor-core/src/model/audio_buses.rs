use serde::{Deserialize, Deserializer, Serialize};

pub const AUDIO_BUS_IDS: [&str; 4] = ["voiceover", "music", "sfx", "master"];
pub const MAX_AUDIO_BUSES: usize = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioBus {
    pub id: String,
    #[serde(deserialize_with = "deserialize_nullable_bus_id")]
    pub output_bus_id: Option<String>,
}

pub(super) fn deserialize_nullable_bus_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

pub fn default_audio_buses() -> Vec<AudioBus> {
    AUDIO_BUS_IDS
        .into_iter()
        .map(|id| AudioBus {
            id: id.to_owned(),
            output_bus_id: (id != "master").then(|| "master".to_owned()),
        })
        .collect()
}

impl super::AudioTrackRole {
    pub const fn default_audio_bus_id(self) -> &'static str {
        match self {
            Self::Unassigned => "master",
            Self::Voiceover => "voiceover",
            Self::Music => "music",
            Self::SoundEffects => "sfx",
        }
    }
}
