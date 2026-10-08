## Why

Issue #68 needs agents to inspect the actual routed, processed mix through waveform, sample-peak, true-peak and LUFS evidence. The verified schema43 evaluator and renderer already own that mix; external approximations would lose component clocks, audio events, bus DSP and narration ducking.

## What Changes

- Add one closed typed headless operation `analyze_audio`, one queued MCP tool `audio_analyze_mix`, and conditional capability `audio_analysis_v1` for an immutable committed project revision and explicit root range.
- Publish a bounded JSON artifact and typed job summary describing original48kHz stereo PCM, deterministic waveform bins, sample peak, nullable LUFS/true peak, loudness range and threshold. Preserve metadata-first polling and explicit opaque resource retrieval.
- Reuse complete root `EvaluatedScene`, current input clocks, per-item processing and summed bus processing; crop after full-root processing so detectors and envelopes stay warm. Resolve selected audio bindings through the existing resource owner without visual rasterization or unrelated visual/font file reads.
- Extend the existing render-work process lifetime and job/resource owners, with bounded streaming/measurement, safe cancellation, path confinement and unchanged historical contracts/gates.

## Capabilities

### New Capabilities

- `audio-mix-analysis`: read-only bounded canonical audio analysis, numerical semantics, structured output, typed failures and compatibility.

### Modified Capabilities

- `agent-bridge`: include audio analysis in the existing reusable render-work boundary while preserving non-render one-shot behavior and all worker isolation scenarios.
- `artifact-resources`: add bounded explicit JSON text resources for analysis jobs under the existing metadata/ownership/retention policy.
- `rendering-export`: require audio analysis to share complete evaluated root audio processing with preview/export while selecting only audio resources.

## Impact

Editor-core evaluation/resource/render-plan/process/orchestration owners, typed headless transport, bridge jobs/Zod/tool/worker/resource adapters, manually reviewed canonical catalogs and every affected consumer, mandatory native/reference/contract/policy/source/package checks, and documentation. No new top-level owner or dependency direction is needed. Capture the independently committed verified67 catalogs before edits; additive rollback must retain all prior raw/expanded semantic proofs and scenarios.

## Compatibility and Non-goals

This is additive: project schema remains43 and headless protocol remains1. Existing request/result/error shapes, tools, worker cases and binary resources remain valid. There are no persisted edits or migrations; analysis is not a `timeline_batch_edit` mutation or draft setter. It creates only an owned preview artifact and a process-local job. History, undo/redo, reopen and project bytes must remain unchanged except existing store opening/recovery behavior. Existing render/edit work limits do not change.

No master normalization/limiting (#69), desktop inspection or narration fixture (#70), provider/network inference, custom routing/detectors, arbitrary public file paths or FFmpeg expressions, new error codes, major contract changes, weakening of existing checks/oracles/timeouts/profiles, merging/deployment or issue closure is proposed. Subsequent branches start this issue's final verified branch; every draft PR targets main and documents cumulative scope and merge order.
