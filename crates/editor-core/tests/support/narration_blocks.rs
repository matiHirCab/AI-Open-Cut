use opencut_editor_core::*;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) fn fixture() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../contracts/narration-blocks-v1.json"
    )))
    .unwrap()
}
pub(super) fn setup() -> (tempfile::TempDir, EditorCore, NarrationProfile) {
    let root = tempfile::tempdir().unwrap();
    let paths = PathPolicy::new(
        root.path().join("projects"),
        [root.path()],
        root.path().join("exports"),
    )
    .unwrap()
    .with_generated_media_root(root.path())
    .unwrap();
    let core = EditorCore::new(paths);
    let profile = core
        .create_narration_profile(NarrationProfileSettings {
            name: "Sentence test".into(),
            language: "en".into(),
            voice_id: SpeechVoiceId("test_voice".into()),
            speed: 1.0,
            text_options: SpeechTextOptions::default(),
        })
        .unwrap();
    (root, core, profile)
}
pub(super) fn input(
    profile: &NarrationProfile,
    text: &str,
    boundaries: Option<Vec<usize>>,
) -> NarrationPlanInput {
    NarrationPlanInput {
        source_text: text.into(),
        profile_id: profile.id.clone(),
        expected_version: profile.version,
        max_text_characters: 4096,
        boundaries,
    }
}
pub(super) fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(path: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(&p, result);
            } else {
                result.insert(p.clone(), std::fs::read(p).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    walk(root, &mut result);
    result
}
pub(super) fn generated(
    root: &Path,
    block: &NarrationPlannedBlock,
    duration: u64,
) -> NarrationGeneratedBlock {
    let path = root.join(format!("{}.wav", block.id));
    std::fs::write(&path, b"owned synthetic unit fixture").unwrap();
    NarrationGeneratedBlock {
        prepared: NarrationPreparedBlock {
            block: block.clone(),
            spoken_segments: vec![block.request.text.clone()],
            spoken_text: block.request.text.clone(),
        },
        path,
        duration_ms: duration,
        origin: GeneratedAssetOrigin::SpeechSynthesis(SpeechGeneration {
            request: block.request.clone(),
            provider_id: "synthesis-fixture".into(),
            model_id: "synthetic-model".into(),
            model_version: None,
            sample_rate_hz: 24000,
            generated_at_ms: 1,
            alignment: None,
        }),
        probe: MediaProbeFacts::default(),
    }
}
pub(super) fn commit_input(
    root: &Path,
    core: &EditorCore,
    profile: &NarrationProfile,
) -> CommitNarrationBlocksInput {
    let plan = core
        .plan_narration_blocks(input(profile, "A. B.", Some(vec![3, 5])))
        .unwrap();
    let project = core
        .create_project("Sentence blocks", ProjectSettings::default())
        .unwrap();
    let state = core.get_project(&project.project_id).unwrap();
    let audio = state
        .tracks
        .iter()
        .find(|t| t.track_type == TrackType::Audio)
        .unwrap();
    let durations = [400, 700];
    let blocks = plan
        .blocks
        .iter()
        .zip(durations)
        .map(|(b, d)| generated(root, b, d))
        .collect();
    CommitNarrationBlocksInput {
        project_id: state.id,
        expected_revision: 0,
        plan,
        track_id: audio.id.clone(),
        start_ms: 100,
        blocks,
        marker_policy: SpeechMarkerPolicy::default(),
        caption: None,
    }
}
