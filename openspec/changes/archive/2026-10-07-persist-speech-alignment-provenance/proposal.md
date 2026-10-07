## Why

Issue #60 requires durable speech timestamps and truthful quality provenance before forced alignment (#61) and marker generation (#62). Dependencies #59, #17, and #82 are closed; the verified implementation predecessor is PR154 head `72bd389bc608b5860ad00583d23e76e769fad06e`, with all11 required CI jobs plus focused native evidence SUCCESS. Main remains `2d748508b7838a7a3150837b6ac2556b50b5dec4`; its original failed logs are preserved. This branch inherits the verified prerequisite while the draft targets main, with merge order154 then60.

## What Changes

- Advance persisted projects from schema37 to schema38 and migrate current state plus every retained undo/redo snapshot through the existing locked atomic generation transaction.
- Add optional provider-neutral speech alignment to generated speech provenance: independent sentence/word/phoneme timed text arrays, explicit native/forced/estimated quality, and bounded aligner identity/version metadata.
- Validate timing, ordering, text, complexity, and duration in editor-core; carry an owned snapshot of optional returned alignment through existing bridge preview/commit/regeneration paths.
- Stage generated speech commit/replacement and any legacy current/history adoption in the existing journal/resource ledger, so failed target/revision/metadata checks cannot publish migration or orphan new resources. Required nullable alignment model fields distinguish omission from null; raw assets use the existing lossless buffer before pre38 presence guards.
- Synchronize canonical persisted/headless/MCP contracts, consumers, fixture parity, and current schema markers while retaining exact historical predecessor proofs.

## Capabilities

### New Capabilities

- `speech-alignment-provenance`: persisted timing and quality semantics, canonical bounds and failure cases.

### Modified Capabilities

- `speech-generation`: preservation of optional synthesis alignment through existing commits and replacements.
- `project-persistence`: atomic schema38 migration and pre-introduction source rejection.
- `contract-governance`: additive alignment contracts and exact schema38 predecessor projection.
- `masked-hero-reveal`: current-schema adoption with exact schema37 predecessor recipe/reference preservation.

## Impact

Core model/validation/migrations/store tests; bridge speech/schema/headless types; canonical contract ownership and catalogs; parity/lifecycle tests and docs. The protocol remains v1. Optional fields preserve existing requests and unaligned speech. **BREAKING persisted version boundary:** older strict response consumers may reject schema38; migration is forward only and rollback requires complete project/history backups.

Required canonical parity adds the dedicated speech-alignment Rust/Zod consumer suites to the existing contracts:check command and synchronizes its protected exact-command policy and additive omission/failure-mask controls. Preserve every prior consumer, command order, pin, workflow/job/environment/deadline and policy assertion; this strengthens the existing gate without adding a transport operation.

## Non-goals

No inference/forced aligner, marker generation, new operation, provider-worker protocol change, audio processing, renderer semantics, automatic timing estimation, new asset references, or changed deletion/GC policy. No skipped checks, relaxed historical digests, test deadlines, or altered unrelated work.
