## Why

PR #114 now passes the recipe checksum check but Linux render parity fails on three generated contour coordinates whose platform math results differ by at most a few ULPs. Raw Debug string equality treats these harmless numeric differences as semantic changes, while Windows native conformance passes.

## What Changes

- Make only the shape golden reference comparison portable for derived contour coordinates, bounded by both 8 ULPs and 1e-12 absolute error.
- Keep all other semantic fields exact, same-process repeated evaluations exact, and all visual/audio/timing tolerances unchanged.
- Add positive reproductions from the Linux failure and negative tests proving meaningful geometry and unrelated semantic changes still fail.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: define narrowly bounded portability for generated shape contour coordinates in stored semantic references.

## Impact

Only core's shape golden test owner and regression documentation. No production renderer, schema, public API, migration, dependency or CI workflow change. The committed LF recipe checksum correction is already pushed as b63c2d26; its validation remains enforced.

## Non-goals

No golden image recapture, blanket floating-point tolerance, scene-wide rounding, platform-specific baseline, relaxed geometry budgets, or changes to established legacy/rule-card comparisons. Existing references and approved shape behavior remain intact.

## Approval

The user explicitly approved these concrete artifacts on 2026-09-06.
