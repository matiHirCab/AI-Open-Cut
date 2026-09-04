# Rendering and Export Specification

## Purpose

Define deterministic preview and export behavior, dependency readiness, progress, and output publication safety.

## Requirements

### Requirement: Explicit renderer readiness
The renderer MUST verify the configured FFmpeg and FFprobe executables plus the required FFmpeg filters before rendering and SHALL report unavailable dependencies with stable errors.

#### Scenario: FFmpeg is unavailable
- **WHEN** the configured FFmpeg executable cannot be used
- **THEN** readiness or rendering fails with `DEPENDENCY_UNAVAILABLE` instead of starting an incomplete render

#### Scenario: Required media tooling is incomplete
- **WHEN** FFprobe is unavailable or FFmpeg lacks overlay, drawtext, or audio-mix support
- **THEN** renderer readiness fails with `DEPENDENCY_UNAVAILABLE`

### Requirement: Deterministic frame preview
The renderer SHALL produce a preview frame for a validated project revision and timestamp using the same canonical composition and animation rules used by export.

#### Scenario: Render a preview frame
- **WHEN** a caller requests a timestamp within a valid project
- **THEN** the renderer produces a managed preview artifact representing the project at that time

### Requirement: Immutable range preview
Range previews SHALL render the requested valid time span from an immutable project revision and SHALL expose bounded monotonic progress through completion.

#### Scenario: Render a preview range
- **WHEN** a caller queues a valid preview range for the current revision
- **THEN** the resulting video uses final-export rendering behavior and progress remains between zero and one

### Requirement: Safe video export
Video export MUST accept an `.mp4` destination, encode H.264 video with AAC audio in the current MVP, render to a temporary file beside the requested destination, and SHALL publish that file only after FFmpeg completes successfully.

#### Scenario: Reject an unsupported export extension
- **WHEN** an MCP client requests an export destination that does not end in `.mp4`
- **THEN** input validation rejects the request before starting a render job

#### Scenario: Complete an export
- **WHEN** FFmpeg completes a valid export successfully
- **THEN** the temporary file is renamed to the final destination and progress reaches one

#### Scenario: FFmpeg fails
- **WHEN** FFmpeg exits unsuccessfully or emits unusable progress/output
- **THEN** export fails with `FFMPEG_FAILED`, removes its temporary output, and does not publish that partial output as the destination

### Requirement: Explicit overwrite authorization
The renderer MUST NOT replace an existing export unless the caller explicitly sets the overwrite option.

#### Scenario: Reject an export collision
- **WHEN** the destination exists and overwrite is false
- **THEN** export fails with `EXPORT_EXISTS` and leaves the existing file unchanged

### Requirement: Canonical evaluated render routing
Frame preview, audiovisual range preview, draft preview, and final export MUST consume the same editor-core `EvaluatedScene` semantics for a fixed project or validated draft snapshot, dimensions, frame rate, and time interval, and downstream render planning MUST NOT inspect persisted project, track, item, transition, or asset records to reconstruct those semantics.

#### Scenario: Render all production entry points
- **WHEN** callers render a frame, audiovisual range, draft frame, and final export from equivalent immutable scene state and output settings
- **THEN** every entry point obtains timing, transforms, animation, layer ordering, text styling, transitions, media, and audio instructions from the same canonical evaluator

#### Scenario: Preserve revision conflict behavior
- **WHEN** a revisioned preview or export request supplies a stale expected revision
- **THEN** it fails with `REVISION_CONFLICT` before scene evaluation, renderer execution, or artifact publication and leaves project state and history unchanged

#### Scenario: Render a validated draft without mutation
- **WHEN** a caller previews a valid retained draft
- **THEN** the renderer evaluates the materialized draft snapshot through the same scene semantics and neither commits the draft nor changes project revision or history

### Requirement: Deterministic preview and export tolerance
Equivalent preview and export requests MUST produce exactly equivalent resolved semantic plans after accounting only for requested interval and output intent; sampled rendered output using fixed synthetic fixtures and identical dimensions and frame rate MUST achieve visual SSIM of at least `0.99`, aligned decoded float-PCM RMS error of at most `0.0001` over the shared audiovisual interval, and stream timing alignment within one output video frame.

#### Scenario: Compare frame, range, and export output
- **WHEN** a frame and audiovisual interval are sampled from preview and final export of the same immutable scene with identical output settings
- **THEN** their semantic plans match exactly and their decoded visual, audio, and timing results remain within the documented tolerance

