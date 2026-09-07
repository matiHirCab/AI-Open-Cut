# Verification: fix-grid-clipping-and-mcp-rollback

Date: 2026-09-07. The user explicitly authorized implementation of the complete fix plan before changes. No public contracts, schema version, migrations, dependencies or encoding settings changed. The original add-procedural-grids archive remains historical evidence.

## Scorecard

| Dimension | Evidence |
| --- | --- |
| Completeness | Implementation and regressions cover both review findings. All seven tasks are complete, including verification, synchronization, archive and final Moon validation. |
| Correctness | All three requirements and seven scenarios have automated evidence below; no scenario is exempted from automation. |
| Coherence | Core evaluation resolves bounded coverage contours; render_artifact consumes existing fill facts without reconstructing grid semantics. Shared shape/SVG rendering paths are preserved. |

## Requirement-to-test mapping

| Requirement and scenarios | Automated evidence |
| --- | --- |
| Geometric fractional grid coverage / Preserve partially covered edges | `render_artifact::shapes::tests::grid_fractional_edges_preserve_partial_stroke_coverage`: right and bottom edges, unchanged half coverage at width 11, and source-over corner coverage. The regression first failed with alpha 64 instead of 128, then passed after geometric clipping. `grid_clipped_dashes_caps_paints_and_empty_coverage` covers butt/round/square caps, dash phase, empty dash coverage, dots, opacity, gradients and scaled edge pixels. |
| Geometric fractional grid coverage / Preserve transformed clipping | `evaluated_scene::shapes::grids::tests::grid_all_patterns_clip_coverage_before_scaled_rasterization` checks all four patterns, every cap, dashes, scales 1 and 2.5, local viewport bounds and deterministic evaluated results. Existing native component/draft/static-animation tests remain active. |
| Geometric fractional grid coverage / Bound generated coverage | `grid_clipping_oracle_orientation_and_peak_budget` uses an independently specified clipped triangle, opposite winding and exact peak budget acceptance/rejection. `grid_all_patterns_clip_coverage_before_scaled_rasterization` and `grid_sampling_and_segment_budgets_fail_closed` verify exact accumulated budgets. The native fixture accepts a descriptor at the mark limit but rejects expanded round-cap work before export-collision handling, preserving destination, project and history bytes. |
| MCP grid rollback reaches domain execution / Fail after creating a grid; Reject a stale revision | `apps/agent-bridge/tests/grid-workflow.ts` now uses valid `delete_item` after `add_grid`, checks structured ITEM_NOT_FOUND/non-retryable and REVISION_CONFLICT/retryable, and compares complete project state including revision. The same helper executes in the source integration and rebuilt packaged smoke suites. |
| Fractional grid render conformance / Render fractional edges through shared evaluation; Preserve failure and legacy guarantees | `renderer::golden::grids::native_grid_render_conformance` adds the reproduced fractional white edge, asserts RGB 128 and exterior black, compares frame/range/export at existing sizes and draft equality, retains SSIM >=0.99, PCM RMS <=0.0001 and one-frame timing thresholds, and checks preflight failure before side effects. The fixture also runs inside full `native_golden_render_conformance`. |

## Implementation audit

Straight grid strokes use the existing tiny-skia dash and outline conventions. Each dash outline is generated separately, bounding temporary stroker geometry. The shared compiler flattens curves at 0.25/density, and Sutherland-Hodgman clips closed polygons while preserving winding. Source, outline and peak clipping work all consume the existing grid/scene budgets; vertex appends check their remaining budget. Empty off-dash coverage is valid. Clipped strokes become evaluated filled coverage with their original local paint. No final viewport-area multiplication remains.

The original lattice oracle now checks retained line geometry while checking dot coverage points against the circle or viewport boundary; clipping has intentionally changed coverage contours. Independent new geometry/pixel tests check the corrected representation rather than blessing changed rendered output. No golden tolerance was weakened.

## Checks

- Passed: `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Passed: `cargo test --workspace -- --test-threads=1`, including architecture and migration/recovery suites. Existing ignored helper/recorder tests remain ignored; required native checks run separately with backend enforcement.
- Passed: bridge `bun run typecheck`, `bun run lint`, `bun run test` (384 tests / 18 files).
- Passed: bridge `bun run contracts:check` (Rust/headless suites plus 320 TypeScript tests / 5 files).
- Passed: `bun run apps/agent-bridge/scripts/run-python-tests.ts` (10 Kokoro and 5 Whisper hermetic tests).
- Passed: focused `cargo test -p opencut-editor-core native_grid_render_conformance --lib -- --nocapture` with required compatible backend (8.78 seconds).
- Passed: source `bun run test:integration` (10 tests) and rebuilt packaged `bun run test:smoke` (5 tests), including structured error and unchanged-state assertions.
- Passed: full `native_golden_render_conformance` with required backend (495.01 seconds), including legacy fixtures and fractional grids.
- Passed: `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` after archive: 22 living specifications and CI parity policy passed.
- Passed: final `git diff --check`.
- Passed: pinned OpenSpec 1.5.0 strict validation of this active change; final living-spec validation runs through Moon after archive.

Native checks resolve absolute paths from `local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin/ffmpeg.exe`, its sibling `ffprobe.exe`, and `crates/editor-core/tests/fixtures/fonts/DejaVuSans.ttf`; `OPENCUT_GOLDEN_REQUIRED=1`. Full native command: `cargo test -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture`.

Rustfmt initially encountered Windows error 1224 on an open mapped source file. Formatting the scoped files via rustfmt stdout and atomic replacement resolved it; the subsequent workspace format check passed. This did not bypass or suppress formatting checks.

Ignored local logs: `local-data/grid-fix-workspace.log`, `grid-fix-contracts.log`, `grid-fix-integration.log`, `grid-fix-packaged.log`, `grid-fix-native.log` and `grid-fix-moon.log`.

## Closure

No implementation/design mismatch identified. All behavior requirements and automated gates passed. The approved change is verified, synchronized and archived. Final Moon validation and `git diff --check` passed; no unresolved conformance findings remain.
