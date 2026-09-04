## MODIFIED Requirements

### Requirement: Independently visible render-parity gate
Continuous integration MUST publish a dedicated required Linux render-parity status that configures explicit FFmpeg, FFprobe, deterministic font, required-gate, and absolute report-path dependencies; executes production preview, audiovisual range, export, and lifecycle conformance with fail-closed critical steps against the selected immutable golden generation; strictly validates the report captured at the declared absolute workspace path; and only then uploads that exact validated observation. The gate MUST contain only its exact reviewed checkout, dependency, toolchain, native-conformance, report-validation, and upload steps in that order. Required CI MUST use exact approved step properties and environments, reject inherited execution defaults, inherited Bash startup hooks, and job containers, and reject `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` from every effective configuration path so reviewed references remain immutable. Critical conformance, validation, and publication steps MUST NOT ignore failures or contain incompatible command alterations.

#### Scenario: Accept reviewed deterministic output
- **WHEN** production preview, audiovisual range preview, final export, and edit/undo/redo/reopen lifecycle behavior match the reviewed fixture under the exact declared Linux sequence and environment
- **THEN** the dedicated render-parity status succeeds and publishes the strictly validated report-only observation

#### Scenario: Reject coordinated output drift
- **WHEN** preview, range, and export drift together from a reviewed frame, decoded audio reference, timing value, semantic plan, or normalized filter graph
- **THEN** the dedicated render-parity status fails even when the newly rendered outputs agree with one another

#### Scenario: Reject a weakened deterministic environment
- **WHEN** a required executable, filter, font identity, selected generation, reference, report field, report destination, approved environment key, step property, or execution setting is missing, additional, invalid, or inconsistent
- **THEN** the dedicated render-parity status fails before accepting or publishing the observation

#### Scenario: Reject an environment-persisting step
- **WHEN** an added or replaced step attempts to activate update or alternate-capture mode for later steps through `GITHUB_ENV` or another unreviewed command
- **THEN** repository policy validation fails because the render leaf sequence is not exact

#### Scenario: Reject inherited golden mutation mode
- **WHEN** golden update or alternate-capture mode is declared at workflow, render-job, native-step, validation-step, or job-container scope
- **THEN** repository policy validation fails before required CI can replace or bypass comparison with the reviewed references

#### Scenario: Reject inherited Bash startup injection
- **WHEN** `BASH_ENV` is declared at workflow or render-job scope with any literal or expression value
- **THEN** repository policy validation fails before startup code can replace or bypass the reviewed render commands

#### Scenario: Reject neutralized or reordered render evidence
- **WHEN** a render step is added, duplicated, replaced, or reordered; a critical step uses a custom shell or ignores failure; execution defaults wrap the command; or report publication moves before strict validation
- **THEN** repository policy validation fails before the weakened render gate can be accepted

#### Scenario: Preserve renderer semantics and report-only budgets
- **WHEN** the dedicated gate's closed verification-only sequence is enforced
- **THEN** golden references, render semantics, conformance tolerances, local deliberate update workflows, and application output remain unchanged and timing or memory observations do not become universal pass/fail budgets
