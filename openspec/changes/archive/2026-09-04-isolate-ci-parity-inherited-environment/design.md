## Context

The current policy rejects two golden modes and `BASH_ENV` while accepting every other workflow- and parity-job-level environment key. That denylist accepts `LD_PRELOAD`, `LD_AUDIT`, `NODE_OPTIONS`, and `PATH`, all of which can influence process startup or command resolution without changing an approved step body. The valid workflow has no inherited environment at either protected scope.

## Goals / Non-Goals

**Goals:**

- Prevent all workflow- and parity-job-level environment inheritance into protected commands.
- Preserve exact reviewed step environments and non-parity job configuration.
- Retain specific golden-mode diagnostics where they already exist.
- Require deliberate specification and policy updates for future parity variables.

**Non-Goals:**

- Treat actions, package scripts, command implementations, or the validator itself as untrusted code.
- Change commands, step environments, check identities, branch protection, application behavior, or persisted evidence.

## Decisions

### Use an empty inherited-environment allowlist

Reject the presence of `workflow.env` and `jobs.<parity>.env`, including empty maps, rather than extending a denylist. This closes both known and future process-control variables without inventing unused benign keys. Environment maps on non-parity jobs remain outside this policy.

At workflow and render-job scope, run the existing golden-mode check first so declarations of `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` retain their specific diagnostic. All other inherited maps fail with a scope-specific isolation error.

### Preserve exact step environments

Continue requiring the current five native-conformance variables, one report-validation variable, and two aggregate-result variables. A future parity setting must be added only to the exact step that consumes it and coordinated across specification, validator, workflow, tests, and documentation.

## Risks / Trade-offs

- **Global metadata can no longer be declared at workflow scope.** Move it to each non-parity job that needs it, or explicitly approve a step-scoped parity value.
- **Even an empty map is rejected.** This keeps the structural rule unambiguous and prevents later keys from being added under an accepted container.

## Migration Plan

Land policy, tests, documentation, and specifications together. No workflow migration is needed. Rollback affects only policy artifacts.

## Open Questions

None.
