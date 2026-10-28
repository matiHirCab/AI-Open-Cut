## Why

At PR229 head fc48d575d2f813e02718feb4c8b82855ae810a77, CI job122835122356 passed45source tests but failed the new combined case: ambient FFprobe was absent in the fake-provider job. Packaged CI was skipped; local installed tools hid the mistake.

## What Changes

- Keep mandatory combined actual-core source/package acceptance using configured fake backends, with a bounded owned PNG/WAV probe fixture.
- Retain explicit genuine mode and every independent FFmpeg decoding assertion; no availability-based selection or skip.
- Preserve common state/source/history/revision/reconnect/no-replay assertions and distinguish evidence by mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `durable-production-requests`: hermetic combined acceptance with retained genuine proof.

## Impact

Private test helper, fixture, smoke call sites and regressions only; no breaking/public/runtime/schema/dependency/workflow/profile/timeout change.

## Non-goals

No PR233 or disposal/deletion/junction/crash/descendant work; no CI installs, relaxed checks, merge/deploy, force push, PR approval/readiness, issue closure, description update or comments. Preserve archives, pins and failed CI evidence.

## Approval

Pending exact-artifact reviewer approval before executable edits.
