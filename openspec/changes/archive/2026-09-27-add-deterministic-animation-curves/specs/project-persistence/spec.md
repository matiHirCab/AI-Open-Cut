## ADDED Requirements

### Requirement: Atomic schema 23 curve activation
Persisted schema 23 MUST activate parameterized channel curves. Core MUST validate source-version current state and every retained undo/redo snapshot under the project lock and migrate every supported older schema to 23 through one recoverable transaction. The 22-to-23 step MUST change only the schema version for previously valid documents; it MUST reject new curve objects in source schemas below 23 before relabeling. Existing IDs, revisions, channel values, timing, media provenance, drafts, and evaluated output MUST remain unchanged. Malformed current/history curves, schema zero, and unknown future versions MUST retain established typed failures and leave authoritative project/history/media bytes unchanged. Reopen MUST not rewrite an unchanged schema-23 generation; interrupted publication MUST recover one complete old or new generation. Older binaries MUST reject schema 23 rather than downgrade it.

#### Scenario: Migrate current and history
- **WHEN** a schema-22 project with nonempty undo and redo is opened
- **THEN** all retained states reach schema 23 atomically and previously supported animation samples remain unchanged

#### Scenario: Reopen an active curve
- **WHEN** a schema-23 project with cubic Bézier and spring channels is saved, reopened, undone, and redone
- **THEN** exact curve parameters and deterministic evaluated samples survive every transition

#### Scenario: Reject malformed or future state
- **WHEN** a current or retained snapshot contains a malformed curve, a curve forbidden by its source schema, or an unknown future version
- **THEN** opening fails with its stable typed error without publishing or rewriting authoritative state

#### Scenario: Recover interrupted migration
- **WHEN** migration is interrupted at any existing transaction fault phase
- **THEN** recovery exposes one complete project/history generation and no mixed schema or partial curve state
