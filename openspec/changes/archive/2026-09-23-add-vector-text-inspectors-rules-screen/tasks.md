## 1. Approval and traceability

- [x] 1.1 Obtain explicit approval of the original proposal, design and two delta specs before implementation; recorded in this task on 2026-09-22.
- [x] 1.2 Obtain explicit approval of the additive `fontSize` amendment across proposal, design, third delta spec and tasks; recorded in this task on 2026-09-22. Scenario-to-test mapping is tracked in the implementation tasks below.

## 2. Additive text edit contract

- [x] 2.1 Update canonical headless/MCP capability and request fixtures and governed consumers with optional `fontSize`, including standalone and batch schemas; confirm old fixtures remain valid and obtain designated CODEOWNER review. The designated CODEOWNER approved PR #123's contract changes on 2026-09-22.
- [x] 2.2 Add editor-core validation/mutation and Rust/TypeScript/headless/MCP parity tests for bounds, target failures, aliases, drafts, revisions, history and reopen; run `bun run contracts:check` from apps/agent-bridge.

## 3. Desktop inspector

Automated `inspector_edit` field/patch tests and desktop `Session` selection, error and history tests cover the state and operation boundaries. GPUI focus and native keyboard interaction cannot be asserted by these headless unit tests; the user completed the generated-project manual smoke on 2026-09-22 as additional evidence.

- [x] 3.1 Add desktop tests for scoped inspection, supported controls and read-only variants/occurrences (I1).
- [x] 3.2 Add selection/revision-bound drafts and structured vector/grid/text fields with Apply/Reset through existing core operations; test unedited field/font/run/layout preservation (I2).
- [x] 3.3 Test representation failures, core finite/complexity rejection, missing/locked targets and conflicts without partial publication or automatic retry (I3).
- [x] 3.4 Test selection/refresh/reset draft invalidation and undo/redo/reopen with authoritative values (I4).

## 4. Core fixture and bridge evidence

- [x] 4.1 Add the exact native rules-screen recipe, independent semantic expectations and fresh-directory generation example, with documented deterministic font/audio provenance (F1).
- [x] 4.2 Add core standalone/aliased batch construction and vector/text edit lifecycle tests, including byte-preserving failed transactions (F1-F2).
- [x] 4.3 Add bridge MCP integration evidence using existing operations and aliases, covering success, invalid input, missing references, stale revisions and later-operation rollback (F1-F2). Confirm canonical contract fixtures remain unchanged using contracts:check.
- [x] 4.4 Add separately reviewed, hash-recorded visual references at all three resolutions and native-gate coverage for all intents/timestamps/lifecycle states, including cold/warm-cache parity and shared-drift detection (F3-F4). The corrected references were visually reviewed at 1920x1080 and 960x540; PR #123 Render parity passed on Linux with native audiovisual/lifecycle and raster-cache steps both successful.
- [x] 4.5 Document fixture generation/reference review, inspector workflow, coordinate/order/font behavior and limitations in docs/desktop-hierarchy.md, docs/render-regression-fixtures.md and the milestone status (I1-I4, F1-F4).

## 5. Required verification

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` on the final inputs; retain full logs and explicit exit statuses. All passed; logs: `%TEMP%/opencut-37-final-clippy2.log` and `%TEMP%/opencut-37-final-workspace-alone.log`.
- [x] 5.2 Run `cargo build -p opencut-desktop` and manually exercise fixture loading, each inspector field family, invalid input, conflict/refresh, selection changes, undo/redo and reopen. User confirmed the full documented native smoke checklist passed on 2026-09-22 against the generated rules-screen demo; build log: `%TEMP%/opencut-37-desktop-build.log`.
- [x] 5.3 Run `cargo test -p opencut-editor-core --lib renderer::golden::native_golden_render_conformance -- --exact` with documented deterministic native dependencies; confirm rules-screen coverage actually executes and no required checks skip. PR #123 Render parity ran the equivalent exact native test with required FFmpeg, FFprobe and font configuration, followed by native cache parity, and passed. The local run first found a missing FFprobe in the extracted toolchain; the corrected local rerun was interrupted before completion. CI provides the completed required-gate evidence.
- [x] 5.4 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`. Run `bun run scripts/run-python-tests.ts` there for hermetic worker regression coverage.
- [x] 5.5 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Before archive, only rejection naming this active change is expected; inspect complete logs and resolve every other failure.
- [x] 5.6 Use `$openspec-verify-change` to audit requirement/scenario coverage, tests, implementation, design and evidence. The five delta requirements and eleven scenarios map to the desktop inspector/unit and manual smoke, core/bridge contract tests, independent fixture tests and successful hosted native render gate. No requirement, scenario or design mismatch remains; the two failed PR statuses are the expected active-change OpenSpec policy and dependent foundation aggregate.

## 6. Finalization

- [x] 6.1 Use `$openspec-sync-specs` and `$openspec-archive-change` after successful implementation checks and conformance review. All three delta specs were synchronized byte-for-byte into their living requirements and the verified change was archived on 2026-09-23.
- [x] 6.2 Rerun `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; both must pass before completion. Both passed after archive with 28/28 strict items; logs: `%TEMP%/opencut-37-final-moon-postarchive.log` and `%TEMP%/opencut-37-final-strict-postarchive.log`.
- [x] 6.3 Report implemented behavior, validation evidence and any limitations against issue #37, preserving unrelated work. PR #123 has the final behavior and validation summary; the final task handoff reports the new CI run status.
