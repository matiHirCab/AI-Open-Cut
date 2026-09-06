## ADDED Requirements

### Requirement: Governed additive component lifecycle contracts
A versioned component-lifecycle-v1 catalog MUST govern duplication payloads, creation aliases, override omission/replacement, unchanged placement semantics, bounds and canonical errors. Contract ownership MUST enumerate every affected native, headless, TypeScript/Zod and MCP consumer. Protocol and MCP catalogs MUST include the new operation and component_lifecycle capability while preserving protocol 1, schema 13, existing operations and stable error retryability. Canonical parity evidence MUST cover valid all-kind values, special slot keys, omitted/empty/null maps, unknown fields, numeric boundaries, aliases and structural versus semantic rejection. Designated CODEOWNER review MUST cover catalogs and consumers before archive.

#### Scenario: Verify lifecycle parity and discovery
- **WHEN** governed consumers process canonical lifecycle fixtures and clients list tools/status
- **THEN** acceptance stages, exact typed values, alias locations, output schemas and capability reporting agree across Rust, TypeScript and MCP

#### Scenario: Preserve existing clients and persisted state
- **WHEN** older valid requests or schema-13 projects and retained history are used with a lifecycle-capable runtime
- **THEN** their meanings and persisted shape remain unchanged with no new migration, and retained migration/future-version rejection tests continue to pass
