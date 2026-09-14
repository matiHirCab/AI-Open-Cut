## 1. Approval and regressions

- [x] 1.1 Obtain explicit approval of proposal, design and scenarios before test-harness edits.
- [x] 1.2 Add configuration regressions before implementing the guard; record CI run 34871014552 as failing integration evidence.

## 2. Test harness

- [x] 2.1 Apply explicit optional/required configuration to both native font tests, preserving every existing assertion and failing on unusable configured dependencies.
- [x] 2.3 Add failing policy mutations, then extend required native CI and exact-command enforcement; run `bun test scripts/validate-ci-gates.test.ts`.
- [x] 2.2 Run `cargo +1.97.0 test -p opencut-editor-core --test font_resolution` unconfigured and configured with required FFmpeg 7.1.1; verify missing/partial/invalid configuration failures and record scenario traceability.

## 3. Verification and finalization

- [x] 3.1 Run `cargo +1.97.0 fmt --all -- --check`, `cargo +1.97.0 clippy --workspace --all-targets -- -D warnings`, and `cargo +1.97.0 test --workspace`.
- [x] 3.2 With RUSTUP_TOOLCHAIN=1.97.0, run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke` in apps/agent-bridge; run `bun run apps/agent-bridge/scripts/run-python-tests.ts` from root.
- [x] 3.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `bunx @moonrepo/cli@2.3.3 run root:openspec-validate`; verify conformance with openspec-verify-change and record blockers, including unrelated active changes.
- [x] 3.4 Synchronize and archive only when repository gates permit; rerun Moon and strict validation after archival. Prepare scoped fixes for PR #119; track remote execution separately in verification.md.
