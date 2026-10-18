//! Original red/blue/tone GUI fixture. Requires a new disposable directory.
use opencut_editor_core::{
    EditorCore, MediaProbeFacts, MediaType, PathPolicy, ProjectSettings, TrackType,
};
use serde_json::json;

fn main() {
    let root = std::path::PathBuf::from(std::env::args_os().nth(1).expect("new fixture directory"));
    assert!(!root.exists(), "fixture directory must be new");
    std::fs::create_dir_all(&root).unwrap();
    let media = root.join("original-tone.wav");
    assert!(
        std::process::Command::new(
            std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into())
        )
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:duration=3"
        ])
        .arg(&media)
        .status()
        .unwrap()
        .success()
    );
    let core = EditorCore::new(
        PathPolicy::new(root.join("projects"), [&root], root.join("exports")).unwrap(),
    );
    let id = core
        .create_project(
            "Original embedded review control",
            ProjectSettings {
                width: 320,
                height: 180,
                fps: 10,
            },
        )
        .unwrap()
        .project_id;
    let initial = core.get_project(&id).unwrap();
    let visual = &initial
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Overlay)
        .unwrap()
        .id;
    let audio = &initial
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Audio)
        .unwrap()
        .id;
    let imported = core
        .import_asset(
            &id,
            0,
            &media,
            MediaType::Audio,
            MediaProbeFacts {
                duration_ms: Some(3000),
                has_audio: true,
                audio_channels: Some(1),
                audio_sample_rate_hz: Some(44100),
                ..Default::default()
            },
        )
        .unwrap();
    let asset = &imported.changed_ids[0];
    let result = core.edit_batch(&id, imported.revision, serde_json::from_value::<Vec<opencut_editor_core::BatchEditOperation>>(json!([
        {"operation":"add_media","trackId":audio,"assetId":asset,"startMs":0,"durationMs":3000,"sourceInMs":0},
        {"operation":"add_solid_color","trackId":visual,"startMs":0,"durationMs":1500,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}},
        {"operation":"add_solid_color","trackId":visual,"startMs":1500,"durationMs":1500,"color":"#0000ff","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}}
    ])).unwrap()).unwrap();
    println!(
        "--project-store {} --project-id {id}",
        core.paths().projects_root().display()
    );
    println!(
        "revision {} · 320x180 · 10 fps · red 0–1500ms · blue 1500–3000ms · original 880Hz audio",
        result.revision
    );
}
