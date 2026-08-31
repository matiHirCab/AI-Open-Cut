## MODIFIED Requirements

### Requirement: Atomic caption commit
Committing a transcription preview MUST validate the current project revision, target asset, caption track, ordered non-overlapping segment timing, word bounds, confidence values, and caption style, then SHALL publish all caption items with durable source-asset provenance as one project revision.

#### Scenario: Commit captions
- **WHEN** a caller commits a valid preview token for an existing asset at the current revision
- **THEN** the core inserts all resulting captions with the source asset identifier as one revision and consumes the preview

#### Scenario: Reject a stale caption commit
- **WHEN** the project revision changed before preview commit
- **THEN** commit fails with `REVISION_CONFLICT` and does not partially insert captions

#### Scenario: Reject a missing caption source
- **WHEN** a transcription commit names an asset absent from the current project
- **THEN** commit fails with `ASSET_NOT_FOUND` and no caption provenance is persisted

## ADDED Requirements

### Requirement: Durable caption-source lifecycle
Caption source provenance MUST remain resolvable while the caption is current or retained in undo/redo history, MUST block logical source-asset deletion while current, and MUST NOT be silently detached by deletion or garbage collection.

#### Scenario: Block deletion while captions are current
- **WHEN** one or more current captions retain provenance for a source asset
- **THEN** deleting that asset fails with `ASSET_IN_USE` and all captions and provenance remain unchanged

#### Scenario: Restore caption provenance through undo and redo
- **WHEN** an edit containing caption provenance is undone and subsequently redone within retained history
- **THEN** each restored caption resolves the same source asset and its managed content remains available

#### Scenario: Reopen caption provenance
- **WHEN** a valid captioned project is closed and reopened
- **THEN** every caption retains and resolves its original source asset identifier

#### Scenario: Reject legacy dangling caption provenance
- **WHEN** a reopened current or retained project snapshot contains caption provenance whose source asset is absent
- **THEN** open fails deterministically with `ASSET_INTEGRITY_FAILED` and identifies the caption-source reference
