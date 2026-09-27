## Why

PR #126's approved 180-minute rules-screen job timeout conflicts with the newer protected CI duration policy on `main`, which pins required jobs and the temporary overall exception to 135 minutes. Resolving only the workflow merge markers would leave CI policy validation failing and would not allow the requested three-hour run.

## What Changes

- Keep the 120-minute default elapsed-time budget, but extend the reviewed temporary exception hard cap to 180 minutes for the long rules-screen shard, with explicit owner, cause, run evidence, and expiry.
- Give only the rules-screen matrix job a 180-minute timeout; retain the existing limits for other jobs and the foundation gate.
- Reconcile the workflow, duration evaluator, structural validator, tests, documentation, and living requirements so all protected checks agree.
- Resolve the four merge conflicts and preserve both branches' independent CI safeguards and other changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Permit a bounded, documented 180-minute rules-screen exception while preserving the 120-minute default budget and all protected gates.

## Impact

- Affects the protected GitHub Actions workflow, `scripts/ci-duration.ts`, `scripts/validate-ci-gates.ts`, their tests, CI documentation, and `openspec/specs/repository-validation/spec.md`.
- No application runtime, public or persisted contract, migration, or cross-language API change. No breaking change.
- Non-goals: changing test selection, skipping a validation job, relaxing failures, or extending the exception indefinitely.
