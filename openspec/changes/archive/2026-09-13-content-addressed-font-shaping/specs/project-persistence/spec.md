## ADDED Requirements

### Requirement: Atomic schema 19 font activation
Schema 19 MUST activate required font catalogs and bindings for text, with empty catalogs valid for snapshots without text. Earlier schema milestones MUST apply as intermediate steps; schema 18 MUST remain accepted and versions above 19 MUST fail with the existing INTERNAL_ERROR compatibility failure. Core MUST source-validate all supported schemas 1-18 and every retained undo/redo snapshot before resolving fonts, reject schema-19 fields in older source versions and reject missing or malformed bindings in schema 19. Current state, retained history, retained draft font bindings and required managed font content MUST publish as one recoverable generation under the project lock. Migration MUST preserve IDs, revisions, text/documents, ordering, timing, transforms, media, provenance and draft operations, while selecting the new explicit text-layout profile. Migration MUST document the one-time shaping/layout change and MUST NOT promise schema-18 pixel identity for text. After successful migration, reopen MUST never re-resolve fonts or rewrite an unchanged generation. Invalid sources, unavailable required fonts or integrity failures MUST publish nothing; source values and authoritative files MUST remain unchanged. No downgrade SHALL be inferred.

#### Scenario: Migrate mixed retained snapshots and drafts
- **WHEN** supported current state and mixed nonempty undo/redo include root, hidden/unused component text, rich-text slots and durable draft edits
- **THEN** all retained state and font content migrate together, content/identity/revisions are preserved, and undo/redo, drafts and repeated reopen retain the new exact layout and hashes

#### Scenario: Reject invalid source and missing fonts atomically
- **WHEN** any snapshot has forbidden source-version fields, malformed current bindings, schema zero, an unknown future version or unresolvable required faces
- **THEN** migration returns the established typed error with unchanged source values and authoritative project/history/draft files and no newly owned font content

#### Scenario: Recover font activation interruptions
- **WHEN** publication is interrupted at every existing transaction fault phase and each added font/draft staging phase
- **THEN** recovery selects one complete old or new generation, never references absent staged fonts and cleans unreferenced transaction files

## MODIFIED Requirements

### Requirement: Atomic schema 18 text document migration
Schema 18 MUST activate required text-item documents. Earlier schema requirements MUST apply at their historical intermediate steps; schema 18 MUST remain a supported intermediate version before schema 19 font activation; versions above the current supported schema MUST fail closed. Core MUST validate source snapshots under their declared versions, reject text-item document fields in schemas below 18 while preserving previously legal rich-text slot values, and migrate supported schemas 1-17 plus every retained undo/redo snapshot under the project lock in one recoverable transaction. Migration MUST create exactly one unstyled run from each original root/component text string and preserve every other field, revision, ID, ordering, timing, transform, reference, asset, provenance, draft and evaluated output at this intermediate step; the subsequent schema-19 shaping transition has its separately documented visual impact. Native schema-18 text MUST require document/text agreement. Invalid source/current/history or schema zero MUST retain established typed errors and leave source inputs and authoritative state/assets unchanged; unknown future versions MUST retain INTERNAL_ERROR with no downgrade. Reopen MUST perform no unnecessary rewrite. Interrupted publication MUST recover a complete old or new generation.

#### Scenario: Migrate mixed supported state and history
- **WHEN** current and nonempty retained undo/redo contain supported older versions, root text, hidden/unused/nested component text, existing rich-text slots and legacy draft edits
- **THEN** all snapshots migrate together to 18 with one-run documents, unchanged legacy rendering/content and functional draft materialization, undo/redo and deterministic reopen

#### Scenario: Reject invalid or future source data
- **WHEN** current, undo or redo contains text-item documents below schema 18, missing/mismatched schema-18 documents, invalid references, schema zero or a version above the current supported schema
- **THEN** open fails with the established error before publication or asset writes and preserves original in-memory values and authoritative bytes

#### Scenario: Recover every migration interruption
- **WHEN** schema-18 migration is interrupted at each existing persistence fault-injection phase
- **THEN** pre-journal failures preserve the old bytes and later recovery exposes one complete authoritative generation with matching current/history and managed transaction cleanup
