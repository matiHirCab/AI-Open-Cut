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
Project mutations and migrations MUST execute while holding the project lock, and each persisted JSON document SHALL be written through a synchronized temporary file and rename so readers never observe a partially written document.

#### Scenario: Publish a project document
- **WHEN** a validated mutation writes a new project or history document under the project lock
- **THEN** the destination contains one complete JSON document rather than a partially written payload

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
