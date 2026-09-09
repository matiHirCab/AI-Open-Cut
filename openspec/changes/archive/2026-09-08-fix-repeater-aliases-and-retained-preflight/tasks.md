## 1. Artifact validation and approval

- [x] 1.1 Run `bunx @fission-ai/openspec@1.5.0 validate fix-repeater-aliases-and-retained-preflight --strict --no-interactive` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; correct artifact errors. Evidence: change valid; full catalog 24 passed, 0 failed.
- [x] 1.2 Obtain explicit approval of proposal, design, all three deltas and tasks before executable edits; record approval in proposal.md. Evidence: user approval on 2026-09-08.
- [x] 1.3 Obtain approval of the 2026-09-08 draft compatibility correction: aliases are batch-only; drafts retain literal-ID operations. Evidence: explicit user approval on 2026-09-08 before executable edits.

## 2. Core alias conformance

- [x] 2.1 Add regressions for both batch replacement aliases, missing/forward aliases, item-before-source error precedence, trailing failure rollback, literal-ID draft materialization and missing-reference behavior, revision conflicts, undo/redo and reopen. Trace: Aliased repeater descriptor replacement / all scenarios. Evidence: the two replacement_alias tests failed before the resolver fix and pass afterward.
- [x] 2.2 Resolve UpdateItem itemId then optional repeater.source.id through the existing resolver; retain literal scope and existing transaction semantics. Evidence: `cargo test -p opencut-editor-core --test repeaters -- --test-threads=1` passed 8/8 on 2026-09-08.

## 3. Retained preflight and ordinary evaluation

- [x] 3.1 Add failing 4,096/4,097 fixtures for visible root, hidden track, hidden repeater, hidden instance, clipped instance and unused definition; add independent-domain and declaration-order regressions. Trace: Retained repeater validation domains.
- [x] 3.2 Add exact/one-over geometry and memory tests plus finite-power/non-finite-conjugation tests for visible and retained content; assert zero generated materialization on later-domain failure using test-only instrumentation. Trace: Validate retained geometry and conjugation; Reject before the first generated clone.
- [x] 3.3 Implement bounded retained occurrence views with independent root/definition budgets, effective slots/clocks, guarded traversal, complete local/outer copy projection and shared exact measurement before publication. Preserve hidden-output filtering and IDs/order/RichText.
- [x] 3.4 Remove expand_flat_repeaters and its call; verify ordinary scenes and nested components perform no repeater snapshot or copy allocation. Trace: Bounded ordinary evaluation without redundant snapshots.
- [x] 3.5 Run `cargo test -p opencut-editor-core --lib evaluated_scene -- --test-threads=1`, `cargo test -p opencut-editor-core --test component_evaluation --test repeaters -- --test-threads=1`, and focused no-artifact facade regressions. Include nested RichText, numeric order, independent repeaters and retained transform cases.

## 4. Transport coverage and documentation

- [x] 4.1 Add headless protocol coverage for batch source/item replacement aliases, missing/forward aliases, rollback, literal-ID drafts, undo/redo and reopen; run `cargo test -p opencut-headless --test protocol -- --test-threads=1`. Trace: Transport parity for aliased repeater replacement.
- [x] 4.2 Extend shared source/packaged MCP workflow and focused TypeScript tests with aliased replacement and failure rollback; render the resulting visible scene. Keep adapters thin and existing schemas unchanged.
- [x] 4.3 Document replacement aliases and retained budget domains in docs/repeaters.md. Add dated follow-up notes to the archived 2026-09-08 correction proposal/tasks, preserving historical evidence and linking this change; explicitly identify the gaps in its former no-findings claim.
- [x] 4.4 Confirm schema 17, protocol 1, public shapes/catalogs, generated ID format and persisted representation are unchanged by this correction. Any actual contract expansion requires amended approval and designated contract review before implementation.

## 5. Verification and closure

- [x] 5.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --test-threads=1`. Final sequential run passed: 12 desktop tests, 265 core units with 7 intentional helper/report-only ignores, all integrations and 25 headless protocol tests. Required full golden and focused repeater conformance both executed and passed with FFmpeg/FFprobe 8.1.2 and reviewed DejaVuSans; no backend skip.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; record results.
- [x] 5.3 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` and `git diff --check` from root; record results.
- [x] 5.4 Repeat strict change/catalog validation and use openspec-verify-change for a fresh requirement/scenario/design/test audit; resolve every discrepancy before sync or archival. Evidence: strict change and catalog 24/24 passed; verification.md maps all four requirements and eleven scenarios with no unresolved findings.
- [x] 5.5 Sync the three deltas and archive with openspec-archive-change. Completed: openspec/changes/archive/2026-09-08-fix-repeater-aliases-and-retained-preflight; all three living specifications synchronized and .openspec.yaml preserved.
- [x] 5.6 Run `moon run root:openspec-validate` with pinned Moon 2.3.3 (or `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` when moon is absent from PATH), final strict catalog validation and `git diff --check`. Completed after archival using pinned bunx Moon 2.3.3: 231 policy tests, strict catalog 23/23, target success; independent final strict validation and whitespace check also passed. All 20 tasks are complete.

## Execution evidence (2026-09-08)

- Core alias regressions: initial failures reproduced; resolver fix passes 8/8 repeater integration tests. Byte rollback, ordered alias errors, literal draft IDs, conflicts, undo/redo/reopen covered.
- Evaluation: 59 passing focused units, one intentionally ignored subprocess entry point executed through its parent regression. Root/hidden/clipped/unused 4,096/4,097 fixtures, independent domain order and late-domain rejection assert zero generated materializations. Exact/one-over raster and scalar segment/byte accumulators pass; native facade rejects hidden invalid raster work without files or backend execution.
- Component integration: 9/9; headless protocol: 25/25 through the contract/full-workspace gates, including two repeater workflows. Existing RichText, nested local/outer repeater, eleven-sibling order and schema-16 recovery evidence remains exercised.
- Bridge: typecheck, lint (63 files), unit (387), contract parity (323 focused TypeScript cases plus native consumers), source integration (10), packaged smoke (5) passed. Source/packaged workflow verifies both replacement aliases, missing/forward errors, trailing rollback and preview.
- Python: 10 unittest and 5 pytest cases passed. git diff --check passed.
- Corrective executable edits are confined to evaluator/measurement/alias resolution and regression tests. Schema 17, protocol 1, public shapes/catalogs, persisted forms and generated ID formatting are unchanged; pre-existing issue-31 public-contract edits were preserved.
- Final full-workspace/native sequential rerun passed (core unit/native phase 627.72 seconds). An earlier overlapping retry encountered Windows LNK1104 on an executable still used by the first successful run; rerunning after that process completed resolved it. No assertion failure was involved. The final complete bridge rerun also passed every gate.
- Native tools: local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin/{ffmpeg,ffprobe}.exe; OPENCUT_GOLDEN_REQUIRED=1; DejaVuSans.ttf SHA-256 ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280.

## Follow-up review — 2026-09-08

Tasks 5.4–5.6 record completed historical checks, but their no-discrepancy conclusion did not cover generated transition-fact budgets or audio introduced by effective asset slots. The reproduced failures are tracked in [fix-repeater-transition-budget-and-effective-audio](../2026-09-08-fix-repeater-transition-budget-and-effective-audio/proposal.md). Preserve these results and require fresh evidence for the correction rather than reusing this checklist as approval.
