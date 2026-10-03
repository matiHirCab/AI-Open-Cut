# Verification: fix-preset-migration-atomicity

Implementation conformance is verified on 2026-10-02. The approved transactional delta has been synchronized and archived; the protected postarchive Moon gate and final strict validation passed. No GitHub publication, review/comment, merge or deployment occurred.

## Completeness and scope

The single modified normative requirement, Transactional preset edits, has eight automated scenarios. All eleven implementation/evidence tasks are complete after the recorded commit/dry-run verification. Independent reviewer correction_review (gpt-6-sol, medium) accepted the concrete restoration proposal before code, accepted the amended speculative asset staging design before that amendment, and found no remaining correctness or ADR blocker in final executable source c454dee2.

This restores the existing failure guarantee. No request/response/catalog/error shape, schema29, compilerVersion1, preset identifier/parameter semantics, renderer tolerance or dependency edge changes. Public contract fixtures/catalogs and owning compiler/renderer/scene/cache logic are unchanged. There is no fresh governed contract gate for this correction. Issue46 remains a separate planning-only change with explicit user design approval recorded on 2026-10-02 at21:19:22UTC, still requiring designated CODEOWNER acceptance for its new public/persisted contracts, and parent publication confirmation. Issue45/PR134 remains unmerged.

## Requirement and scenario traceability

| Scenario | Implementation / automated evidence |
| --- | --- |
| Apply to earlier creation alias | store::edit_presets and apply_edit_batch; animation_presets integration alias/undo/redo cases; actual headless protocol regression |
| Roll back earlier valid operations | complete candidate application/preflight before journal; final_scene_budget_failure_rolls_back_preset_alias_and_existing_draft exercises schema28+29, existing drafts and aliases; rejected_legacy_presets_stage_assets_fonts_history_and_drafts_without_writes |
| Preserve stale/missing/lock/alias errors | loader checks revision before staging; existing operation owners unchanged; locks_alias_misuse_and_batch_limits_publish_nothing plus legacy stale/missing resource fixture cases |
| Reject before publishing legacy migration | prepare_project_data with request-wide rollback ledger; legacy_invalid_preset_does_not_publish_schema_migration, legacy_failed_alias_batch_does_not_publish_schema_migration and actual headless rejected_legacy_preset_requests_preserve_wire_errors_and_persisted_generation |
| Accept migration and edit together | one persist_transaction_with_drafts; schema18 fixture includes components, both history stacks, v1 draft, media and fonts; preset_migration_publication_preserves_precommit_and_recovery_semantics; headless success/undo/redo/reopen |
| Publication faults | nine checkpoints; preset_resource_copy_and_font_write_errors_remove_only_uncommitted_bytes verifies copy corruption/font write failure; preset_journal_sync_error_after_rename_keeps_committed_resources_for_recovery verifies uncertain journal durability and reopen |
| Preexisting destination entries | UncommittedResources::track_absent uses entry_kind; preset_rollback_preserves_dangling_asset_and_font_destination_links preserves link targets and byte maps. Destination scenario explicitly applies when staging/publication reaches that destination, preserving earlier error priority |
| Migration asset font selection | verified speculative copies at the original loader point, before font preparation; preset_asset_staging_preserves_explicit_and_default_font_selection_failures checks DEPENDENCY_UNAVAILABLE and byte preservation; actual headless future-font probe confirms eager/direct error parity and rollback |

Assets/font helpers stay within existing owners. Ordinary reads and non-preset operations retain eager asset migration; preset edits stage documents and fonts, apply once, certify once, bump once, push one undo entry and commit through the existing journal. Journal recovery occurs before preparation, while only this request's persistence-call journal ambiguity becomes the existing recovery-pending warning. Resources are retained after possible journal rename and removed only on precommit failure. Existing files and links never enter the rollback ledger.

Prior array/duplicate-field/duplicate-provenance regressions, preset provenance retirement, replacement order/collisions, unknown historical versions, current/components/retained migration, and preview/export primitive parity remain covered by passing core/transport/contract suites. No compiler/catalog lookup is introduced in rendering or reopening.

