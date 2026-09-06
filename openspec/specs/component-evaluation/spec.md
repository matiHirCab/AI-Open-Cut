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
