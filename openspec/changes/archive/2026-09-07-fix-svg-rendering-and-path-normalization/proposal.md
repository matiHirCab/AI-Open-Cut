## Why

Review of issue #29 reproduced three defects in the current SVG implementation: repeated translucent shapes lose opacity through intermediate byte rounding, small viewports reject large downscaled source geometry, and legal drawing commands after closepath are rejected. Existing tests pass but do not establish these edge cases correctly.

## What Changes

- Compose SVG fills, strokes and siblings in premultiplied linear-light f64 RGBA and encode only once, with explicit peak-memory accounting.
- Compile bounded SVG contours independently of standalone surface sizing, apply viewport mapping, and validate the actual viewport surface and mapped raster precision.
- Normalize supported drawing commands after Z by restoring the subpath start and inserting a budgeted MoveTo where required.
- Add canonical fixtures, independent pixel/geometry regressions, native intent/lifecycle coverage, and accurate documentation and verification evidence.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `svg-ingestion`: Clarify geometry normalization, viewport-based raster limits, mapped precision, accumulated opacity and bounded memory, with regression scenarios for the three confirmed failures.

## Impact

Editor-core validation, evaluated scene compilation and rasterization own the corrections. Canonical SVG fixtures and their Rust/headless/TypeScript/MCP consumers gain coverage. No new private-owner dependency, package, schema migration or public wire field is planned. Schema 15, normalized document version 1 and protocol 1 remain unchanged. Corrected rendering applies to existing SVG documents; newly accepted path continuations lower into the existing persisted vocabulary. This is corrective/additive acceptance, not a new major public contract. Standalone shape behavior and unrelated golden references remain unchanged.

## Non-goals

No additional SVG commands, resources, CSS, fonts, transforms, paint types, editing APIs or permissive fallback. No relaxation of source, geometry, segment, surface or aggregate memory ceilings.

## Approval

The user requested implementation of the detailed plan, including creation and subsequent explicit approval of these artifacts. The user explicitly approved these concrete artifacts on 2026-09-07 ("Approve"). Implementation is authorized.
