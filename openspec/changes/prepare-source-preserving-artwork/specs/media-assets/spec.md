## ADDED Requirements

### Requirement: Source-preserving artwork derivative ownership
The canonical core asset-reference owner SHALL include every artwork derivative source in current state and retained history/draft catalogs, transitively preserving source bytes and rejecting logical deletion when a current or draft derivative requires them and preserving history-only bytes through the existing history retention policy. Derivations MUST identify same-snapshot managed Image source and exact sha256 content hash, bounded approved parameters/dimensions/offset; missing, cyclic, self, ineligible or mismatched provenance MUST fail closed with ASSET_INTEGRITY_FAILED. Current referenced deletion MUST preserve ASSET_IN_USE and actionable artwork-source classification. Existing media/caption/font/slot/event/draft roots and collection behavior MUST remain unchanged.

#### Scenario: Retain source through derivative and history
- **WHEN** only a derivative uses a source, and derivative deletion, drafts, undo/redo, reopen and eventual history eviction/GC are exercised
- **THEN** source remains protected while reachable and only legitimately unreachable owned bytes are collected

#### Scenario: Reject corrupted derivative provenance
- **WHEN** current or retained provenance contains missing source, wrong hash/type/geometry, self reference or cycle
- **THEN** open/commit/integrity rejects atomically with ASSET_INTEGRITY_FAILED and identifies artwork provenance
