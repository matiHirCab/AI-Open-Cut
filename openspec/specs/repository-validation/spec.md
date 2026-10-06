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
An exception to the default CI budget MUST name an accountable owner, cause, measured baseline, evidence link, expiration date, and a positive hard cap no greater than 180 minutes. The protected foundation MUST display the reason and effective cap when the exception applies. An exception MUST be inactive after its expiration date and MUST NOT bypass required test selection, assertions, or failure propagation. The reviewed workflow policy MUST reject missing, duplicated, malformed, unbounded, or unreviewed duration controls. With no active exception, the protected PR workflow MUST use its 120-minute default duration budget, every required leaf job including the rules-screen matrix MUST retain a 135-minute timeout, and the foundation job MUST retain a 10-minute timeout. The separate weekly full-resolution workflow MUST use its own bounded timeout and MUST NOT extend or change the PR duration budget.

#### Scenario: Reviewed exception covers a measured overrun
- **WHEN** a complete, unexpired exception applies and successful required validation finishes above 120 minutes but no later than its hard cap
- **THEN** the protected foundation reports the baseline, evidence, reason, owner, expiration, and measured duration and can succeed

#### Scenario: Exception hard cap is exceeded
- **WHEN** the measured elapsed time exceeds the exception hard cap
- **THEN** the protected foundation fails and reports the hard-cap overrun

#### Scenario: Exception is invalid or expired
- **WHEN** an exception is incomplete, duplicated, malformed, has a cap above 180 minutes, or is past its expiration date
- **THEN** workflow policy validation or the foundation rejects it; the exception cannot authorize an overrun

#### Scenario: Duration control is weakened
- **WHEN** the duration assertion is removed, skipped, masked, detached from the protected foundation, or any required prerequisite is dropped
- **THEN** the repository's CI policy validator rejects the workflow before it can pass the protected gate

#### Scenario: Use the default PR budget without an exception
- **WHEN** the PR workflow uses the bounded rules-screen sample and declares no exception fields
- **THEN** the foundation measures and enforces its 120-minute default budget

#### Scenario: Keep required jobs bounded
- **WHEN** any required PR leaf job declares a timeout other than 135 minutes, the foundation declares a timeout other than 10 minutes, or the weekly full-resolution job exceeds its approved timeout
- **THEN** repository policy validation rejects the workflow

### Requirement: Stable motion-graphics foundation status
Repository validation MUST publish one stable aggregate foundation status that waits for dedicated OpenSpec policy, contract-parity, render-parity, rules-screen-parity, correctness, and packaged-smoke statuses; executes after every terminal prerequisite outcome; reports all six results and the OpenSpec completion attestation; and succeeds only when all six results are exactly `success`, the attestation is exactly `true`, and the duration assertion passes. The reviewed bootstrap MUST emit attestation only after validating the complete workflow, Moon, and proto boundary and successful shell-free protected Moon execution; neither the workflow command nor the Moon child may emit or fabricate it directly. A failed, cancelled, skipped, neutralized, masked, or ignored-failure prerequisite MUST produce a non-successful aggregate status suitable as the single branch-protection target.

