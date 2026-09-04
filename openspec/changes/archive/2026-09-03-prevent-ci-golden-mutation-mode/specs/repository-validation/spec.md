## MODIFIED Requirements

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable parity job identities, dependency relationship, unconditional aggregate execution, explicit leaf-result assertions, exact authoritative contract and render steps, declared working directories, exact critical render environment maps, absence of inherited golden mutation or alternate-capture modes, default failure propagation, strict report validation before publication, and publication of the exact validated report path. Unrelated global environment variables MUST remain permitted.

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow containing every required job, dependency, exact step, deterministic setting, approved environment map, result assertion, failure-propagation rule, validation order, and report publication rule
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Permit unrelated global configuration
- **WHEN** workflow or job environment configuration contains no golden mode variable and does not alter an exact critical-step environment
- **THEN** the CI gate policy check continues evaluating the required parity invariants without rejecting that unrelated configuration

#### Scenario: Detect inherited golden mutation mode
- **WHEN** `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` is declared at workflow, render-job, or critical render-step scope
- **THEN** the CI gate policy check fails regardless of the declared value or expression

#### Scenario: Detect a neutralized critical step
- **WHEN** a critical job or step enables ignored failures or an authoritative command gains shell control flow that can mask its exit status
- **THEN** the CI gate policy check fails with the weakened invariant

#### Scenario: Detect a reordered report publication
- **WHEN** the validated report upload occurs before strict external-report validation
- **THEN** the CI gate policy check fails before the workflow change can be accepted

#### Scenario: Detect a weakened aggregate
- **WHEN** unconditional execution, either leaf result, the exact-success comparison, or explicit failure behavior is removed or changed incompatibly
- **THEN** the CI gate policy check fails before the workflow change can be accepted
