## Why

PR144 exact commit3ff6db678506d2043c2b1dba3161c0d810d036e7 fails Windows hermetic unit CI because the complete MCP predecessor-proof test exceeds its existing5000ms deadline. Its Rust formatting/Clippy/tests and other nine leaf CI jobs passed. Reduce redundant conformance work while retaining the deadline and every proof.

## What Changes

- Reuse the existing read-only module-level independently expanded MCP fixture as the first result, expand the source afresh as the second, and compare the two independently expanded JSON catalogs through exact complete serialized-byte equality instead of a large recursive matcher, and reuse the already serialized current bytes for the unchanged digest assertion.
- Compute the checked compact schema33→32 predecessor once within the same test and reuse it for schema32 expansion and the checked schema31 projection, whose helpers retain input cloning/isolation.
- Preserve every count/capability/digest/malformed-reference/unrelated-drift/current-registration oracle and the same single test and5000ms deadline. No production, contract, fixture, schema, comparator helper, timeout, CI policy or native behavior changes.
- Correct the existing nested inherited video test oracle from future-frame output-grid rounding to the already-approved held presentation timestamp at its exact mapped source clock. Keep all ten timestamps, four render intents, audio, SSIM, geometry, and byte tolerance unchanged. Synchronize the matching stale video sentence in docs/inherited-animation-timing.md without changing any clock or audio implementation.

- Correct only the legacy group travel/seam test's encoded-space half-red band to an independent linear half-red188 oracle. Preserve seam3/15, all geometry/offscreen/draft/history/SSIM controls; use one-byte lossless color precision and existing15-byte encoded precision.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- linear-light-compositing: ensure the independent nested media oracle uses the existing held-source contract without retiming future frames.
- contract-governance: bound the complete current→32→31→pre-linear exact MCP proof without reducing its content or deadlines.

## Impact

Only apps/agent-bridge/tests/contracts.test.ts proof statements and the legacy source-marker expectation/comment in crates/editor-core/src/renderer/golden/inherited_timing.rs, the group native travel test's independent color assertions in crates/editor-core/tests/groups.rs and this OpenSpec lifecycle/living requirement. Canonical digests and catalogs are unchanged. #52 approved WIP is safely checkpointed in stashaec17e0e84671a116ccae55a2c65c7a25ca65901; it resumes only after this verified PR144 correction. Human CODEOWNER review remains pending on draft144; no merge/deploy. Approval required before test edits.
