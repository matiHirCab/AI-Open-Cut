## Why

The current native parity test proves preview and export agree with one another, but it does not preserve a reviewed reference result or report the filter-graph, timing, and memory baseline needed to detect renderer drift across later motion-graphics milestones. Issue #14 needs a deterministic, reviewable baseline before scene-graph and graphics features expand the rendering surface.

## What Changes

- Add one canonical synthetic render-regression fixture owned by editor-core and use it to generate deterministic still-frame, short audiovisual, and normalized filter-graph evidence.
- Check in portable golden manifests and compact deterministic visual/audio reference data, while keeping encoded containers and machine-specific temporary output out of version control.
- Add comparison tooling that enforces the existing SSIM, decoded float-PCM RMS, and one-frame timing tolerances and fails closed when required tools, fonts, fixture metadata, or references are invalid.
- Add an opt-in baseline capture command that records tool/platform identity, phase timings, total render duration, and peak working-set memory without turning the first measurements into release budgets.
- Exercise repeated rendering plus invalid-input and missing-reference failures without mutating project state, revisions, retained history, drafts, or output artifacts.
- Run deterministic golden conformance in Linux CI with explicit FFmpeg, FFprobe, and font configuration; keep performance capture report-only because timing and memory are platform-sensitive.
- Document fixture provenance, normalization, update/review procedure, tolerances, platform scope, and the distinction between golden conformance and performance baselines.
- Non-goals: adding motion-graphics project fields or operations, changing render semantics, exposing a new transport or capability, changing schema version or migrations, accepting renderer expressions or external resources, and setting performance pass/fail budgets.
- Compatibility: this is additive test, fixture, CI, and documentation infrastructure; it does not change public, persisted, provider, error, headless, MCP, or capability contracts and introduces no breaking change.

## Capabilities

### New Capabilities

- `render-regression-fixtures`: Defines canonical visual, audiovisual, filter-graph, timing, and memory baseline capture and deterministic conformance behavior.

### Modified Capabilities

- `rendering-export`: Requires the canonical golden suite to exercise the shared EvaluatedScene preview/range/export path against reviewed visual, audio, timing, and filter-graph references.

## Impact

- Affects editor-core render test support and integration tests, checked-in fixture data, a baseline capture utility, Linux CI rendering checks, and renderer verification documentation.
- Reuses the existing editor-core evaluator, render planner, renderer entry points, stable errors, and path-safe preparation rather than creating parallel project or timeline semantics.
- Requires only the rendering dependencies already installed for native CI (FFmpeg, FFprobe, and the deterministic DejaVu Sans font); no runtime dependency or public contract changes are expected.
