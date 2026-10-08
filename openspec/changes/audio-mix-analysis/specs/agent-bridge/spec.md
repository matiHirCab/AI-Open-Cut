## MODIFIED Requirements

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
