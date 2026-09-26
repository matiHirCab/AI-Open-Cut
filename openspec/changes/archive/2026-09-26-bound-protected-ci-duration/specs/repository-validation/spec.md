## ADDED Requirements

### Requirement: Protected CI elapsed-time budget
The required OpenCut workflow MUST measure elapsed time from the workflow run's recorded start to its final protected foundation assertion after all required validation jobs reach terminal states. Its default budget MUST be 120 minutes. The foundation status MUST fail when the measured duration exceeds the effective budget, when timing evidence cannot be obtained or parsed, or when any required validation result is unsuccessful. A time-budget failure MUST NOT omit or weaken any existing correctness, contract, render, rules-screen, smoke, or OpenSpec assertion.

#### Scenario: Complete within the default budget
- **WHEN** every required validation job succeeds, the workflow's recorded elapsed time is at most 120 minutes, and no exception is active
- **THEN** the protected foundation reports the measured duration and 120-minute budget and can succeed

#### Scenario: Exceed the default budget without an exception
- **WHEN** the measured elapsed time exceeds 120 minutes and no valid reviewed exception applies
- **THEN** the protected foundation reports the overrun and fails

#### Scenario: Timing evidence is unavailable
- **WHEN** the workflow run start cannot be read or parsed, the elapsed time is negative, or the clock data is otherwise invalid
- **THEN** the protected foundation fails without assuming a duration inside budget

#### Scenario: Other validation fails
- **WHEN** any required validation job fails, is cancelled, or is skipped
- **THEN** the protected foundation fails even if elapsed time is inside the effective budget

### Requirement: Bounded and justified CI duration exception
An exception to the default CI budget MUST name an accountable owner, cause, measured baseline, evidence link, expiration date, and a positive hard cap no greater than 135 minutes. The protected foundation MUST display the reason and effective cap when the exception applies. An exception MUST be inactive after its expiration date and MUST NOT bypass required test selection, assertions, or failure propagation. The reviewed workflow policy MUST reject missing, duplicated, malformed, unbounded, or unreviewed duration controls.

#### Scenario: Reviewed exception covers a measured overrun
- **WHEN** a complete, unexpired exception applies and successful required validation finishes above 120 minutes but no later than its hard cap
- **THEN** the protected foundation reports the baseline, evidence, reason, owner, expiration, and measured duration and can succeed

#### Scenario: Exception hard cap is exceeded
- **WHEN** the measured elapsed time exceeds the exception hard cap
- **THEN** the protected foundation fails and reports the hard-cap overrun

#### Scenario: Exception is invalid or expired
- **WHEN** an exception is incomplete, duplicated, malformed, has a cap above 135 minutes, or is past its expiration date
- **THEN** workflow policy validation or the foundation rejects it; the exception cannot authorize an overrun

#### Scenario: Duration control is weakened
- **WHEN** the duration assertion is removed, skipped, masked, detached from the protected foundation, or any required prerequisite is dropped
- **THEN** the repository's CI policy validator rejects the workflow before it can pass the protected gate

## MODIFIED Requirements

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for dedicated OpenSpec policy, contract-parity, render-parity, rules-screen-parity, correctness, and packaged-smoke statuses; executes after every terminal prerequisite outcome; reports all six results and the OpenSpec completion attestation; and succeeds only when all six results are exactly `success`, the attestation is exactly `true`, and the duration assertion passes. The reviewed bootstrap MUST emit attestation only after validating the complete workflow, Moon, and proto boundary and successful shell-free protected Moon execution; neither the workflow command nor the Moon child may emit or fabricate it directly. A failed, cancelled, skipped, neutralized, masked, or ignored-failure prerequisite MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: All six required boundaries complete successfully within the duration budget
- **WHEN** the complete reviewed Moon policy task succeeds, the bootstrap emits `true`, and policy, contract, original render, three-resolution rules-screen parity, correctness, and packaged smoke all report `success` within the effective duration budget
- **THEN** the aggregate logs the attestation and all six results, succeeds, and is available as the single branch-protection target

