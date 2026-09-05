# Verification evidence

User approved the implementation artifacts on 2026-09-04. Canonical-contract CODEOWNER review of the resulting changes is still required.

## Scenario coverage

| Capability | Scenario | Automated evidence |
| --- | --- | --- |
| motion-graphics-architecture | Resolve overlapping visual layers | evaluated_scene::tests::explicit_stacking_respects_tracks_z_index_array_ties_and_hidden_sources (including synthesized ID ties) |
| motion-graphics-architecture | Preserve nonvisual semantics | evaluated_scene::tests::omits_hidden_visuals_and_resolves_audio_ducking; transition_facts_are_bounded_ordered_and_include_both_self_roles; renderer::tests::native_stacking_occlusion_and_render_intents_agree |
| motion-graphics-architecture | Reject complexity before ordering | evaluated_scene::tests::accepts_each_scene_limit_and_rejects_boundary_plus_one |
| motion-graphics-contracts | Discover and exercise stacking | headless protocol::stacking_public_protocol_and_batch_aliases; bridge smoke and packaged-smoke stacking tests |
| motion-graphics-contracts | Verify strict parity and compatibility | stacking::canonical_stacking_payloads_are_strict; bridge contracts.test.ts runtime stacking and complete MCP structural parity |
| motion-graphics-contracts | Reject unsafe fields | stacking::canonical_stacking_payloads_are_strict; bridge canonical fixture rejection; protocol unknown stacking field test |
| project-persistence | Migrate oldest and mixed history | stacking::schema_nine_requires_explicit_valid_order_and_migrates_mixed_history; transform2d::migrate_every_supported_version_and_mixed_history |
| project-persistence | Reopen and traverse migrated history | stacking::schema_nine_requires_explicit_valid_order_and_migrates_mixed_history |
| project-persistence | Reject malformed or future history | store::tests::invalid_retained_visual_properties_abort_migration_without_rewrite; dangling_retained_asset_reference_aborts_before_asset_publication; schema_zero_current_and_retained_state_are_rejected_without_publication; migrations::tests::rejects_future_retained_schema_without_mutating_any_document |
| project-persistence | Recover migration interruption | store::tests::schema_six_migration_recovers_every_publication_phase; schema_six_migration_before_journal_failure_preserves_generation (both now migrate through schema 9) |
| rendering-export | Compare rendered stacking | renderer::tests::native_stacking_occlusion_and_render_intents_agree; stacking::created_track_alias_move_and_draft_preserve_stacking |
| rendering-export | Preserve migrated legacy output | schema_nine_requires_explicit_valid_order_and_migrates_mixed_history compares exact migrated tracks; renderer native legacy preview/range/export and golden conformance tests |
| rendering-export | Reject unsupported complete scenes | renderer::tests::measured_transform_overflow_and_unavailable_backend_publish_nothing; existing all-facade readiness tests |
| timeline-editing | Preserve ordering through lifecycle edits | stacking_lifecycle_and_track_reorder_compatibility; created_track_alias_move_and_draft_preserve_stacking; store generated_audio_is_imported_and_inserted_atomically and transcription tests reopen through ordinal validation |
| timeline-editing | Reject malformed persisted ordering | stacking::schema_nine_requires_explicit_valid_order_and_migrates_mixed_history; canonical_stacking_payloads_are_strict; renderer::tests::malformed_stacking_is_rejected_without_side_effects_for_all_facades |
| timeline-editing | Reorder equal z-index items | stacking_aliases_revisions_rollback_and_history; native_stacking_occlusion_and_render_intents_agree |
| timeline-editing | Set signed z-index boundaries | stacking_aliases_revisions_rollback_and_history (i32 MIN); stacking_lifecycle_and_track_reorder_compatibility (i32 MAX); canonical fixtures (zero and negatives) |
| timeline-editing | Match legacy track reorder | stacking::stacking_lifecycle_and_track_reorder_compatibility |
| timeline-editing | Reject invalid targets and values | stacking_aliases_revisions_rollback_and_history; canonical_stacking_payloads_are_strict; transform2d::transition_and_audio_only_updates_are_rejected (both transform and stacking operations) |
| timeline-editing | Create and reorder by alias | stacking_aliases_revisions_rollback_and_history; created_track_alias_move_and_draft_preserve_stacking; protocol/MCP integration/packaged smoke |
| timeline-editing | Roll back a failing batch | stacking::stacking_aliases_revisions_rollback_and_history |
| timeline-editing | Undo redo and reopen stacking | stacking::stacking_aliases_revisions_rollback_and_history; schema_nine_requires_explicit_valid_order_and_migrates_mixed_history |

