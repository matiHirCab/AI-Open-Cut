# Project Persistence Specification

## Purpose

Define the durable project lifecycle, concurrency, history, and schema compatibility guarantees owned by the editor core.

## Requirements

### Requirement: Durable project lifecycle
The editor core SHALL create projects with validated settings and stable identifiers, persist them beneath the configured project root, list persisted projects, and reopen their current state.

#### Scenario: Create and reopen a project
- **WHEN** a caller creates a project with valid settings and later opens its identifier
- **THEN** the reopened project contains the persisted settings, initial tracks, revision, and timeline state

#### Scenario: Open a missing project
- **WHEN** a caller opens an identifier that has no persisted project
- **THEN** the operation fails with the stable `PROJECT_NOT_FOUND` error

### Requirement: Optimistic revision control
Every mutation of an existing committed project state that accepts an expected revision SHALL compare it with the current persisted revision and SHALL reject stale writers before publishing the requested state change.

#### Scenario: Reject a stale mutation
- **WHEN** a mutation supplies an expected revision different from the current revision
- **THEN** the operation fails with retryable `REVISION_CONFLICT` and the persisted project remains unchanged

### Requirement: Serialized durable persistence
Project mutations, project creation, and migrations MUST execute while holding the project lock, and each logical write of project state plus retained history MUST use one recoverable transaction whose durable commit point identifies a single authoritative generation. Each persisted JSON document SHALL be published through a synchronized temporary file and atomic replacement so readers never observe a partially written document.

#### Scenario: Publish a project generation
- **WHEN** a validated mutation commits new project state and retained history under the project lock
- **THEN** every subsequent locked read observes the project and history from that committed generation rather than a mixed pair

#### Scenario: Reject before the commit point
- **WHEN** persistence fails before the transaction commit point is durably published
- **THEN** the mutation fails and the prior project and history generation remains authoritative

#### Scenario: Interrupt after the commit point
- **WHEN** persistence is interrupted after the transaction commit point but before every destination is materialized
- **THEN** the target generation remains recoverable and the mutation is not reported as rejected

### Requirement: Deterministic interrupted-transaction recovery
The editor core MUST recover a valid interrupted transaction deterministically under the project lock before returning or mutating project state, MUST remove all managed transaction artifacts after successful recovery, and MUST fail closed with non-retryable `PROJECT_RECOVERY_FAILED` when recovery metadata is corrupt, unsupported, or inconsistent.

#### Scenario: Recover every interrupted publication phase
- **WHEN** a project is opened after termination between any two persistence phases following the commit point
- **THEN** recovery publishes the transaction's project and history together, completes any recorded draft consumption, and removes managed transaction artifacts

#### Scenario: Repeat interrupted recovery
- **WHEN** recovery itself is interrupted and the project is opened again
- **THEN** replay converges on the same committed generation without duplicating a mutation or pairing history from another generation

#### Scenario: Reject irrecoverable metadata
- **WHEN** transaction recovery metadata has an unsupported version, invalid content, or a project identity inconsistent with its directory
- **THEN** opening fails with `PROJECT_RECOVERY_FAILED` without guessing, defaulting history, or rewriting the live project documents

### Requirement: Unambiguous acknowledged mutation outcome
The editor core SHALL report a mutation as rejected only before its durable transaction commit point, and SHALL return the committed revision with stable `PERSISTENCE_RECOVERY_PENDING` warning when post-commit materialization remains for deterministic recovery.

#### Scenario: Report post-commit materialization failure
- **WHEN** the transaction commit point is durable but project or history materialization cannot finish before returning to the caller
- **THEN** the result identifies the committed revision and includes `PERSISTENCE_RECOVERY_PENDING` rather than returning a mutation error

#### Scenario: Access after a recovery warning
- **WHEN** a caller accesses the project after receiving `PERSISTENCE_RECOVERY_PENDING`
- **THEN** the core finishes recovery under the lock before evaluating the new request against the committed revision

### Requirement: Retained undo and redo history
The editor core SHALL retain at most 100 project snapshots in the undo stack, maintain redo snapshots until a new edit clears them, and apply history operations using the same revision conflict protections as other committed writes.

#### Scenario: Undo and redo an edit
- **WHEN** a caller undoes a committed edit and then redoes it using the returned revisions
- **THEN** the project state transitions through the retained snapshots and each transition increments the current revision

### Requirement: Deterministic schema compatibility
The editor core MUST migrate supported older project schemas and retained history deterministically under lock, and MUST reject unknown future schema versions without rewriting them.

#### Scenario: Migrate a supported project
- **WHEN** a supported older project is opened
- **THEN** its current state and each retained undo and redo snapshot are deterministically upgraded to the current schema before being returned

#### Scenario: Reject a future schema
- **WHEN** a project declares a schema version newer than the running editor supports
- **THEN** opening fails with `INTERNAL_ERROR` and the stored project is not downgraded or rewritten

### Requirement: Schema-v7 common visual migration is complete and pixel-preserving
Opening any supported schema-version-1-through-6 project MUST deterministically migrate the current project and every retained undo and redo snapshot to schema version 7 under the project lock. Migration MUST preserve every existing transform, visibility value, item identity, timing, ordering, revision, asset reference, and non-visual field exactly; missing common values MUST receive identity transform and `hidden: false` defaults, and the migrated generation MUST evaluate to the same pixels and audio as its schema-v6 source.

#### Scenario: Migrate current state and mixed retained history
- **WHEN** a schema-v6 project has non-empty undo and redo stacks containing supported older snapshots with visible and hidden items and non-default transforms
- **THEN** current state and every retained snapshot become schema v7 in one recoverable generation, existing values remain exact, and only absent common fields receive documented defaults

#### Scenario: Migrate oldest supported project
- **WHEN** a valid schema-v1 project is opened
- **THEN** it follows the deterministic supported migration chain to schema v7 and reopens to an equal schema-v7 state on every later open

#### Scenario: Preserve evaluated output
- **WHEN** equivalent pre-migration and migrated fixtures are evaluated for frame preview, audiovisual range preview, draft preview, and export
- **THEN** they produce equal evaluated semantics and remain within the existing deterministic visual, audio, and timing tolerances

### Requirement: Common visual migration is atomic and fail-closed
The schema-v7 migration MUST deserialize, migrate, and validate the complete current-and-history envelope before project, history, or content-addressed asset publication, MUST publish the migrated project/history pair through one existing crash-consistent transaction, and MUST leave the prior authoritative generation and managed asset store unchanged when any document, snapshot, default, or reference fails validation. Schema version 0 and unknown future versions MUST fail with existing stable compatibility behavior without downgrade or rewrite. Omitted common visual fields on schema-v7 input MUST continue to deserialize to their compatibility defaults without forcing a read-only rewrite; returned state and any later committed serialization MUST contain the explicit canonical fields.

#### Scenario: Reject an invalid retained snapshot
- **WHEN** current state is valid but any retained undo or redo snapshot cannot migrate or validate
- **THEN** open fails before publication and current state, all retained history, and the managed content-addressed asset store remain unchanged

#### Scenario: Accept schema-v7 compatibility defaults
- **WHEN** a schema-v7 document omits `transform` or `hidden` on a timeline item
- **THEN** opening supplies the identity transform and visible default without rewriting solely for those omissions, while returned state and the next committed serialization contain both explicit flattened fields

#### Scenario: Recover an interrupted migration publication
- **WHEN** schema-v7 generation publication is interrupted at any injected persistence phase
- **THEN** deterministic recovery selects one complete authoritative pre-migration or migrated generation and never exposes mixed schema versions

#### Scenario: Reject a future schema
- **WHEN** current state or any retained snapshot declares a schema version newer than 7
- **THEN** open fails closed with existing compatibility behavior and does not downgrade, partially migrate, or rewrite the authoritative generation

#### Scenario: Reject schema version zero
- **WHEN** current state or any retained snapshot declares schema version 0
- **THEN** open fails closed with existing compatibility behavior and does not migrate, rewrite, or publish managed asset content

