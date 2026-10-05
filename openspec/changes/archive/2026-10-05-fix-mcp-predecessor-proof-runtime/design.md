## Context

Windows annotation for job111935808928 on exact3ff6db678506d2043c2b1dba3161c0d810d036e7 identifies only the deterministic MCP predecessor proof at contracts.test.ts386 timing out in5000ms. This test independently expands the current8MB JSON twice, recursively compares it through Vitest, serializes it again for its pinned SHA, and clones the compact33→32 projection twice. The same current catalog is already expanded at module initialization for actual registration parity.

## Goals / Non-Goals

Reduce repeated CPU/allocation work in the MCP test, preserving every exact proof and its existing5000ms deadline. No production or catalog/schema/SDK/expander/projection helper changes, shared-reference aliasing changes, test splitting, timeout extension, flags, fixture regeneration, version changes, CI-policy changes or weakened negative controls.

## Decisions

Use the existing module-level MCP_SURFACE as the first independently expanded current result and call expandMcpSurfaceCatalog on the canonical JSON source for the second. All current uses of MCP_SURFACE are read-only (or explicitly structuredClone before mutation); expected registration parity still consumes it. The full current JSON remains independently re-expanded within this test. No new cross-test memo/cache exists. Any unexpected fixture mutation now fails the exact independent comparison.

Serialize both complete expanded JSON catalogs and compare the entire serialized strings with strict equality, retaining every nested field, array order, scalar and object-key order. The source is imported parsed JSON and the unchanged expander creates plain JSON data; no undefined/function/symbol/cycle/NaN representation is introduced. Full byte comparison is stronger than deep equality for JSON-object insertion order, not a digest-only substitute. Reuse first serialized bytes only for the unchanged pinned current SHA256. Keep78 tool count.

Compute compact schema33→32 once with the unchanged checked projection, expand it and assert unchanged88b55... predecessor digest; pass the same compact source to the unchanged cloning schema32→31 helper, expand and assert unchanged2a3f... digest. Preserve exactly-one linear-light capability assertion and the unchanged pre-linear181c... digest after removing only that already-authorized capability. Existing malformed/missing/cyclic/nonlocal/sibling/unused reference, matching-key drift, descriptions/annotations/current-registration and all property-addition controls remain unchanged. No existing helper semantics change.

## Risks / Trade-offs

Full-text equality relies on the existing JSON-only domain; source parsing/strict expansion and unchanged negative tests enforce it. Reusing a read-only fixture exposes rather than hides unexpected mutations. Actual Windows load is unavailable locally; profile comparable stable runs but require all11 exact-correction-commit CI checks before claiming the failure resolved. No performance promise from Linux measurements.

## Verification / Migration Plan

Capture annotation and existing source/benchmark evidence before edits. Execute focused unchanged fullproof and malformed/drift cases, measure repeated same-runtime before/after proof with all4 pinned digests and immutable sources, run full bridge unit/type/lint/contracts/MCP/package, Rustfmt/strictClippy/workspace and hermeticPython. Independently review exact scoped test diffs and justified reuse of already-passing native checks for remaining suites whose relevant production/fixture inputs remain byte-identical; the corrected nested test requires fresh execution. Refresh pinned strictspecs then prearchiveprotected inventory (onlyown expected rejection), verify/sync/archive only this change, and require final unchangedprotectedgate+strictall. Push correction to existingdraft144; verify all11 CI results against that exact head. Reconcile #52 checkpoint on the verified implementation branch without losing work; no merge/deploy.

## Open Questions

None; independent/parent exact artifact approval precedes implementation.

## Independently diagnosed native oracle amendment (approval before edit)

Fresh tool-enabled workspace testing and the isolated exact nested timing test both fail at root700ms: source marker expected old frame3 versus actualRGB216. This is not a concurrent-load failure. The preexisting test selects a future source presentation timestamp by rounding its mapped root time onto the output frame grid; it applies that assumption even to direct preview. Approved linear-light rendering instead holds the last source presentation timestamp not later than the exact mapped source clock.

The fixture independently defines raw_leaf=((t-100)*1.5+50-100-20)*.5+10; rank-three delay150ms yields source275ms at root700. The lossless10fps source holds frame2 with timestamp200ms until300ms. Its240-byte source and parent opacity0.8 predict encodedRGB approximately217, matching216. Replace ONLY that test oracle frame selection with floor((raw_leaf-150)/100), and its explanatory comment. Do not call any production clock/decode helper. Keep the source media/fixture, all ten root timestamps, all four observations, geometry/opacity/SSIM/audio checks and tolerance12 untouched. No production or timing change, new tolerance, suppression or forced skip.

The exact tool-enabled nested test must pass afresh. Collect the remaining native-enabled workspace diagnostics, then require the standard unfiltered CI workspace and every corrected native suite to pass; qualify unchanged native-suite reuse explicitly, then all mandatory public/runtime checks. Other standalone native evidence may be reused only when the independent reviewer confirms its relevant inputs remain unchanged. Preserve both failed logs and rendered diagnostics. Explicit reviewer and parent amendment approval is required before this Rust test edit; the original approved artifact hashes remain historical evidence.

Synchronize only the stale video resampling sentence in docs/inherited-animation-timing.md to the already implemented held presentation timestamps at exact mapped source clocks. Explicitly preserve fractional inherited clocks, half-open activity and independent continuous audio trim/retiming. This is documentation consistency, not a production change.

## Independently diagnosed group-color oracle amendment (approval before edit)

The fresh native workspace subsequently reaches groups.rs and fails its legacy `(100..155)` red band at x4095: the fixture authors opaque red with opacity0.5 over the mandated opaque-black canvas. With identity parent gain and full interior support, linear red0.5 encodes to round(255*(1.055*0.5^(1/2.4)-0.055))=188, not encoded-space128. The original band has55 values around128 and is stale after approved linear-light composition.

Change only this native travel test's color oracle: calculate independent188 from the transfer formula, check both interior seam-adjacent RGB pixels against [188,0,0] with absolute one-byte lossless precision and existing15-byte encoded precision. Keep the separate old seam-continuity3/15, outside4085/4115<10, offscreen20000 black, repeated reentry250/750ms, every frame/range/export/draft observation, byte-equal draft and existing history/SSIM controls untouched. No production/color algorithm/fixture/tolerance broadening. Wrong encoded128 is60 away and must fail both color checks.

Collect remaining native diagnostics with stable source and no fail-fast. The standard CI unfiltered workspace must pass0 after corrections. Every corrected native test binary/full affected suite must pass with actual configured tools. Other passing native audit suites may be explicitly reused only on unchanged relevant inputs; do not claim a single full-native invocation passed if its aggregate exit failed. Reconciled per-suite native evidence must distinguish failures, corrected fresh passes and qualifying reused passes. This avoids repeating already-passing expensive native suites after an isolated test-only oracle change. Explicit reviewer/parent exact amendment approval is required before group test edits.
