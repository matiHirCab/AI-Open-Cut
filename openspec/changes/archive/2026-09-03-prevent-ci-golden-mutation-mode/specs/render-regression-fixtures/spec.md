## MODIFIED Requirements

### Requirement: Independently visible render-parity gate
Continuous integration MUST publish a dedicated required Linux render-parity status that configures explicit FFmpeg, FFprobe, deterministic font, required-gate, and absolute report-path dependencies; executes production preview, audiovisual range, export, and lifecycle conformance with fail-closed critical steps against the selected immutable golden generation; strictly validates the report captured at the declared absolute workspace path; and only then uploads that exact validated observation. Required CI MUST use exact approved environments for native conformance and report validation, and MUST reject `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` from workflow, job, or critical-step scopes so reviewed references remain immutable. Critical conformance, validation, and publication steps MUST NOT ignore failures or contain incompatible command alterations.

#### Scenario: Accept reviewed deterministic output
- **WHEN** production preview, audiovisual range preview, final export, and edit/undo/redo/reopen lifecycle behavior match the reviewed fixture under the declared Linux environment
- **THEN** the dedicated render-parity status succeeds and publishes the strictly validated report-only observation

#### Scenario: Reject coordinated output drift
- **WHEN** preview, range, and export drift together from a reviewed frame, decoded audio reference, timing value, semantic plan, or normalized filter graph
- **THEN** the dedicated render-parity status fails even when the newly rendered outputs agree with one another

#### Scenario: Reject a weakened deterministic environment
- **WHEN** a required executable, filter, font identity, selected generation, reference, report field, report destination, or approved environment key is missing, additional, invalid, or inconsistent
- **THEN** the dedicated render-parity status fails before accepting or publishing the observation

#### Scenario: Reject inherited golden mutation mode
- **WHEN** golden update or alternate-capture mode is declared at workflow, render-job, native-step, or validation-step scope
- **THEN** repository policy validation fails before required CI can replace or bypass comparison with the reviewed references

#### Scenario: Reject neutralized or reordered render evidence
- **WHEN** a critical render step ignores failure, its authoritative command changes incompatibly, or report publication moves before strict validation
- **THEN** repository policy validation fails before the weakened render gate can be accepted

#### Scenario: Preserve renderer semantics and report-only budgets
- **WHEN** the dedicated gate's verification-only environment is enforced
- **THEN** golden references, render semantics, conformance tolerances, local deliberate update workflows, and application output remain unchanged and timing or memory observations do not become universal pass/fail budgets