#### Scenario: Reopen, undo, and redo after migration
- **WHEN** a migrated project is reopened and the user traverses retained undo and redo history
- **THEN** every returned state is schema v7, uses the common visual defaults, preserves its original revision and visual values, and persists deterministically

### Requirement: Schema-v8 Transform2D migration
The current schema SHALL be 8. Supported schemas 1 through 7 MUST migrate current state and every retained undo/redo snapshot under the project lock to schema 8. Migration MUST preserve legacy transform fields exactly, default transform2d to absent, and preserve revisions, identities, media, provenance, ordering, and legacy evaluated output. Historical schema-v7 migration requirements apply to the intermediate step, followed by this schema-v8 step.

#### Scenario: Upgrade mixed history
- **WHEN** a supported old project with non-default transforms and mixed supported undo/redo snapshots opens
- **THEN** the complete envelope becomes schema 8 in one recoverable generation with unchanged legacy output

#### Scenario: Reopen and traverse history
- **WHEN** a migrated project or a schema-8 project with Transform2D is reopened, undone, and redone
- **THEN** every state is schema 8 and preserves the exact active transform and original history semantics

### Requirement: Schema-v8 migration fails atomically
Core MUST validate the complete migrated envelope before publication or managed-asset writes. Invalid transforms, invalid references, schema 0, and versions above 8 in current state or history MUST preserve the prior authoritative generation and managed assets. Future versions MUST retain the existing INTERNAL_ERROR compatibility behavior. Interrupted migration MUST recover one complete generation. Schema-8 omission of optional transform2d MUST select legacy behavior without a rewrite solely for the omission.

#### Scenario: Reject invalid or future history
- **WHEN** any current or retained snapshot has invalid data, version 0, or an unknown future version
- **THEN** open fails with the existing typed validation/compatibility error and no partial migration or asset publication occurs

#### Scenario: Recover each publication fault
- **WHEN** migration publication is interrupted at any supported persistence injection phase
- **THEN** recovery returns one complete old or new authoritative generation, never a mixed pair

#### Scenario: Read an omitted optional field
- **WHEN** a valid schema-8 item omits transform2d
- **THEN** reading uses its legacy transform without rewriting solely to insert the optional field

### Requirement: Schema-v9 stacking migration
The current schema MUST be 9. Historical schema-v7 and schema-v8 requirements MUST apply to their intermediate steps, followed by migration to 9. Supported schemas 1 through 8 MUST migrate current state and every retained undo/redo snapshot under the project lock, assigning zIndex zero and stackOrder from each snapshot's item arrays. IDs, revisions, transforms, timing, media, provenance, existing order, and evaluated output MUST be preserved.

#### Scenario: Migrate oldest and mixed history
- **WHEN** supported older current state and mixed-version retained undo and redo snapshots are opened
- **THEN** the complete envelope becomes schema 9 in one recoverable generation with explicit ordering values and unchanged legacy output

#### Scenario: Reopen and traverse migrated history
- **WHEN** a migrated project is reopened, undone, and redone
- **THEN** every resulting snapshot retains deterministic schema-9 ordering and the established revision behavior

### Requirement: Stacking migration fails closed
Migration MUST validate the entire envelope before publication or managed-asset writes. Invalid current/history data, schema zero, and versions above 9 MUST leave the previous authoritative generation and managed assets unchanged. Unknown future versions MUST retain INTERNAL_ERROR compatibility behavior. Interrupted publication MUST recover one complete generation.

#### Scenario: Reject malformed or future history
- **WHEN** any retained snapshot is invalid or has schema zero or an unknown future version
- **THEN** opening returns the existing typed validation/compatibility error without partial migration or asset publication

#### Scenario: Recover migration interruption
- **WHEN** publication is interrupted at each supported persistence fault-injection phase
- **THEN** recovery exposes one authoritative complete generation, never a mixed current/history pair

### Requirement: Schema-v10 group migration
The current schema MUST become 10, with prior schema requirements applying to intermediate migrations. Supported schemas 1 through 9 MUST migrate current state and every retained undo/redo snapshot under the project lock in one recoverable generation. Existing items MUST default to unparented without adding groups or changing IDs, revisions, timing, transforms, order, provenance, media, or evaluated output. Omitted optional parent in schema 10 MUST mean unparented without a rewrite solely for omission.

#### Scenario: Migrate complete mixed history
- **WHEN** supported older current state and mixed-version retained snapshots open
- **THEN** all become schema 10 atomically and legacy visual/audio output remains unchanged

#### Scenario: Retain a grouped project
- **WHEN** a schema-10 graph is reopened, undone, and redone
- **THEN** each resulting state preserves exact group properties, parent references, ordering, and established revision semantics

### Requirement: Hierarchy migration fails closed
Core MUST validate every migrated/current/history graph before publication or managed-asset writes. Invalid graphs or data, schema zero, and unknown versions above 10 MUST leave authoritative state and assets unchanged; future versions SHALL retain INTERNAL_ERROR compatibility behavior. Recovery MUST expose one complete old or new generation.

#### Scenario: Reject invalid retained hierarchy
- **WHEN** a retained snapshot contains a missing parent, cycle, cross-scope edge, excessive depth, invalid transform, or future version
- **THEN** opening fails with the established typed error without partial migration

#### Scenario: Recover migration interruptions
- **WHEN** migration encounters each supported persistence fault-injection point
- **THEN** recovery returns one complete authoritative generation with matching current state and history

### Requirement: Atomic schema-11 component migration
The current schema MUST become 11. Earlier schema requirements MUST apply to intermediate migrations. Supported schemas 1–10 MUST migrate current state and every retained undo/redo snapshot under lock, adding empty components without changing root IDs, revisions, timing, ordering, transforms, media, provenance or evaluated output. Schema 11 MUST require its components collection. The complete migrated envelope MUST validate before atomic publication or managed-asset writes; malformed current/history state, schema zero and future versions MUST leave authoritative files unchanged with existing stable compatibility errors. Interrupted publication MUST recover one complete generation.

#### Scenario: Migrate mixed retained history
- **WHEN** a supported project has nonempty undo and redo stacks with older snapshots
- **THEN** the entire envelope migrates deterministically to schema 11 and repeated reopen performs no additional rewrite

#### Scenario: Reject and recover atomically
- **WHEN** retained data is invalid or future-versioned, or publication is interrupted at an injected phase
- **THEN** invalid input publishes nothing and recovery selects a complete old or new generation

### Requirement: Component managed media retention
Core MUST include assets referenced by all definitions, retained history and durable drafts in existing integrity, deletion and garbage-collection decisions. Definition removal MUST NOT discard media still reachable through any retained owner, and component resource fields MUST retain existing confinement and provenance validation.

#### Scenario: Retain unused definition media
- **WHEN** an asset is referenced only by an unused definition, undo/redo snapshot or durable draft
- **THEN** integrity validation sees the reference and collection/deletion preserves it according to existing ownership rules

#### Scenario: Reject unsafe component resources
- **WHEN** a definition contains an invalid asset reference or resource escaping existing managed boundaries
- **THEN** core rejects it without changing files or publishing artifacts

### Requirement: Atomic schema-12 slot migration
The current schema MUST become 12. Earlier schema requirements MUST apply to intermediate migrations. Supported schemas 1–11 MUST migrate current state and all retained undo/redo snapshots under the project lock in one recoverable generation, adding required empty `slots` to component definitions and `slotValues` to nested instances. Other values, IDs, revisions, provenance, media and rendered output MUST remain unchanged. Schema 12 MUST require both fields where applicable and reject malformed current/history values before publication or managed-asset writes. Schema zero and unknown future versions MUST retain existing compatibility errors, including INTERNAL_ERROR for future versions, without rewriting files. Reopening a migrated project MUST perform no additional migration rewrite. No downgrade MUST be inferred.

#### Scenario: Migrate mixed retained history
- **WHEN** a schema-11 project with nested components and mixed supported undo/redo snapshots is opened
- **THEN** the complete envelope becomes schema 12 atomically with deterministic empty slot fields and preserved unrelated values

