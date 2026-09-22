## Why

Issue [#36](https://github.com/matiHirCab/AI-Open-Cut/issues/36) requires precise text and vector raster reuse. Existing core preparation rasterizes shaped text and evaluated shapes/SVG into each render workspace; its per-call shaping memoization does not provide reusable raster results across renders.

## What Changes

- Add bounded, process-local raster reuse in editor-core for shaped text, shapes, and normalized SVG.
- Key reuse by complete effective content, style, verified font identity, sampling scale, project revision and raster implementation version; isolate drafts and projects.
- Keep all validation, resource integrity checks and render preflight effective on cache hits.
- Prove cold/warm equivalence, selective invalidation, bounds, lifecycle safety and shared preview/export semantics with automated evidence.

## Capabilities

### New Capabilities
- `raster-caching`: Bounded deterministic reuse of validated text and vector raster results.

### Modified Capabilities
None. Existing rendering-export, font-resolution, advanced-text-layout, rich-text-documents, shape-items, svg-ingestion and motion-graphics-architecture guarantees remain binding.

## Impact

Changes belong to crates/editor-core's renderer orchestration and render_artifact owner, their tests and renderer documentation. No new private-owner dependency edge is intended. Dependencies listed in issue #36 already have corresponding core implementations and living specifications; this change consumes their evaluated output.

Compatibility is internal and additive: no public operation, capability flag, error code, project schema, canonical transport fixture or migration changes are required. Existing contract fixtures must continue passing unchanged. Cache contents are disposable and never serialized into project state or history.

## Non-goals

Persistent disk caches, new cache management APIs, changing shaping/layout or raster quality, new vector features, caching final composed frames, changing FFmpeg behavior, and skipping canonical validation are excluded.

## Approval

Approved by the user in this task with “Approve”, following presentation of all four planning artifacts. Approval covers the proposed limits and revision policy.
