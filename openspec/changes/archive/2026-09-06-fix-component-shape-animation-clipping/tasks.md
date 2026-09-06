## 1. Approval and failing regressions

- [x] 1.1 Obtain explicit approval of this proposal, design, delta spec and task list before changing implementation or tests.
- [x] 1.2 In core shape evaluation/native tests, reproduce `Preserve translated shapes with constant animation`: compare the static 10x10 red rectangle in a translated 40x40 component on 240x120 output with constant scale keys of 1. Record the pre-fix failure, independently expected bounds and red/background probes.

## 2. Core correction and scenario coverage

- [x] 2.1 Separate local and output canvas dimensions in core's shape affine evaluator. Preserve the original output dimensions through recursive envelope samples, compose transforms before clipping, and retain geometry validation before clipping. Trace to all four new delta scenarios; preserve timing, density and existing limits.
- [x] 2.2 Cover `Compose nested and retimed animation before clipping` with animated position/scale in translated, scaled and nested instances, including trims and timeScale=2. Assert independently calculated clocks, bounds and visible pixels at multiple timestamps.
- [x] 2.3 Cover `Preserve visible portions at output boundaries` with partial/full offscreen occurrences and components larger/smaller than output; assert oversized and non-finite geometry still fails before render execution or writes.
- [x] 2.4 Cover `Separate local units from requested output dimensions` with normalized component positions and requested dimensions different from project settings. Check independently expected placement and matching preview/range/export samples, using a matching-settings project clone for preview where necessary.
- [x] 2.5 Wire the component-animation native cases into the release native conformance gate. Verify exact static/constant-keyframe same-intent image equality, independent pixel probes, read-only project/history/revision behavior and existing cross-codec tolerances. Preserve stored golden references and prior regression fixtures.

## 3. Required validation and evidence

- [x] 3.1 Run focused new core regression tests and record exact commands, pre-fix failures and post-fix results in verification.md. Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` from the repository root; resolve every failure.
- [x] 3.2 Run `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` with OPENCUT_GOLDEN_REQUIRED=1 and the configured FFmpeg, FFprobe and test-font paths. Confirm the new reproduction passes and existing native references/tolerances remain unchanged.
- [x] 3.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test` and `bun run contracts:check`. Record results and confirm no public contract or schema change.
- [x] 3.4 After Rust builds finish, run `bun run test:integration` and `bun run test:smoke` from apps/agent-bridge for source and packaged MCP behavior. From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for required hermetic worker checks.

## 4. Verification and archival

- [x] 4.1 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`. Use openspec-verify-change to reconcile every requirement/scenario, design decision, task and test with code and recorded evidence; resolve all discrepancies and explicitly report failed or skipped required checks as blocking.
- [x] 4.2 Synchronize the accepted delta and archive this verified follow-up using openspec-sync-specs and openspec-archive-change. Keep both previous archives intact; run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` after archival and report completion only when all required gates pass.