## Commands and results

Full uncommitted logs are in /tmp/review-logs and the recoverable issue45-correction-evidence.tar.gz artifact. No full logs are committed. Final executable inputs are c454dee2; subsequent checkpoint/spec/delivery commits are documentation only, verified by an empty diff over crates, apps, contracts, Cargo.toml and Cargo.lock.

- cargo fmt --all --check: exit0.
- cargo clippy --workspace --all-targets -- -D warnings: exit0.
- cargo test --workspace: exit0;856 passed,9 default-ignored benchmarks. Default opt-in returns are not claimed as native evidence.
- Bridge typecheck/lint: exit0; unit431 passed,1 optional native skip; hermetic Python workers10+5 passed. These passes are reused because their source/fixture/toolchain inputs are unchanged.
- bun run contracts:check: exit0; Rust222 and TypeScript361 passed.
- bun run test:integration: final-source exit0,15 passed.
- bun run test:smoke: final-source exit0,9 passed, unchanged timeouts.
- Fresh required preset native tests:2 passed; fresh headless native lifecycle:1 passed.
- Fresh required release native_golden_render_conformance:1 passed,1031.94sec; external report validator1 passed. Timings are report-only under concurrent local render load.
- Required native transform/channel/font cases, cache core13/worker3/bridge5 and default-headless restoration passed in the driver. Their compiler/renderer/scene/cache/fixture inputs and non-preset behavior were unchanged by the later preset-only staging amendment; repeated preset native cases were independently rerun on final source.
- Final-source required PR rules matrix:960x540 all5states/25operations PASS3679.47sec;1280x720 all5states/25operations PASS4807.72sec;1920x1080 all5semanticstates/6operations PASS1660.31sec. No required shard skipped. Driver21152 terminal exit0; fresh960 session66882 terminal exit0.
- Strict all-spec validation:37 passed,0 failed before archival.
- Protected prearchive moon run root:openspec-validate: expected exit1 solely for active fix-preset-migration-atomicity; strict37 passed and no other policy error. This is not a passed protected gate.
- Actual isolated commits, verified incremental bundle and git push --dry-run succeeded. No actual push.

Initial MCP two timeout failures and packaged smoke one timeout occurred during competing full Rust suites/release builds; unchanged reruns passed. Resource contention is an inference, not a proven cause. Initial05dc strict Clippy failure needless_option_as_deref was corrected by c454 and the final strict run passed. Original failure logs are retained.

## Evidence limits

Local heavy checks use a Linux subreaper wrapper to adopt/reap descendants because this container's PID1 does not reap exited render-worker children. Test assertions/product behavior are unchanged; do not claim bare local POSIX containment passed. No new Windows/macOS execution was performed. Nine default-ignored benchmarks are not claimed as passed; the explicitly requested external report helper was executed separately.

Cleanup itself can fail under a refusing filesystem. The correction preserves the primary typed error and appends the cleanup diagnostic; newly created unreferenced files could remain if removal fails. Preexisting resources/documents remain protected. Cleanup-refusal behavior was reviewed statically, not independently fault-injected. This is the documented storage limitation, not a claim of unconditional physical rollback under failed I/O.

GitHub run37036792797 succeeded with11 jobs at original67d05db only. Its foundation elapsed61.29min met default120min without exception. No correction-head CI exists; no correction-head GitHub budget pass is claimed. Optional weekly9/25 cancellation is not a pass.

## Delivery status

Conformance: PASS (11 implementation/evidence tasks,1 requirement,8 scenarios; no remaining implementation mismatch).
Synchronization/archival: complete; archive2026-10-02-fix-preset-migration-atomicity.
Protected postarchive Moon gate: PASS, exit0, CI parity gate policy valid. Final strict all-spec validation: PASS,36 living items,0 failed (the active correction item is now archived).
Local correction verification and archival are complete. PR134 is unchanged/unmerged; correction-head GitHub CI remains unavailable until publication. Issue46 has explicit design approval; implementation starts from this corrected base, with CODEOWNER acceptance and separate publication gates still required.
