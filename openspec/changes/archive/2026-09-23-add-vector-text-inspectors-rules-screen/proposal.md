## Why

Issue #37 completes the static vector/typography milestone: core already supports shapes, grids and rich text, but desktop exposes only hierarchy controls and there is no complete static rules-screen regression fixture. Dependencies #26 and #36 are closed.

## What Changes

- Add minimal structured desktop controls for existing root shape, procedural-grid and text items, retaining read-only component-local inspection.
- Preserve existing core-owned validation, revision checks, atomic edits, history and font ownership; expose errors without speculative publication.
- Add a reproducible native rules-screen recipe with outlined cards, accent bars, corner brackets, diagonal grid, concentric circles and layered EVERY./SINGLE./ONE. words, at 1920x1080, 1280x720 and 960x540.
- Verify independent semantic/pixel references, render-intent parity and lifecycle/failure behavior through existing APIs, including aliased batches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `desktop-hierarchy`: selected vector/text properties and revisioned inspector editing.
- `render-regression-fixtures`: separate static rules-screen fixture, lifecycle and multiresolution conformance.
- `rich-text-documents`: add an optional typed `fontSize` field to `update_item` so existing text can be resized without replacing its identity.

## Impact

Desktop presentation/session tests; editor-core fixture builders and renderer regression tests; native parity integration and fixture documentation. `update_item` gains an optional positive integer `fontSize` field (1–1000), an additive request-contract change. The canonical headless/MCP catalogs and governed Rust/TypeScript consumers and parity tests must be synchronized and receive designated CODEOWNER review. Persisted schema, existing error codes, operation names and old clients remain compatible. Add fixture evidence without changing baseline references or tolerances. No new transport operation, schema migration or dependency edge is planned.

## Non-goals

Animation authoring, desktop preview playback, arbitrary SVG/path source editing, font browsing/import, component-definition editing, full vector drawing tools, new rendering semantics and external graphic panels are excluded. The fixture is an original reproducible composition of the issue's named elements, not a claimed pixel match to an unavailable source image.

## Approval

The user approved the initial artifacts and the additive `fontSize` amendment in this task on 2026-09-22.
