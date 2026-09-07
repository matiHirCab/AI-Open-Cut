## Why

A native reproduction shows that a 10x10 red shape in a 40x40 component translated to (100,20) disappears when constant scale keyframes of 1 are added. The shape evaluator clips root-space animated bounds against component-local dimensions, violating existing component placement and rendering semantics.

## What Changes

- Keep local coordinate dimensions separate from the requested output viewport throughout shape affine evaluation.
- Clip composed animation bounds against the output viewport, retaining local dimensions for normalized coordinates and preserving geometry validation before clipping.
- Add independent evaluator and native regressions combining animation with translated, scaled, nested and retimed component instances.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shape-items`: Explicit component animation clipping, output viewport and local coordinate scenarios under canonical shape evaluation.

## Impact

Core shape affine evaluation and shape rendering tests are affected. Preview, range rendering and export share the correction through EvaluatedScene. This is a bug fix within existing contracts: no public API, schema-14, migration, error-code, capability, dependency or backend change. Existing saved projects become correctly renderable without rewriting them. Prior fixes for raster density, fractional anchors and duplicate decoding remain intact.

## Non-goals

No changes to non-shape evaluator behavior, component boundary masking, animation semantics, normalization rules, resource limits or unrelated working-tree changes. Preserve both previous archives and all existing golden references/tolerances.

## Approval

The user explicitly approved all four concrete artifacts on 2026-09-06 with “Approve”. Implementation is authorized through this task list.
