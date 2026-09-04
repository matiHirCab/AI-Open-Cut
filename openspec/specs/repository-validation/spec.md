# Repository Validation Specification

## Purpose

Define reliable repository validation behavior for canonical branch resolution and required gates.
## Requirements
### Requirement: Canonical default-branch validation
Repository validation MUST use the repository's canonical `main` branch as the VCS comparison base and MUST execute required gates in a clean default-branch checkout without referencing an absent legacy branch.

#### Scenario: Validate a push checkout on the default branch
- **WHEN** continuous integration checks out a push commit on `main` and invokes the required Moon OpenSpec task
- **THEN** Moon constructs the task graph using `main` as its default VCS revision and runs the pinned normalization and strict OpenSpec validators without attempting to resolve `master`

#### Scenario: Validate a pull-request checkout
- **WHEN** continuous integration checks out a pull request targeting `main` and invokes the same Moon task
- **THEN** the task uses a valid repository revision context and runs the identical pinned validators

### Requirement: Validation configuration compatibility
Repository validation MUST preserve the existing application, public contract, persisted data, and required CI job behavior while correcting default-branch resolution.

#### Scenario: Apply the branch-resolution correction
- **WHEN** the canonical VCS default branch is configured for repository validation
- **THEN** no application runtime, public or persisted contract, migration, test selection, or required CI job is removed or changed

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
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, and render step sequences, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, publication of the exact validated report path, and the complete effective Bun, Moon, and proto policy execution boundary. Before repository-controlled Bun or Moon configuration is interpreted, the OpenSpec job MUST install explicit reviewed bootstrap versions and invoke Bun with an empty trusted configuration and automatic dotenv loading disabled. The bootstrap MUST validate every protected workflow, Bun, Moon, and proto source and refuse to launch Moon on any invalid input. The bootstrap MUST invoke only the explicitly qualified root Moon task without a shell, MUST withhold the GitHub output channel from the Moon child, and MUST emit the exact completion attestation only after the child exits successfully. The root project MUST reject inherited tasks and project-wide execution overrides; its canonical Bun configuration, workspace mapping, pinned toolchain, and `.prototools` versions MUST remain stable. Every protected Bun command in the Moon task MUST explicitly use the validated Bun configuration with automatic dotenv loading disabled, and the task MUST execute the real-Moon startup-hook regression on its supported CI platform. Global task configurations MAY serve other projects but MUST NOT inject global environment, implicit execution settings, or external extensions. Environment maps on unrelated GitHub jobs, ignored local dotenv files outside protected execution, and project-local Moon configuration outside the root project MUST remain permitted.

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

### Requirement: Attributed Bun bootstrap regression evidence
Repository validation MUST exercise the hardened bootstrap with a malicious checkout-controlled `bunfig.toml` while every other required workflow, Moon, and proto source is valid. The regression MUST distinguish the canonical Bun-configuration preflight rejection from unrelated missing-source or process failures and MUST prove that the preload, Moon child, and policy attestation remain unreachable. Its independent real-Moon reproduction MUST isolate inherited parent Moon/proto metadata and stores and MUST have a bounded execution budget sufficient for nested startup on the protected Ubuntu runner.

#### Scenario: Reject the malicious Bun configuration for the intended reason
- **WHEN** the real hardened Bun invocation receives a valid reviewed workflow and otherwise-canonical Moon and proto boundary with only `bunfig.toml` altered to preload forgery code
- **THEN** it exits nonzero with the canonical Bun-configuration rejection, creates no preload sentinel, launches no Moon child, and writes no policy attestation

### Requirement: Archive-only OpenSpec merge readiness
The protected repository policy MUST reject every unarchived entry under `openspec/changes` and MUST emit no completion attestation until the directory contains only the canonical `archive` directory. The changes root and archive path MUST be ordinary directories; files, directories, symbolic links, malformed entries, and multiple concurrent entries outside `archive` MUST all fail closed before Moon launches. Active changes MAY exist during local authoring, but they MUST be completed, synchronized, verified, and archived before the protected merge-ready gate can succeed.

#### Scenario: Accept an archive-only repository
- **WHEN** `openspec/changes` and `openspec/changes/archive` are ordinary directories and no other direct entry exists
- **THEN** repository policy continues to the protected Moon task

#### Scenario: Reject an unarchived change
- **WHEN** any file, directory, or symbolic link other than `archive` exists directly under `openspec/changes`
- **THEN** preflight reports every unarchived entry, launches no Moon child, and emits no policy attestation

#### Scenario: Reject an invalid archive boundary
- **WHEN** the changes root or canonical archive path is missing, is not a directory, or is a symbolic link
- **THEN** preflight fails before Moon launch and emits no policy attestation

### Requirement: Portable Transform2D correctness and required native coverage
Font-metric unit tests MUST use checked-in licensed fixtures without host font dependencies. Native Transform2D tests MUST use explicitly configured FFmpeg, FFprobe, and font paths for every subprocess; absent optional configuration SHALL skip only native cases, while partial configuration and missing required dependencies MUST fail. The protected Linux native parity job MUST execute the Transform2D integration target alongside existing golden and headless lifecycle tests, with matching policy validation.

#### Scenario: Run ordinary correctness on each platform
- **WHEN** Windows, Linux, or macOS correctness runs without native configuration
- **THEN** font and non-native tests run without installed rendering tools or system fonts

#### Scenario: Honor explicit native configuration
- **WHEN** valid absolute tools are configured but unavailable on PATH
- **THEN** native Transform2D tests execute successfully using those paths

#### Scenario: Reject incomplete required execution
- **WHEN** native configuration is partial or required mode lacks usable dependencies
- **THEN** the suite fails rather than silently skipping

#### Scenario: Protect native coverage
- **WHEN** the required Transform2D CI command is missing, altered, or neutralized
- **THEN** repository policy validation fails
