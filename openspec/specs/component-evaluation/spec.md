# Component Evaluation Specification

## Purpose

Define typed root component placement and bounded deterministic nested evaluation with local clocks, effective slots and immutable shared definitions.

## Requirements

### Requirement: Typed atomic root instance editing
Core MUST support add_component_instance on root overlay tracks with trackId, componentId, startMs, trimStartMs, durationMs, timeScale and optional slotValues plus existing common visual fields. Create MUST generate an ID and accept resultAlias. component_instance_update MUST accept itemId and the complete componentId/startMs/trimStartMs/durationMs/timeScale tuple; omitted slotValues MUST preserve overrides and an explicit map MUST replace them. Generic move, remove, duplicate, static transform, visibility, ordering and parent edits MUST support root instances using existing semantics; unsupported keyframes, splitting and instance transition endpoints MUST return INVALID_ARGUMENT. Root parents MUST use root scope. Definition references and effective slot values MUST validate against every incoming root and nested instance.

#### Scenario: Create and update through an aliased batch
- **WHEN** a batch creates a definition and places an instance using its alias, then updates that instance using its creation alias
- **THEN** earlier creation references resolve, the batch commits once, and undo/redo/reopen preserves exact definitions, overrides, timing and root items

#### Scenario: Reject invalid edits atomically
- **WHEN** a reference is missing, a value or effective slot is invalid, a track is locked, a revision is stale, or a later batch operation fails
- **THEN** core returns existing ITEM_NOT_FOUND/ASSET_NOT_FOUND, INVALID_ARGUMENT, TRACK_LOCKED or retryable REVISION_CONFLICT as applicable and preserves revision and byte-identical project/history files

#### Scenario: Preserve generic editing and draft behavior
- **WHEN** instances are moved, duplicated, removed, ordered, parented or edited in a draft
- **THEN** existing root edit semantics apply, duplication retains independent overrides, references remain scoped and materialized draft preview matches equivalent committed state

### Requirement: Composed half-open local clocks
For an instance active on [startMs,startMs+durationMs), core MUST map parent time t to trimStartMs+(t-startMs)*timeScale. Nested mappings MUST compose, preserve finite fractional derived times without intermediate integer rounding and intersect all enclosing active intervals. Local media source trims, keyframes, captions, fades and internal transitions MUST use this clock. Persisted times MUST remain nonnegative safe integers with positive durations and finite positive timeScale; mapped source ends MUST fit referenced duration without looping or clamping.

#### Scenario: Evaluate nested fractional clocks
- **WHEN** nested instances have nonzero starts/trims and rates 0.5, 1.5 and 2
- **THEN** timestamps at, before and after mapped boundaries match an independent affine clock oracle, with exclusive ends and no cumulative millisecond rounding

#### Scenario: Reject invalid mapped values
- **WHEN** timing is negative, unsafe, non-finite, zero-duration, overflowing or exceeds the source duration
- **THEN** core rejects it with INVALID_ARGUMENT before mutation or render side effects, while exact valid source endpoints succeed

### Requirement: Instance-local slot resolution and identity
Each occurrence MUST apply overrides before defaults using the existing typed slot rules, retaining absent optional base values and rich-text runs/styles. Shared definitions MUST remain immutable. Repeated occurrences MUST have distinct process-local identities derived from complete instance paths; local parent, binding and transition references MUST resolve only within the occurrence.

#### Scenario: Render independent overrides
- **WHEN** two instances share one definition with different text, rich text, color, duration, visibility and managed-asset overrides
- **THEN** each renders its own effective values, local IDs cannot collide, and repeated evaluation leaves definitions, revision and history unchanged

### Requirement: Bounded expansion before side effects
Core MUST validate hidden and unused definitions and retain existing graph, slot and scene limits. Expansion MUST permit at most 65536 visited item occurrences, including hidden items, groups and instances, and enforce existing 4096 visual/audio/resource/transition limits and 10000 per-channel keyframe/voiceover-range limits on expanded facts. Derived clocks and matrices MUST be finite, existing transformed geometry bounds MUST apply, and overflow MUST return INVALID_ARGUMENT before unbounded traversal, allocation, writes or backend execution.

#### Scenario: Bound repeated DAG expansion
- **WHEN** a small shared graph expands to an inclusive work/scene limit versus one above it
- **THEN** valid boundary input succeeds and overflow fails during bounded preflight without artifacts or project changes

#### Scenario: Validate unreachable invalid content
- **WHEN** unused or hidden content contains a missing definition/asset, cycle or invalid slot
- **THEN** the existing typed validation failure is returned before render preparation even though the content emits no visible layer

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
