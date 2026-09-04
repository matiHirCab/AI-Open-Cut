## Why

Blocking only `BASH_ENV` leaves other inherited process controls able to replace or preempt reviewed commands. For example, workflow- or parity-job-level `LD_PRELOAD` can execute a checked-in library whenever Node or Bash starts, while `PATH` can redirect command resolution. The checked-in workflow requires no inherited parity environment, so an open-ended distinction between benign and dangerous variables creates risk without providing current functionality.

## What Changes

- Reject any workflow-level `env` declaration and any job-level `env` declaration on contract, render, or foundation parity, including empty maps.
- Preserve the exact approved step-level environments used by native conformance, report validation, and aggregate result assertion.
- Continue permitting environment configuration on jobs outside the protected parity boundary.
- Add structural regression coverage for loader hooks, runtime options, path redirection, benign-looking keys, expressions, and empty maps.
- Document that future parity variables must be approved at exact step scope.
- Non-goals: inspect actions, scripts, packages, or validator source as hostile code; change the workflow, public interfaces, renderer, fixtures, goldens, commands, paths, check names, or GitHub administration.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `contract-governance`: Isolate contract parity from workflow- and job-level environment inheritance.
- `render-regression-fixtures`: Isolate render parity while retaining only its exact step-level deterministic environment.
- `repository-validation`: Enforce an empty inherited-environment boundary for every parity job.

## Impact

- Affects the CI policy validator, its focused tests, parity documentation, and the three listed living specifications.
- The valid workflow remains unchanged because it declares no workflow- or parity-job-level environment.
