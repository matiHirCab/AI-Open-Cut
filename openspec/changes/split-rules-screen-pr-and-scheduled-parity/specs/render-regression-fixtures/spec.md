## MODIFIED Requirements

### Requirement: Bounded and attributable rules-screen conformance execution
The protected PR rules-screen matrix MUST run three independent required Linux resolution jobs with no more than three running concurrently. The 960x540 and 1280x720 jobs MUST each execute the existing original, edited, undone, redone, and reopened lifecycle in order, with 0/500/900 ms frame previews, audiovisual range preview, and final export per state (25 renders each). The 1920x1080 PR job MUST construct and check semantic plans and project integrity for all five states in order, and MUST render original and edited states through frame preview at 500 ms, audiovisual range preview, and final export (six renders). A separate weekly and manually dispatchable default-branch job MUST execute the unchanged full 1920x1080 five-state, 25-render conformance. Every executed render MUST retain existing reviewed-reference, visual, audio, timing, font/dependency, and project-integrity assertions. Missing, invalid, duplicated, skipped, or zero-test PR resolution jobs or a PR render/comparison failure MUST fail the protected foundation gate; missing or failed scheduled full evidence MUST fail its own visible Actions job. Successful runs MUST report non-negative elapsed times by resolution, state, and operation for every executed render and state total. Process-tree memory observations MUST remain diagnostic and MUST NOT change rendering acceptance thresholds or report schema.

#### Scenario: Complete the independent resolution jobs
- **WHEN** required PR rules-screen parity runs
- **THEN** the 960x540 and 1280x720 jobs each perform 25 renders across five states, and the 1920x1080 job checks all five semantic states while rendering original and edited with preview at 500 ms, range preview, and export, for exactly 56 PR renders with unchanged assertions and references for executed operations

#### Scenario: Execute full high-resolution evidence weekly
- **WHEN** the weekly schedule fires on the default branch or a maintainer dispatches the full workflow
- **THEN** the full 1920x1080 five-state, 25-render path runs with its original references, tolerances, native dependencies, and failure propagation

#### Scenario: Attribute costs and resource use
- **WHEN** all PR jobs and the scheduled full job succeed
- **THEN** their logs identify each of the 56 PR and 25 scheduled renders by resolution, lifecycle state, and operation, and report diagnostic elapsed and process-tree peak memory values without a universal rendering-performance pass/fail threshold

#### Scenario: Fail closed on omitted or broken evidence
- **WHEN** a PR matrix entry or native test is missing, duplicated, skipped, selects no tests, has missing or invalid native tools/font or scope, or a render or comparison fails
- **THEN** the corresponding PR job or policy validator fails and the foundation gate cannot report success or publish invalid rules-screen evidence

#### Scenario: Fail closed on scheduled evidence
- **WHEN** the weekly full workflow is missing, selects no test, skips a required render, lacks native dependencies, or its comparison fails
- **THEN** policy validation rejects the missing or weakened definition, or the scheduled Actions job reports failure without treating the incomplete run as successful full evidence

### Requirement: Multiresolution rules-screen render conformance
Frame preview, audiovisual range preview and final export MUST render the rules-screen through the production EvaluatedScene at 1920x1080, 1280x720 and 960x540. The required PR jobs MUST compare all original, edited, undone, redone, and reopened outputs at 960x540 and 1280x720, and original and edited outputs at 1920x1080, with independent scene expectations and reviewed visual references at executed timestamps. The weekly full 1920x1080 workflow MUST compare all five states at 0/500/900 ms and every render intent. The 1920x1080 PR job MUST still check independent semantic expectations for all five states. SSIM MUST be at least 0.99, aligned decoded float-PCM RMS error at most 0.0001, and timing deviation at most one output frame. Repeated and cold/warm-cache evaluations MUST preserve semantic plans and decoded visuals in the full conformance path. Native dependency/font failures MUST fail the respective job rather than skip. References MUST remain independently reviewed and hash-recorded; existing golden baselines and tolerances MUST remain intact.

#### Scenario: F3 Compare scales intents and lifecycle states across required jobs
- **WHEN** the PR jobs render their specified intent, timestamp, and lifecycle combinations and the weekly job renders the full 1920x1080 combinations
- **THEN** every executed output keeps named motifs at expected locations and order, and visual, audio, and timing results meet the existing tolerances

#### Scenario: F4 Detect shared drift and missing dependencies
- **WHEN** executed render intents share an incorrect motif/style/order or required renderer/font dependencies are absent
- **THEN** independent expectations or dependency checks fail the corresponding required PR or scheduled conformance without refreshing references or silently accepting incomplete coverage

### Requirement: Optimized golden conformance without reduced evidence
The protected PR render gates and weekly full-resolution gate collectively MUST execute the existing reviewed frame, audiovisual range, export, lifecycle, reference, cache, and report checks in an optimized Rust test profile where required. The original Render parity job MUST retain the non-rules-screen golden suites, native cache evidence, strict report validation, and report publication; the three PR rules-screen jobs MUST retain all three resolutions with the bounded 1920x1080 sample, and the weekly job MUST retain the full 1920x1080 lifecycle. All jobs MUST retain the same fixtures, assertions, tolerances, immutable references, fail-closed native dependencies, and report-only rendering-performance policy. Successful runs MUST expose elapsed time for each named golden conformance suite, resolution, and executed capture in CI logs.

#### Scenario: Complete reviewed evidence across jobs
- **WHEN** all required PR native render jobs and the scheduled full-resolution job run with approved dependencies and reviewed golden generations
- **THEN** every assertion selected for those jobs executes, the original report is strictly validated before publication, and each job's required timing records appear in its log

#### Scenario: Reject drift in any required job
- **WHEN** optimized execution in any PR or scheduled job produces a frame, audio, timing, semantic-plan, filter-graph, cache, or report result outside existing reviewed acceptance criteria
- **THEN** that job fails; a failing PR leaf also fails the protected foundation, and no invalid report is published

#### Scenario: Preserve measurement boundaries
- **WHEN** conformance succeeds on runners with different execution times
- **THEN** elapsed-time observations remain diagnostic and do not change golden pass/fail outcomes
