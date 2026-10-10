## Why

Exact-head Windows correctness at d64e539e failed the existing target_measurement/deadline PID-readiness assertion before observing a backend PID, after earlier unchanged failures in other modes. The cmd-to-Bun-per-phase fixture introduces repeated interpreter startup into bounded lifecycle checks, and retained logs lack phase/outcome diagnostics; this does not establish a production cancellation leak.

## What Changes

- Compile a private std-only native normalization lifetime backend once during fixture setup; invoke the backend directly and spawn a real native descendant of the same binary.
- Preserve all nine phases, 54 scenarios, fault metrics/PCM, state/output/resource assertions and existing 1500ms readiness, 2000ms operation deadline and 5000ms cold-setup bounds. Add live-PID checks before cancellation and phase/outcome diagnostics on readiness failure.
- Fail fixture compilation/setup closed, without optional skips or a script fallback. Compilation is separately bounded setup, not part of the controlled operation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `master-loudness-normalization`: strengthen normalization lifecycle verification through a direct native fixture and retained failure diagnostics.

## Impact

Private bridge test fixtures and their setup only. Existing Rust compiler is required for mandatory fixture setup; no package dependency, production backend, renderer, public operation, capability, provider, schema44/protocol1, migration or protected workflow change.

## Non-goals

No larger test deadlines, assertion removal, automatic retries, forged backend identity, renderer rewrite, audit-branch edits, Moon output repair, fixture deduplication, #80 implementation or rescue-video expansion. Direct native fixture performance is test-harness evidence, not real FFmpeg/media correctness. Exact Windows success and all required CI remain mandatory before advancing.

## Approval

Approved within the user's standing authorization for issue-scoped specification approval and recoverable CI repair, preserving acceptance. Approval precedes executable edits.
