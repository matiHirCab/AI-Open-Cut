## Context

The verified issue60 predecessor owns closed, bounded SpeechAlignment validation and schema38 provenance. Transcription currently resolves a managed asset through core, queues CPU inference, retains an opaque preview and atomically commits captions through core. Issue61 adds known-text inference, not a second editing/persistence system.

## Goals / Non-Goals

Provide known-text alignment through the existing transcription estimate/preview/commit boundary, including real local adapter plumbing and fake-provider tests. Preserve ordinary transcription and all prior proof authorities. Non-goals are those in proposal.md; especially no new persisted field/schema, marker operation, generated speech mutation or renderer behavior.

## Decisions

1. **Existing optional mode:** knownText on transcription estimate/preview selects optional Transcriber.align. Omission calls the unchanged transcribe path. Preview carries optional SpeechAlignment, and alignment-derived transcript segments continue through existing caption commit. A separate tool/job family would duplicate retention and lifecycle, so is rejected. Existing speech timestampSupport stays unchanged; transcription status carries independent knownTextAlignment support/limits, defaulting unsupported for legacy omission. A provider claiming support but lacking align fails closed using TRANSCRIPTION_UNAVAILABLE.
2. **Canonical owner:** EditorCore.validate_speech_alignment resolves the existing probed audio asset through the existing path policy, bounds knownText to nonblank4096 UTF-8 bytes, checks optional expectedRevision and optionally validates forced-quality SpeechAlignment with existing100000-segment/1MiB/4096-byte/safe-integer/order/duration rules. It returns the existing ResolvedAssetInput. Bridge uses it before inference and again against the captured source revision before retention. No core dependency on provider/config is introduced; implementing semantic bounds in Zod/Python alone is rejected. Missing references and revisions use existing core codes. Input text is preserved rather than normalized by the application.
3. **Real local inference:** pinned faster-whisper1.2.0 already supplies decode_audio, feature extraction, encode, tokenizer and find_alignment for supplied tokens. The adapter aligns caller text tokens directly against encoded local audio; it does not run recognition and relabel results or use evenly spaced estimates. Known-text mode is bounded to30 seconds and4096 input UTF-8 bytes, plus the model's text-token context budget before alignment. Actual decoded duration is bounded too. It emits words only, with empty sentence/phoneme arrays, forced quality, provider/model identity and positive ordered spans. Unsupported language/context, empty or degenerate/non-finite/out-of-range output fails rather than inventing timestamps. Language defaults to en for this known-text mode; ordinary transcription language detection remains unchanged. A new heavy dependency/model/provider is unnecessary and rejected. Reference: https://raw.githubusercontent.com/SYSTRAN/faster-whisper/v1.2.0/faster_whisper/transcribe.py (find_alignment) and tokenizer.py.
4. **One bounded queue:** align and transcribe share the existing adapter FIFO/concurrency-one/timeout/cancellation worker request mechanism. close and overload semantics remain. The worker knows media path/text/model only and no project/timeline state. Application snapshots structurally parsed alignment and segments inside the existing retained preview, then validates semantics in core. Provider mutation after completion cannot change retained data. During inference a changed source revision fails before token publication; after a retained preview, ordinary conflict retry still reuses the same token without inference.
5. **Compatibility and immutable evidence:** manual additions to headless/provider/MCP fixtures, ownership and capability declarations use protocol1 and schema38. Independently pin every predecessor canonical raw/semantic authority before executable changes. Exact issue61 addition removal must precede all older projection chains, preserve older raw files/requirements and unrelated-drift negative controls, and never rebuild expected catalogs from live schemas. Add dedicated focused consumers to the existing protected contract command without removing earlier consumers, workflow pins, environment, deadlines or assertions. No general fixture regeneration is allowed.

## Failures, security and rollback

Only core-resolved project media is passed to inference. Provider absolute paths never appear in MCP preview/estimate/job results. Invalid known text, asset duration and alignment use VALIDATION_FAILED; missing asset/project and revision use current stable codes. Unsupported/not-ready mode uses TRANSCRIPTION_UNAVAILABLE. Malformed provider data uses TRANSCRIPTION_INVALID_OUTPUT; worker failures retain TRANSCRIPTION_PROVIDER_FAILED. Queue full, timeout and cancellation preserve existing retryability. No partial project publish occurs during estimate/preview or failure. Caption commits retain their established atomic history/media provenance and reopen behavior.

## Risks / Trade-offs

-30-second/model-context bound is deliberate and advertised; long known narration needs future explicit segmentation rather than incorrect one-window alignment.
- DTW accuracy depends on local model/audio; deterministic hermetic/fake-provider tests verify the wrapper/boundary, not real-model accuracy. No model download/network inference is added or claimed as acceptance evidence.
- Small words/punctuation can yield degenerate spans; reject or semantically merge token punctuation under a covered provider rule, never fabricate positive time.
- Existing local integration/smoke OOM remains separate from unchanged standard CI acceptance. Preserve original failures and diagnostic evidence; final exact-head acceptance still requires all11 jobs.

## Migration and delivery

No persisted schema migration: preview alignment is ephemeral and caption commit reuses current source provenance. Undo/redo/reopen apply to existing captions and are covered for the known-text flow; read-only inference itself has no undo/batch edit. Rollback removes optional mode while existing schema38 projects remain readable. Start from verified e2985f09, keep PR base main and cumulative merge order155 then issue61. Main reconciliation and source review follow user authorization. No merge/deploy/closure.

## Open Questions

None required before scoped implementation. Any deviation from this design requires an approved artifact amendment before executable edits.
