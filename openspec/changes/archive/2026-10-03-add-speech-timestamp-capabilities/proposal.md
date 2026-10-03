## Why

Issue #59 requires clients to discover sentence, word, and phoneme timestamp support independently. Current speech status has no alignment metadata, and sentence chunking must not be mistaken for emitted timestamps.

## What Changes

- Add optional-on-input `timestampSupport` status metadata with independent strict boolean `sentence`, `word`, and `phoneme` fields, normalized to all false when omitted.
- Make the current Kokoro worker and unavailable adapter fallback explicitly advertise all false.
- Synchronize provider and MCP fixtures, ownership evidence, tests, and speech documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `speech-generation`: truthful independent timestamp support discovery and conservative legacy compatibility.

## Impact

This is an additive provider-v1 and MCP response change. Existing provider responses remain valid when the new object is absent; existing request, audio, provenance, and project contracts retain their meaning. Native strict clients must update their status decoders to accept the new response field.

Affected consumers: Kokoro worker, bridge status parsing/service, `tts_get_status` output schema, provider/MCP catalogs, and their tests. The speech ownership entry will include the service and parity consumers; CODEOWNER is `@matiHirCab`. Rust consumes only unchanged synthesis/provenance sections, so no Rust declaration change is warranted.

## Non-goals

No timestamp generation, alignment arrays, requests for alignment, persisted provenance changes (#60/#61), migration, headless mutation, project schema change, batch behavior, timeline editing, or rendering semantics. Discovery creates no artifacts and performs no project mutation; revision conflicts, undo/redo, reopen, render parity, and migration scenarios are inapplicable to this metadata-only issue.
