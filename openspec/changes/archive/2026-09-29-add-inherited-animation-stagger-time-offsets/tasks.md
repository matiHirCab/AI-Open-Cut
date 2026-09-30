## 1. Canonical contract and persisted model

- [x] 1.1 Update canonical contract ownership/catalog/fixtures for staggerMs, timeOffsetMs, supported parent channels and capability reporting; classify every affected public and persisted field as additive and record the designated CODEOWNER review.
- [x] 1.2 Add closed editor-core model fields and schema-26 source-version guards, defaults and validation for current state, history, drafts and raw edits; add boundary and malformed-input tests.
- [x] 1.3 Implement atomic schema-25-to-26 migration of current and retained undo/redo states under the project lock; test older-schema rejection of new fields, fault phases, future-schema rejection, unchanged reopen and deterministic prior output.

## 2. Core edit and evaluation

- [x] 2.1 Extend editor-core typed group/component create, update_item, component_instance_update and duplication plus repeater create/replace edits with bounded timing fields; test standalone, alias-aware batch, revision, missing-reference, locked-track, trailing-failure, undo/redo and reopen scenarios.
- [x] 2.2 Activate supported visual channels on groups and component instances through editor-core compatibility and existing curve/loop sampling; test inherited matrices, opacity, transform2d conflicts and prior simple-operation compatibility.
- [x] 2.3 Compose group and component direct-child stagger clocks with canonical sibling ranking, hidden-child stability, half-open clipping and nested fractional instance clocks; test exact boundaries, loops and retained invalid content.
- [x] 2.4 Apply signed per-copy repeater source-clock offsets to complete visual subtrees while retaining ordinary-source timing, transform/opacity powers, order, identity and visual-only rules; test positive/negative/zero offsets and nested repeaters.
- [x] 2.5 Extend complete retained and generated preflight to shifted clock, geometry, occurrence, transition, resource and other existing fact budgets before clone or artifact work; add exact-limit, one-over, overflow and no-publication tests.
- [x] 2.6 Prove frame, audiovisual range, draft and export consume identical shifted EvaluatedScene semantics with deterministic fixtures within documented visual/audio tolerances.

## 3. Transport, contracts and documentation

- [x] 3.1 Update headless typed requests/results and capability reporting from canonical contract artifacts; add request, batch, compatibility and error-code tests without duplicating core validation.
- [x] 3.2 Update agent-bridge TypeScript/Zod and MCP schemas/annotations plus any governed Python consumer for additive timing fields; prove standalone and batch alias behavior and canonical parity.
- [x] 3.3 Update behavior and migration documentation for coordinate/timing order, clocks, rank, clipping, fallback, limits and public compatibility; keep fixtures, design, delta specs and task status synchronized.

## 4. Required checks and closeout

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; capture full logs and resolve failures.
- [x] 4.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; run `python apps/kokoro-tts/test_worker.py` from the repository root if governed Python/worker behavior changes; capture full logs and resolve failures.
- [x] 4.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and pre-archive `moon run root:openspec-validate`; the protected-gate rejection must name only this active change.
- [x] 4.4 Run `$openspec-verify-change` against all requirements, scenarios, design decisions, tasks and check evidence; resolve every mismatch.
- [x] 4.5 Synchronize and archive this verified change with `$openspec-sync-specs` and `$openspec-archive-change`, then rerun `moon run root:openspec-validate` and strict all-spec validation and require both to pass.

