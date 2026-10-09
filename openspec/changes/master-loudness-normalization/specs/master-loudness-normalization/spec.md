## ADDED Requirements

### Requirement: Closed finite authored master normalization settings
Project SHALL allow optional nonnull closed masterNormalization with required enabled, targetIntegratedLufs in[-70,-5], targetLoudnessRangeLu in[1,50] and targetTruePeakDbtp in[-9,0]. Numeric values MUST be finite; controls SHALL validate even when disabled. Omission and disabled SHALL retain previous audio processing, scene Debug, plans/commands and existing pixel/PCM/timing oracles. Settings SHALL apply once at the complete root master after final balance, never separately to a component, branch or bus. No raw backend expression, requested path, network input or persisted measurement SHALL be accepted.

#### Scenario: Round trip active and disabled controls
- **WHEN** a valid enabled or disabled setting is edited, inspected, serialized or reopened
- **THEN** its exact closed typed controls remain present and disabled rendering retains original behavior

#### Scenario: Reject malformed settings
- **WHEN** a required member is missing or a setting is null, unknown, nonfinite or outside an inclusive bound
- **THEN** the canonical existing typed boundary or INVALID_ARGUMENT error occurs without changing state/history

#### Scenario: Preserve omission and inactive rendering
- **WHEN** historical content has no settings or explicit disabled settings
- **THEN** all original evaluated/plan/filter/command and native pixel/PCM/timing evidence remains valid and no normalization process/readiness requirement runs

### Requirement: Prepared atomic standalone batch and draft edits
Editor-core SHALL own audio_master_set_normalization with required normalization object through existing standalone, timeline_batch_edit and draft typed edits, changedIds=[master], optimistic revisions, final-candidate validation and existing durable transactions. This setter SHALL not create IDs or accept resultAlias; surrounding batch aliases retain their previous semantics. Editing MUST NOT require a rendering backend. Existing revision conflict priority, rollback, current/history/media provenance and draft source/base generation behavior SHALL remain unchanged.

#### Scenario: Apply real standalone and batch edits
- **WHEN** valid settings are submitted standalone or inside a batch with ordinary aliased creation operations
- **THEN** the same normalized root controls are committed once with normal changed IDs/revision and unchanged creation alias semantics

#### Scenario: Fail stale invalid and late batch edits atomically
- **WHEN** a stale request includes invalid controls or a later batch member fails after a normalization edit
- **THEN** existing REVISION_CONFLICT priority or canonical failure is returned and all current/history/managed files remain unchanged

#### Scenario: Retain draft undo redo and reopen behavior
- **WHEN** settings are materialized in a draft, committed, undone, redone or reopened in a fresh process
- **THEN** the exact root settings and existing source/base revision rules remain deterministic with no persisted coefficient/cache/result data

#### Scenario: Fail each durable publication phase
- **WHEN** one of the original eighteen named atomic-publication faults occurs during settings edit or migration
- **THEN** no partially changed current/history/settings/managed state is exposed and existing recovery semantics remain intact

### Requirement: Atomic schema44 current and history adoption
Schema44 SHALL introduce optional masterNormalization while protocol1 remains compatible. Existing valid sources1..43 and every retained undo/redo snapshot MUST migrate atomically under the existing project lock, without inventing controls or changing media/provenance/timestamps beyond existing migration rules. A pre44 source containing the new field, including null or disabled, MUST fail. Schema44 SHALL reject present null/unknown/malformed settings; mixed or future source generations MUST preserve existing rejection/recovery behavior. All older feature catalogs/pins SHALL remain frozen.

#### Scenario: Adopt every previous source and retained snapshot
- **WHEN** sources1..43 with original feature restrictions/current/history are opened
- **THEN** all valid records become44 in one publication and retain original fields while masterNormalization remains absent

#### Scenario: Preserve active44 controls across history
- **WHEN** current and retained44 snapshots include valid distinct enabled/disabled settings
- **THEN** inspect/undo/redo/reopen restore each exact snapshot's controls without recomputing or persisting measurements

#### Scenario: Reject premature malformed mixed and future generations
- **WHEN** a pre44 record contains new controls, a44 record is malformed/null, retained history mixes forbidden generations or a future version appears
- **THEN** existing canonical typed failure is returned without partial publication or reinterpretation

### Requirement: Bounded full root normalization preparation
Core SHALL evaluate the complete authored root once, reuse existing selected source/component/event/routing/item/DSP/duck/sum/master-balance lowering, and prepare active normalization before final frame/range/draft/export/analysis crop or codec. Active root duration SHALL be positive and at most600000ms,48kHzstereo N=48*durationMs<=28800000, private PCM bytes<=230400000 per file and at most three such retained PCM files. Fixed prepared facts SHALL be charged to existing memory/complexity bounds; no full PCM heap allocation or public PCM artifact SHALL occur. Capture SHALL use an explicit private bypass to avoid recursive normalization. Direct pure plans with active unprepared settings and default-unsupported preparation adapters MUST fail safely. Analysis SHALL resolve only selected audio resources without unrelated visual/font opens; ordinary visual preflight/matte/error precedence SHALL remain intact.

