## Why

PR #124's optimized Render parity runs took 205m01s and 336m50s; `rules_screen` consumed 11,526.743s and 19,210.915s. The two-worker same-runner candidate was cancelled at about six hours with 74 of 75 rules-screen calls complete. Its 960x540 group took 120.78 minutes and 1280x720 took 202.62 minutes while contending with the unfinished 1920x1080 group. Independent resolutions can use separate CI runners, but the protected workflow currently binds them to one golden test.

## What Changes

- Keep all three resolutions, five ordered lifecycle states per resolution, and three frame previews, one audiovisual range preview, and one export per state. Preserve all semantic, reviewed-reference, visual, audio, timing, font/dependency, and project-integrity assertions.
- Execute one complete resolution group per protected Linux matrix job, with exactly three entries and at most three concurrent runners. Keep the other golden suites, native cache checks, and strict report validation/publication in the existing Render parity job.
- Require the stable foundation gate to include both render jobs. Pin matrix completeness, required native environment, exact test selection and discovery, failure propagation, and aggregate dependencies in the CI policy validator and adversarial tests.
- Retain per-operation timing diagnostics and sample process-tree memory at a lower rate for the long rules-screen jobs, without changing the existing short-interval sampler or making performance a universal test budget.
- Compare candidate and post-archive protected workflows with both completed baselines. Candidate [36244746913](https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36244746913) completed in 2h03m39s with all three shards passing, 75 render records, and unchanged reference/report checks. This is 81m22s faster than the 205m01s baseline and 3h33m11s faster than the 336m50s baseline. The previously proposed under-two-hour acceptance target was missed by 3m39s; the reviewer explicitly accepted this measured candidate for archival. The final protected workflow still MUST pass, and its elapsed time remains reported and compared with these baselines, without a numeric CI pass/fail budget.

No production rendering, fixture, reference, tolerance, public contract, persisted schema, or migration changes are proposed. The split uses more runner capacity to reduce elapsed CI time.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: preserve the full rules-screen matrix while attributing each resolution's output and timing evidence to a required job.
- `repository-validation`: protect the three-shard matrix, zero-test guard, and four-prerequisite foundation gate.

## Impact

The change affects the native golden test harness, pinned CI workflow, policy validator and tests, and OpenSpec evidence. The existing render baseline report stays owned by the original Render parity job with its current schema and strict publication order.
