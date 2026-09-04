## MODIFIED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for the dedicated OpenSpec policy, contract-parity, and render-parity statuses; executes after every terminal prerequisite outcome; reports all three results and the OpenSpec completion attestation; and succeeds only when all three results are exactly `success` and the policy attestation is exactly `true`. The reviewed bootstrap MUST emit the attestation only after it validates the complete workflow, Moon, and proto boundary and the shell-free protected Moon task exits successfully; neither the workflow command nor the Moon child may emit or fabricate it directly. A failed, cancelled, skipped, neutralized, masked, or ignored-failure prerequisite MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: All required boundaries complete successfully
- **WHEN** the complete reviewed Moon policy task succeeds, the bootstrap emits `true`, and policy, contract, and render all report `success`
- **THEN** the aggregate logs the attestation and all three results, succeeds, and is available as the single branch-protection target

#### Scenario: Policy validation fails
- **WHEN** structural policy validation reports `failure` while both functional parity statuses report `success`
- **THEN** the aggregate still executes, logs its inputs, and fails

#### Scenario: Moon failure cannot forge policy success
- **WHEN** the protected Moon process fails to start or exits nonzero
- **THEN** the bootstrap emits no completion marker and the aggregate fails even when both functional parity statuses report `success`

#### Scenario: Policy execution is skipped or neutralized
- **WHEN** preflight rejects the reviewed boundary or the Moon task does not execute to successful completion
- **THEN** it does not emit the exact completion attestation and the aggregate fails

#### Scenario: One prerequisite fails
- **WHEN** any policy or parity prerequisite reports `failure`
- **THEN** the aggregate still executes, logs its inputs, and fails regardless of the attestation value

#### Scenario: One prerequisite is cancelled or skipped
- **WHEN** any policy or parity prerequisite reports `cancelled` or `skipped`
- **THEN** the aggregate still executes, logs its inputs, and fails regardless of the attestation value

### Requirement: Automated CI gate policy validation
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, and render step sequences, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, publication of the exact validated report path, and the complete effective Moon and proto policy execution boundary. Before repository-controlled Moon configuration is interpreted, the OpenSpec job MUST install explicit reviewed bootstrap versions, validate every protected workflow, Moon, and proto source, and refuse to launch Moon on any invalid input. The bootstrap MUST invoke only the explicitly qualified root Moon task without a shell, MUST withhold the GitHub output channel from the Moon child, and MUST emit the exact completion attestation only after the child exits successfully. The root project MUST reject inherited tasks and project-wide execution overrides; its workspace mapping, pinned toolchain, and `.prototools` versions MUST remain stable; global task configurations MAY serve other projects but MUST NOT inject global environment, implicit execution settings, or external extensions. Environment maps on unrelated GitHub jobs and project-local Moon configuration outside the root project MUST remain permitted.

#### Scenario: Validate before Moon execution
- **WHEN** workflow, root project, workspace, toolchain, proto pins, and global tasks match the reviewed policy
- **THEN** the bootstrap validates them before launching exactly `moon run root:openspec-validate`

#### Scenario: Reject pre-execution environment injection
- **WHEN** root or inherited Moon configuration declares `BASH_ENV`, a forged path, another environment map, or an unapproved execution override
- **THEN** the bootstrap fails without launching Moon or writing the completion attestation

#### Scenario: Reject proto configuration injection
- **WHEN** `.prototools` is missing, gains settings, environment or plugin configuration, uses an alternate source, or changes a reviewed version
- **THEN** validation fails before Moon launches and the completion output remains absent

#### Scenario: Reject Moon project redirection or toolchain mutation
- **WHEN** workspace configuration redirects the root project, changes its default identity, uses an external extension, or toolchain configuration changes a reviewed package manager or version
- **THEN** validation fails before Moon launches and the completion output remains absent

#### Scenario: Isolate global tasks from the protected root
- **WHEN** global task configuration exists for non-root projects without top-level environment, implicit execution settings, or external extension
- **THEN** repository validation permits that configuration while the root task remains excluded from inheritance

#### Scenario: Reject incomplete configuration inventory
- **WHEN** a required Moon or proto source is missing or an unsupported configuration file could affect task resolution without being validated
- **THEN** validation fails before Moon launches and the completion output remains absent

#### Scenario: Keep the output channel outside Moon
- **WHEN** the bootstrap launches the valid protected task
- **THEN** the child process does not receive `GITHUB_OUTPUT` and cannot own the policy step output

#### Scenario: Attest a completed policy task
- **WHEN** pre-execution validation succeeds and the exact Moon child exits with code zero
- **THEN** the bootstrap appends exactly `validated=true` once to the policy step output

#### Scenario: Reject failed policy execution
- **WHEN** pre-execution validation, process startup, or the Moon child fails
- **THEN** the bootstrap exits nonzero and writes no policy attestation

#### Scenario: Reject an inline or masked workflow attestation
- **WHEN** the workflow writes directly to `GITHUB_OUTPUT`, invokes Moon directly, wraps the bootstrap with `|| true`, or adds another command
- **THEN** structural validation fails and the reviewed bootstrap cannot emit a marker for that workflow

#### Scenario: Reject an altered Moon policy task
- **WHEN** the task loses, gains, reorders, duplicates, alters, or neutralizes any required command or fail-closed boundary
- **THEN** pre-execution structural validation fails without launching the task or writing the completion output

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow and complete Moon/proto boundary containing every required job, bootstrap step, command, exact positional protected step, approved property and environment, result and attestation assertion, failure-propagation rule, deterministic render setting, validation order, and report publication rule
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
- **WHEN** the OpenSpec job or any of its steps ignores failures, becomes conditional, uses a custom shell, inherits run defaults, runs in a container, resolves bootstrap versions from repository configuration, or changes its authoritative command, output, or step properties
- **THEN** the CI gate policy check fails before the policy result or completion proof can be accepted as trustworthy

#### Scenario: Detect inherited golden mutation mode
- **WHEN** `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` is declared at workflow, render-job, native-step, validation-step, or job-container scope
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
