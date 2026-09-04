## 1. Close remaining execution surfaces

- [x] 1.1 Reject foundation command defaults and containers, and require the aggregate assertion's exact approved properties and environment.
- [x] 1.2 Reject inherited `BASH_ENV` at workflow and every parity-job scope while preserving benign environment metadata.

## 2. Policy evidence and documentation

- [x] 2.1 Add aggregate regression tests for custom shells, defaults, containers, extra properties, and missing, additional, or altered result variables.
- [x] 2.2 Add literal and expression-valued `BASH_ENV` regression tests for workflow, contract, render, and foundation scopes, plus positive benign environment coverage.
- [x] 2.3 Document the closed aggregate execution model and inherited Bash startup prohibition.
- [x] 2.4 Confirm the checked-in workflow, check identities, contracts, renderer, fixtures, references, and persisted data remain unchanged.

## 3. Verification and closure

- [x] 3.1 Run `bun test scripts/validate-ci-gates.test.ts`, `bun run scripts/validate-ci-gates.ts`, and `moon run openspec-validate` from the repository root.
- [x] 3.2 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root.
- [x] 3.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 3.4 Run configured Linux native golden conformance, lifecycle, and strict external-report validation; compare tracked fixture status before and after.
- [x] 3.5 Verify requirements, design, tasks, tests, documentation, and implementation agree; sync all three deltas and archive the change.
