# Agent Bridge Specification

## Purpose

Define the typed automation boundary over editor-core, including MCP exposure, transports, diagnostics, jobs, and stable errors.

## Requirements

### Requirement: Typed headless boundary
The bridge MUST invoke the typed headless boundary, delegating domain and persistence behavior to editor-core and exposing the supported public protocol version through status negotiation. Non-render requests MUST retain process-per-request execution. Frame preview, audiovisual range preview, materialized draft preview, export and audio analysis SHALL use one reusable render-work worker per bridge client when it is available; overlapping render-work requests MUST use independent one-shot processes without waiting for that worker. Audio analysis SHALL be classified as render-work under this boundary and SHALL use the same request-scoped cancellation, cleanup and protocol; other non-render requests remain process-per-request. Existing single-request CLI input, structured progress/result/error events, health behavior and exit semantics MUST remain compatible.

#### Scenario: Execute a valid headless request
- **WHEN** the bridge sends a supported typed request to the headless process
- **THEN** the process emits schema-compatible events and does not duplicate domain mutation rules in the transport

#### Scenario: Negotiate the current protocol version
- **WHEN** the bridge sends a status request naming the current supported protocol version
- **THEN** the process returns status containing that protocol version and its compatible capabilities

#### Scenario: Reject an unsupported protocol version
- **WHEN** the bridge sends a status request naming an unsupported protocol version
- **THEN** the process returns non-retryable INVALID_ARGUMENT without invoking an editor mutation

#### Scenario: Time out a headless request
- **WHEN** a headless request exceeds its configured deadline
- **THEN** the bridge terminates that request's process tree, cleans owned temporary output and returns retryable HEADLESS_TIMEOUT without cancelling independent requests

#### Scenario: W1 Reuse sequential rendering and preserve overlap
- **WHEN** sequential render requests arrive or another render arrives while the reusable worker is starting, busy or terminating
- **THEN** sequential requests use the available shared renderer and overlapping work starts through the one-shot path without a render queue

#### Scenario: W2 Preserve legacy clients
- **WHEN** a client uses the existing one-shot CLI or sends a non-render bridge operation
- **THEN** existing request/event contracts and process-per-request behavior remain unchanged


#### Scenario: Reuse isolated audio analysis work
- **WHEN** sequential audio analysis and rendering use the reusable worker or overlap requires a one-shot process
- **THEN** the compatible request/event boundary and all original worker scenarios remain valid, with request-isolated cancellation and temporary cleanup for both analysis passes

### Requirement: MCP capability exposure
The bridge SHALL expose project, asset, timeline, draft, render, speech, transcription, and job workflows as validated MCP tools, with project context available through registered resources and reusable prompts, and SHALL expose public protocol-version negotiation through the editor status tool.

#### Scenario: Discover automation capabilities
- **WHEN** an MCP client lists tools, resources, and prompts
- **THEN** it can discover the registered editing workflows and their validated input and output contracts

#### Scenario: Discover protocol compatibility
- **WHEN** an MCP client invokes editor status with the current public protocol version
- **THEN** it receives the same protocol version and compatible capability identifiers reported by the headless boundary

#### Scenario: Reject invalid MCP input
- **WHEN** a client calls a tool with input that does not satisfy its published schema, including an unsupported protocol version
- **THEN** the bridge rejects the request before invoking a provider or editor mutation

### Requirement: Safe local transports
The bridge SHALL support STDIO and Streamable HTTP, MUST default HTTP to loopback, and MUST require bearer authentication for non-loopback binds while enforcing configured host, origin, and body-size restrictions.

#### Scenario: Reject an unauthenticated remote request
- **WHEN** HTTP is bound beyond loopback and a request lacks the configured bearer token
- **THEN** the server returns an unauthorized response without dispatching MCP work

### Requirement: Bounded process-local jobs
Long-running bridge work MUST use a bounded process-local registry with stable identifiers, monotonic bounded progress, expiration, cancellation where safe, and documented loss on bridge restart.

#### Scenario: Cancel cancellable work
- **WHEN** a client cancels a running cancellable job
- **THEN** its abort signal reaches the operation and the terminal job reports retryable `JOB_CANCELLED`

#### Scenario: Protect an atomic commit phase
- **WHEN** a job has entered a commit phase that marked itself non-cancellable
- **THEN** cancellation fails with `JOB_NOT_CANCELLABLE` rather than interrupting the committed mutation

#### Scenario: Reject excess jobs
- **WHEN** the registry is full and cannot evict an eligible terminal entry
- **THEN** new work fails with retryable `JOB_REGISTRY_FULL`

### Requirement: Stable diagnostics and errors
The bridge MUST map core, provider, transport, timeout, and job failures to the canonical error catalog, including catalog-defined retryability, and MUST avoid exposing private paths, tokens, or user media text.

