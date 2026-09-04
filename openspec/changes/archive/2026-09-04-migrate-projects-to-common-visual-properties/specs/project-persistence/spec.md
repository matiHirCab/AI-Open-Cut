## ADDED Requirements

### Requirement: Schema-v7 common visual migration is complete and pixel-preserving
Opening any supported schema-version-1-through-6 project MUST deterministically migrate the current project and every retained undo and redo snapshot to schema version 7 under the project lock. Migration MUST preserve every existing transform, visibility value, item identity, timing, ordering, revision, asset reference, and non-visual field exactly; missing common values MUST receive identity transform and `hidden: false` defaults, and the migrated generation MUST evaluate to the same pixels and audio as its schema-v6 source.

#### Scenario: Migrate current state and mixed retained history
- **WHEN** a schema-v6 project has non-empty undo and redo stacks containing supported older snapshots with visible and hidden items and non-default transforms
- **THEN** current state and every retained snapshot become schema v7 in one recoverable generation, existing values remain exact, and only absent common fields receive documented defaults

#### Scenario: Migrate oldest supported project
- **WHEN** a valid schema-v1 project is opened
- **THEN** it follows the deterministic supported migration chain to schema v7 and reopens to an equal schema-v7 state on every later open

#### Scenario: Preserve evaluated output
- **WHEN** equivalent pre-migration and migrated fixtures are evaluated for frame preview, audiovisual range preview, draft preview, and export
- **THEN** they produce equal evaluated semantics and remain within the existing deterministic visual, audio, and timing tolerances

### Requirement: Common visual migration is atomic and fail-closed
The schema-v7 migration MUST deserialize, migrate, and validate the complete current-and-history envelope before project, history, or content-addressed asset publication, MUST publish the migrated project/history pair through one existing crash-consistent transaction, and MUST leave the prior authoritative generation and managed asset store unchanged when any document, snapshot, default, or reference fails validation. Schema version 0 and unknown future versions MUST fail with existing stable compatibility behavior without downgrade or rewrite. Omitted common visual fields on schema-v7 input MUST continue to deserialize to their compatibility defaults without forcing a read-only rewrite; returned state and any later committed serialization MUST contain the explicit canonical fields.

#### Scenario: Reject an invalid retained snapshot
- **WHEN** current state is valid but any retained undo or redo snapshot cannot migrate or validate
- **THEN** open fails before publication and current state, all retained history, and the managed content-addressed asset store remain unchanged

#### Scenario: Accept schema-v7 compatibility defaults
- **WHEN** a schema-v7 document omits `transform` or `hidden` on a timeline item
- **THEN** opening supplies the identity transform and visible default without rewriting solely for those omissions, while returned state and the next committed serialization contain both explicit flattened fields

#### Scenario: Recover an interrupted migration publication
- **WHEN** schema-v7 generation publication is interrupted at any injected persistence phase
- **THEN** deterministic recovery selects one complete authoritative pre-migration or migrated generation and never exposes mixed schema versions

#### Scenario: Reject a future schema
- **WHEN** current state or any retained snapshot declares a schema version newer than 7
- **THEN** open fails closed with existing compatibility behavior and does not downgrade, partially migrate, or rewrite the authoritative generation

#### Scenario: Reject schema version zero
- **WHEN** current state or any retained snapshot declares schema version 0
- **THEN** open fails closed with existing compatibility behavior and does not migrate, rewrite, or publish managed asset content

#### Scenario: Reopen, undo, and redo after migration
- **WHEN** a migrated project is reopened and the user traverses retained undo and redo history
- **THEN** every returned state is schema v7, uses the common visual defaults, preserves its original revision and visual values, and persists deterministically
