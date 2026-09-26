## 1. Baseline and protected policy

- [x] 1.1 Record the completed 345-minute Render parity run, its golden-test duration, current exact protected command, and unchanged fixtures/tolerances as the comparison baseline. (render-regression-fixtures)
- [x] 1.2 Update the Render parity workflow's golden command to use the optimized Rust test profile with visible successful-test output; keep the remaining commands, step environments, order, and fail-closed behavior. (render-regression-fixtures, repository-validation)
- [x] 1.3 Update the exact CI policy expectation and regression tests; prove the reviewed command passes and omission, profile change, and moving it outside the protected step fail. (repository-validation)

## 2. Golden diagnostics

- [x] 2.1 Add named monotonic elapsed-time observations around all seven existing native golden conformance suites and sampled capture, without changing fixture data, comparisons, or pass/fail criteria. (render-regression-fixtures)
- [x] 2.2 Add focused test evidence for diagnostic coverage and compare existing debug and optimized golden results with required native dependencies; reject any output, tolerance, or report divergence. (render-regression-fixtures)

## 3. Verification and completion

- [x] 3.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the focused optimized golden command with FFmpeg, FFprobe, font, and report settings, and focused CI policy tests. Capture full logs and exit codes.
- [x] 3.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; from the root run `bun run apps/agent-bridge/scripts/run-python-tests.ts`. Capture full logs and exit codes.
- [x] 3.3 Run pinned strict all-spec validation and the pre-archive `moon run root:openspec-validate` gate; require the only expected rejection to name this active change. Run protected Render parity on the Linux runner, confirm every assertion and report gate passes, inspect all named suite durations, and record the achieved job and golden-test reduction against the baseline. If output differs or reduction is not material, revise the design and keep this task incomplete.
- [x] 3.4 Run `$openspec-verify-change`, resolve mismatches, then `$openspec-sync-specs` and `$openspec-archive-change` with designated review where required.
- [x] 3.5 Rerun `moon run root:openspec-validate` and pinned strict all-spec validation after archival and require the final protected CI gate to pass before declaring the optimization complete.
