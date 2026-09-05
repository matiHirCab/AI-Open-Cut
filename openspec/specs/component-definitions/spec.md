# Component Definitions Specification

## Purpose

Define reusable persisted component timelines, scoped nested references, duration bounds and atomic definition management while preserving root rendering.

## Requirements

### Requirement: Stored local component timelines
Projects MUST support reusable component definitions with stable project-unique ID, name, dimensions, explicit positive duration and local tracks. Existing ordinary item and track semantics MUST apply inside the composition, with IDs unique within that scope and parent/transition references confined to it. Component-local parents MUST use component:<id> scope. Typed nested component_instance items MUST be restricted to component overlay tracks and contain common static visual/order properties and componentId, startMs, trimStartMs, durationMs and finite positive timeScale. Root placement, instance transition endpoints and animated instance properties MUST fail with INVALID_ARGUMENT until separately activated.

#### Scenario: Persist an empty or populated definition
- **WHEN** a valid positive-duration definition is created with empty tracks or compatible local items
- **THEN** reads preserve its exact values independently of root tracks and root duration

#### Scenario: Confine local references
- **WHEN** local IDs match IDs in another definition or root, or a local parent or transition points outside its composition
- **THEN** valid independent IDs remain distinct and cross-scope references fail without mutation

### Requirement: Bounded nested component graph and duration
Core MUST validate every definition including hidden and unused content. It MUST reject duplicate definition or local IDs before indexing, missing component references with ITEM_NOT_FOUND, and cycles, malformed values or bounds with INVALID_ARGUMENT. A leaf MUST have depth zero and the longest dependency path MUST be at most 16 edges. At most 512 definitions, 4096 aggregate component tracks and 4096 aggregate component items SHALL be permitted, with existing per-composition graph/media/keyframe limits retained. Repeated valid instances MUST be allowed. Definition and item times MUST be positive where durations are required, nonnegative elsewhere and JavaScript-safe integers; checked local ends MUST fit definition duration, and finite trimStartMs + durationMs * timeScale MUST fit the referenced duration. Dimensions MUST obey existing project limits.

#### Scenario: Validate branching and shared dependencies
- **WHEN** definitions form a diamond, repeated instances, a hidden cycle, or a cycle in an unused definition
- **THEN** acyclic sharing succeeds and every cycle fails without recursive expansion

#### Scenario: Enforce inclusive complexity bounds
- **WHEN** depth is 16 versus 17 or a declared count is at its limit versus one above it
- **THEN** the inclusive limit succeeds and overflow fails before unbounded allocation or traversal

#### Scenario: Validate all timing boundaries
- **WHEN** local or mapped source ends equal their duration boundary, exceed it, overflow, or contain fractional/non-finite/unsafe time values
- **THEN** valid exact endpoints succeed and invalid values fail without clamping or publication

### Requirement: Atomic definition management
Core MUST provide component_create with name,width,height,durationMs,tracks; component_update with the same fields and componentId; and component_delete with componentId. Create MUST assign an ID and accept a creation resultAlias; update MUST replace the complete definition retaining its ID. Only create SHALL produce an alias. Standalone and 1-to-100 ordered batches MUST share current revision checks, one commit/undo step, deterministic reopen, and full rollback. Earlier creation aliases MUST resolve in component targets and nested component references. Each operation MUST validate the evolving graph. Missing targets MUST return ITEM_NOT_FOUND, referenced deletion INVALID_ARGUMENT, locked local content TRACK_LOCKED and stale revisions retryable REVISION_CONFLICT. Updates MUST preserve locked tracks exactly and their relative order; deletion MUST reject any locked local track. Changed IDs MUST identify affected definitions deterministically without ambiguous local child IDs.

#### Scenario: Create and reference in one batch
- **WHEN** a batch creates a definition with an alias then creates or updates another definition referencing it
- **THEN** references resolve, the graph commits once and undo/redo/reopen restores exact states and alias reporting

#### Scenario: Preserve failed mutations
- **WHEN** a target is absent, an alias is unresolved/forward, a referenced definition is deleted, a locked track is altered, a revision is stale, or a later operation fails
- **THEN** the specified error preserves revision and byte-identical project/history files without partial publication

#### Scenario: Update duration of a referenced definition
- **WHEN** an update shortens a definition below an existing instance source end
- **THEN** core rejects the update atomically after validating incoming references

