## ADDED Requirements

### Requirement: Additive rich text transport parity
Protocol 1 MUST advertise `rich_text_documents` and retain existing operation/tool names and compatibility text output fields. Typed headless requests, MCP timeline_add_text and timeline_update_item inputs, timeline_batch_edit members, draft edits and project-state outputs MUST carry the canonical document without dropping run fields. Existing simple requests MUST remain valid. Strict wire schemas MUST reject unknown/null document fields before parsing can discard them; semantic validation MUST remain in core. Canonical versioned catalogs/fixtures and all consumers named by contract ownership MUST agree on the additive request/output/capability change and schema 18. Existing error codes/retryability and provider contracts MUST remain unchanged.

#### Scenario: Discover and round-trip documents
- **WHEN** a protocol-1 client discovers support, creates document text through standalone or aliased batch requests and reads project state
- **THEN** headless and MCP expose identical runs, compatibility text and revision semantics with the advertised capability

#### Scenario: Preserve old clients and reject malformed requests
- **WHEN** existing simple requests or malformed document fields are submitted through each transport and batch/draft surface
- **THEN** simple inputs remain accepted and malformed inputs fail with the established typed error without field loss or state/history mutation

#### Scenario: Verify canonical cross-language evidence
- **WHEN** Rust and TypeScript contract parity checks consume updated positive and negative fixtures
- **THEN** request/output shapes, capability identifiers, version reporting and stable errors agree across every governed consumer
