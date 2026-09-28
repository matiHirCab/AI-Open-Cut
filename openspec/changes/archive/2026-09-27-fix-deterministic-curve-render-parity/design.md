## Context

The approved issue-39 implementation stores validated curve records in schema 23, evaluates them in editor-core, and translates evaluated keyframes into FFmpeg expressions in `render_plan`. The retained FFmpeg 6.1.1 native suite fails the new Bézier draft-frame assertion and a Bézier position-Y assertion; two legacy/native controls pass. The same new draft/frame/range/export test passes with retained FFmpeg 8.1.2. A standalone simple Bézier expression evaluates equally on both binaries, so the defect must be localized against the complete affine filter path before changing its syntax. Separately, `format_number` rounds every parameterized curve constant to six fractional digits, which turns a valid near-critical spring frequency into zero. The living animation and rendering specs require deterministic scalar and decoded-output parity.

## Goals / Non-Goals

**Goals:** Restore correct Bézier visual output with supported FFmpeg 6 and 8, preserve exactly 40 midpoint inversion steps, emit new-curve numeric constants with enough precision for `1e-9` fixed scalar agreement, and demonstrate matching frame, audiovisual range, draft, and export output. Preserve legacy output and the existing error/preflight behavior.

**Non-Goals:** Change accepted requests, parameter bounds, the persisted schema, capability reporting, legacy easing, FFmpeg 9 command-line compatibility, or unrelated golden references.

## Decisions

1. **Serialize only Bézier filter graphs.** Diagnosis found that FFmpeg 6 renders irregular pixels when the full affine graph evaluates the Bézier `st`/`ld` register program concurrently. The same scene becomes correct with `-filter_complex_threads 1`; a standalone scalar expression agrees across FFmpeg 6 and 8. `render_plan` will mark plans containing an evaluated cubic Bézier curve with a private boolean, and `render_process` will add `-filter_complex_threads 1` only for those plans, regardless of FFmpeg version. Plans without Bézier retain their existing command and thread policy. The 40 midpoint steps, generated expression, evaluated-scene semantics, and canonical data remain unchanged. A stateless pure expression would duplicate dependent low/high states exponentially across 40 steps; a per-frame lookup would scale with duration. Both alternatives are rejected. The private flag is computed from typed evaluated keyframes, never from caller text or a string search over the filter graph.
2. **Use a separate full-precision formatter for parameterized segments.** Emit finite curve control points, spring coefficients, and segment endpoint values with 17 fractional digits, sufficient to retain the input `f64` precision under the declared bounds. Keep the existing six-decimal `format_number` path for `hold`, `linear`, legacy easing, and unrelated rendering. Use the same formatted constants in visual and gain expressions; preserve exact stored endpoint selection and property-bound clamping. This avoids a broad filter-graph/golden change from altering the legacy formatter. Reject any non-finite coefficient before artifact work rather than serializing `NaN` or infinity into a backend expression.
3. **Compare scalar expressions independently from encoded media.** Exercise generated expressions at fixed item-local times using a 64-bit numeric FFmpeg output path, then compare with the editor-core sampler within `1e-9`. Include Bézier boundaries, underdamped, critical, overdamped, and damping immediately below/above critical; specifically include `mass=1`, `stiffness=100`, `damping=19.99999999999999`, `initialVelocity=0` at normalized time `0.5`. Decoded frame/range/draft/export comparisons use the existing SSIM, audio RMS, and timing tolerances, not an 8-bit pixel threshold as a scalar-precision proxy. The test must use configured native tools and fail rather than skip in the protected render job.
4. **Keep ownership and compatibility local.** Editor-core remains the only sampler and compiler. Render planning owns the typed Bézier presence flag; process execution only translates that private flag into a command option. No transport or persistence contract changes are needed, so existing Rust/TypeScript/headless/MCP fixtures, schema-23 migrations, and stable error codes remain valid. Run their existing parity suites to detect unintended drift. No golden baseline is updated unless an independently reviewed semantic output change requires it.

## Risks / Trade-offs

- [FFmpeg filter expression semantics vary by version] → Require the same focused native scene to pass with retained 6.1.1 and 8.1.2, plus the protected Linux render job; compare scalar and decoded output separately.
- [More precise literals change generated graphs] → Apply the formatter only to new parameterized segments and review normalized graph differences. Legacy-only golden output must remain unchanged.
- [Near-critical spring equations can amplify rounding] → Cover both sides of the discriminant and compare float64 samples against core at the approved `1e-9` tolerance.
- [Serial Bézier graphs can render more slowly] → Apply the single-thread option only to plans with typed cubic Bézier keyframes; retain existing concurrency for spring-only and legacy scenes, and record native elapsed times without imposing a new budget.

## Migration Plan

No data migration or rollout flag is needed. Correct code reads existing schema-23 projects and renders the same accepted curves. Reverting the code restores the prior compiler but leaves project bytes unchanged. Complete focused checks and OpenSpec conformance before syncing and archiving the change.

## Open Questions

Approval of this revision confirms the private, Bézier-only single-thread FFmpeg command option, including its performance trade-off. The approved 40-step expression and all public/persisted contracts remain unchanged.
