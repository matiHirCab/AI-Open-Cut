# Project Audio Buses Specification

## Purpose

Define bounded built-in project audio routing, transactional edits and compatibility without changing rendered output.

## Requirements

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

### Requirement: Transactional alias-aware routing edits
Core SHALL expose audio_bus_set_route with busId/outputBusId and audio_track_route with scope/trackId/nullable busId as typed standalone and timeline_batch_edit operations. Scope MUST use root or component:<id> with existing component/track alias resolution. Track routing MUST reject missing tracks with TRACK_NOT_FOUND, locked tracks with TRACK_LOCKED and malformed/missing scope or bus references with existing non-retryable INVALID_ARGUMENT. Null track busId MUST clear only explicit routing and restore role fallback; bus routing MUST preserve fixed identities/order and require a valid graph after every ordered edit. Edits MUST retain optimistic retryable REVISION_CONFLICT precedence, one logical revision/history entry, atomic whole-batch rollback, deterministic draft validation/commit, exact undo/redo/reopen and existing path/resource safety. Fixed bus IDs SHALL not create resultAlias IDs.

#### Scenario: Route a created track and component through aliases
- **WHEN** an ordered batch creates a track/component with a result alias and routes the scoped track at the current revision
- **THEN** the core resolves existing aliases and commits the route once with stable records

#### Scenario: Clear a route and restore it through history
- **WHEN** a routed track is cleared with null, then undone, redone and reopened
- **THEN** fallback and explicit states alternate with exact IDs/routes and established revision increments

#### Scenario: Reject stale and late-invalid mutations
- **WHEN** a stale routing request or a batch with an earlier valid route and later missing/locked/invalid/cyclic route is submitted
- **THEN** the existing stable error is returned and complete project/history/draft/resource bytes and revision remain unchanged

#### Scenario: Route materialized drafts
- **WHEN** a valid or invalid bus/track route is prepared through existing draft batch input and committed or rejected
- **THEN** the same core validation, revision conflict and atomic ownership apply without a parallel transport validator

### Requirement: Audio routing preserves existing render semantics
Bus routing alone with absent/neutral/unreachable DSP MUST preserve all existing item volume/fades/channels, role-based ducking, source/timeline timing, resources, normalized filter graphs, semantic plans, pixels and decoded audio. Equivalent legacy/default-routed/explicitly-routed fixtures SHALL remain deterministic across frame preview, audiovisual preview, draft preview and final export within existing SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing bounds. Historical schema39..41 routing capability SHALL continue to describe model/routing support only. Approved schema42 bus DSP SHALL use the separately governed canonical DSP semantics; explicit-bus side-chain ducking MUST remain unadvertised until its later approved issue.

#### Scenario: Compare default and rerouted native media
- **WHEN** a fixed audio/visual fixture is migrated and its bus routes are changed without other edits
- **THEN** canonical evaluated plans/filter graphs remain exact and decoded preview/export output satisfies the original required native oracles

#### Scenario: Preserve old role ducking despite explicit routing
- **WHEN** a legacy music/voiceover track receives explicit bus routing
- **THEN** its established role-based ducking and audio result remain unchanged through this issue

### Requirement: Governed additive routing contracts
Protocol1 SHALL retain existing operations, response/error shapes and aliases while adding project_audio_buses_v1 capability, schema39 reporting, the two uniquely named typed routing operations and corresponding MCP tools. Rust, TypeScript/Zod, draft input/output, standalone and batch consumers MUST agree with canonical audio-buses-v1 fixtures. The seven existing active current-schema catalog headers SHALL advance exactly from38 to39 while every other byte/case remains preserved. All current and historical catalog hashes, schemas, counts and negative controls MUST retain independent predecessor evidence; historical projections SHALL remove only independently captured exact routing additions and explicitly validated schema-version transitions, rejecting unrelated drift. Existing mandatory contracts/native/integration consumers SHALL remain required; frozen counts MUST not be rewritten to hide new additions.

#### Scenario: Exercise real standalone and batch transports
- **WHEN** MCP/headless clients read schema/capability support and submit routing operations alone and in an alias-bearing batch
- **THEN** typed contracts agree and success/failure/history/reopen results match the canonical core

#### Scenario: Retain predecessor drift rejection
- **WHEN** the exact approved additions are projected or unrelated schema/annotation/operation fields are mutated
- **THEN** the independently pinned issue62 and every older proof agree for approved additions and reject unrelated mutation
