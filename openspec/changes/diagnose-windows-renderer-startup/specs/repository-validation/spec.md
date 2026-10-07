## ADDED Requirements

### Requirement: Attributable Windows renderer startup evidence
The Windows renderer-descendant regression fixture MUST preserve its10-second PID observation deadline, exact owned-process handle/live-process check and5-second post-crash cleanup assertion. Startup failure MUST remain failure. Failure reporting SHALL include bounded fixture shell-entry evidence, owned PowerShell stderr, captured correlated worker events and worker diagnostics so typed renderer failures are distinguishable from absent fixture startup. Missing/unreadable evidence MUST be reported explicitly. Evidence sources MUST remain under the owned temporary root, with16KiB per-source and32-event reporting bounds and explicit truncation labels.

#### Scenario: Observe a typed renderer failure before PID publication
- **WHEN** the worker emits a correlated typed failure without a readable renderer PID record
- **THEN** the scenario still fails within the existing startup observation budget and reports that event with shell/PowerShell/worker evidence without claiming successful descendant cleanup

#### Scenario: Observe missing or unreadable startup artifacts
- **WHEN** fixture files are missing, unreadable or exceed evidence bounds
- **THEN** diagnostics identify each condition and explicit truncation without panicking in the diagnostic collector or extending the observation deadline

#### Scenario: Preserve exact cleanup proof
- **WHEN** the fixture publishes its live renderer PID
- **THEN** the test opens that exact process handle, confirms liveness, abruptly terminates the worker and still requires the renderer to terminate within5000ms

### Requirement: Focused Windows evidence alongside unchanged required gates
A focused Windows startup workflow SHALL use the repository-pinned toolchain and execute the exact renderer-descendant scenario with visible diagnostics. Existing required correctness, contract, render, policy, smoke and foundation workflows MUST remain unchanged and authoritative. The focused job MUST NOT replace full Windows acceptance, skip assertions or reclassify failures. Linux-only checks SHALL be reported as local evidence and MUST NOT be called Windows runtime proof.

#### Scenario: Collect real Windows startup evidence
- **WHEN** the isolated diagnostic branch is exercised on Windows
- **THEN** the exact fixture executes under the pinned native Windows toolchain, reports its startup/process evidence, and the unchanged full required gate remains separately enforced
