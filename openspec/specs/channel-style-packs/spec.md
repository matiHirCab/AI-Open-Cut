# Channel style packs

## Purpose

Save bounded reusable native channel styling and explicitly materialize independent editable project content with managed dependencies.

## Requirements

### Requirement: Bounded versioned local channel-pack library
The core SHALL own a shared local channel-pack library under the configured projects root. A pack SHALL have UUID identity, a trimmed case-sensitive unique name of 1..80 Unicode scalars without controls, and a positive monotonically increasing safe-integer version. Create/import SHALL start at version one; update SHALL require expectedVersion and publish exactly the next version. The library SHALL retain only the latest version of at most 16 packs and SHALL be bounded to 64 MiB. Its version-one format SHALL fail closed on corrupt, future, oversized, duplicate-identity, duplicate-name or linked storage. Publication MUST use the existing root lock and atomic durable storage. List/get/export SHALL be read-only and SHALL never invoke providers or resolve ambient fonts. An uncertain publication SHALL return the existing actionable storage error without automatic effect retry.

#### Scenario: Two writers and unavailable library
- **WHEN** two writers update one version, or the library is corrupt, future, linked or oversized
- **THEN** a successful writer publishes one next version, a stale writer receives REVISION_CONFLICT, and invalid storage fails without overwriting authoritative bytes

#### Scenario: Read and export are independent from project runtime
- **WHEN** a saved pack is listed, read or exported with no provider running
- **THEN** its exact saved data is available without opening or changing projects or invoking inference

### Requirement: Portable closed captured native styling
A version-one pack SHALL contain bounded ordinary native component recipes at project component schema version 46, typed slot defaults, exact content-addressed font faces, declared managed-image dependencies, and optional caption-role and narration-profile references. It SHALL contain no executable templates, arbitrary filesystem references, source asset paths, provider configuration or ambient font selectors. Capture/update SHALL require source project expectedRevision and selected components, capture their complete bounded component-reference closure and referenced managed font bytes, and preserve source project/history/drafts/media bytes. Capture SHALL read only explicitly compatible source schemas46/47/48 without recovery, migration or staging; schema46 MUST omit narrationManifests and schema47 and schema48 MUST contain a valid real array. A pending transaction journal, schema45 or older, or future source SHALL fail explicitly and direct the caller to a separate recovery/open/migration action. Capture of selected derived artwork SHALL fail VALIDATION_FAILED before library publication because format1 cannot represent transitive artwork source provenance; unrelated derived assets outside the selected recipe closure SHALL NOT prevent ordinary capture. Portable component schema MUST remain46 for format1 independently of unrelated project manifest migrations. Portable import SHALL accept at most 8 MiB of UTF-8 JSON before deserializing its graph; canonical exported pack data SHALL also fit 8 MiB. Each pack SHALL allow at most 16 component recipes, 1024 total nested timeline items, 32 markers, 32 image dependencies and 32 font faces. Existing stricter native limits SHALL remain authoritative. Unknown fields, repeated raw JSON object keys at any depth (including native slot maps), container nesting deeper than64, duplicate identifiers, incompatible versions, invalid graphs/slots/bindings, audio/global-resource dependencies, unsupported artwork kinds and missing or invalid pinned fonts SHALL fail explicitly before library publication. Imports SHALL create an explicitly named independent copy; no guessed collision rename SHALL occur.

#### Scenario: Capture a caption and scene recipe without changing its source
- **WHEN** an existing project supplies component definitions with typography, color and text slots, nested components, markers and managed image/font dependencies
- **THEN** capture preserves their native semantics and exact referenced font bytes while source project, history, drafts and managed media remain byte-identical

#### Scenario: Pending recovery or legacy capture source
- **WHEN** capture observes a pending transaction journal, including a linked journal entry, or an unsupported older/future source schema
- **THEN** it returns an actionable recovery-required or validation error before any recovery/migration/staging and all source project/history/draft/media/journal bytes remain unchanged

