# Project Persistence Specification

## Purpose

Define the durable project lifecycle, concurrency, history, and schema compatibility guarantees owned by the editor core.

## Requirements

### Requirement: Durable project lifecycle
The editor core SHALL create projects with validated settings and stable identifiers, persist them beneath the configured project root, list persisted projects, and reopen their current state.

#### Scenario: Create and reopen a project
- **WHEN** a caller creates a project with valid settings and later opens its identifier
- **THEN** the reopened project contains the persisted settings, initial tracks, revision, and timeline state

#### Scenario: Open a missing project
- **WHEN** a caller opens an identifier that has no persisted project
- **THEN** the operation fails with the stable `PROJECT_NOT_FOUND` error

### Requirement: Optimistic revision control
Every mutation of an existing committed project state that accepts an expected revision SHALL compare it with the current persisted revision and SHALL reject stale writers before publishing the requested state change.

#### Scenario: Reject a stale mutation
- **WHEN** a mutation supplies an expected revision different from the current revision
- **THEN** the operation fails with retryable `REVISION_CONFLICT` and the persisted project remains unchanged

### Requirement: Serialized durable persistence
Project mutations, project creation, and migrations MUST execute while holding the project lock, and each logical write of project state plus retained history MUST use one recoverable transaction whose durable commit point identifies a single authoritative generation. Each persisted JSON document SHALL be published through a synchronized temporary file and atomic replacement so readers never observe a partially written document.

#### Scenario: Publish a project generation
- **WHEN** a validated mutation commits new project state and retained history under the project lock
- **THEN** every subsequent locked read observes the project and history from that committed generation rather than a mixed pair

#### Scenario: Reject before the commit point
- **WHEN** persistence fails before the transaction commit point is durably published
- **THEN** the mutation fails and the prior project and history generation remains authoritative

#### Scenario: Interrupt after the commit point
- **WHEN** persistence is interrupted after the transaction commit point but before every destination is materialized
- **THEN** the target generation remains recoverable and the mutation is not reported as rejected

### Requirement: Deterministic interrupted-transaction recovery
The editor core MUST recover a valid interrupted transaction deterministically under the project lock before returning or mutating project state, MUST remove all managed transaction artifacts after successful recovery, and MUST fail closed with non-retryable `PROJECT_RECOVERY_FAILED` when recovery metadata is corrupt, unsupported, or inconsistent.

#### Scenario: Recover every interrupted publication phase
- **WHEN** a project is opened after termination between any two persistence phases following the commit point
- **THEN** recovery publishes the transaction's project and history together, completes any recorded draft consumption, and removes managed transaction artifacts

#### Scenario: Repeat interrupted recovery
- **WHEN** recovery itself is interrupted and the project is opened again
- **THEN** replay converges on the same committed generation without duplicating a mutation or pairing history from another generation

#### Scenario: Reject irrecoverable metadata
- **WHEN** transaction recovery metadata has an unsupported version, invalid content, or a project identity inconsistent with its directory
- **THEN** opening fails with `PROJECT_RECOVERY_FAILED` without guessing, defaulting history, or rewriting the live project documents

### Requirement: Unambiguous acknowledged mutation outcome
The editor core SHALL report a mutation as rejected only before its durable transaction commit point, and SHALL return the committed revision with stable `PERSISTENCE_RECOVERY_PENDING` warning when post-commit materialization remains for deterministic recovery.

#### Scenario: Report post-commit materialization failure
- **WHEN** the transaction commit point is durable but project or history materialization cannot finish before returning to the caller
- **THEN** the result identifies the committed revision and includes `PERSISTENCE_RECOVERY_PENDING` rather than returning a mutation error

#### Scenario: Access after a recovery warning
- **WHEN** a caller accesses the project after receiving `PERSISTENCE_RECOVERY_PENDING`
- **THEN** the core finishes recovery under the lock before evaluating the new request against the committed revision

### Requirement: Retained undo and redo history
The editor core SHALL retain at most 100 project snapshots in the undo stack, maintain redo snapshots until a new edit clears them, and apply history operations using the same revision conflict protections as other committed writes.

#### Scenario: Undo and redo an edit
- **WHEN** a caller undoes a committed edit and then redoes it using the returned revisions
- **THEN** the project state transitions through the retained snapshots and each transition increments the current revision

### Requirement: Deterministic schema compatibility
The editor core MUST migrate supported older project schemas and retained history deterministically under lock, and MUST reject unknown future schema versions without rewriting them.

#### Scenario: Migrate a supported project
- **WHEN** a supported older project is opened
- **THEN** its current state and each retained undo and redo snapshot are deterministically upgraded to the current schema before being returned

#### Scenario: Reject a future schema
- **WHEN** a project declares a schema version newer than the running editor supports
- **THEN** opening fails with `INTERNAL_ERROR` and the stored project is not downgraded or rewritten

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

