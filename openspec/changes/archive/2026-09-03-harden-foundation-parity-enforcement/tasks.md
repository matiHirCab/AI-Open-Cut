## 1. Aggregate enforcement

- [x] 1.1 Make `foundation-parity` run for every terminal leaf outcome, log both dependency results, and fail unless both are exactly `success` while preserving stable job identities.
- [x] 1.2 Add a pure aggregate-result assertion and exhaustive tests proving two successes pass while `failure`, `cancelled`, and `skipped` in either leaf fail.

## 2. Workflow policy hardening

- [x] 2.1 Require exact fail-closed contract setup and parity steps with the declared working directory and no job- or step-level ignored failures.
- [x] 2.2 Require exact ordered render setup, conformance, strict report validation, and report upload steps with no ignored failures or neutralized command bodies.
- [x] 2.3 Require the aggregate's exact dependency list, unconditional execution expression, result wiring, logging, and explicit failure assertion.
- [x] 2.4 Add focused negative tests for altered `always()`, missing result checks, ignored failures, neutralized commands, and report upload before validation.

## 3. Documentation and compatibility

- [x] 3.1 Document aggregate execution and failure semantics while retaining `Motion-graphics foundation parity` as the stable maintainer-configured branch-protection target.
- [x] 3.2 Confirm public APIs, contracts, fixtures, versions, renderer behavior, golden references, persisted data, and job identities remain unchanged.

## 4. Verification and closure

- [x] 4.1 Run `bun test scripts/validate-ci-gates.test.ts`, `bun run scripts/validate-ci-gates.ts`, and `moon run openspec-validate` from the repository root.
- [x] 4.2 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root.
- [x] 4.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 4.4 Run the configured Linux native golden conformance, lifecycle, strict external-report validation, and exact report-path checks without changing canonical fixtures.
- [x] 4.5 Use `$openspec-verify-change`, resolve every mismatch, sync the accepted deltas, and archive with `$openspec-archive-change`.
