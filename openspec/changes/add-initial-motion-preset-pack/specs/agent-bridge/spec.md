## ADDED Requirements

### Requirement: Discoverable typed initial motion pack
Headless and MCP MUST forward the unchanged preset operation with the additive closed parameter union and report initial_motion_preset_pack_v1 alongside existing capabilities. Canonical preset, project-schema, headless and MCP structural fixtures and all governed native consumers MUST agree. Existing scalar callers MUST remain valid. Unsupported live identities/versions and semantic boundaries MUST be decided in core; transport schemas MUST reject malformed closed shapes without expansion. Standalone and batch workflows MUST preserve existing aliases, revisions, typed errors, undo/redo and reopen behavior. Source and packaged smoke MUST exercise the five entries against the same canonical fixture.

#### Scenario: Old and new typed clients
- **WHEN** an old scalar request or any valid new pack request is passed through typed headless and MCP standalone/batch surfaces
- **THEN** fixtures and generated primitive/source state agree without transport-owned motion logic

#### Scenario: Failure parity and capability discovery
- **WHEN** clients inspect status or submit malformed fields, unsupported versions, collision, missing reference or stale revision
- **THEN** status exposes exact support and Rust/TypeScript/MCP agree on accepted structure, canonical failure code/retryability and unchanged authoritative state
