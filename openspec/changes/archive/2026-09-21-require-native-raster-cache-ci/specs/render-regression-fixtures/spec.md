## MODIFIED Requirements

### Requirement: Independently visible render-parity gate
Continuous integration MUST publish a dedicated required Linux render-parity status that configures explicit FFmpeg, FFprobe, deterministic font, required-gate, and absolute report-path dependencies; executes production preview, audiovisual range, export, and lifecycle conformance with fail-closed critical steps against the selected immutable golden generation; strictly validates the report captured at the declared absolute workspace path; and only then uploads that exact validated observation. The gate MUST contain only its exact reviewed checkout, deterministic rendering dependency installation, pinned toolchain, locked bridge dependency installation, existing native conformance, native raster-cache conformance, report-validation, and upload steps in that order. Workflow-level and render-job-level environment maps MUST be absent; required CI MUST use only exact approved step environments, reject inherited execution defaults and job containers, and reject `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` from every effective configuration path so reviewed references remain immutable. Critical conformance, validation, and publication steps MUST NOT ignore failures or contain incompatible command alterations.

The native raster-cache step MUST execute core cold/warm/fresh audiovisual cache conformance, instrumented native worker reuse, and actual bridge-to-native avoided-raster-work checks with explicit FFmpeg, FFprobe and deterministic font configuration and both required-mode flags enabled. It MUST rebuild the default headless binary after instrumented tests and verify default transport compatibility before report validation and upload. Existing native suites, public contracts, golden references and tolerances MUST remain unchanged.

#### Scenario: Accept reviewed deterministic output
- **WHEN** production preview, audiovisual range preview, final export, and edit/undo/redo/reopen lifecycle behavior match the reviewed fixture under the exact declared Linux sequence and step-scoped environment
- **THEN** the dedicated render-parity status succeeds and publishes the strictly validated report-only observation

#### Scenario: Reject coordinated output drift
- **WHEN** preview, range, and export drift together from a reviewed frame, decoded audio reference, timing value, semantic plan, or normalized filter graph
- **THEN** the dedicated render-parity status fails even when the newly rendered outputs agree with one another

#### Scenario: Reject a weakened deterministic environment
- **WHEN** inherited workflow or render-job environment is declared, or a required executable, filter, font identity, selected generation, reference, report field, report destination, approved step environment key, step property, or execution setting is missing, additional, invalid, or inconsistent
- **THEN** repository policy validation fails before the dedicated render-parity status can accept or publish the observation

#### Scenario: Reject an environment-persisting step
- **WHEN** an added or replaced step attempts to alter later steps through `GITHUB_ENV` or another unreviewed command
- **THEN** repository policy validation fails because the render leaf sequence is not exact

#### Scenario: Reject inherited golden mutation mode
- **WHEN** golden update or alternate-capture mode is declared at workflow, render-job, native-step, validation-step, or job-container scope
- **THEN** repository policy validation fails before required CI can replace or bypass comparison with the reviewed references

#### Scenario: Reject any inherited process control
- **WHEN** workflow or render-job `env` is present with an empty map, a literal value, or an expression value
- **THEN** repository policy validation fails before inherited configuration can change process startup or command resolution

#### Scenario: Reject neutralized or reordered render evidence
- **WHEN** a render step is added, duplicated, replaced, or reordered; a critical step uses a custom shell or ignores failure; execution defaults wrap the command; or report publication moves before strict validation
- **THEN** repository policy validation fails before the weakened render gate can be accepted

#### Scenario: Preserve renderer semantics and report-only budgets
- **WHEN** the dedicated gate's closed verification-only sequence and isolated environment are enforced
- **THEN** golden references, render semantics, conformance tolerances, local deliberate update workflows, and application output remain unchanged and timing or memory observations do not become universal pass/fail budgets

#### Scenario: C1 Require native cache evidence
- **WHEN** the required Render parity job executes
- **THEN** core cache conformance, feature-enabled worker reuse and actual bridge reuse assertions execute with required dependencies, and any failure fails the leaf and existing foundation aggregate

#### Scenario: C2 Fail closed on absent native prerequisites
- **WHEN** required tools, font configuration or instrumented cache diagnostics are unavailable
- **THEN** the dedicated native cache run fails rather than silently returning or marking its tests skipped

#### Scenario: C3 Restore default compatibility evidence
- **WHEN** instrumented bridge/native cache verification succeeds
- **THEN** CI rebuilds headless without test hooks and executes default transport tests before publishing the validated report
