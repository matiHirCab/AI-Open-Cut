## Why

Review of issue #28 reproduced three failures despite passing existing tests: magnified shapes lose coverage and opacity, fractional dimensions move anchors, and JSON buffering drops duplicate vector fields before validation. This follow-up restores the accepted shape and vector contracts with regression evidence.

## What Changes

- Rasterize at effective output magnification and compensate rendering transforms without changing authored geometry.
- Preserve positive fractional anchor bounds and restrict fallback to zero-extent path axes.
- Preserve duplicate object entries until typed vector decoding in edits, batches, drafts, project documents, components, and history.
- Add independent rendering, decoding, atomicity, and resource-budget regressions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shape-items`: Explicit scale-aware raster coverage, fractional anchors, and allocation failure scenarios.
- `vector-primitives`: Explicit duplicate-preserving decoding through activated shape consumers.

## Impact

Core evaluation, raster artifacts, affine render sampling, model decoding and persistence decoding are affected. Headless and bridge test consumers verify the unchanged wire surface. Public fields, tool signatures, schema 14, error codes, capabilities, and valid serialization remain unchanged; this corrects already-invalid input acceptance and is not a new breaking contract. No migration or dependency edge is introduced. Retained history and legacy migrations must remain readable for valid inputs.

## Non-goals

No new geometry, rendering backend, public API, unrelated refactor, or change to legacy rendering. Do not rewrite the original archived change or overwrite existing working-tree work.

## Approval

The user requested implementation of the plan. Concrete follow-up artifacts require explicit approval before implementation, as specified in that plan and repository instructions. The user explicitly approved these concrete artifacts with "Approve" on 2026-09-06.
