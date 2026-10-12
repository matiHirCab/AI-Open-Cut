## Why

Issue #179 reports native visual preparation advancing while job progress remains zero. Current main confirms that sampled visual materialization receives no progress callback, so clients cannot observe completed preparation work before final encoding.

## What Changes

- Report completed sampled visual-frame work through the existing numeric native progress callback for range preview and export.
- Keep progress finite, bounded and monotonic across preparation and final encoding; reserve one for successfully published and validated artifacts, including cache hits.
- Preserve existing headless/MCP event and job shapes, error identity, cancellation, revision precedence, output/state safety and resource limits.
- Govern the existing progress behavior with a canonical fixture, owned addition proof, deterministic slow/failing preparation and publication tests, and actual source/packaged/native verification.

## Capabilities

### New Capabilities

None. Existing rendering and numeric job-progress surfaces carry the behavior.

### Modified Capabilities

- `rendering-export`: preparation work becomes observable before final encoding, with explicit allocation and truthful successful-publication completion.

## Impact

Core renderer and sampled-visual preparation callbacks; tests for typed headless/worker and MCP job forwarding; canonical behavior fixture and its ownership/CODEOWNERS, predecessor proofs and documentation. Progress remains the existing normalized reported-work fraction; requests, schemas, operations, protocol/project versions and dependencies remain compatible. No persisted migration is introduced.

Base is actual main `2a7333833a9b2766f074c939039360c5959cd796`, with reviewed budget/color/schema45 behavior. Issue179 is open with no comments/dependencies; authorized GitHub collection listing found only PR198 open, so no duplicate implementation. PR198 at `7f7f2952` is independently accepted/all18-green but unmerged; preserve it as an exact-head compatibility target without copying pending diagnostics. Re-fetch main before publication and normally preserve any subsequently merged domain. Main's Windows unit-timeout investigation remains separate and unresolved; all required new-head gates must still pass.

## Non-goals

No durable jobs, decoder/startup redesign, root-cause claim for private installed media, renderer optimization, budget increase, new cancellation policy, progress ETA, generic provider progress change, blanket test-deadline/CI-policy modification, PR merge/deployment or separate desktop/video task access. Audio normalization and other nonsampled preparatory work retain their existing behavior; the scope is sampled visual work and truthful range/export publication completion.