### Requirement: Preserve root rendering
Unused definitions MUST NOT affect root duration, pixels, audio or ordering. Preview, range preview, draft preview and export MUST retain the existing shared evaluation behavior. Direct renderer inputs MUST reject malformed component graphs before output preparation, even when those definitions are unused.

#### Scenario: Render a project with stored definitions
- **WHEN** valid component definitions are added without changing root tracks
- **THEN** existing frame, range and export output remains equivalent to the original project

#### Scenario: Reject invalid direct render input
- **WHEN** a direct render request contains an invalid unused definition graph
- **THEN** validation fails before process execution or artifact publication

### Requirement: Complete component item validation
Core MUST bound every nested millisecond field by JavaScript's safe-integer maximum, including keyframe, media fade, caption generation and word timestamps. Generation timestamps MUST be positive. Only media items MUST accept volume keyframes. Existing keyframe ordering/value rules MUST remain in force without adding keyframe-within-item or fade-within-duration restrictions.

#### Scenario: Enforce nested timing and property boundaries
- **WHEN** a component item has a safe-integer boundary or overflow timestamp, or media versus non-media volume animation
- **THEN** valid boundaries and media animation succeed while unsafe times and non-media volume animation fail with non-retryable INVALID_ARGUMENT without publication

### Requirement: Valid component caption provenance
Component captions MUST contain nonblank provider/model IDs of at most 128 UTF-8 bytes, nonblank language, optional nonempty model version, nonblank original text of at most 4096 UTF-8 bytes, nonblank word text, finite optional source/word confidence in [0,1], positive safe generation time and safe positive word intervals. Bottom margin MUST be at most 4320. Source words MUST remain independent of current caption placement; empty word arrays and absent optional confidence MUST remain valid.

#### Scenario: Preserve moved caption provenance
- **WHEN** a valid caption is moved or trimmed so its source word timestamps fall outside its current interval
- **THEN** component replacement preserves original word timestamps and remains readable through core and MCP

#### Scenario: Reject malformed nested content at every boundary
- **WHEN** invalid caption metadata or nested timing enters create/update, batch, draft, current/history load or direct rendering
- **THEN** core rejects it before publication or rendering; a failing batch preserves revision and byte-identical project/history, and malformed persisted records are not repaired

#### Scenario: Verify canonical cross-language acceptance
- **WHEN** canonical valid, structurally invalid and semantically invalid component fixtures pass through native and MCP consumers
- **THEN** each consumer matches its declared acceptance stage, valid populated definitions round-trip, and source integration and packaged smoke exercise atomic rejection

### Requirement: Compatible slot-aware component editing
Component creation MUST accept optional `slots`, defaulting to empty. Component update MUST accept optional `slots`; omission MUST preserve existing slots, while an explicit array MUST replace them. Existing required definition fields and full track replacement MUST retain their meaning. Nested component_instance requests MUST accept optional `slotValues`, defaulting to empty, with slot IDs as keys and typed values as entries. Every create/update MUST validate all resulting local bindings and incoming instance overrides against the evolving graph. Current locks, graph limits and timing checks MUST remain in force, with effective slot duration/asset values also validated. Draft materialization and direct renderer validation MUST reuse these core rules.

#### Scenario: Preserve older requests
- **WHEN** a pre-slot client creates or updates a definition without the new optional fields
- **THEN** creation yields empty slots, update retains existing slots, and unchanged tracks preserve all prior semantics

#### Scenario: Replace definitions with incoming slot references
- **WHEN** a definition update removes a bound item, removes a used slot, changes its type, or invalidates effective instance timing
- **THEN** the whole operation fails atomically without pruning references or clamping values

### Requirement: Stored slots preserve current rendered output
Slots and stored nested-instance values MUST NOT change root duration, ordering, pixels, audio or fallback selection. Root instance placement and component-instance rendering MUST remain deferred. Frame, range, draft preview and export MUST retain the shared evaluated behavior. Invalid slot content, including hidden/unused definitions, MUST fail before render process execution or output preparation.

#### Scenario: Compare root output with stored slots
- **WHEN** valid definitions receive slots and nested-instance values without root edits
- **THEN** all root render entry points retain equivalent evaluated output

#### Scenario: Reject malformed direct render data
- **WHEN** a direct render input contains invalid bindings or values in an unused definition
- **THEN** core rejects it before preparing or publishing artifacts