#### Scenario: Fail closed and recover interruptions
- **WHEN** any snapshot is malformed/future-versioned or publication is interrupted at each supported injection phase
- **THEN** invalid input publishes nothing and recovery exposes exactly one complete old or new generation

### Requirement: Retain assets referenced by slot values
Managed-asset integrity, deletion and garbage collection MUST include every slot default and instance asset override in current definitions, retained undo/redo and durable drafts, including unused/hidden definitions and overridden defaults. Removing one slot reference MUST NOT release an asset still retained by another owner. Existing core asset errors and path confinement MUST remain unchanged.

#### Scenario: Preserve slot-only media
- **WHEN** an asset is referenced only by a default, override, retained snapshot or durable draft
- **THEN** integrity sees it and existing deletion/collection rules protect it until no retained owner remains

### Requirement: Atomic schema 13 instance activation
Core MUST migrate supported schemas 1–12 and all retained undo/redo snapshots to schema 13 under the project lock using the existing recoverable transaction. The 12-to-13 step MUST change only the version and preserve content, revisions, provenance, assets and rendering for formerly valid input. Input MUST validate under its source schema before upgrade; previously forbidden root instances in schema 12 MUST NOT become valid through relabeling. Malformed current/history data or unknown future versions MUST fail without replacing the authoritative generation. Schema 13 MUST permit validated root instances with required canonical slotValues; older requests MAY omit optional values under the documented edit defaults. No automatic downgrade SHALL be provided.

Core MUST reject non-default legacy transforms on nested instances in source schemas 11–12 before upgrading their version, including hidden and unused definitions, using non-retryable INVALID_ARGUMENT. Current and all retained snapshots MUST receive this check; rejection MUST preserve the caller's original documents and byte-identical authoritative project/history files. Default legacy transforms and supported transform2d values MUST remain migratable, and valid schema-13 non-default legacy transforms MUST remain accepted.

#### Scenario: Migrate retained history and reopen
- **WHEN** a schema-12 project has nonempty undo/redo stacks and populated definitions and slots
- **THEN** current and retained snapshots migrate together preserving all content, undo/redo and repeated reopen remain deterministic, and root output stays equivalent

#### Scenario: Reject invalid and interrupted migration
- **WHEN** a snapshot contains invalid old-schema content, an unknown future version, or migration publication is interrupted
- **THEN** rejection leaves the original files intact or crash recovery selects one complete authoritative generation, never mixed current/history schemas

#### Scenario: Reject forbidden source transforms across current and history
- **WHEN** a schema-11 or schema-12 current, undo or redo snapshot contains a nested instance with non-default legacy position, scale or opacity, including hidden or unused definition content
- **THEN** opening fails with INVALID_ARGUMENT before migration publication and current/history files and in-memory input documents remain unchanged

#### Scenario: Preserve valid transform migration and schema-13 behavior
- **WHEN** supported old snapshots use default legacy transforms with absent or valid transform2d, or schema-13 snapshots use valid non-default legacy transforms
- **THEN** opening succeeds, supported old versions migrate without content changes, schema-13 transforms remain accepted, and mixed history and repeated reopen remain deterministic

### Requirement: Atomic schema 14 shape activation
Schema 14 MUST remain the shape activation milestone; prior milestones MUST apply to intermediate migrations, followed by any newer supported migration in the same recoverable transaction. Supported schemas 1–13 MUST migrate current state and every retained undo/redo snapshot under lock through one recoverable transaction. Source-schema validation MUST reject shape items in schemas below 14, including hidden and unused definitions, before relabeling. The 13-to-14 step MUST only change schema version, preserving all existing content, revisions, references, assets, provenance and evaluated output. No legacy rectangle conversion or downgrade MUST occur.

Every current and retained snapshot MUST validate before publication or managed-asset writes. Invalid geometry, invalid references, schema zero and unknown future versions MUST leave authoritative state unchanged with existing errors; future versions MUST retain INTERNAL_ERROR. Reopening a migrated project MUST be deterministic; native schema-14 projects MUST also undergo the current supported migrations without changing their shape content.

#### Scenario: Migrate mixed current and retained history
- **WHEN** supported older state with mixed nonempty undo/redo and component definitions opens
- **THEN** all snapshots pass the schema-14 activation step and reach the current supported schema atomically without changing legacy content/output and repeated reopening performs no extra migration rewrite

#### Scenario: Reject malformed source or retained state
- **WHEN** any current/undo/redo snapshot contains old-schema shape data, invalid geometry/references, schema zero or a future version
- **THEN** opening fails before publication and leaves in-memory source inputs and authoritative project/history/assets unchanged

#### Scenario: Recover and traverse shape history
- **WHEN** publication is interrupted at each existing fault-injection phase or a schema-14 shape edit is undone/redone and reopened
- **THEN** recovery selects one complete generation and every returned state preserves exact geometry, paint, revisions and history semantics

### Requirement: Atomic schema 16 grid activation
Core MUST activate grids in persisted schema 16 and migrate supported schemas 1-15 and every retained undo/redo snapshot under the project lock through one recoverable transaction, preserving all prior intermediate migration rules. The 15-to-16 step MUST change only schemaVersion; IDs, content, revisions, ordering, references, provenance, media integrity and existing evaluated output MUST remain unchanged. Source schemas below 16 MUST reject grid items, including hidden/unused definitions, before relabeling. Current and retained state MUST validate before publication. Invalid source content or future versions MUST preserve original in-memory input and authoritative files using existing errors, including INTERNAL_ERROR for unknown future versions. Older binaries MUST reject schema 16; no automatic downgrade SHALL be provided.

#### Scenario: Migrate mixed retained generations
- **WHEN** supported current state and mixed nonempty undo/redo snapshots containing legacy, shape, SVG and component content open
- **THEN** all snapshots migrate atomically to 16 without content/output changes, and undo/redo and repeated reopen remain deterministic without unnecessary rewrites

#### Scenario: Reject invalid current or history
- **WHEN** current, undo or redo contains a grid under schema 15 or earlier, invalid schema-16 grid/reference data, schema zero or an unknown future version
- **THEN** opening fails with existing typed errors and byte-identical authoritative project/history/assets and unchanged in-memory source values

#### Scenario: Recover interrupted activation
- **WHEN** migration is interrupted at any existing persistence fault-injection phase
- **THEN** recovery selects one complete old or new authoritative generation and never mixes current and retained schema versions

### Requirement: Atomic schema 17 repeater activation
Persisted schema 17 MUST activate repeater items. Opening each supported schema 1-16 project MUST validate the source snapshot under its declared version, reject repeater records in current state or retained history below schema 17, and migrate current plus every undo/redo snapshot to schema 17 deterministically and atomically under the project lock before returning state. A 16-to-17 migration MUST otherwise preserve revisions, tracks, items, components, IDs, ordering, timing, transforms, assets, provenance, drafts, and evaluated output. Invalid sources and unknown future versions MUST fail without rewriting authoritative files. Interrupted publication at every supported fault phase MUST recover one complete pre- or post-migration generation.

#### Scenario: Migrate mixed current and retained history
- **WHEN** a valid schema-16 project with non-empty undo and redo history, root/component items, media ownership, and drafts is opened
- **THEN** current state and all retained snapshots publish together as schema 17 with byte-equivalent domain content apart from schemaVersion and unchanged evaluated output

#### Scenario: Reject repeater content below its source version
- **WHEN** current state, hidden/unused component content, undo, or redo declares schema 16 or earlier while containing a repeater record
- **THEN** opening fails before migration with the stable invalid-input behavior and authoritative project/history bytes remain unchanged

#### Scenario: Recover schema 16 before the commit point
- **WHEN** schema 16-to-17 activation fails before the migration journal is durably created
- **THEN** project and history bytes remain identical to the schema-16 generation, no managed transaction files remain, and a later reopen can retry migration normally

#### Scenario: Recover schema 16 after every publication phase
- **WHEN** schema 16-to-17 activation is interrupted after journal creation, project publication, history publication, draft cleanup, or journal cleanup
- **THEN** reopen exposes one complete schema-17 current/history generation with every undo/redo snapshot migrated and removes all managed transaction files

