## Context

The completed optimized Render parity runs took 205m01s and 336m50s. Their `rules_screen` suite took 11,526.743s and 19,210.915s respectively, dominating the golden test in both runs. The suite executes five ordered lifecycle states at each of 960x540, 1280x720, and 1920x1080. Each state independently renders three frame previews, one audiovisual range preview, and one final export, then checks reviewed frame/audio/timing references, semantic plans, and project immutability. Each resolution uses its own temporary project and renderer. The logs currently expose only one aggregate `rules_screen` duration, so the cause of the run-to-run variation is unobserved.

The baselines are [run 35994612911](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/35994612911) and [run 36032914723](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36032914723). The protected native command remains `cargo test --release -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact --nocapture`. Each of the 15 cases renders frame previews at 0, 500, and 900 ms, range preview, and final export, for 75 production render operations. Reviewed references, SSIM >= 0.99, PCM RMS <= 0.0001, and one-frame timing tolerance remain unchanged.

## Goals / Non-Goals

**Goals:** Attribute each of the 75 production render calls to a resolution, state, and intent; reduce wall time by overlapping independent resolution work within a two-worker limit; preserve all 15 cases, operation calls, assertions, reference hashes, and fail-closed gate behavior.

**Non-Goals:** Alter production rendering or raster-cache behavior, share a renderer between lifecycle states, skip equivalent-looking states, change preview/export outputs, change tolerances or fixtures, modify CI policy, or impose a runtime budget.

## Decisions

1. Factor the existing per-resolution loop into one helper that retains its current five-state order. Run the 1920x1080 resolution in one scoped worker and the 960x540 then 1280x720 resolutions in the other when at least two workers are available; otherwise run all three serially. This limits simultaneous renders to two and balances the high-resolution case against the two lower-resolution cases. The fixtures and renderers remain separate, and scoped-thread panic propagation keeps failures visible. Parallelizing lifecycle states was rejected because it would change the edit/undo/redo/reopen sequence. Three concurrent resolutions were rejected pending memory evidence. Moving sizes into separate CI jobs would change the protected workflow and is outside this change.
2. Measure with monotonic `Instant` around each existing frame-preview, range-preview, and export call. Emit labeled, non-negative elapsed values after successful operations, plus per-state and per-resolution totals. Preserve the existing suite-level timing in `golden.rs`. Counts and labels must cover 45 frame previews, 15 ranges, and 15 exports. Timing output is diagnostic only. Reusing rendered artifacts or a shared renderer was rejected because cold/warm-cache and lifecycle coverage would change.
3. Keep all existing comparison calls, reference data, thresholds, and report validation intact. The two workers return only after their full resolution groups finish; any panic or missing case fails the golden test. Compare the Linux report schema and deterministic outputs with the previous passing run, and inspect the new timings for a material wall-time improvement. If concurrency causes instability or no material improvement, revise the approach before archiving.

## Risks / Trade-offs

- [Competing FFmpeg processes increase CPU or memory pressure] → Limit to two workers, retain serial fallback, and require the full Linux Render parity run plus all platform correctness checks before archival. Reject the change if runtime or resource behavior is worse.
- [A worker failure is hidden by the other branch] → Use scoped threads and propagate every join panic; never convert failed operations into a timing-only result.
- [Timing changes execution order or output] → Record only monotonic elapsed values around unchanged calls; compare reviewed references and strict report output under the required gate.
- [Runner variation obscures the gain] → Report both observed baselines and the new per-intent timings; do not turn timing into a universal threshold or claim a stable reduction from one run.

## Migration Plan

No public, persisted, schema, or contract migration is needed. The change is test-harness-only and can be reverted without touching reference data or production rendering.

## Open Questions

None before implementation; CI timings will show whether a further production-renderer optimization needs its own change.
