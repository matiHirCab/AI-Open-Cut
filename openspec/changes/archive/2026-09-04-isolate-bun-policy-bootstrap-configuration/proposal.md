## Why

The policy bootstrap currently starts through repository-controlled Bun configuration before its structural preflight, so a `bunfig.toml` preload can forge the required output and exit successfully without running the bootstrap. The real-Moon regression that exercises the related startup-hook threat also exists only as an input and is not executed by the protected CI task.

## What Changes

- Start the policy bootstrap with Bun configuration and automatic dotenv loading disabled.
- Validate the repository's canonical `bunfig.toml` before Moon starts, and run every protected Bun command with that reviewed config and dotenv loading disabled.
- Execute the real-Moon startup-hook regression inside the protected Ubuntu policy task.
- Protect the Bun configuration alongside the existing CI policy boundary and document the expanded pre-execution trust model.
- Preserve all public application behavior, parity job identities, policy output names, and branch-protection configuration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Require the policy bootstrap and protected Moon task to exclude unvalidated Bun configuration and dotenv loading, validate the canonical Bun configuration, and execute the real-Moon regression in CI.

## Impact

The change affects the OpenSpec workflow command, the root Moon task, the CI bootstrap/validator and their tests, CODEOWNERS, and CI policy documentation. It adds no API, contract, schema, renderer, fixture, golden, or persisted-data changes and introduces no breaking public compatibility change.
