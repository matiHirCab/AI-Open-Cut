## 1. Contract and persistence foundation

- [x] 1.1 Update the canonical channel/value, schema-version, operation, MCP, and capability fixtures and contract ownership/parity expectations for active numeric bounds and inactive deferred bounds with prospective targets; classify the additions under ADR 0002 and obtain the designated CODEOWNER review. (motion-graphics-contracts)
- [x] 1.2 Add schema-22 channel storage and deterministic locked migration for current, undo, and redo documents, including malformed-history, future-schema, interrupted-commit, and reopen tests. (project-persistence)

## 2. Editor-core behavior

- [x] 2.1 Add closed channel/value types and canonical bounds, target compatibility, identity, timing, finite-value, inactive-channel, and legacy-collision validation with boundary tests, including both audio gain endpoints and first invalid values. (animation-channels)
- [x] 2.2 Add the replace-channels edit to core standalone/batch transaction handling and test success, missing/locked target, stale revision, aliases, rollback, undo/redo, and unchanged legacy operations. (timeline-editing)
- [x] 2.3 Sample active position X/Y, independent scale X/Y, opacity, and audio gain channels in the shared evaluated scene; test exact/held/interpolated values and fail-closed unsupported persisted input for every active property. (animation-channels, motion-graphics-architecture)
- [x] 2.4 Prove still, range, draft, and export parity and legacy render equivalence with deterministic output fixtures for each active visual property and audio gain combined with base volume, mute, fade, and ducking. (rendering-export)
- [x] 2.5 Put serialized channel types under `model` and semantic channel rules under `validation`; remove the standalone mixed owner, document the placement in ADR 0003, and make the architecture test reject unlisted top-level private modules. (editor-core-architecture)

## 3. Transport and documentation

- [x] 3.1 Extend the typed headless request/response union and headless tests for standalone and batch channel edits and error translation; replace `unknown[]` channel input with the schema-derived type and prove malformed values fail type checking. (agent-bridge)
- [x] 3.2 Extend bridge types, Zod schemas, MCP registration, capability reporting, and MCP integration tests, including alias and failure parity; consume every canonical entry's value, target, activation, and bounds metadata in Rust and TypeScript parity tests. (agent-bridge, motion-graphics-contracts)
- [x] 3.3 Document channel names, prospective targets, active bounds, deferred inactive bounds, value tags, coordinate/timing/interpolation rules, compatibility, and migration for clients. (animation-channels, motion-graphics-contracts)
- [x] 3.4 Change the Windows worker-crash test fixture to wait for a fully readable PID record before process assertions, preserving the existing worker lifetime and protocol behavior; exercise the existing crash-termination scenario repeatedly on Windows. (agent-bridge)

## 4. Verification and archival

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` on the repaired tree; record full logs and exit codes.
- [x] 4.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; run the hermetic Python worker tests and record full logs and exit codes.
- [x] 4.3 Add the native channel output suite to render-parity CI with required FFmpeg/FFprobe settings and update the exact CI policy command/environment expectations and regression tests; rerun Windows correctness and render parity, then run pinned strict all-spec validation and the pre-archive `moon run root:openspec-validate` gate; record the expected rejection only if it names this active change and no other failure.
- [x] 4.4 Run `$openspec-verify-change`, resolve all requirement/design/task/test mismatches, then `$openspec-sync-specs` and `$openspec-archive-change`.
- [x] 4.5 Rerun `moon run root:openspec-validate` and pinned strict all-spec validation after archival; require both to pass before claiming completion.
