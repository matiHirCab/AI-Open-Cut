# Desktop review surfaces

## Purpose
Provide authoritative scoped scene inspection and native frame/audiovisual review controls in the desktop.

## Requirements

### Requirement: Consolidated authoritative desktop inspection
Desktop MUST offer Layer, Cues and Audio inspector tabs over the same authoritative snapshot and full scoped selection. Layer MUST retain existing hierarchy/compositing/animation edits, Cues MUST retain bounded scope/page/marker inspection without requiring item selection, and Audio MUST retain bounded saved-alignment and authored selected-media/bus/master inspection. Tab changes MUST clear hidden edit drafts without publishing a revision or conflating repeated component selections. Existing core edits, error codes, history and reopen semantics MUST remain unchanged.

#### Scenario: R1 Inspect agent-authored scene details
- **WHEN** a loaded project contains hierarchy, markers, masks/effects and audio settings and users switch tabs and scoped selections
- **THEN** the appropriate existing details remain accessible and truthful, hidden drafts are discarded and inspection does not mutate project state

### Requirement: Core-backed revisioned desktop review controls
Desktop MUST expose frame time, range start/end, 540p/720p/project resolution and include-audio controls with explicit milliseconds, project fps and captured revision. Preview execution MUST delegate revision, project/path, range and rendering semantics to existing core facades and run outside the GUI callback. The shell MUST admit only one editor/review task at a time and refuse ordinary window close while work is active, leaving its producer alive until completion. Forced process termination is outside that guarantee. Successful frame review MUST display the generated frame, and successful range review MUST show its captured settings, relative artifact path and warnings with an explicit native-file reveal action. It MUST label external playback and retained stale revisions truthfully and MUST NOT expose arbitrary paths/network inputs, export controls or a second renderer.

#### Scenario: R2 Render and inspect native frame and audio review
- **WHEN** valid controls request review of the displayed revision
- **THEN** native frame and range artifacts are produced through core, the frame is visible, MP4 metadata/reveal is accessible, and authoritative project content/history/revision remain unchanged

#### Scenario: R3 Reject invalid missing or stale review input
- **WHEN** representation parsing fails or core rejects a missing project, stale revision, invalid time/range or unavailable renderer
- **THEN** parsing feedback or the existing core code/message/retryability is visible, no successful artifact is published and project/history remain unchanged

Review metadata MUST remain window-local and reset in a newly opened shell; reopening MUST load exact authoritative content without claiming artifact persistence.

#### Scenario: R4 Preserve captured review identity across history and reopen
- **WHEN** edits, refresh, undo/redo or reopen change the displayed revision
- **THEN** retained metadata in the existing shell keeps its captured identity and indicates staleness, controls use fresh displayed revisions, a new shell starts without retained metadata and authoritative restored/reopened content remains exact

### Requirement: Demonstrated desktop review workflow
The change MUST build and exercise the actual desktop inspector, timeline and review controls against existing deterministic fixture content. Automated tests MUST supplement genuine GUI observations. Unavailable GUI or required native media checks MUST remain explicit acceptance blockers; no placeholder or screenshot of a nonrendering window MAY count as passing review evidence.

#### Scenario: R5 Exercise actual GUI and native media
- **WHEN** the built desktop is used for scoped inspection, tab switching, preview success/failure, edits, history and refresh/reopen
- **THEN** commands, exact observed values and screenshots identify performed interactions and limitations while #70's synthetic recipe remains unchanged

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