#### Scenario: Map a known failure
- **WHEN** a downstream operation returns a cataloged failure
- **THEN** the MCP response contains its stable code, safe message, and canonical retryability

#### Scenario: Report subsystem readiness
- **WHEN** a client requests editor status or runs diagnostics
- **THEN** core, rendering, speech, and transcription readiness are reported independently so optional failures do not masquerade as total editor failure

### Requirement: Complete typed group workflows
Headless and MCP MUST expose add_group, item_set_parent, item_set_z_index and group_ungroup as standalone edits and timeline_batch_edit variants with the existing project/revision envelope and mutation results. Transport adapters MUST delegate graph validation, ungroup behavior, atomicity and persistence to editor-core. Published documentation MUST explain local-preserving promotion, root-time timing, flat ordering, limits, errors and discovery.

#### Scenario: Execute real standalone group workflow
- **WHEN** a client creates a group, reparents a visual, changes its z-index and ungroups through each supported transport
- **THEN** typed results and subsequent project reads expose the expected core state and undo/redo/reopen restore the expected history

#### Scenario: Execute real batch workflow and failures
- **WHEN** the same workflow uses creation aliases in one batch, or encounters malformed input, a missing reference, a locked affected track, a stale revision or a later failed operation
- **THEN** real headless and MCP calls exhibit the specified single-commit or full-rollback behavior and canonical errors without adapter-owned domain mutation logic

### Requirement: Typed component definition workflows
Headless and MCP MUST expose component_create, component_update and component_delete standalone and in timeline_batch_edit with existing project/revision envelopes and mutation results. Creation aliases MUST work through real transports. Adapters MUST delegate graph, duration, lock, persistence and media semantics to editor-core. Documentation MUST describe local coordinates/time/order, explicit duration, scopes, bounds, errors, schema-11 migration and deferred instance rendering.

#### Scenario: Exercise complete definition lifecycle
- **WHEN** a real client creates, updates, references and deletes definitions using valid standalone and aliased batch operations
- **THEN** reads and undo/redo/reopen reflect exact core state through source integration and packaged smoke

#### Scenario: Propagate atomic failures
- **WHEN** malformed input, missing references, cycles, locks, stale revisions or later invalid batch operations occur
- **THEN** real transports preserve canonical errors and no partial project/history state is published

### Requirement: Typed template slot workflows
Headless and MCP MUST expose component_define_slots standalone and inside timeline_batch_edit, and accept the compatible optional slots/slotValues fields on component workflows. Requests and project responses MUST use closed typed definitions and values for all eight kinds. Protocol version 1 MUST advertise additive capability `typed_template_slots`. Adapters MUST delegate semantic binding, effective-value, reference, revision, lock and persistence validation to core. Documentation MUST specify type/constraint bounds, property mapping, local scope, integer-millisecond timing, ordering independence, default precedence, schema migration, errors and deferred rendering.

#### Scenario: Run real slot workflows
- **WHEN** source and packaged clients create, define, override and replace slots through standalone and aliased batch calls
- **THEN** typed reads and undo/redo/reopen reflect exact core state for all eight kinds

#### Scenario: Propagate atomic slot failures
- **WHEN** real calls encounter malformed types, missing references, stale revisions, locks or later batch failure
- **THEN** transport and core acceptance stages match documented contracts and no partial mutation is published

#### Scenario: Preserve override maps through real transports
- **WHEN** source and packaged clients submit special-key overrides or group opacity through standalone component edits and aliased batches, then undo, redo and reopen
- **THEN** typed request/response values match native state exactly, malformed entries fail without mutation, and protocol 1, schema 12 and advertised MCP input/output structural schemas remain unchanged

Shared request and response validation MUST reject unknown own enumerable fields in closed template-slot records before parsing can strip them, while preserving parsed types and complete nested issue paths. Protocol 1, schema 12 and published input/output structural schemas MUST remain unchanged.

#### Scenario: Reject malformed records in real standalone and batch workflows
- **WHEN** source and packaged clients submit canonical malformed defaults or overrides through standalone edits or aliased batches, including a valid operation before the malformed operation
- **THEN** requests fail structural validation and preserve the prior project state, revision and byte-identical project/history files without partial mutation

#### Scenario: Preserve nested validation and schema contracts
- **WHEN** shared request and response schemas validate nested unknown fields or existing malformed values, including under special slot IDs
- **THEN** errors retain full nested record or value paths and unknown-key names, valid values return the existing parsed types, and both input/output JSON schemas and the registered MCP structural catalog remain identical

### Requirement: Typed root component instance workflows
Headless and MCP MUST expose add_component_instance and component_instance_update standalone and inside timeline_batch_edit with existing project/revision envelopes and mutation results. Create MUST accept resultAlias; update MUST NOT produce an alias. Earlier aliases MUST resolve in trackId, componentId, itemId and parent.id where present; local IDs and slot-map keys MUST remain literal. Transport adapters MUST delegate timing, graph, effective slots, locks, persistence and rendering validation to core. Documentation MUST describe schema 13, half-open fractional derived timing, coordinates, ordering, visibility/audio, limits, errors and unsupported behaviors, superseding earlier deferred-rendering documentation only for component_instance_evaluation-capable runtimes.

