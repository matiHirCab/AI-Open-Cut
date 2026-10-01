## Why

PR131 at e1010bb97174d44a5d19c61a90514bf2336845ca fails strict Clippy on all three CI platforms and Linux real-render parity. These failures prevent using issue43 as a verified prerequisite for issue44.

## What Changes

- Correct the fixed-width RGBA byte iteration requested by strict Clippy without changing decoding semantics.
- Reproduce and correct identity color drift and frame/range/export differences for inherited nested and compound sampling on supported FFmpeg versions. Preserve all tolerance thresholds, deterministic fixtures and golden references.
- Preserve schema27, public operations, capability names, inherited clocks, ordering, effects, audio, revisions, safe errors and destination cleanup. No feature expansion.

## Capabilities

### New Capabilities
- `extended-animation-ci-parity`: Portable conformance evidence for the already approved issue43 behavior.

### Modified Capabilities
None; existing extended visual requirements remain binding.

## Impact

Bounded to editor-core raster preparation, render composition, affected independent tests and evidence documentation. Supported CI Linux/Windows/macOS checks remain unchanged. Existing PR131 corrective commits may be pushed normally under the user's explicit new instruction; no merge, force push, deployment or CI/security changes. Issue44 stays isolated and unpublished.

## Non-goals

No motion blur or schema28 in this branch, no contract expansion, tolerance relaxation, skipped native fixtures, golden recapture, account/access changes or unrelated proposals.
