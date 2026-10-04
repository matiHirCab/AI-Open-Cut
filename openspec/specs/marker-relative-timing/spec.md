# Marker Relative Timing Specification

## Purpose

Define scoped cue markers and marker-relative item start timing.

## Requirements

### Requirement: Scoped marker lifecycle
Core SHALL persist cue markers with stable IDs, names and nonnegative JavaScript-safe integer millisecond times in either the root composition or one component definition. Marker IDs and names MUST use the catalog's ASCII identifier grammar (letter first, then up to 127 letters, digits, underscores or hyphens). IDs MUST be unique within the owning composition, names MAY repeat, and each composition MUST contain at most 4096 markers. Every marker operation MUST identify its owning composition as `root` or `component:<existing component ID>`; existing component UUID IDs MUST remain usable as scope suffixes. Marker create, update and delete MUST be atomic revision-checked edits; a referenced marker name MUST not become missing or ambiguous after any edit. Marker records MUST not produce visual or audio instructions.

#### Scenario: Create and edit a root or component marker
- **WHEN** a caller creates a valid marker in the root or an existing component, then updates its name or time at the current revision
- **THEN** the marker is stored in that exact composition, receives a stable ID on creation, and each edit produces one revision and undo step

#### Scenario: Reject invalid or dangling marker edits
- **WHEN** a marker ID, name, kind, scope, time, count, or referenced deletion/rename is invalid
- **THEN** core returns a non-retryable typed error and leaves project, history and revision unchanged

### Requirement: Marker-relative item start timing
Core SHALL support a stored item-start expression `{type:"marker",markerName,offsetMs}` alongside existing numeric `startMs`. The item-start edit MUST identify the containing composition as `root` or `component:<existing component ID>` and an item ID in that composition. A marker expression MUST resolve by exact name only in the item's containing composition; root and component-local markers MUST not fall back to one another. The signed offset and resulting nonnegative start/end MUST be JavaScript-safe integers, and the resulting item interval MUST satisfy all existing item and component bounds. Core MUST retain the expression and keep the public numeric `startMs` synchronized with its resolved value whenever a marker or referenced item changes. Existing numeric start edits MUST clear the expression. Marker expressions MUST not form recursive references.

#### Scenario: Resolve a signed offset in local scope
- **WHEN** a root or component-local item's start references one same-scope named marker with a positive or negative offset
- **THEN** its effective start and interval use that marker's time plus the offset, and moving the marker updates the effective timing deterministically

#### Scenario: Reject missing, ambiguous or out-of-range timing
- **WHEN** the local marker name is absent, matches multiple local markers, or its signed offset produces an invalid start or interval
- **THEN** the edit fails with the stable missing-reference or invalid-argument code before commit, without using a marker from another scope

#### Scenario: Preserve numeric clients
- **WHEN** an existing client creates or edits an item with numeric `startMs` and no expression
- **THEN** its request, stored timing, response and rendering retain their existing behavior

### Requirement: Transactional marker operations
Core SHALL expose marker create, update and delete and item-start expression set/clear as standalone edits and inside `timeline_batch_edit`. Batch operations MUST evaluate in order against one candidate, support existing aliases for newly created marker and item IDs where IDs are accepted, and commit as one revision and undo step or roll back completely. Stale expected revisions MUST return retryable `REVISION_CONFLICT`; unknown or forward aliases MUST retain existing alias failure semantics. Undo, redo and deterministic reopen MUST preserve marker identity, expressions and effective timing.

#### Scenario: Use aliases in an atomic batch
- **WHEN** a batch creates a marker and an item, then addresses their generated IDs through earlier result aliases in subsequent marker or item-start operations
- **THEN** all edits commit together with resolved aliases and one history entry

#### Scenario: Roll back an invalid batch
- **WHEN** a later batch operation makes a marker lookup ambiguous or produces an invalid item interval
- **THEN** no marker, item, alias result, revision or history edit from that batch is published

#### Scenario: Conflict, undo, redo and reopen
- **WHEN** a committed marker-relative edit is submitted with a stale revision, then valid revisions are used for undo, redo and reopen
- **THEN** the stale edit fails unchanged and the valid transitions retain the exact expression and evaluated timing

### Requirement: Strict nested time-expression contract parity
Native and TypeScript consumers MUST accept the same closed milliseconds and marker time-expression variants. Unknown fields inside either variant MUST be rejected before any mutation or migration publication, including standalone and ordered batch edits. Native/headless request parsing MUST retain existing non-retryable `INVALID_ARGUMENT`; the public TypeScript schema MUST retain strict Zod rejection; MCP MUST retain its existing SDK input-validation `CallToolResult` with `isError:true` and validation text, without a structured core error/code/retryability object and without dispatching a headless edit request. This correction MUST NOT normalize or change those distinct existing transport error representations. Supported fields, marker scope/ambiguity resolution, numeric compatibility, alias behavior, optimistic revisions and error precedence for otherwise valid requests MUST remain unchanged. Persisted marker expressions MUST use the same closed variant decoder and existing malformed-document/recovery error mapping rather than silently discard unknown fields. Canonical version1 fixtures MUST govern positive and negative variants across Rust, TypeScript and public transports; this correction MUST NOT add a new expression variant or permit raw executable expressions.

#### Scenario: Reject unknown members in both expression variants
- **WHEN** standalone or batch input supplies an extra member in a milliseconds or marker time-expression object, including an earlier otherwise-valid aliased batch operation
- **THEN** native/headless rejects with existing non-retryable INVALID_ARGUMENT, the public TypeScript schema rejects strictly, and MCP preserves its existing SDK input-validation isError/text result with no structured core error or headless edit dispatch; no surface publishes a project/history/revision, committed alias or managed-resource change

#### Scenario: Preserve supported timing and failure precedence
- **WHEN** a valid numeric or scoped marker expression is submitted at the current revision, or an otherwise-valid request has a stale revision, missing marker/item, locked track or invalid alias
- **THEN** valid timing retains its existing behavior and invalid operations retain their established typed error, retryability and complete rollback

#### Scenario: Reject malformed persisted expression without rewriting
- **WHEN** current, component-local or retained undo/redo source documents or a committed recovery journal contain a marker expression with an unknown member
- **THEN** existing document/recovery failure mapping rejects the closed shape before authoritative publication/replay and preserves the complete previous durable generation
