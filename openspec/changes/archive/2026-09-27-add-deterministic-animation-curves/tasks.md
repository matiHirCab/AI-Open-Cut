## 1. Canonical contracts and persistence

- [x] 1.1 Add active curve shapes, parameter limits, compatibility examples, negative fixtures, and capability version to canonical channel/motion-graphics/MCP catalogs; update governed Rust and TypeScript fixture consumers together. (motion-graphics-contracts)
- [x] 1.2 Add schema-23 source guards and locked migration of current, undo, and redo snapshots, with tests for old-curve preservation, forbidden source fields, malformed history, future schemas, fault recovery, and deterministic reopen. (project-persistence)

## 2. Editor-core behavior

- [x] 2.1 Add typed parameterized curves and core finite/range/terminal-keyframe validation; test all inclusive/exclusive boundaries, unknown fields/variants, and unchanged `hold`/`linear`. (animation-channels)
- [x] 2.2 Implement fixed-iteration Bézier and analytic spring sampling with exact endpoints, all damping regimes, bounded overshoot, and fixed cross-platform scalar fixtures. (animation-channels)
- [x] 2.3 Test standalone and batch alias curve edits, missing references, stale revisions, atomic rollback, undo/redo, and no renderer side effects on invalid input. (animation-channels)
- [x] 2.4 Test equivalent evaluated plans and decoded frame, range, draft, and export output for visual curves and audio gain; prove legacy output preservation and invalid-persisted-curve preflight. (rendering-export)

## 3. Transport and documentation

- [x] 3.1 Update typed headless union/tests and bridge Zod/MCP schemas, capability reporting, integration tests, and packaged smoke for additive curve records in standalone and batch operations. (motion-graphics-contracts)
- [x] 3.2 Document curve coordinates, item-local timing, parameter bounds, interpolation, overshoot, fallback, error behavior, migration, capability discovery, and compatibility in `docs/animation-channels.md`. (animation-channels, motion-graphics-contracts)
- [x] 3.3 Obtain designated CODEOWNER review for the canonical contract and all governed consumers under ADR 0002. (motion-graphics-contracts)

## 4. Verification and archival

- [x] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; capture full logs and exit codes. (all changed Rust scenarios)
- [x] 4.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; capture full logs and exit codes. Python worker tests are not affected because provider workers and protocols do not change. (all changed TypeScript, MCP, and contract scenarios)
- [x] 4.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and the pre-archive `moon run root:openspec-validate`; require that the protected-gate rejection, if any, names only this active change. (all delta specs)
- [x] 4.4 Run `$openspec-verify-change` and resolve every requirement/design/task/test mismatch, then `$openspec-sync-specs`. (all requirements)
- [x] 4.5 Run `$openspec-archive-change`, then rerun `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; require both to pass before declaring completion. (final repository gate)

## Verification status (2026-09-27, Windows)

- Rust formatting, strict Clippy, workspace tests, focused native curve rendering, TypeScript typecheck/lint, MCP integration, packaged smoke, and strict OpenSpec validation pass.
- The exact `bun run test` and `bun run contracts:check` commands pass outside the filesystem sandbox, where their render-worker child processes can terminate normally: 418 unit tests and 352 contract tests pass. Sandbox-only runs timed out in those worker tests.
- Pinned Moon 2.3.3 runs through Bun's cache. Its pre-archive task passes strict spec validation and rejects only `add-deterministic-animation-curves`, the expected active-change boundary.
- The CODEOWNER approved the final contract and governed-consumer changes in this chat.
- Conformance audit: all 6 changed requirements and 20 scenarios map to implementation and automated tests; no design, contract, or test mismatch remains. The four living capability specs now contain the approved deltas. The user approved archival with the post-archive gate pending; the change was archived and both post-archive gates passed.
