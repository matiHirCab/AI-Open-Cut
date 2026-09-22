## MODIFIED Requirements

### Requirement: Typed headless boundary
The bridge MUST invoke the typed headless boundary, delegating domain and persistence behavior to editor-core and exposing the supported public protocol version through status negotiation. Non-render requests MUST retain process-per-request execution. Frame preview, audiovisual range preview, materialized draft preview and export SHALL use one reusable render-only worker per bridge client when it is available; overlapping render requests MUST use independent one-shot processes without waiting for that worker. Existing single-request CLI input, structured progress/result/error events, health behavior and exit semantics MUST remain compatible.

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

## ADDED Requirements

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