#### Scenario: All six required boundaries complete successfully within the duration budget
- **WHEN** the complete reviewed Moon policy task succeeds, the bootstrap emits `true`, and policy, contract, original render, the three-resolution PR rules-screen matrix including its bounded 1920x1080 sample, correctness, and packaged smoke all report `success` within the default duration budget
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
The repository's pinned validation workflow MUST structurally verify the stable OpenSpec policy and parity job identities, dependency relationship, unconditional aggregate execution, explicit prerequisite-result and policy-attestation assertions, exact closed OpenSpec, contract, original render, and rules-screen step sequences, exact approved properties and environments for every protected step, declared working directories, absence of workflow-level and protected-job-level environment maps, absence of golden mutation or alternate-capture modes, absence of protected-job command defaults and containers, default failure propagation, strict report validation before publication, publication of the exact validated report path, and the complete effective Bun, Moon, and proto policy execution boundary. The PR rules-screen matrix MUST contain exactly 960x540, 1280x720, and 1920x1080, use at most three concurrent Linux jobs without short-circuiting failure collection, discover and execute the exact native test in an optimized profile with an explicit bounded PR scope, require step-local tools/font/resolution inputs, and fail if selection matches zero tests. Policy validation MUST also require a separate, enabled weekly and manually dispatchable default-branch full 1920x1080 workflow with its exact optimized test discovery and execution, explicit full scope, reviewed tools/font, bounded timeout, default failure propagation, and no golden mutation; a missing or weakened definition MUST fail protected policy validation. The weekly result MUST remain visible as its own Actions status and MUST NOT be represented as a passing PR result before it runs. Foundation parity MUST directly require and check all six existing PR prerequisite results and the OpenSpec attestation. The original Render parity job MUST retain native cache evidence and strict report validation before its unchanged publication. Before repository-controlled Bun or Moon configuration is interpreted, the OpenSpec job MUST install explicit reviewed bootstrap versions and invoke Bun with an empty trusted configuration and automatic dotenv loading disabled. The bootstrap MUST validate every protected workflow, Bun, Moon, and proto source and refuse to launch Moon on any invalid input. The bootstrap MUST invoke only the explicitly qualified root Moon task without a shell, MUST withhold the GitHub output channel from the Moon child, and MUST emit the exact completion attestation only after the child exits successfully. The root project MUST reject inherited tasks and project-wide execution overrides; its canonical Bun configuration, workspace mapping, pinned toolchain, and `.prototools` versions MUST remain stable. Every protected Bun command in the Moon task MUST explicitly use the validated Bun configuration with automatic dotenv loading disabled, and the task MUST execute the real-Moon startup-hook regression on its supported CI platform. Global task configurations MAY serve other projects but MUST NOT inject global environment, implicit execution settings, or external extensions. Environment maps on unrelated GitHub jobs, ignored local dotenv files outside protected execution, and project-local Moon configuration outside the root project MUST remain permitted.

#### Scenario: Accept all three required resolution shards
- **WHEN** the PR workflow lists exactly 960x540, 1280x720, and 1920x1080 with at most three parallel Linux jobs, explicit bounded scope, the approved tool/font environment, exact test-discovery and test-execution commands, default failure propagation, and a six-result foundation assertion plus an unconditional final duration assertion
- **THEN** policy validation accepts the distributed rules-screen evidence while preserving the original report and all existing gates

#### Scenario: Reject missing, duplicated, or zero-test evidence
- **WHEN** a resolution is missing or duplicated, the matrix or test selector changes, the exact test cannot be discovered, a shard becomes optional, or its command/environment is neutralized
- **THEN** policy validation or the shard fails before the foundation gate can succeed

#### Scenario: Accept reviewed weekly full evidence
- **WHEN** the weekly full-resolution workflow has the approved schedule and manual trigger, exact 1920x1080 full test selection, reviewed native tools and font, bounded timeout, and default failure propagation
- **THEN** structural policy validation accepts the workflow while leaving its runtime result outside the PR aggregate

#### Scenario: Reject a weakened scheduled workflow
- **WHEN** the weekly workflow is missing, disabled by its definition, selects zero tests, changes cadence or scope, omits a native dependency, masks a failure, or changes its bounded timeout
- **THEN** protected policy validation rejects the change before an incomplete scheduled result can be represented as full evidence

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

### Requirement: Protected mandatory raster-cache CI evidence
Repository policy MUST enforce the reviewed render-parity sequence including locked bridge dependency installation, configured native core raster-cache conformance, instrumented native worker and bridge reuse checks, and default headless restoration before default transport verification. The policy MUST validate exact commands, feature selection, working directories and step-local required-mode environment. Existing isolated execution, immutable goldens, failure propagation, report validation/upload ordering and foundation aggregation MUST remain enforced.

#### Scenario: P1 Accept complete cache verification
- **WHEN** the reviewed workflow installs locked bridge dependencies, executes all three configured cache suites, restores the default build and verifies default transport before validating and uploading the existing report
- **THEN** policy validation accepts the expanded sequence and preserves every existing required gate

#### Scenario: P2 Reject removed or disabled native evidence
- **WHEN** any new native command, feature selection, required-mode flag, dependency setup or declared working directory is removed, substituted or weakened
- **THEN** policy validation fails before the altered workflow can be accepted

#### Scenario: P3 Reject instrumented compatibility checks or masked failures
- **WHEN** default restoration is removed or reordered after compatibility checks, or a new critical step is conditional, ignores errors or changes its approved command/environment
- **THEN** policy validation rejects the workflow without relaxing the existing protected sequence or golden/report guarantees

