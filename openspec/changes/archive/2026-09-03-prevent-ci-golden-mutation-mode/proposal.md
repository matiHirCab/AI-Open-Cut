## Why

The render-parity policy validates required environment keys but currently accepts additional or inherited golden mutation flags. `OPENCUT_UPDATE_GOLDENS=1` switches the native harness from comparing reviewed references to replacing them, allowing a renderer regression to evade the gate that issue #16 established.

## What Changes

- Require exact approved environment maps on native conformance and strict report-validation steps.
- Reject golden update and alternate-capture mode variables at workflow, render job, and critical-step scopes while allowing unrelated global environment configuration.
- Add focused policy tests for every effective environment scope and for harmless global configuration.
- Document that required CI is verification-only and that deliberate update/capture modes remain local workflows.
- Non-goals: change the workflow's commands, job identities, public contracts, renderer behavior, canonical fixtures, golden references or thresholds, persisted data, or GitHub repository settings.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `render-regression-fixtures`: Require the CI render gate to reject mutation and alternate-capture modes from every effective environment scope.
- `repository-validation`: Require structural policy evidence for exact render environment maps and forbidden inherited golden modes.

## Impact

- Affects `scripts/validate-ci-gates.ts`, its focused tests, contributor documentation, and the two listed living specifications.
- The valid CI workflow remains unchanged. No production dependency, public interface, fixture, or runtime behavior changes.
