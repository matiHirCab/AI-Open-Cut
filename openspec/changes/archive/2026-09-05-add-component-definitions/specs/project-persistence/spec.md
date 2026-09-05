## ADDED Requirements

### Requirement: Atomic schema-11 component migration
The current schema MUST become 11. Earlier schema requirements MUST apply to intermediate migrations. Supported schemas 1–10 MUST migrate current state and every retained undo/redo snapshot under lock, adding empty components without changing root IDs, revisions, timing, ordering, transforms, media, provenance or evaluated output. Schema 11 MUST require its components collection. The complete migrated envelope MUST validate before atomic publication or managed-asset writes; malformed current/history state, schema zero and future versions MUST leave authoritative files unchanged with existing stable compatibility errors. Interrupted publication MUST recover one complete generation.

#### Scenario: Migrate mixed retained history
- **WHEN** a supported project has nonempty undo and redo stacks with older snapshots
- **THEN** the entire envelope migrates deterministically to schema 11 and repeated reopen performs no additional rewrite

#### Scenario: Reject and recover atomically
- **WHEN** retained data is invalid or future-versioned, or publication is interrupted at an injected phase
- **THEN** invalid input publishes nothing and recovery selects a complete old or new generation

### Requirement: Component managed media retention
Core MUST include assets referenced by all definitions, retained history and durable drafts in existing integrity, deletion and garbage-collection decisions. Definition removal MUST NOT discard media still reachable through any retained owner, and component resource fields MUST retain existing confinement and provenance validation.

#### Scenario: Retain unused definition media
- **WHEN** an asset is referenced only by an unused definition, undo/redo snapshot or durable draft
- **THEN** integrity validation sees the reference and collection/deletion preserves it according to existing ownership rules

#### Scenario: Reject unsafe component resources
- **WHEN** a definition contains an invalid asset reference or resource escaping existing managed boundaries
- **THEN** core rejects it without changing files or publishing artifacts
