## ADDED Requirements

### Requirement: Bounded and attributable rules-screen conformance execution
The native rules-screen conformance suite MUST execute the complete existing three-resolution by five-lifecycle-state matrix and all three frame previews, audiovisual range preview, final export, semantic-plan, reference, audio, timing, and project-integrity assertions for every case. Independent resolutions MAY execute concurrently with no more than two workers; lifecycle states within one resolution MUST execute in their existing order. If concurrency is unavailable, the suite MUST execute the same matrix serially. Worker or render failures MUST fail the required golden gate without silently omitting a case. Successful required runs MUST report non-negative elapsed times identified by resolution, state, and each preview timestamp, range preview, and export operation; these diagnostic timings MUST NOT become universal pass/fail budgets.

#### Scenario: Complete all independent cases
- **WHEN** required native golden conformance runs on a runner with at least two available workers
- **THEN** all 15 resolution/state cases execute exactly once with at most two concurrent resolution workers, preserve each resolution's original-to-reopened order, and apply every existing assertion

#### Scenario: Preserve coverage without parallel capacity
- **WHEN** the runner has only one available worker
- **THEN** the same 15 cases and all existing assertions execute serially

#### Scenario: Attribute render costs
- **WHEN** every rules-screen render operation succeeds
- **THEN** the required log identifies elapsed time for each of the 45 frame previews, 15 range previews, and 15 exports by resolution and lifecycle state, without changing pass/fail thresholds

#### Scenario: Fail closed on a worker or render error
- **WHEN** any resolution worker, render operation, comparison, or reviewed reference check fails
- **THEN** the native golden test fails and the required Render parity gate does not publish an invalid success report
