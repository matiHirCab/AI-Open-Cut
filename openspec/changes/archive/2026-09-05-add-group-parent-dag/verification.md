# Verification: add-group-parent-dag

Applied `openspec-verify-change` against the approved proposal, five delta specs, design, tasks, implementation diff, and test results on 2026-09-05. The empty-track deletion correction remains in force.

## Assessment

Core owns all hierarchy semantics. Transports decode typed input and translate results. No new architecture dependency edges, provider fields, stable errors, protocol major, or desktop authoring UI were introduced. Desktop required no exhaustive-match edits; workspace compilation and architecture tests verify it remains compatible.

Implementation and automated conformance are complete. Designated contract implementation approval was received on 2026-09-05 and recorded in proposal.md. All eleven added requirements were synchronized into five living specs, the change was archived, and the final archive-only Moon gate passed.

## Requirement and scenario evidence

| Requirement / scenarios | Automated evidence |
| --- | --- |
| Non-drawing group timeline nodes: create/edit, unsupported edits, node duplication | `groups::canonical_group_payloads_are_closed`, `group_static_edits_and_nonfinite_values_are_transactional`, `group_unsupported_edits_locks_and_node_only_duplication`; evaluator emits only the child in `groups_compose_geometry_clip_visibility_and_preserve_source_time` |
| Scoped bounded parent graph: cross-track resolution, missing/non-group/scope/cycle errors, exact depth/count | `cross_track_parenting_preserves_child_lifecycle_and_locked_parent`, `group_alias_graph_failures_rollback_and_history`, `canonical_graph_failures_are_rejected_on_open_without_publication`, `exact_parent_depth_boundary`, `persisted_graph_count_duplicate_and_reference_boundaries` |
| Transactional parenting lifecycle: aliases, rollback, locks/revisions, deletion, local-preserving child lifecycle | `group_alias_graph_failures_rollback_and_history`, `cross_track_parenting_preserves_child_lifecycle_and_locked_parent`, `group_unsupported_edits_locks_and_node_only_duplication`; headless `group_protocol_aliases_detachment_history_and_atomic_errors`; MCP `parents groups through standalone and atomic MCP tools with history` |
| Schema-v10 group migration: mixed histories, deterministic reopen and grouped undo/redo | `migration_preserves_every_supported_history_and_rejects_bad_graphs_atomically` covers schemas 1–9 and omitted parent; group lifecycle tests cover schema-10 reopen/history; existing `schema_six_common_visual_defaults_migrate_current_and_retained_history` and `schema_one_projects_and_history_are_migrated_before_persisting` exercise legacy fields, caption source, and assets through the current migration chain |
| Hierarchy migration fails closed: invalid retained graph/future data and publication recovery | Canonical graph failures are tested in both current state and retained history with byte-for-byte non-publication. Existing `invalid_transform_in_retained_history_is_never_published`, `rejects_future_retained_schema_without_mutating_any_document`, and schema-zero tests cover invalid local values/version errors. `supported_migrations_recover_every_publication_phase` covers schema 6 and 9 at all five post-journal phases; `schema_nine_migration_before_journal_failure_preserves_generation` covers pre-journal failure |
| Canonical group ancestor evaluation: independent nested matrix oracle, visibility/intervals, audio/order/immutability | `nested_group_oracle_covers_anchor_skew_scale_audio_and_overflow` independently applies anchor, scale, shear, rotation, and translation to asymmetric corners (1e-9 tolerance), and checks unchanged audio when the parent track is hidden. `groups_compose_geometry_clip_visibility_and_preserve_source_time` checks intersection, hidden items, opacity, original source span and immutable repeat evaluation. Existing flat stacking tests run unchanged |
| Bounded derived geometry: overflow rejected before effects | `invalid_group_graph_and_composed_geometry_have_no_render_side_effects` instruments all renderer facades; nested oracle covers composed overflow. Existing finite-boundary, measured-text overflow, path-safety, oriented-media, and missing-asset-precedence tests run through shared preparation |
| Governed runtime contract: discovery, drift, compatibility/safety | `group-parent-v1.json` supplies operation/default/closed-shape/graph fixtures; Rust consumes structural and semantic failures, TypeScript consumes structural fixtures and proves graph semantics remain delegated. Headless capability tests and MCP structural discovery parity include new operations and group responses. Protocol and MCP lifecycle tests exercise actual typed state. Existing simple-client tests pass |
| Shared parented rendering: all intents, failure before artifact work, legacy goldens | `native_grouped_animated_child_matches_preview_range_and_export` covers group clipping, independently expected rotated pixels, root-time animation, materialized draft equality, and unchanged state/history. `all_visual_sources_share_affine_preview_range_and_export` now runs unparented, nested-parent Transform2D, and nested-parent legacy cases for rectangles, solids, text, media and captions, with transparency, SSIM >= 0.99, PCM RMS <= 0.0001 and one-frame timing tolerance. Side-effect test covers missing groups, composed overflow and unavailable complete backend. Existing `native_golden_render_conformance` and native headless lifecycle tests preserve legacy output/history |