#### Scenario: Exercise real clients
- **WHEN** source integration and packaged clients create, update, batch, preview and export component instances then undo, redo and reopen
- **THEN** typed requests/results agree with native state, aliases resolve correctly and invalid input, missing references, locks and revision conflicts preserve atomicity

### Requirement: Discoverable atomic component lifecycle workflows
Headless and MCP MUST expose component_instance_duplicate standalone and in timeline_batch_edit with existing project/revision envelopes and mutation results. Typed schemas MUST accept itemId, offsetMs and optional closed typed slotValues; batch creation MUST support resultAlias. Runtime status MUST advertise additive component_lifecycle under protocol 1 while retaining existing capabilities. Adapters MUST delegate domain validation and persistence to core. Documentation MUST explain using existing component_create, component_define_slots and add_component_instance for template creation/instantiation, complete override replacement, alias scope, retained coordinates/timing/order, bounds, errors and unchanged schema 13.

#### Scenario: Exercise real complete lifecycle clients
- **WHEN** source integration and packaged clients discover support and create, define, instantiate and duplicate templates standalone and in aliased batches
- **THEN** typed responses, history, reopen and representative preview/export match core behavior, including all supported override kinds

#### Scenario: Preserve failure atomicity through transports
- **WHEN** real clients submit invalid types, missing references, unsafe values, locked targets, stale revisions or a failing trailing edit
- **THEN** structural versus semantic rejection follows the canonical acceptance stage and failed requests preserve project/history bytes and revision

### Requirement: Typed discoverable shape workflows
Headless MUST expose add_shape and MCP MUST expose timeline_add_shape, both standalone and through timeline_batch_edit with existing project/revision envelopes and mutation results. All typed request, project response, batch, draft and component surfaces MUST accept canonical shape records. Adapters MUST delegate semantic validation and atomicity to core. Protocol 1 MUST advertise shape_items in core and aggregate capabilities; a ready complete renderer MUST additionally advertise shape_rendering in rendering and aggregate capabilities. An unavailable renderer MUST omit shape_rendering without hiding editable shape support.

Canonical operation, MCP structural schema/annotation, capability, shape fixture and ownership catalogs MUST match every governed native/TypeScript consumer. Documentation MUST specify geometry, coordinates, timing, ordering, bounds, errors, schema 14, alias usage and complete-backend failure behavior. Existing operation shapes, capabilities, error codes/retryability and provider protocols MUST retain their meanings.

#### Scenario: Discover and exercise real clients
- **WHEN** source and packaged clients discover support and create/edit each shape standalone and in aliased batches
- **THEN** typed responses, project reads, undo/redo/reopen and preview/export reflect core semantics and contract parity passes

#### Scenario: Reject through real transports
- **WHEN** clients submit structural/semantic invalid input, missing references, locked targets, stale revisions or a failed trailing batch edit
- **THEN** the documented validation stage and stable error are preserved with byte-identical prior state/history

#### Scenario: Report partial subsystem readiness
- **WHEN** core supports shapes but rendering dependencies are unavailable
- **THEN** shape_items remains discoverable while shape_rendering is absent and rendering retains its readiness error

### Requirement: Typed discoverable grid workflows
Headless MUST expose add_grid and MCP MUST expose timeline_add_grid using existing protocol-1 project/revision envelopes and mutation results. Request unions, update fields, batches, drafts, component inputs and project responses MUST carry strict canonical grid records. Adapters MUST submit typed values to core for domain validation and atomicity. Core and aggregate capabilities MUST advertise grid_items; rendering and aggregate capabilities MUST advertise grid_rendering only when complete rendering is ready. Existing operations, aliases, errors/retryability and provider protocols MUST retain their meaning; clients MUST use the new capabilities to distinguish grid support, and documentation MUST explicitly state that schema-16 grid-bearing projects require readers supporting the new item variant.

Canonical procedural-grid, operation, MCP structural schema/annotation, capability and ownership catalogs MUST match all governed Rust and TypeScript consumers and receive designated CODEOWNER review. Shared fixtures MUST distinguish representation rejection from semantic rejection and cover exact limits and geometry. Client documentation MUST define dimensions, lattice origin/orientation, spacing, paints, timing, ordering, clipping, limits, aliases, failures, migration and readiness.

#### Scenario: Exercise source and packaged clients
- **WHEN** real source and packaged MCP clients discover support and create/update each pattern standalone and in aliased batches, read projects, undo/redo, reopen and render
- **THEN** typed responses reflect core semantics and shared fixtures, schema/operation/capability parity and workflow assertions pass

