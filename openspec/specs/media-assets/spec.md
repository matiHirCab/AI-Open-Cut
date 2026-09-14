# Media Assets Specification

## Purpose

Define safe ingestion, durable metadata, generated-media provenance, integrity, deletion, and managed-file ownership behavior.

## Requirements

### Requirement: Safe asset ingestion
The headless boundary MUST probe selected media before import, and the editor core MUST resolve the source through the configured path policy, reject traversal or disallowed sources, copy accepted bytes into content-addressed project storage, and persist the resulting asset record under the project lock.

#### Scenario: Import an allowed media file
- **WHEN** a caller imports a supported file from an allowed input root at the current revision
- **THEN** the core stores a managed project copy and returns the new asset identifier and revision

#### Scenario: Reject a disallowed path
- **WHEN** an asset input escapes or falls outside the configured allowed roots
- **THEN** ingestion fails with `PATH_TRAVERSAL` or `PATH_NOT_ALLOWED` and no asset is persisted

### Requirement: Canonical media facts
Persisted assets SHALL record their media type, managed relative path, content hash, and probed media facts needed for validation, deduplication, and integrity checks.

#### Scenario: Persist probed metadata
- **WHEN** supported media is imported successfully
- **THEN** its durable asset record includes a content hash and the applicable duration, dimensions, codec, or sample-rate facts

### Requirement: Generated asset provenance
Generated speech assets MUST retain provider-neutral speech intent plus provider, model, version, sample-rate, and generation metadata sufficient to explain or regenerate the audio.

#### Scenario: Commit generated speech
- **WHEN** synthesized speech is committed as a project asset
- **THEN** the stored asset distinguishes it from an ordinary imported WAV and preserves its speech request and generation provenance

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

### Requirement: Centralized managed font ownership
Core MUST extend its existing ownership discovery and managed-content integrity/collection policy to font catalogs and bindings in current state, all definitions including unused content, retained undo/redo, slot-bearing text and durable drafts. Font content MUST be addressed by its recorded hash and confined to project-managed storage; font records MUST NOT masquerade as timeline audio/video assets. Reachable content MUST NOT be collected. Missing records, missing bytes, hash mismatch and invalid retained face references MUST fail with ASSET_INTEGRITY_FAILED and the owning reference classification without ambient substitution. Lexical and canonical path escapes MUST preserve existing path errors. Removing/replacing text MUST retain fonts needed by another owner or history. Unreachable cleanup failures MUST retain ASSET_GC_FAILED and MUST NOT corrupt committed state.

#### Scenario: Retain fonts through every owner
- **WHEN** a font is reachable only from an unused component, slot-bearing text, a retained draft or an undo/redo snapshot
- **THEN** collection preserves it and the retained owner reopens and renders with the same hash

#### Scenario: Reject damaged or unsafe pinned content
- **WHEN** a bound font record or file is absent, modified, contains an invalid face reference or escapes managed storage
- **THEN** open/integrity/render preflight fails with the specified stable error and no fallback or output publication

#### Scenario: Collect only unreachable font bytes
- **WHEN** the last current, draft and history owner is removed and garbage collection succeeds or encounters a deletion failure
- **THEN** only unreachable managed font files are removed and cleanup failure reports ASSET_GC_FAILED while preserving the committed generation
