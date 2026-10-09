use crate::narration_tests::fixture;

use crate::{
    review::{self, InspectorTab, Request, Review},
    session::{Command, Session, Startup},
};
use opencut_editor_core::{CoreError, EditOperation, ErrorCode, PreviewPreset, Renderer};

fn startup(f: &fixture::Fixture) -> Startup {
    Startup {
        store: f.core.paths().projects_root().to_owned(),
        project_id: f.id.clone(),
    }
}

#[test]
fn review_controls_parse_only_representation_and_keep_typed_core_options() {
    assert_eq!(InspectorTab::default(), InspectorTab::Layer);
    let mut state = Review::default();
    assert!(state.include_audio);
    for invalid in [
        "",
        "-1",
        "1.5",
        "NaN",
        "inf",
        "18446744073709551616",
        "0; rm",
    ] {
        state.values[0] = invalid.into();
        assert!(state.request(false).is_err(), "{invalid}");
    }
    state.values[0] = u64::MAX.to_string();
    assert!(matches!(
        state.request(false).unwrap(),
        Request::Frame(u64::MAX)
    ));
    state.preset = PreviewPreset::P720;
    state.include_audio = false;
    state.values[1] = "2000".into();
    state.values[2] = "1000".into();
    let Request::Range(options) = state.request(true).unwrap() else {
        panic!("range")
    };
    // Reversed intervals remain core validation, not a second desktop rule.
    assert_eq!(options.start_ms, 2000);
    assert_eq!(options.end_ms, 1000);
    assert_eq!(options.include_audio, Some(false));
    assert!(options.fps.is_none());
}

#[test]
fn review_admission_failure_and_late_completion_preserve_authoritative_session() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let project = f.project();
    let before = fixture::inventory(root.path());
    let mut session = Session::default();
    let ticket = session.begin().unwrap();
    session.finish(ticket, Ok(project.clone()));
    let review_ticket = session.begin().unwrap();
    assert!(session.begin().is_none());
    assert!(!session.finish_review(ticket, Ok(())));
    assert!(session.busy);
    assert!(session.finish_review(
        review_ticket,
        Err(CoreError::new(ErrorCode::RevisionConflict, "stale"))
    ));
    assert!(session.needs_refresh);
    assert!(
        session
            .error
            .as_ref()
            .unwrap()
            .contains("REVISION_CONFLICT")
    );
    assert!(session.error.as_ref().unwrap().contains("retryable: true"));
    assert_eq!(
        serde_json::to_value(session.project.as_ref().unwrap()).unwrap(),
        serde_json::to_value(&project).unwrap()
    );
    assert_eq!(fixture::inventory(root.path()), before);
    let fresh = session.begin().unwrap();
    session.finish(fresh, Ok(project));
    assert!(!session.needs_refresh);
    let next = session.begin().unwrap();
    assert!(session.finish_review(next, Ok(())));
    assert!(session.error.is_none());
    assert!(!session.busy);
}

#[test]
fn review_delegates_missing_stale_invalid_range_and_backend_errors_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let startup = startup(&f);
    let project = f.project();
    let renderer = Renderer::new("/missing-review-ffmpeg", "/missing-review-ffprobe", None);
    let before = fixture::inventory(root.path());
    assert_eq!(
        review::execute(&startup, &renderer, project.revision - 1, Request::Frame(0))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        review::execute(
            &startup,
            &renderer,
            project.revision,
            Request::Frame(u64::MAX)
        )
        .unwrap_err()
        .code,
        ErrorCode::ValidationFailed
    );
    let state = Review {
        values: ["0".into(), "1000".into(), "500".into()],
        ..Default::default()
    };
    assert_eq!(
        review::execute(
            &startup,
            &renderer,
            project.revision,
            state.request(true).unwrap()
        )
        .unwrap_err()
        .code,
        ErrorCode::ValidationFailed
    );
    let expected = renderer.render_preview(&project, &f.dir(), 0).unwrap_err();
    let actual =
        review::execute(&startup, &renderer, project.revision, Request::Frame(0)).unwrap_err();
    assert_eq!(actual.code, expected.code);
    assert_eq!(actual.retryable, expected.retryable);
    let missing = Startup {
        project_id: "00000000-0000-4000-8000-000000000000".into(),
        ..startup.clone()
    };
    assert_eq!(
        review::execute(&missing, &renderer, project.revision, Request::Frame(0))
            .unwrap_err()
            .code,
        ErrorCode::ProjectNotFound
    );
    assert_eq!(fixture::inventory(root.path()), before);
}