#### Scenario: Reject transport failures without state change
- **WHEN** real clients send invalid structure or semantics, missing references, stale revisions, locked targets or a batch that fails after grid creation
- **THEN** the existing validation stage, error/retryability and full rollback behavior are preserved

#### Scenario: Distinguish editing from rendering readiness
- **WHEN** the renderer is unavailable while core grid support exists
- **THEN** grid_items remains advertised, grid_rendering is absent, and rendering returns DEPENDENCY_UNAVAILABLE without degraded output

### Requirement: MCP grid rollback reaches domain execution
Source and packaged MCP regression workflows MUST use valid operation identifiers so grid batch rollback exercises editor-core domain execution. A batch creating a grid followed by delete_item for a missing item MUST return non-retryable ITEM_NOT_FOUND with unchanged project content and revision. An otherwise valid stale-revision batch MUST return retryable REVISION_CONFLICT with unchanged state.

#### Scenario: Fail after creating a grid
- **WHEN** a real MCP client submits add_grid followed by a valid delete_item operation targeting an absent item
- **THEN** the response contains ITEM_NOT_FOUND and retryable false, with no grid or revision published

#### Scenario: Reject a stale revision
- **WHEN** a real MCP client submits a valid grid edit batch with an obsolete revision
- **THEN** the response contains REVISION_CONFLICT and retryable true and state remains identical

### Requirement: Typed discoverable repeater workflows
The public protocol MUST add headless edit operation `add_repeater`, batch union membership, repeater-bearing project/draft/component/item responses, and MCP tool `timeline_add_repeater` using strict mirrored schemas for the canonical descriptor. `timeline_batch_edit` MUST accept the same operation and alias rules. Canonical operation, MCP structural schema/annotation, capability, fixture, ownership, Rust, and TypeScript parity evidence MUST be updated together. Capability `repeater_items` MUST report editing support and `repeater_rendering` MUST be available only when a configured local renderer can execute the complete evaluated scene. Existing protocol-1 envelopes, simple operations, errors, and retryability MUST retain their meaning.

#### Scenario: Discover and invoke standalone and batch edits
- **WHEN** a client inspects operations, MCP tools, annotations, schemas, and capabilities and then submits equivalent valid standalone or batched repeater edits
- **THEN** every surface reports the exact canonical identifiers/fields, forwards typed input to editor-core, and returns equivalent revision, changed-ID, alias, item, and typed-error behavior

#### Scenario: Reject malformed and semantic failures consistently
- **WHEN** headless or MCP receives unknown/duplicate/missing fields, invalid values, missing/unsupported/cyclic sources, a locked track, or stale expected revision
- **THEN** transport decoding or core translation returns the established non-retryable `INVALID_ARGUMENT`, `ITEM_NOT_FOUND`, `TRACK_LOCKED`, or retryable `REVISION_CONFLICT` behavior without adapter-side domain rules or partial mutation

#### Scenario: Report readiness without degraded fallback
- **WHEN** editing support exists but no configured local renderer supports every instruction in the repeated evaluated scene
- **THEN** `repeater_items` remains discoverable, `repeater_rendering` is unavailable, and render readiness fails with `DEPENDENCY_UNAVAILABLE` rather than dropping or approximating copies

#### Scenario: Preserve additive compatibility boundaries
- **WHEN** an existing protocol-1 client continues using simple operations against projects without repeaters
- **THEN** its accepted requests and responses retain their meaning, while documentation states that schema-17 repeater-bearing projects and the new item variant require a repeater-aware decoder

### Requirement: Transport parity for aliased repeater replacement
Headless, source MCP and packaged MCP workflows MUST delegate aliased update_item.repeater replacements to editor-core using the existing typed schemas and protocol envelopes. No transport MUST implement its own alias substitution. Existing public shapes, capabilities, errors and retryability MUST remain unchanged.

#### Scenario: Exercise replacement across transports
- **WHEN** native headless, source MCP and packaged clients create and retarget a repeater with earlier source and item aliases
- **THEN** every transport returns equivalent resolved state, revision and alias results, supports undo/redo and reopen, and renders the resulting visible scene

#### Scenario: Preserve transaction failures across transports
- **WHEN** the replacement uses missing or forward aliases or is followed by a failing operation
- **THEN** each transport returns the established core error and preserves authoritative state and history

### Requirement: Transport parity for effective repeater audio rejection
Headless, source MCP and packaged MCP MUST delegate effective repeater source validation to editor-core with unchanged typed schemas and errors. Transports MUST NOT substitute slot values or classify source audio independently.

#### Scenario: Preserve effective-audio failure across transports
- **WHEN** clients submit a batch or draft whose effective asset slots introduce audio into a repeater source
- **THEN** headless, source MCP and packaged MCP return the established core error and preserve authoritative state, revision and history