## Check results

- Rust formatting and strict workspace Clippy: passed.
- Workspace Rust tests with explicit FFmpeg 8.1.2, FFprobe 8.1.2, and checked-in DejaVuSans: passed on the final rerun (231 primary tests; log: local-data/stacking-workspace-tests.log). Five ignored tests are existing subprocess helper/maintenance entrypoints, not skipped scenario coverage.
- TypeScript typecheck, lint, and unit tests: passed (74 unit tests).
- Cross-language contracts: passed (headless unit/protocol tests and 10 TypeScript contract tests).
- MCP integration: passed (7 tests).
- Packaged smoke: passed (3 tests, compiled release sidecar and bridge).
- Native stacking regression: passed, covering z-index, array and track reordering, transparent Transform2D overlays, hidden items, captions, transitions, audio, independent occlusion assertions, SSIM >= 0.99, audio RMS <= 0.0001, and one-frame timing tolerance.
- Pinned OpenSpec strict validation: passed (15 specs/change entries).
- Pinned Moon 2.3.3 root:openspec-validate: 231 policy tests and all strict specs passed; final policy gate failed solely because deterministic-stacking is unarchived.
- Python workers are unchanged; provider-specific tests are not affected by this change.

## Environment observations

The PATH-installed FFmpeg 9.0.1 no longer accepts the existing filter_complex_script flag. Native checks use the already available local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin tools. No tool downloads, backend changes, or version-policy changes were made for this issue. Moon is not on PATH; the exact .prototools version was invoked with bunx @moonrepo/cli@2.3.3.

## Remaining gate

Designated @matiHirCab contract-owner review is pending. After that review, complete conformance verification, sync/archive the change, and rerun the Moon gate with archive-only inventory.

## Rendering-boundary correction verification

The user approved the corrective implementation plan, including the inward validation dependency. The new renderer regression failed before the fix because evaluate_project returned Ok for stackOrder 7. After the fix it passes for gaps, duplicates, swapped ordinals, hidden items/tracks, audio and transitions through preview, range and export. It checks the stable error/message, immutable decoded schema-9 input and no artifact publication; valid consecutive and empty fixtures also evaluate successfully. Persistence and evaluation share validation.rs::validate_project_stacking, called immediately after scene complexity preflight. ADR 0003 and both architecture checks include the approved edge.

Final correction checks:
- cargo fmt --check --all and cargo clippy --workspace --all-targets -- -D warnings: passed.
- cargo test --workspace -- --test-threads=1 with FFmpeg/FFprobe 8.1.2 and checked-in DejaVuSans: passed, 232 primary tests plus helper subprocess runs. Five existing helper/maintenance entrypoints remain ignored by default. Log: local-data/stacking-fix-workspace-final.log.
- The first full run found additional hand-built fixtures with default-zero ordinals, which were corrected at construction. It also hit the existing process-memory sampling assertion under concurrent load; that test passed in isolation and in the complete serial rerun, with no sampler changes.
- Bridge typecheck, lint, 74 unit tests, contracts:check (13 headless and 10 bridge contract tests), 7 integration tests and 3 packaged-smoke tests: passed.
- Strict pinned OpenSpec validation: all 15 entries passed.
- Pinned Moon root:openspec-validate: failed solely on the unarchived deterministic-stacking change after policy tests and strict specs passed. Log: local-data/stacking-fix-moon.log. Archive remains pending final contract-owner review.
- Python provider code/contracts are unchanged; hermetic provider tests are unaffected and were not rerun.

No remaining rendering-boundary mismatch was found. This corrects the earlier false conformance claim rather than relying on its passing tests.

## Explicit archival instruction

After PR #105 was created, the user instructed "archive the spec". Archival and synchronization proceed under that explicit instruction with contract-owner review still pending; task 4.3 stays open and the PR remains draft. This instruction is not recorded as contract approval.

Archive validation: pinned Moon 2.3.3 root:openspec-validate passed with 231 policy tests, 14 living specs and valid CI parity policy. Log: local-data/stacking-archive-moon.log. Eight requirements are synchronized across the five affected living specs.