#### Scenario: Reject unsafe or excessive packages
- **WHEN** import contains unknown executable/path fields, future format or component schema, invalid font bytes or digest, duplicate recipe identities, unsupported audio dependencies or an exceeded resource limit
- **THEN** a typed nonmutating failure occurs and no library or project data is published

#### Scenario: Literal slot names remain compatible
- **WHEN** a valid native recipe uses slot names such as __proto__, constructor or toString
- **THEN** capture, export, import and application preserve literal own-key semantics and native slot validation

#### Scenario: Preserve CP1 across schema47 narration adoption
- **WHEN** equivalent compatible source46,47 and48 projects are captured or an existing CP1 componentSchemaVersion46 pack is imported/exported
- **THEN** sources stay byte-identical, native recipes and old package digests stay intact, forged metadata46 and missing/null metadata47 fail closed, and portable componentSchemaVersion47 is rejected

### Requirement: Explicit managed dependencies and narration defaults
Application SHALL require explicit bindings from every image dependency to an existing managed image asset in the target project. Identity bindings MUST agree with the declaration's content hash, byte count, media kind and probe facts; target integrity SHALL be checked by the existing core asset machinery. Missing bindings, missing assets, incorrect bindings and damaged assets SHALL return existing actionable DEPENDENCY_UNAVAILABLE, ASSET_NOT_FOUND, INVALID_ARGUMENT and integrity errors respectively. No path lookup, network download or guessed same-name/hash substitution SHALL occur. Exact font bytes SHALL use existing static-outline font validation, catalog limits and project-owned managed publication; bound text SHALL never fall back to an ambient face. A caption role SHALL identify an existing text or rich-text slot in one recipe. An optional profile reference SHALL pin its saved identity and expectedVersion; application SHALL either resolve that version, use an explicitly selected replacement reference or explicitly disable the reference. Missing/stale profiles SHALL fail before project effects. Resolved narration metadata SHALL be returned for later explicit generation; pack operations SHALL not claim provider/voice readiness, synthesize speech or modify provider options.

#### Scenario: Missing or mismatched managed artwork
- **WHEN** an image binding is absent, refers to no target asset, mismatches declared metadata or points to corrupt managed content
- **THEN** application returns the corresponding existing typed error without a project revision or managed-font publication

#### Scenario: Missing profile and explicit decision
- **WHEN** a saved profile is missing or stale
- **THEN** application fails without effects unless the caller explicitly supplies an available replacement reference or disables narration defaults

#### Scenario: Pinned typography without ambient fonts
- **WHEN** a valid pack is applied on a machine without the capture machine's system fonts
- **THEN** its ordinary bound text uses the verified bundled faces and does not substitute ambient typography

### Requirement: Independent project materialization and explicit reapplication
Instantiation SHALL require target expectedRevision, exact pack identity/version, explicit dependencies and at most 32 placements. The core SHALL remap all recipe/component/item/marker identities to independent project-owned identities and ordinary native data, validate with existing core rules, publish managed fonts, and commit one revision with one undo entry. Existing track/component locks, slot rules, geometry, timing and resource integrity SHALL remain authoritative. The combined generated native edit batch SHALL remain within the existing 100-operation limit. Reapplication SHALL require the current target expectedRevision and an explicit complete one-to-one recipe-to-existing-component mapping; it SHALL replace those definitions through existing component/marker edit semantics, preserve ordinary per-instance slot overrides and report each mapped component's before/after semantic digest and changed status. Existing native success semantics SHALL apply even when definitions compare equal. Failed prepublication operations MUST roll back only their owned staged resources; journal-published uncertain results MUST retain recovery ownership and MUST NOT be blindly retried. Unrelated target content SHALL remain unchanged.

