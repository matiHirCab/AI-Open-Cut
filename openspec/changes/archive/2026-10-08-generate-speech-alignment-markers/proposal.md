## Why

Issue #62 in epic #8 needs narration timing to become named timeline cues. Issues #60 and #61 now provide validated alignment provenance and known-text alignment, but callers still have to create every marker independently.

## What Changes

- Add core-owned `speech_markers_generate` as an ordinary typed edit, usable standalone and inside ordered `timeline_batch_edit`, from an existing audio asset's persisted alignment or an explicitly supplied validated alignment.
- Add closed `none`, `sentence`, `selected_word`, and `all_word` marker policies. Selected words use explicit zero-based word indices; generated names derive deterministically from timing text and avoid every existing same-scope name.
- Add optional marker policy to generated-speech insertion and preview insertion, creating the audio asset, item, and root markers inside the existing single transaction. Omission preserves current behavior.
- Publish a dedicated canonical fixture, typed Rust/headless/TypeScript/MCP surfaces, additive capability, exact predecessor projections, tests, and documentation.

## Capabilities

### New Capabilities
- `speech-alignment-markers`: bounded deterministic alignment selection, naming, timing, scoped batch edits, and atomic generated-speech insertion.

### Modified Capabilities
- `contract-governance`: governed additive speech-marker edit/policy surfaces and exact historical contract preservation.

## Impact

Core model/marker/timeline/store, headless request forwarding/capabilities, bridge schemas/headless types/timeline and speech workflows, canonical fixtures/ownership/CODEOWNERS, parity tests and protected command policy where needed. This is additive to protocol 1 and schema 38. Existing Marker records already persist all required state, so there is no new persisted field, migration, or renderer semantics. Existing clients, numeric timing, ordinary speech insertion, replacement and transcription retain their current contracts.

## Non-goals

No invented timestamps, segmentation or provider fallback; no provider protocol or known-text algorithm change; no automatic transcription-caption marker commit, speech replacement/regeneration marker updates, audio buses/events, inspection surface, desktop feature, or automatic binding of visual items. Explicit aligned transcription results can use the standalone/batch edit. No merge, deployment or issue closure. PRs target main; this issue branch starts from verified #61 head 76b87baf, and its cumulative PR must merge after PR156 until the user merges that predecessor.
