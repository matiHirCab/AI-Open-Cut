use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, PathPolicy, PreviewDimensions, PreviewPreset,
    PreviewResolution, PreviewReviewOptions, ProjectSettings,
};
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../contracts/preview-review-v1.json")).unwrap()
}

fn setup() -> (tempfile::TempDir, EditorCore, String) {
    let root = tempfile::tempdir().unwrap();
    let core = EditorCore::new(
        PathPolicy::new(
            root.path().join("projects"),
            [root.path()],
            root.path().join("exports"),
        )
        .unwrap(),
    );
    let id = core
        .create_project("Review", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    let edit: EditOperation = serde_json::from_value(json!({"operation":"add_solid_color","trackId":track,"startMs":0,"durationMs":1000,"color":"#ff0000","transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap();
    core.edit(&id, 0, edit).unwrap();
    (root, core, id)
}

fn options(resolution: PreviewResolution) -> PreviewReviewOptions {
    PreviewReviewOptions {
        start_ms: 0,
        end_ms: 1000,
        resolution,
        fps: None,
        include_audio: None,
    }
}

#[test]
fn canonical_review_aspects_rounding_custom_and_defaults() {
    let (_root, core, id) = setup();
    let original = core.get_project(&id).unwrap();
    for case in fixture()["cases"].as_array().unwrap() {
        let mut project = original.clone();
        project.settings = serde_json::from_value(case["project"].clone()).unwrap();
        let resolved = options(serde_json::from_value(case["resolution"].clone()).unwrap())
            .resolve(&project)
            .unwrap();
        assert_eq!(
            json!({"width":resolved.width,"height":resolved.height,"fps":resolved.fps,"includeAudio":resolved.include_audio}),
            case["expected"]
        );
    }
    let default = options(PreviewResolution::default())
        .resolve(&original)
        .unwrap();
    assert_eq!(
        (
            default.width,
            default.height,
            default.fps,
            default.include_audio
        ),
        (1920, 1080, 30, true)
    );
    let mut explicit = options(PreviewResolution::Custom(PreviewDimensions {
        width: 123,
        height: 99,
    }));
    explicit.fps = Some(120);
    explicit.include_audio = Some(false);
    let resolved = explicit.resolve(&original).unwrap();
    assert_eq!(
        (
            resolved.width,
            resolved.height,
            resolved.fps,
            resolved.include_audio
        ),
        (123, 99, 120, false)
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        serde_json::to_value(reopened.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
}

#[test]
fn canonical_review_rejects_bounds_intervals_and_malformed_selections() {
    let (_root, core, id) = setup();
    let project = core.get_project(&id).unwrap();
    for value in fixture()["invalidResolutions"].as_array().unwrap() {
        assert!(
            serde_json::from_value::<PreviewResolution>(value.clone()).is_err(),
            "{value}"
        );
    }
    for (width, height) in [
        (0, 1),
        (1, 0),
        (7681, 4320),
        (7680, 4321),
        (u32::MAX, u32::MAX),
    ] {
        assert_eq!(
            options(PreviewResolution::Custom(PreviewDimensions {
                width,
                height
            }))
            .resolve(&project)
            .unwrap_err()
            .code,
            ErrorCode::ValidationFailed
        );
    }
    for fps in [0, 121, u32::MAX] {
        let mut request = options(PreviewResolution::default());
        request.fps = Some(fps);
        assert_eq!(
            request.resolve(&project).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
    }
    for (start, end) in [(0, 0), (1, 0), (0, 1001)] {
        let mut request = options(PreviewResolution::default());
        request.start_ms = start;
        request.end_ms = end;
        assert_eq!(
            request.resolve(&project).unwrap_err().code,
            ErrorCode::ValidationFailed
        );
    }
    let mut extreme = project.clone();
    extreme.settings.width = 7680;
    extreme.settings.height = 1;
    assert_eq!(
        options(PreviewResolution::Preset(PreviewPreset::P720))
            .resolve(&extreme)
            .unwrap_err()
            .code,
        ErrorCode::ValidationFailed
    );
    assert_eq!(
        serde_json::to_value(core.get_project(&id).unwrap()).unwrap(),
        serde_json::to_value(&project).unwrap()
    );
}
