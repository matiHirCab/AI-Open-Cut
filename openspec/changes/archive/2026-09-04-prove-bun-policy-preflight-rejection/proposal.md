## Why

The real-Bun regression proves that the hardened invocation avoids a malicious preload, but its temporary workspace omits the protected workflow and Moon/proto sources. The process can therefore fail before it validates the malicious `bunfig.toml`, leaving the intended preflight path unproven.

## What Changes

- Provide every canonical policy source except the intentionally malicious `bunfig.toml` in the integration workspace.
- Point the bootstrap at the real reviewed workflow and require the specific Bun-configuration rejection.
- Isolate the nested real-Moon reproduction from Moon/proto metadata and stores inherited from the protected parent task, with a bounded Ubuntu startup budget.
- Correct the upgrade guide to describe the workflow-to-bootstrap-to-Moon execution chain.
- Preserve all production policy behavior, status identities, fixtures, and branch-protection configuration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `repository-validation`: Require the real-Bun regression to isolate the malformed Bun configuration from unrelated missing-source failures and prove its rejection before Moon starts.

## Impact

The change affects only policy regression coverage, documentation, and repository-validation evidence. It changes no workflow, bootstrap, validator, application API, contract, schema, renderer, fixture, golden, or persisted data.
