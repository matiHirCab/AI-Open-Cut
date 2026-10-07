# Speech alignment markers

`speech_markers_generate` creates ordinary scoped cue markers from validated alignment, as a standalone revision-checked tool or an operation in `timeline_batch_edit`. `speech_alignment_markers_v1` advertises support. Protocol 1 and schema 38 are unchanged.

Supply `projectId`, `expectedRevision`, `assetId`, `scope` (`root` or `component:<id>`), `startMs`, and `markerPolicy`. An optional non-null `alignment` accepts the existing provider-neutral record, including an owned known-text transcription preview result. If omitted, core uses the source asset's persisted speech-generation alignment. The source must contain audio and have a known positive duration.

Policies are closed records:

- `{ "type": "none" }`: zero cues; alignment is unnecessary. A standalone edit still uses ordinary one-revision/undo semantics.
- `{ "type": "sentence" }`: one cue at each supplied sentence start.
- `{ "type": "all_word" }`: one cue at each supplied word start.
- `{ "type": "selected_word", "indices": [2, 0] }`: distinct zero-based word ordinals, emitted in source order. Empty, duplicate or missing ordinals reject.

Core validates intrinsic alignment, source duration, safe integer times, placement plus segment start/end, component duration and the existing 4,096-marker scope limit. Missing requested sentence/word timing rejects; no segmentation, interpolation or quality upgrade occurs. Marker time equals placement `startMs` plus the selected segment's `startMs`.

Names retain ASCII letters/digits, collapse other runs to `_`, trim outer underscores, prefix numeric-leading names with `speech_`, and use `word_<ordinal>` or `sentence_<ordinal>` for empty normalized names. Bases are truncated to 120 ASCII bytes. Same-scope collisions use the smallest free `_2`, `_3`, … suffix. `EVERY. SINGLE. ONE.` word timing therefore produces `EVERY`, `SINGLE`, `ONE`, with unique suffixes when those names exist. IDs follow existing marker identity and persist exactly through history/reopen.

Batch component scopes support `component:@earlierAlias`. `resultAlias` is allowed only when exactly one cue is generated; zero/multiple results reject the complete batch. Generated IDs are returned in `changedIds`; marker names can drive existing marker-relative item starts. Markers alone emit no visual/audio instructions, and existing preview/export share the evaluated item timing.

`speech_generate_and_insert`, its `tts_generate_and_insert` alias, and `speech_commit_preview` placement type `insert` accept optional `markerPolicy`. Omission defaults to none. Asset, audio item and root cues commit atomically at the item's start offset in one revision/history entry. A stale preview retry reuses retained audio, alignment and policy without inference. Replacement/regeneration and caption commit keep their existing contracts; explicit aligned transcription can use the marker edit.

Missing asset uses `ASSET_NOT_FOUND`, non-audio `UNSUPPORTED_MEDIA`, missing component `ITEM_NOT_FOUND`, and invalid semantic selection/alignment/limits `VALIDATION_FAILED`. Stale revisions retain retryable `REVISION_CONFLICT`. Existing transport shape failures remain `INVALID_ARGUMENT` or MCP SDK validation text. Every failed batch/insertion rolls back project, history, aliases and tracked candidate resources. Ordinary marker records are reference-free; cues do not add source-asset retention requirements.