#### Scenario: Recover interruption and reject future versions
- **WHEN** any other supported migration publication is interrupted or a project declares schema 18 or newer
- **THEN** reopen recovers one complete generation or fails closed for the future version without partial migration, downgrade, or rewrite

### Requirement: Atomic schema 18 text document migration
Schema 18 MUST activate required text-item documents. Earlier schema requirements MUST apply at their historical intermediate steps; schema 18 MUST remain a supported intermediate version before schema 19 font activation; versions above the current supported schema MUST fail closed. Core MUST validate source snapshots under their declared versions, reject text-item document fields in schemas below 18 while preserving previously legal rich-text slot values, and migrate supported schemas 1-17 plus every retained undo/redo snapshot under the project lock in one recoverable transaction. Migration MUST create exactly one unstyled run from each original root/component text string and preserve every other field, revision, ID, ordering, timing, transform, reference, asset, provenance, draft and evaluated output at this intermediate step; the subsequent schema-19 shaping transition has its separately documented visual impact. Native schema-18 text MUST require document/text agreement. Invalid source/current/history or schema zero MUST retain established typed errors and leave source inputs and authoritative state/assets unchanged; unknown future versions MUST retain INTERNAL_ERROR with no downgrade. Reopen MUST perform no unnecessary rewrite. Interrupted publication MUST recover a complete old or new generation.

#### Scenario: Migrate mixed supported state and history
- **WHEN** current and nonempty retained undo/redo contain supported older versions, root text, hidden/unused/nested component text, existing rich-text slots and legacy draft edits
- **THEN** all snapshots migrate together to 18 with one-run documents, unchanged legacy rendering/content and functional draft materialization, undo/redo and deterministic reopen

#### Scenario: Reject invalid or future source data
- **WHEN** current, undo or redo contains text-item documents below schema 18, missing/mismatched schema-18 documents, invalid references, schema zero or a version above the current supported schema
- **THEN** open fails with the established error before publication or asset writes and preserves original in-memory values and authoritative bytes

#### Scenario: Recover every migration interruption
- **WHEN** schema-18 migration is interrupted at each existing persistence fault-injection phase
- **THEN** pre-journal failures preserve the old bytes and later recovery exposes one complete authoritative generation with matching current/history and managed transaction cleanup

### Requirement: Atomic schema 19 font activation
Schema 19 MUST activate required font catalogs and bindings for text, with empty catalogs valid for snapshots without text. Earlier schema milestones MUST apply as intermediate steps; schema 18 MUST remain accepted and schema 19 MUST remain a supported intermediate version before schema 20 styled-text activation, and versions above the current supported schema MUST fail with the existing INTERNAL_ERROR compatibility failure. Core MUST source-validate all supported schemas 1-18 and every retained undo/redo snapshot before resolving fonts, reject schema-19 fields in older source versions and reject missing or malformed bindings in schema 19. Current state, retained history, retained draft font bindings and required managed font content MUST publish as one recoverable generation under the project lock. Migration MUST preserve IDs, revisions, text/documents, ordering, timing, transforms, media, provenance and draft operations, while selecting the new explicit text-layout profile. Migration MUST document the one-time shaping/layout change and MUST NOT promise schema-18 pixel identity for text. After successful migration, reopen MUST never re-resolve fonts or rewrite an unchanged generation. Invalid sources, unavailable required fonts or integrity failures MUST publish nothing; source values and authoritative files MUST remain unchanged. No downgrade SHALL be inferred.

#### Scenario: Migrate mixed retained snapshots and drafts
- **WHEN** supported current state and mixed nonempty undo/redo include root, hidden/unused component text, rich-text slots and durable draft edits
- **THEN** all retained state and font content migrate together, content/identity/revisions are preserved, and undo/redo, drafts and repeated reopen retain the new exact layout and hashes

#### Scenario: Reject invalid source and missing fonts atomically
- **WHEN** any snapshot has forbidden source-version fields, malformed current bindings, schema zero, an unknown future version or unresolvable required faces
- **THEN** migration returns the established typed error with unchanged source values and authoritative project/history/draft files and no newly owned font content

#### Scenario: Recover font activation interruptions
- **WHEN** publication is interrupted at every existing transaction fault phase and each added font/draft staging phase
- **THEN** recovery selects one complete old or new generation, never references absent staged fonts and cleans unreferenced transaction files

### Requirement: Atomic schema 20 styled text activation
Schema 20 MUST activate optional document spans and text paint stacks. Schema 19 MUST remain a supported intermediate version; the schema-19 future-version rejection MUST henceforth apply to versions above 20. All supported older current snapshots and every retained undo/redo snapshot MUST be source-validated and migrated under the project lock as one recoverable generation. Source versions below 20 MUST reject new span/paint fields, including hidden/unused component and rich-text slot data, before publication. Schema-19 migration MUST preserve absence of new fields and exact content, font bindings/catalogs, IDs, revisions, timing, transforms, ordering, media and provenance. Older migrations MUST retain their existing documented intermediate semantics. Retained drafts MUST preserve operations/bindings and materialize consistently with migrated state; existing draft format/integrity policy MUST govern activation without silently discarding valid drafts. Invalid current/history/draft sources MUST leave authoritative bytes unchanged with established typed errors. Versions above 20 MUST fail with INTERNAL_ERROR and no downgrade. Reopen of unchanged schema 20 MUST perform no unnecessary rewrite or font lookup. Interruption MUST recover one complete old or new generation.

#### Scenario: Migrate current state and retained history
- **WHEN** supported mixed-version snapshots include root/component text, rich-text slots, nonempty undo/redo and retained drafts
- **THEN** activation produces coherent schema-20 state/history with original applicable styles and bindings, working drafts and deterministic undo/redo/reopen

#### Scenario: Reject invalid sources without publication
- **WHEN** current, history or drafts contain forbidden source-version fields, invalid references/styles, schema zero or an unsupported future version
- **THEN** existing typed errors preserve source values and authoritative project/history/draft/font bytes

#### Scenario: Recover interrupted styled text activation
- **WHEN** migration fails at every existing staging/journal/project/history/draft/cleanup fault-injection phase
- **THEN** pre-commit failure preserves the old generation and subsequent recovery exposes exactly one complete generation without leaked transaction files

### Requirement: Atomic typed animation schema migration
The next project schema MUST add optional typed channel storage with empty default to current state and every retained undo/redo snapshot under the project lock. Migration MUST validate every migrated document and publish one recoverable generation or none; existing keyframes and managed-resource provenance MUST remain unchanged. Opening an unknown future schema or malformed channel document MUST fail without rewriting current state, history, or media.

#### Scenario: Migrate retained history
- **WHEN** a supported schema-21 project with nonempty undo and redo stacks is opened
- **THEN** current state and every retained snapshot migrate to the new schema with empty channels and retain the same evaluated output

#### Scenario: Fail migration atomically
- **WHEN** any current or retained document cannot migrate or validate
- **THEN** opening fails and the prior durable generation remains intact

#### Scenario: Reopen an animated project
- **WHEN** an animated project is saved, reopened, undone, and redone
- **THEN** channel identity, values, ordering, and evaluated results remain deterministic

### Requirement: Atomic schema 23 curve activation
Persisted schema 23 MUST activate parameterized channel curves. Core MUST validate source-version current state and every retained undo/redo snapshot under the project lock and migrate every supported older schema to 23 through one recoverable transaction. The 22-to-23 step MUST change only the schema version for previously valid documents; it MUST reject new curve objects in source schemas below 23 before relabeling. Existing IDs, revisions, channel values, timing, media provenance, drafts, and evaluated output MUST remain unchanged. Malformed current/history curves, schema zero, and unknown future versions MUST retain established typed failures and leave authoritative project/history/media bytes unchanged. Reopen MUST not rewrite an unchanged schema-23 generation; interrupted publication MUST recover one complete old or new generation. Older binaries MUST reject schema 23 rather than downgrade it.

#### Scenario: Migrate current and history
- **WHEN** a schema-22 project with nonempty undo and redo is opened
- **THEN** all retained states reach schema 23 atomically and previously supported animation samples remain unchanged

