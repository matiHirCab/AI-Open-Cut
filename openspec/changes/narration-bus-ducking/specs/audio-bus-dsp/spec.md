## MODIFIED Requirements

### Requirement: Canonical summed topological bus evaluation
Every render intent SHALL consume one renderer-neutral evaluated bus graph. Each audio item MUST retain existing source/instance timing, item/event gains, automation/fades/mute and role ducking before bus summation; event captured bus MUST override containing track explicit/role fallback, including component occurrences. Each reachable bus MUST sum direct and upstream inputs then apply gain, ordered peaking EQ, linked compression, optional approved explicit ducking and stereo balance before its parent; master MUST run once. Child-before-parent order MUST be deterministic with fixed-order ties, acyclic depth<=4 and <=32 EQ facts. Active DSP MUST use standard48kHz float stereo conversion; negative balance attenuates right by1+pan, positive attenuates left by1-pan, center leaves both1. Detector clocks MUST advance through silence without added lookahead/tails or duration/source/stream shifts. With absent/disabled/identity/inactive explicit ducking, absent/identity/unreachable DSP SHALL preserve exact legacy semantic plans/filter graphs/RGB/PCM and old role ducking. New metadata/work/live facts MUST be canonically admitted before I/O.

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
