# Repeaters Specification

## Purpose

Define strict persisted repeater descriptors, scoped source graphs, deterministic additional-copy evaluation, and bounded lazy expansion.

## Requirements

### Requirement: Strict bounded repeater descriptors
Core MUST support item type `repeater` with required descriptor `repeater` containing `source`, `copies`, `transformOffset`, and `opacityOffset`. `source` MUST be a closed `{scope,id}` reference to a shape, group, or component instance in the repeater's containing root or component-definition scope. `copies` MUST be an integer in [1,256] and denotes additional copies, excluding the independently evaluated source. `transformOffset` MUST be a closed object containing position `{x,y,unit}`, `scaleX`, `scaleY`, `rotationDeg`, `skewXDeg`, and `skewYDeg`; units and numeric bounds MUST match the corresponding Transform2D fields, scales MUST be positive, and the implicit transform origin MUST be the source-parent coordinate origin. `opacityOffset` MUST be finite in [-1,1]. Every object MUST reject missing, unknown, duplicate, and positional fields in raw edit, batch, draft, component, project, and history input. Raw expressions, executable content, paths, and network resources MUST NOT be accepted.

#### Scenario: Accept boundary descriptors
- **WHEN** root and component-local repeaters name supported sources and supply copy counts 1 or 256, legal pixel or normalized position offsets, positive scale boundaries, bounded rotation/skew, and opacity offsets -1, 0, or 1
- **THEN** strict decoding and core validation preserve every supplied canonical field without expanding persisted copies

#### Scenario: Reject malformed or unsupported descriptors
- **WHEN** any repeater-bearing consumer supplies a missing, unknown, duplicate, null, positional, non-finite, out-of-range, expression-like, path-like, or network-bearing field, or names media, text, SVG, grid, caption, transition, audio-only, or repeater content as the source
- **THEN** decoding or core validation rejects the input with non-retryable `INVALID_ARGUMENT` before mutation, allocation, resource resolution, or renderer work

### Requirement: Scoped acyclic repeater references
Each repeater source MUST resolve within the repeater's containing composition scope, MUST name a shape, group, or component instance, and MUST NOT create or traverse a repeater, group, or component-definition cycle. Canonical project validation MUST establish complete parent/reference invariants and group/component acyclicity, including hidden and unused content, before SVG, component-occurrence, repeater, audio-closure, or resource preflight performs recursive traversal. Every recursive traversal MUST also track its active group/component nodes and return non-retryable `INVALID_ARGUMENT` on re-entry as defense in depth. Missing sources MUST preserve the existing `ITEM_NOT_FOUND` behavior and precedence. Invalid graphs MUST fail before occurrence expansion, rendering, allocation proportional to copies, filesystem I/O, or state publication.

#### Scenario: Reject cyclic graphs before recursive preflight
- **WHEN** a project supplied directly to evaluation or rendering contains a hidden or unused component-definition cycle, or a repeater source reaches a cyclic group hierarchy
- **THEN** canonical validation returns `INVALID_ARGUMENT` deterministically without stack overflow, process abort, recursive preflight work, generated allocation, renderer execution, filesystem I/O, or persisted-state change

#### Scenario: Preserve missing-reference and valid-depth behavior
- **WHEN** a repeater names a missing in-scope source or a valid acyclic component/group graph reaches the existing maximum supported depth
- **THEN** the missing source returns `ITEM_NOT_FOUND` with existing precedence and the valid boundary graph continues to evaluate subject to unchanged occurrence and scene budgets

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

### Requirement: Retained repeater validation domains
Core MUST validate complete repeater expansion independently of visibility before generated-layer materialization. The root expansion and every component definition MUST each have an independent budget domain; independent definitions MUST NOT be added to the root total. Actual instances MUST use effective slots and composed clocks/transforms; standalone definitions MUST use their own canvas and defaults. Hidden items/tracks, hidden or interval-clipped instances, unused definitions, and local repeaters copied externally MUST undergo the existing inclusive occurrence, visual/audio/resource, segment, surface and memory checks. Empty output intervals MUST NOT bypass finite arithmetic or final parent-conjugated matrix validation. Publication MUST retain only visible occurrences and preserve existing identity, order, RichText, bindings and timing.

#### Scenario: Validate hidden and unused layer boundaries
- **WHEN** root, hidden-track, hidden-repeater, hidden-instance, clipped-instance or unused-definition expansion has exactly 4,096 versus 4,097 visual occurrences
- **THEN** the inclusive boundary passes subject to other limits and one-over returns INVALID_ARGUMENT before generated materialization; hidden content produces no visible output

