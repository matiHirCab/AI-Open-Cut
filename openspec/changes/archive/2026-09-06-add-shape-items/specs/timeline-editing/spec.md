## ADDED Requirements

### Requirement: Transactional shape editing
Core MUST expose add_shape with trackId, startMs, durationMs, geometry, required nullable fill/stroke, and optional complete transform2d and parent. Shapes MUST occupy unlocked overlay tracks, default to identity Transform2D, visible state and zero zIndex, and retain existing safe integer timing and stack-order rules. add_shape MUST create one ID and support resultAlias in ordered atomic batches; earlier aliases MUST resolve in trackId and parent.id. Existing update_item MUST accept complete geometry replacement and explicit nullable fill/stroke updates for shapes only, preserving omitted properties and rejecting an invalid resulting shape.

Shapes MUST support existing timing, transform, visibility, keyframe channels for visual position/scale/opacity under their existing Transform2D exclusivity rules, move, trim, split, duplicate, delete, z-index, reorder and parenting workflows. Audio edits and transition-endpoint use MUST fail with INVALID_ARGUMENT. Legacy transform updates MUST activate the existing legacy mode. Component definition workflows MUST accept local shape items with identical geometry validation and existing scoped parenting. No new shape-specific animation or slot-binding property MUST be activated.

#### Scenario: Create and edit with aliases
- **WHEN** a valid batch creates a track/group and shape then updates, parents and reorders the shape through earlier aliases
- **THEN** the complete change commits as one revision and undo step with deterministic changed IDs and alias mappings

#### Scenario: Preserve failure atomicity
- **WHEN** shape editing encounters invalid geometry, incompatible placement, missing track/item/parent, a locked target, stale revision, unresolved alias or later failed batch operation
- **THEN** existing INVALID_ARGUMENT, TRACK_NOT_FOUND, ITEM_NOT_FOUND, TRACK_LOCKED, retryable REVISION_CONFLICT or existing alias errors apply and no partial state/history/files are published

#### Scenario: Preserve lifecycle and legacy operations
- **WHEN** shapes are updated, moved, trimmed, split, duplicated, hidden, reordered, parented, undone, redone and reopened
- **THEN** geometry, paints, local properties and references persist deterministically and existing rectangle and other legacy operations retain their established behavior

#### Scenario: Edit shapes inside a definition
- **WHEN** a component is created or replaced with valid local shapes or an invalid hidden shape
- **THEN** valid shapes participate in existing instance evaluation and invalid content rejects the entire edit before publication
