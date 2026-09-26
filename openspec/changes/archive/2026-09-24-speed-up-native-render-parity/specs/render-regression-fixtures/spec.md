## ADDED Requirements

### Requirement: Optimized golden conformance without reduced evidence
The required native golden conformance MUST execute the existing reviewed frame, audiovisual range, export, lifecycle, reference, and report checks in an optimized Rust test profile. The test MUST retain the same fixtures, assertions, tolerances, immutable references, fail-closed native dependencies, and report-only performance policy. Successful required runs MUST expose elapsed time for each named golden conformance suite and the sampled capture in CI logs; these observations MUST NOT be universal pass/fail budgets.

#### Scenario: Complete the reviewed evidence in an optimized profile
- **WHEN** the required Render parity job runs with its approved dependencies and reviewed golden generation
- **THEN** every existing native golden assertion executes, the same report is strictly validated before publication, and the named suite durations are visible in the job log

#### Scenario: Reject output drift under optimization
- **WHEN** optimized execution produces a frame, audio, timing, semantic-plan, filter-graph, or report result outside the existing reviewed acceptance criteria
- **THEN** the required leaf status fails and no invalid report is published

#### Scenario: Preserve measurement boundaries
- **WHEN** conformance succeeds on runners with different execution times
- **THEN** elapsed-time observations remain diagnostic and do not change golden pass/fail outcomes
