> Correction: subsequent review found rotated-media clipping, late text preflight, and host-dependent tests. The approved [transform2d-review-fixes follow-up](../2026-09-04-transform2d-review-fixes/verification.md) supersedes the completion assessment below and provides corrected verification evidence.

# Transform2D verification — 2026-09-04

## Assessment

Completeness: all 17 tasks complete; approved deltas synchronized and change archived. Correctness: all 12 added requirements mapped to automated evidence. Coherence: core retains domain ownership, transports reuse typed operations, and no module dependency edge was added. No unresolved implementation mismatch.

The user approved the governed contract proposal and tasks on 2026-09-04 and explicitly approved the measurement amendment. These approvals cover the planned contract decisions; no separate post-implementation GitHub review is claimed. Canonical catalogs were manually synchronized, and parity tests only read them.

## Requirement and scenario trace

Paths below are repository-relative. T denotes crates/editor-core/tests/transform2d.rs; E denotes crates/editor-core/src/evaluated_scene.rs; R denotes crates/editor-core/src/renderer.rs; S denotes crates/editor-core/src/store.rs; H denotes apps/headless/tests/protocol.rs.

| Requirement | Scenarios and automated evidence |
| --- | --- |
| Typed static Transform2D updates | Set/clear and legacy switching: T alias_batch_reset_legacy_switch_history_and_atomic_failures. Conflicting/incomplete payloads: same test plus canonical_transform_payloads and H transform2d_round_trips_and_resets_through_public_protocol. Unsupported targets: T transition_and_audio_only_updates_are_rejected. |
| Bounded Transform2D values | Every numeric boundary and nonfinite input: T every_numeric_bound_and_nonfinite_value and canonical_transform_payloads. Incompatible animation in both mutation directions: T incompatible_animation_and_failed_batch_preserve_state. |
| Transform2D transactional semantics | Alias creation, changed IDs, one revision, rollback, missing item, stale revision, undo/redo/reopen: T alias_batch_reset_legacy_switch_history_and_atomic_failures; lock/split/duplicate: T split_duplicate_and_lock_preserve_transform_rules. Existing store transactional tests cover shared history and publication paths. |
| Canonical Transform2D affine evaluation | Independent sequential coordinate oracle, noncentral anchor, both skews/scales, rotation and equivalent units: E affine_tests::independent_sequential_oracle_and_units. Immutable ordered evaluation: E evaluates_owned_flat_layers_in_stable_order_without_mutating_project; all-intent shared lowering: T all_visual_sources_share_affine_preview_range_and_export and H native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts. |
| Bounded affine evaluation | Exact dimensions/area, overflow, nonfinite results: E affine_tests::geometry_boundaries_are_checked_before_clipping. Missing asset precedes transformed geometry: E missing_non_finite_and_invalid_timing_fail_closed and rejects_missing_assets_before_scene_complexity. No allocation/write/execution on measured overflow: R measured_transform_overflow_and_unavailable_backend_publish_nothing. |
| Read-only text measurement precedes affine finalization | Different selected fonts produce different measured anchors without writes: R selected_font_metrics_determine_affine_anchor_before_writes. Measurement overflow has no workspace or artifact: R measured_transform_overflow_and_unavailable_backend_publish_nothing. Prepared rendering reuses measured layout/font. |
| Schema-v8 Transform2D migration | All versions 1–7 and mixed retained history: T migrate_every_supported_version_and_mixed_history. Reopen/undo/redo: T alias_batch_reset_legacy_switch_history_and_atomic_failures. Legacy field preservation and defaults: S schema_six_common_visual_defaults_migrate_current_and_retained_history, schema_one_projects_and_history_are_migrated_before_persisting. |
| Schema-v8 migration fails atomically | Invalid transformed history remains byte-identical: T invalid_transform_in_retained_history_is_never_published. Invalid references/future/zero versions: S existing load-validation and unsupported_project_schema_versions_are_rejected_without_rewrite tests. Every publication phase/recovery: S schema_six_migration_recovers_every_publication_phase, schema_six_migration_before_journal_failure_preserves_generation, persistence_transaction_recovers_every_publication_phase. Optional omission stays absent in T migration tests and existing current-schema loading. |
| Governed runtime Transform2D contract | Rust/TypeScript read contracts/transform2d-v1.json valid/invalid cases: T canonical_transform_payloads and apps/agent-bridge/tests/transform2d.test.ts. Discovery, version, structural MCP parity and legacy requests: H protocol tests and bridge contracts.test.ts. Remaining roadmap fixture status is unchanged. MCP smoke and packaged-smoke exercise alias update, invalid scale, undo, redo, reset. |
| Shared complete Transform2D rendering | All five visual source types use the asymmetric complete transform fixture in T all_visual_sources_share_affine_preview_range_and_export: frame/range/export SSIM >= 0.99; non-silent 440 Hz audio range/export float-PCM RMS <= 0.0001 and duration <= one frame. T rendered_affine_rectangle_has_expected_position_and_rotated_extent checks actual expected pixels. H native lifecycle exercises materialized transformed drafts and preserves project state. Existing native golden verifies legacy animations, captions, transitions, transform positions, audio and timing against reviewed references. |
| Fail closed before affine rendering | R measured_transform_overflow_and_unavailable_backend_publish_nothing asserts dependency failure without execution or writes. Canonical invalid fixtures contain expressions, paths, URLs and executable markup; both native consumers reject them. Source coordinate dimensions >= 65535 fail with DEPENDENCY_UNAVAILABLE before workspace creation because the local remap backend uses 16-bit coordinates. |
| Isolated transformed caption box | R transformed_caption_measurement_has_explicit_box_and_is_read_only checks exact box and insets; T all_visual_sources_share_affine_preview_range_and_export renders its complete noncentral affine transform. Legacy native golden preserves bottom-centered direct captions. |

