## ADDED Requirements

### Requirement: Canonical sound-definition variant ownership
The existing asset owner SHALL include every sound-definition variant in current/retained snapshots and every pending sound_event_register variant in durable-draft reference discovery, integrity and managed-file collection. Current registered variants and pending draft variants MUST block deletion with existing ASSET_IN_USE and actionable sound-definition or draft-operation classification. Replacement MUST release only current roots; retained history/drafts SHALL preserve prior assets/bytes until existing eviction. Missing persisted/draft variants or damaged managed bytes MUST fail with existing ASSET_INTEGRITY_FAILED and ownership classification; established lexical/canonical path errors MUST remain unchanged. Every old media/caption/slot/font/draft root and integrity/deletion/GC case MUST remain intact, with no parallel collector or ambient content substitution.

#### Scenario: Protect current and pending variants
- **WHEN** deletion targets an asset referenced by a registered definition or pending registration draft
- **THEN** existing ASSET_IN_USE protects the record/bytes and identifies its owning sound-definition or draft-operation reference

#### Scenario: Retain replaced variants through history
- **WHEN** registration replacement releases a prior current variant and later edits/undo/redo/history eviction invoke collection
- **THEN** history/draft roots keep old media until legitimately unreachable and reopening restores the same content hashes

#### Scenario: Reject dangling or damaged registry content
- **WHEN** opening/registration/draft access encounters an absent retained variant, modified managed bytes or unsafe path
- **THEN** the established integrity/path error rejects without rewriting authoritative state, fallback content or partial resources
