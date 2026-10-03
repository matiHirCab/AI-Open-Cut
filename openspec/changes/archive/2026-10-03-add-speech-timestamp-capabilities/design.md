## Context

Speech providers own runtime metadata; the bridge validates and exposes it through `tts_get_status`. Kokoro generation emits WAV and duration only. Sentence segmentation and pauses provide no emitted alignment. Rust owns persisted speech intent but does not consume worker status.

## Goals / Non-Goals

Discover independent timestamp support with conservative backward compatibility. Preserve audio synthesis, public error meanings, project behavior, and provider neutrality. Alignment generation and persistence remain excluded as described in the proposal.

## Decisions

- Use `timestampSupport: { sentence: boolean, word: boolean, phoneme: boolean }`. The keys describe timestamps actually available from the provider integration, not hypothetical model internals. No implication between flags. `false` means unsupported through this integration; it does not distinguish theoretical support from unavailable metadata.
- Make only the whole object optional for legacy input and default it to all false. An explicit object must contain all three booleans with no unknown keys; reject nulls, arrays, partial objects, coercible scalars, and unknown types. A partial-default design was rejected because it hides malformed capability declarations.
- Normalize at the service and adapter boundary; type provider `status()` as schema input so older providers can omit the field while bridge consumers receive normalized output. Translate only status-schema validation failures at both boundaries to non-retryable `TTS_INVALID_CAPABILITIES`; preserve unrelated transport/startup errors and the existing unavailable fallback. Malformed status is never silently downgraded to unsupported. Verify this through adapter, service, and MCP error translation.
- Kokoro advertises all false whether loaded, cached, cold, or unavailable. Do not infer timestamp support from readiness, chunking, speed, or voices. Preserve independent declarations from replaceable providers.
- Keep provider contract major v1 and worker package `version` semantics. Presence of the additive object is feature discovery; do not overload package version. The canonical synthetic provider fixture exercises mixed support. Add legacy and malformed fixture cases for TS and Python parity. Update only `tts_get_status.outputSchema` in the MCP catalog and its deliberately pinned expanded digest.
- No headless operations/capability identifiers change because this discovery is owned by the bridge/provider layer. The unchanged Rust provenance fixture remains valid and tested by the full gate.

## Risks / Trade-offs

Strict old status decoders may need to allow the ignorable new field; documented additive response policy governs this change. Legacy providers are conservatively reported unsupported, never assumed capable. Metadata is a provider claim rather than proof of alignment accuracy; generation formats and timing coordinates are outside this issue. Status creates no timestamps, so no coordinate/order/duration validation is introduced.

## Migration Plan

Ship worker, schema, bridge, fixtures, and documentation together. No persisted migration. Rollback restores status metadata only, with no project conversion. Existing requests stay unchanged.

## Verification

Test all eight flag combinations, legacy omission through a real fake-worker adapter and service, explicit malformed metadata, Kokoro cold/ready/loaded status, and unavailable fallback. Assert canonical fixture cases and MCP schema parity. Run Rust formatting, strict workspace Clippy/tests, bridge typecheck/lint/unit/contracts/integration/packaged smoke, hermetic Python, strict OpenSpec and protected gate. Independent spec and implementation reviews precede verification and archival; record delegated approval honestly and request designated owner review on the draft PR.

## Open Questions

None.