#### Scenario: Preserve valid silent-source workflows
- **WHEN** a client submits a valid silent effective source with repeaters
- **THEN** existing aliases, undo/redo, reopening and rendered preview continue to work without public contract changes

### Requirement: Additive rich text transport parity
Protocol 1 MUST advertise `rich_text_documents` and retain existing operation/tool names and compatibility text output fields. Typed headless requests, MCP timeline_add_text and timeline_update_item inputs, timeline_batch_edit members, draft edits and project-state outputs MUST carry the canonical document without dropping run fields. Existing simple requests MUST remain valid. Strict wire schemas MUST reject unknown/null document fields before parsing can discard them; semantic validation MUST remain in core. Canonical versioned catalogs/fixtures and all consumers named by contract ownership MUST agree on the additive request/output/capability change and schema 18. Existing error codes/retryability and provider contracts MUST remain unchanged.

#### Scenario: Discover and round-trip documents
- **WHEN** a protocol-1 client discovers support, creates document text through standalone or aliased batch requests and reads project state
- **THEN** headless and MCP expose identical runs, compatibility text and revision semantics with the advertised capability

#### Scenario: Preserve old clients and reject malformed requests
- **WHEN** existing simple requests or malformed document fields are submitted through each transport and batch/draft surface
- **THEN** simple inputs remain accepted and malformed inputs fail with the established typed error without field loss or state/history mutation

#### Scenario: Verify canonical cross-language evidence
- **WHEN** Rust and TypeScript contract parity checks consume updated positive and negative fixtures
- **THEN** request/output shapes, capability identifiers, version reporting and stable errors agree across every governed consumer

### Requirement: Versioned bounded render worker transport
Worker mode SHALL be explicitly selected with --render-worker, emit a protocol-version-1 readiness event and accept closed newline-delimited envelopes containing requestId and an existing typed render request. Every subsequent progress/result/error event MUST carry that requestId outside the unchanged event payload. Only render_preview, render_preview_range, render_draft_preview and export_video MUST be accepted. Individual lines MUST be bounded to 16777216 UTF-8 bytes excluding the newline before unbounded accumulation. The worker MUST accept one outstanding request at a time. Canonical fixtures and native/TypeScript consumers MUST agree under ADR 0002; no existing MCP or project schema change is required.

#### Scenario: W3 Correlate bounded events
- **WHEN** valid worker requests and events arrive across arbitrary stream chunk boundaries, including an exact line-size limit
- **THEN** each request receives only its own ordered progress and single terminal event and bounded valid framing succeeds

#### Scenario: W4 Reject invalid worker input and output
- **WHEN** an envelope is malformed or oversized, output names an unexpected ID, or a duplicate terminal event occurs
- **THEN** the affected worker is discarded without replay or delivery to another request
- **AND** a valid correlated request naming a non-render operation returns non-retryable INVALID_ARGUMENT without mutation

#### Scenario: W5 Enforce startup and request deadlines
- **WHEN** a worker cannot start, advertises an unsupported version, fails its readiness deadline of five seconds or the remaining request deadline, or exits unexpectedly
- **THEN** startup failure uses DEPENDENCY_UNAVAILABLE, expiration of the request deadline uses retryable HEADLESS_TIMEOUT, and unexpected post-readiness exit or malformed output uses INTERNAL_ERROR
- **AND** the failed request is not automatically replayed

### Requirement: Isolated render request lifetime
Each worker request MUST have immutable validated artifact identity owned by editor-core, without changing process environment between calls. Temporary files MUST remain scoped to that request. Cancellation, timeout and protocol failure MUST terminate the affected process tree before owned temporary cleanup and slot reuse. Unconfirmed termination MUST keep the slot retired. Cleanup MUST NOT remove final published artifacts. A normal typed core error MUST allow a healthy worker to accept another request. Closing the client MUST prevent new reservations and terminate its idle and active workers. Workers MUST load and validate fresh project or draft snapshots for every request and preserve optimistic revision/error precedence.

#### Scenario: W6 Cancel one request without affecting others
- **WHEN** a warm render is cancelled or times out while an overflow render is active
- **THEN** only the affected process tree and owned temporary files are removed, the independent render can finish and the next reusable-worker request starts cold

#### Scenario: W7 Preserve draft and export cleanup
- **WHEN** frame, range, draft or export work fails after temporary artifacts exist
- **THEN** cleanup follows termination and removes only that request's temporary artifacts while preserving authoritative state and any completed final publication

#### Scenario: W8 Recover and close deterministically
- **WHEN** a request returns a normal core error, a worker crashes, or the client closes while idle or active
- **THEN** normal errors permit reuse, crashes permit replacement only for a later request, and closure terminates owned workers without reopening or replay

