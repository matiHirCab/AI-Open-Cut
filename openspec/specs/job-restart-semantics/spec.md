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

### Requirement: Doctor readiness is independent from cleanup ownership
The doctor CLI SHALL release only its unchanged owning runtime guard after diagnostics return and all owned runtime shutdown cleanup succeeds, even if its report is unready because of unavailable subsystems, optional models or diagnostic prerequisites. It SHALL preserve existing unsuccessful diagnostic reports and exit codes. A thrown doctor cleanup failure, failed service shutdown, unconfirmed worker termination, process crash or unsafe/foreign ownership SHALL retain the guard without admitting new producers or deleting foreign artifacts. This correction SHALL preserve all existing job envelopes, capabilities, stable errors, provider contracts, project revisions and crash-reconciliation semantics.

#### Scenario: Complete unsuccessful diagnostics with clean shutdown
- **WHEN** doctor returns an unready report and every runtime service closes successfully
- **THEN** doctor exits unsuccessfully but removes only its own guard, and a fresh runtime using that root can admit an unrelated job

#### Scenario: Complete successful diagnostics with clean shutdown
- **WHEN** doctor returns a ready report and all runtime services close successfully
- **THEN** doctor exits successfully and releases only its own guard

#### Scenario: Preserve ownership after diagnostic cleanup failure
- **WHEN** doctor fixture or writeability-probe cleanup fails, including a later probe after an earlier ordinary write error
- **THEN** unsuccessful shutdown preserves its guard for operator reconciliation even if provider close succeeds

#### Scenario: Preserve ownership after service cleanup failure
- **WHEN** headless, speech or transcription close fails after diagnostics returned
- **THEN** doctor exits unsuccessfully and retains its guard, and a fresh runtime cannot invoke a producer

#### Scenario: Preserve a predecessor guard
- **WHEN** doctor starts in a root containing an existing foreign guard
- **THEN** admission fails with the existing safe retryable error and the foreign marker and private artifacts remain unchanged

### Requirement: Bounded real crash verification
Private restart verification MUST kill and settle each actual predecessor before checking cold successors, preserve every job-state/admission assertion, and complete within unchanged test and helper time limits.

#### Scenario: Repeated real predecessor crashes
- **WHEN** verification exercises queued, running and terminal predecessor jobs followed by repeated cold admission attempts
- **THEN** complete fixture records and confirmed predecessor death support all existing guard inventory and lookup assertions without additional retries or larger limits

#### Scenario: Prepare a self-contained crash fixture
- **WHEN** real crash verification prepares the original fixture and production dependencies
- **THEN** a separately bounded5000ms setup requires a successfully built self-contained JavaScript bundle, every existing successor starts cold with real production behavior, and unavailable or failed preparation fails without source fallback or larger controlled limits

#### Scenario: Fixture fails or the test ends during a live child
- **WHEN** setup, output completeness, launch or the existing watchdog fails, or a test ends with an owned fixture child
- **THEN** verification fails without fallback, closes owned child processes before deleting roots, and refuses successor launch during teardown while preserving diagnostics and all existing time bounds
