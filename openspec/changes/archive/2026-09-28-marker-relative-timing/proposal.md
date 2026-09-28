## Why

Issue #40 needs named timeline cues that agents can edit and use as stable timing anchors. The checked-in motion-graphics catalog defines marker and relative-time fixture shapes, but the current project schema and editing API do not activate them.

## What Changes

- Add project and component-local marker creation, update, deletion, and scoped name lookup.
- Add a typed marker-relative expression for item start timing, with a signed millisecond offset. Keep existing numeric start timing compatible.
- Resolve effective timing in editor-core for committed edits, undo/redo, reopen, preview, and export. Reject missing or ambiguous names, invalid resulting times, and edits that would break stored references atomically.
- Add additive standalone and batch agent operations, capability discovery, canonical fixtures, and documentation.
- Migrate current project state and retained history to a new schema without changing existing project output.

No public field or operation is removed or reinterpreted; the protocol major version stays 1. The persisted schema version advances from 23 to 24, and older binaries must reject it. Marker timing is activated for item starts in this change; marker-relative duration, trim, keyframe, audio-event, and renderer-specific expressions are outside scope.

## Capabilities

### New Capabilities

- `marker-relative-timing`: Marker lifecycle, scoped resolution, item-start expressions, validation, and transaction semantics.

### Modified Capabilities

- `project-persistence`: Schema-24 migration of current project and retained undo/redo snapshots.
- `timeline-editing`: Item-start mutation and batch alias behavior for marker references.
- `agent-bridge`: Typed headless and MCP marker operations and capability discovery.
- `motion-graphics-contracts`: Activate marker and time-expression runtime contract parity from the existing fixture vocabulary.
- `rendering-export`: Require one resolved timing behavior across preview and export without changing legacy output.

## Impact

Editor-core owns the persisted model, migration, lookup, effective timing, limits, and failures. Headless and agent-bridge add typed request/response and MCP surfaces; `contracts/` and parity tests govern the additive public shape. The desktop has no new UI in this change. The prerequisite issues #17 and #11 are closed.
