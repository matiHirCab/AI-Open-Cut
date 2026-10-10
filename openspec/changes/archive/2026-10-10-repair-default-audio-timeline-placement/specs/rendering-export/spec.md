## ADDED Requirements

### Requirement: Canonical ordinary audio sample placement
Every audiovisual render intent MUST physically place ordinary evaluated audio at its authored timeline span before sequential mixing, independently of absent, explicit identity, unreachable or active bus DSP and default or explicit routing. Placement SHALL preserve zero-start content, leading silence, inter-clip gaps, simultaneous overlaps, source trim, item-local fades/automation and globally clocked role ducking. Existing retained and component-mapped clocks MUST NOT receive duplicate delay. This narrowly approved defect correction supersedes exact legacy graph/PCM preservation only where the timestamp-only ordinary mixing path contradicts authored sample placement; all other compatibility and failure guarantees remain unchanged.

#### Scenario: Place a delayed ordinary clip independently of processing
- **WHEN** a500ms tone is authored at2000ms with absent, explicit identity or gainDb=-1 bus DSP
- **THEN** actual decoded output is silent at0.1–0.3seconds and contains the independently specified tone at2.1–2.3seconds, with processing affecting amplitude only as intended

#### Scenario: Preserve gaps overlaps trims and local controls
- **WHEN** multiple ordinary clips have zero/nonzero start, gaps, consecutive boundaries, overlaps, nonzero source-in, fades and local volume/gain automation
- **THEN** decoded PCM follows independent literal timeline/source/control expectations without concatenation, early playback, lost gaps or duplicate delay

#### Scenario: Preserve routing and global ducking clocks
- **WHEN** roles and default/explicit stem/master routes use neutral or active DSP with existing role ducking
- **THEN** timing remains authored and role ducking samples the canonical global clock while local controls retain their source clock

#### Scenario: Preserve preview selection and existing mapped clocks
- **WHEN** audiovisual preview begins inside a clip or gap and the same scene is exported, including retained/component-mapped audio
- **THEN** the shared interval retains the same content within existing decoded-PCM and timing bounds without double delay

#### Scenario: Refuse missing physical-delay support
- **WHEN** the configured FFmpeg lacks the base physical-delay filter required for authored audio placement
- **THEN** base readiness and rendering fail with DEPENDENCY_UNAVAILABLE before destination publication, while project editing remains available and no timestamp-only degraded output is emitted

### Requirement: Independent timing repair and lifecycle evidence
The ordinary-audio correction MUST have mandatory native actual H264/AAC export and audiovisual preview tests with decoded float-PCM expectations independent of emitted graphs. Public/persisted contracts, project/history immutability, canonical typed failure precedence, cleanup and retry MUST remain unchanged. Historical audio/graph witnesses SHALL remain intact; any impacted selected golden replacement MUST receive explicit reviewer approval of concrete old/new evidence and use its existing atomic publication mechanism without weakening gates or tolerances.

#### Scenario: Exercise history failures and retry
- **WHEN** timing edits are undone/redone/reopened or stale/missing/unavailable inputs, render failure/cancellation and a subsequent valid retry are exercised
- **THEN** authored timing is restored, original typed errors occur, rejected rendering leaves state and existing destinations unchanged, partial output is cleaned and valid retry publishes only a complete artifact

#### Scenario: Verify independent evidence and protected completion
- **WHEN** the repair is evaluated for completion
- **THEN** independent sample-clock/native regressions and all mandatory local/CI gates pass on the final implementation, specifications are verified synchronized and archived, and any approved golden change retains historical witnesses and complete provenance
