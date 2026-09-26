## ADDED Requirements

### Requirement: Bounded and attributable rules-screen conformance execution
The protected native rules-screen matrix MUST execute every existing resolution/state/render combination: 960x540, 1280x720, and 1920x1080; original, edited, undone, redone, and reopened in that order per resolution; and 0/500/900 ms frame previews, audiovisual range preview, and final export per state. Each resolution MUST execute in one independent required Linux CI job, with no more than three resolution jobs running concurrently. Every job MUST apply all existing semantic-plan, reviewed-reference, visual, audio, timing, font/dependency, and project-integrity assertions. A missing, invalid, duplicated, skipped, or zero-test resolution job or a render/comparison failure MUST fail the protected foundation gate. Successful required runs MUST report non-negative elapsed times identified by resolution, state, and operation for all 45 frame previews, 15 range previews, and 15 exports, plus each state and resolution total. Long-running process-tree memory observations MUST be diagnostic and MUST NOT change render acceptance thresholds or report schema.

#### Scenario: Complete the independent resolution jobs
- **WHEN** required native rules-screen parity runs in CI
- **THEN** three isolated jobs each execute exactly five lifecycle states in order and 25 render calls, for 15 states and 75 calls in aggregate, with unchanged assertions and reviewed references

#### Scenario: Attribute costs and resource use
- **WHEN** all three jobs succeed
- **THEN** their logs identify each of the 75 render calls by resolution, lifecycle state, and operation, and report diagnostic elapsed and process-tree peak memory values without a universal performance pass/fail budget

#### Scenario: Fail closed on omitted or broken evidence
- **WHEN** a matrix entry or native test is missing, duplicated, skipped, selects no tests, has missing or invalid native tools/font, or a render or comparison fails
- **THEN** the corresponding job or policy validator fails and the foundation gate cannot report success or publish invalid rules-screen evidence

## MODIFIED Requirements

### Requirement: Multiresolution rules-screen render conformance
Frame preview, audiovisual range preview and final export MUST render the rules-screen through the production EvaluatedScene at 1920x1080, 1280x720 and 960x540. Original, edited, undone, redone and reopened states MUST match independent scene expectations and reviewed visual references at the recipe timestamps. The three resolutions MAY run in separate required native jobs, but all five states and every render intent MUST execute in order within each resolution's fixture. SSIM MUST be at least 0.99, aligned decoded float-PCM RMS error at most 0.0001 and timing deviation at most one output frame. Repeated and cold/warm-cache evaluations MUST preserve semantic plans and decoded visuals. Native dependency/font failures MUST fail the required gate rather than skip. References MUST be independently reviewed, hash-recorded and changed only deliberately; existing golden baselines/tolerances MUST remain intact.

#### Scenario: F3 Compare scales intents and lifecycle states across required jobs
- **WHEN** the three required native resolution jobs render every specified intent, timestamp, and lifecycle state
- **THEN** every named motif remains visible at its expected location/order and visual/audio/timing results meet the stated tolerances without dropping cold/warm or history evidence

### Requirement: Optimized golden conformance without reduced evidence
The protected native render gates collectively MUST execute the existing reviewed frame, audiovisual range, export, lifecycle, reference, cache, and report checks in an optimized Rust test profile where required. The original Render parity job MUST retain the non-rules-screen golden suites, native cache evidence, strict report validation, and report publication; the three required rules-screen jobs MUST retain the full rules-screen matrix. All jobs MUST retain the same fixtures, assertions, tolerances, immutable references, fail-closed native dependencies, and report-only performance policy. Successful required runs MUST expose elapsed time for each named golden conformance suite, rules-screen resolution, and sampled capture in CI logs; these observations MUST NOT be universal pass/fail budgets.

#### Scenario: Complete reviewed evidence across jobs
- **WHEN** all required native render jobs run with approved dependencies and reviewed golden generations
- **THEN** every existing native golden assertion executes, the original report is strictly validated before publication, and each job's required timing records appear in its log

#### Scenario: Reject drift in any required job
- **WHEN** optimized execution in any job produces a frame, audio, timing, semantic-plan, filter-graph, cache, or report result outside existing reviewed acceptance criteria
- **THEN** that leaf and the protected foundation status fail, and no invalid report is published
