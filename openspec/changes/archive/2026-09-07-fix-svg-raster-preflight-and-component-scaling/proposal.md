## Why

Review of issue #29 reproduced two remaining rendering defects: finite mapped geometry can silently disappear in tiny-skia, and component-local raster preflight rejects valid cancellation of inner and outer scales. The existing opacity and closepath fixes are implemented; this follow-up addresses only these remaining defects.

## What Changes

- Reject SVG raster coordinates and stroke bounds that the backend cannot represent before artifact preparation, using shared conversion rules.
- Validate complete component occurrences before applying raster limits, including hidden occurrences; validate unreachable definitions as virtual roots.
- Preserve bounded compilation and aggregate accounting without rejecting referenced definitions at an artificial isolated scale.
- Add independent numeric, pixel, public-operation and lifecycle regressions and verification evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `svg-ingestion`: Specify backend raster representability, composed occurrence validation, unreachable-definition handling and failure/public rendering evidence.

## Impact

Editor-core evaluation and SVG artifact preparation own the behavior. Headless and bridge tests exercise the existing transports; canonical evidence and SVG documentation stay synchronized. No new dependency, public API, wire shape, error code, schema or migration is planned: schema 15 and protocol 1 remain unchanged. Previously blank successful renders become non-retryable INVALID_ARGUMENT; valid scale-cancelled component scenes become renderable.

## Non-goals

No SVG subset expansion, new clipping algorithm, point clamping, standalone shape rendering change, alpha/path normalization redesign, unrelated golden regeneration or architecture dependency change.

## Approval

User explicitly approved all four artifacts with "Approve" on 2026-09-07, after their creation and strict validation. Implementation is authorized.
