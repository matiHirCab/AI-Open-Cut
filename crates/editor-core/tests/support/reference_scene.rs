//! Test-owned recipe; all project/scene validation and mutation remains in core.
use opencut_editor_core::{
    BatchEditOperation, CommitGeneratedAssetRequest, EditorCore, GeneratedAssetOrigin,
    MediaProbeFacts, MediaType, PathPolicy, Project, SpeechMarkerPolicy,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn recipe() -> Value {
    serde_json::from_str(include_str!(
        "../../../../contracts/complete-reference-scene-v1.json"
    ))
    .unwrap()
}

pub fn substitute(value: &mut Value, ids: &BTreeMap<String, String>) {
    match value {
        Value::String(s) if ids.contains_key(s) => *s = ids[s].clone(),
        Value::Array(a) => a.iter_mut().for_each(|v| substitute(v, ids)),
        Value::Object(o) => o.values_mut().for_each(|v| substitute(v, ids)),
        _ => {}
    }
}

pub fn source(path: &Path, frequency: f64, amplitude: f64, duration_ms: u64, voice: bool) {
    let size = u32::try_from(duration_ms * 48 * 4).unwrap();
    let mut bytes = Vec::with_capacity(size as usize + 44);
    bytes.extend(b"RIFF");
    bytes.extend((size + 36).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(192000_u32.to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(size.to_le_bytes());
    for frame in 0..duration_ms * 48 {
        let ms = frame / 48;
        let active = !voice
            || [
                (500, 900),
                (1000, 1400),
                (1500, 1900),
                (2400, 2900),
                (3200, 3900),
                (4300, 5000),
            ]
            .iter()
            .any(|&(start, end)| (start..end).contains(&ms));
        let sample = if active {
            (amplitude
                * (std::f64::consts::TAU * frequency * frame as f64 / 48000.0).sin()
                * 32767.0)
                .round() as i16
        } else {
            0
        };
        bytes.extend(sample.to_le_bytes());
        bytes.extend(sample.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}

pub struct Fixture {
    pub core: EditorCore,
    pub id: String,
    pub aliases: BTreeMap<String, String>,
}
impl Fixture {
    pub fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    pub fn dir(&self) -> PathBuf {
        self.core.project_directory(&self.id).unwrap()
    }
}

pub fn seed(root: &Path, batch: bool) -> Fixture {
    let media = root.join("media");
    std::fs::create_dir_all(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [&media], root.join("exports"))
            .unwrap()
            .with_generated_media_root(&media)
            .unwrap(),
    );
    let c = recipe();
    let id = core
        .create_project(
            c["name"].as_str().unwrap(),
            serde_json::from_value(c["settings"].clone()).unwrap(),
        )
        .unwrap()
        .project_id;
    let p = core.get_project(&id).unwrap();
    assert!(p.tracks.iter().all(|t| t.items.is_empty()));
    assert!(p.assets.is_empty());
    assert!(p.components.is_empty());
    let mut ids = BTreeMap::from([
        ("$visual".into(), p.tracks[1].id.clone()),
        ("$audio".into(), p.tracks[2].id.clone()),
    ]);
    let path = media.join("voice.wav");
    source(&path, 437.0, 0.1, 6000, true);
    let voice = core.commit_generated_asset(CommitGeneratedAssetRequest { project_id:id.clone(),expected_revision:0,path,
        track_id:ids["$audio"].clone(),start_ms:0,duration_ms:6000,display_name:"Synthetic reference narration".into(),marker_policy:SpeechMarkerPolicy::None {},
        origin:GeneratedAssetOrigin::SpeechSynthesis(serde_json::from_value(json!({"request":{"text":"EVERY SINGLE ONE rules Starting with Venusaur","language":"en","voiceId":"fixture","speed":1},"providerId":"reference-synthetic-source","modelId":"oscillator","modelVersion":null,"sampleRateHz":48000,"generatedAtMs":1,"alignment":c["alignment"]})).unwrap()),
        probe:MediaProbeFacts {duration_ms:Some(6000),has_audio:true,audio_channels:Some(2),audio_sample_rate_hz:Some(48000),..Default::default()} }).unwrap().asset_id;
    ids.insert("$voice".into(), voice);
    for (name, frequency, amplitude, duration) in [
        ("music", 211.0, 0.08, 6000),
        ("event0", 659.0, 0.1, 300),
        ("event1", 877.0, 0.1, 300),
    ] {
        let path = media.join(format!("{name}.wav"));
        source(&path, frequency, amplitude, duration, false);
        let p = core.get_project(&id).unwrap();
        let asset = core
            .import_asset(
                &id,
                p.revision,
                path,
                MediaType::Audio,
                MediaProbeFacts {
                    duration_ms: Some(duration),
                    has_audio: true,
                    audio_channels: Some(2),
                    audio_sample_rate_hz: Some(48000),
                    ..Default::default()
                },
            )
            .unwrap()
            .changed_ids[0]
            .clone();
        ids.insert(format!("${name}"), asset);
    }
    let mut operations = c["operations"].clone();
    substitute(&mut operations, &ids);
    let aliases = if batch {
        let p = core.get_project(&id).unwrap();
        core.edit_batch::<BatchEditOperation>(
            &id,
            p.revision,
            serde_json::from_value(operations).unwrap(),
        )
        .unwrap()
        .aliases
    } else {
        let mut aliases: BTreeMap<String, String> = BTreeMap::new();
        let mut revision = core.get_project(&id).unwrap().revision;
        for mut operation in operations.as_array().unwrap().iter().cloned() {
            let alias = operation.as_object_mut().unwrap().remove("resultAlias");
            let replacements = aliases
                .iter()
                .map(|(key, value)| (format!("@{key}"), value.clone()))
                .collect();
            substitute(&mut operation, &replacements);
            let label = serde_json::to_string(&operation).unwrap();
            let result = core
                .edit(&id, revision, serde_json::from_value(operation).unwrap())
                .unwrap_or_else(|error| panic!("{label}: {error:?}"));
            assert_eq!(result.revision, revision + 1);
            revision = result.revision;
            if let Some(alias) = alias {
                aliases.insert(
                    alias.as_str().unwrap().to_owned(),
                    result.changed_ids[0].clone(),
                );
            }
        }
        aliases
    };
    Fixture { core, id, aliases }
}
