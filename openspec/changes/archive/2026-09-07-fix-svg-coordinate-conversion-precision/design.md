## Context

The shared SVG raster-path helper currently casts mapped f64 points to f32 before validating integer bounds. The reviewed diagonal polygon maps near plus/minus 500 million raster pixels, where distinct edges ten pixels apart collapse during conversion. All integer bounds remain valid, so the renderer successfully publishes blank output. The preceding opacity, closepath, integer-bound and component-scale corrections are implemented and must remain intact.

## Goals / Non-Goals

**Goals:** Enforce an inclusive 0.25-raster-pixel Euclidean conversion-error threshold before original coordinates are discarded, share checks between evaluation and raster preparation, and demonstrate rejection/acceptance and failure atomicity through public paths.

**Non-Goals:** No clipping algorithm, point clamping, SVG subset expansion, standalone shape or curve-subdivision change, new dependency, public API change, migration or unrelated golden regeneration.

## Decisions

### Check the actual backend conversion

In the existing editor-core SVG raster-path conversion, calculate x64 and y64 after viewport mapping, origin adjustment and sampling density. Reject non-finite original values. Cast to x32/y32, reject non-finite converted values, then compute `(x64 - f64::from(x32)).hypot(y64 - f64::from(y32))`. Reject displacement greater than 0.25; equality is accepted. Only after success may the converted point enter PathBuilder or bounds accumulation. Apply the same operation to every point, even offscreen and move-only contours. Retain all subsequent backend and stroke checks.

Use a named conversion-threshold constant distinct from the curve-flattening tolerance. Keep the existing 0.25/scale subdivision calculation unchanged. This change bounds conversion error only; it does not promise that flattening plus conversion has total error at most 0.25 pixels. The check is O(points) with constant extra storage and introduces no separate raster allocation.

The shared helper is called during evaluation before artifact preparation and defensively during SVG raster preparation. Return non-retryable INVALID_ARGUMENT without source text or sensitive data in the error. Do not duplicate domain validation in headless or bridge.

Alternatives considered: checking only integer bounds misses the demonstrated failure; checking each axis independently permits Euclidean displacement above the selected limit; requiring exact representability rejects ordinary acceptable rounding; clipping and coordinate clamping expand scope or alter geometry. The approved Euclidean round-trip threshold is the selected policy.

### Regression evidence

Add the exact diagonal reproduction and its shorter, representable counterpart from the delta scenarios. Independently check the red interior pixel (50,55) and pixels outside the band; do not derive expected pixels from the production conversion helper. Native output comparisons retain existing encoding tolerances.

Use unit-level raster points to test exact, positive/negative, inclusive and immediately-over-threshold conversion. For example, `(8388608.25, 0)` has a 0.25 error and is accepted, while the next larger f64 value exceeds it; `(8388608.25, 8388608.25)` exceeds the Euclidean threshold although each axis is individually at the limit. Test non-finite conversion and move-only contours. Test viewport/density combinations with computed independent raster-space expectations and exactly representable large values within existing integer bounds.

Extend existing headless and MCP failed-render tests and native frame/range/draft/export atomicity coverage to include the precision failure. Preserve prior integer-overflow, opacity, closepath, clipping and component-scale cases. Compare state/revision/history/draft/output snapshots around failed rendering. Keep canonical ingestion fixtures unchanged unless their governed behavior actually changes: this is a rendering-time rejection, not a new ingestion grammar rule.

## Risks / Trade-offs

- Some previously accepted extreme artwork will now fail instead of silently changing geometry. This is the explicitly selected fail-closed behavior; documentation will explain the conversion threshold.
- Conversion and flattening errors are separate. Mitigation: distinct constants and documentation, no combined-error claim, and retention of independent native pixel tests.
- A future path-building call could bypass the helper. Mitigation: evaluation/preparation sharing and public/native regressions, including offscreen and move-only cases.

## Migration Plan

No persisted or public wire-shape change: schema 15, document 1, protocol 1, aliases and errors remain unchanged. Existing stored SVGs receive the check on subsequent rendering. Rollback is a code revert with no persistent rewrite or migration. Preserve unrelated work.

## Verification

Run required Rust formatting, strict workspace Clippy and tests; bridge typecheck, lint, unit, contract, integration and packaged smoke checks; hermetic Python suites; and required native rendering suites. Record command outcomes and scenario-to-test traceability. Use openspec-verify-change, resolve mismatches, synchronize and archive, then run the protected Moon gate. Required failed or skipped checks block completion.

## Open Questions

None. User explicitly approved all four artifacts on 2026-09-07.
