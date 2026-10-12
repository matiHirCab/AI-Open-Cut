## Why

Illustrated scene admission currently reports generic invalid arguments when existing native resource guards reject work. Issue #186 needs actionable, bounded diagnostics without duplicating core estimation or weakening those guards. Dependency #181 is merged in current main `84da152fac0378396099d8f01fc3b7339690361f`; #186 has no subsequent comments.

## What Changes

- Add optional `renderBudget` details to core-origin budget errors and carry them unchanged through headless, bridge jobs and MCP.
- Identify the limiting guard, its unchanged limit, charged work or conservative bound, units, overflow and bounded anonymous contributors where the accounting supports them.
- Govern canonical diagnostics and MCP schema additions with parity and predecessor compatibility evidence.
- Preserve stable codes, retryability, guard order, atomicity and artifact publication rules.

## Capabilities

### New Capabilities
- `native-render-budget-diagnostics`: core-owned budget diagnostics and thin transport propagation.

### Modified Capabilities
None. Existing rendering-export, shape, grid, motion blur and contract governance requirements remain effective; this capability adds diagnostics to their existing rejection behavior.

## Impact

Core error and evaluated-scene accounting, headless error serialization, bridge error schemas/workflows, canonical contracts, parity tests and documentation. This is an additive optional error response field, with no project/protocol version change or migration, no new operation, capability identifier or dependency.

## Non-goals

No budget increase, independent estimator, renderer redesign, quality reduction, progress (#179), decoder/spawn diagnostics (#177/#178), provider diagnostics or desktop change. Coverage is explicitly limited to existing shape segment/raster allocation accounting, candidate analysis nodes, effect pixel passes, motion blur pixel work, linear composition output-frame work and composition memory accounting. Other guards keep their existing error envelopes until their owning accounting can support reviewed diagnostics.
