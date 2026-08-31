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
The editor core MUST reject deletion of an asset referenced by a current media timeline item, SHALL retain the pre-deletion asset record in undo history, and SHALL allow garbage collection to remove files only from project-managed asset storage once no asset record in the current project or retained snapshots references them.

#### Scenario: Reject deletion of an in-use asset
- **WHEN** a caller deletes an asset still referenced by a current media timeline item
- **THEN** deletion fails with `ASSET_IN_USE` and neither metadata nor managed media is removed

#### Scenario: Delete an unreferenced asset reversibly
- **WHEN** a caller deletes an asset that is not referenced by the current timeline
- **THEN** its current metadata is removed in a new revision while undo history keeps the prior asset and prevents premature file collection

### Requirement: History-aware integrity and garbage collection
Integrity checks and garbage collection SHALL operate under the project lock, account for current and retained history, and report cleanup failures without corrupting committed project state.

#### Scenario: Collect an unreachable managed file
- **WHEN** a managed file is no longer reachable from the current project or retained history
- **THEN** garbage collection may remove it while preserving every reachable asset and snapshot

#### Scenario: Detect damaged managed media
- **WHEN** a managed asset no longer matches its persisted integrity metadata
- **THEN** the operation reports `ASSET_INTEGRITY_FAILED` rather than silently accepting the file
