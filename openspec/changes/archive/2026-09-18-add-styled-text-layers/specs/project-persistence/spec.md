## ADDED Requirements

### Requirement: Atomic schema 20 styled text activation
Schema 20 MUST activate optional document spans and text paint stacks. Schema 19 MUST remain a supported intermediate version; the schema-19 future-version rejection MUST henceforth apply to versions above 20. All supported older current snapshots and every retained undo/redo snapshot MUST be source-validated and migrated under the project lock as one recoverable generation. Source versions below 20 MUST reject new span/paint fields, including hidden/unused component and rich-text slot data, before publication. Schema-19 migration MUST preserve absence of new fields and exact content, font bindings/catalogs, IDs, revisions, timing, transforms, ordering, media and provenance. Older migrations MUST retain their existing documented intermediate semantics. Retained drafts MUST preserve operations/bindings and materialize consistently with migrated state; existing draft format/integrity policy MUST govern activation without silently discarding valid drafts. Invalid current/history/draft sources MUST leave authoritative bytes unchanged with established typed errors. Versions above 20 MUST fail with INTERNAL_ERROR and no downgrade. Reopen of unchanged schema 20 MUST perform no unnecessary rewrite or font lookup. Interruption MUST recover one complete old or new generation.

#### Scenario: Migrate current state and retained history
- **WHEN** supported mixed-version snapshots include root/component text, rich-text slots, nonempty undo/redo and retained drafts
- **THEN** activation produces coherent schema-20 state/history with original applicable styles and bindings, working drafts and deterministic undo/redo/reopen

#### Scenario: Reject invalid sources without publication
- **WHEN** current, history or drafts contain forbidden source-version fields, invalid references/styles, schema zero or an unsupported future version
- **THEN** existing typed errors preserve source values and authoritative project/history/draft/font bytes

#### Scenario: Recover interrupted styled text activation
- **WHEN** migration fails at every existing staging/journal/project/history/draft/cleanup fault-injection phase
- **THEN** pre-commit failure preserves the old generation and subsequent recovery exposes exactly one complete generation without leaked transaction files

## MODIFIED Requirements

### Requirement: Atomic schema 19 font activation
Schema 19 MUST activate required font catalogs and bindings for text, with empty catalogs valid for snapshots without text. Earlier schema milestones MUST apply as intermediate steps; schema 18 MUST remain accepted and schema 19 MUST remain a supported intermediate version before schema 20 styled-text activation, and versions above the current supported schema MUST fail with the existing INTERNAL_ERROR compatibility failure. Core MUST source-validate all supported schemas 1-18 and every retained undo/redo snapshot before resolving fonts, reject schema-19 fields in older source versions and reject missing or malformed bindings in schema 19. Current state, retained history, retained draft font bindings and required managed font content MUST publish as one recoverable generation under the project lock. Migration MUST preserve IDs, revisions, text/documents, ordering, timing, transforms, media, provenance and draft operations, while selecting the new explicit text-layout profile. Migration MUST document the one-time shaping/layout change and MUST NOT promise schema-18 pixel identity for text. After successful migration, reopen MUST never re-resolve fonts or rewrite an unchanged generation. Invalid sources, unavailable required fonts or integrity failures MUST publish nothing; source values and authoritative files MUST remain unchanged. No downgrade SHALL be inferred.

#### Scenario: Migrate mixed retained snapshots and drafts
- **WHEN** supported current state and mixed nonempty undo/redo include root, hidden/unused component text, rich-text slots and durable draft edits
- **THEN** all retained state and font content migrate together, content/identity/revisions are preserved, and undo/redo, drafts and repeated reopen retain the new exact layout and hashes

#### Scenario: Reject invalid source and missing fonts atomically
- **WHEN** any snapshot has forbidden source-version fields, malformed current bindings, schema zero, an unknown future version or unresolvable required faces
- **THEN** migration returns the established typed error with unchanged source values and authoritative project/history/draft files and no newly owned font content

#### Scenario: Recover font activation interruptions
- **WHEN** publication is interrupted at every existing transaction fault phase and each added font/draft staging phase
- **THEN** recovery selects one complete old or new generation, never references absent staged fonts and cleans unreferenced transaction files