#### Scenario: Any rules-screen shard fails or is omitted
- **WHEN** any required rules-screen resolution job fails, is cancelled, skipped, duplicated, or absent
- **THEN** policy validation or the unconditional foundation assertion fails even if the original Render parity job succeeds

#### Scenario: Policy validation fails
- **WHEN** structural policy validation reports `failure` while all five functional validation statuses report `success`
- **THEN** the aggregate still executes, logs its inputs, and fails

#### Scenario: Moon failure cannot forge policy success
- **WHEN** the protected Moon process fails to start or exits nonzero
- **THEN** the bootstrap emits no completion marker and the aggregate fails even when all five functional validation statuses report `success`

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
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, original render, and rules-screen step sequences, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, publication of the exact validated report path, and the complete effective Bun, Moon, and proto policy execution boundary. The rules-screen matrix MUST contain exactly 960x540, 1280x720, and 1920x1080, use at most three concurrent Linux jobs without short-circuiting failure collection, discover and execute the exact native test in an optimized profile, require step-local tools/font/resolution inputs, and fail if selection matches zero tests. Foundation parity MUST directly require and check all six prerequisite results and the OpenSpec attestation. The original Render parity job MUST retain native cache evidence and strict report validation before its unchanged publication. Before repository-controlled Bun or Moon configuration is interpreted, the OpenSpec job MUST install explicit reviewed bootstrap versions and invoke Bun with an empty trusted configuration and automatic dotenv loading disabled. The bootstrap MUST validate every protected workflow, Bun, Moon, and proto source and refuse to launch Moon on any invalid input. The bootstrap MUST invoke only the explicitly qualified root Moon task without a shell, MUST withhold the GitHub output channel from the Moon child, and MUST emit the exact completion attestation only after the child exits successfully. The root project MUST reject inherited tasks and project-wide execution overrides; its canonical Bun configuration, workspace mapping, pinned toolchain, and `.prototools` versions MUST remain stable. Every protected Bun command in the Moon task MUST explicitly use the validated Bun configuration with automatic dotenv loading disabled, and the task MUST execute the real-Moon startup-hook regression on its supported CI platform. Global task configurations MAY serve other projects but MUST NOT inject global environment, implicit execution settings, or external extensions. Environment maps on unrelated GitHub jobs, ignored local dotenv files outside protected execution, and project-local Moon configuration outside the root project MUST remain permitted.

#### Scenario: Accept all three required resolution shards
- **WHEN** the workflow lists exactly 960x540, 1280x720, and 1920x1080 with at most three parallel Linux jobs, the approved tool/font environment, exact test-discovery and test-execution commands, default failure propagation, and a six-result foundation assertion plus an unconditional final duration assertion
- **THEN** policy validation accepts the distributed rules-screen evidence while preserving the original report and all existing gates

#### Scenario: Reject missing, duplicated, or zero-test evidence
- **WHEN** a resolution is missing or duplicated, the matrix or test selector changes, the exact test cannot be discovered, a shard becomes optional, or its command/environment is neutralized
- **THEN** policy validation or the shard fails before the foundation gate can succeed

#### Scenario: Reject weakened aggregate or report ownership
- **WHEN** the foundation gate omits either render result or accepts failure/skipping, or the original Render parity report validation/publication order or path changes
- **THEN** policy validation rejects the workflow

#### Scenario: Validate the isolated Bun and Moon policy boundary
- **WHEN** workflow, canonical Bun configuration, root project, workspace, toolchain, proto pins, and global tasks match the reviewed policy
- **THEN** the bootstrap starts without checkout-controlled Bun configuration or dotenv loading, validates the complete boundary, and then launches exactly `moon run root:openspec-validate`

