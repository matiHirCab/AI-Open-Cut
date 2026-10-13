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

Initial verified publication used main `2a7333833a9b2766f074c939039360c5959cd796`. Actual main is now `0f069ae4d23a356a60555aa71137ba9d92314b63`, including user-merged PR198 whose source tree matches that merged main. Issue179 remains open without comments/dependencies. PR199 at `c2fa914d8ebf4a1d66abca7b5d7c22fd6a458ce2` passed all18 exact-head jobs but now conflicts with main. Reopen only this approved change, normally integrate the complete merged FF domain using the independently accepted strict composition, and run fresh affected publishing checks before commit/push. Pending motif PR200 at `6fd0145465332ee832a2e5836535b6dfdbed3207` remains an exact compatibility target, not a publishing dependency; preserve attributed joint evidence only for unchanged inputs. Current-main CI38327818049 succeeded; prior2a Windows failures are historical and their cause remains unconfirmed. Re-fetch main before publication and require all18 new-head jobs again.

## Non-goals

No durable jobs, decoder/startup redesign, root-cause claim for private installed media, renderer optimization, budget increase, new cancellation policy, progress ETA, generic provider progress change, blanket test-deadline/CI-policy modification, PR merge/deployment or separate desktop/video task access. Audio normalization and other nonsampled preparatory work retain their existing behavior; the scope is sampled visual work and truthful range/export publication completion.