#### Scenario: Admit full root before selected output
- **WHEN** a valid active root is rendered as frame, selected range, draft, export or audio analysis
- **THEN** identical complete-root controls/source/detector clocks are prepared before any final crop and normalization occurs only once after master balance

#### Scenario: Reject bounds unsupported and missing resources before publication
- **WHEN** active work exceeds root/frame/finite/shared complexity bounds, preparation is unsupported or selected audio is missing/unsafe
- **THEN** existing INVALID_ARGUMENT/DEPENDENCY_UNAVAILABLE/resource/path/FFMPEG_FAILED occurs at its canonical boundary with no final artifact or state/history change

#### Scenario: Keep audio analysis independent of unrelated visuals
- **WHEN** the normalized authored mix has valid selected audio but unrelated missing visual/font content
- **THEN** analysis prepares the same normalized root without opening/rasterizing unrelated visual/font files while canonical model validation remains required

### Requirement: Target specific measured processing and explicit fallbacks
Preparation SHALL bounded-stream complete original finite stereo f32 PCM, reject partial/short/extra/nonfinite data and failed exits, derive deterministic pregain=min(0,-12-originalSamplePeakDbfs) only to admit over-range sources safely, and perform target-specific loudnorm measurement followed by measured normalization with identical pregain and explicit48kHzstereo f32 boundaries. Checked inputI/TP/LRA/threshold/target_offset MUST satisfy backend finite measured bounds; no silent clamp SHALL fabricate target success. Linear processing may enter documented dynamic mode. Exact zero original peak SHALL be identity. Null/unmeasurable integrated loudness with finite true peak SHALL be limiting-only with gain<=1 and measured ceiling margin, never falsely promise integrated target attainment or infer silence. Other unsupported measurement evidence MUST fail safely. LRA SHALL be processing guidance, not an exact output-range promise.

#### Scenario: Normalize independently authored stereo tone
- **WHEN** a measurable independently specified stereo tone uses valid target controls
- **THEN** measured two-pass processing reaches the configured integrated target and true-peak ceiling using actual emitted full-root PCM

#### Scenario: Retain dynamic peak limiting and over-range sources
- **WHEN** varying high-crest content requires dynamic limiting or original finite samples exceed full scale
- **THEN** bounded measured processing preserves root frames/channels and actual target evidence without clipping/clamping the original measurement or inventing success

#### Scenario: Preserve exact silence and limiting-only short quiet content
- **WHEN** original PCM has exact peak0 or audible short/very quiet content has unmeasurable integrated loudness
- **THEN** silence stays exact zero while nonzero content follows explicit gain<=1 limiting-only behavior with finite true-peak evidence and no fabricated integrated target

#### Scenario: Reject invalid measured evidence
- **WHEN** reports are missing/truncated/poisoned/nonfinite or measured options cannot be represented safely
- **THEN** existing FFMPEG_FAILED is returned, owned work is cleaned and no completed artifact or fabricated metrics are published

### Requirement: Independent actual output target verification
Core MUST verify actual complete-root precodec48kHzstereo normalizedPCM with the configured backend's independent ebur128 integrated scanner and true-peak evidence. Measurable delivered integratedI SHALL be within0.1LU of the configured target using final0.001LU metadata drained through fixed ametadata into a bounded final tail; actual true peak SHALL not exceed the configured ceiling plus0.01dB fine-report quantization. Existing fixed loudnorm inputTP may supply fine true-peak evidence, but its integratedI MUST NOT substitute for the independent delivered integrated metric, and rounded linear true-peak metadata MUST NOT substitute for tight dB ceiling or quiet fallback evidence. At most one finite constant correction derived from actual EBU integratedI MAY run if admitted against actual measured true-peak headroom and followed by final full verification. Infeasible targets/failing final evidence SHALL return safe FFMPEG_FAILED with no artifact. Guarantees apply to full-root precodec PCM, not equal integrated loudness for every selected crop or unchanged AAC codec peaks. Existing analysis's fixed-input metric semantics SHALL remain explicit and old numeric references unchanged.

#### Scenario: Verify changing audio with independent delivered meter
- **WHEN** independently authored varying audio produces a disagreement between fixed loudnorm inputI and independent EBU deliveredI
- **THEN** configured target acceptance uses actual EBU deliveredI and no gain is added merely to force the legacy analysis display to equal the target

#### Scenario: Admit one bounded safe correction
- **WHEN** actual measured integratedI differs from target and finite true-peak headroom permits a single constant correction
- **THEN** that correction is remeasured on actual complete PCM and only a verified result is admitted

#### Scenario: Reject mutually infeasible measured targets
- **WHEN** integrated and true-peak targets cannot both be satisfied by admitted processing/correction or final measurement fails
- **THEN** existing FFMPEG_FAILED occurs with no final artifact and no fabricated/clamped target success

