## Context

The policy checker validates authoritative step bodies and relative order, but it locates those steps by name within an otherwise open list. GitHub Actions makes values written to `GITHUB_ENV` available to subsequent steps, so an added step can activate golden update mode without changing the native step's exact `env` map. Workflow- or job-level `defaults.run`, per-step custom shells, and job containers can similarly wrap, relocate, or inject configuration into the reviewed commands.

The contract gate has the same open-list weakness: an added step can rewrite governed fixtures before the exact parity command runs. This follow-up closes both leaf jobs without changing their valid checked-in definitions.

## Goals / Non-Goals

**Goals:**

- Accept only the currently reviewed contract- and render-parity step sequences.
- Reject added, duplicated, replaced, or reordered leaf steps.
- Reject inherited or per-step execution wrappers and leaf-job containers.
- Preserve benign workflow- and leaf-job-level environment variables that do not alter golden mode.
- Keep policy failures structural and diagnostic.

**Non-Goals:**

- Change the workflow, renderer, native harness, contracts, fixtures, check names, or branch-protection administration.
- Interpret arbitrary shell programs or every GitHub Actions feature.
- Prevent changes to the validator itself from being reviewed as code.

## Decisions

### Validate closed positional step schemas

Require exactly four contract steps and six render steps. Validate each position against an allowlist of property names and its existing exact values. This rejects `GITHUB_ENV` writers, fixture-rewrite steps, duplicates, and custom step shells without attempting unreliable command-text pattern matching.

Scanning extra commands for `GITHUB_ENV` was rejected because shell indirection and actions can mutate the environment without containing that literal text. Continuing to validate only named steps was rejected because it leaves repository mutation before the authoritative commands unrestricted.

### Reject inherited command defaults and leaf containers

Reject any workflow-level `defaults.run` and any `defaults.run` or `container` on the two leaf jobs. These surfaces can change the shell, working directory, operating environment, or inherited variables without appearing in the step object checked by the current validator.

Adding explicit `shell` and working-directory overrides to every workflow step was rejected because the valid workflow must remain unchanged and a closed policy should fail visibly when its execution model changes.

### Preserve unrelated environment configuration

Continue permitting unrelated keys in workflow and leaf-job `env` maps while rejecting the two golden mode variables in their effective render scopes. Exact step property and environment maps remain closed. This preserves the preceding verification-only policy without treating harmless CI metadata as an execution bypass.

## Risks / Trade-offs

- **Legitimate new leaf setup requires a coordinated policy change.** → Update the relevant requirement, validator schema, and focused tests in the same review.
- **Exact property allowlists are intentionally strict.** → Keep diagnostics tied to the job, position, and unexpected property so maintenance is straightforward.
- **External pinned actions can internally export environment values.** → Keep their exact reviewed action references and inputs; dependency supply-chain pinning remains outside this follow-up.

## Migration Plan

Land the validator, tests, documentation, and specification changes together. The current workflow already satisfies the closed schemas, so no CI migration is required. Rollback reverts only policy artifacts.

## Open Questions

None.
