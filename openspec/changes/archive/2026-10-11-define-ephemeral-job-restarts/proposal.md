## Why

Issue #85 requires truthful job restart semantics after the measured renderer (#15) and bounded process-local lifecycle (#73), both merged into current main. Jobs already lose all state on restart, but clients lack discovery/reconciliation guidance and repeated crashes can leave private files outside process-local accounting.

## What Changes

- Keep jobs ephemeral; document identifiers, ownership, revision binding, expiry, cancellation, retained results and uncertain commits in an ADR based on existing renderer observations and new local job/startup measurements.
- Advertise additive `ephemeral_jobs_v1` through bridge status and health, with a canonical versioned behavior fixture and parity coverage.
- Bound repeated crash accumulation with an exclusive runtime admission marker, acquired before the first producer. An occupied or unsafe marker prevents new jobs with existing retryable `JOB_REGISTRY_FULL`, while reads and project reconciliation remain available. Remove only the owned marker after successful graceful job/provider cleanup; never auto-adopt or delete predecessor artifacts.
- Test actual process death and cold registries for all five states, cancellation, capacity, isolation, cleanup failures, and graceful recovery. Document an explicit operator recovery procedure.

## Capabilities

### New Capabilities
- `job-restart-semantics`: Ephemeral discovery, restart reconciliation and conservative crash admission.

### Modified Capabilities
None. Existing process-local lifecycle, errors and ownership remain applicable.

## Impact

Bridge orchestration/jobs, bridge-only capability reporting, canonical contracts and ownership, tests, ADR and client guidance. The capability identifier is additive; no request/schema/error identifier changes, project migration, durable job database or automatic retry. Crash admission now fails closed until an operator verifies termination and reconciles/quarantines orphan files. Same authorized HTTP runtime remains a shared trust boundary; separate private clients must use separate processes and runtime roots.

## Non-goals

Resumption, cross-client artifact adoption, automatic stale PID detection/deletion, renderer/core changes, transport authentication redesign, exports garbage collection, desktop/video work, deployment or merge. The marker protects process-crash admission; it is not a disk quota or host-power-loss journal.

## Approval

Issue-scoped specification approved under the user's explicit standing authorization in the delegated request, before implementation. Design and tasks must remain within #85.
