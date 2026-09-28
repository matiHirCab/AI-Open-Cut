## ADDED Requirements

### Requirement: Typed marker transport and discovery
Headless and MCP SHALL expose typed marker create, update, delete and item-start timing operations as standalone mutations and within `timeline_batch_edit`, using existing project/revision envelopes, results and aliases. Project responses MUST expose markers, item expressions and effective numeric start times. Status MUST advertise an additive marker-relative-timing capability under protocol 1. Adapters MUST delegate scope, name, timing, bounds, revision and persistence validation to editor-core. Public documentation MUST specify scope lookup, signed offsets, duplicate-name ambiguity, numeric fallback, ordering independence, errors and schema-24 migration.

#### Scenario: Execute standalone and batch marker edits
- **WHEN** a client sends valid marker edits through either transport, including batch aliases for newly created IDs
- **THEN** both transports return equivalent typed results and the same canonical project state

#### Scenario: Preserve typed failures and old clients
- **WHEN** a request has an invalid expression, missing/ambiguous marker, stale revision, or uses only preexisting numeric operations
- **THEN** the transport reports the core's stable typed failure or preserves the numeric client's prior behavior, as applicable

#### Scenario: Discover support
- **WHEN** a protocol-1 client reads status or MCP tool metadata
- **THEN** it can identify marker-relative timing support and the exact validated input/output schemas
