## Why

Issue #28 requires editable vector shapes that render consistently in preview and export. The vector primitives from #27 are implemented, but remain reference-free types without timeline or rendering activation.

## What Changes

- Add persisted ShapeItem geometry for rectangles, rounded rectangles, ellipses, lines, polygons, stars, and structured paths, using existing paints and strokes.
- Add core/headless `add_shape`, MCP `timeline_add_shape`, batch creation aliases, and existing visual-item lifecycle support.
- Evaluate shapes through EvaluatedScene and rasterize through the shared graphics/compositing path with bounded work and deterministic parity evidence.
- Migrate supported projects and all retained history to schema 14 atomically, preserving legacy content and output.
- Synchronize typed consumers, canonical catalogs, capability reporting, documentation, and automated conformance tests.

## Capabilities

### New Capabilities
- `shape-items`: Geometry, validation, editing, evaluation, and rendering of seven vector shape variants.

### Modified Capabilities
- `vector-primitives`: Activate the existing vocabulary for shapes without changing primitive semantics.
- `timeline-editing`: Extend compatible visual editing to shape items and atomic creation.
- `project-persistence`: Schema-14 activation and source-schema-aware current/history migration.
- `agent-bridge`: Typed shape creation and discoverability across headless and MCP.
- `rendering-export`: Canonical bounded shape rendering shared by all output intents.

## Impact

Affected owners: editor-core model/vector/validation/timeline/evaluator/rendering/migration/store, headless transport, agent-bridge contracts/schemas/registration, canonical contract fixtures, rendering evidence, and documentation. Follow ADR 0002 and ADR 0003; do not introduce inward dependencies on transports or providers. Contract review belongs to @matiHirCab.

Public operations and capability identifiers are additive within protocol v1; old request meanings and stable error retryability remain unchanged. Persisted schema 14 is version-negotiated: older binaries reject it, and no downgrade is supplied. Existing rectangle operations and legacy color strings retain their behavior.

## Non-goals

SVG/string path import, external vector assets, network resources, executable renderer expressions, shape morphing, new animation channels, masks/effects, desktop drawing tools, and changes to provider protocols are excluded.

## Approval

Approved by the user in this task on 2026-09-06 ("Approve"). Implementation is authorized against these artifacts.
