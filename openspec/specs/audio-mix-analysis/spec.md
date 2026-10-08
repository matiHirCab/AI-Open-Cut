# Audio mix analysis

## Purpose

Provide immutable bounded evidence about the canonical authored root audio mix through typed jobs and owned JSON resources.

## Requirements

### Requirement: Immutable typed audio analysis
Editor-core SHALL provide closed committed-root audio analysis through `analyze_audio {projectId,expectedRevision,startMs,endMs,waveformBins}` and queued MCP tool `audio_analyze_mix`. The core SHALL validate optimistic revision before semantic options, evaluation, resource or process work. The operation MUST preserve project/current/history bytes, active selection, undo/redo and deterministic reopen except existing store opening/recovery. It SHALL create only a process-local job and an owned generated preview artifact. No persisted schema change, migration, draft mutation or timeline_batch_edit member SHALL be introduced; schema43/protocol1 and existing transactions remain compatible.

#### Scenario: Analyze an immutable committed revision
- **WHEN** a valid request names the current revision and a positive root range
- **THEN** the job produces its typed analysis and artifact without changing project revision/history or active selection

#### Scenario: Preserve revision and missing-project failures
- **WHEN** a stale revision accompanies semantically invalid analysis options or a nonexistent project is requested
- **THEN** the canonical existing REVISION_CONFLICT or PROJECT_NOT_FOUND is returned before analysis resource/process/artifact work

#### Scenario: Keep history and edit boundaries unchanged
- **WHEN** analysis succeeds or fails between an edit, undo, redo and fresh-process reopen, or analysis is submitted as a batch edit
- **THEN** all original state/history behavior and bytes remain unchanged and the unsupported edit member fails through the existing typed boundary

### Requirement: Canonical warm root mix
Analysis SHALL evaluate the complete canonical root EvaluatedScene once and consume its audio layers, bindings, root/component/event/loop clocks, route precedence, legacy role ducking, bus gain/pan/EQ/compression and explicit narration envelopes through the same audio lowering as preview/export. It MUST retain full-root source and detector clocks before final48kHz stereo conversion and exact selected sample-range trim/PTS reset. It MUST NOT implement a parallel evaluator, routing/envelope rule, or input seek to the selected range. Existing frame/range/draft/export plans, commands, Debug and output tolerances SHALL remain unchanged.

#### Scenario: Analyze routed and processed content
- **WHEN** root/component/event layers, fallback/upstream routes, bus DSP and explicit or legacy ducking contribute to the mix
- **THEN** the analyzed original PCM matches the same complete evaluated processing and independently authored source/reference evidence within existing RMS0.0001/timing tolerance

#### Scenario: Keep processors warm at range start
- **WHEN** a range begins after earlier signal has driven a compressor or narration envelope
- **THEN** range PCM equals the corresponding exact frame crop of processed full-root PCM and cannot reset the earlier detector/control clocks

#### Scenario: Preserve every historical render intent
- **WHEN** existing fixtures render as frame, range, draft preview or export after the additive analysis feature
- **THEN** their original plans/filter/command evidence and pixel/audio/timing assertions remain valid

### Requirement: Audio-only selected resource ownership
Analysis MUST validate the entire canonical model/evaluation and resolve only its selected audio media bindings through existing managed media/path ownership. Audio-from-video SHALL resolve its canonical binding. It SHALL not rasterize visuals, resolve unrelated visual files or read fonts. Missing unrelated visual files MUST NOT block otherwise valid audio analysis; selected missing/unsafe audio and invalid model/complexity SHALL fail with existing typed errors before workspace/process/publication. No requested path, network resource or raw expression SHALL be accepted.

#### Scenario: Analyze audio despite an unrelated missing visual
- **WHEN** selected audio is valid but a file used solely by unrelated visual or font content is missing
- **THEN** audio analysis succeeds without opening/rasterizing that visual resource or font while canonical model validation still applies

#### Scenario: Reject missing or unsafe selected audio
- **WHEN** a selected evaluated audio binding is missing, unsafe or points through a forbidden path
- **THEN** existing managed-reference/path errors occur before process/workspace/artifact side effects without exposing unrelated file bytes

#### Scenario: Reuse source input clocks
- **WHEN** audio comes from trimmed media, video, events or clipped/scaled components
- **THEN** canonical media input selection/indices/source trim/duration and audio lowering remain exact rather than being reconstructed by an adapter

#### Scenario: Confine preview publication before work
- **WHEN** the owned previews destination is a symlink, non-directory or escapes its canonical project root, or its identity changes before publication
- **THEN** safe existing PATH_NOT_ALLOWED or FFMPEG_FAILED occurs before any write to that destination, initial unsafe paths fail before workspace/process work, and no outside artifact or project/history mutation occurs

