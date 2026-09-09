# Agent Bridge Specification

## Purpose

Define the typed automation boundary over editor-core, including MCP exposure, transports, diagnostics, jobs, and stable errors.

## Requirements

### Requirement: Typed headless boundary
The bridge MUST invoke a typed, process-per-request headless boundary that accepts one discriminated JSON-lines request contract, emits structured progress, result, or error events, delegates domain and persistence behavior to editor-core, and exposes the supported public protocol version through status negotiation.

#### Scenario: Execute a valid headless request
- **WHEN** the bridge sends a supported typed request to the headless process
- **THEN** the process emits schema-compatible events and does not duplicate domain mutation rules in the transport

#### Scenario: Negotiate the current protocol version
- **WHEN** the bridge sends a status request with the current public protocol version
- **THEN** the process returns status containing that protocol version and its compatible capabilities

#### Scenario: Reject an unsupported protocol version
- **WHEN** the bridge sends a status request naming an unsupported public protocol version
- **THEN** the process returns non-retryable `INVALID_ARGUMENT` without invoking an editor mutation

#### Scenario: Time out a headless request
- **WHEN** a headless request exceeds its configured deadline
- **THEN** the bridge terminates that child process, removes owned preview output, and returns retryable `HEADLESS_TIMEOUT`

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
