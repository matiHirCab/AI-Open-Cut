## MODIFIED Requirements

### Requirement: Fixture-governed synchronization evidence
Every governed cross-language contract change MUST update mandatory canonical fixtures and pass automated parity checks for each affected Rust, TypeScript/Zod, and MCP consumer; those checks MUST verify that derived Rust operation names are accepted by the actual Serde request deserializer, enforce TypeScript types, and compare complete client-visible MCP tool schemas and annotations while excluding description-only schema copy.

#### Scenario: Detect implementation drift
- **WHEN** a Rust wire type, TypeScript validator, MCP declaration, capability or resource identifier, version rule, or stable error diverges from its canonical contract artifact
- **THEN** the standalone contract parity gate fails with the mismatched category and consumer

#### Scenario: Detect a Rust request variant mismatch
- **WHEN** the Rust headless request enum gains, removes, or renames a serialized operation without the canonical operation catalog changing identically
- **THEN** the Rust parity test fails using variant names derived from that enum

#### Scenario: Detect Serde and derived-name drift
- **WHEN** a Rust request variant's Serde wire tag differs from the corresponding derived canonical operation name
- **THEN** the Rust parity test fails even if the checked-in operation catalog still matches the derived name

#### Scenario: Detect an MCP tool definition mismatch
- **WHEN** a registered MCP tool's client-visible structural input schema, structural output schema, or annotations differ from the canonical MCP surface catalog
- **THEN** the TypeScript parity test fails for that named tool

#### Scenario: Ignore MCP schema documentation copy
- **WHEN** only a `description` keyword changes anywhere in a registered tool's input or output JSON Schema
- **THEN** the normalized MCP compatibility definition remains unchanged

#### Scenario: Enforce TypeScript parity in the standalone gate
- **WHEN** a TypeScript-only request union or type constraint diverges from the canonical contract
- **THEN** `bun run contracts:check` fails without relying on a later general CI typecheck step

#### Scenario: Prove an additive workflow
- **WHEN** protocol-version negotiation is added to the status request and response
- **THEN** the same canonical examples are accepted and emitted by Rust, validated by TypeScript, exposed through MCP, and exercised by integration tests
