## ADDED Requirements

### Requirement: Governed component definition runtime contract
A component-definitions-v1 catalog MUST define closed definition/instance/operation payloads, identity scopes, defaults, bounds, durations, failures and migration examples. Rust, TypeScript/Zod, headless/MCP catalogs, ownership and parity consumers MUST agree. Runtime status MUST advertise component_definitions without claiming instance evaluation. Existing root operations and protocol major 1 MUST remain compatible. The general motion-graphics catalog MUST remain fixture-only for unactivated slots, rendering and other roadmap behavior.

#### Scenario: Enforce canonical parity
- **WHEN** canonical valid, omitted/null, unknown-field, wrong-type, numeric-boundary and alias examples run through Rust and TypeScript consumers
- **THEN** structural acceptance and stable semantic error evidence match without duplicated transport domain validation

#### Scenario: Discover compatibility
- **WHEN** a client inspects schema, capabilities and tools
- **THEN** it sees schema 11 and typed definition management while existing simple operations retain their meanings