### Requirement: Protected optimized golden command
Repository policy MUST pin the exact optimized native golden command for non-rules-screen suites inside the existing required Render parity step, exact optimized native test discovery and execution commands inside every PR rules-screen matrix job, and exact optimized full 1920x1080 test discovery and execution inside the weekly workflow. All other approved Render parity commands, environments, cache and report ordering, failure propagation, and publication guarantees MUST remain unchanged. A test selector that matches zero tests MUST fail rather than silently count as a successful shard or scheduled run.

#### Scenario: Accept reviewed split optimized commands
- **WHEN** both render jobs contain their exact approved optimized commands and protected environments
- **THEN** CI policy validation accepts the workflow without weakening the required render or foundation gates

#### Scenario: Reject altered or empty render selection
- **WHEN** a native command is removed, made optional, selects zero tests, changes profile or target, or moves outside its protected step
- **THEN** CI policy validation or explicit discovery fails before the protected foundation gate can succeed

### Requirement: Bounded fake speech-provider worker tests
The repository's bridge unit suite MUST exercise the fake speech provider through its real child-process boundary on Windows, macOS, and Ubuntu. Every test in that real-child fixture, including status, voices, generation, readiness, cancellation, timeout, queueing, and failure cleanup, MUST have a bounded runner deadline longer than the provider's configured control-request deadline, without changing the provider deadline or weakening any semantic assertion. The correctness job MUST fail when those tests report a wrong result, an unexpected provider timeout, or an unhandled runner deadline.

#### Scenario: Worker starts within its provider deadline on a loaded runner
- **WHEN** a supported correctness runner exercises any real-child fake-provider operation within the provider's own deadlines
- **THEN** the assertions complete without the test runner timing out first, and the test verifies the same provider metadata, generated artifact, path readiness, cancellation, timeout, queueing, and cleanup behavior as before

#### Scenario: Unexpected worker stall or incorrect result
- **WHEN** an ordinary status, voice, or generation request unexpectedly exceeds its provider deadline, or its returned result violates an existing assertion
- **THEN** the test fails within its bounded runner deadline and the correctness job fails without ignoring or retrying the failure

#### Scenario: Cancellation of a fake hanging generation under delayed scheduling
- **WHEN** a cancellation or synthesis-timeout test submits the fake worker's hanging generation and scheduling delays the parent callback
- **THEN** the fake generation remains pending until the provider terminates it, and the test observes the expected retryable cancellation or timeout with owned cleanup instead of a successful generation

### Requirement: Bounded rules-screen CI execution

The required PR rules-screen parity matrix job MUST give each resolution shard an explicit 135-minute timeout, consistent with other required leaf jobs. The independent weekly full 1920x1080 job MUST have an explicit 240-minute timeout, preserving margin above the observed roughly 192-minute full run and staying below the hosted-runner limit. Repository policy validation MUST accept only those exact values and MUST reject missing, shorter, longer, or nonnumeric values while preserving required PR shard and scheduled result checks.

#### Scenario: Accept the approved render budget

- **WHEN** the three required PR resolution shards inherit the matrix job's 135-minute timeout, the weekly full-resolution job declares 240 minutes, and all other workflow properties match the reviewed policy
- **THEN** structural CI policy validation accepts both workflows and all jobs retain required test execution and failure propagation

#### Scenario: Reject an altered render budget

- **WHEN** the PR rules-screen matrix job omits its timeout or declares a value other than numeric 135 minutes, or the weekly full-resolution job omits its timeout or declares a value other than numeric 240 minutes
- **THEN** structural CI policy validation rejects the workflow before the protected policy task can attest success

### Requirement: Mandatory native track-matte CI conformance
The existing render-parity native conformance step MUST execute the actual native core track-matte suite and dedicated MCP artifact witness on the exact CI head, inheriting its unchanged six-key required dependency/font/report environment. Immediately after the unchanged font-resolution command it MUST run exactly `cargo test -p opencut-editor-core --test track_mattes_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/track-matte-native.test.ts` in that order. The default headless build MUST precede MCP execution. All existing commands, environment values, steps, dependencies, timeouts, guards, report paths, benchmark, raster-cache default restoration, aggregate and duration budget MUST remain unchanged. Ordinary opt-in skips SHALL NOT substitute for actual mandatory execution. The exact command-body validator and additive regressions MUST reject omission, alteration, success fallback and an instrumented headless build; no model, native oracle or public contract change is authorized.

