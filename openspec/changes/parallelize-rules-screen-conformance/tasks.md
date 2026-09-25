## 1. Baseline and coverage

- [x] 1.1 Record the two completed optimized Render parity runs, their `rules_screen` and total durations, exact 3x5 lifecycle matrix, 75 render operations, current references/tolerances, and unchanged protected CI command. (render-regression-fixtures)
- [x] 1.2 Add focused automated evidence that the planned one- and two-worker schedules each run all 15 cases exactly once in per-resolution lifecycle order and propagate an injected worker failure. (render-regression-fixtures)

## 2. Native conformance optimization

- [ ] 2.1 Add monotonic, labeled timing observations around each rules-screen preview timestamp, range preview, export, state total, and resolution total. Check all 75 operation records exist on a successful native run without turning timings into pass/fail budgets. (render-regression-fixtures)
- [x] 2.2 Run the independent resolution groups in at most two scoped workers when capacity exists, otherwise serially; preserve all existing calls, 15 states, cold/warm behavior, comparisons, reference data, and failure propagation. (render-regression-fixtures)

## 3. Verification

- [ ] 3.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and focused schedule/timing tests; capture logs and exits. The first three and the focused schedule test passed locally; native timing evidence awaits Linux CI.
- [x] 3.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; from the root run `bun run apps/agent-bridge/scripts/run-python-tests.ts`. Capture logs and exits.
- [ ] 3.3 Run pinned strict all-spec validation and the pre-archive `moon run root:openspec-validate` gate, requiring only the active-change rejection. Push the candidate and require Linux Render parity plus Windows, Ubuntu, and macOS correctness to pass; inspect 75 operation timings, unchanged reference and report checks, peak resource behavior, and wall time against both baselines. Revise if the gain is not material or resource behavior is unstable.

## 4. Completion

- [ ] 4.1 Run `$openspec-verify-change`, resolve any mismatch, synchronize the delta with `$openspec-sync-specs`, and archive with `$openspec-archive-change`.
- [ ] 4.2 Run post-archive `moon run root:openspec-validate` and pinned strict all-spec validation; require the final protected CI gate to pass before declaring PR #124 ready.
