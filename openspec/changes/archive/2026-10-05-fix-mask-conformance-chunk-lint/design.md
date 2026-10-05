## Context

User supplies PR143 Ubuntu strict-Clippy diagnostic chunks_exact_to_as_chunks at mask_models.rs762 and793. Actual local Rust1.97.0/Clippy0.1.97 share commit2d8144b788 (2026-07-07); .prototools pinsRust1.97.0 and CI delegates installation to setup-toolchain. A standalone explicit-lint probe confirms this lint is unknown in localClippy; user-provided CI diagnostic establishes different lint support. Exact actual CI version numbers are unavailable through currently accessible log evidence and must not be invented. Log access does not block the supplied compatible correction.

## Goals / Non-Goals

Keep every meaningful native assertion unchanged and make the existing test source compatible with strict Clippy. No production/schema/contract/fixture/workflow/pin/lint-policy changes, merges or deployments.

## Decisions

Replace chunks_exact(3) with as_chunks::<3>().0.iter() and chunks_exact(4) with as_chunks::<4>().0.iter(). Both APIs visit the same ordered complete groups and exclude the same trailing remainder; existing length assertions remain unchanged. PCM iteration now yields &[u8;4], so f32::from_le_bytes(*b) preserves exact little-endian decoding and avoids a redundant fallible conversion. as_chunks is stable sinceRust1.88 and available in pinned1.97. Keeping chunks_exact with an allow would suppress CI policy and is rejected; changing witness thresholds or adding a trivial API-equivalence test would dilute the existing actual native evidence.

## Risks / Trade-offs

Two test-only iterator representations change, with no production risk. Execute the existing16 mandatory native mask tests, fullworkspace/strictClippy and required bridge/contracts/Python/native gates on stable inputs; reuse unchanged passing native evidence only where repository policy permits and record its inputs explicitly. Inspect exactdiff independently. Confirm remoteCI on the exact pushed commit before downstream implementation.

## Migration Plan

None; persisted/public behavior remains unchanged. Existing approved51 planning is preserved externally while working on50; after verifiedCI correction, create/advance51 from verified50 and reconcile exact predecessor SHA while preserving semantic catalogs/digests.

## Open Questions

Exact CI Rust/Clippy runtime versions remain an evidence limitation; supplied diagnostic and supportedAPI suffice for the correction. Required local checks, independentconformance, ownchangearchive/finalpolicy and exactcommitremoteCI success remain completion gates.
