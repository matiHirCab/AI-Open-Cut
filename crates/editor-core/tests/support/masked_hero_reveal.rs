use opencut_editor_core::{
    EditOperation, EditorCore, MediaProbeFacts, MediaType, PathPolicy, ProjectSettings,
};
use serde_json::{Value, json};
#[path = "masked_hero_authoring.rs"]
mod authoring;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
pub fn catalog() -> Value {
    serde_json::from_str(include_str!(
        "../../../../contracts/masked-hero-reveal-v1.json"
    ))
    .unwrap()
}
pub fn reference(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/masked-hero-reveal-v1")
        .join(name)
}
pub fn op(value: Value) -> EditOperation {
    serde_json::from_value(value).unwrap()
}
pub fn inventory(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(path: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.insert(path.clone(), b"directory".to_vec());
                visit(&path, out);
            } else if path.file_name().unwrap() != ".lock" {
                out.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, &mut out);
    out
}
pub struct Fixture {
    pub root: tempfile::TempDir,
    pub core: EditorCore,
    pub id: String,
    pub ids: BTreeMap<String, String>,
}
impl Fixture {
    pub fn new(batch: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let media = root.path().join("media");
        std::fs::create_dir(&media).unwrap();
        let wav = media.join("source.wav");
        std::fs::copy(reference("source.wav"), &wav).unwrap();
        let core = EditorCore::new(
            PathPolicy::new(
                root.path().join("projects"),
                [&media],
                root.path().join("exports"),
            )
            .unwrap(),
        );
        let c = catalog();
        let id = core
            .create_project(
                "Masked hero reveal v1",
                serde_json::from_value::<ProjectSettings>(c["settings"].clone()).unwrap(),
            )
            .unwrap()
            .project_id;
        let project = core.get_project(&id).unwrap();
        let tracks = serde_json::to_value(&project.tracks).unwrap();
        assert_eq!(tracks[1]["name"], "Overlay");
        assert_eq!(tracks[1]["trackType"], "overlay");
        assert_eq!(tracks[2]["name"], "Audio");
        assert_eq!(tracks[2]["trackType"], "audio");
        let imported = core
            .import_asset(
                &id,
                0,
                &wav,
                MediaType::Audio,
                MediaProbeFacts {
                    duration_ms: Some(800),
                    has_audio: true,
                    has_video: false,
                    format_name: Some("wav".into()),
                    audio_codec: Some("pcm_s16le".into()),
                    audio_channels: Some(2),
                    audio_sample_rate_hz: Some(48000),
                    ..Default::default()
                },
            )
            .unwrap();
        let text = serde_json::to_string(&c["operationTranscript"]["aliasBatch"])
            .unwrap()
            .replace("{{overlayTrackId}}", &project.tracks[1].id)
            .replace("{{audioTrackId}}", &project.tracks[2].id)
            .replace("{{audioAssetId}}", &imported.changed_ids[0]);
        let operations: Vec<Value> = serde_json::from_str(&text).unwrap();
        let mut ids = BTreeMap::new();
        if batch {
            ids = core
                .edit_batch::<opencut_editor_core::BatchEditOperation>(
                    &id,
                    1,
                    serde_json::from_value(serde_json::to_value(operations).unwrap()).unwrap(),
                )
                .unwrap()
                .aliases;
        } else {
            for mut value in operations {
                let alias = value.as_object_mut().unwrap().remove("resultAlias");
                let mut text = serde_json::to_string(&value).unwrap();
                for (role, id) in &ids {
                    text = text.replace(&format!("\"@{role}\""), &format!("\"{id}\""));
                }
                let result = core
                    .edit(
                        &id,
                        core.get_project(&id).unwrap().revision,
                        serde_json::from_str(&text).unwrap(),
                    )
                    .unwrap();
                if let Some(alias) = alias {
                    ids.insert(
                        alias.as_str().unwrap().into(),
                        result.changed_ids[0].clone(),
                    );
                }
            }
        }
        let fixture = Self {
            root,
            core,
            id,
            ids,
        };
        fixture.assert_recipe(false);
        fixture
    }
    pub fn rev(&self) -> u64 {
        self.core.get_project(&self.id).unwrap().revision
    }
    pub fn dir(&self) -> PathBuf {
        self.core.project_directory(&self.id).unwrap()
    }
    pub fn reorder(&self) -> EditOperation {
        let mut effects = catalog()["roles"]["hero"]["effects"].clone();
        effects.as_array_mut().unwrap().swap(1, 2);
        op(json!({"operation":"update_item","itemId":self.ids["hero"],"effects":effects}))
    }
    pub fn assert_recipe(&self, reversed: bool) {
        let p = self.core.get_project(&self.id).unwrap();
        authoring::assert_recipe(&p, &self.ids, &catalog(), reversed);
    }
}
