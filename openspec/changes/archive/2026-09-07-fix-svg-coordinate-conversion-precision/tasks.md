## 1. Approval and regressions

- [x] 1.1 Obtain explicit approval of proposal, delta specifications, design and tasks; record approval before implementation edits.
- [x] 1.2 Add failing editor-core coverage for the exact diagonal review reproduction and round-trip precision boundaries; map each test to the delta scenarios.

## 2. Shared checked SVG conversion

- [x] 2.1 Retain f64 raster-space coordinates and reject non-finite values or round-trip Euclidean displacement greater than a named 0.25-pixel conversion threshold before path construction or bounds accumulation. Use the existing shared evaluation/preparation path, including offscreen and move-only contours.
- [x] 2.2 Test exact/positive/negative coordinates, inclusive threshold and adjacent excessive error, combined X/Y displacement, non-finite conversion, mapped viewport and component sampling scales, and exactly representable large coordinates. Preserve all existing backend/stroke/dash/resource/component checks and standalone/curve behavior.
- [x] 2.3 Render the shorter equivalent polygon with independent red-band and outside-band pixel expectations, including pixel (50,55); retain opacity, closepath, clipping, integer-overflow and component-scale regressions.

## 3. Public conformance and documentation

- [x] 3.1 Extend standalone headless and MCP failed-render coverage with the precision reproduction; verify typed errors, stale-revision precedence and unchanged state/revision/history. Preserve existing canonical ingestion and alias/batch evidence; update governed fixtures before consumers only if governed behavior changes and obtain designated review if needed.
- [x] 3.2 Extend native frame/range/draft/export rejection coverage and snapshot checks for unchanged project/history/draft/output files; retain accepted native SVG rendering and lifecycle conformance with independent expectations and existing visual/audio/timing tolerances.
- [x] 3.3 Document the conversion threshold separately from unchanged curve-flattening tolerance, with no combined 0.25-pixel guarantee. Maintain requirement/scenario-to-test traceability and actual verification evidence; do not regenerate unrelated golden references.

## 4. Required verification and archival

- [x] 4.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace -- --test-threads=1`; resolve failures and record results.
- [x] 4.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; resolve failures and record results.
- [x] 4.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts`. With OPENCUT_GOLDEN_REQUIRED=1 and configured OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH and OPENCUT_TEST_FONT_PATH, run root `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`, `cargo test -p opencut-editor-core --lib svg_ -- --test-threads=1`, `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`, `cargo test -p opencut-headless native_svg_numeric_rejection_preserves_state -- --exact` and `cargo test -p opencut-editor-core --test transform2d`. Required missing or skipped checks block completion.
- [x] 4.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`; use openspec-verify-change, resolve all code/design/spec/task/test mismatches and record evidence.
- [x] 4.5 Use openspec-sync-specs and openspec-archive-change to synchronize and archive the verified change, then run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` through the unchanged protected policy. Mark complete only after all required checks pass.