#### Scenario: Execute actual native core and MCP conformance on the CI head
- **WHEN** the required render-parity leaf runs on the exact PR commit with its configured tools/font and required flags
- **THEN** the complete native core matte suite executes, the default headless binary builds and the dedicated MCP matte artifact witness executes with no native opt-in skip before the unchanged raster-cache step

#### Scenario: Reject weakened mandatory matte execution
- **WHEN** any new command is omitted, altered, success-masked or replaced with an instrumented headless build
- **THEN** exact policy validation fails while every prior protected command, environment, sequence, timeout, report and aggregate constraint remains enforced

### Requirement: Portable bounded track-matte test evidence
The font symlink/cycle witness MUST retain positive real file/directory symlink and legacy-first-match controls and MUST exercise repeated real cycle entries before OutOfMemory using a budget derived from existing platform cursor/entry-scratch accounting. Production lookup policy, resource counters and fallback behavior MUST remain unchanged. Windows MCP catalog tests MUST preserve their unchanged5000ms deadlines, all58missing plus58malformed field controls, complete five pinned digests,78tools, two fresh expansions/full serialized bytes and all existing malformed/cyclic/nonlocal/sibling/unused/isolation/current-registration controls. Successful matte projection MUST validate the full exact approved additions before cloning/removing only named fields; rejected projection MUST never mutate source. Negative cases MAY reuse one isolated source only with per-case restoration and complete before/after byte equality. Already-expanded JSON reference copying MAY use a complete fresh recursive clone preserving all scalars, key/array order and isolation; no memo-shared output, deadline extension, split proof, suppression or weakened comparison is permitted.

#### Scenario: Bound actual directory cycles on each platform
- **WHEN** real file/directory symlinks resolve and an isolated directory symlink cycle traverses under the platform-derived admitted budget
- **THEN** positive matches retain canonical legacy paths and at least two actual cycle entries precede OutOfMemory, with all platform cursor/ownership controls intact

#### Scenario: Preserve complete catalog proof within existing deadlines
- **WHEN** the current and all historical MCP catalogs, every missing/malformed matte path and all existing malformed/unrelated drift controls run on supported platforms
- **THEN** every unchanged complete assertion passes within5000ms, successful/failed projection and repeated expanded subtrees remain isolated, and source bytes are unchanged

### Requirement: Mandatory native blend-mode CI conformance
The existing render-parity native conformance step MUST additionally execute the actual native core blend-mode suite `crates/editor-core/tests/blend_modes_native.rs` and dedicated MCP artifact witness `apps/agent-bridge/tests/blend-mode-native.test.ts` on the exact future blend-mode CI head. Immediately after the unchanged three track-matte commands it MUST run exactly `cargo test -p opencut-editor-core --test blend_modes_native -- --nocapture`, `cargo build -p opencut-headless`, and `bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/blend-mode-native.test.ts` in that order, before the unchanged raster-cache step. The default headless build MUST precede the blend MCP witness. It MUST inherit the existing unchanged six-key required dependency/font/report environment (`OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH`, `OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED`, `OPENCUT_GOLDEN_REPORT_PATH`, `OPENCUT_GOLDEN_REQUIRED`, `OPENCUT_TEST_FONT_PATH`), including the existing required values; no new opt-in flag or environment entry is authorized. Every preceding native command, track-matte command, step, dependency, timeout, guard, report path, benchmark, raster-cache instrumentation/default restoration, aggregate constraint and duration budget MUST remain unchanged. Ordinary unit-suite opt-in skips MUST NOT substitute for actual mandatory execution. Exact command-body policy validation and additive regressions MUST reject omission, alteration or success fallback for each new command and replacement of the new default headless build with an instrumented build, while preserving all existing negative controls. This requirement specifies future implementation intent only; these named suites and commands are not claimed to exist or have executed during external proposal preparation.

#### Scenario: Execute actual native core and public blend conformance
- **WHEN** the required render-parity leaf runs on the exact approved blend-mode PR commit with its existing required tools/font/report environment
- **THEN** the complete native blend suite executes, the default headless binary builds and the dedicated public blend artifact witness executes without a native opt-in skip after all unchanged track-matte commands and before the unchanged raster-cache step

#### Scenario: Reject weakened mandatory blend execution
- **WHEN** any added blend command is omitted, altered, success-masked or its default headless build is replaced with an instrumented build
- **THEN** exact policy validation fails and all prior protected commands, six environment entries, step ordering, timeouts, report, duration and aggregate constraints remain enforced
