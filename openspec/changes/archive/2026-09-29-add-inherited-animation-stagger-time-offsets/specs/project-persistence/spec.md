## ADDED Requirements

### Requirement: Atomic schema-26 inherited timing migration
Core MUST migrate supported schema-25 and older current state and every retained undo/redo snapshot to schema 26 under the project lock in one recoverable generation. Source schemas below 26 MUST reject staggerMs and timeOffsetMs before relabeling. Migration MUST give absent fields zero semantics without changing prior visual/audio output, IDs, revisions, ordering, channels, loops or managed-resource provenance. Every snapshot MUST validate before publication. Malformed timing, invalid state/history, schema zero and unknown future versions MUST fail with existing stable typed errors and leave authoritative project, history and asset bytes unchanged. Interrupted publication MUST recover one complete old or new generation; repeated reopen MUST not rewrite a valid schema-26 generation. Older binaries MUST reject schema 26 rather than downgrade it.

#### Scenario: Migrate current state and history
- **WHEN** a schema-25 project has nonempty undo and redo stacks and no new timing fields
- **THEN** current state and every snapshot reach schema 26 atomically with identical output and deterministic undo/redo/reopen

#### Scenario: Reject invalid retained timing
- **WHEN** a source snapshot contains a pre-26 timing field, malformed timing or unsupported future schema
- **THEN** open fails before rewriting authoritative project, history or media state

#### Scenario: Recover interrupted publication
- **WHEN** migration is interrupted at a supported transaction fault phase
- **THEN** recovery exposes exactly one complete old or new generation without mixed schemas

