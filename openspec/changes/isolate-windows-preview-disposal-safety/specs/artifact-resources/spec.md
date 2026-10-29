## ADDED Requirements

### Requirement: Attributable preview disposal safety acceptance

Preview-disposal acceptance MUST preserve the existing outside-file sentinel assertions, twelve native disposal invocations, idempotent missing-output checks, foreign-path/reparse refusal, and cleanup-debt close/retry checks. It SHALL distinguish fixture junction setup/removal from native deletion using isolated controls and phase-specific preservation assertions. Sentinel publication MUST precede the race. Native platform evidence MUST identify the test runtime and source/package executable, and distinguish actual replacement from expected Windows sharing refusal. Linux results MUST NOT be reported as proof of Windows junction safety.

#### Scenario: Fixture-only junction control
- **WHEN** a synthetic junction to an outside sentinel directory is created and removed without invoking native disposal
- **THEN** the sentinel and target directory remain unchanged, and a failure is attributable to fixture/runtime operations rather than native disposal

#### Scenario: Ordinary native deletion control
- **WHEN** actual native disposal is invoked on an owned preview without ancestor replacement
- **THEN** only the owned preview is removed, the outside sentinel remains unchanged and missing-output disposal remains idempotent

#### Scenario: Actual Windows source and package replacement
- **WHEN** source and packaged native disposal run during the existing twelve-call ancestor-replacement race
- **THEN** all outside sentinel checks remain mandatory, the first failing phase is observable, and evidence records replacement/refusal counts and runtime identity without claiming a zero-replacement run exercised redirection

#### Scenario: Evidence-gated fixture correction
- **WHEN** Windows controls establish that fixture junction teardown deletes target contents
- **THEN** the corrected fixture removes only the junction without traversing the target, preserves the sentinel through restoration and cleanup-debt retry, and retains the actual native safety assertion without retries of that assertion
