# Design approval

The owner explicitly approved the concrete issue-45 design. Evidence was forwarded by the parent conversation `01a0f1e8-f3f2-7182-9126-2a8f1dd400dd` on 2026-10-02. The approved proposal snapshot is local commit `32ebd32e329287827e15831528bd9fe041203196`, based on PR #133 head `618357054cd6d2d40275ec6f61163996852227a3`.

Forwarded assistant message, `Sentinel_2d69f2be63e88191a882ac44ba46c3ca`:

> Issue #45’s proposal is ready, and Git commit plus dry-run push both passed. The proposed first version would: Add a versioned basic animation preset using the six existing scalar properties and curve types; leave the creative preset collection for #46. Save both the preset information and its resolved animation, so future preset updates cannot change old projects. Reject overlapping channels unless replacement is explicitly requested. Support standalone edits and atomic batch edits, including undo/redo and reopening. Migrate saved projects and retained history to schema 29; preset authoring inside draft lists and component definitions would stay outside this first version. The specification checks passed. Do you approve this design so I can begin implementation? The details are in the issue #45 chat.

Forwarded user message, `Sentinel_8d87aaa3e9148191a1a72961e18c27cc`:

> Approve

The parent then explicitly instructed implementation, tests and local commits within the bounded approved scope. This records user design approval, not a submitted GitHub review, completed implementation acceptance, or permission to publish a PR, merge, deploy, change credentials/security or alter workflow gates. Issue #44 approval is unrelated. Implementation will follow the prepared delta requirements and tasks; final conformance and required checks remain pending.

## Verification-scope clarification — 2026-10-02

The parent conversation `01a0f1e8-f3f2-7182-9126-2a8f1dd400dd` subsequently forwarded the user's explicit verification instruction:

> Distinguish mandatory PR gates from optional weekly coverage; do not spend extra time on unchanged optional suites if repository policy doesn't require them, but do not stop required checks or weaken thresholds.

`docs/ci-parity-gates.md` explicitly identifies full weekly 1920x1080 coverage as a separate scheduled/manual workflow, outside the PR branch-protection dependencies. Task 6.3 now names the mandatory PR scope rather than requiring this optional full suite. All three required PR shards had already passed before the optional full run was stopped on 2026-10-02 at 16:09:56 UTC. Its nine successful operations, last edited range-preview heartbeat and SIGTERM cancellation are retained as partial evidence, not a full-suite pass. No required test was stopped and no workflow, marker, threshold, assertion, product behavior or normative design requirement changed.

This clarification does not grant designated CODEOWNER acceptance, synchronization/archival approval or external publication authorization. Those decisions remain separate and pending.
