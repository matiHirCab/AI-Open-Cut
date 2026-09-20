## Context

Issue #35 depends on rich documents (#32) and pinned shaping (#33), both closed. The current model has schema-20 span paints and legacy wrapWidthPx, lineSpacingPx, alignment, background color/opacity and padding. At proposal time there was no active approved change. The user approved this design on 2026-09-19; check tasks and verification evidence for implementation status.

## Goals / Non-Goals

Deliver the seven requested layout controls through core-owned evaluation, deterministic persistence and typed agent workflows. Preserve legacy text exactly. Non-goals are listed in proposal.md; in particular this does not add span-level metrics, paragraph spacing, layout animation, clipping or a desktop form.

## Decisions

1. **Opt-in nested contract.** Add optional TextStyle.layout using the exact fields/defaults/limits in the advanced-text-layout delta. Omission preserves the existing branch, including opencut-text-v2 font bindings. Layout is an explicit persisted extension over that shaping profile, not a reinterpretation of legacy projects. Rejected: changing global defaults or scattering new precedence rules over legacy fields, which risks changing existing renders.
2. **Core layout ownership.** Keep reference-free models in model, domain checks in validation, cluster metrics in fonts/shaping, resolved box/fit semantics in evaluated_scene, and rasterization in render_artifact consuming evaluated plans. Preserve ADR 0003 dependencies; do not add outer-layer validation or backend-specific fitting. Bind fonts before layout through existing resource preparation. Rejected: FFmpeg expressions or browser measurements, which cannot ensure pinned shared semantics.
3. **Exact bounded search.** Evaluate integer sizes from the mode's maximum down to 1, stopping at the first fitting candidate. This matches the existing integer font-size range and guarantees the largest result even when wrapping/metrics produce non-monotonic measurements. Rejected: binary search without a monotonicity proof and arbitrary floating-point iteration. Account for candidate glyph work across the entire scene before render output work. Candidate measurements use fixed pixel tracking/line height/padding; all modes preserve authored values.
4. **Box geometry.** Bounds include padding; fitting measures advances and line boxes, never effect ink. This prevents stroke/shadow choices from changing typography. Effects still enlarge the raster allocation, with existing safety limits. Overflow is visible and diagnosed; no clipping or ellipsis. Rounded background uses existing color/opacity behind glyph paints and the outer padding rectangle. Rejected: shrinking background or clipping shadows to force an apparent fit.
5. **Encoding fidelity.** Advanced layout range/export uses matching medium-preset H.264 CRF 18. Native conformance found CRF 23 yielded frame/range SSIM 0.986 for fit_box with overlapping styled glyphs at low resolution. Preserve all legacy encoding choices when layout is absent. This implements the approved shared render tolerance without changing geometry, pixel format or operation contracts.
6. **Additive diagnostics and transport parity.** Return textLayouts only for enabled text through existing render diagnostic envelopes, using evaluated instance identities and paint order. Keep operation names and style replacement behavior. Update canonical ownership, style/project/headless/MCP/capability catalogs and every declared consumer; @matiHirCab reviews contract changes. Rejected: a new layout operation or transport-side geometry logic.

## Risks / Trade-offs

- Descending search can be expensive: cap candidate work globally, cache identical pure measurements where possible without changing accounting semantics, and test worst-case rejection before artifacts. Do not replace exact selection with an approximation.
- Mixed bidi/ligatures can expose placement mistakes: independently assert clusters, advances, mandatory line breaks and alignment, including paint boundaries that must not reshape text.
- Integer fitting has coarser results than fractional fitting: it preserves the current font-size contract and makes the tie rule exact.
- Fixed tracking and line height can make fitting impossible: resolve size 1, report each overflowing axis and preserve all content.
- Visual fixtures need real pinned font/render dependencies: required unavailable checks block implementation completion and are reported, never replaced by skipped tests.

## Migration Plan

Introduce schema 21 using the existing locked atomic migration mechanism; traverse current, retained undo/redo and applicable durable draft snapshots, including hidden and unused component definitions. Retain absent layout and font bindings without ambient resolution. Validate the whole candidate before atomic publication. Inject failures at publication boundaries and compare snapshots, provenance and ownership. Rollback is recovery to pre-migration data with the existing transaction mechanism; no lossy automatic downgrade is supported. Unknown future schemas remain rejected. Update version fixtures and documentation in the same implementation.

## Verification Plan

Every scenario in both delta specs requires automated coverage mapped in tasks.md. Use independent small font metric/layout expectations, lifecycle and migration fault injection, headless/MCP parity, and decoded frame/range/draft/export fixtures. Run Rust formatting, strict workspace Clippy/tests, bridge typecheck/lint/unit/contracts/MCP integration/packaged smoke and hermetic Python tests plus strict all-spec validation. Follow docs/spec-driven-development.md: run the protected gate before archival (only this active change may block), verify conformance, synchronize/archive, then rerun protected and strict all-spec gates. Save full check logs outside the repo with exit status and relevant failures.

## Open Questions

The user approved the contract and its choices on 2026-09-19. No unresolved semantic questions remain. Optional layout uses boxed storage, and the existing item paint vector uses a boxed slice to keep TimelineItem within the strict Clippy size limit; these allocation details preserve JSON serialization.
