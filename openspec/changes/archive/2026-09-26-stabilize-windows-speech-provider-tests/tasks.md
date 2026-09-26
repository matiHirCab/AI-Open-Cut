## 1. Speech-provider fixture

- [x] 1.1 Apply a 20-second runner deadline to each of the 11 real-child cases in `apps/agent-bridge/tests/tts.test.ts`; retain the 10-second provider control deadline and every assertion.
- [x] 1.2 Make `hang` in `apps/agent-bridge/tests/fixtures/fake_tts_worker.py` remain pending until process termination; leave non-hanging fake responses unchanged.
- [x] 1.3 Run `bunx vitest run --config vitest.unit.config.ts tests/tts.test.ts` and `bun run test` from `apps/agent-bridge`, and confirm the timeout, cancellation, queue, and failure assertions still execute.

## 2. Repository verification

- [x] 2.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 2.2 Run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`.
- [x] 2.3 Run the hermetic Python worker tests, CI-policy tests, `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`, and the pre-archive `moon run root:openspec-validate`; record any rejection caused only by this active change.

## Required completion sequence

After all implementation tasks pass, run `$openspec-verify-change` and resolve every mismatch. Then use `$openspec-sync-specs` and `$openspec-archive-change`, repeat strict OpenSpec validation and the protected Moon gate, and push the repair. Require the protected workflow's Windows, Ubuntu, and macOS correctness, render, contract, smoke, and final foundation duration checks to pass before declaring completion; record any unverified check.
