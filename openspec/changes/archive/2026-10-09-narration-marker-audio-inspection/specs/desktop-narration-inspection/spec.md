## ADDED Requirements

### Requirement: Bounded authoritative scoped marker inspection
Desktop SHALL expose read-only root and component-definition marker inspection showing owning scope, stable ID, name, cue kind and local timeMs, with at most32 marker rows per page and total/page counts. Selected-item inspection MUST show its retained markerName/offsetMs expression and authoritative synchronized startMs without cross-scope lookup or a second timing resolver. Marker selection MUST bind scope/ID/revision; refresh, selection changes and history transitions MUST discard stale details. Empty scopes MUST remain inspectable without requiring an item selection.

#### Scenario: D1 Inspect paged root and component cues
- **WHEN** a project has root and component markers, including same-name cues and more than32 markers
- **THEN** scope selection and page navigation show at most32 exact authoritative rows, correct counts and distinct local timing without mutating the project

#### Scenario: D2 Refresh and inspect retained bindings
- **WHEN** the selected item's marker moves or the project is refreshed, undone, redone or reopened
- **THEN** the displayed expression and numeric timing agree with the authoritative state and stale scope/ID/revision details are cleared

### Requirement: Bounded truthful speech and audio inspection
Desktop SHALL show selected media's saved speech alignment presence, exact quality, alignment producer/model metadata, independent sentence/word/phoneme counts and one selected segment's exact text and asset-relative interval. Absence MUST remain absent. It MUST show authored media volume/mute/fades, semantic event snapshot and authored bus routing, and read-only bus DSP/ducking/master normalization settings in at most16 bus summaries per page. Effective values, when displayed, MUST come from existing core results and be distinguished from authored settings; unavailable results MUST be labeled unavailable. Presentation MUST avoid serializing or materializing whole alignment collections and MUST expose no managed absolute path. It MUST remain read-only and keep provider inference and domain validation in their existing owners.

#### Scenario: D3 Inspect aligned unaligned and event media
- **WHEN** selection changes among aligned narration, unaligned media and a semantic audio event
- **THEN** the panel shows exact applicable provenance/settings and absence, clears prior segment selection, and publishes no revision or resource change

#### Scenario: D4 Inspect maximum alignment and bus collections
- **WHEN** a valid project has100000 alignment segments and all four built-in buses
- **THEN** presentation materializes only one selected segment and the bounded current bus page while preserving all stored values and showing truthful counts

### Requirement: Demonstrated desktop narration workflow
The change MUST build the desktop and record actual manual GUI evidence for cue scope/page navigation, aligned/unaligned/event inspection, audio settings, refresh, undo/redo and reopen against the generated narration fixture. Automated presentation/session tests MUST supplement this evidence; unavailable GUI execution MUST remain an explicit acceptance blocker.

#### Scenario: D5 Exercise actual narration inspection
- **WHEN** the generated project is opened in the built desktop and the required inspection/history workflow is performed
- **THEN** observations and commands identify exact displayed state and limitations, with no passing claim for missing manual evidence
