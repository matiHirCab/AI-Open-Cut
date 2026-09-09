## ADDED Requirements

### Requirement: Strict bounded repeater descriptors
Core MUST support item type `repeater` with required descriptor `repeater` containing `source`, `copies`, `transformOffset`, and `opacityOffset`. `source` MUST be a closed `{scope,id}` reference to a shape, group, or component instance in the repeater's containing root or component-definition scope. `copies` MUST be an integer in [1,256] and denotes additional copies, excluding the independently evaluated source. `transformOffset` MUST be a closed object containing position `{x,y,unit}`, `scaleX`, `scaleY`, `rotationDeg`, `skewXDeg`, and `skewYDeg`; units and numeric bounds MUST match the corresponding Transform2D fields, scales MUST be positive, and the implicit transform origin MUST be the source-parent coordinate origin. `opacityOffset` MUST be finite in [-1,1]. Every object MUST reject missing, unknown, duplicate, and positional fields in raw edit, batch, draft, component, project, and history input. Raw expressions, executable content, paths, and network resources MUST NOT be accepted.

#### Scenario: Accept boundary descriptors
- **WHEN** root and component-local repeaters name supported sources and supply copy counts 1 or 256, legal pixel or normalized position offsets, positive scale boundaries, bounded rotation/skew, and opacity offsets -1, 0, or 1
- **THEN** strict decoding and core validation preserve every supplied canonical field without expanding persisted copies

#### Scenario: Reject malformed or unsupported descriptors
- **WHEN** any repeater-bearing consumer supplies a missing, unknown, duplicate, null, positional, non-finite, out-of-range, expression-like, path-like, or network-bearing field, or names media, text, SVG, grid, caption, transition, audio-only, or repeater content as the source
- **THEN** decoding or core validation rejects the input with non-retryable `INVALID_ARGUMENT` before mutation, allocation, resource resolution, or renderer work

### Requirement: Scoped acyclic repeater references
Repeater sources MUST resolve in the containing composition scope, including hidden and inactive records. A missing source MUST return `ITEM_NOT_FOUND`; a cross-scope reference, unsupported source kind, or source expansion closure containing the repeater itself MUST return `INVALID_ARGUMENT`. Validation MUST reject duplicate item IDs before lookup normalization and MUST treat parent, component-instance, and repeater expansion edges as one bounded acyclic occurrence graph. Deleting or ungrouping a referenced source MUST fail with `INVALID_ARGUMENT` until every repeater is removed or retargeted. Validation MUST occur for current state, retained history, drafts, component definitions, and mutations before publication or render work.

#### Scenario: Resolve supported sources across tracks
- **WHEN** a repeater references a shape, group, or component instance on another track in the same root or component scope
- **THEN** the reference resolves without moving or mutating the source or changing the source's independent evaluation

#### Scenario: Reject missing cross-scope and cyclic references
- **WHEN** a repeater names an absent item, an item in another scope, its containing group, or a group/component expansion closure that reaches the repeater again
- **THEN** core returns `ITEM_NOT_FOUND` for the absent ID or `INVALID_ARGUMENT` for the invalid graph and publishes no state, history, aliases, or artifacts

#### Scenario: Preserve referenced sources
- **WHEN** deletion or ungroup would remove a shape, group, or component instance referenced by a repeater
- **THEN** the mutation fails atomically with `INVALID_ARGUMENT` and succeeds only after the reference is removed or retargeted

### Requirement: Deterministic additional-copy semantics
The ordinary source MUST continue to evaluate exactly once at its existing stack position. An active repeater MUST emit exactly `copies` additional source occurrences at the repeater's canonical stack position in ascending one-based copy-index order. A shape copy MUST clone that shape occurrence; a group copy MUST clone the group's complete transitive descendant occurrence set; and a component-instance copy MUST clone its complete resolved definition occurrence with identical slot bindings and local clock. Every copy MUST preserve source timing and intersect it with the repeater's half-open interval and inherited source visibility. Repeater time offsets MUST NOT be applied. For copy index `i`, core MUST normalize the position unit against the source scope canvas, build offset matrix `O`, and apply `O^i` in source-parent coordinates before the source local transform. Copy opacity MUST equal the ordinary source inherited opacity multiplied by `clamp(1 + i * opacityOffset, 0, 1)`. The persisted source and repeater MUST remain unchanged.

#### Scenario: Repeat each supported source
- **WHEN** active repeaters target a shape, a transformed group with nested descendants, and a retimed component instance with slot values
- **THEN** evaluation emits the requested additional occurrence blocks with preserved internal order, source clocks, hierarchy, bindings, and one-based transform/opacity offsets while each source still evaluates normally

#### Scenario: Apply exact interval and opacity behavior
- **WHEN** source and repeater intervals partially overlap and successive positive or negative opacity offsets reach a boundary
- **THEN** copies exist only in the interval intersection and each copy opacity uses the specified clamped formula without changing source opacity or applying a time shift

#### Scenario: Preserve deterministic ordering and identity
- **WHEN** multiple repeaters and ordinary layers share z-index values or a repeated group spans tracks
- **THEN** each generated block occupies its repeater's position under the canonical track/z-index/stack-order comparator, preserves source-subtree order, orders copies by ascending index, and assigns deterministic occurrence identities derived from scope, repeater ID, copy index, and source occurrence path

### Requirement: Lazy bounded occurrence expansion
Persisted projects, history, drafts, and public mutation results MUST retain one repeater record and MUST NOT materialize generated items. Evaluation MUST preflight copy multiplication, matrices, opacity arithmetic, intervals, generated visual/audio facts, and source expansion closures with checked finite arithmetic before collection or surface allocation. Each repeater MUST enforce the inclusive 256-copy limit; all generated and ordinary occurrences together MUST retain the existing inclusive 65,536 expanded-occurrence limit and existing 4,096 visual-layer, 4,096 media-resource, 4,096 audio-layer, vector, surface, and aggregate-memory limits. Boundary work MUST succeed subject to other limits; overflow, non-finite derived values, or one-over-limit work MUST return non-retryable `INVALID_ARGUMENT` without approximation or side effects, including for hidden or inactive definitions.

#### Scenario: Enforce per-repeater and aggregate boundaries
- **WHEN** a valid source is repeated 256 versus 257 times, or nested group/component/repeater multiplication produces exactly 65,536 versus 65,537 occurrences
- **THEN** each inclusive boundary passes preflight subject to downstream limits and each overflow fails before expansion allocation or renderer execution

#### Scenario: Fail closed on derived transform work
- **WHEN** repeated scale, skew, rotation, normalized position, opacity, hierarchy, or nested component multiplication becomes non-finite or exceeds an existing scene/work/memory limit
- **THEN** evaluation returns `INVALID_ARGUMENT`, produces no partial `EvaluatedScene`, performs no renderer/filesystem I/O, and leaves current state and retained history unchanged

#### Scenario: Reopen without persisted expansion
- **WHEN** a repeater edit is saved, undone, redone, reopened, or materialized as a draft
- **THEN** each authoritative snapshot contains one canonical repeater record and repeated evaluation produces equal generated identities, order, transforms, opacities, and intervals