#### Scenario: Keep selected and encoded measurements explicit
- **WHEN** a crop has different integrated loudness or a codec changes decoded sample peaks
- **THEN** documented full-root precodec target semantics and independent selected/decoded tolerances are used without claiming per-crop target equivalence

### Requirement: Shared prepared final master lowering and readiness
A checked finite private prepared descriptor SHALL be consumed by one shared final-master lowering after all prior bus/master processing and before final sample/output-side range crop/codec in frame/range/draft/export/analysis. It SHALL carry no public path/expression and MUST retain dynamic warm/lookahead/tail state. Active normalization SHALL require configured legacy/analysis filters plus ebur128/ametadata and existing conditional DSP/duck readiness before workspace/publication effects; disabled/absent editing/rendering SHALL retain prior availability. Missing each required filter MUST fail truthfully without fallback success.

#### Scenario: Prove actual warm range and end tail equivalence
- **WHEN** actual native full export and selected ranges include earlier driven dynamic processing and final tail
- **THEN** selected PCM equals independent complete-root reference crops within original RMS0.0001/timing tolerance and active frame/draft/export use the same evaluated processing

#### Scenario: Compose routed DSP ducking components and events once
- **WHEN** active root normalization follows routed gain/pan/EQ/compression/ducking with component/event clocks
- **THEN** native reference PCM proves existing processing order then exactly one master normalizer and unchanged visual evidence

#### Scenario: Reject each missing active filter
- **WHEN** any required active normalization filter is unavailable
- **THEN** truthful existing DEPENDENCY_UNAVAILABLE is returned without process/workspace/final publication while old editing and inactive rendering remain available

### Requirement: Isolated bounded normalization lifetime
Every original-capture/measurement/measured-processing/verification/optional-correction child SHALL inherit existing request-owned groups, bounded concurrent diagnostic drainage, exact finite frames, local kill/wait and owned workspace lifetime. At most one correction pair SHALL occur; no unbounded retry/pass/file/result work is allowed. Cancellation, deadline or shutdown MUST kill/reap all owned phases, clean only owned temporary/partial work, preserve overlapping jobs/existing completed outputs and leave project/history unchanged. Failed normalization MUST not publish a completed summary/artifact.

#### Scenario: Cancel every owned phase
- **WHEN** a request is cancelled, times out or shuts down during each active normalization phase
- **THEN** its actual descendants disappear, owned private/partial files are cleaned and unrelated jobs/outputs/state/history are preserved

#### Scenario: Preserve reusable and overlapping requests
- **WHEN** sequential and overlapping normalized preview/export/analysis work is dispatched
- **THEN** existing reusable/one-shot isolation remains valid and one cancellation cannot terminate another request

#### Scenario: Fail workspace stream process and publication safely
- **WHEN** injected workspace/filter/PCM/metric/exit/metadata/publication faults occur
- **THEN** the existing safe error boundary publishes no partial final result and retains every unrelated byte and canonical transaction

#### Scenario: Preserve cold completion and actual completed outputs during controlled lifecycle faults
- **WHEN** a lifecycle case first completes one real cold normalization request and then arms each owned phase for cancellation, deadline or shutdown
- **THEN** the cold request proves successful finite frame completion without changing current/history, controlled work proves actual backend/descendant entry and exact PID absence under the unchanged1500ms phase wait and2000ms deadline, and the previously completed analysis JSON plus unrelated outputs remain byte-identical with all original nine-phase and twenty-seven-mode cases retained

### Requirement: Independently governed additive normalization contracts
Unique audio_master_set_normalization/tool/capability audio_master_normalization_v1 and schema44 reporting SHALL agree across manually authored canonical contracts and all governed Rust/headless/TS/Zod/MCP consumers; protocol1 and every previous operation remain valid. Final all11-verified68 committed raw catalogs/pure expanded86tools MUST be captured before producers; manually reviewed87tool additions and rollback SHALL preserve all prior raw/digest/count/numeric oracles. Complete unchanged implementation/native/source/release-package/policy omission/failure-masking checks, transparent substantive CODEOWNER-role COMMENT, guarded owning-delta archive/protected Moon/strict validation and final all11 exact-head/startup/tree/full-log evidence MUST precede completion or70 implementation. Each draft PR SHALL target main and cumulative successor lineage/merge order remain explicit.

#### Scenario: Discover and exercise real public workflows
- **WHEN** actual source and isolated release-package clients discover support and execute standalone/batch/draft/history/analysis/render workflows
- **THEN** typed controls,44 reporting, successful target evidence and canonical errors agree while backend-unavailable editing remains usable

#### Scenario: Restore exact captured predecessor contracts
- **WHEN** current catalogs/native declarations are verified and new additions rolled back
- **THEN** exact captured86tool schema43 semantics and all historical bytes/pins/counts are restored and unrelated drift fails verification

#### Scenario: Reject omitted masked or stale acceptance evidence
- **WHEN** a governed consumer/native test/required policy gate is omitted, its failure masked or CI belongs to another head/tree
- **THEN** mandatory acceptance fails and completion/successor implementation cannot be claimed
