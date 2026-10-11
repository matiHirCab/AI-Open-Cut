## Why

An ordinary unsuccessful `--doctor` run currently retains the exclusive job-runtime marker even when no producer ran and all shutdown cleanup succeeded. Missing optional models then block unrelated jobs and future doctor attempts with `JOB_REGISTRY_FULL` until manual reconciliation.

## What Changes

- Separate completed diagnostics from aggregate subsystem readiness when deciding whether doctor may release its owning guard.
- Preserve the guard when doctor throws, cleanup fails, termination is unconfirmed, or ownership is unsafe.
- Distinguish writeability-probe deletion failure from ordinary unavailable-directory diagnostics and settle all probes before judging cleanup.
- Add actual CLI and controlled shutdown regression coverage, retaining crash/isolation tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `job-restart-semantics`: explicitly require clean unsuccessful diagnostic runs to release only their own guard.

## Impact

Bridge CLI orchestration, owned doctor probe cleanup, doctor lifecycle tests and job restart guidance only. No public schema, capability, stable error, canonical fixture, provider wire protocol, persistence or project revision change; this corrects implementation of the existing successful-cleanup invariant. No migration is required and old clients remain valid.

## Non-goals

No automatic crash takeover, predecessor artifact adoption/deletion, durable jobs, existing-marker removal, renderer/desktop changes, #181 work or private audit upload.

## Authorization

The user's follow-up explicitly authorizes fixing the audited defect, regression tests and a separate draft PR. Standing authorization covers specification, implementation, verification and push; no merge or deployment is authorized.
