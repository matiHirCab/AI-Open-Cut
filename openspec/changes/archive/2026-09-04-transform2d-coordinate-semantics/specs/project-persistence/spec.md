## ADDED Requirements

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
