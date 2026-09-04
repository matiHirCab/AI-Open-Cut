## Context

The hardened Bun process currently runs from a temporary directory containing the malicious `bunfig.toml` but not the default workflow or remaining Moon/proto boundary. A nonzero exit, empty output, and absent sentinel prove that the preload did not execute, but do not prove that preflight rejected the Bun configuration rather than an earlier missing file.

## Goals / Non-Goals

**Goals:**

- Make the real-Bun regression reach the canonical Bun-configuration validation path.
- Attribute failure to the malicious `bunfig.toml` with an exact diagnostic.
- Keep Moon and the output writer unreachable after that rejection.

**Non-Goals:**

- Change production workflow, bootstrap, validator, or Moon behavior.
- Add a new public interface or status check.
- Modify branch protection, fixtures, or goldens.

## Decisions

### Build a complete temporary policy boundary

The test will copy the reviewed root project, workspace, toolchain, and proto sources into the temporary directory while retaining the malicious Bun configuration. It will pass the real workflow as an explicit absolute `--workflow` argument. This makes `bunfig.toml` the only invalid protected source without adding a test-only bootstrap interface.

### Assert the rejection reason

The hardened child must exit nonzero and report the canonical Bun-configuration diagnostic. The test will continue to require an empty GitHub output and absent preload sentinel. Because this diagnostic is produced synchronously by preflight before `runMoon`, it also proves that Moon was not started.

### Isolate the nested Moon reproduction

When the integration suite itself runs as part of `root:openspec-validate`, Moon injects parent `MOON_*` and `PROTO_*` metadata into the test process. The adversarial reproduction will remove those inherited values, use temporary Moon/proto homes, and mark the child as CI before launching its independent workspace. This avoids parent-task locking without weakening the `BASH_ENV` behavior under test.

## Risks / Trade-offs

- **The test copies canonical configuration text.** This is intentional: it tracks the real reviewed boundary and fails if the bootstrap no longer accepts it.
- **The diagnostic becomes part of regression evidence.** Its exact stable fragment is already used by structural unit tests and is appropriate for identifying the validated failure path.
- **Nested Moon startup can exceed Bun's five-second test default on a cold runner.** Isolate its stores and use a bounded 30-second timeout for this Linux-only integration case while keeping all assertions fail-closed.

## Migration Plan

Land the regression, documentation correction, and repository-validation delta together. Run the real test on Ubuntu, synchronize the living specification, and archive the follow-up. No runtime or data migration is required.

## Open Questions

None.
