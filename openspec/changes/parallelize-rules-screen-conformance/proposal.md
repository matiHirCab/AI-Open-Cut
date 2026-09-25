## Why

PR #124's optimized Render parity job still took 5h 36m on its latest completed run. The unchanged `rules_screen` suite consumed 5h 20m of that time; its three independent resolutions currently run serially, and the job log cannot show which render intent and lifecycle state dominates within them.

## What Changes

- Emit elapsed time for each resolution, lifecycle state, and production preview/range/export operation in the rules-screen native conformance suite.
- Execute independent resolution fixtures with at most two bounded workers while preserving each resolution's five ordered lifecycle states and every existing render, semantic, reference, audio, timing, and no-mutation assertion.
- Compare the new Linux Render parity run with both observed optimized baselines (205 and 337 minutes), inspect each operation's timing, and keep the change only if all checks pass and the improvement is material without unstable resource use.

Non-goals: changing production rendering, fixture recipes, golden references, tolerances, required CI commands or dependencies, report schema, or the number of states/intents/timestamps checked. No public or persisted contract changes, breaking changes, or migrations are proposed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `render-regression-fixtures`: make rules-screen conformance execution bounded and independently timed while retaining its full matrix and fail-closed evidence.

## Impact

The change is confined to the native golden test harness in `crates/editor-core/src/renderer/golden/rules_screen.rs` and its OpenSpec evidence. Production `Renderer`, cross-language contracts, headless/bridge/desktop code, and protected CI policy remain unchanged.
