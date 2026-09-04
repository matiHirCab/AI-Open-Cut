# Why

The policy validator checks only `tasks.openspec-validate` inside `moon.yml`. Moon can still inject project-wide environment variables or inherited task configuration before that task runs, so `BASH_ENV` or a forged `PATH` can neutralize every reviewed command and fabricate the GitHub output without changing the validated task body.

# What Changes

- Address the protected task explicitly as `root:openspec-validate`.
- Isolate the root project from inherited tasks and reject root-level Moon execution overrides.
- Validate the stable workspace and toolchain configuration plus every global task configuration that could affect the root project.
- Reject global task environments and external configuration extension while preserving global tasks for non-root projects.
- Emit no policy attestation until the complete Moon execution boundary has passed validation.
- Preserve stable check names and the aggregate branch-protection target.

# Capabilities

## New Capabilities

- None.

## Modified Capabilities

- `repository-validation`: Include the effective Moon project, workspace, toolchain, and inherited-task configuration in the attested policy boundary.

# Impact

- Affects the CI workflow, root Moon configuration, structural validator, focused tests, parity documentation, and repository-validation specification.
- Public APIs, contracts, renderer behavior, fixtures, goldens, and persisted data remain unchanged.
