# Verification: expose-group-parenting-operations

Issue #21. Implementation approved by the user on 2026-09-05. This report reviews the concrete implementation against the approved proposal, design, three delta specs and tasks using openspec-verify-change.

## Assessment

| Dimension | Result |
| --- | --- |
| Completeness | 20/20 tasks complete. Implementation, correction verification, final CODEOWNER review, specification synchronization, archive and post-archive Moon validation are complete. |
| Correctness | All 5 added requirements and 15 scenarios have automated evidence below. No implementation mismatch found. |
| Coherence | Core owns mutation/locks/aliases; headless and MCP delegate typed input; schema 10 and protocol major 1 remain unchanged; no renderer or provider behavior added. |

## Requirement and scenario evidence

Test names below refer to crates/editor-core/tests/groups.rs unless another file is named.

| Requirement / scenario | Automated evidence |
| --- | --- |
| Atomic local-preserving ungroup: root and nested groups | `ungroup_promotes_immediate_children_preserves_local_state_and_exact_history` compares the entire resulting project to an independent state oracle for both root and nested removal; asserts deterministic changed IDs, cross-track children and retained deeper links. `ungroup_preserves_every_visual_kind_media_integrity_and_caption_provenance` covers group, media, text, solid, rectangle and caption children, including media bytes and caption source metadata. |
| Atomic local-preserving ungroup: empty group | `ungroup_empty_group_normalizes_other_ordinals_and_is_undoable` asserts removal, sibling ordinal changes, exact changed-ID order and undo. |
| Atomic local-preserving ungroup: evaluated semantics | `native_ungroup_matches_explicit_reparent_delete_in_frame_range_and_export` compares actual ungroup to an explicit core reparent/delete sequence, verifies materialized draft state, and renders both at 0, 500 and 900 ms. Lossless frames are identical; each range/export decoded frame differs from its preview by mean RGB error below 3/255. The fixture removes non-identity transform, opacity, hidden and interval contributions; a non-black oracle asserts that the hidden child becomes visible. Existing native group, all-source Transform2D and golden suites retain their own stricter spatial/SSIM/audio checks unchanged. |
| Ungroup failures preserve the complete transaction: missing and invalid targets | `ungroup_canonical_target_failures_and_stale_revision_leave_files_untouched` consumes group-parent-v1 failure fixtures, checks stable errors, and compares project/history bytes. |
| Ungroup failures preserve the complete transaction: affected locks | `ungroup_checks_all_affected_tracks_but_allows_read_only_locks` checks group track, hidden/inactive immediate child track, and legal locked ancestor/deeper-descendant track. Failing edits preserve project/history bytes. |
| Ungroup failures preserve the complete transaction: bounded validation and revisions | The canonical failure test checks stale revision precedence. `ungroup_batch_enforces_exact_operation_count_limits` checks 0, 1, 100 and 101 operations and rollback. Existing `group_static_edits_and_nonfinite_values_are_transactional`, `exact_parent_depth_boundary` and `persisted_graph_count_duplicate_and_reference_boundaries` cover finite values, depth 32/33 and nodes 4096/4097 through the unchanged canonical validation. |
| Ungroup batch aliases and reversible results: create, parent, order and ungroup | `ungroup_alias_creation_lifetime_and_late_failure_are_atomic` executes an evolving alias workflow, asserts retained creation mappings even for removed groups and one undo/redo state. The 100-operation test checks 50 create/remove pairs and deduplicated IDs. |
| Ungroup batch aliases and reversible results: rollback after ungroup | The alias lifecycle test checks removed aliases, unresolved/forward aliases, later failed operations, prohibited result aliases submitted as typed core input, and unchanged persisted bytes. Structural resultAlias rejection is also in canonical payload tests and headless protocol tests. |
| Ungroup batch aliases and reversible results: exact history/reopen | Root/nested state-oracle and all-visual-kind tests verify undo/redo, repeated reads with unchanged bytes, exact properties, assets and provenance. Core calls reopen persisted state; protocol and MCP tests additionally use explicit open_project/project_open calls. |
| Governed additive ungroup contract: discovery | Headless `capability_sets_match_the_canonical_headless_contract` and `canonical_status_requests_negotiate_protocol_version_and_capabilities`; shared MCP `verifyGroupWorkflow` checks catalog-named capability and destructive tool annotation through real discovery. |
| Governed additive ungroup contract: canonical parity | `canonical_group_payloads_are_closed` and bridge runtime group contract tests consume valid, alias, missing, wrong-type, unknown-field and resultAlias fixtures. Bridge MCP surface parity compares all registered normalized input/output definitions and annotations with the manually extended catalog, including every embedded edit union. `verifyGroupWorkflow` consumes canonical semantic failure fixtures through actual MCP/core. |
| Governed additive ungroup contract: compatibility and persistence | Existing group/stacking/Transform2D fixtures and request regression suites pass. `migration_preserves_every_supported_history_and_rejects_bad_graphs_atomically`, retained-history graph tests, and workspace persistence fault-injection tests retain schema-10 migration/recovery/future-reader behavior. No persisted field, schema major, error or provider catalog changed. |
| Complete typed group workflows: standalone | apps/headless/tests/protocol.rs `ungroup_protocol_workflow_aliases_rollback_and_history`; apps/agent-bridge/tests/group-workflow.ts `verifyGroupWorkflow` creates a group and rectangle, parents, sets z-index and ungroups, then verifies exact undo/redo/reopen state. It runs from both source MCP integration and packaged smoke suites. |
| Complete typed group workflows: batch and failures | The same real transport tests cover aliases, missing/non-group targets, malformed input, stale retryable revision, affected locks, and later-operation rollback. MCP additionally checks a removed group alias in a failing batch. |

