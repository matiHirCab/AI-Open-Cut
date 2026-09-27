## Context

The rules-screen parity matrix has three Linux resolution shards. PR #126's 1920x1080 shard was cancelled while the native test was running after roughly 2 hours 15 minutes. The current workflow declares no job timeout, and the repository's structural policy validator checks approved job properties. An uncoordinated workflow edit could therefore be rejected by the protected OpenSpec job.

## Goals / Non-Goals

**Goals:** Declare a 180-minute timeout on each rules-screen shard and make the exact value part of the reviewed CI policy with focused positive and negative tests.

**Non-Goals:** Alter the render test, resolution matrix, other job budgets, aggregate result handling, or GitHub cancellation behavior outside the timeout setting.

## Decisions

- Put `timeout-minutes: 180` on `rules-screen-parity` at the job level, so each matrix child gets the full budget. A step-level timeout would not cover setup and is not the requested job budget.
- Extend the rules-screen job's closed-property validation to require the exact numeric value. Accepting an arbitrary positive timeout would permit unreviewed changes to the protected boundary.
- Test the accepted workflow and rejection of absent, shorter, and longer values. Retain the existing checks for step sequence, environment, failure propagation, and aggregate status.

## Risks / Trade-offs

- [A shard can now consume up to three hours of runner time] → Keep the limit confined to the rules-screen matrix and retain normal failure propagation.
- [The observed cancellation may have been initiated outside the job timeout] → Do not claim this setting resolves every cancellation; use the next PR run to observe the outcome.

## Migration Plan

No data or API migration. The workflow and validator must land together. Revert both and their spec change if the approved job budget needs to be restored.

## Open Questions

None for this bounded change; the exact origin of PR #126's cancellation remains unconfirmed by the available log.
