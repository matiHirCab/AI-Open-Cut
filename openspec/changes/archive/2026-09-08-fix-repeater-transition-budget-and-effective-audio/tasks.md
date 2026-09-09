## 1. Artifact validation and approval

- [x] 1.1 Run `bunx @fission-ai/openspec@1.5.0 validate fix-repeater-transition-budget-and-effective-audio --strict --no-interactive` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; resolve artifact errors. Evidence (2026-09-08): change valid, full catalog 24 passed / 0 failed, four artifacts complete and git diff --check passed.
- [x] 1.2 Obtain explicit approval of the concrete proposal, design, three deltas and tasks; record it before executable edits. Evidence: explicit user approval on 2026-09-08.

## 2. Complete transition budgets

- [x] 2.1 Add failing core regressions for 17 facts times 257 occurrences, exact 4,096/4,097 facts and self-endpoint roles. Trace: Complete retained transition fact budgets / reproduced excess and inclusive boundary scenarios.
- [x] 2.2 Add hidden-transition/track/repeater/instance, clipped-instance, local/exterior copy, independent-domain and declaration-order tests. Assert hidden facts remain unpublished and late failures produce zero generated materializations. Trace: retained transitions and domain isolation scenarios.
- [x] 2.3 Implement separate retained fact metadata and checked complete-projection accumulation before generated copies, preserving visible transition vectors and independent budgets. Run `cargo test -p opencut-editor-core --lib evaluated_scene -- --test-threads=1`.

## 3. Effective visual-only validation

- [x] 3.1 Add failing direct-evaluation and canonical-validation tests for defaults/overrides introducing audio into component or group sources, nested components and local repeaters. Include hidden, muted, clipped and unused content. Trace: Effective visual-only repeater sources / all scenarios.
- [x] 3.2 Extract non-recursive slot application shared with the full resolver, and move audio-closure checking after references/cycles/slots in canonical project validation. Preserve active-node defenses and per-effective-value caching without copy expansion.
- [x] 3.3 Cover opposite effective assets on two instances in both orders, audible-authored/silent-effective sources, missing assets/components, invalid slots, graph cycles and supported depths. Run `cargo test -p opencut-editor-core --test component_evaluation --test template_slots --test repeaters -- --test-threads=1` and the focused evaluator command from 2.3.

## 4. Lifecycle, facade and transport evidence

- [x] 4.1 Add core batch/draft rollback, literal-ID draft, revision conflict, valid undo/redo/reopen and invalid-snapshot no-rewrite tests. Compare authoritative bytes and history. Trace: Atomic effective-audio repeater validation / both scenarios.
- [x] 4.2 Add renderer facade tests for invalid transition budgets and effective audio, asserting no backend execution, generated copies, output artifacts or project changes. Preserve and rerun required native repeater and full golden coverage.
- [x] 4.3 Add native headless regression coverage and extend shared source/packaged MCP workflow for invalid batches/drafts and valid silent-source preview/lifecycle, without adapter-side semantics. Run `cargo test -p opencut-headless --test protocol -- --test-threads=1`. Trace: Transport parity for effective repeater audio rejection / both scenarios.
- [x] 4.4 Update docs/repeaters.md and add a dated qualification to the archived predecessor's proposal/tasks/verification, linking this correction without removing historical results or reopening that archive. Confirm no corrective changes to schema 17, protocol 1, public catalogs, IDs or persisted formats.

## 5. Verification and closure

- [x] 5.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --test-threads=1`. Include alias, RichText, order, interval, independent-source and schema-16 migration/recovery regressions. Run the workspace tests with `OPENCUT_GOLDEN_REQUIRED=1`, explicit reviewed FFmpeg/FFprobe 8.1.2 paths and the reviewed DejaVuSans fixture; verify both full and focused native golden tests executed without backend skips.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; record results.
- [x] 5.3 From root run `bun run apps/agent-bridge/scripts/run-python-tests.ts` and `git diff --check`; record results.
- [x] 5.4 Repeat strict change/catalog validation from 1.1; use openspec-verify-change for a fresh requirement/scenario/design/test audit, record evidence and resolve every discrepancy before sync or archive.
- [x] 5.5 Use openspec-sync-specs and openspec-archive-change to synchronize the three deltas and archive the verified change. Correct predecessor links for the final archive location. Mark complete only after actual archival. Evidence: all three deltas synced; moved to archive/2026-09-08-fix-repeater-transition-budget-and-effective-audio with .openspec.yaml preserved; predecessor links updated.
- [x] 5.6 Run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`, final `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, and `git diff --check`. Record final results and mark closure only after all required gates pass. Evidence (2026-09-08): Moon 2.3.3 target passed, 231 policy tests and 23/23 specs; independent strict catalog validation and git diff --check passed. All 18 tasks complete.
