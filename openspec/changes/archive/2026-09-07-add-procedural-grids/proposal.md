## Why

Issue [#30](https://github.com/matiHirCab/AI-Open-Cut/issues/30) requires editable procedural grids without external assets or renderer-specific expressions. Vector primitives (#27) and shape rendering (#28) are implemented and their issues are closed, so grids can reuse the existing core-owned vector and evaluated-scene pipeline.

## What Changes

- Introduce a bounded `GridItem` with rectangular, diagonal, dot, and isometric patterns, local dimensions, spacing, and vector paints.
- Add headless `add_grid` and MCP `timeline_add_grid`, complete grid replacement through `update_item`, atomic batch aliases, and the common visual lifecycle in root and component scopes.
- Expand grid geometry in editor-core and share evaluated instructions across frame preview, audiovisual range preview, materialized drafts, and final export.
- Advance persisted schema 15 to 16 and atomically migrate supported current state and retained undo/redo history without changing existing visuals or media ownership.
- Publish canonical fixtures, strict Rust/TypeScript schemas, capability evidence, client documentation, and automated conformance tests.

## Capabilities

### New Capabilities

- `procedural-grids`: Strict grid patterns, geometry, paints, complexity limits, and deterministic evaluated rendering.

### Modified Capabilities

- `timeline-editing`: Transactional grid creation, replacement, aliases, and existing visual operations.
- `project-persistence`: Atomic schema-16 grid activation and fail-closed source-version validation.
- `agent-bridge`: Typed discoverable grid tools, capabilities, and governed cross-language parity.
- `rendering-export`: Shared complete procedural-grid rendering and readiness.

## Impact

Core model, validation, timeline, migrations, evaluated scene, vector raster preparation, and item consumers need grid support. Headless request/response declarations, bridge schemas/tool registration, capabilities, contract ownership/catalogs, desktop exhaustive item handling, and integration/smoke evidence must stay synchronized. Existing ADR 0003 ownership and dependency direction remain mandatory.

Public requests are additive under protocol 1: existing operations, aliases, errors, and vector primitives retain their meaning. Persisted schema 16 is a new version: older binaries reject it, and no downgrade is supported. New clients discover `grid_items` and `grid_rendering`; old clients can continue existing requests but must not be assumed to decode the new item variant. Fixtures and documentation must state this compatibility boundary. No provider protocol changes or new rendering dependencies are planned.

## Non-goals

No arbitrary SVG or expression inputs, resources, major/minor line styling, independently styled line families, arbitrary-angle patterns, polar grids, grid-specific animation channels, new slot bindings, new desktop grid-authoring controls, or unrelated renderer/contract cleanup. Standard visual transforms provide rotation, skew, and placement.

## Approval status

Approved by the user in this task on 2026-09-07 with "Approve", covering the proposal, design, five delta specifications and implementation tasks. The user separately approved the resulting contracts on 2026-09-07 with "Approve" in response to the final designated contract-review request linked to verification.md. Both approval gates are satisfied in this task; no GitHub review submission is represented.
