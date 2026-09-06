## Why

Issue #26 (MG-M1-10) needs a usable desktop inspection surface and evidence that reusable rule cards survive real editing and rendering. Dependencies #25 and #14 are complete, but desktop panels are placeholders and no active approved change covers this outcome.

## What Changes

- Connect the desktop to an explicitly selected local core project store and project through the existing EditorCore facade.
- Show selectable root groups, component instances and scoped component contents in an expandable hierarchy, with track/timing information and inspector parent/z-index controls for root visual items.
- Route edits, refresh, undo and redo through the existing revisioned core APIs; expose typed failures without publishing speculative state.
- Add a deterministic reusable rule-card fixture with at least six visual children and three independently slotted instances. Verify parent movement, ordering, lifecycle, still preview, audiovisual range preview and final export.
- Document the desktop entry path, supported controls, scope limitations and fixture verification procedure.

## Capabilities

### New Capabilities

- `desktop-hierarchy`: Core-backed project loading, scoped hierarchy selection and revisioned parent/z-index editing.

### Modified Capabilities

- `render-regression-fixtures`: Add a separate rule-card lifecycle and rendering conformance fixture while retaining the existing flat-scene golden baseline.

## Impact

Owning layers are apps/desktop for presentation/session state and crates/editor-core for fixture construction and conformance tests. Existing headless/MCP operations are exercised by integration tests; documentation and test fixture metadata change alongside them. The desktop gains an inward dependency on the public editor-core facade, already permitted by ADR 0003. No private core dependency edge is introduced.

Compatibility is additive desktop behavior and verification evidence: existing wire operations, capabilities, stable error codes, schema 13, slots, coordinate systems and renderer semantics retain their meanings. No new public transport operation, contract major or persisted field is proposed, so no new migration is needed. Existing migration/current-history tests remain required. Any discovered need to change these surfaces requires an amended proposal before implementation.

## Non-goals

Full timeline authoring, component-local mutation controls, drag-and-drop reparenting, world-preserving reparenting, new slot binding kinds, animation presets, arbitrary SVG/network assets and renderer changes are outside this issue. Preview/export conformance uses production core paths; replacing the desktop preview placeholder with a media player is outside scope.

## Approval

Approved by the user in this task on 2026-09-06 with the message "Approve", covering proposal, delta specs, design and tasks.
