## ADDED Requirements

### Requirement: Transactional procedural grid editing
Core MUST expose add_grid with required trackId, startMs, durationMs and grid plus optional complete transform2d and parent. Grids MUST occupy unlocked overlay tracks, default to identity Transform2D, visible and zero zIndex, and use existing safe-integer half-open timing and ordering rules. Creation MUST return one fresh item ID. update_item MUST accept a grid-only complete grid replacement; omission MUST preserve the descriptor and explicit null MUST fail. Shape geometry/fill/stroke patches MUST NOT target grids. Grids MUST support existing shape-compatible timing, move, trim, split, duplicate, removal, visibility, ordering, parenting, legacy transform and position/scale/opacity keyframe behavior, including existing Transform2D exclusivity. Audio and transition-endpoint edits MUST fail with INVALID_ARGUMENT. Component definition create/update MUST accept local grids with the same validation and scoped parent rules; no grid-specific channels or slot bindings SHALL activate.

#### Scenario: Create replace and traverse the visual lifecycle
- **WHEN** a grid is created, replaced, transformed, animated through supported channels, moved, trimmed, split, duplicated, hidden, reordered, parented, removed, undone/redone and reopened
- **THEN** every resulting descriptor and visual property persists deterministically with existing fresh-ID, revision, ordering and history semantics

#### Scenario: Validate replacement and definitions
- **WHEN** update_item omits grid, supplies a valid complete replacement, targets a different item kind, supplies null or incompatible shape/audio/transition fields, or a definition contains invalid hidden grid content
- **THEN** omission preserves the grid, valid replacement commits, and invalid combinations reject the entire edit with existing non-retryable INVALID_ARGUMENT without publication

### Requirement: Atomic alias-aware grid transactions
add_grid MUST work standalone and within the existing ordered 1-to-100 timeline_batch_edit operations, accepting resultAlias only through existing creation-envelope rules. Earlier trackId and parent.id aliases MUST resolve, and subsequent edits MUST address the created grid through its alias. Success MUST commit one revision/undo step with existing deterministic changed-ID reporting. Missing track/item/parent, locked target, stale revision, unresolved/forward/duplicate aliases and later failed edits MUST retain existing TRACK_NOT_FOUND, ITEM_NOT_FOUND, TRACK_LOCKED, retryable REVISION_CONFLICT and established alias/validation errors and precedence. Failure MUST leave current/history/files and unpublished aliases unchanged.

#### Scenario: Create and edit by aliases
- **WHEN** a batch creates an overlay track, group and grid with aliases and updates, parents and orders the grid through those aliases
- **THEN** it commits once, reports deterministic IDs and alias mappings, and undo/redo/reopen restores exact before/after states

#### Scenario: Preserve failure atomicity
- **WHEN** standalone or batched grid edits encounter missing references, locked tracks, stale revisions, invalid aliases or a failed trailing operation after successful grid creation
- **THEN** the specified existing errors preserve byte-identical authoritative project/history files and publish no partial state
