use opencut_editor_core::{
    EditOperation, EditorCore, ErrorCode, FontConfig, PathPolicy, ProjectSettings, TimelineItem,
};
use serde_json::json;

#[test]
fn unsafe_selectors_and_missing_styles_preserve_authoritative_bytes() {
    let (root, core, id, track) = setup();
    let dir = core.project_directory(&id).unwrap();
    let before = std::fs::read(dir.join("project.json")).unwrap();
    for (path, code) in [
        ("../font.ttf", ErrorCode::PathTraversal),
        (
            "https://example.invalid/font.ttf",
            ErrorCode::PathNotAllowed,
        ),
        ("//server/share/font.ttf", ErrorCode::PathNotAllowed),
    ] {
        let mut edit = serde_json::to_value(add(&track)).unwrap();
        edit["fontPath"] = json!(path);
        assert_eq!(
            core.edit(&id, 0, serde_json::from_value(edit).unwrap())
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    }
    std::fs::remove_file(root.path().join("sources/DejaVuSans-Bold.ttf")).unwrap();
    assert_eq!(
        core.edit(&id, 0, add(&track)).unwrap_err().code,
        ErrorCode::DependencyUnavailable
    );
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), before);
    assert!(!dir.join("fonts").exists());
}

#[test]
fn historical_fields_and_native_missing_bindings_fail_without_rewrite() {
    let (_root, core, id, track) = setup();
    core.edit(&id, 0, add(&track)).unwrap();
    let dir = core.project_directory(&id).unwrap();
    let original = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    for mode in 0..5 {
        let mut invalid = original.clone();
        match mode {
            0 => invalid["schemaVersion"] = json!(18),
            1 => {
                invalid["tracks"][1]["items"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("fontBinding");
            }
            2 => invalid["tracks"][1]["items"][0]["fontBinding"]["profile"] = json!("future"),
            3 => invalid["schemaVersion"] = json!(20),
            _ => {
                invalid["schemaVersion"] = json!(18);
                invalid["fonts"] = serde_json::Value::Null;
            }
        }
        let bytes = serde_json::to_vec(&invalid).unwrap();
        std::fs::write(dir.join("project.json"), &bytes).unwrap();
        assert!(core.get_project(&id).is_err());
        assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
    }
}

#[test]
fn stale_legacy_edit_does_not_activate_fonts() {
    let (_root, core, id, track) = setup();
    let dir = core.project_directory(&id).unwrap();
    let mut legacy = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    legacy["schemaVersion"] = json!(18);
    legacy.as_object_mut().unwrap().remove("fonts");
    let bytes = serde_json::to_vec(&legacy).unwrap();
    std::fs::write(dir.join("project.json"), &bytes).unwrap();
    assert_eq!(
        core.edit(&id, 1, add(&track)).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(std::fs::read(dir.join("project.json")).unwrap(), bytes);
    assert!(!dir.join("fonts").exists());
}

#[test]
fn legacy_draft_rejects_explicit_null_font_fields() {
    let (_root, core, id, track) = setup();
    let draft = core.create_draft(&id, 0, vec![add(&track)], None).unwrap();
    let mut invalid = serde_json::to_value(&draft).unwrap();
    invalid["version"] = json!(1);
    invalid["fontCatalog"] = serde_json::Value::Null;
    invalid.as_object_mut().unwrap().remove("fontSteps");
    let path = core
        .project_directory(&id)
        .unwrap()
        .join("drafts")
        .join(format!("{}.json", draft.id));
    let bytes = serde_json::to_vec(&invalid).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    assert!(core.get_draft(&id, &draft.id).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn component_replacement_preserves_bindings_until_selector_changes() {
    let (root, core, id, track) = setup();
    core.edit(&id, 0, add(&track)).unwrap();
    let mut text =
        serde_json::to_value(&core.get_project(&id).unwrap().tracks[1].items[0]).unwrap();
    text["id"] = json!("local-text");
    text.as_object_mut().unwrap().remove("fontBinding");
    let create = json!({"operation":"component_create","name":"Text component","width":320,"height":180,"durationMs":1000,"tracks":[{"id":"local","name":"Local","trackType":"overlay","items":[text]}],"slots":[]});
    let created = core
        .edit(&id, 1, serde_json::from_value(create).unwrap())
        .unwrap();
    let before = core.get_project(&id).unwrap().components[0].clone();
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::remove_file(root.path().join("sources").join(name)).unwrap();
    }
    let mut update = serde_json::to_value(&before).unwrap();
    update.as_object_mut().unwrap().remove("id");
    update["componentId"] = json!(before.id);
    update["operation"] = json!("component_update");
    update["tracks"][0]["items"][0]
        .as_object_mut()
        .unwrap()
        .remove("fontBinding");
    update["tracks"][0]["items"][0]["color"] = json!("#ff0000");
    let edited = core
        .edit(
            &id,
            created.revision,
            serde_json::from_value(update.clone()).unwrap(),
        )
        .unwrap();
    let after = core.get_project(&id).unwrap();
    let TimelineItem::Text(original) = &before.tracks[0].items[0] else {
        panic!()
    };
    let TimelineItem::Text(retained) = &after.components[0].tracks[0].items[0] else {
        panic!()
    };
    assert_eq!(retained.font_binding, original.font_binding);
    update["tracks"][0]["items"][0]["fontFamily"] = json!("missing family");
    core.edit(
        &id,
        edited.revision,
        serde_json::from_value(update).unwrap(),
    )
    .unwrap();
    let after = core.get_project(&id).unwrap();
    let TimelineItem::Text(rebound) = &after.components[0].tracks[0].items[0] else {
        panic!()
    };
    assert_eq!(rebound.font_binding.as_ref().unwrap().warnings.len(), 1);
}

#[test]
fn explicit_reset_resolves_while_paint_updates_retain_binding() {
    let (_root, core, id, track) = setup();
    let added = core.edit(&id, 0, add(&track)).unwrap();
    let item = &added.changed_ids[0];
    let before = core.get_project(&id).unwrap();
    let paint =
        serde_json::from_value(json!({"operation":"update_item","itemId":item,"color":"#ff0000"}))
            .unwrap();
    let updated = core.edit(&id, added.revision, paint).unwrap();
    assert_eq!(core.get_project(&id).unwrap().fonts, before.fonts);
    let reset = serde_json::from_value(
        json!({"operation":"update_item","itemId":item,"fontFamily":"absent family"}),
    )
    .unwrap();
    core.edit(&id, updated.revision, reset).unwrap();
    let project = core.get_project(&id).unwrap();
    let TimelineItem::Text(text) = project.find_item(item).unwrap() else {
        panic!()
    };
    assert_eq!(text.font_binding.as_ref().unwrap().warnings.len(), 1);
}

#[test]
fn hidden_font_integrity_fails_before_render_artifacts() {
    let (_root, core, id, track) = setup();
    let added = core.edit(&id, 0, add(&track)).unwrap();
    let mut project = core.get_project(&id).unwrap();
    assert_eq!(project.tracks[1].items[0].id(), added.changed_ids[0]);
    project.tracks[1].items[0].visual_properties_mut().hidden = true;
    let dir = core.project_directory(&id).unwrap();
    let face = project.fonts.values().next().unwrap();
    std::fs::remove_file(dir.join(&face.relative_path)).unwrap();
    let renderer =
        opencut_editor_core::Renderer::new("unavailable-ffmpeg", "unavailable-ffprobe", None);
    assert_eq!(
        renderer.render_preview(&project, &dir, 0).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
    assert!(
        !dir.join("previews").exists()
            || std::fs::read_dir(dir.join("previews"))
                .unwrap()
                .next()
                .is_none()
    );
    assert!(std::fs::read_dir(&dir).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".opencut-work-")
    }));
}

#[test]
fn discarded_draft_releases_only_unowned_font_content() {
    let (_root, core, id, track) = setup();
    let draft = core.create_draft(&id, 0, vec![add(&track)], None).unwrap();
    let dir = core.project_directory(&id).unwrap();
    assert_eq!(std::fs::read_dir(dir.join("fonts")).unwrap().count(), 4);
    core.discard_draft(&id, &draft.id).unwrap();
    // Collection follows the existing successful project-commit policy.
    core.edit(
        &id,
        0,
        serde_json::from_value(
            json!({"operation":"create_track","name":"GC","trackType":"overlay"}),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(std::fs::read_dir(dir.join("fonts")).unwrap().count(), 0);
}

fn setup() -> (tempfile::TempDir, EditorCore, String, String) {
    let root = tempfile::tempdir().unwrap();
    let fonts = root.path().join("sources");
    std::fs::create_dir(&fonts).unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/fonts");
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::copy(source.join(name), fonts.join(name)).unwrap();
    }
    let paths = PathPolicy::new(
        root.path().join("projects"),
        [&fonts],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(paths).with_font_config(FontConfig {
        roots: vec![fonts],
        default_path: None,
    });
    let id = core
        .create_project("Fonts", ProjectSettings::default())
        .unwrap()
        .project_id;
    let track = core.get_project(&id).unwrap().tracks[1].id.clone();
    (root, core, id, track)
}

fn add(track: &str) -> EditOperation {
    serde_json::from_value(json!({"operation":"add_text", "trackId":track, "text":"office AV é", "fontFamily":"DejaVu Sans", "fontSize":48, "color":"#ffffff", "startMs":0, "durationMs":1000, "transform":{"positionX":0,"positionY":0,"scale":1,"opacity":1}})).unwrap()
}

#[test]
fn pinned_faces_survive_source_removal_and_history() {
    let (root, core, id, track) = setup();
    let revision = core.get_project(&id).unwrap().revision;
    let added = core.edit(&id, revision, add(&track)).unwrap();
    let before = core.get_project(&id).unwrap();
    assert_eq!(before.fonts.len(), 4);
    let TimelineItem::Text(text) = before.find_item(&added.changed_ids[0]).unwrap() else {
        panic!()
    };
    let binding = text.font_binding.clone().unwrap();
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::remove_file(root.path().join("sources").join(name)).unwrap();
    }
    let reopened = EditorCore::new(core.paths().clone());
    let after = reopened.get_project(&id).unwrap();
    assert_eq!(after.fonts, before.fonts);
    let undone = reopened.undo(&id, added.revision).unwrap();
    reopened.redo(&id, undone.revision).unwrap();
    let restored = reopened.get_project(&id).unwrap();
    let TimelineItem::Text(text) = restored.find_item(&added.changed_ids[0]).unwrap() else {
        panic!()
    };
    assert_eq!(text.font_binding.as_ref(), Some(&binding));
}

#[test]
fn tampered_managed_font_fails_closed() {
    let (_root, core, id, track) = setup();
    core.edit(&id, core.get_project(&id).unwrap().revision, add(&track))
        .unwrap();
    let project = core.get_project(&id).unwrap();
    let face = project.fonts.values().next().unwrap();
    std::fs::write(
        core.project_directory(&id)
            .unwrap()
            .join(&face.relative_path),
        b"changed",
    )
    .unwrap();
    assert_eq!(
        core.get_project(&id).unwrap_err().code,
        ErrorCode::AssetIntegrityFailed
    );
}

#[test]
fn invalid_batch_and_stale_revision_publish_no_binding() {
    let (_root, core, id, track) = setup();
    let before = core.get_project(&id).unwrap();
    let bad: EditOperation =
        serde_json::from_value(json!({"operation":"update_item", "itemId":"missing", "text":"x"}))
            .unwrap();
    assert_eq!(
        core.edit_batch(&id, before.revision, vec![add(&track), bad])
            .unwrap_err()
            .code,
        ErrorCode::ItemNotFound
    );
    assert_eq!(
        core.edit(&id, before.revision + 1, add(&track))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    let after = core.get_project(&id).unwrap();
    assert_eq!(after.revision, before.revision);
    assert!(after.fonts.is_empty());
    assert!(!core.project_directory(&id).unwrap().join("fonts").exists());
}

#[test]
fn schema_18_migration_pins_current_and_retained_history() {
    let (_root, core, id, track) = setup();
    core.edit(&id, core.get_project(&id).unwrap().revision, add(&track))
        .unwrap();
    let dir = core.project_directory(&id).unwrap();
    let mut project = serde_json::to_value(core.get_project(&id).unwrap()).unwrap();
    fn legacy(value: &mut serde_json::Value) {
        value["schemaVersion"] = json!(18);
        value.as_object_mut().unwrap().remove("fonts");
        for track in value["tracks"].as_array_mut().unwrap() {
            for item in track["items"].as_array_mut().unwrap() {
                item.as_object_mut().unwrap().remove("fontBinding");
            }
        }
    }
    legacy(&mut project);
    std::fs::write(
        dir.join("project.json"),
        serde_json::to_vec(&project).unwrap(),
    )
    .unwrap();
    let mut history: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("history.json")).unwrap()).unwrap();
    for key in ["undo", "redo"] {
        for snapshot in history[key].as_array_mut().unwrap() {
            legacy(snapshot);
        }
    }
    std::fs::write(
        dir.join("history.json"),
        serde_json::to_vec(&history).unwrap(),
    )
    .unwrap();
    let migrated = core.get_project(&id).unwrap();
    assert_eq!(migrated.schema_version, 19);
    assert_eq!(migrated.fonts.len(), 4);
    let bytes = std::fs::read(dir.join("project.json")).unwrap();
    core.get_project(&id).unwrap();
    assert_eq!(bytes, std::fs::read(dir.join("project.json")).unwrap());
}

#[test]
fn native_preview_uses_pinned_glyphs_after_reopen_and_source_removal() {
    native_pinned_text(None);
}

#[test]
fn native_mandatory_separators_agree_across_render_intents() {
    native_pinned_text(Some("AV\u{2028}אב\u{2029}ffi\r\nZ"));
}

fn native_pinned_text(text: Option<&str>) {
    use opencut_editor_core::{ExportOptions, PreviewRangeOptions};
    let (root, core, id, track) = setup();
    let mut operation = serde_json::to_value(add(&track)).unwrap();
    if let Some(text) = text {
        operation["text"] = json!(text);
    }
    core.edit(
        &id,
        core.get_project(&id).unwrap().revision,
        serde_json::from_value(operation).unwrap(),
    )
    .unwrap();
    let renderer = opencut_editor_core::Renderer::new(
        std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into()),
        std::env::var_os("OPENCUT_FFPROBE_PATH").unwrap_or_else(|| "ffprobe".into()),
        None,
    );
    let dir = core.project_directory(&id).unwrap();
    let before = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 0)
        .unwrap();
    let image = std::fs::read(dir.join(before.relative_path)).unwrap();
    let project = core.get_project(&id).unwrap();
    if let Some(text) = text {
        let mut oracle = project.clone();
        let TimelineItem::Text(item) = &mut oracle.tracks[1].items[0] else {
            panic!("expected text");
        };
        item.text = text
            .replace("\r\n", "\n")
            .replace(['\u{2028}', '\u{2029}'], "\n");
        item.document = opencut_editor_core::RichTextDocument::plain(item.text.clone());
        let expected = renderer.render_preview(&oracle, &dir, 0).unwrap();
        assert_eq!(
            image,
            std::fs::read(dir.join(expected.relative_path)).unwrap()
        );
    }
    let draft = core.create_draft(&id, project.revision, vec![serde_json::from_value(json!({"operation":"update_item","itemId":project.tracks[1].items[0].id(),"color":"#ffffff"})).unwrap()], None).unwrap();
    let render_video = |project: &opencut_editor_core::Project, suffix: &str| {
        let range = renderer
            .render_preview_range(
                project,
                &dir,
                PreviewRangeOptions {
                    start_ms: 0,
                    end_ms: 1000,
                    width: project.settings.width,
                    height: project.settings.height,
                    fps: project.settings.fps,
                    include_audio: false,
                },
                |_| {},
            )
            .unwrap();
        let export = root.path().join(format!("{suffix}.mp4"));
        renderer
            .export_video(
                project,
                &dir,
                ExportOptions {
                    output: &export,
                    width: project.settings.width,
                    height: project.settings.height,
                    overwrite: false,
                },
                |_| {},
            )
            .unwrap();
        [dir.join(range.relative_path), export].map(|path| {
            let output = std::process::Command::new(
                std::env::var_os("OPENCUT_FFMPEG_PATH").unwrap_or_else(|| "ffmpeg".into()),
            )
            .args(["-v", "error", "-i"])
            .arg(path)
            .args([
                "-frames:v",
                "1",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ])
            .output()
            .unwrap();
            assert!(output.status.success());
            assert!(
                output
                    .stdout
                    .iter()
                    .filter(|channel| **channel > 180)
                    .count()
                    > 100,
                "glyphs must visibly render"
            );
            output.stdout
        })
    };
    let baseline = render_video(&project, "before");
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::remove_file(root.path().join("sources").join(name)).unwrap();
    }
    let after = renderer
        .render_preview(&core.get_project(&id).unwrap(), &dir, 0)
        .unwrap();
    assert_eq!(image, std::fs::read(dir.join(after.relative_path)).unwrap());
    let reopened = EditorCore::new(core.paths().clone());
    assert_eq!(
        baseline,
        render_video(&reopened.get_project(&id).unwrap(), "after")
    );
    let candidate = reopened.get_draft_state(&id, &draft.id).unwrap().project;
    let preview = renderer.render_preview(&candidate, &dir, 0).unwrap();
    assert_eq!(
        image,
        std::fs::read(dir.join(preview.relative_path)).unwrap()
    );
    assert_eq!(
        reopened.get_project(&id).unwrap().revision,
        project.revision
    );
}

#[test]
fn draft_only_fonts_survive_reopen_and_commit_without_sources() {
    let (root, core, id, track) = setup();
    let before = core.get_project(&id).unwrap();
    let draft = core
        .create_draft(&id, before.revision, vec![add(&track)], None)
        .unwrap();
    assert_eq!(draft.font_catalog.as_ref().unwrap().len(), 4);
    for name in [
        "DejaVuSans.ttf",
        "DejaVuSans-Bold.ttf",
        "DejaVuSans-Oblique.ttf",
        "DejaVuSans-BoldOblique.ttf",
    ] {
        std::fs::remove_file(root.path().join("sources").join(name)).unwrap();
    }
    let candidate = core.get_draft_state(&id, &draft.id).unwrap();
    assert_eq!(
        candidate.project.fonts,
        *draft.font_catalog.as_ref().unwrap()
    );
    assert_eq!(core.get_project(&id).unwrap().revision, before.revision);
    core.commit_draft(&id, &draft.id, before.revision).unwrap();
    assert_eq!(
        core.get_project(&id).unwrap().fonts,
        candidate.project.fonts
    );
}
