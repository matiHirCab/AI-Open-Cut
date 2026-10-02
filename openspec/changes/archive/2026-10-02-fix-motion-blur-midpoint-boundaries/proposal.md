## Why

Independent review of issue44 head 28a1ecf3 found floating cancellation moves the
fourth midpoint of a 360-degree, five-sample exposure at 500 ms/25 FPS to 507
instead of 508. A held parent key at 508 produces 80/20 rather than 60/40
coverage. Real FFmpeg 6 and 7 reproduce frame MSE 81.28125 against the unchanged
independent 1.0 limit. PR132 head b03ca51d still contains that sampler.

## What Changes

- Floor midpoint offsets as exact integer ratios of the stored finite binary64
  angle, without epsilon snapping or floating subtraction.
- Cover every count 1..16, representable neighbors, fractional periods,
  subnormals, high integer roots, deterministic clipping and an independent
  held-parent frame/range/draft/export oracle on actual FFmpeg 6 and 7.
- Preserve inherited EvaluatedScene clocks, disabled compatibility, rendering,
  audio, limits, revisions, batches, history migration and stable errors.

## Capabilities

### Modified Capabilities
- `motion-blur-sampling`: clarify exact midpoint flooring and boundary conformance.

## Impact

Canonical arithmetic remains solely in editor-core. This corrects the existing
mathematical timing contract and adds fixture cases/documentation; request shapes,
protocol/schema/capability identifiers and acceptance bounds remain unchanged.
No migration, new operation, dependency, media/resource access or CI-policy change.
All prior commits, PR132 archival and human approval are preserved. A fresh
CODEOWNER review applies to this correction before archival. Commit on top of
b03ca51dcf1f1bf00bb8f67f9bf45504363dc9cc without rebase or force push; deliver only
missing corrective commits in a small superseding private Library bundle if
publication remains blocked. No credentials, merge or deployment.
