# Repository Validation Delta

## MODIFIED Requirements

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, and render step sequences, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, publication of the exact validated report path, and the complete effective Moon policy execution boundary. The workflow MUST invoke only the explicitly qualified root Moon task, and every Moon phase MUST remain exact, ordered, and fail-closed. The root project MUST reject inherited tasks and project-wide execution overrides; its workspace mapping and pinned toolchain MUST remain stable; global task configurations MAY serve other projects but MUST NOT inject global environment, implicit execution settings, or external extensions. Only the final validator MAY emit the completion attestation after workflow, root project, workspace, toolchain, and every discovered global-task configuration pass validation. Environment maps on unrelated GitHub jobs and project-local Moon configuration outside the root project MUST remain permitted.

#### Scenario: Validate the isolated Moon policy boundary
- **WHEN** the workflow invokes `root:openspec-validate`, the root project disables task inheritance, workspace and toolchain configuration match the reviewed values, and global tasks do not inject environment or extensions
- **THEN** repository validation accepts the effective Moon execution boundary and the final validator may emit the completion proof

#### Scenario: Reject root project execution injection
- **WHEN** root `moon.yml` declares an environment map, platform, toolchain, Docker setting, altered inheritance control, or another unapproved root property
- **THEN** validation fails without invoking the output writer

#### Scenario: Reject Moon project redirection or toolchain mutation
- **WHEN** workspace configuration redirects the root project, changes its default identity, uses an external extension, or toolchain configuration changes the Bun package manager or pinned version
- **THEN** validation fails without invoking the output writer

#### Scenario: Isolate global tasks from the protected root
- **WHEN** global task configuration exists for non-root projects without a top-level environment or external extension
- **THEN** repository validation permits that configuration while the root task remains excluded from inheritance

#### Scenario: Reject global Moon environment injection
- **WHEN** any discovered global task configuration declares top-level `env` or `extends`, including an empty map or literal or expression-valued process control
- **THEN** validation fails before the protected task can accept inherited execution configuration

#### Scenario: Reject incomplete Moon configuration inventory
- **WHEN** a required Moon source is missing or an unsupported configuration file could affect task resolution without being validated
- **THEN** validation fails without emitting the completion proof

#### Scenario: Validate policy completion proof
- **WHEN** the workflow invokes only the canonical qualified Moon task and its exact final validator emits the output after all preceding policy commands and configuration checks succeed
- **THEN** the CI gate policy check accepts the completion-proof structure

#### Scenario: Reject an inline or masked workflow attestation
- **WHEN** the workflow writes directly to `GITHUB_OUTPUT`, wraps the qualified Moon command with `|| true`, or adds any other command
- **THEN** the CI gate policy check fails and the reviewed Moon task cannot emit a marker for that workflow

#### Scenario: Reject an altered Moon policy task
- **WHEN** the task loses, gains, reorders, duplicates, alters, or neutralizes any required command, removes a fail-closed command boundary, or invokes the attesting validator before the final position
- **THEN** structural validation fails without invoking the output writer

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow and complete Moon boundary containing every required job, command, exact positional protected step, approved property and environment, result and attestation assertion, failure-propagation rule, validation order, and report publication rule without inherited protected-job environment
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
