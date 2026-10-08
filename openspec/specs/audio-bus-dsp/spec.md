# Audio Bus DSP Specification

## Purpose

Define canonical bounded normalized audio bus processing, shared evaluated routing, transactional persistence, conditional rendering dependencies and independently governed additive public conformance.

## Requirements

### Requirement: Closed bounded normalized bus processing settings
Schema42 built-in buses SHALL accept optional nonnull closed dsp. Omission MUST retain identity and omit serialized fields. A present block MUST require finite gainDb[-120,24], pan[-1,1], ordered eq array0..8 of closed frequencyHz[20,20000], q[0.1,10], gainDb[-24,24] peaking bands, and nullable compressor. A present closed compressor SHALL require finite thresholdDb[-60,0], ratio[1,20], attackMs[0.01,2000], releaseMs[0.01,9000], makeupGainDb[0,24], with downward/peak/maximum-link/hard-knee/full-wet semantics. All settings MUST remain inspectable normalized numbers, not backend expressions/resources. Unknown/null/nonfinite/out-of-bounds fields and excess bands MUST fail nonretryable INVALID_ARGUMENT before mutation. Four fixed buses/routes and existing complexity/resource limits MUST remain unchanged.

#### Scenario: Store and inspect normalized DSP
- **WHEN** a fixed bus receives valid nonidentity settings
- **THEN** its exact normalized values roundtrip under schema42 with no executable/path/expression input

#### Scenario: Reject malformed and excessive settings
- **WHEN** settings have invalid numbers, null block, missing/extra fields, more than8 bands, unknown bus or unsafe input
- **THEN** core rejects before publication and prior project/history/draft/resource bytes remain exact

#### Scenario: Preserve absent and explicit identity settings
- **WHEN** DSP is absent or gain0/pan0/all EQ gains0/compressor null
- **THEN** old bus shape is retained on omission and neutral processing retains exact old scene/plan/filter/output behavior

### Requirement: Prepared transactional bus DSP edits
Core SHALL expose typed audio_bus_set_dsp with fixed busId and required dsp through the existing standalone, ordered batch and materialized-draft transaction. It MUST preserve one logical revision/history entry, optimistic retryable REVISION_CONFLICT precedence, atomic late-failure rollback, stable INVALID_ARGUMENT bus/model errors and existing alias rules; fixed bus setters MUST NOT create resultAlias IDs. Validation/preview/rebase/commit, undo/redo/reopen, source-matched draft persistence and all original precommit rollback/postcommit complete-generation recovery boundaries SHALL remain canonical, with no new asset owner or transport validator.

#### Scenario: Edit standalone and ordered batch state
- **WHEN** a valid setter is submitted alone or with existing alias-bearing track/event edits
- **THEN** all operations commit once through canonical validation and complete inspectable bus state

#### Scenario: Reject conflicts late failures and persistence faults
- **WHEN** a stale request, invalid later operation or injected persistence phase occurs
- **THEN** rejected precommit candidates preserve every prior byte and error precedence, while durable postcommit faults recover the exact complete target generation under unchanged journal rules

#### Scenario: Restore materialized DSP drafts and history
- **WHEN** valid DSP intent is previewed/rebased/committed then undone/redone/reopened
- **THEN** normalized settings, selected content/routes/clocks and resources agree without speculative publication

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

### Requirement: Conditional DSP readiness before artifact side effects
Legacy base renderer readiness/capabilities MUST remain compatible. Active evaluated DSP SHALL require volume/aformat/pan/equalizer/acompressor support through the existing process port after canonical model/resource checks but before collision inspection/workspace/raster/execution/publication. Unsupported active DSP MUST fail DEPENDENCY_UNAVAILABLE without partial output or project/history/draft mutation. Base-only FFmpeg MUST retain ordinary rendering. audio_bus_dsp_v1 rendering capability MUST appear only with successful base and DSP readiness; typed normalized intent editing remains supported without render readiness. No adapter SHALL reconstruct bus rules.

#### Scenario: Retain legacy availability without DSP filters
- **WHEN** tooling supports original required filters but lacks a DSP filter
- **THEN** legacy readiness/rendering remains available, DSP capability is absent, and active DSP rejects before artifact side effects

#### Scenario: Reject invalid and unavailable rendering unchanged
- **WHEN** invalid models/resources/revisions or unavailable DSP dependencies are requested
- **THEN** original typed validation/error precedence occurs and no workspace, raster, destination overwrite or partial artifact is published

### Requirement: Independently governed additive DSP contracts
Protocol1 SHALL add unique audio_bus_set_dsp tool/operation and audio_bus_dsp_v1 support, closed normalized models and schema42 reporting across Rust/Zod/headless/MCP/project/draft consumers. Seven active41 headers SHALL advance42 only; all frozen catalogs/counts/literal hashes/native oracles SHALL remain immutable. Exact independently captured verified64 raw/expanded/headless/ownership projections MUST admit only explicitly recorded additions and reject unrelated old operation/error/schema/annotation/capability drift. Every prior full consumer/gate remains mandatory; new consumers MUST be enforceable against omission/failure masking.

#### Scenario: Exercise real public DSP workflows
- **WHEN** actual headless/source MCP/isolated packaged clients set bus DSP alone and with batches/drafts
- **THEN** typed models/capabilities/errors/revisions/history/reopen agree with canonical core

#### Scenario: Reject unrelated contract and consumer drift
- **WHEN** a producer or gate changes beyond approved additions or omits/masks a required consumer
- **THEN** independent predecessor and policy negatives reject without altering old hashes/counts/oracles

#### Scenario: Retain deterministic catalog expansion within existing verification bounds
- **WHEN** additive DSP schemas and every historical projection are expanded under the unchanged standard contract test
- **THEN** preserve complete ordered JSON trees, independent mutable schema occurrences, strict reference errors and every literal digest/count while avoiding redundant intermediate cloning; retain the original five-second test timeout and every assertion
