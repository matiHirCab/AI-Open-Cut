use opencut_editor_core::{
    AudioTrackRole, EditOperation, EditorCore, PathPolicy, ProjectSettings, TrackType,
};

#[test]
fn facade_and_persisted_shapes_survive_module_extraction() {
    let root = tempfile::tempdir().unwrap();
    let media = root.path().join("media");
    std::fs::create_dir_all(&media).unwrap();
    let policy = PathPolicy::new(
        root.path().join("projects"),
        [&media],
        root.path().join("exports"),
    )
    .unwrap();
    let core = EditorCore::new(policy.clone());
    let created = core
        .create_project("Compatibility", ProjectSettings::default())
        .unwrap();
    let draft = core
        .create_draft(
            &created.project_id,
            created.revision,
            vec![EditOperation::CreateTrack {
                name: "Extra overlay".into(),
                track_type: TrackType::Overlay,
                index: None,
                audio_role: AudioTrackRole::Unassigned,
                ducking: None,
            }],
            Some("compatible draft".into()),
        )
        .unwrap();

    let project_before =
        serde_json::to_value(core.get_project(&created.project_id).unwrap()).unwrap();
    let draft_value = serde_json::to_value(&draft).unwrap();
    assert_eq!(
        draft_value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "baseRevision",
            "createdAtMs",
            "id",
            "label",
            "operations",
            "projectId",
            "updatedAtMs",
            "version"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    let result_value = serde_json::to_value(&created).unwrap();
    assert_eq!(
        result_value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        [
            "aliases",
            "changedIds",
            "projectId",
            "revision",
            "summary",
            "warnings"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );

    let reopened = EditorCore::new(policy);
    let project_after =
        serde_json::to_value(reopened.get_project(&created.project_id).unwrap()).unwrap();
    assert_eq!(project_after, project_before);
    assert_eq!(
        reopened
            .get_draft(&created.project_id, &draft.id)
            .unwrap()
            .base_revision,
        0
    );
}
