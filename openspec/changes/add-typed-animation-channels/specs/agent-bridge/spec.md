## ADDED Requirements

### Requirement: Typed animation transport parity
Headless request/response unions and MCP Zod schemas, tool registration, and batch schemas MUST expose the same additive replace-channels operation and closed payload. Transports MUST pass typed inputs to editor-core and preserve its stable error code, retryability, revision, changed IDs, and alias mapping.

#### Scenario: Standalone and batch parity
- **WHEN** an agent edits valid channels directly or through an alias in `timeline_batch_edit`
- **THEN** both transports return the same canonical result and persisted channel state

#### Scenario: Preserve typed failures
- **WHEN** core rejects an invalid channel, missing target, or stale revision
- **THEN** headless and MCP expose the corresponding stable error without a partial mutation