#### Scenario: Reopen an active curve
- **WHEN** a schema-23 project with cubic Bézier and spring channels is saved, reopened, undone, and redone
- **THEN** exact curve parameters and deterministic evaluated samples survive every transition

#### Scenario: Reject malformed or future state
- **WHEN** a current or retained snapshot contains a malformed curve, a curve forbidden by its source schema, or an unknown future version
- **THEN** opening fails with its stable typed error without publishing or rewriting authoritative state

#### Scenario: Recover interrupted migration
- **WHEN** migration is interrupted at any existing transaction fault phase
- **THEN** recovery exposes one complete project/history generation and no mixed schema or partial curve state

### Requirement: Atomic schema 24 marker activation
Persisted schema 24 SHALL add empty root and component marker collections and optional item-start expressions. Core MUST migrate current state and every retained undo/redo snapshot from all supported older schemas under the project lock through one recoverable transaction. The 23-to-24 step MUST add only empty marker defaults and the schema label to previously valid documents; it MUST reject marker records or expressions mislabeled as source schema 23. Legacy item IDs, revisions, timing, ordering, media provenance, drafts and evaluated output MUST remain unchanged. Malformed current/history marker data, schema zero and future versions MUST fail closed without authoritative rewrites; unchanged schema-24 reopen MUST not rewrite its generation.

#### Scenario: Migrate current state and history
- **WHEN** a schema-23 project with nonempty undo and redo history is opened
- **THEN** every retained snapshot reaches schema 24 atomically with empty marker defaults and unchanged legacy output

#### Scenario: Preserve marker references through history
- **WHEN** a schema-24 project containing root and component markers and item-start expressions is saved, reopened, undone and redone
- **THEN** all IDs, names, scopes, offsets and resolved timings remain deterministic

#### Scenario: Reject malformed or future snapshots
- **WHEN** current state or retained history has malformed markers, a marker field forbidden by its source schema, or an unknown future schema
- **THEN** opening fails with the established typed error without publishing a partial migration

#### Scenario: Recover an interrupted migration
- **WHEN** schema migration is interrupted at any durable transaction fault phase
- **THEN** recovery exposes one complete old or new project/history generation

### Requirement: Atomic schema-25 animation-loop migration
Core MUST migrate supported schema-24 and older current state and all retained undo/redo snapshots to schema 25 under the project lock in one recoverable generation. Existing channels without `loop` MUST retain their exact values and evaluated visual/audio output. Source schemas below 25 MUST reject loop fields before version relabeling. Every current and retained snapshot MUST validate before publication or managed-asset writes. Invalid source/history data, schema zero, and unknown future versions MUST leave authoritative state unchanged with the existing stable compatibility errors. Interrupted publication MUST recover one complete generation; repeated reopen MUST not rewrite valid migrated state. No downgrade MUST be inferred.

#### Scenario: Migrate current state and retained history
- **WHEN** a schema-24 project has channels and nonempty undo/redo history
- **THEN** current and retained snapshots reach schema 25 atomically, unlooped output remains identical, and undo/redo and repeated reopen are deterministic

#### Scenario: Reject invalid or future retained state
- **WHEN** any source snapshot contains a pre-25 loop field, malformed loop, zero/future schema, or invalid retained data
- **THEN** open fails with the established typed error before rewriting authoritative project, history, or asset state

#### Scenario: Recover an interrupted migration
- **WHEN** publication is interrupted at a supported fault-injection phase
- **THEN** recovery exposes exactly one complete old or new generation, never mixed current/history schemas

### Requirement: Atomic schema-26 inherited timing migration
Core MUST migrate supported schema-25 and older current state and every retained undo/redo snapshot to schema 26 under the project lock in one recoverable generation. Source schemas below 26 MUST reject staggerMs and timeOffsetMs before relabeling. Migration MUST give absent fields zero semantics without changing prior visual/audio output, IDs, revisions, ordering, channels, loops or managed-resource provenance. Every snapshot MUST validate before publication. Malformed timing, invalid state/history, schema zero and unknown future versions MUST fail with existing stable typed errors and leave authoritative project, history and asset bytes unchanged. Interrupted publication MUST recover one complete old or new generation; repeated reopen MUST not rewrite a valid schema-26 generation. Older binaries MUST reject schema 26 rather than downgrade it.

#### Scenario: Migrate current state and history
- **WHEN** a schema-25 project has nonempty undo and redo stacks and no new timing fields
- **THEN** current state and every snapshot reach schema 26 atomically with identical output and deterministic undo/redo/reopen

#### Scenario: Reject invalid retained timing
- **WHEN** a source snapshot contains a pre-26 timing field, malformed timing or unsupported future schema
- **THEN** open fails before rewriting authoritative project, history or media state

#### Scenario: Recover interrupted publication
- **WHEN** migration is interrupted at a supported transaction fault phase
- **THEN** recovery exposes exactly one complete old or new generation without mixed schemas

### Requirement: Atomic inherited pre-publication validation
For candidates containing inherited child timing, signed copy timing or active parent animation, core MUST run canonical derived-clock, projection, known intrinsic geometry, inherited raster and complexity preflight before publishing project state, history, draft state or staged resources. Existing per-operation field and reference validation MUST retain its order. Ordered batches and draft operation lists MUST preflight their final candidate once before publication. Hidden and unused retained content MUST be checked. Unsafe derived time, overflow or excessive inherited bounds MUST return non-retryable INVALID_ARGUMENT with unchanged authoritative project/history/draft bytes, revision, aliases, resources and artifacts, and without generated-copy materialization. Stale revisions MUST retain retryable REVISION_CONFLICT and missing or locked references MUST retain their existing typed failures. Valid older zero-timing behavior and public wire shapes MUST remain compatible; no schema version change or silent repair MUST occur.

#### Scenario: Reject excessive inherited scale before an edit commit
- **WHEN** an affected candidate contains a 500-pixel-wide rectangle whose parent scale-X channel reaches 100
- **THEN** standalone or alias-aware batch publication fails with INVALID_ARGUMENT before changing revision, history, resources or artifacts

#### Scenario: Reject a hidden stagger overflow
- **WHEN** a hidden group's stagger delay pushes a ranked child ending at u64 maximum beyond representable time
- **THEN** candidate publication fails with INVALID_ARGUMENT before generated materialization and preserves authoritative bytes and revision

#### Scenario: Preserve every draft boundary
- **WHEN** draft creation, update, rebase or commit produces unsafe inherited timing or excessive derived bounds
- **THEN** the operation fails atomically without publishing draft or project changes or staged resources

#### Scenario: Validate current state and retained history before migration publication
- **WHEN** opening or migrating current state or any retained undo/redo snapshot encounters an unsafe inherited candidate
- **THEN** it fails before resource or transaction publication without rewriting authoritative state, and valid retained generations preserve undo/redo, reopen and interrupted-publication recovery

#### Scenario: Keep compatible successes and error precedence
- **WHEN** older valid requests omit timing, or an affected edit has a stale revision, missing reference, locked track or trailing invalid operation
- **THEN** valid older behavior remains unchanged and existing failure codes, retryability and rollback remain intact without exposing partially generated aliases

### Requirement: Atomic schema-27 extended visual animation migration
Core MUST migrate every supported older project to schema 27 under the project lock, adding full-source crop and empty effect stacks without altering IDs, provenance, revisions, legacy channels/curves/loops, timing, stacking, or rendered output. Migration MUST validate current state, every retained undo/redo snapshot, and retained draft state/candidates before atomically publishing one complete generation. Old source schemas containing prematurely authored crop/effect fields or newly activated channels MUST fail closed rather than gaining retrospective validity. Schema zero, malformed source/history/drafts, unknown future schemas, and interrupted writes MUST preserve established stable failures and recovery guarantees. Older binaries MUST reject schema 27. Valid schema-27 reopen MUST be deterministic and MUST NOT rewrite authoritative bytes.

Retained version-2 draft validity MUST NOT depend on extended channels, crop or effects being present in the draft or its base. Validation MUST use an applicable matching retained base where available and preserve established valid-stale-draft validation and REVISION_CONFLICT behavior without replaying onto an unrelated current revision. An invalid target in a legacy-only retained draft with an applicable base MUST fail with its existing typed error before any migration publication.

