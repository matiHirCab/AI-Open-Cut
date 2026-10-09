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
