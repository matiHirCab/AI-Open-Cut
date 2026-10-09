## MODIFIED Requirements

### Requirement: Immutable typed audio analysis
Editor-core SHALL provide closed committed-root audio analysis through `analyze_audio {projectId,expectedRevision,startMs,endMs,waveformBins}` and queued MCP tool `audio_analyze_mix`. The core SHALL validate optimistic revision before semantic options, evaluation, resource or process work. The operation MUST preserve project/current/history bytes, active selection, undo/redo and deterministic reopen except existing store opening/recovery. It SHALL create only a process-local job and an owned generated preview artifact. No persisted schema change, migration, draft mutation or timeline_batch_edit member SHALL be introduced; schema44 adoption is owned separately by master-loudness-normalization, protocol1 and existing transactions remain compatible, and analysis introduces no additional persisted mutation.

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
Analysis SHALL evaluate the complete canonical root EvaluatedScene once and consume its audio layers, bindings, root/component/event/loop clocks, route precedence, legacy role ducking, bus gain/pan/EQ/compression, explicit narration envelopes and enabled authored root master normalization through the same audio lowering as preview/export. It MUST retain full-root source and detector clocks before final48kHz stereo conversion and exact selected sample-range trim/PTS reset. It MUST NOT implement a parallel evaluator, routing/envelope rule, or input seek to the selected range. Existing frame/range/draft/export plans, commands, Debug and output tolerances SHALL remain unchanged when authored normalization is absent or disabled; active authored normalization SHALL use the same independently verified full-root processing before analysis range crop.

#### Scenario: Analyze routed and processed content
- **WHEN** root/component/event layers, fallback/upstream routes, bus DSP and explicit or legacy ducking contribute to the mix
- **THEN** the analyzed original PCM matches the same complete evaluated processing and independently authored source/reference evidence within existing RMS0.0001/timing tolerance

#### Scenario: Keep processors warm at range start
- **WHEN** a range begins after earlier signal has driven a compressor or narration envelope
- **THEN** range PCM equals the corresponding exact frame crop of processed full-root PCM and cannot reset the earlier detector/control clocks

#### Scenario: Preserve every historical render intent
- **WHEN** existing fixtures render as frame, range, draft preview or export after the additive analysis feature
- **THEN** their original plans/filter/command evidence and pixel/audio/timing assertions remain valid

### Requirement: Deterministic original PCM statistics
For N positive frames and B=min(waveformBins,N), bin k SHALL cover half-open frame bounds floor(k*N/B)..floor((k+1)*N/B). Bins MUST be nonempty, contiguous and cover exactly N. Each SHALL contain startFrame/endFrame and closed left/right finite min/max/RMS statistics from the final authored finite float PCM, including enabled root master normalization before selected cropping. Analysis itself MUST apply no extra normalization or clipping; every historical absent/disabled PCM statistic and above-full-scale oracle remains unchanged. Summary SHALL expose actualBinCount/frameCount, linearSamplePeak=max absolute sample across both channels, and samplePeakDbfs=20log10(peak) or null for zero. Amplitudes SHALL be allowed above1. No producer-derived expected value SHALL replace independently authored numeric references.

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
Analysis SHALL measure the final authored selected stereo PCM including enabled root master processing through the configured local backend using fixed loudnorm input reporting (I=-24,TP=-2,LRA=7), discard normalized output, and expose integratedLufs,truePeakDbtp,loudnessRangeLu,thresholdLufs. Negative-infinite integrated/true peak SHALL map to null; LRA SHALL be finite/nonnegative and threshold finite. Null integrated loudness MUST NOT imply silence. Other malformed/nonfinite input report values SHALL fail safely; unused analysis measurement normalized output/target-offset values SHALL not determine original PCM statistics. These retained fixed loudnorm input integrated metrics MUST NOT substitute for the separate master-normalization EBU delivered-target verification; documented differences near relative gates SHALL not cause extra gain solely to force this analysis display to the configured target. Metrics SHALL document0.01dB reporting resolution and pass independent native cross-checks within0.05dB/LU while retaining existing PCM/timing tolerances.

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

### Requirement: Independently governed additive analysis contracts
The unique operation/tool/capability, result/job/JSON resource additions and conditional readiness SHALL agree across manually authored canonical fixtures, Rust/headless, TypeScript/Zod and MCP discovery. Protocol1, approved schema44 adoption under master-loudness-normalization and every prior request/tool/output/error/worker scenario SHALL remain compatible; this analysis feature's captured schema43 catalogs and all numeric oracles remain frozen through explicit successor rollback. Before executable edits, the final all11-verified predecessor's committed raw/expanded contracts MUST be independently captured. Additive rollback projections SHALL retain every old frozen raw byte/digest/count/oracle and mandatory consumers. Required Rust/TS/native/source/isolated-package/policy omission/failure-masking coverage SHALL run under unchanged gates/timeouts/profiles/tolerances. Final all11 exact-head CI/startup/source-tested-merge tree equality SHALL precede issue completion and successor implementation.

#### Scenario: Discover support conditionally
- **WHEN** the configured renderer supports the required analysis filters or one is unavailable
- **THEN** the compatible analysis tool remains typed/discoverable and capability reporting/analysis readiness truthfully reflects support without disabling existing editing or unrelated capabilities

#### Scenario: Compare independent predecessor projections
- **WHEN** current catalogs and native declarations are checked
- **THEN** they match the manually reviewed additive fixtures and roll back to captured85-tool schema43 predecessor semantics while every historical raw/digest/count proof remains intact

#### Scenario: Reject missing or masked required evidence
- **WHEN** an analysis consumer/required native or policy gate is omitted or its failure is masked
- **THEN** repository verification fails and issue completion cannot be claimed without the unchanged complete final checks
