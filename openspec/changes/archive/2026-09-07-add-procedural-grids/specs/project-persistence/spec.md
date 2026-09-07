## ADDED Requirements

### Requirement: Atomic schema 16 grid activation
Core MUST activate grids in persisted schema 16 and migrate supported schemas 1-15 and every retained undo/redo snapshot under the project lock through one recoverable transaction, preserving all prior intermediate migration rules. The 15-to-16 step MUST change only schemaVersion; IDs, content, revisions, ordering, references, provenance, media integrity and existing evaluated output MUST remain unchanged. Source schemas below 16 MUST reject grid items, including hidden/unused definitions, before relabeling. Current and retained state MUST validate before publication. Invalid source content or future versions MUST preserve original in-memory input and authoritative files using existing errors, including INTERNAL_ERROR for unknown future versions. Older binaries MUST reject schema 16; no automatic downgrade SHALL be provided.

#### Scenario: Migrate mixed retained generations
- **WHEN** supported current state and mixed nonempty undo/redo snapshots containing legacy, shape, SVG and component content open
- **THEN** all snapshots migrate atomically to 16 without content/output changes, and undo/redo and repeated reopen remain deterministic without unnecessary rewrites

#### Scenario: Reject invalid current or history
- **WHEN** current, undo or redo contains a grid under schema 15 or earlier, invalid schema-16 grid/reference data, schema zero or an unknown future version
- **THEN** opening fails with existing typed errors and byte-identical authoritative project/history/assets and unchanged in-memory source values

#### Scenario: Recover interrupted activation
- **WHEN** migration is interrupted at any existing persistence fault-injection phase
- **THEN** recovery selects one complete old or new authoritative generation and never mixes current and retained schema versions
