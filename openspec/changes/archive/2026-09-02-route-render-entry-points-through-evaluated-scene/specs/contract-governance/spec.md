## ADDED Requirements

### Requirement: Canonical rendering-semantics capability synchronization
The additive `evaluated_scene_rendering` capability identifier MUST have one canonical checked-in owner and MUST remain synchronized with Rust headless status production, TypeScript status typing and Zod validation, MCP status exposure, canonical headless and MCP catalogs, and standalone parity evidence.

#### Scenario: Add canonical capability support
- **WHEN** canonical evaluated-scene rendering becomes available to clients
- **THEN** the current protocol major version, canonical fixtures, Rust producer, TypeScript/Zod consumer, MCP surface, and parity tests all accept and report the identical `evaluated_scene_rendering` identifier

#### Scenario: Detect capability drift
- **WHEN** any governed producer, validator, MCP schema, or canonical catalog omits, renames, or reports the capability inconsistently
- **THEN** the standalone contract parity gate fails with the mismatched capability surface

#### Scenario: Preserve existing render contracts
- **WHEN** the new capability is added
- **THEN** all existing frame-preview, range-preview, draft-preview, export request and response shapes remain valid and no project schema, provider contract, stable error, or protocol major version changes
