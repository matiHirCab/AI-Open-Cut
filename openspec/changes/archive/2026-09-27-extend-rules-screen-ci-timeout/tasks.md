## 1. Protected CI policy

- [x] 1.1 Add focused `scripts/validate-ci-gates.test.ts` cases for the accepted numeric 180-minute rules-screen job timeout and rejection of absent, shorter, longer, and nonnumeric values.
- [x] 1.2 Set `timeout-minutes: 180` on the `rules-screen-parity` matrix job in `.github/workflows/bun-ci.yml` and require that exact value in `scripts/validate-ci-gates.ts`, retaining all existing protected checks.
- [x] 1.3 Document the three-hour per-shard job budget and its limit in `docs/ci-parity-gates.md`.

## 2. Verification and closure

- [x] 2.1 Run `bun --config=bunfig.toml --no-env-file test scripts/validate-ci-gates.test.ts scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts`, `bun --config=NUL --no-env-file run scripts/run-ci-policy.ts`, and `moon run root:openspec-validate`; record full logs and results.
- [x] 2.2 Run repository-required final checks on a stable tree: `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, `bun run test:smoke`, and `bun run apps/agent-bridge/scripts/run-python-tests.ts` (bridge commands from `apps/agent-bridge` where applicable). Record failures or environment limits explicitly.
- [x] 2.3 Verify implementation and scenario coverage with `$openspec-verify-change`, recording the passing checks and expected pre-archive gate rejection.

After implementation verification, synchronize and archive this change with `$openspec-sync-specs` and `$openspec-archive-change`, then rerun `moon run root:openspec-validate` and strict all-spec validation as required by the repository lifecycle.
