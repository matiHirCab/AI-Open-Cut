## Context

Issue68 follows verified issue67 head27f3fb2812a23d1aa8a5ccd73d60c4cae17c1625, all11 CI37830821588/startup37830821580 and source/tested-merge tree equality. Its branch starts that exact head. Before executable edits,58 committed predecessor files and the85-tool pure expanded catalog were captured independently; expanded SHA256674c363a16edfdc733bbfc0a94af10bed70e68880b9b006fb3326ccbefcd5ed4. The canonical project is schema43/protocol1. Existing resource, render-plan, process, renderer, bridge job, worker and opaque artifact owners already provide the needed inward dependency direction.

## Goals / Non-Goals

**Goals:** read-only committed-revision analysis of the actual root mix; bounded original PCM statistics and loudness evidence; typed cancellable jobs and metadata-first JSON resources; preservation of all prior rendering, state/history and contract proofs. See proposal for exclusions. Analysis has no persisted edit, migration, batch alias creation or draft setter; standalone headless operation and queued MCP job are the applicable public surfaces. Existing batch transactions/history/reopen retain their original behavior and tests.

## Decisions

### Immutable typed request and limits

Use closed `analyze_audio {projectId,expectedRevision,startMs,endMs,waveformBins}` and `audio_analyze_mix`, with capability `audio_analysis_v1`. Core validates revision before analysis option/model/resource work. Require0<=startMs<endMs<=project.duration_ms(), complete project duration<=600000ms and bins1..4096. Output is48kHz stereo; integer milliseconds map exactly48frames. Expected range frames=(end-start)*48<=28800000 and PCM bytes<=230400000. These are analysis-only work limits; preserve existing rendering/edit limits. Keep schema43/protocol1 because nothing is persisted. A generic render mutation/batch member would introduce inappropriate history/side effects; arbitrary path/FFmpeg request fields are excluded.

### Reuse evaluated full-root audio

Evaluate the complete canonical project once at its project canvas/fps. An audio-only plan nested under the existing render-plan owner uses the existing per-item `append_audio_layer` and `audio_bus_dsp::compile` paths and evaluated media/input facts. Do not rebuild source timing, event/track/fallback routes, role ducking, explicit envelopes, DSP or components from persisted records. Keep the original silence anchor/media input indexing and root source trim/duration; a bounded2x2 black/nullsink anchor preserves indices without visual rasterization. Append48kHz stereo format/resampling, exact sample-index range trim and PTS reset only after complete root summed processing. Input seeking to the selected range or a separate audio evaluator would cold-start compressors and control clocks and is rejected. Existing RenderPlan/Debug/commands and all historical output behavior remain unchanged.

### Select audio resources within the existing owner

Validate the entire canonical model/evaluation, then resolve only evaluated audio-layer media bindings through render_artifact's existing managed path/input owner. Audio-from-video still resolves its canonical asset binding. No visual raster/font/geometry reads occur, and an unrelated missing visual file does not block audio-only analysis; missing or unsafe selected audio fails before workspace/process/publication. Refactor shared binding/input helpers only with exact historical input/command assertions. Dependency readiness uses the unchanged configured renderer backend and an additional analysis-filter check for `aformat`, `aresample`, `atrim`, `asetpts`, `amix`, `loudnorm`, `volume`, `afade`, `atempo`, `adelay`, `anullsrc`, `color`, `nullsink`, plus conditional DSP/ducking readiness when the evaluated graph needs them. The public capability therefore promises the complete admitted per-item analysis surface; editing remains independent of analysis readiness. No new top-level module/ownership edge is introduced.

### Bounded PCM and measurement

Expose a conservative default-unsupported analysis method on the existing ProcessExecutor port; SystemProcessExecutor implements it. Nested render-plan analysis DTO/statistics code owns option/numeric/bounds rules, while the process owner streams48kHz stereo f32le output, computes bounded accumulator statistics and writes only that original selected PCM inside an owned RenderWorkspace. Drain bounded stderr concurrently. Require exact expected frames, no partial/trailing frame, finite samples, bounded bytes and successful exit; kill/reap on invalid output and discard workspace. No whole-PCM heap Vec or public PCM artifact. Progress uses the existing callback, remains finite0..1, and cannot contaminate the PCM stdout stream.

