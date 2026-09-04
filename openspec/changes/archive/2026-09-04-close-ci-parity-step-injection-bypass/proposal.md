## Why

The parity policy validates named authoritative steps but accepts additional steps and inherited execution defaults. A preceding step can write `OPENCUT_UPDATE_GOLDENS=1` to `GITHUB_ENV`, causing native conformance to replace reviewed references instead of comparing them while the structural validator still succeeds.

## What Changes

- Require the contract- and render-parity jobs to contain only their reviewed steps in the exact approved order.
- Require every reviewed step to contain only its approved properties, preventing custom shells or alternate action/run behavior.
- Reject workflow- or leaf-job-level `defaults.run` and leaf-job containers that can wrap commands, relocate execution, or inject environment variables.
- Preserve unrelated workflow and leaf-job environment variables except for the already forbidden golden mutation modes.
- Add structural regression tests for `GITHUB_ENV`, contract/fixture rewriting steps, duplicate or reordered steps, custom shells, inherited defaults, and container environments.
- Document that changes to either leaf sequence require a coordinated policy and specification update.
- Non-goals: change the valid workflow, public contracts, renderer behavior, fixtures, golden references, persisted data, check names, or GitHub administration.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `contract-governance`: Require the contract-parity gate to use a closed, exact authoritative step sequence.
- `render-regression-fixtures`: Require the render-parity gate to reject execution wrappers and environment-persisting steps that can bypass reviewed golden comparison.
- `repository-validation`: Require structural validation of exact leaf sequences and every effective command-execution surface.

## Impact

- Affects `scripts/validate-ci-gates.ts`, its focused tests, CI parity documentation, and the three listed living specifications.
- The checked-in workflow remains unchanged. No application code, dependency, contract, fixture, schema, or runtime behavior changes.
