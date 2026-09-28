## ADDED Requirements

### Requirement: Governed runtime marker contract
Marker and time-expression runtime fields, operations, capability, persisted schema, stable errors and MCP schemas SHALL be manually synchronized with their canonical versioned catalogs and every governed Rust and TypeScript consumer. The existing fixture-only marker vocabulary SHALL remain the field and variant authority: marker records use scoped ID, name, `timeMs` and `kind:"cue"`; relative expressions use `type:"marker"`, `markerName` and signed `offsetMs`. Activation MUST preserve the catalog's duplicate-name ambiguity classification, exact composition scope and 4096-marker inclusive limit. The catalog's scope validator MUST additionally accept `component:<existing UUID ID>` so runtime component references remain usable, while preserving marker ID/name grammar and rejecting malformed scopes. Public changes MUST remain additive under protocol 1 and pass cross-language parity checks.

#### Scenario: Detect contract drift
- **WHEN** a headless operation, TypeScript union, MCP schema, capability, persisted project field or error differs from its reviewed catalog
- **THEN** the contract parity gate fails with the affected surface

#### Scenario: Preserve fixture semantics at runtime
- **WHEN** a runtime marker or expression reaches an identifier, scope, signed-integer, duplicate-name or collection boundary represented by the canonical fixtures
- **THEN** core accepts or rejects it with the fixture's exact semantics and both language parity suites agree