#### Scenario: W9 Revalidate project state and artifact identity
- **WHEN** project revision or draft state changes between worker requests, or an invalid artifact request ID is supplied
- **THEN** core reads the current requested snapshot and preserves REVISION_CONFLICT and other existing errors, invalid identity returns INVALID_ARGUMENT before artifact writes, and separate requests cannot reuse temporary filenames through mutable environment state

### Requirement: Typed animation transport parity
Headless request/response unions and MCP Zod schemas, tool registration, and batch schemas MUST expose the same additive replace-channels operation and closed payload. The exported TypeScript headless edit union MUST type channel input according to the closed channel schema so malformed channel records fail type checking before transport use. Transports MUST pass typed inputs to editor-core and preserve its stable error code, retryability, revision, changed IDs, and alias mapping.

#### Scenario: Standalone and batch parity
- **WHEN** an agent edits valid channels directly or through an alias in `timeline_batch_edit`
- **THEN** both transports return the same canonical result and persisted channel state

#### Scenario: Preserve typed failures
- **WHEN** core rejects an invalid channel, missing target, or stale revision
- **THEN** headless and MCP expose the corresponding stable error without a partial mutation

#### Scenario: Reject malformed TypeScript channel input
- **WHEN** a bridge caller constructs a headless channel edit with a wrong value tag, missing keyframe field, or unknown channel name
- **THEN** the exported edit type rejects that payload during type checking

### Requirement: Typed marker transport and discovery
Headless and MCP SHALL expose typed marker create, update, delete and item-start timing operations as standalone mutations and within `timeline_batch_edit`, using existing project/revision envelopes, results and aliases. Project responses MUST expose markers, item expressions and effective numeric start times. Status MUST advertise an additive marker-relative-timing capability under protocol 1. Adapters MUST delegate scope, name, timing, bounds, revision and persistence validation to editor-core. Public documentation MUST specify scope lookup, signed offsets, duplicate-name ambiguity, numeric fallback, ordering independence, errors and schema-24 migration.

#### Scenario: Execute standalone and batch marker edits
- **WHEN** a client sends valid marker edits through either transport, including batch aliases for newly created IDs
- **THEN** both transports return equivalent typed results and the same canonical project state

#### Scenario: Preserve typed failures and old clients
- **WHEN** a request has an invalid expression, missing/ambiguous marker, stale revision, or uses only preexisting numeric operations
- **THEN** the transport reports the core's stable typed failure or preserves the numeric client's prior behavior, as applicable

#### Scenario: Discover support
- **WHEN** a protocol-1 client reads status or MCP tool metadata
- **THEN** it can identify marker-relative timing support and the exact validated input/output schemas

### Requirement: Typed loop transport through existing operations
Headless request/response unions and MCP Zod schemas, tool registration, and batch schemas MUST expose the optional closed loop record through existing `set_animation_channels` and `timeline_batch_edit` operations as additive protocol-1 input. The exported TypeScript edit union MUST reject malformed loop records at type checking; runtime adapters MUST preserve editor-core's canonical result, revision, changed IDs, alias mapping, stable error code, and retryability. Status MUST advertise the additive loop capability. Adapters MUST not implement independent loop semantics. Public documentation MUST state bounds, half-open timing, seam and finite-exhaustion behavior, unsupported targets, errors, compatibility, and migration.

#### Scenario: Execute standalone and aliased batch loops
- **WHEN** a client submits a valid looped channel directly or addresses a newly created item alias in a batch
- **THEN** headless and MCP return equivalent typed results and persisted state

#### Scenario: Preserve transport failures and old requests
- **WHEN** a request has malformed loop shape, invalid core semantics, missing target, stale revision, or no loop field
- **THEN** transport rejects malformed shape or relays the core's stable result without partial mutation, and old requests keep prior behavior

### Requirement: Typed discoverable preset edit parity
Headless MUST expose `apply_animation_preset` in its typed nested edit union under the existing `edit`/`edit_batch` request envelopes. MCP MUST register `timeline_apply_animation_preset` and accept the same operation in `timeline_batch_edit`. All surfaces MUST carry typed `itemId`, mandatory `presetId`/`presetVersion`, closed scalar-tween or initial-motion-pack parameters and optional collision policy, use current project/revision envelopes and `WriteResult`, and preserve core result/error/retryability/changed-ID/alias semantics. Exported TypeScript types MUST reject missing mandatory fields, wrong parameter/value/curve tags and unsupported property names. Adapters MUST delegate catalog dispatch, compatibility, timing, finite semantic bounds, collisions, migration and publication to core.

Status/capability reporting MUST retain `animation_presets_v1`, add `initial_motion_preset_pack_v1`, report current project schema 30, and retain protocol major 1 and all existing identifiers/meanings. Tool input metadata, public documentation and canonical fixtures MUST expose the exact scalar seed and five pack IDs/versions, units, absolute item-local timing, default curve/policy, collision rules, provenance lifecycle and draft/direct-definition exclusions. Project/component responses MUST accept the optional provenance sidecar while preserving the old channel shape. Valid older request payloads MUST remain accepted.

