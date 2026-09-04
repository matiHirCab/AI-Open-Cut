## Why

The policy validator currently runs inside the Moon task whose effective configuration it validates. A root `BASH_ENV` override is therefore applied before validation and can neutralize every reviewed command while forging the expected GitHub output.

## What Changes

- Bootstrap the policy check outside Moon and validate the complete workflow, Moon, and proto boundary before invoking any Moon task.
- Install the exact Bun and Moon bootstrap versions without reading pull-request configuration first.
- Keep `GITHUB_OUTPUT` out of the Moon child process and let only the bootstrap emit the completion attestation after a successful child exit.
- Validate the exact `.prototools` configuration and protect every policy surface with CODEOWNERS.
- Preserve the stable CI job names and aggregate branch-protection target.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `repository-validation`: Require pre-execution validation and bootstrap-owned policy attestation for the complete Moon/proto execution boundary.

## Impact

- Affects the OpenSpec CI job, root Moon task, structural validator, a new internal bootstrap runner, policy tests, CODEOWNERS, and CI documentation.
- Public APIs, contracts, schemas, renderer behavior, fixtures, goldens, and persisted data remain unchanged.
