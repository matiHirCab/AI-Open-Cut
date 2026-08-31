## MODIFIED Requirements

### Requirement: Timeline media deletion guard
The editor core MUST reject deletion of an asset referenced by a current media timeline item, current caption provenance, or retained durable draft, SHALL return `ASSET_IN_USE` with the blocking reference class, SHALL retain the pre-deletion asset record in undo history, and MUST NOT silently detach a persisted reference.

#### Scenario: Reject deletion of an asset used by media
- **WHEN** a caller deletes an asset still referenced by a current media timeline item
- **THEN** deletion fails with `ASSET_IN_USE`, identifies the media reference class, and neither metadata nor managed media is removed

#### Scenario: Reject deletion of a caption source
- **WHEN** a caller deletes an asset still referenced by current caption provenance
- **THEN** deletion fails with `ASSET_IN_USE`, identifies the caption-source reference class, and preserves the caption and asset unchanged

#### Scenario: Reject deletion of an asset retained by a draft
- **WHEN** a caller deletes an asset referenced by a durable draft operation
- **THEN** deletion fails with `ASSET_IN_USE`, identifies the draft reference class, and the draft remains reopenable

#### Scenario: Delete an unreferenced asset reversibly
- **WHEN** a caller deletes an asset that is not referenced by current media, current caption provenance, or a durable draft
- **THEN** its current metadata is removed in a new revision while undo history keeps the prior asset and prevents premature file collection

#### Scenario: Reject deletion at a stale revision
- **WHEN** a caller requests asset deletion with a revision other than the current project revision
- **THEN** deletion fails with `REVISION_CONFLICT` before reference or metadata changes occur

### Requirement: History-aware integrity and garbage collection
Integrity checks and garbage collection SHALL operate under the project lock, MUST use one ownership policy for current state, durable drafts, and retained undo/redo history, and SHALL report cleanup failures without corrupting committed project state.

#### Scenario: Retain a caption source through history
- **WHEN** caption provenance and its source asset exist in a retained undo or redo snapshot
- **THEN** garbage collection preserves the managed source content and undo or redo restores valid provenance

#### Scenario: Retain an asset for a durable draft
- **WHEN** a durable draft references a managed asset that remains in its retained asset catalog
- **THEN** garbage collection preserves the managed content and reopening the draft resolves the same asset

#### Scenario: Collect an unreachable managed file
- **WHEN** a managed file is no longer reachable from the current project, any durable draft, or retained history
- **THEN** garbage collection removes it only from project-managed storage while preserving every reachable asset, draft, and snapshot

#### Scenario: Report managed-file cleanup failure
- **WHEN** metadata commits successfully but an unreachable managed file cannot be removed
- **THEN** the committed result includes `ASSET_GC_FAILED` without corrupting project state

#### Scenario: Detect damaged managed media
- **WHEN** a managed asset no longer matches its persisted integrity metadata
- **THEN** the operation reports `ASSET_INTEGRITY_FAILED` rather than silently accepting the file

## ADDED Requirements

### Requirement: Persisted asset reference integrity
Every persisted media, caption-source, history, and durable-draft asset identifier MUST resolve deterministically through the applicable retained asset catalog, and persisted dangling references MUST fail closed with `ASSET_INTEGRITY_FAILED` and an actionable reference classification.

#### Scenario: Detect a legacy dangling caption source
- **WHEN** persisted caption provenance names an asset absent from the containing project snapshot
- **THEN** project open fails with `ASSET_INTEGRITY_FAILED` and identifies the caption-source reference

#### Scenario: Detect a dangling history reference
- **WHEN** a retained undo or redo snapshot contains a media or caption asset identifier absent from that snapshot's asset records
- **THEN** project open fails with `ASSET_INTEGRITY_FAILED` and identifies the retained reference deterministically

#### Scenario: Detect a dangling draft reference
- **WHEN** a durable draft names an asset that cannot be resolved against the retained project asset catalog
- **THEN** draft reopen, preview, rebase, or commit fails with `ASSET_INTEGRITY_FAILED` and identifies the draft reference

#### Scenario: Preserve valid legacy state without rewriting shape
- **WHEN** an existing project, history, and draft contain only resolvable asset references
- **THEN** they remain readable without adding or removing persisted fields
