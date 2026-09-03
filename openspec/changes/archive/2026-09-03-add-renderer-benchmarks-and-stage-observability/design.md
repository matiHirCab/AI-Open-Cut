## Context

Issue #14 established a deterministic golden fixture and schema-2 performance report. The harness currently times evaluation, filter construction, complete frame/range/export calls, total capture, and peak resident memory. Decode, composition, and encoding occur inside one FFmpeg process, while the current renderer has no separate native vector/text raster backend. Relabeling the existing whole-render duration would be misleading, and subtracting noisy nested durations could produce invalid negative observations.

Issue #15 needs comparable evidence without creating a transport surface or a second scene model. The benchmark must therefore stay beside the editor-core golden harness, use production evaluator/resource/plan ownership, and state precisely which measurements overlap.

## Goals / Non-Goals

**Goals:**

- Produce finite, non-negative measurements for evaluation, rasterization, filter construction, decoding, compositing, encoding, and peak process-tree memory.
- Attribute every timing to a named, documented workload and render intent.
- Exercise frame preview, audiovisual range preview, and export from one immutable evaluated scene and the same production plans used by conformance rendering.
- Keep reports comparable only when fixture, environment, tool, font, sampling, stage-definition, and intent identity match.
- Preserve stable failures, revisions, history, deterministic reopen behavior, path safety, and atomic golden publication.

**Non-Goals:**

- No production-facing telemetry callback, public request/response, capability, or persisted field.
- No universal budget or CI timing threshold.
- No claim that independently timed FFmpeg workloads sum to end-to-end duration.
- No synthetic implementation of future masks, vectors, groups, components, or rich-text behavior merely to populate a benchmark.

## Decisions

### Keep observation private to editor-core

Introduce a crate-private typed stage observation model used by renderer tests and the golden harness. Production entry points continue to return the same artifacts, errors, progress, and warnings. The observer records durations and work counts only at owning boundaries; outer layers receive no new fields and do not reproduce validation or timeline rules.

This avoids an unnecessary cross-language contract and leaves a deliberate future decision about user-addressable progress and diagnostics.

### Define direct workloads instead of derived residuals

Evaluation measures `evaluate_project`. Rasterization measures the renderer-owned graphics-resource preparation boundary and carries a work-item count; until a native raster backend produces layers, it records zero duration with zero work rather than counting FFmpeg `drawtext` as native rasterization. Filter construction measures production resource preparation plus `build_render_plan` and filter-script creation, excluding evaluation.

Decode, composite, and encode are direct benchmark workloads derived from the same immutable production `RenderPlan`:

- decode opens every declared media input for its production source interval and writes decoded streams to null without filters or an encoder;
- composite executes the production filter graph and writes mapped uncompressed video/audio to null;
- encode executes the normal production output command and publishes only the ordinary conformance artifact.

Each workload is timed independently. Decode and composite may repeat work that also occurs during encode, so the report marks stage timings as non-additive and retains end-to-end intent duration separately. No timing is calculated by subtraction or clamped after measurement.

The benchmark-only FFmpeg commands are built from typed `RenderPlan` data in the process owner. They accept no caller expressions, paths, or network inputs. Tests compare their input ordering, source intervals, filter script, maps, and intent bounds with the production command.

### Use a strict schema-3 observation matrix

The performance report advances from schema 2 to schema 3. It retains fixture/Git/platform/tool/font identity, units, one warm-up, three measured samples, median timing aggregation, maximum memory aggregation, and process-tree memory scope. It adds a stage-definition version, `nonAdditive` timing semantics, per-intent observations for frame/range/export, the six named timing fields, end-to-end duration, and work counts for decoded inputs, rasterized layers, composited layers, encoded video streams, and encoded audio streams.

Strict deserialization rejects unknown fields or versions. Validation rejects non-finite or negative timing, inconsistent intent sets, duplicate stages, missing units or aggregation metadata, impossible work counts, and nonzero raster duration with zero raster work. Peak memory remains the maximum sampled aggregate resident set of the harness and recursively discovered FFmpeg/FFprobe descendants.

The golden fixture revision advances with the checked-in schema-3 report. Existing project data and retained histories do not migrate because this is a test-fixture format, not the project schema. Golden update mode stages and validates the complete generation and atomically changes `CURRENT`, preserving the existing rollback/reopen guarantees.

### Preserve deterministic conformance and lifecycle evidence

Stage benchmarking is opt-in report capture layered on the existing golden invocation. The measured runs must still agree on semantic plan, normalized filter graph, decoded frames/audio, and timing tolerance before a report is accepted. Benchmark probes never publish outputs and use the same bounded temporary workspace cleanup.

Unit tests inject a recording process owner to prove ordering and early termination: invalid input, missing references, and stale revisions produce no downstream render-stage observations or artifacts. Existing native lifecycle coverage continues to prove successful render, undo, redo, draft isolation, and deterministic reopen; the report records stable scene/plan identity rather than mutating state.

### Keep the benchmark matrix honest about implemented features

The canonical milestone-zero scene remains the required portable benchmark because it is the only reviewed fixture spanning visual layers, animated text, media/audio, frame, range, and export in current runtime semantics. The report contains separate intent observations, which are the renderer benchmarks required by this change. Roadmap scenes for static rules, masked reveals, and later rich motion graphics are added only when their canonical runtime features exist; placeholders or renderer-specific approximations would violate the evaluator ownership rule.

## Risks / Trade-offs

- [Independent FFmpeg workloads increase capture time] -> Keep capture opt-in, retain one warm-up and three measured samples, and use the existing one-second 160x90 fixture.
- [Stage timings overlap] -> Mark them non-additive in machine-readable metadata and documentation; retain end-to-end duration as the latency measure.
- [A zero raster stage can be misread as missing data] -> Require an explicit zero work count and document that current FFmpeg text processing belongs to composite workload; reject omitted stages.
- [Benchmark-only commands could drift from production planning] -> Build them in the process owner from `RenderPlan` and test argument parity for inputs, intervals, filter graph, mappings, and bounds.
- [Platform/tool variance can dominate small scenes] -> Compare only identical observation identities and do not add budgets in this change.

## Migration Plan

Capture and atomically install a new immutable golden generation whose manifest points to the schema-3 performance report. Keep project schema and public contracts unchanged. Rollback selects the prior complete golden generation in version control and removes the additive instrumentation/tests; no user project or history requires conversion.

## Open Questions

None. Public renderer telemetry and performance budgets remain explicitly deferred.
