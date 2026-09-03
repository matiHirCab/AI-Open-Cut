## Why

The existing `flat-scene-av-v1` golden harness reports evaluator, filter construction, whole frame/range/export duration, and peak process-tree memory. Those totals cannot show whether a renderer change moved cost into graphics rasterization, media decoding, composition, or encoding, so issue #15 and the later job-durability decision do not yet have the stage-level evidence they require.

## What Changes

- Extend the editor-core-owned render benchmark harness with explicit evaluation, rasterization, filter-construction, decoding, compositing, and encoding observations plus peak process-tree memory.
- Define stage meanings and work counters so a stage with no current native raster work is reported as zero work rather than silently omitted, and so independently measured FFmpeg workloads are never presented as additive critical-path time.
- Benchmark frame preview, audiovisual range preview, and final export from one immutable evaluated scene and production render plans, retaining the existing golden conformance checks.
- Replace the checked-in performance observation with a strict versioned report containing per-intent samples, aggregation metadata, stage work counts, and complete environment identity.
- Add deterministic success and failure-path coverage, including invalid input, missing references, revision conflicts, undo/redo, and reopen behavior where applicable.
- Document capture, interpretation, comparison, and review rules, and update the milestone plan with the resulting scope.
- Non-goals: public telemetry APIs, MCP/headless operations, project-schema changes, persisted history migration, release performance budgets, network resources, raw FFmpeg expressions, or benchmarks for motion-graphics features that are not implemented yet.
- Compatibility: additive internal observability and test/fixture schema evolution only. Public, persisted, provider, capability, stable-error, headless, and MCP contracts remain unchanged.

## Capabilities

### Modified Capabilities

- `render-regression-fixtures`: Adds stage-separated benchmark capture, strict report validation, and interpretation rules to the canonical renderer fixture.

## Impact

- Affects editor-core renderer/process instrumentation, the native golden benchmark harness and tests, the checked-in performance fixture, renderer benchmark documentation, and required Linux benchmark artifacts.
- Keeps canonical project, timeline, evaluator, path, revision, history, and render-plan rules in editor-core and derives all benchmark workloads from production `EvaluatedScene` and `RenderPlan` values.
- Requires no new runtime dependency and no public or persisted contract migration.
