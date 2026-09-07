## Why

Review of issue #30 reproduced a double coverage reduction at fractional grid viewport edges and identified an MCP rollback test rejected during decoding instead of after a successful edit.

## What Changes

- Clip bounded evaluated grid coverage contours geometrically before antialiasing instead of multiplying raster coverage by viewport area.
- Resolve line dashes and stroke caps before rectangular polygon clipping; retain local paint coordinates, ordering, density and budgets.
- Exercise domain failure after grid creation through real source and packaged MCP clients, asserting stable errors and unchanged state.
- Add independent pixel, geometry, budget and native output regressions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `procedural-grids`: Correct fractional coverage and bound clipping work.
- `agent-bridge`: Prove domain rollback through source and packaged MCP.
- `rendering-export`: Native fractional edge conformance across intents.

## Impact

Only internal grid evaluation/rasterization and regression workflows change. No public contracts, dependencies, schema version, migrations or encoding-quality changes. Preserve existing shape/SVG paths and the previous archived change as historical evidence.

## Approval

The user explicitly approved this concrete plan on 2026-09-07 with "PLEASE IMPLEMENT THIS PLAN", including the algorithm, regressions and verification/archival lifecycle, before implementation. These artifacts record that approved scope; no additional contract approval is required because public contracts do not change.
