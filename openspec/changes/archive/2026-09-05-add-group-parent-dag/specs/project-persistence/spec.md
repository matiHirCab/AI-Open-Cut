## ADDED Requirements

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
