## Why

Issue61, within epic8, requires provider-neutral known-text alignment through the existing local speech/transcription boundary. Issues59 and60 are implemented: timestamp support is truthful and independent, and core schema38 owns bounded alignment provenance. Ordinary faster-whisper transcription currently recognizes text; it has no known-text mode. This change provides a real local alignment adapter and fake-provider acceptance coverage without inventing timing or adding timeline placement policy.

## What Changes

- Add optional knownText to existing transcription estimate/preview requests; omission retains ordinary transcription. Advertise separate known-text alignment support/limits in transcription status, with unsupported legacy fallback.
- Add a core-owned, read-only validate_speech_alignment operation that bounds known text, resolves an existing probed audio asset, optionally checks the source revision, and validates returned forced alignment against that asset. It returns the existing provider-only resolved input; no project mutation.
- Extend the provider-neutral Transcriber boundary with optional align. The existing CPU faster-whisper1.2.0 adapter uses its audio encoder and known-token alignment path for at most30 seconds and4096 UTF-8 bytes, rejects token/model/output limits rather than fabricating timing, and returns word alignment with forced quality and real producer/model identity. Sentence/phoneme support remains false for this mode.
- Retain optional alignment in the existing expiring transcription preview. Existing caption commit, conflict retry, discard/expiry, cancellation, queueing and close remain authoritative. No absolute media path is returned through MCP.
- Manually synchronize canonical provider/headless/MCP fixtures, ownership, capability declarations, exact historical projections and required parity coverage.

## Capabilities

### New Capabilities
- known-text-alignment: bounded read-only validation, truthful provider support, real local known-token alignment and fake-provider acceptance.

### Modified Capabilities
- transcription-captions: optional knownText mode and optional retained alignment, preserving existing caption lifecycle.
- contract-governance: additive provider/headless/MCP additions and exact predecessor preservation.

## Impact

Core owns text/timing/duration/revision/reference semantics; headless remains typed transport; bridge owns orchestration/retention; Python owns CPU inference. Protocol1 and project schema38 remain unchanged. New uniquely named operation/capability and optional fields are additive; no identifier is reused or narrowed. Legacy status omitting alignment support normalizes to unsupported. Existing transcription requests/results retain meaning.

## Non-goals

No markers, batch timeline operation, audio buses/events, speech-provider timestamp declaration change, persistence of new asset metadata on existing media, schema migration, renderer/export change, external inference service, automatic model download or arbitrary client path. Caption commit continues its existing atomic core mutation; batch aliases are not applicable to read-only inference. Future issue62 owns marker policy.

## Verified predecessor and delivery

Start from e2985f09e565f6386d4804da266a964c1c3bc75b: issue60 PR155, all11 exact-head CI37679754701 successful, focused37679754747 successful. Main remains c756a999 (user-merged PR154). This issue's draft PR MUST target main and contain issue60 until its user merge; merge order155 then this PR. Preserve later user merges without discarding work. Completion requires substantive delegated owner review, the full repository workflow and all11 exact-final-head CI. No merge/deployment/issue closure is authorized.
