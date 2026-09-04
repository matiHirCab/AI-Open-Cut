# Context

Moon applies project-level environment variables and may merge global task configuration before executing a local task. Validating only the local task object therefore does not prove that its effective shell, command lookup, or dependencies are unchanged.

# Goals / Non-Goals

**Goals:**

- Bind CI to the explicit root task instead of ambient project resolution.
- Prevent root project environment, platform, toolchain, Docker, or inheritance overrides.
- Prevent global task configuration from injecting environment or remote extensions into the root execution boundary.
- Validate all Moon sources before emitting the completion marker.
- Preserve unrelated Moon tasks and application interfaces.

**Non-Goals:**

- Prohibit project-local Moon configuration under `apps/` or `crates/`.
- Prevent a coordinated replacement of the complete workflow and verifier without external trusted enforcement.

# Decisions

## Address and isolate the root project

The workflow invokes `moon run root:openspec-validate`. Root `moon.yml` contains only its schema, `workspace`, and `tasks`, and declares `workspace.inheritedTasks.include: []`. The protected task remains exact and fail-closed; unrelated sibling tasks remain permitted.

## Validate the complete Moon boundary

The validator accepts an internal bundle containing root project, workspace, toolchain, and discovered global-task sources. It requires the current stable workspace project mapping and Bun toolchain, rejects external extension, and rejects unexpected Moon configuration files. Global task files remain allowed for other projects but cannot declare top-level `env` or `extends`.

The CLI discovers supported `.moon/tasks` YAML files deterministically and performs every Moon-boundary check before invoking the output writer. The task inputs include all reviewed Moon configuration paths even though caching remains disabled.

## Fail closed on incomplete inputs

Missing required configuration, duplicate or unsupported configuration paths, parse failures, root inheritance, execution overrides, global environments, or extensions prevent attestation. Local validation without `GITHUB_OUTPUT` remains successful only after the same checks pass.

# Risks / Trade-offs

- Root Moon configuration becomes intentionally rigid; execution-affecting changes require coordinated policy updates.
- Global tasks remain available, but repository-wide environment injection and remote task extensions are disallowed because they cannot be proven irrelevant before root execution.
- The trust boundary remains reviewed repository code plus external branch protection.

# Migration Plan

Land workflow qualification, root isolation, validation, tests, documentation, and the delta together. Run every required repository gate, confirm golden hashes, synchronize the delta, and archive this follow-up.

# Open Questions

None.