#### Scenario: Validate retained geometry and conjugation
- **WHEN** retained expansion reaches an exact versus one-over segment, surface or memory boundary, or finite offset powers yield a non-finite final parent-conjugated matrix
- **THEN** exact budgets pass subject to other limits and invalid work fails with INVALID_ARGUMENT before copies, raster allocation, renderer or artifact I/O

#### Scenario: Keep independent domains independent
- **WHEN** multiple unused definitions individually fit their limits but their sum exceeds one domain's limit, or definitions are reordered
- **THEN** validation accepts each independent domain consistently without adding its work to the root budget

#### Scenario: Preserve effective nested occurrences
- **WHEN** nested instances resolve slot overrides and RichText, contain local repeaters, and are copied by two outer repeaters
- **THEN** complete projected work is validated and published visible occurrences preserve deterministic identities, numeric order, intervals, transforms, opacity and isolated bindings, with independent source copies

### Requirement: Bounded ordinary evaluation without redundant snapshots
Ordinary evaluation MUST NOT deep-clone its entire visual collection for repeater discovery. Repeater expansion MUST use one shared projection/publication path with lightweight source references and MUST complete retained-domain validation before cloning any generated layer.

#### Scenario: Evaluate ordinary geometry without copy allocation
- **WHEN** a project without repeaters evaluates ordinary shapes or nested components
- **THEN** no repeater snapshot or generated-copy materialization occurs and semantic output remains unchanged

#### Scenario: Reject before the first generated clone
- **WHEN** any later repeater or independent retained domain exceeds a budget after earlier valid candidates
- **THEN** evaluation returns INVALID_ARGUMENT with zero generated-layer materializations and no partial scene or artifact

### Requirement: Complete retained transition fact budgets
Core MUST enforce the inclusive 4,096 transition-fact limit across all ordinary and generated visual occurrences in each independent root or component-definition validation domain. Each attached endpoint role MUST count as one fact, including both roles when endpoints coincide. Hidden transitions/tracks, hidden or clipped occurrences, and nested local/exterior copies MUST participate in validation without changing visible publication. Checked accumulation MUST reject overflow or excess with non-retryable INVALID_ARGUMENT before any generated layer materialization, backend execution or artifact I/O.

#### Scenario: Reject the reproduced expanded transition excess
- **WHEN** a component with 17 attached transition facts is evaluated ordinarily and repeated 256 additional times
- **THEN** the 4,369 projected facts return INVALID_ARGUMENT with zero generated materializations and no partial scene

#### Scenario: Accept exact and reject one-over fact limits
- **WHEN** root or local/exterior repeater expansion produces exactly 4,096 versus 4,097 endpoint facts, including self-endpoint roles
- **THEN** the exact boundary succeeds subject to other limits and one-over fails before generated copies

#### Scenario: Validate retained transitions without publishing them
- **WHEN** transitions, tracks, repeaters or instances are hidden or interval-clipped and retained expansion reaches or exceeds the transition limit
- **THEN** the same inclusive boundary applies while hidden transition facts and hidden occurrences remain absent from published output

#### Scenario: Isolate domains and reject later failures
- **WHEN** independent definitions individually fit but their sum exceeds one domain, or a later domain exceeds the limit after earlier valid candidates
- **THEN** valid independent definitions are accepted regardless of declaration order and the later invalid domain fails with zero generated materializations and no artifact I/O

### Requirement: Effective visual-only repeater sources
Repeater source closures MUST be visual-only after resolving component defaults and instance overrides. Core MUST validate root closures, standalone definitions with defaults, and actual effective instances, including group descendants, nested components and local repeaters. An effective media asset with audio MUST cause non-retryable INVALID_ARGUMENT regardless of visibility, clipping or mute. Validation MUST establish references, acyclicity and slot validity before effective-audio traversal, retain active-node cycle defenses, and isolate results for distinct effective values of the same definition. Valid silent sources MUST preserve existing IDs, ordering, bindings, timing and RichText behavior without audio repetition.

#### Scenario: Reject audio introduced by slots
- **WHEN** a default or instance asset override introduces audio into a repeated component or a component descendant of a repeated group, including nested components and local repeaters
- **THEN** canonical validation and direct evaluation return INVALID_ARGUMENT before state publication, generated materialization or renderer/artifact work

#### Scenario: Preserve effective-value isolation and silent sources
- **WHEN** different instances of one definition resolve silent and audible assets, or a source resolves an authored audible asset to a silent effective asset
- **THEN** classification follows each effective value independently of declaration order, silent sources remain valid subject to independent definition validation, and audible repeated sources are rejected

#### Scenario: Validate retained closures safely
- **WHEN** an effective audible source is hidden, muted, clipped or unused, or the input contains invalid references, cycles or slot values
- **THEN** retained audio is still rejected and invalid references, cycles and slots retain their established typed failures without unbounded recursion