## Implementation and contract review

- `crates/editor-core/src/model.rs`: additive GroupUngroup in public/remote operation unions and strict field allowlist. Existing batch creator checks reject resultAlias for this non-creator.
- `crates/editor-core/src/timeline.rs`: resolves groupId aliases, validates target kind and all affected tracks, promotes direct children, removes one node, and reuses graph validation, ordinal normalization and transaction publication. Changes require no new dependency edge.
- `apps/headless/src/main.rs`: adds discovery capability; typed edit transport continues to consume the core operation union.
- Bridge HeadlessEdit, Zod edit/batch schemas, standalone schema and thin MCP registrar agree with the canonical input and existing mutation output. Tool annotation is destructive because it removes a group.
- `group-parent-v1`, headless/MCP catalogs, ownership and CODEOWNERS were updated together. Existing operation names/meanings, aliases, schema 10 and protocol 1 are retained. Canonical MCP changes were authored from the intended schema, not generated by parity tests.
- `docs/group-parenting.md` documents promotion, visible consequences, timing, flat ordering, aliases, changed IDs, locks, errors, limits and schema compatibility.

## Executed checks

- `cargo fmt --check --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed, including final tests.
- `cargo test --workspace -- --test-threads=1`: passed with native FFmpeg/FFprobe 8.1.2, checked-in DejaVuSans and OPENCUT_GOLDEN_REQUIRED=1. Includes native goldens, all-source Transform2D, group rendering, migration/recovery and native headless lifecycle. Five subprocess helper tests are intentionally ignored by the parent harness and exercised by owning tests. The subsequently added exact batch-boundary test also passed separately; final strict Clippy and formatting passed again.
- Bridge `bun run typecheck` and `bun run lint`: passed after final helper extraction.
- `bun run test`: 76 unit tests passed.
- `bun run contracts:check`: passed (typecheck, 3 headless unit tests, 12 protocol tests and 12 bridge contract tests); bridge canonical parity reran successfully after final ownership/catalog updates.
- `bun run test:integration`: all 9 tests passed. The extracted shared group workflow reran against source MCP successfully after its final canonical-fixture update.
- `bun run test:smoke`: all 4 packaged tests passed, including the same final canonical group workflow.
- Pinned OpenSpec `validate --all --strict --no-interactive`: 15/15 items passed.
- Pinned Moon 2.3.3 `run root:openspec-validate`: workflow normalization, 231 policy tests and 15 OpenSpec items passed; final policy rejects the active change inventory. This required gate remains pending until archive and rerun.
- `git diff --check`: passed. No Python/provider behavior or contract changed; worker-specific tests are unaffected.

Initial native execution encountered sandbox denial, then installed FFmpeg 9's pre-existing lack of `filter_complex_script`. Retrying with the already retained compatible 8.1.2 tools resolved both; no renderer compatibility workaround was added. `moon` is absent from PATH, so the unchanged target was run with `bunx --package @moonrepo/cli@2.3.3 moon run root:openspec-validate`. Tool binaries, build output and logs remain only in ignored target directories.

## Completion gates

The previously outstanding designated owner review, specification synchronization, archive and final Moon validation are complete; see the final approval and CI completion evidence below. Earlier failed active-inventory checks are retained as historical evidence.

## Approved review correction: null resultAlias

The subsequent read-only review reproduced a contract violation: batch decoding discarded explicit null resultAlias, allowing group_ungroup to commit while standalone/MCP rejected it. This superseded the original clean assessment. The user approved the correction plan on 2026-09-05.

Core now decodes BatchEditOperation with a private presence-aware helper using the existing deserialize_double_option. GroupUngroup rejects a present field before converting to the unchanged public representation. Other operations keep their existing optional alias behavior and serialization. No public schema, capability, protocol major or persisted version changed.

Evidence for the added scenario:

- `canonical_group_payloads_are_closed` consumes all invalid ungroup fixtures, including null and wrong-type resultAlias, as both EditOperation and BatchEditOperation; valid omitted-alias ungroup fixtures parse successfully.
- `batch_alias_presence_preserves_other_operations_and_duplicate_rejection` proves omitted/null/string behavior and serialized round-trip for creation and non-creation operations, wrong-type rejection and duplicate null/string field rejection using raw JSON.
- `ungroup_null_alias_batch_is_rejected_before_any_publication` submits null/string/number aliases standalone and after a valid batch edit, asserting non-retryable INVALID_ARGUMENT, unchanged revision, and byte-identical project/history files.
- Bridge canonical parity covers invalid ungroup standalone and timeline batch schemas. The shared MCP workflow rejects a null-alias batch after a valid edit and reopens the exact prior state; it passes against both source and packaged binaries.

Final post-correction checks: formatting, strict workspace Clippy, bridge typecheck/lint, 76 unit tests, 12 contract parity tests plus headless tests, 9 integration tests, 4 packaged smoke tests, and 15/15 strict OpenSpec items pass. Moon passes content/policy tests but still rejects the active change inventory. Initial native workspace compilation encountered a Windows executable lock from concurrent integration; the serialized rerun passed the complete workspace with native FFmpeg 8.1.2 and required goldens, including all 23 group tests and 13 protocol tests. Python/provider contracts remain unaffected.


Final openspec-verify-change assessment after the approved correction: all 5 requirements and 15 scenarios have conformance evidence; no code/spec/design mismatch remains. The original review finding is resolved. The designated contract review and dependent archive/final Moon validation were subsequently completed as recorded below. No provider tests were added or skipped as affected work because no provider surface changed.

## Final owner approval and CI completion

Final CODEOWNER approval: on 2026-09-05, the user replied "I approve" to the explicit request to approve the final implementation and contracts as CODEOWNER and authorize synchronization, archive, validation and push for PR #107. This satisfies the designated owner review gate for the implementation, canonical contracts, consumers and parity evidence.

PR #107 CI run 33968695776 passed correctness on Linux, macOS and Windows, contract parity, render parity and packaged integration/smoke. OpenSpec failed only because this change was still active; foundation parity failed as its dependent aggregate. All five added requirements and 15 scenarios were synchronized into the three living specs, preserving existing requirements. The change was archived to openspec/changes/archive/2026-09-05-expose-group-parenting-operations. Final pinned Moon 2.3.3 root:openspec-validate passed with archive-only inventory: 231 policy tests, 14/14 living specs and CI parity policy validation. Local log: target/issue21-archive-moon.log (ignored). All 20 tasks are complete; no outstanding correctness, coherence or completion issues remain.
