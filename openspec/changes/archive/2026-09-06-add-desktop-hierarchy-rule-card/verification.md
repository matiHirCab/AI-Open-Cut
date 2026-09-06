# Verification: add-desktop-hierarchy-rule-card

Approval: user message "Approve", 2026-09-06, covering proposal, design, delta specs and tasks.

## Assessment

| Dimension | Result |
| --- | --- |
| Completeness | 18/18 tasks complete, including archival and the final Moon gate |
| Correctness | All six requirements and their scenarios mapped to implementation and passing evidence below |
| Coherence | Desktop presentation depends inward on EditorCore; no duplicated domain validation, public transport change, persisted field or new private core edge |
| Findings | No remaining critical issues, warnings or specification mismatches |

The current project schema and retained-history migration behavior are unchanged. Existing migration and architecture suites passed. No cross-language contract ownership review is needed because only tests consume existing contracts; canonical catalogs and capabilities retain their current meanings.

## Requirement and scenario evidence

| Requirements/scenarios | Implementation and automated evidence |
| --- | --- |
| Explicit project session; load/refresh; empty, unsafe, missing and future-schema selection | apps/desktop/src/session.rs; tests.rs startup_and_load and invalid_load |
| Scoped hierarchy; repeated instances; cross-track parentage, hidden nodes and bounded expansion | apps/desktop/src/hierarchy.rs and panels/browser.rs; tests.rs repeated_instances, cross_track_groups and bounded_rows |
| Parent/detach and signed z-index; malformed input, missing reference, cycles, locks and conflict | panels/inspector.rs, shell.rs and session.rs; tests.rs history_and_conflict and failed_edits; native UI below |
| History/reopen, external edits and removed selections | session.rs; tests.rs history_and_conflict and selection_reconciliation; native UI below |
| Six visual children, three independently slotted instances, parent movement, standalone edits, atomic aliases and failure rollback | crates/editor-core/tests/support/rule_card.rs, fixtures/rule-card/recipe.json and tests/rule_card.rs; rule_card_lifecycle and rule_card_failures_preserve_files |
| Five lifecycle states across still/range/export; independent transforms, ordering, slots, repeatability, pixels, audio, timing and failure gates | renderer/golden/rule_card.rs; native_golden_render_conformance, rule_card_oracle_detects_slot_transform_and_order_drift, rule_card_metadata_and_coordinated_drift_fail_closed and rule_card_missing_dependencies_fail_required_gate |
| Agent-addressable standalone and batch compatibility | apps/agent-bridge/tests/rule-card-workflow.ts via component-workflow.ts in both MCP integration and packaged smoke |

All normative requirements have automated coverage; no automation exemption was needed. UI projections decode input and display validated state; core decides mutation legality. Component-local rows intentionally show stored definition properties, while root instance rows display stored override maps. The preview player remains outside the approved scope.

## Executed validation

- cargo fmt --check --all: passed.
- cargo clippy --workspace --all-targets -- -D warnings: passed, including the final test additions.
- cargo test --workspace: passed, including desktop, core unit/integration, migrations, architecture and headless protocol. The standard explicitly ignored maintenance helpers remain opt-in; the required native test was separately executed with dependencies configured.
- cargo build -p opencut-desktop and cargo test -p opencut-desktop: passed, 12 desktop tests.
- cargo test -p opencut-editor-core --test rule_card: passed, 2 lifecycle/atomicity tests.
- Final cargo test -p opencut-editor-core --lib rule_card_: passed, 3 automated failure/oracle checks; deliberate recapture is explicitly ignored in normal runs.
- Explicit initial capture_rule_card_references invocation: passed, including subsequent five-state conformance. Original decoded frame was visually inspected: three overlapping cards, distinct rule numbers/text/icons and opacity.
- Required cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact: passed (417.97 seconds), covering both immutable rule-card and existing flat-scene references. Environment: Windows x64, FFmpeg/FFprobe 8.1.2 essentials, explicit DejaVu Sans SHA-256 ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280, OPENCUT_GOLDEN_REQUIRED=1. Final changes after this run only tightened the bounded reference reader, pre-publication recapture checks and negative-test coverage; targeted tests and strict Clippy passed afterward.
- From apps/agent-bridge: bun run typecheck and bun run lint passed; bun run test passed (85 tests); bun run contracts:check passed (headless and 21 bridge parity tests); bun run test:integration passed (9 tests); bun run test:smoke passed (4 packaged tests).
- bun run scripts/run-python-tests.ts: passed (10 unittest tests and 5 pytest tests).
- bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive: passed all 18 pre-archive entries.
- git diff --check: passed.
- Final Moon/archive result is recorded below.

Local command logs are under ignored local-data/issue26-*.log. Cargo 1.93.0 and Bun 1.4.0 were used locally. Moon 2.3.3 is available through its pinned Bun package.

Initial fixture failures were resolved by supplying canonical local stackOrder and making the fake probe report the rule-tone's actual one-second duration. A transient Windows linker error occurred when a targeted test tried to replace the still-running native test executable; rerunning after native completion passed. No failed required check remains.

## Native Windows interaction evidence

Manually operated the built application through computer-use against a newly generated synthetic project on 2026-09-06:

1. Loaded Three rule cards at revision 7; saw the root group, all timeline instances and local timing.
2. Expanded the parent and each of its three instances. Selected equal local child IDs across different occurrences and observed distinct instance paths and read-only controls.
3. Selected instance0 from the hierarchy; the matching timeline row highlighted. Inspector displayed Plan/Think/1, icon reference and opacity 1.
4. Focused z-index, cleared with Ctrl+A, entered 4 and pressed Enter: revision 8, z-index 4 in inspector and timeline.
5. Detached the instance: revision 9, root tree position and Parent None; reparented to the group: revision 10.
6. Undo restored detached state at revision 11; Redo restored the group at revision 12.
7. Made an external typed headless edit to z-index 6 at revision 13. Desktop Undo at displayed revision 12 returned REVISION_CONFLICT with retryable true and retained the displayed state. Refresh loaded revision 13 and z-index 6.
8. Closed and freshly relaunched the desktop; hierarchy and root timeline reconstructed at revision 13 with the persisted z-index 6.
9. Started without arguments: explicit No project loaded state, no fabricated project. Started with missing project ID: PROJECT_NOT_FOUND with retryable false, no loaded candidate.
10. Closed all test windows after verification.

## Final archival and Moon gate

Archived as 2026-09-06-add-desktop-hierarchy-rule-card with all six delta requirements merged into living specs. The only unchecked task at the moment of archival was archival itself plus its post-archive Moon gate; it is now complete. No active change remains.

Pinned Moon 2.3.3 (`bunx @moonrepo/cli@2.3.3 run root:openspec-validate`) passed: 231 repository policy tests, all 18 living specifications and CI parity gate policy. No required check remains failed or skipped.
