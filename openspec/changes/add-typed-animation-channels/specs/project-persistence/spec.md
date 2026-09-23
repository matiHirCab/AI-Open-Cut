## ADDED Requirements

### Requirement: Atomic typed animation schema migration
The next project schema MUST add optional typed channel storage with empty default to current state and every retained undo/redo snapshot under the project lock. Migration MUST validate every migrated document and publish one recoverable generation or none; existing keyframes and managed-resource provenance MUST remain unchanged. Opening an unknown future schema or malformed channel document MUST fail without rewriting current state, history, or media.

#### Scenario: Migrate retained history
- **WHEN** a supported schema-21 project with nonempty undo and redo stacks is opened
- **THEN** current state and every retained snapshot migrate to the new schema with empty channels and retain the same evaluated output

#### Scenario: Fail migration atomically
- **WHEN** any current or retained document cannot migrate or validate
- **THEN** opening fails and the prior durable generation remains intact

#### Scenario: Reopen an animated project
- **WHEN** an animated project is saved, reopened, undone, and redone
- **THEN** channel identity, values, ordering, and evaluated results remain deterministic