### Requirement: Explicit analysis work and process bounds
Core SHALL require integer0<=startMs<endMs<=project duration, complete project duration<=600000ms and waveformBins1..4096. Output SHALL be stereo48000Hz with expected frames48*(endMs-startMs)<=28800000 and byte bound8*frameCount<=230400000. These limits MUST apply only to analysis. Processing SHALL use bounded buffers and concurrent bounded stderr drainage, reject nonfinite/partial/oversized/short PCM or failed exit, kill/reap failed descendants, and require exactly expected frames before publication. JSON serialization SHALL be at most4MiB and all public numeric values finite except explicit null logarithmic/unmeasurable metrics. Progress SHALL remain finite0..1.

#### Scenario: Enforce admission before effects
- **WHEN** a range, bin count, whole-project duration, finite model value or canonical shared complexity is invalid
- **THEN** the existing typed INVALID_ARGUMENT/model error occurs before resource/workspace/process/publication and ordinary rendering/editing limits remain unchanged

#### Scenario: Reject malformed process streams
- **WHEN** execution emits a nonfinite sample, trailing partial stereo frame, too few/many bytes, a failed exit after valid samples, or malformed measurement evidence
- **THEN** the analysis fails safely with existing FFMPEG_FAILED, kills/reaps owned descendants, drains bounded stderr and publishes no artifact

#### Scenario: Bound result and memory
- **WHEN** a maximum admitted analysis or an injected executor result is processed
- **THEN** buffer/result work stays bounded, all result invariants and4MiB serialization are checked, and no full PCM heap buffer or public PCM artifact is created

### Requirement: Deterministic original PCM statistics
For N positive frames and B=min(waveformBins,N), bin k SHALL cover half-open frame bounds floor(k*N/B)..floor((k+1)*N/B). Bins MUST be nonempty, contiguous and cover exactly N. Each SHALL contain startFrame/endFrame and closed left/right finite min/max/RMS statistics from the original unnormalized/unclipped float PCM. Summary SHALL expose actualBinCount/frameCount, linearSamplePeak=max absolute sample across both channels, and samplePeakDbfs=20log10(peak) or null for zero. Amplitudes SHALL be allowed above1. No producer-derived expected value SHALL replace independently authored numeric references.

#### Scenario: Partition unequal bins correctly
- **WHEN** an independent five-frame two-channel fixture requests two bins
- **THEN** bounds are0..2 and2..5 and channel extrema/RMS/global peak match independent arithmetic, including the boundary frame

#### Scenario: Preserve signs and over-range peaks
- **WHEN** original PCM has negative-only, right-only, zero or above-full-scale values
- **THEN** channel extrema/RMS and global linear/logarithmic peak describe those actual values without clipping or normalization

#### Scenario: Limit bins to available frames
- **WHEN** requested bins exceed positive frame count
- **THEN** actualBinCount equals the frame count and every bin contains exactly one frame

### Requirement: Input loudness and true-peak evidence
Analysis SHALL measure the original selected stereo PCM through the configured local backend using fixed loudnorm input reporting (I=-24,TP=-2,LRA=7), discard normalized output, and expose integratedLufs,truePeakDbtp,loudnessRangeLu,thresholdLufs. Negative-infinite integrated/true peak SHALL map to null; LRA SHALL be finite/nonnegative and threshold finite. Null integrated loudness MUST NOT imply silence. Other malformed/nonfinite input report values SHALL fail safely; unused normalized output/target-offset values SHALL not determine original PCM statistics. Metrics SHALL document0.01dB reporting resolution and pass independent native cross-checks within0.05dB/LU while retaining existing PCM/timing tolerances.

#### Scenario: Measure an independently defined tone
- **WHEN** native analysis consumes a deterministic stereo tone with independently authored peak/RMS/loudness reference
- **THEN** sample/true peak and loudness metrics describe the original tone within documented reporting/native tolerance

#### Scenario: Represent silence without fabricated loudness
- **WHEN** the original selected PCM is silence
- **THEN** extrema/RMS/linear peak are zero, samplePeakDbfs/integratedLufs/truePeakDbtp are null, and LRA/threshold remain finite

#### Scenario: Preserve short audible true peak
- **WHEN** a short audible range has unmeasurable integrated loudness but finite true peak
- **THEN** integratedLufs is null while the true peak/sample metrics remain finite and audible; nullI is not converted to silence

