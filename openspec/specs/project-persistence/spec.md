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