#### Scenario: Repeat a render deterministically
- **WHEN** the same immutable scene and render request are evaluated repeatedly
- **THEN** evaluation and planning produce equal instructions and any decoded-content differences remain within the same documented tolerance

### Requirement: Fail before render side effects
Invalid input, missing references, non-finite evaluated values, binding inconsistency, canonical complexity-limit failures, and lexical or canonical project-root path escapes MUST retain their stable typed errors and MUST occur before export-collision inspection, temporary-name allocation, graphics rasterization, FFmpeg execution, temporary workspace creation, file writes, or artifact publication.

#### Scenario: Reject a canonical or symlink escape before mutation
- **WHEN** a bound media request is lexically project-relative but canonicalizes outside the project root
- **THEN** frame preview, range preview, and export return `PATH_NOT_ALLOWED` without inspecting an existing export collision or creating, writing, executing, or publishing any render artifact

#### Scenario: Reject a missing media reference
- **WHEN** any render entry point evaluates a visible media item whose asset reference is missing
- **THEN** it fails with `ASSET_NOT_FOUND`, invokes no artifact or process adapter, and publishes no partial output

#### Scenario: Reject invalid evaluated work
- **WHEN** any render entry point encounters an invalid interval, non-finite value, unsafe resource request, or exceeded evaluated-scene limit
- **THEN** it returns the canonical stable error before downstream I/O and leaves project, revision, undo/redo history, drafts, and existing destination files unchanged

### Requirement: Evaluated-scene rendering capability
When the rendering subsystem is ready and every production render entry point uses canonical evaluated-scene semantics, headless and MCP status MUST report the additive `evaluated_scene_rendering` capability in both rendering-subsystem and aggregate capability lists without changing the current protocol major version.

#### Scenario: Detect canonical rendering support
- **WHEN** a client requests status from a ready renderer
- **THEN** the typed response and MCP-validated response include `evaluated_scene_rendering` and preserve all existing capability identifiers

#### Scenario: Renderer is unavailable
- **WHEN** rendering readiness fails
- **THEN** `evaluated_scene_rendering` is absent from the rendering and aggregate capability lists and the existing readiness error remains available without exposing internal paths

### Requirement: Reviewed golden evidence for shared render routing
The canonical shared-render guarantee MUST be backed by a required native golden suite that renders a fixed non-empty EvaluatedScene through frame preview, audiovisual range preview, and final export, compares each entry point with reviewed visual, audio, timing, semantic-plan, and normalized-filter-graph references, and proves failures occur before render or publication side effects.

#### Scenario: Exercise every production output intent
- **WHEN** required native CI evaluates and renders the canonical fixture at its declared frame and interval
- **THEN** frame preview, range preview, and export all consume the same semantic scene, satisfy the documented SSIM, decoded-audio RMS, and one-frame timing tolerances, and match the reviewed semantic and filter-graph evidence

#### Scenario: Fail before fixture output on invalid work
- **WHEN** the canonical fixture is varied to contain invalid timing, a missing media reference, or a stale expected revision
- **THEN** the existing typed failure occurs before renderer execution, temporary or golden-reference writes, artifact publication, or project/history mutation

### Requirement: All render intents consume common visual properties without output changes
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST obtain transform and visibility semantics from the canonical common visual-properties value through the shared `EvaluatedScene` path. For content representable in schema version 6, migration to schema version 7 MUST NOT change evaluated instructions, active intervals, layer ordering, filter graphs, decoded pixels, audio samples, timing, or artifact publication behavior beyond existing documented tolerances.

#### Scenario: Compare pre-migration and migrated render intents
- **WHEN** the same schema-v6 fixture and its migrated schema-v7 state are rendered as a frame, audiovisual range, materialized draft preview, and final export
- **THEN** semantic plans are equal and visual, audio, and timing results satisfy the existing golden parity tolerances

#### Scenario: Keep new identity defaults non-operative
- **WHEN** a migrated caption or transition receives an identity transform solely because all variants now own common visual properties
- **THEN** evaluation and rendering preserve the schema-v6 caption or transition result and do not apply a new transform behavior

#### Scenario: Reject invalid common values before rendering
- **WHEN** common visual properties contain a non-finite, out-of-bounds, or otherwise invalid legacy transform
- **THEN** editor-core returns the existing typed validation failure before graphics preparation, filesystem path resolution, backend execution, or artifact publication
