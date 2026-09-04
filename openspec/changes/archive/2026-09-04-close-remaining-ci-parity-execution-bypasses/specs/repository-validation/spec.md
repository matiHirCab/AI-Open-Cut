## MODIFIED Requirements

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable parity job identities, dependency relationship, unconditional aggregate execution, explicit leaf-result assertions, exact closed contract and render step sequences, exact approved properties and environments for every parity step, declared working directories, absence of inherited golden mutation or alternate-capture modes, absence of inherited Bash startup hooks, absence of parity-job command defaults and containers, default failure propagation, strict report validation before publication, and publication of the exact validated report path. Unrelated workflow- or parity-job-level environment variables MUST remain permitted.

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow containing every required job, exact positional leaf step, approved step property, deterministic setting, approved environment map, result assertion, failure-propagation rule, validation order, and report publication rule
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Permit unrelated global configuration
- **WHEN** workflow or parity-job environment configuration contains neither a golden mode variable nor `BASH_ENV` and does not alter an exact critical-step environment
- **THEN** the CI gate policy check continues evaluating the required parity invariants without rejecting that unrelated configuration

#### Scenario: Detect an added or reordered leaf step
- **WHEN** either leaf job gains, loses, duplicates, replaces, or reorders a step relative to its reviewed sequence
- **THEN** the CI gate policy check fails before unreviewed preparation or repository mutation can affect parity evidence

#### Scenario: Detect inherited execution alteration
- **WHEN** workflow or parity-job command defaults, a custom parity-step shell, a parity-job container, or inherited `BASH_ENV` changes how an authoritative command executes
- **THEN** the CI gate policy check fails before the altered execution model can be accepted

#### Scenario: Detect inherited golden mutation mode
- **WHEN** `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` is declared at workflow, render-job, critical render-step, or render-container scope
- **THEN** the CI gate policy check fails regardless of the declared value or expression

#### Scenario: Detect a neutralized critical step
- **WHEN** a critical job or step enables ignored failures or an authoritative command gains shell control flow that can mask its exit status
- **THEN** the CI gate policy check fails with the weakened invariant

#### Scenario: Detect a weakened aggregate execution model
- **WHEN** the aggregate changes its unconditional execution, dependency result bindings, exact-success comparison, failure behavior, approved assertion properties, environment, shell, inherited defaults, or runner container
- **THEN** the CI gate policy check fails before the aggregate can report success without executing its reviewed assertion

#### Scenario: Detect a reordered report publication
- **WHEN** the validated report upload occurs before strict external-report validation
- **THEN** the CI gate policy check fails before the workflow change can be accepted
