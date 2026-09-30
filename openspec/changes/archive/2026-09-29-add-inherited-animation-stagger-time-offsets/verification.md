# Verification: inherited animation, stagger, and copy offsets

## Approval and compatibility

The user explicitly approved the OpenSpec proposal and its additive contract scope in this chat before implementation. The designated owner in `.github/CODEOWNERS` is `@matiHirCab`; `gh api user --jq .login` returned `matiHirCab` in this session. This records the existing owner approval; repository pull-request review requirements still apply when this local work is submitted.

Optional public timing fields and the uniquely named capability are additive. Existing aliases, errors, retryability, transform2d conflicts and group creation defaults remain unchanged. Persisted schema 26 follows the established future-version rejection rule and migrates current plus retained history together. No Python protocol consumer owns these visual timing fields, and no provider code changed.

## Completeness, correctness, and coherence

All implementation belongs to the approved tasks. Editor-core owns models, source-version guards, bounds, edits, migration and occurrence evaluation. Transports submit typed input and expose the canonical catalogs. EvaluatedScene owns clocks, interval paths, inherited transform stages, preflight and independent audio clocks; render planning compiles those facts through the existing curve/loop expression compiler. No outer-layer domain validator or new dependency edge was introduced.

| Requirements/scenarios | Automated evidence |
| --- | --- |
| Governed timing fields, capability, malformed input, older clients | `inherited_animation_timing.rs`, bridge `inherited-animation-timing.test.ts`, exact registered MCP/catalog equality and `contracts:check`; existing group/repeater/component contract suites |
| Atomic schema migration, retained validation, future rejection and recovery | `schema_26_timing_migrates_history_and_rejects_old_field_injection`, raw pre-26 zero-field rejection, `supported_migrations_recover_every_publication_phase` and `supported_migration_before_journal_failure_preserves_generation` (both now include schema 25), existing persistence reopen/history/integrity suites |
| Standalone/batch/aliases, rollback, revisions, duplication, lock, undo/redo, draft and reopen | `timing_edits_are_alias_aware_atomic_and_reversible`, `component_timing_updates_duplicate_and_locked_tracks_are_atomic`, headless `inherited_timing_batch_round_trip_and_rollback`, shared MCP group workflow in integration and packaged smoke |
| Canonical child ranks, hidden stability, fractional nested clocks and parent phase | `group_stagger_delays_direct_visual_children_and_keeps_hidden_ranks`, `component_stagger_uses_definition_local_clock_after_time_scale`, `inherited_stagger_preserves_fractional_clocks_and_parent_phase`; existing typed curve/loop conformance and native scalar seam tests |
| Parent position/independent scales/opacity and shifted source subtrees | `native_inherited_timing_render_conformance` uses independent pixel/matrix/time oracles; nested fractional test checks each controller's own channel clock; existing ancestor matrix, curve/loop, transform conflict and channel-target tests |
| Signed copy clocks, intervals, ordinary source preservation and nesting | `signed_copy_offsets_shift_complete_component_source_clock`, `shifted_sources_can_enter_ancestor_clips_but_cannot_escape_them`, existing nested group/component/repeater/rich-text/identity conformance |
| Inclusive and excessive occurrence/fact/geometry limits, hidden/unused retained content, no materialization | `shifted_transition_budget_boundaries_preflight_before_copies`, `hidden_stagger_overflow_and_unused_animated_extent_fail_before_copies`, existing exact/one-over occurrence, resources, transition and hidden unused transform preflight tests |
| Shared frame/range/draft/export and independent audio | native audiovisual fixture: frame vs range vs export SSIM >= 0.99, color-channel tolerance 12/255, range/export decoded PCM RMS <= 0.0001; `staggered_nested_instance_preserves_audio_clock` |

## Check evidence

Full local logs are ignored files under `C:/Users/matia/Desktop/AI-Open-Cut/AI-Open-Cut/target/`.

| Command | Result | Log |
| --- | --- | --- |
| `cargo fmt --check --all` | exit 0 | `issue42-fmt.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 | `issue42-workspace-clippy.log` |
| `cargo test --workspace` | exit 0 | `issue42-workspace-tests.log` |
| `bun run typecheck` | exit 0 | `issue42-typecheck.log` |
| `bun run lint` | exit 0 | `issue42-lint.log` |
| `bun run test:unit` | exit 0, 421 passed, 1 optional real-provider test skipped | `issue42-unit.log` |
| `bun run contracts:check` | exit 0, all governed Rust/headless checks plus 354 TypeScript tests | `issue42-contracts.log` |
| `bun run test:integration` | exit 0, 13 tests | `issue42-integration.log` |
| `bun run test:smoke` | exit 0, 8 packaged tests | `issue42-packaged-smoke.log` |
| Hermetic environment Python `apps/kokoro-tts/test_worker.py` | exit 0, 10 tests | `issue42-python.log` |
| Native inherited audiovisual conformance | exit 0 | `issue42-native.log` |
| Strict all-spec validation | exit 0, 32 items | `issue42-openspec-strict.log` |

Rust's optional ignored benchmark/golden-maintenance tests remain opt-in. The affected native fixture was explicitly run with compatible FFmpeg/FFprobe 7.1.1 and the checked-in DejaVu Sans font. Initial native testing with the installed FFmpeg 8 failed because that binary removed the existing filter_complex_script option; the supported binary passed. The default Python interpreter initially lacked soundfile; the repository's existing hermetic test environment passed. Shared-junction dependencies initially caused Biome fixture diagnostics; installing the unchanged frozen lock into this worktree made lint pass. Those superseded failures were resolved without altering unrelated code or suppressing checks.

The final added preflight test was run separately after the workspace pass; it changes test-only input and passes. The unchanged implementation and all other passing tests retain their evidence. Formatting and workspace strict Clippy were rerun after adding it.

## Specification gates

`moon run root:openspec-validate` exited 1 before archival only because `add-inherited-animation-stagger-time-offsets` remained active; all policy tests and 32 strict specification items passed. Full log: `issue42-openspec-prearchive.log`. There was no unrelated rejection.

OpenSpec verify-change assessment: implementation tasks 1.1–3.3 and checks 4.1–4.3 are complete; all 11 changed requirements have automated scenario evidence in the table above, and all five design decisions match canonical code ownership and compatibility. No implementation/specification mismatch, critical issue, or unaddressed warning remains. Task 4.5 completed after synchronization, archival, and passing final gates.

All eight capability deltas were synchronized into living specifications, preserving unrelated requirements/scenarios. The verified change was archived to `openspec/changes/archive/2026-09-29-add-inherited-animation-stagger-time-offsets/`.

The final `moon run root:openspec-validate` exited 0: policy tests, 32 strict spec items, and the unchanged protected CI parity policy all passed. Full log: `issue42-openspec-final.log`. Separate strict all-spec validation also passed. All 17 tasks are complete; no required check or conformance issue remains blocked.