#### Scenario: Match standalone and alias batch results
- **WHEN** equivalent valid preset requests run through headless edit and MCP standalone/batch, including an earlier creation alias
- **THEN** results expose the same committed primitive/provenance state, revision, changed IDs and batch alias mapping

#### Scenario: Return core failures through every transport
- **WHEN** a structurally valid unknown preset/version, collision, missing target, locked track, stale revision or final candidate rejection occurs
- **THEN** headless/MCP preserve the core stable error and retryability without partial mutation; malformed wire structures fail before mutation

#### Scenario: Type-check caller input and discover support
- **WHEN** bridge type fixtures construct malformed preset edits or a client inspects capability/tool metadata
- **THEN** malformed edits fail TypeScript checking and clients can distinguish the six-entry catalog (scalar compiler1 and pack compiler2), current schema30 and documented authoring exclusions under protocol 1

### Requirement: Discoverable typed initial motion pack
Headless and MCP MUST forward the unchanged preset operation with the additive closed parameter union and report initial_motion_preset_pack_v1 alongside existing capabilities. Canonical preset, project-schema, headless and MCP structural fixtures and all governed native consumers MUST agree. Existing scalar callers MUST remain valid. Unsupported live identities/versions and semantic boundaries MUST be decided in core; transport schemas MUST reject malformed closed shapes without expansion. Standalone and batch workflows MUST preserve existing aliases, revisions, typed errors, undo/redo and reopen behavior. Source and packaged smoke MUST exercise the five entries against the same canonical fixture.

#### Scenario: Old and new typed clients
- **WHEN** an old scalar request or any valid new pack request is passed through typed headless and MCP standalone/batch surfaces
- **THEN** fixtures and generated primitive/source state agree without transport-owned motion logic

#### Scenario: Failure parity and capability discovery
- **WHEN** clients inspect status or submit malformed fields, unsupported versions, collision, missing reference or stale revision
- **THEN** status exposes exact support and Rust/TypeScript/MCP agree on accepted structure, canonical failure code/retryability and unchanged authoritative state

### Requirement: Compatible audiovisual review tool
MCP SHALL expose the additive `preview_review_range` tool with resolution `540p|720p|project|{width,height}`, default project, optional fps, and includeAudio default true. Explicit false SHALL opt out. Legacy `preview_render_range` SHALL retain its existing required custom resolution/fps and audio-off omission behavior. Headless SHALL expose additive `render_review_range` with the same typed resolution selection, project selection/fps defaults and audio omission true. Existing `render_preview_range` SHALL preserve its required explicit fields, audio choices, and historically broader positive-only numeric dimension acceptance. Existing job envelopes, cancellation and error retryability SHALL remain unchanged.

#### Scenario: Queue default and selected reviews
- **WHEN** a client submits default, preset, or complete custom input to preview_review_range
- **THEN** the bridge queues the existing range workflow with typed selections and audio true unless explicitly false

#### Scenario: Preserve legacy input
- **WHEN** a client submits a previously valid legacy MCP or headless custom request
- **THEN** dimensions, frame rate, audio selection and job response semantics remain unchanged

#### Scenario: Reject malformed selection
- **WHEN** a caller supplies unknown preset, unknown fields, incorrect types, partial custom selection, or conflicting preset/custom fields
- **THEN** schema/deserialization errors use existing `INVALID_ARGUMENT` behavior and canonical semantic conflicts use `VALIDATION_FAILED`, with no persisted mutation or output

### Requirement: Discoverable review preset support
Ready rendering status SHALL report `preview_review_presets_v1` in subsystem and aggregate headless/MCP capability lists without changing protocol major1. Unready rendering SHALL omit it. Canonical catalogs and cross-language fixtures SHALL govern the new tool, request fields, defaults and capability.

#### Scenario: Discover ready and unavailable review
- **WHEN** status is requested with rendering ready or unavailable
- **THEN** the new capability is present only when ready and existing readiness errors/capabilities remain unchanged

### Requirement: Reconciled animation and artifact delivery contracts
The bridge MUST preserve schema31 source-clock/Scalar/Pack animation, existing speech timestamp capabilities and version2 artifact delivery together in the combined public contract. Metadata-first job status and its status resource MUST retain optional opaque job-owned artifact descriptors without eager binary reads; explicit PNG/WAV inline opt-in and PNG/WAV/MP4 owned resource reads MUST retain the imported artifact-resources compatibility, confinement, authentication, retention and safe failure requirements. Artifact delivery MUST NOT alter headless/provider contracts, project schema, revisions, animation sampling or generated-artifact commit handles. Canonical catalogs, ownership consumers and live discovery MUST describe this complete union without discarding a contract lane.

