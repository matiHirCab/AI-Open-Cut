## MODIFIED Requirements

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable parity job identities, dependency relationship, unconditional aggregate execution, explicit leaf-result assertions, exact closed contract and render step sequences, exact approved properties and environments for every parity step, declared working directories, absence of workflow-level and parity-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of parity-job command defaults and containers, default failure propagation, strict report validation before publication, and publication of the exact validated report path. Environment maps on non-parity jobs MUST remain permitted.

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow containing every required job, exact positional leaf step, approved step property and environment, result assertion, failure-propagation rule, validation order, and report publication rule without inherited parity environment
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Permit non-parity job configuration
- **WHEN** a job outside contract, render, and foundation parity declares an environment map
- **THEN** the CI gate policy check continues evaluating the required parity invariants without rejecting that unrelated job configuration

#### Scenario: Reject inherited parity environment
- **WHEN** `workflow.env` or any parity-job `env` is present, including an empty map or keys such as `BASH_ENV`, `PATH`, `LD_PRELOAD`, `LD_AUDIT`, `NODE_OPTIONS`, or benign-looking metadata
- **THEN** the CI gate policy check fails before any inherited process control can be accepted

#### Scenario: Detect an added or reordered leaf step
- **WHEN** either leaf job gains, loses, duplicates, replaces, or reorders a step relative to its reviewed sequence
- **THEN** the CI gate policy check fails before unreviewed preparation or repository mutation can affect parity evidence

#### Scenario: Detect inherited execution alteration
- **WHEN** workflow or parity-job environment, workflow or parity-job command defaults, a custom parity-step shell, or a parity-job container changes how an authoritative command executes
- **THEN** the CI gate policy check fails before the altered execution model can be accepted

#### Scenario: Detect inherited golden mutation mode
- **WHEN** `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` is declared at workflow, render-job, critical render-step, or render-container scope
- **THEN** the CI gate policy check fails with the specific verification-bypass diagnostic regardless of the declared value or expression

#### Scenario: Detect a neutralized critical step
- **WHEN** a critical job or step enables ignored failures or an authoritative command gains shell control flow that can mask its exit status
- **THEN** the CI gate policy check fails with the weakened invariant

#### Scenario: Detect a weakened aggregate execution model
- **WHEN** the aggregate changes its unconditional execution, dependency result bindings, exact-success comparison, failure behavior, approved assertion properties, step environment, shell, inherited job environment or defaults, or runner container
- **THEN** the CI gate policy check fails before the aggregate can report success without executing its reviewed assertion

#### Scenario: Detect a reordered report publication
- **WHEN** the validated report upload occurs before strict external-report validation
- **THEN** the CI gate policy check fails before the workflow change can be accepted
