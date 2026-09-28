## Context

The protected PR matrix currently runs 25 native operations for each of three rules-screen resolutions. On PR #127, 1920×1080 took about 7.5 minutes per operation and GitHub cancelled it at 180 minutes after 23 of 25 operations. The same shard previously took about 192 minutes on `main`. The protected foundation also has a 180-minute hard cap, so increasing only the job timeout cannot make the present design pass. Existing specs and policy tests require all 75 calls in the PR workflow.

## Goals / Non-Goals

**Goals:**

- Keep a required 1920×1080 PR signal for original and edited scene output across preview, audiovisual range, and export, while leaving all 25 calls at 960×540 and 1280×720 required on PRs.
- Preserve the exact full 1920×1080 lifecycle fixture, references, comparison thresholds, and 25-call coverage in a weekly and manually dispatchable workflow on the default branch.
- Make omitted, invalid, skipped, or failing checks visible: fail closed in native tests, validate both workflow definitions structurally, and let the scheduled job fail normally in Actions.
- Restore the protected workflow's 120-minute default budget without the temporary exception.

**Non-Goals:**

- Changing renderer output, scene semantics, FFmpeg parameters, golden baselines, public contracts, schema, or migrations.
- Making the weekly workflow a PR branch-protection dependency or claiming that its first scheduled execution can occur before merge.
- Guaranteed notification delivery or exact schedule start time; GitHub controls scheduled workflow execution.

## Decisions

1. **One conformance implementation with explicit scope.** An exact, validated test scope selects `pr` or `full`. In `pr`, 1920×1080 still constructs all five lifecycle states and checks semantic plans and project integrity, but renders only original and edited states: one 500-ms frame preview, one audiovisual range preview, and one export per state (six calls). The 960×540 and 1280×720 resolutions always run the existing full five-state, 25-call path. In `full`, 1920×1080 executes the current 25-call path. Missing or invalid CI scope fails closed. This keeps one fixture, reference loader, and comparison implementation; separate fixtures risk drift. The six 1920 calls should take roughly 45 minutes at the observed 7.5 minutes per call, leaving margin under the 120-minute foundation budget while 1280×720 remains the expected critical path.
2. **Independent weekly full job.** Add a workflow scheduled for Mondays at 03:17 UTC and enabled for `workflow_dispatch`. It runs on the default branch with read-only repository permissions, Ubuntu FFmpeg and the reviewed DejaVu font, the pinned toolchain, exact optimized native test discovery, and explicit `full` scope at 1920×1080. Give it a 240-minute timeout, above the observed ~192 minutes and below GitHub's six-hour hosted-job limit. Its red Actions status and full native log expose failures. The workflow does not publish new golden baselines or mask test errors. Running it only on schedule avoids a 3-hour PR gate; manual dispatch allows immediate post-merge or incident checks.
3. **Guard both paths as reviewed policy.** Update the existing structural CI policy validator and tests to require the PR matrix, exact scopes, six-call selection, fail-closed commands, the weekly/manual workflow, its pinned dependencies and full selector, and its bounded timeout. The protected foundation continues to require all six existing prerequisite result categories and all three PR resolution shards. It does not depend on the weekly result, since a schedule-only workflow has no result on an unmerged PR. The OpenSpec bootstrap must reject removal or weakening of the scheduled workflow definition.
4. **Restore the ordinary duration rule.** Remove the temporary exception environment from the foundation step and make the duration script accept either no exception (120-minute default) or one complete, reviewed exception; partial exception fields fail. Preserve the existing duration-function tests for legitimate future exceptions. Set the PR rules-screen matrix timeout to the ordinary 135-minute leaf limit. Keep the scheduled workflow outside the PR duration measurement.

Alternatives considered: raising the PR cap would still exceed the recorded full-resolution runtime and prolong every PR; dropping 1920×1080 entirely would lose high-resolution evidence; running the full 25 calls on every PR with a faster runner would introduce an unreviewed cost and runner dependency. The selected split preserves continuous high-resolution sampling and periodic full evidence without changing rendering behavior.

## Risks / Trade-offs

- A regression limited to an omitted 1920×1080 lifecycle state or timestamp can reach `main` and be detected at the next weekly run. Mitigation: keep full lower-resolution PR coverage, high-resolution original/edited output across all three intents, exact scheduled full coverage, and manual dispatch.
- GitHub schedules can be delayed or disabled for inactive public repositories. Mitigation: document the schedule and manual dispatch path, structurally require the workflow definition, and treat the red or absent scheduled run as an operational follow-up rather than as PR gate evidence.
- The 1280×720 full shard could still exceed 120 minutes on a slower runner. Mitigation: retain timing logs and the fail-closed foundation budget; optimize that path through a separate reviewed change if measurements show a new overrun.
- A weekly workflow failure remains visible in Actions but does not retroactively block merged PRs. Mitigation: document that limitation and require maintainers to review failures; no success is fabricated by PR policy validation.

## Migration Plan

No data migration is needed. Merge the changed test, protected workflow, scheduled workflow, policy validator, tests, and docs together. The first scheduled run can occur only after the workflow reaches the default branch; use manual dispatch after merge for earlier full evidence if needed. To roll back, revert this complete change through a new approved OpenSpec change so the test selector, workflow policy, and living specs remain synchronized.

## Open Questions

None. The user selected weekly cadence.
