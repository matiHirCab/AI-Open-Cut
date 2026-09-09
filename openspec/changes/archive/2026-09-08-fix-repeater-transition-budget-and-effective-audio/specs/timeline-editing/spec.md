## ADDED Requirements

### Requirement: Atomic effective-audio repeater validation
Core MUST apply effective visual-only repeater validation consistently to edits, batches, drafts and reopened projects. Invalid effective-audio sources MUST return non-retryable INVALID_ARGUMENT without publishing changed state, aliases, revision or history. Stale mutations MUST retain retryable REVISION_CONFLICT and existing precedence. Schema 17, protocol 1, public shapes, deterministic IDs and persisted representations MUST remain unchanged.

#### Scenario: Roll back effective-audio mutations and drafts
- **WHEN** a batch or draft creates or updates an instance asset override that makes a repeater source audible, including after earlier valid operations
- **THEN** core returns INVALID_ARGUMENT and authoritative files, revisions, undo/redo history and alias publication remain unchanged

#### Scenario: Preserve valid lifecycle and error compatibility
- **WHEN** valid silent-source edits are saved, undone, redone, reopened or materialized as drafts, an invalid audible-source snapshot is reopened, or a mutation uses a stale revision
- **THEN** valid state and deterministic evaluation survive the lifecycle, invalid snapshots fail without rewriting files, and stale mutations retain REVISION_CONFLICT without changes
