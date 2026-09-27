## 1. Merge and protected CI policy

- [x] 1.1 Merge current `origin/main` into PR #126 in the isolated worktree, resolving the four conflicts while preserving both branches' unrelated safeguards and changes.
- [x] 1.2 Add policy tests for numeric 180-minute rules-screen timeout acceptance and missing, shorter, longer, and nonnumeric rejection; retain tests for 135-minute limits on other required jobs.
- [x] 1.3 Update the workflow and structural validator so only rules-screen parity has a 180-minute timeout and the temporary overall exception cap is exactly 180 minutes; keep the default 120-minute budget, other job limits, all prerequisites, and fail-closed duration assertion.
- [x] 1.4 Update duration-evaluator tests and implementation for the 180-minute hard cap, including boundary, overrun, malformed, and expired exception behavior.
- [x] 1.5 Synchronize CI documentation and the merged living repository-validation spec with the approved exception and its evidence and limitations.

## 2. Verification

- [x] 2.1 Run `bun --config=bunfig.toml --no-env-file test scripts/validate-ci-gates.test.ts scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts scripts/ci-duration.test.ts` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; retain full logs.
- [x] 2.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and from `apps/agent-bridge` run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke`; run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the root. Retain full logs and report any failure.
- [x] 2.3 Run the protected pre-archive `moon run root:openspec-validate`, then verify requirements, scenarios, design, tasks, and evidence with `$openspec-verify-change`; record the expected archive-only rejection separately from any other failure.

After implementation verification, synchronize and archive this change with `$openspec-sync-specs` and `$openspec-archive-change`, rerun the protected Moon gate and strict all-spec validation, and update PR #126.
