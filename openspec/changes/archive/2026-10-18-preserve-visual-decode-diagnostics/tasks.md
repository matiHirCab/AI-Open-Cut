## 1. Scope and approval

- [x] 1.1 Verify #177/#206/#208/#213, current dependencies/comments/PRs, baseline and independent render ownership; preserve #208 and private machine boundary.
- [x] 1.2 Approve proposal, delta requirements, design and exact scenario verification under delegated user authority before implementation. No contract/schema/catalog changes required.

## 2. Core diagnostics and tests

- [x] 2.1 Add the bounded reader/failure classification and collect exit/stderr before reporting decoder errors; preserve cleanup, frame selection and existing public fields.
- [x] 2.2 Add deterministic short/interrupted/error/empty/partial reader and fake child process tests, including sanitized diagnostics, cleanup, nonzero complete output and missing executable.
- [x] 2.3 Add real synthetic 720p30 boundary control and document diagnostics, failure prioritization and private-runtime limitations.

- [x] 2.4 Resolve independent-review preservation limitation: retain bounded captured stderr on collector error, retry Interrupted, preserve the existing fallible wrapper, and add injected-reader regression coverage.

- [x] 2.5 Preserve bounded failure cleanup after empty/partial EOF with a 100ms exit-status grace, then terminate/reap; add a deterministic still-alive producer regression.

## 3. Verification and finalization

- [x] 3.1 Pass `cargo test -p opencut-editor-core visual_decode`, configured native `OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg cargo test -p opencut-editor-core --lib render_process::tests::native_visual_decode_720p30_boundary_control -- --ignored --exact`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.
- [x] 3.2 From apps/agent-bridge pass `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`; root `bun run apps/agent-bridge/scripts/run-python-tests.ts`. Run configured `cargo test --release -p opencut-editor-core --lib renderer::golden::native_golden_render_conformance -- --exact --nocapture` with OPENCUT_GOLDEN_REQUIRED=1 and the reviewed checked-in font, plus `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`; record exact commands/results/limitations.
- [x] 3.3 Pass `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; inspect `moon run root:openspec-validate` before archival, accepting only this active-change rejection. Record local results, native/runtime limitations and baseline CI separately.
- [x] 3.4 Review the final diff deeply and record requirement/scenario/test traceability, including justification for defensive standard-library pipe/wait cases that cannot be induced through a normally piped child.

## Subsequent delivery gates

After every local implementation checkbox passes, use openspec-verify-change to resolve requirement/design/task/code/test mismatches, then openspec-sync-specs and openspec-archive-change for this change only. Pass final protected Moon and strict all-spec validation. Refresh GitHub state, commit/push only verified work and open a draft PR. Require protected aggregate/native/platform statuses on the exact published head before reporting completion; distinguish baseline CI failures from regressions. Keep #177 open for private Windows reproduction. No merge/deployment. These delivery gates are sequenced after implementation verification and archival; they are not asserted complete by the implementation checkboxes.
