## Why

The #49 protocol capability test unconditionally requires a ready renderer, so the hermetic Contract parity job fails when media executables are legitimately unavailable. Production reporting already gates rendering capabilities correctly; the test must verify both readiness outcomes while retaining fail-closed required native evidence.

## What Changes

- Validate returned boolean readiness and exact canonical capability lists in ordinary protocol tests, with canonical rendering capabilities present only when ready and stable nonretryable DEPENDENCY_UNAVAILABLE when unavailable.
- Under OPENCUT_GOLDEN_REQUIRED=1 additionally require ready=true; never skip the assertion or weaken native conformance.
- Exercise controlled unavailable executables and configured required native tools; retain the explicit missing-renderer health regression.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rendering-export`: Readiness-aware hermetic protocol conformance and mandatory native readiness evidence for linear composition support.

## Non-goals

No production renderer, reporter, contract catalog, capability, schema, workflow, tolerance or toolchain changes. No activation of #50 mask models; all #50 work remains stashed and paused.

## Impact

Only the readiness expectation in apps/headless/tests/protocol.rs changes. This corrects test-environment assumptions without changing protocol-v1 requests/responses or the approved #49 runtime semantics. Required local contract/Rust checks and native configured support evidence remain mandatory; independent review, archival and unchanged protected policy validation precede parent publication.
