## ADDED Requirements

### Requirement: Artwork metadata schema migration
Current and retained supported historical snapshots SHALL migrate deterministically under the existing project lock to schema47 with optional artwork derivation absent for legacy assets. Derivation-bearing snapshots MUST require schema47 and canonical asset integrity. Old public requests, aliases, speech origins and stable errors MUST retain meaning; additive headless/CLI operations and optional MCP project-state metadata SHALL be governed by canonical fixture parity and exact predecessor inverse proofs. Unknown future schemas MUST fail closed without mutation.

#### Scenario: Migrate retained legacy history
- **WHEN** a schema46 or older supported project and its retained undo/redo are opened and reopened
- **THEN** schema47 publication is atomic/idempotent, original asset/source/speech intent is preserved and no derivation is fabricated

#### Scenario: Reject future or premature derivation
- **WHEN** unsupported future schema or derivation metadata tagged with an older schema is loaded
- **THEN** typed failure preserves authoritative project/history bytes without guessed downgrade or silent field removal