When a structurally valid retained version-2 draft's base revision is absent from current state and retained undo/redo history, core MUST preserve its established stale-draft behavior without attempting candidate replay against current state. Structural, catalog and resource validation MUST remain enforced; migration MUST preserve that draft's authoritative bytes. Project reads, edits and draft discard MUST remain available after history eviction. Applying or previewing that stale draft MUST retain REVISION_CONFLICT without authoritative writes.

#### Scenario: Migrate retained history and drafts
- **WHEN** an older project contains nonempty undo and redo histories and retained drafts
- **THEN** current and retained state migrate atomically with identity defaults and unchanged undo/redo, revisions, provenance, and legacy output

#### Scenario: Reject invalid retained state before publication
- **WHEN** any source snapshot or draft contains premature fields, an unsupported channel, invalid target, or future schema
- **THEN** open fails without changing authoritative project, history, draft, or managed resource bytes

#### Scenario: Recover and reopen deterministically
- **WHEN** migration is interrupted at a supported fault phase or a valid migrated project is reopened repeatedly
- **THEN** recovery exposes one complete old or new generation and subsequent valid reopen preserves bytes and sampled output

#### Scenario: Reject invalid legacy-only retained draft target
- **WHEN** current state and all retained undo/redo snapshots are at schema 26 and a retained version-2 legacy-only trim draft with an applicable base references a missing item
- **THEN** open returns ITEM_NOT_FOUND before schema-27 publication and preserves authoritative project, history, draft and managed-resource bytes

#### Scenario: Preserve a valid stale legacy draft
- **WHEN** a schema-26 project contains a valid stale legacy-only version-2 draft with an applicable retained base
- **THEN** schema-27 migration preserves draft bytes and IDs without applying its operations to current state, and draft access retains the established REVISION_CONFLICT behavior

#### Scenario: Evict a valid stale draft base during ordinary edits
- **WHEN** a valid trim draft refers to an item deleted after its base revision, later current state has extended effects, and ordinary edits evict the base from bounded undo/redo history
- **THEN** the edit causing eviction and subsequent project reads, edits, reopen and draft discard succeed without replaying the stale draft against unrelated current state

#### Scenario: Migrate a stale draft whose base is no longer retained
- **WHEN** a supported older project has a structurally valid version-2 draft with an unavailable base, including operations referencing an item absent from current state
- **THEN** schema-27 adoption preserves draft bytes, migrates current and history atomically, and draft application or preview returns REVISION_CONFLICT without changing authoritative bytes

### Requirement: Atomic schema 29 preset provenance migration
Editor-core MUST migrate every supported older project through the schema29 provenance stage and onward to the current schema31 under the existing project lock, including current state, all retained undo/redo snapshots and items in root/component-definition tracks. Earlier documents MUST gain empty provenance without inferring preset authorship from existing channels. Migration MUST preserve existing IDs, revision/timestamps, keyframes/channels/curves/loops, hierarchy, assets/fonts and managed resource bytes. Source documents below schema 29 MUST reject any `animationPresetProvenance` field, including empty/null values, rather than silently drop or activate it. Valid schema29 projects MUST complete the schema30 stage and schema31 upgrade without relabeling or recompilation, while valid current-schema projects MUST reopen idempotently without rewriting their stored generation.

#### Scenario: Migrate current and complete retained history
- **WHEN** a supported older project with undo/redo history and component-definition animation opens
- **THEN** current state and every retained snapshot migrate together through schema29 and schema30 to schema31 with empty provenance and equal prior primitive/render semantics

#### Scenario: Reject premature provenance
- **WHEN** current state or a retained snapshot below schema 29 contains a provenance field, including an empty object
- **THEN** migration fails closed without publishing current state/history or modifying managed bytes

#### Scenario: Repeat a completed migration
- **WHEN** a valid migrated current-schema project is opened repeatedly
- **THEN** its authoritative project/history bytes, revision/timestamps, resolved channels and provenance remain unchanged

### Requirement: Fail-closed complete provenance generations
Migration and publication MUST validate all current/retained provenance shapes, identities, finite parameters, primitive/reference safety and existing candidate limits before publishing a complete project/history generation. A malformed current document, malformed retained undo or redo snapshot, unsupported future project schema, or pre-commit I/O failure MUST preserve the previous authoritative generation. Unknown future project schemas MUST retain the established `INTERNAL_ERROR` failure and MUST NOT be rewritten; a historical preset source version absent from the compilation catalog MUST remain distinguishable from a future project schema. Valid interrupted commits MUST retain existing recovery and `PERSISTENCE_RECOVERY_PENDING` behavior. Rolling back an upgrade MUST require a prior complete backed-up generation rather than dropping fields or downgrading the current schema31.

#### Scenario: Reject invalid retained provenance atomically
- **WHEN** a retained undo or redo snapshot contains null/orphaned/non-finite provenance or invalid resolved primitives
- **THEN** the entire migration/publication fails through the existing typed validation path and current state/history/resources remain the previous generation

#### Scenario: Preserve future-version rejection
- **WHEN** current state or retained history declares a project schema newer than the current supported schema31
- **THEN** core returns `INTERNAL_ERROR` and leaves the complete generation unchanged

#### Scenario: Recover a committed provenance generation
- **WHEN** project/history publication is interrupted after the existing durable commit point
- **THEN** recovery converges on the complete saved primitives/provenance/history generation without compiling or applying the preset twice

### Requirement: Atomic schema31 retained-clock migration
Core MUST persist retained animation clocks only in schema31. Supported schema1–30 documents MUST migrate deterministically through the actual closed scalar/Pack provenance model and semantic/source validator, without recompiling or changing source animation/provenance/media records, including root tracks, component definitions and every retained undo/redo snapshot, under the existing project lock and atomic publication protocol. The explicit30→31 adapter MUST preserve tagged Pack parameters and original untagged scalar parameters, effective curve/iterations, channels, loops, motion blur, markers, IDs, revisions, timestamps, assets and fonts. Pack parameters before30, clocks before31, malformed source records and unknown future schemas MUST fail closed without modifying authoritative disk/current/history. Already31 supported variants and clocks MUST reopen idempotently.

#### Scenario: Migrate current components and retained history
- **WHEN** a supported project with component animations and retained undo/redo snapshots is reopened
- **THEN** every snapshot becomes schema31 atomically, absent clocks preserve original output and a subsequent edited reopen restores exact retained clocks

#### Scenario: Preserve all recognized motion pack variants
- **WHEN** a schema30 generation contains scalar provenance and impact_slam, slide_left, scan, pulse or radar_expand Pack parameters in root items, components, undo or redo snapshots
- **THEN** validated30→31 migration preserves exact original provenance and primitive records without catalog recompilation or scalar retagging

#### Scenario: Fail closed and retain disk state
- **WHEN** any current or retained snapshot contains malformed provenance/clocks, a pre30 Pack, a pre31 clock or a future version, or staged migration publication fails
- **THEN** migration fails with the established non-retryable error and leaves the complete persisted bundle unchanged or recoverable through the existing atomic journal protocol

#### Scenario: Recover recognized30 journals through full validation
- **WHEN** a recognized schema30 transaction journal contains supported provenance in current/components/all retained history
- **THEN** complete source validation precedes replay and locked atomic migration to31 preserves all source records; malformed journals fail PROJECT_RECOVERY_FAILED before replay

#### Scenario: Preserve known31 and mixed supported history
- **WHEN** valid schema31 clocks and both provenance variants are reopened or a generation contains older scalar snapshots alongside schema30 Pack snapshots
- **THEN** recognized snapshots retain source attribution and migrate as one generation, and already31 records reopen idempotently

