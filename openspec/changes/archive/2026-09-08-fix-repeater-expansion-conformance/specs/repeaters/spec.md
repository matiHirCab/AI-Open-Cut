## MODIFIED Requirements

### Requirement: Scoped acyclic repeater references
Each repeater source MUST resolve within the repeater's containing composition scope, MUST name a shape, group, or component instance, and MUST NOT create or traverse a repeater, group, or component-definition cycle. Canonical project validation MUST establish complete parent/reference invariants and group/component acyclicity, including hidden and unused content, before SVG, component-occurrence, repeater, audio-closure, or resource preflight performs recursive traversal. Every recursive traversal MUST also track its active group/component nodes and return non-retryable `INVALID_ARGUMENT` on re-entry as defense in depth. Missing sources MUST preserve the existing `ITEM_NOT_FOUND` behavior and precedence. Invalid graphs MUST fail before occurrence expansion, rendering, allocation proportional to copies, filesystem I/O, or state publication.

#### Scenario: Reject cyclic graphs before recursive preflight
- **WHEN** a project supplied directly to evaluation or rendering contains a hidden or unused component-definition cycle, or a repeater source reaches a cyclic group hierarchy
- **THEN** canonical validation returns `INVALID_ARGUMENT` deterministically without stack overflow, process abort, recursive preflight work, generated allocation, renderer execution, filesystem I/O, or persisted-state change

#### Scenario: Preserve missing-reference and valid-depth behavior
- **WHEN** a repeater names a missing in-scope source or a valid acyclic component/group graph reaches the existing maximum supported depth
- **THEN** the missing source returns `ITEM_NOT_FOUND` with existing precedence and the valid boundary graph continues to evaluate subject to unchanged occurrence and scene budgets

### Requirement: Deterministic additional-copy semantics
The ordinary source MUST continue to evaluate exactly once at its existing stack position. An active repeater MUST emit exactly `copies` additional source occurrences at the repeater's canonical stack position in ascending one-based copy-index order. A shape copy MUST clone that shape occurrence; a group copy MUST clone the group's complete transitive descendant occurrence set, including recursively resolved component-instance occurrences; and a component-instance copy MUST clone its complete resolved definition occurrence with identical slot bindings and local clock, including valid nested repeater occurrences. Every copy MUST preserve source timing and intersect it with the repeater's half-open interval and inherited source visibility. Repeater time offsets MUST NOT be applied. For copy index `i`, core MUST normalize the position unit against the source scope canvas, build offset matrix `O`, and apply `O^i` in source-parent coordinates before the source local transform. Copy opacity MUST equal the ordinary source inherited opacity multiplied by `clamp(1 + i * opacityOffset, 0, 1)`. The persisted source and repeater MUST remain unchanged.

#### Scenario: Repeat each supported source
- **WHEN** active repeaters target a shape, a transformed group with nested shape and component-instance descendants, and a retimed component instance with slot values or valid nested repeater occurrences
- **THEN** evaluation emits the requested complete additional occurrence blocks with preserved internal order, source clocks, hierarchy, bindings, and one-based transform/opacity offsets while each source still evaluates normally

#### Scenario: Include component descendants in group copies
- **WHEN** a root or component-local repeater targets a group whose direct or nested descendants include a component instance with recursively resolved visual occurrences
- **THEN** every group copy includes each resolved component occurrence exactly once alongside all flat descendants, transformed as one source subtree around the group's parent origin

#### Scenario: Preserve rich-text bindings through nested repeaters
- **WHEN** a component instance supplies a rich-text document to a text layer contained by a component-local repeated group and an outer repeater copies that component occurrence
- **THEN** the ordinary text, every local group copy, and every outer component copy contain the identical resolved text, rich runs, styles, identity scope, order, and intersected interval without modifying another nested scope that reuses the target ID

#### Scenario: Apply exact interval and opacity behavior
- **WHEN** source and repeater intervals partially overlap and successive positive or negative opacity offsets reach a boundary
- **THEN** copies exist only in the interval intersection and each copy opacity uses the specified clamped formula without changing source opacity or applying a time shift

