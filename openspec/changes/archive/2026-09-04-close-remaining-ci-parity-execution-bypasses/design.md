## Context

The foundation aggregate validates its command text and result variables but does not close the step property set or reject job-level command defaults and containers. GitHub Actions accepts a custom shell template such as `true {0}`, which ignores the generated script and succeeds. The same replacement can be inherited from `defaults.run.shell`.

The policy also permits unrelated workflow and job environment variables. Bash reads the file named by `BASH_ENV` before executing each non-interactive script, so a parity job can skip its reviewed commands without changing their exact YAML bodies. The valid workflow uses none of these surfaces.

## Goals / Non-Goals

**Goals:**

- Make the foundation aggregate execute only through its reviewed job and step configuration.
- Require exactly the two approved aggregate result variables.
- Reject `BASH_ENV` everywhere it can be inherited by a parity script.
- Preserve benign workflow and parity-job environment metadata.
- Keep failures structural and diagnostic.

**Non-Goals:**

- Change any checked-in parity job, command, result expression, renderer, contract, fixture, golden reference, or public interface.
- Ban every possible environment variable or interpret arbitrary shell programs.
- Change branch-protection administration.

## Decisions

### Close the aggregate execution schema

Apply the existing defaults and container rejections to `foundation-parity`. Require its assertion step to have exactly `name`, `env`, and `run`, and require its environment to have exactly the two current `needs.*.result` bindings. This rejects custom shells and all unreviewed step properties without modifying the valid workflow.

### Reject the confirmed Bash startup hook

Add a common inherited-environment check for `BASH_ENV` at workflow scope and on all three parity jobs. Reject the key regardless of its literal or expression value. Continue allowing other job metadata such as `CI_LOG_LEVEL` and `RUST_BACKTRACE`, subject to the existing exact critical-step environment rules.

A blanket environment allowlist was rejected because the accepted policy explicitly permits unrelated configuration. Extending the denylist beyond the confirmed `BASH_ENV` injection surface was rejected because it would broaden policy without a demonstrated requirement.

## Risks / Trade-offs

- **A legitimate future aggregate property will fail policy validation.** Update the requirement, validator, and tests deliberately in the same review.
- **The inherited-variable check is intentionally targeted.** New demonstrated shell startup injection mechanisms require an explicit follow-up instead of silently redefining unrelated configuration.

## Migration Plan

Land the validator, tests, documentation, and specifications together. The current workflow already satisfies the stricter policy, so no workflow or branch-protection migration is required. Rollback reverts only policy artifacts.

## Open Questions

None.
