## ADDED Requirements

### Requirement: Atomic schema-12 slot migration
The current schema MUST become 12. Earlier schema requirements MUST apply to intermediate migrations. Supported schemas 1–11 MUST migrate current state and all retained undo/redo snapshots under the project lock in one recoverable generation, adding required empty `slots` to component definitions and `slotValues` to nested instances. Other values, IDs, revisions, provenance, media and rendered output MUST remain unchanged. Schema 12 MUST require both fields where applicable and reject malformed current/history values before publication or managed-asset writes. Schema zero and unknown future versions MUST retain existing compatibility errors, including INTERNAL_ERROR for future versions, without rewriting files. Reopening a migrated project MUST perform no additional migration rewrite. No downgrade MUST be inferred.

#### Scenario: Migrate mixed retained history
- **WHEN** a schema-11 project with nested components and mixed supported undo/redo snapshots is opened
- **THEN** the complete envelope becomes schema 12 atomically with deterministic empty slot fields and preserved unrelated values

#### Scenario: Fail closed and recover interruptions
- **WHEN** any snapshot is malformed/future-versioned or publication is interrupted at each supported injection phase
- **THEN** invalid input publishes nothing and recovery exposes exactly one complete old or new generation

### Requirement: Retain assets referenced by slot values
Managed-asset integrity, deletion and garbage collection MUST include every slot default and instance asset override in current definitions, retained undo/redo and durable drafts, including unused/hidden definitions and overridden defaults. Removing one slot reference MUST NOT release an asset still retained by another owner. Existing core asset errors and path confinement MUST remain unchanged.

#### Scenario: Preserve slot-only media
- **WHEN** an asset is referenced only by a default, override, retained snapshot or durable draft
- **THEN** integrity sees it and existing deletion/collection rules protect it until no retained owner remains
