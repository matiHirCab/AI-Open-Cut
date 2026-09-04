## 1. Contract and migration fixtures

- [x] 1.1 Add failing editor-core serialization fixtures for schema-v7 common visual properties across every timeline-item variant, preserving flattened `transform` and `hidden` keys.
- [x] 1.2 Add deterministic v1/v6-to-v7 migration fixtures covering current state, mixed undo/redo history, non-default transforms, visibility, idempotent reopen, invalid retained snapshots, future versions, and injected persistence failures.
- [x] 1.3 Update the canonical headless/project contract examples to schema v7 and add failing Rust and TypeScript/Zod parity expectations before changing consumers.

## 2. Editor-core model and migration

- [x] 2.1 Introduce editor-core `VisualProperties` with legacy `Transform` and `hidden` state, flatten it into every timeline-item variant, and provide canonical immutable/mutable accessors without adding later-milestone fields.
- [x] 2.2 Update timeline creation, transform updates, visibility updates, duplication, splitting, drafts, batches, and validation to construct and mutate the common value while preserving revisions, aliases, rollback, changed IDs, and stable errors.
- [x] 2.3 Advance the project schema constant to 7 and implement deterministic whole-envelope migration and validation for current state plus every retained undo/redo snapshot under the existing recoverable transaction.
- [x] 2.4 Update scene evaluation and render preparation to read common values while keeping caption/transition identity transforms non-operative and preserving existing evaluated semantics.

## 3. Governed consumers and documentation

- [x] 3.1 Update headless Rust project serialization, agent-bridge TypeScript declarations, strict Zod schemas, MCP project responses, and their tests to match the canonical schema-v7 fixture without changing operation or capability catalogs.
- [x] 3.2 Update project migration, motion-graphics fixture activation, and compatibility documentation to record flattened common properties, schema-v7 defaults, future-version rejection, and the issue #18/#19 boundary.

## 4. Focused conformance

- [x] 4.1 Run focused editor-core model, migration, persistence, timeline, evaluated-scene, render-plan, renderer, and render-artifact tests, including success, invalid input, missing item/reference, revision conflict, batch rollback/aliases, undo/redo, reopen, and failure injection.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, and `bun run test` from `apps/agent-bridge`, plus the affected MCP integration and packaged smoke tasks declared by the workspace.
- [x] 4.4 Run the golden semantic/filter-graph, frame-preview, audiovisual-range-preview, draft-preview, and export conformance checks and confirm existing visual/audio/timing tolerances remain satisfied.
- [x] 4.5 Run `moon run root:openspec-validate` and record any platform-dependent skipped check with the required justification; no failed required check may remain.

## 5. Verification and archival

- [x] 5.1 Use `$openspec-verify-change` and resolve every mismatch among issue #17, requirements, design, tasks, fixtures, code, and verification evidence.
- [x] 5.2 Obtain the contract CODEOWNER review required by ADR 0002 for canonical project-shape and governed-consumer updates.
- [x] 5.3 Use `$openspec-archive-change` after verification and approval so accepted deltas update the living specifications and no active change remains.

## 6. Review corrections

- [x] 6.1 Reopen the issue #17 change and amend proposal, design, delta specs, and tasks to cover validation ordering, living-spec coherence, migration fault coverage, schema-zero rejection, and schema-v7 compatibility defaults.
- [x] 6.2 Obtain explicit approval for the amended OpenSpec artifacts before changing executable behavior.
- [x] 6.3 Validate migrated current/history visual properties and retained references before side-effectful legacy asset normalization.
- [x] 6.4 Extend invalid visual and dangling-reference migration tests to prove project/history bytes, transaction files, and the content-addressed asset store remain unchanged on rejection.
- [x] 6.5 Add migration-specific tests for every injected persistence phase plus current and retained schema-zero rejection.
- [x] 6.6 Add schema-v7 omitted-field compatibility tests proving read-only idempotence and explicit canonical response/next-write serialization.
- [x] 6.7 Synchronize the corrected living requirement, run every required focused/workspace/contract/integration/smoke/render/OpenSpec check, verify the change, and rearchive it with no incomplete task.
