## 1. Approval and regression contracts

- [x] 1.1 Obtain explicit approval of these proposal/design/delta/tasks artifacts and record evidence in proposal.md before implementation edits.
- [x] 1.2 Update canonical SVG fixtures with corrected acceptance and budget failures before consumers; add failing core regressions for all three review findings and map them to the delta scenarios. Preserve schema 15/document 1/protocol 1 and obtain designated CODEOWNER review for affected contracts and consumers.

## 2. Source normalization

- [x] 2.1 Track closed-subpath initial/current points and insert MoveTo before post-Z L/H/V/Q/C; preserve explicit M and existing unsupported-command rejection. Replace incorrect rejection expectations and verify independent normalized coordinates.
- [x] 2.2 Enforce remaining path/document capacity before synthetic moves and primitive/path expansion. Test exact limits and one-over-limit failures, with revision precedence and batch rollback evidence for the command-budget scenarios.

## 3. Evaluated geometry and resource budgets

- [x] 3.1 Separate bounded contour compilation from standalone surface sizing without changing standalone shapes; map SVG geometry/stroke/dash before raster precision validation and size the real viewport without dummy padding. Test downscaled/nonzero-origin/clipped geometry, curves, strokes and magnification.
- [x] 3.2 Propagate remaining scene capacity and preflight actual viewport dimensions/area, mapped precision and 44-byte/pixel memory accounting before expansion/allocation. Test inclusive/overflow bounds, hidden content and component expansion while retaining existing global limits.

## 4. Raster composition

- [x] 4.1 Add SVG premultiplied linear-light f64 accumulation with reusable fill/stroke coverage and one final PAM encoding. Keep item/inherited opacity in its existing layer stage and use checked/fallible large allocations.
- [x] 4.2 Verify analytical alpha for 500 #ff000001 rectangles, mixed colors, fill/stroke overlap and document order within one encoded channel value. Verify standalone shape encoding remains unchanged.

## 5. Public and native conformance

- [x] 5.1 Consume canonical regressions through Rust headless and bridge structural/parity tests, standalone MCP and alias-bearing batches; verify invalid input and stale revisions preserve typed errors, revision and state/history.
- [x] 5.2 Extend native SVG fixtures across frame/range/draft/export and undo/redo/reopen with all three corrections, item opacity, transformed/retimed components, independent pixel/geometry oracles and synthetic audio. Retain exact plans, SSIM >=0.99, PCM RMS <=0.0001 and one-frame timing tolerance without regenerating unrelated references.
- [x] 5.3 Update SVG documentation for normalization, mapped limits and composition; maintain requirement/scenario-to-test traceability and actual verification evidence.

## 6. Required verification and archival

- [x] 6.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` (use `-- --test-threads=1` if needed for the existing process sampler); resolve failures and record results.
- [x] 6.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; resolve failures and record results.
- [x] 6.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`. From root, with OPENCUT_GOLDEN_REQUIRED=1 and configured OPENCUT_FFMPEG_PATH/OPENCUT_FFPROBE_PATH/OPENCUT_TEST_FONT_PATH, run `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`, `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`, and `cargo test -p opencut-editor-core --test transform2d`. Missing dependencies or skipped required checks block completion.
- [x] 6.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`. Use openspec-verify-change and resolve every implementation/design/spec/task/test mismatch, recording all actual results and limitations.
- [x] 6.5 Use openspec-sync-specs and openspec-archive-change to synchronize accepted deltas and archive this verified follow-up, then run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` through the unchanged protected policy. Mark complete only when all required checks pass.