### Requirement: Atomic schema 30 motion pack migration
Core MUST migrate every supported older schema through the schema30 motion-pack stage and onward to current schema31 under the existing lock and recoverable atomic journal, including current project, component items and every retained undo/redo snapshot. Schema29 scalar provenance, channels, revisions/timestamps, media/font integrity and rendered output MUST remain unchanged; migration MUST infer no labels. New tagged source shapes in pre30 item envelopes, malformed source generations and unknown future schemas MUST fail closed before migration publication. Recovery MUST preserve existing committed-warning semantics. Older builds MUST reject schema30; rollback MUST use a complete pre-migration project/history backup.

#### Scenario: Migrate retained scalar sources unchanged
- **WHEN** schema29 current/components/undo/redo contain valid scalar sources and channels
- **THEN** all snapshots pass through30 and become31 atomically with identical source/channel content and render output, and second reopen does not rewrite files

#### Scenario: Fail closed across the generation
- **WHEN** current/component/undo/redo has malformed or premature tagged source data, duplicate raw fields/maps, array records or a future schema
- **THEN** open fails through the canonical persisted-input path without changing either authoritative file or managed resources

#### Scenario: Recover interrupted migration
- **WHEN** a fault occurs at each existing migration publication phase
- **THEN** reopening recovers one complete accepted generation and retains project/history/source consistency without recompilation

### Requirement: Structurally scoped introduced-field validation
Historical source-version validation MUST distinguish actual persisted model fields from legal dynamic component slot identifiers. The introduction guards for `animationChannels` below22, item `startTime` below24, actual inherited `staggerMs` and repeater `timeOffsetMs` below26, item `crop`/`effects` and actual extended channel properties below27, and item `motionBlur` below28 MUST inspect their owning root/component track items or channel/descriptor locations, not arbitrary nested dictionary keys. A legal slotValues entry whose slot ID equals any such name MUST retain its typed value and migrate to current schema31 without being interpreted as animation metadata. Actual premature fields MUST still fail closed by presence, including explicit zero/null/empty records. Supported current state and every retained undo/redo snapshot MUST use the same source validation under the existing lock and atomic complete-generation protocol. Existing structural clocks/Scalar/Pack provenance guards, future-schema errors, finite/resource safety and recovery semantics MUST remain unchanged.

#### Scenario: Migrate legal colliding slot identifiers
- **WHEN** a supported historical root or nested component instance contains a legal Number slotValues entry named animationChannels, startTime, staggerMs, timeOffsetMs, crop, effects or motionBlur at a source version before the similarly named model field was introduced
- **THEN** migration validates the actual slot binding/value and preserves that slot ID/value, components, animation, assets/fonts, revisions and timestamps through schema31 without inferring introduced fields or recompiling provenance

#### Scenario: Migrate every retained legal slot generation atomically
- **WHEN** a supported current project or schema31 current project retains undo/redo snapshots containing those legal historical slot identifiers
- **THEN** current state and all retained snapshots migrate and publish as one validated generation, reopen idempotently, and Undo/Redo restore the exact intended slot values

#### Scenario: Reject actual premature model fields and preserve safety
- **WHEN** an actual root/component item, channel or repeater descriptor contains a field before its introduction, including zero/null/empty presence, or any retained source has malformed clocks/provenance, invalid slot values or an unknown future schema
- **THEN** validation retains its established failure and leaves current project/history/managed resources unchanged; legal dictionary names do not bypass the genuine source-version or semantic checks

#### Scenario: Preserve failure and recovery atomicity
- **WHEN** a malformed committed journal or pre-commit publication failure occurs while validating/migrating legal colliding slot generations
- **THEN** existing recovery/fault semantics apply, malformed sources are rejected before replay/publication, and no partial current/history migration or rewritten authoritative bundle is exposed

### Requirement: Schema33 mask model and channel migration
Persisted schema33 MUST activate closed mask animation targets/properties and approved static/animated mask rendering. Optional stored masks MUST retain empty-stack omission defaults and all #50 model fields/limits/IDs. Source schemas1…31 MUST reject any present masks field, including empty/null, on root/component items before defaults; source schemas1…32 MUST additionally reject new mask target/property identifiers, including empty-key channels. Schema32 static masks MUST remain legal and migrate without loss. Supported earlier current state and every retained undo/redo snapshot MUST migrate deterministically to33 under the existing project lock, preserving IDs, revisions, provenance, timing and managed resources. Newly activated renderability/work/precision checks MUST validate the complete migrated generations before publication. Valid nonempty32 masks MUST intentionally acquire approved mask output; absent/empty masks MUST preserve previous output. Unknown future/zero versions MUST retain existing fail-closed compatibility errors. Reading valid33 with omitted masks MUST NOT rewrite solely to materialize defaults.

#### Scenario: Migrate retained history and reopen deterministically
- **WHEN** valid earlier current state has both undo and redo snapshots, schema32 stored masks and legacy-only or applicable mask-bearing drafts
- **THEN** all applicable documents migrate together to33, authored masks/legacy clocks persist exactly, undo/redo reproduce the correct activated generation and repeat reads do not rewrite authoritative files

#### Scenario: Reject premature and future retained fields
- **WHEN** any retained source contains masks before32, mask targets/properties before33, malformed current mask/channel data or an unsupported future version
- **THEN** open fails before publishing current/history/draft migration or changing any managed resource

#### Scenario: Reject newly unsafe prior metadata atomically
- **WHEN** valid earlier metadata in any current/history/applicable-draft candidate exceeds activated renderability/work/inverse-precision bounds
- **THEN** adoption fails with existing typed errors and old authoritative/resource bytes unchanged, without clipping, repairing or dropping stored models

### Requirement: Atomic mask activation migration and draft validation
Core MUST validate the complete cloned current/history envelope and retained drafts before one existing crash-consistent migration publication. Applicable draft operations MUST replay against the matching current/retained base revision, never an unrelated generation, and complete replayable candidates MUST satisfy activated renderability. Earlier-than32 source envelopes MUST reject present masks operations before migration; earlier-than33 source envelopes MUST reject new mask target/property operations, including unavailable-base drafts. Valid schema33 drafts with unavailable bases MUST retain structural/catalog/resource validation and existing stale REVISION_CONFLICT behavior without candidate replay or owner-dependent certification against unrelated current state. Draft envelope version2 MUST remain unchanged. Failed migration, recovery, validation or draft mutation MUST preserve authoritative current/history/draft bytes and resources; existing journal recovery MUST precede new migration. Existing staged pre-publication selector/resource validation MUST cover channel-only mask edits as well as masks-bearing edits, preventing invalid create/update/rebase/commit/preview from publishing a resource rewrite or migration prefix.

#### Scenario: Preserve an invalid applicable draft generation
- **WHEN** an applicable retained draft has invalid mask/channel metadata or a missing owning item/target during migration
- **THEN** its existing structural/core failure blocks publication with every authoritative document/resource unchanged

#### Scenario: Preserve unavailable-base stale drafts
- **WHEN** a structurally/catalog/resource-valid schema33 draft with typed masks or mask channels has had its base evicted from retained history
- **THEN** reads/edits/discard remain available and preview/apply retain existing REVISION_CONFLICT without replay or owner-dependent certification on unrelated current state

#### Scenario: Preserve crash-consistent generation boundaries
- **WHEN** migration fails before commit or journal recovery encounters interrupted publication
- **THEN** readers observe a complete old/new generation under existing recovery rules, never a mixed current/history/draft publication

#### Scenario: Preserve source-matched channel-only staged requests
- **WHEN** a channel-only mask draft/edit has invalid targets/renderability on its applicable base while resource selectors could otherwise be rewritten or migration could occur
- **THEN** final-candidate failure publishes no project/history/draft/selector/resource prefix and retains existing acknowledged-commit/recovery behavior

### Requirement: Atomic schema 34 scoped matte adoption
Schema33 current state and all retained undo/redo snapshots MUST deterministically migrate to34 defaulting absent matte/matteOnly to None/false, under the project lock and existing staged/journal transaction. IDs/revisions/timestamps, ordering, media/font resources/provenance and unrelated semantics MUST be preserved; repeated reads SHALL NOT rewrite adopted files. Future schema versions MUST reject. Raw pre34 source envelopes MUST reject presence of matte/matteOnly on root/component items even null/false before applying defaults, and equivalent authored retained-draft operation fields MUST be guarded at their actual source bases.

