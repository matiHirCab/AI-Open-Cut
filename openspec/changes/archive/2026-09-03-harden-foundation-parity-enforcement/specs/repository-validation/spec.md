## MODIFIED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for both the dedicated contract-parity and render-parity statuses, executes after every terminal leaf outcome, reports both leaf results, and succeeds only when both results are exactly `success`. A failed, cancelled, or skipped leaf MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: Both parity boundaries pass
- **WHEN** the dedicated contract-parity and render-parity statuses both report `success`
- **THEN** the aggregate motion-graphics foundation status logs both results, succeeds, and is available as the single branch-protection target

#### Scenario: One parity boundary fails
- **WHEN** either dedicated parity status reports `failure`
- **THEN** the aggregate status still executes, logs both results, and fails

#### Scenario: One parity boundary is cancelled or skipped
- **WHEN** either dedicated parity status reports `cancelled` or `skipped`
- **THEN** the aggregate status still executes, logs both results, and fails

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable parity job identities, dependency relationship, unconditional aggregate execution, explicit leaf-result assertions, exact authoritative contract and render steps, declared working directories, deterministic render configuration, default failure propagation, strict report validation before publication, and publication of the exact validated report path.

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow containing every required job, dependency, exact step, deterministic setting, result assertion, failure-propagation rule, validation order, and report publication rule
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Detect a neutralized critical step
- **WHEN** a critical job or step enables ignored failures or an authoritative command gains shell control flow that can mask its exit status
- **THEN** the CI gate policy check fails with the weakened invariant

#### Scenario: Detect a reordered report publication
- **WHEN** the validated report upload occurs before strict external-report validation
- **THEN** the CI gate policy check fails before the workflow change can be accepted

#### Scenario: Detect a weakened aggregate
- **WHEN** unconditional execution, either leaf result, the exact-success comparison, or explicit failure behavior is removed or changed incompatibly
- **THEN** the CI gate policy check fails before the workflow change can be accepted
