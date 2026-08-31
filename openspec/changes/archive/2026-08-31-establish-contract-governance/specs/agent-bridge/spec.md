## MODIFIED Requirements

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
