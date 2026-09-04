## Context

Issue #16 introduced dedicated contract and render parity jobs plus a stable aggregate status. The aggregate currently relies on GitHub Actions' implicit success condition, so a failed or skipped dependency skips the aggregate; GitHub treats a skipped required job as successful for merge protection. The structural policy checker also recognizes command substrings without binding them to fail-closed steps or enforcing report validation before upload.

The correction is confined to repository validation. Stable job identities, authoritative contract/render commands, fixtures, runtime behavior, public contracts, persisted data, and renderer output remain unchanged.

## Goals / Non-Goals

**Goals:**

- Make the aggregate job complete successfully only when both leaf results are exactly `success`.
- Preserve one stable branch-protection target while retaining independently visible leaf failures.
- Reject critical workflow steps that are neutralized, relocated, altered, skipped, or reordered incompatibly.
- Provide focused policy tests for every reviewed failure mode.

**Non-Goals:**

- Change GitHub repository settings, job IDs or display names, toolchain versions, application code, contracts, fixtures, golden thresholds, or renderer behavior.
- Build a general-purpose GitHub Actions interpreter or execute synthetic workflows remotely.

## Decisions

### Run the aggregate unconditionally and assert dependency results

Set `foundation-parity.if` to `${{ always() }}` so dependency failure, cancellation, or skip does not suppress the required aggregate check. Pass each direct dependency result through a fixed environment variable and use a short Bash step that logs both values and exits nonzero unless both equal `success`.

Using the default dependency condition was rejected because skipped required jobs are accepted by GitHub branch protection. Requiring both leaf jobs directly in branch protection was rejected because the approved design calls for one stable aggregate target.

### Validate exact critical-step structure

The policy checker will identify critical steps by their structural fields and require exact command bodies, expected working directories and environment, default fail-fast behavior, and required ordering. It will reject job- or step-level `continue-on-error`, altered command bodies such as `|| true`, and report upload before strict validation.

Allowing arbitrary extra shell text was rejected because static substring presence cannot prove failure propagation. Building a shell parser was rejected as unnecessary for a small, intentionally stable workflow boundary.

### Model aggregate outcomes in a pure helper

Add an exported helper that accepts the two GitHub dependency result strings, returns only for two `success` values, and throws otherwise. Use it for exhaustive unit evidence covering `failure`, `cancelled`, and `skipped`; keep the workflow step and structural validator synchronized with those semantics.

Depending only on YAML inspection was rejected because it would not directly demonstrate the result truth table required by the acceptance criteria.

## Risks / Trade-offs

- **Exact command validation requires intentional updates when commands evolve.** → Treat changes to this small foundation boundary as reviewed specification changes and update tests in the same change.
- **`always()` also schedules the aggregate after cancellation of a dependency.** → The job performs no checkout or external work and exits immediately with a failing result, which is the desired required-check behavior.
- **Local tests cannot emulate GitHub's hosted scheduler.** → Combine the documented GitHub result semantics with structural assertions and exhaustive pure truth-table tests.

## Migration Plan

Update the workflow and policy checker together, run repository and Linux-native parity verification, then keep `Motion-graphics foundation parity` as the documented branch-protection target after the workflow reaches `main`. Rollback must restore the workflow, validator, tests, documentation, and specification wording together.

## Open Questions

None.