#[test]
#[ignore = "configured actual FFmpeg/FFprobe desktop review evidence; run explicitly before archive"]
fn desktop_review_native_frame_audio_range_history_and_reopen() {
    let ffmpeg = std::env::var_os("OPENCUT_FFMPEG_PATH").expect("configured ffmpeg");
    let ffprobe = std::env::var_os("OPENCUT_FFPROBE_PATH").expect("configured ffprobe");
    let renderer = Renderer::new(ffmpeg, ffprobe.clone(), None);
    renderer.readiness().unwrap();
    let root = tempfile::tempdir().unwrap();
    let f = fixture::seed(&root.path().join("fixture"), true);
    let startup = startup(&f);
    let initial = f.project();
    assert!(initial.assets.iter().any(|asset| asset.id == f.asset));
    assert!(
        initial
            .assets
            .iter()
            .find(|asset| asset.id == f.plain_asset)
            .unwrap()
            .origin
            .is_none()
    );
    let authoritative = || {
        fixture::inventory(root.path())
            .into_iter()
            .filter(|(path, _)| !path.components().any(|part| part.as_os_str() == "previews"))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let before = authoritative();
    let frame =
        review::execute(&startup, &renderer, initial.revision, Request::Frame(650)).unwrap();
    assert_eq!(frame.rendered.mime_type, "image/png");
    assert_eq!(frame.dimensions, (64, 64));
    assert_eq!(frame.fps, 10);
    assert!(frame.rendered.size_bytes > 0);
    assert!(frame.path.starts_with(f.dir().join("previews")));
    let direct = renderer.render_preview(&initial, &f.dir(), 650).unwrap();
    assert_eq!(
        std::fs::read(&frame.path).unwrap(),
        std::fs::read(f.dir().join(direct.relative_path)).unwrap()
    );
    let mut state = Review {
        preset: PreviewPreset::Project,
        ..Default::default()
    };
    state.publish(frame);
    let range = review::execute(
        &startup,
        &renderer,
        initial.revision,
        state.request(true).unwrap(),
    )
    .unwrap();
    let output = std::process::Command::new(ffprobe)
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&range.path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let streams: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let streams = streams["streams"].as_array().unwrap();
    let video = streams
        .iter()
        .find(|stream| stream["codec_type"] == "video")
        .unwrap();
    assert_eq!(video["width"], 64);
    assert_eq!(video["height"], 64);
    assert_eq!(video["r_frame_rate"], "10/1");
    assert!(streams.iter().any(|stream| stream["codec_type"] == "audio"));
    assert_eq!(authoritative(), before);
    state.publish(range);
    assert_eq!(state.frame.as_ref().unwrap().revision, initial.revision);
    f.core
        .edit(
            &f.id,
            initial.revision,
            EditOperation::ItemSetZIndex {
                item_id: f.aliases["visual0"].clone(),
                z_index: 7,
            },
        )
        .unwrap();
    assert!(state.frame.as_ref().unwrap().stale(&f.project()));
    assert!(state.range.as_ref().unwrap().stale(&f.project()));
    let edited = f.project();
    let content = |project: &opencut_editor_core::Project| {
        let mut value = serde_json::to_value(project).unwrap();
        value.as_object_mut().unwrap().remove("revision");
        value.as_object_mut().unwrap().remove("updatedAtMs");
        value
    };
    f.core.undo(&f.id, f.project().revision).unwrap();
    assert_eq!(content(&f.project()), content(&initial));
    assert_eq!(f.project().revision, edited.revision + 1);
    assert!(f.project().updated_at_ms >= edited.updated_at_ms);
    f.core.redo(&f.id, f.project().revision).unwrap();
    assert_eq!(content(&f.project()), content(&edited));
    assert_eq!(f.project().revision, edited.revision + 2);
    assert!(f.project().updated_at_ms >= edited.updated_at_ms);
    let reopened = startup.execute(Command::Refresh).unwrap();
    assert_eq!(
        serde_json::to_value(&reopened).unwrap(),
        serde_json::to_value(f.project()).unwrap()
    );
    assert_eq!(state.frame.as_ref().unwrap().revision, initial.revision);
    assert_eq!(state.range.as_ref().unwrap().revision, initial.revision);
    assert_eq!(
        review::execute(&startup, &renderer, initial.revision, Request::Frame(650))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
}
