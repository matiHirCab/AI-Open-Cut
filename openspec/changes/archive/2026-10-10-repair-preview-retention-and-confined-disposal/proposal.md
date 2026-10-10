## Why

The independent epic #9 review reproduced two violations of the existing artifact retention contract on main `64335ba1`: persistent disposal failures admit further preview outputs beyond the count/byte budgets, and replacing a checked preview ancestor redirects path-based deletion outside the project. Repair these failure boundaries without changing rendered media or public editor semantics.

## What Changes

- Reserve bounded preview production capacity before starting producers; retain failed-disposal debt and refuse further preview admission until debt is settled. Preserve exact 32-artifact/64MiB inclusive limits and safe retryable `JOB_REGISTRY_FULL` behavior.
- Delegate deletion to a confined native owner that anchors directory/file handles throughout deletion on Linux, macOS and Windows. Reject links/reparse points and unsafe paths; handle ancestor replacement without unlinking unrelated files.
- Make unsuccessful graceful cleanup observable with safe existing error semantics, preserving retry capability and ownership of remaining artifacts.
- Convert the original two reproductions into regression tests and cover persistent failure/retry/close, overlapping producers, ancestor/file replacement and native source/default-package behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `artifact-resources`: clarify producer reservations, failed-disposal debt and retryable cleanup; require race-safe confined deletion through the full operation.
- `platform-renderer-packaging`: preserve the existing four-role runtime layout while requiring genuine native cleanup proof on all supported platforms.

## Impact

Affected layers: process-local bridge job admission/retention, native artifact I/O in editor-core, a private headless cleanup adapter, and focused tests/docs. Native syscall support is confined to owning filesystem code; transports do not acquire project/timeline semantics.

Compatibility: no public MCP/headless operation, catalog, capability, persisted schema, renderer output, provider contract or migration changes are planned. The private cleanup adapter is not a public protocol operation and ships inside the existing headless runtime role. Existing stable safe errors and process-local ownership remain authoritative. Any implementation discovery requiring a public/persisted change must be reflected in approved artifacts before implementation.

## Non-goals

No rendering/cache redesign, durable jobs, project/media/history deletion, cleanup of predecessor-process files, new packaging role, broad refactor, merge or deployment. Existing release limits, assertions, goldens and CI gates remain unchanged.
