## ADDED Requirements

### Requirement: Typed animation transport parity
Headless request/response unions and MCP Zod schemas, tool registration, and batch schemas MUST expose the same additive replace-channels operation and closed payload. The exported TypeScript headless edit union MUST type channel input according to the closed channel schema so malformed channel records fail type checking before transport use. Transports MUST pass typed inputs to editor-core and preserve its stable error code, retryability, revision, changed IDs, and alias mapping.

#### Scenario: Standalone and batch parity
- **WHEN** an agent edits valid channels directly or through an alias in `timeline_batch_edit`
- **THEN** both transports return the same canonical result and persisted channel state

#### Scenario: Preserve typed failures
- **WHEN** core rejects an invalid channel, missing target, or stale revision
- **THEN** headless and MCP expose the corresponding stable error without a partial mutation

#### Scenario: Reject malformed TypeScript channel input
- **WHEN** a bridge caller constructs a headless channel edit with a wrong value tag, missing keyframe field, or unknown channel name
- **THEN** the exported edit type rejects that payload during type checking