#### Scenario: Reject pre-bootstrap Bun preload forgery
- **WHEN** checkout-controlled Bun configuration attempts to preload code that writes the policy output or exits before the bootstrap body
- **THEN** the workflow invocation does not load that configuration and preflight rejects it without launching Moon or writing the completion attestation

#### Scenario: Exclude dotenv process-control injection
- **WHEN** a checkout or ignored local dotenv file declares `BASH_ENV`, `NODE_OPTIONS`, or another process-control value
- **THEN** neither the policy bootstrap nor any protected Bun command loads that file

#### Scenario: Reject altered canonical Bun configuration
- **WHEN** `bunfig.toml` is absent, gains `preload`, imports, settings, or other properties, or differs from the reviewed dependency-install policy
- **THEN** validation fails before Moon launches and the completion output remains absent

#### Scenario: Execute the real-Moon startup-hook regression
- **WHEN** the protected policy task runs on Ubuntu with Moon available
- **THEN** it executes the regression that reproduces the vulnerable startup hook and proves the bootstrap blocks the same configuration before child execution

#### Scenario: Reject root project execution injection
- **WHEN** root `moon.yml` declares an environment map, platform, toolchain, Docker setting, altered inheritance control, or another unapproved root property
- **THEN** validation fails without launching Moon or invoking the output writer

#### Scenario: Reject Moon project redirection or toolchain mutation
- **WHEN** workspace configuration redirects the root project, changes its default identity, uses an external extension, or toolchain or proto configuration changes a reviewed package manager or version
- **THEN** validation fails without launching Moon or invoking the output writer

#### Scenario: Reject proto configuration injection
- **WHEN** `.prototools` is missing, gains settings, environment or plugin configuration, uses an alternate source, or changes a reviewed version
- **THEN** validation fails before Moon launches and the completion output remains absent

#### Scenario: Isolate global tasks from the protected root
- **WHEN** global task configuration exists for non-root projects without a top-level environment or external extension
- **THEN** repository validation permits that configuration while the root task remains excluded from inheritance

#### Scenario: Reject global Moon environment injection
- **WHEN** any discovered global task configuration declares top-level `env` or `extends`, including an empty map or literal or expression-valued process control
- **THEN** validation fails before the protected task can accept inherited execution configuration or launch

#### Scenario: Reject incomplete policy configuration inventory
- **WHEN** a required Bun, Moon, or proto source is missing or an unsupported configuration file could affect task resolution without being validated
- **THEN** validation fails without launching Moon or emitting the completion proof

#### Scenario: Validate policy completion proof
- **WHEN** pre-execution validation succeeds and the exact shell-free Moon child exits with code zero
- **THEN** only the bootstrap appends exactly `validated=true` once to the policy step output

#### Scenario: Keep the output channel outside Moon
- **WHEN** the bootstrap launches the valid protected task
- **THEN** the child process does not receive `GITHUB_OUTPUT` and cannot own the policy step output

#### Scenario: Reject failed policy execution
- **WHEN** pre-execution validation, process startup, or the Moon child fails
- **THEN** the bootstrap exits nonzero and writes no policy attestation

#### Scenario: Reject an inline or masked workflow attestation
- **WHEN** the workflow writes directly to `GITHUB_OUTPUT`, invokes Moon directly, omits or alters Bun isolation flags, wraps the bootstrap with `|| true`, or adds any other command
- **THEN** the CI gate policy check fails and the reviewed bootstrap cannot emit a marker for that workflow

#### Scenario: Reject an altered Moon policy task
- **WHEN** the task loses, gains, reorders, duplicates, alters, or neutralizes any required command, removes a fail-closed boundary, omits Bun isolation flags, or stops executing the real-Moon regression
- **THEN** structural validation fails without launching the task or invoking the output writer

#### Scenario: Validate the required gate structure
- **WHEN** repository validation reads a workflow and complete Bun, Moon, and proto boundary containing every required job, command, exact positional protected step, approved property and environment, result and attestation assertion, failure-propagation rule, validation order, and report publication rule without inherited protected-job environment
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
