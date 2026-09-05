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