Run a second fixed measurement process on that original PCM with loudnorm I=-24,TP=-2,LRA=7 and JSON reporting; discard its normalized output. Parse only bounded final input_i/input_tp/input_lra/input_thresh evidence. Negative infinity for I/TP becomes null; zero PCM sample peak also has nullable dBFS. Preserve a short audible signal's finite true peak even when integrated loudness is unmeasurable. Never treat nullI as proof of silence, use normalized output statistics for waveform/peak, or validate unused target_offset/output infinity. LRA is finite/nonnegative; threshold finite. Other malformed/nonfinite report values fail safely as existing FFMPEG_FAILED. Reporting resolution is0.01dB; native loudness cross-check tolerance<=0.05dB/LU, independent PCM RMS<=0.0001 and existing timing tolerances remain unchanged.

### Deterministic result

AudioAnalysisResult contains `artifact` and typed `summary`. Summary contains startMs,endMs,sampleRateHz=48000,channels=2,frameCount,actualBinCount,linearSamplePeak,nullable samplePeakDbfs,nullable integratedLufs,nullable truePeakDbtp,loudnessRangeLu and thresholdLufs. The JSON artifact is `{version:1,summary,bins}` with at most4096 nonempty contiguous bins; each has startFrame,endFrame and closed left/right `{min,max,rms}` statistics. For N frames and B=min(requestedBins,N), bin k uses floor(k*N/B)..floor((k+1)*N/B). In particular N=5,B=2 partitions0..2 and2..5; assigning floor(frameIndex*B/N) is incorrect. Global peak is max absolute original sample and dBFS=20log10(peak), null at zero; amplitudes may exceed1. No clipping/normalization precedes statistics. Bound serialized JSON to4MiB, validate result invariants before publication even for an injected executor, and publish only an opaque generated `.json` preview artifact via existing atomic artifact I/O.

### Reuse jobs, worker and resources

Add process-local job kind `audio_analysis` and optional `audioAnalysis` summary while preserving the existing speech result union. Queue via startTask and typed headless call; job tools use existing artifact-creating WRITE annotations despite read-only project state. Extend only the existing render-work allow-list/request-owned cleanup (.json/workspace) for reusable and one-shot paths; existing deadline/cancel/overlap/descendant reaping applies to both process passes. Polls expose summary/metadata/resource_link without opening waveform bytes. Explicit resources/read returns bounded UTF8 JSON text with application/json, under current UUID/registry/path/symlink/retention/session ownership checks; includeBinary does not embed JSON. Bound actual opened-handle reads to4MiB plus one detection byte so a stat/read race cannot cause unbounded heap allocation. Tampered malformed/nonUTF8/oversized JSON and missing files use existing safe VALIDATION_FAILED without exposing paths. Expired/foreign/restarted jobs retain JOB_NOT_FOUND.

### Independently governed additive contracts

Manually author the new DTO/request/tool/capability/job/resource additions and ownership category against captured committed bytes before producer behavior changes. Add rollback projections for headless/MCP/ownership/worker additions before existing ducking/DSP/older projections. Retain every previous frozen catalog/raw byte/digest/tool count and historical native assertion; update current consumers only through explicit new-tool exclusions or new additive schemas. No historical proof regeneration from runtime producers. Add mandatory native/Rust/TS/source/package consumers and policy omission/failure-masking negatives without changing existing gates, timeouts, profiles or tolerances.

## Risks / Trade-offs

- Full-root processing can cost more than a cold selected range → explicit ten-minute work admission and existing cancellable deadlines; correct detector/envelope state takes precedence.
- FFmpeg integrated loudness can be undefined for short audible PCM → nullableI independent of finiteTP/sample peak, with direct short-tone and silence coverage.
- Original PCM can exceed nominal full scale → report its actual finite extrema/peaks; normalization/limiting is issue69.
- Local large suites have documented memory/disk caveats → preserve original failures, use focused diagnostics and accept only unchanged complete standard CI.
- Artifact tampering/races → opened-handle confinement plus bounded reads, exact parse/result checks and opaque process-local ownership.

## Migration Plan

No persisted shape/version or migration is needed. Current/retained project bytes, undo/redo and reopen remain identical. New request/tool/capability and optional job/resource fields are additive; older clients and all existing binary delivery remain valid. Roll back the issue by removing its additive code/contracts under a new scoped change, without rewriting user projects. Every PR targets main and is cumulative until predecessor user merges; merge161→162→issue68→69→70. Final all11 exact-head CI/startup/source-tested-merge equality is required before69 implementation.

## Open Questions

None required for implementation. The bounds, bins, null handling, resource selection, output fields, annotations, failure precedence and compatibility decisions above are concrete and must be reviewed in the scoped approval before executable edits.
