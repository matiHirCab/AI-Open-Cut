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
    pub width: u32,
    pub height: u32,
}

impl Fixture {
    pub fn project(&self) -> Project {
        self.core.get_project(&self.id).unwrap()
    }

    pub fn resize_impact_word(&self) {
        let revision = self.project().revision;
        let size = 144 * self.width / 1920;
        self.core
            .edit(
                &self.id,
                revision,
                operation(json!({"operation":"update_item","itemId":self.aliases["word1_face"],"fontSize":size})),
            )
            .unwrap();
    }
}

pub fn operation(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}

fn tone_bytes() -> Vec<u8> {
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

pub fn seed(root: &Path) -> Fixture {
    seed_at(root, 1920, 1080)
}

fn scale_number(value: &mut Value, factor: f64) {
    if let Some(number) = value.as_f64() {
        *value = json!(number * factor);
    }
}

fn scale_stroke(value: &mut Value, factor: f64) {
    if !value.is_null() {
        scale_number(&mut value["width"], factor);
        if let Some(entries) = value["dash"].as_array_mut() {
            for entry in entries {
                scale_number(entry, factor);
            }
        }
        scale_number(&mut value["dashOffset"], factor);
    }
}

fn scale_recipe(recipe: &mut Value, factor: f64) {
    for edit in recipe["operations"].as_array_mut().unwrap() {
        match edit["operation"].as_str().unwrap() {
            "add_shape" => {
                let geometry = &mut edit["geometry"];
                for name in ["width", "height"] {
                    if geometry.get(name).is_some() {
                        scale_number(&mut geometry[name], factor);
                    }
                }
                if let Some(radii) = geometry.get_mut("radii") {
                    for name in ["topLeft", "topRight", "bottomRight", "bottomLeft"] {
                        scale_number(&mut radii[name], factor);
                    }
                }
                for name in ["start", "end"] {
                    if let Some(point) = geometry.get_mut(name) {
                        for axis in ["x", "y"] {
                            scale_number(&mut point[axis], factor);
                        }
                    }
                }
                if let Some(commands) = geometry
                    .pointer_mut("/path/commands")
                    .and_then(Value::as_array_mut)
                {
                    for command in commands {
                        if let Some(point) = command.get_mut("to") {
                            for axis in ["x", "y"] {
                                scale_number(&mut point[axis], factor);
                            }
                        }
                    }
                }
                scale_stroke(&mut edit["stroke"], factor);
            }
            "add_grid" => {
                for name in ["width", "height"] {
                    scale_number(&mut edit["grid"][name], factor);
                }
                for name in ["spacing", "spacingX", "spacingY"] {
                    if edit["grid"]["pattern"].get(name).is_some() {
                        scale_number(&mut edit["grid"]["pattern"][name], factor);
                    }
                }
                scale_stroke(&mut edit["grid"]["pattern"]["stroke"], factor);
            }
            "add_text" => {
                let size = edit["fontSize"].as_u64().unwrap();
                edit["fontSize"] = json!((size as f64 * factor).round() as u32);
                for name in ["positionX", "positionY"] {
                    scale_number(&mut edit["transform"][name], factor);
                }
            }
            "update_item" => {
                if let Some(size) = edit["fontSize"].as_u64() {
                    edit["fontSize"] = json!((size as f64 * factor).round() as u32);
                }
            }
            _ => {}
        }
        if let Some(position) = edit
            .get_mut("transform2d")
            .and_then(|t| t.get_mut("position"))
        {
            for axis in ["x", "y"] {
                scale_number(&mut position[axis], factor);
            }
        }
    }
}

pub fn seed_at(root: &Path, width: u32, height: u32) -> Fixture {
    assert!(matches!(
        (width, height),
        (1920, 1080) | (1280, 720) | (960, 540)
    ));
    let media = root.join("media");
    std::fs::create_dir_all(&media).unwrap();
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [&media], root.join("exports")).unwrap(),
    );
    let id = core
        .create_project(
            "Static rules screen",
            ProjectSettings {
                width,
                height,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let project = core.get_project(&id).unwrap();
    let mut replacements = BTreeMap::new();
    for (placeholder, kind) in [
        ("$overlay", TrackType::Overlay),
        ("$audio", TrackType::Audio),
    ] {
        replacements.insert(
            placeholder.to_owned(),
            project
                .tracks
                .iter()
                .find(|track| track.track_type == kind)
                .unwrap()
                .id
                .clone(),
        );
    }
    let tone = media.join("tone.wav");
    std::fs::write(&tone, tone_bytes()).unwrap();
    let imported = core
        .import_asset(
            &id,
            0,
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
        serde_json::from_str(include_str!("../fixtures/rules-screen/recipe.json")).unwrap();
    assert_eq!(recipe["schemaVersion"], 1);
    assert_eq!(
        recipe["canvas"],
        json!({"width":1920,"height":1080,"fps":10})
    );
    assert_eq!(recipe["durationMs"], 1000);
    assert_eq!(recipe["timestampsMs"], json!([0, 500, 900]));
    assert_eq!(
        recipe["fontSha256"],
        "ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280"
    );
    assert_eq!(
        recipe["syntheticAudio"],
        json!({"sampleRateHz":48000,"frequencyHz":440,"channels":1})
    );
    if width != 1920 {
        scale_recipe(&mut recipe, f64::from(width) / 1920.0);
    }
    substitute(&mut recipe, &replacements);
    let edits: Vec<BatchEditOperation> =
        serde_json::from_value(recipe["operations"].clone()).unwrap();
    let result = core.edit_batch(&id, imported.revision, edits).unwrap();
    Fixture {
        core,
        id,
        aliases: result.aliases.into_iter().collect(),
        width,
        height,
    }
}
