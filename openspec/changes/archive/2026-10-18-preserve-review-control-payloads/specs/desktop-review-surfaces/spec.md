## MODIFIED Requirements

### Requirement: Embedded captured-range playback
Desktop MUST offer play/pause, bounded seek, artifact-relative and captured-project time display and looping of an existing completed range. Playback MUST consume only the internally captured core artifact, preserve original dimensions/fps/audio/project/revision metadata and stale labels, and MUST NOT render, mutate project/history, accept arbitrary paths/URLs or reinterpret an old interval as current. Common controller admission MUST preserve typed play/pause and loop values, and production desktop builds MUST pass strict warnings on supported and unsupported platforms. The first supported backend SHALL be installed Linux libmpv 2 with software-render API; missing backend, unsupported platform, absent media or codec/audio failure MUST be explicit and retain the existing external-reveal fallback without claiming playback success.

#### Scenario: Play seek pause and loop a completed review
- **WHEN** a supported backend opens a completed range and users play, pause, seek or enable loop
- **THEN** the embedded image and actual audio follow the captured artifact, time remains within its interval with 200ms verification tolerance, pause stops advancement and loop restarts the same interval

#### Scenario: Preserve captured identity after history changes
- **WHEN** edits, refresh or undo/redo advance the displayed revision while a review is retained
- **THEN** playback and all metadata retain the captured revision/interval and indicate staleness without changing authoritative project bytes

#### Scenario: Report unavailable or failed playback truthfully
- **WHEN** the platform/backend is unsupported, a file is missing/evicted or nonregular, or a codec/audio load fails
- **THEN** the panel shows a fixed truthful playback error, offers external reveal and publishes no replacement render or project mutation

#### Scenario: Preserve control values without a compiled native backend
- **WHEN** desktop is compiled with strict warnings on a platform without the Linux backend, or an injected controller backend receives play/pause and loop changes
- **THEN** production compilation succeeds without warning suppression and the controller preserves both boolean directions while keeping seek clamped and the platform fallback explicit
