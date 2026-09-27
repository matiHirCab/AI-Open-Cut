## Context

PR #126 adds a 180-minute timeout to the rules-screen matrix job and an exact structural policy check. Since that PR branched, `main` merged `bound-protected-ci-duration`: it caps required jobs at 135 minutes, permits a temporary overall exception of at most 135 minutes against a 120-minute default, and enforces that cap in the final foundation job. The two policies cannot coexist unchanged. The 1920x1080 job in PR #126 was still rendering when its run was cancelled at roughly 135 minutes; the log does not identify the cancellation initiator.

## Goals / Non-Goals

**Goals:** Preserve `main`'s protected duration audit and all required checks while allowing the approved three-hour rules-screen shard and a matching temporary overall exception. Resolve the four merge conflicts without losing either branch's independent changes.

**Non-Goals:** Remove the 120-minute default, change test selection or rendering behavior, disable the duration audit, raise other leaf-job limits, or guarantee that GitHub will not cancel a run for an external reason.

## Decisions

- Merge the latest `main` into the PR branch, retaining its foundation prerequisites, duration audit, and closed policy checks. Replacing `main`'s validator with the PR's older version would silently remove required safeguards.
- Set only `rules-screen-parity` to a numeric 180-minute job timeout. Leave the other required jobs at 135 minutes and the foundation job at 10 minutes.
- Extend the temporary overall exception's hard cap and evaluator maximum to 180 minutes, retaining the 120-minute default, owner, reason, evidence, and 2026-10-26 expiration. Use PR #126's cancelled run as evidence of the shard still running near the prior cap, without claiming the cap caused its cancellation.
- Require exact structural validation of the 180-minute shard timeout and the reviewed duration exception. Add positive and negative tests for the evaluator's 180-minute boundary and the validator's timeout rejection cases.

## Risks / Trade-offs

- [More runner time and a longer required-status critical path] → Limit the exception to the affected shard, retain its expiration, and keep all required tests and the aggregate assertion.
- [The prior cancellation may have an external cause] → State this limitation in the documentation and verify the next GitHub run separately.
- [Conflicting policy tests may falsely pass after a partial merge] → Run the full policy suite, the real Moon gate, and the affected repository checks after the merge and archival.

## Migration Plan

No application or data migration. Merge the PR with updated policy, tests, docs, and spec in one review. Roll back the exception and timeout together if the three-hour budget is not approved. Preserve the original `main` duration evidence and record PR #126's later cancellation separately.

## Open Questions

The cause of the PR #126 cancellation is not proven by the available GitHub log. The next CI run will show whether the new limit is sufficient, but cannot by itself distinguish every external cancellation cause.
