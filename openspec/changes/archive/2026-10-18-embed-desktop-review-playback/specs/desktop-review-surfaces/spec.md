## ADDED Requirements

### Requirement: Embedded captured-range playback
Desktop MUST offer play/pause, bounded seek, artifact-relative and captured-project time display and looping of an existing completed range. Playback MUST consume only the internally captured core artifact, preserve original dimensions/fps/audio/project/revision metadata and stale labels, and MUST NOT render, mutate project/history, accept arbitrary paths/URLs or reinterpret an old interval as current. The first supported backend SHALL be installed Linux libmpv 2 with software-render API; missing backend, unsupported platform, absent media or codec/audio failure MUST be explicit and retain the existing external-reveal fallback without claiming playback success.

#### Scenario: Play seek pause and loop a completed review
- **WHEN** a supported backend opens a completed range and users play, pause, seek or enable loop
- **THEN** the embedded image and actual audio follow the captured artifact, time remains within its interval with 200ms verification tolerance, pause stops advancement and loop restarts the same interval

#### Scenario: Preserve captured identity after history changes
- **WHEN** edits, refresh or undo/redo advance the displayed revision while a review is retained
- **THEN** playback and all metadata retain the captured revision/interval and indicate staleness without changing authoritative project bytes

#### Scenario: Report unavailable or failed playback truthfully
- **WHEN** the platform/backend is unsupported, a file is missing/evicted or nonregular, or a codec/audio load fails
- **THEN** the panel shows a fixed truthful playback error, offers external reveal and publishes no replacement render or project mutation

### Requirement: Bounded owned playback lifecycle
Each player MUST own and release its native context, worker and presentation resources. Replacement, repeated load, navigation and window close MUST stop the old player; stale generations MUST NOT publish into a newer player. Commands and latest-frame storage MUST remain bounded, display frames SHALL fit 1280x720 preserving aspect, and old GPUI decoded assets/atlas entries MUST be evicted. Playback MUST disable external configuration/scripts, arbitrary protocol/network loading and automatic external-track discovery, without changing system security configuration.

#### Scenario: Replace and close repeatedly
- **WHEN** users replace a review, repeat open/stop, navigate away or close its window
- **THEN** old admission stops, the worker/context/resources are released in ownership order, pending frames do not accumulate and late old-generation results cannot replace current playback

#### Scenario: Demonstrate supported runtime and GUI conformance
- **WHEN** verification runs the supported backend on original synthetic video/audio
- **THEN** automated controller/native tests and actual GUI interactions demonstrate pixels, audio, timing, seek/loop, errors, stale labels and cleanup; missing required runtime/GUI evidence blocks completion rather than counting as a skip
