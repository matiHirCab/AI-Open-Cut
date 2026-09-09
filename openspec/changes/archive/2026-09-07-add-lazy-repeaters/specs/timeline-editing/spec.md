## ADDED Requirements

### Requirement: Transactional repeater editing
Core MUST expose `add_repeater` with required `trackId`, `startMs`, `durationMs`, and complete `repeater`. Repeaters MUST occupy unlocked overlay tracks, default to visible and zero zIndex, and retain identity common transforms with no parent. `update_item` MUST accept repeater-only complete `repeater` replacement; omission MUST preserve the descriptor and explicit null MUST fail. Repeaters MUST support existing timing, move, trim, split, duplicate, delete, z-index, reorder, and visibility workflows. Parent assignment, legacy/common transform changes, Transform2D changes, keyframes, audio edits, transition-endpoint use, and source kinds outside shape/group/component instance MUST fail with `INVALID_ARGUMENT`. Component definition create/update MUST accept local repeaters with identical scope and validation rules.

#### Scenario: Create replace and traverse the supported lifecycle
- **WHEN** a valid repeater is created, replaced, moved, trimmed, split, duplicated, hidden, reordered, deleted, undone, redone, and reopened
- **THEN** its descriptor, timing, source reference, stack position, and generated evaluation remain deterministic with existing revision/history semantics

#### Scenario: Reject unsupported edits atomically
- **WHEN** a repeater is placed on a non-overlay or locked track, receives null/incomplete replacement, parenting, transforms, keyframes, audio, or transition edits, or a non-repeater receives a repeater patch
- **THEN** core returns the existing `TRACK_LOCKED` or non-retryable `INVALID_ARGUMENT` behavior and publishes no partial state or history

### Requirement: Alias-aware repeater transactions
`add_repeater` MUST work standalone and inside the existing ordered 1-to-100 `timeline_batch_edit`, accept `resultAlias` only through existing creation-envelope rules, resolve earlier aliases in `trackId` and `repeater.source.id`, and allow later operations to address the created repeater through its alias. Success MUST commit one revision and undo step with existing deterministic changed-ID and creation-mapping behavior. Missing sources, stale revisions, locked tracks, unresolved/forward/duplicate aliases, invalid graphs, exceeded limits, and later failed operations MUST retain existing error precedence and roll back the complete batch.

#### Scenario: Create sources and repeater by alias
- **WHEN** a batch creates an overlay track and supported source, adds a repeater referencing the source alias, then reorders or updates the repeater through its alias
- **THEN** all edits commit once with deterministic IDs/mappings and undo/redo/reopen restores exact before/after states

#### Scenario: Roll back invalid alias or trailing work
- **WHEN** add_repeater uses an unresolved or forward source alias, duplicates a result alias, creates a cyclic reference after earlier valid edits, or a later batch operation fails
- **THEN** the established alias or typed core error is returned and no revision, project/history file, alias mapping, or artifact is published
