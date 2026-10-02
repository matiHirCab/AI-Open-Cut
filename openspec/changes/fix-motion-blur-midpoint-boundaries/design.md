## Context and decision

The inherited issue43 pipeline and issue44 temporal composition are reused.
The expression width*((i+0.5)/N-0.5) rounds intermediate subtractions before
flooring. The equivalent exact offset is
angle*1000*(2*i+1-N)/(720*projectFPS*N). Decode the validated positive binary64
angle into its integer significand divided by a power of two. Use checked u128
products and floor positive ratios or negate the ceiling of negative ratios.
All enabled angles are <=360: binary denominator shift is 44..1074 and numerator
is below 2^68. If the checked power/product denominator exceeds u128 it exceeds
that numerator, yielding zero for positive offsets and minus one for negative
nonzero offsets. Subnormals remain valid; do not use epsilon snapping or truncate
high bits with an unchecked shift. Zero coefficients remain zero.

Add the offset to the integer root with existing saturating signed addition and
clamp once to [0,durationMs-1]. Adjacent roots above 2^53 remain exact. Preserve
all inherited fractional clocks, equally weighted duplicates, disabled/single
sample compatibility and the existing canonical validation/resource limits.
No schema or public operation changes, migration, seed, audio change or fallback
change. Invalid inputs retain existing non-retryable typed errors.

## Independent verification

For every N1..16, angle18*N at25FPS gives integer offsets2*i+1-N. Test those
values and their representable neighbors with integer-only oracles. Also test
dyadic fractional angles and non-dividing FPS1,24,25,30,59,60,144,u32MAX,
subnormals, timeline ends, duration1, duplicate clipping and high roots. Canonical
constant cases use Python Fraction of the parsed binary64 value rather than the
implementation's decomposition.

The real native fixture animates a held parent translation and disjoint white
8x8 leaves. Fixed analytic 3/5 and2/5 coverage produces the independent reference;
it never obtains coverage from renderer samples. Render frame500, materialized
draft500 and range500..580. Final25FPS export selects its actual480ms grid frame
with the equivalent held boundary488, rather than mislabeling it500. Preserve
MSE<=1,SSIM>=.99,PCM RMS<=.0001 and one-frame timing. Capture before/after on
actual FFmpeg6 and7 and retain failed logs. Audio controls are zero-shutter scenes.
Assert authoritative project/history bytes are unchanged by rendering.

Rerun workspace format/strict Clippy/tests, bridge/type/contracts/unit/integration/
mocked smoke/Python, actual native animation/cache/headless/geometry/font suites
and policy gates on the reconciled branch. Reuse the historical seven-corpus and
56-case rules-screen evidence only after hashing unchanged tracked inputs and
proving no fixture authors motion blur; explicitly label them reused, not rerun.
No golden, threshold, timeout, workflow, dependency or budget changes.

## Review, reconciliation and delivery

Remote PR132 head b03ca51d adds only approved archival/living-spec records to28.
Keep those records byte-for-byte. The prior human CODEOWNER approval is for28,
not automatically for this numerical fix. This separate change records delegated
spec approval before code and remains active until fresh implementation review.
Pre-archive protected rejection naming only this change is expected, not a pass.
Preserve the original local d37eec8e spec-approval commit on its existing branch;
this branch starts directly from the current PR head, with ordinary new commits.
The new recovery ZIP must supersede the previously delivered28 ZIP and include
only commits missing from b03ca51d, exact base/head/tree, hashes and verified
fresh import instructions. Private user Library only; no merge/deploy/auth changes.
