## ADDED Requirements

### Requirement: Independent nested held-media conformance oracle
The existing real-native nested inherited video conformance oracle MUST select the last source presentation timestamp not later than its independently computed exact mapped source clock. It MUST NOT select a future frame by rounding its mapped root time to the output frame grid. For the authored lossless10fps fixture, root700ms maps to source275ms after inherited clocks and rank-three delay150ms and MUST hold frame2 at source200ms. The correction SHALL affect only the test frame-selection expression/comment, preserve all ten root timestamps and all four frame/draft/range/export observations, retain source fixtures, audio/SSIM/geometry/opacity assertions and byte tolerance12, and call no production clock/decode helper. The matching inherited-timing documentation SHALL describe these held samples, unchanged fractional inherited clocks, half-open activity and independent continuous audio retiming. Production timing, catalogs, schemas, budgets, tolerances and CI policy MUST remain unchanged.

#### Scenario: Hold the independently mapped fractional source sample
- **WHEN** the existing nested lossless source is observed at root700ms under the authored inherited clocks and rank-three delay
- **THEN** the independent source clock275ms holds frame2 instead of the future source300ms frame3, and every unchanged audio, geometry, alpha, SSIM and four-intent pixel assertion passes with real native tools

### Requirement: Independent linear half-red group seam oracle
The existing native long-distance group travel/seam test MUST derive expected half-opacity red over opaque black from the approved linear-light transfer function, producing RGB[188,0,0]. Both full-interior pixels adjacent to the4096 tile seam MUST match within one byte for lossless preview/draft and within the existing15-byte encoded bound for range/export. The test MUST retain its separate seam-continuity3/15 bounds, outside4085/4115 darkness, offscreen20000 darkness,250/750ms reentry, all frame/draft/range/export observations, draft byte equality, history and SSIM controls. Fixture/production/color/schema/budget/CI behavior SHALL remain unchanged; the wrong encoded-space128 result SHALL fail the absolute color oracle.

#### Scenario: Preserve linear color and continuity through travel and reentry
- **WHEN** the authored red rectangle at opacity0.5 reaches position4090 and later reenters after offscreen travel
- **THEN** both seam-adjacent interior pixels have independently predicted linear half-red188, unchanged seam/geometry/offscreen/draft/history/SSIM controls pass across every original observation, and encoded-space128 fails without expanding any existing tolerance