#### Scenario: Two independent fresh projects
- **WHEN** one pack is instantiated separately into two fresh projects using explicit managed artwork bindings
- **THEN** their ordinary content and pinned styling are equivalent but identities, managed ownership and subsequent project edits are independent

#### Scenario: Library update never propagates
- **WHEN** the saved pack changes after a project has applied it
- **THEN** existing content, undo/redo, drafts, reopening and native rendering remain independent from the pack library and profile library

#### Scenario: Explicit reapply preserves instance overrides
- **WHEN** a caller reapplies a newer pack through explicit component mapping at the current revision
- **THEN** mapped definition defaults change, per-instance overrides and unrelated content remain, the result reports concrete digest changes, and the operation commits one revision/undo entry

#### Scenario: Stale revision, locked content and persistence faults
- **WHEN** apply/reapply encounters a stale project revision, a native lock violation, invalid candidate or injected storage failure
- **THEN** existing conflict/lock/validation/storage errors and rollback or journal recovery semantics apply without partial authoritative edits or unowned resource deletion

### Requirement: Additive headless and source CLI access
The feature SHALL expose six uniquely named headless operations channel_pack_list, channel_pack_get, channel_pack_capture, channel_pack_update, channel_pack_import and channel_pack_apply, with closed typed requests and governed result shapes. A source CLI SHALL provide list/get/capture/update/import/export/apply commands; export SHALL emit portable package JSON and import SHALL pass the raw package JSON to core validation. CLI UTF-8 input and output SHALL be bounded to 9 MiB; packageJson SHALL retain its independent 8 MiB bound. Domain validation SHALL remain in core; adapters SHALL only perform structural transport/size checks and translate errors. Existing operations, MCP registrations and installed desktop behavior SHALL remain compatible. New CLI availability SHALL be documented as source availability rather than installed desktop readiness.

#### Scenario: Real source CLI round trip
- **WHEN** a caller captures, exports, imports and applies a pack through the source CLI and actual headless binary
- **THEN** the typed results agree with native operations, duplicate/unknown package fields remain visible to core validation, and oversized input/output fails before emission or project effects

#### Scenario: Existing clients remain valid
- **WHEN** existing headless, MCP, provider and project contracts run alongside the new uniquely named operations
- **THEN** their declarations, stable errors, capabilities and request/response semantics remain unchanged except approved additive operation catalog entries

### Requirement: Native rendering, governance and lifecycle evidence
The canonical channel-pack fixture SHALL be consumed by actual Rust and TypeScript request/result and behavior tests with required-consumer negative controls. Historical contract predecessors, raw bytes, digests and policy gates SHALL remain intact through exact additive inverses; no historical repinning or masked required command SHALL be accepted. Verification SHALL exercise real native rendering of independently materialized scenes, captions/text slots, color defaults and pinned font/artwork content in two projects, source immutability, explicit reapply, revisions/history, reopen and supported legacy project migration. Pack output SHALL use the existing native renderer, with independent pixel/content oracles rather than a new renderer. Source and actual assembled headless workflows SHALL both be verified. Existing decoder limitations SHALL be reported only when reproduced; illustrated release ownership and spoken-text audition scope SHALL remain separate. All repository-required checks, conformance review, spec synchronization/archive and final protected gates SHALL pass before publication as a verified draft, and all exact-head required CI checks SHALL be tracked to terminal status without claiming failed or pending checks passed.

#### Scenario: Canonical consumers cannot be omitted
- **WHEN** a required Rust/TypeScript consumer or callback is removed, skipped or replaced with comments or a success mask
- **THEN** parity/governance tests fail while unchanged historical raw predecessor proofs still hold

#### Scenario: Real native workflow remains editable and reopenable
- **WHEN** source and assembled native workflows render original configurable scenes after independent application, explicit reapply, undo/redo, reopen and legacy migration
- **THEN** native pixels/text/colors and managed ownership satisfy independent fixture expectations without consulting a current pack library or introducing a new rendering engine