## Implementation and compatibility evidence

model.rs, timeline.rs and validation.rs own typed values and mutation/persistence constraints. migrations.rs advances the existing recoverable envelope to schema 8. evaluated_scene.rs owns source dimensions, finite matrices, inverse mapping and outward bounds; render_artifact.rs performs read-only font measurement before resource writes; renderer.rs finalizes and checks backend readiness; render_plan.rs lowers bounded premultiplied bilinear affine sampling. The current media path has no separate fit/crop stage, so its pre-scale source box is its probed native width/height. Other existing render paths remain unchanged.

Legacy EvaluatedVisualLayer debug output omits absent affine fields to preserve reviewed semantic fixtures. Golden normalization now recognizes escaped Windows drive colons after slash normalization; an explicit expected-token assertion covers this fix. Reviewed golden references were not changed.

## Executed checks

- cargo fmt --check --all: PASS.
- cargo clippy --workspace --all-targets -- -D warnings: PASS.
- cargo test --workspace: PASS with real native rendering enabled, including the reviewed golden, all ten Transform2D integration tests and native headless draft lifecycle.
- Bridge bun run typecheck, lint, test: PASS; 73 unit tests across 14 files.
- Bridge bun run contracts:check: PASS; Rust protocol and 9 TypeScript canonical contract tests.
- Bridge bun run test:integration: PASS (6 tests).
- Bridge bun run test:smoke: PASS (2 packaged tests).
- bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive: PASS (15 items before sync).
- moon run root:openspec-validate: PASS via bunx @moonrepo/cli@2.3.3 (the pinned executable was absent from PATH); 228 policy tests, 14 living specifications, and final CI policy validation passed.

Native runs used FFmpeg/FFprobe 8.1.2 and the reviewed DejaVuSans font, SHA256 ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280. Tool paths and OPENCUT_GOLDEN_REQUIRED=1 were set explicitly. Downloaded tools/logs remain ignored local data. Five existing isolated helper tests are marked ignored in ordinary discovery and invoked by their parent harnesses; no required feature scenario was skipped. Python worker/provider surfaces were unchanged, so their hermetic tests are outside this approved change's affected scope.

Resolved check failures: Windows font-path normalization in golden comparison, an audio test fixture missing top-level hasAudio metadata, and CRLF formatting in the fake media fixture. Final reruns pass; no reference regeneration or warning suppression was used.
