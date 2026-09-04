## 1. Workflow policy evidence

- [x] 1.1 Add a structural CI-policy validator for the dedicated `contract-parity`, `render-parity`, and aggregate foundation jobs, including dependency, command, deterministic render environment, strict report validation, and exact upload-path assertions.
- [x] 1.2 Add focused automated cases proving the policy validator accepts the repository workflow and rejects representative missing-job, missing-dependency, weakened-command, and mismatched-report-path inputs.
- [x] 1.3 Wire the policy validator into `moon run openspec-validate` so a workflow weakening fails an existing required repository-validation status.

## 2. Dedicated parity jobs

- [x] 2.1 Extract `bun run contracts:check` into a stable Ubuntu `contract-parity` job with pinned toolchain and frozen JavaScript dependency setup; remove only its duplicate from the broader correctness matrix.
- [x] 2.2 Extract configured native golden conformance, edit/undo/redo/reopen lifecycle coverage, strict external-report validation, and exact report upload into a stable Ubuntu `render-parity` job; remove only the duplicate native parity work from packaged smoke.
- [x] 2.3 Add the stable aggregate foundation job with explicit dependencies on both parity jobs and verify it cannot succeed when either dependency does not pass.

## 3. Documentation and compatibility

- [x] 3.1 Document the three CI status names, the aggregate branch-protection target, deterministic Linux dependencies, exact local reproduction commands, and the maintainer step needed to activate branch protection.
- [x] 3.2 Confirm canonical contract catalogs, Rust/TypeScript/Zod/MCP declarations, capability/version reporting, stable errors, project schema/history, golden generations, tolerances, and renderer output remain unchanged.

## 4. Verification and closure

- [x] 4.1 Run the CI-policy validator and its focused tests, then run `moon run openspec-validate` from the repository root.
- [x] 4.2 Run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, and `bun run test` from `apps/agent-bridge`.
- [x] 4.3 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 4.4 Run `bun run test:integration` and `bun run test:smoke` from `apps/agent-bridge`, documenting any environment-dependent exception rather than silently skipping it.
- [x] 4.5 With explicit FFmpeg, FFprobe, deterministic font, required-gate, and absolute report-path environment variables, run `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`, `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`, and `cargo test -p opencut-editor-core renderer::golden::validate_external_performance_report -- --ignored --exact`.
- [x] 4.6 Use `$openspec-verify-change`, resolve every requirements/design/tasks/tests/workflow mismatch, obtain designated owner review, and archive with `$openspec-archive-change` so accepted deltas update the living specifications.
