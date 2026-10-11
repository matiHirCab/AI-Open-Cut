## 1. Regression and implementation

- [x] 1.1 Add a real source CLI regression for clean unsuccessful diagnostics and record its failure before fixing production.
- [x] 1.2 Cover ready diagnostics, thrown doctor cleanup, each service-close failure and foreign marker preservation through the actual CLI/guard boundary.
- [x] 1.3 Correct CLI cleanup eligibility independently of report readiness and clarify restart guidance without changing crash/isolation contracts.

## 2. Verification and delivery

- [x] 2.1 Run bridge `bun run typecheck`, `bun run lint`, focused lifecycle tests and `bun run test --no-file-parallelism`; record full logs and any unavailable checks.
- [ ] 2.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, bridge `bun run contracts:check --no-file-parallelism`, `bun run test:integration`, `bun run test:smoke`, and `bun run apps/agent-bridge/scripts/run-python-tests.ts`; record unavailable checks honestly.
- [ ] 2.3 Run strict OpenSpec validation and `moon run root:openspec-validate` preflight; verify implementation/spec/design/scenario conformance before synchronization and archival. Native or other required check failures block archival.

## Post-verification workflow

These steps follow conformance verification and are tracked separately to avoid requiring archival or PR creation before the pre-archive implementation task check.

- Synchronize and archive only after all required implementation checks pass, then pass the protected post-archive gate. Status: pending.
- Push the scoped verified implementation and open a draft PR with accurate validation/CI status. Status: pending. No merge, deployment or private audit upload.
