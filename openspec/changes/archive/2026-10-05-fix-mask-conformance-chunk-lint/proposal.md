## Why

PR143 CI rejects two existing mask native-conformance assertions under strict Clippy because chunks_exact_to_as_chunks requires fixed-width array iteration. Preserve the independent visible-red and nonzero PCM witnesses while using the compatible typed array API.

## What Changes

- Replace only RGB chunks_exact(3) and PCM chunks_exact(4) with as_chunks::<3/4>().0.iter(); decode PCM directly from the resulting four-byte array.
- Preserve exact length checks, complete-group iteration, thresholds, all-intent identity assertions and test execution.
- Compare actual local toolchain evidence with available CI evidence without weakening lint policy, and rerun all required checks plus exact-commit remote CI.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- mask-models: strengthen existing native identity-conformance portability through a narrowly scoped test implementation correction.

## Non-goals

No production, mask semantics, schema, capability, public contract, fixture, tolerance, toolchain pin, workflow or warning suppression changes. No merge or deployment.

## Impact

Only crates/editor-core/tests/mask_models.rs and specification evidence. as_chunks is stable since Rust1.88 and compatible with pinned1.97. Existing test/API/output behavior remains unchanged. User explicitly requests this correction and delegates issue-scoped specification approval; independent review precedes implementation and completion.
