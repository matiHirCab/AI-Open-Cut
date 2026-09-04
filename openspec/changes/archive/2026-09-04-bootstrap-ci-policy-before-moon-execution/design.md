## Context

The current final validator detects forbidden Moon configuration only after Moon has already resolved that configuration and started the task shell. A root `BASH_ENV` can therefore execute first, replace `bun` and `bunx`, and write the expected step output. The repository remains personal, so this change strengthens the reviewed-code boundary without claiming a cryptographic boundary against coordinated workflow and verifier replacement.

## Goals / Non-Goals

**Goals:**

- Validate workflow, Moon, and proto configuration before Moon interprets project configuration.
- Prevent the Moon child from owning or directly accessing the GitHub step-output channel.
- Emit the existing policy attestation only after the exact protected task exits successfully.
- Pin bootstrap tools independently of pull-request configuration and protect policy files through CODEOWNERS.

**Non-Goals:**

- Add an organization-level required workflow or use `pull_request_target`.
- Make a pull-request-controlled workflow cryptographically tamper-proof.
- Change application, contract, render, fixture, or persisted-data behavior.

## Decisions

### Bootstrap outside Moon

The OpenSpec job installs explicit Bun and Moon versions before checkout, then checks out the repository and invokes a Bun bootstrap directly. Installing from `.prototools` or calling Moon before validation was rejected because either path would consume the configuration being evaluated.

### Separate validation, execution, and attestation

The bootstrap loads every structural source and calls the pure validator before spawning `moon run root:openspec-validate` with an argument vector rather than a shell. It removes `GITHUB_OUTPUT` from the child environment, waits for exit code zero, and only then appends `validated=true` itself. Keeping the writer in the final Moon command was rejected because startup hooks run before that command.

### Validate proto configuration as part of the boundary

`.prototools` is required to match the reviewed Moon 2.3.3, Bun 1.4.0, and Rust 1.97.0 pins exactly. Extra settings, environment, plugin declarations, alternate proto configuration, missing files, or changed versions fail before Moon launches.

### Preserve the aggregate interface

The policy step retains `id: policy`, its job output remains `policy_validated`, and the foundation aggregate remains unchanged. CODEOWNERS covers the workflow, Moon/proto sources, bootstrap, validator, and tests so `main` can require owner review without changing the single required status name.

## Risks / Trade-offs

- **The workflow and bootstrap are still pull-request-controlled reviewed code.** → Document the limitation, require CODEOWNER approval on `main`, and keep the validator's accepted sequence exact.
- **Bootstrap setup adds two explicit actions.** → Pin tool versions through action inputs, disable automatic setup and caching, and validate their order and properties.
- **The child could only attest by discovering an output path through unrelated runner behavior.** → Remove `GITHUB_OUTPUT` from its environment and retain exact, reviewed child commands; no secrets or privileged event are introduced.

## Migration Plan

Land the workflow, bootstrap, pure validator, exact proto validation, tests, CODEOWNERS, documentation, and specification delta together. Run the complete repository validation suite and the isolated adversarial regression, synchronize the living requirement, and archive this follow-up. Rollback restores all of those files together and returns to the prior reviewed-code limitation.

## Open Questions

None.
