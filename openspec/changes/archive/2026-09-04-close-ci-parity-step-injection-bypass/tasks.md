## 1. Closed leaf-job policy

- [x] 1.1 Require exact positional contract- and render-parity step sequences with exact approved property sets and existing values.
- [x] 1.2 Reject workflow and leaf-job `defaults.run`, custom leaf-step shells, and leaf-job containers while preserving unrelated workflow and job environment variables.

## 2. Policy evidence and documentation

- [x] 2.1 Add negative tests for added, duplicate, missing, replaced, and reordered leaf steps, including contract/fixture mutation and both golden flags through `GITHUB_ENV`.
- [x] 2.2 Add negative tests for unexpected step properties, custom shells, workflow/leaf defaults, and leaf containers, plus positive benign inherited environment coverage.
- [x] 2.3 Document closed leaf sequences and the coordinated review required for future leaf-step changes.
- [x] 2.4 Confirm the valid workflow, check identities, application code, contracts, renderer, fixtures, references, and persisted data remain unchanged.

## 3. Verification and closure

- [x] 3.1 Run `bun test scripts/validate-ci-gates.test.ts`, `bun run scripts/validate-ci-gates.ts`, and `moon run openspec-validate` from the repository root.
- [x] 3.2 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root.
- [x] 3.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 3.4 Run configured Linux native golden conformance, lifecycle, and strict external-report validation; compare tracked fixture status before and after.
- [x] 3.5 Use `$openspec-verify-change`, resolve every mismatch, sync all three deltas, and archive with `$openspec-archive-change`.
