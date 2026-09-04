## Why

The parity policy closes the two leaf step sequences but still accepts execution controls that can make required checks report success without running their reviewed assertions. A custom `shell` or inherited `defaults.run.shell` can replace the foundation aggregate script with a command that always succeeds, while `BASH_ENV` can run code before GitHub Actions invokes any parity script.

## What Changes

- Apply the reviewed shell, defaults, container, property, and environment policy to the foundation aggregate.
- Require the aggregate assertion step to contain only its current `name`, exact two-result `env`, and fail-closed `run` body.
- Reject inherited `BASH_ENV` at workflow and parity-job scope while continuing to permit benign environment metadata.
- Add regression tests for aggregate execution wrappers, altered aggregate environments, and literal or expression-valued `BASH_ENV` declarations.
- Document the complete parity execution boundary.
- Non-goals: change the valid workflow, public contracts, renderer behavior, fixtures, golden references, persisted data, check names, or GitHub administration.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `contract-governance`: Prevent the contract gate's Bash startup from being redirected through inherited environment configuration.
- `render-regression-fixtures`: Prevent the render gate's Bash startup from being redirected through inherited environment configuration.
- `repository-validation`: Require exact aggregate execution properties and reject inherited Bash startup injection across every parity job.

## Impact

- Affects `scripts/validate-ci-gates.ts`, its focused tests, CI parity documentation, and the three listed living specifications.
- The checked-in workflow remains unchanged. No application code, dependency, contract, fixture, schema, or runtime behavior changes.
