## ADDED Requirements

### Requirement: Bounded named content-addressed sound library
Schema40 projects SHALL contain a required nonnull soundDefinitions array, initially empty, of at most512 unique case-sensitive named closed records. Each record MUST require event, ordered variantAssetIds, defaultGainDb, busId and variantSeed. Names SHALL use [A-Za-z][A-Za-z0-9_-]{0,127}; variant references MUST use bounded ASCII [A-Za-z0-9][A-Za-z0-9_-]{0,127}, admitting UUIDs and resolving existing batch aliases before that check; variants MUST contain1..32 distinct existing asset IDs with distinct canonical sha256 content hashes and positive recorded sizes, resolving to audio or audio-bearing video in the containing snapshot's canonical managed asset catalog. Gain MUST be finite within inclusive[-120,24]dB, seed MUST be an integer within[0,9007199254740991], and bus MUST reference a declared built-in project bus. Extra fields, unknown buses, empty/duplicate/oversized/invalid definitions, non-audio/unhashed variants and arbitrary path/URL inputs MUST fail closed with existing nonretryable INVALID_ARGUMENT; a missing registration asset MUST use ASSET_NOT_FOUND. Existing asset policy SHALL own real managed-byte/path integrity.

#### Scenario: Register a bounded valid definition
- **WHEN** a valid named definition references ordered managed audio variants and an existing bus
- **THEN** its exact name/order/gain/bus/seed are stored without altering the referenced assets or existing timeline values

#### Scenario: Reject invalid bounds and references
- **WHEN** a definition or complete registry has malformed/duplicate/extra identities, empty/too many/duplicate-ID/duplicate-content variants, ineligible/missing/unhashed media, invalid gain/seed or unknown bus
- **THEN** canonical owning-layer validation returns the specified stable error before publication, preserving complete bytes/resources

#### Scenario: Select deterministic variants
- **WHEN** canonical core selection resolves a registered event with its saved seed or an explicit safe seed
- **THEN** it chooses ordered index seed modulo variant count with the same asset hash/gain/bus on every reopen, rejects unsafe seeds and returns INVALID_ARGUMENT for an unknown definition

### Requirement: Atomic alias-aware sound registration
Core SHALL expose sound_event_register with all definition fields as a typed protocol1 standalone and timeline_batch_edit operation and durable-draft input. Registration SHALL append a new named identity or replace that identity at its original stable index, return exactly that identity through existing WriteResult and support resultAlias using existing ordered alias resolution. It MUST preserve optimistic retryable REVISION_CONFLICT precedence, existing VALIDATION_FAILED alias errors, one logical revision/history entry, whole-batch rollback, materialized-draft validation/commit and exact undo/redo/reopen. Every registration-bearing standalone/batch/draft path MUST use existing prepared migration/resource transactions so failed requests cannot publish incidental adoption or media copies.

#### Scenario: Create replace and alias a named event
- **WHEN** valid registrations create a name then replace it, including a later ordered alias reference at the current revision
- **THEN** one stable named record and ordered variants are committed at its stable index, aliases identify the same name and exact history/reopen restores each version

#### Scenario: Reject stale and late-invalid registrations
- **WHEN** a stale registration or an ordered batch with an earlier valid registration and later invalid/missing reference is submitted
- **THEN** the specified stable error is returned and complete current/history/draft/resource bytes and revision remain unchanged

#### Scenario: Register through materialized drafts
- **WHEN** valid or invalid registrations are created, updated, rebased, previewed or committed through existing drafts
- **THEN** the same core rules, conflict and atomic resource ownership apply without transport-side domain decisions or failed legacy adoption

### Requirement: Sound definitions preserve rendered semantics
Schema40 registry adoption, registration, replacement and pure selection alone MUST preserve every existing item, role/ducking, timing, volume/fade/channel, normalized filter graph, evaluated plan, pixel and decoded audio result. Equivalent legacy/default/registered states SHALL satisfy existing preview/export/draft native SSIM>=0.99, aligned float-PCM RMS<=0.0001 and one-frame timing bounds. No sound-event placement or audible rendering activation, DSP or side-chain capability SHALL be advertised by this definition-only feature.

#### Scenario: Compare native metadata-only states
- **WHEN** a fixed audio/visual fixture is migrated and valid sound definitions are registered/replaced without timeline edits
- **THEN** evaluated plans/filter graphs remain exact and original RGB/PCM preview/export/draft oracles pass

#### Scenario: Preserve prior ducking and activation boundaries
- **WHEN** an existing role-routed/explicitly-routed fixture gains a sound definition with independent gain/bus/seed
- **THEN** its existing role-based ducking/audio remains unchanged and status advertises definitions without claiming placement or DSP

### Requirement: Independently governed additive sound-definition contracts
Protocol1 SHALL retain every old operation, schema/error/response/annotation/alias while adding sound_event_register, semantic_sound_event_definitions_v1 capability and schema40 reporting. Canonical semantic-sound-events-v1, Rust, TypeScript/Zod, MCP and durable-draft consumers MUST agree. The seven active current-schema catalog headers SHALL advance exactly39→40 with every other byte preserved; all frozen feature/historical catalogs/counts and older drift controls MUST remain independently pinned. Predecessor projections SHALL remove only independently captured exact additions and validated reporting transitions, rejecting unrelated drift. Every original required contract/native/integration/package consumer MUST remain required alongside the new consumers.

#### Scenario: Exercise real registration transports
- **WHEN** real MCP/headless clients read support and register/replace definitions alone or in alias-bearing batches/drafts
- **THEN** canonical typed contracts agree with core success/failure/resource/history/reopen behavior

#### Scenario: Preserve every predecessor drift proof
- **WHEN** exact additions are projected or unrelated fields, catalogs, operations or annotations are mutated
- **THEN** the independently captured issue65 and all older proofs accept only authorized additions and reject unrelated mutation without rewriting old oracles
