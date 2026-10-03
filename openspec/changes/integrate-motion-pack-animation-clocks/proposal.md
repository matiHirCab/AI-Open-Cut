# Integrate schema30 motion packs with schema31 animation clocks

Test setup amendment: the larger combined Pack/clock MCP fixture makes five whole-catalog copies in the existing supporting-surface drift test exceed its unchanged5s budget even on a quiet four-CPU container. Avoid copying untouched tool definitions when testing a replaced supporting array; retain the complete baseline and every existing case/assertion. This authorizes only immutable test setup changes, not production behavior, timeout/config changes or parity weakening. Independent approval and implementation review are required before applying it.

## Why
Issue46 draftPR135 introduces closed tagged motion-pack provenance in schema30. Issue47 draftPR136 independently retains source clocks in schema31 and intentionally rejects reserved30. Both cannot be integrated safely without the actual Pack-capable decoder and explicit validated30→31 path. User explicitly requested this integration gate; GitHub mergeability is insufficient.

## What Changes
Locally combine exact issue46 d3b616fc and issue47 5388a17c on integration/issue-46-47-schema-20261003, preserving both implementations. Accept recognized schema30 only through the merged closed scalar/Pack decoder and provenance validator, then atomically migrate current/components/all undo/redo to31 without recompilation, retagging scalar or altering source primitives. Preserve pre30-Pack and pre31-clock rejection and unknownfuture>31 fail-closed policy. Verify journal recovery, interrupted migration and migrated pack editing through both Rust and bridge consumers.

## Capabilities
Modified: project-persistence (explicit supported30→31 migration), animation-edit-semantics (retained edits preserve tagged pack attribution).

## Impact
Rust model/migration/recovery validation, governed schema31 bridge/catalog fixtures and migration/edit tests. No merge/deploy, gate/tolerance weakening, new presets, or issue48. Chosen order:46 before47, contingent on exact combined-tree verification and independent review. Publication packaging/base is tracked externally after evidence exists.

## Native parity amendment
Exact FFmpeg6.1 reproduction exposed a pixelwise affine rectangle atlas-border defect that changes source/edit samples. Extend the existing shape-only nearest lookup rule to rectangle pointwise premultiply/unpremultiply/opacity stages, preserving bilinear geometric interpolation. Verify both unclocked source and retained segments against unchanged independent equations/tolerances on FFmpeg6.1 and7.1. The static reference uses the existing identity vignette0 effect for a full transparent workspace; this changes no arithmetic expectation, actual animation, goldens or tolerance.

The same unchanged tests exposed a distinct CPU-raster trim-seam failure on FFmpeg6: looped static PAM inputs default to25fps while the canonical canvas uses10fps, so overlay scheduling can evaluate a stale pre-seam sample. Normalize only prepared sampled-input cadence to canvas fps before its existing placement. Already canvas-fps encoded sampled intervals keep their cadence; source-time evaluation, exact visible bounds and ordinary media inputs remain unchanged.

An exact1600ms CPU draft seam also exposes a redundant FFmpeg floating-point activation comparison after AVTB placement. Sampled raster pixels already encode canonical activity in the shared draw path, and the existing motion-blur sampled branch deliberately relies on that authority. Use the same compositing rule for all sampled CPU inputs, retaining placement and EOF-pass behavior, and verify inactive gaps/end boundaries remain transparent. Ordinary media/affine activation rules are unchanged.

Independent review reproduced a retained-clock precision defect inside the accepted integer range: offset2^52 plus represented local0.25 loses its fraction before a10ms repeat modulo. Preserve integer whole time and represented fractional local time separately through signed offset composition, loop reduction, key selection and relative interpolation; mirror stable integer-origin/phase reduction in renderer expressions. This realizes existing fractional-clock semantics without shrinking accepted bounds or changing persisted metadata.
