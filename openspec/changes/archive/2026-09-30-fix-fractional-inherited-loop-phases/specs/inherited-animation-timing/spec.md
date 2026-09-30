## ADDED Requirements

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
