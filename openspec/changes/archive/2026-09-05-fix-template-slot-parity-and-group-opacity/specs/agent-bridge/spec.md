## MODIFIED Requirements

### Requirement: Typed template slot workflows
Headless and MCP MUST expose component_define_slots standalone and inside timeline_batch_edit, and accept the compatible optional slots/slotValues fields on component workflows. Requests and project responses MUST use closed typed definitions and values for all eight kinds. Protocol version 1 MUST advertise additive capability `typed_template_slots`. Adapters MUST delegate semantic binding, effective-value, reference, revision, lock and persistence validation to core. Documentation MUST specify type/constraint bounds, property mapping, local scope, integer-millisecond timing, ordering independence, default precedence, schema migration, errors and deferred rendering.

#### Scenario: Run real slot workflows
- **WHEN** source and packaged clients create, define, override and replace slots through standalone and aliased batch calls
- **THEN** typed reads and undo/redo/reopen reflect exact core state for all eight kinds

#### Scenario: Propagate atomic slot failures
- **WHEN** real calls encounter malformed types, missing references, stale revisions, locks or later batch failure
- **THEN** transport and core acceptance stages match documented contracts and no partial mutation is published

#### Scenario: Preserve override maps through real transports
- **WHEN** source and packaged clients submit special-key overrides or group opacity through standalone component edits and aliased batches, then undo, redo and reopen
- **THEN** typed request/response values match native state exactly, malformed entries fail without mutation, and protocol 1, schema 12 and advertised MCP input/output structural schemas remain unchanged