#### Scenario: Preserve animation and speech job delivery across reconciliation
- **WHEN** clients use retained-clock editing/render jobs and speech timestamp jobs on the combined branch
- **THEN** animation and speech results retain their established structured metadata while completed output status defaults to metadata/resource links and explicit supported binary retrieval preserves identical bytes

#### Scenario: Preserve artifact failures without project mutation
- **WHEN** clients request expired, foreign, malformed, unsafe or missing artifact resources, or poll metadata after a stale animation edit
- **THEN** existing safe artifact error codes and revision-conflict semantics remain unchanged, no unrelated bytes or tokens are disclosed, and project/history/revision remain unchanged

#### Scenario: Preserve all canonical and workflow parity coverage
- **WHEN** the combined contract, unit, real-MCP and packaged gates execute
- **THEN** live schema/capability/resource output matches the complete reviewed catalog and all existing animation, speech and artifact cases execute with unchanged assertions, individual timeout values and protected CI budgets

### Requirement: Typed discoverable root master normalization workflows
MCP SHALL expose unique audio_master_set_normalization and the same closed controls in existing typed standalone/batch/draft edits, delegating every model/clock/reference/atomicity rule to core. Status SHALL report schema44/protocol1 and truthful conditional audio_master_normalization_v1 rendering support; backend-unavailable editing remains usable. Current source and isolated release-package workflows MUST cover discovery, settings/history/draft/reopen and canonical failures while every older tool/request/catalog/pin remains preserved through explicit successor rollback.

#### Scenario: Exercise actual standalone batch draft and history clients
- **WHEN** actual source and isolated release-package clients discover and edit normalization controls through standalone/batch/draft workflows and undo/redo/reopen
- **THEN** typed normalized controls and current44 reporting agree across transports and stale/invalid/late-batch failures preserve unchanged state/history

#### Scenario: Distinguish editing and active rendering availability
- **WHEN** an active normalization filter is unavailable or a real normalized render/analysis job succeeds or is cancelled
- **THEN** capability reporting, canonical active failure, owned job/resource cleanup and unaffected editing/inactive rendering remain truthful and compatible

### Requirement: Complete bounded MCP conformance validation
Source and packaged MCP conformance clients MUST retain full published JSON Schema output validation and all workflow assertions. Any allocation correction SHALL be justified by phase/process profiling and preserve invalid-output rejection, format checks, unions, nested constraints and exact schema identity. It MUST NOT skip workflows, disable validation, increase memory limits to hide exhaustion or claim a production leak without evidence.

#### Scenario: Execute complete source and package evidence
- **WHEN** source and packaged clients exercise every existing workflow
- **THEN** all results are schema-validated and all original assertions execute within the supported host's resource budget

#### Scenario: Reject malformed output after an allocation correction
- **WHEN** output violates required fields, additional-property policy, formats, a union branch or a nested constraint
- **THEN** validation still rejects it and clients do not treat it as successful

### Requirement: Settled preview cancellation and immutable review revisions
The bridge MUST keep every admitted unsettled producer charged to job capacity even after visible cancellation or TTL expiry, reject excess admission with retryable JOB_REGISTRY_FULL and await those producers during close. Cancellation SHALL remain immediate and idempotent, preserve JOB_CANCELLED, protect noncancellable commit phases, ignore late progress/completion and dispose cancelled late preview outputs. Requested project/revision MUST remain immutable through edits, undo/redo and reopen; retained older output SHALL remain explicitly tagged with its original revision. Stale dispatch MUST preserve canonical REVISION_CONFLICT without publishing output or changing project/history. Public schemas, capabilities, error catalogs and persistence SHALL remain unchanged.

#### Scenario: L1 Retain unsettled cancellation capacity
- **WHEN** cancelled work has not settled and further admission, expiry or shutdown occurs
- **THEN** capacity cannot be reclaimed, close awaits the producer and late progress/results cannot revive the job

#### Scenario: L2 Preserve revision and independent work
- **WHEN** a project changes while preview work or a retained result exists, or another preview is cancelled
- **THEN** completed output retains its requested revision, independent work remains usable and stale requests fail canonically without authoritative mutation

### Requirement: Confirm overflow termination before temporary cleanup
One-shot headless overflow cancellation, timeout and malformed-protocol failure MUST terminate the request process tree before deleting its owned temporary outputs. Unconfirmed termination MUST leave temporary outputs untouched until observed exit. Cleanup MUST refuse unsafe request/project identifiers and export paths rather than deriving deletion paths outside configured roots. Persistent and overflow calls SHALL remain independently cancellable and preserve existing safe typed failures.

#### Scenario: L3 Cancel an overlapping one-shot renderer
- **WHEN** an overflow renderer is cancelled or times out while creating temporary outputs
- **THEN** cleanup follows confirmed process termination, no producer recreates cleaned temporary files and unrelated renders/final outputs remain intact
