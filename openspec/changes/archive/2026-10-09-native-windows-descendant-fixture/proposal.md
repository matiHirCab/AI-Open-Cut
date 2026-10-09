## Why
PR #170 head26333329 fails unchanged full Windows acceptance before PID publication: owned cmd.exe and powershell.exe exist, but no PID record appears within10 seconds. Focused original/instrumented comparison passes. This establishes unreliable interpreter-based publication, not a proven production containment defect or a precise PowerShell startup cause.

## What Changes
Use a test-owned native Rust renderer descendant launched through the existing batch wrapper for standard crash acceptance. Compile before starting the worker, publish its own PID with newline, and remain alive until worker containment terminates it. Keep10-second observation, exact owned live-process handle, abrupt worker crash and5000ms cleanup assertion unchanged. Keep both exact original/instrumented PowerShell bodies as sequential focused comparison controls with failures remaining failures. Preserve all required gates and production code.

## Capabilities
### Modified Capabilities
- `repository-validation`: native deterministic descendant fixture and retained interpreter comparison.

## Impact
Headless test fixture, portable helper protocol checks and specification/evidence. No production, protocol, renderer, schema or workflow gate change.

## Approval
The delegated user explicitly requests fixing #170's failure, preserving assertions and updating the same PR. This bounded specification is approved under that authority before implementation; actual Windows exact-head CI remains required and pending.
