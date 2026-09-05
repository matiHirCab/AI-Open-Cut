# Verification: add-typed-template-slots

Verified against the approved proposal, design, five delta specifications and tasks on 2026-09-05. Implementation, automated conformance checks and finalization are complete. The user explicitly approved the completed implementation and public contract diff on 2026-09-05 in response to the designated CODEOWNER review request, following the earlier specification approval. All accepted deltas are synchronized and the change is archived.

## Completeness and correctness

All ten behavioral requirements have implementation and automated evidence. No unresolved implementation/specification mismatch was found. Core owns model, effective-value validation, atomic editing, media ownership and persistence; adapters expose typed inputs and translate core results. The architecture tests pass without adding private-owner dependencies. Rendering remains unchanged.

The following test names are in `crates/editor-core/tests/template_slots.rs` unless another file is specified.

| Approved scenario | Automated evidence |
| --- | --- |
| Round-trip all eight kinds | canonical_all_kinds_roundtrip_defaults_overrides_history_and_reopen; actual MCP component workflow |
| Validate absent and invalid values | required_optional_unknown_values_and_definition_replacement_are_atomic; canonical_invalid_and_closed_wire_records |
| Enforce inclusive limits | bounds_constraints_unicode_and_nonfinite_typed_values; aggregate_slot_and_text_limits_are_inclusive; rich_text_metadata_duration_and_override_bounds_cover_exact_endpoints |
| Count Unicode consistently | rich_text_metadata_duration_and_override_bounds_cover_exact_endpoints; bounds_constraints_unicode_and_nonfinite_typed_values; contracts.test.ts runtime slot fixture/closed-value test |
| Confine managed assets | slot_only_assets_are_retained_in_current_drafts_and_history; canonical_invalid_and_closed_wire_records |
| Resolve local identity and compatible properties | locks_scope_duplicate_writers_and_effective_domain_rules; canonical_all_kinds_roundtrip_defaults_overrides_history_and_reopen; core binding lookup uses local stable IDs, independent of track position |
| Reject invalid bindings and effective values | effective_asset_and_duration_are_validated_together_and_defaults_remain_validated; locks_scope_duplicate_writers_and_effective_domain_rules |
| Create and define slots by alias | apps/headless/tests/protocol.rs template_slots_standalone_and_alias_batches_have_typed_atomic_results; apps/agent-bridge/tests/component-workflow.ts |
| Reject stale, locked and partially valid edits | required_optional_unknown_values_and_definition_replacement_are_atomic; locks_scope_duplicate_writers_and_effective_domain_rules; actual MCP workflow |
| Migrate mixed retained history | schema11_nested_history_migrates_and_schema12_fields_are_required; components.rs all_supported_current_and_mixed_history_migrate_atomically |
| Fail closed and recover interruptions | corrupt_slot_values_in_current_history_and_drafts_never_publish; schema11_nested_history_migrates_and_schema12_fields_are_required; store.rs supported_migrations_recover_every_publication_phase now includes populated schema-11 nested definitions |
| Preserve slot-only media | slot_only_assets_are_retained_in_current_drafts_and_history |
| Preserve older requests | required_optional_unknown_values_and_definition_replacement_are_atomic; canonical component fixtures; contracts.test.ts schema-12 response/request-default test |
| Replace definitions with incoming slot references | required_optional_unknown_values_and_definition_replacement_are_atomic; effective_asset_and_duration_are_validated_together_and_defaults_remain_validated |
| Compare root output with stored slots | components.rs native_unused_definitions_preserve_frame_range_export_and_draft_output now uses slot-bearing definitions, run with actual FFmpeg |
| Reject malformed direct render data | invalid_unused_slots_fail_before_render_output_or_process_execution |
| Compare native and bridge evidence | canonical fixture Rust tests; apps/agent-bridge/tests/contracts.test.ts; real headless protocol test |
| Run real slot workflows | apps/agent-bridge/tests/component-workflow.ts shared by source smoke.test.ts and packaged-smoke.test.ts: all eight kinds, defaults, standalone definition, aliased batch, overrides, undo/redo/reopen |
| Propagate atomic slot failures | same source/packaged workflow covers malformed types, missing component, stale revision, locked target and later batch failure with unchanged state |

Existing target rules can be tighter than generic slot limits: text items retain their existing byte bound and nonempty requirement, enum alignment targets accept only three choices, and duration values must fit local/source intervals. No such restrictions are silently removed. Rich text remains a typed document; no plain-text rendering fallback was added.

## Validation results

- PASS `cargo fmt --check --all`.
- PASS `cargo clippy --workspace --all-targets -- -D warnings`.
- PASS `cargo test --workspace` after the exact-float parsing correction, including native golden, preview/range/export, migration fault-injection, architecture, retained-history and slot tests. The existing five ignored maintenance/helper tests retain their established status; subprocess helper tests execute through their parent tests. No required native conformance test was skipped in this run.
- PASS `bun run contracts:check` (typecheck, Rust headless tests including the new direct slot workflow, 16 TypeScript contract tests).
- PASS `bun run typecheck`, `bun run lint`, `bun run test` (80 tests, 14 files).
- PASS `bun run test:integration` (9 tests, including all eight slot kinds and locked bindings).
- PASS `bun run test:smoke` after rebuilding the final exact-float runtime (4 packaged tests).
- PASS `bun run scripts/run-python-tests.ts` (10 speech tests and 5 transcription tests).
- PASS `bunx @fission-ai/openspec@1.5.0 validate add-typed-template-slots --strict --no-interactive`.
- PASS `git diff --check`.
- PASS `moon run root:openspec-validate`, invoked through pinned `bunx @moonrepo/cli@2.3.3` after archival: workflow normalization, 231 policy tests, all 16 living specifications and the CI parity gate pass. The earlier expected active-change inventory block is resolved through approved archival without weakening policy.

Native tool configuration used existing local `target/ffmpeg6/extracted/ffmpeg-6.1.1-full_build/bin/ffmpeg.exe`, sibling ffprobe.exe, and `crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf` through the documented environment variables. Native workspace tests ran outside the Windows sandbox because the existing process-tree sampler needs process inspection. Initial sandbox sampler failure and attempts with incompatible system FFmpeg 9 / a different font were resolved through correct test tooling, without changing renderer behavior or goldens. Test logs under ignored target/ are local evidence, not committed artifacts.

## Review findings resolved

1. Updated the project response schema to 12 and required slotValues on returned nested instances while preserving request defaults.
2. Enabled serde_json float_roundtrip in editor-core after an exact safe-integer duration-constraint test revealed previously committed data could fail validation on reopen. The exact endpoint now passes repeated persistence reads.
3. Reused complete component validation for derived default/override candidates, including interacting asset/duration values, without mutating base tracks.
4. Added defaults and overrides to current/history/draft asset ownership, including overridden defaults and slot-only assets.

## Finalization

The designated contract review was explicitly approved in this task on 2026-09-05. All ten added requirements across template-slots, component-definitions, project-persistence, agent-bridge and motion-graphics-contracts were synchronized into living specifications. The verified change is archived at openspec/changes/archive/2026-09-05-add-typed-template-slots/. The final Moon gate passed with exit code 0. All 17 tasks are complete. No commits, pushes or issue-closing actions were performed.
