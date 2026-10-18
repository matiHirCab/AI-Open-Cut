## Why

#226's first exact-head CI caught a new #213 regression: macOS and Windows strict Clippy reject the unread `Pause(bool)` payload when the Linux backend is absent. This is separate from historical Windows test timeouts. Local Linux implementation/runtime checks had passed; remote completion remains blocked.

## What Changes

- Preserve every typed command payload explicitly in common controller admission, including play/pause on builds without a native backend.
- Extend the existing controller scenario to verify both pause and loop directions without changing behavior, bounds or platform support.
- Retain strict warnings and verify the corrected head on required platform CI.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `desktop-review-surfaces`: portable controller compilation and unchanged typed play/pause semantics.

## Impact and approval

Only the private playback controller, its existing tests and issue-scoped specification artifacts. No backend ABI, GUI callbacks, rendering, public/persisted contracts, dependencies, #208/narration, security or deployment changes. Approved under delegated maintainer authority before follow-up executable edits.
