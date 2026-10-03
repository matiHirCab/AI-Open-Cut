## ADDED Requirements

### Requirement: Governed versioned animation preset contracts
The checked-in `contracts/animation-presets-v1.json` MUST canonically define the one-entry catalog, property/value/curve and timing bounds, normalized defaults, source records, fixed resolved primitives, lifecycle and collision examples, and expected valid/invalid outcomes. Contract ownership and designated CODEOWNER coverage MUST include all affected native declarations and consumers. Headless/capability, MCP structural input/output/annotation and project-version artifacts MUST be synchronized under ADR 0002; existing errors/retryability and protocol major 1 MUST remain unchanged. Tests MUST read the same manually reviewed artifacts and independently compare Rust, TypeScript/Zod, headless and MCP behavior; tests MUST NOT generate the expected catalogs from live compiler or registrations.

#### Scenario: Compare fixed successful and failing envelopes
- **WHEN** Rust and TypeScript/transport consumers check canonical seed, curves, endpoint bounds, timing, collision, provenance and unsupported-version fixtures
- **THEN** they agree on fixed expansion/effective parameters and accepted/rejected outcomes using the checked-in source of truth

#### Scenario: Verify additive capability and structural schemas
- **WHEN** contract parity inspects all registered MCP tools and headless nested edits, status and project responses
- **THEN** the new preset tool/capability and optional source field match the reviewed version-1 catalogs and digest while every existing identifier/error remains stable

#### Scenario: Distinguish compilation and historical source versions
- **WHEN** fixtures include an unsupported new application version and a structurally valid persisted historical version absent from the current catalog
- **THEN** compilation rejects the first and project reading/rendering preserves the second without compiler dispatch
