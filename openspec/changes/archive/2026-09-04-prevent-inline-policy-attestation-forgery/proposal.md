# Why

The OpenSpec workflow step currently emits its success attestation immediately after the validation command in the same mutable shell body. Changing that command to `moon run openspec-validate || true` masks the validator failure and still executes the output write, allowing the protected aggregate to receive both a successful job result and a forged `true` attestation.

# What Changes

- Restore the workflow policy step to the sole canonical `moon run openspec-validate` command.
- Emit the attestation only from the final structural-validator invocation inside the reviewed Moon task.
- Validate the exact Moon task command sequence as part of repository CI policy.
- Add regressions for inline output writes, masked commands, altered task ordering, and suppressed attestation.
- Preserve stable job names and the single aggregate branch-protection target.
- Non-goals: prevent deliberate replacement of the complete verifier and output producer without an external trusted workflow, or change application and persisted interfaces.

# Capabilities

## New Capabilities

- None.

## Modified Capabilities

- `repository-validation`: Bind the policy completion proof to the full reviewed Moon task rather than to mutable inline workflow shell code.

# Impact

- Affects the CI workflow, Moon validation task, structural validator, focused tests, parity documentation, and repository-validation specification.
- Adds one internal validator CLI flag; public contracts, fixtures, goldens, renderer behavior, and data remain unchanged.
