# Verification: fix-svg-rendering-and-path-normalization

## Scope and approval

The user explicitly approved the concrete proposal, design, delta specifications and tasks on 2026-09-07. The canonical correction scenarios follow that approved scope. CODEOWNER routing remains unchanged; no GitHub review submission, commit, merge or publication is represented as having occurred. Schema 15, document version 1 and protocol 1 remain unchanged.

## Requirement and scenario coverage

| Requirement / scenarios | Implementation | Automated evidence |
| --- | --- | --- |
| Explicit SVG geometry and complexity: supported continuation | validation/svg.rs tracks subpath initial/current points and inserts normalized moves | svg_close_continuations_and_normalized_budgets; canonical post-close-continuation fixture consumed by Rust/headless/TS/MCP |
| Explicit SVG geometry and complexity: normalized command limits | parser receives remaining document capacity; inserted moves count before insertion | svg_close_continuations_and_normalized_budgets; canonical implicit-move-path-budget; existing source/XML/numeric boundary tests |
| Explicit SVG geometry and complexity: downscaled viewport, drawing semantics | bounded compile_contours separate from standalone sizing; mapped viewport surface and stroke metrics | svg_repeated_alpha_and_downscaled_viewport_regressions; svg_mapped_limits_precision_and_remaining_segments; SVG native oracle |
| Explicit SVG geometry and complexity: actual surface/precision/work/memory boundaries | svg_with_budget, authored-document preflight, remaining expanded/refinement budgets, 44-byte pixel accounting | svg_mapped_limits_precision_and_remaining_segments checks exact dimensions/area, one-over limits, mapped curve/stroke/dash values, underflow, exact remaining work and aggregate bytes; svg_hidden_and_unused_content_cannot_bypass_mapped_limits |
| Shared evaluated SVG rendering: repeated translucent composition | reusable coverage masks and premultiplied linear-light f64 accumulator; one PAM encoding | svg_repeated_alpha_and_downscaled_viewport_regressions compares 500 layers to analytical alpha; svg_float_fill_stroke_and_sibling_order checks independently calculated ordered RGBA; existing clipping/linear-light test |
| Shared evaluated SVG rendering: public/lifecycle/intent parity | unchanged headless/MCP forwarding, common evaluated source across output intents | canonical core/headless/bridge fixtures and both MCP smoke workflows; updated native SVG fixture spans downscale, post-close path, translucent stack, item opacity, retimed component, frame/range/draft/export, undo/redo/reopen and synthetic audio |
| Preserved compatibility, errors and security | unchanged wire types, schema and structured VectorPath validator; bounded offline parsing | workspace contracts/migration/history/architecture suites; canonical invalid inputs, stale revisions and batch rollback; complete legacy native reference suite |

The reviewed path and alpha failures were reproduced before correction: the new opacity test produced [255,0,0,128] instead of [255,0,0,219], and the canonical post-close request failed with drawing requires moveTo. Review also reproduced the source-space surface rejection through the headless renderer. All corrected cases now have automated assertions.

Native color edges align to the established video chroma sampling grid; the independent reference is still derived from source coordinates and analytical opacity. No SSIM/audio/timing threshold or unrelated golden reference was changed. Temporary mismatch diagnostics were removed. Coverage allocations now use try_reserve_exact and Pixmap::from_vec; physical memory exhaustion is not induced in tests.

## Executed checks

- Rust formatting and strict workspace Clippy: passed.
- Serial workspace tests: passed, including 227 core unit tests, all integration suites, desktop and headless. Six existing explicit helper/recapture/report entry points remain intentionally ignored; native validation runs separately with required dependencies.
- Bridge typecheck and lint: passed. Bridge unit tests: 382 passed.
- Canonical contracts gate: passed, including Rust/headless and 318 TS parity tests.
- MCP integration: 10 passed. Packaged runtime smoke: 5 passed.
- Hermetic Python workers: 10 Kokoro and 5 transcription tests passed.
- Pinned strict OpenSpec validator: 22 items passed before synchronization/archival.
- git diff --check: passed.
- Updated native SVG conformance: passed with configured FFmpeg/FFprobe 8.1.2, DejaVuSans and OPENCUT_GOLDEN_REQUIRED=1.

## Final verification

The complete required native golden suite passed in 247.13 seconds, preserving all unrelated reference gates. Native headless lifecycle passed and configured Transform2D passed all 13 tests. After the final SVG-only coverage allocation adjustment, strict workspace Clippy and focused SVG unit/integration/native tests were rerun successfully. No required check remains failed or skipped; the six existing helper entry points retain their intended harness behavior.

Both modified requirements have automated scenario evidence. The implementation follows the approved owning-layer design; no unresolved correctness or coherence mismatch remains. The two updated requirements were synchronized into the living SVG specification and this follow-up was archived on 2026-09-07. The protected Moon 2.3.3 root:openspec-validate gate passed: 231 policy tests, 21 living specifications and CI parity policy validation. All 16 tasks are complete.
