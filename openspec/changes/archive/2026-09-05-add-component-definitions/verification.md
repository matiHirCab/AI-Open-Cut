# Verification: add-component-definitions

Issue #22. The user approved the proposal, design, delta requirements and implementation tasks with “Approve” on 2026-09-05. This report applies openspec-verify-change to the resulting implementation; proposal approval is distinct from the final designated CODEOWNER review.

## Assessment

| Dimension | Result |
| --- | --- |
| Completeness | 22/22 tasks complete. Correction implementation, executable checks, renewed final CODEOWNER approval, specification synchronization, re-archive and final Moon validation are complete. |
| Correctness | All 10 requirements and 22 scenarios have automated evidence below. The reopened correction resolves unsafe nested times, invalid caption provenance and non-media volume keyframes. No remaining correction mismatch identified. |
| Coherence | Model, validation, timeline, migration and asset ownership remain inside existing core owners; no new private dependency edge. Root evaluation is unchanged. Schema 11 replaces the stale schema-v8 roadmap label without reusing a shipped version. |

## Requirement and scenario evidence

Unless qualified otherwise, tests are in crates/editor-core/tests/components.rs.

| Requirement / scenarios | Evidence |
| --- | --- |
| Stored local component timelines: persist empty/populated definitions | `canonical_component_operations_are_closed` consumes empty, nested and omitted-default canonical fixtures. `aliased_lifecycle_incoming_durations_locks_and_rollback` verifies complete state and exact undo/redo. |
| Stored local component timelines: confine local references | `full_graph_scope_identity_and_longest_path_boundaries` uses repeated local IDs in independent definitions, legal component-local parents and illegal root-scope parents. `component_item_defaults_closed_fields_and_unsupported_root_placement` rejects unknown nested fields and root placement without file changes. Existing group transition-endpoint regressions remain active; the shared validator also excludes nested instances from transition endpoints. |
| Bounded nested component graph and duration: branching/shared references | `canonical_component_operations_are_closed` consumes the canonical diamond, shared nesting, self/indirect cycle, duplicate and missing-reference examples. `full_graph_scope_identity_and_longest_path_boundaries` validates unused definitions and repeated local instance IDs in separate compositions. The aliased lifecycle test has two valid instances of the same leaf. |
| Bounded nested component graph and duration: inclusive limits | `canonical_aggregate_count_limits_are_inclusive` exercises 512/513 definitions and 4096/4097 aggregate tracks/items. `full_graph_scope_identity_and_longest_path_boundaries` exercises longest path 16/17. Existing parent depth/node and keyframe validation tests continue to cover the reused rules. |
| Bounded nested component graph and duration: timing boundaries | `numeric_boundaries_and_missing_references_publish_nothing` covers zero/unsafe duration, dimension bounds, parent/source overflow, zero/negative/nonfinite time scales and unchanged files. Canonical nested fixtures have exact source/local endpoints; source-duration fixtures exercise overflow. Native equivalence uses the full 1000 ms root interval. |
| Atomic definition management: create/reference in a batch | `aliased_lifecycle_incoming_durations_locks_and_rollback` creates and nests through aliases, checks definition changed IDs and exact history. Shared real MCP `verifyComponentWorkflow` creates a leaf and consumer via aliases then deletes in dependency order, as one undoable revision. |
| Atomic definition management: preserve failures | The aliased lifecycle test covers missing targets, referenced deletion, cycles, locked local-track replacement/deletion, stale revision and failure after a valid earlier edit. `component_protocol_aliases_failures_and_exact_history` in apps/headless/tests/protocol.rs also proves non-retryable malformed-alias decoding and byte-identical files for failing batches. Existing alias and batch-boundary tests cover the unchanged resolver/envelope limits. |
| Atomic definition management: incoming duration constraints | The aliased lifecycle test rejects shortening the referenced leaf below an existing instance's source interval, preserving the exact generation. |
| Preserve root rendering: stored definitions | `native_unused_definitions_preserve_frame_range_export_and_draft_output` compares decoded pixels for root-only, committed-definition and durable-draft states across frame preview, range preview and MP4 export; root duration stays 1000 ms and output remains red and byte-equal after decoding. |
| Preserve root rendering: invalid direct input | The same native test submits an invalid unused definition directly to Renderer and observes INVALID_ARGUMENT. Existing no-side-effect renderer tests exercise the reused preflight path before workspace/process/publication. |
| Atomic schema-11 component migration: mixed retained history | `all_supported_current_and_mixed_history_migrate_atomically` consumes canonical source-version/current-history matrices for every schema 1–10, verifies schema-11 components in all snapshots and byte-identical repeated reopen. Existing migration/golden regressions verify unchanged root fields, provenance and output. |
| Atomic schema-11 component migration: rejection/recovery | `invalid_current_and_retained_components_never_rewrite` covers schema zero, future versions and malformed definitions in both current and history. Store `supported_migrations_recover_every_publication_phase` now includes schema 10 and validates every durable publication phase; existing pre-journal failure and retained-envelope tests still pass. |
| Component managed media retention: definitions/history/drafts | `definitions_and_durable_drafts_retain_media_through_history` protects media used only in a definition or durable draft, verifies deletion blocking, removes the current reference and asset, then restores retained history with unchanged managed bytes. The same asset inventory includes caption source references. Existing caption provenance/retained-reference tests remain active. |
| Component managed media retention: unsafe/missing references | The media-retention test rejects missing, traversal-shaped and URL-shaped asset IDs before mutation and compares project/history bytes. Existing canonical managed-path confinement remains the only file-resolution policy; no component operation imports resources or executes an instance. |
| Governed component definition runtime contract: parity | `canonical_component_operations_are_closed` checks Rust standalone/batch acceptance, closed fields and null/string/wrong-type aliases. Bridge canonical tests consume the same operation, definition, semantic and omitted-default fixtures; semantic graph decisions remain core-owned. Exact MCP input/output schema and annotation parity passes against the manually authored catalog, including every embedded batch and project shape. |
| Governed component definition runtime contract: discovery | Headless capability/catalog tests and the new protocol lifecycle verify additive discovery and schema 11. Bridge exact tool-registration parity verifies all three standalone tools. Existing legacy/simple request suites pass with updated current-schema expectations. |
| Typed component definition workflows: lifecycle/failures | Headless `component_protocol_aliases_failures_and_exact_history` plus shared `verifyComponentWorkflow` in apps/agent-bridge/tests/component-workflow.ts cover create/update/delete, aliased nested creation, cycles, missing targets, rollback, undo/redo and reopen through real source integration and packaged binaries. |

