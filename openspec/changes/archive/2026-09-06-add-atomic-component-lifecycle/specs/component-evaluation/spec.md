## ADDED Requirements

### Requirement: Atomic component instance duplication with overrides
Core MUST provide component_instance_duplicate with itemId, required offsetMs and optional slotValues. The source MUST be a root component_instance. The edit MUST append one fresh-ID copy to its existing overlay track, retain its component reference, root parent, visual fields, z-index, trim, duration and timeScale, and set startMs to checked source startMs plus offsetMs. offsetMs and resulting start/end times MUST obey existing nonnegative JavaScript-safe integer bounds. Omitted slotValues MUST copy source overrides; an explicit map MUST replace the complete map, including empty maps. Shared definitions and the source MUST remain unchanged. The mutation result MUST report the new ID and support resultAlias under existing batch envelope semantics. Existing duplicate_items MUST retain its meaning.

#### Scenario: Duplicate with independent values
- **WHEN** a root instance is duplicated with omitted overrides or valid replacement maps covering all eight slot kinds and legal special keys
- **THEN** the copy retains the specified placement fields, gets a distinct ID and requested offset, uses the copied or replacement values exactly, and later copy edits leave the source and definition unchanged

#### Scenario: Resolve defaults after clearing overrides
- **WHEN** duplication specifies an empty override map
- **THEN** defaults and optional base values apply, while any required value without a default fails with INVALID_ARGUMENT

#### Scenario: Reject invalid duplicate candidates
- **WHEN** the source is absent, is not a root instance, its track is locked, an override references a missing slot/asset, or timing, value type, effective binding or stored aggregate complexity is invalid
- **THEN** the edit fails with ITEM_NOT_FOUND, INVALID_ARGUMENT, TRACK_LOCKED or ASSET_NOT_FOUND as applicable and preserves the revision and byte-identical project/history files

#### Scenario: Enforce existing inclusive bounds
- **WHEN** a duplicate reaches versus exceeds an existing timing or slot/text bound, or contains non-finite numbers, unsafe resource forms or unknown closed-record fields
- **THEN** valid inclusive boundaries succeed and invalid candidates fail before project/history publication under existing core and structural validation rules

### Requirement: Transactional complete component lifecycle
Creation, slot definition, placement and duplication MUST compose through existing standalone edits and ordered 1-to-100 timeline_batch_edit operations. Earlier creation aliases MUST resolve in duplication itemId and its resultAlias MUST be usable by later edits; slot keys and values MUST remain literal. Existing duplicate/unresolved/forward alias errors MUST remain unchanged. A successful batch MUST commit once with one undo step; stale revisions MUST fail with retryable REVISION_CONFLICT and later failures MUST roll back the complete batch. Undo, redo and reopen MUST preserve exact IDs, order, overrides and definitions. Duplication MUST reuse shared component evaluation without new timing, coordinate, visibility, audio, ordering or fallback rules.

#### Scenario: Build and duplicate a template in one transaction
- **WHEN** a batch creates a component, defines slots, places an instance, duplicates it using aliases and edits the duplicate through its alias
- **THEN** all results resolve in order with one revision/undo step and undo/redo/reopen restores exact before/after snapshots

#### Scenario: Reject conflicts and alias or trailing failures
- **WHEN** a lifecycle request has a stale revision, an invalid alias envelope, an unresolved/forward alias or a later invalid operation
- **THEN** the existing canonical error is returned and no project/history changes from the request are published

#### Scenario: Preserve rendering and saved compatibility
- **WHEN** a duplicate is compared with an equivalent explicitly placed instance across frame, range, draft preview and export, including after schema-13 reopen or supported current/history migration
- **THEN** both use equivalent evaluated facts and deterministic output, existing simple edits remain valid, and unknown future schemas still fail closed

#### Scenario: Preserve expanded render preflight
- **WHEN** duplicated content reaches versus exceeds existing expanded-occurrence or scene bounds
- **THEN** valid bounded content renders and excessive expansion fails with INVALID_ARGUMENT before render artifact preparation or backend execution, preserving the saved project/history
