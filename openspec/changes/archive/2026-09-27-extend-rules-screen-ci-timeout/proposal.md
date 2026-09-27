## Why

PR #126's 1920x1080 rules-screen parity job was cancelled after about 2 hours 15 minutes while the native render test was still running. The job log does not establish what initiated the cancellation, but the long-running shard needs an explicit three-hour CI budget so its intended limit is reviewable and stable.

## What Changes

- Set the rules-screen parity matrix job timeout to 180 minutes for each resolution shard.
- Require the CI policy validator to accept exactly that timeout and reject a missing or changed value.
- Add focused policy tests and update the CI gate documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Require an explicit three-hour rules-screen parity job timeout in the protected workflow policy.

## Impact

- Affects `.github/workflows/bun-ci.yml`, `scripts/validate-ci-gates.ts`, its focused tests, and `docs/ci-parity-gates.md`.
- No application, public API, persisted data, migration, or cross-language contract changes. No breaking changes.
- Non-goals: changing render test selection, concurrency, other CI jobs, or behavior when GitHub cancels a run for reasons unrelated to the job timeout.
