# Known-text speech alignment

Use optional `knownText` in `transcription_estimate` or `transcription_preview` to align supplied text to an existing managed audio asset. Omit it for ordinary recognition. Discover `transcription_get_status.knownTextAlignment`: support, independent sentence/word/phoneme flags, maximum duration and UTF-8 text bytes. Legacy providers omitting the record expose unsupported mode. Speech-provider timestamp support is unchanged.

The CPU faster-whisper1.2.0 adapter aligns supplied tokens to encoded local audio using its model alignment path. It reports words only, with `quality: forced` and real provider/model identity, and defaults the known-text language to en. It requires at most30 seconds/4096 UTF-8 bytes and the model's token context; actual decoded duration is checked. Degenerate, non-finite, overlapping/out-of-duration timing fails rather than being interpolated or relabeled. No model download or network inference is automatic. Long narration needs separately planned segmentation.

Core's typed `validate_speech_alignment` resolves the source and owns text/revision/timing/count/UTF-8/duration validation. Estimate does no inference. Preview retains an owned optional alignment and caption transcript behind the existing expiring token, exposes no absolute path and does not edit the project. A source revision change during inference rejects before token publication. A later `transcription_commit_preview` conflict retains the token for a retry without inference. Commit preserves existing atomic captions/source provenance, undo/redo and reopen. Discard/expiry/close release retention.

This adds no persisted project field, schema migration, marker placement or timeline batch operation. Protocol1 and schema38 remain. Validation/reference/revision failures retain existing core codes; unsupported mode uses TRANSCRIPTION_UNAVAILABLE, malformed provider output uses TRANSCRIPTION_INVALID_OUTPUT, and provider/queue/timeout/cancellation failures keep existing transcription codes and retryability.

Known text is sent verbatim to inference, and alignment word strings retain the provider's values. Caption segments retain the existing caption schema's outer whitespace trimming.

Canonical additions: known-text-alignment-v1.json, transcription-provider-v1.json, headless-protocol-v1.json and manually governed MCP declarations. Fake-provider and hermetic model-boundary tests verify supplied-token execution and failure paths without claiming real-model timing accuracy.