#### Scenario: Reject poisoned input measurement
- **WHEN** bounded final measurement evidence is missing, malformed or contains invalid input metrics
- **THEN** existing safe FFMPEG_FAILED is returned with no fabricated zero result or published artifact

### Requirement: Structured analysis artifacts and jobs
Headless SHALL return closed AudioAnalysisResult containing artifact metadata and a typed summary with startMs,endMs,sampleRateHz48000,channels2,frameCount,actualBinCount,linearSamplePeak,nullable samplePeakDbfs/integratedLufs/truePeakDbtp,finite nonnegative loudnessRangeLu and finite thresholdLufs. The bounded application/json artifact SHALL contain version1, the same summary and its waveform bins. MCP SHALL expose process-local job kind audio_analysis with optional audioAnalysis summary and the existing owned resource descriptor/link. Polling MUST be metadata-first without opening artifact bytes; existing speech result, job fields and binary content policy SHALL remain valid. The queued artifact-creating tool SHALL retain existing WRITE annotation semantics.

#### Scenario: Deliver typed metadata first
- **WHEN** a completed analysis job is polled normally or through its job-status resource
- **THEN** the same finite summary and opaque JSON descriptor/link are exposed without waveform/file reads, even if its backing file subsequently disappears

#### Scenario: Expose explicit waveform evidence
- **WHEN** the retained job's artifact resource is explicitly read
- **THEN** its bounded versioned JSON text exposes the exact matching summary and deterministic bins

#### Scenario: Preserve speech and existing jobs
- **WHEN** original preview/export/speech/transcription jobs run or includeBinary is supplied
- **THEN** their previous result/metadata/binary behavior remains unchanged and analysis JSON is not embedded as a legacy binary block

### Requirement: Isolated cancellable analysis lifetime
Audio analysis SHALL use the existing render-work reusable/one-shot dispatch and process-local job lifecycle. Cancellation/deadline/shutdown MUST terminate and reap rendering/measurement descendants, clean only request-owned temporary JSON/workspace files, and leave unrelated concurrent jobs intact. Retention, eviction, restart and foreign job access SHALL preserve existing JOB_NOT_FOUND. Failed or cancelled work MUST publish neither partial output nor a completed analysis summary. Artifact reads MUST retain existing session/UUID/path/symlink confinement and safe errors.

#### Scenario: Cancel either analysis pass
- **WHEN** a job is cancelled, times out or the bridge shuts down during PCM production or measurement
- **THEN** owned descendants are reaped and owned temporary work is removed without affecting another job or publishing partial results

#### Scenario: Preserve reusable and overlapping work
- **WHEN** sequential analysis/render work and overlapping analysis work are dispatched
- **THEN** the available shared worker is reused, overlap uses an independent one-shot process and cancellation remains request-isolated

#### Scenario: Preserve retention and ownership failures
- **WHEN** an expired/evicted/restarted/foreign job or malformed/traversing/symlinked resource is read
- **THEN** existing JOB_NOT_FOUND or safe VALIDATION_FAILED is returned without file/path disclosure or project mutation

### Requirement: Independently governed additive analysis contracts
The unique operation/tool/capability, result/job/JSON resource additions and conditional readiness SHALL agree across manually authored canonical fixtures, Rust/headless, TypeScript/Zod and MCP discovery. Protocol1/schema43 and every prior request/tool/output/error/worker scenario SHALL remain compatible. Before executable edits, the final all11-verified predecessor's committed raw/expanded contracts MUST be independently captured. Additive rollback projections SHALL retain every old frozen raw byte/digest/count/oracle and mandatory consumers. Required Rust/TS/native/source/isolated-package/policy omission/failure-masking coverage SHALL run under unchanged gates/timeouts/profiles/tolerances. Final all11 exact-head CI/startup/source-tested-merge tree equality SHALL precede issue completion and successor implementation.

#### Scenario: Discover support conditionally
- **WHEN** the configured renderer supports the required analysis filters or one is unavailable
- **THEN** the compatible analysis tool remains typed/discoverable and capability reporting/analysis readiness truthfully reflects support without disabling existing editing or unrelated capabilities

#### Scenario: Compare independent predecessor projections
- **WHEN** current catalogs and native declarations are checked
- **THEN** they match the manually reviewed additive fixtures and roll back to captured85-tool schema43 predecessor semantics while every historical raw/digest/count proof remains intact

#### Scenario: Reject missing or masked required evidence
- **WHEN** an analysis consumer/required native or policy gate is omitted or its failure is masked
- **THEN** repository verification fails and issue completion cannot be claimed without the unchanged complete final checks
