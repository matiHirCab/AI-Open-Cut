# Agent Bridge Specification

## Purpose

Define the typed automation boundary over editor-core, including MCP exposure, transports, diagnostics, jobs, and stable errors.

## Requirements

### Requirement: Typed headless boundary
The bridge MUST invoke a typed, process-per-request headless boundary that accepts one discriminated JSON-lines request contract, emits structured progress, result, or error events, and delegates domain and persistence behavior to editor-core.

#### Scenario: Execute a valid headless request
- **WHEN** the bridge sends a supported typed request to the headless process
- **THEN** the process emits schema-compatible events and does not duplicate domain mutation rules in the transport

#### Scenario: Time out a headless request
- **WHEN** a headless request exceeds its configured deadline
- **THEN** the bridge terminates that child process, removes owned preview output, and returns retryable `HEADLESS_TIMEOUT`

### Requirement: MCP capability exposure
The bridge SHALL expose project, asset, timeline, draft, render, speech, transcription, and job workflows as validated MCP tools, with project context available through registered resources and reusable prompts.

#### Scenario: Discover automation capabilities
- **WHEN** an MCP client lists tools, resources, and prompts
- **THEN** it can discover the registered editing workflows and their validated input and output contracts

#### Scenario: Reject invalid MCP input
- **WHEN** a client calls a tool with input that does not satisfy its published schema
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