## Implementation review

- Project schema 11 contains required components; old supported records receive an empty collection. Existing root operation meanings and protocol major 1 are retained. Unknown future schemas still fail closed.
- Component DAG depth is computed with bounded iterative leaf removal over stored references, without expanding reusable timelines. All definitions, including unused records, are validated. Collection limits precede indexing.
- Component operations use the existing evolving candidate and commit boundary. Only creation generates an ID/alias. Replacement checks locked track content and relative order. Definition deletion rejects incoming references.
- Root parent validation is factored into a shared composition-scope validator. Both load and direct evaluation invoke component validation. Nested instances cannot be placed at root or used as transition endpoints.
- Asset inventories include definitions and definition-bearing draft operations; retained project/history inventories reuse the same path. No new outward dependency or renderer semantics were added. Changes in renderer test literals only initialize the new Project field.
- New component MCP schemas preserve existing omitted common visual/track defaults. Full serialized project output remains explicit. Root schemas and existing operations retain their prior behavior.
- Component catalog, ownership, CODEOWNERS, native consumers, headless/MCP catalogs and documentation are synchronized. The large MCP catalog diff is from embedding the complete component track schemas in existing project outputs and edit unions; parity does not regenerate the fixture.

## Correction requirement and scenario evidence

The user explicitly requested implementation of the full correction plan on 2026-09-05. The proposal, design, two additional requirements and four scenarios were recorded before implementation. Earlier final approval covered the previous version, not these corrections.

