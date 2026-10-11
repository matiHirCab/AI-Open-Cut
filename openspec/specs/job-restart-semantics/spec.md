# Job Restart Semantics Specification

## Purpose

Define discoverable ephemeral bridge job restart behavior, conservative private crash admission, truthful cleanup and authoritative-state reconciliation before retry.

## Requirements

### Requirement: Discoverable ephemeral job restart contract
The bridge SHALL advertise `ephemeral_jobs_v1` in MCP editor status and CLI health, governed by a versioned behavior fixture. Jobs SHALL retain opaque random identifiers, immutable originating project/revision and `persistence: process`; their metadata, results, artifact links and generated commit tokens SHALL NOT be rehydrated after restart. Terminal expiry, cancellation, protected commits and bounded retention SHALL preserve their existing semantics and error codes. Cold lookup/cancellation/resources SHALL return JOB_NOT_FOUND for predecessor identifiers whether work was queued, running, completed, failed or cancelled. A missing job SHALL NOT imply that an authoritative project commit or final export failed.

#### Scenario: Discover additive restart semantics
- **WHEN** an existing client reads bridge editor status or CLI health
- **THEN** the additive capability identifies ephemeral jobs without changing protocol, project schema, headless capabilities or existing job/error envelopes

#### Scenario: Restart every job state
- **WHEN** the bridge dies with provider-queued, running, completed, failed or cancelled jobs and a new registry starts
- **THEN** predecessor IDs and resources are unavailable, no private result/token is adopted and new IDs remain distinct

#### Scenario: Reconcile uncertain commit before retry
- **WHEN** a client loses a job acknowledgement around an atomic mutation or export
- **THEN** published guidance requires authoritative state/revision/history or approved export inspection before retry and does not promise resumability or exactly-once execution

### Requirement: Conservative bounded crash admission
Before a production job can invoke its producer, the runtime SHALL acquire one exclusive private admission guard in its configured work root. An occupied or unsafe guard SHALL reject producer admission with existing retryable JOB_REGISTRY_FULL without reading predecessor private outputs, while status and project reconciliation remain available. Cancelled speech output SHALL remain owned until worker exit is confirmed and final cleanup succeeds, including a worker that ignores graceful termination. Only the owning runtime SHALL remove its unchanged guard after successful job and provider cleanup; cleanup failure or process death SHALL retain it. Unconfirmed speech worker shutdown or remaining owned speech cleanup debt SHALL fail shutdown with safe VALIDATION_FAILED and permit cleanup retry without reopening producer admission. Repeated crashes/restarts without operator reconciliation SHALL admit no additional producers or grow guard inventory. Existing #73 in-process bounds SHALL remain applicable; committed assets, approved exports and interrupted native workspace sizes SHALL NOT be represented as a disk quota. Doctor synthesis SHALL use the same admission protection; failed doctor fixture deletion SHALL report safe VALIDATION_FAILED rather than permit successful guard release.

#### Scenario: Repeated process crashes fail closed
- **WHEN** a process dies after acquiring its guard and successors repeatedly attempt jobs
- **THEN** one guard remains, all attempts fail before task execution and private predecessor files remain untouched

#### Scenario: Graceful cleanup permits fresh work
- **WHEN** all owned producers and provider cleanup successfully finish on close
- **THEN** the owning guard is removed and a fresh runtime can admit work with new IDs

#### Scenario: Unsafe or failed cleanup preserves ownership
- **WHEN** guard root is a symlink, a foreign guard exists, an owned guard is replaced or cleanup fails
- **THEN** admission/removal fails safely without exposing private paths or deleting foreign files, and failed cleanup does not release the guard

#### Scenario: Preserve transport and private runtime isolation
- **WHEN** private clients use distinct bridge processes/work roots or an HTTP caller lacks valid authentication
- **THEN** they cannot observe/adopt another runtime's job output and existing HTTP authorization remains enforced; explicitly shared authorized HTTP runtime access remains unchanged
