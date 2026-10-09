//! Test-only typed authoring recipe and independently declared local source samples.
use opencut_editor_core::{
    BatchEditOperation, CommitGeneratedAssetRequest, EditOperation, EditorCore,
    GeneratedAssetOrigin, MediaProbeFacts, MediaType, PathPolicy, Project, SpeechMarkerPolicy,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub fn recipe() -> Value {
    serde_json::from_str(include_str!(
        "../../../../contracts/narration-driven-fixture-v1.json"
    ))
    .unwrap()
}
pub fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

pub fn samples(frequency: f64, amplitude: f64, duration: u64, voice: bool) -> Vec<i16> {
    (0..duration * 48)
        .flat_map(|frame| {
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
            [sample, sample]
        })
        .collect()
}
pub fn wav(path: &Path, samples: &[i16]) {
    let length = u32::try_from(samples.len() * 2).unwrap();
    let mut bytes = Vec::with_capacity(length as usize + 44);
    bytes.extend(b"RIFF");
    bytes.extend((length + 36).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16_u32.to_le_bytes());
    bytes.extend(1_u16.to_le_bytes());
    bytes.extend(2_u16.to_le_bytes());
    bytes.extend(48000_u32.to_le_bytes());
    bytes.extend(192000_u32.to_le_bytes());
    bytes.extend(4_u16.to_le_bytes());
    bytes.extend(16_u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend(length.to_le_bytes());
    for sample in samples {
        bytes.extend(sample.to_le_bytes());
    }
    std::fs::write(path, bytes).unwrap();
}
pub fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(inventory(&path));
        } else if path.file_name().unwrap() != ".lock" {
            result.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    result
}
pub struct Fixture {
    pub core: EditorCore,
    pub id: String,
    pub asset: String,
    pub plain_asset: String,
    pub aliases: BTreeMap<String, String>,
}
impl Fixture {
    pub fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }
    pub fn dir(&self) -> PathBuf {
        self.core.paths().projects_root().join(&self.id)
    }
}
pub fn seed(root: &Path, batch: bool) -> Fixture {
    std::fs::create_dir_all(root).unwrap();
    let media = root.join("media");
    std::fs::create_dir(&media).unwrap();
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
    let initial = core.get_project(&id).unwrap();
    let narration = initial
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Audio)
        .unwrap()
        .id
        .clone();
    let visual = initial
        .tracks
        .iter()
        .find(|track| track.track_type == opencut_editor_core::TrackType::Overlay)
        .unwrap()
        .id
        .clone();
    let voice = media.join("narration.wav");
    wav(&voice, &samples(437.0, 0.1, 6000, true));
    let asset = core.commit_generated_asset(CommitGeneratedAssetRequest {
        project_id: id.clone(), expected_revision: 0, path: voice, track_id: narration.clone(),
        start_ms: 0, duration_ms: 6000, display_name: "Synthetic narration".into(),
        marker_policy: SpeechMarkerPolicy::None {},
        origin: GeneratedAssetOrigin::SpeechSynthesis(serde_json::from_value(json!({
            "request":{"text":"EVERY SINGLE ONE rules Starting with Venusaur","language":"en","voiceId":"fixture","speed":1},
            "providerId":"synthetic-source","modelId":"oscillator","modelVersion":null,
            "sampleRateHz":48000,"generatedAtMs":1,"alignment":c["alignment"]
        })).unwrap()),
        probe: MediaProbeFacts {duration_ms:Some(6000),has_audio:true,audio_channels:Some(2),audio_sample_rate_hz:Some(48000), ..Default::default()},
    }).unwrap().asset_id;
    let mut revision = 1;
    let mut imported = vec![];
    for (name, frequency, amplitude, duration) in [
        ("plain", 437.0, 0.1, 6000),
        ("music", 211.0, 0.08, 6000),
        ("variant0", 659.0, 0.1, 300),
        ("variant1", 877.0, 0.1, 300),
    ] {
        let source = media.join(format!("{name}.wav"));
        wav(
            &source,
            &samples(frequency, amplitude, duration, name == "plain"),
        );
        imported.push(
            core.import_asset(
                &id,
                revision,
                source,
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
                .clone(),
        );
        revision += 1;
    }
    let mut edits = vec![
        json!({"operation":"audio_track_route","scope":"root","trackId":narration,"busId":"voiceover"}),
        json!({"operation":"create_track","name":"Music","trackType":"audio","resultAlias":"music"}),
        json!({"operation":"audio_track_route","scope":"root","trackId":"@music","busId":"music"}),
        json!({"operation":"add_media","trackId":"@music","assetId":imported[1],"startMs":0,"durationMs":6000,"sourceInMs":0}),
        json!({"operation":"create_track","name":"Events","trackType":"audio","resultAlias":"events"}),
        json!({"operation":"sound_event_register","event":"narration_accent","variantAssetIds":[imported[2],imported[3]],"defaultGainDb":-6,"busId":"sfx","variantSeed":1}),
        json!({"operation":"speech_markers_generate","assetId":asset,"scope":"root","startMs":0,"markerPolicy":{"type":"sentence"}}),
        json!({"operation":"audio_bus_set_ducking","busId":"music","ducking":c["ducking"]}),
        json!({"operation":"audio_master_set_normalization","normalization":c["normalization"]}),
    ];
    for (index, cue) in c["cues"].as_array().unwrap().iter().enumerate() {
        let alias = format!("visual{index}");
        edits.extend([
            json!({"operation":"add_rectangle","trackId":visual,"startMs":0,"durationMs":400,"width":64,"height":64,"color":cue["color"],"transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1},"resultAlias":alias}),
            json!({"operation":"apply_animation_preset","itemId":format!("@{alias}"),"presetId":"scalar_tween","presetVersion":1,"parameters":c["visual"]["parameters"]}),
            json!({"operation":"set_item_start_time","scope":"root","itemId":format!("@{alias}"),"time":{"type":"marker","markerName":cue["name"],"offsetMs":0}}),
            json!({"operation":"timeline_add_audio_event","scope":"root","trackId":"@events","event":"narration_accent","at":{"type":"marker","markerName":cue["name"],"offsetMs":0},"durationMs":300,"gainDb":-3,"resultAlias":format!("event{index}")}),
        ]);
    }
    let mut aliases = BTreeMap::new();
    if batch {
        let result = core
            .edit_batch(
                &id,
                revision,
                serde_json::from_value::<Vec<BatchEditOperation>>(json!(edits)).unwrap(),
            )
            .unwrap();
        aliases = result.aliases;
    } else {
        for mut edit in edits {
            let alias = edit
                .as_object_mut()
                .unwrap()
                .remove("resultAlias")
                .and_then(|value| value.as_str().map(str::to_owned));
            for key in ["trackId", "itemId"] {
                if let Some(name) = edit[key].as_str().and_then(|value| value.strip_prefix('@')) {
                    edit[key] = json!(aliases[name]);
                }
            }
            let result = core.edit(&id, revision, op(edit)).unwrap();
            if let Some(alias) = alias {
                aliases.insert(alias, result.changed_ids[0].clone());
            }
            revision += 1;
        }
    }
    Fixture {
        core,
        id,
        asset,
        plain_asset: imported[0].clone(),
        aliases,
    }
}
