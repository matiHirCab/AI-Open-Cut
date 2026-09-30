## ADDED Requirements

### Requirement: Deterministic child stagger clocks
Groups and component instances MUST accept optional staggerMs as an integer in [0,60000], with omission equivalent to zero. A group MUST rank its direct visual children in canonical containing-scope track/z-index/stack-order/ID order; a component instance MUST independently rank direct top-level visual children in its definition by the same comparator. Rank MUST be zero-based; hidden and clipped visual children, including controllers, MUST retain rank; audio-only items, captions and transitions MUST not consume rank. At containing-scope time t, direct child rank r MUST evaluate its complete occurrence branch at t minus r times staggerMs. Nested delays MUST compose in their respective local clocks without intermediate integer rounding. Parent activity, channel sampling and clipping MUST remain on the undelayed parent clock. Missing or zero stagger MUST preserve old output.

#### Scenario: Delay ordered children
- **WHEN** two visible children follow one hidden visual sibling under a staggered group or component instance
- **THEN** their branch activity and samples use ranks one and two, independent of visibility, and a zero-stagger parent preserves old timing

#### Scenario: Compose nested fractional clocks
- **WHEN** an instance with fractional timeScale contains a staggered group, nested instance and looped child channels
- **THEN** mapped child clocks, held or looped phase, and half-open activity match an independent affine-clock oracle in frame, range and export

### Requirement: Inherited parent motion
Active visual position X/Y, scale X/Y and opacity channels on a group or component instance MUST sample on that parent's own item-local clock before child stagger or repeater copy offsets. The sampled parent transform MUST compose outside descendant transforms in the existing column-vector matrix order, and sampled opacity MUST multiply descendant opacity. Every render intent MUST consume the same EvaluatedScene facts. Audio timing and gain MUST retain their existing independent behavior.

#### Scenario: Combine parent and child motion
- **WHEN** a group and nested component instance each animate supported channels while descendants animate and stagger
- **THEN** matrices and opacity equal ordered parent-child composition at keyframes, between keyframes and loop seams

### Requirement: Bounded shifted occurrence time
Core MUST use checked finite arithmetic for rank and copy multiplication, nested affine mapping, source activity and half-open intersections. It MUST enforce existing per-repeater, aggregate occurrence, fact, geometry and resource limits on complete shifted closures, including hidden and unused content, before generated materialization or artifact I/O. Invalid persisted or edit values, overflow, non-finite or unsafe derived time, or complexity excess MUST return non-retryable INVALID_ARGUMENT without changing revision, history, resources or artifacts.

#### Scenario: Reject invalid or excessive timing
- **WHEN** an offset is out of range or a nested shift yields unsafe derived time or exceeds an expanded-scene limit
- **THEN** core fails before generated materialization and state or artifact publication

#### Scenario: Preserve independent render intents
- **WHEN** frame preview, audiovisual range preview, draft preview and final export evaluate the same revision and composition time
- **THEN** each consumes identical shifted EvaluatedScene facts within established visual and audio tolerances

