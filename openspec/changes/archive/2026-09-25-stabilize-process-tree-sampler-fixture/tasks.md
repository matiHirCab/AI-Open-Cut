## 1. Diagnose and stabilize the fixture

- [x] 1.1 Preserve the failed Windows job log and record the nested helper's failed 32 MiB observation, the fixed 300 ms hold, and a focused local baseline run. (render-regression-fixtures)
- [x] 1.2 Add a complete readiness and release handshake to the isolated child-allocation fixture, with bounded waits, early-exit detection, unconditional child release/reaping, and diagnostics. Keep the production sampler and 32 MiB threshold unchanged. (render-regression-fixtures)
- [x] 1.3 Exercise successful observation and an observation or readiness failure path without leaving a child or sampler worker running. (render-regression-fixtures)

## 2. Required verification

- [x] 2.1 Run the focused sampler test repeatedly on Windows, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; capture logs and exits.
- [x] 2.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; from the root run `bun run apps/agent-bridge/scripts/run-python-tests.ts`. Capture logs and exits.
- [x] 2.3 Run pinned strict all-spec validation and pre-archive `moon run root:openspec-validate`, requiring the only expected rejection to identify this active change. Push the fix and require Windows correctness and Render parity CI to pass without weakening any gate.

## 3. Completion

- [x] 3.1 Run `$openspec-verify-change`, resolve mismatches, and synchronize and archive the change with `$openspec-sync-specs` and `$openspec-archive-change`.
- [x] 3.2 Rerun `moon run root:openspec-validate` and pinned strict all-spec validation after archival.

The final protected CI gate must pass on the archived commit before declaring PR #124 ready.
