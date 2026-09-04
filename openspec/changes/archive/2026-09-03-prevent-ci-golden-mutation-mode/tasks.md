## 1. Verification-only environment policy

- [x] 1.1 Require the native conformance step to use exactly the five approved environment keys and the report-validation step to use exactly its approved report path.
- [x] 1.2 Reject `OPENCUT_UPDATE_GOLDENS` and `OPENCUT_CAPTURE_GOLDENS_TO` at workflow, render-job, native-step, and report-validation-step scopes while permitting unrelated inherited configuration.

## 2. Policy evidence and documentation

- [x] 2.1 Add negative tests for both forbidden mode variables at workflow, job, and critical-step scopes, including value-independent rejection.
- [x] 2.2 Add exact-map rejection tests for unexpected native and validation environment keys plus a positive unrelated-global-variable case.
- [x] 2.3 Document required CI as verification-only and preserve local deliberate update and capture workflows.
- [x] 2.4 Confirm the valid workflow, stable job identities, public contracts, renderer behavior, fixtures, references, and persisted data remain unchanged.

## 3. Verification and closure

- [x] 3.1 Run `bun test scripts/validate-ci-gates.test.ts`, `bun run scripts/validate-ci-gates.ts`, and `moon run openspec-validate` from the repository root.
- [x] 3.2 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`, plus `bun run apps/agent-bridge/scripts/run-python-tests.ts` from the repository root.
- [x] 3.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 3.4 Run configured Linux native golden conformance, lifecycle, and strict external-report validation; compare tracked fixture status before and after.
- [x] 3.5 Use `$openspec-verify-change`, resolve every mismatch, sync the accepted deltas, and archive with `$openspec-archive-change`.
