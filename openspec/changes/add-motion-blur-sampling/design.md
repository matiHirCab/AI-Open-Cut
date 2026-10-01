## Context

Issue44 initially stacked on PR131 at e1010bb97174d44a5d19c61a90514bf2336845ca,
with main at 08a543f62a471b704715f2fa6ae32ed93c94f517. Its corrective head
81a6ec682bd65a3b31d84b4d2aee7eef276c0bb4 passed all 11 CI jobs and was merged
externally at 2026-10-01T20:56:03Z. Current main is
b2be8f219743639db106330f5839a2efcf77c053, with the same corrected code tree.
Local issue44 now contains that main commit and preserves its original commits.
No agent merged PR131 or published issue44. Issue43 owns continuous visual
certification, inherited affine sampling, linear-premultiplied raster effects and
bounded lossless sampled preparation in editor-core.

## Goals / Non-Goals

Goals: typed leaf shutter settings, deterministic inherited motion, complete lifecycle/contract evidence and native render parity. Non-goals are listed in proposal.md. No audio redesign or network/path-bearing input.

## Decisions

1. Extend VisualProperties and existing update_item, rather than adding operations or changing channel semantics. Optional motionBlur is additive; a zero-angle record disables authored blur. Only issue43-supported raster leaves are eligible.
2. Use centered midpoint exposure in root output time, floor once to integer root milliseconds and preserve inherited fractional clocks. Alternatives: stochastic jitter is nondeterministic; trailing shutter introduces a different phase; converting each inherited clock to integer breaks inherited timing. Explicit clamping duplicates boundary samples without renormalization. Root resolution is documented and testable.
3. Retain scene facts for every inherited ancestor. Evaluate each sample through issue43 sample/sample_transform, not sampled local transforms alone. Do not collapse outside repeater ancestors onto source clocks.
4. Average each leaf in premultiplied linear light on the canvas after sample effects and affine, before canonical layer stacking. Alternative full-scene supersampling changes cross-layer occlusion semantics and requires flattening unrelated layers; leaf blur defines a bounded, observable scope. Animated source time, geometry, visibility and transitions participate consistently.
5. Apply checked sample-weighted scene pixel/effect/geometry budgets before I/O, retaining existing tighter limits. Stream one output frame at a time; preserve issue43 safe process diagnostics and cleanup. No new dependency edges or external dependencies.
6. Adopt schema 28 under the existing lock/atomic generation workflow, current plus retained history and drafts. Reject premature fields in older generations. Consumers remain transports only. Canonical contract fixtures govern bounds, compatibility, support reporting and scenarios.

## Risks / Trade-offs

- Integer-root sampling limits temporal resolution to milliseconds; independent fixtures must demonstrate the documented policy at high fps and small angles.
- Per-leaf averaging retains existing stacking; crossing leaves do not implement whole-scene camera exposure. Document this expressly.
- Sample multiplication increases CPU and decoding work; checked preflight, occurrence limits, cache dependencies and measured bounded performance are required.
- PR131's corrected exact-head CI is terminal success; designated issue44
  contract-owner implementation review remains required before archival.

## Migration Plan

Keep schema 27 controls as deterministic byte-preservation fixtures. Migrate all retained generations together and prove failed validation/publication preserves original bytes. No rollback/downgrade of schema 28; older readers reject future schema. No publishing or merging is authorized.

## Open Questions

None about intended semantics. Environment/verification limitations must be recorded in verification.md without converting skipped tests into successes.

## Reconciliation verification follow-up (2026-10-01)

PR131's separately approved corrective update is published at
81a6ec682bd65a3b31d84b4d2aee7eef276c0bb4. Its exact-head CI is terminal success (run 36918625589, all 11 jobs).
The aggregate confirms policy attestation and 49.03 minutes within 120 minutes. This local
branch merges that verified head, preserving every original issue44 commit. Preserve
the three original issue44 commits and do not publish issue44 or merge PR131.

Retain enabled-shutter activity outside the center sample while adopting the
corrected explicit RGBA sampled overlay and vector-only legacy channel lookup.
Keep one independent extended-animation MCP case with all blur assertions and
the existing timeout. New SSIM comparisons must trim both selected inputs and
assert a single stats line, preserving timestamps and SSIM>=0.99. The independent
analytic parent-translation oracle retains samples 462/487/512/537 ms, coverage
weights and MSE<=1.0; explicitly select the corrected RGBA final composition
without replacing any numeric oracle with actual renderer output.

Strengthen the already-required rejected-record fixtures with null, missing-field
and wrong-type records. Check published sample/pixel limits against core constants.
Exercise every rejected canonical record through actual headless standalone and
batch requests (a valid earlier batch edit must not publish), checking existing
INVALID_ARGUMENT/non-retryable errors and authoritative project/history bytes.
Also exercise null standalone/batch MCP inputs and unchanged project state.
This is failure-path evidence for existing requirements, with no new behavior,
contract meaning, tolerance, golden or CI-policy change. Re-run final mandatory
checks after reconciliation; external designated-owner review remains required.

## Deterministic disabled-exposure fallback

Native captures of omitted and zero-angle settings have byte-identical graphs and
inputs but differ in 48 RGB pixels. The inherited affine coordinates use st/ld
registers 4..7; FFmpeg 6 blend slices share the expression register state. Diagnostic
single-thread execution passes the unchanged exact compatibility checks. Extend
the existing private serial_bezier_filters command guard when a visual layer has
animated ancestors, a finalized affine, and no sampled input. Those facts identify
the legacy branch that emits register-based inherited coordinates. Existing
Bézier visual/audio conditions remain unchanged. Plain root and sampled graphs
retain their prior selection. No upstream branch update is part of this correction.

The independent analytic oracle must explicitly include canonical final YUV420
conversion after RGBA stacking, matching the renderer's unchanged output plan.
The diagnostic reaches its MSE assertion once compatibility is deterministic;
missing that final conversion produces MSE 1.7252604166666667. Add the documented
conversion, keeping every midpoint, coverage value and MSE<=1.0 expectation.

Verification uses the unmodified actual FFmpeg executable, repeated exact disabled
controls, all inherited/transform fixtures, native goldens and complete PR rules
matrices. Record performance rather than changing concurrency or CI budgets.
