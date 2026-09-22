## Why

Issue #36 has passing local native cache evidence, but CI omits the instrumented headless/bridge tests and silently returns from the cache audiovisual test without native configuration. A green foundation status therefore does not enforce the new conformance evidence.

## What Changes

- Extend the required Render parity job to execute native core cache conformance, instrumented headless reuse and actual bridge-to-native reuse with required-mode flags.
- Install the existing locked bridge dependencies in that job, rebuild headless without hooks after instrumented checks, and verify default transport compatibility.
- Update the closed workflow policy and mutation tests together; retain exact commands, environments, ordering, failure propagation and existing report validation/upload.
- Document local reproduction and distinguish Windows verification from remote Linux CI execution.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `render-regression-fixtures`: Expand the reviewed render-parity sequence to include mandatory cache conformance and default-build restoration.
- `repository-validation`: Protect the new native commands, required flags, dependency setup and build ordering against weakening.

## Impact

Affected files are `.github/workflows/bun-ci.yml`, `scripts/validate-ci-gates.ts`, its policy regression tests, and CI/render verification documentation. Existing rendering tests and runtime behavior remain unchanged. No public contract, persisted schema, dependency version or editor-core ownership edge changes; no migration is needed. Existing status names and the foundation aggregate remain compatible.

## Non-goals

Renderer or worker behavior changes, new test hooks, golden updates, timing thresholds, a new CI platform matrix, remote execution or publication, commits and PR creation.

## Approval

The user explicitly approved the proposal, design, both delta specifications and tasks with "yes" on 2026-09-21. Implementation and the listed verification/archive lifecycle are authorized through these tasks.
