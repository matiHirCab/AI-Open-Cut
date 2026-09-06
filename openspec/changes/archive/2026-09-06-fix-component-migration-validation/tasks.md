## 1. Approval

- [x] 1.1 Obtain explicit approval of proposal, design, delta specification and tasks before implementation; record it in proposal.md.

## 2. Core regressions and fix

- [x] 2.1 Add failing source-transform rejection tests for schemas 11–12 across current/undo/redo, non-default x/y position, scale and opacity, and hidden/unused definitions. Assert INVALID_ARGUMENT, byte-identical project/history and unchanged typed migration inputs (Reject forbidden source transforms across current and history).
- [x] 2.2 Add positive tests for default legacy transforms, supported transform2d, valid schema-13 non-default transforms, mixed history and repeated reopen (Preserve valid transform migration and schema-13 behavior).
- [x] 2.3 Implement the core source-schema guard before version changes, preserving existing root rejection, unsupported-version errors, clone atomicity, normalization and post-migration validation. Run the new focused tests and retain migration fault coverage (all delta scenarios).

## 3. Validation and completion

- [x] 3.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --test-threads=1`. Enable OPENCUT_GOLDEN_REQUIRED=1, supported FFmpeg/ffprobe paths and OPENCUT_TEST_FONT_PATH for native coverage; also run `cargo test -p opencut-editor-core --test component_evaluation -- --test-threads=1`.
- [x] 3.2 From apps/agent-bridge run `bun run contracts:check`, `bun run lint`, `bun run test:unit`, `bun run test:integration` and `bun run test:smoke`. From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`. Record every failed or skipped required check.
- [x] 3.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`; use openspec-verify-change and record requirement/scenario-to-test evidence. Resolve all mismatches.
- [x] 3.4 Use openspec-archive-change to merge the accepted delta and archive this follow-up, preserving the original archive. Run `moon run root:openspec-validate` on the archive-only tree and record the result before completion.
