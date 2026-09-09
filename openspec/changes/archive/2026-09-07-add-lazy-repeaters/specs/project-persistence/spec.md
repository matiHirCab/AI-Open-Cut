## ADDED Requirements

### Requirement: Atomic schema 17 repeater activation
Persisted schema 17 MUST activate repeater items. Opening each supported schema 1-16 project MUST validate the source snapshot under its declared version, reject repeater records in current state or retained history below schema 17, and migrate current plus every undo/redo snapshot to schema 17 deterministically and atomically under the project lock before returning state. A 16-to-17 migration MUST otherwise preserve revisions, tracks, items, components, IDs, ordering, timing, transforms, assets, provenance, drafts, and evaluated output. Invalid sources and unknown future versions MUST fail without rewriting authoritative files. Interrupted publication MUST recover one complete pre- or post-migration generation.

#### Scenario: Migrate mixed current and retained history
- **WHEN** a valid schema-16 project with non-empty undo and redo history, root/component items, media ownership, and drafts is opened
- **THEN** current state and all retained snapshots publish together as schema 17 with byte-equivalent domain content apart from schemaVersion and unchanged evaluated output

#### Scenario: Reject repeater content below its source version
- **WHEN** current state, hidden/unused component content, undo, or redo declares schema 16 or earlier while containing a repeater record
- **THEN** opening fails before migration with the stable invalid-input behavior and authoritative project/history bytes remain unchanged

#### Scenario: Recover interruption and reject future versions
- **WHEN** publication is interrupted at any supported fault phase or a project declares schema 18 or newer
- **THEN** reopen recovers one complete generation or fails closed for the future version without partial migration, downgrade, or rewrite
