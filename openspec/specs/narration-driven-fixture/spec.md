# narration-driven-fixture

## Purpose

Demonstrate the existing narration authoring and rendering contracts with a deterministic synthetic fixture and independent conformance evidence.

## Requirements

### Requirement: Deterministic bounded narration recipe
The suite SHALL provide a versioned synthetic6000ms narration recipe with six sentence cues EVERY at500ms, SINGLE at1000ms, ONE at1500ms, rules at2400ms, Starting with at3200ms and Venusaur at4300ms. Sentence ends SHALL be900,1400,1900,2900,3900 and5000ms respectively. Words SHALL be independently declared, splitting Starting and with within its sentence; phonemes SHALL be empty. Quality MUST remain estimated with explicit synthetic alignment producer identity and required nullable model metadata, without claiming measured or inferred speech. Canonical marker names SHALL be EVERY, SINGLE, ONE, rules, Starting_with and Venusaur. Every cue MUST bind an existing visual preset and a deterministic registered semantic audio event through existing marker expressions, with explicit literal expected effective timing, selected variant/content/gain/bus evidence. The recipe MUST exercise narration activity ducking and enabled master normalization while keeping sources local, finite and bounded.

#### Scenario: N1 Generate the integrated narration example
- **WHEN** the recipe is authored in a fresh store through existing typed core operations
- **THEN** all six markers, visual preset attributions and event snapshots agree with literal expected timing/content and saved audio controls, without network inference or new persisted fields

### Requirement: Saved and explicit alignment compatibility
The fixture SHALL exercise speech_markers_generate using omitted alignment on an asset with saved speech provenance and explicit alignment on an otherwise unaligned audio-bearing asset. Both paths MUST produce the same cue names/times under sentence policy with the same placement offset and preserve exact quality/producer/independent granularity data. Selected-word standalone and batch generation, including a one-result alias, MUST also retain the existing API. Worker contract, protocol1, schema44 and existing operation/error/capability catalogs MUST remain unchanged.

#### Scenario: N2 Compare saved and explicit alignment
- **WHEN** identical declared alignment is consumed through saved provenance and the explicit optional alignment field in fresh equivalent scopes
- **THEN** generated cue names/times agree, saved provenance survives undo/redo/reopen exactly, and the explicit path does not fabricate persisted synthesis provenance

#### Scenario: N3 Preserve alignment validation failures
- **WHEN** null/malformed alignment, overlap/out-of-duration segments, missing audio source, unsupported media, missing scope or invalid word selection is submitted
- **THEN** existing decoding or core typed failures/retryability remain unchanged and no project/history/revision/managed-resource bytes are published

### Requirement: Existing atomic narration authoring lifecycle
The integrated example MUST use existing standalone and ordered timeline_batch_edit operations with prior creation aliases. Stale revisions, missing/ambiguous cue names, missing event definitions, forward aliases and late invalid operations MUST preserve existing error precedence and complete transaction/resource rollback. Marker movement MUST update both bound visuals and events in core; undo/redo/reopen MUST preserve exact IDs, expressions, preset metadata, captured event/audio controls and saved alignment. No migration SHALL be introduced; existing schema44 current and retained history MUST remain readable unchanged.

#### Scenario: N4 Reject and restore integrated authoring
- **WHEN** standalone/batch operations succeed, conflict or fail late, then valid edits are undone, redone and reopened
- **THEN** literal expected complete authoritative state/history/resources and typed errors agree, aliases resolve only in order and no partial narration scene escapes rejection

#### Scenario: N5 Move the narration cue
- **WHEN** a cue time is updated within valid bounds or an attempted update makes dependent intervals invalid
- **THEN** valid visual/event starts move together under core evaluation and invalid changes preserve the complete previous generation

### Requirement: Independent shared native narration evidence
The fixture SHALL prove exact semantic-plan parity for equivalent full/range/draft preview and export selections using the shared evaluated scene. Native visual references MUST cover cue boundaries, preset animation and compositing; audio references MUST cover event placement, bus routing and narration ducking with independently declared inputs/equations. Existing SSIM>=0.99, aligned decoded float-PCM RMS<=0.0001 and one-output-frame timing tolerances MUST remain unchanged. Enabled normalization MUST additionally satisfy existing complete-root precodec delivered target/ceiling guarantees using independent measurement, with crop/codec limitations documented. Current implementation output MUST NOT be used to manufacture expected references. Original frozen contracts and render suites MUST remain required.

#### Scenario: N6 Compare cue boundaries and native media
- **WHEN** the fixed recipe is rendered around all six cue boundaries through preview/draft/export and normalization is measured on complete-root precodec output
- **THEN** exact semantic and independent visual/audio/timing evidence meets the existing thresholds and target/ceiling guarantees without weakening predecessor assertions

#### Scenario: N7 Preserve independent failure detection
- **WHEN** a negative control shifts an event/cue, changes captured routing/gain or violates normalization evidence
- **THEN** the appropriate independent oracle fails and no failing or unavailable evidence is reported as successful

### Requirement: Required integrated native narration gate
Configured required native CI SHALL invoke the existing integrated six-cue narration oracle with actual FFmpeg, FFprobe and the declared font, preserving all predecessor commands, independent negative controls and thresholds. The CI policy MUST reject omission or failure masking of that exact integrated invocation.

#### Scenario: Enforce integrated narration evidence
- **WHEN** required native CI runs with configured tools
- **THEN** integrated cues, presets, captured events, ducking and normalized preview/export execute the existing independent oracle

#### Scenario: Reject missing or masked integrated evidence
- **WHEN** the configured integrated narration command is removed or its failure is masked
- **THEN** policy validation fails instead of attesting complete native coverage
