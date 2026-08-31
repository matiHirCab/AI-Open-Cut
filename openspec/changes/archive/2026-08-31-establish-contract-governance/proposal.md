## Why

OpenCut's public contracts are repeated across Rust request and response types, TypeScript/Zod schemas, MCP declarations, error catalogs, and fixtures without a documented authority or a single drift gate. Issue #84 must establish a reviewable workflow before motion-graphics contracts add more compatibility surfaces.

## What Changes

- Establish an explicit ownership matrix and compatibility policy for public requests, responses, errors, capabilities, MCP resources, and version negotiation.
- Select and document either deterministic code generation or fixture-governed manual synchronization, including rejected alternatives, unsupported type-system features, and migration cost.
- Add a cross-language parity gate that fails CI when Rust, TypeScript, MCP exposure, and canonical fixtures drift.
- Exercise the selected workflow with a backward-compatible protocol-version field exposed end to end through headless status and MCP status.
- Add contributor and OpenSpec guidance for changing public contracts and obtaining the required review evidence.
- Non-goals: generating editor domain models, changing persisted project schema, redesigning MCP transport, or introducing a breaking protocol version.

## Capabilities

### New Capabilities

- `contract-governance`: Defines canonical ownership, compatibility rules, synchronized fixtures, review requirements, and CI drift evidence for cross-language public contracts.

### Modified Capabilities

- `agent-bridge`: Adds explicit protocol-version negotiation metadata to the existing headless and MCP status contract.

## Impact

- Affects `contracts/`, Rust headless protocol types and tests, TypeScript/Zod bridge schemas and tests, MCP status exposure, CI configuration, contributor guidance, and a new architecture decision record.
- Public status responses gain an additive field; existing request shapes, error codes, persisted projects, and provider protocols remain compatible.
- Contract changes require coordinated Rust, TypeScript, MCP, fixture, documentation, and reviewer updates enforced by the parity gate.