### Requirement: Schema-v8 Transform2D migration
The current schema SHALL be 8. Supported schemas 1 through 7 MUST migrate current state and every retained undo/redo snapshot under the project lock to schema 8. Migration MUST preserve legacy transform fields exactly, default transform2d to absent, and preserve revisions, identities, media, provenance, ordering, and legacy evaluated output. Historical schema-v7 migration requirements apply to the intermediate step, followed by this schema-v8 step.

#### Scenario: Upgrade mixed history
- **WHEN** a supported old project with non-default transforms and mixed supported undo/redo snapshots opens
- **THEN** the complete envelope becomes schema 8 in one recoverable generation with unchanged legacy output

#### Scenario: Reopen and traverse history
- **WHEN** a migrated project or a schema-8 project with Transform2D is reopened, undone, and redone
- **THEN** every state is schema 8 and preserves the exact active transform and original history semantics

### Requirement: Schema-v8 migration fails atomically
Core MUST validate the complete migrated envelope before publication or managed-asset writes. Invalid transforms, invalid references, schema 0, and versions above 8 in current state or history MUST preserve the prior authoritative generation and managed assets. Future versions MUST retain the existing INTERNAL_ERROR compatibility behavior. Interrupted migration MUST recover one complete generation. Schema-8 omission of optional transform2d MUST select legacy behavior without a rewrite solely for the omission.

#### Scenario: Reject invalid or future history
- **WHEN** any current or retained snapshot has invalid data, version 0, or an unknown future version
- **THEN** open fails with the existing typed validation/compatibility error and no partial migration or asset publication occurs

#### Scenario: Recover each publication fault
- **WHEN** migration publication is interrupted at any supported persistence injection phase
- **THEN** recovery returns one complete old or new authoritative generation, never a mixed pair

#### Scenario: Read an omitted optional field
- **WHEN** a valid schema-8 item omits transform2d
- **THEN** reading uses its legacy transform without rewriting solely to insert the optional field

### Requirement: Schema-v9 stacking migration
The current schema MUST be 9. Historical schema-v7 and schema-v8 requirements MUST apply to their intermediate steps, followed by migration to 9. Supported schemas 1 through 8 MUST migrate current state and every retained undo/redo snapshot under the project lock, assigning zIndex zero and stackOrder from each snapshot's item arrays. IDs, revisions, transforms, timing, media, provenance, existing order, and evaluated output MUST be preserved.

#### Scenario: Migrate oldest and mixed history
- **WHEN** supported older current state and mixed-version retained undo and redo snapshots are opened
- **THEN** the complete envelope becomes schema 9 in one recoverable generation with explicit ordering values and unchanged legacy output

#### Scenario: Reopen and traverse migrated history
- **WHEN** a migrated project is reopened, undone, and redone
- **THEN** every resulting snapshot retains deterministic schema-9 ordering and the established revision behavior

### Requirement: Stacking migration fails closed
Migration MUST validate the entire envelope before publication or managed-asset writes. Invalid current/history data, schema zero, and versions above 9 MUST leave the previous authoritative generation and managed assets unchanged. Unknown future versions MUST retain INTERNAL_ERROR compatibility behavior. Interrupted publication MUST recover one complete generation.

#### Scenario: Reject malformed or future history
- **WHEN** any retained snapshot is invalid or has schema zero or an unknown future version
- **THEN** opening returns the existing typed validation/compatibility error without partial migration or asset publication

#### Scenario: Recover migration interruption
- **WHEN** publication is interrupted at each supported persistence fault-injection phase
- **THEN** recovery exposes one authoritative complete generation, never a mixed current/history pair

### Requirement: Schema-v10 group migration
The current schema MUST become 10, with prior schema requirements applying to intermediate migrations. Supported schemas 1 through 9 MUST migrate current state and every retained undo/redo snapshot under the project lock in one recoverable generation. Existing items MUST default to unparented without adding groups or changing IDs, revisions, timing, transforms, order, provenance, media, or evaluated output. Omitted optional parent in schema 10 MUST mean unparented without a rewrite solely for omission.

#### Scenario: Migrate complete mixed history
- **WHEN** supported older current state and mixed-version retained snapshots open
- **THEN** all become schema 10 atomically and legacy visual/audio output remains unchanged

#### Scenario: Retain a grouped project
- **WHEN** a schema-10 graph is reopened, undone, and redone
- **THEN** each resulting state preserves exact group properties, parent references, ordering, and established revision semantics

### Requirement: Hierarchy migration fails closed
Core MUST validate every migrated/current/history graph before publication or managed-asset writes. Invalid graphs or data, schema zero, and unknown versions above 10 MUST leave authoritative state and assets unchanged; future versions SHALL retain INTERNAL_ERROR compatibility behavior. Recovery MUST expose one complete old or new generation.

#### Scenario: Reject invalid retained hierarchy
- **WHEN** a retained snapshot contains a missing parent, cycle, cross-scope edge, excessive depth, invalid transform, or future version
- **THEN** opening fails with the established typed error without partial migration

#### Scenario: Recover migration interruptions
- **WHEN** migration encounters each supported persistence fault-injection point
- **THEN** recovery returns one complete authoritative generation with matching current state and history

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
