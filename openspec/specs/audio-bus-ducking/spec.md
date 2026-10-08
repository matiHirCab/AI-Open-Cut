# Audio Bus Ducking Specification

## Purpose

Define bounded explicit narration clip-activity envelopes, canonical preprocessing routing and precise clocks, atomic schema43 transactions, conditional rendering readiness and independently governed additive public conformance.

## Requirements

### Requirement: Closed bounded explicit bus ducking settings
Schema43 built-in buses SHALL accept optional nonnull closed ducking requiring enabled boolean, sourceBusId existing fixed ID different from target, finite gain[0,1] and integer attackMs[0,2000]/releaseMs[0,9000]. Omission MUST serialize absent; disabled/gain1 values SHALL remain inspectable identity. Unknown/null/incomplete/nonfinite/out-of-bound settings and self/missing sources MUST fail INVALID_ARGUMENT before mutation, including unused buses. Fixed identities/order/acyclic signal routes remain unchanged; preprocessing control references SHALL add no signal edge or feedback.

#### Scenario: Store inspect and retain normalized controls
- **WHEN** a fixed bus receives valid enabled or disabled/identity ducking
- **THEN** exact normalized values roundtrip and omission retains the previous bus shape

#### Scenario: Reject malformed numbers and references
- **WHEN** any settings are incomplete/null/unknown/nonfinite/out-of-bound or reference self/missing source
- **THEN** canonical validation rejects unchanged with typed nonretryable INVALID_ARGUMENT

### Requirement: Prepared transactional bus ducking edits
Core SHALL expose typed audio_bus_set_ducking(busId,ducking) through existing standalone/ordered batch/materialized draft preparation. It MUST preserve optimistic REVISION_CONFLICT precedence, one revision/history entry, fixed-bus noncreating aliases, atomic rollback, deterministic undo/redo/reopen and existing18 publication-fault/source-matched-draft recovery boundaries. No transport SHALL reconstruct these rules or add resource ownership.

#### Scenario: Edit standalone batch and scoped alias state
- **WHEN** valid ducking is submitted alone or alongside alias-bearing scoped track/event edits
- **THEN** canonical prepared validation commits once and controls remain inspectable

#### Scenario: Reject conflicts late failures and publication faults
- **WHEN** a stale/invalid later operation or any existing publication fault occurs
- **THEN** rejected precommit state preserves every byte/resource and committed durable faults recover the complete exact generation

#### Scenario: Restore draft and retained history intent
- **WHEN** controls are validated/previewed/rebased/committed then undone/redone/reopened
- **THEN** saved settings and selected content/routes/clocks agree without speculative publication

### Requirement: Canonical routed narration clip activity envelopes
All render intents SHALL consume one bounded renderer-neutral minimum attack/hold/release envelope per active target bus after summed DSP compression and before balance/output. Selected positive-base-gain audio layer spans reaching sourceBusId, including upstream buses, SHALL define conservative clip activity before bus processors; hidden/muted/excluded/zero-base-gain inputs SHALL not contribute. Fades/automation and decoded silence MUST NOT be advertised as amplitude detection. Captured event bus SHALL override track explicit/role fallback and precise clipped component/root clocks SHALL be retained. Merge sorted adjacent/overlapping half-open activity, cap64 merged intervals per configured source and512 piecewise segments per bus under existing4096-layer/four-node/resource/work admission. Gain SHALL ramp from1 at start-attack to configured gain at start, hold during activity and release to1 at end+release; minimum SHALL combine overlaps, zero times SHALL change instantaneously, and anticipatory clipping SHALL retain full attack denominator. No duration/tail/source/stream shift or signal feedback SHALL be introduced.

#### Scenario: Apply explicit routing independent of roles
- **WHEN** narration inputs/events/upstream stems reach the selected source bus and target inputs reach a configured bus
- **THEN** captured/explicit/fallback routes select one shared envelope and each bus/master runs once in declared order

#### Scenario: Retain component clocks and selected spans
- **WHEN** narration/event occurrences have clipped offset/scaled component clocks or gaps
- **THEN** source activity and target control use the same precise root-time facts for range/draft/export

#### Scenario: Apply attack release overlap and instantaneous controls
- **WHEN** source intervals have clipped anticipatory attack, hold, adjacent overlapping ramps or zero attack/release
- **THEN** independent piecewise references agree with minimum-envelope output and exact boundary semantics

#### Scenario: Keep conservative clip activity explicit
- **WHEN** a selected positive-base-gain source has automation/fades or silent decoded samples
- **THEN** its selected clip span remains activity while muted/hidden/zero-base-gain content is excluded

#### Scenario: Preserve old role behavior and inactive processing
- **WHEN** new controls are absent/disabled/gain1/no-source/no-positive-target or old role settings coexist with explicit controls
- **THEN** inactive scenes/Debug/plans/filter graphs/output remain exact and active controls compose with unchanged saved per-item role ducking once per bus

#### Scenario: Reject excessive canonical work before effects
- **WHEN** source intervals or envelope work exceeds approved bounds in current or retained selected content
- **THEN** canonical INVALID_ARGUMENT occurs before I/O/publication with unchanged resources and no raw expression input

### Requirement: Conditional ducking rendering readiness
Unchanged base readiness SHALL remain available without ducking/DSP filters. Active duck-only graphs SHALL require volume/aformat/pan, while actual active DSP retains its existing five-filter requirements. Missing required support MUST fail DEPENDENCY_UNAVAILABLE after canonical model/resource validation and before collision/workspace/raster/process/artifact effects. Status SHALL add audio_bus_ducking_v1 only with base and actual ducking support; storing/editing controls MUST remain independent of rendering availability.

#### Scenario: Retain legacy and duck-only availability
- **WHEN** tooling has original filters and ducking support but lacks EQ/compression
- **THEN** legacy and duck-only requests remain available while genuine DSP still rejects

#### Scenario: Reject each missing filter before side effects
- **WHEN** active ducking lacks volume/aformat/pan or input models/resources are invalid
- **THEN** stable validation/dependency precedence applies and no collision overwrite/workspace/raster/process/artifact is published

### Requirement: Independently governed additive ducking contracts
Protocol1 SHALL add unique audio_bus_set_ducking/audio_bus_ducking_v1, closed normalized bus/operation models and schema43 reporting across Rust/Zod/headless/MCP/draft consumers. Seven active42 headers SHALL advance43 only; every frozen42/41/40/39 and older catalog/count/hash/native oracle SHALL remain immutable. Exact independently captured final66 raw/semantic/expanded/headless/ownership proofs MUST admit only manually approved additions and reject unrelated old model/annotation/error/operation/capability drift. Every old/new consumer SHALL remain mandatory with omission/failure-masking negatives and unchanged verification timeouts/profiles.

#### Scenario: Exercise real source packaged and headless workflows
- **WHEN** actual clients set controls standalone and through batches/drafts then history/reopen
- **THEN** typed models/capabilities/revisions/errors/persisted results agree with canonical core

#### Scenario: Preserve every predecessor and reject unrelated drift
- **WHEN** only approved additions are projected or unrelated old fields/capabilities/annotations are mutated
- **THEN** final66 and every older frozen proof agree for approved changes and reject drift without regenerated producer expectations

#### Scenario: Require complete new consumers and native proof
- **WHEN** a required ducking consumer/native command is omitted substituted or failure-masked
- **THEN** protected policy rejects while every previous gate/oracle remains required
