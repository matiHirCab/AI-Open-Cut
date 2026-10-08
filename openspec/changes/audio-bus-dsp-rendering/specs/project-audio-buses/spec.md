## MODIFIED Requirements

### Requirement: Bounded built-in project bus routing
Schema39 and later projects SHALL contain audioBuses in stable voiceover, music, sfx, master order with exactly those four distinct IDs. Each closed record MUST contain id and nullable outputBusId; schema39..41 MUST forbid DSP fields and schema42 MAY include only the optional nonnull normalized dsp defined by audio-bus-dsp. Master MUST have null output and every other bus MUST route to an existing different bus through an acyclic path of at most four nodes ending at master. Initial/default buses SHALL route each stem directly to master. Root/component audio or video tracks SHALL accept optional nullable audioBusId; absent/null SHALL use audioRole fallback voiceover/music/sound_effects to voiceover/music/sfx and unassigned to master. Explicit routing SHALL override fallback routing without modifying audioRole, ducking, items or existing settings. Overlay/caption tracks MUST reject non-null routing. Existing IDs are references, not paths, code, expressions or resources; unknown buses and invalid graphs MUST fail with non-retryable INVALID_ARGUMENT. Custom bus creation/deletion and unrecognized bus fields MUST fail closed; schema42 DSP does not change routing identities/order/graph limits.

#### Scenario: Preserve defaults and resolve an explicit route
- **WHEN** a new project or a migrated track has no explicit routing, then a valid audio/video track is assigned an existing bus
- **THEN** default role routing is retained until the assignment, which resolves through the bounded project route while all prior track values remain exact

#### Scenario: Reject malformed bus models
- **WHEN** input has missing/duplicate/extra/out-of-order buses, incomplete/unknown record fields, nonterminal master, absent stem output, missing output reference, self route or cycle
- **THEN** canonical core validation rejects it unchanged with the existing typed non-retryable error

#### Scenario: Validate component and silent track records
- **WHEN** any root/component track, including empty/hidden/muted content, references an unknown bus or an ineligible overlay/caption track has explicit routing
- **THEN** complete-candidate validation rejects before publication and omission retains existing compatibility


### Requirement: Audio routing preserves existing render semantics
Bus routing alone with absent/neutral/unreachable DSP MUST preserve all existing item volume/fades/channels, role-based ducking, source/timeline timing, resources, normalized filter graphs, semantic plans, pixels and decoded audio. Equivalent legacy/default-routed/explicitly-routed fixtures SHALL remain deterministic across frame preview, audiovisual preview, draft preview and final export within existing SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing bounds. Historical schema39..41 routing capability SHALL continue to describe model/routing support only. Approved schema42 bus DSP SHALL use the separately governed canonical DSP semantics; explicit-bus side-chain ducking MUST remain unadvertised until its later approved issue.

#### Scenario: Compare default and rerouted native media
- **WHEN** a fixed audio/visual fixture is migrated and its bus routes are changed without other edits
- **THEN** canonical evaluated plans/filter graphs remain exact and decoded preview/export output satisfies the original required native oracles

#### Scenario: Preserve old role ducking despite explicit routing
- **WHEN** a legacy music/voiceover track receives explicit bus routing
- **THEN** its established role-based ducking and audio result remain unchanged through this issue

