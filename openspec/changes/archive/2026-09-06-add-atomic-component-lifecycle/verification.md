# Verification: add-atomic-component-lifecycle

Date: 2026-09-06. Workflow: openspec-verify-change. Proposal approval and the validation-boundary amendment are recorded in proposal.md and amendment.md.

## Implementation and ownership

The new core EditOperation/Serde variant is closed and rejects null overrides. Timeline alias resolution treats duplication as a single-ID creator, resolves only the source itemId, clones the instance, offsets its start with checked safe-integer arithmetic, optionally replaces its override map, and appends it with a fresh ID and stack order. Existing validation/store paths own graph/slot validity, locks, revisions, atomic publication and history. No dependency edge, renderer code, persisted shape, provider or migration implementation changed.

Headless exposes the edit through its existing typed edit envelope and advertises component_lifecycle. TypeScript, Zod and MCP register the additive operation standalone and in batches/draft schemas. Canonical MCP input/output schemas, annotations, protocol capability order and ownership entries agree with consumers. Documentation is in docs/component-lifecycle.md with links from evaluation and contract documentation.

## Scenario traceability

| Delta scenario | Automated evidence |
| --- | --- |
| Duplicate with independent values | component_lifecycle.rs: lifecycle_aliases_overrides_history_and_reopen; all_slot_kinds_and_special_keys_duplicate_without_materializing_definitions; component-workflow.ts real all-kind duplication |
| Resolve defaults after clearing overrides | lifecycle_aliases_overrides_history_and_reopen (empty map accepted with defaults); all_slot_kinds_and_special_keys_duplicate_without_materializing_definitions (required no-default clear rejected) |
| Reject invalid duplicate candidates | failures_preserve_project_and_history_bytes; safe_time_end_boundary_and_wrong_source_type_are_atomic; canonical_lifecycle_operations_are_closed; retained template_slots.rs managed-asset/effective-binding validation regressions |
| Enforce existing inclusive bounds | safe_time_end_boundary_and_wrong_source_type_are_atomic; duplication_enforces_aggregate_text_at_the_inclusive_boundary (1048576 total scalars, 4096 per value); canonical native/Zod malformed value fixtures |
| Build and duplicate a template in one transaction | lifecycle_aliases_overrides_history_and_reopen; protocol.rs: component_lifecycle_native_contract_and_atomic_history; instance-workflow.ts source and packaged calls |
| Reject conflicts and alias or trailing failures | failures_preserve_project_and_history_bytes; native protocol lifecycle test; instance-workflow.ts real stale/locked/invalid/trailing-failure snapshots; retained batch alias-envelope tests |
| Preserve rendering and saved compatibility | duplicate_matches_explicit_placement_in_preview_range_draft_and_export compares decoded pixels from draft/explicit/committed-reopened candidates in all three render intents; retained component_evaluation.rs nested rendering and store current/history migration tests |
| Preserve expanded render preflight | duplicated_expansion_fails_before_render_artifacts (65536 inclusive hidden occurrences, duplication exceeds bound); retained evaluated_scene::tests::accepts_each_scene_limit_and_rejects_boundary_plus_one |
| Exercise real complete lifecycle clients | instance-workflow.ts and component-workflow.ts invoked by source smoke.test.ts and packaged-smoke.test.ts; native protocol lifecycle test |
| Preserve failure atomicity through transports | Native protocol invalid fixtures and project/history byte comparisons; source/packaged instance-workflow failure snapshots |
| Verify lifecycle parity and discovery | contracts.test.ts lifecycle fixture acceptance and registered MCP definition equality; headless capability catalog test; component_lifecycle.rs canonical Serde fixtures |
| Preserve existing clients and persisted state | Existing duplicate_items and schema-13 declarations unchanged; workspace compatibility/migration tests; lifecycle undo/redo/reopen tests; protocol-1 contract suite |

All changed normative behavior has automated evidence. Provider workers and provider contracts are unchanged, so no new Python-specific checks are required for this scope. Existing ignored Rust tests are child-process helper entry points or explicitly opt-in benchmarks; their caller tests remain enabled.

## Validation evidence

- `cargo fmt --check --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed with RUST_TEST_THREADS=1, real FFmpeg/FFprobe 7.1.1 and the bundled DejaVuSans.ttf. This includes all 8 new lifecycle tests, native golden conformance, architecture checks, existing migration/history tests and 18 headless protocol tests. The earlier parallel run hit the existing process-tree memory sampler's resource-sensitive assertion; serial execution passed without weakening production or test logic.
- `bun run typecheck`: passed.
- `bun run lint`: passed.
- `bun run test`: 85 tests passed across 14 files.
- `bun run contracts:check`: passed (headless unit/protocol tests and 21 TypeScript contract tests); the final run also passed the additional unsafe-resource and null-number lifecycle fixtures.
- `bun run test:integration`: 9 source integration tests passed, including complete lifecycle and all slot kinds.
- `bun run test:smoke`: 4 packaged smoke tests passed, including the shared lifecycle workflows.
- Pinned OpenSpec `validate --all --strict --no-interactive`: 18 items passed.
- `moon run openspec-validate` via pinned @moonrepo/cli@2.3.3: final post-archive run passed normalization, all 231 policy tests, all 17 living specifications and the CI parity/archive-only policy. The earlier pre-archive rejection is resolved.
- `git diff --check`: passed.

Real rendering uses temporary FFmpeg/FFprobe 7.1.1 and the repository's DejaVuSans.ttf fixture. The installed newer FFmpeg rejected the existing filter_complex_script option; its installation and renderer implementation were not changed. The host Arial font was not the reviewed golden identity; the final run uses the bundled fixture.

## Final contract-owner approval

Designated owner @matiHirCab approved the concrete canonical contract and consumer implementation in PR #111 at commit 69f91e1a88ee6e545c41971a8e9719d4043aa773 on 2026-09-06 with the explicit message "Approve final contract review and commit". This approval is distinct from the earlier proposal and amendment approvals.
The four approved requirements are synchronized to the agent-bridge, component-evaluation and motion-graphics-contracts living specs. The change is archived at openspec/changes/archive/2026-09-06-add-atomic-component-lifecycle. Post-archive Moon validation passed; executable implementation and previously verified scenario coverage are unchanged.


## Final assessment

| Dimension | Result |
| --- | --- |
| Completeness | Implementation, scenario coverage and executable checks complete; 15/15 lifecycle tasks complete, including designated owner approval, synchronization, archival and the post-archive Moon gate. |
| Correctness | Approved behavior and amendment match code; no unresolved scenario mismatch. |
| Coherence | Core owns semantics, transports adapt typed input, schema/protocol compatibility is preserved, no new dependency edge. |

All required local checks and lifecycle gates passed. Contract-owner review, living-spec synchronization and archival are complete. No unresolved completeness, correctness or coherence findings remain. Remote PR checks remain subject to GitHub CI.
