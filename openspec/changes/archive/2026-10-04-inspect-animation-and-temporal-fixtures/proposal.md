## Why

Issue48 requires minimum desktop animation controls and deterministic verification of overshoot, loops, inherited stagger and range-boundary continuity. Desktop currently exposes vector/text fields but no animation controls; its preview remains a documented placeholder, while the existing core owns all animation evaluation and rendering.

## What Changes

- Add bounded authored-animation inspection and existing-channel/keyframe controls to the current core-backed desktop inspector.
- Expose scalar values/source key times, existing curve parameters, loop mode/iterations, group/component stagger and signed repeater time offset through existing typed operations.
- Show all channel kinds, scoped targets, retained clocks, original preset provenance and legacy animation read-only where editing is unsupported; preserve component-local read-only selection.
- Use independently authored codec-matched full static-geometry sequences for encoded comparisons alongside analytic/lossless checks, within unchanged native work and tolerance bounds.
- Add an independently specified small synthetic temporal fixture and fresh-directory generator, numeric expectations and native frame/range/materialized-draft/export conformance.
- Complete canonical intrinsic Rectangle and intent-aware existing Media measurement for the approved CPU sampling path, preserving affine-only calculation, preflight safety and public contracts.
- Reuse bounded canonical CPU visual sampling for supported animated sources at nonaligned frame/range origins, fixing exact requested-time evaluation without changing codec selection or audio/public contracts; cover static-affine Rectangle half-open activity and record remaining Caption/unshaped-Text limitations explicitly.
- Correct the local animated-geometry raster envelope to conservatively include valid parameterized-curve overshoot using the existing canonical curve bounds, without changing sampling or public contracts.
- Document the actual inspector workflow and record desktop build plus genuine manual GUI evidence, keeping unavailable checks explicitly blocking.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `desktop-hierarchy`: authored animation inspection and revisioned bounded scalar/curve/loop/controller field edits.
- `render-regression-fixtures`: independent temporal animation recipe, source expectations, lifecycle/failure coverage and native conformance.

## Impact

Desktop presentation/field preparation/session tests, the editor-core-owned local animated-geometry envelope and test fixtures and fixture generator, and documentation. No new public core sampler, headless/MCP operation, capability, persisted schema, migration or provider behavior is required. Existing schema31 clocks, Scalar/Pack attribution rules, candidate safety limits, marker lifecycle, history and issue71 preview request ownership remain unchanged. New fixture metadata is test-only and must not redefine public contracts. Non-goals: desktop playback/live-pixel preview, raw JSON/expression authoring, key/channel creation or deletion, compound collection editing, new curves or parameter bounds, golden recapture, tolerance/timeout weakening, merge/deployment or subsequent issues.
