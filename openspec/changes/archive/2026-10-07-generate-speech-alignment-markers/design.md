## Context

The branch begins at verified #61 head 76b87baf (all 11 required jobs in run 37691923440). Main contains the user's #60 merge d040cdec. Core already owns scoped Marker records, alignment bounds, atomic edit candidates, batch aliases and generated-speech resource rollback. No active predecessor change remains.

## Goals / Non-Goals

Implement #62's four policies through core and public transports, including atomic speech insertion. Preserve all predecessor behavior and failure coverage. Non-goals are those in proposal.md; transcription clients can supply the owned forced-alignment preview to the explicit marker edit, but caption commit and speech replacement are unchanged.

## Decisions

1. Add `SpeechMarkerPolicy`, a closed tagged union: `{type:"none"}`, `{type:"sentence"}`, `{type:"all_word"}`, or `{type:"selected_word",indices:[...]}`. Indices are unique nonnegative word ordinals, at most 4096, and are evaluated in source order; missing ordinals reject. Explicit indices distinguish repeated words. Alternatives: matching strings introduces locale/case/occurrence ambiguity; free-form callback policies violate closed contracts.
2. Add EditOperation `speech_markers_generate` with `scope`, `assetId`, `startMs`, `markerPolicy`, and optional non-null `alignment`. Core resolves the audio-bearing asset in the candidate and validates positive duration and intrinsic/duration alignment bounds. Omitted alignment selects persisted speech-generation alignment; explicit alignment supports #61's ephemeral forced result without adding a persisted format. Quality/producer fields are accepted as existing provenance, never relabeled. No timestamps are inferred. `none` creates zero markers without requiring alignment; other policies require the requested nonempty granularity. Alternatives: bridge loops lose atomicity/core ownership; persisting redundant marker source state would require migrations without serving this issue.
3. Marker time is startMs + selected segment.startMs; segment end plus start must also fit JavaScript-safe milliseconds. Component-local placement must fit that component's duration; root has no new duration restriction. Convert each segment text to a deterministic ASCII base: retain ASCII letters/digits, replace runs of other characters with `_`, trim outer `_`, prefix `speech_` if the first byte is not a letter, use `sentence_<1-based ordinal>` / `word_<1-based ordinal>` if empty, truncate the ASCII base to 120 bytes. Reserve each same-scope existing name; collisions choose the smallest available `_2`, `_3`, ... suffix. Names are unique, ASCII-valid and at most 128 bytes. IDs use the existing UUID-backed marker identity; they are stable through history/reopen, while names and times are deterministic for the same candidate. No inference from punctuation or phonemes when sentence/word timings are absent.
4. Batch scope aliases resolve existing `component:@alias` references. The generated IDs are returned through changedIds in source order. The existing scalar resultAlias may be used only when exactly one marker is created, with complete rollback for zero/multiple results; multiple markers are addressable by returned IDs or unique names. This avoids inventing array-alias semantics. Existing edit transaction behavior gives one revision/history entry even for an explicit standalone none policy; omitted none in speech insertion creates only the pre-existing asset/item edit.
5. Optional default-none markerPolicy is forwarded by generated speech insertion and speech preview placement type insert into existing core commit_generated_asset. Core runs the same generator after adding the candidate asset/item and before authoritative publication. Root offset is the inserted item's startMs. Any failure uses tracked resource rollback and retains preview/conflict retry behavior. The replacement placement variant does not acquire markerPolicy. Regeneration never silently moves or duplicates existing cues. Existing result records retain their exact shapes; callers use marker_list for generated cue inspection until #70.
6. Add canonical speech-alignment-markers-v1 fixture plus speech_alignment_markers_v1 capability. Manually synchronize affected catalogs and CODEOWNER consumers. Capture immutable #61 raw catalogs and semantic/expanded hashes; project exactly the reviewed additions before invoking all existing historical proofs. Full parity command retains all existing consumers and adds marker coverage. No provider wire protocol, project schema, stable error catalog or renderer change.

## Failures and rollback

Malformed closed policy/request shapes use existing transport INVALID_ARGUMENT/Zod rejection; semantic policy, selection, missing alignment/granularity and bounds violations use non-retryable VALIDATION_FAILED. Missing source asset uses ASSET_NOT_FOUND, non-audio UNSUPPORTED_MEDIA, missing component ITEM_NOT_FOUND, stale revision retryable REVISION_CONFLICT. Existing project/storage/integrity precedence is preserved. No project/history/revision/aliases or managed resource can escape a failed transaction. Generated markers contain no asset references and do not alter asset retention or render instructions.

## Risks / Trade-offs

- ASCII marker names may lose non-ASCII wording; deterministic fallback/ordinal names and original alignment metadata remain inspectable. Test punctuation, numeric prefixes, Unicode-only text, truncation and collisions.
- Alignment can contain 100,000 segments while one composition holds 4096 markers; count/select validation rejects before constructing/publishing excess output, with existing maximum bounds unchanged.
- A none standalone edit retains generic revision semantics; document it and test it rather than inventing a second mutation pathway.
- Local worker memory/process-reaping caveats remain observable; preserve original failures, use diagnostic environment changes only as diagnostics, and require unchanged standard exact-head CI before completion.

## Migration Plan

Schema 38 and retained history formats are unchanged: generated records are ordinary existing markers. Existing schema38 projects and predecessor fixtures remain exact, and new marker records persist through normal undo/redo/reopen. No deployment or merge is performed. Cumulative PR targets main and requires PR156 first until user integration is independently reconciled.

## Open Questions

None requiring user input; decisions above are issue-scoped and approved under delegated specification authority before executable edits.

## User merge reconciliation

At 22:47 UTC the user reported PR156 merged. Fresh GitHub and Git verification confirmed merge e22c34367e7dcdf1ab473222059e9a8a779eea42 contains verified 76b87baf as an ancestor and has exactly the same tree. This already-created issue62 branch was fast-forwarded through that merge without changing or discarding any in-progress edits. Its PR will show issue62 scope against main; cumulative ordering with the predecessor is now satisfied. Postmerge CI37697399278 has all11 checks successful, independently inspected at issue62 verification time; it supplements the completed11-job exact-head evidence for unchanged76b87baf source.
