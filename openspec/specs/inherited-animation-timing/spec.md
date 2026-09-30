# Inherited Animation Timing Specification

## Purpose

Define inherited visual animation and deterministic staggered and shifted occurrence clocks.

## Requirements

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

### Requirement: Staggered repeater controller branch timing
A repeater controller ranked as a direct visual child of a component instance MUST evaluate its complete generated source branch using that controller's ranked delay in addition to the one-based signed copy offset. The source branch's own timing MUST be preserved within that shifted evaluation. The containing-scope delay MUST be converted through the affine occurrence clock without intermediate rounding and MUST apply exactly once to source activity, media sampling and source-subtree animation stages. Repeater clipping, external ancestor clocks, ordinary source occurrences, canonical ranks, generated order and identity MUST retain their documented meanings. Audio MUST retain independent timing and MUST NOT be repeated.

#### Scenario: Restore a delayed short source
- **WHEN** an instance uses staggerMs 200 and its definition contains a rank-zero shape active during [0,100) followed by a rank-one repeater with one copy and timeOffsetMs zero
- **THEN** the ordinary source is active during [0,100) and the additional copy is active during [200,300), clipped to the instance and repeater intervals

#### Scenario: Compose signed offsets and fractional controllers
- **WHEN** a staggered repeater has zero, positive or negative timeOffsetMs inside fractional and nested component clocks with hidden visual siblings
- **THEN** every copy samples the independent affine-clock result for its ranked delay plus signed offset exactly once, hidden children retain rank, and activity has exclusive ends

#### Scenario: Preserve nested source stages and outside parents
- **WHEN** a staggered controller copies a source group or instance containing parent animation, child animation and a nested repeater
- **THEN** the complete source subtree uses the shifted clock while outside ancestors remain unshifted, ordinary sources retain timing, and internal occurrence order and identity remain deterministic

### Requirement: Independent inherited render conformance
Frame preview, audiovisual range preview, materialized draft preview and final export MUST conform to independent expected inherited clocks, parent-child matrices, opacity and media sampling for nested fractional instances, staggered groups, signed repeaters and animated descendants. Evidence MUST include cubic Bezier and spring segments, repeat and ping-pong seams, starts, interiors and exclusive ends; equality between two outputs using the same evaluator MUST NOT replace independent expected behavior. Audio MUST conform to independent source trim, component retiming, gain and activity expectations without visual stagger or repetition. Existing visual and audio tolerances MUST remain unchanged. Required native verification MUST fail explicitly when its compatible tools or pinned font are unavailable.

#### Scenario: Render nested curves and loops against an oracle
- **WHEN** nested fractional instances, animated staggered groups and animated children cross curve segments and repeat or ping-pong seams
- **THEN** all four render intents agree with independent time, matrix and opacity expectations at selected boundaries and interiors within the documented tolerances

#### Scenario: Sample visual media while preserving audio
- **WHEN** visual media samples shift through staggered component clocks and independent audio uses source trim and component retiming
- **THEN** decoded frames match independently selected source samples and decoded audio matches independent activity and sample expectations without additional visual delay or audio copies

### Requirement: Current inherited timing documentation and evidence
Current animation-channel, repeater and inherited-timing documentation MUST agree with schema-26 parent targets, signed offsets, ranked controller delays, fallback, half-open clipping and rejection semantics. Follow-up verification MUST map requirements and scenarios to inspected tests and executed checks, state skipped or unavailable evidence, and distinguish historical claims from corrected conformance evidence without rewriting the original issue-42 archive.

#### Scenario: Discover accurate current behavior
- **WHEN** a client reads the existing animation-channel and repeater guides alongside the inherited-timing guide and catalogs
- **THEN** the guides consistently describe supported parent animation and signed per-copy timing with their compatibility constraints

#### Scenario: Report the corrected evidence
- **WHEN** the follow-up is verified
- **THEN** its report identifies the repaired regressions and independent oracle coverage, records actual check results and limitations, and leaves the original archived artifacts unchanged

### Requirement: Independent fractional visual loop conformance
Inherited parent and descendant visual loop sampling MUST retain fractional mapped time through nested component clocks, group/component stagger and signed repeater offsets, without intermediate integer phase rounding. Frame, audiovisual range, materialized draft and export MUST match independently calculated phase, transform and opacity expectations immediately around repeat seams, ping-pong turns and finite exhaustion, including nonzero first-keyframe offsets. Existing integer-phase output, half-open activity, independent audio, source-media output-grid resampling, schema 26, stable errors, resource bounds and public fields MUST remain compatible. Required native verification MUST fail explicitly when compatible tools or the pinned font are unavailable; tolerances and canonical references MUST NOT be weakened.

#### Scenario: Reproduce the fractional parent seam across intents
- **WHEN** a component with timeScale 0.995 contains a parent repeat position-X channel with linear keys (0 ms, 0), (99 ms, 100), (100 ms, 0), and a child rectangle occupying pixels 10 through 19 before parent translation
- **THEN** at root 100 ms, all four intents place the child at pixels 60 through 69 from independent local time 99.5 ms and translation 50, instead of the incorrectly rounded initial position

#### Scenario: Compose nested fractional phases
- **WHEN** nested fractional instances, ranked group/component delays and positive or negative copy offsets place parent or child loops around a seam, turn or finite endpoint
- **THEN** independent scalar and native expectations confirm each applicable clock exactly once, unchanged outside-parent clocks, exclusive activity ends and independent audio without output-to-output equality replacing the oracle

#### Scenario: Preserve compatibility and accurate verification
- **WHEN** this follow-up is verified against integer-clock, zero-offset, curve, audio, contract and publication regressions
- **THEN** existing compatible results remain unchanged, prior archives remain intact, and the follow-up report records actual passing checks and unresolved limitations without claiming unsupported conformance
