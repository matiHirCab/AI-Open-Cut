## ADDED Requirements

### Requirement: Canonical range review preset selection
Editor-core SHALL resolve range review selection as `540p`, `720p`, `project`, or exact custom width and height. Omitted selection SHALL use project dimensions. The height presets SHALL use height 540 or 720 and project aspect ratio, with width rounded to the nearest even integer, ties upward, and minimum 2. Project/custom dimensions SHALL remain exact. Resolved width MUST be 1..7680, height 1..4320, and fps 1..120; omitted fps SHALL use project fps. No adapter SHALL duplicate this resolution policy.

#### Scenario: Resolve named presets across aspects
- **WHEN** a review requests 540p or 720p for landscape, square, portrait, or a half-even width tie
- **THEN** height is the selected height and width follows the defined integer aspect rounding without cropping or project mutation

#### Scenario: Preserve project and custom settings
- **WHEN** selection is project or omitted, or a complete custom dimension pair is provided
- **THEN** dimensions remain exact, omitted fps uses project fps, and explicit fps overrides project fps

#### Scenario: Reject conflicting or unsupported settings
- **WHEN** new typed resolution is malformed, resolved review size exceeds bounds, or fps is outside 1..120
- **THEN** typed resolution errors return `INVALID_ARGUMENT` and canonical bound/fps errors return `VALIDATION_FAILED` before rendering side effects and all persisted state remains unchanged

### Requirement: Audio enabled immutable review
New range-review requests SHALL enable audio on omission and honor explicit false. Range review SHALL reuse existing export EvaluatedScene semantics and audio mixing for the immutable requested revision, leave project/revision/history/drafts unchanged, and preserve missing-reference, revision-conflict, path-safety and failure-order guarantees.

#### Scenario: Review with audio or explicit silence
- **WHEN** a new range review omits includeAudio or explicitly sets false on a fixture containing audio
- **THEN** omission produces the export-equivalent audio stream and false produces no audio stream, without changing visual scene semantics

#### Scenario: Repeat and reopen immutable review
- **WHEN** a valid review is repeated and the project is reopened after completion
- **THEN** dimensions and semantic plans remain deterministic, revision/history remain unchanged, and decoded preview/export output remains within existing visual/audio/timing tolerances

#### Scenario: Preserve early failures
- **WHEN** a review supplies a stale revision or references missing media or an escaping bound path
- **THEN** the existing `REVISION_CONFLICT`, `ASSET_NOT_FOUND`, or `PATH_NOT_ALLOWED` is returned before artifact publication or render side effects
