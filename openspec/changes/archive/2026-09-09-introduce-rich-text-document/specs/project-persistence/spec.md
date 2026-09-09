## ADDED Requirements

### Requirement: Atomic schema 18 text document migration
Schema 18 MUST activate required text-item documents. Earlier schema requirements MUST apply at their historical intermediate steps; schema 18 MUST now be supported and versions above 18 MUST fail closed. Core MUST validate source snapshots under their declared versions, reject text-item document fields in schemas below 18 while preserving previously legal rich-text slot values, and migrate supported schemas 1-17 plus every retained undo/redo snapshot under the project lock in one recoverable transaction. Migration MUST create exactly one unstyled run from each original root/component text string and preserve every other field, revision, ID, ordering, timing, transform, reference, asset, provenance, draft and evaluated output. Native schema-18 text MUST require document/text agreement. Invalid source/current/history or schema zero MUST retain established typed errors and leave source inputs and authoritative state/assets unchanged; unknown future versions MUST retain INTERNAL_ERROR with no downgrade. Reopen MUST perform no unnecessary rewrite. Interrupted publication MUST recover a complete old or new generation.

#### Scenario: Migrate mixed supported state and history
- **WHEN** current and nonempty retained undo/redo contain supported older versions, root text, hidden/unused/nested component text, existing rich-text slots and legacy draft edits
- **THEN** all snapshots migrate together to 18 with one-run documents, unchanged legacy rendering/content and functional draft materialization, undo/redo and deterministic reopen

#### Scenario: Reject invalid or future source data
- **WHEN** current, undo or redo contains text-item documents below schema 18, missing/mismatched schema-18 documents, invalid references, schema zero or a version above 18
- **THEN** open fails with the established error before publication or asset writes and preserves original in-memory values and authoritative bytes

#### Scenario: Recover every migration interruption
- **WHEN** schema-18 migration is interrupted at each existing persistence fault-injection phase
- **THEN** pre-journal failures preserve the old bytes and later recovery exposes one complete authoritative generation with matching current/history and managed transaction cleanup
