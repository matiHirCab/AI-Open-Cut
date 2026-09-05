## ADDED Requirements

### Requirement: Governed additive ungroup contract
The group-parent-v1 runtime catalog MUST govern group_ungroup input, alias semantics, outcomes and failure examples. Canonical headless/MCP catalogs, ownership mappings, Rust requests, TypeScript/Zod standalone and batch unions and registered input/output schemas MUST agree. Headless and MCP status MUST advertise additive group_ungroup capability while retaining existing capabilities, protocol major, schema 10 and stable error retryability. Existing add_group, item_set_parent and item_set_z_index contracts MUST retain their meaning. The remaining roadmap vocabulary MUST stay fixture-only.

#### Scenario: Discover ungroup support
- **WHEN** a client negotiates status and lists MCP tools
- **THEN** it discovers group_ungroup and its typed project/revision/groupId input alongside existing group and ordering operations

#### Scenario: Enforce canonical parity
- **WHEN** canonical valid, alias, missing-field, wrong-type, unknown-field and invalid target fixtures are exercised by Rust and TypeScript consumers
- **THEN** structural acceptance, core semantic outcomes and stable errors agree, including rejection of resultAlias on ungroup and unsafe extra fields

#### Scenario: Preserve compatibility and persistence
- **WHEN** existing simple requests and schema-10 grouped projects, or supported older current/history envelopes, are used after this addition
- **THEN** existing request meanings, migration/recovery behavior and reader rules remain intact with no new persisted fields, while future schemas still fail closed
