## 1. Approval and regression evidence

- [x] 1.1 Obtain explicit approval of proposal, delta specifications, design and tasks; record evidence before implementation edits.
- [x] 1.2 Add failing editor-core regressions for the exact extreme-viewBox and cancelling-component-scale reproductions, mapped to the delta scenarios. Preserve existing opacity and post-Z regressions.

## 2. SVG raster representability

- [x] 2.1 Implement shared SVG raster-coordinate conversion and backend-compatible outward-rounded bound/dimension checks in editor-core evaluation and preparation; return INVALID_ARGUMENT before artifacts without changing standalone shape output.
- [x] 2.2 Validate conservative cap/miter stroke envelopes, dash conversion/construction and intermediates without silent fallback. Test inclusive and overflow conversions, negative coordinates, viewport-crossing geometry, fill/stroke/caps/miters/dashes and valid offscreen/degenerate coverage with independent pixel expectations.

## 3. Complete occurrence validation

- [x] 3.1 Separate normalized-document checks from transformed raster sizing; validate complete component/group ancestry with existing keyframe scale bounds and clocks, including hidden occurrences and unreachable virtual roots.
- [x] 3.2 Remove premature isolated SVG surface checks from intermediate evaluation; compile using complete transforms and remaining budgets without duplicate intermediate accounting. Preserve all graph, work, surface and memory limits.
- [x] 3.3 Test nested groups/instances, legacy transforms, Transform2D, hidden content, unreachable nested definitions, differently scaled instances, exact limits and aggregate overflow. Verify cancelling scales match identity-scale pixels and truly excessive composed scenes fail.

## 4. Public and native conformance

- [x] 4.1 Synchronize applicable canonical rendering evidence before its Rust/headless/MCP consumers, without changing public wire shapes; obtain designated CODEOWNER review if governed fixtures change. Cover standalone calls and alias-bearing batches where applicable.
- [x] 4.2 Extend native SVG frame/range/draft/export and undo/redo/reopen conformance with scale cancellation and numeric rejection. Verify unchanged revision/state/history/artifacts on failure, stale-revision precedence, independent pixel expectations and existing semantic/SSIM/audio/timing tolerances.
- [x] 4.3 Update SVG documentation and requirement-to-test traceability; record actual verification commands, outcomes and limitations without unrelated golden regeneration.

## 5. Verification and completion

- [x] 5.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace -- --test-threads=1`; resolve failures.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`; resolve failures.
- [x] 5.3 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for the hermetic worker suites. With OPENCUT_GOLDEN_REQUIRED=1 and configured OPENCUT_FFMPEG_PATH, OPENCUT_FFPROBE_PATH and OPENCUT_TEST_FONT_PATH, run root `cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact`, `cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact` and `cargo test -p opencut-editor-core --test transform2d`. Missing or skipped required checks block completion.
- [x] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `git diff --check`; use openspec-verify-change, resolve all mismatches and record verification evidence.
- [x] 5.5 Use openspec-sync-specs and openspec-archive-change to synchronize and archive the verified change; then run `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` through the unchanged protected policy. Mark completion only after every required gate passes.
