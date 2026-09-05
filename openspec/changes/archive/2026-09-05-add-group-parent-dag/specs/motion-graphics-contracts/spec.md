## ADDED Requirements

### Requirement: Governed runtime group contract
A dedicated versioned group-parent-v1 catalog MUST define closed GroupItem, parent, operation, defaults, numeric/graph bounds, ordering/timing/coordinate rules, and success/failure examples. The motion-graphics-v1 catalog SHALL stay fixture-only. Canonical ownership, model, headless requests/responses, MCP schemas/tools, batch variants, project responses, and Rust/TypeScript parity MUST agree. Capability reporting MUST add group_parenting while retaining existing identifiers and protocol major version. New request fields/operations SHALL be additive; existing simple requests and response fields MUST retain their meanings. Schema-10 group content MUST be documented as requiring a group-aware reader.

#### Scenario: Discover and address groups
- **WHEN** a client inspects headless status and MCP discovery
- **THEN** it sees group_parenting plus typed standalone and batch add_group/item_set_parent support and can read the resulting group and parent values

#### Scenario: Reject cross-language drift
- **WHEN** Rust and TypeScript consume valid, boundary, malformed, unknown-field, invalid-number, scope, missing-parent, cycle, and depth fixtures
- **THEN** structural acceptance and core semantic failure evidence match the canonical expectations without duplicating graph validation in transports

#### Scenario: Preserve compatibility and safety
- **WHEN** existing clients submit simple unparented operations or new clients submit path/URI/expression values in structured reference or transform fields
- **THEN** simple requests retain their behavior and malformed new inputs fail through existing typed decoding/validation without new stable errors, provider surfaces, or resource access
