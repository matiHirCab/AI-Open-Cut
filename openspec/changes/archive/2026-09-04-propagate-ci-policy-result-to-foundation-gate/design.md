## Context

`moon run openspec-validate` detects structural parity-gate weakening in the independently visible `openspec` job. `foundation-parity` currently depends only on `contract-parity` and `render-parity`, so its result does not reflect a failed policy job even though it is the documented single branch-protection target.

## Goals / Non-Goals

**Goals:**

- Propagate policy, contract, and render results through one stable aggregate status.
- Preserve unconditional aggregate execution and exact-success semantics for every terminal dependency result.
- Close step, environment, shell, defaults, container, conditional-skip, and ignored-failure bypasses in the policy job.
- Keep the three prerequisite jobs parallel and independently diagnosable.

**Non-Goals:**

- Treat simultaneous hostile edits to reviewed workflow and validator source as an administrative trust-boundary substitute.
- Change check names, branch-protection administration, application behavior, contracts, fixtures, or goldens.

## Decisions

### Add OpenSpec as a direct aggregate dependency

Use `needs: [openspec, contract-parity, render-parity]` and retain `${{ always() }}`. Bind the direct results to exact `OPENSPEC_RESULT`, `CONTRACT_PARITY_RESULT`, and `RENDER_PARITY_RESULT` variables. The aggregate logs all three and exits nonzero unless every value is exactly `success`.

The prerequisite jobs remain independent rather than making parity execution wait for policy validation. This preserves feedback latency while ensuring the final protected result waits for every boundary.

### Govern the policy job as an exact sequence

Require the existing checkout with `fetch-depth: 0`, pinned toolchain setup, and `moon run openspec-validate` step in that order. Reject extra, missing, replaced, duplicated, or reordered steps; altered step properties; custom conditions or shells; ignored failures; inherited environment; run defaults; and containers.

### Extend the pure aggregate model

Change `assertFoundationParityResults` to accept policy, contract, and render results. Exhaustively evaluate the four documented terminal values for all three inputs, producing 64 cases with only the all-success tuple accepted.

## Risks / Trade-offs

- **The aggregate waits for one more job.** All prerequisites still run concurrently, so only the final status waits for the longest existing gate.
- **The OpenSpec job becomes intentionally rigid.** Any legitimate policy setup change requires a coordinated specification, validator, workflow, and test update.

## Migration Plan

Land the workflow, validator, tests, documentation, and living requirement together. After verification, synchronize the repository-validation delta and archive this follow-up. Maintainers continue requiring `Motion-graphics foundation parity` after the workflow reaches the default branch.

## Open Questions

None.
