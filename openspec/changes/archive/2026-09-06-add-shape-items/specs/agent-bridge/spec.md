## ADDED Requirements

### Requirement: Typed discoverable shape workflows
Headless MUST expose add_shape and MCP MUST expose timeline_add_shape, both standalone and through timeline_batch_edit with existing project/revision envelopes and mutation results. All typed request, project response, batch, draft and component surfaces MUST accept canonical shape records. Adapters MUST delegate semantic validation and atomicity to core. Protocol 1 MUST advertise shape_items in core and aggregate capabilities; a ready complete renderer MUST additionally advertise shape_rendering in rendering and aggregate capabilities. An unavailable renderer MUST omit shape_rendering without hiding editable shape support.

Canonical operation, MCP structural schema/annotation, capability, shape fixture and ownership catalogs MUST match every governed native/TypeScript consumer. Documentation MUST specify geometry, coordinates, timing, ordering, bounds, errors, schema 14, alias usage and complete-backend failure behavior. Existing operation shapes, capabilities, error codes/retryability and provider protocols MUST retain their meanings.

#### Scenario: Discover and exercise real clients
- **WHEN** source and packaged clients discover support and create/edit each shape standalone and in aliased batches
- **THEN** typed responses, project reads, undo/redo/reopen and preview/export reflect core semantics and contract parity passes

#### Scenario: Reject through real transports
- **WHEN** clients submit structural/semantic invalid input, missing references, locked targets, stale revisions or a failed trailing batch edit
- **THEN** the documented validation stage and stable error are preserved with byte-identical prior state/history

#### Scenario: Report partial subsystem readiness
- **WHEN** core supports shapes but rendering dependencies are unavailable
- **THEN** shape_items remains discoverable while shape_rendering is absent and rendering retains its readiness error
