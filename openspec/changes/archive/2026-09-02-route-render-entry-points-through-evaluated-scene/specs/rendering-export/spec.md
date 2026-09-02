## ADDED Requirements

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
