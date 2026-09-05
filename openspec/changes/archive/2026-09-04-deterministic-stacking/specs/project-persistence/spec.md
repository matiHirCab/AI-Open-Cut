## ADDED Requirements

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