#### Scenario: Adopt all retained generations and resources
- **WHEN** genuine schema33 current/undo/redo/applicable drafts coexist with managed media/fonts/provenance
- **THEN** all generations adopt34 atomically with unchanged unrelated values/resources and reopened undo/redo reproduce correct state without further read rewrite

#### Scenario: Reject premature or malformed retained data atomically
- **WHEN** any pre34 current/history/component source or draft contains new fields, or any source version is future/malformed/cyclic/dangling
- **THEN** full preflight rejects before adoption and current/history/draft/resource bytes remain unchanged

### Requirement: Source-matched matte drafts and journal recovery
Applicable drafts MUST be validated against their own current/matching retained bases before staged adoption, including hidden/unused graph and complexity checks. A valid schema34 unavailable-base draft MUST remain byte-preserved and current project read/edit/discard available, but draft preview/apply/commit MUST return REVISION_CONFLICT; it SHALL NOT be replayed onto current. Mask/matte-bearing legacy draft create/update failures MUST not incidentally adopt project/history/resources. Existing prepublication rollback and post-journal recovery MUST preserve one complete valid generation.

#### Scenario: Validate stale available and unavailable draft bases
- **WHEN** a valid stale draft's target exists only in its retained base, or no matching base exists for a valid schema34 draft
- **THEN** available-base validation uses that base, unavailable-base operations stay revision-conflicted and both preserve draft bytes without current-state replay

#### Scenario: Preserve recovery across publication phases
- **WHEN** draft/graph failure or injected publication interruption occurs before or after journal publication
- **THEN** prepublication failure preserves exact authoritative/resources and postpublication reopen recovers one complete valid current/history/draft/resource generation

### Requirement: Atomic schema 35 default-normal blend migration
Verified schema34 current and all retained undo/redo snapshots MUST adopt35 with absent blendMode default normal in one existing project-locked staged transaction. IDs/revisions/timestamps/order and all unrelated mask/matte/media/font/provenance values/resources MUST remain; reopened reads SHALL NOT rewrite adopted state. Raw pre35 root/component item sources MUST reject blendMode presence even normal/null before defaults. Retained draft new fields MUST be guarded against actual current/matching retained source bases, and future/malformed/unsupported sources MUST reject atomically.

#### Scenario: Adopt genuine predecessor current history and resources
- **WHEN** genuine schema34 current/undo/redo/applicable drafts contain old masks/mattes and managed media/fonts/provenance
- **THEN**35 adoption preserves semantic default-normal output, resources and revisions across all retained generations, with deterministic undo/redo/reopen and no read rewrite

#### Scenario: Reject premature malformed or future retained fields
- **WHEN** any pre35 root/component/current/history/draft source contains blendMode normal/null/non-normal or future/invalid retained data
- **THEN** staged preflight rejects before publication and every authoritative/resource byte remains unchanged

### Requirement: Source-matched blend drafts and complete journal recovery
Applicable drafts MUST validate against their own matching retained bases before adoption; valid35 unavailable-base drafts MUST be byte-preserved with current read/edit/discard available but preview/apply/commit REVISION_CONFLICT. Blend-bearing create/update failures on legacy bases SHALL NOT incidentally adopt current/history/resources. Existing prepublication exact rollback and post-journal recovery MUST retain one complete valid generation.

#### Scenario: Validate stale and unavailable bases without current replay
- **WHEN** a draft target exists only in retained state or no matching base exists for a valid35 draft
- **THEN** available-base validation uses that base, unavailable-base operations stay revision-conflicted and draft bytes remain unchanged

#### Scenario: Preserve failure and interrupted publication atomicity
- **WHEN** invalid later edits or injected persistence interruptions occur around all journal/publication phases
- **THEN** failures before publication preserve all bytes/resources and reopen after published journal recovers one complete valid current/history/draft/resource generation

### Requirement: Ordered-effect retained generation conformance without new migration
Existing ordered-effect current/undo/redo/draft generations and already-supported historical source adoption MUST preserve exact effect IDs/order/values and unrelated revisions/order/resources/provenance. Evidence MUST use genuine reviewed schema27-or-later sources where effects are valid and the actual verified predecessor adoption, not an invented schema advance or downlabeling new masked/matted/blended data. Invalid retained effect/target/source data MUST fail existing preflight atomically; valid no-rewrite reopen and source-matched stale/unavailable-base draft behavior MUST remain unchanged. No migration/schema/journal-format change is authorized. Under the explicit reviewed atomic-adoption amendment, effects-bearing edits and retained ordered generations MUST reuse the existing staged composition transaction owner so failed operations cannot publish incidental adoption.

#### Scenario: Preserve both ordered generations through existing adoption
- **WHEN** current and retained history contain opposite valid effects and an applicable old-base draft with managed resources/provenance in a genuine supported source
- **THEN** existing adoption preserves all ordered generations and native semantics, undo/redo/reopen reproduce correct arrays/pixels and subsequent reads do not rewrite state

#### Scenario: Reject invalid retained sources before publication
- **WHEN** any current/undo/redo/component/draft generation has invalid effects/targets/future source or genuine pre27 introduced-effect data
- **THEN** existing stable failure preserves complete authoritative/draft/resources and published-journal recovery evidence retains one complete valid generation without new transaction semantics

#### Scenario: Adopt ordered edits within one existing transaction
- **WHEN** an effects-only edit, alias batch or draft mutation on a supported legacy project fails before the existing journal commit, including a removed or retyped targeted effect
- **THEN** all authoritative/history/draft/managed-resource bytes remain unchanged; after the existing journal commit, recovery yields one complete correct ordered generation without new journal or migration semantics

### Requirement: Atomic schema36 color-control activation
Core MUST activate schema36 through existing complete-generation migrations and project locking/staged adoption. Supported sources through35 MUST retain exact older effects, current/undo/redo/components/drafts/media/fonts/provenance and source-matched policies while markers advance atomically. Nonempty color_adjustment records in any source generation below36 MUST reject before adoption; malformed current/retained effects and future versions MUST fail with existing stable typed errors and complete byte preservation. Current36 valid color effects MUST survive edit/history/reopen without reads rewriting state. No journal format, lock semantics or transaction owner changes are authorized.

#### Scenario: Migrate authentic schema35 generations together
- **WHEN** genuine35 current/history/component sources and matching drafts use older supported effects with real managed media/fonts and provenance
- **THEN** all applicable generations migrate once to36 with complete exact retained semantics/resources, undo/redo and subsequent no-rewrite reopen

#### Scenario: Reject premature malformed and future sources
- **WHEN** current/undo/redo/component/draft data contains premature color controls, malformed parameters/targets or a future schema
- **THEN** canonical preflight fails before any authoritative/draft/resource publication and all bytes/revisions remain unchanged

#### Scenario: Preserve current color-control histories
- **WHEN** current36 projects use different color-control values and opposite ordered stacks in current/history/drafts
- **THEN** accepted operations, undo/redo and fresh reopen restore exact arrays/values and independent native expected generations

### Requirement: Source-matched color-control drafts and complete recovery
Draft validation MUST resolve its existing matching current/undo/redo source rather than downlabeling new data or assuming the current marker proves every retained base. Matched pre36 drafts with new color records MUST reject; valid unmatched/stale drafts retain predecessor behavior. Effects-bearing operations MUST reuse existing staging selectors, journal point and resource owner, covering pre-journal complete rollback and post-journal complete-generation recovery without partial resource adoption.

#### Scenario: Preserve matched stale and unavailable base policies
- **WHEN** a draft refers to matching old/current/retained base or a valid unavailable/stale base
- **THEN** new records reject on matched pre36 bases, valid unavailable/stale drafts remain preserved under the existing preview/commit conflict policy and source-independent fields retain meaning

#### Scenario: Recover complete color-effect generations
- **WHEN** color-control standalone/alias edits and draft create/update/rebase/commit fail at existing applicable fault phases
- **THEN** precommit errors preserve full state/history/draft/media/font bytes; postcommit recovery through a fresh core yields the complete intended generation and exact undo/redo without new journal semantics
