# Why

`OpenSpec validation` currently governs its own workflow structure, but `continue-on-error` can convert the validation step's failure into a successful job conclusion. The protected aggregate observes only `needs.openspec.result`, so it can accept a policy job that did not complete its authoritative command successfully.

# What Changes

- Emit a policy-success attestation only after the authoritative OpenSpec command completes successfully.
- Make the foundation aggregate require that attestation in addition to exact-success job results.
- Structurally validate the exact policy step identifier, command order, job output, aggregate binding, and assertion.
- Add regression coverage for ignored failures, skipped or neutralized execution, and altered or forged attestations.
- Keep all visible job names and the single aggregate branch-protection target stable.
- Non-goals: change repository-host settings, public contracts, renderer behavior, fixtures, goldens, paths, or persisted data.

# Capabilities

## New Capabilities

- None.

## Modified Capabilities

- `repository-validation`: Require proof that structural policy validation actually reached successful completion before the foundation aggregate can pass.

# Impact

- Affects the CI workflow, structural policy validator, focused tests, parity documentation, and repository-validation specification.
- Adds only an internal job output and aggregate input; application and public interfaces remain unchanged.
