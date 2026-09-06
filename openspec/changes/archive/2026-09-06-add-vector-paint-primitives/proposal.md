## Why

Issue #27 (MG-M2-01) needs a canonical, validated vocabulary for vector paint and geometry before issue #28 introduces ShapeItem rendering. Today the core exposes solid colors and rectangles but has no reusable typed gradients, strokes, corner radii, or vector paths.

## What Changes

- Add reusable strict Rust types and pure core validation for RGBA colors, solid/linear/radial paints, gradient stops, strokes, dashes, caps, joins, corner radii, and absolute line/quadratic/cubic Bezier path commands.
- Define finite numeric bounds, explicit collection limits, coordinate and color semantics, deterministic failure behavior, and safe serialization.
- Add a dedicated versioned canonical fixture catalog and mirrored strict TypeScript schemas with shared parity evidence and ownership registration.
- Document these as available core primitives that are not yet activated in persisted timeline items or renderer instructions.

## Capabilities

### New Capabilities
- `vector-primitives`: Canonical bounded vector data, validation, serialization, and cross-language evidence.

### Modified Capabilities
None. Existing persistence, editing, transport, and rendering requirements remain in force.

## Non-goals

ShapeItem, timeline_add_shape, rasterization, SVG ingestion, grids, repeaters, animation, and rich text belong to subsequent issues. This change does not add a project field, timeline mutation, headless operation, MCP tool, render instruction, or advertised rendering capability.

## Impact

Core gains an inward-owned module exported through its public Rust library. Contracts gain vector-primitives-v1.json and an ownership entry; bridge schemas and fixture tests gain reusable declarations without capability registration. Documentation explains activation boundaries. Existing public wire shapes, legacy color strings, project schema 13, revisions, undo/redo, retained history, and EvaluatedScene remain unchanged. The addition is compatible; no breaking contract or data migration is proposed. Future persistence/render activation requires its own approved change and migration.

## Approval

Approved explicitly by the user on 2026-09-06 with “Approve”, covering this prerequisite primitive library, the design and numeric/serialization semantics, and runtime activation reserved for #28.
