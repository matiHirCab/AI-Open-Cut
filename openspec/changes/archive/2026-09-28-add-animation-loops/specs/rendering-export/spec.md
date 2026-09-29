## ADDED Requirements

### Requirement: Shared loop evaluation across render intents
Frame preview, audiovisual range preview, materialized draft preview, and final export MUST consume one editor-core evaluated-scene loop phase at equivalent absolute composition timestamps for active visual channels and audio gain. A range starting inside a later cycle MUST match a frame or export covering that timestamp; range start MUST not reset phase. Semantic plans MUST agree exactly and decoded outputs MUST satisfy the existing visual, audio, and timing tolerances on supported FFmpeg 6 and 8. Invalid persisted loops or non-finite evaluated work MUST fail canonical preflight before destination inspection, renderer execution, temporary files, or artifact publication. Projects without loops MUST retain existing visual/audio output.

#### Scenario: Compare later-cycle range samples
- **WHEN** a finite or infinite visual loop and an audio-gain loop are evaluated at the same timestamp by a frame, a range starting inside a later cycle, a draft, and export
- **THEN** their evaluated values and semantic plans agree and decoded output meets the established tolerances

#### Scenario: Compare exact seams
- **WHEN** repeat and ping-pong loops are rendered at and immediately around cycle seams
- **THEN** each intent uses the documented exact endpoint/phase and produces deterministic visual/audio output within existing tolerances

#### Scenario: Reject invalid persisted loops without output
- **WHEN** a persisted project contains an invalid loop or the evaluated loop work is non-finite
- **THEN** every intent returns the stable typed error before inspecting or changing output destinations or artifacts

#### Scenario: Preserve unlooped rendering
- **WHEN** a project uses only existing legacy keyframes or typed channels without loops
- **THEN** every render intent retains equivalent pre-activation visual and audio output