Shared Transform2D validation retains all existing finite and numeric bounds; groups call that same validator. Composition anchors are not derived from descendants. Legacy animated descendants validate object geometry independently of travel, then clip the conservative endpoint envelope to the composition and precompute non-overlapping sampling tiles of at most 4096 by 4096 pixels. Media/font paths remain in resource bindings rather than scene facts. Remaining motion-graphics catalog vocabulary remains fixture-only.

## Checks

- `cargo fmt --check --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: final rerun passed.
- `cargo test --workspace`: native run passed; a subsequent concurrent run hit the existing `process_tree_sampler_observes_a_child_allocation` timing-sensitive memory assertion. No group/golden assertion failed. Final `cargo test --workspace -- --test-threads=1` with native tools passed the entire workspace, including all 14 group tests and the process sampler. Five subprocess helper tests remain intentionally ignored in the parent process and are exercised by their owning tests.
- Bridge `typecheck`, `lint`, `test:unit`: passed; 76 unit tests across 14 files.
- Bridge `contracts:check`: passed; 12 fixture/discovery tests plus 3 headless unit and 11 protocol tests.
- Bridge `test:integration`: passed, 8 tests.
- Bridge `test:smoke`: passed, 3 packaged runtime tests.
- Pinned OpenSpec strict validation: 15 items passed, no failures.
- Pinned Moon 2.3.3 `root:openspec-validate`: 231 policy tests and strict specs passed; final archive-only gate rejected `add-group-parent-dag` because it remains active. This is a completion blocker until review and archival.
- Python provider-specific tests are unaffected: no Python/provider behavior or contracts changed. Packaged smoke still exercises its hermetic fake provider path.

Native checks used FFmpeg/FFprobe 8.1.2 and the checked-in DejaVuSans font with `OPENCUT_GOLDEN_REQUIRED=1`. The tool archive was SHA-256 verified before use. Installed FFmpeg 9.0.1 does not accept the repository's existing `-filter_complex_script` option; no unrelated backend compatibility change was made.

Local tools are retained only under ignored `target/group-test-tools/ffmpeg-8.1.2-essentials_build/bin` for reproducibility. The Moon target was invoked through `bunx --package @moonrepo/cli@2.3.3 moon run root:openspec-validate` because the local proto plugin registry was unreachable. This used the pinned release and the unchanged repository target.

## Previously required final actions (completed 2026-09-05)

1. Obtain @matiHirCab review of `group-parent-v1.json`, headless/MCP/ownership catalogs, native and TypeScript consumers, and parity evidence (AGENTS.md and ADR 0002).
2. Complete the OpenSpec archive workflow to merge deltas into living specs.
3. Rerun the pinned Moon target with archive-only inventory; report its result before claiming merge readiness.

## Correction verification (2026-09-05)

The user-approved follow-up corrects all three reproduced review findings. Correctness review and OpenSpec verification were rerun against the final implementation and all five delta specs. No further implementation, scenario, or design mismatch was identified. The earlier three findings are resolved.

| Dimension | Result |
| --- | --- |
| Completeness | 22/22 tasks complete, including designated review, archival and the final archive-only gate |
| Correctness | All delta requirements mapped to automated evidence, including the corrections below |
| Coherence | Core owns anchor evaluation, sampling geometry and endpoint validation; transports and public contracts remain unchanged by the corrections |

| Correction | Regression evidence and result |
| --- | --- |
| All nine legacy text anchors | `legacy_text_anchors_and_animated_sampling_are_composed_before_parents` checks measured styled-text geometry at non-unit scale against independent coordinates within 1e-9. `native_identity_parent_preserves_every_styled_text_anchor` passes all 18 static/animated cases. Native glyph bounds permit one two-pixel chroma cell because the legacy overlay snaps YUV placement while affine sampling preserves fractional coordinates. Existing nested-parent all-source conformance retains SSIM >= 0.99. Explicit Transform2D anchors and captions retain their existing paths. |
| Travel distance and tiled sampling | `native_long_distance_motion_tiles_reentry_and_draft_agree` renders a 20-by-10 rectangle through x=20000, offscreen and back, across a 4096-pixel tile boundary on a 4200-pixel composition. Frame, range, draft and export succeed; lossless adjacent seam pixels both equal red 127 at half opacity. Encoded frames retain SSIM >= 0.99; codec variation is measured separately from the lossless seam assertion. Draft pixels agree exactly and authoritative state/history bytes remain unchanged. Evaluator tests cover a 7680-pixel composition and empty offscreen sampling; existing overflow instrumentation still rejects oversized geometry before writes or execution. |
| Persisted group transition endpoints | `persisted_group_transition_endpoints_fail_closed_in_every_snapshot` passes both endpoints, visible/hidden records and current/undo/redo state (12 cases), checking INVALID_ARGUMENT and byte-for-byte authoritative-file preservation. `group_endpoints_are_rejected_before_draft_publication` verifies no draft publication. Shared renderer side-effect instrumentation rejects hidden transitions to either group endpoint across renderer facades. Existing mutation and valid-transition tests remain passing. |

Final correction checks: Rust formatting and workspace strict Clippy passed; the complete serial native workspace suite passed, including 14 group tests and 13 Transform2D tests. Native golden and all-source audiovisual tests retain SSIM >= 0.99, aligned PCM RMS <= 0.0001 and one-frame timing tolerances. Bridge typecheck, lint, 76 unit tests, 12 contract parity tests, 8 MCP integration tests and 3 packaged smoke tests passed. Contract parity additionally passed the 3 headless unit and 11 protocol tests. No Python provider code or contract changed, so worker-specific tests remain unaffected.

Strict OpenSpec validation passed all 15 items. Moon passed all 231 policy tests and strict validation, then exited 1 solely because `add-group-parent-dag` is unarchived. This failure is explicitly retained as a completion blocker.

### Completion gate resolution

The user replied "Approve" to the designated contract implementation review request on 2026-09-05. Approval evidence is recorded in proposal.md. Task 4.5 is complete.

All eleven added requirements were synchronized into the five owning living specs, preserving existing content. The verified change was moved to `openspec/changes/archive/2026-09-05-add-group-parent-dag`. Task 5.5 is complete.

The final pinned Moon 2.3.3 `root:openspec-validate` run exited 0: 231 policy tests passed, strict validation passed all 14 living specs, and the archive-only inventory passed. This supersedes the earlier pre-archive failure recorded above. Tasks 5.6 and 6.4 are complete.

All 22 tasks are complete. No remaining correctness findings, specification mismatches, or completion blockers were identified.
