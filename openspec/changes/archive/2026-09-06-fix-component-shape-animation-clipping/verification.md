# Verification: fix-component-shape-animation-clipping

## Approval and reproduction

The user approved proposal, design, delta scenarios and tasks on 2026-09-06 before implementation.

Before the fix, `cargo test -p opencut-editor-core component_animation_clips_in_output_space --lib -- --nocapture` failed: finalized visual layer count was 0 instead of 1. With the configured native toolchain, `cargo test --release -p opencut-editor-core native_component_animation_conformance --lib -- --nocapture` failed at the red-interior assertion for animated=true. The static rectangle rendered correctly. Both failures were observed before editing the evaluator.

## Implementation and traceability

The only production edit in this follow-up is `crates/editor-core/src/evaluated_scene/shapes.rs::affine`: output_canvas remains the caller's viewport, local_canvas resolves component units, recursive samples retain output_canvas, and root-space envelope intersection uses output_canvas. Matrix composition, inverse density compensation, finite checks and affine geometry limits still precede clipping. No public API, schema, migration, renderer backend, ownership edge or error contract changed.

All new tests are in `crates/editor-core/src/renderer/golden/shapes.rs`, wired through shape conformance into the existing required native golden gate.

| Delta scenario | Automated evidence |
| --- | --- |
| Preserve translated shapes with constant animation | `component_animation_clips_in_output_space`; `component_animation_conformance` reproduces the exact 10x10/40x40/(100,20)/240x120 case; `check_component_animation_rendering` compares static and constant scale/position frames exactly within each of preview, range and export at three timestamps. |
| Compose nested and retimed animation before clipping | `component_animation_cases` includes translation, scale, nested composition, direct retiming, nested retiming, trim offsets and rate=2. `component_animation_envelopes_and_local_units` checks independent clock arithmetic and corner bounds; native probes check those authored-to-root calculations at three timestamps per case. |
| Preserve visible portions at output boundaries | Left/right partial and fully offscreen cases, with component dimensions 40 and 400. Native probes check visible red portions, exterior pixels and entirely black frames. `component_animation_offscreen_geometry_still_validates` rejects oversized authored geometry, valid authored geometry magnified beyond limits, infinity and NaN before any backend call or artifact write. The same instance transform succeeds before geometry is made invalid. |
| Separate local units from requested output dimensions | A normalized companion resolves (0.5,0.5) in 40x40 to local (20,20), with exact padded matrix translation (119,39). An animated shape in the same component is at root x=180, outside persisted width=160 but inside requested width=240. Tests evaluate both viewports and compare range/export to a matching-settings preview clone; native pixels at (125,45) verify local units. |
| Existing canonical evaluation scenarios | Existing shape occurrence/clock/aggregate tests, vector and ShapeItem contract tests, fractional/degenerate anchor and density tests, native vector semantics and legacy golden gate remain required. |

Repeated finalization is checked for idempotence. Project bytes (including revision) remain equal across evaluation and rendering. Native tests also preserve project/history sentinel files; existing persistence/retained-history and migration tests remain in the workspace and contract gates.

## Native comparison policy

No golden reference or existing tolerance was changed. Static/constant-keyframe comparisons are exact decoded pixel equality within each intent. Cross-codec comparisons retain SSIM >= 0.99. The original preview reproduction requires red > 240 and green < 10. New encoded interior probes require red > 220 and green/blue < 35; a first run demonstrated H.264's solid interior [239,0,19], so these new probes explicitly account for codec chroma quantization. Probes avoid geometry edges. Fully offscreen and distant exterior pixels remain < 10 per channel.

Native environment uses FFmpeg/FFprobe 8.1.2 from local-data/transform2d-tools/github-build/ffmpeg-8.1.2-essentials_build/bin and the repository DejaVuSans.ttf, with OPENCUT_GOLDEN_REQUIRED=1. No update/recapture variable is set.

## Gate results

Logs are local evidence under ignored `local-data/component-clipping-verification/`.

- PASS: focused evaluator regressions (`cargo test -p opencut-editor-core component_animation_ --lib -- --nocapture`); conditional native test is separately enforced by the release gate.
- PASS: `cargo fmt --check --all` and `cargo clippy --workspace --all-targets -- -D warnings`, including the final test additions.
- PASS: `cargo test --workspace`, rerun after the final test additions.
- PASS: `cargo test --release -p opencut-editor-core native_golden_render_conformance --lib -- --nocapture` (98.58 seconds), including the final constant-position fixture, all new native cases and unchanged legacy goldens.
- PASS: `bun run typecheck`, `bun run lint`, `bun run test` (380 tests), from apps/agent-bridge.
- PASS: `bun run contracts:check` (Rust protocol/vector/shape fixtures and 316 TypeScript parity tests).
- PASS: from apps/kokoro-tts, `bun run ../agent-bridge/scripts/run-python-tests.ts` (10 unittest and 5 pytest cases).
- PASS: pinned OpenSpec strict validation (21 items) and `git diff --check`.
- PASS: `bun run test:integration` from apps/agent-bridge (10 source MCP tests).
- PASS: `bun run test:smoke` from apps/agent-bridge (5 packaged MCP tests).
- PASS: synchronized living shape requirements and archived this follow-up, preserving both earlier archives.
- PASS: `bunx @moonrepo/cli@2.3.3 run root:openspec-validate` after archival (20 specs plus CI parity policy).

The workspace retains six pre-existing ignored helper/maintenance entries: four isolated subprocess helpers invoked by their parent tests, an explicit golden recapture command, and an external report-only validator. The latter two are not required checks for this fix; no reference recapture or external performance report is requested. Required native rendering runs separately in release mode and passed.

## OpenSpec verification assessment

Completeness: 13/13 tasks complete; all implementation, regression, required validation and lifecycle tasks passed.
Correctness: one modified requirement, all four new scenarios mapped above; existing scenarios retained.
Coherence: the production fix follows the approved separation of coordinate dimensions and preserves core ownership. New tests use the existing native test owner and unchanged reference policy.

No critical, warning or suggestion discrepancies remain in code, tests, requirement coverage or design. All required checks passed. Living requirements are synchronized, this change is archived, and the post-archive Moon gate passed. No required check remains failed or skipped.
