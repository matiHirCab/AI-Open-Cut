## MODIFIED Requirements

### Requirement: Attributable Windows renderer startup evidence
The Windows renderer-descendant regression fixture MUST preserve its10-second PID observation deadline, exact owned-process handle/live-process check and5-second post-crash cleanup assertion. Startup failure MUST remain failure. The standard fixture SHALL launch a test-owned native executable through the batch wrapper, publishing its own newline-terminated PID before blocking; compilation SHALL finish before the worker observation budget starts and failures SHALL fail closed. Failure reporting SHALL include bounded fixture shell-entry evidence, owned renderer stderr (PowerShell stderr for retained controls), captured correlated worker events and worker diagnostics so typed renderer failures are distinguishable from absent fixture startup. Missing/unreadable evidence MUST be reported explicitly. Evidence sources MUST remain under the owned temporary root, with16KiB per-source and32-event reporting bounds and explicit truncation labels.

#### Scenario: Observe a typed renderer failure before PID publication
- **WHEN** the worker emits a correlated typed failure without a readable renderer PID record
- **THEN** the scenario still fails within the existing startup observation budget and reports that event with shell/PowerShell/worker evidence without claiming successful descendant cleanup

#### Scenario: Observe missing or unreadable startup artifacts
- **WHEN** fixture files are missing, unreadable or exceed evidence bounds
- **THEN** diagnostics identify each condition and explicit truncation without panicking in the diagnostic collector or extending the observation deadline

#### Scenario: Preserve exact cleanup proof
- **WHEN** the fixture publishes its live renderer PID
- **THEN** the test opens that exact process handle, confirms liveness, abruptly terminates the worker and still requires the renderer to terminate within5000ms

#### Scenario: Compare original and instrumented startup without unrelated process disclosure
- **WHEN** focused evidence passes without establishing the original failure's cause
- **THEN** the diagnostic job executes the native fixture followed by the exact original and instrumented PowerShell fixture bodies sequentially with unchanged deadlines/assertions and reports startup elapsed time and at most64 processes reachable through the owned worker's parent-child tree using read-only Windows snapshots
- **AND** unrelated process identities are omitted, snapshot failures are explicit and this serialized comparison is distinguished from unchanged full-workspace acceptance

#### Scenario: Bound malformed-byte evidence expansion
- **WHEN** a file contains invalid UTF-8 bytes whose replacement characters expand rendered evidence
- **THEN** the rendered source remains bounded16KiB at a valid UTF-8 boundary with an explicit truncation label

#### Scenario: Preserve native standard acceptance and original diagnostic controls
- **WHEN** the required full-workspace suite executes without the focused diagnostic flag
- **THEN** only the native descendant batch body exercises startup and cleanup, with unchanged deadlines/assertions and correlated worker/process reporting
- **AND** both original and instrumented PowerShell exercises run sequentially only under `OPENCUT_WINDOWS_STARTUP_COMPARISON=1` in the separate focused workflow, preventing concurrent diagnostic warm-up from substituting for standard acceptance

#### Scenario: Distinguish absent from incomplete owned PID publication
- **WHEN** PID observation fails because the owned record is absent, unreadable, incomplete or contains control bytes
- **THEN** failure evidence includes a bounded escaped rendering of that record, preserving incomplete terminators and NUL bytes visibly without accepting an invalid PID or extending the deadline

#### Scenario: Validate owned native publication independently
- **WHEN** the native fixture is launched directly with an owned PID destination or malformed arguments
- **THEN** it publishes its own live process identifier with a complete newline, remains alive for the cleanup assertion, and rejects malformed arguments without publishing a PID

