## ADDED Requirements

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
