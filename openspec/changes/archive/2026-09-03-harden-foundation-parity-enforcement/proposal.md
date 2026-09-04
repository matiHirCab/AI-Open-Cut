## Why

The new aggregate foundation status is skipped when either leaf parity job fails, and GitHub treats skipped jobs as successful required checks. The workflow-policy validator also accepts critical steps weakened with `continue-on-error` or reordered so publication precedes validation, so the gate does not yet enforce the reviewed merge boundary.

## What Changes

- Make the aggregate foundation job run after every leaf outcome and fail explicitly unless both parity jobs report `success`.
- Strengthen structural workflow validation around exact critical steps, execution order, working directories, failure propagation, and report publication.
- Add negative tests for skipped, failed, cancelled, neutralized, reordered, and otherwise weakened gate configurations.
- Clarify contributor documentation for aggregate result handling and the stable branch-protection status.
- Non-goals: change public contracts, canonical fixtures, renderer behavior, golden references or thresholds, persisted data, tool versions, stable job identities, or repository-host branch-protection settings.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `contract-governance`: Require the dedicated contract command to remain fail-closed and structurally non-neutralizable.
- `render-regression-fixtures`: Require native conformance, report validation, and publication to remain ordered and fail-closed.
- `repository-validation`: Require the aggregate status to execute for every leaf outcome and fail unless both leaves succeed, with policy evidence for that behavior.

## Impact

- Affects `.github/workflows/bun-ci.yml`, `scripts/validate-ci-gates.ts`, its focused tests, contributor documentation, and the three listed living specifications.
- Adds no production dependency and changes no runtime, public, persisted, fixture, rendering, or ownership contract.
