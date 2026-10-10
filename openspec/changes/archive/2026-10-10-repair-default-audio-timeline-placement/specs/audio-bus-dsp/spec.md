## MODIFIED Requirements

### Requirement: Closed bounded normalized bus processing settings
Schema42 built-in buses SHALL accept optional nonnull closed dsp. Omission MUST retain identity and omit serialized fields. A present block MUST require finite gainDb[-120,24], pan[-1,1], ordered eq array0..8 of closed frequencyHz[20,20000], q[0.1,10], gainDb[-24,24] peaking bands, and nullable compressor. A present closed compressor SHALL require finite thresholdDb[-60,0], ratio[1,20], attackMs[0.01,2000], releaseMs[0.01,9000], makeupGainDb[0,24], with downward/peak/maximum-link/hard-knee/full-wet semantics. All settings MUST remain inspectable normalized numbers, not backend expressions/resources. Unknown/null/nonfinite/out-of-bounds fields and excess bands MUST fail nonretryable INVALID_ARGUMENT before mutation. Four fixed buses/routes and existing complexity/resource limits MUST remain unchanged. The approved rendering-export ordinary-audio sample-placement correction SHALL supersede exact legacy graph/PCM preservation solely for erroneous timestamp-only clip placement; absent, identity and active processing MUST retain the same authored placement, and every unrelated compatibility guarantee MUST remain unchanged.

#### Scenario: Store and inspect normalized DSP
- **WHEN** a fixed bus receives valid nonidentity settings
- **THEN** its exact normalized values roundtrip under schema42 with no executable/path/expression input

#### Scenario: Reject malformed and excessive settings
- **WHEN** settings have invalid numbers, null block, missing/extra fields, more than8 bands, unknown bus or unsafe input
- **THEN** core rejects before publication and prior project/history/draft/resource bytes remain exact

#### Scenario: Preserve absent and explicit identity settings
- **WHEN** DSP is absent or gain0/pan0/all EQ gains0/compressor null
- **THEN** old bus shape is retained on omission and neutral processing retains exact old scene/plan/filter/output behavior

### Requirement: Canonical summed topological bus evaluation
Every render intent SHALL consume one renderer-neutral evaluated bus graph. Each audio item MUST retain existing source/instance timing, item/event gains, automation/fades/mute and role ducking before bus summation; event captured bus MUST override containing track explicit/role fallback, including component occurrences. Each reachable bus MUST sum direct and upstream inputs then apply gain, ordered peaking EQ, linked compression, optional approved explicit ducking and stereo balance before its parent; master MUST run once. Child-before-parent order MUST be deterministic with fixed-order ties, acyclic depth<=4 and <=32 EQ facts. Active DSP MUST use standard48kHz float stereo conversion; negative balance attenuates right by1+pan, positive attenuates left by1-pan, center leaves both1. Detector clocks MUST advance through silence without added lookahead/tails or duration/source/stream shifts. With absent/disabled/identity/inactive explicit ducking, absent/identity/unreachable DSP SHALL preserve exact legacy semantic plans/filter graphs/RGB/PCM and old role ducking. New metadata/work/live facts MUST be canonically admitted before I/O. The approved rendering-export ordinary-audio sample-placement correction SHALL supersede exact legacy graph/PCM preservation solely for erroneous timestamp-only clip placement; absent, identity and active processing MUST retain the same authored placement, and every unrelated compatibility guarantee MUST remain unchanged.

#### Scenario: Apply shared nonlinear compression after overlap summation
- **WHEN** overlapping items route into one compressor bus
- **THEN** the detector processes their summed signal and native output matches an independent reference rather than per-item compression

#### Scenario: Process nested routes and captured event buses once
- **WHEN** buses route through other stems and events/components use captured or explicit/fallback routes
- **THEN** each selected node and master runs once in declared order with unchanged selected content/gain and composition clocks

#### Scenario: Preserve neutral and unreachable legacy output
- **WHEN** all reached DSP is absent/identity or changed DSP has no audible input
- **THEN** original scenes/plans/filter graphs and required legacy pixels/audio remain exact

#### Scenario: Preserve a known silent routed signal
- **WHEN** every selected input reaching changed DSP has zero evaluated item gain, including volume automation on that zero-gain item
- **THEN** retain the exact legacy scene/plan/filter/readiness behavior because item gain multiplies every audio control; positive-gain upstream routes still activate processing and all authored settings still validate

#### Scenario: Compare native gain balance EQ and compressor across intents
- **WHEN** fixed normalized effects are rendered in frame/range/draft/export and history/reopen
- **THEN** common evaluated semantics satisfy original SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing bounds against independent native references
