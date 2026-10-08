use opencut_editor_core::{
    AudioAnalysisDocument, AudioAnalysisOptions, AudioAnalysisResult, AudioAnalysisSummary,
    EditOperation, EditorCore, ErrorCode, PathPolicy, ProjectSettings, Renderer,
};
use serde_json::{Value, json};

fn numeric_json(value: Value) -> Value {
    match value {
        Value::Number(number) => serde_json::to_value(number.as_f64().unwrap()).unwrap(),
        Value::Array(items) => Value::Array(items.into_iter().map(numeric_json).collect()),
        Value::Object(items) => Value::Object(
            items
                .into_iter()
                .map(|(key, value)| (key, numeric_json(value)))
                .collect(),
        ),
        value => value,
    }
}

#[test]
fn closed_analysis_dtos_match_reviewed_catalog_and_never_become_mutations() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../contracts/audio-analysis-v1.json")).unwrap();
    for example in ["summaryExample", "silenceExample", "shortAudibleExample"] {
        let value = catalog[example].clone();
        let summary: AudioAnalysisSummary = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            numeric_json(serde_json::to_value(summary).unwrap()),
            numeric_json(value.clone())
        );
        for key in value.as_object().unwrap().keys() {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<AudioAnalysisSummary>(missing).is_err(),
                "required {key}"
            );
        }
        let mut extra = value;
        extra["privatePath"] = json!("/private");
        assert!(serde_json::from_value::<AudioAnalysisSummary>(extra).is_err());
    }
    for input in [
        catalog["input"].clone(),
        json!({"operation":"analyze_audio","resultAlias":"analysis"}),
    ] {
        assert!(serde_json::from_value::<EditOperation>(input).is_err());
    }
    let summary = catalog["silenceExample"].clone();
    let document = json!({"version":1,"summary":summary,"bins":[{"startFrame":0,"endFrame":4800,"left":{"min":0,"max":0,"rms":0},"right":{"min":0,"max":0,"rms":0}}]});
    assert_eq!(
        numeric_json(
            serde_json::to_value(
                serde_json::from_value::<AudioAnalysisDocument>(document.clone()).unwrap()
            )
            .unwrap()
        ),
        numeric_json(document.clone())
    );
    let result = json!({"artifact":{"relativePath":"previews/analysis.json","mimeType":"application/json","sizeBytes":1,"warnings":[]},"summary":summary});
    assert_eq!(
        numeric_json(
            serde_json::to_value(
                serde_json::from_value::<AudioAnalysisResult>(result.clone()).unwrap()
            )
            .unwrap()
        ),
        numeric_json(result)
    );
    let mut invalid = document;
    invalid["bins"][0]["left"]["normalized"] = json!(true);
    assert!(serde_json::from_value::<AudioAnalysisDocument>(invalid).is_err());
}

#[test]
fn analysis_option_admission_precedes_dependency_and_preserves_current_history_and_reopen() {
    let root = tempfile::tempdir().unwrap();
    let policy = PathPolicy::new(
        root.path().join("projects"),
        [root.path()],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(policy.clone());
    let id = core
        .create_project("Analysis", ProjectSettings::default())
        .unwrap()
        .project_id;
    let initial = core.get_project(&id).unwrap();
    let op:EditOperation=serde_json::from_value(json!({"operation":"add_solid_color","trackId":initial.tracks[1].id,"startMs":0,"durationMs":1000,"color":"#123456","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap();
    core.edit(&id, 0, op).unwrap();
    let project = core.get_project(&id).unwrap();
    let dir = root.path().join("projects").join(&id);
    let bytes = std::fs::read(dir.join("project.json")).unwrap();
    let history = std::fs::read(dir.join("history.json")).unwrap();
    let renderer = Renderer::new(
        root.path().join("missing-ffmpeg"),
        root.path().join("missing-ffprobe"),
        None,
    );
    for (start_ms, end_ms, waveform_bins) in [
        (0, 0, 1),
        (1000, 1000, 1),
        (0, 1001, 1),
        (0, 1000, 0),
        (0, 1000, 4097),
        (u64::MAX, u64::MAX, 1),
    ] {
        assert_eq!(
            renderer
                .analyze_audio(
                    &project,
                    &dir,
                    AudioAnalysisOptions {
                        start_ms,
                        end_ms,
                        waveform_bins
                    },
                    |_| {}
                )
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
        assert_eq!(std::fs::read(dir.join("history.json")).unwrap(), history);
    }
    assert_eq!(
        renderer
            .analyze_audio(
                &project,
                &dir,
                AudioAnalysisOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    waveform_bins: 1
                },
                |_| {}
            )
            .unwrap_err()
            .code,
        ErrorCode::DependencyUnavailable
    );
    assert_eq!(
        serde_json::to_value(EditorCore::new(policy).get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&project).unwrap()
    );
    assert!(!std::fs::read_dir(&dir).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-")
    }));
}
