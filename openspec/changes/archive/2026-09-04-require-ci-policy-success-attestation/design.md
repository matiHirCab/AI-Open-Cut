# Context

GitHub Actions distinguishes a failed step outcome from the successful conclusion produced by `continue-on-error`. Because the OpenSpec policy validator executes in the same job it validates, a structural violation can fail the command while the job result consumed by `foundation-parity` remains `success`.

# Goals / Non-Goals

**Goals:**

- Prove that the exact OpenSpec validation command reached successful completion.
- Reject a missing, skipped, reordered, altered, or forged success marker structurally.
- Make the aggregate fail when the policy job result is successful but the completion proof is absent.
- Preserve stable check names and the single required aggregate.

**Non-Goals:**

- Replace repository review or branch protection as the trust boundary for coordinated edits to the workflow, validator, and aggregate.
- Change application behavior, contracts, fixtures, goldens, or persisted data.

# Decisions

## Emit a step output after policy validation

Give the policy step the exact identifier `policy` and run a two-line fail-closed script: first `moon run openspec-validate`, then `echo "validated=true" >> "$GITHUB_OUTPUT"`. GitHub's reviewed default shell exits on a failing first command, so ignored failure at step or job level cannot produce the marker. A skipped or neutralized step likewise emits no marker.

Expose the marker through the sole job output `policy_validated: ${{ steps.policy.outputs.validated }}`. The structural validator requires the exact step identifier, command body, output map, and command order.

## Require result and attestation in the aggregate

Bind the job output to exact aggregate variable `OPENSPEC_POLICY_VALIDATED`. Log it with the three dependency results and fail unless all results are `success` and the attestation is exactly `true`.

This keeps the existing result diagnostics while distinguishing a genuinely completed policy run from a failure masked by workflow execution settings.

## Extend the pure aggregate model

Change `assertFoundationParityResults` to accept the attestation. Evaluate all 64 terminal-result tuples against `true`, empty, `false`, and an arbitrary value. Only the all-success tuple with `true` is accepted.

# Risks / Trade-offs

- **The OpenSpec command becomes a two-line exact script.** Legitimate changes to policy execution or output naming require coordinated workflow, validator, specification, and test updates.
- **The marker is an internal reviewed convention, not a cryptographic attestation.** It closes independent execution-neutralization mutations; coordinated changes to every reviewed enforcement surface remain a code-review concern.

# Migration Plan

Land the job output, aggregate assertion, validator, tests, documentation, and requirement together. Verify all repository gates, synchronize the repository-validation delta, and archive this follow-up. Maintainers continue requiring `Motion-graphics foundation parity` after the workflow reaches the default branch.

# Open Questions

None.
