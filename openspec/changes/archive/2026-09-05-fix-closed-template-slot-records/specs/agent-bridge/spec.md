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

Shared request and response validation MUST reject unknown own enumerable fields in closed template-slot records before parsing can strip them, while preserving parsed types and complete nested issue paths. Protocol 1, schema 12 and published input/output structural schemas MUST remain unchanged.

#### Scenario: Reject malformed records in real standalone and batch workflows
- **WHEN** source and packaged clients submit canonical malformed defaults or overrides through standalone edits or aliased batches, including a valid operation before the malformed operation
- **THEN** requests fail structural validation and preserve the prior project state, revision and byte-identical project/history files without partial mutation

#### Scenario: Preserve nested validation and schema contracts
- **WHEN** shared request and response schemas validate nested unknown fields or existing malformed values, including under special slot IDs
- **THEN** errors retain full nested record or value paths and unknown-key names, valid values return the existing parsed types, and both input/output JSON schemas and the registered MCP structural catalog remain identical