| Requirement / scenario | Automated evidence |
| --- | --- |
| Complete component item validation / nested timing and property boundaries | 46 manually authored itemValidationFixtures distinguish Rust decoding, MCP structural acceptance and core validity. canonical_component_item_validation_is_atomic_at_every_core_boundary covers safe maximum/overflow keyframes, fades and provenance times, fractional/negative decoding, and volume on media versus text/solid/rectangle. Valid maximum keyframes/fades deliberately exceed item duration, guarding against new interval restrictions. |
| Valid component caption provenance / moved caption | caption-moved-words has current interval [0,1000) and source words [2000,3000). The core fixture test creates/replaces it, checks a durable draft, exact undo/redo and reopen. Shared MCP workflow creates it and parses the returned project in integration and packaged smoke. Empty word arrays and omitted/null confidence are also fixtures. |
| Valid component caption provenance / malformed nested content | The fixture test rejects every decodable invalid fixture on standalone create/update, late batch and draft creation with byte-identical project/history; direct rendering rejects before output directory creation. Injected current, undo and redo records fail unchanged. typed_nonfinite_component_caption_confidence_is_rejected covers NaN and both infinities on source/word confidence. component_nested_input_failures_preserve_headless_generation checks real JSON-line errors, non-retryability and unchanged files for standalone/batch invalid nested keyframes. |
| Valid component caption provenance / canonical cross-language acceptance | Bridge test matches canonical component item structural acceptance independently of core semantics consumes all 46 fixtures through headless edit, MCP create/update and batch schemas. Shared verifyComponentWorkflow sends every structurally accepted invalid fixture through actual MCP late batches, asserting INVALID_ARGUMENT and unchanged state; populated captions remain readable in both integration and packaged smoke. |

## Correction implementation review

All new executable rules live in validate_components; root timeline/transcription validation and adapters are unchanged. Existing call paths cover mutations, drafts, history/current reads and direct render preflight. Keyframes retain ordering/value semantics, with safe times and media-only volume; fades gain safe-time checks without duration restrictions. Caption checks preserve optional defaults and immutable source timing without current-placement containment or source rewriting.

Canonical fixtures and consumers were extended without changes to public shapes, MCP schemas, capability, protocol major, schema 11, migration policy or provider behavior. Malformed existing records fail closed without repair. Final conformance review found no remaining correction mismatch; the earlier findings are covered by the named regressions.

## Executed correction checks

- Rust formatting and strict workspace Clippy: passed without warning suppression.
- Workspace tests: passed with retained FFmpeg/FFprobe 8.1.2, checked-in DejaVuSans and OPENCUT_GOLDEN_REQUIRED=1. Includes 176 core unit tests, 12 component tests, architecture, migration/recovery, native goldens, 23 groups, 13 Transform2D and 15 headless protocol tests. Five existing helper/report tests remain intentionally ignored; no new skip.
- Bridge typecheck/lint: passed. Initial test formatting/lint errors were corrected before successful final checks.
- Bridge unit tests: 78 passed.
- Contract parity: passed (15 headless protocol tests, headless unit tests and 14 bridge contract tests).
- MCP integration: 9 passed.
- Packaged smoke: 4 passed with freshly built binaries.
- Pinned strict OpenSpec: 16/16 living specifications and active changes passed before archive; all 15 living specifications passed after archive.
- Pinned Moon 2.3.3: passed after archive with 231 policy tests, all 15 living specifications, CI parity policy and archive-only inventory. The earlier active-inventory failure is resolved.
- git diff --check: passed.
- Python/provider tests remain unaffected because provider behavior and contracts did not change.

Ignored local logs: target/components-correction-clippy.log, components-correction-workspace.log, components-correction-contracts.log, components-correction-integration.log, components-correction-smoke.log and components-correction-moon.log.

## Final correction approval and review

On 2026-09-05, designated CODEOWNER @matiHirCab approved the correction and requested a final review. That review found no remaining actionable defects. All 12 component tests (including native rendering) and 26 bridge contract/schema tests passed again. The three original review findings are resolved by the canonical regressions and component-scoped validation; no new implementation edits were needed during this review.

All 10 requirements and 22 scenarios are synchronized with the living specifications. The change is archived at openspec/changes/archive/2026-09-05-add-component-definitions. Final pinned Moon passed with 231 policy tests, 15 living specifications and archive-only inventory (target/components-correction-archive-moon.log). All 22 tasks are complete.

The full executable suites were run on the correction in the preceding implementation turn; this final review repeated the focused suites and final policy gate. The five existing ignored helper/report tests and unaffected Python/provider suites remain as documented above. No commit, push or PR publication was performed.
