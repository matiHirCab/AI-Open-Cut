## Why

The workflow policy validator runs in `OpenSpec validation`, but the documented single branch-protection target currently observes only contract and render parity. A structural mutation can therefore fail policy validation while both parity leaves and the aggregate still report success. The aggregate must propagate the policy result so every detected weakening reaches the protected status.

## What Changes

- Make `foundation-parity` depend on OpenSpec validation in addition to contract and render parity.
- Log and require exact `success` results from all three dependencies.
- Treat the OpenSpec job as an exact reviewed policy boundary and reject execution-altering configuration or steps.
- Extend structural and result-matrix tests for policy failure, cancellation, skip, and job weakening.
- Keep the stable job names and the single aggregate branch-protection target.
- Non-goals: change repository-host settings, public contracts, renderer behavior, fixtures, goldens, paths, or persisted data.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `repository-validation`: Propagate structural policy validation into the stable aggregate status and govern the policy job itself.

## Impact

- Affects the CI workflow, policy validator, focused tests, parity documentation, and repository-validation specification.
- Contract and render jobs remain independent and continue running in parallel with OpenSpec validation.
