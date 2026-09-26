## Context

Completed optimized Render parity runs [35994612911](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/35994612911) and [36032914723](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36032914723) took 205m01s and 336m50s. Their `rules_screen` suite took 11,526.743s and 19,210.915s. The two-worker same-runner candidate [36146054095](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36146054095) was cancelled in its native step at about six hours. Its log contains 74 of 75 operation records: 960x540 completed in 120.78 minutes, 1280x720 in 202.62 minutes, and 1920x1080 was cancelled during the reopened-state export. No rules-screen peak memory record or final validated report was produced. These concurrent-runner timings do not establish a dedicated-runner estimate; the 1920x1080 shard may still exceed the two-hour target. The suite's 3 resolutions × 5 states × 5 render operations produce 75 required calls. Reviewed references, SSIM >= 0.99, PCM RMS <= 0.0001, one-frame timing tolerance, and project-integrity checks remain authoritative.

Living render and repository-validation specs require a closed golden command and three-prerequisite foundation gate. Amend both before moving rules-screen to separate protected jobs.

## Goals / Non-Goals

**Goals:** Preserve all 15 cases and 75 render operations with five-state order within each resolution, reduce runner contention, fail if any job or assertion is omitted, and finish the protected workflow under two hours on candidate and post-archive runs.

**Non-Goals:** Drop states or intents, change references/tolerances, parallelize lifecycle mutations within one resolution, change production rendering or report schema, or make timing/memory a universal pass/fail threshold. A production-renderer optimization requires separate approved scope if the split misses the two-hour target.

## Decisions

1. Keep one required native test named `renderer::golden::rules_screen::native_rules_screen_resolution_conformance`. Its closed input `OPENCUT_RULES_SCREEN_RESOLUTION` accepts only `960x540`, `1280x720`, or `1920x1080`; absent or invalid input fails when `OPENCUT_GOLDEN_REQUIRED=1`. Each invocation verifies the reviewed font, builds its own fixture, runs original → edited → undone → redone → reopened, and checks 15 previews, 5 ranges, and 5 exports. Remove the same-runner two-worker schedule and corresponding test. The existing golden test still runs all other suites and owns sampled capture and report publication.
2. Add `rules-screen-parity` as an exact three-entry `ubuntu-latest` matrix with `max-parallel: 3` and `fail-fast: false`. Each job installs deterministic FFmpeg/font and pinned Rust, sets native tool/font/resolution variables at the test step only, verifies the exact libtest name appears in `--list`, then runs it in the optimized profile with `--exact --nocapture`. A missing test, missing prerequisite, render error, or comparison failure fails that shard. The original `render-parity` job keeps its other suites, cache sequence, and validated report artifact.
3. Change foundation parity to depend directly on OpenSpec validation, contract parity, Render parity, and rules-screen parity; its unconditional assertion requires all four results to be exactly `success` and the OpenSpec attestation to be `true`. Extend `validate-ci-gates.ts` and mutation tests to pin exact matrix, steps, command, environment, failure settings, and aggregate result bindings.
4. Keep per-operation/state/resolution timing records. Give the long rules-screen suite a 250 ms process-tree memory sample interval through a new sampler constructor, leaving the established 5 ms constructor and its transient-child tests untouched. Log diagnostic peak per resolution; do not add it to the report schema or use it as a pass/fail budget.
5. Use the completed current run's partial timings and documented missing memory/report evidence. Candidate and final post-archive protected workflows must pass and finish under 120 minutes from workflow start to foundation completion. If the slowest 1920x1080 shard misses, do not archive; use its operation breakdown for a separately approved production-renderer optimization.

## Risks / Trade-offs

- [More runner capacity and repeated compilation] → Bound the matrix to three entries and `max-parallel: 3`; report runner use with wall-time gain.
- [A matrix value silently runs zero tests] → Pin the exact libtest name in a `--list` check and execution command, reject missing/duplicate entries in policy tests, and require one test per shard.
- [Cache/lifecycle evidence changes across jobs] → Keep all five states and renders in one fixture per resolution in original order; only resolutions are separated.
- [Sampling changes runtime] → Use 250 ms only for long rules-screen jobs and preserve 5 ms behavior elsewhere.
- [Runner variation obscures gain] → Compare candidate and post-archive critical-path wall times with both baselines; do not archive if either exceeds 120 minutes.

## Migration Plan

No public or persisted migration is needed. The CI policy and workflow change together, so a partial split cannot validate.

## Open Questions

The dedicated-runner timing of the 1920x1080 group is unknown. The proposed matrix will establish whether a separately scoped renderer optimization is necessary.
