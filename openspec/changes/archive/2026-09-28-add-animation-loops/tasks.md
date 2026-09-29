## 1. Canonical contract and persistence groundwork

- [x] 1.1 Add reviewed schema-25 loop fixtures and exact canonical channel/headless/MCP/capability metadata, update governed consumer inventory and MCP digest, and add failing Rust/TypeScript parity tests (motion-graphics-contracts).
- [x] 1.2 Add failing migration tests for schema-24 current state, nonempty mixed undo/redo history, pre-25 loop injection, malformed/future history, interrupted publication, deterministic reopen, and unchanged legacy visual/audio output (project-persistence).
- [x] 1.3 Implement core schema-25 migration of current state and retained history under the existing lock/recoverable transaction; make 1.2 pass (project-persistence).

## 2. Editor-core loop semantics

- [x] 2.1 Add failing tests for finite/infinite repeat and ping-pong at first/last keyframe, each exact seam, one millisecond on either side, finite exhaustion, pre-loop hold, later-cycle range start, large safe timestamps, reverse Bézier/spring, property bounds, and visual/audio independence (animation-loops).
- [x] 2.2 Add failing validation tests for one keyframe, zero span, unequal repeat endpoints, zero/fractional/excess counts, unknown fields/variants, non-finite values, and unavailable channels; implement typed model, validation, and constant-work checked phase mapping, then make 2.1 and 2.2 pass (animation-loops and animation-channels).
- [x] 2.3 Add failing core transaction tests for standalone and aliased batch edits, later-operation rollback, missing/locked targets, stale revision, undo/redo, deterministic reopen and old unlooped requests; implement through existing replace-channels path (animation-channels).
- [x] 2.4 Add tests at exact seams and matching absolute timestamps for frame, audiovisual range with nonzero start, draft, and export; verify shared evaluated values and decoded visual/audio tolerance on supported FFmpeg 6 and 8 without legacy output drift (rendering-export).

## 3. Typed agent surfaces and documentation

- [x] 3.1 Update headless Rust and bridge TypeScript/Zod channel input, status capability and existing batch schema; add standalone/batch integration tests for valid loops, malformed shape, core errors, alias resolution and rollback (agent-bridge).
- [x] 3.2 Update MCP schema parity, packaged smoke and user documentation for wire shape, count, half-open boundaries, exact seams, phase, finite exhaustion, errors, compatibility and schema migration (agent-bridge and motion-graphics-contracts).

## 4. Verification and completion

- [x] 4.1 Run `bun run contracts:check` from `apps/agent-bridge`; from repository root run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; from `apps/agent-bridge` run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke`. Run relevant hermetic Python worker tests if worker input/output changes, otherwise document why none are affected. Capture complete logs and fix every failure.
- [x] 4.2 Run strict `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and the pre-archive `moon run root:openspec-validate`; record the expected active-change-only protected-gate rejection and resolve any other failure.
- [x] 4.3 Use `$openspec-verify-change` to compare every requirement, scenario, design decision, task and test against implementation, and resolve all mismatches with recorded evidence limits.
- [x] 4.4 Use `$openspec-sync-specs` and `$openspec-archive-change`, then rerun strict all-spec validation and `moon run root:openspec-validate` until both pass.
