## ADDED Requirements

### Requirement: Atomic schema-25 animation-loop migration
Core MUST migrate supported schema-24 and older current state and all retained undo/redo snapshots to schema 25 under the project lock in one recoverable generation. Existing channels without `loop` MUST retain their exact values and evaluated visual/audio output. Source schemas below 25 MUST reject loop fields before version relabeling. Every current and retained snapshot MUST validate before publication or managed-asset writes. Invalid source/history data, schema zero, and unknown future versions MUST leave authoritative state unchanged with the existing stable compatibility errors. Interrupted publication MUST recover one complete generation; repeated reopen MUST not rewrite valid migrated state. No downgrade MUST be inferred.

#### Scenario: Migrate current state and retained history
- **WHEN** a schema-24 project has channels and nonempty undo/redo history
- **THEN** current and retained snapshots reach schema 25 atomically, unlooped output remains identical, and undo/redo and repeated reopen are deterministic

#### Scenario: Reject invalid or future retained state
- **WHEN** any source snapshot contains a pre-25 loop field, malformed loop, zero/future schema, or invalid retained data
- **THEN** open fails with the established typed error before rewriting authoritative project, history, or asset state

#### Scenario: Recover an interrupted migration
- **WHEN** publication is interrupted at a supported fault-injection phase
- **THEN** recovery exposes exactly one complete old or new generation, never mixed current/history schemas
