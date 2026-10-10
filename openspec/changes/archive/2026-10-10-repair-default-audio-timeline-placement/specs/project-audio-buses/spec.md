## MODIFIED Requirements

### Requirement: Audio routing preserves existing render semantics
Bus routing alone with absent/neutral/unreachable DSP and absent/disabled/identity/inactive explicit ducking MUST preserve all existing item volume/fades/channels, role-based ducking, source/timeline timing, resources, normalized filter graphs, semantic plans, pixels and decoded audio. Equivalent legacy/default-routed/explicitly-routed fixtures SHALL remain deterministic across frame preview, audiovisual preview, draft preview and final export within existing SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing bounds. Historical schema39..41 routing capability SHALL continue to describe model/routing support only. Approved schema42 bus DSP SHALL use the separately governed canonical DSP semantics; Approved schema43 narration clip-activity ducking SHALL use audio-bus-ducking semantics without changing saved role settings; old routing and inactive controls remain identity. The approved rendering-export ordinary-audio sample-placement correction SHALL supersede exact legacy graph/PCM preservation solely for erroneous timestamp-only clip placement; absent, identity and active processing MUST retain the same authored placement, and every unrelated compatibility guarantee MUST remain unchanged.

#### Scenario: Compare default and rerouted native media
- **WHEN** a fixed audio/visual fixture is migrated and its bus routes are changed without other edits
- **THEN** canonical evaluated plans/filter graphs remain exact and decoded preview/export output satisfies the original required native oracles

#### Scenario: Preserve old role ducking despite explicit routing
- **WHEN** a legacy music/voiceover track receives explicit bus routing
- **THEN** its established role-based ducking and audio result remain unchanged through this issue
