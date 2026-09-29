## ADDED Requirements

### Requirement: Typed loop transport through existing operations
Headless request/response unions and MCP Zod schemas, tool registration, and batch schemas MUST expose the optional closed loop record through existing `set_animation_channels` and `timeline_batch_edit` operations as additive protocol-1 input. The exported TypeScript edit union MUST reject malformed loop records at type checking; runtime adapters MUST preserve editor-core's canonical result, revision, changed IDs, alias mapping, stable error code, and retryability. Status MUST advertise the additive loop capability. Adapters MUST not implement independent loop semantics. Public documentation MUST state bounds, half-open timing, seam and finite-exhaustion behavior, unsupported targets, errors, compatibility, and migration.

#### Scenario: Execute standalone and aliased batch loops
- **WHEN** a client submits a valid looped channel directly or addresses a newly created item alias in a batch
- **THEN** headless and MCP return equivalent typed results and persisted state

#### Scenario: Preserve transport failures and old requests
- **WHEN** a request has malformed loop shape, invalid core semantics, missing target, stale revision, or no loop field
- **THEN** transport rejects malformed shape or relays the core's stable result without partial mutation, and old requests keep prior behavior
