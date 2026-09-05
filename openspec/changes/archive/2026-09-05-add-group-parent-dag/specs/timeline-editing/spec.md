## ADDED Requirements

### Requirement: Non-drawing group timeline nodes
Core MUST support GroupItem on overlay tracks with stable ID, integer-millisecond start/duration, common visual properties, and optional parent. Groups MUST default to identity Transform2D, visible, and zero zIndex; existing timing and ordering limits SHALL apply. Groups MUST accept static Transform2D, visibility, timing, move, reorder, and duplication edits. Group legacy transform updates, keyframes, split, audio, and transition-endpoint use MUST fail with INVALID_ARGUMENT. Group duplication MUST copy only the node and its parent, without changing children. Existing item behavior SHALL remain compatible.

#### Scenario: Create and edit a group
- **WHEN** a group is created on an unlocked overlay track and receives a valid static transform
- **THEN** it persists with canonical ordering and produces no independent drawable or audio instruction

#### Scenario: Reject unsupported group edits
- **WHEN** a group is placed on a non-overlay track or receives keyframes, legacy transform, split, audio, or transition-endpoint edits
- **THEN** core returns INVALID_ARGUMENT without changing state or history

#### Scenario: Duplicate a node
- **WHEN** a group with descendants is duplicated
- **THEN** only a new group node is created with the source properties and parent and existing descendants retain their original parent

### Requirement: Scoped bounded parent graph
Visual media, text, solids, rectangles, captions, and groups MUST support optional parent {scope,id}, where scope MUST be root in this milestone and id MUST name a group in the same project composition. Tracks SHALL NOT define composition scope. Audio-only and transition items MUST remain unparented. Core MUST reject missing parents with ITEM_NOT_FOUND and non-group targets, cross-scope references, self/indirect cycles, invalid values, and ancestor paths exceeding 32 edges with INVALID_ARGUMENT. A root item has depth zero. Graph validation MUST include hidden and inactive nodes, reject duplicate IDs before lookup normalization, apply existing project limits plus at most 4096 visual/group nodes in root, and validate before publication or render work.

#### Scenario: Resolve across tracks
- **WHEN** a visual item references a group on a different root-composition track
- **THEN** the valid reference resolves without changing either item's flat stacking or track

#### Scenario: Reject missing and invalid references
- **WHEN** parent assignment references an absent ID, a non-group, another scope, or creates a direct or indirect cycle including hidden nodes
- **THEN** core reports the specified typed failure and publishes nothing

#### Scenario: Enforce exact depth and count boundaries
- **WHEN** a graph has 32 versus 33 ancestor edges or 4096 versus 4097 visual/group nodes
- **THEN** the inclusive boundary is accepted and overflow fails with INVALID_ARGUMENT before traversal expansion or rendering

### Requirement: Transactional parenting lifecycle
Core MUST expose add_group with trackId/startMs/durationMs and optional complete transform2d and parent, and item_set_parent with itemId and required parent object or null. Null SHALL detach without changing local properties; reparenting SHALL preserve local rather than world transforms. Both operations MUST work standalone and in ordered timeline_batch_edit with aliases for created group/item IDs, existing revision and lock checks, one commit/undo step, and complete rollback. Parenting SHALL require the target item's track unlocked; reading a parent on a locked track SHALL remain legal. Deleting a referenced group MUST fail with INVALID_ARGUMENT. Track deletion MUST retain the existing empty-track-only rule and VALIDATION_FAILED for nonempty tracks. Callers MUST detach/reparent surviving children, delete items, then delete the empty track. Supported moves, splits, and duplicates of existing visual items MUST preserve their parent.

#### Scenario: Create and parent by alias
- **WHEN** a batch creates a group and visual item then assigns the group alias as parent to the item alias
- **THEN** the resolved graph commits once with existing deterministic changed-ID and alias response conventions and undo/redo restores the complete graph

#### Scenario: Preserve failures atomically
- **WHEN** a stale revision, locked target, missing item, invalid parent, or later failed batch operation occurs
- **THEN** existing REVISION_CONFLICT, TRACK_LOCKED, ITEM_NOT_FOUND, or specified invalid-input behavior leaves current state, history, files, and aliases unpublished

#### Scenario: Delete without dangling references
- **WHEN** a referenced group is deleted directly or a nonempty track is deleted
- **THEN** group deletion fails with INVALID_ARGUMENT until children are detached or reparented, and nonempty-track deletion retains VALIDATION_FAILED until its items are deleted

#### Scenario: Preserve local properties through lifecycle
- **WHEN** a child is reparented, detached, moved, duplicated, or split using supported existing operations and the project is reopened
- **THEN** local values and retained parent references remain deterministic and no automatic world-transform compensation occurs

### Requirement: Group transition endpoints fail closed
Core MUST reject either transition endpoint resolving to a group with INVALID_ARGUMENT in current state, retained history, drafts, mutations and direct renderer input, including hidden records. Existing missing-endpoint behavior SHALL remain unchanged. Invalid persisted endpoints MUST NOT be repaired or published.

#### Scenario: Reject persisted group endpoints atomically
- **WHEN** current state or an undo/redo snapshot contains a visible or hidden transition referencing a group from either endpoint
- **THEN** loading fails with INVALID_ARGUMENT and leaves authoritative files unchanged
