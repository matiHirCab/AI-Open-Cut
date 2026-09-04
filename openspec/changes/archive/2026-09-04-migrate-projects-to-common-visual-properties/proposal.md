## Why

Visual timeline variants currently duplicate `transform` and `hidden` fields, which prevents later motion-graphics milestones from adding richer transforms, stacking, hierarchy, masks, and effects through one canonical visual contract. Issue #17 establishes that shared persisted seam now, while schema-v6 projects can still be migrated deterministically without changing their rendered pixels.

## What Changes

- Add an editor-core-owned `VisualProperties` value shared by media, text, solid-color, rectangle, caption, and transition timeline items, containing the visual state supported by this milestone: the existing transform and visibility values. Its fields remain flattened in serialized items so current field names and edit-operation payloads stay compatible.
- Bump the persisted project schema from version 6 to version 7 and deterministically migrate the current project plus every retained undo/redo snapshot to explicit common visual properties under the existing project lock and recoverable generation transaction.
- Preserve existing add/update/visibility operations as compatibility sugar over the common visual properties; no new operation, batch alias, capability identifier, stable error code, or renderer behavior is introduced.
- Update the canonical project/protocol fixtures and governed Rust and TypeScript consumers to describe schema-v7 project documents while retaining the existing version-1 operation meanings.
- Prove pixel-equivalent shared `EvaluatedScene` behavior across frame preview, range preview, draft preview, and export, plus migration failure atomicity and deterministic reopen/undo/redo behavior.
- Persisted project documents advance to schema version 7 and make common visual defaults explicit while retaining existing item field names. Older binaries remain required to reject the future schema rather than downgrade or rewrite it; schema-v6 documents remain accepted by the new binary through migration.
- Validate migrated current/history visual properties and retained references before side-effectful legacy asset normalization, so a rejected envelope publishes neither project metadata nor content-addressed asset files.
- Accept omitted common visual fields as compatibility defaults even on schema-v7 input. Read-only reopen remains idempotent; project responses and the next committed serialization emit the canonical explicit `transform` and `hidden` keys.

### Non-goals

- Transform units, anchors, independent scale axes, rotation, skew, or a new transform evaluation order (issue #18).
- Explicit z-index, item/track reorder operations, groups, parent DAGs, components, slots, masks, effects, or animation-model changes (issues #19 and later).
- New headless or MCP operations, aliases, capability identifiers, renderer expressions, paths, network resources, or UI controls.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `timeline-editing`: Visual timeline variants share one canonical visual-properties value while existing editing operations keep their current meanings.
- `project-persistence`: Schema-v6 current state and all retained history migrate atomically and deterministically to schema v7, with future schemas rejected without rewrite.
- `motion-graphics-architecture`: The first persisted motion-graphics milestone activates common visual properties without yet activating Transform2D, stacking, hierarchy, masks, effects, or components.
- `contract-governance`: The canonical project fixture and every governed consumer advance together while protocol-v1 edit operations remain compatible.
- `rendering-export`: Evaluation and every render intent consume the migrated common properties without changing current visual or audio output.

## Impact

- `crates/editor-core`: project model, migration owner, validation/timeline accessors, persistence tests, evaluated-scene projection, render fixtures, and schema-version fixtures.
- `contracts`: canonical headless/project examples and ownership-governed parity evidence for the schema-v7 serialized shape.
- `apps/headless` and `apps/agent-bridge`: typed project response declarations, strict Zod schemas, fixtures, and parity tests; existing edit request shapes remain unchanged.
- Documentation: project schema/migration and motion-graphics activation notes.
- Compatibility: additive at the serialized-field and operation layers, with an intentional schema-version boundary, deterministic v6-to-v7 migration, and fail-closed older-reader behavior.
- Review correction: the previously completed EvaluatedScene milestone remains non-persisted, but its historical schema-v6 no-migration constraint is superseded by this separately approved schema-v7 milestone.
