## 1. OpenSpec contract

- [x] 1.1 Propose and approve `fix-golden-report-artifact-path` with workspace-anchored Linux report requirements and unchanged runtime semantics.

## 2. Linux workflow

- [x] 2.1 Set `OPENCUT_GOLDEN_REPORT_PATH` to the GitHub workspace `target` artifact and create its parent before native conformance.
- [x] 2.2 Keep schema validation and artifact upload consuming the same workspace-relative report.

## 3. Verification

- [x] 3.1 Reproduce the Linux workflow golden and lifecycle commands, then validate the schema-2 report fields and selected digest.
- [x] 3.2 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 3.3 Run `bun run contracts:check`, lint, unit, integration, packaged smoke, and `bun run scripts/run-python-tests.ts` from `apps/agent-bridge`.
- [x] 3.4 Run `bunx @moonrepo/cli@2.3.3 run openspec-validate` and `git diff --check`.

## 4. OpenSpec completion

- [x] 4.1 Verify implementation completeness, correctness, and design coherence with `$openspec-verify-change`.
- [x] 4.2 Synchronize `render-regression-fixtures` with `$openspec-sync-specs` and archive with `$openspec-archive-change`.