#### Scenario: Preserve deterministic ordering and identity
- **WHEN** multiple repeaters and ordinary layers share z-index values, a repeated group spans tracks and mixes flat/component descendants, or a repeated component contains at least eleven equal-z siblings
- **THEN** each generated block occupies its repeater's position under the canonical track/z-index/stack-order comparator, preserves numeric source-subtree order for every sibling count, orders copies by ascending index, and assigns deterministic occurrence identities derived from scope, repeater ID, copy index, and source occurrence path

### Requirement: Lazy bounded occurrence expansion
Persisted projects, history, drafts, and public mutation results MUST retain one repeater record and MUST NOT materialize generated items. Evaluation MUST use a lightweight bounded ordinary-source index and MUST project all same-scope repeaters before cloning or publishing any generated layer. That complete projection MUST cover copy multiplication, effective intervals, opacity arithmetic, final composed matrices, generated visual/audio facts, complete transitive source closures, vector segments, raster bounds and surfaces, and aggregate memory with checked finite arithmetic. Matrix validation MUST cover `parent * O^i * parent_inverse`, not only `O^i`. Group-source closure counting MUST include every nested group controller and descendant exactly once per source occurrence without double-counting the ordinary flat scope. Each repeater MUST enforce the inclusive 256-copy limit; all generated and ordinary occurrences together MUST retain the existing inclusive 65,536 expanded-occurrence limit and existing 4,096 visual-layer, 4,096 media-resource, 4,096 audio-layer, vector, surface, and aggregate-memory limits. Boundary work MUST succeed subject to other limits; overflow, non-finite derived values, or one-over-limit work MUST return non-retryable `INVALID_ARGUMENT` before generated-layer materialization, renderer execution, or surface allocation and without approximation or side effects, including for hidden or inactive definitions.

#### Scenario: Enforce per-repeater and aggregate boundaries
- **WHEN** a valid source is repeated 256 versus 257 times, or nested group/component/repeater multiplication produces exactly 65,536 versus 65,537 occurrences
- **THEN** each inclusive boundary passes preflight subject to downstream limits and each overflow fails before expansion allocation or renderer execution

#### Scenario: Count nested group source closures transitively
- **WHEN** a hidden or inactive repeater targets an outer group containing an inner group and enough nested group descendants to produce exactly 65,536 total ordinary/generated occurrences, followed by one additional ordinary occurrence
- **THEN** the exact-boundary project passes, the one-over project returns `INVALID_ARGUMENT` during preflight, and no evaluated collection, renderer work, or persisted state is produced or changed

#### Scenario: Fail closed on derived transform work
- **WHEN** repeated scale, skew, rotation, normalized position, opacity, hierarchy, or nested component multiplication becomes non-finite or exceeds an existing scene/work/memory limit
- **THEN** evaluation returns `INVALID_ARGUMENT`, produces no partial `EvaluatedScene`, performs no renderer/filesystem I/O, and leaves current state and retained history unchanged

#### Scenario: Preflight complete expanded visual and geometry budgets
- **WHEN** expanded work produces exactly 4,096 visual layers or exactly an existing vector/raster/surface/aggregate-memory boundary, versus 4,097 visual layers or one unit above any such boundary
- **THEN** every exact boundary succeeds subject to the other unchanged limits and every one-over case returns `INVALID_ARGUMENT` before any generated layer is cloned or published, with an empty result on failure

#### Scenario: Preflight final composed matrices
- **WHEN** a repeated offset power `O^i` is finite but composition with the persisted source parent as `parent * O^i * parent_inverse` is non-finite or produces non-finite measured bounds
- **THEN** projection returns `INVALID_ARGUMENT` before generated-layer materialization or renderer work

#### Scenario: Reopen without persisted expansion
- **WHEN** a repeater edit is saved, undone, redone, reopened, or materialized as a draft
- **THEN** each authoritative snapshot contains one canonical repeater record and repeated evaluation produces equal generated identities, order, transforms, opacities, and intervals
