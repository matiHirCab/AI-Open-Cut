## Why

The implemented integer-bound and component-scale fixes pass their regressions, but review reproduced a remaining precision defect: conversion of distinct mapped f64 points to f32 can collapse a visible diagonal polygon into blank output. Rendering must reject excessive conversion error before losing the original coordinates.

## What Changes

- Check Euclidean f64-to-f32 round-trip displacement of every SVG raster-space point against an inclusive 0.25-pixel threshold before path construction or bounds calculation.
- Reject non-finite or excessive conversion error with non-retryable INVALID_ARGUMENT before artifact preparation, including offscreen and move-only contours.
- Add exact review, threshold, mapped-scale, independent pixel and public failure-atomicity regressions.
- Document conversion tolerance separately from unchanged curve-flattening tolerance; no combined 0.25-pixel guarantee is introduced.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `svg-ingestion`: Specify bounded raster-coordinate conversion error and corresponding rendering/rejection evidence.

## Impact

Editor-core owns the checked conversion shared by evaluation and SVG raster preparation. Headless/MCP changes are test coverage through existing transports. Schema 15, document 1, protocol 1, public APIs, error codes and persisted models remain unchanged. Ingestion semantics remain unchanged; previously accepted rendering with excessive conversion loss instead fails explicitly. No dependency or architecture edge is introduced.

## Non-goals

No SVG subset expansion, clipping algorithm, point clamping, standalone shape change, curve-subdivision change, migration, unrelated golden regeneration or replacement of existing stroke/dash/integer-bound/component/resource checks.

## Approval

User explicitly approved all four artifacts with "Approve" on 2026-09-07. Implementation is authorized.
