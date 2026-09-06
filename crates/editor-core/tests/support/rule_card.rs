use opencut_editor_core::{
    BatchEditOperation, EditOperation, EditorCore, MediaProbeFacts, MediaType, PathPolicy, Project,
    ProjectSettings, TrackType,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub struct Fixture {
    pub core: EditorCore,
    pub id: String,
    pub aliases: BTreeMap<String, String>,
}

impl Fixture {
    pub fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }

    pub fn move_parent(&self) {
        let mut transform = opencut_editor_core::Transform2D::default();
        transform.position.x = 6.0;
        transform.position.y = 4.0;
        self.core.edit(&self.id, self.project().revision, operation(json!({
            "operation":"update_item", "itemId":self.aliases["parent"], "transform2d":transform
        }))).unwrap();
    }
}

pub fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

// An uncompressed 8x8 BGR bitmap avoids native encoders and external media.
pub fn icon_bytes(rgb: [u8; 3]) -> Vec<u8> {
    let mut bytes = vec![0; 54];
    bytes[0..2].copy_from_slice(b"BM");
    bytes[2..6].copy_from_slice(&246_u32.to_le_bytes());
    bytes[10..14].copy_from_slice(&54_u32.to_le_bytes());
    bytes[14..18].copy_from_slice(&40_u32.to_le_bytes());
    bytes[18..22].copy_from_slice(&8_u32.to_le_bytes());
    bytes[22..26].copy_from_slice(&8_u32.to_le_bytes());
    bytes[26..28].copy_from_slice(&1_u16.to_le_bytes());
    bytes[28..30].copy_from_slice(&24_u16.to_le_bytes());
    for _ in 0..64 {
        bytes.extend_from_slice(&[rgb[2], rgb[1], rgb[0]]);
    }
    bytes
}

pub fn tone_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&96036_u32.to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&48000_u32.to_le_bytes());
    bytes.extend_from_slice(&96000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&96000_u32.to_le_bytes());
    for n in 0..48000 {
        let sample =
            ((std::f64::consts::TAU * 440.0 * f64::from(n) / 48000.0).sin() * 4096.0) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

pub fn seed(root: &Path) -> Fixture {
    let media = root.join("media");
    std::fs::create_dir_all(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [&media], root.join("exports")).unwrap(),
    );
    let id = core
        .create_project(
            "Three rule cards",
            ProjectSettings {
                width: 160,
                height: 90,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let project = core.get_project(&id).unwrap();
    let mut replacements = BTreeMap::new();
    for (key, kind) in [
        ("$overlay", TrackType::Overlay),
        ("$audio", TrackType::Audio),
    ] {
        replacements.insert(
            key.to_owned(),
            project
                .tracks
                .iter()
                .find(|t| t.track_type == kind)
                .unwrap()
                .id
                .clone(),
        );
    }
    for (i, color) in [[240, 80, 80], [80, 240, 80], [80, 80, 240]]
        .into_iter()
        .enumerate()
    {
        let path = media.join(format!("icon{i}.bmp"));
        std::fs::write(&path, icon_bytes(color)).unwrap();
        let result = core
            .import_asset(
                &id,
                i as u64,
                path,
                MediaType::Image,
                MediaProbeFacts {
                    has_video: true,
                    video_width: Some(8),
                    video_height: Some(8),
                    ..Default::default()
                },
            )
            .unwrap();
        replacements.insert(format!("$icon{i}"), result.changed_ids[0].clone());
    }
    let tone = media.join("tone.wav");
    std::fs::write(&tone, tone_bytes()).unwrap();
    let imported = core
        .import_asset(
            &id,
            3,
            tone,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(1000),
                has_audio: true,
                audio_channels: Some(1),
                audio_sample_rate_hz: Some(48000),
                ..Default::default()
            },
        )
        .unwrap();
    replacements.insert("$tone".into(), imported.changed_ids[0].clone());
    let mut recipe: Value =
        serde_json::from_str(include_str!("../fixtures/rule-card/recipe.json")).unwrap();
    substitute(&mut recipe, &replacements);
    let edits: Vec<BatchEditOperation> = serde_json::from_value(recipe).unwrap();
    let result = core.edit_batch(&id, 4, edits).unwrap();
    Fixture {
        core,
        id,
        aliases: result.aliases.into_iter().collect(),
    }
}

fn substitute(value: &mut Value, replacements: &BTreeMap<String, String>) {
    match value {
        Value::String(s) => {
            if let Some(replacement) = replacements.get(s) {
                *s = replacement.clone();
            }
        }
        Value::Array(values) => {
            for value in values {
                substitute(value, replacements);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                substitute(value, replacements);
            }
        }
        _ => {}
    }
}
