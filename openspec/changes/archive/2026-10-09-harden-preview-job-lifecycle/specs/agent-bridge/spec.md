## ADDED Requirements

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
