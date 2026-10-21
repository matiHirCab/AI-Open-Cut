use super::*;
mod support {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/support/narration_blocks.rs"
    ));
}
use support::*;

#[test]
fn narration_fault_phases_never_acknowledge_a_partial_generation() {
    use crate::persistence::PersistencePhase;
    assert_eq!(fixture()["version"], 1);
    for phase in [
        PersistencePhase::BeforeJournal,
        PersistencePhase::AfterJournal,
        PersistencePhase::AfterProject,
        PersistencePhase::AfterHistory,
        PersistencePhase::AfterDraftUpdates,
        PersistencePhase::AfterDraftCleanup,
        PersistencePhase::AfterJournalCleanup,
        PersistencePhase::GarbageCollection,
    ] {
        let (root, core, profile) = setup();
        let input = commit_input(root.path(), &core, &profile);
        let project_id = input.project_id.clone();
        let block_ids = input
            .plan
            .blocks
            .iter()
            .map(|b| b.id.clone())
            .collect::<Vec<_>>();
        let before = inventory(root.path());
        core.persistence_faults.inject(phase);
        let result = core.commit_narration_blocks(input);
        if phase == PersistencePhase::BeforeJournal {
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::InternalError,
                "{phase:?}"
            );
            assert_eq!(inventory(root.path()), before, "{phase:?}");
        } else {
            let result = result.unwrap();
            assert_eq!(result.write.revision, 1, "{phase:?}");
            let state = core.get_project(&project_id).unwrap();
            assert_eq!(state.narration_manifests, [result.manifest]);
            assert_eq!(
                state.narration_manifests[0]
                    .blocks
                    .iter()
                    .map(|b| b.id.clone())
                    .collect::<Vec<_>>(),
                block_ids
            );
            assert!(state.assets.iter().all(|a| a.content_hash.is_some()));
        }
    }
}

#[test]
fn corrupt_manifest_journal_is_rejected_before_recovery_can_publish_any_document() {
    let (root, core, profile) = setup();
    let input = commit_input(root.path(), &core, &profile);
    let result = core.commit_narration_blocks(input).unwrap();
    let dir = core.project_directory(&result.write.project_id).unwrap();
    let mut candidate = core.get_project(&result.write.project_id).unwrap();
    candidate.revision += 1;
    candidate.narration_manifests[0].blocks[0].binding.item_id = "missing-native-item".into();
    let history: History = read_json(core.storage.as_ref(), &history_path(&dir)).unwrap();
    let transaction = crate::persistence::ProjectTransaction {
        version: 1,
        project: candidate,
        history,
        committed_draft_id: None,
        draft_updates: BTreeMap::new(),
    };
    std::fs::write(
        transaction_path(&dir),
        serde_json::to_vec(&transaction).unwrap(),
    )
    .unwrap();
    let before = inventory(root.path());
    assert_eq!(
        core.get_project(&result.write.project_id).unwrap_err().code,
        ErrorCode::ProjectRecoveryFailed
    );
    assert_eq!(inventory(root.path()), before);
}
