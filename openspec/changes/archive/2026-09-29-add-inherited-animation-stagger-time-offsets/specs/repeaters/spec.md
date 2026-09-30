## MODIFIED Requirements

### Requirement: Strict bounded repeater descriptors
Core MUST support item type `repeater` with required descriptor `repeater` containing `source`, `copies`, `transformOffset`, `opacityOffset`, and optional `timeOffsetMs`. `source` MUST be a closed `{scope,id}` reference to a shape, group, or component instance in the repeater's containing root or component-definition scope. `copies` MUST be an integer in [1,256] and denotes additional copies, excluding the independently evaluated source. `transformOffset` MUST be a closed object containing position `{x,y,unit}`, `scaleX`, `scaleY`, `rotationDeg`, `skewXDeg`, and `skewYDeg`; units and numeric bounds MUST match the corresponding Transform2D fields, scales MUST be positive, and the implicit transform origin MUST be the source-parent coordinate origin. `opacityOffset` MUST be finite in [-1,1]. Missing `timeOffsetMs` MUST mean zero; a present value MUST be a signed integer in [-60000,60000], and derived copy-index multiplication MUST be checked. Every object MUST reject missing required, unknown, duplicate, and positional fields in raw edit, batch, draft, component, project, and history input. Raw expressions, executable content, paths, and network resources MUST NOT be accepted.

#### Scenario: Accept boundary descriptors
- **WHEN** root and component-local repeaters name supported sources and supply copy counts 1 or 256, legal pixel or normalized position offsets, positive scale boundaries, bounded rotation/skew, and opacity offsets -1, 0, or 1
- **THEN** strict decoding and core validation preserve every supplied canonical field without expanding persisted copies

#### Scenario: Reject malformed or unsupported descriptors
- **WHEN** any repeater-bearing consumer supplies a missing, unknown, duplicate, null, positional, non-finite, out-of-range, expression-like, path-like, or network-bearing field, or names media, text, SVG, grid, caption, transition, audio-only, or repeater content as the source
- **THEN** decoding or core validation rejects the input with non-retryable `INVALID_ARGUMENT` before mutation, allocation, resource resolution, or renderer work

#### Scenario: Accept and reject copy timing fields
- **WHEN** a repeater supplies timeOffsetMs at -60000, 0 or 60000, omits it, or supplies an out-of-range, fractional, non-finite or unknown timing value
- **THEN** bounded integers and omission preserve canonical storage while invalid input returns INVALID_ARGUMENT before mutation

### Requirement: Deterministic additional-copy semantics
The ordinary source MUST continue to evaluate exactly once at its existing stack position. An active repeater MUST emit exactly `copies` additional source occurrences at the repeater's canonical stack position in ascending one-based copy-index order. A shape copy MUST clone that shape occurrence; a group copy MUST clone the group's complete transitive descendant occurrence set, including recursively resolved component-instance occurrences; and a component-instance copy MUST clone its complete resolved definition occurrence with identical slot bindings and local clock, including valid nested repeater occurrences. For additional copy index i, core MUST evaluate the complete source branch at containing-scope time t minus i times timeOffsetMs; the ordinary source at index zero MUST keep its original timing. Positive offsets MUST delay and negative offsets MUST advance copies. Source activity MUST use shifted time while each copy MUST intersect the unshifted repeater and inherited ancestor half-open intervals and visibility. For copy index `i`, core MUST normalize the position unit against the source scope canvas, build offset matrix `O`, and apply `O^i` in source-parent coordinates before the source local transform. Copy opacity MUST equal the ordinary source inherited opacity multiplied by `clamp(1 + i * opacityOffset, 0, 1)`. The persisted source and repeater MUST remain unchanged.

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
- **THEN** copies exist only in the interval intersection and each copy opacity uses the specified clamped formula without changing source opacity, with source activity shifted only when timeOffsetMs is nonzero

#### Scenario: Preserve deterministic ordering and identity
- **WHEN** multiple repeaters and ordinary layers share z-index values, a repeated group spans tracks and mixes flat/component descendants, or a repeated component contains at least eleven equal-z siblings
- **THEN** each generated block occupies its repeater's position under the canonical track/z-index/stack-order comparator, preserves numeric source-subtree order for every sibling count, orders copies by ascending index, and assigns deterministic occurrence identities derived from scope, repeater ID, copy index, and source occurrence path

#### Scenario: Offset each copy clock
- **WHEN** a repeater with positive or negative timeOffsetMs copies an animated group or retimed component instance
- **THEN** each copy samples its complete branch at the one-based offset, clips to the undelayed repeater interval, and leaves the ordinary source unchanged
