# Repair shutdown fixture portability

## Why
Exact-head Windows CI failed before the shutdown signal during fixture readiness. The fixture uses a .cmd wrapper as a directly spawned headless executable, while the production adapter intentionally spawns without a shell. Existing diagnostics discard bridge stderr and omit the failed job error. This evidence supports fixing the fixture launch boundary; it does not prove a shutdown regression.

## What Changes
Compile the existing tiny headless fixture to a host-native Bun executable. Preserve production launch semantics and every readiness, worker/bridge cleanup and deadline assertion. Include job status and bounded bridge stderr in assertion diagnostics.

## Capabilities
- Modified: known-text-alignment (portable shutdown conformance evidence).

## Impact
Only the shutdown test and specification/report artifacts; no production semantics, schemas, dependencies, workflow commands or thresholds change. Explicit parent remediation instruction and standing issue-scoped specification authority approve this bounded repair before implementation. Windows execution remains remote evidence to obtain, not a claimed local pass.

Windows Node process.kill(SIGTERM) unconditionally terminates its target (https://nodejs.org/api/process.html), so it cannot exercise the bridge handler. Preserve the POSIX SIGTERM regression and exercise the existing stdin-end shutdown entry on all platforms, including Linux, with the identical active-inference, 3s cleanup and 10s inference assertions. Own the child streams in the test and use the SDK public generic JSON-RPC stream transport, without private SDK access or product changes.
