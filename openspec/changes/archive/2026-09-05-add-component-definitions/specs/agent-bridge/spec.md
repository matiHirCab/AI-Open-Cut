## ADDED Requirements

### Requirement: Typed component definition workflows
Headless and MCP MUST expose component_create, component_update and component_delete standalone and in timeline_batch_edit with existing project/revision envelopes and mutation results. Creation aliases MUST work through real transports. Adapters MUST delegate graph, duration, lock, persistence and media semantics to editor-core. Documentation MUST describe local coordinates/time/order, explicit duration, scopes, bounds, errors, schema-11 migration and deferred instance rendering.

#### Scenario: Exercise complete definition lifecycle
- **WHEN** a real client creates, updates, references and deletes definitions using valid standalone and aliased batch operations
- **THEN** reads and undo/redo/reopen reflect exact core state through source integration and packaged smoke

#### Scenario: Propagate atomic failures
- **WHEN** malformed input, missing references, cycles, locks, stale revisions or later invalid batch operations occur
- **THEN** real transports preserve canonical errors and no partial project/history state is published
