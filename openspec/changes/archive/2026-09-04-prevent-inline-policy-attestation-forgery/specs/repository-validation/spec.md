# Repository Validation Delta

## MODIFIED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for the dedicated OpenSpec policy, contract-parity, and render-parity statuses; executes after every terminal prerequisite outcome; reports all three results and the OpenSpec completion attestation; and succeeds only when all three results are exactly `success` and the policy attestation is exactly `true`. The attestation MUST be emitted by the final validator in the reviewed Moon policy task only after the complete task succeeds; the workflow command MUST NOT emit or fabricate it inline. A failed, cancelled, skipped, neutralized, masked, or ignored-failure prerequisite MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: All required boundaries complete successfully
- **WHEN** the complete reviewed Moon policy task succeeds, its final validator emits `true`, and policy, contract, and render all report `success`
- **THEN** the aggregate logs the attestation and all three results, succeeds, and is available as the single branch-protection target

#### Scenario: Policy validation fails
- **WHEN** structural policy validation reports `failure` while both functional parity statuses report `success`
- **THEN** the aggregate still executes, logs its inputs, and fails

#### Scenario: Policy failure is masked outside the Moon task
- **WHEN** a workflow shell wrapper converts a failed `moon run openspec-validate` exit status into success
- **THEN** no final-validator marker exists and the aggregate fails even when both functional parity statuses report `success`

#### Scenario: Policy execution is skipped or neutralized
- **WHEN** the reviewed Moon task does not execute through its final successful validator
- **THEN** it does not emit the exact completion attestation and the aggregate fails

#### Scenario: One prerequisite fails
- **WHEN** any policy or parity prerequisite reports `failure`
- **THEN** the aggregate still executes, logs its inputs, and fails regardless of the attestation value

#### Scenario: One prerequisite is cancelled or skipped
- **WHEN** any policy or parity prerequisite reports `cancelled` or `skipped`
- **THEN** the aggregate still executes, logs its inputs, and fails regardless of the attestation value

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, and render step sequences, the exact ordered and fail-closed Moon policy task, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, and publication of the exact validated report path. The workflow MUST invoke only the canonical Moon task, every Moon phase MUST be joined so a failure prevents later commands, and only the final validator in that task MAY emit the completion attestation after workflow and Moon configuration validation succeeds. Environment maps on unrelated jobs MUST remain permitted.

#### Scenario: Validate policy completion proof
- **WHEN** the workflow invokes only the canonical Moon task and its exact final validator emits the output after all preceding policy commands succeed
- **THEN** the CI gate policy check accepts the completion-proof structure

#### Scenario: Reject an inline or masked workflow attestation
- **WHEN** the workflow writes directly to `GITHUB_OUTPUT`, wraps the Moon command with `|| true`, or adds any other command
- **THEN** the CI gate policy check fails and the reviewed Moon task cannot emit a marker for that workflow

#### Scenario: Reject an altered Moon policy task
- **WHEN** the task loses, gains, reorders, duplicates, alters, or neutralizes any required command, removes a fail-closed command boundary, or invokes the attesting validator before the final position
- **THEN** structural validation fails without invoking the output writer

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow and Moon task containing every required job, command, exact positional protected step, approved property and environment, result and attestation assertion, failure-propagation rule, validation order, and report publication rule without inherited protected-job environment
- **THEN** the CI gate policy check succeeds without executing application behavior

#### Scenario: Permit unrelated job configuration
- **WHEN** a job outside OpenSpec policy, contract, render, and foundation parity declares an environment map
- **THEN** the CI gate policy check continues evaluating required invariants without rejecting that unrelated job configuration

#### Scenario: Reject inherited protected-job environment
- **WHEN** `workflow.env` or any protected-job `env` is present, including an empty map or literal or expression-valued process control
- **THEN** the CI gate policy check fails before inherited configuration can alter reviewed execution

#### Scenario: Detect an added or reordered protected step
- **WHEN** the OpenSpec, contract, or render job gains, loses, duplicates, replaces, or reorders a step relative to its reviewed sequence
- **THEN** the CI gate policy check fails before unreviewed preparation can affect policy or parity evidence

#### Scenario: Detect policy-job neutralization
- **WHEN** the OpenSpec job or any of its steps ignores failures, becomes conditional, uses a custom shell, inherits run defaults, runs in a container, or changes its authoritative command, output, or step properties
- **THEN** the CI gate policy check fails before the policy result or completion proof can be accepted as trustworthy

#### Scenario: Detect inherited golden mutation mode
- **WHEN** `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` is declared at workflow, render-job, critical render-step, or render-container scope
- **THEN** the CI gate policy check fails with the specific verification-bypass diagnostic regardless of the declared value or expression

#### Scenario: Detect a neutralized parity step
- **WHEN** a parity job or critical step enables ignored failures or an authoritative command gains shell control flow that can mask its exit status
- **THEN** the CI gate policy check fails with the weakened invariant

#### Scenario: Detect a weakened aggregate execution model
- **WHEN** the aggregate changes its unconditional execution, three direct dependencies, prerequisite result bindings, policy-attestation binding, exact-success comparison, failure behavior, approved assertion properties, step environment, shell, inherited job environment or defaults, or runner container
- **THEN** the CI gate policy check fails before the aggregate can report success without executing its reviewed assertion

#### Scenario: Detect a reordered report publication
- **WHEN** the validated report upload occurs before strict external-report validation
- **THEN** the CI gate policy check fails before the workflow change can be accepted
