## ADDED Requirements

### Requirement: Schema-v7 project evidence remains cross-language exact
The canonical project/protocol fixture, Rust declarations, TypeScript declarations, strict Zod schemas, headless serialization, MCP project responses, and parity tests governed for persisted project shape MUST agree on schema version 7 and the flattened common `transform` and `hidden` fields for every timeline-item variant. Existing protocol-version-1 edit request fields, response meanings, operation identifiers, capability identifiers, annotations, and stable errors MUST remain unchanged.

#### Scenario: Verify governed consumers
- **WHEN** the contract parity gate compares a schema-v7 project containing every timeline-item variant
- **THEN** every governed Rust and TypeScript/Zod consumer accepts and emits the same exact additive shape and the canonical catalog matches their structural schemas

#### Scenario: Preserve existing edit clients
- **WHEN** an existing client sends any valid schema-v6-era add, update-transform, visibility, standalone, draft, or batch request
- **THEN** the protocol accepts the unchanged request shape and returns the same operation-level meaning while project state reports schema version 7

#### Scenario: Reject ungoverned activation
- **WHEN** parity evidence detects a new operation, capability, error, motion-graphics runtime concept, or changed retryability not authorized by this change
- **THEN** validation fails rather than accepting the unreviewed contract expansion
