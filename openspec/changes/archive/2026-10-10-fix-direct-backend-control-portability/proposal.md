## Why

The required Windows setup control tries to create a newline filename, which Windows rejects before the verifier. Keep the unsafe-path negative control mandatory on every platform.

## What Changes

Reject unsafe installation and discovered directory strings before filesystem resolution; preserve all existing direct native/unique/contained checks. Use an injected unsafe candidate and unsafe absolute installation path in hermetic controls, without creating unsupported filenames, skipping tests or changing native acceptance.

## Capabilities

### Modified Capabilities
- `motion-release-gates`: Retain platform-independent fail-closed setup controls.

## Impact

Only private setup helper/test and archived evidence. Standing issue-scoped delegation approves this bounded correction before edits. No production behavior, public contract, workflow, pin, cache assertion or media change.
