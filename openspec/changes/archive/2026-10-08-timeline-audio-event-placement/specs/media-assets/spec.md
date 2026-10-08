## ADDED Requirements

### Requirement: Centralized captured audio-event content ownership
Existing core managed-media reference collection SHALL retain captured event assetId in current/components, all history and materialized drafts even when a definition is replaced. Captured contentHash MUST match the managed eligible asset's canonical sha256; malformed/dangling persisted references MUST retain existing ASSET_INTEGRITY_FAILED and media integrity rules. Current/draft referenced deletion MUST use ASSET_IN_USE, and history-only content SHALL remain until eligible garbage collection under existing ownership policy. Version2 placement-bearing drafts MUST persist an optional nonnull ordered deduplicated bounded audioEventAssetIds root list derived by canonical replay, retaining stale pending selected bytes after definition replacement. Successful create/update/rebase MUST recapture roots; same-base replay MUST reject mismatched roots; rebase SHALL explicitly re-resolve intent against its new base. Old drafts without placement SHALL omit the field. No parallel collector or transport validator SHALL be introduced.

#### Scenario: Replace definition and retain placed old content
- **WHEN** a definition changes away from a placed event's selected asset, then deletion/undo/history/GC are exercised
- **THEN** current and draft event roots block deletion, history preserves exact content and only unreferenced eligible bytes are collected

#### Scenario: Reject damaged or missing captured content
- **WHEN** a retained event points to absent/ineligible media, mismatched hash or damaged managed bytes
- **THEN** canonical integrity validation rejects without publishing candidate or incidental migration resources
