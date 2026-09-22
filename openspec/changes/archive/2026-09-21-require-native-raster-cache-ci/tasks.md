## 1. Approval and baseline

- [x] 1.1 Obtain explicit approval of this proposal, design, both delta specs and tasks; record approval before implementation. Preserve the issue-36 implementation, both prior archives and unrelated work.
- [x] 1.2 Record scenario-to-test traceability and reusable check evidence in verification.md. Confirm pinned Bun 1.4.0, Moon 2.3.3, Rust 1.97.0 and actual native tools/font. No contract or migration changes are authorized.

## 2. Required workflow and policy

- [x] 2.1 Add the locked bridge installation and serial native raster-cache step to Render parity with exact required-mode environment, core and instrumented worker/bridge commands, default rebuild and default headless tests (C1-C3, P1).
- [x] 2.2 Extend the closed policy sequence and exact commands/environments/working directories; retain existing failure, isolation, golden and report guards (P1-P3).
- [x] 2.3 Add mutation regressions for each omitted/substituted cache command, missing feature/flag/setup, wrong workspace, removed/reordered default restoration and masked/conditional failures; update existing sequence-index assertions without weakening them (P1-P3).
- [x] 2.4 Update docs/ci-parity-gates.md and docs/render-regression-fixtures.md with mandatory CI coverage, exact local reproduction, default restoration and remaining platform limitations (C1-C3).

## 3. Verification

- [x] 3.1 Run `bun --config=bunfig.toml --no-env-file test scripts/validate-ci-gates.test.ts scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts`, retaining full logs and exits. Exercise the supported real-Moon regression; report platform-specific skips explicitly.
- [x] 3.2 Execute the design's complete native cache command sequence with required flags, compatible local FFmpeg/FFprobe 7.1.1 and the reviewed DejaVuSans.ttf. Confirm actual test counts, native bridge diagnostics and default restoration. Do not change goldens or assertions. Record that Windows execution is not remote Linux CI evidence.
- [x] 3.3 Run `cargo fmt --check --all` and `git diff --check`. Audit valid unchanged-input evidence for required `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, bridge `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`, and `bun run scripts/run-python-tests.ts`; rerun checks whose inputs/environment changed or whose evidence is unavailable. Record reuse and failures explicitly.
- [x] 3.4 Run strict pinned `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`. Before archival only rejection naming this active change is expected; any other failure blocks progress.
- [x] 3.5 Apply openspec-verify-change; reconcile C1-C3/P1-P3 with code, policy mutations, docs, tasks and logs. Resolve all mismatches before archival.

## 4. Finalization

- [x] 4.1 Synchronize with openspec-sync-specs and archive with openspec-archive-change, preserving prior archives.
- [x] 4.2 Pass the unchanged protected Moon gate and strict all-spec validation after archival, then record completion and limitations. Do not commit, push, create a PR or dispatch remote CI.
