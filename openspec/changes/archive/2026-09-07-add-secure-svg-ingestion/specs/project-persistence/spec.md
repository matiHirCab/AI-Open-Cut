## MODIFIED Requirements

### Requirement: Atomic schema 14 shape activation
Schema 14 MUST remain the shape activation milestone; prior milestones MUST apply to intermediate migrations, followed by any newer supported migration in the same recoverable transaction. Supported schemas 1–13 MUST migrate current state and every retained undo/redo snapshot under lock through one recoverable transaction. Source-schema validation MUST reject shape items in schemas below 14, including hidden and unused definitions, before relabeling. The 13-to-14 step MUST only change schema version, preserving all existing content, revisions, references, assets, provenance and evaluated output. No legacy rectangle conversion or downgrade MUST occur.

Every current and retained snapshot MUST validate before publication or managed-asset writes. Invalid geometry, invalid references, schema zero and unknown future versions MUST leave authoritative state unchanged with existing errors; future versions MUST retain INTERNAL_ERROR. Reopening a migrated project MUST be deterministic; native schema-14 projects MUST also undergo the current supported migrations without changing their shape content.

#### Scenario: Migrate mixed current and retained history
- **WHEN** supported older state with mixed nonempty undo/redo and component definitions opens
- **THEN** all snapshots pass the schema-14 activation step and reach the current supported schema atomically without changing legacy content/output and repeated reopening performs no extra migration rewrite

#### Scenario: Reject malformed source or retained state
- **WHEN** any current/undo/redo snapshot contains old-schema shape data, invalid geometry/references, schema zero or a future version
- **THEN** opening fails before publication and leaves in-memory source inputs and authoritative project/history/assets unchanged

#### Scenario: Recover and traverse shape history
- **WHEN** publication is interrupted at each existing fault-injection phase or a schema-14 shape edit is undone/redone and reopened
- **THEN** recovery selects one complete generation and every returned state preserves exact geometry, paint, revisions and history semantics
