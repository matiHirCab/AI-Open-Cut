## ADDED Requirements

### Requirement: Atomic schema 24 marker activation
Persisted schema 24 SHALL add empty root and component marker collections and optional item-start expressions. Core MUST migrate current state and every retained undo/redo snapshot from all supported older schemas under the project lock through one recoverable transaction. The 23-to-24 step MUST add only empty marker defaults and the schema label to previously valid documents; it MUST reject marker records or expressions mislabeled as source schema 23. Legacy item IDs, revisions, timing, ordering, media provenance, drafts and evaluated output MUST remain unchanged. Malformed current/history marker data, schema zero and future versions MUST fail closed without authoritative rewrites; unchanged schema-24 reopen MUST not rewrite its generation.

#### Scenario: Migrate current state and history
- **WHEN** a schema-23 project with nonempty undo and redo history is opened
- **THEN** every retained snapshot reaches schema 24 atomically with empty marker defaults and unchanged legacy output

#### Scenario: Preserve marker references through history
- **WHEN** a schema-24 project containing root and component markers and item-start expressions is saved, reopened, undone and redone
- **THEN** all IDs, names, scopes, offsets and resolved timings remain deterministic

#### Scenario: Reject malformed or future snapshots
- **WHEN** current state or retained history has malformed markers, a marker field forbidden by its source schema, or an unknown future schema
- **THEN** opening fails with the established typed error without publishing a partial migration

#### Scenario: Recover an interrupted migration
- **WHEN** schema migration is interrupted at any durable transaction fault phase
- **THEN** recovery exposes one complete old or new project/history generation
